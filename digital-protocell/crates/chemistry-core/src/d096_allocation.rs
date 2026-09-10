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
pub const EQUATION_VERSION_FINITE_CATALYTIC_ALLOCATION_V2: &str =
    "autopoietic_material_mesh_finite_catalytic_allocation_v2_activated_material";
pub const FINITE_ALLOCATION_V2_SCHEMA_VERSION: u32 = 3;
pub const EQUATION_VERSION_FINITE_CATALYTIC_ALLOCATION_V3: &str =
    "autopoietic_material_mesh_finite_catalytic_allocation_v3_intensive_gain";
pub const FINITE_ALLOCATION_V3_SCHEMA_VERSION: u32 = 4;
pub const FUNCTIONS: usize = 4;

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
        self.0
            .iter()
            .all(|x| x.is_finite() && *x >= params.allocation_min && *x <= params.allocation_max)
            && (self.0.iter().sum::<f64>() - params.total_budget).abs() <= 1e-12
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

    pub fn candidate_hash_v2(self, params: &AllocationParams) -> String {
        let mut bytes = EQUATION_VERSION_FINITE_CATALYTIC_ALLOCATION_V2
            .as_bytes()
            .to_vec();
        bytes.extend_from_slice(&FINITE_ALLOCATION_V2_SCHEMA_VERSION.to_le_bytes());
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

    pub fn candidate_hash_v3(self, params: &AllocationParams) -> String {
        let mut bytes = EQUATION_VERSION_FINITE_CATALYTIC_ALLOCATION_V3
            .as_bytes()
            .to_vec();
        bytes.extend_from_slice(&FINITE_ALLOCATION_V3_SCHEMA_VERSION.to_le_bytes());
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct ExpressionLedger {
    pub synthesis: [f64; FUNCTIONS],
    pub material_consumed: f64,
    /// D096-v2 catalyst precursor taken directly from activated material A.
    /// Historical v1 always records zero here.
    #[serde(default)]
    pub catalyst_precursor_consumed: f64,
    pub activation_consumed: f64,
    pub maintenance_consumed: f64,
    pub turnover_waste: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AllocationMutationLedger {
    pub parent: AllocationGenotype,
    pub offspring: AllocationGenotype,
    pub mutated: bool,
    pub source_index: Option<usize>,
    pub target_index: Option<usize>,
    pub transferred: f64,
    pub rng_state_after: u64,
}

fn mutation_random(state: &mut u64) -> f64 {
    let mut x = (*state).max(1);
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    (x as f64) / (u64::MAX as f64)
}

/// Apply the frozen D-096 mutation contract at one lawful reproduction event.
///
/// The mutation is blind to environment, survival, phenotype, and fitness. It
/// moves material allocation between two coordinates and therefore preserves
/// the exact finite simplex without clipping or renormalization.
pub fn mutate_allocation_at_reproduction(
    parent: AllocationGenotype,
    params: &AllocationParams,
    seed: u64,
) -> AllocationMutationLedger {
    assert!(parent.valid(params), "invalid frozen allocation");
    let mut rng = seed.max(1);
    let mut offspring = parent;
    let mut source_index = None;
    let mut target_index = None;
    let mut transferred = 0.0;
    let mut mutated = false;
    if mutation_random(&mut rng) < params.mutation_probability {
        let pair = (mutation_random(&mut rng) * (FUNCTIONS * (FUNCTIONS - 1)) as f64)
            .floor()
            .min((FUNCTIONS * (FUNCTIONS - 1) - 1) as f64) as usize;
        let source = pair / (FUNCTIONS - 1);
        let offset = pair % (FUNCTIONS - 1);
        let target = if offset >= source { offset + 1 } else { offset };
        let u1 = mutation_random(&mut rng).max(f64::MIN_POSITIVE);
        let u2 = mutation_random(&mut rng);
        let normal = (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos();
        let delta = (normal.abs() * params.mutation_sigma)
            .min(offspring.0[source])
            .min(params.allocation_max - offspring.0[target]);
        if delta > 0.0 {
            offspring.0[source] -= delta;
            offspring.0[target] += delta;
            source_index = Some(source);
            target_index = Some(target);
            transferred = delta;
            mutated = true;
        }
    }
    debug_assert!(offspring.valid(params));
    AllocationMutationLedger {
        parent,
        offspring,
        mutated,
        source_index,
        target_index,
        transferred,
        rng_state_after: rng,
    }
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

pub fn allocation_v2_schema_load_ok(mesh: &MaterialMesh, params: &AllocationParams) -> bool {
    mesh.equation_id == EQUATION_VERSION_FINITE_CATALYTIC_ALLOCATION_V2
        && mesh.schema_version == FINITE_ALLOCATION_V2_SCHEMA_VERSION
        && mesh
            .finite_allocation
            .is_some_and(|state| state.genotype.valid(params))
}

pub fn allocation_v3_schema_load_ok(mesh: &MaterialMesh, params: &AllocationParams) -> bool {
    mesh.equation_id == EQUATION_VERSION_FINITE_CATALYTIC_ALLOCATION_V3
        && mesh.schema_version == FINITE_ALLOCATION_V3_SCHEMA_VERSION
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
    let maturation_coupled = next.is_maturation_coupled();
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
        if maturation_coupled {
            // MaturationCoupledV4 stores young structure as a physical subpool
            // of total edge structure. D-096 withdraws a uniform fraction of
            // structural material, so the same fraction must be withdrawn from
            // every structural subpool. This preserves the young:mature ratio
            // and cannot create m_young > m on fully-young fission edges.
            edge.m_young *= fraction_left;
            // tracer_m is an observer-only tagged subset of edge.m.
            edge.tracer_m *= fraction_left;
        }
    }
    let activated_spent = ledger.activation_consumed + maintenance;
    next.interior.a -= activated_spent / area;
    if maturation_coupled {
        // In the production V4 material contract, dissipated activated
        // material is transferred to the existing waste pool. Historical
        // non-V4 D-096 semantics remain byte-for-byte unchanged.
        next.interior.w += activated_spent / area;
    }
    next.interior.w += ledger.turnover_waste / area;
    *mesh = next;
    Ok(ledger)
}

/// D096-v2 activated-material expression.
///
/// The synthesis rate, genotype allocation, activation overhead, maintenance,
/// and turnover constants are exactly the historical D096 values. Only the
/// catalyst precursor substrate changes: synthesized catalyst material is
/// withdrawn from A rather than from the load-bearing structural mesh.
pub fn expression_step_activated_material_v2(
    mesh: &mut MaterialMesh,
    params: &AllocationParams,
    dt: f64,
) -> Result<ExpressionLedger, ExpressionReject> {
    if !allocation_v2_schema_load_ok(mesh, params) {
        return Err(ExpressionReject::IncompatibleSchema);
    }
    if !dt.is_finite() || dt <= 0.0 {
        return Err(ExpressionReject::InvalidStep);
    }
    let mut next = mesh.clone();
    let maturation_coupled = next.is_maturation_coupled();
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
    let max_syn_a = ((activated - maintenance) / (1.0 + params.activation_cost)).max(0.0) / dt;
    let actual_syn = j_syn.min(max_syn_a);
    let mut ledger = ExpressionLedger::default();
    for i in 0..FUNCTIONS {
        let j = state.genotype.0[i] * actual_syn;
        let turnover = params.turnover_rate * state.catalysts[i];
        state.catalysts[i] = (state.catalysts[i] + (j - turnover) * dt).max(0.0);
        ledger.synthesis[i] = j * dt;
        ledger.turnover_waste += turnover * dt;
    }
    ledger.catalyst_precursor_consumed = ledger.synthesis.iter().sum();
    ledger.material_consumed = 0.0;
    ledger.activation_consumed = params.activation_cost * ledger.catalyst_precursor_consumed;
    ledger.maintenance_consumed = maintenance;
    let activated_spent = ledger.catalyst_precursor_consumed
        + ledger.activation_consumed
        + ledger.maintenance_consumed;
    if activated_spent > activated + 1e-12 {
        return Err(ExpressionReject::InsufficientActivatedResource);
    }
    next.interior.a -= activated_spent / area;
    if maturation_coupled {
        next.interior.w += (ledger.activation_consumed + maintenance) / area;
    }
    next.interior.w += ledger.turnover_waste / area;
    *mesh = next;
    Ok(ledger)
}

/// D096-v3 keeps the complete D096-v2 activated-material expression law.
/// Its only versioned change is the intensive interpretation in
/// [`function_gain`]; expression never normalizes or otherwise changes the
/// physical catalyst stocks.
pub fn expression_step_activated_material_v3(
    mesh: &mut MaterialMesh,
    params: &AllocationParams,
    dt: f64,
) -> Result<ExpressionLedger, ExpressionReject> {
    if !allocation_v3_schema_load_ok(mesh, params) {
        return Err(ExpressionReject::IncompatibleSchema);
    }
    if !dt.is_finite() || dt <= 0.0 {
        return Err(ExpressionReject::InvalidStep);
    }
    let mut next = mesh.clone();
    let maturation_coupled = next.is_maturation_coupled();
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
    let max_syn_a = ((activated - maintenance) / (1.0 + params.activation_cost)).max(0.0) / dt;
    let actual_syn = j_syn.min(max_syn_a);
    let mut ledger = ExpressionLedger::default();
    for i in 0..FUNCTIONS {
        let j = state.genotype.0[i] * actual_syn;
        let turnover = params.turnover_rate * state.catalysts[i];
        state.catalysts[i] = (state.catalysts[i] + (j - turnover) * dt).max(0.0);
        ledger.synthesis[i] = j * dt;
        ledger.turnover_waste += turnover * dt;
    }
    ledger.catalyst_precursor_consumed = ledger.synthesis.iter().sum();
    ledger.material_consumed = 0.0;
    ledger.activation_consumed = params.activation_cost * ledger.catalyst_precursor_consumed;
    ledger.maintenance_consumed = maintenance;
    let activated_spent = ledger.catalyst_precursor_consumed
        + ledger.activation_consumed
        + ledger.maintenance_consumed;
    if activated_spent > activated + 1e-12 {
        return Err(ExpressionReject::InsufficientActivatedResource);
    }
    next.interior.a -= activated_spent / area;
    if maturation_coupled {
        next.interior.w += (ledger.activation_consumed + maintenance) / area;
    }
    next.interior.w += ledger.turnover_waste / area;
    *mesh = next;
    Ok(ledger)
}

pub fn catalytic_gain(catalyst: f64) -> f64 {
    1.0 + catalyst.max(0.0) / (0.1 + catalyst.max(0.0))
}

pub fn function_gain(mesh: &MaterialMesh, index: usize) -> f64 {
    mesh.finite_allocation
        .map(|state| {
            if mesh.equation_id == EQUATION_VERSION_FINITE_CATALYTIC_ALLOCATION_V3
                && mesh.schema_version == FINITE_ALLOCATION_V3_SCHEMA_VERSION
            {
                let area = mesh.area();
                assert!(
                    area.is_finite() && area > 0.0,
                    "D096-v3 gain requires positive physical area"
                );
                catalytic_gain(state.catalysts[index] / area)
            } else {
                catalytic_gain(state.catalysts[index])
            }
        })
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
        if !mesh.can_advance_physics() {
            break;
        }
    }
    PreFissionOutcome {
        reserve_change: mesh.interior.r * mesh.area() - initial_reserve,
        structural_change: mesh.total_structural_mass() - initial_material,
        activated_produced,
        damage_applied,
        final_material: mesh.total_structural_mass() + mesh.total_bound_membrane(),
        survived: mesh.observer_viable(),
    }
}
