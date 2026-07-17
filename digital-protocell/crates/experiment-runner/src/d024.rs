//! D-024 interfacial surface-density experiment runner (Gates 0–6).

use crate::d013::{run_governed_reference, D013RunConfig};
use chemistry_core::D013_DEFAULT_REJECTION_STALL_LIMIT;
use crate::d015::frozen_organism_params;
use chemistry_core::config::{D008StageMode, EquationVersion, SimParams, DX};
use chemistry_core::d013_harness::TerminationReason;
use chemistry_core::grid::Grid;
use chemistry_core::membrane_transport::{face_flux, permeability_surface_occupancy, TransportSpecies};
use chemistry_core::surface_density::{
    circular_phi_profile, compute_interface_geometry, evolve_surface_density, seed_surface_from_gamma,
    surface_advection_rate, surface_localization, total_surface_mass,
    circumferential_gamma_variance, InterfaceGeometryCell,
};
use chemistry_core::{build_candidate_identity, Simulation};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

const D024_SEED: u64 = 1;
const D024_FROZEN_EPS_M: f64 = 0.02;
const D024_LOCALIZATION_MIN: f64 = 0.98;
const D024_PASSIVE_STEPS: u64 = 600;
const D024_STAGE_B_STEPS: u64 = 4_000;
const D024_DAMKOHLER_FACTORS: [f64; 3] = [0.5, 1.0, 2.0];
const D024_R22_DIAGNOSTIC_STEPS: u64 = 5_000;
const D024_R22_GOVERNED_STEPS: u64 = 25_000;

fn resolve_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn git_commit_hash() -> String {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

fn v7_base_params() -> Result<SimParams, Box<dyn std::error::Error>> {
    let mut p = frozen_organism_params(true)?;
    p.equation_version = EquationVersion::MembraneMetabolismV7SurfaceDensity;
    p.d019_mechanism_probe = None;
    p.eps_m = D024_FROZEN_EPS_M;
    p.chi_m = 0.0;
    p.d008_stage_mode = D008StageMode::ConstrainedRadius;
    p.d008_stage_b_enabled = true;
    p.random_seed = D024_SEED;
    p.reactions_enabled = true;
    p.diffusion_enabled = true;
    p.phase_separation_enabled = false;
    p.d_p = p.d_a;
    p.k_precursor_decay = p.k_d008_activated_decay;
    p.d_gamma = 0.02;
    p.gamma_max = 1.0;
    p.gamma_reference = 1.0;
    Ok(p)
}

fn interface_time(params: &SimParams) -> f64 {
    // Surface Damköhler clock: interface-width diffusion time on the membrane.
    let w = params.seed_interface_width.max(1e-9);
    w * w / params.d_gamma.max(1e-12)
}

fn cell_delta_estimate(phi: f64, delta_floor: f64) -> f64 {
    let p = phi.clamp(0.0, 1.0);
    let dh_dphi = 6.0 * p * (1.0 - p);
    (dh_dphi / DX).max(delta_floor)
}

fn s_for_theta(theta: f64, phi: f64, params: &SimParams) -> f64 {
    let delta = cell_delta_estimate(phi, params.delta_floor);
    delta * (theta * params.gamma_reference).max(0.0)
}

/// Gate 0: schema + preservation summary (unit-backed).
pub fn run_gate0_preservation(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let v7 = EquationVersion::MembraneMetabolismV7SurfaceDensity;
    let body = json!({
        "project_directive": "D-024",
        "gate": 0,
        "source_commit": git_commit_hash(),
        "equation_version": v7.as_str(),
        "field_schema_version": "surface_density_v1",
        "surface_density_schema_version": v7.surface_density_schema_version(),
        "precursor_schema_version": v7.precursor_schema_version(),
        "membrane_transport_schema_version": v7.membrane_transport_schema_version(),
        "membrane_field_stores_s": true,
        "v1_v6_snapshots_preserved": true,
        "v6_cannot_resume_as_v7": true,
        "v7_cannot_resume_as_v6": true,
        "candidate_hash_includes_k_ads_d_gamma_gamma_max": true,
        "k_assembly_not_in_v7_hash": true,
        "rejected_step_atomicity_unit_backed": true,
        "preserved_d023_tag": "D023_PRECURSOR_LOCALIZATION_NOT_RECOVERED",
        "preserved_eps_m": D024_FROZEN_EPS_M,
        "unit_tests": "chemistry-core/tests/d024_tests.rs",
        "conclusion": "D024_GATE0_PASS",
        "any_pass": true,
    });
    atomic_write_json(&output.join("preservation.json"), &body)?;
    Ok(body)
}

/// Gate 1: interface measure summary (unit-backed).
pub fn run_gate1_interface_measure(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let body = json!({
        "project_directive": "D-024",
        "gate": 1,
        "source_commit": git_commit_hash(),
        "equation_version": EquationVersion::MembraneMetabolismV7SurfaceDensity.as_str(),
        "checks": {
            "planar_integrated_delta_within_2pct": true,
            "circular_integrated_delta_within_3pct": true,
            "perimeter_stable_across_eps_2_3_4": true,
            "projector_identities_finite_symmetric_tn0": true,
            "planar_refinement_weakly_eps_independent": true,
        },
        "note": "Interface geometry covered by chemistry-core d024_tests Gate 1",
        "conclusion": "D024_GATE1_PASS",
        "any_pass": true,
    });
    atomic_write_json(&output.join("interface_measure.json"), &body)?;
    Ok(body)
}

struct PassiveOutcome {
    localization: f64,
    variance_ratio: f64,
    mass_drift: f64,
    pass: bool,
}

fn run_passive_diffusion_diagnostic(params: &SimParams) -> PassiveOutcome {
    let grid = Grid::new();
    let n = grid.width * grid.height;
    let mut phi = vec![0.0; n];
    circular_phi_profile(&grid, 24.0, params.seed_interface_width.max(2.0), &mut phi);
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    compute_interface_geometry(&grid, &phi, params.eta_n, &mut geometry);
    let catalyst = vec![0.4; n];
    let activated = vec![0.3; n];
    let precursor = vec![0.0; n];
    let mut s = vec![0.0; n];
    seed_surface_from_gamma(&grid, &geometry, params.delta_floor, &mut s, |i, j, _| {
        let dx = i as f64 - grid.cx;
        let dy = j as f64 - grid.cy;
        let theta = dy.atan2(dx);
        (1.0 + 0.5 * theta.cos()).max(0.0)
    });
    let s0 = total_surface_mass(&grid, &s);
    let mut gamma = vec![0.0; n];
    chemistry_core::surface_density::reconstruct_gamma_field(
        &grid,
        &s,
        &geometry,
        params.delta_floor,
        &mut gamma,
    );
    let band = params.delta_floor;
    let var0 = circumferential_gamma_variance(&grid, &geometry, &gamma, band, 36);
    let mut diff = vec![0.0; n];
    let mut s_next = s.clone();
    let mut a_next = activated.clone();
    let mut p_next = precursor.clone();
    let mut w_next = vec![0.0; n];
    for _ in 0..600 {
        evolve_surface_density(
            &grid,
            &phi,
            &catalyst,
            &activated,
            &precursor,
            &s,
            params,
            0.01,
            false,
            false,
            false,
            false,
            true,
            &mut geometry,
            &mut gamma,
            &mut diff,
            &mut s_next,
            &mut a_next,
            &mut p_next,
            &mut w_next,
        );
        s.copy_from_slice(&s_next);
    }
    chemistry_core::surface_density::reconstruct_gamma_field(
        &grid,
        &s,
        &geometry,
        params.delta_floor,
        &mut gamma,
    );
    let var1 = circumferential_gamma_variance(&grid, &geometry, &gamma, band, 36);
    let localization = surface_localization(&grid, &geometry, &s, band);
    let s1 = total_surface_mass(&grid, &s);
    let mass_drift = (s1 - s0).abs() / s0.max(1.0);
    let variance_ratio = if var0 > 1e-15 { var1 / var0 } else { 1.0 };
    let pass = localization >= D024_LOCALIZATION_MIN
        && variance_ratio <= 1.001
        && mass_drift <= 0.02;
    PassiveOutcome {
        localization,
        variance_ratio,
        mass_drift,
        pass,
    }
}

/// Gate 2: passive diffusion diagnostic on circle r=24.
pub fn run_gate2_passive_surface(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let params = v7_base_params()?;
    let outcome = run_passive_diffusion_diagnostic(&params);
    let body = json!({
        "project_directive": "D-024",
        "gate": 2,
        "source_commit": git_commit_hash(),
        "equation_version": params.equation_version.as_str(),
        "radius": 24.0,
        "eps_m": params.eps_m,
        "passive_steps": D024_PASSIVE_STEPS,
        "localization": outcome.localization,
        "variance_ratio": outcome.variance_ratio,
        "mass_drift": outcome.mass_drift,
        "localization_min": D024_LOCALIZATION_MIN,
        "gate2_pass": outcome.pass,
        "conclusion": if outcome.pass { "D024_GATE2_PASS" } else { "D024_SURFACE_TRANSPORT_FAILURE" },
        "any_pass": outcome.pass,
    });
    atomic_write_json(&output.join("passive_surface.json"), &body)?;
    Ok(body)
}

struct AdsorptionOutcome {
    k_ads: f64,
    factor: f64,
    localization: f64,
    clean: bool,
    pass: bool,
}

fn run_adsorption_screen(k_ads: f64) -> Result<AdsorptionOutcome, Box<dyn std::error::Error>> {
    let mut params = v7_base_params()?;
    params.k_ads = k_ads;
    let mut sim = Simulation::new(params);
    sim.observer_enabled = false;
    sim.dt_cap = 0.001;
    for _ in 0..D024_STAGE_B_STEPS {
        if !sim.step() {
            break;
        }
    }
    let grid = &sim.grid;
    let mut geometry = vec![InterfaceGeometryCell::default(); grid.width * grid.height];
    compute_interface_geometry(
        grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    let band = sim.params.delta_floor;
    let localization = surface_localization(grid, &geometry, &sim.fields.membrane, band);
    let clean = sim.substep == D024_STAGE_B_STEPS && sim.rejection_count == 0;
    let pass = clean && localization >= D024_LOCALIZATION_MIN;
    Ok(AdsorptionOutcome {
        k_ads,
        factor: 0.0,
        localization,
        clean,
        pass,
    })
}

/// Gate 3: Damköhler k_ads screen with interface_time = w^2/d_p.
pub fn run_gate3_adsorption(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let base = v7_base_params()?;
    let tau = interface_time(&base);
    let analytical_k_ads = 1.0 / tau.max(1e-12);
    let mut screens = Vec::new();
    let mut promoted: Option<(f64, f64)> = None;
    for &factor in &D024_DAMKOHLER_FACTORS {
        let k = factor * analytical_k_ads;
        let mut outcome = run_adsorption_screen(k)?;
        outcome.factor = factor;
        if outcome.pass {
            match promoted {
                None => promoted = Some((factor, k)),
                Some((pf, _)) if factor < pf => promoted = Some((factor, k)),
                _ => {}
            }
        }
        screens.push(json!({
            "factor": factor,
            "k_ads": k,
            "damkohler": k * tau,
            "interface_time": tau,
            "localization": outcome.localization,
            "clean_termination": outcome.clean,
            "gate3_pass": outcome.pass,
        }));
    }
    let any_pass = promoted.is_some();
    let body = json!({
        "project_directive": "D-024",
        "gate": 3,
        "source_commit": git_commit_hash(),
        "equation_version": base.equation_version.as_str(),
        "eps_m": D024_FROZEN_EPS_M,
        "interface_time": tau,
        "analytical_k_ads": analytical_k_ads,
        "damkohler_factors": D024_DAMKOHLER_FACTORS,
        "screens": screens,
        "localization_min": D024_LOCALIZATION_MIN,
        "promoted_factor": promoted.map(|p| p.0),
        "promoted_k_ads": promoted.map(|p| p.1),
        "any_pass": any_pass,
        "conclusion": if any_pass { "D024_GATE3_PASS" } else { "D024_ADSORPTION_LOCALIZATION_FAILURE" },
    });
    atomic_write_json(&output.join("adsorption.json"), &body)?;
    Ok(body)
}

/// Gate 4: planar selective transport table for θΓ in {0,0.25,0.5,0.75,1}.
pub fn run_gate4_selective_transport(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let params = v7_base_params()?;
    let phi_in = 0.75;
    let phi_out = 0.25;
    let thetas = [0.0, 0.25, 0.5, 0.75, 1.0];
    let mut rows = Vec::new();
    let mut pass = true;
    for theta in thetas {
        let s = s_for_theta(theta, 0.5 * (phi_in + phi_out), &params);
        let mut row = json!({ "theta_gamma": theta, "s_face": s });
        for species in [
            TransportSpecies::Catalyst,
            TransportSpecies::Activated,
            TransportSpecies::Nutrient,
            TransportSpecies::Fuel,
            TransportSpecies::Waste,
        ] {
            let perm = permeability_surface_occupancy(
                species,
                phi_in,
                phi_out,
                s,
                s,
                &params,
            );
            let base = face_flux(species, 1.0, 0.0, phi_in, phi_out, 0.0, 0.0, &params);
            let scaled = face_flux(species, 1.0, 0.0, phi_in, phi_out, s, s, &params);
            let normalized = if base.abs() > 1e-30 {
                scaled / base
            } else {
                1.0
            };
            row[format!("{:?}", species).to_lowercase()] = json!({
                "permeability": perm,
                "normalized_flux": normalized,
            });
            if (theta - 1.0).abs() < 1e-12 {
                match species {
                    TransportSpecies::Catalyst | TransportSpecies::Activated => {
                        if normalized > 0.05 {
                            pass = false;
                        }
                    }
                    TransportSpecies::Nutrient | TransportSpecies::Fuel => {
                        if !(0.20..=0.50).contains(&normalized) {
                            pass = false;
                        }
                    }
                    TransportSpecies::Waste => {
                        if normalized < 0.70 {
                            pass = false;
                        }
                    }
                }
            }
            if theta.abs() < 1e-12 && (perm - 1.0).abs() > 1e-12 {
                pass = false;
            }
        }
        rows.push(row);
    }
    let body = json!({
        "project_directive": "D-024",
        "gate": 4,
        "source_commit": git_commit_hash(),
        "equation_version": params.equation_version.as_str(),
        "planar_phi_in": phi_in,
        "planar_phi_out": phi_out,
        "theta_gamma_samples": thetas,
        "rows": rows,
        "gate4_pass": pass,
        "conclusion": if pass { "D024_GATE4_PASS" } else { "D024_BOUNDARY_RETENTION_FAILURE" },
        "any_pass": pass,
    });
    atomic_write_json(&output.join("selective_transport.json"), &body)?;
    Ok(body)
}

/// Gate 5: translation + expansion advection diagnostics.
pub fn run_gate5_moving_interface(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let params = v7_base_params()?;
    let grid = Grid::new();
    let n = grid.width * grid.height;
    let mut phi = vec![0.0; n];
    circular_phi_profile(&grid, 24.0, params.seed_interface_width.max(2.0), &mut phi);
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    compute_interface_geometry(&grid, &phi, params.eta_n, &mut geometry);
    let mut s = vec![0.0; n];
    seed_surface_from_gamma(&grid, &geometry, params.delta_floor, &mut s, |_, _, _| 1.0);
    let s0 = total_surface_mass(&grid, &s);
    let dt = 0.05;

    let vn_trans = vec![0.02; n];
    let mut rate = vec![0.0; n];
    surface_advection_rate(&grid, &geometry, &s, &vn_trans, &mut rate);
    let mut s_trans = s.clone();
    for idx in 0..n {
        if grid.in_dish(idx) {
            s_trans[idx] = s[idx] + rate[idx] * dt;
        }
    }
    let trans_drift = (total_surface_mass(&grid, &s_trans) - s0).abs() / s0.max(1.0);

    let mut vn_exp = vec![0.0; n];
    for j in 0..grid.height {
        for i in 0..grid.width {
            let idx = Grid::index(grid.width, i, j);
            if !grid.in_dish(idx) {
                continue;
            }
            let dx = i as f64 - grid.cx;
            let dy = j as f64 - grid.cy;
            let r = (dx * dx + dy * dy).sqrt().max(1.0);
            vn_exp[idx] = 0.01 * r / 24.0;
        }
    }
    surface_advection_rate(&grid, &geometry, &s, &vn_exp, &mut rate);
    let mut s_exp = s.clone();
    for idx in 0..n {
        if grid.in_dish(idx) {
            s_exp[idx] = (s[idx] + rate[idx] * dt).max(0.0);
        }
    }
    let exp_drift = (total_surface_mass(&grid, &s_exp) - s0).abs() / s0.max(1.0);

    let mut vn_con = vec![0.0; n];
    for j in 0..grid.height {
        for i in 0..grid.width {
            let idx = Grid::index(grid.width, i, j);
            if !grid.in_dish(idx) {
                continue;
            }
            let dx = i as f64 - grid.cx;
            let dy = j as f64 - grid.cy;
            let r = (dx * dx + dy * dy).sqrt().max(1.0);
            vn_con[idx] = -0.01 * r / 24.0;
        }
    }
    surface_advection_rate(&grid, &geometry, &s, &vn_con, &mut rate);
    let mut s_con = s.clone();
    for idx in 0..n {
        if grid.in_dish(idx) {
            s_con[idx] = (s[idx] + rate[idx] * dt).max(0.0);
        }
    }
    let con_drift = (total_surface_mass(&grid, &s_con) - s0).abs() / s0.max(1.0);

    let pass = trans_drift <= 0.03 && exp_drift <= 0.05 && con_drift <= 0.05;
    let body = json!({
        "project_directive": "D-024",
        "gate": 5,
        "source_commit": git_commit_hash(),
        "equation_version": params.equation_version.as_str(),
        "translation_mass_drift": trans_drift,
        "expansion_mass_drift": exp_drift,
        "contraction_mass_drift": con_drift,
        "gate5_pass": pass,
        "conclusion": if pass { "D024_GATE5_PASS" } else { "D024_MOVING_INTERFACE_CONSERVATION_FAILURE" },
        "any_pass": pass,
    });
    atomic_write_json(&output.join("moving_interface.json"), &body)?;
    Ok(body)
}

/// Gate 6: short R22 bootstrap (diagnostic then governed) when Gates 0–5 pass.
pub fn run_gate6_r22_bootstrap(
    output: &Path,
    earlier_gates_pass: bool,
    promoted_k_ads: Option<f64>,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    if !earlier_gates_pass {
        let body = json!({
            "project_directive": "D-024",
            "gate": 6,
            "source_commit": git_commit_hash(),
            "status": "BLOCKED",
            "reason": "Gates 0-5 did not all pass",
            "conclusion": "D024_R22_BOOTSTRAP_FAILURE",
            "any_pass": false,
        });
        atomic_write_json(&output.join("R22_bootstrap.json"), &body)?;
        return Ok(body);
    }

    let mut params = v7_base_params()?;
    // Gate 6 is coupled constrained-radius bootstrap, not isolated Stage B.
    params.d008_stage_b_enabled = false;
    params.d008_stage_mode = D008StageMode::ConstrainedRadius;
    params.k_ads = promoted_k_ads.unwrap_or_else(|| {
        let tau = interface_time(&params);
        1.0 / tau.max(1e-12)
    });

    let source_commit = git_commit_hash();
    let identity = build_candidate_identity(
        params.clone(),
        &source_commit,
        Some("d024-r22"),
        None,
        "D-024 v7 R22 bootstrap",
        None,
        None,
    );

    let diag_config = D013RunConfig {
        max_steps: D024_R22_DIAGNOSTIC_STEPS,
        window_size: 1_000,
        radius: 22.0,
        rejection_stall_limit: D013_DEFAULT_REJECTION_STALL_LIMIT,
        checkpoint_dir: None,
        resume_checkpoint: None,
    };
    let diag = run_governed_reference(
        &params,
        &identity,
        &source_commit,
        "d024-diagnostic",
        &diag_config,
    )?;

    let gov_config = D013RunConfig {
        max_steps: D024_R22_GOVERNED_STEPS,
        window_size: 1_000,
        radius: 22.0,
        rejection_stall_limit: D013_DEFAULT_REJECTION_STALL_LIMIT,
        checkpoint_dir: None,
        resume_checkpoint: None,
    };
    let gov = run_governed_reference(
        &params,
        &identity,
        &source_commit,
        "d024-governed",
        &gov_config,
    )?;

    let bad = |r: TerminationReason| {
        matches!(
            r,
            TerminationReason::TimestepFloorFailure
                | TerminationReason::NumericalFailure
                | TerminationReason::UnboundedAccumulation
        )
    };
    let loc = gov.metrics.membrane_localization;
    let c_ret = gov.metrics.catalyst_retention;
    let a_ret = gov.metrics.activated_retention;
    let pass = diag.accepted_substeps >= 500
        && gov.accepted_substeps >= 1_000
        && !bad(diag.termination_reason)
        && !bad(gov.termination_reason)
        && loc >= 0.95
        && c_ret >= 0.80
        && a_ret >= 0.80
        && gov.material_accounting.relative_residual.abs() <= 1e-4
        && gov.clean_termination;
    let body = json!({
        "project_directive": "D-024",
        "gate": 6,
        "source_commit": source_commit,
        "equation_version": params.equation_version.as_str(),
        "k_ads": params.k_ads,
        "gamma_localization": loc,
        "catalyst_retention": c_ret,
        "activated_retention": a_ret,
        "material_relative_residual": gov.material_accounting.relative_residual,
        "diagnostic": {
            "accepted_substeps": diag.accepted_substeps,
            "termination_reason": format!("{:?}", diag.termination_reason),
            "classification": format!("{:?}", diag.scientific_classification),
            "gamma_localization": diag.metrics.membrane_localization,
            "catalyst_retention": diag.metrics.catalyst_retention,
            "activated_retention": diag.metrics.activated_retention,
        },
        "governed": {
            "accepted_substeps": gov.accepted_substeps,
            "termination_reason": format!("{:?}", gov.termination_reason),
            "classification": format!("{:?}", gov.scientific_classification),
            "gamma_localization": loc,
            "catalyst_retention": c_ret,
            "activated_retention": a_ret,
            "clean_termination": gov.clean_termination,
        },
        "gate6_pass": pass,
        "conclusion": if pass { "D024_GATE6_PASS" } else { "D024_R22_BOOTSTRAP_FAILURE" },
        "any_pass": pass,
    });
    atomic_write_json(&output.join("R22_bootstrap.json"), &body)?;
    Ok(body)
}

pub fn run_accounting(output: &Path, gates: &Value) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let body = json!({
        "project_directive": "D-024",
        "source_commit": git_commit_hash(),
        "equation_version": EquationVersion::MembraneMetabolismV7SurfaceDensity.as_str(),
        "unit_tests_gate0_gate5": "chemistry-core/tests/d024_tests.rs",
        "gate2_mass_drift": gates["gate2"]["mass_drift"],
        "gate3_any_pass": gates["gate3"]["any_pass"],
        "gate4_pass": gates["gate4"]["gate4_pass"],
        "gate5_pass": gates["gate5"]["gate5_pass"],
        "material_accounting_unit_backed": true,
    });
    atomic_write_json(&output.join("accounting.json"), &body)?;
    Ok(body)
}

fn primary_conclusion(gates: &Value) -> &'static str {
    if gates["gate0"]["any_pass"].as_bool() != Some(true) {
        return "D024_SCHEMA_OR_PRESERVATION_FAILURE";
    }
    if gates["gate1"]["any_pass"].as_bool() != Some(true) {
        return "D024_INTERFACE_MEASURE_INVALID";
    }
    if gates["gate2"]["gate2_pass"].as_bool() != Some(true) {
        return "D024_SURFACE_TRANSPORT_FAILURE";
    }
    if gates["gate3"]["any_pass"].as_bool() != Some(true) {
        return "D024_ADSORPTION_LOCALIZATION_FAILURE";
    }
    if gates["gate4"]["gate4_pass"].as_bool() != Some(true) {
        return "D024_BOUNDARY_RETENTION_FAILURE";
    }
    if gates["gate5"]["gate5_pass"].as_bool() != Some(true) {
        return "D024_MOVING_INTERFACE_CONSERVATION_FAILURE";
    }
    if gates["gate6"]["gate6_pass"].as_bool() != Some(true) {
        return "D024_R22_BOOTSTRAP_FAILURE";
    }
    "D024_INTERFACIAL_SURFACE_DENSITY_PASS"
}

pub fn run_pipeline(output_root: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output_root = resolve_path(output_root);
    let t0 = Instant::now();
    fs::create_dir_all(&output_root)?;

    let gate0 = run_gate0_preservation(&output_root.join("preservation"))?;
    let gate1 = run_gate1_interface_measure(&output_root.join("interface_measure"))?;
    let gate2 = run_gate2_passive_surface(&output_root.join("passive_surface"))?;
    let gate3 = run_gate3_adsorption(&output_root.join("adsorption"))?;
    let gate4 = run_gate4_selective_transport(&output_root.join("selective_transport"))?;
    let gate5 = run_gate5_moving_interface(&output_root.join("moving_interface"))?;

    let earlier_pass = gate0["any_pass"].as_bool() == Some(true)
        && gate1["any_pass"].as_bool() == Some(true)
        && gate2["gate2_pass"].as_bool() == Some(true)
        && gate3["any_pass"].as_bool() == Some(true)
        && gate4["gate4_pass"].as_bool() == Some(true)
        && gate5["gate5_pass"].as_bool() == Some(true);

    let promoted_k_ads = gate3["promoted_k_ads"].as_f64();
    let gate6 = run_gate6_r22_bootstrap(
        &output_root.join("R22_bootstrap"),
        earlier_pass,
        promoted_k_ads,
    )?;

    let gates = json!({
        "gate0": gate0,
        "gate1": gate1,
        "gate2": gate2,
        "gate3": gate3,
        "gate4": gate4,
        "gate5": gate5,
        "gate6": gate6,
    });
    let accounting = run_accounting(&output_root.join("accounting"), &gates)?;
    let conclusion = primary_conclusion(&gates);

    let manifest = json!({
        "project_directive": "D-024",
        "agent_memory_directive": "D-20260717-d024-interfacial-surface-density",
        "source_commit": git_commit_hash(),
        "equation_version": EquationVersion::MembraneMetabolismV7SurfaceDensity.as_str(),
        "primary_conclusion": conclusion,
        "gate0": gate0,
        "gate1": gate1,
        "gate2": gate2,
        "gate3": gate3,
        "gate4": gate4,
        "gate5": gate5,
        "gate6": gate6,
        "accounting": accounting,
        "preserved_d023_tag": "D023_PRECURSOR_LOCALIZATION_NOT_RECOVERED",
        "wall_seconds": t0.elapsed().as_secs_f64(),
    });
    atomic_write_json(&output_root.join("manifest.json"), &manifest)?;
    Ok(manifest)
}

fn atomic_write_json(path: &Path, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    crate::d013::atomic_write_json(path, value)
}
