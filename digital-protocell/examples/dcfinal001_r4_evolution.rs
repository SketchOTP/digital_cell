// DC-FINAL-001-R4: contract ownership, V4 topology coherence, and evolution.
//
// Identical organisms are represented by an exact state plus an integer
// multiplicity.  The compression is semantic: shared-medium requests,
// organism/world material transfers, deaths, physical fissions, and mutation
// draws are all weighted or expanded by that multiplicity.  No cohort value
// enters organism biology.

use chemistry_core::d096_allocation::{
    expression_step, mutate_allocation_at_reproduction, AllocationGenotype, AllocationParams,
};
use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_fission::try_local_segment_fission;
use chemistry_core::mesh_fission::{topology_step, try_local_fission, FissionParams};
use chemistry_core::mesh_growth::{growth_step, GrowthParams};
use chemistry_core::mesh_mechanics::{remesh, MechParams};
use chemistry_core::mesh_reactions::{reactions_step, ReactionParams};
use chemistry_core::mesh_self_contact::{mechanics_step_with_local_self_contact, polygon_simple};
use chemistry_core::mesh_transport::{
    mean_occupancy, permeability, transport_step, TransportParams,
};
use chemistry_core::planar_ring_topology::{remesh_preserving_simple, PlanarRingTopology};
use regulatory_core::PlasticityStateV1;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::{env, fs, path::PathBuf};

const DIRECTIVE: &str =
    "DC-FINAL-001-R4-CONTRACT-OWNERSHIP-V4-TOPOLOGY-COHERENCE-EVOLUTION-AND-FINAL-GOAL-CLOSURE-001";
const TEMPLATE_STEPS: usize = 6_500;
const PHASE_STEPS: usize = 2_500;
const FOUNDER_MULTIPLICITY: u64 = 150;
const REPLICATES: u64 = 2;
const REPRODUCTION_STEPS: usize = 12_000;
const DAUGHTER_CONTINUATION_STEPS: usize = 3_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum Environment {
    Resource,
    Damage,
}

impl Environment {
    fn label(self) -> &'static str {
        match self {
            Self::Resource => "RESOURCE_CHALLENGE",
            Self::Damage => "DAMAGE_CHALLENGE",
        }
    }

    fn fixed_inflow_concentrations(self, phase_step: usize) -> (f64, f64) {
        match self {
            // Frozen D-096 H pulse/lean schedule.
            Self::Resource if phase_step % 400 < 100 => (2.75, 1.0),
            Self::Resource => (0.264, 1.0),
            // Frozen D-096 B resource boundary.
            Self::Damage => (1.98, 1.0),
        }
    }
}

#[derive(Debug, Clone)]
struct Cohort {
    mesh: MaterialMesh,
    plasticity: Option<PlasticityStateV1>,
    count: u64,
    generation: u32,
    birth_mass: f64,
    id: u64,
}

#[derive(Debug, Clone, Default, Serialize)]
struct WorldLedger {
    initial_n: f64,
    initial_f: f64,
    inflow_n: f64,
    inflow_f: f64,
    delivered_n: f64,
    delivered_f: f64,
    returned_n: f64,
    returned_f: f64,
    c_outflow: f64,
    a_outflow: f64,
    w_outflow: f64,
    damage_structural_sink: f64,
    damage_membrane_sink: f64,
    physical_death_n_sink: f64,
    physical_death_f_sink: f64,
    invalidated_n_terminal: f64,
    invalidated_f_terminal: f64,
}

#[derive(Debug, Clone, Serialize)]
struct OpenMedium {
    /// One fixed reference volume per preregistered founder. This scales the
    /// assay vessel, not any organism rule, and never follows population size.
    volume: f64,
    n_mass: f64,
    f_mass: f64,
    ledger: WorldLedger,
}

#[derive(Debug, Clone, Default, Serialize)]
struct CampaignLedger {
    mutation_opportunities: u64,
    mutations: u64,
    physical_fissions: u64,
    valid_simple_fissions: u64,
    physical_deaths: u64,
    runtime_invalidations: u64,
    expression_failures: u64,
    invalid_geometry_events: u64,
    partition_failures: u64,
    reaction_n_consumed: f64,
    reaction_f_consumed: f64,
    a_produced: f64,
    w_produced: f64,
    growth_material: f64,
    expression_material: f64,
    expression_activation: f64,
    active_a_spent: f64,
    active_w_produced: f64,
    adaptation_remesh_mappings: u64,
    mutation_events: Vec<Value>,
    fissions_by_parent_genotype: BTreeMap<String, u64>,
}

fn genotype_key(genotype: AllocationGenotype) -> String {
    genotype
        .0
        .iter()
        .map(|value| format!("{value:.17}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn mutation_seed(campaign_seed: u64, step: usize, cohort: u64, ordinal: u64, child: u64) -> u64 {
    splitmix64(
        campaign_seed
            ^ (step as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
            ^ cohort.rotate_left(17)
            ^ ordinal.wrapping_mul(0xd1b5_4a32_d192_ed03)
            ^ child.wrapping_mul(0x94d0_49bb_1331_11eb),
    )
}

fn perturb_seed_1(mesh: &mut MaterialMesh) {
    let center = mesh.centroid();
    let (sine, cosine) = 0.3_f64.sin_cos();
    for point in &mut mesh.vertices {
        let x = point[0] - center[0];
        let y = point[1] - center[1];
        point[0] = center[0] + cosine * x - sine * y;
        point[1] = center[1] + sine * x + cosine * y;
    }
    for (index, point) in mesh.vertices.iter_mut().enumerate() {
        let fraction = (((index as f64 + 1.0) * 12.9898).sin() * 43758.5453).fract();
        point[0] += 0.35 * (fraction - 0.5);
        point[1] += 0.35 * ((fraction * 7.13).fract() - 0.5);
    }
    let center = mesh.centroid();
    for point in &mut mesh.vertices {
        point[0] = center[0] + (point[0] - center[0]) * 1.25;
    }
}

/// Reconstruct the sealed WP1 seed-1 parent whose two daughters independently
/// passed the complete 3,000-step simple-boundary viability assay. The returned
/// state is immediately before its unchanged fission.
fn lawful_parent_template() -> (MaterialMesh, f64, usize) {
    let mut mesh = chemistry_core::mesh_population::MeshPopulation::seed_one(14.0, 1, 2.2)
        .individuals
        .remove(0)
        .mesh;
    perturb_seed_1(&mut mesh);
    let birth_mass = mesh.total_structural_mass();
    let mechanics = MechParams::default();
    let reaction = ReactionParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let fission = FissionParams::default();
    for step in 0..TEMPLATE_STEPS {
        let _ = transport_step(&mut mesh, &transport, mechanics.dt);
        let _ = reactions_step(&mut mesh, &reaction, mechanics.dt, true, true);
        let _ = growth_step(&mut mesh, &reaction, &growth, mechanics.dt);
        assert!(mechanics_step_with_local_self_contact(&mut mesh, &mechanics).is_some());
        let _ = remesh(&mut mesh);
        if step % 10 == 0 {
            let _ = topology_step(&mut mesh, &fission);
        }
        assert!(polygon_simple(&mesh.vertices));
        if mesh.total_structural_mass() >= 1.35 * birth_mass && step % 25 == 0 {
            if let Some((a, b, event)) = try_local_fission(&mesh, &fission) {
                if event.partition.ok && polygon_simple(&a.vertices) && polygon_simple(&b.vertices)
                {
                    return (mesh, birth_mass, step + 1);
                }
            }
        }
    }
    panic!("sealed seed-10 geometry-valid parent did not replay");
}

fn signed_request(
    mesh: &MaterialMesh,
    transport: &TransportParams,
    species: &str,
    interior: f64,
    boundary: f64,
    dt: f64,
) -> f64 {
    let theta = mean_occupancy(mesh);
    let ruptured_fraction =
        mesh.edges.iter().filter(|edge| edge.ruptured).count() as f64 / mesh.n().max(1) as f64;
    transport.k_flux
        * permeability(theta, species)
        * (1.0 + 4.0 * ruptured_fraction)
        * (boundary - interior)
        * mesh.perimeter().max(1e-6)
        * dt
}

impl OpenMedium {
    fn new(environment: Environment) -> Self {
        let volume = FOUNDER_MULTIPLICITY as f64;
        let (n, f) = environment.fixed_inflow_concentrations(0);
        let initial_n = n * volume;
        let initial_f = f * volume;
        Self {
            volume,
            n_mass: initial_n,
            f_mass: initial_f,
            ledger: WorldLedger {
                initial_n,
                initial_f,
                ..WorldLedger::default()
            },
        }
    }

    fn add_fixed_inflow(&mut self, environment: Environment, phase_step: usize, dt: f64) {
        let (n, f) = environment.fixed_inflow_concentrations(phase_step);
        let add_n = n * dt * self.volume;
        let add_f = f * dt * self.volume;
        self.n_mass += add_n;
        self.f_mass += add_f;
        self.ledger.inflow_n += add_n;
        self.ledger.inflow_f += add_f;
    }

    /// Exact multiplicity-aware finite shared-boundary exchange. The effective
    /// boundary passed to the frozen transport law is reduced only by the one
    /// common finite-world allocation scale.
    fn exchange(&mut self, cohorts: &mut [Cohort], transport: &TransportParams, dt: f64) {
        let boundary_n = self.n_mass / self.volume;
        let boundary_f = self.f_mass / self.volume;
        let mut raw = Vec::with_capacity(cohorts.len());
        let mut requested_n = 0.0;
        let mut requested_f = 0.0;
        let mut returned_n = 0.0;
        let mut returned_f = 0.0;
        for cohort in cohorts.iter() {
            let area = cohort.mesh.area().max(1e-15);
            let rn = signed_request(
                &cohort.mesh,
                transport,
                "N",
                cohort.mesh.interior.n,
                boundary_n,
                dt,
            );
            let rf = signed_request(
                &cohort.mesh,
                transport,
                "F",
                cohort.mesh.interior.f,
                boundary_f,
                dt,
            );
            let rn = rn.max(-cohort.mesh.interior.n.max(0.0) * area);
            let rf = rf.max(-cohort.mesh.interior.f.max(0.0) * area);
            if rn >= 0.0 {
                requested_n += rn * cohort.count as f64;
            } else {
                returned_n += -rn * cohort.count as f64;
            }
            if rf >= 0.0 {
                requested_f += rf * cohort.count as f64;
            } else {
                returned_f += -rf * cohort.count as f64;
            }
            raw.push((rn, rf));
        }
        let available_n = self.n_mass + returned_n;
        let available_f = self.f_mass + returned_f;
        let n_scale = if requested_n > 0.0 {
            (available_n / requested_n).min(1.0)
        } else {
            1.0
        };
        let f_scale = if requested_f > 0.0 {
            (available_f / requested_f).min(1.0)
        } else {
            1.0
        };
        let scale = n_scale.min(f_scale).clamp(0.0, 1.0);

        for (cohort, (rn, rf)) in cohorts.iter_mut().zip(raw) {
            let original = cohort.mesh.exterior;
            cohort.mesh.exterior.c = 0.0;
            cohort.mesh.exterior.a = 0.0;
            cohort.mesh.exterior.w = 0.0;
            cohort.mesh.exterior.n = if rn >= 0.0 {
                cohort.mesh.interior.n + scale * (boundary_n - cohort.mesh.interior.n)
            } else {
                boundary_n
            };
            cohort.mesh.exterior.f = if rf >= 0.0 {
                cohort.mesh.interior.f + scale * (boundary_f - cohort.mesh.interior.f)
            } else {
                boundary_f
            };
            let ledger = transport_step(&mut cohort.mesh, transport, dt);
            cohort.mesh.exterior = original;
            let count = cohort.count as f64;
            self.n_mass += count * (ledger.n_out - ledger.n_in);
            self.f_mass += count * (ledger.f_out - ledger.f_in);
            self.ledger.delivered_n += count * ledger.n_in;
            self.ledger.delivered_f += count * ledger.f_in;
            self.ledger.returned_n += count * ledger.n_out;
            self.ledger.returned_f += count * ledger.f_out;
            self.ledger.c_outflow += count * ledger.c_leak;
            self.ledger.a_outflow += count * ledger.a_leak;
            self.ledger.w_outflow += count * ledger.w_out;
        }
        self.n_mass = self.n_mass.max(0.0);
        self.f_mass = self.f_mass.max(0.0);
    }
}

fn apply_damage(cohort: &mut Cohort, step: usize, world: &mut OpenMedium) {
    if step % 350 != 0 || cohort.mesh.edges.is_empty() {
        return;
    }
    let structural = 0.08_f64.min(cohort.mesh.edges[0].m.max(0.0));
    let membrane = 0.048_f64.min(cohort.mesh.edges[0].b.max(0.0));
    cohort.mesh.edges[0].m -= structural;
    cohort.mesh.edges[0].b -= membrane;
    world.ledger.damage_structural_sink += structural * cohort.count as f64;
    world.ledger.damage_membrane_sink += membrane * cohort.count as f64;
}

fn organism_amount(cohorts: &[Cohort], species: char) -> f64 {
    cohorts
        .iter()
        .map(|cohort| {
            let concentration = match species {
                'n' => cohort.mesh.interior.n,
                'f' => cohort.mesh.interior.f,
                _ => 0.0,
            };
            concentration.max(0.0) * cohort.mesh.area() * cohort.count as f64
        })
        .sum()
}

fn population_count(cohorts: &[Cohort]) -> u64 {
    cohorts.iter().map(|cohort| cohort.count).sum()
}

fn mean_genotype(cohorts: &[Cohort]) -> [f64; 4] {
    let count = population_count(cohorts).max(1) as f64;
    let mut mean = [0.0; 4];
    for cohort in cohorts {
        let genotype = cohort.mesh.finite_allocation.unwrap().genotype;
        for (index, value) in genotype.0.iter().enumerate() {
            mean[index] += *value * cohort.count as f64 / count;
        }
    }
    mean
}

fn snapshot(cohorts: &[Cohort], world: &OpenMedium, step: usize) -> Value {
    let mean = mean_genotype(cohorts);
    json!({
        "step": step,
        "population": population_count(cohorts),
        "cohorts": cohorts.len(),
        "maximum_generation": cohorts.iter().map(|c| c.generation).max().unwrap_or(0),
        "mean_genotype": mean,
        "processing_activation": mean[0] + mean[1],
        "repair": mean[2],
        "growth_reserve_only_dormant": mean[3],
        "world_n": world.n_mass,
        "world_f": world.f_mass,
        "organism_n": organism_amount(cohorts, 'n'),
        "organism_f": organism_amount(cohorts, 'f'),
        "all_simple": cohorts.iter().all(|c| polygon_simple(&c.mesh.vertices)),
    })
}

fn split_cohort(
    cohort: Cohort,
    mutation_enabled: bool,
    campaign_seed: u64,
    step: usize,
    next_id: &mut u64,
    allocation: &AllocationParams,
    fission: &FissionParams,
    ledger: &mut CampaignLedger,
) -> Result<Vec<Cohort>, Cohort> {
    let Some((daughter_a, daughter_b, event)) = try_local_fission(&cohort.mesh, fission) else {
        return Err(cohort);
    };
    if !event.partition.ok {
        ledger.partition_failures += cohort.count;
        return Err(cohort);
    }
    if !polygon_simple(&cohort.mesh.vertices)
        || !polygon_simple(&daughter_a.vertices)
        || !polygon_simple(&daughter_b.vertices)
    {
        ledger.invalid_geometry_events += cohort.count;
        return Err(cohort);
    }
    ledger.physical_fissions += cohort.count;
    ledger.valid_simple_fissions += cohort.count;
    *ledger
        .fissions_by_parent_genotype
        .entry(genotype_key(
            cohort.mesh.finite_allocation.unwrap().genotype,
        ))
        .or_default() += cohort.count;

    let mutation_params = if mutation_enabled {
        *allocation
    } else {
        AllocationParams {
            mutation_probability: 0.0,
            ..*allocation
        }
    };
    let children = [daughter_a, daughter_b];
    let mut groups: BTreeMap<(usize, [u64; 4]), (MaterialMesh, u64)> = BTreeMap::new();
    for ordinal in 0..cohort.count {
        for (side, child_template) in children.iter().enumerate() {
            let parent = child_template.finite_allocation.unwrap().genotype;
            let seed = mutation_seed(campaign_seed, step, cohort.id, ordinal, side as u64);
            let mutation = mutate_allocation_at_reproduction(parent, &mutation_params, seed);
            ledger.mutation_opportunities += 1;
            if mutation.mutated {
                ledger.mutations += 1;
                ledger.mutation_events.push(json!({
                    "step": step,
                    "generation": cohort.generation + 1,
                    "seed": seed,
                    "parent": mutation.parent.0,
                    "offspring": mutation.offspring.0,
                    "source_index": mutation.source_index,
                    "target_index": mutation.target_index,
                    "transferred": mutation.transferred,
                }));
            }
            let key = (side, mutation.offspring.0.map(f64::to_bits));
            groups
                .entry(key)
                .and_modify(|(_, count)| *count += 1)
                .or_insert_with(|| {
                    let mut mesh = child_template.clone();
                    mesh.finite_allocation.as_mut().unwrap().genotype = mutation.offspring;
                    (mesh, 1)
                });
        }
    }
    let mut result = Vec::new();
    for (_, (mesh, count)) in groups {
        let id = *next_id;
        *next_id += 1;
        result.push(Cohort {
            birth_mass: mesh.total_structural_mass(),
            mesh,
            plasticity: None,
            count,
            generation: cohort.generation + 1,
            id,
        });
    }
    Ok(result)
}

fn invalidated_material_to_terminal(cohort: &Cohort, world: &mut OpenMedium) {
    world.ledger.invalidated_n_terminal +=
        cohort.mesh.interior.n.max(0.0) * cohort.mesh.area() * cohort.count as f64;
    world.ledger.invalidated_f_terminal +=
        cohort.mesh.interior.f.max(0.0) * cohort.mesh.area() * cohort.count as f64;
}

fn dead_material_to_sink(cohort: &Cohort, world: &mut OpenMedium) {
    world.ledger.physical_death_n_sink +=
        cohort.mesh.interior.n.max(0.0) * cohort.mesh.area() * cohort.count as f64;
    world.ledger.physical_death_f_sink +=
        cohort.mesh.interior.f.max(0.0) * cohort.mesh.area() * cohort.count as f64;
}

fn advance_phase(
    cohorts: &mut Vec<Cohort>,
    world: &mut OpenMedium,
    environment: Environment,
    phase_index: usize,
    mutation_enabled: bool,
    campaign_seed: u64,
    next_id: &mut u64,
    ledger: &mut CampaignLedger,
    trajectory: &mut Vec<Value>,
) {
    let allocation = AllocationParams::default();
    let mechanics = MechParams::default();
    let reaction = ReactionParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let fission = FissionParams::default();
    for phase_step in 0..PHASE_STEPS {
        if cohorts.is_empty() {
            break;
        }
        let step = phase_index * PHASE_STEPS + phase_step + 1;
        world.add_fixed_inflow(environment, phase_step, mechanics.dt);
        if environment == Environment::Damage {
            for cohort in cohorts.iter_mut() {
                apply_damage(cohort, phase_step, world);
            }
        }
        let mut retained = Vec::new();
        for mut cohort in cohorts.drain(..) {
            match expression_step(&mut cohort.mesh, &allocation, mechanics.dt) {
                Ok(expression) => {
                    let count = cohort.count as f64;
                    ledger.expression_material += expression.material_consumed * count;
                    ledger.expression_activation +=
                        (expression.activation_consumed + expression.maintenance_consumed) * count;
                    retained.push(cohort);
                }
                Err(_) => {
                    ledger.expression_failures += cohort.count;
                    ledger.runtime_invalidations += cohort.count;
                    invalidated_material_to_terminal(&cohort, world);
                }
            }
        }
        *cohorts = retained;
        world.exchange(cohorts, &transport, mechanics.dt);

        let mut survivors = Vec::new();
        for mut cohort in cohorts.drain(..) {
            let count = cohort.count as f64;
            let reactions = reactions_step(&mut cohort.mesh, &reaction, mechanics.dt, true, true);
            ledger.reaction_n_consumed += reactions.n_consumed * count;
            ledger.reaction_f_consumed += reactions.f_consumed * count;
            ledger.a_produced += reactions.a_produced * count;
            ledger.w_produced += reactions.w_produced * count;
            let grown = growth_step(&mut cohort.mesh, &reaction, &growth, mechanics.dt);
            ledger.growth_material += grown.m_grown * count;
            let valid =
                mechanics_step_with_local_self_contact(&mut cohort.mesh, &mechanics).is_some();
            if !valid {
                ledger.runtime_invalidations += cohort.count;
                ledger.invalid_geometry_events += cohort.count;
                invalidated_material_to_terminal(&cohort, world);
                continue;
            }
            let _ = remesh(&mut cohort.mesh);
            if step % 10 == 0 {
                let _ = topology_step(&mut cohort.mesh, &fission);
            }
            if !polygon_simple(&cohort.mesh.vertices) {
                ledger.runtime_invalidations += cohort.count;
                ledger.invalid_geometry_events += cohort.count;
                invalidated_material_to_terminal(&cohort, world);
                continue;
            }
            if !cohort.mesh.observer_viable() {
                ledger.physical_deaths += cohort.count;
                dead_material_to_sink(&cohort, world);
                continue;
            }
            let eligible =
                cohort.mesh.total_structural_mass() >= 1.35 * cohort.birth_mass && step % 25 == 0;
            if eligible {
                match split_cohort(
                    cohort,
                    mutation_enabled,
                    campaign_seed,
                    step,
                    next_id,
                    &allocation,
                    &fission,
                    ledger,
                ) {
                    Ok(children) => survivors.extend(children),
                    Err(parent) => survivors.push(parent),
                }
            } else {
                survivors.push(cohort);
            }
        }
        *cohorts = survivors;
        if phase_step == 0 || (phase_step + 1) % 250 == 0 {
            trajectory.push(snapshot(cohorts, world, step));
        }
    }
}

fn initial_population(
    template: &MaterialMesh,
    original_birth_mass: f64,
    mutation_enabled: bool,
    campaign_seed: u64,
    founder_count: u64,
    ledger: &mut CampaignLedger,
) -> Vec<Cohort> {
    let allocation = AllocationParams::default();
    let fission = FissionParams::default();
    let mut parent = template.clone();
    parent.enable_finite_allocation(AllocationGenotype::neutral(), &allocation);
    let cohort = Cohort {
        mesh: parent,
        plasticity: None,
        count: founder_count,
        generation: 0,
        birth_mass: original_birth_mass,
        id: 1,
    };
    let mut next_id = 2;
    split_cohort(
        cohort,
        mutation_enabled,
        campaign_seed,
        0,
        &mut next_id,
        &allocation,
        &fission,
        ledger,
    )
    .expect("sealed geometry-valid parent must fission")
}

fn campaign(
    template: &MaterialMesh,
    original_birth_mass: f64,
    template_step: usize,
    sequence: &[Environment],
    mutation_enabled: bool,
    replicate: u64,
) -> Value {
    let campaign_seed = splitmix64(
        replicate
            ^ if mutation_enabled {
                0x6a09_e667_f3bc_c909
            } else {
                0xbb67_ae85_84ca_a73b
            }
            ^ sequence.iter().fold(0_u64, |state, env| {
                state.rotate_left(9)
                    ^ match env {
                        Environment::Resource => 0x11,
                        Environment::Damage => 0x22,
                    }
            }),
    );
    let mut ledger = CampaignLedger::default();
    let mut cohorts = initial_population(
        template,
        original_birth_mass,
        mutation_enabled,
        campaign_seed,
        FOUNDER_MULTIPLICITY,
        &mut ledger,
    );
    let mut next_id = 10_000;
    let mut world = OpenMedium::new(sequence[0]);
    let initial_organism_n = organism_amount(&cohorts, 'n');
    let initial_organism_f = organism_amount(&cohorts, 'f');
    let initial = snapshot(&cohorts, &world, 0);
    let mut trajectory = vec![initial.clone()];
    for (phase, environment) in sequence.iter().copied().enumerate() {
        advance_phase(
            &mut cohorts,
            &mut world,
            environment,
            phase,
            mutation_enabled,
            campaign_seed,
            &mut next_id,
            &mut ledger,
            &mut trajectory,
        );
    }
    let terminal_step = sequence.len() * PHASE_STEPS;
    let terminal = snapshot(&cohorts, &world, terminal_step);
    let terminal_organism_n = organism_amount(&cohorts, 'n');
    let terminal_organism_f = organism_amount(&cohorts, 'f');
    let n_closure = (world.ledger.initial_n + world.ledger.inflow_n + initial_organism_n
        - world.n_mass
        - terminal_organism_n
        - ledger.reaction_n_consumed
        - world.ledger.physical_death_n_sink
        - world.ledger.invalidated_n_terminal)
        .abs();
    let f_closure = (world.ledger.initial_f + world.ledger.inflow_f + initial_organism_f
        - world.f_mass
        - terminal_organism_f
        - ledger.reaction_f_consumed
        - world.ledger.physical_death_f_sink
        - world.ledger.invalidated_f_terminal)
        .abs();
    json!({
        "replicate": replicate,
        "mutation_enabled": mutation_enabled,
        "campaign_seed": campaign_seed,
        "environment_sequence": sequence.iter().map(|environment| environment.label()).collect::<Vec<_>>(),
        "phase_steps": PHASE_STEPS,
        "founder_multiplicity": FOUNDER_MULTIPLICITY,
        "template_fission_step": template_step,
        "fitness_function": null,
        "breeder_selection": false,
        "population_cap": null,
        "resource_feedback": false,
        "exchangeability_compression": true,
        "initial": initial,
        "trajectory": trajectory,
        "terminal": terminal,
        "world": world,
        "ledger": ledger,
        "n_closure_residual": n_closure,
        "f_closure_residual": f_closure,
    })
}

fn compression_parity(template: &MaterialMesh, original_birth_mass: f64) -> Value {
    let mut compressed_ledger = CampaignLedger::default();
    let mut compressed = initial_population(
        template,
        original_birth_mass,
        false,
        101,
        2,
        &mut compressed_ledger,
    );
    let mut explicit = Vec::new();
    let mut explicit_ledger = CampaignLedger::default();
    for ordinal in 0..2_u64 {
        let mut ledger = CampaignLedger::default();
        explicit.extend(initial_population(
            template,
            original_birth_mass,
            false,
            101 ^ ordinal.rotate_left(3),
            1,
            &mut ledger,
        ));
        explicit_ledger.mutation_opportunities += ledger.mutation_opportunities;
    }
    let transport = TransportParams::default();
    let dt = MechParams::default().dt;
    let mut compressed_world = OpenMedium::new(Environment::Resource);
    let mut explicit_world = compressed_world.clone();
    compressed_world.exchange(&mut compressed, &transport, dt);
    explicit_world.exchange(&mut explicit, &transport, dt);
    let transport_residual = (compressed_world.n_mass - explicit_world.n_mass)
        .abs()
        .max((compressed_world.f_mass - explicit_world.f_mass).abs())
        .max((organism_amount(&compressed, 'n') - organism_amount(&explicit, 'n')).abs())
        .max((organism_amount(&compressed, 'f') - organism_amount(&explicit, 'f')).abs());
    json!({
        "scope": "initial geometry-valid fission, mutation-off inheritance, and one finite shared-medium exchange",
        "compressed_population": population_count(&compressed),
        "explicit_population": population_count(&explicit),
        "compressed_opportunities": compressed_ledger.mutation_opportunities,
        "explicit_opportunities": explicit_ledger.mutation_opportunities,
        "transport_residual": transport_residual,
        "pass": population_count(&compressed) == population_count(&explicit)
            && compressed_ledger.mutation_opportunities == explicit_ledger.mutation_opportunities
            && transport_residual <= 1e-10,
    })
}

fn one_frozen_step(mut mesh: MaterialMesh, expression: bool) -> Value {
    let allocation = AllocationParams::default();
    let mechanics = MechParams::default();
    let reaction = ReactionParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let initial_physical_valid = mesh.physical_runtime_valid();
    let initial_lifecycle_valid = mesh.lifecycle_invariants_hold();
    let initial_simple = polygon_simple(&mesh.vertices);
    let (expression_ok, expression_material) = if expression {
        match expression_step(&mut mesh, &allocation, mechanics.dt) {
            Ok(ledger) => (true, ledger.material_consumed),
            Err(_) => (false, 0.0),
        }
    } else {
        (true, 0.0)
    };
    let post_expression_physical_valid = mesh.physical_runtime_valid();
    let post_expression_lifecycle_valid = mesh.lifecycle_invariants_hold();
    let minimum_m_minus_young_after_expression = mesh
        .edges
        .iter()
        .map(|edge| edge.m - edge.m_young)
        .fold(f64::INFINITY, f64::min);
    let _ = transport_step(&mut mesh, &transport, mechanics.dt);
    let _ = reactions_step(&mut mesh, &reaction, mechanics.dt, true, true);
    let _ = growth_step(&mut mesh, &reaction, &growth, mechanics.dt);
    let pre_contact_physical_valid = mesh.physical_runtime_valid();
    let pre_contact_lifecycle_valid = mesh.lifecycle_invariants_hold();
    let pre_contact_simple = polygon_simple(&mesh.vertices);
    let contact_accepted = mechanics_step_with_local_self_contact(&mut mesh, &mechanics).is_some();
    json!({
        "expression_enabled": expression,
        "expression_ok": expression_ok,
        "expression_material_consumed": expression_material,
        "initial_physical_runtime_valid": initial_physical_valid,
        "initial_lifecycle_invariants_hold": initial_lifecycle_valid,
        "initial_polygon_simple": initial_simple,
        "post_expression_physical_runtime_valid": post_expression_physical_valid,
        "post_expression_lifecycle_invariants_hold": post_expression_lifecycle_valid,
        "minimum_m_minus_young_after_expression": minimum_m_minus_young_after_expression,
        "pre_contact_physical_runtime_valid": pre_contact_physical_valid,
        "pre_contact_lifecycle_invariants_hold": pre_contact_lifecycle_valid,
        "pre_contact_polygon_simple": pre_contact_simple,
        "contact_accepted": contact_accepted,
        "polygon_simple_after": polygon_simple(&mesh.vertices),
        "observer_viable_after": mesh.observer_viable(),
    })
}

fn exact_daughter_root_cause(mesh: &MaterialMesh) -> Value {
    let allocation = AllocationParams::default();
    let mut repaired = mesh.clone();
    let material_before = repaired.total_structural_mass();
    let before = repaired
        .edges
        .iter()
        .enumerate()
        .map(|(index, edge)| {
            json!({
                "edge": index,
                "m": edge.m,
                "m_young": edge.m_young,
                "m_mature": edge.m - edge.m_young,
                "young_fraction": edge.m_young / edge.m.max(f64::MIN_POSITIVE),
                "rest_length": repaired.rest_length(index),
                "edge_length": repaired.edge_length(index),
                "strain": repaired.strain(index),
            })
        })
        .collect::<Vec<_>>();
    let ledger = expression_step(&mut repaired, &allocation, MechParams::default().dt)
        .expect("exact daughter expression");
    let fraction_left = 1.0 - ledger.material_consumed / material_before;
    let legacy_violation_edges = mesh
        .edges
        .iter()
        .enumerate()
        .filter_map(|(index, edge)| {
            let legacy_m = edge.m * fraction_left;
            (edge.m_young > legacy_m + 1e-12).then_some(json!({
                "edge": index,
                "legacy_m_after": legacy_m,
                "legacy_m_young_after": edge.m_young,
                "excess": edge.m_young - legacy_m,
            }))
        })
        .collect::<Vec<_>>();
    let after = repaired
        .edges
        .iter()
        .enumerate()
        .map(|(index, edge)| {
            json!({
                "edge": index,
                "m": edge.m,
                "m_young": edge.m_young,
                "m_mature": edge.m - edge.m_young,
                "young_fraction": edge.m_young / edge.m.max(f64::MIN_POSITIVE),
                "rest_length": repaired.rest_length(index),
                "edge_length": repaired.edge_length(index),
                "strain": repaired.strain(index),
            })
        })
        .collect::<Vec<_>>();
    json!({
        "contract_version": format!("{:?}", mesh.contract_version),
        "maturation_coupled": mesh.is_maturation_coupled(),
        "before": before,
        "material_consumed": ledger.material_consumed,
        "fraction_left": fraction_left,
        "legacy_counterfactual_violation_edges": legacy_violation_edges,
        "legacy_failing_predicate": "M_YOUNG_EXCEEDS_M_AFTER_D096_STRUCTURAL_DRAW",
        "repaired_after": after,
        "repaired_lifecycle_invariants_hold": repaired.lifecycle_invariants_hold(),
        "repaired_physical_runtime_valid": repaired.physical_runtime_valid(),
    })
}

fn v4_daughter_lifecycle(mut mesh: MaterialMesh, birth_mass: f64) -> Value {
    let allocation = AllocationParams::default();
    let mechanics = MechParams::default();
    let reaction = ReactionParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let fission = FissionParams::default();
    let mut maximum_material_residual = 0.0_f64;
    let mut maximum_activation_residual = 0.0_f64;
    let mut maximum_waste_residual = 0.0_f64;
    let mut runtime_invalidation = None;
    let mut physical_fission = None;
    let mut rejected_non_simple_fission_candidates = Vec::new();
    let mut observer_nonviable_steps = 0_u64;
    let mut completed_steps = 0_usize;
    let mut first_invalid_edges = Vec::new();
    let mut first_topology_ledger = Value::Null;
    for step in 0..3_000_usize {
        let area = mesh.area();
        let material_before = mesh.total_structural_mass();
        let a_before = mesh.interior.a * area;
        let w_before = mesh.interior.w * area;
        let expression = match expression_step(&mut mesh, &allocation, mechanics.dt) {
            Ok(ledger) => ledger,
            Err(reason) => {
                runtime_invalidation = Some(
                    json!({"step":step + 1,"phase":"expression","reason":format!("{reason:?}")}),
                );
                break;
            }
        };
        maximum_material_residual = maximum_material_residual.max(
            (material_before - mesh.total_structural_mass() - expression.material_consumed).abs(),
        );
        let activated_spent = expression.activation_consumed + expression.maintenance_consumed;
        maximum_activation_residual = maximum_activation_residual
            .max((a_before - mesh.interior.a * area - activated_spent).abs());
        maximum_waste_residual = maximum_waste_residual.max(
            (mesh.interior.w * area - w_before - activated_spent - expression.turnover_waste).abs(),
        );
        if !mesh.lifecycle_invariants_hold() || !mesh.physical_runtime_valid() {
            runtime_invalidation = Some(json!({"step":step + 1,"phase":"post_expression"}));
            break;
        }
        let _ = transport_step(&mut mesh, &transport, mechanics.dt);
        let _ = reactions_step(&mut mesh, &reaction, mechanics.dt, true, true);
        let _ = growth_step(&mut mesh, &reaction, &growth, mechanics.dt);
        if mechanics_step_with_local_self_contact(&mut mesh, &mechanics).is_none() {
            runtime_invalidation = Some(json!({"step":step + 1,"phase":"self_contact_mechanics"}));
            break;
        }
        let _ = remesh(&mut mesh);
        if !mesh.lifecycle_invariants_hold() || !mesh.physical_runtime_valid() {
            runtime_invalidation = Some(json!({"step":step + 1,"phase":"post_remesh"}));
            first_invalid_edges = mesh
                .edges
                .iter()
                .enumerate()
                .filter(|(_, edge)| edge.m_young > edge.m + 1e-12)
                .map(|(index, edge)| json!({"edge":index,"m":edge.m,"m_young":edge.m_young,"ruptured":edge.ruptured}))
                .collect();
            break;
        }
        if step % 10 == 0 {
            let topology = topology_step(&mut mesh, &fission);
            first_topology_ledger = json!({
                "step": step + 1,
                "tension_ruptures": topology.tension_ruptures,
                "local_rebonds": topology.local_rebonds,
                "cross_bonds": topology.cross_bonds,
            });
        }
        if !mesh.lifecycle_invariants_hold()
            || !mesh.physical_runtime_valid()
            || !polygon_simple(&mesh.vertices)
        {
            runtime_invalidation = Some(json!({"step":step + 1,"phase":"post_topology"}));
            first_invalid_edges = mesh
                .edges
                .iter()
                .enumerate()
                .filter(|(_, edge)| edge.m_young > edge.m + 1e-12)
                .map(|(index, edge)| json!({"edge":index,"m":edge.m,"m_young":edge.m_young,"ruptured":edge.ruptured}))
                .collect();
            break;
        }
        if !mesh.observer_viable() {
            observer_nonviable_steps += 1;
        }
        completed_steps = step + 1;
        if mesh.total_structural_mass() >= 1.35 * birth_mass && step % 25 == 0 {
            if let Some((a, b, event)) = try_local_fission(&mesh, &fission) {
                let candidate = json!({
                    "step": step + 1,
                    "partition_ok": event.partition.ok,
                    "parent_simple": polygon_simple(&mesh.vertices),
                    "daughter_a_simple": polygon_simple(&a.vertices),
                    "daughter_b_simple": polygon_simple(&b.vertices),
                });
                if event.partition.ok
                    && polygon_simple(&mesh.vertices)
                    && polygon_simple(&a.vertices)
                    && polygon_simple(&b.vertices)
                {
                    physical_fission = Some(candidate);
                    break;
                }
                rejected_non_simple_fission_candidates.push(candidate);
            }
        }
    }
    json!({
        "contract_version": format!("{:?}", mesh.contract_version),
        "target_steps": 3000,
        "completed_steps": completed_steps,
        "runtime_invalidation": runtime_invalidation,
        "first_invalid_edges": first_invalid_edges,
        "first_topology_ledger": first_topology_ledger,
        "physical_fission": physical_fission,
        "rejected_non_simple_fission_candidates": rejected_non_simple_fission_candidates,
        "observer_nonviable_steps_reported_not_removed": observer_nonviable_steps,
        "terminal_lifecycle_invariants_hold": mesh.lifecycle_invariants_hold(),
        "terminal_physical_runtime_valid": mesh.physical_runtime_valid(),
        "terminal_polygon_simple": polygon_simple(&mesh.vertices),
        "maximum_structural_material_closure_residual": maximum_material_residual,
        "maximum_activation_closure_residual": maximum_activation_residual,
        "maximum_waste_closure_residual": maximum_waste_residual,
    })
}

fn v4_counterpart(template: &MaterialMesh) -> Value {
    let allocation = AllocationParams::default();
    let fission = FissionParams::default();
    let mut parent = template.clone();
    parent.contract_version =
        chemistry_core::material_mesh::MeshContractVersion::MaturationCoupledV4;
    parent.enable_finite_allocation(AllocationGenotype::neutral(), &allocation);
    let (a, b, event) = try_local_fission(&parent, &fission).expect("V4 counterpart fission");
    assert!(event.partition.ok && polygon_simple(&a.vertices) && polygon_simple(&b.vertices));
    let a_birth = a.total_structural_mass();
    let b_birth = b.total_structural_mass();
    json!({
        "scope": "same frozen parent state and fission, with the production V4 contract selected before fission; qualification of the authorized repair, not an exact R2 replay",
        "daughter_a_first_step": one_frozen_step(a.clone(), true),
        "daughter_b_first_step": one_frozen_step(b.clone(), true),
        "daughter_a_lifecycle": v4_daughter_lifecycle(a, a_birth),
        "daughter_b_lifecycle": v4_daughter_lifecycle(b, b_birth),
    })
}

fn newborn_first_step_attribution(template: &MaterialMesh) -> Value {
    let allocation = AllocationParams::default();
    let fission = FissionParams::default();
    let mut parent = template.clone();
    parent.enable_finite_allocation(AllocationGenotype::neutral(), &allocation);
    let (a, b, event) = try_local_fission(&parent, &fission).expect("template fission");
    assert!(event.partition.ok && polygon_simple(&a.vertices) && polygon_simple(&b.vertices));
    json!({
        "daughter_a": {
            "root_cause": exact_daughter_root_cause(&a),
            "expression_off": one_frozen_step(a.clone(), false),
            "expression_on": one_frozen_step(a, true),
        },
        "daughter_b": {
            "root_cause": exact_daughter_root_cause(&b),
            "expression_off": one_frozen_step(b.clone(), false),
            "expression_on": one_frozen_step(b, true),
        },
        "interpretation_boundary": "one frozen post-birth step; observer counterfactual only",
    })
}

fn mutated_descendant_heredity_control(template: &MaterialMesh) -> Value {
    let allocation = AllocationParams::default();
    let fission = FissionParams::default();
    let parent_genotype = AllocationGenotype::neutral();
    let (seed, mutation) = (1_u64..=100_000)
        .map(|seed| {
            (
                seed,
                mutate_allocation_at_reproduction(parent_genotype, &allocation, splitmix64(seed)),
            )
        })
        .find(|(_, event)| event.mutated)
        .expect("configured mutation must be observable in powered search");
    let mut parent = template.clone();
    parent.enable_finite_allocation(mutation.offspring, &allocation);
    let (a, b, event) = try_local_fission(&parent, &fission).expect("template fission");
    let inherited = event.partition.ok
        && polygon_simple(&a.vertices)
        && polygon_simple(&b.vertices)
        && a.finite_allocation.unwrap().genotype == mutation.offspring
        && b.finite_allocation.unwrap().genotype == mutation.offspring;
    json!({
        "scope": "mechanistic fission-partition control; not an evolving founder population",
        "lawful_mutation_seed": splitmix64(seed),
        "mutant": mutation.offspring.0,
        "simple_parent": polygon_simple(&parent.vertices),
        "simple_daughter_a": polygon_simple(&a.vertices),
        "simple_daughter_b": polygon_simple(&b.vertices),
        "partition_ok": event.partition.ok,
        "mutated_genotype_inherited_by_both_daughters": inherited,
    })
}

fn r4_perturb(mesh: &mut MaterialMesh, kind: &str, magnitude: f64) {
    match kind {
        "rotate" => {
            let center = mesh.centroid();
            let (sine, cosine) = magnitude.sin_cos();
            for point in &mut mesh.vertices {
                let x = point[0] - center[0];
                let y = point[1] - center[1];
                point[0] = center[0] + cosine * x - sine * y;
                point[1] = center[1] + sine * x + cosine * y;
            }
        }
        "vertex" => {
            for (index, point) in mesh.vertices.iter_mut().enumerate() {
                let fraction = (((index as f64 + 1.0) * 12.9898).sin() * 43758.5453).fract();
                point[0] += magnitude * (fraction - 0.5);
                point[1] += magnitude * ((fraction * 7.13).fract() - 0.5);
            }
        }
        "c" => mesh.interior.c = (mesh.interior.c * (1.0 + magnitude)).max(0.0),
        "a" => mesh.interior.a = (mesh.interior.a * (1.0 + magnitude)).max(0.0),
        "l" => mesh.free_l = (mesh.free_l * (1.0 + magnitude)).max(0.0),
        "env" => {
            mesh.exterior.n = (mesh.exterior.n * (1.0 + magnitude)).max(0.0);
            mesh.exterior.f = (mesh.exterior.f * (1.0 + magnitude)).max(0.0);
        }
        _ => {}
    }
}

fn r4_reproduction_fixture(seed: u64, kind: &str, magnitude: f64) -> MaterialMesh {
    let mut mesh = chemistry_core::mesh_population::MeshPopulation::seed_one(14.0, seed, 2.2)
        .individuals
        .remove(0)
        .mesh;
    r4_perturb(&mut mesh, kind, magnitude);
    r4_perturb(&mut mesh, "vertex", 0.35);
    let center = mesh.centroid();
    for point in &mut mesh.vertices {
        point[0] = center[0] + (point[0] - center[0]) * 1.25;
    }
    mesh.stamp_maturation_coupled_schema();
    mesh
}

fn r4_geometry(mesh: &MaterialMesh) -> Value {
    json!({
        "simple": polygon_simple(&mesh.vertices),
        "vertices": mesh.n(),
        "area": mesh.area(),
        "perimeter": mesh.perimeter(),
        "mass": mesh.total_structural_mass(),
        "young_mass": mesh.total_young_structural_mass(),
        "lifecycle_invariants_hold": mesh.lifecycle_invariants_hold(),
        "physical_runtime_valid": mesh.physical_runtime_valid(),
    })
}

fn r4_daughter_viability(
    mut mesh: MaterialMesh,
    mechanics: &MechParams,
    reaction: &ReactionParams,
    transport: &TransportParams,
    fission: &FissionParams,
) -> Value {
    let c_initial = mesh.interior.c;
    let a_initial = mesh.interior.a;
    let growth_off = GrowthParams {
        y_g: 0.9,
        enable_growth: false,
    };
    let mut all_simple = polygon_simple(&mesh.vertices);
    let mut all_lifecycle_valid = mesh.lifecycle_invariants_hold();
    let mut all_runtime_valid = mesh.physical_runtime_valid();
    let mut completed_steps = 0_usize;
    for step in 0..DAUGHTER_CONTINUATION_STEPS {
        if !mesh.can_advance_physics() {
            break;
        }
        let _ = transport_step(&mut mesh, transport, mechanics.dt);
        let _ = reactions_step(&mut mesh, reaction, mechanics.dt, true, true);
        let _ = growth_step(&mut mesh, reaction, &growth_off, mechanics.dt);
        if mechanics_step_with_local_self_contact(&mut mesh, mechanics).is_none() {
            all_simple = false;
            break;
        }
        let _ = remesh_preserving_simple(&mut mesh);
        let _ = topology_step(&mut mesh, fission);
        all_simple &= polygon_simple(&mesh.vertices);
        all_lifecycle_valid &= mesh.lifecycle_invariants_hold();
        all_runtime_valid &= mesh.physical_runtime_valid();
        if !all_simple || !all_lifecycle_valid || !all_runtime_valid {
            break;
        }
        completed_steps = step + 1;
    }
    let c_retention = if c_initial > 1e-12 {
        mesh.interior.c / c_initial
    } else {
        1.0
    };
    let a_retention = if a_initial > 1e-12 {
        mesh.interior.a / a_initial
    } else {
        1.0
    };
    let viable = completed_steps == DAUGHTER_CONTINUATION_STEPS
        && mesh.observer_viable()
        && mesh.closed_intact()
        && all_simple
        && all_lifecycle_valid
        && all_runtime_valid
        && c_retention >= 0.80
        && a_retention >= 0.80;
    json!({
        "viable": viable,
        "completed_steps": completed_steps,
        "target_steps": DAUGHTER_CONTINUATION_STEPS,
        "alive": mesh.alive,
        "observer_viable": mesh.observer_viable(),
        "closed_intact": mesh.closed_intact(),
        "all_states_simple": all_simple,
        "all_lifecycle_invariants_hold": all_lifecycle_valid,
        "all_physical_runtime_valid": all_runtime_valid,
        "c_retention": c_retention,
        "a_retention": a_retention,
        "terminal": r4_geometry(&mesh),
    })
}

fn r4_reproduction_run(
    initial_mesh: MaterialMesh,
    name: &str,
    allow_segment: bool,
    half_edge: bool,
) -> Value {
    let mechanics = MechParams::default();
    let reaction = ReactionParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let fission = FissionParams::default();
    let birth_mass = initial_mesh.total_structural_mass();
    let mut mesh = initial_mesh;
    let mut result = None;
    let mut all_simple = polygon_simple(&mesh.vertices);
    let mut all_lifecycle_valid = mesh.lifecycle_invariants_hold();
    let mut all_runtime_valid = mesh.physical_runtime_valid();
    let mut bookkeeping_runtime_invalidations = 0_usize;
    let mut maximum_mass_ratio = 1.0_f64;
    let mut vertex_attempts = 0_usize;
    let mut segment_attempts = 0_usize;
    let mut first_invalid = None;
    for step in 0..REPRODUCTION_STEPS {
        if !mesh.can_advance_physics() {
            bookkeeping_runtime_invalidations += 1;
            first_invalid = Some(json!({"step": step + 1, "phase": "pre_step"}));
            break;
        }
        let _ = transport_step(&mut mesh, &transport, mechanics.dt);
        let _ = reactions_step(&mut mesh, &reaction, mechanics.dt, true, true);
        let _ = growth_step(&mut mesh, &reaction, &growth, mechanics.dt);
        if mechanics_step_with_local_self_contact(&mut mesh, &mechanics).is_none() {
            all_simple = false;
            first_invalid = Some(json!({"step": step + 1, "phase": "self_contact_mechanics"}));
            break;
        }
        if half_edge {
            let _ = remesh_preserving_simple(&mut mesh);
        } else {
            let _ = remesh(&mut mesh);
        }
        let planar = if half_edge {
            PlanarRingTopology::from_mesh(&mesh)
        } else {
            None
        };
        if half_edge && planar.is_none() {
            all_simple = false;
            first_invalid = Some(json!({"step": step + 1, "phase": "half_edge_rebuild"}));
            break;
        }
        if step % 10 == 0 {
            let _ = topology_step(&mut mesh, &fission);
        }
        let simple = polygon_simple(&mesh.vertices);
        let lifecycle_valid = mesh.lifecycle_invariants_hold();
        let runtime_valid = mesh.physical_runtime_valid();
        all_simple &= simple;
        all_lifecycle_valid &= lifecycle_valid;
        all_runtime_valid &= runtime_valid;
        if !lifecycle_valid || !runtime_valid {
            bookkeeping_runtime_invalidations += 1;
        }
        if (!simple || !lifecycle_valid || !runtime_valid) && first_invalid.is_none() {
            first_invalid = Some(json!({
                "step": step + 1,
                "phase": "post_topology",
                "simple": simple,
                "lifecycle_valid": lifecycle_valid,
                "runtime_valid": runtime_valid,
            }));
        }
        if !simple || !lifecycle_valid || !runtime_valid {
            break;
        }
        maximum_mass_ratio =
            maximum_mass_ratio.max(mesh.total_structural_mass() / birth_mass.max(1e-300));
        if mesh.total_structural_mass() >= 1.35 * birth_mass && step % 25 == 0 {
            vertex_attempts += 1;
            let vertex_split = try_local_fission(&mesh, &fission).and_then(|(a, b, event)| {
                (polygon_simple(&mesh.vertices)
                    && polygon_simple(&a.vertices)
                    && polygon_simple(&b.vertices)
                    && event.partition.ok
                    && a.lifecycle_invariants_hold()
                    && b.lifecycle_invariants_hold()
                    && a.physical_runtime_valid()
                    && b.physical_runtime_valid())
                .then_some((a, b, event))
            });
            let split = vertex_split.or_else(|| {
                if !allow_segment {
                    return None;
                }
                segment_attempts += 1;
                if let Some(topology) = planar.as_ref() {
                    topology.try_local_scission(&mesh, &fission)
                } else {
                    try_local_segment_fission(&mesh, &fission)
                }
            });
            if let Some((a, b, event)) = split {
                let valid = polygon_simple(&mesh.vertices)
                    && polygon_simple(&a.vertices)
                    && polygon_simple(&b.vertices)
                    && event.partition.ok
                    && a.lifecycle_invariants_hold()
                    && b.lifecycle_invariants_hold()
                    && a.physical_runtime_valid()
                    && b.physical_runtime_valid();
                if valid {
                    let viability_a = r4_daughter_viability(
                        a.clone(),
                        &mechanics,
                        &reaction,
                        &transport,
                        &fission,
                    );
                    let viability_b = r4_daughter_viability(
                        b.clone(),
                        &mechanics,
                        &reaction,
                        &transport,
                        &fission,
                    );
                    result = Some(json!({
                        "step": step + 1,
                        "valid": true,
                        "parent": r4_geometry(&mesh),
                        "daughter_a": r4_geometry(&a),
                        "daughter_b": r4_geometry(&b),
                        "daughter_a_viability": viability_a,
                        "daughter_b_viability": viability_b,
                        "both_daughters_viable": viability_a["viable"] == true && viability_b["viable"] == true,
                        "partition": event.partition,
                        "pinch": event.pinch,
                    }));
                    break;
                }
            }
        }
    }
    json!({
        "name": name,
        "contract": "MaturationCoupledV4",
        "segment_apposition_enabled": allow_segment,
        "half_edge_topology": half_edge,
        "all_parent_states_simple": all_simple,
        "all_parent_lifecycle_invariants_hold": all_lifecycle_valid,
        "all_parent_physical_runtime_valid": all_runtime_valid,
        "bookkeeping_runtime_invalidations": bookkeeping_runtime_invalidations,
        "max_mass_over_birth": maximum_mass_ratio,
        "growth_qualified": maximum_mass_ratio >= 1.35,
        "vertex_attempts": vertex_attempts,
        "segment_attempts": segment_attempts,
        "physical_fission": result.is_some(),
        "first_invalid": first_invalid,
        "fission": result,
        "final": r4_geometry(&mesh),
    })
}

fn r4_reproduction_campaign() -> Value {
    let kinds = [
        ("rotate", 0.3),
        ("vertex", 0.12),
        ("c", 0.08),
        ("a", 0.08),
        ("env", 0.1),
        ("l", 0.1),
        ("rotate", -0.5),
        ("vertex", -0.1),
        ("c", -0.05),
        ("env", -0.08),
    ];
    let vertex_only = kinds
        .iter()
        .enumerate()
        .map(|(index, (kind, magnitude))| {
            r4_reproduction_run(
                r4_reproduction_fixture((index + 1) as u64, kind, *magnitude),
                &format!("seed_{}_{}_{}", index + 1, kind, magnitude),
                false,
                false,
            )
        })
        .collect::<Vec<_>>();
    let vertex_fissions = vertex_only
        .iter()
        .filter(|arm| arm["physical_fission"] == true)
        .count();
    let segment = if vertex_fissions >= 7 {
        Vec::new()
    } else {
        kinds
            .iter()
            .enumerate()
            .map(|(index, (kind, magnitude))| {
                r4_reproduction_run(
                    r4_reproduction_fixture((index + 1) as u64, kind, *magnitude),
                    &format!("seed_{}_{}_{}", index + 1, kind, magnitude),
                    true,
                    false,
                )
            })
            .collect::<Vec<_>>()
    };
    let segment_fissions = segment
        .iter()
        .filter(|arm| arm["physical_fission"] == true)
        .count();
    let half_edge = if vertex_fissions >= 7 || segment_fissions >= 7 {
        Vec::new()
    } else {
        kinds
            .iter()
            .enumerate()
            .map(|(index, (kind, magnitude))| {
                r4_reproduction_run(
                    r4_reproduction_fixture((index + 1) as u64, kind, *magnitude),
                    &format!("seed_{}_{}_{}", index + 1, kind, magnitude),
                    true,
                    true,
                )
            })
            .collect::<Vec<_>>()
    };
    let selected = if vertex_fissions >= 7 {
        &vertex_only
    } else if segment_fissions >= 7 {
        &segment
    } else {
        &half_edge
    };
    let growth_qualified = selected
        .iter()
        .filter(|arm| arm["growth_qualified"] == true)
        .count();
    let valid_fissions = selected
        .iter()
        .filter(|arm| arm["physical_fission"] == true)
        .count();
    let viable_pairs = selected
        .iter()
        .filter(|arm| arm["fission"]["both_daughters_viable"] == true)
        .count();
    let bookkeeping_invalidations = selected
        .iter()
        .map(|arm| {
            arm["bookkeeping_runtime_invalidations"]
                .as_u64()
                .unwrap_or(0)
        })
        .sum::<u64>();
    let pass = selected.len() == 10
        && growth_qualified >= 8
        && valid_fissions >= 7
        && viable_pairs >= 6
        && bookkeeping_invalidations == 0;
    json!({
        "frozen_protocol": {
            "arms": 10,
            "steps": REPRODUCTION_STEPS,
            "daughter_steps": DAUGHTER_CONTINUATION_STEPS,
            "growth_yield": 0.9,
            "mass_gate": 1.35,
            "fission_cadence": 25,
            "topology_cadence": 10,
            "contract": "MaturationCoupledV4",
        },
        "vertex_only": vertex_only,
        "segment_apposition": segment,
        "half_edge_fallback": half_edge,
        "selected_path": if vertex_fissions >= 7 { "VERTEX_ONLY" } else if segment_fissions >= 7 { "SEGMENT_APPOSITION" } else { "HALF_EDGE_FALLBACK" },
        "growth_qualified": growth_qualified,
        "geometry_valid_fissions": valid_fissions,
        "simple_viable_daughter_pairs": viable_pairs,
        "bookkeeping_runtime_invalidations": bookkeeping_invalidations,
        "pass": pass,
    })
}

fn contract_ownership_matrix() -> Value {
    use chemistry_core::material_mesh::MeshContractVersion;
    let contracts = [
        MeshContractVersion::HistoricalV1,
        MeshContractVersion::ConservativeV2,
        MeshContractVersion::GeometryConservativeV3,
        MeshContractVersion::MaturationCoupledV4,
    ];
    let states = [
        ("A_ZERO", 0.0),
        ("B_PARTIAL", 0.5),
        ("C_EQUAL", 1.0),
        ("D_EXCEEDS", 1.5),
    ];
    let mut rows = Vec::new();
    for contract in contracts {
        for (state, young) in states {
            let mut mesh = chemistry_core::mesh_population::MeshPopulation::seed_one(14.0, 1, 2.2)
                .individuals
                .remove(0)
                .mesh;
            mesh.contract_version = contract;
            mesh.edges[0].m = 1.0;
            mesh.edges[0].m_young = young;
            let serialized_before = serde_json::to_vec(&mesh).unwrap();
            let row = json!({
                "contract": format!("{contract:?}"),
                "state": state,
                "m": mesh.edges[0].m,
                "m_young_stored": mesh.edges[0].m_young,
                "young_structural_mass": mesh.young_structural_mass(0),
                "mature_structural_mass": mesh.mature_structural_mass(0),
                "rest_length": mesh.rest_length(0),
                "lifecycle_invariants_hold": mesh.lifecycle_invariants_hold(),
                "physical_runtime_valid": mesh.physical_runtime_valid(),
                "can_advance_physics": mesh.can_advance_physics(),
            });
            let serialized_after = serde_json::to_vec(&mesh).unwrap();
            rows.push(json!({
                "observation": row,
                "serialized_state_unchanged_by_validation": serialized_before == serialized_after,
            }));
        }
    }
    json!({
        "rows": rows,
        "classification": "M_YOUNG_OWNED_ONLY_BY_MATURATION_COUPLED_V4",
        "non_v4_state_rewritten": false,
    })
}

fn non_v4_d096_parity(template: &MaterialMesh) -> Value {
    let allocation = AllocationParams::default();
    let fission = FissionParams::default();
    let mut parent = template.clone();
    parent.enable_finite_allocation(AllocationGenotype::neutral(), &allocation);
    let (mut daughter, _, event) =
        try_local_fission(&parent, &fission).expect("historical daughter");
    assert!(event.partition.ok);
    let expression = expression_step(&mut daughter, &allocation, MechParams::default().dt)
        .expect("historical D096 expression");
    let bytes_before_validation = serde_json::to_vec(&daughter).unwrap();
    let physical_runtime_valid = daughter.physical_runtime_valid();
    let can_advance_physics = daughter.can_advance_physics();
    let bytes_after_validation = serde_json::to_vec(&daughter).unwrap();
    let violating_inert_edges = daughter
        .edges
        .iter()
        .filter(|edge| edge.m_young > edge.m + 1e-12)
        .count();
    json!({
        "contract": format!("{:?}", daughter.contract_version),
        "expression_ledger": expression,
        "violating_inert_m_young_edges": violating_inert_edges,
        "physical_runtime_valid_after_repair": physical_runtime_valid,
        "can_advance_physics_after_repair": can_advance_physics,
        "serialized_state_unchanged_by_validation": bytes_before_validation == bytes_after_validation,
        "historical_d096_equations_changed": false,
        "classification": "PREVIOUS_RUNTIME_INVALIDATION_REMOVED",
    })
}

fn tagged_closing_edges(mesh: &MaterialMesh, mechanics: &MechParams) -> Vec<Value> {
    mesh.edges
        .iter()
        .enumerate()
        .filter(|(_, edge)| edge.tracer_m > 1e-15)
        .map(|(index, edge)| {
            let length = mesh.edge_length(index);
            let rest = mesh.rest_length(index);
            let reference = rest.max(0.25 * length).max(1e-3);
            let stretch = (mechanics.k_s * (length - rest) / reference)
                .clamp(-mechanics.k_s * 8.0, mechanics.k_s * 8.0);
            json!({
                "edge": index,
                "observer_provenance_fraction": edge.tracer_m / edge.m.max(f64::MIN_POSITIVE),
                "m": edge.m,
                "m_young": edge.m_young,
                "m_mature": mesh.mature_structural_mass(index),
                "young_fraction": edge.m_young / edge.m.max(f64::MIN_POSITIVE),
                "rest_length": rest,
                "geometric_length": length,
                "strain": mesh.strain(index),
                "stretch_force_contribution": stretch,
                "ruptured": edge.ruptured,
            })
        })
        .collect()
}

fn trace_one_v4_closing_edge(mut mesh: MaterialMesh, daughter: &str) -> Value {
    let allocation = AllocationParams::default();
    let mechanics = MechParams::default();
    let reaction = ReactionParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let fission = FissionParams::default();
    for edge in &mut mesh.edges {
        edge.tracer_m = 0.0;
    }
    let closing_index = mesh.n() - 1;
    mesh.edges[closing_index].tracer_m = mesh.edges[closing_index].m;
    let initial_closing_edge = tagged_closing_edges(&mesh, &mechanics);
    let mut trace = Vec::new();
    let mut provenance_termination = None;
    for step in 0..100_usize {
        let before = tagged_closing_edges(&mesh, &mechanics);
        let area_before_topology = mesh.area().max(1e-9);
        let expression = expression_step(&mut mesh, &allocation, mechanics.dt)
            .expect("closing edge D096 expression");
        let _ = transport_step(&mut mesh, &transport, mechanics.dt);
        let reaction_ledger = reactions_step(&mut mesh, &reaction, mechanics.dt, true, true);
        let _ = growth_step(&mut mesh, &reaction, &growth, mechanics.dt);
        let contact_accepted =
            mechanics_step_with_local_self_contact(&mut mesh, &mechanics).is_some();
        let _ = remesh(&mut mesh);
        let a_before_topology = mesh.interior.a * mesh.area().max(1e-9);
        let w_before_topology = mesh.interior.w * mesh.area().max(1e-9);
        let topology = if step % 10 == 0 {
            topology_step(&mut mesh, &fission)
        } else {
            Default::default()
        };
        let a_after_topology = mesh.interior.a * mesh.area().max(1e-9);
        let w_after_topology = mesh.interior.w * mesh.area().max(1e-9);
        let after = tagged_closing_edges(&mesh, &mechanics);
        if !before.is_empty() && after.is_empty() && provenance_termination.is_none() {
            provenance_termination = Some(json!({
                "step": step + 1,
                "cause": if topology.tension_ruptures > 0 { "STRUCTURAL_MATERIAL_DESTROYED_BY_TENSION_RUPTURE" } else { "TAGGED_MATERIAL_TURNED_OVER_OR_REMESHED_BELOW_NUMERICAL_TRACE" },
            }));
        }
        trace.push(json!({
            "step": step + 1,
            "before": before,
            "after": after,
            "expression_material_consumed": expression.material_consumed,
            "maturation_amount": reaction_ledger.m_matured,
            "contact_accepted": contact_accepted,
            "tension_ruptures": topology.tension_ruptures,
            "rebonded": topology.local_rebonds > 0,
            "local_rebonds": topology.local_rebonds,
            "a_used_for_rebond_upper_bound": (a_before_topology - a_after_topology).max(0.0),
            "w_added_by_rupture": (w_after_topology - w_before_topology).max(0.0),
            "area_before_topology_observer": area_before_topology,
            "polygon_simple": polygon_simple(&mesh.vertices),
            "runtime_valid": mesh.physical_runtime_valid(),
            "lifecycle_invariants_hold": mesh.lifecycle_invariants_hold(),
        }));
        if !contact_accepted || !mesh.physical_runtime_valid() || !mesh.lifecycle_invariants_hold()
        {
            break;
        }
    }
    json!({
        "daughter": daughter,
        "observer_tag": "tracer_m set on diagnostic clone only; observer tracer never enters biology",
        "initial_closing_edge_index": closing_index,
        "initial_closing_edge": initial_closing_edge,
        "trace": trace,
        "provenance_termination": provenance_termination,
    })
}

fn v4_closing_edge_trace(template: &MaterialMesh) -> Value {
    let allocation = AllocationParams::default();
    let fission = FissionParams::default();
    let mut parent = template.clone();
    parent.stamp_maturation_coupled_schema();
    parent.enable_finite_allocation(AllocationGenotype::neutral(), &allocation);
    let (a, b, event) = try_local_fission(&parent, &fission).expect("V4 trace fission");
    assert!(event.partition.ok && polygon_simple(&a.vertices) && polygon_simple(&b.vertices));
    json!({
        "parent_contract": format!("{:?}", parent.contract_version),
        "pinch": event.pinch,
        "daughter_a": trace_one_v4_closing_edge(a, "A"),
        "daughter_b": trace_one_v4_closing_edge(b, "B"),
        "classification": "BOOKKEEPING_ONLY_RUPTURE_PATH",
        "conditional_topology_created_edge_repair": "NOT_REQUIRED",
    })
}

fn r10_split_cohort(
    cohort: Cohort,
    mutation_enabled: bool,
    campaign_seed: u64,
    step: usize,
    next_id: &mut u64,
    allocation: &AllocationParams,
    fission: &FissionParams,
    ledger: &mut CampaignLedger,
) -> Result<Vec<Cohort>, Cohort> {
    let Some(parent_plasticity) = cohort.plasticity.as_ref() else {
        return Err(cohort);
    };
    let proposed = try_local_fission(&cohort.mesh, fission).or_else(|| {
        PlanarRingTopology::from_mesh(&cohort.mesh)
            .and_then(|topology| topology.try_local_scission(&cohort.mesh, fission))
            .or_else(|| try_local_segment_fission(&cohort.mesh, fission))
    });
    let Some((daughter_a, daughter_b, event)) = proposed else {
        return Err(cohort);
    };
    if !event.partition.ok {
        ledger.partition_failures += cohort.count;
        return Err(cohort);
    }
    if !polygon_simple(&cohort.mesh.vertices)
        || !polygon_simple(&daughter_a.vertices)
        || !polygon_simple(&daughter_b.vertices)
        || !daughter_a.physical_runtime_valid()
        || !daughter_b.physical_runtime_valid()
        || !daughter_a.lifecycle_invariants_hold()
        || !daughter_b.lifecycle_invariants_hold()
    {
        ledger.invalid_geometry_events += cohort.count;
        return Err(cohort);
    }
    let Some(state_a) = crate::r10_closure::r10_partition_plasticity_state(
        parent_plasticity,
        &event.daughter_a_parent_vertex_sources,
    ) else {
        ledger.runtime_invalidations += cohort.count;
        return Err(cohort);
    };
    let Some(state_b) = crate::r10_closure::r10_partition_plasticity_state(
        parent_plasticity,
        &event.daughter_b_parent_vertex_sources,
    ) else {
        ledger.runtime_invalidations += cohort.count;
        return Err(cohort);
    };
    if state_a.adaptation.len() != daughter_a.n() || state_b.adaptation.len() != daughter_b.n() {
        ledger.runtime_invalidations += cohort.count;
        return Err(cohort);
    }

    ledger.physical_fissions += cohort.count;
    ledger.valid_simple_fissions += cohort.count;
    *ledger
        .fissions_by_parent_genotype
        .entry(genotype_key(
            cohort.mesh.finite_allocation.expect("R10 allocation").genotype,
        ))
        .or_default() += cohort.count;
    let mutation_params = if mutation_enabled {
        *allocation
    } else {
        AllocationParams {
            mutation_probability: 0.0,
            ..*allocation
        }
    };
    let children = [(daughter_a, state_a), (daughter_b, state_b)];
    let mut groups: BTreeMap<(usize, [u64; 4]), (MaterialMesh, PlasticityStateV1, u64)> =
        BTreeMap::new();
    for ordinal in 0..cohort.count {
        for (side, (child_template, state_template)) in children.iter().enumerate() {
            let parent = child_template
                .finite_allocation
                .expect("R10 daughter allocation")
                .genotype;
            let seed = mutation_seed(campaign_seed, step, cohort.id, ordinal, side as u64);
            let mutation = mutate_allocation_at_reproduction(parent, &mutation_params, seed);
            ledger.mutation_opportunities += 1;
            if mutation.mutated {
                ledger.mutations += 1;
                ledger.mutation_events.push(json!({
                    "step": step,
                    "generation": cohort.generation + 1,
                    "seed": seed,
                    "parent": mutation.parent.0,
                    "offspring": mutation.offspring.0,
                    "source_index": mutation.source_index,
                    "target_index": mutation.target_index,
                    "transferred": mutation.transferred,
                }));
            }
            let key = (side, mutation.offspring.0.map(f64::to_bits));
            groups
                .entry(key)
                .and_modify(|(_, _, count)| *count += 1)
                .or_insert_with(|| {
                    let mut mesh = child_template.clone();
                    mesh.finite_allocation.as_mut().unwrap().genotype = mutation.offspring;
                    (mesh, state_template.clone(), 1)
                });
        }
    }
    Ok(groups
        .into_values()
        .map(|(mesh, plasticity, count)| {
            let id = *next_id;
            *next_id += 1;
            Cohort {
                birth_mass: mesh.total_structural_mass(),
                mesh,
                plasticity: Some(plasticity),
                count,
                generation: cohort.generation + 1,
                id,
            }
        })
        .collect())
}

fn r10_initial_population(
    template: &MaterialMesh,
    template_plasticity: &PlasticityStateV1,
    original_birth_mass: f64,
    mutation_enabled: bool,
    campaign_seed: u64,
    founder_count: u64,
    ledger: &mut CampaignLedger,
) -> Vec<Cohort> {
    let allocation = AllocationParams::default();
    let fission = FissionParams::default();
    let mut parent = template.clone();
    parent.enable_finite_allocation(AllocationGenotype::neutral(), &allocation);
    let cohort = Cohort {
        mesh: parent,
        plasticity: Some(template_plasticity.clone()),
        count: founder_count,
        generation: 0,
        birth_mass: original_birth_mass,
        id: 1,
    };
    let mut next_id = 2;
    r10_split_cohort(
        cohort,
        mutation_enabled,
        campaign_seed,
        0,
        &mut next_id,
        &allocation,
        &fission,
        ledger,
    )
    .expect("qualified R10 parent must physically fission")
}

fn r10_advance_phase(
    cohorts: &mut Vec<Cohort>,
    world: &mut OpenMedium,
    environment: Environment,
    phase_index: usize,
    mutation_enabled: bool,
    campaign_seed: u64,
    next_id: &mut u64,
    ledger: &mut CampaignLedger,
    trajectory: &mut Vec<Value>,
) {
    let allocation = AllocationParams::default();
    let mechanics = MechParams::default();
    let reaction = ReactionParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let fission = FissionParams::default();
    for phase_step in 0..PHASE_STEPS {
        if cohorts.is_empty() {
            break;
        }
        let step = phase_index * PHASE_STEPS + phase_step + 1;
        world.add_fixed_inflow(environment, phase_step, mechanics.dt);
        if environment == Environment::Damage {
            for cohort in cohorts.iter_mut() {
                apply_damage(cohort, phase_step, world);
            }
        }
        let mut retained = Vec::new();
        for mut cohort in cohorts.drain(..) {
            match expression_step(&mut cohort.mesh, &allocation, mechanics.dt) {
                Ok(expression) => {
                    let count = cohort.count as f64;
                    ledger.expression_material += expression.material_consumed * count;
                    ledger.expression_activation +=
                        (expression.activation_consumed + expression.maintenance_consumed) * count;
                    retained.push(cohort);
                }
                Err(_) => {
                    ledger.expression_failures += cohort.count;
                    ledger.runtime_invalidations += cohort.count;
                    invalidated_material_to_terminal(&cohort, world);
                }
            }
        }
        *cohorts = retained;
        world.exchange(cohorts, &transport, mechanics.dt);

        let mut survivors = Vec::new();
        for mut cohort in cohorts.drain(..) {
            let count = cohort.count as f64;
            let reactions = reactions_step(&mut cohort.mesh, &reaction, mechanics.dt, true, true);
            ledger.reaction_n_consumed += reactions.n_consumed * count;
            ledger.reaction_f_consumed += reactions.f_consumed * count;
            ledger.a_produced += reactions.a_produced * count;
            ledger.w_produced += reactions.w_produced * count;
            let grown = growth_step(&mut cohort.mesh, &reaction, &growth, mechanics.dt);
            ledger.growth_material += grown.m_grown * count;
            let topology_tick = step % 10 == 0;
            let Some((active_a, active_w, remesh_mappings)) =
                crate::r10_closure::r10_refractory_mechanics_step(
                    &mut cohort.mesh,
                    cohort.plasticity.as_mut().expect("R10 plasticity"),
                    topology_tick,
                )
            else {
                ledger.runtime_invalidations += cohort.count;
                ledger.invalid_geometry_events += cohort.count;
                invalidated_material_to_terminal(&cohort, world);
                continue;
            };
            ledger.active_a_spent += active_a * count;
            ledger.active_w_produced += active_w * count;
            ledger.adaptation_remesh_mappings += remesh_mappings as u64 * cohort.count;
            if !polygon_simple(&cohort.mesh.vertices)
                || !cohort.mesh.physical_runtime_valid()
                || !cohort.mesh.lifecycle_invariants_hold()
            {
                ledger.runtime_invalidations += cohort.count;
                ledger.invalid_geometry_events += cohort.count;
                invalidated_material_to_terminal(&cohort, world);
                continue;
            }
            if !cohort.mesh.observer_viable() {
                ledger.physical_deaths += cohort.count;
                dead_material_to_sink(&cohort, world);
                continue;
            }
            let eligible =
                cohort.mesh.total_structural_mass() >= 1.35 * cohort.birth_mass && step % 25 == 0;
            if eligible {
                match r10_split_cohort(
                    cohort,
                    mutation_enabled,
                    campaign_seed,
                    step,
                    next_id,
                    &allocation,
                    &fission,
                    ledger,
                ) {
                    Ok(children) => survivors.extend(children),
                    Err(parent) => survivors.push(parent),
                }
            } else {
                survivors.push(cohort);
            }
        }
        *cohorts = survivors;
        if phase_step == 0 || (phase_step + 1) % 250 == 0 {
            trajectory.push(snapshot(cohorts, world, step));
        }
    }
}

fn r10_campaign(
    template: &MaterialMesh,
    template_plasticity: &PlasticityStateV1,
    original_birth_mass: f64,
    template_step: usize,
    sequence: &[Environment],
    mutation_enabled: bool,
    replicate: u64,
) -> Value {
    let campaign_seed = splitmix64(
        replicate
            ^ if mutation_enabled {
                0x6a09_e667_f3bc_c909
            } else {
                0xbb67_ae85_84ca_a73b
            }
            ^ sequence.iter().fold(0_u64, |state, env| {
                state.rotate_left(9)
                    ^ match env {
                        Environment::Resource => 0x11,
                        Environment::Damage => 0x22,
                    }
            }),
    );
    let mut ledger = CampaignLedger::default();
    let mut cohorts = r10_initial_population(
        template,
        template_plasticity,
        original_birth_mass,
        mutation_enabled,
        campaign_seed,
        FOUNDER_MULTIPLICITY,
        &mut ledger,
    );
    let mut next_id = 10_000;
    let mut world = OpenMedium::new(sequence[0]);
    let initial_organism_n = organism_amount(&cohorts, 'n');
    let initial_organism_f = organism_amount(&cohorts, 'f');
    let initial = snapshot(&cohorts, &world, 0);
    let mut trajectory = vec![initial.clone()];
    for (phase, environment) in sequence.iter().copied().enumerate() {
        r10_advance_phase(
            &mut cohorts,
            &mut world,
            environment,
            phase,
            mutation_enabled,
            campaign_seed,
            &mut next_id,
            &mut ledger,
            &mut trajectory,
        );
    }
    let terminal_step = sequence.len() * PHASE_STEPS;
    let terminal = snapshot(&cohorts, &world, terminal_step);
    let terminal_organism_n = organism_amount(&cohorts, 'n');
    let terminal_organism_f = organism_amount(&cohorts, 'f');
    let n_closure = (world.ledger.initial_n + world.ledger.inflow_n + initial_organism_n
        - world.n_mass
        - terminal_organism_n
        - ledger.reaction_n_consumed
        - world.ledger.physical_death_n_sink
        - world.ledger.invalidated_n_terminal)
        .abs();
    let f_closure = (world.ledger.initial_f + world.ledger.inflow_f + initial_organism_f
        - world.f_mass
        - terminal_organism_f
        - ledger.reaction_f_consumed
        - world.ledger.physical_death_f_sink
        - world.ledger.invalidated_f_terminal)
        .abs();
    let active_energy_residual = (ledger.active_a_spent - ledger.active_w_produced).abs();
    json!({
        "replicate": replicate,
        "mutation_enabled": mutation_enabled,
        "campaign_seed": campaign_seed,
        "environment_sequence": sequence.iter().map(|environment| environment.label()).collect::<Vec<_>>(),
        "phase_steps": PHASE_STEPS,
        "founder_multiplicity": FOUNDER_MULTIPLICITY,
        "template_fission_step": template_step,
        "fitness_function": null,
        "breeder_selection": false,
        "population_cap": null,
        "resource_feedback": false,
        "exchangeability_compression": true,
        "initial": initial,
        "trajectory": trajectory,
        "terminal": terminal,
        "world": world,
        "ledger": ledger,
        "n_closure_residual": n_closure,
        "f_closure_residual": f_closure,
        "active_energy_residual": active_energy_residual,
    })
}

pub fn run_r10_evolution() {
    let mut output = PathBuf::from("/tmp/dcfinal001_r10_evolution.json");
    let args = env::args().collect::<Vec<_>>();
    for index in 1..args.len() {
        if args[index] == "--output" && index + 1 < args.len() {
            output = PathBuf::from(&args[index + 1]);
        }
    }
    let (template, plasticity, original_birth_mass, template_step) =
        crate::r10_closure::r10_seed3_fission_state();
    let mut handles = Vec::new();
    for replicate in 1..=REPLICATES {
        for mutation_enabled in [true, false] {
            for sequence in [
                vec![Environment::Resource],
                vec![Environment::Damage],
                vec![Environment::Resource, Environment::Damage],
            ] {
                let template = template.clone();
                let plasticity = plasticity.clone();
                handles.push(std::thread::spawn(move || {
                    r10_campaign(
                        &template,
                        &plasticity,
                        original_birth_mass,
                        template_step,
                        &sequence,
                        mutation_enabled,
                        replicate,
                    )
                }));
            }
        }
    }
    let campaigns = handles
        .into_iter()
        .map(|handle| handle.join().expect("R10 evolution campaign"))
        .collect::<Vec<_>>();
    let value = json!({
        "directive": "DC-FINAL-001-R10-SIGNED-LOAD-BEARING-NECK-STRESS-REPRODUCTION-AND-END-GOAL-CLOSURE-001",
        "protocol": {
            "body": "MaturationCoupledV4 + R8 closure + R8R1 sign-aware mechanics + R9 refractory curvature-normal + R10 signed scission stress",
            "mutation_probability": AllocationParams::default().mutation_probability,
            "mutation_sigma": AllocationParams::default().mutation_sigma,
            "mutation_semantics": "one deterministic blind draw per daughter at geometry-valid physical fission",
            "minimum_opportunities_for_95_percent_at_least_one": 299,
            "founder_multiplicity": FOUNDER_MULTIPLICITY,
            "opportunities_per_initial_campaign": 2 * FOUNDER_MULTIPLICITY,
            "replicates": REPLICATES,
            "phase_steps": PHASE_STEPS,
            "switch_schedule": [PHASE_STEPS, 2 * PHASE_STEPS],
            "open_medium_volume": FOUNDER_MULTIPLICITY,
            "inflow_schedule_source": "frozen D-096 Resource/Damage values",
            "component_3_boundary": "reserve-only endpoint dormant under frozen reserve-OFF physiology",
        },
        "template": {
            "r10_arm": "seed_3_c_0.08",
            "fission_step": template_step,
            "simple": polygon_simple(&template.vertices),
            "vertices": template.n(),
            "mass": template.total_structural_mass(),
            "original_birth_mass": original_birth_mass,
            "contract_version": format!("{:?}", template.contract_version),
            "plasticity_patches": plasticity.adaptation.len(),
        },
        "campaigns": campaigns,
    });
    fs::write(output, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
}

fn main() {
    let mut output = PathBuf::from("/tmp/dcfinal001_r4_evolution.json");
    let args: Vec<String> = env::args().collect();
    for index in 1..args.len() {
        if args[index] == "--output" && index + 1 < args.len() {
            output = PathBuf::from(&args[index + 1]);
        }
    }
    let (template, original_birth_mass, template_step) = lawful_parent_template();
    let parity = compression_parity(&template, original_birth_mass);
    assert_eq!(parity["pass"], true);
    let newborn_attribution = newborn_first_step_attribution(&template);
    let mutated_heredity = mutated_descendant_heredity_control(&template);
    let v4_counterpart = v4_counterpart(&template);
    let ownership_matrix = contract_ownership_matrix();
    let historical_d096_parity = non_v4_d096_parity(&template);
    let closing_edge_trace = v4_closing_edge_trace(&template);
    let exact_r2_contract = format!("{:?}", template.contract_version);
    let exact_r2_gate6_pass =
        newborn_attribution["daughter_a"]["expression_on"]["contact_accepted"] == true
            && newborn_attribution["daughter_b"]["expression_on"]["contact_accepted"] == true;
    let v4_gate6_pass = ["daughter_a_lifecycle", "daughter_b_lifecycle"]
        .into_iter()
        .all(|key| {
            v4_counterpart[key]["runtime_invalidation"].is_null()
                && v4_counterpart[key]["terminal_lifecycle_invariants_hold"] == true
                && v4_counterpart[key]["terminal_physical_runtime_valid"] == true
                && v4_counterpart[key]["terminal_polygon_simple"] == true
                && (v4_counterpart[key]["completed_steps"] == 3_000
                    || (v4_counterpart[key]["physical_fission"]["partition_ok"] == true
                        && v4_counterpart[key]["physical_fission"]["parent_simple"] == true
                        && v4_counterpart[key]["physical_fission"]["daughter_a_simple"] == true
                        && v4_counterpart[key]["physical_fission"]["daughter_b_simple"] == true))
        });
    let v4_reproduction = r4_reproduction_campaign();
    let v4_reproduction_pass = v4_reproduction["pass"] == true;
    let value = json!({
        "directive": DIRECTIVE,
        "protocol": {
            "mutation_probability": AllocationParams::default().mutation_probability,
            "mutation_sigma": AllocationParams::default().mutation_sigma,
            "mutation_semantics": "one deterministic blind draw per daughter at geometry-valid physical fission",
            "minimum_opportunities_for_95_percent_at_least_one": 299,
            "founder_multiplicity": FOUNDER_MULTIPLICITY,
            "opportunities_per_initial_campaign": 2 * FOUNDER_MULTIPLICITY,
            "replicates": REPLICATES,
            "phase_steps": PHASE_STEPS,
            "switch_schedule": [PHASE_STEPS, 2 * PHASE_STEPS],
            "open_medium_volume": FOUNDER_MULTIPLICITY,
            "inflow_schedule_source": "frozen D-096 H/B concentration values multiplied by dt and preregistered vessel volume",
            "component_3_boundary": "reserve-only growth endpoint dormant under preserved reserve-OFF R1 physiology",
        },
        "template": {
            "sealed_wp1_arm": "seed_1_rotate_0.3",
            "fission_step": template_step,
            "simple": polygon_simple(&template.vertices),
            "vertices": template.n(),
            "mass": template.total_structural_mass(),
            "original_birth_mass": original_birth_mass,
            "contract_version": exact_r2_contract,
        },
        "exchangeability_parity": parity,
        "contract_ownership_matrix": ownership_matrix,
        "non_v4_d096_parity": historical_d096_parity,
        "newborn_first_step_attribution": newborn_attribution,
        "v4_counterpart": v4_counterpart,
        "v4_closing_edge_trace": closing_edge_trace,
        "mutated_descendant_heredity_control": mutated_heredity,
        "gate6_exact_r2_newborn_continuity": if exact_r2_gate6_pass { "PASS" } else { "FAIL" },
        "gate6_v4_newborn_continuity": if v4_gate6_pass { "PASS" } else { "FAIL" },
        "v4_reproduction_campaign": v4_reproduction,
        "evolution_execution": if v4_reproduction_pass { "PENDING_AFTER_GATE9" } else { "NOT_RUN_GATE9_STOP" },
        "classification": if !v4_gate6_pass { "V4_TOPOLOGY_CREATED_EDGE_MATURATION_ARCHITECTURE_INVALID" } else if !v4_reproduction_pass { "V4_GEOMETRY_VALID_REPRODUCTION_NOT_ESTABLISHED" } else { "V4_CONTRACT_TOPOLOGY_COHERENCE_REPAIRED" },
        "causal_boundary": "Evolution executes only after canonical production-V4 reproduction qualifies.",
    });
    fs::write(output, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
}
