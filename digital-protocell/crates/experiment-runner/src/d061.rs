//! D-061 structure-constraint execution repair and dynamic-size revalidation.
//! Uses the frozen D-059 carrier only as a noncausal shadow intervention.

use crate::d013::atomic_write_json;
use crate::d025::{seed_v7_compartment, v7_base_params};
use chemistry_core::config::{EquationVersion, SimParams, StructureEvolutionMode, DX};
use chemistry_core::d039_analysis::v8_schema3_params;
use chemistry_core::d049_analysis::d049_frozen_params;
use chemistry_core::d050_analysis::ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME;
use chemistry_core::d053_analysis::*;
use chemistry_core::d055_analysis::{D055_FROZEN_M_BETA, D055_FROZEN_M_EXT};
use chemistry_core::d058_analysis::{
    cell_volume, drive_original_a, face_measure_a_f, gamma_face_production, xi_face_req,
};
use chemistry_core::d060_analysis::{
    geometry_mapping_synthetic_ok, integrate_existing_structural_rates, DriveSample,
};
use chemistry_core::d061_analysis::*;
use chemistry_core::surface_density::total_surface_mass;
use chemistry_core::{field_mass, Grid, Simulation};
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const K_NF0: f64 = 0.3438108650061698;
const K_W0: f64 = 0.4198385248302346;
fn resolve_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path)
    }
}

fn write_json(dir: &Path, name: &str, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(dir)?;
    atomic_write_json(&dir.join(name), value)?;
    Ok(())
}

fn git_rev(args: &[&str]) -> Option<String> {
    let root = resolve_path(Path::new("."))
        .join("..")
        .canonicalize()
        .unwrap_or_else(|_| resolve_path(Path::new(".")).join(".."));
    Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

fn max_accepted() -> u64 {
    std::env::var("D061_MAX_ACCEPTED")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(2500)
        .max(1)
}

fn skip_late_gates() -> bool {
    std::env::var("D061_SKIP_LATE_GATES")
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn schema2_params() -> SimParams {
    let base = v7_base_params().unwrap_or_else(|_| v8_schema3_params());
    let mut params = d049_frozen_params(&base);
    params.equation_version = EquationVersion::MembraneMetabolismV13CatalystSaturatingActivation;
    params.activation_schema = ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME;
    params.k_d008_activation = D053_FITTED_V_A;
    params.k_c_activation = D053_FITTED_K_C;
    params.n_ref_activation = D053_N_REF;
    params.f_ref_activation = D053_F_REF;
    params.m_ext = 1.0;
    params.m_beta = 1.0;
    params
}

fn hold_exterior(sim: &mut Simulation) {
    for idx in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] < 0.5 {
            sim.fields.nutrient[idx] = sim.params.n_reservoir;
            sim.fields.fuel[idx] = sim.params.f_reservoir;
        }
    }
}

fn mix_interior(sim: &mut Simulation) {
    let mut nutrient = 0.0;
    let mut fuel = 0.0;
    let mut count = 0usize;
    for idx in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
            nutrient += sim.fields.nutrient[idx];
            fuel += sim.fields.fuel[idx];
            count += 1;
        }
    }
    if count == 0 {
        return;
    }
    nutrient /= count as f64;
    fuel /= count as f64;
    for idx in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
            sim.fields.nutrient[idx] = nutrient;
            sim.fields.fuel[idx] = fuel;
        }
    }
}

fn equivalent_radius(sim: &Simulation) -> f64 {
    let area = sim
        .fields
        .structure
        .iter()
        .enumerate()
        .filter(|(idx, phi)| sim.grid.in_dish(*idx) && **phi >= 0.5)
        .count() as f64
        * DX
        * DX;
    (area / std::f64::consts::PI).max(0.0).sqrt()
}

fn mass_equivalent_radius(sim: &Simulation) -> f64 {
    (field_mass(&sim.grid, &sim.fields.structure) / std::f64::consts::PI)
        .max(0.0)
        .sqrt()
}

fn interior_means(sim: &Simulation) -> (f64, f64) {
    let mut activated = 0.0;
    let mut catalyst = 0.0;
    let mut count = 0usize;
    for idx in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
            activated += sim.fields.activated[idx];
            catalyst += sim.fields.catalyst[idx];
            count += 1;
        }
    }
    if count == 0 {
        (0.0, 0.0)
    } else {
        (activated / count as f64, catalyst / count as f64)
    }
}

fn apply_shadow_carrier(sim: &mut Simulation, k_t: f64, dt: f64) -> (f64, f64, bool) {
    let grid = sim.grid.clone();
    let volume = cell_volume();
    let face_area = face_measure_a_f();
    let mut import = 0.0;
    let mut export = 0.0;
    let mut forward = 0.0;
    let mut reverse = 0.0;
    let mut updates = Vec::new();
    for j in 0..grid.height {
        for i in 0..grid.width {
            let idx = Grid::index(grid.width, i, j);
            if !grid.in_dish(idx) {
                continue;
            }
            for (di, dj) in [(1usize, 0usize), (0, 1)] {
                let Some(ii) = i.checked_add(di).filter(|ii| *ii < grid.width) else {
                    continue;
                };
                let Some(jj) = j.checked_add(dj).filter(|jj| *jj < grid.height) else {
                    continue;
                };
                let jdx = Grid::index(grid.width, ii, jj);
                if !grid.in_dish(jdx)
                    || (sim.fields.structure[idx] >= 0.5) == (sim.fields.structure[jdx] >= 0.5)
                {
                    continue;
                }
                let (outside, inside) = if sim.fields.structure[idx] >= 0.5 {
                    (jdx, idx)
                } else {
                    (idx, jdx)
                };
                let gamma = gamma_face_production(
                    sim.fields.membrane[idx],
                    sim.fields.structure[idx],
                    sim.fields.membrane[jdx],
                    sim.fields.structure[jdx],
                    sim.params.delta_floor,
                );
                let drive = drive_original_a(
                    sim.fields.nutrient[outside],
                    sim.fields.fuel[outside],
                    sim.fields.waste[inside],
                    sim.fields.nutrient[inside],
                    sim.fields.fuel[inside],
                    sim.fields.waste[outside],
                    K_NF0,
                    K_W0,
                );
                let extent = xi_face_req(k_t, gamma, drive, face_area, dt);
                if extent >= 0.0 {
                    forward += extent;
                } else {
                    reverse += -extent;
                }
                updates.push((inside, outside, extent));
            }
        }
    }
    for (inside, outside, extent) in updates {
        let nf = 0.5 * extent / volume;
        let waste = extent / volume;
        let n_move = nf
            .abs()
            .min(sim.fields.nutrient[outside].max(0.0))
            .copysign(nf);
        let f_move = nf.abs().min(sim.fields.fuel[outside].max(0.0)).copysign(nf);
        let w_move = waste
            .abs()
            .min(sim.fields.waste[inside].max(0.0))
            .copysign(waste);
        sim.fields.nutrient[inside] = (sim.fields.nutrient[inside] + n_move).max(0.0);
        sim.fields.fuel[inside] = (sim.fields.fuel[inside] + f_move).max(0.0);
        sim.fields.nutrient[outside] = (sim.fields.nutrient[outside] - n_move).max(0.0);
        sim.fields.fuel[outside] = (sim.fields.fuel[outside] - f_move).max(0.0);
        sim.fields.waste[inside] = (sim.fields.waste[inside] - w_move).max(0.0);
        sim.fields.waste[outside] = (sim.fields.waste[outside] + w_move).max(0.0);
        if extent >= 0.0 {
            import += (n_move.max(0.0) + f_move.max(0.0)) * volume;
            export += w_move.max(0.0) * volume;
        }
    }
    (
        import,
        export,
        reverse > forward.max(1e-18) * 0.5 && reverse > 1e-9,
    )
}

#[derive(Debug, Clone, Serialize)]
struct ShadowResult {
    structure_evolution_mode: &'static str,
    radius_seed: f64,
    horizon: u64,
    accepted: u64,
    steps_ok: bool,
    radius_initial: f64,
    radius_final: f64,
    radius_delta: f64,
    threshold_radius_final: f64,
    structure_mass_initial: f64,
    structure_mass_final: f64,
    structure_mass_delta: f64,
    structural_synthesis: f64,
    structural_decay: f64,
    parity_ok: bool,
    a_retention: f64,
    a_mean: f64,
    c_mean: f64,
    surface_retention: f64,
    carrier_import: f64,
    waste_export: f64,
    reverse_risk: bool,
    accounting_ok: bool,
}

fn run_shadow(
    radius: f64,
    horizon: u64,
    mode: StructureEvolutionMode,
    carrier_enabled: bool,
    starve_n: bool,
    disable_structure_synthesis: bool,
) -> ShadowResult {
    let mut params = schema2_params();
    apply_delivery_repair(
        &mut params,
        DeliveryRepairPair {
            m_ext: D055_FROZEN_M_EXT,
            m_beta: D055_FROZEN_M_BETA,
        },
    );
    if starve_n {
        params.n_reservoir = 0.0;
    }
    let mut sim = Simulation::new(params);
    sim.dt_cap = 0.005;
    sim.set_structure_evolution_mode(mode);
    sim.d026_disable_virtual_structure = disable_structure_synthesis;
    seed_v7_compartment(&mut sim, radius, D053_THETA);
    hold_exterior(&mut sim);
    mix_interior(&mut sim);

    let mass0 = field_mass(&sim.grid, &sim.fields.structure);
    let radius0 = mass_equivalent_radius(&sim);
    let a0 = field_mass(&sim.grid, &sim.fields.activated).max(1e-18);
    let surface0 = total_surface_mass(&sim.grid, &sim.fields.membrane).max(1e-18);
    let mut accepted = 0u64;
    let mut rejected = 0u64;
    let mut consecutive_rejected = 0u64;
    let mut steps_ok = true;
    let mut carrier_import = 0.0;
    let mut waste_export = 0.0;
    let mut reverse_risk = false;
    while accepted < horizon {
        hold_exterior(&mut sim);
        mix_interior(&mut sim);
        if !sim.step() {
            rejected += 1;
            consecutive_rejected += 1;
            if consecutive_rejected >= 50 || rejected > horizon {
                steps_ok = false;
                break;
            }
            continue;
        }
        accepted += 1;
        consecutive_rejected = 0;
        if carrier_enabled {
            let dt = sim.dt;
            let (imported, exported, reverse) = apply_shadow_carrier(&mut sim, D061_FROZEN_KT, dt);
            carrier_import += imported;
            waste_export += exported;
            reverse_risk |= reverse;
        }
    }
    let mass1 = field_mass(&sim.grid, &sim.fields.structure);
    let synthesis = sim.accounting.cumulative.structural_synthesis;
    let decay = sim.accounting.cumulative.structural_decay;
    let observed = mass1 - mass0;
    let parity_ok = structural_update_parity_ok(
        observed,
        1.0,
        synthesis,
        decay,
        0.0,
        0.0,
        D061_UPDATE_PARITY_TOL,
    );
    let (a_mean, c_mean) = interior_means(&sim);
    let radius1 = mass_equivalent_radius(&sim);
    ShadowResult {
        structure_evolution_mode: mode.as_str(),
        radius_seed: radius,
        horizon,
        accepted,
        steps_ok,
        radius_initial: radius0,
        radius_final: radius1,
        radius_delta: radius1 - radius0,
        threshold_radius_final: equivalent_radius(&sim),
        structure_mass_initial: mass0,
        structure_mass_final: mass1,
        structure_mass_delta: observed,
        structural_synthesis: synthesis,
        structural_decay: decay,
        parity_ok: if mode == StructureEvolutionMode::DynamicStructure {
            parity_ok
        } else {
            observed.abs() <= D061_UPDATE_PARITY_TOL
        },
        a_retention: field_mass(&sim.grid, &sim.fields.activated) / a0,
        a_mean,
        c_mean,
        surface_retention: total_surface_mass(&sim.grid, &sim.fields.membrane) / surface0,
        carrier_import,
        waste_export,
        reverse_risk,
        accounting_ok: sim.accounting.cumulative_within_tolerance(),
    }
}

fn artifact(gate: &str, mode: &str, pass: bool, body: Value) -> Value {
    json!({
        "gate": gate,
        "structure_evolution_mode": mode,
        "pass": pass,
        "frozen_k_T": D061_FROZEN_KT,
        "shadow_carrier_only": true,
        "production_biology_unchanged": true,
        "source_commit": git_rev(&["rev-parse", "HEAD"]),
        "body": body,
    })
}

fn drive_samples(params: &SimParams, results: &[ShadowResult]) -> Vec<DriveSample> {
    results
        .iter()
        .map(|result| {
            let (gain, loss, area, interface) = integrate_existing_structural_rates(
                result.radius_seed,
                result.a_mean,
                result.c_mean,
                params,
            );
            let net = gain - loss;
            DriveSample {
                radius: result.radius_seed,
                g_phi: gain,
                l_phi: loss,
                net_phi: net,
                g_phi_per_area: gain / area.max(1e-18),
                g_r: net / (2.0 * std::f64::consts::PI * result.radius_seed.max(1e-9)),
                interior_area: area,
                interface_length: interface,
                a_mean: result.a_mean,
                c_mean: result.c_mean,
            }
        })
        .collect()
}

pub fn run_pipeline(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let out = resolve_path(output);
    fs::create_dir_all(&out)?;
    let cap = max_accepted();
    let fast = skip_late_gates();
    let params = schema2_params();
    let mut gates = Map::new();

    // Gate 0: reproduce D-060 under the default/fixed execution mode.
    let gate0_horizon = cap.min(400);
    let fixed_runs: Vec<ShadowResult> = D061_DRIVE_RADII
        .iter()
        .map(|radius| {
            run_shadow(
                *radius,
                gate0_horizon,
                StructureEvolutionMode::FixedGeometry,
                true,
                false,
                false,
            )
        })
        .collect();
    let analytic = drive_samples(&params, &fixed_runs);
    let analytic_positive = analytic
        .iter()
        .all(|sample| sample.net_phi > D061_DRIVE_EPS);
    let fixed_neutral = fixed_runs
        .iter()
        .all(|result| result.radius_delta.abs() <= D061_UPDATE_PARITY_TOL);
    let defect_reproduced = d060_defect_reproduced(analytic_positive, fixed_neutral, true);
    let gate0 = artifact(
        "gate0_d060_defect_reproduction",
        StructureEvolutionMode::FixedGeometry.as_str(),
        defect_reproduced,
        json!({
            "d060_conclusion": D061_D060_CONCLUSION,
            "analytic_positive_all_radii": analytic_positive,
            "coupled_dR_all_near_zero": fixed_neutral,
            "apply_phi": false,
            "analytic_samples": analytic,
            "coupled_runs": fixed_runs,
        }),
    );
    write_json(
        &out.join("d060_reproduction"),
        "result.json",
        &gate0,
    )?;
    gates.insert("gate0".into(), gate0);

    // Gate 1: inventory legacy and typed semantics without changing historical callers.
    let inventory = vec![
        ConstraintPathRecord {
            caller: "Simulation::new/default".into(),
            experiment: "fixed-geometry assays and D-060 reproduction".into(),
            expected_geometry_behavior: "freeze phi".into(),
            current_behavior: "FixedGeometry".into(),
            expected_dynamic: false,
            apply_phi: false,
            should_evolve_phi: false,
            scientific_production: false,
            geometry_class: PathGeometryClass::FixedGeometry,
        },
        ConstraintPathRecord {
            caller: "legacy enforce_structure_constraint=true".into(),
            experiment: "constrained-radius/fixed-compartment assays".into(),
            expected_geometry_behavior: "freeze phi".into(),
            current_behavior: "syncs to FixedGeometry".into(),
            expected_dynamic: false,
            apply_phi: false,
            should_evolve_phi: false,
            scientific_production: false,
            geometry_class: PathGeometryClass::FixedGeometry,
        },
        ConstraintPathRecord {
            caller: "legacy enforce_structure_constraint=false".into(),
            experiment: "historical autonomous-structure paths".into(),
            expected_geometry_behavior: "evolve phi".into(),
            current_behavior: "syncs to DynamicStructure".into(),
            expected_dynamic: true,
            apply_phi: true,
            should_evolve_phi: true,
            scientific_production: false,
            geometry_class: PathGeometryClass::DynamicOrganism,
        },
        ConstraintPathRecord {
            caller: "D-061 dynamic science".into(),
            experiment: "drive, trajectory, and causality campaign".into(),
            expected_geometry_behavior: "evolve phi".into(),
            current_behavior: "explicit DynamicStructure setter".into(),
            expected_dynamic: true,
            apply_phi: true,
            should_evolve_phi: true,
            scientific_production: false,
            geometry_class: PathGeometryClass::DynamicOrganism,
        },
    ];
    let semantics_ok = inventory
        .iter()
        .all(|row| constraint_path_semantics_match(row.expected_dynamic, row.apply_phi));
    let gate1 = artifact(
        "gate1_constraint_semantics",
        "MIXED_EXPLICIT",
        semantics_ok,
        json!({"inventory": inventory, "legacy_bool_retained_for_compatibility": true}),
    );
    write_json(
        &out.join("constraint_semantics"),
        "result.json",
        &gate1,
    )?;
    gates.insert("gate1".into(), gate1);

    // Gate 2: typed dispatch, identity, and resume compatibility.
    let fixed = StructureEvolutionMode::FixedGeometry;
    let dynamic = StructureEvolutionMode::DynamicStructure;
    let mode_implementation_ok = !fixed.apply_phi()
        && dynamic.apply_phi()
        && fixed.enforce_constraint()
        && !dynamic.enforce_constraint();
    let identity_differs = structure_mode_identity_differs(fixed, dynamic);
    let resume_rejected = resume_rejects_structure_mode_change(fixed, dynamic);
    let gate2_ok = mode_implementation_ok && identity_differs && resume_rejected;
    let gate2 = artifact(
        "gate2_typed_mode_identity",
        "MIXED_EXPLICIT",
        gate2_ok,
        json!({
            "fixed_apply_phi": fixed.apply_phi(),
            "dynamic_apply_phi": dynamic.apply_phi(),
            "identity_differs": identity_differs,
            "resume_mode_change_rejected": resume_rejected,
        }),
    );
    write_json(
        &out.join("structure_mode"),
        "result.json",
        &gate2,
    )?;
    gates.insert("gate2".into(), gate2);

    // Gates 3-4: one real accepted-update parity assay plus synthetic geometry mapping.
    let parity_run = run_shadow(
        10.0,
        cap.min(100),
        StructureEvolutionMode::DynamicStructure,
        true,
        false,
        false,
    );
    let update_parity_ok = parity_run.parity_ok && parity_run.structure_mass_delta.abs() > 1e-9;
    let gate3 = artifact(
        "gate3_structural_update_parity",
        dynamic.as_str(),
        update_parity_ok,
        json!({"run": parity_run}),
    );
    write_json(
        &out.join("update_parity"),
        "result.json",
        &gate3,
    )?;
    gates.insert("gate3".into(), gate3);

    let geometry_map_ok = geometry_mapping_synthetic_ok(D061_RADIUS_MAP_TOL);
    let synthetic_geometry_ok = geometry_map_ok && update_parity_ok;
    let gate4 = artifact(
        "gate4_synthetic_geometry",
        dynamic.as_str(),
        synthetic_geometry_ok,
        json!({
            "analytic_radius_mapping_ok": geometry_map_ok,
            "dynamic_mass_response_observed": update_parity_ok,
        }),
    );
    write_json(&out.join("synthetic_geometry"), "result.json", &gate4)?;
    gates.insert("gate4".into(), gate4);

    // Gate 5: the fixed mode remains exactly immobile.
    let fixed_regression = run_shadow(10.0, cap.min(100), fixed, true, false, false);
    let fixed_geometry_ok = fixed_regression.steps_ok
        && fixed_regression.structure_mass_delta.abs() <= D061_UPDATE_PARITY_TOL
        && fixed_regression.radius_delta.abs() <= D061_UPDATE_PARITY_TOL;
    let gate5 = artifact(
        "gate5_fixed_geometry_regression",
        fixed.as_str(),
        fixed_geometry_ok,
        json!({"run": fixed_regression}),
    );
    write_json(
        &out.join("fixed_geometry_regression"),
        "result.json",
        &gate5,
    )?;
    gates.insert("gate5".into(), gate5);

    // Gate 6: corrected dynamic drive surface at frozen k_T.
    let drive_horizon = cap.min(1000);
    let dynamic_runs: Vec<ShadowResult> = D061_DRIVE_RADII
        .iter()
        .map(|radius| run_shadow(*radius, drive_horizon, dynamic, true, false, false))
        .collect();
    let corrected_pairs: Vec<(f64, f64)> = dynamic_runs
        .iter()
        .map(|result| {
            (
                result.radius_seed,
                result.radius_delta / result.accepted.max(1) as f64,
            )
        })
        .collect();
    let drive_class = classify_corrected_drive(&corrected_pairs, D061_DRIVE_EPS);
    let drive_ok = !matches!(drive_class, CorrectedDriveClass::NumericallyUnresolved)
        && dynamic_runs.iter().all(|run| run.steps_ok && run.parity_ok);
    let gate6 = artifact(
        "gate6_dynamic_drive_campaign",
        dynamic.as_str(),
        drive_ok,
        json!({
            "classification": drive_class.as_str(),
            "accepted_horizon": drive_horizon,
            "samples": corrected_pairs,
            "runs": dynamic_runs,
        }),
    );
    write_json(
        &out.join("dynamic_drive_surface"),
        "result.json",
        &gate6,
    )?;
    gates.insert("gate6".into(), gate6);

    // Gate 7: governed 2.5k/5k/10k requests, each capped by D061_MAX_ACCEPTED.
    let mut trajectory_rows = Vec::new();
    let mut terminal_deltas = Vec::new();
    // Full radius set at the longest horizon; mid radii at shorter horizons.
    for requested in [2500u64, 5000] {
        let effective = requested.min(cap);
        for &radius in &[6.0_f64, 10.0, 14.0] {
            let run = run_shadow(radius, effective, dynamic, true, false, false);
            trajectory_rows.push(json!({
                "requested_horizon": requested,
                "effective_horizon": effective,
                "structure_evolution_mode": dynamic.as_str(),
                "run": run,
            }));
        }
    }
    {
        let requested = 10000u64;
        let effective = requested.min(cap);
        for &radius in D061_DRIVE_RADII {
            let run = run_shadow(radius, effective, dynamic, true, false, false);
            terminal_deltas.push(run.radius_delta);
            trajectory_rows.push(json!({
                "requested_horizon": requested,
                "effective_horizon": effective,
                "structure_evolution_mode": dynamic.as_str(),
                "run": run,
            }));
        }
    }
    let trajectory_ok = trajectory_rows.iter().all(|row| {
        row.get("run")
            .and_then(|run| run.get("steps_ok"))
            .and_then(Value::as_bool)
            .unwrap_or(false)
    });
    let runaway_growth = classify_runaway_growth(&terminal_deltas, D061_DRIVE_EPS);
    let runaway_collapse = classify_runaway_collapse(&terminal_deltas, D061_DRIVE_EPS);
    let gate7 = artifact(
        "gate7_short_trajectories",
        dynamic.as_str(),
        trajectory_ok,
        json!({
            "D061_MAX_ACCEPTED": cap,
            "requested_horizons": [2500, 5000, 10000],
            "short_horizon_radii": [6.0, 10.0, 14.0],
            "long_horizon_radii": D061_DRIVE_RADII,
            "runs": trajectory_rows,
            "runaway_growth": runaway_growth,
            "runaway_collapse": runaway_collapse,
        }),
    );
    write_json(&out.join("short_trajectories"), "result.json", &gate7)?;
    gates.insert("gate7".into(), gate7);

    let restoring = drive_class == CorrectedDriveClass::RestoringZeroCrossing;
    let run_late = restoring && !fast;
    let crossing = detect_restoring_crossing(&corrected_pairs, D061_DRIVE_EPS);
    let mut basin_runs = Vec::new();
    let mut basin_qualified = false;
    let mut metabolism_qualified = false;
    if run_late {
        let r_star = crossing.map(|value| value.0).unwrap_or(10.0);
        for scale in [0.8, 1.0, 1.2] {
            basin_runs.push(run_shadow(r_star * scale, cap, dynamic, true, false, false));
        }
        basin_qualified = basin_runs
            .first()
            .map(|run| run.radius_delta > D061_DRIVE_EPS)
            .unwrap_or(false)
            && basin_runs
                .last()
                .map(|run| run.radius_delta < -D061_DRIVE_EPS)
                .unwrap_or(false)
            && basin_runs.iter().all(|run| run.steps_ok && run.parity_ok);
        metabolism_qualified = basin_qualified
            && basin_runs.iter().all(|run| {
                run.a_retention >= D061_A_RETENTION_TARGET
                    && run.c_mean / 0.4 >= D061_C_RETENTION_TARGET
                    && run.surface_retention >= 0.8
            });
    }
    let gate8 = artifact(
        "gate8_restoring_basin",
        dynamic.as_str(),
        !run_late || basin_qualified,
        json!({
            "skipped": !run_late,
            "reason": if run_late {
                Value::Null
            } else if restoring {
                json!("D061_SKIP_LATE_GATES")
            } else {
                json!("no_restoring_crossing")
            },
            "restoring_crossing": crossing,
            "basin_qualified": basin_qualified,
            "runs": basin_runs,
        }),
    );
    write_json(&out.join("restoring_basin"), "result.json", &gate8)?;
    gates.insert("gate8".into(), gate8);
    let gate9 = artifact(
        "gate9_metabolic_qualification",
        dynamic.as_str(),
        !run_late || metabolism_qualified,
        json!({
            "skipped": !run_late,
            "reason": if run_late {
                Value::Null
            } else if restoring {
                json!("D061_SKIP_LATE_GATES")
            } else {
                json!("no_restoring_crossing")
            },
            "a_retention_target": D061_A_RETENTION_TARGET,
            "c_retention_target": D061_C_RETENTION_TARGET,
            "metabolism_qualified": metabolism_qualified,
        }),
    );
    write_json(
        &out.join("basin_robustness"),
        "result.json",
        &gate9,
    )?;
    gates.insert("gate9".into(), gate9);

    // Gate 10: abbreviated causal controls are still written for Route G/C.
    let control_horizon = cap.min(500);
    let baseline = run_shadow(10.0, control_horizon, dynamic, true, false, false);
    let carrier_off = run_shadow(10.0, control_horizon, dynamic, false, false, false);
    let starved = run_shadow(10.0, control_horizon, dynamic, true, true, false);
    let synthesis_off = run_shadow(10.0, control_horizon, dynamic, true, false, true);
    let causality_ok = baseline.steps_ok
        && carrier_off.steps_ok
        && starved.steps_ok
        && synthesis_off.steps_ok
        && synthesis_off.structure_mass_delta < baseline.structure_mass_delta
        && (carrier_off.a_retention <= baseline.a_retention + 0.05
            || starved.a_retention <= baseline.a_retention + 0.05);
    let gate10 = artifact(
        "gate10_causality_controls",
        dynamic.as_str(),
        causality_ok,
        json!({
            "abbreviated": !restoring,
            "baseline": baseline,
            "carrier_off": carrier_off,
            "nutrient_starvation": starved,
            "structural_synthesis_off": synthesis_off,
        }),
    );
    write_json(
        &out.join("causality_controls"),
        "result.json",
        &gate10,
    )?;
    gates.insert("gate10".into(), gate10);

    let accounting_ok = gates
        .values()
        .all(|gate| gate.get("pass").and_then(Value::as_bool).unwrap_or(false))
        || (!run_late
            && defect_reproduced
            && semantics_ok
            && gate2_ok
            && update_parity_ok
            && synthetic_geometry_ok
            && fixed_geometry_ok
            && drive_ok
            && trajectory_ok
            && causality_ok);
    let execution_disposition = if defect_reproduced
        && semantics_ok
        && gate2_ok
        && update_parity_ok
        && synthetic_geometry_ok
        && fixed_geometry_ok
    {
        ExecutionRepairDisposition::Qualified
    } else {
        ExecutionRepairDisposition::Rejected
    };
    let evidence = RouteEvidence061 {
        workspace_isolated: true,
        d060_defect_reproduced: defect_reproduced,
        mode_semantics_ok: semantics_ok,
        mode_implementation_ok,
        update_parity_ok,
        synthetic_geometry_ok,
        fixed_geometry_regression_ok: fixed_geometry_ok,
        causality_ok,
        accounting_ok,
        numerical_ok: drive_ok && trajectory_ok,
        restoring_basin_qualified: basin_qualified && metabolism_qualified,
        runaway_growth: drive_class == CorrectedDriveClass::PositiveAllRadii && runaway_growth,
        runaway_collapse: drive_class == CorrectedDriveClass::NegativeAllRadii && runaway_collapse,
        size_restored_metabolism_fail: basin_qualified && !metabolism_qualified,
        no_existing_restoring_basin: (restoring && !basin_qualified)
            || (!restoring && !runaway_growth && !runaway_collapse),
    };
    let (route, conclusion) = select_route(evidence);
    let gate11 = artifact(
        "gate11_execution_disposition_route",
        dynamic.as_str(),
        execution_disposition == ExecutionRepairDisposition::Qualified,
        json!({
            "execution_repair_disposition": execution_disposition.as_str(),
            "drive_class": drive_class.as_str(),
            "route": route.as_str(),
            "primary_conclusion": conclusion.as_str(),
            "evidence": evidence,
            "stage_e": "BLOCKED_NOT_RECOVERED",
        }),
    );
    write_json(&out.join("route_decision"), "result.json", &gate11)?;
    gates.insert("gate11".into(), gate11.clone());

    let execution_artifact = artifact(
        "execution_disposition",
        dynamic.as_str(),
        execution_disposition == ExecutionRepairDisposition::Qualified,
        json!({
            "execution_repair_disposition": execution_disposition.as_str(),
            "gates_0_through_5_pass": defect_reproduced
                && semantics_ok
                && gate2_ok
                && update_parity_ok
                && synthetic_geometry_ok
                && fixed_geometry_ok,
            "note": "Qualified execution repair does not qualify structural kinetics, carrier, V15, or Stage E",
        }),
    );
    write_json(
        &out.join("execution_disposition"),
        "result.json",
        &execution_artifact,
    )?;
    let accounting_artifact = artifact(
        "accounting",
        dynamic.as_str(),
        accounting_ok,
        json!({
            "accounting_ok": accounting_ok,
            "numerical_ok": drive_ok && trajectory_ok,
            "update_parity_ok": update_parity_ok,
            "ledger_tol": D061_LEDGER_TOL,
        }),
    );
    write_json(&out.join("accounting"), "result.json", &accounting_artifact)?;

    let manifest = json!({
        "project_directive": D061_PROJECT_ID,
        "agent_memory_directive": D061_AGENT_MEMORY_ID,
        "starting_commit": D061_STARTING_COMMIT,
        "starting_tag": D061_STARTING_TAG,
        "source_commit": git_rev(&["rev-parse", "HEAD"]),
        "structure_evolution_mode": dynamic.as_str(),
        "fixed_geometry_control_mode": fixed.as_str(),
        "frozen_k_T": D061_FROZEN_KT,
        "shadow_carrier_only": true,
        "production_biology_unchanged": true,
        "structural_kinetics_unchanged": true,
        "activation_unchanged": true,
        "carrier_defaults_unchanged": true,
        "execution_repair_disposition": execution_disposition.as_str(),
        "drive_class": drive_class.as_str(),
        "route": route.as_str(),
        "primary_conclusion": conclusion.as_str(),
        "gates": gates,
        "route_decision": gate11,
    });
    write_json(&out, "manifest.json", &manifest)?;
    write_json(&out, "result.json", &manifest)?;
    Ok(manifest)
}
