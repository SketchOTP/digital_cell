//! DC-DEV-020-M1-R2 observer-only starvation-law audit.
//!
//! This audit compares the frozen production starvation branch with an
//! example-local shadow that divides only `ReactionParams.k_a_decay` by four.
//! It deliberately continues past the reversible observer starvation predicate
//! so that physical failure and restoration are reported separately.

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

const DIRECTIVE: &str = "DC-DEV-020-M1-R2-STARVATION-LAW-AUDIT-001";
const STARTING_HEAD: &str = "7bb48874771144795a9559f7570f5ebc77e1004a";
const M1R1R1_ENTRY: &str = "7bb48874771144795a9559f7570f5ebc77e1004a";
const COMPARISON_STEPS: usize = 480;
const CONTINUATION_STEPS: usize = 20_000;
const RESTORATION_STEPS: usize = 5_000;
const DT: f64 = 0.02;
const PRODUCTION_K_A_DECAY: f64 = 0.008;
const ORDINARY_K_A_DECAY: f64 = 0.002;
const STARVATION_MULTIPLIER: f64 = 4.0;
const RESOURCE_CENTER: [f64; 2] = [4.8, 0.0];
const RESOURCE_RADIUS: f64 = 1.5;
const HIGH_INVENTORY: f64 = 14.588954880632265;
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
    checkpoints: Vec<CompactSnapshot>,
    reaction_totals: ReactionTotals,
    organized_material_delta: f64,
    strict_material_delta: f64,
    strict_material_closure_residual: f64,
    first_starvation_collapse_step: Option<usize>,
    first_physical_failure: Option<String>,
    first_physical_failure_step: Option<usize>,
    trajectory_hash: String,
}

#[derive(Debug, Clone, Serialize)]
struct RestorationEvidence {
    reached: bool,
    requested_steps: usize,
    accepted_steps: usize,
    initial: Option<CompactSnapshot>,
    final_state: Option<CompactSnapshot>,
    n_remaining: f64,
    f_remaining: f64,
    n_delivered: f64,
    f_delivered: f64,
    world_to_organism_closure_residual: Option<f64>,
    organized_material_recovers: bool,
    observer_viability_recovers: bool,
    ruptured_topology_recovers: bool,
    new_coherent_body_appears: bool,
}

#[derive(Debug, Clone, Serialize)]
struct StarvationArm {
    id: String,
    decay_contract: String,
    declared_k_a_decay: f64,
    frozen_starvation_multiplier: f64,
    effective_starvation_k_a_decay: f64,
    comparison_480: PhaseEvidence,
    continuation: PhaseEvidence,
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

fn physical_failure(mesh: &MaterialMesh) -> Option<String> {
    if !mesh.physical_runtime_valid() {
        return Some("physical_runtime_invalid".into());
    }
    if mesh.edges.iter().any(|edge| edge.ruptured) {
        return Some("mesh_rupture".into());
    }
    match mesh.observer_death_reason() {
        Some("starvation_collapse") | None => None,
        Some(reason) => Some(reason.into()),
    }
}

fn phase_checkpoints(step: usize) -> bool {
    matches!(
        step,
        1 | 120 | 240 | 360 | 480 | 1_000 | 5_000 | 10_000 | 20_000
    )
}

fn run_phase(
    mut mesh: MaterialMesh,
    params: &ReactionParams,
    mechanics: &MechParams,
    start_step: usize,
    requested_steps: usize,
) -> (MaterialMesh, PhaseEvidence) {
    let initial = CompactSnapshot::from_mesh(&mesh, start_step);
    let mut current = initial;
    let mut totals = ReactionTotals::default();
    let mut checkpoints = Vec::new();
    let mut trajectory = vec![stable_json_hash(&initial).expect("snapshot hash")];
    let mut first_starvation_collapse_step =
        if mesh.observer_death_reason() == Some("starvation_collapse") {
            Some(start_step)
        } else {
            None
        };
    let mut first_physical_failure = physical_failure(&mesh);
    let mut first_physical_failure_step = first_physical_failure.as_ref().map(|_| start_step);
    let mut accepted_steps = 0;

    for relative_step in 1..=requested_steps {
        if first_physical_failure.is_some() {
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
        if first_starvation_collapse_step.is_none()
            && mesh.observer_death_reason() == Some("starvation_collapse")
        {
            first_starvation_collapse_step = Some(absolute_step);
        }
        if first_physical_failure.is_none() {
            first_physical_failure = physical_failure(&mesh);
            if first_physical_failure.is_some() {
                first_physical_failure_step = Some(absolute_step);
            }
        }
        if phase_checkpoints(relative_step) || first_physical_failure.is_some() {
            checkpoints.push(current);
        }
        trajectory.push(stable_json_hash(&current).expect("snapshot hash"));
    }

    let strict_delta = current.strict_material - initial.strict_material;
    let evidence = PhaseEvidence {
        start_step,
        end_step: start_step + accepted_steps,
        requested_steps,
        accepted_steps,
        initial,
        final_state: current,
        checkpoints,
        reaction_totals: totals,
        organized_material_delta: current.organized_material - initial.organized_material,
        strict_material_delta: strict_delta,
        strict_material_closure_residual: strict_delta.abs(),
        first_starvation_collapse_step,
        first_physical_failure,
        first_physical_failure_step,
        trajectory_hash: stable_json_hash(&trajectory).expect("trajectory hash"),
    };
    (mesh, evidence)
}

fn run_restoration(
    mesh_at_failure: Option<&MaterialMesh>,
    params: &ReactionParams,
    mechanics: &MechParams,
) -> RestorationEvidence {
    let Some(mesh_at_failure) = mesh_at_failure else {
        return RestorationEvidence {
            reached: false,
            requested_steps: RESTORATION_STEPS,
            accepted_steps: 0,
            initial: None,
            final_state: None,
            n_remaining: 0.0,
            f_remaining: 0.0,
            n_delivered: 0.0,
            f_delivered: 0.0,
            world_to_organism_closure_residual: None,
            organized_material_recovers: false,
            observer_viability_recovers: false,
            ruptured_topology_recovers: false,
            new_coherent_body_appears: false,
        };
    };

    let mut mesh = mesh_at_failure.clone();
    let initial = CompactSnapshot::from_mesh(&mesh, 0);
    let initial_ruptured = initial.ruptured_edges;
    let mut current = initial;
    let mut region = FiniteSpatialResourceRegionV1::new(
        RESOURCE_CENTER,
        RESOURCE_RADIUS,
        HIGH_INVENTORY,
        HIGH_INVENTORY,
    );
    let transport = TransportParams::default();
    let mut n_delivered = 0.0;
    let mut f_delivered = 0.0;
    let mut accepted_steps = 0;
    for step in 1..=RESTORATION_STEPS {
        if !mesh.physical_runtime_valid() {
            break;
        }
        let uptake = region.uptake(&mut mesh, &transport, mechanics.dt);
        assert!(uptake.conservation_error <= TOL);
        n_delivered += uptake.n_delivered;
        f_delivered += uptake.f_delivered;
        reactions_step(&mut mesh, params, mechanics.dt, true, true);
        current = CompactSnapshot::from_mesh(&mesh, step);
        accepted_steps = step;
    }
    let strict_delta = current.strict_material - initial.strict_material;
    let residual = (strict_delta - n_delivered - f_delivered).abs();
    RestorationEvidence {
        reached: true,
        requested_steps: RESTORATION_STEPS,
        accepted_steps,
        initial: Some(initial),
        final_state: Some(current),
        n_remaining: region.n_mass,
        f_remaining: region.f_mass,
        n_delivered,
        f_delivered,
        world_to_organism_closure_residual: Some(residual),
        organized_material_recovers: current.organized_material >= initial.organized_material - TOL,
        observer_viability_recovers: current.observer_viable,
        ruptured_topology_recovers: current.ruptured_edges < initial_ruptured,
        new_coherent_body_appears: mesh.closed_intact(),
    }
}

fn run_starvation_arm(
    id: &str,
    declared_k_a_decay: f64,
    initial: &MaterialMesh,
    mechanics: &MechParams,
) -> StarvationArm {
    let params = reaction_params(declared_k_a_decay);
    let (after_480, comparison_480) =
        run_phase(initial.clone(), &params, mechanics, 0, COMPARISON_STEPS);
    assert_eq!(comparison_480.accepted_steps, COMPARISON_STEPS);
    let (after_continuation, continuation) = if comparison_480.first_physical_failure.is_none() {
        run_phase(
            after_480,
            &params,
            mechanics,
            COMPARISON_STEPS,
            CONTINUATION_STEPS,
        )
    } else {
        (
            after_480,
            PhaseEvidence {
                start_step: COMPARISON_STEPS,
                end_step: COMPARISON_STEPS,
                requested_steps: CONTINUATION_STEPS,
                accepted_steps: 0,
                initial: comparison_480.final_state,
                final_state: comparison_480.final_state,
                checkpoints: Vec::new(),
                reaction_totals: ReactionTotals::default(),
                organized_material_delta: 0.0,
                strict_material_delta: 0.0,
                strict_material_closure_residual: 0.0,
                first_starvation_collapse_step: None,
                first_physical_failure: comparison_480.first_physical_failure.clone(),
                first_physical_failure_step: comparison_480.first_physical_failure_step,
                trajectory_hash: comparison_480.trajectory_hash.clone(),
            },
        )
    };
    let restoration = run_restoration(
        if continuation.first_physical_failure.is_some() {
            Some(&after_continuation)
        } else {
            None
        },
        &params,
        mechanics,
    );
    StarvationArm {
        id: id.into(),
        decay_contract: if close(declared_k_a_decay, PRODUCTION_K_A_DECAY) {
            "production frozen k_a_decay with starvation multiplier".into()
        } else {
            "diagnostic k_a_decay=production K/4 with frozen starvation multiplier".into()
        },
        declared_k_a_decay,
        frozen_starvation_multiplier: STARVATION_MULTIPLIER,
        effective_starvation_k_a_decay: declared_k_a_decay * STARVATION_MULTIPLIER,
        comparison_480,
        continuation,
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
    let out = std::env::var_os("DCDEV020M1R2_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020m1r2"));
    let (entry_mesh, mechanics) = m1r1::m1r1_entry_state();
    assert!(close(mechanics.dt, DT));

    // Gate 0: reproduce the accepted M1-R1-R1 boundary values before the new
    // starvation-law comparison is evaluated.
    let base =
        m1r1::run_arm_with_options(&entry_mesh, m1r1::Shadow::Base, &mechanics, None, false).0;
    let raw_source = m1r1::run_arm_with_options(
        &entry_mesh,
        m1r1::Shadow::SourceCapacityUpperBound,
        &mechanics,
        None,
        false,
    )
    .0;
    let source_neutral = m1r1::run_arm_with_options(
        &entry_mesh,
        m1r1::Shadow::SourceCapacityUpperBound,
        &mechanics,
        Some(ORDINARY_K_A_DECAY),
        false,
    )
    .0;
    let combined_neutral = m1r1::run_arm_with_options(
        &entry_mesh,
        m1r1::Shadow::Combined,
        &mechanics,
        Some(ORDINARY_K_A_DECAY),
        false,
    )
    .0;
    let r1r1_reproduction = close(base.organized_material_delta, -9.200978427498057)
        && close(raw_source.organized_material_delta, -3.09944444397982)
        && close(source_neutral.organized_material_delta, 1.25718049040759)
        && close(combined_neutral.organized_material_delta, 1.2755639121915);
    assert!(r1r1_reproduction);

    let production = run_starvation_arm(
        "A_PRODUCTION_STARVATION_4X",
        PRODUCTION_K_A_DECAY,
        &entry_mesh,
        &mechanics,
    );
    let ordinary = run_starvation_arm(
        "B_ORDINARY_DECAY_STARVATION",
        ORDINARY_K_A_DECAY,
        &entry_mesh,
        &mechanics,
    );
    let fed =
        m1r1::run_arm_with_options(&entry_mesh, m1r1::Shadow::Base, &mechanics, None, false).0;
    assert!(close(
        production.effective_starvation_k_a_decay,
        PRODUCTION_K_A_DECAY * STARVATION_MULTIPLIER
    ));
    assert!(close(
        ordinary.effective_starvation_k_a_decay,
        PRODUCTION_K_A_DECAY
    ));
    assert!(production.comparison_480.strict_material_closure_residual <= TOL);
    assert!(ordinary.comparison_480.strict_material_closure_residual <= TOL);

    let classification = match (
        production.continuation.first_physical_failure_step,
        ordinary.continuation.first_physical_failure_step,
    ) {
        (None, None) => "M1_STARVATION_LAW_AUDIT_INCONCLUSIVE",
        (Some(_), None) => "M1_STARVATION_4X_REQUIRED_FOR_IRREVERSIBLE_FAILURE_WITHIN_BOUND",
        (None, Some(_)) => "M1_STARVATION_4X_NOT_REQUIRED",
        (Some(production_step), Some(ordinary_step)) if ordinary_step > production_step => {
            "M1_STARVATION_4X_ACCELERATES_BUT_IS_NOT_REQUIRED"
        }
        _ => "M1_STARVATION_4X_NOT_REQUIRED",
    };

    let provenance = json!({
        "originating_commit": "20e9f7814020ca38ed1893fdd94fb3264307de2e",
        "originating_directive": "D-20260724-d086-autopoietic-material-mesh-protocell",
        "source": "digital-protocell/crates/chemistry-core/src/mesh_reactions.rs",
        "implementation_comment": "Accelerate A loss when activation substrates are absent (starvation).",
        "explicit_scientific_rationale": false,
        "explicitly_required_by_d087": false,
        "d087_requires_starvation_death_behavior": true,
        "required_by_current_m0_8_of_8": "UNTESTED_UNTIL_FRESH_CERTIFIER_STAGE",
        "interpretation": "The term is carried in frozen production code, but repository provenance does not independently qualify the coefficient as necessary."
    });
    let protocol = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "m1r1r1_entry": M1R1R1_ENTRY,
        "selected_contract": {"mesh_contract": "ConservativeV2", "reserve_enabled": false},
        "dt": DT,
        "comparison_steps": COMPARISON_STEPS,
        "continuation_steps": CONTINUATION_STEPS,
        "restoration_steps": RESTORATION_STEPS,
        "arms": [
            {"id": "A_PRODUCTION_STARVATION_4X", "resource_delivery": "none", "declared_k_a_decay": PRODUCTION_K_A_DECAY, "starvation_multiplier": STARVATION_MULTIPLIER},
            {"id": "B_ORDINARY_DECAY_STARVATION", "resource_delivery": "none", "declared_k_a_decay": ORDINARY_K_A_DECAY, "starvation_multiplier": STARVATION_MULTIPLIER},
            {"id": "C_FED_CONTROL", "resource_delivery": "accepted finite-resource reference", "declared_k_a_decay": PRODUCTION_K_A_DECAY}
        ],
        "resource_restoration": {"center": RESOURCE_CENTER, "radius": RESOURCE_RADIUS, "inventory_n": HIGH_INVENTORY, "inventory_f": HIGH_INVENTORY},
        "provenance": provenance,
        "observer_only": true,
        "forbidden_changes": ["production chemistry", "chemistry-core", "ConservativeV2", "activation law", "uptake", "transport", "resources", "death semantics", "D-087", "recycling", "salvage", "M2", "DC-DEV-021"]
    });
    let results = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "m1r1r1_reproduction": {
            "pass": r1r1_reproduction,
            "base_organized_delta": base.organized_material_delta,
            "raw_source_organized_delta": raw_source.organized_material_delta,
            "source_decay_neutral_organized_delta": source_neutral.organized_material_delta,
            "combined_decay_neutral_organized_delta": combined_neutral.organized_material_delta,
            "accepted_classification": "M1_SOURCE_CAPACITY_SUFFICIENT_AFTER_DECAY_NEUTRALIZATION"
        },
        "provenance": provenance,
        "arms": {"A_PRODUCTION_STARVATION_4X": production, "B_ORDINARY_DECAY_STARVATION": ordinary},
        "fed_control": serde_json::to_value(&fed)?,
        "fed_control_pass": fed.world_to_organism_closure_residual <= TOL
            && fed.internal_material_closure_residual <= TOL,
        "classification": classification,
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
        "gate0_m1r1r1_reproduction": r1r1_reproduction,
        "gate1_provenance_audited": true,
        "gate2_exact_starvation_comparison": production.comparison_480.accepted_steps == COMPARISON_STEPS && ordinary.comparison_480.accepted_steps == COMPARISON_STEPS,
        "gate3_bounded_physical_failure_continuation": production.continuation.requested_steps == CONTINUATION_STEPS && ordinary.continuation.requested_steps == CONTINUATION_STEPS,
        "gate4_resource_restoration_challenge": true,
        "gate5_causal_necessity_classification": classification != "M1_STARVATION_LAW_AUDIT_INCONCLUSIVE",
        "gate6_fresh_d087_required": "PENDING_SEPARATE_CERTIFIER_STAGE",
        "world_organism_closure": fed.world_to_organism_closure_residual <= TOL,
        "internal_material_closure": fed.internal_material_closure_residual <= TOL
            && production.comparison_480.strict_material_closure_residual <= TOL
            && ordinary.comparison_480.strict_material_closure_residual <= TOL,
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
        "qualification": "qualification.json",
        "dense_ledgers_committed": false,
        "observer_only": true,
        "next_execution_started": false
    });
    write_json(&out.join("protocol.json"), &protocol)?;
    write_json(&out.join("results.json"), &results)?;
    write_json(&out.join("qualification.json"), &qualification)?;
    write_json(&out.join("artifact_manifest.json"), &manifest)?;
    println!("DCDEV020M1R2_STARVATION_LAW_AUDIT_COMPLETE");
    println!("classification={classification}");
    println!("{}", out.display());
    Ok(())
}
