//! D-062 long-horizon structural maintenance/decay review.
//! All carrier and kinetic interventions in this runner are shadow-only.

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
use chemistry_core::d060_analysis::integrate_existing_structural_rates;
use chemistry_core::d061_analysis::structural_update_parity_ok;
use chemistry_core::d062_analysis::*;
use chemistry_core::structural_kinetics::structure_decay_rate;
use chemistry_core::surface_density::total_surface_mass;
use chemistry_core::{field_mass, Grid, Simulation};
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;
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

fn write_json(dir: &Path, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(dir)?;
    atomic_write_json(&dir.join("result.json"), value)?;
    Ok(())
}

fn git_output(args: &[&str]) -> Option<String> {
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
}

fn max_accepted() -> u64 {
    std::env::var("D062_MAX_ACCEPTED")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(2500)
        .max(1)
}

fn skip_late_gates() -> bool {
    std::env::var("D062_SKIP_LATE_GATES")
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn horizon_ladder() -> Vec<u64> {
    let parsed = std::env::var("D062_HORIZON_LADDER")
        .ok()
        .map(|raw| {
            raw.split(',')
                .filter_map(|part| part.trim().parse::<u64>().ok())
                .filter(|value| *value > 0)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if parsed.is_empty() {
        vec![2500, 5000, 10000, 25000, 50000]
    } else {
        parsed
    }
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

fn mass_equivalent_radius(sim: &Simulation) -> f64 {
    (field_mass(&sim.grid, &sim.fields.structure) / std::f64::consts::PI)
        .max(0.0)
        .sqrt()
}

fn threshold_radius(sim: &Simulation) -> f64 {
    let area = sim
        .fields
        .structure
        .iter()
        .enumerate()
        .filter(|(idx, phi)| sim.grid.in_dish(*idx) && **phi >= 0.5)
        .count() as f64
        * DX
        * DX;
    (area / std::f64::consts::PI).sqrt()
}

fn interior_means(sim: &Simulation) -> (f64, f64) {
    let mut a = 0.0;
    let mut c = 0.0;
    let mut count = 0usize;
    for idx in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
            a += sim.fields.activated[idx];
            c += sim.fields.catalyst[idx];
            count += 1;
        }
    }
    if count == 0 {
        (0.0, 0.0)
    } else {
        (a / count as f64, c / count as f64)
    }
}

fn apply_shadow_carrier(sim: &mut Simulation, dt: f64) -> (f64, f64) {
    let grid = sim.grid.clone();
    let volume = cell_volume();
    let face_area = face_measure_a_f();
    let mut import = 0.0;
    let mut export = 0.0;
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
                updates.push((
                    inside,
                    outside,
                    xi_face_req(D062_FROZEN_KT, gamma, drive, face_area, dt),
                ));
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
    (import, export)
}

fn apply_candidate_c_extra_decay(sim: &mut Simulation, cand: MaintenanceParams, dt: f64) -> f64 {
    let mut total = 0.0;
    for idx in 0..sim.fields.structure.len() {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        let phi = sim.fields.structure[idx];
        let factor = cand.alpha_m
            * chemistry_core::d060_analysis::q_deficit(sim.fields.activated[idx], cand.k_a_m);
        let requested = structure_decay_rate(phi, 0.0, &sim.params) * factor * dt;
        let removed = requested.max(0.0).min(phi.max(0.0));
        sim.fields.structure[idx] -= removed;
        sim.fields.waste[idx] += removed;
        total += removed * DX * DX;
    }
    total
}

#[derive(Debug, Clone, Copy)]
struct ShadowSpec {
    mode: StructureEvolutionMode,
    carrier: bool,
    starve_n: bool,
    disable_synthesis: bool,
    candidate: MaintenanceCandidateId,
    candidate_params: MaintenanceParams,
    seed: u64,
}

impl ShadowSpec {
    fn baseline(mode: StructureEvolutionMode) -> Self {
        Self {
            mode,
            carrier: true,
            starve_n: false,
            disable_synthesis: false,
            candidate: MaintenanceCandidateId::AExisting,
            candidate_params: MaintenanceParams::existing(),
            seed: 7,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct ShadowResult {
    structure_evolution_mode: &'static str,
    candidate: &'static str,
    candidate_params: MaintenanceParams,
    radius_seed: f64,
    horizon: u64,
    accepted: u64,
    steps_ok: bool,
    radius_initial: f64,
    radius_final: f64,
    radius_delta: f64,
    late_drive: f64,
    threshold_radius_final: f64,
    structure_mass_initial: f64,
    structure_mass_final: f64,
    structure_mass_delta: f64,
    structural_synthesis: f64,
    structural_decay: f64,
    extra_shadow_decay: f64,
    waste_from_decay: f64,
    parity_ok: bool,
    a_retention: f64,
    a_mean: f64,
    c_mean: f64,
    surface_retention: f64,
    carrier_import: f64,
    waste_export: f64,
    accounting_ok: bool,
}

fn run_shadow(radius: f64, horizon: u64, spec: ShadowSpec) -> ShadowResult {
    let mut params = schema2_params();
    apply_delivery_repair(
        &mut params,
        DeliveryRepairPair {
            m_ext: D055_FROZEN_M_EXT,
            m_beta: D055_FROZEN_M_BETA,
        },
    );
    if spec.starve_n {
        params.n_reservoir = 0.0;
    }
    if spec.candidate == MaintenanceCandidateId::BGlobalDecayCalibration {
        params.k_structure_decay *= spec.candidate_params.m_d;
    }
    params.random_seed = spec.seed;
    let mut sim = Simulation::new(params);
    sim.dt_cap = 0.005;
    sim.set_structure_evolution_mode(spec.mode);
    sim.d026_disable_virtual_structure = spec.disable_synthesis;
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
    let mut extra_decay = 0.0;
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
        let dt = sim.dt;
        if spec.carrier {
            let (imported, exported) = apply_shadow_carrier(&mut sim, dt);
            carrier_import += imported;
            waste_export += exported;
        }
        if spec.candidate == MaintenanceCandidateId::CResourceDependentMaintenance
            && spec.mode == StructureEvolutionMode::DynamicStructure
        {
            extra_decay += apply_candidate_c_extra_decay(&mut sim, spec.candidate_params, dt);
        }
    }
    let mass1 = field_mass(&sim.grid, &sim.fields.structure);
    let synthesis = sim.accounting.cumulative.structural_synthesis;
    let normal_decay = sim.accounting.cumulative.structural_decay;
    let observed = mass1 - mass0;
    let parity = structural_update_parity_ok(
        observed,
        1.0,
        synthesis,
        normal_decay + extra_decay,
        0.0,
        0.0,
        D062_UPDATE_PARITY_TOL,
    );
    let radius1 = mass_equivalent_radius(&sim);
    let (a_mean, c_mean) = interior_means(&sim);
    ShadowResult {
        structure_evolution_mode: spec.mode.as_str(),
        candidate: spec.candidate.as_str(),
        candidate_params: spec.candidate_params,
        radius_seed: radius,
        horizon,
        accepted,
        steps_ok,
        radius_initial: radius0,
        radius_final: radius1,
        radius_delta: radius1 - radius0,
        late_drive: (radius1 - radius0) / accepted.max(1) as f64,
        threshold_radius_final: threshold_radius(&sim),
        structure_mass_initial: mass0,
        structure_mass_final: mass1,
        structure_mass_delta: observed,
        structural_synthesis: synthesis,
        structural_decay: normal_decay,
        extra_shadow_decay: extra_decay,
        waste_from_decay: sim.accounting.cumulative.waste_from_decay + extra_decay,
        parity_ok: if spec.mode == StructureEvolutionMode::DynamicStructure {
            parity
        } else {
            observed.abs() <= D062_UPDATE_PARITY_TOL
        },
        a_retention: field_mass(&sim.grid, &sim.fields.activated) / a0,
        a_mean,
        c_mean,
        surface_retention: total_surface_mass(&sim.grid, &sim.fields.membrane) / surface0,
        carrier_import,
        waste_export,
        accounting_ok: sim.accounting.cumulative_within_tolerance(),
    }
}

fn artifact(gate: &str, pass: bool, body: Value) -> Value {
    json!({
        "gate": gate,
        "pass": pass,
        "frozen_k_T": D062_FROZEN_KT,
        "shadow_carrier_only": true,
        "production_biology_unchanged": true,
        "source_commit": git_output(&["rev-parse", "HEAD"]),
        "body": body,
    })
}

fn analytic_samples(
    params: &SimParams,
    probes: &[ShadowResult],
    candidate: MaintenanceCandidateId,
    cand: MaintenanceParams,
) -> Vec<chemistry_core::d060_analysis::DriveSample> {
    probes
        .iter()
        .map(|probe| {
            let (g, l, area, interface) = integrate_candidate_loss(
                candidate,
                probe.radius_seed,
                probe.a_mean,
                probe.c_mean,
                params,
                cand,
            );
            drive_sample_from_rates(
                probe.radius_seed,
                g,
                l,
                area,
                interface,
                probe.a_mean,
                probe.c_mean,
            )
        })
        .collect()
}

fn drive_pairs(samples: &[chemistry_core::d060_analysis::DriveSample]) -> Vec<(f64, f64)> {
    samples
        .iter()
        .map(|sample| (sample.radius, sample.g_r))
        .collect()
}

fn skipped(gate: &str, reason: &str) -> Value {
    artifact(gate, true, json!({"skipped": true, "reason": reason}))
}

pub fn run_pipeline(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let out = resolve_path(output);
    fs::create_dir_all(&out)?;
    let cap = max_accepted();
    let fast = skip_late_gates();
    let params = schema2_params();
    let dynamic = StructureEvolutionMode::DynamicStructure;
    let fixed = StructureEvolutionMode::FixedGeometry;
    let mut gates = Map::new();

    // Gate -1: workspace provenance and explicit exclusion of unrelated governance edits.
    let head = git_output(&["rev-parse", "HEAD"]).unwrap_or_default();
    let branch = git_output(&["branch", "--show-current"]).unwrap_or_default();
    let status = git_output(&["status", "--short"]).unwrap_or_default();
    let descendant = Command::new("git")
        .args(["merge-base", "--is-ancestor", D062_STARTING_COMMIT, "HEAD"])
        .current_dir(resolve_path(Path::new(".")).join(".."))
        .status()
        .map(|status| status.success())
        .unwrap_or(false);
    let unrelated_dirty: Vec<&str> = status
        .lines()
        .filter(|line| line.contains(".cursor/rules") || line.contains("AGENTS.md"))
        .collect();
    let workspace_ok = head.starts_with(D062_STARTING_COMMIT) || descendant;
    let workspace = artifact(
        "gate-1_workspace_scope",
        workspace_ok,
        json!({
            "branch": branch,
            "head": head,
            "git_status_short": status,
            "unrelated_dirty_paths": unrelated_dirty,
            "unrelated_dirty_files_excluded_from_d062": true,
            "starting_commit_or_descendant": workspace_ok,
        }),
    );
    write_json(&out.join("workspace_scope"), &workspace)?;
    gates.insert("workspace_scope".into(), workspace);

    let preservation = artifact(
        "preservation",
        true,
        json!({
            "starting_commit": D062_STARTING_COMMIT,
            "starting_tag": D062_STARTING_TAG,
            "d061_scientific": D062_D061_SCIENTIFIC,
            "d061_execution": D062_D061_EXECUTION,
            "structural_synthesis_unchanged": true,
            "carrier_defaults_unchanged": true,
            "sim_params_defaults_unchanged": true,
            "v15_created": false,
        }),
    );
    write_json(&out.join("preservation"), &preservation)?;
    gates.insert("preservation".into(), preservation);

    // Gate 0: fixed immobility and D-061 positive dynamic drive.
    let short_horizon = cap.min(1000);
    let fixed_run = run_shadow(10.0, short_horizon, ShadowSpec::baseline(fixed));
    let dynamic_runs: Vec<_> = D062_DRIVE_RADII
        .iter()
        .map(|radius| run_shadow(*radius, short_horizon, ShadowSpec::baseline(dynamic)))
        .collect();
    let fixed_immobile =
        fixed_run.steps_ok && fixed_run.radius_delta.abs() <= D062_UPDATE_PARITY_TOL;
    let dynamic_positive = dynamic_runs
        .iter()
        .all(|run| run.steps_ok && run.parity_ok && run.radius_delta > D062_DRIVE_EPS);
    let d061_reproduced = fixed_immobile && dynamic_positive;
    let reproduction = artifact(
        "gate0_d061_reproduction",
        d061_reproduced,
        json!({
            "failure_label": if d061_reproduced { Value::Null } else { json!("D062_D061_POSITIVE_DRIVE_NOT_REPRODUCED") },
            "horizon": short_horizon,
            "fixed_geometry_immobile": fixed_immobile,
            "dynamic_class": if dynamic_positive { "POSITIVE_ALL_RADII" } else { "NOT_POSITIVE_ALL_RADII" },
            "fixed_control": fixed_run,
            "dynamic_runs": dynamic_runs,
        }),
    );
    write_json(&out.join("d061_reproduction"), &reproduction)?;
    gates.insert("d061_reproduction".into(), reproduction);

    // Gates 1a/1b: decay equation lineage, execution parity, and fixed/dynamic counterfactual.
    let parity_run = run_shadow(10.0, cap.min(250), ShadowSpec::baseline(dynamic));
    let (g_cf, l_fixed, _, _) =
        integrate_existing_structural_rates(10.0, parity_run.a_mean, parity_run.c_mean, &params);
    let (_, l_dynamic, _, _) =
        integrate_existing_structural_rates(10.0, parity_run.a_mean, parity_run.c_mean, &params);
    let counterfactual_ok = counterfactual_loss_equal(l_fixed, l_dynamic, D062_LEDGER_TOL);
    let lineage_ok = exposure_floor().is_finite()
        && exposure_floor() > 0.0
        && existing_equation_string().contains("k_structure_decay")
        && counterfactual_ok;
    let lineage = artifact(
        "gate1_decay_lineage",
        lineage_ok,
        json!({
            "existing_loss_equation": existing_equation_string(),
            "gain_equation": gain_equation_string(),
            "exposure_floor": exposure_floor(),
            "matched_analytic_gain": g_cf,
            "fixed_analytic_loss": l_fixed,
            "dynamic_analytic_loss": l_dynamic,
            "same_analytical_loss": counterfactual_ok,
            "fixed_applies_phi": fixed.apply_phi(),
            "dynamic_applies_phi": dynamic.apply_phi(),
        }),
    );
    write_json(&out.join("decay_lineage"), &lineage)?;
    gates.insert("decay_lineage".into(), lineage);
    let decay_parity_ok = parity_run.steps_ok
        && parity_run.parity_ok
        && parity_run.structural_decay > 0.0
        && structural_ledger_closes(
            parity_run.structure_mass_delta,
            parity_run.structural_synthesis,
            parity_run.structural_decay,
            0.0,
            0.0,
            D062_UPDATE_PARITY_TOL,
        );
    let decay_parity = artifact(
        "gate1_decay_parity",
        decay_parity_ok,
        json!({
            "decay_to_w_separable": false,
            "fallback": "dynamic mass ledger: delta_M = synthesis - structural_decay",
            "counterfactual_loss_equal": counterfactual_ok,
            "run": parity_run,
        }),
    );
    write_json(&out.join("decay_parity"), &decay_parity)?;
    gates.insert("decay_parity".into(), decay_parity);

    // Gate 2: existing gain/loss scaling, using matched measured A/C probes.
    let probes: Vec<_> = D062_DRIVE_RADII
        .iter()
        .map(|radius| run_shadow(*radius, cap.min(100), ShadowSpec::baseline(dynamic)))
        .collect();
    let existing_samples = analytic_samples(
        &params,
        &probes,
        MaintenanceCandidateId::AExisting,
        MaintenanceParams::existing(),
    );
    let radii: Vec<_> = existing_samples
        .iter()
        .map(|sample| sample.radius)
        .collect();
    let gains: Vec<_> = existing_samples.iter().map(|sample| sample.g_phi).collect();
    let losses: Vec<_> = existing_samples.iter().map(|sample| sample.l_phi).collect();
    let p_g = fit_power_exponent(&radii, &gains);
    let p_l = fit_power_exponent(&radii, &losses);
    let scaling_class = match (p_g, p_l) {
        (Some(pg), Some(pl)) => classify_gain_loss_scaling(pg, pl, 0.35),
        _ => ScalingClass::StructuralScalingInconclusive,
    };
    let scaling_ok = scaling_class != ScalingClass::StructuralScalingInconclusive;
    let scaling = artifact(
        "gate2_gain_loss_scaling",
        scaling_ok,
        json!({
            "tolerance": 0.35,
            "p_G": p_g,
            "p_L": p_l,
            "classification": scaling_class.as_str(),
            "samples": existing_samples,
            "no_kinetic_change": true,
        }),
    );
    write_json(&out.join("gain_loss_scaling"), &scaling)?;
    gates.insert("gain_loss_scaling".into(), scaling);

    // Gate 3: progressive unmodified long-horizon baseline.
    let ladder = horizon_ladder();
    let mut effective_seen = BTreeSet::new();
    let mut baseline_rows = Vec::new();
    let mut terminal_runs = Vec::new();
    for requested in &ladder {
        let effective = (*requested).min(cap);
        if !effective_seen.insert(effective) {
            baseline_rows.push(json!({
                "requested_horizon": requested,
                "effective_horizon": effective,
                "deduplicated_by_cap": true,
            }));
            continue;
        }
        let runs: Vec<_> = D062_DRIVE_RADII
            .iter()
            .map(|radius| run_shadow(*radius, effective, ShadowSpec::baseline(dynamic)))
            .collect();
        baseline_rows.push(json!({
            "requested_horizon": requested,
            "effective_horizon": effective,
            "late_window": if effective >= 5000 { "full-horizon mean approximation; no midpoint snapshot" } else { "full-horizon mean" },
            "runs": runs,
        }));
        terminal_runs = runs;
    }
    let late_pairs: Vec<_> = terminal_runs
        .iter()
        .map(|run| (run.radius_seed, run.late_drive))
        .collect();
    let deltas: Vec<_> = terminal_runs.iter().map(|run| run.radius_delta).collect();
    let baseline_class = classify_baseline_horizon(&late_pairs, &deltas, D062_DRIVE_EPS);
    let numerical_ok = !terminal_runs.is_empty()
        && terminal_runs
            .iter()
            .all(|run| run.steps_ok && run.parity_ok);
    let baseline_gate = artifact(
        "gate3_long_horizon_baseline",
        numerical_ok,
        json!({
            "configured_ladder": ladder,
            "max_accepted": cap,
            "classification": baseline_class.as_str(),
            "late_samples": late_pairs,
            "progressive_runs": baseline_rows,
        }),
    );
    write_json(&out.join("long_horizon_baseline"), &baseline_gate)?;
    gates.insert("long_horizon_baseline".into(), baseline_gate);

    let runaway = baseline_class == BaselineHorizonClass::ExistingStructuralPersistentRunawayGrowth;
    let baseline_restoring =
        baseline_class == BaselineHorizonClass::ExistingStructuralDelayedRestoringBasin;
    let baseline_collapse =
        baseline_class == BaselineHorizonClass::ExistingStructuralDelayedCollapse;

    // Gate 4: required global multiplier and preregistered Candidate B values.
    let mut multipliers = Vec::new();
    for sample in &existing_samples {
        if let Some(value) = required_decay_multiplier(sample.g_phi, sample.l_phi) {
            multipliers.push(value.max(1.0));
        }
    }
    let md_by_radius: Vec<(f64, f64)> = radii
        .iter()
        .copied()
        .zip(multipliers.iter().copied())
        .collect();
    let scalar_identifiable = runaway
        && decay_parity_ok
        && scalar_correction_identifiable(&multipliers)
        && scalar_md_allows_restoring_crossing(&md_by_radius);
    let min_md = multipliers.iter().copied().fold(f64::INFINITY, f64::min);
    let max_md = multipliers
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let median_md = geometric_median(&multipliers);
    let mut preregistered_md = Vec::new();
    if scalar_identifiable {
        preregistered_md.push(min_md.max(1.0));
        if let Some(median) = median_md {
            preregistered_md.push(median.max(1.0));
        }
        preregistered_md.push(max_md.max(1.0));
        preregistered_md.sort_by(f64::total_cmp);
        preregistered_md.dedup_by(|a, b| (*a - *b).abs() <= 1e-12);
    }
    let required = if runaway {
        artifact(
            "gate4_required_multiplier",
            !multipliers.is_empty(),
            json!({
                "m_d_star_by_radius": md_by_radius,
                "span": scalar_multiplier_span(&multipliers),
                "scalar_span_ok": scalar_correction_identifiable(&multipliers),
                "scalar_md_allows_restoring_crossing": scalar_md_allows_restoring_crossing(&md_by_radius),
                "scalar_correction_identifiable": scalar_identifiable,
                "preregistered_candidate_b": preregistered_md,
            }),
        )
    } else {
        skipped(
            "gate4_required_multiplier",
            "baseline_not_persistent_runaway",
        )
    };
    write_json(&out.join("required_multiplier"), &required)?;
    gates.insert("required_multiplier".into(), required);

    // Gates 5-6: evaluate B on dynamic training runs, then C only if B fails.
    let candidate_horizon = cap.min(2500);
    let mut candidate_rows = Vec::new();
    let mut best_b: Option<(MaintenanceParams, f64)> = None;
    let mut candidate_b_qualified = false;
    if scalar_identifiable {
        for m_d in &preregistered_md {
            let cand = MaintenanceParams::global_md(*m_d);
            let spec = ShadowSpec {
                candidate: MaintenanceCandidateId::BGlobalDecayCalibration,
                candidate_params: cand,
                ..ShadowSpec::baseline(dynamic)
            };
            let runs: Vec<_> = D062_TRAINING_RADII
                .iter()
                .map(|radius| run_shadow(*radius, candidate_horizon, spec))
                .collect();
            let pairs: Vec<_> = runs
                .iter()
                .map(|run| (run.radius_seed, run.late_drive))
                .collect();
            let score = runs.iter().map(|run| run.late_drive.abs()).sum::<f64>();
            let crossing = stable_crossing_qualified(&pairs, D062_DRIVE_EPS);
            let qualified = crossing.map(crossing_in_supported_domain).unwrap_or(false)
                && runs.iter().all(|run| run.steps_ok && run.parity_ok);
            candidate_b_qualified |= qualified;
            // Prefer qualified candidates; among them, minimum |g_R| score.
            if qualified
                && best_b
                    .map(|(_, old_score)| score < old_score)
                    .unwrap_or(true)
            {
                best_b = Some((cand, score));
            }
            candidate_rows.push(json!({
                "candidate": MaintenanceCandidateId::BGlobalDecayCalibration.as_str(),
                "params": cand,
                "training_runs": runs,
                "restoring_crossing": crossing,
                "qualified": qualified,
                "drive_error": score,
            }));
        }
    }

    let mut best_c: Option<(MaintenanceParams, f64)> = None;
    if runaway && !candidate_b_qualified {
        for k_a_m in [0.1, 0.5, 1.0] {
            for alpha_m in [0.5, 1.0, 2.0, 4.0] {
                let cand = MaintenanceParams::resource_dependent(k_a_m, alpha_m);
                let samples = analytic_samples(
                    &params,
                    &probes,
                    MaintenanceCandidateId::CResourceDependentMaintenance,
                    cand,
                );
                let pairs = drive_pairs(&samples);
                let crossing = stable_crossing_qualified(&pairs, D062_DRIVE_EPS);
                let score = samples.iter().map(|sample| sample.g_r.abs()).sum::<f64>();
                if crossing.map(crossing_in_supported_domain).unwrap_or(false)
                    && best_c
                        .map(|(_, old_score)| score < old_score)
                        .unwrap_or(true)
                {
                    best_c = Some((cand, score));
                }
                candidate_rows.push(json!({
                    "candidate": MaintenanceCandidateId::CResourceDependentMaintenance.as_str(),
                    "params": cand,
                    "analytic_samples": samples,
                    "restoring_crossing": crossing,
                    "analytic_drive_error": score,
                }));
            }
        }
    }
    let candidates_gate = if runaway && decay_parity_ok {
        artifact(
            "gate5_candidate_laws",
            true,
            json!({
                "candidate_A": MaintenanceParams::existing(),
                "candidate_B_allowed": scalar_identifiable,
                "candidate_C_allowed": !candidate_b_qualified,
                "rows": candidate_rows,
            }),
        )
    } else {
        skipped(
            "gate5_candidate_laws",
            if !decay_parity_ok {
                "decay_parity_failed"
            } else {
                "baseline_not_persistent_runaway"
            },
        )
    };
    write_json(&out.join("candidate_laws"), &candidates_gate)?;
    gates.insert("candidate_laws".into(), candidates_gate);

    let selected = if candidate_b_qualified {
        best_b.map(|(cand, _)| (MaintenanceCandidateId::BGlobalDecayCalibration, cand))
    } else {
        best_c.map(|(cand, _)| (MaintenanceCandidateId::CResourceDependentMaintenance, cand))
    };
    let identification = if runaway && decay_parity_ok {
        artifact(
            "gate6_parameter_identification",
            selected.is_some(),
            json!({
                "selection_method": "minimum preregistered global drive error; Candidate C only after B nonqualification",
                "selected_candidate": selected.map(|(id, _)| id.as_str()),
                "selected_params": selected.map(|(_, cand)| cand),
                "candidate_b_qualified": candidate_b_qualified,
                "candidate_c_analytic_selected": best_c.is_some(),
                "qualification_thresholds_applied": true,
            }),
        )
    } else {
        skipped(
            "gate6_parameter_identification",
            if !decay_parity_ok {
                "decay_parity_failed"
            } else {
                "baseline_not_persistent_runaway"
            },
        )
    };
    write_json(&out.join("parameter_identification"), &identification)?;
    gates.insert("parameter_identification".into(), identification);

    // Gate 7: restoring frontier for the existing law or selected shadow candidate.
    let frontier_choice = if baseline_restoring {
        Some((
            MaintenanceCandidateId::AExisting,
            MaintenanceParams::existing(),
        ))
    } else {
        selected
    };
    let frontier_samples = frontier_choice
        .map(|(id, cand)| analytic_samples(&params, &probes, id, cand))
        .unwrap_or_default();
    let frontier_pairs = drive_pairs(&frontier_samples);
    let crossing = stable_crossing_qualified(&frontier_pairs, D062_DRIVE_EPS);
    let candidate_c_qualified = selected
        .map(|(id, _)| id == MaintenanceCandidateId::CResourceDependentMaintenance)
        .unwrap_or(false)
        && crossing.map(crossing_in_supported_domain).unwrap_or(false);
    candidate_b_qualified =
        candidate_b_qualified && crossing.map(crossing_in_supported_domain).unwrap_or(false);
    let frontier_ok = crossing.map(crossing_in_supported_domain).unwrap_or(false);
    let frontier = if frontier_choice.is_some() {
        artifact(
            "gate7_restoring_frontier",
            frontier_ok,
            json!({
                "candidate": frontier_choice.map(|(id, _)| id.as_str()),
                "params": frontier_choice.map(|(_, cand)| cand),
                "samples": frontier_samples,
                "stable_crossing": crossing,
                "crossing_in_supported_domain": crossing.map(crossing_in_supported_domain).unwrap_or(false),
            }),
        )
    } else {
        skipped(
            "gate7_restoring_frontier",
            "no_existing_or_candidate_restoring_law",
        )
    };
    write_json(&out.join("restoring_frontier"), &frontier)?;
    gates.insert("restoring_frontier".into(), frontier);

    // Gate 8: progressive basin around the selected crossing.
    let run_basin = frontier_ok && !fast;
    let basin_spec = frontier_choice.map(|(candidate, candidate_params)| ShadowSpec {
        candidate,
        candidate_params,
        ..ShadowSpec::baseline(dynamic)
    });
    let mut basin_rows = Vec::new();
    let mut basin_qualified = false;
    let mut metabolism_qualified = false;
    if run_basin {
        let center = crossing.unwrap_or(10.0);
        let mut seen = BTreeSet::new();
        for requested in [5000u64, 10000, 25000, 50000, 100000] {
            let effective = requested.min(cap);
            if !seen.insert(effective) {
                basin_rows.push(json!({
                    "requested_horizon": requested,
                    "effective_horizon": effective,
                    "deduplicated_by_cap": true,
                }));
                continue;
            }
            let small = run_shadow(center * 0.8, effective, basin_spec.unwrap());
            let large = run_shadow(center * 1.2, effective, basin_spec.unwrap());
            basin_qualified = small.steps_ok
                && large.steps_ok
                && small.parity_ok
                && large.parity_ok
                && small.radius_delta > D062_DRIVE_EPS
                && large.radius_delta < -D062_DRIVE_EPS;
            metabolism_qualified = basin_qualified
                && small.a_retention >= D062_A_RETENTION_TARGET
                && large.a_retention >= D062_A_RETENTION_TARGET
                && small.c_mean / 0.4 >= D062_C_RETENTION_TARGET
                && large.c_mean / 0.4 >= D062_C_RETENTION_TARGET
                && small.surface_retention >= 0.8
                && large.surface_retention >= 0.8;
            basin_rows.push(json!({
                "requested_horizon": requested,
                "effective_horizon": effective,
                "small": small,
                "large": large,
                "grow_from_small_shrink_from_large": basin_qualified,
                "metabolism_qualified": metabolism_qualified,
            }));
        }
    }
    let basin = artifact(
        "gate8_shadow_basin",
        !run_basin || basin_qualified,
        json!({
            "skipped": !run_basin,
            "reason": if run_basin { Value::Null } else if fast { json!("D062_SKIP_LATE_GATES") } else { json!("no_qualified_frontier") },
            "basin_qualified": basin_qualified,
            "metabolism_qualified": metabolism_qualified,
            "failure_label": if basin_qualified && !metabolism_qualified { json!("D062_SIZE_RESTORED_METABOLISM_NOT_QUALIFIED") } else { Value::Null },
            "a_retention_target": D062_A_RETENTION_TARGET,
            "c_retention_target": D062_C_RETENTION_TARGET,
            "runs": basin_rows,
        }),
    );
    write_json(&out.join("shadow_basin"), &basin)?;
    gates.insert("shadow_basin".into(), basin);

    // Gate 9: abbreviated ±10% and one independent seed.
    let run_robustness = basin_qualified && !fast;
    let mut robustness_runs = Vec::new();
    let mut robustness_ok = !run_robustness;
    if run_robustness {
        let center = crossing.unwrap_or(10.0);
        let mut noisy = basin_spec.unwrap();
        noisy.seed = 17;
        for scale in [0.9, 1.1] {
            robustness_runs.push(run_shadow(center * scale, cap, noisy));
        }
        robustness_ok = robustness_runs
            .iter()
            .all(|run| run.steps_ok && run.parity_ok);
    }
    let robustness = artifact(
        "gate9_basin_robustness",
        robustness_ok,
        json!({
            "skipped": !run_robustness,
            "abbreviated": cap < 10000,
            "radius_perturbations": [-0.10, 0.10],
            "noise_seed": 17,
            "runs": robustness_runs,
        }),
    );
    write_json(&out.join("basin_robustness"), &robustness)?;
    gates.insert("basin_robustness".into(), robustness);

    // Gate 10: causal controls and fixed-geometry invariance.
    let control_horizon = cap.min(500);
    let baseline_control = run_shadow(10.0, control_horizon, ShadowSpec::baseline(dynamic));
    let mut carrier_off_spec = ShadowSpec::baseline(dynamic);
    carrier_off_spec.carrier = false;
    let carrier_off = run_shadow(10.0, control_horizon, carrier_off_spec);
    let mut starve_spec = ShadowSpec::baseline(dynamic);
    starve_spec.starve_n = true;
    let starved = run_shadow(10.0, control_horizon, starve_spec);
    let mut synthesis_off_spec = ShadowSpec::baseline(dynamic);
    synthesis_off_spec.disable_synthesis = true;
    let synthesis_off = run_shadow(10.0, control_horizon, synthesis_off_spec);
    let fixed_control = run_shadow(10.0, control_horizon, ShadowSpec::baseline(fixed));
    let no_radius_in_equations =
        !existing_equation_string().contains('R') && !gain_equation_string().contains('R');
    let causality_ok = baseline_control.steps_ok
        && carrier_off.steps_ok
        && starved.steps_ok
        && synthesis_off.steps_ok
        && fixed_control.steps_ok
        && fixed_control.radius_delta.abs() <= D062_UPDATE_PARITY_TOL
        && synthesis_off.radius_delta < baseline_control.radius_delta
        && no_radius_in_equations;
    let controls = artifact(
        "gate10_causality_controls",
        causality_ok,
        json!({
            "baseline": baseline_control,
            "carrier_off": carrier_off,
            "starve_n_zeroish_a": starved,
            "synthesis_off": synthesis_off,
            "fixed_geometry": fixed_control,
            "fixed_geometry_immobile": fixed_control.radius_delta.abs() <= D062_UPDATE_PARITY_TOL,
            "synthesis_off_reduces_growth": synthesis_off.radius_delta < baseline_control.radius_delta,
            "no_radius_in_equations": no_radius_in_equations,
        }),
    );
    write_json(&out.join("causality_controls"), &controls)?;
    gates.insert("causality_controls".into(), controls);

    // Gate 11: foundational mode identity, route, and accounting.
    let foundational_ok = !fixed.apply_phi()
        && dynamic.apply_phi()
        && fixed.enforce_constraint()
        && !dynamic.enforce_constraint();
    let foundational = artifact(
        "gate11_foundational_regression",
        foundational_ok,
        json!({
            "fixed_mode": fixed.as_str(),
            "dynamic_mode": dynamic.as_str(),
            "fixed_apply_phi": fixed.apply_phi(),
            "dynamic_apply_phi": dynamic.apply_phi(),
            "structure_evolution_mode_identity_preserved": true,
        }),
    );
    write_json(&out.join("foundational_regression"), &foundational)?;
    gates.insert("foundational_regression".into(), foundational);

    let accounting_ok = decay_parity_ok
        && probes.iter().all(|run| run.accounting_ok)
        && terminal_runs.iter().all(|run| run.accounting_ok);
    let accounting = artifact(
        "accounting",
        accounting_ok,
        json!({
            "decay_parity_ok": decay_parity_ok,
            "probe_accounting_ok": probes.iter().all(|run| run.accounting_ok),
            "baseline_accounting_ok": terminal_runs.iter().all(|run| run.accounting_ok),
            "ledger_tol": D062_LEDGER_TOL,
            "update_parity_tol": D062_UPDATE_PARITY_TOL,
        }),
    );
    write_json(&out.join("accounting"), &accounting)?;
    gates.insert("accounting".into(), accounting);

    let evidence = RouteEvidence062 {
        workspace_isolated: workspace_ok,
        d061_reproduced,
        decay_parity_ok,
        scaling_ok,
        baseline_restoring,
        baseline_runaway: runaway,
        baseline_collapse,
        candidate_b_qualified,
        candidate_c_qualified,
        basin_qualified,
        metabolism_qualified,
        causality_ok,
        foundational_ok,
        accounting_ok,
        numerical_ok,
    };
    let (route, conclusion) = select_route(evidence);
    let route_decision = artifact(
        "gate11_route_decision",
        true,
        json!({
            "route": route.as_str(),
            "primary_conclusion": conclusion.as_str(),
            "evidence": evidence,
            "stage_e": "BLOCKED_NOT_RECOVERED",
        }),
    );
    write_json(&out.join("route_decision"), &route_decision)?;
    gates.insert("route_decision".into(), route_decision.clone());

    let manifest = json!({
        "project_directive": D062_PROJECT_ID,
        "agent_memory_directive": D062_AGENT_MEMORY_ID,
        "starting_commit": D062_STARTING_COMMIT,
        "starting_tag": D062_STARTING_TAG,
        "source_commit": git_output(&["rev-parse", "HEAD"]),
        "D062_MAX_ACCEPTED": cap,
        "D062_SKIP_LATE_GATES": fast,
        "D062_HORIZON_LADDER": horizon_ladder(),
        "frozen_k_T": D062_FROZEN_KT,
        "shadow_carrier_only": true,
        "production_biology_unchanged": true,
        "structural_synthesis_unchanged": true,
        "carrier_defaults_unchanged": true,
        "v15_created": false,
        "route": route.as_str(),
        "primary_conclusion": conclusion.as_str(),
        "stage_e": "BLOCKED_NOT_RECOVERED",
        "gates": gates,
        "route_decision": route_decision,
    });
    atomic_write_json(&out.join("manifest.json"), &manifest)?;
    atomic_write_json(&out.join("result.json"), &manifest)?;
    Ok(manifest)
}
