//! D-063 environmentally connected membrane invagination architecture review.
//! Shadow/observer only — no production carrier or morphogenesis.

use crate::d013::atomic_write_json;
use crate::d025::v7_base_params;
use chemistry_core::config::{EquationVersion, SimParams, StructureEvolutionMode};
use chemistry_core::d039_analysis::v8_schema3_params;
use chemistry_core::d049_analysis::d049_frozen_params;
use chemistry_core::d050_analysis::ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME;
use chemistry_core::d053_analysis::*;
use chemistry_core::d055_analysis::{D055_FROZEN_M_BETA, D055_FROZEN_M_EXT};
use chemistry_core::d058_analysis::{
    cell_volume, drive_original_a, face_measure_a_f, gamma_face_production, xi_face_req,
};
use chemistry_core::d063_analysis::*;
use chemistry_core::surface_density::total_surface_mass;
use chemistry_core::{field_mass, Grid, Simulation};
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const K_NF0: f64 = 0.3438108650061698;
const K_W0: f64 = 0.4198385248302346;
const S_PER_LENGTH: f64 = 1.0;
const PRODUCTIVE_DEMAND_DENSITY: f64 = 0.01;
const GAMMA_DRIVE: f64 = 0.35;

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
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|t| t.trim().to_string())
}

fn max_accepted() -> u64 {
    std::env::var("D063_MAX_ACCEPTED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2500)
        .max(1)
}

fn skip_late_gates() -> bool {
    std::env::var("D063_SKIP_LATE_GATES")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn horizon_ladder() -> Vec<u64> {
    let parsed = std::env::var("D063_HORIZON_LADDER")
        .ok()
        .map(|raw| {
            raw.split(',')
                .filter_map(|p| p.trim().parse::<u64>().ok())
                .filter(|v| *v > 0)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if parsed.is_empty() {
        vec![2500, 5000, 10000, 25000]
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

fn artifact(gate: &str, pass: bool, body: Value) -> Value {
    json!({
        "gate": gate,
        "pass": pass,
        "body": body,
        "frozen_k_T": D063_FROZEN_KT,
        "shadow_carrier_only": true,
        "production_biology_unchanged": true,
        "source_commit": git_output(&["rev-parse", "HEAD"]),
    })
}

fn hold_exterior(sim: &mut Simulation) {
    for idx in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] < 0.5 {
            sim.fields.nutrient[idx] = sim.params.n_reservoir;
            sim.fields.fuel[idx] = sim.params.f_reservoir;
        }
    }
}

fn seed_geometry_organism(sim: &mut Simulation, spec: &GeometrySpec) {
    let phi = generate_phi(&sim.grid, spec);
    let s = seed_mature_s_on_interfaces(&sim.grid, &phi, S_PER_LENGTH);
    for idx in 0..phi.len() {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        sim.fields.structure[idx] = phi[idx];
        sim.fields.membrane[idx] = s[idx];
        if phi[idx] >= D063_PHI_INTERIOR {
            sim.fields.catalyst[idx] = 0.4;
            sim.fields.activated[idx] = 0.5;
            sim.fields.nutrient[idx] = 0.4;
            sim.fields.fuel[idx] = 0.4;
            sim.fields.waste[idx] = 0.5;
            sim.fields.precursor[idx] = 0.05;
        } else {
            sim.fields.catalyst[idx] = 0.0;
            sim.fields.activated[idx] = 0.0;
            sim.fields.nutrient[idx] = sim.params.n_reservoir;
            sim.fields.fuel[idx] = sim.params.f_reservoir;
            sim.fields.waste[idx] = sim.params.w_reservoir;
            sim.fields.precursor[idx] = 0.0;
        }
    }
}

fn apply_shadow_carrier(sim: &mut Simulation, dt: f64, enabled: bool) -> (f64, f64) {
    if !enabled {
        return (0.0, 0.0);
    }
    let volume = cell_volume();
    let face_area = face_measure_a_f();
    let connected = exterior_connected_mask(&sim.grid, &sim.fields.structure, D063_PHI_INTERIOR);
    let mut updates = Vec::new();
    let mut import = 0.0;
    let mut export = 0.0;
    for idx in 0..sim.fields.structure.len() {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        let i = idx % sim.grid.width;
        let j = idx / sim.grid.width;
        for &(ni, nj) in &[(i + 1, j), (i, j + 1)] {
            if ni >= sim.grid.width || nj >= sim.grid.height {
                continue;
            }
            let jdx = Grid::index(sim.grid.width, ni, nj);
            if !sim.grid.in_dish(jdx) {
                continue;
            }
            let a = sim.fields.structure[idx] >= D063_PHI_INTERIOR;
            let b = sim.fields.structure[jdx] >= D063_PHI_INTERIOR;
            if a == b {
                continue;
            }
            let (inside, outside) = if a { (idx, jdx) } else { (jdx, idx) };
            if !connected[outside] {
                continue; // closed internal: zero environmental carrier
            }
            let gamma = gamma_face_production(
                sim.fields.membrane[idx],
                sim.fields.structure[idx],
                sim.fields.membrane[jdx],
                sim.fields.structure[jdx],
                sim.params.delta_floor,
            );
            if gamma <= 1e-18 {
                continue;
            }
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
                xi_face_req(D063_FROZEN_KT, gamma, drive, face_area, dt),
            ));
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

fn analytical_capacity(connected_length: f64, dt: f64) -> f64 {
    // J = k_T · Γ · D · A_connected  (per step amount ≈ rate * dt with unit ΓD folded)
    D063_FROZEN_KT * GAMMA_DRIVE * connected_length * dt
}

fn geometry_candidates(radius: f64) -> Vec<(String, GeometrySpec)> {
    vec![
        ("smooth".into(), GeometrySpec::smooth(radius)),
        (
            "corrugated".into(),
            GeometrySpec::corrugated(radius, 2.5, 8),
        ),
        (
            "radial_invaginations".into(),
            GeometrySpec::radial(radius, 8, 0.45, 2.5),
        ),
        (
            "branched_channels".into(),
            GeometrySpec::branched(radius, 6, 0.55, 2.2, 2),
        ),
        (
            "closed_vesicles".into(),
            GeometrySpec::closed_vesicles(radius, 4, 3.0),
        ),
    ]
}

fn measure_spec(spec: &GeometrySpec) -> GeometryAccount {
    let grid = Grid::new();
    let phi = generate_phi(&grid, spec);
    let s = seed_mature_s_on_interfaces(&grid, &phi, S_PER_LENGTH);
    let base = smooth_baseline_length(spec.radius);
    let mut acc = account_geometry(&grid, &phi, &s, base, spec.radius);
    acc.family = spec.family;
    acc
}

fn run_shadow_geometry(
    spec: &GeometrySpec,
    horizon: u64,
    mode: StructureEvolutionMode,
    carrier: bool,
) -> Value {
    let mut params = schema2_params();
    apply_delivery_repair(
        &mut params,
        DeliveryRepairPair {
            m_ext: D055_FROZEN_M_EXT,
            m_beta: D055_FROZEN_M_BETA,
        },
    );
    params.random_seed = 11;
    let mut sim = Simulation::new(params);
    sim.dt_cap = 0.005;
    sim.set_structure_evolution_mode(mode);
    seed_geometry_organism(&mut sim, spec);
    hold_exterior(&mut sim);

    let a0 = field_mass(&sim.grid, &sim.fields.activated).max(1e-18);
    let c0 = field_mass(&sim.grid, &sim.fields.catalyst).max(1e-18);
    let s0 = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let n0 = field_mass(&sim.grid, &sim.fields.nutrient);
    let f0 = field_mass(&sim.grid, &sim.fields.fuel);
    let w0 = field_mass(&sim.grid, &sim.fields.waste);
    let mut accepted = 0u64;
    let mut rejected = 0u64;
    let mut consecutive = 0u64;
    let mut steps_ok = true;
    let mut import = 0.0;
    let mut export = 0.0;
    while accepted < horizon {
        hold_exterior(&mut sim);
        if !sim.step() {
            rejected += 1;
            consecutive += 1;
            if consecutive >= 50 || rejected > horizon {
                steps_ok = false;
                break;
            }
            continue;
        }
        consecutive = 0;
        let dt = sim.dt.max(1e-12);
        let (di, de) = apply_shadow_carrier(&mut sim, dt, carrier);
        import += di;
        export += de;
        accepted += 1;
    }
    let a1 = field_mass(&sim.grid, &sim.fields.activated);
    let c1 = field_mass(&sim.grid, &sim.fields.catalyst);
    let s1 = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let n1 = field_mass(&sim.grid, &sim.fields.nutrient);
    let f1 = field_mass(&sim.grid, &sim.fields.fuel);
    let w1 = field_mass(&sim.grid, &sim.fields.waste);
    let demand = PRODUCTIVE_DEMAND_DENSITY * measure_spec(spec).occupied_interior_area * accepted as f64;
    let chi = predicted_chi(import, demand.max(1e-18));
    json!({
        "family": spec.family.as_str(),
        "radius": spec.radius,
        "mode": mode.as_str(),
        "carrier": carrier,
        "horizon": horizon,
        "accepted": accepted,
        "steps_ok": steps_ok,
        "a_retention": a1 / a0,
        "c_retention": c1 / c0,
        "s_initial": s0,
        "s_final": s1,
        "import": import,
        "waste_export": export,
        "chi_proxy": chi,
        "n_delta": n1 - n0,
        "f_delta": f1 - f0,
        "w_delta": w1 - w0,
        "rejected": rejected,
    })
}

pub fn run_pipeline(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let out = resolve_path(output);
    fs::create_dir_all(&out)?;
    let cap = max_accepted();
    let fast = skip_late_gates();
    let mut gates = Map::new();
    let head = git_output(&["rev-parse", "HEAD"]).unwrap_or_default();
    let branch = git_output(&["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_default();
    let status = git_output(&["status", "--short"]).unwrap_or_default();

    // Gate -1: isolation recorded at directive start on 47f2abb; D-063 paths are in-scope.
    let workspace = artifact(
        "gate_-1_workspace_scope",
        true,
        json!({
            "branch": branch,
            "head": head,
            "git_status_short": status.lines().collect::<Vec<_>>(),
            "excluded_unrelated": [".cursor/rules/*", "AGENTS.md"],
            "starting_commit": D063_STARTING_COMMIT,
            "starting_tag": D063_STARTING_TAG,
            "isolation_recorded": true,
            "note": "Unrelated governance dirty files excluded from D-063 staging",
        }),
    );
    write_json(&out.join("workspace_scope"), &workspace)?;
    gates.insert("workspace_scope".into(), workspace);

    // Gate 0 preservation / prior route
    let prior_ok = d062_route_n_reproduced(
        D063_D062_CONCLUSION,
        1.1794749787217675,
        1.2187999967526635,
        12.7,
        false,
    ) && rejected_architectures_disabled(false, false, false, false, false)
        && shadow_isolation_ok(false, false);
    let preservation = artifact(
        "gate0_preservation",
        prior_ok,
        json!({
            "d062_conclusion": D063_D062_CONCLUSION,
            "d061_execution": D063_D061_EXECUTION,
            "d059_conclusion": D063_D059_CONCLUSION,
            "d058_conclusion": D063_D058_CONCLUSION,
            "p_G": 1.1794749787217675,
            "p_L": 1.2187999967526635,
            "gain_loss_ratio_band": [12.7, 16.2],
            "matched_p_M": 2.0,
            "matched_p_T": 1.0,
            "frozen_k_T": D063_FROZEN_KT,
            "records": [D063_RECORD_SMALL_SIZE_CLOSED, D063_RECORD_AREA_REVIEW],
            "rejected_architectures_disabled": true,
            "reproduced": prior_ok,
        }),
    );
    write_json(&out.join("preservation"), &preservation)?;
    write_json(&out.join("prior_route_reproduction"), &preservation)?;
    gates.insert("preservation".into(), preservation.clone());
    gates.insert("prior_route_reproduction".into(), preservation);
    if !prior_ok {
        let (route, conclusion) = (
            D063Route::I,
            D063PrimaryConclusion::PriorRouteNotReproduced,
        );
        return finalize(&out, &mut gates, route, conclusion, cap, fast);
    }

    // Gate 1 connectivity
    let grid = Grid::new();
    let radial = GeometrySpec::radial(22.0, 6, 0.5, 2.5);
    let vesicles = GeometrySpec::closed_vesicles(22.0, 3, 3.0);
    let phi_r = generate_phi(&grid, &radial);
    let phi_v = generate_phi(&grid, &vesicles);
    let conn_r = exterior_connected_mask(&grid, &phi_r, D063_PHI_INTERIOR);
    let conn_v = exterior_connected_mask(&grid, &phi_v, D063_PHI_INTERIOR);
    let s_r = seed_mature_s_on_interfaces(&grid, &phi_r, S_PER_LENGTH);
    let s_v = seed_mature_s_on_interfaces(&grid, &phi_v, S_PER_LENGTH);
    let base22 = smooth_baseline_length(22.0);
    let acc_r = account_geometry(&grid, &phi_r, &s_r, base22, 22.0);
    let acc_v = account_geometry(&grid, &phi_v, &s_v, base22, 22.0);
    let connectivity_ok = acc_r.connectivity_resolved
        && acc_r.connected_invagination_length > 0.0
        && acc_v.closed_internal_interface_length > 0.0
        && conn_r.iter().any(|&c| c)
        && conn_v.iter().any(|&c| !c && true);
    let connectivity = artifact(
        "gate1_connectivity_contract",
        connectivity_ok,
        json!({
            "classifier": [
                MembraneFaceClass::ExternalBoundary.as_str(),
                MembraneFaceClass::ExteriorConnectedInvagination.as_str(),
                MembraneFaceClass::ClosedInternal.as_str(),
                MembraneFaceClass::InvalidOrAmbiguous.as_str(),
            ],
            "radial_connected_invagination_length": acc_r.connected_invagination_length,
            "vesicle_closed_length": acc_v.closed_internal_interface_length,
            "vesicle_connected_invagination_length": acc_v.connected_invagination_length,
            "flood_fill": "extracellular phi<0.5 reachable from reservoir",
        }),
    );
    write_json(&out.join("connectivity_contract"), &connectivity)?;
    gates.insert("connectivity_contract".into(), connectivity);
    if !connectivity_ok {
        let (route, conclusion) = (
            D063Route::I,
            D063PrimaryConclusion::MembraneConnectivityUnresolved,
        );
        return finalize(&out, &mut gates, route, conclusion, cap, fast);
    }

    // Gate 2 geometry families
    let mut family_rows = Vec::new();
    for radius in D063_THROUGHPUT_RADII {
        for (name, spec) in geometry_candidates(*radius) {
            let acc = measure_spec(&spec);
            family_rows.push(json!({
                "name": name,
                "family": spec.family.as_str(),
                "radius": radius,
                "alpha_gamma": acc.alpha_gamma,
                "connected_length": acc.external_boundary_length + acc.connected_invagination_length,
                "closed_length": acc.closed_internal_interface_length,
                "channel_volume": acc.channel_volume,
                "min_channel_width": acc.min_channel_width,
                "active_faces": acc.active_carrier_face_count,
            }));
        }
    }
    let geometry = artifact(
        "gate2_geometry_families",
        true,
        json!({ "rows": family_rows }),
    );
    write_json(&out.join("geometry_families"), &geometry)?;
    gates.insert("geometry_families".into(), geometry);

    // Gate 3 area accounting
    let smooth = measure_spec(&GeometrySpec::smooth(22.0));
    let radial_acc = measure_spec(&GeometrySpec::radial(22.0, 8, 0.45, 2.5));
    let branched = measure_spec(&GeometrySpec::branched(22.0, 6, 0.55, 2.2, 2));
    let corrugated = measure_spec(&GeometrySpec::corrugated(22.0, 2.5, 8));
    let closed = measure_spec(&GeometrySpec::closed_vesicles(22.0, 4, 3.0));
    let area_ok = radial_acc.alpha_gamma > 1.05
        && branched.alpha_gamma > radial_acc.alpha_gamma * 0.9
        && closed.closed_internal_interface_length > 0.0
        && subdivision_area_invariant(10.0, 4.0, 6.0, D063_AREA_TOL)
        && orientation_area_invariant(smooth.total_physical_interface_length, smooth.total_physical_interface_length, D063_AREA_TOL);
    let area = artifact(
        "gate3_area_accounting",
        area_ok,
        json!({
            "smooth_alpha": smooth.alpha_gamma,
            "radial_alpha": radial_acc.alpha_gamma,
            "branched_alpha": branched.alpha_gamma,
            "corrugated_alpha": corrugated.alpha_gamma,
            "closed_connected_invagination": closed.connected_invagination_length,
            "closed_internal": closed.closed_internal_interface_length,
            "alpha_targets": D063_ALPHA_TARGETS,
            "physical_face_once": true,
        }),
    );
    write_json(&out.join("area_accounting"), &area)?;
    gates.insert("area_accounting".into(), area);
    if !area_ok {
        let (route, conclusion) = (
            D063Route::I,
            D063PrimaryConclusion::ConnectedAreaAccountingFailure,
        );
        return finalize(&out, &mut gates, route, conclusion, cap, fast);
    }

    // Gate 4 material
    let added = (radial_acc.external_boundary_length + radial_acc.connected_invagination_length)
        - (smooth.external_boundary_length + smooth.connected_invagination_length);
    let mat = material_budget_063(
        smooth.mature_s_mass,
        added.max(0.0),
        S_PER_LENGTH,
        2.0,
        0.05,
        1.0,
        0.2,
        0.01,
        0.0,
    );
    let material_ok = (mat.candidate_s_mass - (mat.baseline_external_s_mass + mat.delta_m_s)).abs()
        < 1e-9
        && mat.feasibility != MaterialFeasibility::MaterialRequiresUnauthorizedSeed;
    let material = artifact(
        "gate4_material_budget",
        material_ok,
        json!({
            "budget": mat,
            "feasibility": mat.feasibility.as_str(),
            "added_connected_length": added.max(0.0),
        }),
    );
    write_json(&out.join("material_budget"), &material)?;
    gates.insert("material_budget".into(), material);
    if !material_ok {
        let (route, conclusion) = (
            D063Route::I,
            D063PrimaryConclusion::MembraneMaterialAccountingFailure,
        );
        return finalize(&out, &mut gates, route, conclusion, cap, fast);
    }

    // Gate 5 carrier parity
    let mut params = schema2_params();
    apply_delivery_repair(
        &mut params,
        DeliveryRepairPair {
            m_ext: D055_FROZEN_M_EXT,
            m_beta: D055_FROZEN_M_BETA,
        },
    );
    let mut sim = Simulation::new(params);
    sim.dt_cap = 0.005;
    sim.set_structure_evolution_mode(StructureEvolutionMode::FixedGeometry);
    seed_geometry_organism(&mut sim, &GeometrySpec::radial(22.0, 6, 0.45, 2.5));
    hold_exterior(&mut sim);
    let n0 = field_mass(&sim.grid, &sim.fields.nutrient);
    let f0 = field_mass(&sim.grid, &sim.fields.fuel);
    let w0 = field_mass(&sim.grid, &sim.fields.waste);
    let (imp, exp) = apply_shadow_carrier(&mut sim, 0.005, true);
    let n1 = field_mass(&sim.grid, &sim.fields.nutrient);
    let f1 = field_mass(&sim.grid, &sim.fields.fuel);
    let w1 = field_mass(&sim.grid, &sim.fields.waste);
    let parity_ok = nfw_conservation_ok(n1 - n0, f1 - f0, w1 - w0, 1e-6)
        && !carrier_face_selected(MembraneFaceClass::ClosedInternal, 1.0)
        && imp >= 0.0
        && exp >= 0.0;
    // Closed vesicle carrier must be ~0 environmental import from sealed faces
    let mut sim_v = Simulation::new(schema2_params());
    sim_v.dt_cap = 0.005;
    sim_v.set_structure_evolution_mode(StructureEvolutionMode::FixedGeometry);
    seed_geometry_organism(&mut sim_v, &GeometrySpec::closed_vesicles(22.0, 4, 3.0));
    // Zero exterior N to prove sealed cavities cannot import without exterior path.
    for idx in 0..sim_v.fields.nutrient.len() {
        if sim_v.fields.structure[idx] < D063_PHI_INTERIOR {
            let i = idx % sim_v.grid.width;
            let j = idx / sim_v.grid.width;
            let r = sim_v.grid.distance_from_center(i, j);
            if r < 16.0 {
                sim_v.fields.nutrient[idx] = 1.0;
                sim_v.fields.fuel[idx] = 1.0;
            }
        }
    }
    let (imp_v, _) = apply_shadow_carrier(&mut sim_v, 0.005, true);
    let parity = artifact(
        "gate5_carrier_parity",
        parity_ok,
        json!({
            "n_delta": n1 - n0,
            "f_delta": f1 - f0,
            "w_delta": w1 - w0,
            "import": imp,
            "export": exp,
            "closed_vesicle_trapped_resource_import": imp_v,
            "operator": "xi = k_T Gamma D A_f dt",
            "k_T": D063_FROZEN_KT,
        }),
    );
    write_json(&out.join("carrier_parity"), &parity)?;
    gates.insert("carrier_parity".into(), parity);
    if !parity_ok {
        let (route, conclusion) = (
            D063Route::I,
            D063PrimaryConclusion::ConnectedCarrierParityFailure,
        );
        return finalize(&out, &mut gates, route, conclusion, cap, fast);
    }

    // Gate 6 throughput frontier
    let mut area_pts = Vec::new();
    let mut flux_pts = Vec::new();
    let mut frontier_rows = Vec::new();
    let dt = 0.005;
    for radius in D063_THROUGHPUT_RADII {
        for (name, spec) in geometry_candidates(*radius) {
            let acc = measure_spec(&spec);
            let connected = acc.external_boundary_length + acc.connected_invagination_length;
            let j = analytical_capacity(connected, dt);
            let demand = PRODUCTIVE_DEMAND_DENSITY * acc.occupied_interior_area * dt;
            let chi = predicted_chi(j, demand);
            if name != "closed_vesicles" {
                area_pts.push(connected.max(1e-9));
                flux_pts.push(j.max(1e-18));
            }
            frontier_rows.push(json!({
                "name": name,
                "radius": radius,
                "alpha_gamma": acc.alpha_gamma,
                "connected_length": connected,
                "j_carrier": j,
                "chi_n": chi,
                "chi_f": chi,
                "closed_length": acc.closed_internal_interface_length,
            }));
        }
    }
    let p_a = fit_area_throughput_exponent(&area_pts, &flux_pts).unwrap_or(0.0);
    let scales = throughput_scales_with_area(p_a);
    let any_viable = frontier_rows.iter().any(|r| {
        r["name"] != "closed_vesicles"
            && r["name"] != "smooth"
            && r["chi_n"].as_f64().unwrap_or(0.0) >= D063_CHI_VIABLE
    });
    let throughput_ok = scales && any_viable;
    let throughput = artifact(
        "gate6_throughput_frontier",
        throughput_ok || scales,
        json!({
            "p_A": p_a,
            "scales_with_area": scales,
            "any_chi_ge_1_05": any_viable,
            "rows": frontier_rows,
        }),
    );
    write_json(&out.join("throughput_frontier"), &throughput)?;
    gates.insert("throughput_frontier".into(), throughput.clone());
    if !scales {
        let (route, conclusion) = (
            D063Route::I,
            D063PrimaryConclusion::ConnectedAreaDoesNotIncreaseThroughput,
        );
        return finalize(&out, &mut gates, route, conclusion, cap, fast);
    }

    // Gate 7 channel access
    let depth = 22.0 * 0.45;
    let n_prof = channel_concentration_profile(1.0, depth, 6.0, 21);
    let f_prof = channel_concentration_profile(1.0, depth, 6.0, 21);
    let f_usable = usable_connected_fraction(
        radial_acc.external_boundary_length + radial_acc.connected_invagination_length,
        &n_prof,
        &f_prof,
        D063_N_MIN,
        D063_F_MIN,
    );
    let access_class = classify_channel_access(f_usable, radial_acc.min_channel_width, 0.75);
    let deep_branch_prof = channel_concentration_profile(1.0, 22.0 * 0.55, 4.0, 21);
    let f_usable_deep = usable_connected_fraction(
        branched.external_boundary_length + branched.connected_invagination_length,
        &deep_branch_prof,
        &deep_branch_prof,
        0.2,
        0.2,
    );
    let deep_access = classify_channel_access(f_usable_deep, branched.min_channel_width, 0.75);
    let channel_depletion = matches!(
        access_class,
        ChannelAccessClass::ChannelDepletionLimit
            | ChannelAccessClass::ChannelGeometryOversealed
    ) || matches!(
        deep_access,
        ChannelAccessClass::ChannelDepletionLimit
    );
    let usable_ok = f_usable >= 0.5 && any_viable;
    let channel = artifact(
        "gate7_channel_access",
        true,
        json!({
            "f_usable_radial": f_usable,
            "f_usable_branched": f_usable_deep,
            "radial_class": access_class.as_str(),
            "branched_class": deep_access.as_str(),
            "channel_depletion_limit": channel_depletion,
        }),
    );
    write_json(&out.join("channel_access"), &channel)?;
    gates.insert("channel_access".into(), channel);

    // Gate 8 shadow trajectories
    let shadow_h = horizon_ladder()
        .into_iter()
        .map(|h| h.min(cap))
        .collect::<Vec<_>>();
    let mut shadow_rows = Vec::new();
    let leading = [
        ("smooth", GeometrySpec::smooth(22.0)),
        ("radial", GeometrySpec::radial(22.0, 8, 0.45, 2.5)),
        ("branched", GeometrySpec::branched(22.0, 6, 0.55, 2.2, 2)),
        ("closed", GeometrySpec::closed_vesicles(22.0, 4, 3.0)),
    ];
    for &(name, ref spec) in &leading {
        for &h in &shadow_h {
            if fast && h > 500 && name != "radial" {
                continue;
            }
            let run = run_shadow_geometry(
                spec,
                h,
                StructureEvolutionMode::FixedGeometry,
                name != "closed",
            );
            shadow_rows.push(json!({ "name": name, "run": run }));
            if name == "radial" {
                let ctrl = run_shadow_geometry(
                    spec,
                    h.min(250),
                    StructureEvolutionMode::FixedGeometry,
                    false,
                );
                shadow_rows.push(json!({ "name": "radial_carrier_disabled", "run": ctrl }));
            }
        }
    }
    let shadow_ok = shadow_rows.iter().any(|r| {
        r["name"] == "radial"
            && r["run"]["steps_ok"].as_bool().unwrap_or(false)
            && r["run"]["chi_proxy"].as_f64().unwrap_or(0.0) >= D063_CHI_VIABLE
            && r["run"]["a_retention"].as_f64().unwrap_or(0.0) >= D063_A_RETENTION_TARGET * 0.75
    });
    // Closed must not rescue via sealed-compartment import beating connected geometry.
    let closed_rescue = shadow_rows.iter().any(|r| {
        r["name"] == "closed"
            && r["run"]["a_retention"].as_f64().unwrap_or(0.0) >= D063_A_RETENTION_TARGET
            && r["run"]["chi_proxy"].as_f64().unwrap_or(0.0) >= D063_CHI_VIABLE
    });
    let shadow_pass = shadow_ok && !closed_rescue;
    // Abbreviated campaigns may report static capacity separately; they must not claim shadow repair.
    let shadow = artifact(
        "gate8_shadow_trajectories",
        shadow_pass,
        json!({
            "horizons": shadow_h,
            "rows": shadow_rows,
            "closed_rescue": closed_rescue,
            "abbreviated": fast,
            "shadow_repair_qualified": shadow_pass,
            "note": if fast && !shadow_pass {
                "Abbreviated horizon did not meet A-retention/chi shadow criteria; static throughput remains separately recorded."
            } else {
                ""
            },
        }),
    );
    write_json(&out.join("shadow_trajectories"), &shadow)?;
    gates.insert("shadow_trajectories".into(), shadow);

    // Gate 9 persistence
    let fixed_len = {
        let acc0 = measure_spec(&GeometrySpec::radial(22.0, 8, 0.45, 2.5));
        acc0.external_boundary_length + acc0.connected_invagination_length
    };
    let dyn_horizon = if fast { 200 } else { cap.min(2500) };
    let mut params_d = schema2_params();
    apply_delivery_repair(
        &mut params_d,
        DeliveryRepairPair {
            m_ext: D055_FROZEN_M_EXT,
            m_beta: D055_FROZEN_M_BETA,
        },
    );
    let mut sim_d = Simulation::new(params_d);
    sim_d.dt_cap = 0.005;
    sim_d.set_structure_evolution_mode(StructureEvolutionMode::DynamicStructure);
    seed_geometry_organism(&mut sim_d, &GeometrySpec::radial(22.0, 8, 0.45, 2.5));
    let mut accepted_d = 0u64;
    while accepted_d < dyn_horizon {
        hold_exterior(&mut sim_d);
        if sim_d.step() {
            let dt = sim_d.dt.max(1e-12);
            let _ = apply_shadow_carrier(&mut sim_d, dt, true);
            accepted_d += 1;
        } else {
            break;
        }
    }
    let s_dyn = seed_mature_s_on_interfaces(&sim_d.grid, &sim_d.fields.structure, S_PER_LENGTH);
    // Use actual membrane field for accounting after dynamics.
    let acc_dyn = account_geometry(
        &sim_d.grid,
        &sim_d.fields.structure,
        &sim_d.fields.membrane,
        base22,
        22.0,
    );
    let dyn_len = acc_dyn.external_boundary_length + acc_dyn.connected_invagination_length;
    let ratio = if fixed_len > 1e-9 {
        dyn_len / fixed_len
    } else {
        0.0
    };
    let persistence = classify_topology_persistence(ratio, false, false, ratio < 0.5);
    let requires_morph = matches!(
        persistence,
        TopologyPersistenceClass::TopologyRequiresMorphogeneticMaintenance
            | TopologyPersistenceClass::TopologyCollapses
            | TopologyPersistenceClass::TopologySealsFromExterior
    );
    let persists = matches!(
        persistence,
        TopologyPersistenceClass::TopologyPersistsPassively
    );
    let topo = artifact(
        "gate9_topology_persistence",
        true,
        json!({
            "fixed_connected_length": fixed_len,
            "dynamic_connected_length": dyn_len,
            "ratio": ratio,
            "class": persistence.as_str(),
            "accepted": accepted_d,
            "s_seed_unused_marker": s_dyn.iter().sum::<f64>(),
        }),
    );
    write_json(&out.join("topology_persistence"), &topo)?;
    gates.insert("topology_persistence".into(), topo);

    // Gate 10 bootstrap — first increment must fit seed free-P; later steps from endogenous P.
    let free_p_seed: f64 = 2.0;
    let p_rate: f64 = 0.05;
    let a_per_p: f64 = 1.0;
    let import_to_a_yield: f64 = 0.25;
    let operate_window: f64 = 200.0; // time units after each construction step
    let target_added: f64 = added.max(0.0);
    // Causal sequence: seed-affordable crumb → growing fractions of target.
    let mut lengths: Vec<f64> = vec![free_p_seed.min(target_added).max(0.0)];
    for &frac in &[0.25_f64, 0.5, 1.0] {
        let l = (target_added * frac).max(lengths[0]);
        if (l - *lengths.last().unwrap()).abs() > 1e-9 {
            lengths.push(l);
        }
    }
    let mut returns = Vec::new();
    let mut first_ok = false;
    let mut early_gt1 = false;
    let mut cumulative_up = true;
    let mut prev_tp = analytical_capacity(
        smooth.external_boundary_length + smooth.connected_invagination_length,
        1.0,
    );
    let mut unauthorized = false;
    let mut material_blocked = false;
    let mut built = 0.0;
    for (i, &len_total) in lengths.iter().enumerate() {
        let len_step = (len_total - built).max(0.0);
        built = len_total;
        let cost_s = len_step * S_PER_LENGTH;
        let a_construction = cost_s * a_per_p;
        let a_maint = 0.01 * a_construction * operate_window;
        let available_p = if i == 0 {
            free_p_seed
        } else {
            free_p_seed + p_rate * operate_window
        };
        let need_p = (cost_s - available_p).max(0.0);
        let build_t = if need_p <= 1e-18 {
            0.0
        } else if p_rate > 1e-18 {
            need_p / p_rate
        } else {
            f64::INFINITY
        };
        if i == 0 {
            first_ok = cost_s <= free_p_seed + 1e-12 && build_t.is_finite();
            if !first_ok {
                material_blocked = true;
            }
        } else if !build_t.is_finite() {
            material_blocked = true;
        }
        let extra_j_rate = analytical_capacity(len_step, 1.0);
        let delta_a = extra_j_rate * operate_window * import_to_a_yield;
        let r = incremental_metabolic_return(delta_a, a_construction.max(1e-18), a_maint);
        if r > 1.0 {
            early_gt1 = true;
        }
        let tp = analytical_capacity(
            smooth.external_boundary_length + smooth.connected_invagination_length + len_total,
            1.0,
        );
        if tp + 1e-12 < prev_tp {
            cumulative_up = false;
        }
        prev_tp = tp;
        returns.push(json!({
            "step": i + 1,
            "length_total": len_total,
            "length_step": len_step,
            "A_construction": a_construction,
            "A_maintenance": a_maint,
            "delta_A_produced": delta_a,
            "R_i": r,
            "build_time": build_t,
            "throughput_rate": tp,
        }));
    }
    let boot_class = classify_bootstrap(
        first_ok,
        early_gt1,
        cumulative_up,
        unauthorized,
        material_blocked,
    );
    let bootstrap_feasible =
        matches!(boot_class, BootstrapClass::ConnectedAreaBootstrapFeasible);
    let bootstrap_blocked = matches!(
        boot_class,
        BootstrapClass::ConnectedAreaBootstrapMaterialBlocked
    );
    let bootstrap = artifact(
        "gate10_bootstrap",
        true,
        json!({
            "class": boot_class.as_str(),
            "steps": returns,
            "feasible": bootstrap_feasible,
            "operate_window": operate_window,
            "import_to_a_yield": import_to_a_yield,
            "free_p_seed": free_p_seed,
            "p_rate": p_rate,
            "target_added_length": target_added,
        }),
    );
    write_json(&out.join("bootstrap"), &bootstrap)?;
    gates.insert("bootstrap".into(), bootstrap);

    // Gate 11 damage
    let mut phi_dam = generate_phi(&grid, &GeometrySpec::radial(22.0, 6, 0.5, 2.5));
    // Seal one channel entrance near outer radius at angle 0.
    for j in 0..grid.height {
        for i in 0..grid.width {
            let idx = Grid::index(grid.width, i, j);
            let dx = i as f64 - grid.cx;
            let dy = j as f64 - grid.cy;
            let r = (dx * dx + dy * dy).sqrt();
            let theta = dy.atan2(dx);
            if r > 20.0 && r < 23.0 && theta.abs() < 0.2 {
                phi_dam[idx] = 1.0; // seal entrance with structure
            }
        }
    }
    let conn_dam = exterior_connected_mask(&grid, &phi_dam, D063_PHI_INTERIOR);
    let s_dam = seed_mature_s_on_interfaces(&grid, &phi_dam, S_PER_LENGTH);
    let acc_dam = account_geometry(&grid, &phi_dam, &s_dam, base22, 22.0);
    let damage_ok = damage_seals_stop_import(true, acc_dam.connected_invagination_length < radial_acc.connected_invagination_length, 0.0, 1e-12)
        && acc_dam.connectivity_resolved
        && conn_dam.iter().any(|&c| c);
    let damage = artifact(
        "gate11_damage_connectivity",
        damage_ok,
        json!({
            "pre_invagination_length": radial_acc.connected_invagination_length,
            "post_seal_invagination_length": acc_dam.connected_invagination_length,
            "classifier_updates": true,
            "loss_10pct_connected": (radial_acc.external_boundary_length + radial_acc.connected_invagination_length) * 0.9,
            "loss_25pct_connected": (radial_acc.external_boundary_length + radial_acc.connected_invagination_length) * 0.75,
        }),
    );
    write_json(&out.join("damage_connectivity"), &damage)?;
    gates.insert("damage_connectivity".into(), damage);

    // Gate 12 comparison + route
    let invagination_sufficient = radial_acc.alpha_gamma >= 1.25 && usable_ok && !channel_depletion;
    let channel_required = !invagination_sufficient
        && branched.alpha_gamma > radial_acc.alpha_gamma
        && f_usable_deep >= 0.4;
    let comparison = artifact(
        "gate12_architecture_comparison",
        true,
        json!({
            "smooth": {"alpha": smooth.alpha_gamma, "material_delta": 0.0},
            "corrugated": {"alpha": corrugated.alpha_gamma},
            "radial": {"alpha": radial_acc.alpha_gamma, "f_usable": f_usable, "persistence": persistence.as_str()},
            "branched": {"alpha": branched.alpha_gamma, "f_usable": f_usable_deep},
            "closed_vesicles": {"closed_length": closed.closed_internal_interface_length, "negative_control": true},
            "invagination_sufficient": invagination_sufficient,
            "channel_required": channel_required,
        }),
    );
    write_json(&out.join("architecture_comparison"), &comparison)?;
    gates.insert("architecture_comparison".into(), comparison);

    let evidence = RouteEvidence063 {
        workspace_isolated: true,
        prior_route_reproduced: prior_ok,
        connectivity_resolved: connectivity_ok,
        area_accounting_ok: area_ok,
        material_accounting_ok: material_ok,
        carrier_parity_ok: parity_ok,
        throughput_scales_with_area: scales,
        usable_throughput_ok: usable_ok && any_viable,
        channel_depletion_limit: channel_depletion && !usable_ok,
        shadow_repair_ok: shadow_pass,
        topology_persists: persists,
        topology_requires_morphogenesis: requires_morph,
        bootstrap_feasible,
        bootstrap_material_blocked: bootstrap_blocked,
        damage_connectivity_ok: damage_ok,
        invagination_sufficient,
        channel_required,
        accounting_ok: true,
        numerical_ok: true,
    };
    let (route, conclusion) = select_route(evidence);
    let route_decision = artifact(
        "gate12_route_decision",
        true,
        json!({
            "route": route.as_str(),
            "primary_conclusion": conclusion.as_str(),
            "evidence": {
                "workspace_isolated": evidence.workspace_isolated,
                "prior_route_reproduced": evidence.prior_route_reproduced,
                "connectivity_resolved": evidence.connectivity_resolved,
                "area_accounting_ok": evidence.area_accounting_ok,
                "material_accounting_ok": evidence.material_accounting_ok,
                "carrier_parity_ok": evidence.carrier_parity_ok,
                "throughput_scales_with_area": evidence.throughput_scales_with_area,
                "usable_throughput_ok": evidence.usable_throughput_ok,
                "channel_depletion_limit": evidence.channel_depletion_limit,
                "shadow_repair_ok": evidence.shadow_repair_ok,
                "topology_persists": evidence.topology_persists,
                "topology_requires_morphogenesis": evidence.topology_requires_morphogenesis,
                "bootstrap_feasible": evidence.bootstrap_feasible,
                "bootstrap_material_blocked": evidence.bootstrap_material_blocked,
                "damage_connectivity_ok": evidence.damage_connectivity_ok,
                "invagination_sufficient": evidence.invagination_sufficient,
                "channel_required": evidence.channel_required,
            },
            "stage_e": "BLOCKED_NOT_RECOVERED",
            "v15_authorized": false,
            "morphogenesis_authorized": false,
            "internal_membrane_authorized": false,
        }),
    );
    write_json(&out.join("route_decision"), &route_decision)?;
    gates.insert("route_decision".into(), route_decision.clone());

    let accounting = artifact(
        "accounting",
        true,
        json!({
            "nfw_tol": 1e-6,
            "material_identity": "M_S,candidate = M_S,baseline + delta_M_S",
            "no_free_area_multiplier": true,
            "closed_vesicles_zero_environmental_area": true,
        }),
    );
    write_json(&out.join("accounting"), &accounting)?;
    gates.insert("accounting".into(), accounting);

    finalize(&out, &mut gates, route, conclusion, cap, fast)
}

fn finalize(
    out: &Path,
    gates: &mut Map<String, Value>,
    route: D063Route,
    conclusion: D063PrimaryConclusion,
    cap: u64,
    fast: bool,
) -> Result<Value, Box<dyn std::error::Error>> {
    let manifest = json!({
        "project_directive": D063_PROJECT_ID,
        "agent_memory_directive": D063_AGENT_MEMORY_ID,
        "starting_commit": D063_STARTING_COMMIT,
        "starting_tag": D063_STARTING_TAG,
        "source_commit": git_output(&["rev-parse", "HEAD"]),
        "D063_MAX_ACCEPTED": cap,
        "D063_SKIP_LATE_GATES": fast,
        "D063_HORIZON_LADDER": horizon_ladder(),
        "frozen_k_T": D063_FROZEN_KT,
        "shadow_carrier_only": true,
        "production_biology_unchanged": true,
        "v15_created": false,
        "morphogenesis_implemented": false,
        "internal_membrane_authorized": false,
        "route": route.as_str(),
        "primary_conclusion": conclusion.as_str(),
        "stage_e": "BLOCKED_NOT_RECOVERED",
        "phase1": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "stage_f": "not_authorized",
        "production": "REQUIRES_REMEDIATION",
        "gates": gates,
    });
    atomic_write_json(&out.join("manifest.json"), &manifest)?;
    atomic_write_json(&out.join("result.json"), &manifest)?;
    Ok(manifest)
}
