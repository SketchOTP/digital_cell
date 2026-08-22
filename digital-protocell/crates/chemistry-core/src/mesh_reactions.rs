//! D-086 mesh structural production, turnover, membrane binding, damage, death.

use crate::autocatalytic_copying::{edge_copying_step, edge_loss_step};
use crate::autocatalytic_edges::{merge_acs_ledgers, node_production_step, node_turnover_step};
use crate::autocatalytic_nodes::{
    node_activation_gain, node_building_gain, AutocatalyticLedger, AutocatalyticParams,
};
use crate::catalyst_composition::{
    composition_z, copy_production_fluxes, ensure_composition_initialized, g_build, g_harvest,
    sync_total_c, turnover_composition, CompositionLedger, CompositionParams,
};
use crate::material_mesh::MaterialMesh;
use crate::metabolic_reserve::{
    reserve_metab_step_with_controls, reserve_store_potential_amount, reserve_store_with_limit,
    ReserveFluxControls, ReserveLedger, ReserveParams,
};
use crate::template_copying::copying_step;
use crate::template_motifs::{catalyst_binding_step, template_activity_gains};
use crate::template_network::{scale_bound_catalyst, NetworkLedger, NetworkParams};
use crate::template_network_binding::network_binding_step;
use crate::template_network_expression::{network_activation_gain, network_building_gain};
use crate::template_partition::diffuse_templates;
use crate::template_polymer::{
    hydrolysis_step, merge_template_ledgers, monomer_production_step, TemplateLedger,
    TemplateParams, XorShift64,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeshChemistrySchema {
    HistoricalV1,
    ConservativeV2,
}

impl Default for MeshChemistrySchema {
    fn default() -> Self {
        Self::HistoricalV1
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ReactionParams {
    pub k_build: f64,
    pub k_turn: f64,
    pub k_eps: f64,
    pub g0: f64,
    pub yield_a_to_m: f64,
    pub k_bind: f64,
    pub k_unbind: f64,
    pub k_act: f64,
    pub k_c_prod: f64,
    pub k_c_turn: f64,
    pub k_a_decay: f64,
    pub q_c: f64,
    /// Historical v1 is the serde/default path. R9 explicitly opts into the
    /// versioned conservative mesh contract.
    #[serde(default, skip_serializing_if = "is_historical_mesh_schema")]
    pub mesh_schema: MeshChemistrySchema,
    /// D-089 compositional catalysis (default off → frozen scalar path).
    #[serde(default)]
    pub composition: CompositionParams,
    /// D-091 metabolic reserve (default off → D-088 surplus-A growth).
    #[serde(default)]
    pub reserve: ReserveParams,
    /// D-092 catalytic template polymer (default off → D-091 reserve path).
    #[serde(default)]
    pub template: TemplateParams,
    /// D-093 template-encoded catalytic network (default off).
    #[serde(default)]
    pub network: NetworkParams,
    /// D-094 distributed autocatalytic-set heredity (default off).
    #[serde(default)]
    pub autocatalytic: AutocatalyticParams,
}

impl Default for ReactionParams {
    fn default() -> Self {
        // Steady design: undamaged g≈g0 keeps build ≲ turnover; high strain (damage)
        // raises g→g0+1 so local rebuild exceeds turnover (Gate 6 repair).
        Self {
            k_build: 0.065,
            k_turn: 0.018,
            k_eps: 0.35,
            g0: 0.22,
            yield_a_to_m: 1.0,
            k_bind: 1.2,
            k_unbind: 0.06,
            k_act: 0.24,
            k_c_prod: 0.018,
            k_c_turn: 0.01,
            k_a_decay: 0.008,
            q_c: 0.3,
            mesh_schema: MeshChemistrySchema::HistoricalV1,
            composition: CompositionParams::default(),
            reserve: ReserveParams::default(),
            template: TemplateParams::default(),
            network: NetworkParams::default(),
            autocatalytic: AutocatalyticParams::default(),
        }
    }
}

fn is_historical_mesh_schema(schema: &MeshChemistrySchema) -> bool {
    *schema == MeshChemistrySchema::HistoricalV1
}

impl ReactionParams {
    pub fn conservative_v2() -> Self {
        let mut params = Self::default();
        params.mesh_schema = MeshChemistrySchema::ConservativeV2;
        params
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReactionLedger {
    pub a_consumed_build: f64,
    pub m_produced: f64,
    pub m_to_w: f64,
    pub b_to_w: f64,
    pub w_produced: f64,
    pub a_produced: f64,
    pub n_consumed: f64,
    pub f_consumed: f64,
    pub bind_extent: f64,
    pub unbind_extent: f64,
    /// Observer/accounting: catalyst mass produced (concentration·area units).
    pub c_produced: f64,
    /// Observer/accounting: catalyst turned over.
    pub c_turned: f64,
    /// Observer/accounting: activated material decayed directly to waste.
    #[serde(default)]
    pub a_decayed: f64,
    /// Observer/accounting: free membrane L produced from A.
    pub l_produced: f64,
    #[serde(default)]
    pub composition: CompositionLedger,
    #[serde(default)]
    pub reserve: ReserveLedger,
    #[serde(default)]
    pub template: TemplateLedger,
    #[serde(default)]
    pub network: NetworkLedger,
    #[serde(default)]
    pub autocatalytic: AutocatalyticLedger,
    /// R9-R4 observer accounting. These fields do not affect state updates.
    #[serde(default)]
    pub a_to_c: f64,
    #[serde(default)]
    pub a_before_reserve: f64,
    #[serde(default)]
    pub a_after_reserve: f64,
    #[serde(default)]
    pub a_after_pre_maintenance_reserve: f64,
    #[serde(default)]
    pub a_after_post_maintenance_storage: f64,
    #[serde(default)]
    pub a_before_structural_build: f64,
    #[serde(default)]
    pub a_after_structural_build: f64,
    #[serde(default)]
    pub a_before_membrane_production: f64,
    #[serde(default)]
    pub a_after_membrane_production: f64,
    #[serde(default)]
    pub structural_demand_a: f64,
    #[serde(default)]
    pub membrane_demand_a: f64,
    #[serde(default)]
    pub a_to_r_before_later_demand: f64,
    #[serde(default)]
    pub reserve_closure_residual: f64,
    /// R9-R5 observer accounting: A stock entering the reaction step.
    #[serde(default)]
    pub a_stock_entering: f64,
    /// R9-R5 observer accounting: R stock entering the reaction step.
    #[serde(default)]
    pub r_stock_entering: f64,
    /// R9-R6 phase-order ledger: A after unchanged N/F activation.
    #[serde(default)]
    pub a_after_activation: f64,
    /// R9-R6 phase-order ledger: reserve stock immediately before release/loss.
    #[serde(default)]
    pub r_before_release: f64,
    /// R9-R6 phase-order ledger: A entering catalyst production.
    #[serde(default)]
    pub a_before_catalyst_production: f64,
    /// R9-R6 phase-order ledger: A available before final A→R storage.
    #[serde(default)]
    pub a_before_final_storage: f64,
    /// R9-R5 observer accounting: A actually spent on structural maintenance.
    #[serde(default)]
    pub a_to_m: f64,
    /// R9-R5 observer accounting: A actually spent on membrane maintenance.
    #[serde(default)]
    pub a_to_l: f64,
    /// R9-R5 observer accounting: frozen charging potential before the step's
    /// diagnostic storage cap is applied.
    #[serde(default)]
    pub reserve_store_potential: f64,
    /// R9-R5 observer accounting: positive new-A surplus available after
    /// productive and loss fluxes, before A→R storage.
    #[serde(default)]
    pub new_a_surplus: f64,
    /// R9-R5 observer accounting: A→R funded by same-step newly produced A.
    #[serde(default)]
    pub a_to_r_same_step_new_a: f64,
    /// R9-R5 observer accounting: A→R funded by pre-existing A.
    #[serde(default)]
    pub a_to_r_pre_existing_a: f64,
    /// R9-R5 observer accounting: reserve material used directly for M/L.
    #[serde(default)]
    pub diagnostic_liquid_r_used: f64,
    /// R9-R5-R1 observer accounting: reserve made available to the pre-throttle
    /// M/L shadow demand calculation during this step.
    #[serde(default)]
    pub diagnostic_liquid_r_available: f64,
    /// R9-R5-R1 observer accounting: reserve consumed for structural M.
    #[serde(default)]
    pub diagnostic_liquid_r_used_for_m: f64,
    /// R9-R5-R1 observer accounting: reserve consumed for membrane L.
    #[serde(default)]
    pub diagnostic_liquid_r_used_for_l: f64,
    /// R9-R5 observer accounting: net activation-equivalent production/loss.
    #[serde(default)]
    pub net_activation_equivalent: f64,
    /// R9-R5 observer accounting: A+R closure residual after all internal
    /// transfers and direct diagnostic reserve use.
    #[serde(default)]
    pub activation_equivalent_closure_residual: f64,
}

/// Observer-only reserve scheduling for the R9-R4 audit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReserveDiagnosticMode {
    Full,
    StoreOff,
    ReleaseOff,
    LossOff,
    MaintenancePriority,
    /// Defer only A→R and cap it by positive same-step A surplus.
    SurplusOnlyStore,
    /// Preserve frozen reserve fluxes while allowing R as a diagnostic-only
    /// 1:1 substitute for existing A-dependent M/L maintenance.
    LiquidReserveUpperBound,
    /// Diagnostic-only upper bound that exposes existing R to the unchanged
    /// M/L demand equations before low A can throttle those equations.
    LiquidReservePreThrottleUpperBound,
    /// Apply both independent observer-only counterfactuals.
    SurplusOnlyStoreLiquidReserveUpperBound,
    /// Apply surplus-only storage and the pre-throttle liquidity shadow.
    SurplusOnlyStoreLiquidReservePreThrottleUpperBound,
    /// Execute unchanged R→A/R→W before productive chemistry and unchanged
    /// A→R storage after productive chemistry.
    MobilizeFirstStoreLast,
}

#[inline]
pub fn q_catalyst(c: f64, k_c: f64) -> f64 {
    let c = c.max(0.0);
    if k_c <= 0.0 {
        return if c > 0.0 { 1.0 } else { 0.0 };
    }
    c / (k_c + c)
}

#[inline]
pub fn g_strain(eps: f64, g0: f64, k_eps: f64) -> f64 {
    let pos = eps.max(0.0);
    // Bounded strain boost (≤ +0.45) — enough for local repair, not global runaway.
    g0 + 0.45 * pos / (k_eps.max(1e-15) + pos)
}

pub fn structural_build_flux(mesh: &MaterialMesh, i: usize, p: &ReactionParams) -> f64 {
    structural_build_flux_with_a(mesh, i, p, mesh.interior.a)
}

#[inline]
pub fn structural_build_flux_with_a(
    mesh: &MaterialMesh,
    i: usize,
    p: &ReactionParams,
    activation_a: f64,
) -> f64 {
    if mesh.edges[i].ruptured {
        return 0.0;
    }
    let qc = q_catalyst(mesh.interior.c, p.q_c);
    let a = activation_a.max(0.0);
    let g = g_strain(mesh.strain(i), p.g0, p.k_eps);
    // Scale by current length so mass-damaged edges (small ℓ⁰) still rebuild;
    // remesh split/merge preserves Σℓ so total demand stays remesh-invariant.
    let ell = mesh.edge_length(i);
    let gb = if p.composition.enable {
        let z = composition_z(mesh.interior.c_h, mesh.interior.c_b);
        g_build(z, p.composition.sigma)
    } else if p.autocatalytic.enable {
        node_building_gain(mesh, &p.autocatalytic, p.q_c)
    } else if p.network.enable {
        network_building_gain(mesh, &p.network, p.q_c)
    } else if p.template.enable {
        template_activity_gains(mesh, &p.template).1
    } else {
        crate::d096_allocation::function_gain(mesh, 2)
    };
    p.k_build * qc * a * g * ell * gb
}

/// Local chemistry + structure/membrane update for one accepted dt.
pub fn reactions_step(
    mesh: &mut MaterialMesh,
    p: &ReactionParams,
    dt: f64,
    enable_build: bool,
    enable_metab: bool,
) -> ReactionLedger {
    reactions_step_with_reserve_mode(
        mesh,
        p,
        dt,
        enable_build,
        enable_metab,
        ReserveDiagnosticMode::Full,
    )
}

/// Execute the normal reaction step with a bounded observer-only reserve mode.
pub fn reactions_step_with_reserve_mode(
    mesh: &mut MaterialMesh,
    p: &ReactionParams,
    dt: f64,
    enable_build: bool,
    enable_metab: bool,
    reserve_mode: ReserveDiagnosticMode,
) -> ReactionLedger {
    let mut led = ReactionLedger::default();
    if !mesh.can_advance_physics() {
        return led;
    }
    let n_edges = mesh.n();
    let area = mesh.area().max(1e-6);
    led.a_stock_entering = mesh.interior.a.max(0.0) * area;
    led.r_stock_entering = mesh.interior.r.max(0.0) * area;
    let surplus_only_store = matches!(
        reserve_mode,
        ReserveDiagnosticMode::SurplusOnlyStore
            | ReserveDiagnosticMode::SurplusOnlyStoreLiquidReserveUpperBound
    );
    let liquid_reserve = matches!(
        reserve_mode,
        ReserveDiagnosticMode::LiquidReserveUpperBound
            | ReserveDiagnosticMode::SurplusOnlyStoreLiquidReserveUpperBound
    );
    let liquid_pre_throttle = matches!(
        reserve_mode,
        ReserveDiagnosticMode::LiquidReservePreThrottleUpperBound
            | ReserveDiagnosticMode::SurplusOnlyStoreLiquidReservePreThrottleUpperBound
    );
    let liquid_for_m_l = liquid_reserve || liquid_pre_throttle;
    let mobilize_first_store_last = reserve_mode == ReserveDiagnosticMode::MobilizeFirstStoreLast;

    if enable_metab {
        if p.composition.enable {
            ensure_composition_initialized(&mut mesh.interior);
        }
        // N+F → A+W (lumped interior).
        let qc = q_catalyst(mesh.interior.c, p.q_c);
        let gh = if p.composition.enable {
            let z = composition_z(mesh.interior.c_h, mesh.interior.c_b);
            g_harvest(z, p.composition.sigma)
        } else if p.autocatalytic.enable {
            node_activation_gain(mesh, &p.autocatalytic, p.q_c)
        } else if p.network.enable {
            network_activation_gain(mesh, &p.network, p.q_c)
        } else if p.template.enable {
            template_activity_gains(mesh, &p.template).0
        } else if mesh.finite_allocation.is_some() {
            let processing = crate::d096_allocation::function_gain(mesh, 0);
            let activation = crate::d096_allocation::function_gain(mesh, 1);
            (processing * activation).sqrt()
        } else {
            1.0
        };
        let extent =
            p.k_act * qc * gh * mesh.interior.n.max(0.0) * mesh.interior.f.max(0.0) * dt * area;
        let n_take = extent.min(mesh.interior.n.max(0.0) * area) / area;
        let f_take = extent.min(mesh.interior.f.max(0.0) * area) / area;
        let taken = n_take.min(f_take);
        mesh.interior.n = (mesh.interior.n - taken).max(0.0);
        mesh.interior.f = (mesh.interior.f - taken).max(0.0);
        mesh.interior.a += taken;
        mesh.interior.w += taken;
        led.n_consumed += taken * area;
        led.f_consumed += taken * area;
        led.a_produced += taken * area;
        led.w_produced += taken * area;
        led.a_after_activation = mesh.interior.a.max(0.0) * area;
        led.a_before_catalyst_production = led.a_after_activation;

        // Catalyst production / turnover (new C unlabeled; turnover ages tracer).
        // Composition mode: copy with μ during production only; turnover equal on both types.
        // D-093: turnover applies equally to free and bound catalyst; production enters free.
        let c_before = mesh.interior.c.max(0.0);
        let c_prod = p.k_c_prod * mesh.interior.a.max(0.0) * dt;
        let c_turn = p.k_c_turn * c_before * dt;
        if c_before > 1e-15 && c_turn > 0.0 {
            let frac = (c_turn / c_before).clamp(0.0, 1.0);
            mesh.interior.tracer_c = (mesh.interior.tracer_c * (1.0 - frac)).max(0.0);
            if p.network.enable {
                scale_bound_catalyst(mesh, 1.0 - frac);
            }
        }
        if p.composition.enable {
            let c_h0 = mesh.interior.c_h.max(0.0);
            let c_b0 = mesh.interior.c_b.max(0.0);
            let (j_h, j_b) = copy_production_fluxes(c_prod, c_h0, c_b0, p.composition.mu);
            let (c_h1, c_b1, t_h, t_b) = turnover_composition(c_h0, c_b0, c_turn);
            mesh.interior.c_h = (c_h1 + j_h).max(0.0);
            mesh.interior.c_b = (c_b1 + j_b).max(0.0);
            sync_total_c(&mut mesh.interior);
            // Conversion mass = μ-driven alternate-type production (observer).
            let ph0 = crate::catalyst_composition::p_h(c_h0, c_b0);
            let pb0 = crate::catalyst_composition::p_b(c_h0, c_b0);
            let conv = c_prod * p.composition.mu * (ph0 + pb0); // = μ J_C when pool nonempty
            led.composition.conversion_events += conv * area;
            led.composition.c_h_produced += j_h * area;
            led.composition.c_b_produced += j_b * area;
            led.composition.c_h_turned += t_h * area;
            led.composition.c_b_turned += t_b * area;
        } else {
            mesh.interior.c = (c_before + c_prod - c_turn).max(0.0);
        }
        mesh.interior.a = (mesh.interior.a - c_prod).max(0.0);
        mesh.interior.w += c_turn;
        led.c_produced += c_prod * area;
        led.a_to_c += c_prod * area;
        led.c_turned += c_turn * area;
        led.w_produced += c_turn * area;

        let a_dec = {
            // Accelerate A loss when activation substrates are absent (starvation).
            let starve = if mesh.interior.n.max(0.0) * mesh.interior.f.max(0.0) < 1e-8 {
                4.0
            } else {
                1.0
            };
            p.k_a_decay * starve * mesh.interior.a.max(0.0) * dt
        };
        mesh.interior.a = (mesh.interior.a - a_dec).max(0.0);
        mesh.interior.w += a_dec;
        led.a_decayed += a_dec * area;
        led.w_produced += a_dec * area;

        // D-091 metabolic reserve. Full mode preserves the existing ordering;
        // the other modes are bounded observer-only R9-R4/R9-R5 controls.
        led.a_before_reserve = mesh.interior.a.max(0.0) * area;
        led.r_before_release = mesh.interior.r.max(0.0) * area;
        led.reserve_store_potential = reserve_store_potential_amount(mesh, p, dt);
        let pre_controls = match reserve_mode {
            ReserveDiagnosticMode::Full => ReserveFluxControls::FULL,
            ReserveDiagnosticMode::StoreOff => ReserveFluxControls::STORE_OFF,
            ReserveDiagnosticMode::ReleaseOff => ReserveFluxControls::RELEASE_OFF,
            ReserveDiagnosticMode::LossOff => ReserveFluxControls::LOSS_OFF,
            ReserveDiagnosticMode::MaintenancePriority => {
                ReserveFluxControls::RELEASE_AND_LOSS_ONLY
            }
            ReserveDiagnosticMode::SurplusOnlyStore
            | ReserveDiagnosticMode::SurplusOnlyStoreLiquidReserveUpperBound
            | ReserveDiagnosticMode::SurplusOnlyStoreLiquidReservePreThrottleUpperBound => {
                ReserveFluxControls::RELEASE_AND_LOSS_ONLY
            }
            ReserveDiagnosticMode::LiquidReserveUpperBound
            | ReserveDiagnosticMode::LiquidReservePreThrottleUpperBound => {
                ReserveFluxControls::FULL
            }
            ReserveDiagnosticMode::MobilizeFirstStoreLast => {
                ReserveFluxControls::RELEASE_AND_LOSS_ONLY
            }
        };
        let reserve_area = mesh.area().max(1e-6);
        let reserve_a0 = mesh.interior.a.max(0.0) * reserve_area;
        let reserve_r0 = mesh.interior.r.max(0.0) * reserve_area;
        let pre_ledger = reserve_metab_step_with_controls(mesh, p, dt, pre_controls);
        let reserve_a1 = mesh.interior.a.max(0.0) * reserve_area;
        let reserve_r1 = mesh.interior.r.max(0.0) * reserve_area;
        led.reserve = pre_ledger.clone();
        led.reserve_closure_residual +=
            (reserve_a0 + reserve_r0 - reserve_a1 - reserve_r1 - pre_ledger.r_to_w).abs();
        led.a_after_pre_maintenance_reserve = mesh.interior.a.max(0.0) * area;
        led.a_after_reserve = led.a_after_pre_maintenance_reserve;

        // D-092/D-093 template polymer chemistry (monomers, copying, hydrolysis).
        if p.template.enable {
            let mut rng = XorShift64::new(mesh.template_rng.max(1));
            let mut next_id = mesh.next_template_id.max(1);
            let mut tled = monomer_production_step(mesh, p, dt);
            let cled = copying_step(mesh, p, dt, &mut rng, &mut next_id);
            mesh.next_template_id = next_id;
            merge_template_ledgers(&mut tled, &cled);
            let hled = hydrolysis_step(mesh, p, dt, &mut rng);
            merge_template_ledgers(&mut tled, &hled);
            if p.network.enable {
                // D-093 overlapping pair-site network binding (not fixed motifs).
                led.network = network_binding_step(mesh, p, dt);
            } else {
                // D-092 fixed-motif complexes (immutable historical path).
                let bled = catalyst_binding_step(mesh, p, dt);
                merge_template_ledgers(&mut tled, &bled);
            }
            diffuse_templates(mesh, dt, 0.02);
            mesh.template_rng = rng.state();
            led.template = tled;
            // Template ligation/monomer A costs already deducted; account waste.
            led.w_produced += led.template.w_produced;
        } else if p.network.enable {
            led.network = network_binding_step(mesh, p, dt);
        }

        // D-094 autocatalytic-set chemistry (nodes/edges; supplements baseline C).
        if p.autocatalytic.enable {
            let mut rng = XorShift64::new(mesh.template_rng.wrapping_add(0xD094_ACE5));
            let mut acs = node_production_step(mesh, &p.autocatalytic, dt);
            merge_acs_ledgers(&mut acs, &node_turnover_step(mesh, &p.autocatalytic, dt));
            merge_acs_ledgers(
                &mut acs,
                &edge_copying_step(mesh, &p.autocatalytic, dt, &mut rng),
            );
            merge_acs_ledgers(
                &mut acs,
                &edge_loss_step(mesh, &p.autocatalytic, dt, &mut rng),
            );
            mesh.template_rng = rng.state().wrapping_add(1);
            led.autocatalytic = acs;
            led.w_produced += led.autocatalytic.w_produced;
        }
    }

    led.a_before_structural_build = mesh.interior.a.max(0.0) * area;
    if liquid_pre_throttle {
        led.diagnostic_liquid_r_available = mesh.interior.r.max(0.0) * area;
    }

    // Per-edge build / turnover / bind.
    for i in 0..n_edges {
        if mesh.edges[i].ruptured {
            continue;
        }
        if enable_build {
            let activation_for_flux = if liquid_pre_throttle {
                mesh.interior.a.max(0.0) + mesh.interior.r.max(0.0)
            } else {
                mesh.interior.a.max(0.0)
            };
            let j_build = structural_build_flux_with_a(mesh, i, p, activation_for_flux) * dt;
            let need_a = j_build / p.yield_a_to_m.max(1e-15);
            led.structural_demand_a += need_a;
            let have = mesh.interior.a.max(0.0) * area;
            let a_baseline = if liquid_pre_throttle {
                structural_build_flux_with_a(mesh, i, p, mesh.interior.a.max(0.0)) * dt
                    / p.yield_a_to_m.max(1e-15)
            } else {
                need_a
            };
            let a_take = a_baseline.min(have);
            let r_have = mesh.interior.r.max(0.0) * area;
            let r_take = if liquid_for_m_l {
                (need_a - a_take).max(0.0).min(r_have)
            } else {
                0.0
            };
            let take = a_take + r_take;
            let dm = take * p.yield_a_to_m;
            mesh.interior.a = (mesh.interior.a - a_take / area).max(0.0);
            mesh.interior.r = (mesh.interior.r - r_take / area).max(0.0);
            let w_product = if p.mesh_schema == MeshChemistrySchema::ConservativeV2
                || mesh.uses_observer_only_death()
            {
                (take - dm).max(0.0)
            } else {
                take
            };
            mesh.interior.w += w_product / area;
            mesh.edges[i].m += dm;
            led.a_consumed_build += take;
            led.a_to_m += a_take;
            led.diagnostic_liquid_r_used += r_take;
            led.diagnostic_liquid_r_used_for_m += r_take;
            led.m_produced += dm;
            led.w_produced += w_product;

            let turn_scale = 1.0 / (1.0 + 2.0 * mesh.strain(i).max(0.0));
            let turn = p.k_turn * turn_scale * mesh.edges[i].m.max(0.0) * dt;
            let rem = turn.min(mesh.edges[i].m.max(0.0));
            mesh.edges[i].m -= rem;
            // Tracer ages with turnover.
            if mesh.edges[i].m + rem > 1e-15 {
                let frac = rem / (mesh.edges[i].m + rem);
                mesh.edges[i].tracer_m = (mesh.edges[i].tracer_m * (1.0 - frac)).max(0.0);
            }
            mesh.interior.w += rem / area;
            led.m_to_w += rem;
            led.w_produced += rem;
        }

        // L ⇌ b (new binds unlabeled; unbind ages bound tracer).
        let cap = mesh.b_max_for_edge(i);
        let theta = mesh.occupancy(i);
        let bind = p.k_bind * mesh.free_l.max(0.0) * (1.0 - theta) * dt;
        let unbind = p.k_unbind * mesh.edges[i].b.max(0.0) * dt;
        let bind_a = bind
            .min(mesh.free_l.max(0.0))
            .min((cap - mesh.edges[i].b).max(0.0));
        let unbind_a = unbind.min(mesh.edges[i].b.max(0.0));
        if mesh.edges[i].b > 1e-15 && unbind_a > 0.0 {
            let frac = (unbind_a / mesh.edges[i].b).clamp(0.0, 1.0);
            mesh.edges[i].tracer_b = (mesh.edges[i].tracer_b * (1.0 - frac)).max(0.0);
        }
        mesh.free_l = (mesh.free_l - bind_a + unbind_a).max(0.0);
        mesh.edges[i].b = (mesh.edges[i].b + bind_a - unbind_a).max(0.0);
        led.bind_extent += bind_a;
        led.unbind_extent += unbind_a;
    }

    led.a_after_structural_build = mesh.interior.a.max(0.0) * area;
    led.a_before_membrane_production = led.a_after_structural_build;

    // Free membrane reserve production from A — only when binding capacity remains
    // (prevents saturated free_l from draining A after θ→1).
    if enable_metab {
        let qc = q_catalyst(mesh.interior.c, p.q_c);
        let gb = if p.composition.enable {
            let z = composition_z(mesh.interior.c_h, mesh.interior.c_b);
            g_build(z, p.composition.sigma)
        } else if p.autocatalytic.enable {
            node_building_gain(mesh, &p.autocatalytic, p.q_c)
        } else if p.network.enable {
            network_building_gain(mesh, &p.network, p.q_c)
        } else if p.template.enable {
            template_activity_gains(mesh, &p.template).1
        } else if mesh.finite_allocation.is_some() {
            crate::d096_allocation::function_gain(mesh, 2)
        } else {
            1.0
        };
        let peri = mesh.perimeter().max(1e-6);
        let theta = {
            let mut s = 0.0;
            let mut n = 0.0;
            for i in 0..n_edges {
                if mesh.edges[i].ruptured {
                    continue;
                }
                s += mesh.occupancy(i);
                n += 1.0;
            }
            if n <= 0.0 {
                1.0
            } else {
                s / n
            }
        };
        let reserve_cap = 0.15 * peri;
        if theta < 0.95 || mesh.free_l < reserve_cap {
            let activation_for_flux = if liquid_pre_throttle {
                mesh.interior.a.max(0.0) + mesh.interior.r.max(0.0)
            } else {
                mesh.interior.a.max(0.0)
            };
            let l_prod = 0.02 * qc * gb * activation_for_flux * peri * dt;
            led.membrane_demand_a += l_prod;
            let room = (reserve_cap - mesh.free_l).max(0.0) + (1.0 - theta) * peri;
            let l_baseline = if liquid_pre_throttle {
                0.02 * qc * gb * mesh.interior.a.max(0.0) * peri * dt
            } else {
                l_prod
            };
            let a_take = l_baseline
                .min(mesh.interior.a.max(0.0) * area)
                .min(room.max(0.0));
            let r_have = mesh.interior.r.max(0.0) * area;
            let r_take = if liquid_for_m_l {
                (l_prod - a_take).max(0.0).min(r_have)
            } else {
                0.0
            };
            let take = a_take + r_take;
            mesh.interior.a = (mesh.interior.a - a_take / area).max(0.0);
            mesh.interior.r = (mesh.interior.r - r_take / area).max(0.0);
            let w_product = if p.mesh_schema == MeshChemistrySchema::ConservativeV2
                || mesh.uses_observer_only_death()
            {
                0.0
            } else {
                take
            };
            mesh.interior.w += w_product / area;
            mesh.free_l += take;
            led.l_produced += take;
            led.a_to_l += a_take;
            led.diagnostic_liquid_r_used += r_take;
            led.diagnostic_liquid_r_used_for_l += r_take;
            led.w_produced += w_product;
        }
    }

    led.a_after_membrane_production = mesh.interior.a.max(0.0) * area;
    led.a_before_final_storage = led.a_after_membrane_production;

    // R9-R5 attribution is deliberately an accounting decomposition. It does
    // not alter any frozen production coefficient or introduce a controller.
    led.new_a_surplus = (led.a_produced + led.reserve.r_to_a
        - led.a_to_c
        - led.a_decayed
        - led.a_to_m
        - led.a_to_l)
        .max(0.0);

    if surplus_only_store && enable_metab {
        let cap = led.new_a_surplus.min(led.reserve_store_potential);
        let post = reserve_store_with_limit(mesh, p, dt, cap);
        led.reserve.a_to_r += post.a_to_r;
        led.reserve.r_to_a += post.r_to_a;
        led.reserve.r_to_w += post.r_to_w;
        led.reserve.r_to_m += post.r_to_m;
        led.reserve.w_from_r_growth += post.w_from_r_growth;
        led.reserve.rejected_steps += post.rejected_steps;
        led.a_after_post_maintenance_storage = mesh.interior.a.max(0.0) * area;
        led.a_after_reserve = led.a_after_post_maintenance_storage;
        let reserve_a1 = mesh.interior.a.max(0.0) * area;
        let reserve_r1 = mesh.interior.r.max(0.0) * area;
        led.reserve_closure_residual +=
            (led.a_stock_entering + led.r_stock_entering + led.a_produced
                - led.a_to_c
                - led.a_decayed
                - led.a_to_m
                - led.a_to_l
                - led.diagnostic_liquid_r_used
                - led.reserve.r_to_w
                - reserve_a1
                - reserve_r1)
                .abs();
    }

    if mobilize_first_store_last && enable_metab {
        let reserve_area = mesh.area().max(1e-6);
        let reserve_a0 = mesh.interior.a.max(0.0) * reserve_area;
        let reserve_r0 = mesh.interior.r.max(0.0) * reserve_area;
        let post = reserve_metab_step_with_controls(mesh, p, dt, ReserveFluxControls::STORE_ONLY);
        let reserve_a1 = mesh.interior.a.max(0.0) * reserve_area;
        let reserve_r1 = mesh.interior.r.max(0.0) * reserve_area;
        led.reserve_closure_residual +=
            (reserve_a0 + reserve_r0 - reserve_a1 - reserve_r1 - post.r_to_w).abs();
        led.reserve.a_to_r += post.a_to_r;
        led.reserve.r_to_a += post.r_to_a;
        led.reserve.r_to_w += post.r_to_w;
        led.reserve.r_to_m += post.r_to_m;
        led.reserve.w_from_r_growth += post.w_from_r_growth;
        led.reserve.rejected_steps += post.rejected_steps;
        led.a_after_post_maintenance_storage = reserve_a1;
        led.a_after_reserve = reserve_a1;
    }

    if reserve_mode == ReserveDiagnosticMode::MaintenancePriority && enable_metab {
        let reserve_area = mesh.area().max(1e-6);
        let reserve_a0 = mesh.interior.a.max(0.0) * reserve_area;
        let reserve_r0 = mesh.interior.r.max(0.0) * reserve_area;
        let post = reserve_metab_step_with_controls(mesh, p, dt, ReserveFluxControls::STORE_ONLY);
        let reserve_a1 = mesh.interior.a.max(0.0) * reserve_area;
        let reserve_r1 = mesh.interior.r.max(0.0) * reserve_area;
        led.reserve_closure_residual +=
            (reserve_a0 + reserve_r0 - reserve_a1 - reserve_r1 - post.r_to_w).abs();
        led.reserve.a_to_r += post.a_to_r;
        led.reserve.r_to_a += post.r_to_a;
        led.reserve.r_to_w += post.r_to_w;
        led.reserve.r_to_m += post.r_to_m;
        led.reserve.w_from_r_growth += post.w_from_r_growth;
        led.reserve.rejected_steps += post.rejected_steps;
        led.a_after_post_maintenance_storage = mesh.interior.a.max(0.0) * area;
        led.a_after_reserve = led.a_after_post_maintenance_storage;
    } else {
        if !surplus_only_store {
            led.a_after_post_maintenance_storage = led.a_after_reserve;
        }
    }
    if led.structural_demand_a + led.membrane_demand_a > 1e-15 {
        led.a_to_r_before_later_demand =
            if reserve_mode == ReserveDiagnosticMode::MaintenancePriority {
                0.0
            } else {
                led.reserve.a_to_r
            };
    }

    led.a_to_r_same_step_new_a = led.reserve.a_to_r.min(led.new_a_surplus);
    led.a_to_r_pre_existing_a = (led.reserve.a_to_r - led.a_to_r_same_step_new_a).max(0.0);
    led.net_activation_equivalent = led.a_produced
        - led.a_to_c
        - led.a_decayed
        - led.a_to_m
        - led.a_to_l
        - led.diagnostic_liquid_r_used
        - led.reserve.r_to_w;
    let final_activation = mesh.interior.a.max(0.0) * area + mesh.interior.r.max(0.0) * area;
    led.activation_equivalent_closure_residual =
        (led.a_stock_entering + led.r_stock_entering + led.a_produced
            - led.a_to_c
            - led.a_decayed
            - led.a_to_m
            - led.a_to_l
            - led.diagnostic_liquid_r_used
            - led.reserve.r_to_w
            - final_activation)
            .abs();

    // Rupture check.
    for e in &mut mesh.edges {
        if e.m < mesh.bond_threshold {
            e.ruptured = true;
            e.m = e.m.max(0.0);
        }
    }

    evaluate_death(mesh);
    led
}

pub fn evaluate_death(mesh: &mut MaterialMesh) {
    if mesh.uses_observer_only_death() {
        return;
    }
    if !mesh.alive {
        return;
    }
    let ruptured = mesh.edges.iter().filter(|e| e.ruptured).count();
    let n = mesh.n().max(1);
    if ruptured * 2 >= n {
        mesh.alive = false;
        mesh.death_reason = Some("mesh_rupture".into());
        return;
    }
    if mesh.interior.c < 1e-4 && mesh.total_structural_mass() < mesh.bond_threshold * n as f64 {
        mesh.alive = false;
        mesh.death_reason = Some("catalytic_structural_loss".into());
        return;
    }
    if mesh.interior.a < 1e-5 && mesh.interior.c < 1e-4 {
        mesh.alive = false;
        mesh.death_reason = Some("activated_catalyst_collapse".into());
        return;
    }
    // Starvation: activated pool exhausted and activation substrates gone.
    if mesh.interior.a < 0.02 && mesh.interior.n * mesh.interior.f < 1e-8 {
        mesh.alive = false;
        mesh.death_reason = Some("starvation_collapse".into());
    }
}

pub fn apply_structural_damage(mesh: &mut MaterialMesh, fraction: f64) -> f64 {
    let mut removed = 0.0;
    let n = mesh.n();
    let count = ((n as f64) * fraction.clamp(0.0, 1.0)).round() as usize;
    let area = mesh.area().max(1e-6);
    for i in 0..count.min(n) {
        let rem = mesh.edges[i].m * 0.5;
        mesh.edges[i].m -= rem;
        removed += rem;
        mesh.interior.w += rem / area;
        if mesh.edges[i].m < mesh.bond_threshold {
            mesh.edges[i].ruptured = true;
            continue;
        }
        // Plastic local shortening toward new rest length (wound contracts; no target shape).
        let l0 = mesh.rest_length(i);
        let len = mesh.edge_length(i);
        if len > l0 + 1e-9 {
            let j = (i + 1) % n;
            let a = mesh.vertices[i];
            let b = mesh.vertices[j];
            let t = [(b[0] - a[0]) / len, (b[1] - a[1]) / len];
            let shrink = 0.5 * (len - l0);
            mesh.vertices[i][0] += shrink * t[0];
            mesh.vertices[i][1] += shrink * t[1];
            mesh.vertices[j][0] -= shrink * t[0];
            mesh.vertices[j][1] -= shrink * t[1];
        }
    }
    removed
}

pub fn apply_membrane_damage(mesh: &mut MaterialMesh, fraction: f64) -> f64 {
    let mut removed = 0.0;
    let n = mesh.n();
    let count = ((n as f64) * fraction.clamp(0.0, 1.0)).round() as usize;
    for i in 0..count.min(n) {
        let rem = mesh.edges[i].b * 0.5;
        mesh.edges[i].b -= rem;
        removed += rem;
        // Damaged membrane units return to free reserve for local rebinding.
        mesh.free_l += rem * 0.5;
        mesh.interior.w += rem * 0.5 / mesh.area().max(1e-6);
    }
    removed
}

pub fn apply_local_rupture(mesh: &mut MaterialMesh, i: usize) {
    if i < mesh.edges.len() {
        let rem = mesh.edges[i].m;
        mesh.edges[i].m = 0.0;
        mesh.edges[i].ruptured = true;
        mesh.interior.w += rem / mesh.area().max(1e-6);
    }
}

/// Attempt local rebond of a ruptured edge if free ends are close and A/C available.
pub fn try_local_rebond(mesh: &mut MaterialMesh, max_dist: f64) -> bool {
    if !mesh.can_advance_physics() {
        return false;
    }
    if mesh.interior.a < 0.05 || mesh.interior.c < 0.05 {
        return false;
    }
    let n = mesh.n();
    for i in 0..n {
        if !mesh.edges[i].ruptured {
            continue;
        }
        let a = mesh.vertices[i];
        let b = mesh.vertices[(i + 1) % n];
        let dist = (b[0] - a[0]).hypot(b[1] - a[1]);
        if dist <= max_dist {
            let need = mesh.rho_s * dist;
            let area = mesh.area().max(1e-6);
            let have = mesh.interior.a.max(0.0) * area;
            if have >= need {
                mesh.interior.a = (mesh.interior.a - need / area).max(0.0);
                mesh.edges[i].m = need;
                mesh.edges[i].ruptured = false;
                return true;
            }
        }
    }
    false
}

/// Pulse-chase: mark a fraction of structural/membrane/catalyst mass as tracer.
pub fn pulse_tracers(mesh: &mut MaterialMesh, frac: f64) {
    let f = frac.clamp(0.0, 1.0);
    for e in &mut mesh.edges {
        e.tracer_m = e.m * f;
        e.tracer_b = e.b * f;
    }
    mesh.interior.tracer_c = mesh.interior.c.max(0.0) * f;
}

pub fn tracer_structural_fraction(mesh: &MaterialMesh) -> f64 {
    let m = mesh.total_structural_mass().max(1e-15);
    mesh.edges.iter().map(|e| e.tracer_m).sum::<f64>() / m
}

pub fn tracer_membrane_fraction(mesh: &MaterialMesh) -> f64 {
    let b = mesh.total_bound_membrane().max(1e-15);
    mesh.edges.iter().map(|e| e.tracer_b).sum::<f64>() / b
}

pub fn tracer_catalyst_fraction(mesh: &MaterialMesh) -> f64 {
    let c = mesh.interior.c.max(1e-15);
    (mesh.interior.tracer_c / c).clamp(0.0, 1.0)
}
