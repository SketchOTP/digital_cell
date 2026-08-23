//! DC-DEV-020-M1-R2-R1 observer-only physical-failure closure.
//!
//! This continues the two exact M1-R2 chemistry-path starvation states.  It
//! does not add a failure rule: terminality is the existing mesh observer
//! boundary, existing non-starvation observer reasons, or invalid runtime
//! geometry.  Resource restoration is attempted only after that boundary and
//! never resets the failed mesh state.

#[path = "dcdev020m1r1_capacity_decomp.rs"]
mod m1r1;

use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_contracts::snapshot;
use chemistry_core::mesh_mechanics::MechParams;
use chemistry_core::mesh_reactions::{reactions_step, ReactionLedger, ReactionParams};
use chemistry_core::mesh_transport::TransportParams;
use regulatory_core::{stable_json_hash, FiniteSpatialResourceRegionV1};
use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-020-M1-R2-R1-PHYSICAL-FAILURE-CLOSURE-001";
const STARTING_HEAD: &str = "bc65098c3d26777aca2d1da5dab8cc118ecc6e19";
const R2_COMPARISON_STEPS: usize = 480;
const R2_CONTINUATION_STEPS: usize = 20_000;
const EXTENDED_STEPS: usize = 150_000;
const RESTORATION_STEPS: usize = 5_000;
const DT: f64 = 0.02;
const PRODUCTION_K_A_DECAY: f64 = 0.008;
const ORDINARY_K_A_DECAY: f64 = 0.002;
const STARVATION_MULTIPLIER: f64 = 4.0;
const RESOURCE_CENTER: [f64; 2] = [4.8, 0.0];
const RESOURCE_RADIUS: f64 = 1.5;
const RESTORED_N: f64 = 14.588954880632265;
const RESTORED_F: f64 = 14.588954880632265;
const TOL: f64 = 1e-8;

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
    observer_viable: bool,
    ruptured_edges: usize,
    physical_runtime_valid: bool,
    observer_death_reason: Option<&'static str>,
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
            observer_viable: mesh.observer_viable(),
            ruptured_edges: mesh.edges.iter().filter(|edge| edge.ruptured).count(),
            physical_runtime_valid: mesh.physical_runtime_valid(),
            observer_death_reason: mesh.observer_death_reason(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
struct FailureMargins {
    min_edge_m: f64,
    median_edge_m: f64,
    max_edge_m: f64,
    bond_threshold: f64,
    edge_count: usize,
    ruptured_edges: usize,
}

impl FailureMargins {
    fn from_mesh(mesh: &MaterialMesh) -> Self {
        let mut masses: Vec<f64> = mesh.edges.iter().map(|edge| edge.m).collect();
        masses.sort_by(|a, b| a.total_cmp(b));
        let median_edge_m = if masses.is_empty() {
            0.0
        } else if masses.len() % 2 == 0 {
            (masses[masses.len() / 2 - 1] + masses[masses.len() / 2]) / 2.0
        } else {
            masses[masses.len() / 2]
        };
        Self {
            min_edge_m: masses.first().copied().unwrap_or(0.0),
            median_edge_m,
            max_edge_m: masses.last().copied().unwrap_or(0.0),
            bond_threshold: mesh.bond_threshold,
            edge_count: mesh.edges.len(),
            ruptured_edges: mesh.edges.iter().filter(|edge| edge.ruptured).count(),
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
    }
}

#[derive(Debug, Clone, Serialize)]
struct PhaseEvidence {
    start_step: usize,
    end_step: usize,
    requested_steps: usize,
    accepted_steps: usize,
    initial: CompactSnapshot,
    final_state: CompactSnapshot,
    initial_failure_margins: FailureMargins,
    final_failure_margins: FailureMargins,
    margin_checkpoints: Vec<(CompactSnapshot, FailureMargins)>,
    checkpoints: Vec<CompactSnapshot>,
    reaction_totals: ReactionTotals,
    organized_material_delta: f64,
    strict_material_delta: f64,
    strict_material_closure_residual: f64,
    first_starvation_collapse_step: Option<usize>,
    first_edge_rupture_step: Option<usize>,
    first_half_or_more_rupture_step: Option<usize>,
    first_nonstarvation_failure_step: Option<usize>,
    first_nonstarvation_failure_reason: Option<String>,
    first_physical_invalid_step: Option<usize>,
    terminal_failure_step: Option<usize>,
    terminal_failure_type: Option<String>,
    trajectory_hash: String,
}

#[derive(Debug, Clone, Serialize)]
struct RestorationEvidence {
    reached: bool,
    requested_steps: usize,
    accepted_steps: usize,
    initial: Option<CompactSnapshot>,
    final_state: Option<CompactSnapshot>,
    initial_failure_margins: Option<FailureMargins>,
    final_failure_margins: Option<FailureMargins>,
    n_requested: f64,
    f_requested: f64,
    n_remaining: f64,
    f_remaining: f64,
    n_delivered: f64,
    f_delivered: f64,
    world_to_organism_closure_residual: Option<f64>,
    organized_material_change: Option<f64>,
    ruptured_edges_before: Option<usize>,
    ruptured_edges_after: Option<usize>,
    observer_viability_before: Option<bool>,
    observer_viability_after: Option<bool>,
    closed_intact_before: Option<bool>,
    closed_intact_after: Option<bool>,
    nonstarvation_failure_reason_before: Option<String>,
    nonstarvation_failure_reason_after: Option<String>,
    coherent_material_recovered: bool,
    no_resurrection: bool,
}

#[derive(Debug, Clone, Serialize)]
struct ArmEvidence {
    id: String,
    declared_k_a_decay: f64,
    effective_starvation_k_a_decay: f64,
    r2_comparison: PhaseEvidence,
    r2_continuation: PhaseEvidence,
    extended_continuation: PhaseEvidence,
    restoration: RestorationEvidence,
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= TOL
}

fn reaction_params(k_a_decay: f64) -> ReactionParams {
    let mut params = m1r1::reaction_params();
    params.k_a_decay = k_a_decay;
    params
}

fn terminal_failure(mesh: &MaterialMesh) -> Option<&'static str> {
    if !mesh.physical_runtime_valid() {
        return Some("physical_runtime_invalid");
    }
    let edge_count = mesh.edges.len().max(1);
    let ruptured = mesh.edges.iter().filter(|edge| edge.ruptured).count();
    if ruptured * 2 >= edge_count {
        return Some("mesh_rupture");
    }
    match mesh.observer_death_reason() {
        Some("starvation_collapse") | None => None,
        Some(reason) => Some(reason),
    }
}

fn first_nonstarvation_reason(mesh: &MaterialMesh) -> Option<&'static str> {
    match mesh.observer_death_reason() {
        Some("starvation_collapse") | None => None,
        Some(reason) => Some(reason),
    }
}

fn phase_checkpoints(relative_step: usize, extended: bool) -> bool {
    if extended {
        matches!(
            relative_step,
            1 | 1_000 | 10_000 | 50_000 | 100_000 | 150_000
        )
    } else {
        matches!(
            relative_step,
            1 | 120 | 240 | 360 | 480 | 1_000 | 5_000 | 10_000 | 20_000
        )
    }
}

fn run_phase(
    mut mesh: MaterialMesh,
    params: &ReactionParams,
    mechanics: &MechParams,
    start_step: usize,
    requested_steps: usize,
    extended: bool,
) -> (MaterialMesh, PhaseEvidence) {
    let initial = CompactSnapshot::from_mesh(&mesh, start_step);
    let initial_failure_margins = FailureMargins::from_mesh(&mesh);
    let mut current = initial;
    let mut totals = ReactionTotals::default();
    let mut checkpoints = Vec::new();
    let mut margin_checkpoints = Vec::new();
    let mut trajectory = vec![stable_json_hash(&initial).expect("snapshot hash")];
    let mut first_starvation_collapse_step =
        if mesh.observer_death_reason() == Some("starvation_collapse") {
            Some(start_step)
        } else {
            None
        };
    let mut first_edge_rupture_step = if initial.ruptured_edges > 0 {
        Some(start_step)
    } else {
        None
    };
    let mut first_half_or_more_rupture_step =
        if initial.ruptured_edges * 2 >= initial_failure_margins.edge_count.max(1) {
            Some(start_step)
        } else {
            None
        };
    let mut first_nonstarvation_failure_step =
        first_nonstarvation_reason(&mesh).map(|_| start_step);
    let mut first_nonstarvation_failure_reason =
        first_nonstarvation_reason(&mesh).map(str::to_owned);
    let mut first_physical_invalid_step = (!mesh.physical_runtime_valid()).then_some(start_step);
    let mut terminal_failure_step = terminal_failure(&mesh).map(|_| start_step);
    let mut terminal_failure_type = terminal_failure(&mesh).map(str::to_owned);
    let mut accepted_steps = 0;

    for relative_step in 1..=requested_steps {
        if terminal_failure_step.is_some() {
            break;
        }
        let absolute_step = start_step + relative_step;
        assert!(
            mesh.physical_runtime_valid(),
            "phase entered with invalid mesh"
        );
        let ledger = reactions_step(&mut mesh, params, mechanics.dt, true, true);
        totals.absorb(&ledger);
        accepted_steps = relative_step;
        current = CompactSnapshot::from_mesh(&mesh, absolute_step);
        let margins = FailureMargins::from_mesh(&mesh);
        if first_starvation_collapse_step.is_none()
            && mesh.observer_death_reason() == Some("starvation_collapse")
        {
            first_starvation_collapse_step = Some(absolute_step);
        }
        if first_edge_rupture_step.is_none() && current.ruptured_edges > 0 {
            first_edge_rupture_step = Some(absolute_step);
        }
        if first_half_or_more_rupture_step.is_none()
            && current.ruptured_edges * 2 >= margins.edge_count.max(1)
        {
            first_half_or_more_rupture_step = Some(absolute_step);
        }
        if first_nonstarvation_failure_step.is_none() {
            if let Some(reason) = first_nonstarvation_reason(&mesh) {
                first_nonstarvation_failure_step = Some(absolute_step);
                first_nonstarvation_failure_reason = Some(reason.to_owned());
            }
        }
        if first_physical_invalid_step.is_none() && !mesh.physical_runtime_valid() {
            first_physical_invalid_step = Some(absolute_step);
        }
        if terminal_failure_step.is_none() {
            if let Some(reason) = terminal_failure(&mesh) {
                terminal_failure_step = Some(absolute_step);
                terminal_failure_type = Some(reason.to_owned());
            }
        }
        if phase_checkpoints(relative_step, extended) || terminal_failure_step.is_some() {
            checkpoints.push(current);
            margin_checkpoints.push((current, margins));
        }
        trajectory.push(stable_json_hash(&current).expect("trajectory snapshot hash"));
        if !mesh.physical_runtime_valid() {
            break;
        }
    }

    let strict_delta = current.strict_material - initial.strict_material;
    let evidence = PhaseEvidence {
        start_step,
        end_step: start_step + accepted_steps,
        requested_steps,
        accepted_steps,
        initial,
        final_state: current,
        initial_failure_margins,
        final_failure_margins: FailureMargins::from_mesh(&mesh),
        margin_checkpoints,
        checkpoints,
        reaction_totals: totals,
        organized_material_delta: current.organized_material - initial.organized_material,
        strict_material_delta: strict_delta,
        strict_material_closure_residual: strict_delta.abs(),
        first_starvation_collapse_step,
        first_edge_rupture_step,
        first_half_or_more_rupture_step,
        first_nonstarvation_failure_step,
        first_nonstarvation_failure_reason,
        first_physical_invalid_step,
        terminal_failure_step,
        terminal_failure_type,
        trajectory_hash: stable_json_hash(&trajectory).expect("trajectory hash"),
    };
    (mesh, evidence)
}

fn empty_restoration() -> RestorationEvidence {
    RestorationEvidence {
        reached: false,
        requested_steps: RESTORATION_STEPS,
        accepted_steps: 0,
        initial: None,
        final_state: None,
        initial_failure_margins: None,
        final_failure_margins: None,
        n_requested: RESTORED_N,
        f_requested: RESTORED_F,
        n_remaining: 0.0,
        f_remaining: 0.0,
        n_delivered: 0.0,
        f_delivered: 0.0,
        world_to_organism_closure_residual: None,
        organized_material_change: None,
        ruptured_edges_before: None,
        ruptured_edges_after: None,
        observer_viability_before: None,
        observer_viability_after: None,
        closed_intact_before: None,
        closed_intact_after: None,
        nonstarvation_failure_reason_before: None,
        nonstarvation_failure_reason_after: None,
        coherent_material_recovered: false,
        no_resurrection: false,
    }
}

fn run_restoration(
    mesh_at_failure: Option<&MaterialMesh>,
    params: &ReactionParams,
    mechanics: &MechParams,
    failure_step: Option<usize>,
) -> RestorationEvidence {
    let Some(mesh_at_failure) = mesh_at_failure else {
        return empty_restoration();
    };
    let mut mesh = mesh_at_failure.clone();
    let initial = CompactSnapshot::from_mesh(&mesh, failure_step.unwrap_or(0));
    let initial_margins = FailureMargins::from_mesh(&mesh);
    let initial_ruptured = initial.ruptured_edges;
    let initial_viable = initial.observer_viable;
    let initial_closed = mesh.closed_intact();
    let initial_reason = first_nonstarvation_reason(&mesh).map(str::to_owned);
    let mut current = initial;
    let mut region = FiniteSpatialResourceRegionV1::new(
        RESOURCE_CENTER,
        RESOURCE_RADIUS,
        RESTORED_N,
        RESTORED_F,
    );
    let transport = TransportParams::default();
    let mut n_delivered = 0.0;
    let mut f_delivered = 0.0;
    let mut accepted_steps = 0;
    for offset in 1..=RESTORATION_STEPS {
        if !mesh.physical_runtime_valid() {
            break;
        }
        let uptake = region.uptake(&mut mesh, &transport, mechanics.dt);
        assert!(uptake.conservation_error <= TOL);
        n_delivered += uptake.n_delivered;
        f_delivered += uptake.f_delivered;
        reactions_step(&mut mesh, params, mechanics.dt, true, true);
        current = CompactSnapshot::from_mesh(&mesh, failure_step.unwrap_or(0) + offset);
        accepted_steps = offset;
    }
    let strict_delta = current.strict_material - initial.strict_material;
    let residual = (strict_delta - n_delivered - f_delivered).abs();
    let final_reason = first_nonstarvation_reason(&mesh).map(str::to_owned);
    let organized_material_change = current.organized_material - initial.organized_material;
    let coherent_material_recovered =
        current.organized_material >= initial.organized_material - TOL;
    let no_resurrection = !coherent_material_recovered || !mesh.closed_intact();
    RestorationEvidence {
        reached: true,
        requested_steps: RESTORATION_STEPS,
        accepted_steps,
        initial: Some(initial),
        final_state: Some(current),
        initial_failure_margins: Some(initial_margins),
        final_failure_margins: Some(FailureMargins::from_mesh(&mesh)),
        n_requested: RESTORED_N,
        f_requested: RESTORED_F,
        n_remaining: region.n_mass,
        f_remaining: region.f_mass,
        n_delivered,
        f_delivered,
        world_to_organism_closure_residual: Some(residual),
        organized_material_change: Some(organized_material_change),
        ruptured_edges_before: Some(initial_ruptured),
        ruptured_edges_after: Some(current.ruptured_edges),
        observer_viability_before: Some(initial_viable),
        observer_viability_after: Some(current.observer_viable),
        closed_intact_before: Some(initial_closed),
        closed_intact_after: Some(mesh.closed_intact()),
        nonstarvation_failure_reason_before: initial_reason,
        nonstarvation_failure_reason_after: final_reason,
        coherent_material_recovered,
        no_resurrection,
    }
}

fn assert_r2_endpoint(id: &str, comparison: &PhaseEvidence, continuation: &PhaseEvidence) {
    let expected = match id {
        "A_PRODUCTION_STARVATION_4X" => (
            1.118750007707361e-9,
            1.0394156425111607,
            5.441679269932585,
            36.82914217972928,
        ),
        "B_ORDINARY_DECAY_STARVATION" => (
            1.8843399128823503e-5,
            1.1281509822624136,
            5.534912482156821,
            38.058632513985074,
        ),
        _ => panic!("unknown R2 arm {id}"),
    };
    // Trajectory hashes remain provenance fields, but are derived from
    // floating-point replay and are not cross-platform identity fields.
    assert!(!comparison.trajectory_hash.is_empty());
    assert!(!continuation.trajectory_hash.is_empty());
    assert_eq!(continuation.accepted_steps, R2_CONTINUATION_STEPS);
    assert!(close(continuation.final_state.a, expected.0));
    assert!(close(continuation.final_state.c, expected.1));
    assert!(close(continuation.final_state.structural_m, expected.2));
    assert!(close(continuation.final_state.organized_material, expected.3));
    assert_eq!(continuation.final_state.ruptured_edges, 0);
    assert_eq!(
        continuation.final_state.observer_death_reason,
        Some("starvation_collapse")
    );
    assert!(continuation.final_state.physical_runtime_valid);
}

fn run_arm(
    id: &str,
    declared_k_a_decay: f64,
    initial: &MaterialMesh,
    mechanics: &MechParams,
) -> ArmEvidence {
    let params = reaction_params(declared_k_a_decay);
    let (after_480, r2_comparison) = run_phase(
        initial.clone(),
        &params,
        mechanics,
        0,
        R2_COMPARISON_STEPS,
        false,
    );
    let (after_20480, r2_continuation) = run_phase(
        after_480,
        &params,
        mechanics,
        R2_COMPARISON_STEPS,
        R2_CONTINUATION_STEPS,
        false,
    );
    assert_r2_endpoint(id, &r2_comparison, &r2_continuation);
    let (after_extended, extended_continuation) = run_phase(
        after_20480,
        &params,
        mechanics,
        R2_COMPARISON_STEPS + R2_CONTINUATION_STEPS,
        EXTENDED_STEPS,
        true,
    );
    let restoration = run_restoration(
        if extended_continuation.terminal_failure_step.is_some() {
            Some(&after_extended)
        } else {
            None
        },
        &params,
        mechanics,
        extended_continuation.terminal_failure_step,
    );
    ArmEvidence {
        id: id.to_owned(),
        declared_k_a_decay,
        effective_starvation_k_a_decay: declared_k_a_decay * STARVATION_MULTIPLIER,
        r2_comparison,
        r2_continuation,
        extended_continuation,
        restoration,
    }
}

fn write_json(path: &Path, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::env::var_os("DCDEV020M1R2R1_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020m1r2r1"));
    let (entry_mesh, mechanics) = m1r1::m1r1_entry_state();
    assert!(close(mechanics.dt, DT));
    let production = run_arm(
        "A_PRODUCTION_STARVATION_4X",
        PRODUCTION_K_A_DECAY,
        &entry_mesh,
        &mechanics,
    );
    let ordinary = run_arm(
        "B_ORDINARY_DECAY_STARVATION",
        ORDINARY_K_A_DECAY,
        &entry_mesh,
        &mechanics,
    );
    let ordinary_terminal = ordinary
        .extended_continuation
        .terminal_failure_step
        .is_some();
    let production_terminal = production
        .extended_continuation
        .terminal_failure_step
        .is_some();
    let classification = match (production_terminal, ordinary_terminal) {
        (false, false) => "M1_NO_IRREVERSIBLE_FAILURE_WITHIN_EXTENDED_BOUND",
        (true, false) => "M1_4X_ONLY_IRREVERSIBLE_FAILURE_ESTABLISHED",
        (false, true) => "M1_ORDINARY_DECAY_IRREVERSIBLE_FAILURE_ESTABLISHED",
        // Both arms reaching the existing terminal boundary establishes that
        // ordinary decay is sufficient; the 4x-only class is reserved for
        // the case where ordinary decay does not fail within the bound.
        (true, true) => "M1_ORDINARY_DECAY_IRREVERSIBLE_FAILURE_ESTABLISHED",
    };
    let restoration_pass = [&production, &ordinary].iter().all(|arm| {
        !arm.restoration.reached
            || (arm.restoration.accepted_steps == RESTORATION_STEPS
                && arm
                    .restoration
                    .world_to_organism_closure_residual
                    .unwrap_or(f64::INFINITY)
                    <= TOL
                && arm.restoration.no_resurrection)
    });
    let closure_pass = [&production, &ordinary].iter().all(|arm| {
        arm.r2_comparison.strict_material_closure_residual <= TOL
            && arm.r2_continuation.strict_material_closure_residual <= TOL
            && arm.extended_continuation.strict_material_closure_residual <= TOL
            && (!arm.restoration.reached
                || arm
                    .restoration
                    .world_to_organism_closure_residual
                    .unwrap_or(f64::INFINITY)
                    <= TOL)
    });
    let protocol = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "entry_state": "accepted M1-R2 exact endpoint at step 20480; no founder regeneration",
        "r2_comparison_steps": R2_COMPARISON_STEPS,
        "r2_continuation_steps": R2_CONTINUATION_STEPS,
        "extended_continuation_steps": EXTENDED_STEPS,
        "restoration_steps": RESTORATION_STEPS,
        "restoration_resource": {"n": RESTORED_N, "f": RESTORED_F, "center": RESOURCE_CENTER, "radius": RESOURCE_RADIUS},
        "arms": [
            {"id": "A_PRODUCTION_STARVATION_4X", "declared_k_a_decay": PRODUCTION_K_A_DECAY, "effective_starvation_k_a_decay": PRODUCTION_K_A_DECAY * STARVATION_MULTIPLIER},
            {"id": "B_ORDINARY_DECAY_STARVATION", "declared_k_a_decay": ORDINARY_K_A_DECAY, "effective_starvation_k_a_decay": ORDINARY_K_A_DECAY * STARVATION_MULTIPLIER}
        ],
        "terminal_failure_condition": ["ruptured_edges * 2 >= edge_count", "observer_death_reason is non-starvation", "physical_runtime_valid == false"],
        "observer_only": true,
        "forbidden_changes": ["chemistry-core", "k_a_decay", "4x production branch", "activation", "uptake", "transport", "resources", "death criteria", "D-091", "D-087", "recycling", "salvage", "M2", "DC-DEV-021"]
    });
    let results = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "arms": {"A_PRODUCTION_STARVATION_4X": production, "B_ORDINARY_DECAY_STARVATION": ordinary},
        "classification": classification,
        "restoration_pass": restoration_pass,
        "internal_material_closure": closure_pass,
        "production_biology_changed": false,
        "chemistry_core_changed": false,
        "parameter_search": false,
        "recycling": false,
        "next_execution_started": false
    });
    let qualification = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "observer_only": true,
        "e0_exact_r2_endpoint_reproduction": true,
        "e1_failure_margin_instrumentation": true,
        "e2_extended_starvation": true,
        "e3_no_reset_restoration": restoration_pass,
        "e4_strict_material_closure": closure_pass,
        "e4_fresh_d087_required": "PENDING_SEPARATE_CERTIFIER_STAGE",
        "production_biology_changed": false,
        "chemistry_core_changed": false,
        "parameter_search": false,
        "recycling": false,
        "m1_production_change_authorized": false,
        "m2_authorized": false,
        "recycling_authorized": false,
        "dc_dev_021_authorized": false,
        "next_execution_started": false,
        "classification": classification
    });
    let manifest = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "artifact_files": ["protocol.json", "results.json", "qualification.json", "artifact_manifest.json"],
        "authoritative_result": "results.json",
        "dense_ledgers_committed": false,
        "observer_only": true,
        "next_execution_started": false
    });
    write_json(&out.join("protocol.json"), &protocol)?;
    write_json(&out.join("results.json"), &results)?;
    write_json(&out.join("qualification.json"), &qualification)?;
    write_json(&out.join("artifact_manifest.json"), &manifest)?;
    println!("DCDEV020M1R2R1_PHYSICAL_FAILURE_CLOSURE_COMPLETE");
    println!("classification={classification}");
    println!("production_terminal={production_terminal}");
    println!("ordinary_terminal={ordinary_terminal}");
    println!("restoration_pass={restoration_pass}");
    println!("internal_material_closure={closure_pass}");
    println!("{}", out.display());
    Ok(())
}
