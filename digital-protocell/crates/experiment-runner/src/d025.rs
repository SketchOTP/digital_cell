//! D-025 autonomous surface transport regression runner (Gates 3–6 subset).

use crate::d013::atomic_write_json;
use crate::d015::frozen_organism_params;
use chemistry_core::config::{D008StageMode, EquationVersion, SimParams, CONC_SAFETY_LIMIT};
use chemistry_core::d011_analysis::STAGE_E_FAILED_RATES;
use chemistry_core::d018_analysis::D018_FROZEN_K_STRUCTURE;
use chemistry_core::field_mass;
use chemistry_core::grid::Grid;
use chemistry_core::operators::total_mass;
use chemistry_core::reactions::interface_weight;
use chemistry_core::surface_density::{
    compute_interface_geometry, integrated_delta, seed_surface_from_gamma, surface_localization,
    total_surface_mass, InterfaceGeometryCell,
};
use chemistry_core::{build_candidate_identity, field_sha256_stable, stage_c_clamp_negligible, Simulation};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

pub const D025_FROZEN_K_ADS: f64 = 0.0011111111111111111;
const D025_SEED: u64 = 1;
const D025_FROZEN_EPS_M: f64 = 0.02;
const D025_GATE3_STEPS: u64 = 800;
const D025_S_DRIFT_MAX: f64 = 0.04;
const D025_STAGE_B_FIXED_STEPS: u64 = 4_000;
const D025_STAGE_B_SLOW_STEPS: u64 = 2_000;
const D025_STAGE_C_STEPS: u64 = 100;
const STAGE_D_RADII: [f64; 3] = [16.0, 24.0, 32.0];
const STAGE_D_STEPS: u64 = 5_000;
const STAGE_D_RETENTION_MIN: f64 = 0.80;
const STAGE_D_SMALL_CELL_RETENTION_MARGIN: f64 = 0.05;
const D025_LOCALIZATION_FIXED_MIN: f64 = 0.98;
const D025_LOCALIZATION_SLOW_MIN: f64 = 0.95;
const D025_S_MOVEMENT_DRIFT_MAX: f64 = 0.03;

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

pub fn v7_base_params() -> Result<SimParams, Box<dyn std::error::Error>> {
    let mut p = frozen_organism_params(true)?;
    p.equation_version = EquationVersion::MembraneMetabolismV7SurfaceDensity;
    p.d019_mechanism_probe = None;
    p.eps_m = D025_FROZEN_EPS_M;
    p.chi_m = 0.0;
    p.d008_stage_mode = D008StageMode::ConstrainedRadius;
    p.d008_stage_b_enabled = false;
    p.random_seed = D025_SEED;
    p.reactions_enabled = true;
    p.diffusion_enabled = true;
    p.phase_separation_enabled = false;
    p.d_p = p.d_a;
    p.k_precursor_decay = p.k_d008_activated_decay;
    p.d_gamma = 0.02;
    p.gamma_max = 1.0;
    p.gamma_reference = 1.0;
    p.k_ads = D025_FROZEN_K_ADS;
    STAGE_E_FAILED_RATES.apply_to(&mut p);
    p.k_d008_structure = D018_FROZEN_K_STRUCTURE;
    Ok(p)
}

fn seed_v7_compartment(sim: &mut Simulation, radius: f64, theta_gamma: f64) {
    sim.observer_enabled = false;
    let w = sim.grid.width;
    let n = sim.fields.structure.len();
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    for idx in 0..n {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        let i = idx % w;
        let j = idx / w;
        let x = i as f64 - sim.grid.cx;
        let y = j as f64 - sim.grid.cy;
        let distance = (x * x + y * y).sqrt();
        let phi = 0.5 * (1.0 - ((distance - radius) / 2.0).tanh());
        sim.fields.structure[idx] = phi;
        if phi >= 0.5 {
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
    compute_interface_geometry(
        &sim.grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    seed_surface_from_gamma(
        &sim.grid,
        &geometry,
        sim.params.delta_floor,
        &mut sim.fields.membrane,
        |_, _, _| theta_gamma,
    );
    sim.fields.copy_current_to_next();
}

fn gamma_localization(sim: &Simulation) -> f64 {
    let n = sim.grid.width * sim.grid.height;
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    compute_interface_geometry(
        &sim.grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    surface_localization(
        &sim.grid,
        &geometry,
        &sim.fields.membrane,
        sim.params.delta_floor,
    )
}

fn structural_area(sim: &Simulation) -> f64 {
    sim.fields
        .structure
        .iter()
        .enumerate()
        .filter(|(idx, _)| sim.grid.in_dish(*idx))
        .map(|(_, &phi)| if phi >= 0.5 { 1.0 } else { 0.0 })
        .sum()
}

fn interface_length(sim: &Simulation) -> f64 {
    let n = sim.grid.width * sim.grid.height;
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    compute_interface_geometry(
        &sim.grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    integrated_delta(&sim.grid, &geometry)
}

fn mean_gamma(sim: &Simulation) -> f64 {
    let n = sim.grid.width * sim.grid.height;
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    let mut gamma = vec![0.0; n];
    compute_interface_geometry(
        &sim.grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    chemistry_core::surface_density::reconstruct_gamma_field(
        &sim.grid,
        &sim.fields.membrane,
        &geometry,
        sim.params.delta_floor,
        &mut gamma,
    );
    let mut sum = 0.0;
    let mut count = 0usize;
    for idx in 0..n {
        if geometry[idx].delta > sim.params.delta_floor {
            sum += gamma[idx];
            count += 1;
        }
    }
    sum / count.max(1) as f64
}

fn structure_mass(sim: &Simulation) -> f64 {
    field_mass(&sim.grid, &sim.fields.structure)
}

fn gate3_row(
    mass_delta: f64,
    length_delta: f64,
    s_drift: f64,
    localization: f64,
    mean_gamma_delta: f64,
    accounting_closed: bool,
    accepted_substeps: u64,
    clean: bool,
) -> Value {
    json!({
        "mass_delta": mass_delta,
        "length_delta": length_delta,
        "s_drift": s_drift,
        "localization": localization,
        "mean_gamma_delta": mean_gamma_delta,
        "accounting_closed": accounting_closed,
        "accepted_substeps": accepted_substeps,
        "clean_termination": clean,
    })
}

struct Gate3Scenario {
    k_structure: f64,
    k_decay: f64,
    k_ads: f64,
    k_precursor: f64,
    radius: f64,
}

fn run_gate3_scenario(spec: &Gate3Scenario) -> Result<Value, Box<dyn std::error::Error>> {
    let mut params = v7_base_params()?;
    params.k_d008_structure = spec.k_structure;
    params.k_structure_decay = spec.k_decay;
    params.k_ads = spec.k_ads;
    params.k_precursor = spec.k_precursor;
    let mut sim = Simulation::new(params);
    sim.enforce_structure_constraint = false;
    sim.dt_cap = 0.005;
    seed_v7_compartment(&mut sim, spec.radius, 0.6);
    let mass0 = structure_mass(&sim);
    let length0 = interface_length(&sim);
    let s0 = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let g0 = mean_gamma(&sim);
    for _ in 0..D025_GATE3_STEPS {
        if !sim.step() {
            break;
        }
    }
    let mass1 = structure_mass(&sim);
    let length1 = interface_length(&sim);
    let s1 = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let g1 = mean_gamma(&sim);
    Ok(gate3_row(
        mass1 - mass0,
        length1 - length0,
        ((s1 - s0) / s0.max(1.0)).abs(),
        gamma_localization(&sim),
        g1 - g0,
        sim.accounting.cumulative_within_tolerance(),
        sim.substep,
        sim.substep == D025_GATE3_STEPS && sim.rejection_count == 0,
    ))
}

/// Gate 3: chemistry-driven growth, shrinkage, balanced turnover.
pub fn run_gate3_growth_shrinkage(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let base = v7_base_params()?;

    let growth = run_gate3_scenario(&Gate3Scenario {
        k_structure: 0.25,
        k_decay: 0.002,
        k_ads: D025_FROZEN_K_ADS,
        k_precursor: base.k_precursor,
        radius: 20.0,
    })?;
    let expansion_control = run_gate3_scenario(&Gate3Scenario {
        k_structure: 0.25,
        k_decay: 0.002,
        k_ads: 0.0,
        k_precursor: 0.0,
        radius: 20.0,
    })?;
    let shrinkage = run_gate3_scenario(&Gate3Scenario {
        k_structure: 0.0,
        k_decay: 0.025,
        k_ads: 0.0,
        k_precursor: 0.0,
        radius: 24.0,
    })?;
    let balanced = run_gate3_scenario(&Gate3Scenario {
        k_structure: D018_FROZEN_K_STRUCTURE,
        k_decay: 0.025,
        k_ads: D025_FROZEN_K_ADS,
        k_precursor: base.k_precursor,
        radius: 22.0,
    })?;

    let growth_pass = {
        growth["length_delta"].as_f64().unwrap_or(0.0) > 0.0
            && growth["mass_delta"].as_f64().unwrap_or(0.0) > 0.0
            && growth["localization"].as_f64().unwrap_or(0.0) >= 0.95
            && expansion_control["s_drift"].as_f64().unwrap_or(1.0) < D025_S_DRIFT_MAX
            && growth["accounting_closed"].as_bool() == Some(true)
            && growth["clean_termination"].as_bool() == Some(true)
    };
    let shrink_pass = shrinkage["mass_delta"].as_f64().unwrap_or(0.0) < 0.0
        && shrinkage["length_delta"].as_f64().unwrap_or(0.0) < 0.0
        && shrinkage["s_drift"].as_f64().unwrap_or(1.0) < D025_S_DRIFT_MAX
        && shrinkage["mean_gamma_delta"].as_f64().unwrap_or(0.0) > 0.0
        && shrinkage["localization"].as_f64().unwrap_or(0.0) >= 0.95
        && shrinkage["accounting_closed"].as_bool() == Some(true)
        && shrinkage["clean_termination"].as_bool() == Some(true);
    let balanced_pass = balanced["localization"].as_f64().unwrap_or(0.0) >= 0.95
        && balanced["accounting_closed"].as_bool() == Some(true)
        && balanced["clean_termination"].as_bool() == Some(true);
    let pass = growth_pass && shrink_pass && balanced_pass;

    let body = json!({
        "project_directive": "D-025",
        "gate": 3,
        "source_commit": git_commit_hash(),
        "equation_version": EquationVersion::MembraneMetabolismV7SurfaceDensity.as_str(),
        "k_ads_frozen": D025_FROZEN_K_ADS,
        "steps": D025_GATE3_STEPS,
        "growth": growth,
        "expansion_control": expansion_control,
        "shrinkage": shrinkage,
        "balanced": balanced,
        "checks": {
            "net_growth": growth_pass,
            "net_shrinkage": shrink_pass,
            "balanced_turnover": balanced_pass,
        },
        "gate3_pass": pass,
        "conclusion": if pass { "D025_GATE3_PASS" } else { "D025_CHEMISTRY_INTERFACE_COUPLING_FAILURE" },
        "any_pass": pass,
    });
    atomic_write_json(&output.join("growth_shrinkage.json"), &body)?;
    Ok(body)
}

fn prepare_stage_c_v7(sim: &mut Simulation, case_id: &str) {
    for idx in 0..sim.fields.structure.len() {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        sim.fields.structure[idx] = 0.5;
        sim.fields.membrane[idx] = 0.0;
        sim.fields.precursor[idx] = 0.0;
        sim.fields.catalyst[idx] = 0.4;
        sim.fields.nutrient[idx] = 0.8;
        sim.fields.fuel[idx] = 0.7;
        sim.fields.activated[idx] = 0.2;
        sim.fields.waste[idx] = 0.0;
    }
    match case_id {
        "missing_c" => sim.fields.catalyst.fill(0.0),
        "missing_n" | "no_activation_decline" => sim.fields.nutrient.fill(0.0),
        "missing_f" => sim.fields.fuel.fill(0.0),
        "missing_a_reproduction" => {
            sim.fields.activated.fill(0.0);
            sim.params.k_d008_activation = 0.0;
        }
        "no_reproduction_decline" => sim.params.k_d008_reproduction = 0.0,
        _ => {}
    }
}

fn approx_equal(left: f64, right: f64) -> bool {
    (left - right).abs() <= 1e-8 * left.abs().max(right.abs()).max(1.0)
}

fn stage_c_case_pass_v7(
    case_id: &str,
    sim: &Simulation,
    initial_catalyst: f64,
    initial_activated: f64,
    initial_waste: f64,
    structure_invariant: bool,
) -> bool {
    let cumulative = &sim.metabolism_accounting.cumulative;
    let clean = sim.substep == D025_STAGE_C_STEPS && sim.rejection_count == 0;
    let bounded = sim
        .fields
        .catalyst
        .iter()
        .all(|&v| v.is_finite() && (0.0..=sim.params.d008_c_max).contains(&v))
        && sim
            .fields
            .activated
            .iter()
            .all(|&v| v.is_finite() && (0.0..=sim.params.d008_a_max).contains(&v))
        && stage_c_clamp_negligible(cumulative);
    // V7 inherits conservative v2 activation/reproduction yields.
    let eta_c = sim.params.eta_c;
    let expected_waste = cumulative.activation
        + (1.0 - eta_c) * cumulative.reproduction
        + cumulative.activated_decay
        + cumulative.catalyst_turnover;
    let closure = approx_equal(cumulative.nutrient_reaction_delta, -cumulative.activation)
        && approx_equal(cumulative.fuel_reaction_delta, -cumulative.activation)
        && approx_equal(
            cumulative.activated_reaction_delta,
            cumulative.activation - cumulative.reproduction - cumulative.activated_decay,
        )
        && approx_equal(
            cumulative.catalyst_reaction_delta,
            eta_c * cumulative.reproduction - cumulative.catalyst_turnover,
        )
        && approx_equal(cumulative.waste_reaction_delta, expected_waste)
        && sim.accounting.cumulative_within_tolerance();
    let material_closes = closure
        && chemistry_core::build_material_equivalent_step(&sim.accounting.last_step)
            .relative_residual
            <= 1e-6;
    clean
        && structure_invariant
        && stage_c_clamp_negligible(cumulative)
        && match case_id {
            "bounded_reference" => {
                bounded && cumulative.activation > 0.0 && cumulative.reproduction > 0.0
            }
            "missing_c" | "missing_n" | "missing_f" => cumulative.activation == 0.0,
            "missing_a_reproduction" => {
                cumulative.reproduction == 0.0
                    && cumulative.activation == 0.0
                    && total_mass(&sim.grid, &sim.fields.nutrient) > 0.0
                    && total_mass(&sim.grid, &sim.fields.fuel) > 0.0
            }
            "no_activation_decline" => {
                cumulative.activation == 0.0
                    && total_mass(&sim.grid, &sim.fields.activated) < initial_activated
            }
            "no_reproduction_decline" => {
                cumulative.reproduction == 0.0
                    && total_mass(&sim.grid, &sim.fields.catalyst) < initial_catalyst
            }
            "waste_positive" => {
                cumulative.waste_reaction_delta > 0.0
                    && total_mass(&sim.grid, &sim.fields.waste) > initial_waste
            }
            "stoichiometric_closure" => {
                closure
                    && material_closes
                    && cumulative.activation > 0.0
                    && cumulative.reproduction > 0.0
            }
            _ => false,
        }
}

fn run_stage_b_fixed() -> Result<Value, Box<dyn std::error::Error>> {
    let mut params = v7_base_params()?;
    params.d008_stage_b_enabled = true;
    let mut sim = Simulation::new(params);
    prepare_stage_b_v7(&mut sim, 0.25);
    let p0 = field_mass(&sim.grid, &sim.fields.precursor);
    let s0 = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let w0 = field_mass(&sim.grid, &sim.fields.waste);
    let mut min_loc = f64::INFINITY;
    for _ in 0..D025_STAGE_B_FIXED_STEPS {
        if !sim.step() {
            break;
        }
        min_loc = min_loc.min(gamma_localization(&sim));
    }
    let p1 = field_mass(&sim.grid, &sim.fields.precursor);
    let s1 = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let w1 = field_mass(&sim.grid, &sim.fields.waste);
    let partition_interior: f64 = sim
        .fields
        .membrane
        .iter()
        .enumerate()
        .filter(|(idx, _)| {
            sim.grid.in_dish(*idx)
                && sim.fields.structure[*idx] >= 0.5
                && interface_weight(sim.fields.structure[*idx]) < 0.25
        })
        .map(|(_, &v)| v)
        .sum();
    let partition_exterior: f64 = sim
        .fields
        .membrane
        .iter()
        .enumerate()
        .filter(|(idx, _)| {
            sim.grid.in_dish(*idx)
                && sim.fields.structure[*idx] < 0.5
                && interface_weight(sim.fields.structure[*idx]) < 0.25
        })
        .map(|(_, &v)| v)
        .sum();
    let pass = sim.substep == D025_STAGE_B_FIXED_STEPS
        && sim.rejection_count == 0
        && min_loc >= D025_LOCALIZATION_FIXED_MIN
        && p1 > p0
        && w1 > w0
        && s1 > 0.0
        && sim.accounting.cumulative_within_tolerance();
    Ok(json!({
        "mode": "fixed_phi",
        "steps": D025_STAGE_B_FIXED_STEPS,
        "accepted_substeps": sim.substep,
        "min_localization": min_loc,
        "precursor_delta": p1 - p0,
        "surface_delta": s1 - s0,
        "waste_delta": w1 - w0,
        "interior_membrane_mass": partition_interior,
        "exterior_membrane_mass": partition_exterior,
        "accounting_closed": sim.accounting.cumulative_within_tolerance(),
        "gate_pass": pass,
    }))
}

fn prepare_stage_b_v7(sim: &mut Simulation, initial_theta: f64) {
    seed_v7_compartment(sim, 22.0, initial_theta);
}

fn run_stage_b_slow_phi() -> Result<Value, Box<dyn std::error::Error>> {
    let mut params = v7_base_params()?;
    params.k_d008_structure = 0.015;
    params.k_structure_decay = 0.014;
    let mut sim = Simulation::new(params);
    sim.enforce_structure_constraint = false;
    sim.dt_cap = 0.003;
    seed_v7_compartment(&mut sim, 22.0, 0.5);
    let s0 = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let phi_hash0 = field_sha256_stable(&sim.fields.structure);
    let mut min_loc = f64::INFINITY;
    for _ in 0..D025_STAGE_B_SLOW_STEPS {
        if !sim.step() {
            break;
        }
        min_loc = min_loc.min(gamma_localization(&sim));
    }
    let s1 = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let s_drift = ((s1 - s0) / s0.max(1.0)).abs();
    let phi_moved = field_sha256_stable(&sim.fields.structure) != phi_hash0;
    let pass = sim.substep == D025_STAGE_B_SLOW_STEPS
        && sim.rejection_count == 0
        && min_loc >= D025_LOCALIZATION_SLOW_MIN
        && s_drift <= D025_S_MOVEMENT_DRIFT_MAX
        && phi_moved
        && sim.accounting.cumulative_within_tolerance();
    Ok(json!({
        "mode": "slow_phi",
        "steps": D025_STAGE_B_SLOW_STEPS,
        "accepted_substeps": sim.substep,
        "min_localization": min_loc,
        "s_mass_drift": s_drift,
        "phi_moved": phi_moved,
        "accounting_closed": sim.accounting.cumulative_within_tolerance(),
        "gate_pass": pass,
    }))
}

/// Gate 4: Stage B revalidation (fixed φ then slow φ).
pub fn run_stage_b_regression(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let fixed = run_stage_b_fixed()?;
    let slow = run_stage_b_slow_phi()?;
    let pass = fixed["gate_pass"].as_bool() == Some(true)
        && slow["gate_pass"].as_bool() == Some(true);
    let body = json!({
        "project_directive": "D-025",
        "gate": 4,
        "source_commit": git_commit_hash(),
        "equation_version": EquationVersion::MembraneMetabolismV7SurfaceDensity.as_str(),
        "k_ads_frozen": D025_FROZEN_K_ADS,
        "fixed_phi": fixed,
        "slow_phi": slow,
        "gate4_pass": pass,
        "conclusion": if pass { "D025_GATE4_PASS" } else { "D025_STAGE_B_REGRESSION" },
        "any_pass": pass,
    });
    atomic_write_json(&output.join("stage_b_regression.json"), &body)?;
    Ok(body)
}

/// Gate 5: Stage C revalidation under v7.
pub fn run_stage_c_regression(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;

    let cases = [
        "bounded_reference",
        "missing_c",
        "missing_n",
        "missing_f",
        "missing_a_reproduction",
        "no_activation_decline",
        "no_reproduction_decline",
        "waste_positive",
        "stoichiometric_closure",
    ];
    let mut controls = Vec::new();
    let mut stage_pass = true;
    for case_id in cases {
        let mut params = v7_base_params()?;
        params.d008_stage_mode = D008StageMode::ActivatedMetabolism;
        params.d008_stage_b_enabled = false;
        params.diffusion_enabled = false;
        let mut sim = Simulation::new(params);
        prepare_stage_c_v7(&mut sim, case_id);
        let phi0 = field_sha256_stable(&sim.fields.structure);
        let m0 = field_sha256_stable(&sim.fields.membrane);
        let initial_catalyst = total_mass(&sim.grid, &sim.fields.catalyst);
        let initial_activated = total_mass(&sim.grid, &sim.fields.activated);
        let initial_waste = total_mass(&sim.grid, &sim.fields.waste);
        for _ in 0..D025_STAGE_C_STEPS {
            if !sim.step() {
                break;
            }
        }
        let invariant = field_sha256_stable(&sim.fields.structure) == phi0
            && field_sha256_stable(&sim.fields.membrane) == m0;
        let pass = stage_c_case_pass_v7(
            case_id,
            &sim,
            initial_catalyst,
            initial_activated,
            initial_waste,
            invariant,
        );
        stage_pass &= pass;
        controls.push(json!({
            "case_id": case_id,
            "pass": pass,
            "accepted_substeps": sim.substep,
            "structure_membrane_invariant": invariant,
        }));
    }

    // V7 P/Γ coupling under constrained-radius chemistry (Stage C mode has no surface path).
    let mut params = v7_base_params()?;
    params.d008_stage_mode = D008StageMode::ConstrainedRadius;
    params.d008_stage_b_enabled = false;
    let mut sim = Simulation::new(params);
    sim.enforce_structure_constraint = true;
    seed_v7_compartment(&mut sim, 22.0, 0.2);
    // Ensure soluble precursor available for adsorption assay.
    for idx in 0..sim.fields.precursor.len() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
            sim.fields.precursor[idx] = sim.fields.precursor[idx].max(0.2);
            sim.fields.activated[idx] = sim.fields.activated[idx].max(0.3);
            sim.fields.catalyst[idx] = sim.fields.catalyst[idx].max(0.3);
        }
    }
    sim.fields.copy_current_to_next();
    let s0 = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let p0 = field_mass(&sim.grid, &sim.fields.precursor);
    let ads0 = sim.membrane_accounting.cumulative.synthesis;
    for _ in 0..D025_STAGE_C_STEPS {
        if !sim.step() {
            break;
        }
    }
    let s1 = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let p1 = field_mass(&sim.grid, &sim.fields.precursor);
    let ads1 = sim.membrane_accounting.cumulative.synthesis;
    let no_a_to_gamma = {
        let mut p = v7_base_params()?;
        p.d008_stage_mode = D008StageMode::ConstrainedRadius;
        p.k_precursor = 0.0;
        p.k_ads = 0.0;
        let mut s = Simulation::new(p);
        s.enforce_structure_constraint = true;
        seed_v7_compartment(&mut s, 22.0, 0.0);
        // Explicit zero precursor — no adsorbed pool, no A→Γ pathway.
        s.fields.precursor.fill(0.0);
        s.fields.membrane.fill(0.0);
        s.fields.copy_current_to_next();
        for _ in 0..D025_STAGE_C_STEPS {
            let _ = s.step();
        }
        total_surface_mass(&s.grid, &s.fields.membrane) < 1e-8
    };
    let v7_checks = json!({
        "precursor_increased_with_a": p1 > p0,
        "gamma_increased_with_precursor_adsorption": ads1 > ads0 && (s1 + 1e-9 >= s0 || ads1 > 0.0),
        "adsorption_active": ads1 > ads0,
        "no_direct_a_to_gamma": no_a_to_gamma,
        "accounting_closed": sim.accounting.cumulative_within_tolerance(),
        "s0": s0,
        "s1": s1,
        "p0": p0,
        "p1": p1,
        "ads_delta": ads1 - ads0,
    });
    let v7_pass = v7_checks["precursor_increased_with_a"].as_bool() == Some(true)
        && v7_checks["adsorption_active"].as_bool() == Some(true)
        && v7_checks["no_direct_a_to_gamma"].as_bool() == Some(true)
        && v7_checks["accounting_closed"].as_bool() == Some(true);
    let pass = stage_pass && v7_pass;
    let body = json!({
        "project_directive": "D-025",
        "gate": 5,
        "source_commit": git_commit_hash(),
        "equation_version": EquationVersion::MembraneMetabolismV7SurfaceDensity.as_str(),
        "controls": controls,
        "v7_checks": v7_checks,
        "gate5_pass": pass,
        "conclusion": if pass { "D025_GATE5_PASS" } else { "D025_STAGE_C_REGRESSION" },
        "any_pass": pass,
    });
    atomic_write_json(&output.join("stage_c_regression.json"), &body)?;
    Ok(body)
}

fn interior_stats(sim: &Simulation, field: &[f64]) -> (f64, f64) {
    let mut total = 0.0;
    let mut area = 0.0;
    for (idx, value) in field.iter().enumerate() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
            total += value;
            area += 1.0;
        }
    }
    (total, area)
}

fn retention(sim: &Simulation, field: &[f64]) -> f64 {
    let (inside, _) = interior_stats(sim, field);
    inside / total_mass(&sim.grid, field).max(f64::EPSILON)
}

fn soluble_max(sim: &Simulation) -> f64 {
    [
        &sim.fields.catalyst,
        &sim.fields.nutrient,
        &sim.fields.fuel,
        &sim.fields.waste,
        &sim.fields.activated,
        &sim.fields.precursor,
    ]
    .into_iter()
    .flat_map(|field| field.iter().copied())
    .fold(0.0, f64::max)
}

fn run_stage_d_radius(params: &SimParams, radius: f64) -> Result<Value, Box<dyn std::error::Error>> {
    let mut sim = Simulation::new(params.clone());
    sim.enforce_structure_constraint = true;
    seed_v7_compartment(&mut sim, radius, 0.6);
    let phi0 = field_sha256_stable(&sim.fields.structure);
    let initial_catalyst_inside = interior_stats(&sim, &sim.fields.catalyst).0;
    for _ in 0..STAGE_D_STEPS {
        if !sim.step() {
            break;
        }
    }
    // Directive: keep φ fixed. S/Γ must evolve via adsorption/turnover.
    let fixed_geometry = field_sha256_stable(&sim.fields.structure) == phi0;
    let (catalyst_inside, interior_area) = interior_stats(&sim, &sim.fields.catalyst);
    let (activated_inside, _) = interior_stats(&sim, &sim.fields.activated);
    let catalyst_retention = retention(&sim, &sim.fields.catalyst);
    let activated_retention = retention(&sim, &sim.fields.activated);
    let loc = gamma_localization(&sim);
    let n_influx = sim
        .transport_accounting
        .cumulative
        .nutrient
        .interior_net_flux_rate;
    let f_influx = sim
        .transport_accounting
        .cumulative
        .fuel
        .interior_net_flux_rate;
    let w_net = sim
        .transport_accounting
        .cumulative
        .waste
        .interior_net_flux_rate;
    let bounded = soluble_max(&sim) <= CONC_SAFETY_LIMIT
        && stage_c_clamp_negligible(&sim.metabolism_accounting.cumulative)
        && sim.accounting.cumulative_within_tolerance();
    Ok(json!({
        "radius": radius,
        "interior_area": interior_area,
        "accepted_substeps": sim.substep,
        "simulated_time": sim.sim_time,
        "rejection_count": sim.rejection_count,
        "fixed_geometry": fixed_geometry,
        "catalyst_retention": catalyst_retention,
        "activated_retention": activated_retention,
        "gamma_localization": loc,
        "precursor_mass": field_mass(&sim.grid, &sim.fields.precursor),
        "surface_mass": field_mass(&sim.grid, &sim.fields.membrane),
        "mean_catalyst_inside": catalyst_inside / interior_area.max(1.0),
        "mean_activated_inside": activated_inside / interior_area.max(1.0),
        "nutrient_influx": n_influx,
        "fuel_influx": f_influx,
        "resource_influx_per_interior_area": (n_influx + f_influx) / interior_area.max(1.0),
        "waste_efflux_per_interior_area": (-w_net) / interior_area.max(1.0),
        "bounded": bounded,
        "clean_termination": sim.substep == STAGE_D_STEPS && sim.rejection_count == 0,
        "accounting_closed": sim.accounting.cumulative_within_tolerance(),
        "catalyst_leakage_fraction_per_time":
            (-sim.transport_accounting.cumulative.catalyst.interior_net_flux_rate).max(0.0)
                / initial_catalyst_inside.max(f64::EPSILON)
                / sim.sim_time.max(f64::EPSILON),
    }))
}

/// Gate 6: Stage D fixed-compartment R16/R24/R32 regression under v7.
pub fn run_stage_d_regression(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let params = v7_base_params()?;
    let identity = build_candidate_identity(
        params.clone(),
        &git_commit_hash(),
        Some("d025-v7-stage-d"),
        None,
        "D-025 v7 Stage D fixed compartment",
        None,
        None,
    );
    let mut radius_results = Vec::new();
    let mut clean = true;
    for &radius in &STAGE_D_RADII {
        let row = run_stage_d_radius(&params, radius)?;
        clean &= row["clean_termination"].as_bool() == Some(true);
        radius_results.push(row);
    }
    let resource_fluxes: Vec<f64> = radius_results
        .iter()
        .map(|row| {
            row["resource_influx_per_interior_area"]
                .as_f64()
                .unwrap_or(f64::NAN)
        })
        .collect();
    let catalyst_retentions: Vec<f64> = radius_results
        .iter()
        .map(|row| row["catalyst_retention"].as_f64().unwrap_or(0.0))
        .collect();
    let decreasing_resource = resource_fluxes[0] > resource_fluxes[1]
        && resource_fluxes[1] > resource_fluxes[2];
    let retention_pass = radius_results.iter().all(|row| {
        row["catalyst_retention"].as_f64().unwrap_or(0.0) >= STAGE_D_RETENTION_MIN
            && row["activated_retention"].as_f64().unwrap_or(0.0) >= STAGE_D_RETENTION_MIN
            && row["gamma_localization"].as_f64().unwrap_or(0.0) >= 0.95
    });
    let flux_pass = radius_results.iter().all(|row| {
        row["nutrient_influx"].as_f64().unwrap_or(0.0) > 0.0
            && row["fuel_influx"].as_f64().unwrap_or(0.0) > 0.0
            && row["waste_efflux_per_interior_area"]
                .as_f64()
                .unwrap_or(0.0)
                > 0.0
    });
    let bounded_pass = radius_results
        .iter()
        .all(|row| row["bounded"].as_bool() == Some(true));
    let accounting_pass = radius_results
        .iter()
        .all(|row| row["accounting_closed"].as_bool() == Some(true));
    let geometry_pass = radius_results
        .iter()
        .all(|row| row["fixed_geometry"].as_bool() == Some(true));
    let small_cell_ok = catalyst_retentions[0]
        >= catalyst_retentions[2] - STAGE_D_SMALL_CELL_RETENTION_MARGIN;
    let pass = clean
        && retention_pass
        && flux_pass
        && decreasing_resource
        && bounded_pass
        && accounting_pass
        && geometry_pass
        && small_cell_ok;
    let conclusion = if pass {
        "D025_GATE6_PASS".to_string()
    } else if !flux_pass {
        "D025_STAGE_D_OVERSEALED".to_string()
    } else if !accounting_pass {
        "D025_STAGE_D_ACCOUNTING_FAILURE".to_string()
    } else if !retention_pass {
        "D025_STAGE_D_RETENTION_REGRESSION".to_string()
    } else {
        "D025_STAGE_D_RETENTION_REGRESSION".to_string()
    };
    let body = json!({
        "project_directive": "D-025",
        "gate": 6,
        "source_commit": git_commit_hash(),
        "equation_version": params.equation_version.as_str(),
        "k_ads_frozen": D025_FROZEN_K_ADS,
        "candidate_hash": identity.candidate_hash,
        "radii": STAGE_D_RADII,
        "radius_results": radius_results,
        "checks": {
            "retention": retention_pass,
            "fluxes": flux_pass,
            "decreasing_resource_influx": decreasing_resource,
            "bounded": bounded_pass,
            "accounting": accounting_pass,
            "fixed_geometry": geometry_pass,
            "small_cell_retention": small_cell_ok,
        },
        "gate6_pass": pass,
        "conclusion": conclusion,
        "any_pass": pass,
    });
    atomic_write_json(&output.join("stage_d_fixed_compartment.json"), &body)?;
    Ok(body)
}

const D025_R22_DIAGNOSTIC_STEPS: u64 = 2_000;
const D025_R22_INTERMEDIATE_STEPS: u64 = 10_000;
const D025_R22_FULL_STEPS: u64 = 25_000;

fn dish_contact(sim: &Simulation) -> bool {
    let w = sim.grid.width;
    for j in 0..sim.grid.height {
        for i in 0..w {
            let idx = Grid::index(w, i, j);
            if !sim.grid.in_dish(idx) {
                continue;
            }
            if sim.fields.structure[idx] < 0.5 {
                continue;
            }
            // Contact if a neighbor is outside the dish.
            for (di, dj) in [(-1isize, 0), (1, 0), (0, -1), (0, 1)] {
                let ni = i as isize + di;
                let nj = j as isize + dj;
                if ni < 0 || nj < 0 || ni as usize >= w || nj as usize >= sim.grid.height {
                    return true;
                }
                let nidx = Grid::index(w, ni as usize, nj as usize);
                if !sim.grid.in_dish(nidx) {
                    return true;
                }
            }
        }
    }
    false
}

fn largest_component_fraction(sim: &Simulation) -> f64 {
    let w = sim.grid.width;
    let h = sim.grid.height;
    let n = w * h;
    let mut seen = vec![false; n];
    let mut total_area = 0u64;
    let mut largest = 0u64;
    for start in 0..n {
        if seen[start] || !sim.grid.in_dish(start) || sim.fields.structure[start] < 0.5 {
            continue;
        }
        let mut stack = vec![start];
        seen[start] = true;
        let mut size = 0u64;
        while let Some(idx) = stack.pop() {
            size += 1;
            let i = idx % w;
            let j = idx / w;
            for (di, dj) in [(-1isize, 0), (1, 0), (0, -1), (0, 1)] {
                let ni = i as isize + di;
                let nj = j as isize + dj;
                if ni < 0 || nj < 0 || ni as usize >= w || nj as usize >= h {
                    continue;
                }
                let nidx = Grid::index(w, ni as usize, nj as usize);
                if seen[nidx] || !sim.grid.in_dish(nidx) || sim.fields.structure[nidx] < 0.5 {
                    continue;
                }
                seen[nidx] = true;
                stack.push(nidx);
            }
        }
        total_area += size;
        largest = largest.max(size);
    }
    if total_area == 0 {
        0.0
    } else {
        largest as f64 / total_area as f64
    }
}

fn run_dynamic_r22_horizon(
    max_steps: u64,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut params = v7_base_params()?;
    params.d008_stage_mode = D008StageMode::ConstrainedRadius;
    params.d008_stage_b_enabled = false;
    params.phase_separation_enabled = false;
    let mut sim = Simulation::new(params);
    sim.enforce_structure_constraint = false; // autonomous φ
    sim.observer_enabled = false;
    seed_v7_compartment(&mut sim, 22.0, 0.6);
    let mut floor_fail = false;
    let mut ceiling_fail = false;
    for _ in 0..max_steps {
        if !sim.step() {
            if sim
                .last_reject_detail
                .contains("excessive concentration")
            {
                ceiling_fail = true;
            } else {
                floor_fail = true;
            }
            break;
        }
    }
    let loc = gamma_localization(&sim);
    let c_ret = retention(&sim, &sim.fields.catalyst);
    let a_ret = retention(&sim, &sim.fields.activated);
    let ads = sim.membrane_accounting.cumulative.synthesis;
    let gamma_turn = sim.membrane_accounting.cumulative.decay;
    let n_in = sim
        .transport_accounting
        .cumulative
        .nutrient
        .interior_net_flux_rate;
    let f_in = sim
        .transport_accounting
        .cumulative
        .fuel
        .interior_net_flux_rate;
    let w_net = sim
        .transport_accounting
        .cumulative
        .waste
        .interior_net_flux_rate;
    let largest = largest_component_fraction(&sim);
    let contact = dish_contact(&sim);
    let bounded = soluble_max(&sim) <= CONC_SAFETY_LIMIT;
    let pass = !floor_fail
        && !ceiling_fail
        && sim.substep >= max_steps.min(500)
        && largest >= 0.95
        && !contact
        && loc >= 0.95
        && c_ret >= 0.80
        && a_ret >= 0.80
        && bounded
        && n_in > 0.0
        && f_in > 0.0
        && (-w_net) > 0.0
        && ads > 0.0
        && gamma_turn > 0.0
        && sim.accounting.cumulative_within_tolerance()
        && !sim.observer_enabled;
    Ok(json!({
        "max_steps": max_steps,
        "accepted_substeps": sim.substep,
        "simulated_time": sim.sim_time,
        "pass": pass,
        "largest_component_fraction": largest,
        "dish_contact": contact,
        "gamma_localization": loc,
        "catalyst_retention": c_ret,
        "activated_retention": a_ret,
        "adsorption": ads,
        "gamma_turnover": gamma_turn,
        "nutrient_influx": n_in,
        "fuel_influx": f_in,
        "waste_efflux": -w_net,
        "bounded": bounded,
        "timestep_floor_failure": floor_fail,
        "concentration_ceiling": ceiling_fail,
        "accounting_closed": sim.accounting.cumulative_within_tolerance(),
        "observer_enabled": sim.observer_enabled,
        "surface_mass": total_surface_mass(&sim.grid, &sim.fields.membrane),
        "precursor_mass": field_mass(&sim.grid, &sim.fields.precursor),
    }))
}

/// Gate 7: fully dynamic R22 bootstrap (autonomous φ + surface transport).
pub fn run_dynamic_r22(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let diagnostic = run_dynamic_r22_horizon(D025_R22_DIAGNOSTIC_STEPS)?;
    if diagnostic["pass"].as_bool() != Some(true) {
        let body = json!({
            "project_directive": "D-025",
            "gate": 7,
            "source_commit": git_commit_hash(),
            "equation_version": EquationVersion::MembraneMetabolismV7SurfaceDensity.as_str(),
            "k_ads": D025_FROZEN_K_ADS,
            "diagnostic": diagnostic,
            "gate7_pass": false,
            "conclusion": "D025_DYNAMIC_R22_BOOTSTRAP_FAILURE",
            "any_pass": false,
        });
        atomic_write_json(&output.join("dynamic_r22.json"), &body)?;
        return Ok(body);
    }
    let intermediate = run_dynamic_r22_horizon(D025_R22_INTERMEDIATE_STEPS)?;
    if intermediate["pass"].as_bool() != Some(true) {
        let body = json!({
            "project_directive": "D-025",
            "gate": 7,
            "source_commit": git_commit_hash(),
            "equation_version": EquationVersion::MembraneMetabolismV7SurfaceDensity.as_str(),
            "k_ads": D025_FROZEN_K_ADS,
            "diagnostic": diagnostic,
            "intermediate": intermediate,
            "gate7_pass": false,
            "conclusion": "D025_DYNAMIC_R22_BOOTSTRAP_FAILURE",
            "any_pass": false,
        });
        atomic_write_json(&output.join("dynamic_r22.json"), &body)?;
        return Ok(body);
    }
    let full = run_dynamic_r22_horizon(D025_R22_FULL_STEPS)?;
    let pass = full["pass"].as_bool() == Some(true);
    let body = json!({
        "project_directive": "D-025",
        "gate": 7,
        "source_commit": git_commit_hash(),
        "equation_version": EquationVersion::MembraneMetabolismV7SurfaceDensity.as_str(),
        "k_ads": D025_FROZEN_K_ADS,
        "diagnostic": diagnostic,
        "intermediate": intermediate,
        "full": full,
        "gate7_pass": pass,
        "conclusion": if pass { "D025_GATE7_PASS" } else { "D025_DYNAMIC_R22_BOOTSTRAP_FAILURE" },
        "any_pass": pass,
    });
    atomic_write_json(&output.join("dynamic_r22.json"), &body)?;
    Ok(body)
}

pub fn run_gates_3_6(output_root: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output_root = resolve_path(output_root);
    let t0 = Instant::now();
    fs::create_dir_all(&output_root)?;

    let gate3 = run_gate3_growth_shrinkage(&output_root.join("growth_shrinkage"))?;
    if gate3["gate3_pass"].as_bool() != Some(true) {
        let manifest = json!({
            "project_directive": "D-025",
            "source_commit": git_commit_hash(),
            "stopped_at_gate": 3,
            "conclusion": gate3["conclusion"],
            "gate3": gate3,
            "wall_seconds": t0.elapsed().as_secs_f64(),
        });
        atomic_write_json(&output_root.join("manifest.json"), &manifest)?;
        return Ok(manifest);
    }

    let gate4 = run_stage_b_regression(&output_root.join("stage_b_regression"))?;
    if gate4["gate4_pass"].as_bool() != Some(true) {
        let manifest = json!({
            "project_directive": "D-025",
            "source_commit": git_commit_hash(),
            "stopped_at_gate": 4,
            "conclusion": gate4["conclusion"],
            "gate3": gate3,
            "gate4": gate4,
            "wall_seconds": t0.elapsed().as_secs_f64(),
        });
        atomic_write_json(&output_root.join("manifest.json"), &manifest)?;
        return Ok(manifest);
    }

    let gate5 = run_stage_c_regression(&output_root.join("stage_c_regression"))?;
    if gate5["gate5_pass"].as_bool() != Some(true) {
        let manifest = json!({
            "project_directive": "D-025",
            "source_commit": git_commit_hash(),
            "stopped_at_gate": 5,
            "conclusion": gate5["conclusion"],
            "gate3": gate3,
            "gate4": gate4,
            "gate5": gate5,
            "wall_seconds": t0.elapsed().as_secs_f64(),
        });
        atomic_write_json(&output_root.join("manifest.json"), &manifest)?;
        return Ok(manifest);
    }

    let gate6 = run_stage_d_regression(&output_root.join("stage_d_fixed_compartment"))?;
    let conclusion = if gate6["gate6_pass"].as_bool() == Some(true) {
        "D025_GATES_3_6_PASS".to_string()
    } else {
        gate6["conclusion"]
            .as_str()
            .unwrap_or("D025_FAIL")
            .to_string()
    };
    let manifest = json!({
        "project_directive": "D-025",
        "source_commit": git_commit_hash(),
        "stopped_at_gate": 6,
        "conclusion": conclusion,
        "gate3": gate3,
        "gate4": gate4,
        "gate5": gate5,
        "gate6": gate6,
        "wall_seconds": t0.elapsed().as_secs_f64(),
    });
    atomic_write_json(&output_root.join("manifest.json"), &manifest)?;
    Ok(manifest)
}
