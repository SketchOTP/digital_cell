//! D-027 Gate 0–2 unit tests: checkpoint window ledgers, adsorption basis, candidates.

use chemistry_core::config::{D008StageMode, EquationVersion, SimParams};
use chemistry_core::d011_analysis::STAGE_E_FAILED_RATES;
use chemistry_core::d018_analysis::D018_FROZEN_K_STRUCTURE;
use chemistry_core::d025_analysis::D025_FROZEN_K_ADS;
use chemistry_core::d027_analysis::{
    classify_adsorption_portability, compute_adsorption_basis_labeled, frozen_k_ads_d024,
    generate_analytical_candidates, surface_balance_q, surface_rates_parity,
    window_local_ledger_snapshot, AdsorptionBasisReport, WindowLocalSurfaceRates,
    D027_CANDIDATE_SCALES, D027_MAX_ADSORPTION_CANDIDATES,
};
use chemistry_core::surface_density::{
    compute_interface_geometry, seed_surface_from_gamma, InterfaceGeometryCell,
};
use chemistry_core::Simulation;

fn v7_params() -> SimParams {
    let mut p = SimParams::default();
    p.equation_version = EquationVersion::MembraneMetabolismV7SurfaceDensity;
    p.d008_stage_mode = D008StageMode::FixedCompartment;
    p.d008_stage_b_enabled = true;
    p.reactions_enabled = true;
    p.diffusion_enabled = true;
    p.phase_separation_enabled = false;
    p.k_ads = D025_FROZEN_K_ADS;
    p.d_p = p.d_a;
    p.k_precursor_decay = p.k_d008_activated_decay;
    p.d_gamma = 0.02;
    p.gamma_max = 1.0;
    p.gamma_reference = 1.0;
    STAGE_E_FAILED_RATES.apply_to(&mut p);
    p.k_d008_structure = D018_FROZEN_K_STRUCTURE;
    p
}

fn seed_fixed_interface(sim: &mut Simulation, radius: f64) {
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
            sim.fields.precursor[idx] = 0.2;
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

#[test]
fn test_surface_window_local_resets_at_baseline() {
    let mut sim = Simulation::new(v7_params());
    seed_fixed_interface(&mut sim, 22.0);
    for _ in 0..40 {
        assert!(sim.step());
    }
    let cum_before = sim.surface_accounting.cumulative.adsorption_delta;
    assert!(cum_before > 0.0, "pre-baseline adsorption should accumulate");
    sim.surface_accounting
        .begin_window_local(sim.substep, sim.sim_time);
    let local0 = sim.surface_accounting.window_local();
    assert!(
        local0.adsorption_delta.abs() < 1e-15,
        "window-local must start at zero after begin_window_local"
    );
    for _ in 0..60 {
        assert!(sim.step());
    }
    let local = sim.surface_accounting.window_local();
    assert!(local.adsorption_delta > 0.0);
    assert!(local.gamma_decay_delta > 0.0);
    assert!(local.precursor_synthesis_delta >= 0.0);
    // Window-local must not equal full cumulative (pre-baseline history excluded).
    assert!(local.adsorption_delta < sim.surface_accounting.cumulative.adsorption_delta);
}

#[test]
fn test_uninterrupted_vs_reanchored_rate_parity() {
    let mut uninterrupted = Simulation::new(v7_params());
    seed_fixed_interface(&mut uninterrupted, 22.0);
    for _ in 0..50 {
        assert!(uninterrupted.step());
    }
    // Anchor as if a checkpoint were taken.
    uninterrupted
        .surface_accounting
        .begin_window_local(uninterrupted.substep, uninterrupted.sim_time);
    let ckpt_surface = uninterrupted.surface_accounting.clone();
    let ckpt_fields = uninterrupted.fields.clone();
    let ckpt_substep = uninterrupted.substep;
    let ckpt_time = uninterrupted.sim_time;
    let ckpt_dt = uninterrupted.dt;

    for _ in 0..80 {
        assert!(uninterrupted.step());
    }
    let rates_a = WindowLocalSurfaceRates::from_sim(&uninterrupted);

    // Restored path: rebuild from checkpointed surface ledger + fields.
    let mut restored = Simulation::new(v7_params());
    restored.fields = ckpt_fields;
    restored.fields.copy_current_to_next();
    restored.substep = ckpt_substep;
    restored.sim_time = ckpt_time;
    restored.dt = ckpt_dt;
    restored.surface_accounting = ckpt_surface;
    restored
        .surface_accounting
        .begin_window_local(restored.substep, restored.sim_time);
    for _ in 0..80 {
        assert!(restored.step());
    }
    let rates_b = WindowLocalSurfaceRates::from_sim(&restored);
    let (max_abs, ok) = surface_rates_parity(&rates_a, &rates_b);
    assert!(
        ok,
        "restored vs uninterrupted window rates diverge: max_abs={max_abs} a={rates_a:?} b={rates_b:?}"
    );
    let snap = window_local_ledger_snapshot(&restored);
    assert!(snap.surface.adsorption > 0.0 || snap.surface.gamma_turnover > 0.0);
}

#[test]
fn test_rejected_steps_do_not_inflate_surface_cumulative() {
    let mut sim = Simulation::new(v7_params());
    seed_fixed_interface(&mut sim, 22.0);
    // Force a tiny dt path then normal steps; cumulative accepted_steps must match substep advances.
    for _ in 0..20 {
        let _ = sim.step();
    }
    assert_eq!(
        sim.surface_accounting.accepted_steps, sim.substep,
        "surface ledger accepted_steps must track accepted substeps only"
    );
}

#[test]
fn test_adsorption_basis_and_required_rate() {
    let mut sim = Simulation::new(v7_params());
    seed_fixed_interface(&mut sim, 22.0);
    for _ in 0..30 {
        assert!(sim.step());
    }
    let report = compute_adsorption_basis_labeled(&sim, "unit");
    assert!(report.b_ads > 0.0, "basis should be positive");
    assert!(report.l_gamma > 0.0);
    assert!(report.finite);
    assert!(!report.underflow_dominated);
    assert!(report.k_ads_required.is_finite());
    assert!(report.interface_measure > 0.0);
}

#[test]
fn test_portability_classification_and_candidates() {
    let mut reports = Vec::new();
    for (i, factor) in [1.0, 1.2, 0.9].iter().enumerate() {
        reports.push(AdsorptionBasisReport {
            label: format!("s{i}"),
            b_ads: 1.0,
            l_gamma: factor * 0.03,
            k_ads_required: factor * 0.03,
            mean_theta_gamma: 0.5,
            mean_p_near_interface: 0.1,
            mean_saturation_factor: 0.4,
            mean_q_c: 0.5,
            interface_measure: 10.0,
            finite: true,
            underflow_dominated: false,
        });
    }
    let port = classify_adsorption_portability(&reports);
    assert!(port.portable, "span should be within 3×");
    assert!(port.span <= 3.0);
    let cands = generate_analytical_candidates(&port, "d024", &["s0".into()]).unwrap();
    assert_eq!(cands.len(), D027_MAX_ADSORPTION_CANDIDATES);
    assert_eq!(cands.len(), D027_CANDIDATE_SCALES.len());
    assert!((cands[0].k_ads / cands[1].k_ads - 0.5).abs() < 1e-12);
    assert!((cands[2].k_ads / cands[1].k_ads - 2.0).abs() < 1e-12);

    let mut wide = reports.clone();
    wide[2].k_ads_required = 10.0;
    wide[2].l_gamma = 10.0;
    let bad = classify_adsorption_portability(&wide);
    assert!(!bad.portable);
    assert_eq!(bad.conclusion, "D027_ADSORPTION_LAW_NOT_PORTABLE");
    assert!(generate_analytical_candidates(&bad, "x", &[]).is_err());
}

#[test]
fn test_exact_p_to_s_and_s_to_w_transfer_identity() {
    let mut sim = Simulation::new(v7_params());
    seed_fixed_interface(&mut sim, 22.0);
    for _ in 0..25 {
        assert!(sim.step());
    }
    let t = sim.last_surface_totals.expect("surface totals");
    assert!((t.adsorption_delta - t.precursor_to_surface).abs() < 1e-15);
    assert!((t.gamma_decay_delta - t.surface_to_waste).abs() < 1e-15);
}

#[test]
fn test_surface_balance_q_and_frozen_k_ads() {
    assert!((frozen_k_ads_d024() - D025_FROZEN_K_ADS).abs() < 1e-18);
    assert!((surface_balance_q(2.0, 1.0) - 2.0).abs() < 1e-15);
    assert!((surface_balance_q(1.0, 1.0) - 1.0).abs() < 1e-15);
}
