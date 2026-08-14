//! D-096 inherited finite catalytic-production allocation.
//!
//! The genotype is a fixed simplex. Expression spends real structural material
//! and activated resource; allocation is never clipped or normalized.

use crate::candidate_identity::sha256_hex;
use crate::material_mesh::MaterialMesh;
use serde::{Deserialize, Serialize};

pub const EQUATION_VERSION_FINITE_CATALYTIC_ALLOCATION: &str =
    "autopoietic_material_mesh_finite_catalytic_allocation_v1";
pub const FINITE_ALLOCATION_SCHEMA_VERSION: u32 = 2;
pub const FUNCTIONS: usize = 4;
const RNG_MULTIPLIER: u64 = 0x9E37_79B9_7F4A_7C15;
const RNG_INCREMENT: u64 = 0xD1B5_4A32_D192_ED03;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AllocationGenotype(pub [f64; FUNCTIONS]);

impl Default for AllocationGenotype {
    fn default() -> Self {
        Self::neutral()
    }
}

impl AllocationGenotype {
    pub const fn pulse() -> Self {
        Self([0.45, 0.25, 0.10, 0.20])
    }

    pub const fn damage() -> Self {
        Self([0.20, 0.20, 0.45, 0.15])
    }

    pub const fn neutral() -> Self {
        Self([0.25, 0.25, 0.25, 0.25])
    }

    pub fn valid(self, params: &AllocationParams) -> bool {
        self.0.iter().all(|x| {
            x.is_finite() && *x >= params.allocation_min && *x <= params.allocation_max
        }) && (self.0.iter().sum::<f64>() - params.total_budget).abs() <= 1e-12
    }

    pub fn candidate_hash(self, params: &AllocationParams) -> String {
        let mut bytes = EQUATION_VERSION_FINITE_CATALYTIC_ALLOCATION
            .as_bytes()
            .to_vec();
        bytes.extend_from_slice(&FINITE_ALLOCATION_SCHEMA_VERSION.to_le_bytes());
        for value in self.0 {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for value in [
            params.total_budget,
            params.allocation_min,
            params.allocation_max,
            params.mutation_probability,
            params.mutation_sigma,
            params.synthesis_rate,
            params.activation_cost,
            params.maintenance_rate,
            params.turnover_rate,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        sha256_hex(&bytes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AllocationParams {
    pub total_budget: f64,
    pub allocation_min: f64,
    pub allocation_max: f64,
    pub mutation_probability: f64,
    pub mutation_sigma: f64,
    pub synthesis_rate: f64,
    pub activation_cost: f64,
    pub maintenance_rate: f64,
    pub turnover_rate: f64,
}

impl Default for AllocationParams {
    fn default() -> Self {
        Self {
            total_budget: 1.0,
            allocation_min: 0.0,
            allocation_max: 1.0,
            mutation_probability: 0.01,
            mutation_sigma: 0.15,
            synthesis_rate: 1e-3,
            activation_cost: 0.2,
            maintenance_rate: 1e-5,
            turnover_rate: 1e-4,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct AllocationState {
    pub genotype: AllocationGenotype,
    pub catalysts: [f64; FUNCTIONS],
}

/// Evidence that the physical finite-allocation catalyst pool was partitioned
/// rather than copied during a qualified mesh fission.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CatalystPartitionAudit {
    pub fraction_a: f64,
    pub fraction_b: f64,
    pub pre_catalyst: [f64; FUNCTIONS],
    pub daughter_a_catalyst: [f64; FUNCTIONS],
    pub daughter_b_catalyst: [f64; FUNCTIONS],
    pub residuals: [f64; FUNCTIONS],
    pub max_residual: f64,
    pub conserved: bool,
}

pub fn partition_catalysts(
    parent: AllocationState,
    fraction_a: f64,
    fraction_b: f64,
) -> (AllocationState, AllocationState, CatalystPartitionAudit) {
    let pre = parent.catalysts;
    let mut daughter_a = parent;
    let mut daughter_b = parent;
    daughter_a.catalysts = std::array::from_fn(|i| pre[i] * fraction_a);
    daughter_b.catalysts = std::array::from_fn(|i| pre[i] * fraction_b);
    let residuals = std::array::from_fn(|i| {
        (daughter_a.catalysts[i] + daughter_b.catalysts[i] - pre[i]).abs()
    });
    let max_residual = residuals.iter().copied().fold(0.0, f64::max);
    let audit = CatalystPartitionAudit {
        fraction_a,
        fraction_b,
        pre_catalyst: pre,
        daughter_a_catalyst: daughter_a.catalysts,
        daughter_b_catalyst: daughter_b.catalysts,
        residuals,
        max_residual,
        conserved: (fraction_a + fraction_b - 1.0).abs() <= 1e-12
            && max_residual <= 1e-12 * (1.0 + pre.iter().copied().fold(0.0, f64::max)),
    };
    (daughter_a, daughter_b, audit)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutationReject {
    InvalidParameters,
    InvalidParent,
    InvalidResult,
}

/// Complete provenance for one D-096 genotype-copy decision.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AllocationMutationRecord {
    pub operator: &'static str,
    pub provenance: &'static str,
    pub seed: u64,
    pub mutation_probability: f64,
    pub mutation_sigma: f64,
    pub mutation_occurred: bool,
    pub source: Option<usize>,
    pub target: Option<usize>,
    pub pre_genotype: AllocationGenotype,
    pub post_genotype: AllocationGenotype,
    pub raw_abs_normal: f64,
    pub applied_delta: f64,
}

#[derive(Debug, Clone, Copy)]
struct D096Rng(u64);

impl D096Rng {
    fn new(seed: u64) -> Self {
        Self(seed ^ RNG_MULTIPLIER)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(RNG_MULTIPLIER)
            .wrapping_add(RNG_INCREMENT);
        self.0
    }

    fn unit(&mut self) -> f64 {
        (self.next_u64() as f64 / (u64::MAX as f64 + 1.0)).max(f64::MIN_POSITIVE)
    }

    fn normal(&mut self) -> f64 {
        let radius = (-2.0 * self.unit().ln()).sqrt();
        let angle = std::f64::consts::TAU * self.unit();
        radius * angle.cos()
    }
}

pub fn mutate_allocation_genotype(
    pre_genotype: AllocationGenotype,
    params: &AllocationParams,
    seed: u64,
) -> Result<AllocationMutationRecord, MutationReject> {
    if !params.total_budget.is_finite()
        || !params.allocation_min.is_finite()
        || !params.allocation_max.is_finite()
        || !params.mutation_probability.is_finite()
        || !params.mutation_sigma.is_finite()
        || params.allocation_min > params.allocation_max
        || !(0.0..=1.0).contains(&params.mutation_probability)
        || params.mutation_sigma < 0.0
    {
        return Err(MutationReject::InvalidParameters);
    }
    if !pre_genotype.valid(params) {
        return Err(MutationReject::InvalidParent);
    }
    let mut rng = D096Rng::new(seed);
    let mut post = pre_genotype;
    let mut source = None;
    let mut target = None;
    let mut raw_abs_normal = 0.0;
    let mut applied_delta = 0.0;
    let mutation_occurred = rng.unit() < params.mutation_probability;
    if mutation_occurred {
        let selected_source = (rng.next_u64() as usize) % FUNCTIONS;
        let mut selected_target = (rng.next_u64() as usize) % (FUNCTIONS - 1);
        if selected_target >= selected_source {
            selected_target += 1;
        }
        raw_abs_normal = (rng.normal() * params.mutation_sigma).abs();
        let cap = (post.0[selected_source] - params.allocation_min)
            .min(params.allocation_max - post.0[selected_target])
            .max(0.0);
        applied_delta = raw_abs_normal.min(cap);
        if applied_delta <= 0.0 {
            return Err(MutationReject::InvalidResult);
        }
        post.0[selected_source] -= applied_delta;
        post.0[selected_target] += applied_delta;
        source = Some(selected_source);
        target = Some(selected_target);
    }
    if !post.valid(params) {
        return Err(MutationReject::InvalidResult);
    }
    Ok(AllocationMutationRecord {
        operator: "D096AllocationMutationOperator",
        provenance: "DC-SR-004B;D-096_GATE6;existing_fixed_simplex_contract",
        seed,
        mutation_probability: params.mutation_probability,
        mutation_sigma: params.mutation_sigma,
        mutation_occurred,
        source,
        target,
        pre_genotype,
        post_genotype: post,
        raw_abs_normal,
        applied_delta,
    })
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct ExpressionLedger {
    pub synthesis: [f64; FUNCTIONS],
    pub material_consumed: f64,
    pub activation_consumed: f64,
    pub maintenance_consumed: f64,
    pub turnover_waste: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpressionReject {
    IncompatibleSchema,
    InvalidAllocation,
    InvalidStep,
    InsufficientMaterial,
    InsufficientActivatedResource,
}

pub fn allocation_schema_load_ok(mesh: &MaterialMesh, params: &AllocationParams) -> bool {
    mesh.equation_id == EQUATION_VERSION_FINITE_CATALYTIC_ALLOCATION
        && mesh.schema_version == FINITE_ALLOCATION_SCHEMA_VERSION
        && mesh
            .finite_allocation
            .is_some_and(|state| state.genotype.valid(params))
}

pub fn expression_step(
    mesh: &mut MaterialMesh,
    params: &AllocationParams,
    dt: f64,
) -> Result<ExpressionLedger, ExpressionReject> {
    if !allocation_schema_load_ok(mesh, params) {
        return Err(ExpressionReject::IncompatibleSchema);
    }
    if !dt.is_finite() || dt <= 0.0 {
        return Err(ExpressionReject::InvalidStep);
    }
    let mut next = mesh.clone();
    let area = next.area().max(1e-9);
    let material = next.total_structural_mass().max(0.0);
    let activated = (next.interior.a.max(0.0) * area).max(0.0);
    if material <= 0.0 {
        return Err(ExpressionReject::InsufficientMaterial);
    }
    if activated <= 0.0 {
        return Err(ExpressionReject::InsufficientActivatedResource);
    }
    let state = next.finite_allocation.as_mut().expect("schema checked");
    if !state.genotype.valid(params) {
        return Err(ExpressionReject::InvalidAllocation);
    }
    let total_c = state.catalysts.iter().sum::<f64>();
    let j_syn = params.synthesis_rate * material.min(activated / params.activation_cost);
    let maintenance = (params.maintenance_rate * total_c * dt).min(activated);
    let max_syn_a = ((activated - maintenance) / params.activation_cost).max(0.0) / dt;
    let actual_syn = j_syn.min(max_syn_a);
    let mut ledger = ExpressionLedger::default();
    for i in 0..FUNCTIONS {
        let j = state.genotype.0[i] * actual_syn;
        let turnover = params.turnover_rate * state.catalysts[i];
        state.catalysts[i] = (state.catalysts[i] + (j - turnover) * dt).max(0.0);
        ledger.synthesis[i] = j * dt;
        ledger.turnover_waste += turnover * dt;
    }
    ledger.material_consumed = ledger.synthesis.iter().sum();
    ledger.activation_consumed = params.activation_cost * ledger.material_consumed;
    ledger.maintenance_consumed = maintenance;
    if ledger.material_consumed > material + 1e-12 {
        return Err(ExpressionReject::InsufficientMaterial);
    }
    let fraction_left = (1.0 - ledger.material_consumed / material).max(0.0);
    for edge in &mut next.edges {
        edge.m *= fraction_left;
    }
    next.interior.a -= (ledger.activation_consumed + maintenance) / area;
    next.interior.w += ledger.turnover_waste / area;
    *mesh = next;
    Ok(ledger)
}

pub fn catalytic_gain(catalyst: f64) -> f64 {
    1.0 + catalyst.max(0.0) / (0.1 + catalyst.max(0.0))
}

pub fn function_gain(mesh: &MaterialMesh, index: usize) -> f64 {
    mesh.finite_allocation
        .map(|state| catalytic_gain(state.catalysts[index]))
        .unwrap_or(1.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssayEnvironment {
    H,
    B,
    Neutral,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct EnvironmentLedger {
    pub nutrient: f64,
    pub fuel: f64,
    pub structural_damage: f64,
    pub membrane_damage: f64,
}

/// External assay forcing. The enum remains in the harness; only resulting
/// nutrient/fuel and physical damage enter the organism.
pub fn apply_assay_environment(
    mesh: &mut MaterialMesh,
    environment: AssayEnvironment,
    step: u64,
) -> EnvironmentLedger {
    let (nutrient, fuel) = match environment {
        AssayEnvironment::H if step % 400 < 100 => (2.75, 1.0),
        AssayEnvironment::H => (0.264, 1.0),
        AssayEnvironment::B => (1.98, 1.0),
        AssayEnvironment::Neutral => (1.54, 1.0),
    };
    mesh.exterior.n = nutrient;
    mesh.exterior.f = fuel;
    let mut ledger = EnvironmentLedger {
        nutrient,
        fuel,
        ..EnvironmentLedger::default()
    };
    if environment == AssayEnvironment::B && step % 350 == 0 {
        let structural = 0.08_f64.min(mesh.edges[0].m.max(0.0));
        let membrane = 0.048_f64.min(mesh.edges[0].b.max(0.0));
        mesh.edges[0].m -= structural;
        mesh.edges[0].b -= membrane;
        ledger.structural_damage = structural;
        ledger.membrane_damage = membrane;
    }
    ledger
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct PreFissionOutcome {
    pub reserve_change: f64,
    pub structural_change: f64,
    pub activated_produced: f64,
    pub damage_applied: f64,
    pub final_material: f64,
    pub survived: bool,
}

pub fn pre_fission_assay(
    genotype: AllocationGenotype,
    environment: AssayEnvironment,
    seed: u64,
    steps: usize,
) -> PreFissionOutcome {
    use crate::mesh_growth::{growth_step, GrowthParams};
    use crate::mesh_reactions::{reactions_step, ReactionParams};
    use crate::mesh_transport::{transport_step, TransportParams};
    use crate::metabolic_reserve::ReserveParams;

    let allocation = AllocationParams::default();
    let mut mesh = MaterialMesh::seed_regular(
        12 + (seed % 3) as usize,
        8.0,
        0.0,
        0.0,
        1.0,
        0.8,
        crate::material_mesh::LumpedChem {
            c: 1.0,
            a: 0.5,
            n: 0.8,
            f: 0.8,
            r: 0.5,
            ..crate::material_mesh::LumpedChem::default()
        },
        crate::material_mesh::LumpedChem::default(),
        1.0,
    );
    mesh.enable_finite_allocation(genotype, &allocation);
    let area = mesh.area();
    let mut reaction = ReactionParams::default();
    reaction.reserve = ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, area);
    reaction.reserve.enable = true;
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let initial_reserve = mesh.interior.r * area;
    let initial_material = mesh.total_structural_mass();
    let mut activated_produced = 0.0;
    let mut damage_applied = 0.0;
    for step in 0..steps {
        let env = apply_assay_environment(&mut mesh, environment, step as u64);
        damage_applied += env.structural_damage + env.membrane_damage;
        if expression_step(&mut mesh, &allocation, 0.02).is_err() {
            break;
        }
        let _ = transport_step(&mut mesh, &transport, 0.02);
        let chemistry = reactions_step(&mut mesh, &reaction, 0.02, true, true);
        activated_produced += chemistry.a_produced;
        let _ = growth_step(&mut mesh, &reaction, &growth, 0.02);
        if !mesh.alive {
            break;
        }
    }
    PreFissionOutcome {
        reserve_change: mesh.interior.r * mesh.area() - initial_reserve,
        structural_change: mesh.total_structural_mass() - initial_material,
        activated_produced,
        damage_applied,
        final_material: mesh.total_structural_mass() + mesh.total_bound_membrane(),
        survived: mesh.alive,
    }
}
