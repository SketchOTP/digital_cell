//! D-090 spatial competition step and selection campaign helpers.

use crate::catalyst_composition::composition_z;
use crate::mesh_fission::{try_local_fission, FissionParams};
use crate::mesh_growth::{growth_step, merge_growth_into_reaction, GrowthParams};
use crate::mesh_mechanics::{mechanics_step, remesh, MechParams};
use crate::mesh_population::{MeshIndividual, MeshPopulation, PopStepLedger};
use crate::mesh_reactions::{
    apply_membrane_damage, apply_structural_damage, evaluate_death, reactions_step, ReactionParams,
};
use crate::mesh_transport::TransportParams;
use crate::spatial_shared_dish::SpatialDish;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DishObs {
    pub living: usize,
    pub deaths: usize,
    pub fissions: usize,
    pub biomass: f64,
    pub freq_c_h_mass: f64,
    pub freq_c_h_count: f64,
    pub mean_z: f64,
    pub descendants_h: usize,
    pub descendants_b: usize,
    pub dish_n: f64,
    pub dish_f: f64,
    pub max_gen: u32,
}

/// One population step on a spatial dish. Damage is scheduled by caller via flags.
pub fn spatial_dish_step(
    pop: &mut MeshPopulation,
    dish: &mut SpatialDish,
    mech: &MechParams,
    react: &ReactionParams,
    transport: &TransportParams,
    growth: &GrowthParams,
    fission: &FissionParams,
    enable_mech: bool,
    apply_damage_m: f64,
    apply_damage_s: f64,
) -> PopStepLedger {
    dish.tick += 1;
    dish.supply_step(mech.dt);
    dish.diffuse(mech.dt);

    let mut led = PopStepLedger::default();
    let mut newborns: Vec<MeshIndividual> = Vec::new();
    let mut next_id = pop.next_lineage;
    let tick = dish.tick;

    if apply_damage_m > 0.0 || apply_damage_s > 0.0 {
        for ind in pop.individuals.iter_mut() {
            if !ind.mesh.alive {
                continue;
            }
            if apply_damage_m > 0.0 {
                let _ = apply_membrane_damage(&mut ind.mesh, apply_damage_m);
            }
            if apply_damage_s > 0.0 {
                let _ = apply_structural_damage(&mut ind.mesh, apply_damage_s);
            }
        }
    }

    for ind in pop.individuals.iter_mut() {
        if !ind.mesh.alive {
            continue;
        }
        let _ = dish.transport_organism(&mut ind.mesh, transport, mech.dt);
        let r = reactions_step(&mut ind.mesh, react, mech.dt, true, true);
        let g = growth_step(&mut ind.mesh, react, growth, mech.dt);
        merge_growth_into_reaction(&mut led.reactions, &g);
        led.reactions.a_produced += r.a_produced;
        led.reactions.m_produced += r.m_produced;
        led.reactions.n_consumed += r.n_consumed;
        led.reactions.f_consumed += r.f_consumed;
        led.reactions.w_produced += r.w_produced;
        led.reactions.c_produced += r.c_produced;
        led.reactions.c_turned += r.c_turned;
        led.reactions.composition.c_h_produced += r.composition.c_h_produced;
        led.reactions.composition.c_b_produced += r.composition.c_b_produced;
        led.reactions.composition.conversion_events += r.composition.conversion_events;
        led.growth.m_grown += g.m_grown;
        led.growth.a_consumed_growth += g.a_consumed_growth;
        led.growth.a_surplus_total += g.a_surplus_total;

        if enable_mech {
            mechanics_step(&mut ind.mesh, mech);
            remesh(&mut ind.mesh);
        }
        if tick % 10 == 0 {
            let _ = crate::mesh_fission::topology_step(&mut ind.mesh, fission);
        }
        let grown_enough = ind.mesh.total_structural_mass() >= 1.35 * ind.birth_mass.max(1e-9);
        if grown_enough && tick % 15 == 0 {
            if let Some((d1, d2, ev)) = try_local_fission(&ind.mesh, fission) {
                ind.mesh.alive = false;
                ind.mesh.death_reason = Some("fissioned".into());
                let gen = ind.generation + 1;
                let id_a = next_id;
                next_id += 1;
                let id_b = next_id;
                next_id += 1;
                let clade = ind.clade;
                newborns.push(MeshIndividual {
                    birth_mass: d1.total_structural_mass(),
                    mesh: d1,
                    lineage_id: id_a,
                    generation: gen,
                    clade,
                });
                newborns.push(MeshIndividual {
                    birth_mass: d2.total_structural_mass(),
                    mesh: d2,
                    lineage_id: id_b,
                    generation: gen,
                    clade,
                });
                pop.fission_log.push(ev);
                led.fissions += 1;
            }
        }
        evaluate_death(&mut ind.mesh);
    }
    pop.next_lineage = next_id;
    pop.individuals.extend(newborns);
    led
}

pub fn observe_spatial_dish(pop: &MeshPopulation, dish: &SpatialDish) -> DishObs {
    let mut living = 0usize;
    let mut deaths = 0usize;
    let mut biomass = 0.0;
    let mut ch = 0.0;
    let mut cb = 0.0;
    let mut zsum = 0.0;
    let mut descendants_h = 0usize;
    let mut descendants_b = 0usize;
    let mut count_h = 0usize;
    let mut count_b = 0usize;
    let mut max_gen = 0u32;
    for ind in &pop.individuals {
        if !ind.mesh.alive {
            deaths += 1;
            continue;
        }
        living += 1;
        max_gen = max_gen.max(ind.generation);
        biomass += ind.mesh.total_structural_mass();
        let a = ind.mesh.area().max(1e-9);
        ch += ind.mesh.interior.c_h.max(0.0) * a;
        cb += ind.mesh.interior.c_b.max(0.0) * a;
        zsum += composition_z(ind.mesh.interior.c_h, ind.mesh.interior.c_b);
        if ind.clade > 0 {
            descendants_h += 1;
            count_h += 1;
        } else if ind.clade < 0 {
            descendants_b += 1;
            count_b += 1;
        } else {
            let z = composition_z(ind.mesh.interior.c_h, ind.mesh.interior.c_b);
            if z >= 0.0 {
                count_h += 1;
            } else {
                count_b += 1;
            }
        }
    }
    let tot = ch + cb;
    let ct = (count_h + count_b).max(1) as f64;
    DishObs {
        living,
        deaths,
        fissions: pop.fission_log.len(),
        biomass,
        freq_c_h_mass: if tot > 1e-15 { ch / tot } else { 0.5 },
        freq_c_h_count: count_h as f64 / ct,
        mean_z: if living > 0 {
            zsum / living as f64
        } else {
            0.0
        },
        descendants_h,
        descendants_b,
        dish_n: dish.total_n(),
        dish_f: dish.total_f(),
        max_gen,
    }
}

/// Place preconditioned founders onto a competition dish without overwriting chemistry.
pub fn assemble_population(
    founders: Vec<MeshIndividual>,
    dish: &SpatialDish,
    ring_radius: f64,
) -> MeshPopulation {
    let n = founders.len().max(1);
    let cx = dish.origin[0] + dish.nx as f64 * dish.dx * 0.5;
    let cy = dish.origin[1] + dish.ny as f64 * dish.dx * 0.5;
    let mut pop = MeshPopulation {
        individuals: Vec::new(),
        next_lineage: 1,
        fission_log: Vec::new(),
    };
    for (i, mut ind) in founders.into_iter().enumerate() {
        let ang = (i as f64) * std::f64::consts::TAU / n as f64;
        let tx = cx + ring_radius * ang.cos();
        let ty = cy + ring_radius * ang.sin();
        let c = ind.mesh.centroid();
        for v in &mut ind.mesh.vertices {
            v[0] += tx - c[0];
            v[1] += ty - c[1];
        }
        dish.sync_mesh_exterior(&mut ind.mesh);
        ind.lineage_id = pop.next_lineage;
        pop.next_lineage += 1;
        pop.individuals.push(ind);
    }
    pop
}
