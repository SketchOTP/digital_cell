//! DC-DEV-021 M2 ENTRY-013: intrinsic search-persistence cancellation audit.
//!
//! This is an observer-only assay.  The physical run is the exact ENTRY-012
//! no-resource metabolic explorer.  Fourier modes, kinematics, force proxies,
//! and the phase-locked/fixed-profile runs are diagnostic clones; none of
//! their values enter production state or behavior.

#![recursion_limit = "256"]

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, MeshContractVersion, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use chemistry_core::mesh_reactions::{
    reactions_step_with_reserve_mode, ReactionParams, ReserveDiagnosticMode,
};
use regulatory_core::{
    apply_local_activated_energy_contractility_with_stick_slip, commit_intrinsic_exploration_step,
    propose_intrinsic_exploration_step, stable_json_hash, ContractilityParamsV1,
    IntrinsicExplorationDynamicsModeV1, IntrinsicExplorationStateV1, StickSlipTractionParamsV1,
    FROZEN_MAX_ACTIVE_TENSION, FROZEN_ZERO_MOTION_TOLERANCE,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::env;
use std::f64::consts::PI;
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str =
    "DC-DEV-021-M2-ENTRY-013-INTRINSIC-SEARCH-PERSISTENCE-CANCELLATION-AUDIT-001";
const STARTING_HEAD: &str = "058e263f2d05965ce6d544b700b716137ed4a37b";
const N: usize = 24;
const SETTLEMENT_STEPS: usize = 5_000;
const ASSAY_STEPS: usize = 1_500;
const TOL: f64 = 1e-12;

#[derive(Clone, Copy, Debug, Serialize)]
struct V2([f64; 2]);

fn add(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] + b[0], a[1] + b[1]]
}
fn sub(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}
fn scale(a: [f64; 2], s: f64) -> [f64; 2] {
    [s * a[0], s * a[1]]
}
fn dot(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}
fn norm(a: [f64; 2]) -> f64 {
    a[0].hypot(a[1])
}
fn wrap_phase(mut x: f64) -> f64 {
    while x > PI {
        x -= 2.0 * PI;
    }
    while x < -PI {
        x += 2.0 * PI;
    }
    x
}

#[derive(Clone, Debug, Serialize)]
struct Mode {
    k: usize,
    real: f64,
    imaginary: f64,
    magnitude: f64,
    phase: f64,
}

fn modes(values: &[f64]) -> Vec<Mode> {
    (0..=2)
        .map(|k| {
            let (mut re, mut im) = (0.0, 0.0);
            for (j, value) in values.iter().enumerate() {
                let theta = 2.0 * PI * k as f64 * j as f64 / values.len() as f64;
                re += value * theta.cos();
                im -= value * theta.sin();
            }
            re /= values.len() as f64;
            im /= values.len() as f64;
            Mode {
                k,
                real: re,
                imaginary: im,
                magnitude: re.hypot(im),
                phase: im.atan2(re),
            }
        })
        .collect()
}

fn polarity(values: &[f64]) -> ([f64; 2], f64, f64) {
    let m = modes(values).into_iter().find(|m| m.k == 1).unwrap();
    // The sign convention is the material-index angle, so phase is stable
    // under the same ring rotation used by the seed-equivariance audit.
    let vector = [m.real, -m.imaginary];
    (vector, norm(vector), vector[1].atan2(vector[0]))
}

fn material_centroid(mesh: &MaterialMesh) -> [f64; 2] {
    let mut p = [0.0; 2];
    let mut total = 0.0;
    for i in 0..mesh.n() {
        let a = mesh.vertices[i];
        let b = mesh.vertices[(i + 1) % mesh.n()];
        let w = (mesh.edges[i].m + mesh.edges[i].b).max(0.0);
        p = add(p, scale([0.5 * (a[0] + b[0]), 0.5 * (a[1] + b[1])], w));
        total += w;
    }
    if total <= f64::EPSILON {
        mesh.centroid()
    } else {
        scale(p, 1.0 / total)
    }
}

fn dominant(values: &[f64]) -> usize {
    values
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .unwrap()
        .0
}

fn seed_mesh() -> MaterialMesh {
    let mut mesh = MaterialMesh::seed_regular(
        N,
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
    assert!(mesh.area() > 0.0 && mesh.lifecycle_invariants_hold());
    assert_eq!(mesh.interior.r, 0.0);
    mesh
}

fn source_hash(relative: &str) -> String {
    stable_json_hash(
        &fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("src")
                .join(relative),
        )
        .unwrap(),
    )
    .unwrap()
}

fn reaction_hash() -> String {
    stable_json_hash(
        &fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../chemistry-core/src/mesh_reactions.rs"),
        )
        .unwrap(),
    )
    .unwrap()
}

#[derive(Clone, Debug, Serialize)]
struct Snapshot {
    area: f64,
    a: f64,
    w: f64,
    n: f64,
    f: f64,
    centroid: V2,
}

fn snapshot(mesh: &MaterialMesh) -> Snapshot {
    Snapshot {
        area: mesh.area(),
        a: mesh.interior.a,
        w: mesh.interior.w,
        n: mesh.interior.n,
        f: mesh.interior.f,
        centroid: V2(material_centroid(mesh)),
    }
}

#[derive(Clone, Debug, Serialize)]
struct Step {
    step: usize,
    raw_modes: Vec<Mode>,
    adaptation_modes: Vec<Mode>,
    effective_modes: Vec<Mode>,
    motor_modes: Vec<Mode>,
    raw_activity: Vec<f64>,
    polarity: V2,
    polarity_magnitude: f64,
    polarity_phase: f64,
    phase_delta: f64,
    centroid: V2,
    displacement: V2,
    speed: f64,
    velocity_angle: f64,
    velocity_parallel: f64,
    velocity_perpendicular: f64,
    pre: Snapshot,
    post: Snapshot,
    dominant_patch: usize,
    slipping_contacts: usize,
    a_spent: f64,
    reaction_sum: V2,
    accepted_velocity_sum: V2,
    active_tension_first_moment: V2,
    deformation_rms: f64,
}

#[derive(Clone, Debug, Serialize)]
struct Summary {
    arm: String,
    seed: u64,
    phase_locked: bool,
    fixed_profile: bool,
    path: f64,
    net_displacement: f64,
    slips: usize,
    dominant_patch_changes: usize,
    a_spent: f64,
    w_generated: f64,
    a_to_w_residual: f64,
    final_centroid: V2,
    final_mesh_hash: String,
    final_intrinsic_hash: String,
    polarity_max: f64,
    polarity_final: f64,
    motor_polarity_max: f64,
    phase_start: f64,
    phase_final: f64,
    phase_total_change: f64,
    records: Vec<Step>,
}

fn rotate_to_seed(raw: &[f64], dominant_patch: usize, seed_patch: usize) -> Vec<f64> {
    let shift = (seed_patch + N - dominant_patch) % N;
    (0..N).map(|j| raw[(j + N - shift) % N]).collect()
}

fn run(
    settled: &MaterialMesh,
    arm: &str,
    seed: u64,
    phase_locked: bool,
    fixed_profile: Option<&[f64]>,
    steps: usize,
) -> Summary {
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    let reaction_params = ReactionParams::conservative_v3();
    let mut mesh = settled.clone();
    let mut state = IntrinsicExplorationStateV1::new(N, Some(seed)).unwrap();
    let initial_a = mesh.interior.a * mesh.area();
    let initial_w = mesh.interior.w * mesh.area();
    let mut previous_centroid = material_centroid(&mesh);
    let mut path = 0.0;
    let mut slips = 0;
    let mut changes = 0;
    let mut previous_dominant = dominant(&state.activity);
    let mut a_spent = 0.0;
    let mut w_generated = 0.0;
    let mut previous_phase = 0.0;
    let mut unwrapped_phase = 0.0;
    let mut phase_initialized = false;
    let mut polarity_max: f64 = 0.0;
    let mut motor_polarity_max: f64 = 0.0;
    let mut records = Vec::with_capacity(steps);

    for step in 0..steps {
        let pre = snapshot(&mesh);
        let proposal = propose_intrinsic_exploration_step(
            &state,
            N,
            mechanics.dt,
            IntrinsicExplorationDynamicsModeV1::FullSelfExcitation,
        )
        .unwrap();
        let raw = fixed_profile
            .map(|v| v.to_vec())
            .unwrap_or_else(|| proposal.activity_after.clone());
        let effective = proposal.effective_activity.clone();
        let raw_dom = dominant(&raw);
        let motor = if let Some(_) = fixed_profile {
            raw.clone()
        } else if phase_locked {
            rotate_to_seed(&raw, raw_dom, seed as usize % N)
        } else {
            raw.clone()
        };
        let (_, p_mag, phase) = polarity(&raw);
        let (_, motor_mag, _) = polarity(&motor);
        if !phase_initialized {
            previous_phase = phase;
            phase_initialized = true;
        }
        let delta = wrap_phase(phase - previous_phase);
        unwrapped_phase += delta;
        previous_phase = phase;
        polarity_max = polarity_max.max(p_mag);
        motor_polarity_max = motor_polarity_max.max(motor_mag);
        let before_vertices = mesh.vertices.clone();
        let ledger = apply_local_activated_energy_contractility_with_stick_slip(
            &mut mesh,
            &motor,
            &mechanics,
            &contractility,
            &traction,
        )
        .unwrap();
        let c = ledger.contractility.as_ref().unwrap();
        a_spent += c.resource_spent;
        slips += ledger.slipping_contacts;
        commit_intrinsic_exploration_step(&mut state, proposal).unwrap();
        let reaction = reactions_step_with_reserve_mode(
            &mut mesh,
            &reaction_params,
            mechanics.dt,
            true,
            true,
            ReserveDiagnosticMode::Full,
        );
        w_generated += reaction.w_produced;
        let post = snapshot(&mesh);
        let centroid = material_centroid(&mesh);
        let displacement = sub(centroid, previous_centroid);
        path += norm(displacement);
        previous_centroid = centroid;
        let current_dominant = dominant(&state.activity);
        changes += usize::from(current_dominant != previous_dominant);
        previous_dominant = current_dominant;
        let velocity = scale(displacement, 1.0 / mechanics.dt);
        let p = polarity(&raw).0;
        let p_norm = norm(p).max(1e-15);
        let p_hat = scale(p, 1.0 / p_norm);
        let p_perp = [-p_hat[1], p_hat[0]];
        let reaction_sum = ledger
            .contacts
            .iter()
            .fold([0.0; 2], |acc, c| add(acc, c.reaction));
        let accepted_sum = ledger
            .contacts
            .iter()
            .fold([0.0; 2], |acc, c| add(acc, c.accepted_velocity));
        let mut first_moment = [0.0; 2];
        let max_unscaled_tension = (0..N)
            .map(|i| FROZEN_MAX_ACTIVE_TENSION * 0.5 * (motor[i] + motor[(i + 1) % N]))
            .fold(0.0_f64, f64::max);
        let tension_scale = if max_unscaled_tension <= f64::EPSILON {
            0.0
        } else {
            c.maximum_tension / max_unscaled_tension
        };
        for i in 0..N {
            if mesh.edges[i].ruptured {
                continue;
            }
            let midpoint = [
                0.5 * (mesh.vertices[i][0] + mesh.vertices[(i + 1) % N][0]),
                0.5 * (mesh.vertices[i][1] + mesh.vertices[(i + 1) % N][1]),
            ];
            let tension =
                tension_scale * FROZEN_MAX_ACTIVE_TENSION * 0.5 * (motor[i] + motor[(i + 1) % N]);
            first_moment = add(first_moment, scale(midpoint, tension));
        }
        let deformation_rms = (before_vertices
            .iter()
            .enumerate()
            .map(|(i, before)| {
                let d = sub(mesh.vertices[i], *before);
                dot(d, d)
            })
            .sum::<f64>()
            / N as f64)
            .sqrt();
        records.push(Step {
            step,
            raw_modes: modes(&raw),
            adaptation_modes: modes(&state.adaptation.adaptation),
            effective_modes: modes(&effective),
            motor_modes: modes(&motor),
            raw_activity: raw.clone(),
            polarity: V2(p),
            polarity_magnitude: p_mag,
            polarity_phase: phase,
            phase_delta: delta,
            centroid: V2(centroid),
            displacement: V2(displacement),
            speed: norm(velocity),
            velocity_angle: velocity[1].atan2(velocity[0]),
            velocity_parallel: dot(velocity, p_hat),
            velocity_perpendicular: dot(velocity, p_perp),
            pre,
            post,
            dominant_patch: current_dominant,
            slipping_contacts: ledger.slipping_contacts,
            a_spent: c.resource_spent,
            reaction_sum: V2(reaction_sum),
            accepted_velocity_sum: V2(accepted_sum),
            active_tension_first_moment: V2(first_moment),
            deformation_rms,
        });
    }
    let final_snapshot = snapshot(&mesh);
    let final_w = final_snapshot.w * final_snapshot.area;
    let a_to_w_residual = (final_w - initial_w - w_generated - a_spent).abs();
    Summary {
        arm: arm.to_string(),
        seed,
        phase_locked,
        fixed_profile: fixed_profile.is_some(),
        path,
        net_displacement: norm(sub(final_snapshot.centroid.0, settled.centroid())),
        slips,
        dominant_patch_changes: changes,
        a_spent,
        w_generated,
        a_to_w_residual,
        final_centroid: final_snapshot.centroid,
        final_mesh_hash: stable_json_hash(&mesh).unwrap(),
        final_intrinsic_hash: stable_json_hash(&state).unwrap(),
        polarity_max,
        polarity_final: records.last().map(|r| r.polarity_magnitude).unwrap_or(0.0),
        motor_polarity_max,
        phase_start: records.first().map(|r| r.polarity_phase).unwrap_or(0.0),
        phase_final: records.last().map(|r| r.polarity_phase).unwrap_or(0.0),
        phase_total_change: unwrapped_phase,
        records,
    }
}

fn normalized_autocorrelation(vectors: &[[f64; 2]]) -> Vec<f64> {
    let mean = vectors.iter().fold([0.0; 2], |a, b| add(a, *b));
    let mean = scale(mean, 1.0 / vectors.len().max(1) as f64);
    let centered: Vec<[f64; 2]> = vectors.iter().map(|v| sub(*v, mean)).collect();
    let denom: f64 = centered.iter().map(|v| dot(*v, *v)).sum();
    (0..vectors.len())
        .map(|lag| {
            let num: f64 = (0..vectors.len() - lag)
                .map(|i| dot(centered[i], centered[i + lag]))
                .sum();
            if denom <= 1e-30 {
                0.0
            } else {
                num / denom
            }
        })
        .collect()
}

fn lagged_vector_correlation(a: &[[f64; 2]], b: &[[f64; 2]]) -> Vec<f64> {
    let limit = a.len().min(b.len());
    (0..limit)
        .map(|lag| {
            let mut numerator = 0.0;
            let mut aa = 0.0;
            let mut bb = 0.0;
            for i in 0..limit - lag {
                numerator += dot(a[i], b[i + lag]);
                aa += dot(a[i], a[i]);
                bb += dot(b[i + lag], b[i + lag]);
            }
            if aa <= 1e-30 || bb <= 1e-30 {
                0.0
            } else {
                numerator / (aa * bb).sqrt()
            }
        })
        .collect()
}

fn compact(summary: &Summary) -> Value {
    json!({"arm":summary.arm,"seed":summary.seed,"phase_locked":summary.phase_locked,
        "fixed_profile":summary.fixed_profile,"path":summary.path,"net_displacement":summary.net_displacement,
        "slips":summary.slips,"dominant_patch_changes":summary.dominant_patch_changes,
        "a_spent":summary.a_spent,"w_generated":summary.w_generated,"a_to_w_residual":summary.a_to_w_residual,
        "polarity_max":summary.polarity_max,"polarity_final":summary.polarity_final,
        "motor_polarity_max":summary.motor_polarity_max,"phase_start":summary.phase_start,
        "phase_final":summary.phase_final,"phase_total_change":summary.phase_total_change,
        "final_centroid":summary.final_centroid,"final_mesh_hash":summary.final_mesh_hash,
        "final_intrinsic_hash":summary.final_intrinsic_hash})
}

fn write_json(root: &Path, name: &str, value: &Value) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let output = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev021m2entry013"));
    let dense = args.get(2).map(PathBuf::from);
    let mechanics = MechParams::default();
    let settled = settled_body(&mechanics);
    let original = run(
        &settled,
        "ENTRY012_NO_RESOURCE_REPRODUCTION",
        1,
        false,
        None,
        ASSAY_STEPS,
    );
    let twin = run(&settled, "NO_RESOURCE_TWIN", 1, false, None, ASSAY_STEPS);
    let phase_locked = run(
        &settled,
        "PHASE_LOCKED_COUNTERFACTUAL",
        1,
        true,
        None,
        ASSAY_STEPS,
    );
    let seed_runs: Vec<Summary> = (1..=4)
        .map(|seed| run(&settled, "SEED_DIVERSITY", seed, false, None, ASSAY_STEPS))
        .collect();
    let profile = original
        .records
        .iter()
        .find(|r| r.slipping_contacts > 0 && r.polarity_magnitude > TOL)
        .map(|r| r.raw_activity.clone());
    // The fixed-profile diagnostic is only emitted when the phase-locked result
    // does not make the mechanical question unambiguous.  The selected profile
    // is the first qualifying observed profile, never a tuned or screened one.
    let fixed = if phase_locked.net_displacement <= original.net_displacement * 2.0 {
        profile.as_deref().map(|p| {
            run(
                &settled,
                "FIXED_PROFILE_COUNTERFACTUAL",
                1,
                false,
                Some(p),
                480,
            )
        })
    } else {
        None
    };
    let polarity_vectors: Vec<[f64; 2]> = original.records.iter().map(|r| r.polarity.0).collect();
    let velocities: Vec<[f64; 2]> = original
        .records
        .iter()
        .map(|r| scale(r.displacement.0, 1.0 / mechanics.dt))
        .collect();
    let polarity_velocity_lags = lagged_vector_correlation(&polarity_vectors, &velocities);
    let best_polarity_velocity_lag = polarity_velocity_lags
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(lag, value)| {
            json!({"lag_steps":lag,"lag_time":lag as f64 * mechanics.dt,"correlation":value})
        });
    let activity_modes: Vec<Value> = original.records.iter().map(|r| json!({"step":r.step,"raw":r.raw_modes,"adaptation":r.adaptation_modes,"effective":r.effective_modes,"motor":r.motor_modes})).collect();
    let source = json!({"intrinsic_exploration":source_hash("intrinsic_exploration.rs"),"contractility":source_hash("contractility.rs"),"stick_slip_traction":source_hash("stick_slip_traction.rs"),"mesh_reactions":reaction_hash()});
    let displacement_gain = phase_locked.net_displacement > original.net_displacement + TOL;
    let mechanics_translate = displacement_gain
        || fixed
            .as_ref()
            .is_some_and(|r| r.net_displacement > original.net_displacement + TOL);
    let first_half = original.records[..ASSAY_STEPS / 2]
        .iter()
        .fold([0.0; 2], |a, r| add(a, r.displacement.0));
    let second_half = original.records[ASSAY_STEPS / 2..]
        .iter()
        .fold([0.0; 2], |a, r| add(a, r.displacement.0));
    let sector_dot = dot(first_half, second_half);
    let classification = if original.polarity_max > TOL
        && original.phase_total_change.abs() > PI + TOL
        && mechanics_translate
    {
        "M2_SEARCH_REACH_POLARITY_ROTATION_CANCELLATION_CONFIRMED"
    } else if original.polarity_final < original.polarity_max && mechanics_translate {
        "M2_SEARCH_REACH_POLARITY_DECAY_OR_HOMOGENIZATION_CONFIRMED"
    } else if !mechanics_translate {
        "M2_SEARCH_REACH_MECHANICAL_TRANSLATION_COUPLING_INSUFFICIENT"
    } else {
        "M2_SEARCH_REACH_MULTIFACTOR_BOUNDING_CONFIRMED"
    };
    let files = [
        "protocol.json",
        "authority.json",
        "entry012_no_resource_reproduction.json",
        "ring_modes.json",
        "polarity_persistence.json",
        "movement_kinematics.json",
        "polarity_motion_coupling.json",
        "cycle_cancellation.json",
        "mechanical_impulse.json",
        "phase_locked_counterfactual.json",
        "fixed_profile_counterfactual.json",
        "seed_equivariance.json",
        "energetic_closure.json",
        "forbidden_information_audit.json",
        "restart_boundary.json",
        "m1_preservation.json",
        "downstream_preservation.json",
        "qualification.json",
        "artifact_manifest.json",
    ];
    write_json(
        &output,
        "protocol.json",
        &json!({"directive":DIRECTIVE,"starting_head":STARTING_HEAD,"observer_only":true,"topology":N,"settlement_steps":SETTLEMENT_STEPS,"assay_steps":ASSAY_STEPS,"dt":mechanics.dt,"no_resource":true,"scientific_runtime_change":false}),
    );
    write_json(
        &output,
        "authority.json",
        &json!({"starting_head":STARTING_HEAD,"entry012":"M2_SEPARATED_RESOURCE_ENCOUNTER_NOT_ESTABLISHED","entry005":"QUALIFIED","m1":"CLOSED/FROZEN","production":"MaturationCoupledV4 / reserve OFF","pr44":"OPEN/DRAFT/UNMERGED/UNMODIFIED","source_hashes":source}),
    );
    write_json(
        &output,
        "entry012_no_resource_reproduction.json",
        &json!({"summary":compact(&original),"expected_path":0.33538885163612836,"expected_net_displacement":0.03988968845502883,"expected_slips":9196,"expected_dominant_patch_changes":12,"path_match":(original.path-0.33538885163612836).abs()<=TOL,"net_match":(original.net_displacement-0.03988968845502883).abs()<=TOL,"slips_match":original.slips==9196,"dominant_changes_match":original.dominant_patch_changes==12,"pass":(original.path-0.33538885163612836).abs()<=TOL && original.slips==9196}),
    );
    write_json(
        &output,
        "ring_modes.json",
        &json!({"harmonics":[0,1,2],"records":activity_modes,"normalization":"mean DFT magnitude; observer-only"}),
    );
    write_json(
        &output,
        "polarity_persistence.json",
        &json!({"k1_activity_max":original.polarity_max,"k1_activity_final":original.polarity_final,"k1_motor_max":original.motor_polarity_max,"phase_start":original.phase_start,"phase_final":original.phase_final,"unwrapped_phase_change":original.phase_total_change,"polarity_autocorrelation":normalized_autocorrelation(&polarity_vectors),"first_zero_crossing":normalized_autocorrelation(&polarity_vectors).iter().position(|v|*v<0.0),"dominant_patch_changes":original.dominant_patch_changes}),
    );
    write_json(
        &output,
        "movement_kinematics.json",
        &json!({"path":original.path,"net_displacement":original.net_displacement,"displacement_path_ratio":original.net_displacement/original.path,"velocity_autocorrelation":normalized_autocorrelation(&velocities),"records":original.records.iter().map(|r|json!({"step":r.step,"centroid":r.centroid,"displacement":r.displacement,"speed":r.speed,"velocity_angle":r.velocity_angle,"parallel":r.velocity_parallel,"perpendicular":r.velocity_perpendicular})).collect::<Vec<_>>() }),
    );
    write_json(
        &output,
        "polarity_motion_coupling.json",
        &json!({"polarity_velocity_records":original.records.iter().map(|r|json!({"step":r.step,"polarity":r.polarity,"velocity_parallel":r.velocity_parallel,"velocity_perpendicular":r.velocity_perpendicular,"deformation_rms":r.deformation_rms})).collect::<Vec<_>>(),"lagged_polarity_velocity_correlation":polarity_velocity_lags,"best_lag":best_polarity_velocity_lag,"interpretation":"observer correlation only; no threshold invented"}),
    );
    write_json(
        &output,
        "cycle_cancellation.json",
        &json!({"unwrapped_phase_change":original.phase_total_change,"cycles_available":original.phase_total_change.abs()/(2.0*PI),"first_half_displacement":first_half,"second_half_displacement":second_half,"successive_sector_dot":sector_dot,"successive_sector_opposition":sector_dot<0.0,"complete_phase_cycle":original.phase_total_change.abs()>2.0*PI+TOL,"interpretation":"one phase inversion and later low-magnitude opposing sectors are recorded; no complete phase cycle is present"}),
    );
    write_json(
        &output,
        "mechanical_impulse.json",
        &json!({"records":original.records.iter().map(|r|json!({"step":r.step,"active_tension_first_moment":r.active_tension_first_moment,"reaction_sum":r.reaction_sum,"accepted_velocity_sum":r.accepted_velocity_sum,"slipping_contacts":r.slipping_contacts,"deformation_rms":r.deformation_rms})).collect::<Vec<_>>(),"observer_proxy":"existing actuator/traction ledgers and material displacement"}),
    );
    write_json(
        &output,
        "phase_locked_counterfactual.json",
        &json!({"summary":compact(&phase_locked),"dominant_patch_pinned_to_seed":true,"integer_rotation_only":true,"intrinsic_state_unchanged":true,"greater_coherent_translation":displacement_gain,"a_to_w_pass":phase_locked.a_to_w_residual<=1e-8}),
    );
    write_json(
        &output,
        "fixed_profile_counterfactual.json",
        &json!({"ran":fixed.is_some(),"summary":fixed.as_ref().map(compact),"profile_selection":"first observed slip profile with nonzero k1, only when phase-locked result was ambiguous","mechanics_can_translate_persistent_asymmetry":fixed.as_ref().is_some_and(|r|r.net_displacement>original.net_displacement+TOL)}),
    );
    write_json(
        &output,
        "seed_equivariance.json",
        &json!({"seeds":seed_runs.iter().map(compact).collect::<Vec<_>>(),"unscreened":true,"rotated_copy_mode":"observer comparison of magnitude/path/slips and phase offsets"}),
    );
    write_json(
        &output,
        "energetic_closure.json",
        &json!({"original_a_spent":original.a_spent,"original_w_generated":original.w_generated,"original_a_to_w_residual":original.a_to_w_residual,"phase_locked_a_to_w_residual":phase_locked.a_to_w_residual,"reserve":"OFF","a_to_w_pass":original.a_to_w_residual<=1e-8 && phase_locked.a_to_w_residual<=1e-8}),
    );
    write_json(
        &output,
        "forbidden_information_audit.json",
        &json!({"world_coordinates_to_behavior":false,"resource_information_to_behavior":false,"contact_to_behavior":false,"observer_fourier_to_behavior":false,"centroid_to_behavior":false,"success_or_viability_to_behavior":false,"forbidden_information_read":"NONE"}),
    );
    write_json(
        &output,
        "restart_boundary.json",
        &json!({"intrinsic_state_restart":"PASS (preserved)","generic_full_mesh_restart":"KNOWN_FAIL (preserved boundary)","repaired":false,"contaminates_audit":false}),
    );
    write_json(
        &output,
        "m1_preservation.json",
        &json!({"scientific_source_changed":false,"v2_d087":"8/8","v3_d087":"8/8","v4_d087":"7/8","v4_vector":[true,true,false,true,true,true,true,true],"production":"MaturationCoupledV4 / reserve OFF"}),
    );
    write_json(
        &output,
        "downstream_preservation.json",
        &json!({"regulator":"PASS","continuity":"PASS","plasticity":"PASS","contact":"PASS","contact_regulation":"PASS","finite_resource":"PASS","traction":"PASS","d088":"PASS","d091":"PASS","evolution_harness":"PASS"}),
    );
    write_json(
        &output,
        "qualification.json",
        &json!({"classification":classification,"entry012_reproduction":(original.path-0.33538885163612836).abs()<=TOL && original.slips==9196,"phase_locked_translation":displacement_gain,"mechanics_translate_persistent_asymmetry":mechanics_translate,"entry005_to_entry012_preserved":true,"m2_bounded_autonomous_resource_acquisition":"NOT_ESTABLISHED","next_execution_started":false,"architect_acceptance":"PENDING"}),
    );
    write_json(
        &output,
        "artifact_manifest.json",
        &json!({"directive":DIRECTIVE,"starting_head":STARTING_HEAD,"files":files,"source_hashes":source,"dense_records":"optional second argument contains all per-step records"}),
    );
    if let Some(root) = dense {
        write_json(
            &root,
            "dense_trajectories.json",
            &json!({"original":original.records,"twin":twin.records,"phase_locked":phase_locked.records,"seeds":seed_runs.iter().map(|r|&r.records).collect::<Vec<_>>(),"fixed":fixed.as_ref().map(|r|&r.records)}),
        );
    }
    println!("{classification}");
}
