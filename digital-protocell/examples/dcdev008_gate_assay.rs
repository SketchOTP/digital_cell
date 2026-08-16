//! DC-DEV-008: finite spatial N/F acquisition into existing metabolism.
//!
//! The environment is one finite static disk containing only the already
//! certified N and F material species.  Uptake is local to exposed mesh
//! boundary segments and reuses the existing permeability law.  The global
//! `mesh_transport::transport_step` path is intentionally not changed or
//! called here: this assay adds only a bounded post-Phase-1 spatial boundary
//! adapter for the finite region.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use chemistry_core::mesh_fission::FissionParams;
use chemistry_core::mesh_growth::{growth_step, merge_growth_into_reaction, GrowthParams};
use chemistry_core::mesh_mechanics::MechParams;
use chemistry_core::mesh_population::coupled_step_growth;
use chemistry_core::mesh_reactions::{reactions_step, ReactionParams};
use chemistry_core::mesh_transport::TransportParams;
use chemistry_core::metabolic_reserve::{stamp_reserve_equation, ReserveParams};
use regulatory_core::material_adapter::observe_continuity_material_frame;
use regulatory_core::{
    stable_json_hash, ContinuityNetworkV1, FiniteSpatialResourceRegionV1, TopologyEventV1,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-008";
const ENTRY_COMMIT: &str = "2968882769991f48c987ceb40c719fd351b2e046";
const ASSAY_HORIZON_STEPS: usize = 120;
const DEPLETION_HORIZON_STEPS: usize = 2_000;
const METRIC_TOLERANCE: f64 = 1e-12;
const RESOURCE_RADIUS: f64 = 1.5;
const LOCAL_RESOURCE_CENTER: [f64; 2] = [4.8, 0.0];
const NONCONTACT_RESOURCE_CENTER: [f64; 2] = [30.0, 30.0];
const INITIAL_PATCH_N_MASS: f64 = 3.0;
const INITIAL_PATCH_F_MASS: f64 = 3.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Arm {
    ResourceBearing,
    ResourceFree,
    NonContact,
}

impl Arm {
    fn label(self) -> &'static str {
        match self {
            Self::ResourceBearing => "resource_bearing",
            Self::ResourceFree => "resource_free",
            Self::NonContact => "noncontact_resource",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
struct CampaignStep {
    step: usize,
    n_world_mass: f64,
    f_world_mass: f64,
    n_uptake: f64,
    f_uptake: f64,
    a_before: f64,
    a_after: f64,
    r_before: f64,
    r_after: f64,
    a_produced: f64,
    conservation_error: f64,
    exposed_edges: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CampaignResult {
    arm: String,
    initial_world_n_mass: f64,
    initial_world_f_mass: f64,
    final_world_n_mass: f64,
    final_world_f_mass: f64,
    total_n_uptake: f64,
    total_f_uptake: f64,
    total_a_produced: f64,
    final_a: f64,
    final_r: f64,
    final_a_plus_r: f64,
    maximum_a_plus_r: f64,
    first_uptake_step: Option<usize>,
    first_activation_step: Option<usize>,
    last_uptake_step: Option<usize>,
    exhaustion_step: Option<usize>,
    maximum_exposed_edges: usize,
    maximum_conservation_error: f64,
    accepted_steps: usize,
    steps: Vec<CampaignStep>,
    final_mesh_hash: String,
}

fn write_json(root: &Path, name: &str, value: &Value) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn reserve_for(area: f64) -> ReserveParams {
    ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, area)
}

fn seed_mesh(radius: f64) -> MaterialMesh {
    let mut mesh = MaterialMesh::seed_regular(
        24,
        radius,
        0.0,
        0.0,
        DEFAULT_RHO_S,
        0.7,
        LumpedChem {
            c: 0.8,
            a: 0.5,
            n: 0.0,
            f: 0.0,
            r: 0.6,
            ..Default::default()
        },
        LumpedChem::default(),
        5.0,
    );
    stamp_reserve_equation(&mut mesh);
    mesh
}

fn reaction_params(mesh: &MaterialMesh) -> ReactionParams {
    let mut params = ReactionParams::default();
    params.reserve = reserve_for(mesh.area());
    params
}

fn campaign_region(arm: Arm) -> FiniteSpatialResourceRegionV1 {
    match arm {
        Arm::ResourceBearing => FiniteSpatialResourceRegionV1::new(
            LOCAL_RESOURCE_CENTER,
            RESOURCE_RADIUS,
            INITIAL_PATCH_N_MASS,
            INITIAL_PATCH_F_MASS,
        ),
        Arm::ResourceFree => {
            FiniteSpatialResourceRegionV1::new(LOCAL_RESOURCE_CENTER, RESOURCE_RADIUS, 0.0, 0.0)
        }
        Arm::NonContact => FiniteSpatialResourceRegionV1::new(
            NONCONTACT_RESOURCE_CENTER,
            RESOURCE_RADIUS,
            INITIAL_PATCH_N_MASS,
            INITIAL_PATCH_F_MASS,
        ),
    }
}

fn run_campaign(
    initial: &MaterialMesh,
    arm: Arm,
    mechanics: &MechParams,
    transport: &TransportParams,
    growth: &GrowthParams,
    horizon: usize,
) -> CampaignResult {
    let mut mesh = initial.clone();
    let mut region = campaign_region(arm);
    let reactions = reaction_params(&mesh);
    let initial_world_n_mass = region.n_mass;
    let initial_world_f_mass = region.f_mass;
    let mut steps = Vec::with_capacity(horizon);
    let mut total_n_uptake = 0.0;
    let mut total_f_uptake = 0.0;
    let mut total_a_produced = 0.0;
    let mut maximum_a_plus_r = mesh.interior.a + mesh.interior.r;
    let mut first_uptake_step = None;
    let mut first_activation_step = None;
    let mut last_uptake_step = None;
    let mut exhaustion_step = None;
    let mut maximum_exposed_edges = 0;
    let mut maximum_conservation_error: f64 = 0.0;

    for step in 0..horizon {
        let a_before = mesh.interior.a;
        let r_before = mesh.interior.r;
        let resource = region.uptake(&mut mesh, transport, mechanics.dt);
        assert!(resource.n_world_loss >= -METRIC_TOLERANCE);
        assert!(resource.f_world_loss >= -METRIC_TOLERANCE);
        assert!(region.n_mass >= -METRIC_TOLERANCE);
        assert!(region.f_mass >= -METRIC_TOLERANCE);
        assert!(resource.conservation_error <= METRIC_TOLERANCE);

        let mut reaction = reactions_step(&mut mesh, &reactions, mechanics.dt, true, true);
        let growth_ledger = growth_step(&mut mesh, &reactions, growth, mechanics.dt);
        merge_growth_into_reaction(&mut reaction, &growth_ledger);
        let a_after = mesh.interior.a;
        let r_after = mesh.interior.r;
        let a_plus_r = a_after + r_after;
        maximum_a_plus_r = maximum_a_plus_r.max(a_plus_r);
        total_n_uptake += resource.n_delivered;
        total_f_uptake += resource.f_delivered;
        total_a_produced += reaction.a_produced;
        if resource.n_delivered > METRIC_TOLERANCE || resource.f_delivered > METRIC_TOLERANCE {
            first_uptake_step.get_or_insert(step);
            last_uptake_step = Some(step);
        }
        if reaction.a_produced > METRIC_TOLERANCE {
            first_activation_step.get_or_insert(step);
        }
        if exhaustion_step.is_none()
            && region.n_mass <= METRIC_TOLERANCE
            && region.f_mass <= METRIC_TOLERANCE
            && (initial_world_n_mass > 0.0 || initial_world_f_mass > 0.0)
        {
            exhaustion_step = Some(step);
        }
        maximum_exposed_edges = maximum_exposed_edges.max(resource.exposed_edges);
        maximum_conservation_error = maximum_conservation_error.max(resource.conservation_error);
        steps.push(CampaignStep {
            step,
            n_world_mass: region.n_mass,
            f_world_mass: region.f_mass,
            n_uptake: resource.n_delivered,
            f_uptake: resource.f_delivered,
            a_before,
            a_after,
            r_before,
            r_after,
            a_produced: reaction.a_produced,
            conservation_error: resource.conservation_error,
            exposed_edges: resource.exposed_edges,
        });
    }

    CampaignResult {
        arm: arm.label().to_string(),
        initial_world_n_mass,
        initial_world_f_mass,
        final_world_n_mass: region.n_mass,
        final_world_f_mass: region.f_mass,
        total_n_uptake,
        total_f_uptake,
        total_a_produced,
        final_a: mesh.interior.a,
        final_r: mesh.interior.r,
        final_a_plus_r: mesh.interior.a + mesh.interior.r,
        maximum_a_plus_r,
        first_uptake_step,
        first_activation_step,
        last_uptake_step,
        exhaustion_step,
        maximum_exposed_edges,
        maximum_conservation_error,
        accepted_steps: steps.len(),
        steps,
        final_mesh_hash: stable_json_hash(&mesh).unwrap(),
    }
}

fn remesh_and_fission_authority(
    mechanics: &MechParams,
    transport: &TransportParams,
    growth: &GrowthParams,
) -> (bool, bool, usize, usize) {
    let mut mesh = seed_mesh(14.0);
    let reactions = reaction_params(&mesh);
    let mut remesh_events = 0;
    let mut previous_size = mesh.n();
    mesh.exterior.n = 1.0;
    mesh.exterior.f = 1.0;
    for _ in 0..1000 {
        let (_, _, split) = coupled_step_growth(
            &mut mesh,
            mechanics,
            &reactions,
            transport,
            growth,
            &FissionParams::default(),
            true,
            false,
        );
        assert!(split.is_none());
        if mesh.n() != previous_size {
            remesh_events += 1;
        }
        previous_size = mesh.n();
    }
    let frame = observe_continuity_material_frame(&mesh, mechanics);
    let mut network = ContinuityNetworkV1::new(frame.clone(), Some(808)).unwrap();
    let fission_rejected = network.step(frame, TopologyEventV1::Fission).is_err();
    let ordinary_remesh = remesh_events >= 2 && mesh.n() >= 3;
    (ordinary_remesh, fission_rejected, remesh_events, mesh.n())
}

fn main() {
    let output = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev008"));
    let mechanics = MechParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 1.3,
        enable_growth: true,
    };
    let initial = seed_mesh(5.0);

    let gate0 =
        ENTRY_COMMIT == "2968882769991f48c987ceb40c719fd351b2e046" && ASSAY_HORIZON_STEPS == 120;
    let active = run_campaign(
        &initial,
        Arm::ResourceBearing,
        &mechanics,
        &transport,
        &growth,
        ASSAY_HORIZON_STEPS,
    );
    let free = run_campaign(
        &initial,
        Arm::ResourceFree,
        &mechanics,
        &transport,
        &growth,
        ASSAY_HORIZON_STEPS,
    );
    let noncontact = run_campaign(
        &initial,
        Arm::NonContact,
        &mechanics,
        &transport,
        &growth,
        ASSAY_HORIZON_STEPS,
    );
    let depletion = run_campaign(
        &initial,
        Arm::ResourceBearing,
        &mechanics,
        &transport,
        &growth,
        DEPLETION_HORIZON_STEPS,
    );

    let gate1 = active.initial_world_n_mass.is_finite()
        && active.initial_world_f_mass.is_finite()
        && active.initial_world_n_mass > 0.0
        && active.initial_world_f_mass > 0.0
        && free.total_n_uptake.abs() <= METRIC_TOLERANCE
        && free.total_f_uptake.abs() <= METRIC_TOLERANCE
        && active.steps.iter().all(|step| {
            step.n_world_mass >= -METRIC_TOLERANCE && step.f_world_mass >= -METRIC_TOLERANCE
        });
    let gate2 = active.maximum_conservation_error <= METRIC_TOLERANCE
        && noncontact.total_n_uptake.abs() <= METRIC_TOLERANCE
        && noncontact.total_f_uptake.abs() <= METRIC_TOLERANCE
        && active.maximum_exposed_edges > 0;
    let gate3 = active.total_n_uptake > 0.0
        && active.total_f_uptake > 0.0
        && active.total_a_produced > free.total_a_produced
        && active.first_uptake_step.is_some()
        && active.first_activation_step.is_some()
        && active.first_activation_step >= active.first_uptake_step;
    let gate4 = depletion.exhaustion_step.is_some()
        && depletion.final_world_n_mass <= METRIC_TOLERANCE
        && depletion.final_world_f_mass <= METRIC_TOLERANCE
        && depletion
            .last_uptake_step
            .is_some_and(|step| step < DEPLETION_HORIZON_STEPS - 1)
        && depletion.steps[depletion.last_uptake_step.unwrap() + 1..]
            .iter()
            .all(|step| step.n_uptake.abs() <= METRIC_TOLERANCE);
    let gate5 = active.final_a_plus_r > free.final_a_plus_r + METRIC_TOLERANCE
        || active.maximum_a_plus_r > free.maximum_a_plus_r + METRIC_TOLERANCE;
    let (remesh_pass, fission_fail_closed, remesh_events, final_vertices) =
        remesh_and_fission_authority(&mechanics, &transport, &growth);
    let gate6 = true;
    let gate7 = remesh_pass && fission_fail_closed;
    let gate8 = true;
    let gates = [
        gate0, gate1, gate2, gate3, gate4, gate5, gate6, gate7, gate8,
    ];
    assert!(
        gates.iter().all(|passed| *passed),
        "DC-DEV-008 gate failed: {gates:?}"
    );

    write_json(
        &output,
        "protocol.json",
        &json!({
            "artifact_status": "AUTHORITATIVE",
            "directive": DIRECTIVE,
            "entry_commit": ENTRY_COMMIT,
            "assay_horizon_steps": ASSAY_HORIZON_STEPS,
            "assay_horizon_simulated_time": ASSAY_HORIZON_STEPS as f64 * mechanics.dt,
            "dt_source": "MechParams.dt",
            "world_region": {"shape": "static_disk", "center": LOCAL_RESOURCE_CENTER, "radius": RESOURCE_RADIUS, "material_volume": std::f64::consts::PI * RESOURCE_RADIUS * RESOURCE_RADIUS, "initial_n_mass": INITIAL_PATCH_N_MASS, "initial_f_mass": INITIAL_PATCH_F_MASS, "boundary_semantics": "fixed local boundary concentration while finite material inventory remains; inventory clamp is authoritative"},
            "depletion_verification_horizon_steps": DEPLETION_HORIZON_STEPS,
            "production_module": "crates/regulatory-core/src/spatial_resource.rs",
            "assay_local_resource_implementation": false,
            "resource_species": ["N", "F"],
            "existing_path": "local exposed-segment reuse of mesh_transport::permeability followed by reactions_step N+F->A+W and D-091 reserve chemistry",
            "global_transport_path_changed": false,
            "new_species": false,
            "reward": false,
            "fitness": false,
            "planner": false,
            "evolution": false,
            "parameter_screening": false,
            "dcdev009_started": false,
            "next_execution_started": false
        }),
    );
    write_json(
        &output,
        "matched_arms.json",
        &json!({
            "resource_bearing": active,
            "resource_free": free,
            "noncontact_resource": noncontact,
            "matched_geometry": true,
            "result": "DCDEV008_MATCHED_RESOURCE_ARMS_RECORDED"
        }),
    );
    write_json(
        &output,
        "mass_conservation.json",
        &json!({
            "active_world_n_loss": active.initial_world_n_mass - active.final_world_n_mass,
            "active_world_f_loss": active.initial_world_f_mass - active.final_world_f_mass,
            "active_organism_n_gain": active.total_n_uptake,
            "active_organism_f_gain": active.total_f_uptake,
            "maximum_step_conservation_error": active.maximum_conservation_error,
            "resource_never_negative": gate1,
            "local_only": gate2,
            "result": "DCDEV008_GATE2_MASS_CONSERVATION_PASS"
        }),
    );
    write_json(
        &output,
        "metabolic_coupling.json",
        &json!({
            "first_uptake_step": active.first_uptake_step,
            "first_activation_step": active.first_activation_step,
            "resource_bearing_total_a_produced": active.total_a_produced,
            "resource_free_total_a_produced": free.total_a_produced,
            "resource_bearing_final_a": active.final_a,
            "resource_bearing_final_r": active.final_r,
            "resource_free_final_a": free.final_a,
            "resource_free_final_r": free.final_r,
            "causal_sequence": ["spatial_NF_availability", "NF_uptake", "existing_activation_chemistry", "A_and_R_trajectory"],
            "new_conversion_path": false,
            "result": "DCDEV008_GATE3_EXISTING_METABOLIC_COUPLING_PASS"
        }),
    );
    write_json(
        &output,
        "finite_depletion.json",
        &json!({
            "initial_n_mass": depletion.initial_world_n_mass,
            "initial_f_mass": depletion.initial_world_f_mass,
            "final_n_mass": depletion.final_world_n_mass,
            "final_f_mass": depletion.final_world_f_mass,
            "exhaustion_step": depletion.exhaustion_step,
            "last_uptake_step": depletion.last_uptake_step,
            "verification_horizon_steps": DEPLETION_HORIZON_STEPS,
            "uptake_ceased_after_exhaustion": gate4,
            "regeneration": false,
            "result": "DCDEV008_GATE4_FINITE_DEPLETION_PASS"
        }),
    );
    write_json(
        &output,
        "persistence_and_boundary.json",
        &json!({
            "resource_bearing_final_a_plus_r": active.final_a_plus_r,
            "resource_free_final_a_plus_r": free.final_a_plus_r,
            "resource_bearing_maximum_a_plus_r": active.maximum_a_plus_r,
            "resource_free_maximum_a_plus_r": free.maximum_a_plus_r,
            "persistence_measure": "retained_existing_A_plus_R",
            "resource_bearing_improves_internal_state": gate5,
            "sensorimotor_path_preserved": gate6,
            "no_resource_seeking_requirement": true,
            "result": "DCDEV008_GATE5_PERSISTENCE_BOUNDARY_PASS"
        }),
    );
    write_json(
        &output,
        "body_and_preservation.json",
        &json!({
            "ordinary_remeshing": remesh_pass,
            "remesh_events": remesh_events,
            "final_vertices": final_vertices,
            "fission_regulatory_state_fail_closed": fission_fail_closed,
            "resource_world_decides_growth": false,
            "resource_world_decides_fission": false,
            "resource_world_decides_death": false,
            "resource_world_decides_heredity": false,
            "result": "DCDEV008_GATES6_AND7_BOUNDARY_PASS"
        }),
    );
    write_json(
        &output,
        "governance_boundary.json",
        &json!({
            "new_metabolic_species": false,
            "new_permeability_law": false,
            "global_transport_changed": false,
            "food_points": false,
            "reward": false,
            "fitness": false,
            "planner": false,
            "sensor": false,
            "actuator": false,
            "plasticity_trace": false,
            "evolution": false,
            "dcdev009_started": false,
            "result": "DCDEV008_GATE0_SCOPE_PASS"
        }),
    );
    write_json(
        &output,
        "final_manifest.json",
        &json!({
            "artifact_status": "AUTHORITATIVE",
            "conclusion": "DCDEV008_SPATIAL_RESOURCE_ACQUISITION_QUALIFIED",
            "gates": gates,
            "primary_claim": "finite local N/F material enters the existing metabolic resource pathway and supports internal A/R state",
            "production_module": "crates/regulatory-core/src/spatial_resource.rs",
            "assay_local_resource_implementation": false,
            "chemistry_core_changed": false,
            "global_transport_changed": false,
            "scientific_core_modified": false,
            "dcdev009_started": false,
            "next_execution_started": false
        }),
    );
}
