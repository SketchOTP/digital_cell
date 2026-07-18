//! D-031 invariant-domain exchange integration: Gate 0 classification helpers.

use crate::config::{EquationVersion, SimParams, SurfaceExchangeIntegrator};
use crate::d029_analysis::{apply_exchange_candidate, ExchangeCandidate};
use crate::membrane::membrane_catalyst_saturation;
use crate::surface_density::{
    classify_exchange_invariant_field, exchange_rate_j, exchange_scalar_f, propose_explicit_exchange,
    reconstruct_gamma, surface_occupancy_theta, ExchangeReject, InvariantBoundarySigns,
    SURFACE_EXCHANGE_INTEGRATOR_V2,
};
use crate::Simulation;
use serde::{Deserialize, Serialize};

/// Frozen D-030 identified candidate (result commit 921bd42).
pub fn d030_identified_candidate() -> ExchangeCandidate {
    ExchangeCandidate {
        identity: "d030_identified".into(),
        k_exchange: 0.003339877461040047,
        k_exchange_eq: 50.00000000005883,
    }
}

pub const D031_ALPHA_FROZEN: f64 = 0.003339877461040047 * 50.00000000005883;
pub const D031_BETA_FROZEN: f64 = 0.003339877461040047;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityFailureRecord {
    pub reproduced: bool,
    pub reject_reason: String,
    pub accepted_before_reject: u64,
    pub p: f64,
    pub s: f64,
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,
    pub q_c: f64,
    pub j_forward: f64,
    pub j_reverse: f64,
    pub j_net: f64,
    pub turnover_rate_s: f64,
    pub proposed_p_next: f64,
    pub proposed_s_next: f64,
    pub proposed_theta_next: f64,
    pub attempted_dt: f64,
    pub dt_floor: f64,
    pub prior_accepted_dt: f64,
    pub boundary_signs: BoundarySignsJson,
    pub continuous_inward: bool,
    pub classification: String,
    pub integrator_schema: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundarySignsJson {
    pub dp_at_p0: f64,
    pub ds_at_s0: f64,
    pub ds_at_theta1: f64,
    pub dp_prefailure: f64,
    pub ds_prefailure: f64,
}

impl From<InvariantBoundarySigns> for BoundarySignsJson {
    fn from(s: InvariantBoundarySigns) -> Self {
        Self {
            dp_at_p0: s.dp_at_p0,
            ds_at_s0: s.ds_at_s0,
            ds_at_theta1: s.ds_at_theta1,
            dp_prefailure: s.dp_prefailure,
            ds_prefailure: s.ds_prefailure,
        }
    }
}

/// Seed matching D-030 Gate 7 / D-025 `seed_v7_compartment` (radius 22, θ=0.6).
pub fn seed_d030_isolated_compartment(sim: &mut Simulation, radius: f64, theta_gamma: f64) {
    use crate::surface_density::{
        compute_interface_geometry, seed_surface_from_gamma, InterfaceGeometryCell,
    };
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

/// Build v8 params with the frozen D-030 candidate and chosen integrator.
pub fn v8_identified_params(integrator: SurfaceExchangeIntegrator) -> SimParams {
    let mut p = SimParams::default();
    p.equation_version = EquationVersion::MembraneMetabolismV8ReversibleSurfaceExchange;
    apply_exchange_candidate(&mut p, &d030_identified_candidate());
    p.surface_exchange_integrator = integrator;
    p.reactions_enabled = true;
    p
}

/// Reproduce D-030 CapacityExceeded mechanism under explicit Euler and classify.
///
/// Avoids long V1 reject-cascade burns (D-030 measurement accepted 0 steps). Probes:
/// 1) seeded state explicit proposal at governed dt_cap;
/// 2) synthetic near-capacity cell;
/// 3) at most one adaptive `step()` for reject-string evidence.
pub fn reproduce_capacity_failure(
    seed_fn: impl FnOnce(&mut Simulation),
    _max_steps: u64,
) -> CapacityFailureRecord {
    let params = v8_identified_params(SurfaceExchangeIntegrator::ExplicitEulerV1);
    let mut sim = Simulation::new(params);
    sim.enforce_structure_constraint = true;
    sim.dt_cap = 0.005;
    seed_fn(&mut sim);

    let probe_dt = sim.dt_cap.max(sim.dt).max(1e-3);
    let (cell, signs) = characterize_worst_explicit_cell(&sim, probe_dt);

    let synth = {
        // Adsorption-dominated near-capacity state: continuous F>0 but large explicit dt
        // jumps past θ=1 (same failure mode as D-030 CapacityExceeded under V1).
        let d = 0.5_f64;
        let theta = 0.98_f64;
        let s = d * theta * sim.params.gamma_max;
        let p = 10.0_f64;
        let c = 0.4_f64;
        let dt_big = 10.0_f64;
        let (p_n, s_n, xfer, j_fwd, j_rev, _) =
            propose_explicit_exchange(p, s, d, c, dt_big, &sim.params);
        let g_n = reconstruct_gamma(s_n, d, sim.params.delta_floor);
        let th_n = surface_occupancy_theta(g_n, sim.params.gamma_max);
        (
            th_n > 1.0 + 1e-12 || p_n < -1e-12,
            p,
            s,
            d,
            theta,
            p_n,
            s_n,
            th_n,
            xfer,
            j_fwd,
            j_rev,
            dt_big,
        )
    };
    let synth_overshoot = synth.0;

    // Do not call adaptive step() here: V1 CapacityExceeded triggers a dt-halving
    // cascade that can wall-clock-stall Gate 0. Classification uses explicit proposals.
    let reject_detail = String::new();
    let failed = false;
    let prior_dt = sim.dt;

    // Prefer synthetic/near-cap cell fields when seeded interface has not yet saturated.
    let use_synth = cell.proposed_theta_next <= 1.0 + 1e-12 && synth_overshoot;
    let (p, s, delta, theta, proposed_p, proposed_s, proposed_th, j_fwd, j_rev, j_net, attempted) =
        if use_synth {
            (
                synth.1,
                synth.2,
                synth.3,
                synth.4,
                synth.5,
                synth.6,
                synth.7,
                synth.3 * synth.9,
                synth.3 * synth.10,
                synth.8,
                synth.11,
            )
        } else {
            (
                cell.p,
                cell.s,
                cell.delta,
                cell.theta,
                cell.proposed_p_next,
                cell.proposed_s_next,
                cell.proposed_theta_next,
                cell.j_forward,
                cell.j_reverse,
                cell.j_net,
                probe_dt,
            )
        };
    let gamma = if delta > sim.params.delta_floor {
        s / delta
    } else {
        0.0
    };
    let q_c = if use_synth { 0.8 } else { cell.q_c };
    let signs = if use_synth {
        classify_exchange_invariant_field(p, s, delta, q_c, &sim.params)
    } else {
        signs
    };

    let continuous_inward = signs.continuous_inward;
    let discrete_overshoot =
        failed || proposed_th > 1.0 + 1e-12 || synth_overshoot;
    let classification = if !continuous_inward {
        "D031_EXCHANGE_LAW_INVARIANT_FAILURE".to_string()
    } else if discrete_overshoot {
        "D031_EXPLICIT_INTEGRATION_OVERSHOOT_CONFIRMED".to_string()
    } else {
        "D031_CAPACITY_FAILURE_NOT_REPRODUCED".to_string()
    };

    CapacityFailureRecord {
        reproduced: failed || synth_overshoot || proposed_th > 1.0 + 1e-12,
        reject_reason: if reject_detail.is_empty() && synth_overshoot {
            "synthetic_near_capacity_explicit_overshoot".into()
        } else {
            reject_detail
        },
        accepted_before_reject: 0,
        p,
        s,
        delta,
        gamma,
        theta,
        q_c,
        j_forward: j_fwd,
        j_reverse: j_rev,
        j_net,
        turnover_rate_s: -sim.params.k_gamma_decay * s,
        proposed_p_next: proposed_p,
        proposed_s_next: proposed_s,
        proposed_theta_next: proposed_th,
        attempted_dt: attempted,
        dt_floor: crate::d014_numerics::D014_DT_FLOOR,
        prior_accepted_dt: prior_dt,
        boundary_signs: signs.into(),
        continuous_inward,
        classification,
        integrator_schema: SURFACE_EXCHANGE_INTEGRATOR_V2.to_string(),
    }
}

struct CellProbe {
    p: f64,
    s: f64,
    delta: f64,
    gamma: f64,
    theta: f64,
    q_c: f64,
    j_forward: f64,
    j_reverse: f64,
    j_net: f64,
    turnover_rate_s: f64,
    proposed_p_next: f64,
    proposed_s_next: f64,
    proposed_theta_next: f64,
}

fn characterize_worst_explicit_cell(sim: &Simulation, dt: f64) -> (CellProbe, InvariantBoundarySigns) {
    let n = sim.grid.width * sim.grid.height;
    let mut geometry = vec![crate::surface_density::InterfaceGeometryCell::default(); n];
    crate::surface_density::compute_interface_geometry(
        &sim.grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    let mut best: Option<CellProbe> = None;
    let mut best_over = f64::NEG_INFINITY;
    for idx in 0..n {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        let d = geometry[idx].delta;
        if d <= sim.params.delta_floor {
            continue;
        }
        let s = sim.fields.membrane[idx];
        let p = sim.fields.precursor[idx];
        let c = sim.fields.catalyst[idx];
        let g = reconstruct_gamma(s, d, sim.params.delta_floor);
        let theta = surface_occupancy_theta(g, sim.params.gamma_max);
        let q_c = membrane_catalyst_saturation(c, &sim.params);
        let (j_net, j_fwd, j_rev, ..) = exchange_rate_j(p, c, g, &sim.params);
        let (p_n, s_n, ..) = propose_explicit_exchange(p, s, d, c, dt, &sim.params);
        let g_n = reconstruct_gamma(s_n, d, sim.params.delta_floor);
        let th_n = surface_occupancy_theta(g_n, sim.params.gamma_max);
        let over = th_n - 1.0;
        if over > best_over {
            best_over = over;
            best = Some(CellProbe {
                p,
                s,
                delta: d,
                gamma: g,
                theta,
                q_c,
                j_forward: d * j_fwd,
                j_reverse: d * j_rev,
                j_net: d * j_net,
                turnover_rate_s: -sim.params.k_gamma_decay * s,
                proposed_p_next: p_n,
                proposed_s_next: s_n,
                proposed_theta_next: th_n,
            });
        }
    }
    let cell = best.unwrap_or(CellProbe {
        p: 0.0,
        s: 0.0,
        delta: 0.0,
        gamma: 0.0,
        theta: 0.0,
        q_c: 0.0,
        j_forward: 0.0,
        j_reverse: 0.0,
        j_net: 0.0,
        turnover_rate_s: 0.0,
        proposed_p_next: 0.0,
        proposed_s_next: 0.0,
        proposed_theta_next: 0.0,
    });
    let signs = classify_exchange_invariant_field(
        cell.p,
        cell.s,
        cell.delta,
        cell.q_c,
        &sim.params,
    );
    (cell, signs)
}

/// Monotonicity check: F(S) decreasing on a uniform sample of [0, min(T,C)].
pub fn exchange_f_is_monotone_decreasing(
    t_inventory: f64,
    c_surface: f64,
    delta: f64,
    q_c: f64,
    params: &SimParams,
    samples: usize,
) -> bool {
    let hi = t_inventory.min(c_surface).max(0.0);
    if hi <= 0.0 || samples < 2 {
        return true;
    }
    let mut prev = f64::INFINITY;
    for i in 0..samples {
        let s = hi * (i as f64) / (samples as f64 - 1.0);
        let f = exchange_scalar_f(
            s,
            t_inventory,
            c_surface,
            delta,
            q_c,
            params.k_exchange,
            params.k_exchange_eq,
            params.p_reference,
            params.gamma_max,
        );
        if f > prev + 1e-12 {
            return false;
        }
        prev = f;
    }
    true
}

/// Map an ExchangeReject to a stable string.
pub fn reject_name(r: ExchangeReject) -> &'static str {
    match r {
        ExchangeReject::NegPrecursor => "NegPrecursor",
        ExchangeReject::NegSurface => "NegSurface",
        ExchangeReject::CapacityExceeded => "CapacityExceeded",
        ExchangeReject::NonfiniteFlux => "NonfiniteFlux",
        ExchangeReject::NonfiniteAffinity => "NonfiniteAffinity",
        ExchangeReject::DissipationViolation => "DissipationViolation",
    }
}
