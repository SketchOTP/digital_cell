//! Multi-individual mesh population (bookkeeping containers only).
//!
//! No biological state machine fields (`ready_to_divide`, target size, etc.).
//! Observer-only lineage IDs may be attached after topology changes.

use crate::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use crate::mesh_fission::{topology_step, try_local_fission, FissionEvent, FissionParams};
use crate::mesh_growth::{growth_step, merge_growth_into_reaction, GrowthLedger, GrowthParams};
use crate::mesh_mechanics::{mechanics_step, remesh, MechParams};
use crate::mesh_reactions::{reactions_step, ReactionLedger, ReactionParams};
use crate::mesh_transport::{transport_step, TransportParams};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshIndividual {
    pub mesh: MaterialMesh,
    /// Observer-only lineage id (never enters chemistry/mechanics).
    pub lineage_id: u64,
    pub generation: u32,
    /// Mass at birth/seed — used only to gate fission until surplus growth occurred.
    pub birth_mass: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MeshPopulation {
    pub individuals: Vec<MeshIndividual>,
    pub next_lineage: u64,
    pub fission_log: Vec<FissionEvent>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PopStepLedger {
    pub reactions: ReactionLedger,
    pub growth: GrowthLedger,
    pub fissions: usize,
}

impl MeshPopulation {
    pub fn seed_one(radius: f64, seed: u64, exterior_scale: f64) -> Self {
        let n = 24 + ((seed % 3) as usize);
        let interior = LumpedChem {
            c: 0.8,
            a: 0.5,
            n: 0.4,
            f: 0.4,
            w: 0.1,
            tracer_c: 0.0,
        };
        let exterior = LumpedChem {
            c: 0.0,
            a: 0.0,
            n: 1.0 * exterior_scale,
            f: 1.0 * exterior_scale,
            w: 0.0,
            tracer_c: 0.0,
        };
        let mesh = MaterialMesh::seed_regular(
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
        let birth_mass = mesh.total_structural_mass();
        Self {
            individuals: vec![MeshIndividual {
                mesh,
                lineage_id: 1,
                generation: 0,
                birth_mass,
            }],
            next_lineage: 2,
            fission_log: Vec::new(),
        }
    }

    pub fn living_count(&self) -> usize {
        self.individuals.iter().filter(|i| i.mesh.alive).count()
    }

    pub fn step(
        &mut self,
        mech: &MechParams,
        react: &ReactionParams,
        transport: &TransportParams,
        growth: &GrowthParams,
        fission: &FissionParams,
        enable_mech: bool,
    ) -> PopStepLedger {
        let mut led = PopStepLedger::default();
        let mut newborns: Vec<MeshIndividual> = Vec::new();
        let mut next_id = self.next_lineage;
        // Fission attempts are local but O(n²) pinch search — evaluate periodically.
        static STEP_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let tick = STEP_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        for ind in self.individuals.iter_mut() {
            if !ind.mesh.alive {
                continue;
            }
            let _ = transport_step(&mut ind.mesh, transport, mech.dt);
            let r = reactions_step(&mut ind.mesh, react, mech.dt, true, true);
            let g = growth_step(&mut ind.mesh, react, growth, mech.dt);
            merge_growth_into_reaction(&mut led.reactions, &g);
            led.reactions.a_produced += r.a_produced;
            led.reactions.m_produced += r.m_produced;
            led.reactions.n_consumed += r.n_consumed;
            led.reactions.f_consumed += r.f_consumed;
            led.reactions.w_produced += r.w_produced;
            led.growth.m_grown += g.m_grown;
            led.growth.a_consumed_growth += g.a_consumed_growth;
            led.growth.a_surplus_total += g.a_surplus_total;

            if enable_mech {
                mechanics_step(&mut ind.mesh, mech);
                remesh(&mut ind.mesh);
            }
            if tick % 10 == 0 {
                let _ = topology_step(&mut ind.mesh, fission);
            }

            let grown_enough =
                ind.mesh.total_structural_mass() >= 1.35 * ind.birth_mass.max(1e-9);
            if grown_enough && tick % 25 == 0 {
                if let Some((d1, d2, ev)) = try_local_fission(&ind.mesh, fission) {
                    ind.mesh.alive = false;
                    ind.mesh.death_reason = Some("fissioned".into());
                    let gen = ind.generation + 1;
                    let id_a = next_id;
                    next_id += 1;
                    let id_b = next_id;
                    next_id += 1;
                    let m1 = d1.total_structural_mass();
                    let m2 = d2.total_structural_mass();
                    newborns.push(MeshIndividual {
                        mesh: d1,
                        lineage_id: id_a,
                        generation: gen,
                        birth_mass: m1,
                    });
                    newborns.push(MeshIndividual {
                        mesh: d2,
                        lineage_id: id_b,
                        generation: gen,
                        birth_mass: m2,
                    });
                    self.fission_log.push(ev);
                    led.fissions += 1;
                }
            }
        }
        self.next_lineage = next_id;
        self.individuals.extend(newborns);
        led
    }
}

/// Coupled single-mesh step used by assays.
pub fn coupled_step_growth(
    mesh: &mut MaterialMesh,
    mech: &MechParams,
    react: &ReactionParams,
    transport: &TransportParams,
    growth: &GrowthParams,
    fission: &FissionParams,
    enable_mech: bool,
    enable_fission: bool,
) -> (
    ReactionLedger,
    GrowthLedger,
    Option<(MaterialMesh, MaterialMesh, FissionEvent)>,
) {
    let _ = transport_step(mesh, transport, mech.dt);
    let mut r = reactions_step(mesh, react, mech.dt, true, true);
    let g = growth_step(mesh, react, growth, mech.dt);
    merge_growth_into_reaction(&mut r, &g);
    if enable_mech {
        mechanics_step(mesh, mech);
        remesh(mesh);
    }
    let _ = topology_step(mesh, fission);
    let split = if enable_fission {
        try_local_fission(mesh, fission)
    } else {
        None
    };
    (r, g, split)
}
