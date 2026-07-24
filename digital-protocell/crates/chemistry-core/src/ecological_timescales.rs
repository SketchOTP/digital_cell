//! D-090 ecological timescale observers and Gate-3 contracts.

use crate::mesh_fission::FissionParams;
use crate::mesh_growth::{growth_step, local_maintenance_a_rate, GrowthParams};
use crate::mesh_mechanics::{mechanics_step, remesh, MechParams};
use crate::mesh_population::MeshIndividual;
use crate::mesh_reactions::{
    apply_membrane_damage, apply_structural_damage, evaluate_death, reactions_step, ReactionParams,
};
use crate::mesh_transport::TransportParams;
use crate::spatial_shared_dish::SpatialDish;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimescaleReport {
    pub t_limit: Option<f64>,
    pub t_damage: Option<f64>,
    pub t_growth10: Option<f64>,
    pub t_fission: Option<f64>,
    pub frac_growth_a_from_post_transfer: f64,
    pub frac_a_spent_repair: f64,
    pub inherited_a_fraction_of_fission_cost: f64,
    pub founder_viability: f64,
    pub resource_limited_before_fission: bool,
    pub scarcity_before_growth10: bool,
    pub pass_resource_limited_h: bool,
    pub pass_construction_demand_b: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemandEstimate {
    pub m_maintenance_nf: f64,
    pub g_fission_nf: f64,
    pub t_f_median: f64,
}

/// Rough maintenance N+F demand rate from local maintenance A and stoichiometry observer.
pub fn estimate_maintenance_nf_rate(ind: &MeshIndividual, react: &ReactionParams) -> f64 {
    let mesh = &ind.mesh;
    let mut a_maint = 0.0;
    for i in 0..mesh.n() {
        a_maint += local_maintenance_a_rate(mesh, i, react);
    }
    // Activation: N+F → A; observer treats 1 N + 1 F ≈ 1 A mass (order-of-magnitude ecology sizing).
    a_maint * 2.0
}

/// Estimate M, G, T_f from isolated founders under a rich calibration ecology.
pub fn estimate_demands(
    founders: &[MeshIndividual],
    _dish0: &SpatialDish,
    react: &ReactionParams,
    growth: &GrowthParams,
    mech: &MechParams,
    transport: &TransportParams,
    fission: &FissionParams,
    max_steps: usize,
) -> DemandEstimate {
    let mut m_sum = 0.0;
    let mut g_vals = Vec::new();
    let mut tf_vals = Vec::new();
    for (fi, f0) in founders.iter().enumerate() {
        m_sum += estimate_maintenance_nf_rate(f0, react);
        // Rich calibration dish so fission is reachable for demand measurement.
        let mut dish = SpatialDish::new(
            8,
            8,
            2.5,
            [0.0, 0.0],
            800.0,
            800.0,
            60.0,
            60.0,
            3.0,
        );
        let mut ind = f0.clone();
        let c = ind.mesh.centroid();
        let tx = dish.origin[0] + dish.nx as f64 * dish.dx * 0.5 - c[0] + (fi as f64) * 0.01;
        let ty = dish.origin[1] + dish.ny as f64 * dish.dx * 0.5 - c[1];
        for v in &mut ind.mesh.vertices {
            v[0] += tx;
            v[1] += ty;
        }
        let birth = ind.birth_mass.max(ind.mesh.total_structural_mass());
        ind.birth_mass = birth;
        let mut nf_consumed = 0.0;
        let mut t_f = None;
        let mut t = 0.0;
        let mut maint_integrated = 0.0;
        for _ in 0..max_steps {
            if !ind.mesh.alive {
                break;
            }
            maint_integrated += estimate_maintenance_nf_rate(&ind, react) * mech.dt;
            dish.tick += 1;
            dish.supply_step(mech.dt);
            dish.diffuse(mech.dt);
            let tled = dish.transport_organism(&mut ind.mesh, transport, mech.dt);
            nf_consumed += tled.n_in.max(0.0) + tled.f_in.max(0.0);
            let _ = reactions_step(&mut ind.mesh, react, mech.dt, true, true);
            let _ = growth_step(&mut ind.mesh, react, growth, mech.dt);
            mechanics_step(&mut ind.mesh, mech);
            remesh(&mut ind.mesh);
            let grown = ind.mesh.total_structural_mass() >= 1.35 * birth.max(1e-9);
            if grown && dish.tick % 15 == 0 {
                if let Some((d1, d2, _)) = crate::mesh_fission::try_local_fission(&ind.mesh, fission)
                {
                    let _ = (d1, d2);
                    t_f = Some(t);
                    break;
                }
            }
            evaluate_death(&mut ind.mesh);
            t += mech.dt;
        }
        let g_i = (nf_consumed - maint_integrated).max(nf_consumed * 0.25).max(40.0);
        g_vals.push(g_i);
        if let Some(tf) = t_f {
            tf_vals.push(tf);
        }
    }
    let n = founders.len().max(1) as f64;
    tf_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let t_f_median = if tf_vals.is_empty() {
        1.0 / react.k_c_turn.max(1e-3) * 1.5
    } else {
        tf_vals[tf_vals.len() / 2]
    };
    let g_mean = g_vals.iter().sum::<f64>() / n;
    DemandEstimate {
        m_maintenance_nf: m_sum.max(8.0),
        g_fission_nf: g_mean.max(40.0),
        t_f_median: t_f_median.max(20.0),
    }
}

fn resource_limiting(dish: &SpatialDish, init_n: f64, init_f: f64) -> bool {
    // Scarcity: bath below 50% of initial inventory (or concentration below maintenance-friendly floor).
    dish.total_n() < 0.5 * init_n || dish.total_f() < 0.5 * init_f
}

/// Probe timescales for one ecology candidate with a founder cohort (no mutation).
pub fn probe_timescales(
    founders: &[MeshIndividual],
    dish0: &SpatialDish,
    react: &ReactionParams,
    growth: &GrowthParams,
    mech: &MechParams,
    transport: &TransportParams,
    fission: &FissionParams,
    damage_frac: f64,
    damage_times: &[f64],
    max_steps: usize,
    ecology: &str,
) -> TimescaleReport {
    let init_n = dish0.total_n();
    let init_f = dish0.total_f();
    let mut dish = dish0.clone();
    let mut pop: Vec<MeshIndividual> = founders.to_vec();
    let mut ages = vec![0.0; pop.len()];
    let m0: Vec<f64> = pop.iter().map(|i| i.mesh.total_structural_mass()).collect();
    let a0: Vec<f64> = pop
        .iter()
        .map(|i| i.mesh.interior.a.max(0.0) * i.mesh.area())
        .collect();
    let mut a_from_post = 0.0;
    let mut a_total_growth = 0.0;
    let mut a_repair = 0.0;
    let mut a_prod_all = 0.0;
    let mut t_limit = None;
    let mut t_damage = None;
    let mut t_growth10 = None;
    let mut t_fission = None;
    let mut t = 0.0;
    let mut damage_idx = 0usize;
    let mut alive0 = pop.iter().filter(|i| i.mesh.alive).count();

    for _ in 0..max_steps {
        dish.tick += 1;
        dish.supply_step(mech.dt);
        dish.diffuse(mech.dt);

        // Scheduled damage events (construction ecology).
        if damage_frac > 0.0 && damage_idx < damage_times.len() && t + 1e-12 >= damage_times[damage_idx]
        {
            for ind in &mut pop {
                if ind.mesh.alive {
                    let _ = apply_membrane_damage(&mut ind.mesh, damage_frac);
                    let _ = apply_structural_damage(&mut ind.mesh, damage_frac * 0.6);
                }
            }
            if t_damage.is_none() {
                t_damage = Some(t);
            }
            damage_idx += 1;
        }

        let mut newborns = Vec::new();
        for (ii, ind) in pop.iter_mut().enumerate() {
            if !ind.mesh.alive {
                continue;
            }
            let tled = dish.transport_organism(&mut ind.mesh, transport, mech.dt);
            let _ = tled;
            let r = reactions_step(&mut ind.mesh, react, mech.dt, true, true);
            a_prod_all += r.a_produced;
            // Proxy: A produced after transfer counts as post-transfer activation.
            a_from_post += r.a_produced;
            let g = growth_step(&mut ind.mesh, react, growth, mech.dt);
            a_total_growth += g.a_consumed_growth;
            // Repair proxy: membrane production under damaged occupancy.
            a_repair += r.m_produced / react.yield_a_to_m.max(1e-15) * 0.25;
            mechanics_step(&mut ind.mesh, mech);
            remesh(&mut ind.mesh);
            ages[ii] += mech.dt;
            let grown = ind.mesh.total_structural_mass() >= 1.35 * ind.birth_mass.max(1e-9);
            if grown && dish.tick % 15 == 0 {
                if let Some((d1, d2, ev)) =
                    crate::mesh_fission::try_local_fission(&ind.mesh, fission)
                {
                    let _ = ev;
                    ind.mesh.alive = false;
                    ind.mesh.death_reason = Some("fissioned".into());
                    if t_fission.is_none() {
                        t_fission = Some(t);
                    }
                    newborns.push(MeshIndividual {
                        birth_mass: d1.total_structural_mass(),
                        mesh: d1,
                        lineage_id: 0,
                        generation: ind.generation + 1,
                        clade: ind.clade,
                    });
                    newborns.push(MeshIndividual {
                        birth_mass: d2.total_structural_mass(),
                        mesh: d2,
                        lineage_id: 0,
                        generation: ind.generation + 1,
                        clade: ind.clade,
                    });
                }
            }
            evaluate_death(&mut ind.mesh);
        }
        pop.extend(newborns);
        ages.resize(pop.len(), 0.0);

        if t_limit.is_none() && resource_limiting(&dish, init_n, init_f) {
            t_limit = Some(t);
        }
        if t_growth10.is_none() {
            let grew = pop.iter().zip(m0.iter()).any(|(ind, m)| {
                ind.mesh.alive && ind.mesh.total_structural_mass() >= m * 1.10
            });
            // Compare only original founders' masses when still present.
            let grew_f = founders.iter().enumerate().any(|(i, _)| {
                pop.get(i)
                    .map(|ind| {
                        ind.generation == 0
                            && ind.mesh.alive
                            && ind.mesh.total_structural_mass() >= m0[i] * 1.10
                    })
                    .unwrap_or(false)
            });
            if grew || grew_f {
                t_growth10 = Some(t);
            }
        }

        if t_fission.is_some() {
            break;
        }
        t += mech.dt;
        if pop.iter().all(|i| !i.mesh.alive) {
            break;
        }
    }

    let alive1 = pop
        .iter()
        .filter(|i| i.mesh.alive && i.generation == 0)
        .count();
    let viability = if alive0 == 0 {
        0.0
    } else {
        alive1 as f64 / alive0 as f64
    };

    let inherited_a: f64 = a0.iter().sum();
    let fission_a_cost = a_total_growth.max(inherited_a);
    let inherited_frac = if fission_a_cost > 1e-12 {
        inherited_a / (inherited_a + a_from_post).max(1e-12)
    } else {
        0.0
    };
    let frac_post = if (a_from_post + inherited_a) > 1e-12 {
        a_from_post / (a_from_post + inherited_a)
    } else {
        0.0
    };
    let frac_repair = if a_prod_all > 1e-12 {
        (a_repair / a_prod_all).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let resource_before_fission = match (t_limit, t_fission) {
        (Some(tl), Some(tf)) => tl < tf,
        (Some(_), None) => true,
        _ => false,
    };
    let scarcity_before_g10 = match (t_limit, t_growth10) {
        (Some(tl), Some(tg)) => tl < tg,
        (Some(_), None) => true,
        _ => false,
    };

    let pass_h = ecology == "H"
        && resource_before_fission
        && scarcity_before_g10
        && frac_post >= 0.80
        && inherited_frac < 0.10
        && viability >= 0.80
        && t_fission.map(|tf| t_limit.map(|tl| tl < tf).unwrap_or(false)).unwrap_or(true);

    let pass_b = ecology == "B"
        && t_damage.is_some()
        && frac_repair >= 0.25
        && viability >= 0.80
        && t_fission
            .map(|tf| t_damage.map(|td| td < tf).unwrap_or(false))
            .unwrap_or(true);

    TimescaleReport {
        t_limit,
        t_damage,
        t_growth10,
        t_fission,
        frac_growth_a_from_post_transfer: frac_post,
        frac_a_spent_repair: frac_repair,
        inherited_a_fraction_of_fission_cost: inherited_frac,
        founder_viability: viability,
        resource_limited_before_fission: resource_before_fission,
        scarcity_before_growth10: scarcity_before_g10,
        pass_resource_limited_h: pass_h,
        pass_construction_demand_b: pass_b,
    }
}
