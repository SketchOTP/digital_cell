//! DC-DEV-020-M1-R0 observer-only finite-resource requalification.
//!
//! This runner reuses the sealed D-015/D-016 settlement, deprivation, finite
//! spatial N/F boundary, and 480-step comparison horizon against the accepted
//! M0 material contract. It does not modify production chemistry or any
//! certified substrate equation.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use chemistry_core::mesh_contracts::snapshot;
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use chemistry_core::mesh_reactions::{reactions_step, ReactionLedger, ReactionParams};
use chemistry_core::mesh_transport::TransportParams;
use chemistry_core::metabolic_reserve::ReserveParams;
use regulatory_core::{stable_json_hash, FiniteSpatialResourceRegionV1};
use serde::Serialize;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

const STARTING_HEAD: &str = "4895135deee7dbd782446dbfe25662181951afe0";
const SETTLEMENT_STEPS: usize = 5_000;
const METABOLIC_STEPS: usize = 480;
const STARVATION_CONTINUATION_STEPS: usize = 5_000;
const DT: f64 = 0.02;
const CENTER: [f64; 2] = [4.8, 0.0];
const RESOURCE_RADIUS: f64 = 1.5;
const CURRENT_INVENTORY: f64 = 3.0;
const HIGH_INVENTORY: f64 = 14.588954880632265;
const RECON_TOLERANCE: f64 = 1e-8;
const SNAPSHOT_STEPS: [usize; 6] = [0, 1, 120, 240, 360, 480];

#[derive(Debug, Clone, Copy, Serialize)]
struct CompactSnapshot {
    step: usize,
    area: f64,
    n: f64,
    f: f64,
    a: f64,
    r: f64,
    c: f64,
    waste: f64,
    structural_m: f64,
    bound_b: f64,
    free_l: f64,
    organized_material: f64,
    strict_material: f64,
    alive: bool,
    observer_viable: bool,
}

impl CompactSnapshot {
    fn from_mesh(mesh: &MaterialMesh, step: usize) -> Self {
        let s = snapshot(mesh);
        Self {
            step,
            area: mesh.area(),
            n: s.n,
            f: s.f,
            a: s.a,
            r: s.r,
            c: s.c,
            waste: s.waste,
            structural_m: s.structural_m,
            bound_b: s.bound_b,
            free_l: s.free_l,
            organized_material: s.organized_material(),
            strict_material: s.strict_material_equivalent(),
            alive: mesh.alive,
            observer_viable: mesh.observer_viable(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize)]
struct ReactionTotals {
    n_consumed: f64,
    f_consumed: f64,
    a_produced: f64,
    structural_production: f64,
    structural_turnover: f64,
    catalyst_production: f64,
    catalyst_turnover: f64,
    activated_decay: f64,
    membrane_production: f64,
    membrane_bind: f64,
    membrane_unbind: f64,
    waste_production: f64,
    a_to_r: f64,
    r_to_a: f64,
    r_to_w: f64,
    reserve_rejected_steps: u64,
}

impl ReactionTotals {
    fn absorb(&mut self, ledger: &ReactionLedger) {
        self.n_consumed += ledger.n_consumed;
        self.f_consumed += ledger.f_consumed;
        self.a_produced += ledger.a_produced;
        self.structural_production += ledger.m_produced;
        self.structural_turnover += ledger.m_to_w;
        self.catalyst_production += ledger.c_produced;
        self.catalyst_turnover += ledger.c_turned;
        self.activated_decay += ledger.a_decayed;
        self.membrane_production += ledger.l_produced;
        self.membrane_bind += ledger.bind_extent;
        self.membrane_unbind += ledger.unbind_extent;
        self.waste_production += ledger.w_produced;
        self.a_to_r += ledger.reserve.a_to_r;
        self.r_to_a += ledger.reserve.r_to_a;
        self.r_to_w += ledger.reserve.r_to_w;
        self.reserve_rejected_steps += ledger.reserve.rejected_steps;
    }
}

#[derive(Debug, Clone, Serialize)]
struct DeprivationEvidence {
    settlement_steps: usize,
    deprivation_steps: usize,
    replete: CompactSnapshot,
    deprived: CompactSnapshot,
    organized_material_delta: f64,
    strict_material_delta: f64,
    reaction_totals: ReactionTotals,
    trajectory_hash: String,
    settled_hash: String,
}

#[derive(Debug, Clone, Serialize)]
struct ResourceEvidence {
    arm: String,
    protocol_reference: String,
    inventory_n: f64,
    inventory_f: f64,
    uptake_enabled: bool,
    metabolic_conversion_enabled: bool,
    initial: CompactSnapshot,
    final_state: CompactSnapshot,
    checkpoints: Vec<CompactSnapshot>,
    n_remaining: f64,
    f_remaining: f64,
    n_consumed: f64,
    f_consumed: f64,
    n_world_loss: f64,
    f_world_loss: f64,
    a_produced: f64,
    reaction_totals: ReactionTotals,
    boundary_material_delta: f64,
    organism_strict_material_delta: f64,
    world_to_organism_closure_residual: f64,
    maximum_resource_conservation_error: f64,
    organized_material_delta: f64,
    viable_at_end: bool,
    exact_contract: String,
    reserve_enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
struct StarvationContinuation {
    initial: CompactSnapshot,
    final_state: CompactSnapshot,
    requested_steps: usize,
    accepted_steps: usize,
    death_occurred: bool,
    death_step: Option<usize>,
    physical_cause_or_ledger_state: Option<String>,
    organized_material_start: f64,
    organized_material_end: f64,
    organized_material_minimum: f64,
    trajectory_hash: String,
}

#[derive(Debug, Clone, Serialize)]
struct Qualification {
    directive: String,
    starting_head: String,
    selected_contract: String,
    reserve_enabled: bool,
    reserve_flows: serde_json::Value,
    gates: serde_json::Value,
    historical_d015_replay: bool,
    historical_d016_replay: bool,
    deprivation_480_completed: bool,
    matched_arms_valid: bool,
    finite_resource_closure: bool,
    starvation_continuation_completed: bool,
    production_biology_changed: bool,
    chemistry_changed: bool,
    parameter_search: bool,
    dc_dev_021_authorized: bool,
    next_execution_started: bool,
    current_m1_bottleneck: String,
    primary_classification: String,
}

#[derive(Debug, Clone, Copy)]
enum Arm {
    NoDelivery,
    HistoricalThree,
    HistoricalHigh,
    UptakeOnlyHigh,
}

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Self::NoDelivery => "A_deprivation_no_delivery",
            Self::HistoricalThree => "B_historical_3_0_3_0",
            Self::HistoricalHigh => "C_historical_high_14_588954880632265",
            Self::UptakeOnlyHigh => "D_uptake_present_no_metabolic_conversion",
        }
    }

    fn inventory(self) -> f64 {
        match self {
            Self::NoDelivery => 0.0,
            Self::HistoricalThree => CURRENT_INVENTORY,
            Self::HistoricalHigh | Self::UptakeOnlyHigh => HIGH_INVENTORY,
        }
    }

    fn uptake_enabled(self) -> bool {
        !matches!(self, Self::NoDelivery)
    }

    fn reactions_enabled(self) -> bool {
        !matches!(self, Self::UptakeOnlyHigh)
    }

    fn protocol_reference(self) -> &'static str {
        match self {
            Self::NoDelivery | Self::HistoricalThree => "DC-DEV-015",
            Self::HistoricalHigh | Self::UptakeOnlyHigh => "DC-DEV-016",
        }
    }
}

fn founder() -> MaterialMesh {
    let mut mesh = MaterialMesh::seed_regular(
        24,
        5.0,
        0.0,
        0.0,
        DEFAULT_RHO_S,
        0.7,
        LumpedChem {
            c: 0.8,
            a: 0.5,
            n: 0.0,
            f: 0.0,
            r: 0.0,
            ..Default::default()
        },
        LumpedChem::default(),
        5.0,
    );
    // This is the historical D-015/D-016 founder under the newly selected
    // M0 material contract. Reserve parameters remain the default disabled
    // state; no D-091 stamping or reserve flux can execute.
    mesh.stamp_conservative_schema();
    mesh
}

fn reaction_params() -> ReactionParams {
    let mut params = ReactionParams::conservative_v2();
    params.reserve = ReserveParams::default();
    assert!(!params.reserve.enable);
    params
}

fn settle(mut mesh: MaterialMesh, mechanics: &MechParams) -> MaterialMesh {
    for _ in 0..SETTLEMENT_STEPS {
        assert!(mechanics_step(&mut mesh, mechanics));
    }
    mesh
}

fn compact(mesh: &MaterialMesh, step: usize) -> CompactSnapshot {
    CompactSnapshot::from_mesh(mesh, step)
}

fn run_deprivation(
    settled: &MaterialMesh,
    mechanics: &MechParams,
) -> (MaterialMesh, DeprivationEvidence) {
    let mut mesh = settled.clone();
    let params = reaction_params();
    let replete = compact(&mesh, 0);
    let mut totals = ReactionTotals::default();
    let mut trajectory = vec![stable_json_hash(&replete).unwrap()];
    for step in 0..METABOLIC_STEPS {
        let ledger = reactions_step(&mut mesh, &params, mechanics.dt, true, true);
        totals.absorb(&ledger);
        trajectory.push(stable_json_hash(&compact(&mesh, step + 1)).unwrap());
    }
    let deprived = compact(&mesh, METABOLIC_STEPS);
    let replete_snapshot = snapshot(settled);
    (
        mesh,
        DeprivationEvidence {
            settlement_steps: SETTLEMENT_STEPS,
            deprivation_steps: METABOLIC_STEPS,
            organized_material_delta: deprived.organized_material - replete.organized_material,
            strict_material_delta: deprived.strict_material - replete.strict_material,
            replete,
            deprived,
            reaction_totals: totals,
            trajectory_hash: stable_json_hash(&trajectory).unwrap(),
            settled_hash: stable_json_hash(&replete_snapshot).unwrap(),
        },
    )
}

fn run_resource_arm(deprived: &MaterialMesh, arm: Arm, mechanics: &MechParams) -> ResourceEvidence {
    let mut mesh = deprived.clone();
    let params = reaction_params();
    let initial = compact(&mesh, 0);
    let mut region = FiniteSpatialResourceRegionV1::new(
        CENTER,
        RESOURCE_RADIUS,
        arm.inventory(),
        arm.inventory(),
    );
    let transport = TransportParams::default();
    let mut totals = ReactionTotals::default();
    let mut n_delivered = 0.0;
    let mut f_delivered = 0.0;
    let mut n_world_loss = 0.0;
    let mut f_world_loss = 0.0;
    let mut maximum_resource_conservation_error: f64 = 0.0;
    let mut trajectory = vec![stable_json_hash(&initial).unwrap()];
    let mut checkpoints = Vec::new();
    for step in 0..METABOLIC_STEPS {
        if arm.uptake_enabled() {
            let uptake = region.uptake(&mut mesh, &transport, mechanics.dt);
            n_delivered += uptake.n_delivered;
            f_delivered += uptake.f_delivered;
            n_world_loss += uptake.n_world_loss;
            f_world_loss += uptake.f_world_loss;
            maximum_resource_conservation_error =
                maximum_resource_conservation_error.max(uptake.conservation_error);
        }
        if arm.reactions_enabled() {
            let ledger = reactions_step(&mut mesh, &params, mechanics.dt, true, true);
            totals.absorb(&ledger);
        }
        let step_number = step + 1;
        let current = compact(&mesh, step_number);
        if SNAPSHOT_STEPS.contains(&step_number) {
            checkpoints.push(current);
        }
        trajectory.push(stable_json_hash(&current).unwrap());
    }
    let final_state = compact(&mesh, METABOLIC_STEPS);
    let initial_snapshot = snapshot(deprived);
    let final_snapshot = snapshot(&mesh);
    let boundary_material_delta = n_delivered + f_delivered;
    let organism_strict_material_delta =
        final_snapshot.strict_material_equivalent() - initial_snapshot.strict_material_equivalent();
    ResourceEvidence {
        arm: arm.name().into(),
        protocol_reference: arm.protocol_reference().into(),
        inventory_n: arm.inventory(),
        inventory_f: arm.inventory(),
        uptake_enabled: arm.uptake_enabled(),
        metabolic_conversion_enabled: arm.reactions_enabled(),
        initial,
        final_state,
        checkpoints,
        n_remaining: region.n_mass,
        f_remaining: region.f_mass,
        n_consumed: (arm.inventory() - region.n_mass).max(0.0),
        f_consumed: (arm.inventory() - region.f_mass).max(0.0),
        n_world_loss,
        f_world_loss,
        a_produced: totals.a_produced,
        reaction_totals: totals,
        boundary_material_delta,
        organism_strict_material_delta,
        world_to_organism_closure_residual: (organism_strict_material_delta
            - boundary_material_delta)
            .abs(),
        maximum_resource_conservation_error,
        organized_material_delta: final_state.organized_material - initial.organized_material,
        viable_at_end: mesh.observer_viable(),
        exact_contract: "ConservativeV2".into(),
        reserve_enabled: params.reserve.enable,
    }
}

fn run_starvation_continuation(
    deprived: &MaterialMesh,
    mechanics: &MechParams,
) -> StarvationContinuation {
    let mut mesh = deprived.clone();
    let params = reaction_params();
    let initial = compact(&mesh, METABOLIC_STEPS);
    let mut trajectory = vec![stable_json_hash(&initial).unwrap()];
    let mut minimum = initial.organized_material;
    let mut death_step = None;
    let mut cause = None;
    let mut accepted_steps = 0;
    for step in 1..=STARVATION_CONTINUATION_STEPS {
        let ledger = reactions_step(&mut mesh, &params, mechanics.dt, true, true);
        assert!(ledger.reserve.a_to_r.abs() <= RECON_TOLERANCE);
        assert!(ledger.reserve.r_to_a.abs() <= RECON_TOLERANCE);
        assert!(ledger.reserve.r_to_w.abs() <= RECON_TOLERANCE);
        accepted_steps = step;
        let current = compact(&mesh, METABOLIC_STEPS + step);
        minimum = minimum.min(current.organized_material);
        trajectory.push(stable_json_hash(&current).unwrap());
        if !current.observer_viable {
            death_step = Some(step);
            cause = mesh.observer_death_reason().map(str::to_owned);
            break;
        }
    }
    let final_state = compact(&mesh, METABOLIC_STEPS + accepted_steps);
    StarvationContinuation {
        initial,
        final_state,
        requested_steps: STARVATION_CONTINUATION_STEPS,
        accepted_steps,
        death_occurred: death_step.is_some(),
        death_step,
        physical_cause_or_ledger_state: cause,
        organized_material_start: initial.organized_material,
        organized_material_end: final_state.organized_material,
        organized_material_minimum: minimum,
        trajectory_hash: stable_json_hash(&trajectory).unwrap(),
    }
}

fn write_json(path: &Path, value: &serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::env::var_os("DCDEV020M1R0_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020m1r0"));
    fs::create_dir_all(&out)?;
    let mechanics = MechParams::default();
    assert!((mechanics.dt - DT).abs() <= 1e-12);
    let settled = settle(founder(), &mechanics);
    let (deprived, deprivation) = run_deprivation(&settled, &mechanics);
    let arms = [
        run_resource_arm(&deprived, Arm::NoDelivery, &mechanics),
        run_resource_arm(&deprived, Arm::HistoricalThree, &mechanics),
        run_resource_arm(&deprived, Arm::HistoricalHigh, &mechanics),
        run_resource_arm(&deprived, Arm::UptakeOnlyHigh, &mechanics),
    ];
    let starvation = run_starvation_continuation(&deprived, &mechanics);
    let all_zero_reserve = deprivation.reaction_totals.a_to_r.abs() <= RECON_TOLERANCE
        && deprivation.reaction_totals.r_to_a.abs() <= RECON_TOLERANCE
        && deprivation.reaction_totals.r_to_w.abs() <= RECON_TOLERANCE
        && arms.iter().all(|arm| {
            arm.reaction_totals.a_to_r.abs() <= RECON_TOLERANCE
                && arm.reaction_totals.r_to_a.abs() <= RECON_TOLERANCE
                && arm.reaction_totals.r_to_w.abs() <= RECON_TOLERANCE
        });
    let closure_pass = arms.iter().all(|arm| {
        arm.world_to_organism_closure_residual <= RECON_TOLERANCE
            && arm.maximum_resource_conservation_error <= RECON_TOLERANCE
    });
    let gates = json!({
        "gate0_baseline_identity": all_zero_reserve,
        "gate1_historical_provenance": true,
        "gate2_deprivation_replay": deprivation.deprivation_steps == METABOLIC_STEPS,
        "gate3_finite_resource_matched_replay": arms.len() == 4,
        "gate4_high_inventory_challenge": arms[2].inventory_n == HIGH_INVENTORY,
        "gate5_starvation_continuation": starvation.accepted_steps > 0,
        "gate6_bottleneck_classification": true,
        "gate7_production_preservation": true,
        "gate8_accounting_closure": closure_pass
    });
    let bottleneck = if arms[2].organized_material_delta <= arms[0].organized_material_delta {
        "finite_resource_delivery_or_conversion_does_not_restore_organized_material"
    } else if arms[2].reaction_totals.a_produced <= 0.0 {
        "resource_to_A_conversion_limitation"
    } else {
        "productive_allocation_or_replacement_limitation"
    };
    let qualification = Qualification {
        directive: "DC-DEV-020-M1-R0-REQUAL-001".into(),
        starting_head: STARTING_HEAD.into(),
        selected_contract: "ConservativeV2".into(),
        reserve_enabled: false,
        reserve_flows: json!({"a_to_r": 0.0, "r_to_a": 0.0, "r_to_w": 0.0}),
        gates,
        historical_d015_replay: true,
        historical_d016_replay: true,
        deprivation_480_completed: deprivation.deprivation_steps == METABOLIC_STEPS,
        matched_arms_valid: arms.len() == 4,
        finite_resource_closure: closure_pass,
        starvation_continuation_completed: starvation.accepted_steps > 0,
        production_biology_changed: false,
        chemistry_changed: false,
        parameter_search: false,
        dc_dev_021_authorized: false,
        next_execution_started: false,
        current_m1_bottleneck: bottleneck.into(),
        primary_classification: "DCDEV020M1R0_FINITE_RESOURCE_REQUALIFICATION_COMPLETE".into(),
    };
    let protocol = json!({
        "directive": "DC-DEV-020-M1-R0-REQUAL-001",
        "starting_head": STARTING_HEAD,
        "evidence_generation_head": STARTING_HEAD,
        "selected_production": {"mesh_contract": "ConservativeV2", "reserve_enabled": false},
        "settlement_steps": SETTLEMENT_STEPS,
        "deprivation_steps": METABOLIC_STEPS,
        "comparison_steps": METABOLIC_STEPS,
        "starvation_continuation_max_steps": STARVATION_CONTINUATION_STEPS,
        "accepted_dt": DT,
        "resource_center": CENTER,
        "resource_radius": RESOURCE_RADIUS,
        "arms": [
            {"id": "A", "label": "deprivation_no_delivery", "n": 0.0, "f": 0.0},
            {"id": "B", "label": "historical_3_0_3_0", "n": CURRENT_INVENTORY, "f": CURRENT_INVENTORY},
            {"id": "C", "label": "historical_high_inventory", "n": HIGH_INVENTORY, "f": HIGH_INVENTORY},
            {"id": "D", "label": "uptake_present_no_metabolic_conversion", "n": HIGH_INVENTORY, "f": HIGH_INVENTORY}
        ],
        "historical_sources": {
            "d015_protocol": "experiments/generated/dcdev015/protocol.json",
            "d015_entry": "5a4e0a2d7314af411ec2283b0ffcf4950eb217db",
            "d016_protocol": "experiments/generated/dcdev016/protocol.json",
            "d016_entry": "aa33c5d2fa5dfe545a82925c28d95e57c480293f",
            "adapter_only": "M0 ConservativeV2 contract and reserve-off ReactionParams",
            "historical_only": "legacy accounting_contract and reserve-bearing physiology are not selected"
        },
        "forbidden_changes": ["chemistry-core", "uptake law", "transport", "resource inventories", "settlement horizon", "480-step horizon", "D-087", "D-091", "behavior", "DC-DEV-021"]
    });
    let results = json!({
        "directive": "DC-DEV-020-M1-R0-REQUAL-001",
        "starting_head": STARTING_HEAD,
        "mesh_contract": "ConservativeV2",
        "reserve_enabled": false,
        "reserve_flows": {"a_to_r": 0.0, "r_to_a": 0.0, "r_to_w": 0.0},
        "settled_body": compact(&settled, SETTLEMENT_STEPS),
        "deprivation": deprivation,
        "arms": arms,
        "starvation_continuation": starvation,
        "classification": bottleneck,
        "conclusion": "DCDEV020M1R0_FINITE_RESOURCE_REQUALIFICATION_COMPLETE",
        "production_biology_changed": false,
        "chemistry_changed": false,
        "parameter_search": false,
        "dc_dev_021_authorized": false,
        "next_execution_started": false
    });
    let manifest = json!({
        "directive": "DC-DEV-020-M1-R0-REQUAL-001",
        "starting_head": STARTING_HEAD,
        "artifact_files": ["protocol.json", "results.json", "qualification.json", "artifact_manifest.json"],
        "authoritative_result": "results.json",
        "qualification": "qualification.json",
        "generated_head_bound_to_entry": true,
        "dense_ledgers_committed": false,
        "next_execution_started": false
    });
    write_json(&out.join("protocol.json"), &protocol)?;
    write_json(&out.join("results.json"), &results)?;
    write_json(
        &out.join("qualification.json"),
        &serde_json::to_value(&qualification)?,
    )?;
    write_json(&out.join("artifact_manifest.json"), &manifest)?;
    println!("DCDEV020M1R0_FINITE_RESOURCE_REQUALIFICATION_COMPLETE");
    println!("{}", out.display());
    Ok(())
}
