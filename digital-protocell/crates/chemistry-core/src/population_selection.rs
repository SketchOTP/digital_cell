//! D-089 shared-dish competition — observer metrics only (no fitness controller).

use crate::catalyst_composition::{composition_z, set_composition_from_z};
use crate::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use crate::mesh_fission::FissionParams;
use crate::mesh_growth::{GrowthParams, GrowthLedger, merge_growth_into_reaction};
use crate::mesh_mechanics::{mechanics_step, remesh, MechParams};
use crate::mesh_fission::{topology_step, try_local_fission};
use crate::mesh_population::{MeshIndividual, MeshPopulation, PopStepLedger};
use crate::mesh_reactions::{evaluate_death, reactions_step, ReactionParams};
use crate::mesh_transport::{transport_step, TransportParams};
use serde::{Deserialize, Serialize};

/// Finite shared resource bath (mass pools). No population cap or culling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedBath {
    pub n_mass: f64,
    pub f_mass: f64,
    pub w_mass: f64,
    pub volume: f64,
    pub supply_n: f64,
    pub supply_f: f64,
}

impl SharedBath {
    pub fn resource_limited() -> Self {
        // Moderate stock: division possible, scarcity binds mid-campaign.
        Self {
            n_mass: 220.0,
            f_mass: 220.0,
            w_mass: 0.0,
            volume: 100.0,
            supply_n: 65.0,
            supply_f: 65.0,
        }
    }

    pub fn construction_demand() -> Self {
        Self {
            n_mass: 1600.0,
            f_mass: 1600.0,
            w_mass: 0.0,
            volume: 100.0,
            supply_n: 90.0,
            supply_f: 90.0,
        }
    }

    pub fn neutral() -> Self {
        Self {
            n_mass: 400.0,
            f_mass: 400.0,
            w_mass: 0.0,
            volume: 100.0,
            supply_n: 60.0,
            supply_f: 60.0,
        }
    }

    pub fn conc_n(&self) -> f64 {
        self.n_mass / self.volume.max(1e-9)
    }
    pub fn conc_f(&self) -> f64 {
        self.f_mass / self.volume.max(1e-9)
    }
    pub fn conc_w(&self) -> f64 {
        self.w_mass / self.volume.max(1e-9)
    }

    pub fn sync_mesh_exterior(&self, mesh: &mut MaterialMesh) {
        mesh.exterior.n = self.conc_n();
        mesh.exterior.f = self.conc_f();
        mesh.exterior.w = self.conc_w();
        mesh.exterior.c = 0.0;
        mesh.exterior.a = 0.0;
    }

    pub fn supply_step(&mut self, dt: f64) {
        self.n_mass = (self.n_mass + self.supply_n * dt).max(0.0);
        self.f_mass = (self.f_mass + self.supply_f * dt).max(0.0);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DishSnapshot {
    pub living: usize,
    pub deaths: usize,
    pub fissions: usize,
    pub biomass: f64,
    pub freq_c_h: f64,
    pub mean_z: f64,
    pub descendants_h: usize,
    pub descendants_b: usize,
    pub bath_n: f64,
    pub bath_f: f64,
}

/// Seed equal populations with prescribed composition z and shared exterior.
pub fn seed_competition(
    n_each: usize,
    z_h: f64,
    z_b: f64,
    radius: f64,
    seed: u64,
    bath: &SharedBath,
) -> MeshPopulation {
    let mut pop = MeshPopulation {
        individuals: Vec::new(),
        next_lineage: 1,
        fission_log: Vec::new(),
    };
    let r = if radius < 10.0 { 10.0 } else { radius };
    for i in 0..n_each {
        let mut mesh = seed_composed(r, seed.wrapping_add(i as u64), z_h, bath);
        bipolar_elongate(&mut mesh, 1.25);
        // Slight spatial offset so organisms don't stack.
        let ang = (i as f64) * std::f64::consts::TAU / (n_each as f64).max(1.0);
        let cx = 18.0 * ang.cos();
        let cy = 18.0 * ang.sin();
        for v in &mut mesh.vertices {
            v[0] += cx;
            v[1] += cy;
        }
        let birth = mesh.total_structural_mass();
        let id = pop.next_lineage;
        pop.next_lineage += 1;
        pop.individuals.push(MeshIndividual {
            mesh,
            lineage_id: id,
            generation: 0,
            birth_mass: birth,
            clade: 1,
        });
    }
    for i in 0..n_each {
        let mut mesh = seed_composed(
            r,
            seed.wrapping_add(1000 + i as u64),
            z_b,
            bath,
        );
        bipolar_elongate(&mut mesh, 1.25);
        let ang = std::f64::consts::PI
            + (i as f64) * std::f64::consts::TAU / (n_each as f64).max(1.0);
        let cx = 18.0 * ang.cos();
        let cy = 18.0 * ang.sin();
        for v in &mut mesh.vertices {
            v[0] += cx;
            v[1] += cy;
        }
        let birth = mesh.total_structural_mass();
        let id = pop.next_lineage;
        pop.next_lineage += 1;
        pop.individuals.push(MeshIndividual {
            mesh,
            lineage_id: id,
            generation: 0,
            birth_mass: birth,
            clade: -1,
        });
    }
    pop
}

fn bipolar_elongate(mesh: &mut MaterialMesh, scale: f64) {
    let c = mesh.centroid();
    for v in &mut mesh.vertices {
        let dx = v[0] - c[0];
        v[0] = c[0] + dx * scale;
    }
}

pub fn seed_composed(radius: f64, seed: u64, z: f64, bath: &SharedBath) -> MaterialMesh {
    let n = 24 + ((seed % 3) as usize);
    let mut interior = LumpedChem {
        c: 0.8,
        a: 0.5,
        n: 0.4,
        f: 0.4,
        w: 0.1,
        tracer_c: 0.0,
        c_h: 0.0,
        c_b: 0.0,
        r: 0.0,
    };
    set_composition_from_z(&mut interior, z);
    let exterior = LumpedChem {
        c: 0.0,
        a: 0.0,
        n: bath.conc_n(),
        f: bath.conc_f(),
        w: bath.conc_w(),
        tracer_c: 0.0,
        c_h: 0.0,
        c_b: 0.0,
        r: 0.0,
    };
    MaterialMesh::seed_regular(
        n,
        radius,
        40.0,
        40.0,
        DEFAULT_RHO_S,
        0.7,
        interior,
        exterior,
        5.0,
    )
}

/// One population step against a finite shared bath (no fitness selection).
pub fn dish_step(
    pop: &mut MeshPopulation,
    bath: &mut SharedBath,
    mech: &MechParams,
    react: &ReactionParams,
    transport: &TransportParams,
    growth: &GrowthParams,
    fission: &FissionParams,
    enable_mech: bool,
    damage_membrane_frac: f64,
    damage_struct_frac: f64,
) -> PopStepLedger {
    use crate::mesh_reactions::{apply_membrane_damage, apply_structural_damage};
    static STEP_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let tick = STEP_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    bath.supply_step(mech.dt);
    let mut led = PopStepLedger::default();
    let mut newborns: Vec<MeshIndividual> = Vec::new();
    let mut next_id = pop.next_lineage;

    for ind in pop.individuals.iter_mut() {
        if !ind.mesh.alive {
            continue;
        }
        bath.sync_mesh_exterior(&mut ind.mesh);
        let tled = transport_step(&mut ind.mesh, transport, mech.dt);
        // Deplete shared bath by influx mass.
        bath.n_mass = (bath.n_mass - tled.n_in).max(0.0);
        bath.f_mass = (bath.f_mass - tled.f_in).max(0.0);
        bath.w_mass = (bath.w_mass + tled.w_out).max(0.0);

        let r = reactions_step(&mut ind.mesh, react, mech.dt, true, true);
        let g = growth_step_local(&mut ind.mesh, react, growth, mech.dt);
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

        if damage_membrane_frac > 0.0 && tick % 100 == 0 {
            let _ = apply_membrane_damage(&mut ind.mesh, damage_membrane_frac);
        }
        if damage_struct_frac > 0.0 && tick % 100 == 0 {
            let _ = apply_structural_damage(&mut ind.mesh, damage_struct_frac);
        }

        if enable_mech {
            mechanics_step(&mut ind.mesh, mech);
            remesh(&mut ind.mesh);
        }
        if tick % 10 == 0 {
            let _ = topology_step(&mut ind.mesh, fission);
        }
        let grown_enough =
            ind.mesh.total_structural_mass() >= 1.35 * ind.birth_mass.max(1e-9);
        // Pinch search is O(n²); attempt on a stride once grown.
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

fn growth_step_local(
    mesh: &mut MaterialMesh,
    react: &ReactionParams,
    growth: &GrowthParams,
    dt: f64,
) -> GrowthLedger {
    crate::mesh_growth::growth_step(mesh, react, growth, dt)
}

pub fn observe_dish(pop: &MeshPopulation, bath: &SharedBath, z_h_threshold: f64) -> DishSnapshot {
    let mut living = 0usize;
    let mut deaths = 0usize;
    let mut biomass = 0.0;
    let mut ch = 0.0;
    let mut cb = 0.0;
    let mut zsum = 0.0;
    let mut descendants_h = 0usize;
    let mut descendants_b = 0usize;
    for ind in &pop.individuals {
        if !ind.mesh.alive {
            deaths += 1;
            continue;
        }
        living += 1;
        biomass += ind.mesh.total_structural_mass();
        let a = ind.mesh.area().max(1e-9);
        ch += ind.mesh.interior.c_h.max(0.0) * a;
        cb += ind.mesh.interior.c_b.max(0.0) * a;
        let z = composition_z(ind.mesh.interior.c_h, ind.mesh.interior.c_b);
        zsum += z;
        // Prefer observer clade (founder inheritance); fall back to composition threshold.
        if ind.clade > 0 || (ind.clade == 0 && z >= z_h_threshold) {
            descendants_h += 1;
        } else if ind.clade < 0 || (ind.clade == 0 && z <= -z_h_threshold) {
            descendants_b += 1;
        }
    }
    let tot = ch + cb;
    DishSnapshot {
        living,
        deaths,
        fissions: pop.fission_log.len(),
        biomass,
        freq_c_h: if tot > 1e-15 { ch / tot } else { 0.5 },
        mean_z: if living > 0 {
            zsum / living as f64
        } else {
            0.0
        },
        descendants_h,
        descendants_b,
        bath_n: bath.n_mass,
        bath_f: bath.f_mass,
    }
}
