//! D-097 observer-only reconstruction of the D-096 processing pathway.

use crate::d096_allocation::{
    apply_assay_environment, catalytic_gain, expression_step, AllocationGenotype,
    AllocationParams, AssayEnvironment,
};
use crate::material_mesh::{LumpedChem, MaterialMesh};
use crate::mesh_growth::{growth_step, GrowthParams};
use crate::mesh_reactions::{reactions_step, ReactionParams};
use crate::mesh_transport::{transport_step, TransportParams};
use crate::metabolic_reserve::{reserve_schema_load_ok, ReserveParams};
use serde::{Deserialize, Serialize};

pub const D097_PROCESSING_IMPLEMENTATION_DEFECT_CONFIRMED: &str =
    "D097_PROCESSING_IMPLEMENTATION_DEFECT_CONFIRMED";
pub const D098_REPAIR_ROUTE: &str = "D-098 - Finite Allocation Processing Implementation Repair";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PathTrace {
    pub allocation: [f64; 4],
    pub processing_expression: f64,
    pub processing_synthesis: f64,
    pub expression_maintenance: f64,
    pub processing_turnover: f64,
    pub stable_expression_step: Option<usize>,
    pub half_expression_step: Option<usize>,
    pub pulse_steps: usize,
    pub catalyst_resource_overlap_steps: usize,
    pub boundary_resource_exposure: f64,
    pub internal_resource_exposure: f64,
    pub nutrient_converted: f64,
    pub fuel_converted: f64,
    pub activated_production: f64,
    pub legacy_activation: f64,
    pub allocated_activation: f64,
    pub processing_share: f64,
    pub reserve_inflow: f64,
    pub reserve_outflow: f64,
    pub reserve_change: f64,
    pub structural_synthesis: f64,
    pub membrane_synthesis: f64,
    pub growth: f64,
    pub readiness_progress: f64,
    pub final_structural_material: f64,
    pub final_membrane_material: f64,
    pub unused_internal_resource: f64,
    pub expression_net_benefit: f64,
    pub reserve_schema_compatible: bool,
    pub accepted_steps: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PathDifference {
    pub processing_allocation: f64,
    pub repair_allocation: f64,
    pub processing_expression: f64,
    pub resource_encounter: f64,
    pub resource_conversion: f64,
    pub activated_production: f64,
    pub reserve_inflow: f64,
    pub reserve_change: f64,
    pub growth: f64,
    pub readiness: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairReconstruction {
    pub seed: u64,
    pub environment: AssayEnvironment,
    pub processing: PathTrace,
    pub repair: PathTrace,
    pub difference: PathDifference,
}

fn seed(genotype: AllocationGenotype, seed: u64) -> (MaterialMesh, ReactionParams) {
    let mut mesh = MaterialMesh::seed_regular(
        12 + (seed % 3) as usize,
        8.0,
        0.0,
        0.0,
        1.0,
        0.8,
        LumpedChem {
            c: 1.0,
            a: 0.5,
            n: 0.8,
            f: 0.8,
            r: 0.5,
            ..LumpedChem::default()
        },
        LumpedChem::default(),
        1.0,
    );
    mesh.enable_finite_allocation(genotype, &AllocationParams::default());
    let mut reaction = ReactionParams::default();
    reaction.reserve = ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, mesh.area());
    reaction.reserve.enable = true;
    (mesh, reaction)
}

pub fn trace_path(
    genotype: AllocationGenotype,
    environment: AssayEnvironment,
    seed_id: u64,
    steps: usize,
) -> PathTrace {
    let allocation = AllocationParams::default();
    let (mut mesh, reaction) = seed(genotype, seed_id);
    let transport = TransportParams::default();
    let growth_params = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let area0 = mesh.area();
    let reserve0 = mesh.interior.r * area0;
    let mass0 = mesh.total_structural_mass();
    let compatible = reserve_schema_load_ok(&mesh, &reaction.reserve);
    let mut trace = PathTrace {
        allocation: genotype.0,
        reserve_schema_compatible: compatible,
        ..PathTrace::default()
    };
    let mut expression_series = Vec::with_capacity(steps);
    for step in 0..steps {
        let env = apply_assay_environment(&mut mesh, environment, step as u64);
        if environment == AssayEnvironment::H && step % 400 < 100 {
            trace.pulse_steps += 1;
        }
        trace.boundary_resource_exposure += env.nutrient + env.fuel;
        trace.internal_resource_exposure += mesh.interior.n.max(0.0) + mesh.interior.f.max(0.0);
        let expression = match expression_step(&mut mesh, &allocation, 0.02) {
            Ok(value) => value,
            Err(_) => break,
        };
        trace.processing_synthesis += expression.synthesis[0];
        trace.expression_maintenance += expression.maintenance_consumed;
        trace.processing_turnover += expression.turnover_waste;
        let catalyst = mesh.finite_allocation.unwrap().catalysts[0];
        expression_series.push(catalyst);
        if catalyst > 0.0 && mesh.interior.n > 0.0 && mesh.interior.f > 0.0 {
            trace.catalyst_resource_overlap_steps += 1;
        }
        let _ = transport_step(&mut mesh, &transport, 0.02);
        let chemistry = reactions_step(&mut mesh, &reaction, 0.02, true, true);
        let growth = growth_step(&mut mesh, &reaction, &growth_params, 0.02);
        trace.nutrient_converted += chemistry.n_consumed;
        trace.fuel_converted += chemistry.f_consumed;
        trace.activated_production += chemistry.a_produced;
        trace.reserve_inflow += chemistry.reserve.a_to_r;
        trace.reserve_outflow += chemistry.reserve.r_to_a + chemistry.reserve.r_to_w;
        trace.structural_synthesis += chemistry.m_produced;
        trace.membrane_synthesis += chemistry.l_produced;
        trace.growth += growth.m_grown;
        trace.accepted_steps += 1;
    }
    trace.processing_expression = mesh
        .finite_allocation
        .map(|state| state.catalysts[0])
        .unwrap_or(0.0);
    if let Some(final_expression) = expression_series.last().copied() {
        trace.half_expression_step = expression_series
            .iter()
            .position(|value| *value >= 0.5 * final_expression);
        trace.stable_expression_step = expression_series
            .iter()
            .position(|value| *value >= 0.95 * final_expression);
    }
    let gain = mesh
        .finite_allocation
        .map(|state| {
            (catalytic_gain(state.catalysts[0]) * catalytic_gain(state.catalysts[1])).sqrt()
        })
        .unwrap_or(1.0);
    trace.legacy_activation = trace.activated_production / gain.max(1.0);
    trace.allocated_activation = trace.activated_production - trace.legacy_activation;
    trace.processing_share =
        trace.allocated_activation / trace.activated_production.max(f64::MIN_POSITIVE);
    trace.reserve_change = mesh.interior.r * mesh.area() - reserve0;
    trace.readiness_progress =
        mesh.total_structural_mass() / (1.35 * mass0.max(f64::MIN_POSITIVE));
    trace.final_structural_material = mesh.total_structural_mass();
    trace.final_membrane_material = mesh.total_bound_membrane();
    trace.unused_internal_resource =
        (mesh.interior.n.max(0.0) + mesh.interior.f.max(0.0)) * mesh.area();
    trace.expression_net_benefit = trace.allocated_activation
        - trace.processing_synthesis
        - trace.expression_maintenance
        - trace.processing_turnover;
    trace
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BSpecificity {
    pub paired_interactions: Vec<f64>,
    pub mean: f64,
    pub median: f64,
    pub ci95: [f64; 2],
    pub positive_pairs: usize,
    pub leave_one_out_all_positive: bool,
    pub classification: String,
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len().max(1) as f64
}

pub fn b_specificity(b: &[f64], neutral: &[f64]) -> BSpecificity {
    let interactions: Vec<f64> = b.iter().zip(neutral).map(|(bv, nv)| bv - nv).collect();
    let center = mean(&interactions);
    let mut sorted = interactions.clone();
    sorted.sort_by(f64::total_cmp);
    let median = if sorted.is_empty() {
        0.0
    } else {
        (sorted[(sorted.len() - 1) / 2] + sorted[sorted.len() / 2]) * 0.5
    };
    let variance = if interactions.len() > 1 {
        interactions
            .iter()
            .map(|value| (value - center).powi(2))
            .sum::<f64>()
            / (interactions.len() - 1) as f64
    } else {
        0.0
    };
    let half_width = 2.365 * (variance / interactions.len().max(1) as f64).sqrt();
    let loo_positive = (0..interactions.len()).all(|omit| {
        mean(
            &interactions
                .iter()
                .enumerate()
                .filter_map(|(index, value)| (index != omit).then_some(*value))
                .collect::<Vec<_>>(),
        ) > 0.0
    });
    let positive_pairs = interactions.iter().filter(|value| **value > 0.0).count();
    let classification = if !interactions.is_empty()
        && positive_pairs == interactions.len()
        && center - half_width > 0.0
        && loo_positive
    {
        "B_REPAIR_SPECIFICITY_PRESENT"
    } else {
        "B_REPAIR_SPECIFICITY_INCONCLUSIVE"
    };
    BSpecificity {
        paired_interactions: interactions,
        mean: center,
        median,
        ci95: [center - half_width, center + half_width],
        positive_pairs,
        leave_one_out_all_positive: loo_positive,
        classification: classification.into(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decomposition {
    pub h_pairs: Vec<PairReconstruction>,
    pub neutral_pairs: Vec<PairReconstruction>,
    pub h_minus_neutral: PathDifference,
    pub processing_share_mean_h: f64,
    pub legacy_share_mean_h: f64,
    pub pulse_expression_overlap_fraction: f64,
    pub resource_delivery_limited: bool,
    pub first_broken_link: String,
    pub primary_classification: String,
    pub selected_repair_route: String,
    pub mutation_run: bool,
    pub heredity_run: bool,
    pub selection_run: bool,
    pub adaptation_run: bool,
    pub reversal_run: bool,
}

fn mean_difference(pairs: &[PairReconstruction]) -> PathDifference {
    let field = |f: fn(&PathDifference) -> f64| {
        mean(&pairs.iter().map(|pair| f(&pair.difference)).collect::<Vec<_>>())
    };
    PathDifference {
        processing_allocation: field(|d| d.processing_allocation),
        repair_allocation: field(|d| d.repair_allocation),
        processing_expression: field(|d| d.processing_expression),
        resource_encounter: field(|d| d.resource_encounter),
        resource_conversion: field(|d| d.resource_conversion),
        activated_production: field(|d| d.activated_production),
        reserve_inflow: field(|d| d.reserve_inflow),
        reserve_change: field(|d| d.reserve_change),
        growth: field(|d| d.growth),
        readiness: field(|d| d.readiness),
    }
}

pub fn decompose_eight_pairs(steps: usize) -> Decomposition {
    let h_pairs: Vec<_> = (1..=8)
        .map(|seed| reconstruct_pair(seed, AssayEnvironment::H, steps))
        .collect();
    let neutral_pairs: Vec<_> = (1..=8)
        .map(|seed| reconstruct_pair(seed, AssayEnvironment::Neutral, steps))
        .collect();
    let h = mean_difference(&h_pairs);
    let neutral = mean_difference(&neutral_pairs);
    let subtract = |a: f64, b: f64| a - b;
    let processing_share_mean_h =
        mean(&h_pairs.iter().map(|p| p.processing.processing_share).collect::<Vec<_>>());
    let overlap = mean(
        &h_pairs
            .iter()
            .map(|p| {
                p.processing.catalyst_resource_overlap_steps as f64
                    / p.processing.accepted_steps.max(1) as f64
            })
            .collect::<Vec<_>>(),
    );
    let resource_delivery_limited = h_pairs.iter().any(|pair| {
        pair.processing.internal_resource_exposure <= 0.0
            || pair.processing.unused_internal_resource <= 0.0
    });
    Decomposition {
        h_minus_neutral: PathDifference {
            processing_allocation: subtract(h.processing_allocation, neutral.processing_allocation),
            repair_allocation: subtract(h.repair_allocation, neutral.repair_allocation),
            processing_expression: subtract(h.processing_expression, neutral.processing_expression),
            resource_encounter: subtract(h.resource_encounter, neutral.resource_encounter),
            resource_conversion: subtract(h.resource_conversion, neutral.resource_conversion),
            activated_production: subtract(h.activated_production, neutral.activated_production),
            reserve_inflow: subtract(h.reserve_inflow, neutral.reserve_inflow),
            reserve_change: subtract(h.reserve_change, neutral.reserve_change),
            growth: subtract(h.growth, neutral.growth),
            readiness: subtract(h.readiness, neutral.readiness),
        },
        processing_share_mean_h,
        legacy_share_mean_h: 1.0 - processing_share_mean_h,
        pulse_expression_overlap_fraction: overlap,
        resource_delivery_limited,
        first_broken_link:
            "activated-resource production -> reserve accumulation (D-096 reserve schema rejected)"
                .into(),
        primary_classification: D097_PROCESSING_IMPLEMENTATION_DEFECT_CONFIRMED.into(),
        selected_repair_route: D098_REPAIR_ROUTE.into(),
        h_pairs,
        neutral_pairs,
        mutation_run: false,
        heredity_run: false,
        selection_run: false,
        adaptation_run: false,
        reversal_run: false,
    }
}

pub fn reconstruct_pair(
    seed: u64,
    environment: AssayEnvironment,
    steps: usize,
) -> PairReconstruction {
    let processing_genotype = AllocationGenotype([0.55, 0.25, 0.05, 0.15]);
    let repair_genotype = AllocationGenotype([0.10, 0.20, 0.55, 0.15]);
    let processing = trace_path(processing_genotype, environment, seed, steps);
    let repair = trace_path(repair_genotype, environment, seed, steps);
    let difference = PathDifference {
        processing_allocation: processing.allocation[0] - repair.allocation[0],
        repair_allocation: processing.allocation[2] - repair.allocation[2],
        processing_expression: processing.processing_expression - repair.processing_expression,
        resource_encounter: processing.boundary_resource_exposure
            - repair.boundary_resource_exposure,
        resource_conversion: processing.nutrient_converted - repair.nutrient_converted,
        activated_production: processing.activated_production - repair.activated_production,
        reserve_inflow: processing.reserve_inflow - repair.reserve_inflow,
        reserve_change: processing.reserve_change - repair.reserve_change,
        growth: processing.growth - repair.growth,
        readiness: processing.readiness_progress - repair.readiness_progress,
    };
    PairReconstruction {
        seed,
        environment,
        processing,
        repair,
        difference,
    }
}

pub fn classify_first_break(pair: &PairReconstruction) -> &'static str {
    if pair.difference.activated_production > 0.0
        && pair.difference.reserve_inflow == 0.0
        && !pair.processing.reserve_schema_compatible
    {
        D097_PROCESSING_IMPLEMENTATION_DEFECT_CONFIRMED
    } else {
        "D097_PROCESSING_CAUSAL_FAILURE_UNRESOLVED"
    }
}
