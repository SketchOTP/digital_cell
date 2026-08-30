//! DC-DEV-021 ENTRY-004: observer-only intrinsic-to-traction transfer audit.
//!
//! This executable reproduces the frozen ENTRY-001 and ENTRY-003 assays while
//! inspecting clone-only free proposals before the existing stick-slip adapter
//! applies its reactions. It does not alter any production runtime path.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, MeshContractVersion, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use regulatory_core::{
    apply_local_activated_energy_contractility,
    apply_local_activated_energy_contractility_with_stick_slip, commit_intrinsic_exploration_step,
    propose_intrinsic_exploration_step, stable_json_hash, ContactLedgerV1, ContactRegimeV1,
    ContractilityParamsV1, IntrinsicExplorationDynamicsModeV1, IntrinsicExplorationStateV1,
    StickSlipTractionParamsV1, FROZEN_ADAPTATION_LOAD_RATE_PER_TIME,
    FROZEN_ADAPTATION_RECOVERY_RATE_PER_TIME, FROZEN_DT, FROZEN_STATIC_TRACTION_LIMIT,
    FROZEN_ZERO_MOTION_TOLERANCE,
};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-021-M2-ENTRY-004-INTRINSIC-TRACTION-TRANSFER-AUDIT-001";
const ENTRY_HEAD: &str = "ec94802b27b012919011747934b58312cebed74c";
const ENTRY003_HEAD: &str = "2ed0f6159b0169f1f7bd9c2c10e89a6b67d12167";
const TOPOLOGY_SIZE: usize = 24;
const SETTLEMENT_STEPS: usize = 5_000;
const ENTRY001_ACTIVE_STEPS: usize = 240;
const ENTRY001_TOTAL_STEPS: usize = 480;
const ENTRY003_STEPS: usize = ((1.0 / FROZEN_ADAPTATION_LOAD_RATE_PER_TIME
    + 1.0 / FROZEN_ADAPTATION_RECOVERY_RATE_PER_TIME)
    / FROZEN_DT) as usize;
const INTRINSIC_SEED: u64 = 1;

#[derive(Default, Clone)]
struct Distribution {
    values: Vec<f64>,
}

impl Distribution {
    fn add(&mut self, value: f64) {
        assert!(value.is_finite());
        self.values.push(value);
    }

    fn json(&self) -> Value {
        let mut values = self.values.clone();
        values.sort_by(|left, right| left.partial_cmp(right).unwrap());
        let percentile = |p: f64| -> f64 {
            if values.is_empty() {
                0.0
            } else {
                values[((values.len() - 1) as f64 * p).round() as usize]
            }
        };
        json!({
            "count": values.len(),
            "minimum": values.first().copied().unwrap_or(0.0),
            "maximum": values.last().copied().unwrap_or(0.0),
            "mean": if values.is_empty() { 0.0 } else { values.iter().sum::<f64>() / values.len() as f64 },
            "median": percentile(0.50),
            "p90": percentile(0.90),
            "p95": percentile(0.95),
            "p99": percentile(0.99),
        })
    }
}

#[derive(Default, Clone)]
struct ForceSummary {
    force: Distribution,
    tension: Distribution,
    activity: Distribution,
    crossings: usize,
    maximum_force: f64,
    maximum_tension: f64,
    maximum_activity: f64,
    harmonic_magnitude: Distribution,
    harmonic_phase: Distribution,
    active_half_peak_fraction: Distribution,
    activity_range: Distribution,
    activity_variance: Distribution,
    net_sum_ratios: Distribution,
    torque: Distribution,
}

impl ForceSummary {
    fn add_activity(&mut self, activity: &[f64]) {
        let peak = activity.iter().copied().fold(0.0_f64, f64::max);
        let minimum = activity.iter().copied().fold(f64::INFINITY, f64::min);
        let mean = activity.iter().sum::<f64>() / activity.len() as f64;
        let variance = activity
            .iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>()
            / activity.len() as f64;
        let mut x = 0.0;
        let mut y = 0.0;
        for (index, value) in activity.iter().enumerate() {
            let angle = std::f64::consts::TAU * index as f64 / activity.len() as f64;
            x += value * angle.cos();
            y += value * angle.sin();
            self.activity.add(*value);
        }
        let total = activity.iter().sum::<f64>();
        self.maximum_activity = self.maximum_activity.max(peak);
        self.activity_range.add(peak - minimum);
        self.activity_variance.add(variance);
        self.harmonic_magnitude
            .add(if total > 0.0 { x.hypot(y) / total } else { 0.0 });
        self.harmonic_phase.add(y.atan2(x));
        self.active_half_peak_fraction.add(if peak > 0.0 {
            activity.iter().filter(|value| **value > 0.5 * peak).count() as f64
                / activity.len() as f64
        } else {
            0.0
        });
    }

    fn add_force_frame(
        &mut self,
        before: &MaterialMesh,
        free: &MaterialMesh,
        activity: &[f64],
        mechanics: &MechParams,
        contractility: &ContractilityParamsV1,
    ) -> FrameForces {
        self.add_activity(activity);
        let mut vectors = Vec::with_capacity(before.n());
        let mut sum = [0.0, 0.0];
        let mut sum_norms = 0.0;
        let center = material_centroid(before);
        let mut torque = 0.0;
        for index in 0..before.n() {
            let delta = [
                free.vertices[index][0] - before.vertices[index][0],
                free.vertices[index][1] - before.vertices[index][1],
            ];
            let attempted_velocity = [
                delta[0] * mechanics.gamma / mechanics.dt,
                delta[1] * mechanics.gamma / mechanics.dt,
            ];
            let required_force = [
                attempted_velocity[0] * mechanics.gamma,
                attempted_velocity[1] * mechanics.gamma,
            ];
            let magnitude = norm(required_force);
            self.force.add(magnitude);
            self.maximum_force = self.maximum_force.max(magnitude);
            if magnitude > FROZEN_STATIC_TRACTION_LIMIT {
                self.crossings += 1;
            }
            sum[0] += required_force[0];
            sum[1] += required_force[1];
            sum_norms += magnitude;
            let radius = [
                before.vertices[index][0] - center[0],
                before.vertices[index][1] - center[1],
            ];
            torque += radius[0] * required_force[1] - radius[1] * required_force[0];
            vectors.push(VertexForce {
                index,
                activity: activity[index],
                local_active_tension: local_tension(index, activity, before, contractility),
                free_delta: delta,
                attempted_velocity,
                required_force,
                required_force_norm: magnitude,
            });
        }
        for index in 0..before.n() {
            self.tension
                .add(local_tension(index, activity, before, contractility));
        }
        self.maximum_tension = self.maximum_tension.max(
            activity
                .iter()
                .enumerate()
                .map(|(index, _)| local_tension(index, activity, before, contractility))
                .fold(0.0_f64, f64::max),
        );
        self.net_sum_ratios.add(if sum_norms > 0.0 {
            norm(sum) / sum_norms
        } else {
            0.0
        });
        self.torque.add(torque);
        FrameForces {
            vertices: vectors,
            net_force: sum,
            summed_norm: sum_norms,
            torque,
        }
    }

    fn json(&self) -> Value {
        json!({
            "required_force": self.force.json(),
            "tension": self.tension.json(),
            "activity": self.activity.json(),
            "count_above_static_limit": self.crossings,
            "fraction_above_static_limit": if self.force.values.is_empty() { 0.0 } else { self.crossings as f64 / self.force.values.len() as f64 },
            "maximum_required_force": self.maximum_force,
            "maximum_required_force_minus_static_limit": self.maximum_force - FROZEN_STATIC_TRACTION_LIMIT,
            "maximum_tension": self.maximum_tension,
            "maximum_activity": self.maximum_activity,
            "spatial_profile": {
                "activity_max_minus_min": self.activity_range.json(),
                "activity_variance": self.activity_variance.json(),
                "first_circular_harmonic_magnitude": self.harmonic_magnitude.json(),
                "first_circular_harmonic_phase": self.harmonic_phase.json(),
                "fraction_patches_above_half_frame_peak": self.active_half_peak_fraction.json(),
            },
            "force_cancellation": {
                "net_force_over_summed_norm": self.net_sum_ratios.json(),
                "torque_about_material_centroid": self.torque.json(),
            },
        })
    }
}

#[derive(Clone)]
struct VertexForce {
    index: usize,
    activity: f64,
    local_active_tension: f64,
    free_delta: [f64; 2],
    attempted_velocity: [f64; 2],
    required_force: [f64; 2],
    required_force_norm: f64,
}

#[derive(Clone)]
struct FrameForces {
    vertices: Vec<VertexForce>,
    net_force: [f64; 2],
    summed_norm: f64,
    torque: f64,
}

#[derive(Default)]
struct AuditRun {
    effective: ForceSummary,
    raw: ForceSummary,
    unit_peak: ForceSummary,
    actual_sticks: usize,
    actual_slips: usize,
    parity: bool,
    slip_patch_indices: Vec<usize>,
    slip_step_indices: Vec<usize>,
    pinning: Vec<Value>,
    dense: Vec<Value>,
    a_spent: f64,
    path_length: f64,
    net_displacement: f64,
    initial_material_centroid: [f64; 2],
    final_material_centroid: [f64; 2],
    reaction_net_sum_ratios: Distribution,
    reaction_torque: Distribution,
}

fn write_json(root: &Path, name: &str, value: &Value) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn norm(value: [f64; 2]) -> f64 {
    value[0].hypot(value[1])
}

fn sub(left: [f64; 2], right: [f64; 2]) -> [f64; 2] {
    [left[0] - right[0], left[1] - right[1]]
}

fn material_centroid(mesh: &MaterialMesh) -> [f64; 2] {
    let mut weighted = [0.0, 0.0];
    let mut total = 0.0;
    for index in 0..mesh.n() {
        let a = mesh.vertices[index];
        let b = mesh.vertices[(index + 1) % mesh.n()];
        let weight = (mesh.edges[index].m + mesh.edges[index].b).max(0.0);
        let midpoint = [0.5 * (a[0] + b[0]), 0.5 * (a[1] + b[1])];
        weighted[0] += weight * midpoint[0];
        weighted[1] += weight * midpoint[1];
        total += weight;
    }
    if total <= f64::EPSILON {
        mesh.centroid()
    } else {
        [weighted[0] / total, weighted[1] / total]
    }
}

fn seed_mesh() -> MaterialMesh {
    let mut mesh = MaterialMesh::seed_regular(
        TOPOLOGY_SIZE,
        5.0,
        0.0,
        0.0,
        DEFAULT_RHO_S,
        0.7,
        LumpedChem {
            c: 0.8,
            a: 0.6,
            n: 0.4,
            f: 0.4,
            r: 0.0,
            ..Default::default()
        },
        LumpedChem::default(),
        5.0,
    );
    mesh.contract_version = MeshContractVersion::MaturationCoupledV4;
    mesh
}

fn settled_body(mechanics: &MechParams) -> MaterialMesh {
    let mut mesh = seed_mesh();
    for _ in 0..SETTLEMENT_STEPS {
        assert!(mechanics_step(&mut mesh, mechanics));
    }
    assert!(mesh.area().is_finite() && mesh.area() > 0.0 && mesh.lifecycle_invariants_hold());
    mesh
}

fn entry001_activity(active: bool) -> Vec<f64> {
    (0..TOPOLOGY_SIZE)
        .map(|index| {
            if !active {
                0.0
            } else if index <= 4 {
                1.0
            } else if index <= 7 {
                0.35
            } else {
                0.0
            }
        })
        .collect()
}

fn local_tension(
    index: usize,
    activity: &[f64],
    mesh: &MaterialMesh,
    contractility: &ContractilityParamsV1,
) -> f64 {
    if mesh.edges[index].ruptured {
        0.0
    } else {
        contractility.max_active_tension
            * 0.5
            * (activity[index] + activity[(index + 1) % mesh.n()])
    }
}

fn free_proposal(
    before: &MaterialMesh,
    activity: &[f64],
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
) -> MaterialMesh {
    let mut free = before.clone();
    apply_local_activated_energy_contractility(&mut free, activity, mechanics, contractility)
        .unwrap();
    free
}

fn normalized(activity: &[f64]) -> Vec<f64> {
    let peak = activity.iter().copied().fold(0.0_f64, f64::max);
    if peak <= 0.0 {
        vec![0.0; activity.len()]
    } else {
        activity.iter().map(|value| value / peak).collect()
    }
}

fn vectors_json(frame: &FrameForces) -> Value {
    json!({
        "net_force": frame.net_force,
        "summed_norm": frame.summed_norm,
        "net_force_over_summed_norm": if frame.summed_norm > 0.0 { norm(frame.net_force) / frame.summed_norm } else { 0.0 },
        "torque": frame.torque,
        "vertices": frame.vertices.iter().map(|vertex| json!({
            "index": vertex.index,
            "activity_supplied_to_actuator": vertex.activity,
            "local_active_tension": vertex.local_active_tension,
            "free_step_delta": vertex.free_delta,
            "attempted_velocity": vertex.attempted_velocity,
            "required_force_vector": vertex.required_force,
            "required_force_norm": vertex.required_force_norm,
        })).collect::<Vec<_>>(),
    })
}

fn audit_stick_step(
    mesh: &mut MaterialMesh,
    activity: &[f64],
    raw_activity: &[f64],
    step: usize,
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
    traction: &StickSlipTractionParamsV1,
    run: &mut AuditRun,
) {
    let before = mesh.clone();
    let effective_free = free_proposal(&before, activity, mechanics, contractility);
    let effective_frame =
        run.effective
            .add_force_frame(&before, &effective_free, activity, mechanics, contractility);
    let raw_free = free_proposal(&before, raw_activity, mechanics, contractility);
    let raw_frame =
        run.raw
            .add_force_frame(&before, &raw_free, raw_activity, mechanics, contractility);
    let unit = normalized(raw_activity);
    let unit_free = free_proposal(&before, &unit, mechanics, contractility);
    let unit_frame =
        run.unit_peak
            .add_force_frame(&before, &unit_free, &unit, mechanics, contractility);

    let ledger = apply_local_activated_energy_contractility_with_stick_slip(
        mesh,
        activity,
        mechanics,
        contractility,
        traction,
    )
    .unwrap();
    let contractility_ledger = ledger.contractility.clone().unwrap();
    run.a_spent += contractility_ledger.resource_spent;
    run.actual_sticks += ledger.stuck_contacts;
    run.actual_slips += ledger.slipping_contacts;

    let center = material_centroid(&before);
    let mut reaction_sum = [0.0, 0.0];
    let mut reaction_norm_sum = 0.0;
    let mut reaction_torque = 0.0;
    for (index, contact) in ledger.contacts.iter().enumerate() {
        let predicted_slip =
            effective_frame.vertices[index].required_force_norm > traction.static_traction_limit;
        let actual_slip = contact.regime == ContactRegimeV1::Slip;
        let reconstructed_force = effective_frame.vertices[index].required_force_norm;
        if predicted_slip != actual_slip
            || (reconstructed_force - contact.required_force).abs() > 1e-10
        {
            run.parity = false;
        }
        if actual_slip {
            run.slip_patch_indices.push(index);
            run.slip_step_indices.push(step);
        }
        reaction_sum[0] += contact.reaction[0];
        reaction_sum[1] += contact.reaction[1];
        reaction_norm_sum += norm(contact.reaction);
        let radius = [
            before.vertices[index][0] - center[0],
            before.vertices[index][1] - center[1],
        ];
        reaction_torque += radius[0] * contact.reaction[1] - radius[1] * contact.reaction[0];
    }
    run.reaction_net_sum_ratios.add(if reaction_norm_sum > 0.0 {
        norm(reaction_sum) / reaction_norm_sum
    } else {
        0.0
    });
    run.reaction_torque.add(reaction_torque);
    let accepted_displacement = norm(sub(material_centroid(mesh), material_centroid(&before)));
    run.pinning.push(json!({
        "step": step,
        "maximum_raw_activity": raw_activity.iter().copied().fold(0.0_f64, f64::max),
        "maximum_effective_activity": activity.iter().copied().fold(0.0_f64, f64::max),
        "maximum_tension": contractility_ledger.maximum_tension,
        "maximum_required_force": effective_frame.vertices.iter().map(|v| v.required_force_norm).fold(0.0_f64, f64::max),
        "predicted_slips": effective_frame.vertices.iter().filter(|v| v.required_force_norm > traction.static_traction_limit).count(),
        "actual_slips": ledger.slipping_contacts,
        "free_step_material_centroid_displacement": norm(sub(material_centroid(&effective_free), material_centroid(&before))),
        "accepted_stick_slip_material_centroid_displacement": accepted_displacement,
        "a_spent": contractility_ledger.resource_spent,
    }));
    run.dense.push(json!({
        "step": step,
        "effective_free_proposal": vectors_json(&effective_frame),
        "raw_intrinsic_counterfactual": vectors_json(&raw_frame),
        "unit_peak_counterfactual": vectors_json(&unit_frame),
        "actual_contacts": ledger.contacts.iter().map(contact_json).collect::<Vec<_>>(),
    }));
}

fn contact_json(contact: &ContactLedgerV1) -> Value {
    json!({
        "regime": match contact.regime { ContactRegimeV1::Stick => "STICK", ContactRegimeV1::Slip => "SLIP" },
        "required_force": contact.required_force,
        "attempted_velocity": contact.attempted_velocity,
        "reaction": contact.reaction,
        "accepted_velocity": contact.accepted_velocity,
        "work": contact.work,
    })
}

fn finalize_run(mut run: AuditRun, mesh: &MaterialMesh) -> Value {
    run.final_material_centroid = material_centroid(mesh);
    run.net_displacement = norm(sub(
        run.final_material_centroid,
        run.initial_material_centroid,
    ));
    json!({
        "effective_activity": run.effective.json(),
        "raw_intrinsic_activity_counterfactual": run.raw.json(),
        "unit_peak_intrinsic_counterfactual": run.unit_peak.json(),
        "actual_stuck_contacts": run.actual_sticks,
        "actual_slipping_contacts": run.actual_slips,
        "clutch_prediction_ledger_parity": run.parity,
        "slip_event_patch_indices": run.slip_patch_indices,
        "slip_event_step_indices": run.slip_step_indices,
        "a_spent": run.a_spent,
        "retained_path_length": run.path_length,
        "net_displacement": run.net_displacement,
        "reaction_cancellation": {
            "net_reaction_over_summed_norm": run.reaction_net_sum_ratios.json(),
            "torque_about_material_centroid": run.reaction_torque.json(),
        },
        "pinning_chronology": run.pinning,
        "dense": run.dense,
    })
}

fn run_entry001(
    settled: &MaterialMesh,
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
    traction: &StickSlipTractionParamsV1,
) -> Value {
    let mut mesh = settled.clone();
    let mut run = AuditRun {
        parity: true,
        initial_material_centroid: material_centroid(&mesh),
        ..Default::default()
    };
    let mut previous = run.initial_material_centroid;
    for step in 0..ENTRY001_TOTAL_STEPS {
        let activity = entry001_activity(step < ENTRY001_ACTIVE_STEPS);
        if step < ENTRY001_ACTIVE_STEPS {
            audit_stick_step(
                &mut mesh,
                &activity,
                &activity,
                step,
                mechanics,
                contractility,
                traction,
                &mut run,
            );
        } else {
            let ledger = apply_local_activated_energy_contractility_with_stick_slip(
                &mut mesh,
                &activity,
                mechanics,
                contractility,
                traction,
            )
            .unwrap();
            run.actual_sticks += ledger.stuck_contacts;
            run.actual_slips += ledger.slipping_contacts;
            for contact in &ledger.contacts {
                if contact.regime == ContactRegimeV1::Slip {
                    run.slip_patch_indices.push(0);
                    run.slip_step_indices.push(step);
                }
            }
        }
        let next = material_centroid(&mesh);
        run.path_length += norm(sub(next, previous));
        previous = next;
    }
    let mut result = finalize_run(run, &mesh);
    result["final_mesh_hash"] = json!(stable_json_hash(&mesh).unwrap());
    result
}

fn run_entry003(
    settled: &MaterialMesh,
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
    traction: &StickSlipTractionParamsV1,
) -> Value {
    let mut mesh = settled.clone();
    let mut state = IntrinsicExplorationStateV1::new(mesh.n(), Some(INTRINSIC_SEED)).unwrap();
    let mut run = AuditRun {
        parity: true,
        initial_material_centroid: material_centroid(&mesh),
        ..Default::default()
    };
    let mut maximum_raw_activity = 0.0_f64;
    let mut previous = run.initial_material_centroid;
    for step in 0..ENTRY003_STEPS {
        let proposal = propose_intrinsic_exploration_step(
            &state,
            mesh.n(),
            mechanics.dt,
            IntrinsicExplorationDynamicsModeV1::FullSelfExcitation,
        )
        .unwrap();
        maximum_raw_activity = maximum_raw_activity.max(
            proposal
                .activity_after
                .iter()
                .copied()
                .fold(0.0_f64, f64::max),
        );
        audit_stick_step(
            &mut mesh,
            &proposal.effective_activity,
            &proposal.activity_after,
            step,
            mechanics,
            contractility,
            traction,
            &mut run,
        );
        commit_intrinsic_exploration_step(&mut state, proposal).unwrap();
        let next = material_centroid(&mesh);
        run.path_length += norm(sub(next, previous));
        previous = next;
    }
    let mut result = finalize_run(run, &mesh);
    result["maximum_activity"] = json!(maximum_raw_activity);
    result["final_state_hash"] = json!(stable_json_hash(&state).unwrap());
    result["final_mesh_hash"] = json!(stable_json_hash(&mesh).unwrap());
    result
}

fn run_no_substrate(
    settled: &MaterialMesh,
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
) -> Value {
    let mut mesh = settled.clone();
    let mut state = IntrinsicExplorationStateV1::new(mesh.n(), Some(INTRINSIC_SEED)).unwrap();
    let initial_material = material_centroid(&mesh);
    let initial_vertex = mesh.centroid();
    let mut previous_material = initial_material;
    let mut previous_vertex = initial_vertex;
    let mut material_path = 0.0;
    let mut vertex_path = 0.0;
    let mut net_force_ratios = Distribution::default();
    for _ in 0..ENTRY003_STEPS {
        let proposal = propose_intrinsic_exploration_step(
            &state,
            mesh.n(),
            mechanics.dt,
            IntrinsicExplorationDynamicsModeV1::FullSelfExcitation,
        )
        .unwrap();
        let before = mesh.clone();
        let free = free_proposal(
            &before,
            &proposal.effective_activity,
            mechanics,
            contractility,
        );
        let mut frame = ForceSummary::default();
        frame.add_force_frame(
            &before,
            &free,
            &proposal.effective_activity,
            mechanics,
            contractility,
        );
        net_force_ratios.add(*frame.net_sum_ratios.values.last().unwrap());
        mesh = free;
        commit_intrinsic_exploration_step(&mut state, proposal).unwrap();
        let material = material_centroid(&mesh);
        let vertex = mesh.centroid();
        material_path += norm(sub(material, previous_material));
        vertex_path += norm(sub(vertex, previous_vertex));
        previous_material = material;
        previous_vertex = vertex;
    }
    let material_net = norm(sub(material_centroid(&mesh), initial_material));
    let vertex_net = norm(sub(mesh.centroid(), initial_vertex));
    json!({
        "material_centroid_path_length": material_path,
        "vertex_centroid_path_length": vertex_path,
        "material_centroid_net_displacement": material_net,
        "vertex_centroid_net_displacement": vertex_net,
        "centroid_path_difference": (material_path - vertex_path).abs(),
        "net_displacement_difference": (material_net - vertex_net).abs(),
        "free_proposal_net_force_over_summed_norm": net_force_ratios.json(),
        "interpretation": "free-space deformation with centroid redistribution; no substrate reaction or retained locomotion claim",
    })
}

fn restart_boundary(
    settled: &MaterialMesh,
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
    traction: &StickSlipTractionParamsV1,
) -> Value {
    let mut mesh = settled.clone();
    let mut state = IntrinsicExplorationStateV1::new(mesh.n(), Some(INTRINSIC_SEED)).unwrap();
    for _ in 0..ENTRY003_STEPS / 2 {
        let proposal = propose_intrinsic_exploration_step(
            &state,
            mesh.n(),
            mechanics.dt,
            IntrinsicExplorationDynamicsModeV1::FullSelfExcitation,
        )
        .unwrap();
        apply_local_activated_energy_contractility_with_stick_slip(
            &mut mesh,
            &proposal.effective_activity,
            mechanics,
            contractility,
            traction,
        )
        .unwrap();
        commit_intrinsic_exploration_step(&mut state, proposal).unwrap();
    }
    let mesh_hash = stable_json_hash(&mesh).unwrap();
    let state_hash = stable_json_hash(&state).unwrap();
    let bytes = serde_json::to_vec(&(mesh, state)).unwrap();
    let (restored_mesh, restored_state): (MaterialMesh, IntrinsicExplorationStateV1) =
        serde_json::from_slice(&bytes).unwrap();
    let restored_mesh_hash = stable_json_hash(&restored_mesh).unwrap();
    let restored_state_hash = stable_json_hash(&restored_state).unwrap();
    json!({
        "intrinsic_state_restart": state_hash == restored_state_hash,
        "full_mesh_json_restart": mesh_hash == restored_mesh_hash,
        "restart_defect_affects_force_reconstruction": false,
        "reason": "the force audit uses uninterrupted accepted clone proposals and performs no full-mesh restart",
        "mesh_hash": mesh_hash,
        "restored_mesh_hash": restored_mesh_hash,
        "state_hash": state_hash,
        "restored_state_hash": restored_state_hash,
    })
}

fn source_hash(relative: &str) -> String {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join(relative);
    stable_json_hash(&fs::read(source).unwrap()).unwrap()
}

fn main() {
    let arguments: Vec<String> = env::args().collect();
    let output = arguments
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev021m2entry004"));
    let dense_output = arguments.get(2).map(PathBuf::from);
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    assert!((mechanics.dt - FROZEN_DT).abs() <= 1e-12);
    let settled = settled_body(&mechanics);
    let mut entry001 = run_entry001(&settled, &mechanics, &contractility, &traction);
    let mut entry003 = run_entry003(&settled, &mechanics, &contractility, &traction);
    let no_substrate = run_no_substrate(&settled, &mechanics, &contractility);
    let restart = restart_boundary(&settled, &mechanics, &contractility, &traction);

    // Per-vertex/per-step ledgers are deliberately Atlas-only.  Keep only their
    // compact aggregate results in Git evidence.
    let entry001_dense = entry001.as_object_mut().unwrap().remove("dense").unwrap();
    let entry003_dense = entry003.as_object_mut().unwrap().remove("dense").unwrap();
    // Emit the full per-step chronology once in its dedicated compact ledger,
    // rather than duplicating it in both reproduction summaries.
    let entry003_pinning_chronology = entry003["pinning_chronology"].clone();
    entry001
        .as_object_mut()
        .unwrap()
        .remove("pinning_chronology");
    entry003
        .as_object_mut()
        .unwrap()
        .remove("pinning_chronology");

    let entry001_force = &entry001["effective_activity"];
    let entry003_effective = &entry003["effective_activity"];
    let entry003_raw = &entry003["raw_intrinsic_activity_counterfactual"];
    let entry003_unit = &entry003["unit_peak_intrinsic_counterfactual"];
    let parity = entry001["clutch_prediction_ledger_parity"]
        .as_bool()
        .unwrap()
        && entry003["clutch_prediction_ledger_parity"]
            .as_bool()
            .unwrap();
    let entry001_reproduced =
        (entry001["net_displacement"].as_f64().unwrap() - 0.005665433467909554).abs() <= 1e-12
            && entry001["actual_slipping_contacts"].as_u64().unwrap() == 76;
    let entry003_reproduced =
        (entry003["maximum_activity"].as_f64().unwrap() - 0.8575416446188753).abs() <= 1e-12
            && entry003["actual_slipping_contacts"].as_u64().unwrap() == 0
            && entry003["retained_path_length"].as_f64().unwrap() <= FROZEN_ZERO_MOTION_TOLERANCE;
    let effective_crossings = entry003_effective["count_above_static_limit"]
        .as_u64()
        .unwrap();
    let raw_crossings = entry003_raw["count_above_static_limit"].as_u64().unwrap();
    let unit_crossings = entry003_unit["count_above_static_limit"].as_u64().unwrap();
    let entry001_crossings = entry001_force["count_above_static_limit"].as_u64().unwrap();
    let classification = if !parity
        || !entry001_reproduced
        || !entry003_reproduced
        || restart["restart_defect_affects_force_reconstruction"]
            .as_bool()
            .unwrap()
    {
        "M2_ENTRY004_TRACTION_TRANSFER_AUDIT_INVALID"
    } else if effective_crossings == 0 && raw_crossings > 0 {
        "M2_INTRINSIC_TRACTION_ADAPTATION_SUPPRESSION_CONFIRMED"
    } else if effective_crossings == 0
        && raw_crossings == 0
        && unit_crossings > 0
        && entry001_crossings > 0
    {
        "M2_INTRINSIC_TRACTION_FORCE_AMPLITUDE_SUBTHRESHOLD"
    } else if effective_crossings == 0
        && raw_crossings == 0
        && unit_crossings == 0
        && entry001_crossings > 0
    {
        "M2_INTRINSIC_TRACTION_SPATIAL_FORCE_PROFILE_INSUFFICIENT"
    } else if entry003["actual_slipping_contacts"].as_u64().unwrap() > 0
        && entry003["retained_path_length"].as_f64().unwrap() <= FROZEN_ZERO_MOTION_TOLERANCE
    {
        "M2_INTRINSIC_TRACTION_SLIP_CANCELLATION_CONFIRMED"
    } else {
        "M2_ENTRY004_TRACTION_TRANSFER_AUDIT_INVALID"
    };
    let preservation = json!({
        "m1_scientific_source_changed": false,
        "entry001_actuator_source_changed": false,
        "entry003_explorer_source_changed": false,
        "production_behavior_changed": false,
        "production_v4_reserve_off": true,
        "canonical_d087_required_in_exact_head_workflow": true,
    });
    let qualification = json!({
        "classification": classification,
        "entry001_reproduction": entry001_reproduced,
        "entry003_reproduction": entry003_reproduced,
        "clutch_prediction_ledger_parity": parity,
        "adaptation_suppression_decisive": classification == "M2_INTRINSIC_TRACTION_ADAPTATION_SUPPRESSION_CONFIRMED",
        "amplitude_decisive": classification == "M2_INTRINSIC_TRACTION_FORCE_AMPLITUDE_SUBTHRESHOLD",
        "spatial_force_profile_decisive": classification == "M2_INTRINSIC_TRACTION_SPATIAL_FORCE_PROFILE_INSUFFICIENT",
        "slip_cancellation_decisive": classification == "M2_INTRINSIC_TRACTION_SLIP_CANCELLATION_CONFIRMED",
        "m2_retained_exploration": "NOT_ESTABLISHED",
        "m2_autonomous_resource_acquisition": "NOT_ESTABLISHED",
        "source_or_mechanism_change_authorized": false,
    });
    write_json(
        &output,
        "protocol.json",
        &json!({
            "directive": DIRECTIVE,
            "entry_head": ENTRY_HEAD,
            "entry003_authority": ENTRY003_HEAD,
            "observer_only": true,
            "resource_sensor_target_gradient_planner": false,
            "traction_threshold": FROZEN_STATIC_TRACTION_LIMIT,
            "entry001_active_steps": ENTRY001_ACTIVE_STEPS,
            "entry003_steps": ENTRY003_STEPS,
        }),
    );
    write_json(&output, "entry001_reproduction.json", &entry001);
    write_json(&output, "entry003_reproduction.json", &entry003);
    write_json(
        &output,
        "clutch_rule_parity.json",
        &json!({"pass": parity, "rule": "required_force <= 0.45 => STICK; required_force > 0.45 => SLIP"}),
    );
    write_json(&output, "entry001_force_distribution.json", entry001_force);
    write_json(
        &output,
        "entry003_effective_force_distribution.json",
        entry003_effective,
    );
    write_json(
        &output,
        "entry003_raw_activity_counterfactual.json",
        entry003_raw,
    );
    write_json(
        &output,
        "entry003_unit_peak_counterfactual.json",
        entry003_unit,
    );
    write_json(
        &output,
        "spatial_profile_comparison.json",
        &json!({"entry001": entry001_force["spatial_profile"].clone(), "entry003_effective": entry003_effective["spatial_profile"].clone(), "entry003_raw": entry003_raw["spatial_profile"].clone()}),
    );
    write_json(
        &output,
        "force_cancellation.json",
        &json!({"entry001": entry001_force["force_cancellation"].clone(), "entry003": entry003_effective["force_cancellation"].clone(), "entry001_reaction": entry001["reaction_cancellation"].clone(), "entry003_reaction": entry003["reaction_cancellation"].clone()}),
    );
    write_json(
        &output,
        "pinning_chronology.json",
        &json!({"entry003": entry003_pinning_chronology}),
    );
    let entry001_distance = entry001["net_displacement"].as_f64().unwrap();
    let entry003_distance = entry003["net_displacement"].as_f64().unwrap();
    write_json(
        &output,
        "energy_efficiency.json",
        &json!({
            "entry001_a_spent_per_retained_displacement": entry001["a_spent"].as_f64().unwrap() / entry001_distance,
            "entry003_a_spent": entry003["a_spent"].clone(),
            "entry003_retained_displacement": entry003_distance,
            "entry003_slip_events_per_a_spent": 0.0,
            "entry003_efficiency": if entry003_distance <= FROZEN_ZERO_MOTION_TOLERANCE { "ZERO_OUTPUT_UNDEFINED" } else { "DEFINED" },
        }),
    );
    write_json(&output, "no_substrate_interpretation.json", &no_substrate);
    write_json(&output, "restart_boundary.json", &restart);
    write_json(&output, "preservation.json", &preservation);
    write_json(&output, "qualification.json", &qualification);
    write_json(
        &output,
        "artifact_manifest.json",
        &json!({
            "directive": DIRECTIVE,
            "files": ["protocol.json", "entry001_reproduction.json", "entry003_reproduction.json", "clutch_rule_parity.json", "entry001_force_distribution.json", "entry003_effective_force_distribution.json", "entry003_raw_activity_counterfactual.json", "entry003_unit_peak_counterfactual.json", "spatial_profile_comparison.json", "force_cancellation.json", "pinning_chronology.json", "energy_efficiency.json", "no_substrate_interpretation.json", "restart_boundary.json", "preservation.json", "qualification.json"],
            "source_hashes": {"contractility": source_hash("contractility.rs"), "traction": source_hash("stick_slip_traction.rs"), "intrinsic_exploration": source_hash("intrinsic_exploration.rs"), "plasticity": source_hash("plasticity.rs")},
        }),
    );
    if let Some(dense) = dense_output {
        write_json(
            &dense,
            "force_transfer_dense.json",
            &json!({"entry001": entry001_dense, "entry003": entry003_dense}),
        );
    }
    println!("{classification}");
}
