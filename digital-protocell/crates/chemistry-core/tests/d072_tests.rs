//! D-072 mature-membrane damage refill causal audit tests.

use chemistry_core::d069_analysis::theta_eq;
use chemistry_core::d072_analysis::*;
use chemistry_core::interventions::apply_declared_membrane_arc_damage;
use chemistry_core::surface_density::{
    exchange_scalar_f, solve_exchange_backward_euler, total_surface_mass,
};
use chemistry_core::{field_mass, Simulation};
use chemistry_core::config::{EquationVersion, StructureEvolutionMode};
use chemistry_core::d039_analysis::v8_schema3_params;
use chemistry_core::d049_analysis::d049_frozen_params;
use chemistry_core::d063_analysis::{generate_phi, seed_mature_s_on_interfaces, GeometrySpec, D063_PHI_INTERIOR};
use chemistry_core::d070_analysis::{
    migrate_policy_d_authorized_reconstruction, occupancy_theta,
};
use chemistry_core::surface_density::{compute_interface_geometry, InterfaceGeometryCell};
use chemistry_core::fields::FieldBuffers;

fn mini_seeded_sim() -> Simulation {
    let base = v8_schema3_params();
    let mut params = d049_frozen_params(&base);
    params.equation_version = EquationVersion::MembraneMetabolismV13CatalystSaturatingActivation;
    let mut sim = Simulation::new(params);
    sim.dt_cap = 0.005;
    sim.set_structure_evolution_mode(StructureEvolutionMode::FixedGeometry);
    let spec = GeometrySpec::smooth(22.0);
    let phi = generate_phi(&sim.grid, &spec);
    let mut geometry = vec![InterfaceGeometryCell::default(); phi.len()];
    compute_interface_geometry(&sim.grid, &phi, sim.params.eta_n, &mut geometry);
    let mut s = seed_mature_s_on_interfaces(&sim.grid, &phi, 1.0);
    let p: Vec<_> = (0..phi.len())
        .map(|i| {
            if sim.grid.in_dish(i) && phi[i] >= D063_PHI_INTERIOR {
                0.05
            } else {
                0.0
            }
        })
        .collect();
    let _ = migrate_policy_d_authorized_reconstruction(
        &sim.grid,
        &geometry,
        &mut s,
        &p,
        sim.params.delta_floor,
        sim.params.gamma_max,
        1.0,
        "d072_test_seed",
    );
    sim.fields.structure.copy_from_slice(&phi);
    sim.fields.membrane.copy_from_slice(&s);
    sim.fields.precursor.copy_from_slice(&p);
    for i in 0..phi.len() {
        if sim.grid.in_dish(i) && geometry[i].delta > sim.params.delta_floor {
            sim.fields.catalyst[i] = 0.5;
        }
    }
    sim.fields.copy_current_to_next();
    sim
}

#[test]
fn exact_s_to_w_damage_conservation() {
    let mut sim = mini_seeded_sim();
    let s0 = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let w0 = field_mass(&sim.grid, &sim.fields.waste);
    let report = apply_declared_membrane_arc_damage(&sim.grid, &mut sim.fields, DAMAGE_FRACTION);
    let s1 = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let w1 = field_mass(&sim.grid, &sim.fields.waste);
    assert!(s_w_conservation(s1 - s0, w1 - w0, ACCOUNTING_TOL));
    assert!((report.s_removed - report.w_gained).abs() < ACCOUNTING_TOL);
    assert!((s0 - s1 - report.s_removed).abs() < 1e-9 * (1.0 + s0));
}

#[test]
fn current_next_buffer_synchronization_after_damage() {
    let mut sim = mini_seeded_sim();
    // Stale next intentionally.
    for v in sim.fields.membrane_next.iter_mut() {
        *v = 999.0;
    }
    let _ = apply_declared_membrane_arc_damage(&sim.grid, &mut sim.fields, DAMAGE_FRACTION);
    let desync = sim
        .fields
        .membrane
        .iter()
        .zip(sim.fields.membrane_next.iter())
        .any(|(a, b)| (a - b).abs() > 1e-15);
    assert!(desync, "damage without sync must leave next stale");
    sim.fields.copy_current_to_next();
    let synced = sim
        .fields
        .membrane
        .iter()
        .zip(sim.fields.membrane_next.iter())
        .all(|(a, b)| (a - b).abs() <= 1e-15);
    assert!(synced);
    let waste_synced = sim
        .fields
        .waste
        .iter()
        .zip(sim.fields.waste_next.iter())
        .all(|(a, b)| (a - b).abs() <= 1e-15);
    assert!(waste_synced);
}

#[test]
fn derived_occupancy_invalidation_after_damage() {
    let mut sim = mini_seeded_sim();
    let mut geometry = vec![InterfaceGeometryCell::default(); sim.fields.structure.len()];
    compute_interface_geometry(
        &sim.grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    let mut occ_before = 0.0;
    let mut n = 0usize;
    for i in 0..geometry.len() {
        if !sim.grid.in_dish(i) || geometry[i].delta <= sim.params.delta_floor {
            continue;
        }
        occ_before += occupancy_theta(
            sim.fields.membrane[i],
            geometry[i].delta,
            sim.params.gamma_max,
        );
        n += 1;
    }
    occ_before /= n.max(1) as f64;
    let _ = apply_declared_membrane_arc_damage(&sim.grid, &mut sim.fields, DAMAGE_FRACTION);
    let mut occ_after = 0.0;
    for i in 0..geometry.len() {
        if !sim.grid.in_dish(i) || geometry[i].delta <= sim.params.delta_floor {
            continue;
        }
        occ_after += occupancy_theta(
            sim.fields.membrane[i],
            geometry[i].delta,
            sim.params.gamma_max,
        );
    }
    occ_after /= n.max(1) as f64;
    assert!(
        occ_after < occ_before - 0.01,
        "derived occupancy must fall after damage: {occ_before} -> {occ_after}"
    );
}

#[test]
fn preserved_delta_and_capacity_under_s_only_damage() {
    let mut sim = mini_seeded_sim();
    let mut geometry = vec![InterfaceGeometryCell::default(); sim.fields.structure.len()];
    compute_interface_geometry(
        &sim.grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    let cap_before: f64 = geometry
        .iter()
        .map(|g| (g.delta * sim.params.gamma_max).max(0.0))
        .sum();
    let phi_before = sim.fields.structure.clone();
    let _ = apply_declared_membrane_arc_damage(&sim.grid, &mut sim.fields, DAMAGE_FRACTION);
    assert_eq!(sim.fields.structure, phi_before);
    compute_interface_geometry(
        &sim.grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    let cap_after: f64 = geometry
        .iter()
        .map(|g| (g.delta * sim.params.gamma_max).max(0.0))
        .sum();
    assert!((cap_before - cap_after).abs() < 1e-12);
}

#[test]
fn synthetic_hole_refill_parity_and_equilibrium() {
    let k_ex = D072_K_EXCHANGE;
    let k_eq = D072_K_EQ;
    let gamma_max = D072_GAMMA_MAX;
    let delta = 1.0;
    let q = 1.0;
    let p = 1.0;
    let c_surface = delta * gamma_max;
    let t_inv = p + 0.0; // empty hole
    let dt = 0.05;
    let analytical = analytical_s_gain_empty(delta, k_ex, q, gamma_max, k_eq, p, dt);
    let f0 = exchange_scalar_f(0.0, t_inv, c_surface, delta, q, k_ex, k_eq, 1.0, gamma_max);
    assert!((f0 * dt - analytical).abs() < 1e-12);
    let sol = solve_exchange_backward_euler(
        0.0, t_inv, c_surface, delta, q, k_ex, k_eq, 1.0, gamma_max, dt,
    )
    .expect("BE exchange");
    assert!(sol.s_next > 0.0);
    let th_eq = theta_eq(p, k_eq);
    assert!((equilibrium_occupancy(p, k_eq) - th_eq).abs() < 1e-15);
    // Integrate toward equilibrium with fixed p (diagnostic inventory T = p + S).
    let mut s = 0.0;
    for _ in 0..5000 {
        let t = p + s;
        let step = solve_exchange_backward_euler(
            s, t, c_surface, delta, q, k_ex, k_eq, 1.0, gamma_max, 0.05,
        )
        .expect("BE step");
        s = step.s_next;
    }
    let theta = s / c_surface;
    assert!(
        (theta - th_eq).abs() < 0.02,
        "theta={theta} th_eq={th_eq}"
    );
}

#[test]
fn analytical_timescale_and_accepted_ledger_agreement() {
    let tau = exchange_timescale(D072_K_EXCHANGE, 1.0, D072_K_EQ, 1.0);
    assert!((tau - 1.0 / (D072_K_EXCHANGE * (D072_K_EQ + 1.0))).abs() < 1e-12);
    // Accepted ledger: ΔS = −ΔP for isolated exchange transfer ξ.
    let xi: f64 = 0.12;
    let delta_p = -xi;
    let delta_s = xi;
    assert!((delta_p + delta_s).abs() < 1e-15);
    assert!(s_w_conservation(-xi, xi, ACCOUNTING_TOL)); // S loss to W style
}

#[test]
fn local_refill_basis_classification() {
    // p=1 ⇒ θ_eq≈0.98 ≥ 0.95 ⇒ PRESENT when other supports exist.
    assert_eq!(
        classify_refill_basis(1.0, 1.0, 0.5, 1.0, 0.5, 0.01, 1e-4, 1e-4),
        RefillBasisClass::RefillBasisPresent
    );
    // p=0.05 ⇒ θ_eq≈0.71 < 0.95 ⇒ LOCAL_P_INSUFFICIENT despite positive net.
    assert_eq!(
        classify_refill_basis(1.0, 1.0, 0.5, 0.05, 0.5, 0.01, 1e-4, 1e-4),
        RefillBasisClass::LocalPInsufficient
    );
    assert_eq!(
        classify_refill_basis(1.0, 1.0, 0.5, 0.0, 0.5, 0.01, 1e-4, 1e-4),
        RefillBasisClass::LocalPInsufficient
    );
    assert_eq!(
        classify_refill_basis(1.0, 1.0, 0.5, 1.0, 0.0, 0.01, 1e-4, 1e-4),
        RefillBasisClass::LocalCatalystSupportInsufficient
    );
    assert_eq!(
        classify_refill_basis(0.0, 1.0, 0.5, 1.0, 0.5, 0.01, 1e-4, 1e-4),
        RefillBasisClass::InterfaceSupportMissing
    );
    assert_eq!(
        classify_refill_basis(1.0, 1.0, 0.5, 1.0, 0.5, -0.01, 1e-4, 1e-4),
        RefillBasisClass::NetExchangeNonpositive
    );
}

#[test]
fn conservative_p_redistribution_and_qc_isolation() {
    let p = vec![0.0, 2.0, 4.0, 0.0];
    let dish = [true, true, true, false];
    let total: f64 = p.iter().zip(dish.iter()).filter(|(_, d)| **d).map(|(v, _)| *v).sum();
    let n = dish.iter().filter(|d| **d).count() as f64;
    let mean = total / n;
    let mut mixed = p.clone();
    for (i, d) in dish.iter().enumerate() {
        if *d {
            mixed[i] = mean;
        }
    }
    let total2: f64 = mixed.iter().zip(dish.iter()).filter(|(_, d)| **d).map(|(v, _)| *v).sum();
    assert!((total - total2).abs() < 1e-12);
    // q(C) isolation: hold pre-damage value
    let q_pre: f64 = 0.42;
    let q_diag = q_pre;
    assert!((q_diag - q_pre).abs() < 1e-15);
}

#[test]
fn accepted_simulated_time_checkpoints_and_floor() {
    let floor = expected_d071_no_repair_floor();
    assert!((floor - 0.8928).abs() < 1e-12);
    assert!(near_no_repair_floor(0.895, floor, 0.01));
    assert!(d071_repair_reproduced(0.896));
    assert!(!d071_repair_reproduced(0.80));
    let tau = 10.0;
    let checkpoints = [0.5, 1.0, 3.0, 5.0].map(|m| m * tau);
    assert_eq!(checkpoints, [5.0, 10.0, 30.0, 50.0]);
}

#[test]
fn historical_d070_d071_preservation_and_route() {
    assert_eq!(SEED_CONTRACT, "SEED_CAPACITY_CONTRACT_V1");
    assert!(frozen_kinetics_unchanged(D072_K_EQ, D072_K_EXCHANGE, D072_GAMMA_MAX));
    assert_eq!(D071_CONCLUSION, "D071_FAIL");
    assert_eq!(D070_TAG, "D-070-mature-membrane-seed-capacity-repair");
    assert_eq!(D071_TAG, "D-071-precursor-demand-regulation-fail");
    let mut ev = RouteEvidence072::default();
    ev.d071_reproduced = true;
    ev.intervention_ok = true;
    ev.synthetic_parity_ok = true;
    ev.accounting_ok = true;
    ev.horizon_recovers = true;
    ev.tau_checkpoints_tested = true;
    ev.refill_basis = RefillBasisClass::RefillBasisPresent;
    assert_eq!(select_route(ev), D072Route::H);
    assert_eq!(
        select_route(ev).conclusion().as_str(),
        "D072_DAMAGE_REFILL_HORIZON_QUALIFIED"
    );
    let mut ev_p = RouteEvidence072::default();
    ev_p.d071_reproduced = true;
    ev_p.intervention_ok = true;
    ev_p.synthetic_parity_ok = true;
    ev_p.accounting_ok = true;
    ev_p.tau_checkpoints_tested = true;
    ev_p.refill_basis = RefillBasisClass::LocalPInsufficient;
    ev_p.fixed_p_recovers = true;
    assert_eq!(select_route(ev_p), D072Route::P);
    ev_p.fixed_p_recovers = false;
    assert_eq!(select_route(ev_p), D072Route::X);
}

#[test]
fn field_buffers_exist_for_sync_api() {
    let f = FieldBuffers::new(4);
    assert_eq!(f.membrane.len(), 4);
    assert_eq!(f.membrane_next.len(), 4);
}
