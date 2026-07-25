//! D-094 mandatory zero-generation causality audit of D-093 selection failure.
//!
//! Matched organism conditions under pulse-lean H ecology; classify first blocker.

use crate::d090_dish::{assemble_population, observe_spatial_dish, spatial_dish_step};
use crate::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use crate::mesh_fission::FissionParams;
use crate::mesh_growth::GrowthParams;
use crate::mesh_mechanics::MechParams;
use crate::mesh_population::MeshIndividual;
use crate::mesh_reactions::ReactionParams;
use crate::mesh_transport::TransportParams;
use crate::metabolic_reserve::{stamp_reserve_equation, ReserveParams};
use crate::seasonal_ecology::{PulseLeanSchedule, PulseLeanState, PULSE_PERIOD_MULTS};
use crate::spatial_shared_dish::SpatialDish;
use crate::template_network::{
    c_free, derive_k_site, stamp_network_equation, NetworkParams, RHO_NETWORK,
};
use crate::template_network_binding::sum_channel_masses;
use crate::template_polymer::{seed_founder_chains, TemplateParams};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

const EPS: f64 = 1e-15;
const FOUNDER_H: &str = "BHHBBHHBBHHB";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZeroGenBlocker {
    CatalystSequestration,
    TemplateCopyingCost,
    ReserveChargingSuppression,
    GrowthSuppression,
    EcologyHorizonTooShort,
    NumericalOrHarnessDefect,
    OtherNetworkCost,
}

impl ZeroGenBlocker {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CatalystSequestration => "CATALYST_SEQUESTRATION",
            Self::TemplateCopyingCost => "TEMPLATE_COPYING_COST",
            Self::ReserveChargingSuppression => "RESERVE_CHARGING_SUPPRESSION",
            Self::GrowthSuppression => "GROWTH_SUPPRESSION",
            Self::EcologyHorizonTooShort => "ECOLOGY_HORIZON_TOO_SHORT",
            Self::NumericalOrHarnessDefect => "NUMERICAL_OR_HARNESS_DEFECT",
            Self::OtherNetworkCost => "OTHER_NETWORK_COST",
        }
    }
}

#[derive(Debug, Clone)]
struct ArmObs {
    name: &'static str,
    max_gen: u32,
    fissions: usize,
    alive: bool,
    death_reason: Option<String>,
    mass0: f64,
    mass1: f64,
    mass_ratio: f64,
    a_mean: f64,
    r_mean: f64,
    r_max_seen: f64,
    c_free_mean: f64,
    c_bound_mean: f64,
    c_tot_mean: f64,
    bound_fraction_mean: f64,
    template_count_mean: f64,
    steps: usize,
}

fn seed_mesh(radius: f64, seed: u64, ext: f64) -> MaterialMesh {
    let n = 24 + ((seed % 3) as usize);
    MaterialMesh::seed_regular(
        n,
        radius,
        40.0,
        40.0,
        DEFAULT_RHO_S,
        0.7,
        LumpedChem {
            c: 0.8,
            a: 0.5,
            n: 0.4,
            f: 0.4,
            w: 0.1,
            ..Default::default()
        },
        LumpedChem {
            n: 1.0 * ext,
            f: 1.0 * ext,
            ..Default::default()
        },
        5.0,
    )
}

fn elongate(mesh: &mut MaterialMesh) {
    let c = mesh.centroid();
    for v in &mut mesh.vertices {
        v[0] = c[0] + (v[0] - c[0]) * 1.55;
        v[1] = c[1] + (v[1] - c[1]) * 0.72;
    }
}

fn derive_horizons() -> (f64, f64, f64, f64, f64, f64) {
    let react = ReactionParams::default();
    let t_replace = 1.0 / react.k_turn.max(1e-9);
    let mut mesh = seed_mesh(5.0, 1, 1.0);
    let mech = MechParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        enable_growth: false,
        y_g: 0.0,
    };
    let fission = FissionParams::default();
    let mut a_samples = Vec::new();
    let mut maint = 0.0;
    for s in 0..2500 {
        let _ = crate::mesh_population::coupled_step_growth(
            &mut mesh,
            &mech,
            &react,
            &transport,
            &growth,
            &fission,
            true,
            false,
        );
        crate::mesh_reactions::evaluate_death(&mut mesh);
        if s > 500 {
            a_samples.push(mesh.interior.a);
            maint += mesh.total_structural_mass();
        }
    }
    a_samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let a_median = a_samples[a_samples.len() / 2];
    let a_q25 = a_samples[a_samples.len() / 4];
    let t_maint = (maint / 2000.0).max(1.0);
    let area = mesh.area();
    let fission_a_cost = 0.15 * a_median * area;
    (t_replace, t_maint, a_median, a_q25, fission_a_cost, area)
}

fn selected_reserve() -> ReserveParams {
    let (t_replace, t_maint, a_median, a_q25, fission_a_cost, area) = derive_horizons();
    ReserveParams::derived(t_replace, t_maint, a_median, a_q25, 2.0, fission_a_cost, area)
}

fn selected_network(reserve: &ReserveParams) -> NetworkParams {
    let t_maint = 1.0 / reserve.k_release.max(1e-9);
    let mut mesh = seed_mesh(5.0, 1, 1.0);
    stamp_network_equation(&mut mesh);
    let k_site = derive_k_site(0.9, mesh.area(), 8);
    let k_d = 0.14;
    NetworkParams::derived(reserve, t_maint, k_d, k_site)
}

fn with_network(
    mut react: ReactionParams,
    reserve: ReserveParams,
    tmpl: TemplateParams,
    net: NetworkParams,
) -> ReactionParams {
    react.reserve = reserve;
    react.template = tmpl;
    react.network = net;
    react.composition.enable = false;
    react
}

fn seed_network_org(seq: &str, n_chains: usize, seed: u64) -> MeshIndividual {
    let mut mesh = seed_mesh(12.0, seed, 0.5);
    elongate(&mut mesh);
    stamp_network_equation(&mut mesh);
    mesh.interior.r = 0.6;
    mesh.interior.a = 0.8;
    mesh.interior.u_h = 0.2;
    mesh.interior.u_b = 0.2;
    let next_id = mesh.next_template_id.max(1);
    let next = seed_founder_chains(&mut mesh, seq, n_chains, next_id);
    mesh.next_template_id = next;
    let c = mesh.centroid();
    for (i, t) in mesh.templates.iter_mut().enumerate() {
        let f = (i as f64 + 0.5) / n_chains.max(1) as f64;
        t.pos = [c[0] + (f - 0.5) * 10.0, c[1]];
    }
    MeshIndividual {
        birth_mass: mesh.total_structural_mass(),
        mesh,
        lineage_id: seed,
        generation: 0,
        clade: 0,
    }
}

fn seed_reserve_org(seed: u64, reserve: &ReserveParams) -> MeshIndividual {
    let mut mesh = seed_mesh(12.0, seed, 0.5);
    elongate(&mut mesh);
    stamp_reserve_equation(&mut mesh);
    mesh.interior.r = 0.6;
    mesh.interior.a = 0.8;
    let _ = reserve;
    MeshIndividual {
        birth_mass: mesh.total_structural_mass(),
        mesh,
        lineage_id: seed,
        generation: 0,
        clade: 0,
    }
}

fn compact_dish() -> SpatialDish {
    SpatialDish::new(8, 8, 2.5, [0.0, 0.0], 120.0, 120.0, 0.0, 0.0, 3.0)
}

fn bound_catalyst_mass(mesh: &MaterialMesh) -> f64 {
    let (hh, hb, bh, bb) = sum_channel_masses(mesh);
    hh + hb + bh + bb
}

fn run_arm(
    name: &'static str,
    mut ind: MeshIndividual,
    react: &ReactionParams,
    n_steps: usize,
) -> ArmObs {
    let mech = MechParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let fission = FissionParams::default();
    let mut dish = compact_dish();
    let mut pop = assemble_population(vec![ind], &dish, 8.0);
    let t_maint = 1.0 / react.reserve.k_release.max(1e-9);
    let period = PULSE_PERIOD_MULTS[0] * t_maint * 4.0;
    let mut pulse = PulseLeanState::new(PulseLeanSchedule {
        cycle_period: period,
        pulse_fraction: 0.20,
        cycle_nf_budget: 1.10 * 0.05 * period,
        lean_nf_rate: 0.0,
    });
    let mass0 = pop.individuals[0].mesh.total_structural_mass();
    let mut a_sum = 0.0f64;
    let mut r_sum = 0.0f64;
    let mut r_max = 0.0f64;
    let mut cf_sum = 0.0f64;
    let mut cb_sum = 0.0f64;
    let mut ct_sum = 0.0f64;
    let mut tmpl_sum = 0.0f64;
    let mut n_samp = 0.0f64;
    for _ in 0..n_steps {
        pulse.supply_step(&mut dish, mech.dt);
        let _ = spatial_dish_step(
            &mut pop,
            &mut dish,
            &mech,
            react,
            &transport,
            &growth,
            &fission,
            true,
            0.0,
            0.0,
        );
        if let Some(i) = pop.individuals.iter().find(|i| i.mesh.alive) {
            a_sum += i.mesh.interior.a;
            r_sum += i.mesh.interior.r;
            r_max = r_max.max(i.mesh.interior.r as f64);
            let cf = c_free(&i.mesh);
            let cb = bound_catalyst_mass(&i.mesh);
            cf_sum += cf;
            cb_sum += cb;
            ct_sum += i.mesh.interior.c;
            tmpl_sum += i.mesh.templates.len() as f64;
            n_samp += 1.0;
        }
        if pop.individuals.is_empty() {
            break;
        }
    }
    let obs = observe_spatial_dish(&pop, &dish);
    let living = pop.individuals.iter().find(|i| i.mesh.alive);
    let (alive, death_reason, mass1) = if let Some(i) = living {
        (true, None, i.mesh.total_structural_mass())
    } else if let Some(i) = pop.individuals.first() {
        (false, i.mesh.death_reason.clone(), i.mesh.total_structural_mass())
    } else {
        (false, Some("extinct".into()), 0.0)
    };
    let denom = n_samp.max(1.0);
    ArmObs {
        name,
        max_gen: obs.max_gen,
        fissions: obs.fissions,
        alive,
        death_reason,
        mass0,
        mass1,
        mass_ratio: mass1 / mass0.max(EPS),
        a_mean: a_sum / denom,
        r_mean: r_sum / denom,
        r_max_seen: r_max,
        c_free_mean: cf_sum / denom,
        c_bound_mean: cb_sum / denom,
        c_tot_mean: ct_sum / denom,
        bound_fraction_mean: {
            let cf = cf_sum / denom;
            let cb = cb_sum / denom;
            cb / (cb + cf.max(EPS))
        },
        template_count_mean: tmpl_sum / denom,
        steps: n_steps,
    }
}

fn classify(arms: &[ArmObs]) -> (ZeroGenBlocker, String) {
    let full = arms.iter().find(|a| a.name == "network_full").unwrap();
    let bind_off = arms.iter().find(|a| a.name == "binding_disabled").unwrap();
    let expr_off = arms.iter().find(|a| a.name == "expression_disabled").unwrap();
    let no_tmpl = arms.iter().find(|a| a.name == "templates_absent").unwrap();
    let reserve = arms.iter().find(|a| a.name == "d091_reserve_control").unwrap();

    // Harness/horizon: if reserve control also fails to grow/fission, ecology/horizon issue.
    if reserve.max_gen == 0 && reserve.mass_ratio < 1.20 && full.max_gen == 0 {
        if reserve.alive && full.alive {
            return (
                ZeroGenBlocker::EcologyHorizonTooShort,
                "D-091 reserve control also fails fission under matched H horizon".into(),
            );
        }
        if !reserve.alive && !full.alive {
            return (
                ZeroGenBlocker::NumericalOrHarnessDefect,
                "both reserve control and network arms die without fission".into(),
            );
        }
    }

    // If disabling binding restores growth/fission while full does not → sequestration.
    let bind_rescues = bind_off.mass_ratio > full.mass_ratio + 0.08
        || bind_off.max_gen > full.max_gen
        || (bind_off.fissions > full.fissions);
    let high_bound = full.bound_fraction_mean > 0.45 || full.c_free_mean < 0.25 * full.c_tot_mean;
    if bind_rescues && high_bound {
        return (
            ZeroGenBlocker::CatalystSequestration,
            "binding-off restores growth/fission; free catalyst suppressed when bound".into(),
        );
    }

    // Templates present / expression off vs absent: copying cost if no_tmpl or expr_off grow better.
    let copy_cost = (no_tmpl.mass_ratio > full.mass_ratio + 0.08
        || expr_off.mass_ratio > full.mass_ratio + 0.08)
        && full.template_count_mean > 0.0
        && full.a_mean + 1e-9 < no_tmpl.a_mean;
    if copy_cost && !high_bound {
        return (
            ZeroGenBlocker::TemplateCopyingCost,
            "removing templates or expression raises A/growth relative to full network".into(),
        );
    }

    // Reserve charging: full has low R while bind_off/reserve have higher R and growth.
    if full.r_max_seen < 0.35 * reserve.r_max_seen.max(EPS)
        && full.mass_ratio < 1.15
        && (bind_off.r_max_seen > full.r_max_seen * 1.3 || reserve.r_max_seen > full.r_max_seen * 1.3)
    {
        return (
            ZeroGenBlocker::ReserveChargingSuppression,
            "network full suppresses R peak vs binding-off/reserve control".into(),
        );
    }

    if full.mass_ratio < 1.15
        && (bind_off.mass_ratio > 1.25 || reserve.mass_ratio > 1.25 || no_tmpl.mass_ratio > 1.25)
    {
        return (
            ZeroGenBlocker::GrowthSuppression,
            "structural mass growth suppressed under full network vs controls".into(),
        );
    }

    if full.max_gen == 0 && reserve.max_gen >= 1 {
        return (
            ZeroGenBlocker::OtherNetworkCost,
            "reserve control completes generations; full network does not".into(),
        );
    }

    if full.max_gen == 0 && full.alive && full.mass_ratio >= 1.20 {
        return (
            ZeroGenBlocker::EcologyHorizonTooShort,
            "mass grows but fission never completes within campaign horizon".into(),
        );
    }

    (
        ZeroGenBlocker::OtherNetworkCost,
        "no single dominant axis; residual network metabolic burden".into(),
    )
}

/// Run matched zero-generation causality audit; write JSON under `out`.
pub fn run_zero_generation_audit(out: &Path) -> Result<Value, String> {
    fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let reserve = selected_reserve();
    let t_gen = 2.0 / reserve.k_release.max(1e-9);
    let tmpl = TemplateParams::derived(t_gen);
    let net = selected_network(&reserve);
    let n_steps = 8_000;

    let mut arms = Vec::new();

    // 1) Full D-093 network expression
    {
        let react = with_network(ReactionParams::default(), reserve, tmpl, net);
        let ind = seed_network_org(FOUNDER_H, 4, 42);
        arms.push(run_arm("network_full", ind, &react, n_steps));
    }
    // 2) Network binding disabled
    {
        let react = with_network(
            ReactionParams::default(),
            reserve,
            tmpl,
            net.with_binding_off(),
        );
        let ind = seed_network_org(FOUNDER_H, 4, 43);
        arms.push(run_arm("binding_disabled", ind, &react, n_steps));
    }
    // 3) Templates present, network expression disabled (network.enable=false)
    {
        let mut net_off = net;
        net_off.enable = false;
        let mut react = with_network(ReactionParams::default(), reserve, tmpl, net_off);
        react.template.enable = true;
        react.template.enable_binding = false; // no D-092 motif expression
        let ind = seed_network_org(FOUNDER_H, 4, 44);
        arms.push(run_arm("expression_disabled", ind, &react, n_steps));
    }
    // 4) Templates completely absent (network stamp, polymer empty, network off)
    {
        let mut net_off = net;
        net_off.enable = false;
        let mut react = with_network(ReactionParams::default(), reserve, tmpl, net_off);
        react.template.enable = false;
        let mut ind = seed_network_org(FOUNDER_H, 0, 45);
        ind.mesh.templates.clear();
        arms.push(run_arm("templates_absent", ind, &react, n_steps));
    }
    // 5) D-091 reserve organism control
    {
        let mut react = ReactionParams::default();
        react.reserve = reserve;
        react.reserve.enable = true;
        react.composition.enable = false;
        react.template.enable = false;
        react.network.enable = false;
        let ind = seed_reserve_org(46, &reserve);
        arms.push(run_arm("d091_reserve_control", ind, &react, n_steps));
    }

    let (blocker, rationale) = classify(&arms);
    let arm_json: Vec<Value> = arms
        .iter()
        .map(|a| {
            json!({
                "name": a.name,
                "max_gen": a.max_gen,
                "fissions": a.fissions,
                "alive": a.alive,
                "death_reason": a.death_reason,
                "mass0": a.mass0,
                "mass1": a.mass1,
                "mass_ratio": a.mass_ratio,
                "a_mean": a.a_mean,
                "r_mean": a.r_mean,
                "r_max_seen": a.r_max_seen,
                "c_free_mean": a.c_free_mean,
                "c_bound_mean": a.c_bound_mean,
                "c_tot_mean": a.c_tot_mean,
                "bound_fraction_mean": a.bound_fraction_mean,
                "template_count_mean": a.template_count_mean,
                "steps": a.steps,
            })
        })
        .collect();

    let report = json!({
        "audit": "d093_zero_generation_causality",
        "schema_note": "uses D-093 network schema organisms vs D-091 reserve control",
        "rho_network": RHO_NETWORK,
        "n_steps": n_steps,
        "arms": arm_json,
        "first_causal_blocker": blocker.as_str(),
        "rationale": rationale,
        "defect_conclusion_if_implementation": "D094_D093_REPRODUCTION_COUPLING_DEFECT",
        "scientific_route_if_network_cost": "proceed to distributed autocatalytic-set architecture",
    });
    let path = out.join("audit.json");
    fs::write(&path, serde_json::to_string_pretty(&report).unwrap()).map_err(|e| e.to_string())?;
    Ok(report)
}
