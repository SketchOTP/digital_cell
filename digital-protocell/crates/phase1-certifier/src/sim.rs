//! Independent simulation driver — uses mesh kernels, never d086_analysis gates.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_REBOND_DIST, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, remesh, MechParams};
use chemistry_core::mesh_reactions::{
    pulse_tracers, reactions_step_with_reserve_mode, tracer_catalyst_fraction,
    tracer_membrane_fraction, tracer_structural_fraction, try_local_rebond, MeshChemistrySchema,
    ReactionLedger, ReactionParams, ReserveDiagnosticMode,
};
use chemistry_core::mesh_transport::{transport_step, TransportParams};
use chemistry_core::metabolic_reserve::{stamp_reserve_equation, ReserveParams};
use serde::{Deserialize, Serialize};

use crate::frozen::{frozen_reactions, frozen_transport, FROZEN_CENTER, FROZEN_SCHEMA};
use crate::metrics::{replacement_report, retention_report, ReplacementReport, RetentionReport};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepLedger {
    pub reactions: ReactionLedger,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccumLedger {
    pub m_produced: f64,
    pub m_to_w: f64,
    pub bind_extent: f64,
    pub unbind_extent: f64,
    pub c_produced: f64,
    pub c_turned: f64,
    pub l_produced: f64,
    pub a_produced: f64,
    #[serde(default)]
    pub reserve_a_to_r: f64,
    #[serde(default)]
    pub reserve_r_to_a: f64,
    #[serde(default)]
    pub reserve_r_to_w: f64,
    #[serde(default)]
    pub reserve_rejected_steps: u64,
    #[serde(default)]
    pub a_to_c: f64,
    #[serde(default)]
    pub a_to_r_before_later_demand: f64,
    #[serde(default)]
    pub structural_demand_a: f64,
    #[serde(default)]
    pub membrane_demand_a: f64,
    #[serde(default)]
    pub reserve_closure_residual: f64,
    #[serde(default)]
    pub a_stock_entering: f64,
    #[serde(default)]
    pub r_stock_entering: f64,
    #[serde(default)]
    pub a_after_activation: f64,
    #[serde(default)]
    pub r_before_release: f64,
    #[serde(default)]
    pub a_before_catalyst_production: f64,
    #[serde(default)]
    pub a_before_final_storage: f64,
    #[serde(default)]
    pub a_to_m: f64,
    #[serde(default)]
    pub a_to_l: f64,
    #[serde(default)]
    pub reserve_store_potential: f64,
    #[serde(default)]
    pub new_a_surplus: f64,
    #[serde(default)]
    pub a_to_r_same_step_new_a: f64,
    #[serde(default)]
    pub a_to_r_pre_existing_a: f64,
    #[serde(default)]
    pub diagnostic_liquid_r_used: f64,
    #[serde(default)]
    pub diagnostic_liquid_r_available: f64,
    #[serde(default)]
    pub diagnostic_liquid_r_used_for_m: f64,
    #[serde(default)]
    pub diagnostic_liquid_r_used_for_l: f64,
    #[serde(default)]
    pub net_activation_equivalent: f64,
    #[serde(default)]
    pub activation_equivalent_closure_residual: f64,
}

impl AccumLedger {
    pub fn absorb(&mut self, r: &ReactionLedger) {
        self.m_produced += r.m_produced;
        self.m_to_w += r.m_to_w;
        self.bind_extent += r.bind_extent;
        self.unbind_extent += r.unbind_extent;
        self.c_produced += r.c_produced;
        self.c_turned += r.c_turned;
        self.l_produced += r.l_produced;
        self.a_produced += r.a_produced;
        self.reserve_a_to_r += r.reserve.a_to_r;
        self.reserve_r_to_a += r.reserve.r_to_a;
        self.reserve_r_to_w += r.reserve.r_to_w;
        self.reserve_rejected_steps += r.reserve.rejected_steps;
        self.a_to_c += r.a_to_c;
        self.a_to_r_before_later_demand += r.a_to_r_before_later_demand;
        self.structural_demand_a += r.structural_demand_a;
        self.membrane_demand_a += r.membrane_demand_a;
        self.reserve_closure_residual += r.reserve_closure_residual;
        self.a_stock_entering += r.a_stock_entering;
        self.r_stock_entering += r.r_stock_entering;
        self.a_after_activation += r.a_after_activation;
        self.r_before_release += r.r_before_release;
        self.a_before_catalyst_production += r.a_before_catalyst_production;
        self.a_before_final_storage += r.a_before_final_storage;
        self.a_to_m += r.a_to_m;
        self.a_to_l += r.a_to_l;
        self.reserve_store_potential += r.reserve_store_potential;
        self.new_a_surplus += r.new_a_surplus;
        self.a_to_r_same_step_new_a += r.a_to_r_same_step_new_a;
        self.a_to_r_pre_existing_a += r.a_to_r_pre_existing_a;
        self.diagnostic_liquid_r_used += r.diagnostic_liquid_r_used;
        self.diagnostic_liquid_r_available += r.diagnostic_liquid_r_available;
        self.diagnostic_liquid_r_used_for_m += r.diagnostic_liquid_r_used_for_m;
        self.diagnostic_liquid_r_used_for_l += r.diagnostic_liquid_r_used_for_l;
        self.net_activation_equivalent += r.net_activation_equivalent;
        self.activation_equivalent_closure_residual += r.activation_equivalent_closure_residual;
    }
}

pub fn seed_mesh(radius: f64, seed: u64) -> MaterialMesh {
    let n = 24 + ((seed % 3) as usize);
    let interior = LumpedChem {
        c: 0.8,
        a: 0.5,
        n: 0.4,
        f: 0.4,
        w: 0.1,
        tracer_c: 0.0,
        c_h: 0.0,
        c_b: 0.0,
        r: 0.0,
        u_h: 0.0,
        u_b: 0.0,
        k_h: 0.0,
        k_b: 0.0,
        q_k: 0.0,
        q_e: 0.0,
        k_a: 0.0,
        k_r: 0.0,
        k_node_b: 0.0,
    };
    let exterior = LumpedChem {
        c: 0.0,
        a: 0.0,
        n: 1.0,
        f: 1.0,
        w: 0.0,
        tracer_c: 0.0,
        c_h: 0.0,
        c_b: 0.0,
        r: 0.0,
        u_h: 0.0,
        u_b: 0.0,
        k_h: 0.0,
        k_b: 0.0,
        q_k: 0.0,
        q_e: 0.0,
        k_a: 0.0,
        k_r: 0.0,
        k_node_b: 0.0,
    };
    let mut mesh = MaterialMesh::seed_regular(
        n,
        radius,
        40.0,
        40.0,
        DEFAULT_RHO_S,
        0.7,
        interior,
        exterior,
        5.0,
    );
    if reserve_enabled() {
        stamp_reserve_equation(&mut mesh);
    }
    if conservative_v2_enabled() {
        if geometry_conservative_enabled() {
            mesh.stamp_geometry_conservative_schema();
        } else {
            mesh.stamp_conservative_schema();
        }
    }
    mesh
}

/// Select the physical/material contract for production and the observer-only R9 matrix.
///
/// The R9-R3 selector is deliberately independent from reserve selection. The
/// older R9-R2 switch remains supported so historical workflows retain their
/// exact behavior when explicitly selecting HistoricalV1 or ConservativeV2.
/// The ordinary M0 production default is ConservativeV2.
pub fn conservative_v2_enabled() -> bool {
    !matches!(selected_mesh_schema(), MeshChemistrySchema::HistoricalV1)
}

/// Opt-in experimental material contract for R6-R2.  The chemistry schema
/// selector remains independent so ConservativeV2 and its historical evidence
/// are unchanged unless this explicit diagnostic switch is present.
pub fn geometry_conservative_enabled() -> bool {
    matches!(
        std::env::var("DCDEV020M1R6R2_GEOMETRY_CONTRACT")
            .ok()
            .as_deref(),
        Some("1") | Some("on") | Some("ON") | Some("true")
    ) && conservative_v2_enabled()
}

pub fn selected_mesh_schema() -> MeshChemistrySchema {
    match std::env::var("DCDEV020R9R3_CONTRACT").ok().as_deref() {
        Some("ConservativeV3") => MeshChemistrySchema::ConservativeV3,
        Some("ConservativeV2") => MeshChemistrySchema::ConservativeV2,
        Some("HistoricalV1") => MeshChemistrySchema::HistoricalV1,
        Some(_) => MeshChemistrySchema::HistoricalV1,
        None => match std::env::var("DCDEV020R9R2_V2").ok().as_deref() {
            Some("0") => MeshChemistrySchema::HistoricalV1,
            _ => MeshChemistrySchema::ConservativeV2,
        },
    }
}

/// Select D-091 reserve physiology independently from the material contract.
///
/// Reserve is not part of the ordinary M0 production default. Diagnostic and
/// historical callers must opt in explicitly with the R9-R3 reserve selector.
pub fn reserve_enabled() -> bool {
    match std::env::var("DCDEV020R9R3_RESERVE").ok().as_deref() {
        Some("1") | Some("on") | Some("ON") | Some("true") => true,
        Some("0") | Some("off") | Some("OFF") | Some("false") => false,
        Some(_) => false,
        None => false,
    }
}

pub fn contract_label() -> &'static str {
    match selected_mesh_schema() {
        MeshChemistrySchema::HistoricalV1 => "HistoricalV1",
        MeshChemistrySchema::ConservativeV2 => "ConservativeV2",
        MeshChemistrySchema::ConservativeV3 => "ConservativeV3",
    }
}

pub fn equation_lineage() -> &'static str {
    if reserve_enabled() {
        chemistry_core::metabolic_reserve::EQUATION_VERSION_METABOLIC_RESERVE
    } else {
        FROZEN_SCHEMA
    }
}

pub fn reaction_params_for(mesh: &MaterialMesh) -> ReactionParams {
    let mut params = match selected_mesh_schema() {
        MeshChemistrySchema::HistoricalV1 => frozen_reactions(),
        MeshChemistrySchema::ConservativeV2 => ReactionParams::conservative_v2(),
        MeshChemistrySchema::ConservativeV3 => ReactionParams::conservative_v3(),
    };
    if reserve_enabled() {
        params.reserve = ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, mesh.area());
    }
    params
}

/// Lawful perturbation of a seeded mesh (deterministic path for Gate 3).
pub fn perturb_mesh(mesh: &mut MaterialMesh, kind: &str, mag: f64) {
    match kind {
        "vertex_noise" => {
            for (i, v) in mesh.vertices.iter_mut().enumerate() {
                let s = ((i as f64 + 1.0) * 12.9898).sin() * 43758.5453;
                let f = s - s.floor();
                v[0] += mag * (f - 0.5);
                v[1] += mag * ((f * 7.13).fract() - 0.5);
            }
        }
        "c_noise" => {
            mesh.interior.c = (mesh.interior.c * (1.0 + mag)).max(0.0);
        }
        "a_noise" => {
            mesh.interior.a = (mesh.interior.a * (1.0 + mag)).max(0.0);
        }
        "l_noise" => {
            mesh.free_l = (mesh.free_l * (1.0 + mag)).max(0.0);
        }
        "env_nf" => {
            mesh.exterior.n = (mesh.exterior.n * (1.0 + mag)).max(0.0);
            mesh.exterior.f = (mesh.exterior.f * (1.0 + mag)).max(0.0);
        }
        "rotate" => {
            let c = mesh.centroid();
            let ang = mag;
            let (s, co) = (ang.sin(), ang.cos());
            for v in &mut mesh.vertices {
                let x = v[0] - c[0];
                let y = v[1] - c[1];
                v[0] = c[0] + co * x - s * y;
                v[1] = c[1] + s * x + co * y;
            }
        }
        "translate" => {
            for v in &mut mesh.vertices {
                v[0] += mag;
                v[1] += mag * 0.5;
            }
        }
        _ => {}
    }
}

pub fn dish_contact(mesh: &MaterialMesh) -> bool {
    mesh.vertices
        .iter()
        .any(|p| p[0] < 2.0 || p[1] < 2.0 || p[0] > 78.0 || p[1] > 78.0)
}

pub fn coupled_step(
    mesh: &mut MaterialMesh,
    mech: &MechParams,
    react: &ReactionParams,
    transport: &TransportParams,
    build: bool,
    metab: bool,
) -> ReactionLedger {
    coupled_step_with_reserve_mode(
        mesh,
        mech,
        react,
        transport,
        build,
        metab,
        reserve_diagnostic_mode_from_env(),
    )
}

/// Select only the bounded R9 observer mode when explicitly requested. With
/// no selector, the established production trajectory remains exactly Full.
pub fn reserve_diagnostic_mode_from_env() -> ReserveDiagnosticMode {
    match std::env::var("DCDEV020R9R5_MODE").ok().as_deref() {
        Some("SURPLUS_ONLY_STORE") => ReserveDiagnosticMode::SurplusOnlyStore,
        Some("LIQUID_RESERVE_UB") => ReserveDiagnosticMode::LiquidReserveUpperBound,
        Some("LIQUID_RESERVE_PRETHROTTLE_UB") => {
            ReserveDiagnosticMode::LiquidReservePreThrottleUpperBound
        }
        Some("SURPLUS_ONLY_STORE_LIQUID_RESERVE_UB") => {
            ReserveDiagnosticMode::SurplusOnlyStoreLiquidReserveUpperBound
        }
        Some("SURPLUS_ONLY_STORE_LIQUID_RESERVE_PRETHROTTLE_UB") => {
            ReserveDiagnosticMode::SurplusOnlyStoreLiquidReservePreThrottleUpperBound
        }
        Some("MOBILIZE_FIRST_STORE_LAST") => ReserveDiagnosticMode::MobilizeFirstStoreLast,
        _ => ReserveDiagnosticMode::Full,
    }
}

pub fn coupled_step_with_reserve_mode(
    mesh: &mut MaterialMesh,
    mech: &MechParams,
    react: &ReactionParams,
    transport: &TransportParams,
    build: bool,
    metab: bool,
    reserve_mode: ReserveDiagnosticMode,
) -> ReactionLedger {
    let _ = transport_step(mesh, transport, mech.dt);
    let led = reactions_step_with_reserve_mode(mesh, react, mech.dt, build, metab, reserve_mode);
    mechanics_step(mesh, mech);
    remesh(mesh);
    try_local_rebond(mesh, DEFAULT_REBOND_DIST);
    led
}

pub fn run_coupled(mesh: &mut MaterialMesh, steps: usize, build: bool, metab: bool) -> AccumLedger {
    let mech = FROZEN_CENTER;
    let react = reaction_params_for(mesh);
    let transport = frozen_transport();
    let mut acc = AccumLedger::default();
    for _ in 0..steps {
        if !mesh.can_advance_physics() {
            break;
        }
        let led = coupled_step(mesh, &mech, &react, &transport, build, metab);
        acc.absorb(&led);
    }
    acc
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnoverAudit {
    pub retention_c: RetentionReport,
    pub retention_a: RetentionReport,
    pub structural: ReplacementReport,
    pub membrane: ReplacementReport,
    pub catalyst: ReplacementReport,
    pub catalyst_label_amount_initial: f64,
    pub catalyst_label_amount_final: f64,
    pub catalyst_legacy_concentration_ratio: f64,
    pub d086_tracer_interpretation: Vec<String>,
    pub dual_requirement_pass: bool,
    pub d086_pool_m: f64,
    pub d086_pool_b: f64,
    pub d086_pool_c: f64,
    pub d086_soft_pass: bool,
}

/// Convert the catalyst pulse tracer concentration to the labeled material
/// amount required by the D-087 replacement metric.
pub fn catalyst_label_amount(tracer_concentration: f64, area: f64) -> f64 {
    tracer_concentration * area
}

pub fn audit_turnover(steps: usize) -> TurnoverAudit {
    let mut mesh = seed_mesh(14.0, 2);
    pulse_tracers(&mut mesh, 1.0);
    let label_m0: f64 = mesh.edges.iter().map(|e| e.tracer_m).sum();
    let label_b0: f64 = mesh.edges.iter().map(|e| e.tracer_b).sum();
    let label_c0 = mesh.interior.tracer_c;
    let catalyst_label_amount_initial = catalyst_label_amount(label_c0, mesh.area());
    let mut series_c = vec![mesh.interior.c];
    let mut series_a = vec![mesh.interior.a];
    let mut mass_m = Vec::new();
    let mut mass_b = Vec::new();
    let mut mass_c = Vec::new();
    let mut acc = AccumLedger::default();
    let mech = FROZEN_CENTER;
    let react = reaction_params_for(&mesh);
    let transport = frozen_transport();
    for _ in 0..steps {
        if !mesh.can_advance_physics() {
            break;
        }
        let led = coupled_step(&mut mesh, &mech, &react, &transport, true, true);
        acc.absorb(&led);
        series_c.push(mesh.interior.c);
        series_a.push(mesh.interior.a);
        mass_m.push(mesh.total_structural_mass());
        mass_b.push(mesh.total_bound_membrane());
        mass_c.push(mesh.interior.c * mesh.area().max(1e-6));
    }
    let mean = |v: &[f64]| {
        if v.is_empty() {
            0.0
        } else {
            v.iter().sum::<f64>() / v.len() as f64
        }
    };
    let label_m_t: f64 = mesh.edges.iter().map(|e| e.tracer_m).sum();
    let label_b_t: f64 = mesh.edges.iter().map(|e| e.tracer_b).sum();
    let label_c_t = mesh.interior.tracer_c;
    let catalyst_label_amount_final = catalyst_label_amount(label_c_t, mesh.area());
    let catalyst_legacy_concentration_ratio = if label_c0 <= 1e-15 {
        0.0
    } else {
        (label_c_t / label_c0).clamp(0.0, 1.0)
    };
    // Gross replacement: newly incorporated material (production / bind / c_prod).
    let structural = replacement_report(
        "m",
        mean(&mass_m),
        acc.m_produced,
        label_m0,
        label_m_t,
        mesh.total_structural_mass(),
    );
    let membrane = replacement_report(
        "b",
        mean(&mass_b),
        acc.bind_extent,
        label_b0,
        label_b_t,
        mesh.total_bound_membrane(),
    );
    let catalyst = replacement_report(
        "C",
        mean(&mass_c),
        acc.c_produced,
        catalyst_label_amount_initial,
        catalyst_label_amount_final,
        (mesh.interior.c * mesh.area()).max(1e-15),
    );
    let dual = structural.r_x_ok
        && structural.f_label_ok
        && membrane.r_x_ok
        && membrane.f_label_ok
        && catalyst.r_x_ok
        && catalyst.f_label_ok;
    let d086_pool_m = tracer_structural_fraction(&mesh);
    let d086_pool_b = tracer_membrane_fraction(&mesh);
    let d086_pool_c = tracer_catalyst_fraction(&mesh);
    TurnoverAudit {
        retention_c: retention_report("C", &series_c, acc.c_produced),
        retention_a: retention_report("A", &series_a, acc.a_produced),
        structural,
        membrane,
        catalyst,
        catalyst_label_amount_initial,
        catalyst_label_amount_final,
        catalyst_legacy_concentration_ratio,
        d086_tracer_interpretation: vec![
            crate::metrics::interpret_d086_tracer("m", 0.35),
            crate::metrics::interpret_d086_tracer("b", 0.00),
            crate::metrics::interpret_d086_tracer("c", 0.23),
            format!(
                "independent_recompute_f_pool: m={d086_pool_m:.3} b={d086_pool_b:.3} c={d086_pool_c:.3} (D-086 reported ~0.35/0.00/0.23)"
            ),
        ],
        dual_requirement_pass: dual,
        d086_pool_m,
        d086_pool_b,
        d086_pool_c,
        d086_soft_pass: d086_pool_m < 0.55 && d086_pool_b < 0.70 && d086_pool_c < 0.70,
    }
}

pub fn fingerprint(mesh: &MaterialMesh) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    mesh.n().hash(&mut h);
    for v in &mesh.vertices {
        ((v[0] * 1e6).round() as i64).hash(&mut h);
        ((v[1] * 1e6).round() as i64).hash(&mut h);
    }
    ((mesh.total_structural_mass() * 1e6).round() as i64).hash(&mut h);
    ((mesh.interior.c * 1e6).round() as i64).hash(&mut h);
    ((mesh.interior.a * 1e6).round() as i64).hash(&mut h);
    mesh.alive.hash(&mut h);
    h.finish()
}

pub fn pass_basin_row(mesh: &MaterialMesh, a0: f64, c0: f64, aa0: f64) -> bool {
    let a1 = mesh.area();
    let c_ret = if c0 <= 1e-15 {
        1.0
    } else {
        mesh.interior.c / c0
    };
    let a_ret = if aa0 <= 1e-15 {
        1.0
    } else {
        mesh.interior.a / aa0
    };
    mesh.alive
        && mesh.closed_intact()
        && !dish_contact(mesh)
        && a1 > 0.2 * a0
        && a1 < 5.0 * a0
        && c_ret >= crate::metrics::RETENTION_MIN
        && a_ret >= crate::metrics::RETENTION_MIN
}

pub use chemistry_core::mesh_reactions::{
    apply_local_rupture, apply_membrane_damage, apply_structural_damage, evaluate_death,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_reserve_observer_mode_has_exact_default_trajectory_parity() {
        std::env::set_var("DCDEV020R9R3_CONTRACT", "ConservativeV2");
        std::env::set_var("DCDEV020R9R3_RESERVE", "1");
        let mut ordinary = seed_mesh(14.0, 2);
        let mut observer = ordinary.clone();
        let react = reaction_params_for(&ordinary);
        let transport = frozen_transport();
        for _ in 0..256 {
            let a = coupled_step(
                &mut ordinary,
                &FROZEN_CENTER,
                &react,
                &transport,
                true,
                true,
            );
            let b = coupled_step_with_reserve_mode(
                &mut observer,
                &FROZEN_CENTER,
                &react,
                &transport,
                true,
                true,
                ReserveDiagnosticMode::Full,
            );
            assert_eq!(
                serde_json::to_value(a).unwrap(),
                serde_json::to_value(b).unwrap()
            );
            assert_eq!(fingerprint(&ordinary), fingerprint(&observer));
        }
    }

    #[test]
    fn r9r5_surplus_storage_is_capped_without_rejected_reserve_steps() {
        std::env::set_var("DCDEV020R9R3_CONTRACT", "ConservativeV2");
        std::env::set_var("DCDEV020R9R3_RESERVE", "1");
        let mut mesh = seed_mesh(14.0, 2);
        let react = reaction_params_for(&mesh);
        let transport = frozen_transport();
        for _ in 0..256 {
            let led = coupled_step_with_reserve_mode(
                &mut mesh,
                &FROZEN_CENTER,
                &react,
                &transport,
                true,
                true,
                ReserveDiagnosticMode::SurplusOnlyStore,
            );
            assert!(led.reserve.a_to_r <= led.reserve_store_potential + 1e-8);
            assert!(led.reserve.a_to_r <= led.new_a_surplus + 1e-8);
            assert_eq!(led.reserve.rejected_steps, 0);
            assert!(led.activation_equivalent_closure_residual <= 1e-6);
        }
    }

    #[test]
    fn r9r5_liquid_upper_bound_has_separate_nonnegative_accounting() {
        std::env::set_var("DCDEV020R9R3_CONTRACT", "ConservativeV2");
        std::env::set_var("DCDEV020R9R3_RESERVE", "1");
        let mut mesh = seed_mesh(14.0, 2);
        let react = reaction_params_for(&mesh);
        let transport = frozen_transport();
        for _ in 0..256 {
            let led = coupled_step_with_reserve_mode(
                &mut mesh,
                &FROZEN_CENTER,
                &react,
                &transport,
                true,
                true,
                ReserveDiagnosticMode::LiquidReserveUpperBound,
            );
            assert!(led.diagnostic_liquid_r_used >= 0.0);
            assert!(led.activation_equivalent_closure_residual <= 1e-6);
            assert_eq!(led.reserve.rejected_steps, 0);
        }
    }

    #[test]
    fn r9r5r1_prethrottle_liquidity_is_exercised_before_m_l_demand_suppression() {
        std::env::set_var("DCDEV020R9R3_CONTRACT", "ConservativeV2");
        std::env::set_var("DCDEV020R9R3_RESERVE", "1");
        let mut full = seed_mesh(14.0, 2);
        let mut shadow = full.clone();
        for mesh in [&mut full, &mut shadow] {
            mesh.interior.a = 1e-5;
            mesh.interior.r = 0.5;
            mesh.interior.c = 2.0;
            mesh.interior.n = 0.0;
            mesh.interior.f = 0.0;
        }
        let mut react = reaction_params_for(&full);
        // Hold the constructed reserve stock in place so the test isolates the
        // pre-throttle M/L availability question from ordinary D-091 release.
        react.reserve.k_release = 0.0;
        let full_led = reactions_step_with_reserve_mode(
            &mut full,
            &react,
            FROZEN_CENTER.dt,
            true,
            true,
            ReserveDiagnosticMode::Full,
        );
        let shadow_led = reactions_step_with_reserve_mode(
            &mut shadow,
            &react,
            FROZEN_CENTER.dt,
            true,
            true,
            ReserveDiagnosticMode::LiquidReservePreThrottleUpperBound,
        );
        assert!(full_led.diagnostic_liquid_r_used.abs() <= 1e-12);
        assert!(shadow_led.structural_demand_a > full_led.structural_demand_a);
        assert!(shadow_led.membrane_demand_a > full_led.membrane_demand_a);
        assert!(shadow_led.diagnostic_liquid_r_available > 0.0);
        assert!(shadow_led.diagnostic_liquid_r_used > 0.0);
        assert!(shadow_led.diagnostic_liquid_r_used_for_m > 0.0);
        assert!(shadow_led.diagnostic_liquid_r_used_for_l > 0.0);
        assert!(shadow_led.m_produced > full_led.m_produced);
        assert!(shadow_led.l_produced > full_led.l_produced);
        assert!(
            shadow_led.activation_equivalent_closure_residual <= 1e-6,
            "closure residual {} reserve residual {} r_to_a {} r_to_w {} used {} a_to_m {} a_to_l {}",
            shadow_led.activation_equivalent_closure_residual,
            shadow_led.reserve_closure_residual,
            shadow_led.reserve.r_to_a,
            shadow_led.reserve.r_to_w,
            shadow_led.diagnostic_liquid_r_used,
            shadow_led.a_to_m,
            shadow_led.a_to_l
        );
    }

    #[test]
    fn r9r6_mobilizes_before_production_and_stores_after_demand() {
        std::env::set_var("DCDEV020R9R3_CONTRACT", "ConservativeV2");
        std::env::set_var("DCDEV020R9R3_RESERVE", "1");
        let mut shadow = seed_mesh(14.0, 2);
        shadow.interior.a = 0.02;
        shadow.interior.r = 0.8;
        shadow.interior.c = 2.0;
        shadow.interior.n = 0.0;
        shadow.interior.f = 0.0;
        let react = reaction_params_for(&shadow);
        let transport = frozen_transport();
        let led = coupled_step_with_reserve_mode(
            &mut shadow,
            &FROZEN_CENTER,
            &react,
            &transport,
            true,
            true,
            ReserveDiagnosticMode::MobilizeFirstStoreLast,
        );
        assert!(led.reserve.r_to_a > 0.0);
        assert!(led.reserve.a_to_r > 0.0);
        assert!(led.a_before_catalyst_production > 0.0);
        assert!(led.a_before_final_storage >= 0.0);
        assert_eq!(led.reserve.r_to_m, 0.0);
        assert!(led.activation_equivalent_closure_residual <= 1e-6);
        assert!(led.reserve_closure_residual <= 1e-6);
        assert_eq!(led.reserve.rejected_steps, 0);
    }
}
