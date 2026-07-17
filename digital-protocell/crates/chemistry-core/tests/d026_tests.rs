//! D-026 Gate 0 dynamic/constrained parity and Gate 1 observability schema tests.

use chemistry_core::config::{D008StageMode, EquationVersion, SimParams};
use chemistry_core::d011_analysis::STAGE_E_FAILED_RATES;
use chemistry_core::d018_analysis::D018_FROZEN_K_STRUCTURE;
use chemistry_core::d025_analysis::{D025ProductiveRates, D025_FROZEN_K_ADS};
use chemistry_core::d026_analysis::{
    analytic_seed_conservation_check, classify_mechanism, global_rate_bounds_ok,
    productive_rates_within_global_bounds, productive_rates_within_round_bounds,
    run_runner_parity, sample_stage_e_observability, settle_constrained, D026_MAX_CANDIDATES,
    D026_SETTLE_STEPS,
};
use chemistry_core::surface_density::{
    compute_interface_geometry, seed_surface_from_gamma, total_surface_mass, InterfaceGeometryCell,
};
use chemistry_core::{field_sha256_stable, Simulation};

const D026_FROZEN_K_ADS: f64 = 0.0011111111111111111;

fn v7_stage_e_params() -> SimParams {
    let mut p = SimParams::default();
    p.equation_version = EquationVersion::MembraneMetabolismV7SurfaceDensity;
    p.d008_stage_mode = D008StageMode::ConstrainedRadius;
    p.d008_stage_b_enabled = false;
    p.reactions_enabled = true;
    p.diffusion_enabled = true;
    p.phase_separation_enabled = false;
    p.k_ads = D026_FROZEN_K_ADS;
    p.d_p = p.d_a;
    p.k_precursor_decay = p.k_d008_activated_decay;
    p.d_gamma = 0.02;
    p.gamma_max = 1.0;
    p.gamma_reference = 1.0;
    STAGE_E_FAILED_RATES.apply_to(&mut p);
    p.k_d008_structure = D018_FROZEN_K_STRUCTURE;
    p
}

fn seed_v7_stage_e(sim: &mut Simulation, radius: f64) {
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
        |_, _, _| 0.6,
    );
    sim.fields.copy_current_to_next();
}

fn v7_stage_e_sim() -> Simulation {
    let mut sim = Simulation::new(v7_stage_e_params());
    sim.enforce_structure_constraint = true;
    sim.dt_cap = 0.005;
    seed_v7_stage_e(&mut sim, 22.0);
    settle_constrained(&mut sim, D026_SETTLE_STEPS);
    sim
}

#[test]
fn test_gate0_dynamic_constrained_one_step_chemistry_parity() {
    let base = v7_stage_e_sim();
    let report = run_runner_parity(&base);
    assert!(
        report.path_a_accepted && report.path_b_accepted,
        "steps must accept: a={} b={} notes={:?}",
        report.path_a_accepted,
        report.path_b_accepted,
        report.notes
    );
    assert!(
        report.gate0_pass,
        "gate0 parity failed max_diff={} metric={} notes={:?} diffs={:?}",
        report.max_abs_diff,
        report.max_abs_diff_metric,
        report.notes,
        report.diffs.iter().filter(|d| !d.within_tolerance).collect::<Vec<_>>()
    );
}

#[test]
fn test_virtual_structural_a_consumption_parity() {
    let base = v7_stage_e_sim();
    let report = run_runner_parity(&base);
    let vp = report
        .diffs
        .iter()
        .find(|d| d.name == "virtual_production")
        .expect("virtual_production diff");
    let vd = report
        .diffs
        .iter()
        .find(|d| d.name == "virtual_decay")
        .expect("virtual_decay diff");
    assert!(vp.within_tolerance, "virtual_production mismatch {:?}", vp);
    assert!(vd.within_tolerance, "virtual_decay mismatch {:?}", vd);
    let da = report
        .diffs
        .iter()
        .find(|d| d.name == "delta_a")
        .expect("delta_a diff");
    assert!(da.within_tolerance, "activated mass delta mismatch {:?}", da);
}

#[test]
fn test_virtual_structural_w_production_parity() {
    let base = v7_stage_e_sim();
    let report = run_runner_parity(&base);
    let dw = report
        .diffs
        .iter()
        .find(|d| d.name == "delta_w")
        .expect("delta_w diff");
    assert!(dw.within_tolerance, "waste mass delta mismatch {:?}", dw);
}

#[test]
fn test_constraint_ledger_isolation() {
    let base = v7_stage_e_sim();
    let report = run_runner_parity(&base);
    assert!(report.path_a_constraint_flux_zero);
    assert!(report.path_b_constraint_isolated);
}

#[test]
fn test_fixed_phi_surface_advection_disablement() {
    let base = v7_stage_e_sim();
    let report = run_runner_parity(&base);
    assert!(report.path_b_phi_unchanged);
    assert!(report.path_b_advection_disabled);
    assert!(
        report.surface_mass_parity,
        "surface mass parity failed diff={}",
        report.surface_mass_abs_diff
    );
}

#[test]
fn test_stage_e_observability_schema_populates_finite_fields() {
    let mut sim = v7_stage_e_sim();
    assert!(sim.step());
    let sample = sample_stage_e_observability(&sim);
    assert!(sample.mass_c.is_finite() && sample.mass_c > 0.0);
    assert!(sample.mass_a.is_finite() && sample.mass_a > 0.0);
    assert!(sample.mass_p.is_finite());
    assert!(sample.mass_s.is_finite() && sample.mass_s > 0.0);
    assert!(sample.interface_measure.is_finite() && sample.interface_measure > 0.0);
    assert!(sample.structural_mass.is_finite() && sample.structural_mass > 0.0);
    assert!(sample.mean_internal_n.is_finite());
    assert!(sample.mean_internal_f.is_finite());
    assert!(sample.mean_internal_w.is_finite());
}

#[test]
fn test_a_production_demand_accounting_fields_present() {
    let mut sim = v7_stage_e_sim();
    assert!(sim.step());
    let sample = sample_stage_e_observability(&sim);
    assert!(sample.a_production_activation.is_finite());
    assert!(sample.a_consumption_catalyst_reproduction.is_finite());
    assert!(sample.a_consumption_precursor_production.is_finite());
    assert!(sample.a_consumption_virtual_structural.is_finite());
    assert!(sample.a_consumption_decay.is_finite());
    assert!(sample.activation_to_demand.is_finite());
    assert!(sample.activation_to_leakage.is_finite());
}

#[test]
fn test_surface_occupancy_quantiles_present() {
    let sim = v7_stage_e_sim();
    let sample = sample_stage_e_observability(&sim);
    let s = &sample.surface;
    assert!(s.mean_gamma.is_finite() && s.mean_gamma > 0.0);
    assert!(s.median_gamma.is_finite());
    assert!(s.p25_gamma <= s.p50_gamma && s.p50_gamma <= s.p75_gamma);
    assert!(s.min_gamma <= s.mean_gamma);
}

#[test]
fn test_low_coverage_fraction_in_unit_interval() {
    let sim = v7_stage_e_sim();
    let sample = sample_stage_e_observability(&sim);
    let s = &sample.surface;
    for frac in [
        s.fraction_below_0_25_gamma_ref,
        s.fraction_below_0_50_gamma_ref,
        s.fraction_below_0_75_gamma_ref,
    ] {
        assert!((0.0..=1.0).contains(&frac), "fraction out of range: {frac}");
    }
}

#[test]
fn test_analytic_seed_conservation_stub() {
    let sim = v7_stage_e_sim();
    let report = analytic_seed_conservation_check(&sim);
    assert!(report.within_seed_tol);
    assert!(report.total_cnpfwas.is_finite() && report.total_cnpfwas > 0.0);
}

#[test]
fn test_targeted_rate_bounds_helpers() {
    assert!(global_rate_bounds_ok(0.25));
    assert!(global_rate_bounds_ok(4.0));
    assert!(!global_rate_bounds_ok(0.24));
    assert!(!global_rate_bounds_ok(4.01));
    assert!(productive_rates_within_round_bounds(0.67));
    assert!(productive_rates_within_round_bounds(1.5));
    assert!(!productive_rates_within_round_bounds(0.66));
    let reference = D025ProductiveRates::from_legacy(&STAGE_E_FAILED_RATES, D025_FROZEN_K_ADS);
    let candidate = reference;
    assert!(productive_rates_within_global_bounds(&candidate, &reference));
}

#[test]
fn test_max_candidate_count_constant() {
    assert_eq!(D026_MAX_CANDIDATES, 5);
}

#[test]
fn test_precursor_a_consumption_uses_synthesis_delta() {
    let mut sim = v7_stage_e_sim();
    assert!(sim.step());
    let sample = sample_stage_e_observability(&sim);
    let syn = sim
        .last_surface_totals
        .expect("surface totals")
        .precursor_synthesis_delta;
    assert!(syn >= 0.0);
    assert!(
        (sample.a_consumption_precursor_production - syn).abs() < 1e-12,
        "precursor A consumption must use synthesis delta, got {} vs {}",
        sample.a_consumption_precursor_production,
        syn
    );
    if syn > 1e-12 && sim.membrane_accounting.last_step.synthesis > 1e-12 {
        assert!(
            (sample.a_consumption_precursor_production - sim.membrane_accounting.last_step.synthesis)
                .abs()
                > 1e-12,
            "precursor consumption must differ from adsorption when both active"
        );
    }
}

#[test]
fn test_freeze_surface_disables_s_change() {
    let mut sim = v7_stage_e_sim();
    sim.d026_freeze_surface = true;
    let s0 = total_surface_mass(&sim.grid, &sim.fields.membrane);
    for _ in 0..50 {
        assert!(sim.step());
    }
    let s1 = total_surface_mass(&sim.grid, &sim.fields.membrane);
    assert!(
        (s1 - s0).abs() < 1e-12,
        "freeze_surface must hold S: {s0} -> {s1}"
    );
}

#[test]
fn test_no_leak_control_zeros_a_transport_flux() {
    let mut sim = v7_stage_e_sim();
    sim.d026_disable_a_normal_transport = true;
    assert!(sim.step());
    let flux = sim.transport_accounting.last_step.activated.interior_net_flux_rate;
    assert!(
        flux.abs() < 1e-12,
        "disable A transport must zero interior flux, got {flux}"
    );
}

#[test]
fn test_disable_virtual_structure_zeros_demand() {
    let mut sim = v7_stage_e_sim();
    sim.d026_disable_virtual_structure = true;
    assert!(sim.step());
    assert!(
        sim.constraint_accounting.last_step.virtual_production.abs() < 1e-12
    );
    let sample = sample_stage_e_observability(&sim);
    assert!(sample.a_consumption_virtual_structural.abs() < 1e-12);
}

#[test]
fn test_disable_catalyst_reproduction_reduces_reproduction() {
    let mut base = v7_stage_e_sim();
    assert!(base.step());
    let repro_base = base.metabolism_accounting.last_step.reproduction;

    let mut ctrl = v7_stage_e_sim();
    ctrl.d026_disable_catalyst_reproduction = true;
    assert!(ctrl.step());
    assert!(ctrl.metabolism_accounting.last_step.reproduction.abs() < 1e-12);
    if repro_base > 1e-12 {
        assert!(ctrl.metabolism_accounting.last_step.reproduction < repro_base);
    }
}

#[test]
fn test_disable_precursor_synthesis_zeros_synthesis_delta() {
    let mut sim = v7_stage_e_sim();
    sim.d026_disable_precursor_synthesis = true;
    sim.d026_freeze_surface = true;
    assert!(sim.step());
    let syn = sim
        .last_surface_totals
        .expect("surface totals")
        .precursor_synthesis_delta;
    assert!(syn.abs() < 1e-12);
}

#[test]
fn test_classify_mechanism_returns_valid_label() {
    let sim = v7_stage_e_sim();
    let sample = sample_stage_e_observability(&sim);
    assert!(!classify_mechanism(&sample).as_str().is_empty());
}

#[test]
fn test_path_b_phi_hash_unchanged_after_parity_step() {
    let base = v7_stage_e_sim();
    let phi0 = field_sha256_stable(&base.fields.structure);
    let mut path_b = base.clone();
    path_b.enforce_structure_constraint = true;
    assert!(path_b.step());
    assert_eq!(field_sha256_stable(&path_b.fields.structure), phi0);
}

#[test]
fn test_path_a_phi_may_change_when_structure_active() {
    let base = v7_stage_e_sim();
    let phi0 = field_sha256_stable(&base.fields.structure);
    let mut path_a = base.clone();
    path_a.enforce_structure_constraint = false;
    if path_a.step() {
        let moved = field_sha256_stable(&path_a.fields.structure) != phi0;
        let s_before = total_surface_mass(&path_a.grid, &base.fields.membrane);
        let s_after = total_surface_mass(&path_a.grid, &path_a.fields.membrane);
        let _ = (moved, s_before, s_after);
    }
}
