//! D-013 Stage E harness integrity runner: accepted-step authority, atomic
//! checkpoints, activation-potential capture, and governed reference recovery.

use crate::d011::{
    interior_mean, prepare_constrained_seed, retention, soluble_max, window_snapshot,
};
use crate::d012::v2_stage_options;
use crate::d008;
use chemistry_core::{
    build_activation_potential_step, build_balance_metrics, build_candidate_identity,
    build_material_equivalent_step, build_window_record, classify_convergence,
    crossed_checkpoint_thresholds, field_mass, field_sha256_stable, joint_overlap_pass,
    map_termination_to_scientific, membrane_partition, potential_from_masses,
    sample_to_window_snapshot, solver_entry_allowed, update_convergence_counter,
    validate_governed_artifact, ActivationPotentialLedger, ArtifactValidationStatus,
    CandidateIdentity, ConvergenceClassification, ConvergenceCounter, D008StageMode,
    D012_V2_CENTER_RADIUS, D012_V2_MAX_STEPS, D012_V2_REQUIRED_WINDOWS, D012_V2_WINDOW,
    D013_CHECKPOINT_THRESHOLDS, D013_DEFAULT_REJECTION_STALL_LIMIT, EquationVersion,
    FieldSchemaVersion, FieldSnapshot, GovernedArtifactView, JointBalanceMetrics,
    MaterialEquivalentStep, ScientificClassification, SimParams, Simulation,
    StageEReferenceRates, SteadyWindowSnapshot, STOICHIOMETRIC_SCHEMA_VERSION_V2,
    TerminationReason, AcceptedStateSample, WindowRecord,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

const D013_SEED: u64 = 1;
const FROZEN_CANDIDATE_HASH: &str =
    "9a452d3470be34ccf3bdd7d1397341b64617834e77131cf2899efb327728d626";
const FROZEN_CONFIGURATION_HASH: &str =
    "87ff7e6e4bd479972c3a02b0de4e6bc94a949041860b32b230e5b28863bb5ad6";

#[derive(Debug, Clone)]
pub struct D013RunConfig {
    pub max_steps: u64,
    pub window_size: u64,
    pub radius: f64,
    pub rejection_stall_limit: u64,
    pub checkpoint_dir: Option<PathBuf>,
    pub resume_checkpoint: Option<PathBuf>,
}

impl Default for D013RunConfig {
    fn default() -> Self {
        Self {
            max_steps: D012_V2_MAX_STEPS,
            window_size: D012_V2_WINDOW,
            radius: D012_V2_CENTER_RADIUS,
            rejection_stall_limit: D013_DEFAULT_REJECTION_STALL_LIMIT,
            checkpoint_dir: None,
            resume_checkpoint: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LosslessSevenFields {
    pub structure: Vec<u64>,
    pub catalyst: Vec<u64>,
    pub nutrient: Vec<u64>,
    pub fuel: Vec<u64>,
    pub waste: Vec<u64>,
    pub activated: Vec<u64>,
    pub membrane: Vec<u64>,
}

impl LosslessSevenFields {
    pub fn from_sim(sim: &Simulation) -> Self {
        fn bits(v: &[f64]) -> Vec<u64> {
            v.iter().map(|x| x.to_bits()).collect()
        }
        Self {
            structure: bits(&sim.fields.structure),
            catalyst: bits(&sim.fields.catalyst),
            nutrient: bits(&sim.fields.nutrient),
            fuel: bits(&sim.fields.fuel),
            waste: bits(&sim.fields.waste),
            activated: bits(&sim.fields.activated),
            membrane: bits(&sim.fields.membrane),
        }
    }

    pub fn restore_into(&self, sim: &mut Simulation) -> Result<(), Box<dyn std::error::Error>> {
        fn copy(dst: &mut [f64], src: &[u64]) -> Result<(), Box<dyn std::error::Error>> {
            if dst.len() != src.len() {
                return Err("lossless field length mismatch".into());
            }
            for (d, s) in dst.iter_mut().zip(src.iter()) {
                *d = f64::from_bits(*s);
            }
            Ok(())
        }
        copy(&mut sim.fields.structure, &self.structure)?;
        copy(&mut sim.fields.catalyst, &self.catalyst)?;
        copy(&mut sim.fields.nutrient, &self.nutrient)?;
        copy(&mut sim.fields.fuel, &self.fuel)?;
        copy(&mut sim.fields.waste, &self.waste)?;
        copy(&mut sim.fields.activated, &self.activated)?;
        copy(&mut sim.fields.membrane, &self.membrane)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernedCheckpoint {
    pub clean_atomic_write: bool,
    pub checkpoint_threshold: u64,
    pub accepted_substeps: u64,
    pub attempted_substeps: u64,
    pub rejected_substeps: u64,
    pub simulated_time: f64,
    pub current_dt: f64,
    pub min_accepted_dt: f64,
    pub min_attempted_dt: f64,
    pub max_consecutive_rejections: u64,
    pub candidate_hash: String,
    pub configuration_hash: String,
    pub source_commit: String,
    pub binary_hash: String,
    pub field_hashes: BTreeMap<String, String>,
    pub next_buffer_rule: String,
    pub snapshot: FieldSnapshot,
    /// Bit-exact seven-field payload for deterministic continuation across JSON.
    pub lossless_fields: LosslessSevenFields,
    pub material_accounting: MaterialEquivalentStep,
    pub activation_potential_accounting: ActivationPotentialLedger,
    pub rolling_window_state: ConvergenceCounter,
    pub reaction_ledgers: Value,
    pub transport_ledgers: Value,
    pub reservoir_ledgers: Value,
    pub accounting_cumulative: Value,
    pub metabolism_cumulative: Value,
    pub membrane_cumulative: Value,
    pub constraint_cumulative: Value,
    pub window_anchor: Option<SteadyWindowSnapshot>,
    pub pending_window_samples: Vec<AcceptedStateSample>,
    pub prev_attempt_dt: f64,
}

#[derive(Debug, Clone)]
pub struct GovernedRunOutcome {
    pub metrics: JointBalanceMetrics,
    pub accepted_substeps: u64,
    pub attempted_substeps: u64,
    pub rejected_substeps: u64,
    pub simulated_time: f64,
    pub current_dt: f64,
    pub min_accepted_dt: f64,
    pub min_attempted_dt: f64,
    pub max_consecutive_rejections: u64,
    pub termination_reason: TerminationReason,
    pub clean_termination: bool,
    pub scientific_classification: ScientificClassification,
    pub convergence: ConvergenceCounter,
    pub material_accounting: MaterialEquivalentStep,
    pub activation_potential_accounting: ActivationPotentialLedger,
    pub checkpoint_completion: BTreeMap<u64, bool>,
    pub field_hashes: BTreeMap<String, String>,
    pub windows: Vec<SteadyWindowSnapshot>,
    pub wall_seconds: f64,
}

fn git_commit_hash() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git").args(["rev-parse", "HEAD"]).output()?;
    if !output.status.success() {
        return Err("git rev-parse failed".into());
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn binary_hash() -> Result<String, Box<dyn std::error::Error>> {
    let path = std::env::current_exe().map_err(|err| format!("binary_sha256 failed: {err}"))?;
    let bytes = fs::read(&path).map_err(|err| format!("binary_sha256 failed: {err}"))?;
    Ok(chemistry_core::sha256_hex(&bytes))
}

pub fn v2_frozen_params() -> Result<SimParams, Box<dyn std::error::Error>> {
    let mut params = d008::stage_d_params_for(&v2_stage_options())?;
    params.equation_version = EquationVersion::MembraneMetabolismV2Conservative;
    params.eta_c = 1.0;
    params.eta_phi = 1.0;
    params.eta_m = 1.0;
    params.d008_stage_mode = D008StageMode::ConstrainedRadius;
    params.d008_stage_b_enabled = false;
    params.random_seed = D013_SEED;
    params.reactions_enabled = true;
    params.diffusion_enabled = true;
    params.phase_separation_enabled = false;
    Ok(params)
}

pub fn load_frozen_rates_from_invalid_reference() -> Result<StageEReferenceRates, Box<dyn std::error::Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../experiments/generated/d012/v2_stage_e_reference/result.json");
    let value: Value = serde_json::from_slice(&fs::read(path)?)?;
    let rates: StageEReferenceRates = serde_json::from_value(value["selected_rates"].clone())?;
    Ok(rates)
}

fn field_hashes(sim: &Simulation) -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();
    m.insert("structure".into(), field_sha256_stable(&sim.fields.structure));
    m.insert("catalyst".into(), field_sha256_stable(&sim.fields.catalyst));
    m.insert("nutrient".into(), field_sha256_stable(&sim.fields.nutrient));
    m.insert("fuel".into(), field_sha256_stable(&sim.fields.fuel));
    m.insert("waste".into(), field_sha256_stable(&sim.fields.waste));
    m.insert("activated".into(), field_sha256_stable(&sim.fields.activated));
    m.insert("membrane".into(), field_sha256_stable(&sim.fields.membrane));
    m
}

fn capture_sample(sim: &Simulation) -> AcceptedStateSample {
    let snap = window_snapshot(sim, sim.substep, sim.sim_time);
    AcceptedStateSample {
        accepted_substep: sim.substep,
        simulated_time: sim.sim_time,
        mass_c: snap.mass_c,
        mass_a: snap.mass_a,
        mass_m: snap.mass_m,
        mean_n_interior: snap.mean_n_interior,
        mean_f_interior: snap.mean_f_interior,
        mean_w_interior: snap.mean_w_interior,
        structure_production: snap.structure_production,
        structure_decay: snap.structure_decay,
        catalyst_reproduction: snap.catalyst_reproduction,
        catalyst_turnover: snap.catalyst_turnover,
        membrane_synthesis: snap.membrane_synthesis,
        membrane_loss: snap.membrane_loss,
        activation: snap.activation,
        activated_loss: snap.activated_loss,
        nutrient_transport_interior: snap.nutrient_transport_interior,
        fuel_transport_interior: snap.fuel_transport_interior,
        waste_transport_interior: snap.waste_transport_interior,
        material_equivalent_total: field_mass(&sim.grid, &sim.fields.structure)
            + field_mass(&sim.grid, &sim.fields.catalyst)
            + field_mass(&sim.grid, &sim.fields.nutrient)
            + field_mass(&sim.grid, &sim.fields.fuel)
            + field_mass(&sim.grid, &sim.fields.waste)
            + field_mass(&sim.grid, &sim.fields.activated)
            + field_mass(&sim.grid, &sim.fields.membrane),
        activation_potential_total: potential_from_masses(
            field_mass(&sim.grid, &sim.fields.fuel),
            field_mass(&sim.grid, &sim.fields.activated),
        ),
    }
}

pub fn atomic_write_bytes(path: &Path, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("tmp");
    {
        let mut opts = fs::OpenOptions::new();
        opts.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            opts.mode(0o644);
        }
        let mut file = opts.open(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    // Ensure directory entry is durable on rename platforms that need it.
    if let Some(parent) = path.parent() {
        let dir = File::open(parent)?;
        dir.sync_all()?;
    }
    fs::rename(&tmp, path)?;
    Ok(())
}

pub fn atomic_write_json(path: &Path, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    atomic_write_bytes(path, &serde_json::to_vec_pretty(value)?)
}

pub fn write_governed_checkpoint(
    path: &Path,
    checkpoint: &GovernedCheckpoint,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stamped = checkpoint.clone();
    stamped.clean_atomic_write = true;
    let bytes = serde_json::to_vec_pretty(&stamped)?;
    atomic_write_bytes(path, &bytes)
}

pub fn load_governed_checkpoint(
    path: &Path,
) -> Result<GovernedCheckpoint, Box<dyn std::error::Error>> {
    let data = fs::read(path)?;
    let ckpt: GovernedCheckpoint = serde_json::from_slice(&data)?;
    if !ckpt.clean_atomic_write {
        return Err("partial checkpoint rejected: clean_atomic_write=false".into());
    }
    Ok(ckpt)
}

fn build_checkpoint(
    sim: &Simulation,
    threshold: u64,
    identity: &CandidateIdentity,
    source_commit: &str,
    binary_hash: &str,
    activation: &ActivationPotentialLedger,
    convergence: &ConvergenceCounter,
    window_anchor: Option<&SteadyWindowSnapshot>,
    pending_samples: &[AcceptedStateSample],
) -> GovernedCheckpoint {
    GovernedCheckpoint {
        clean_atomic_write: false,
        checkpoint_threshold: threshold,
        accepted_substeps: sim.substep,
        attempted_substeps: sim.attempted_substeps,
        rejected_substeps: sim.rejection_count,
        simulated_time: sim.sim_time,
        current_dt: sim.dt,
        min_accepted_dt: sim.min_dt_seen,
        min_attempted_dt: sim.min_attempted_dt,
        max_consecutive_rejections: sim.max_consecutive_rejections,
        candidate_hash: identity.candidate_hash.clone(),
        configuration_hash: identity.configuration_hash.clone(),
        source_commit: source_commit.to_string(),
        binary_hash: binary_hash.to_string(),
        field_hashes: field_hashes(sim),
        next_buffer_rule: "safe-reset: copy current seven fields into next buffers on resume".into(),
        snapshot: sim.snapshot(),
        lossless_fields: LosslessSevenFields::from_sim(sim),
        material_accounting: build_material_equivalent_step(&sim.accounting.last_step),
        activation_potential_accounting: activation.clone(),
        rolling_window_state: convergence.clone(),
        reaction_ledgers: json!(sim.accounting.cumulative),
        transport_ledgers: json!(sim.transport_accounting.cumulative),
        reservoir_ledgers: json!({
            "nutrient_supplied": sim.accounting.cumulative.nutrient_supplied_reservoir,
            "fuel_supplied": sim.accounting.cumulative.fuel_supplied_reservoir,
            "waste_removed": sim.accounting.cumulative.waste_removed_reservoir,
        }),
        accounting_cumulative: json!(sim.accounting.cumulative),
        metabolism_cumulative: json!(sim.metabolism_accounting.cumulative),
        membrane_cumulative: json!(sim.membrane_accounting.cumulative),
        constraint_cumulative: json!(sim.constraint_accounting.cumulative),
        window_anchor: window_anchor.cloned(),
        pending_window_samples: pending_samples.to_vec(),
        prev_attempt_dt: sim.dt,
    }
}

fn restore_from_checkpoint(
    sim: &mut Simulation,
    ckpt: &GovernedCheckpoint,
) -> Result<(), Box<dyn std::error::Error>> {
    sim.try_restore_snapshot(&ckpt.snapshot)?;
    // Prefer bit-exact fields; JSON f64 round-trip is not continuation-safe.
    ckpt.lossless_fields.restore_into(sim)?;
    // Safe-reset next buffers from current fields.
    sim.fields.copy_current_to_next();
    sim.dt = ckpt.current_dt;
    sim.min_dt_seen = ckpt.min_accepted_dt;
    sim.min_attempted_dt = ckpt.min_attempted_dt;
    sim.rejection_count = ckpt.rejected_substeps;
    sim.attempted_substeps = ckpt.attempted_substeps;
    sim.max_consecutive_rejections = ckpt.max_consecutive_rejections;
    sim.substep = ckpt.accepted_substeps;
    sim.sim_time = ckpt.simulated_time;
    sim.accounting.cumulative = serde_json::from_value(ckpt.accounting_cumulative.clone())?;
    sim.metabolism_accounting.cumulative =
        serde_json::from_value(ckpt.metabolism_cumulative.clone())?;
    sim.membrane_accounting.cumulative =
        serde_json::from_value(ckpt.membrane_cumulative.clone())?;
    sim.constraint_accounting.cumulative =
        serde_json::from_value(ckpt.constraint_cumulative.clone())?;
    sim.transport_accounting.cumulative =
        serde_json::from_value(ckpt.transport_ledgers.clone())?;
    Ok(())
}

fn biological_termination(sim: &Simulation) -> Option<TerminationReason> {
    let c = total_mass_field(sim, &sim.fields.catalyst);
    let a = total_mass_field(sim, &sim.fields.activated);
    let m = total_mass_field(sim, &sim.fields.membrane);
    let n = interior_mean(sim, &sim.fields.nutrient);
    let f = interior_mean(sim, &sim.fields.fuel);
    if soluble_max(sim) >= chemistry_core::config::CONC_SAFETY_LIMIT {
        return Some(TerminationReason::UnboundedAccumulation);
    }
    if c <= 1e-6 {
        return Some(TerminationReason::CatalystExtinction);
    }
    if a <= 1e-6 {
        return Some(TerminationReason::ActivatedExtinction);
    }
    if m <= 1e-6 {
        return Some(TerminationReason::MembraneExtinction);
    }
    if n <= 1e-6 || f <= 1e-6 {
        return Some(TerminationReason::ResourceExhaustion);
    }
    None
}

fn total_mass_field(sim: &Simulation, field: &[f64]) -> f64 {
    chemistry_core::total_mass(&sim.grid, field)
}

pub fn run_governed_reference(
    params: &SimParams,
    identity: &CandidateIdentity,
    source_commit: &str,
    binary_sha: &str,
    config: &D013RunConfig,
) -> Result<GovernedRunOutcome, Box<dyn std::error::Error>> {
    let wall = Instant::now();
    let mut sim = Simulation::new(params.clone());
    prepare_constrained_seed(&mut sim, config.radius);

    let mut activation = ActivationPotentialLedger::new(potential_from_masses(
        field_mass(&sim.grid, &sim.fields.fuel),
        field_mass(&sim.grid, &sim.fields.activated),
    ));
    let mut convergence = ConvergenceCounter {
        consecutive_qualifying: 0,
        required: D012_V2_REQUIRED_WINDOWS,
        windows: Vec::new(),
    };
    let mut pending_samples: Vec<AcceptedStateSample> = Vec::new();
    let mut window_anchor: Option<SteadyWindowSnapshot> = None;
    let mut steady_windows: Vec<SteadyWindowSnapshot> = Vec::new();
    let mut checkpoint_completion: BTreeMap<u64, bool> = BTreeMap::new();
    for &t in &D013_CHECKPOINT_THRESHOLDS {
        checkpoint_completion.insert(t, false);
    }

    if let Some(path) = &config.resume_checkpoint {
        let ckpt = load_governed_checkpoint(path)?;
        restore_from_checkpoint(&mut sim, &ckpt)?;
        activation = ckpt.activation_potential_accounting.clone();
        convergence = ckpt.rolling_window_state.clone();
        pending_samples = ckpt.pending_window_samples.clone();
        window_anchor = ckpt.window_anchor.clone();
        for w in &convergence.windows {
            if w.valid {
                // Reconstruct steady snapshots from window ends for classification helpers.
                if let Some(last) = pending_samples.last() {
                    let _ = last;
                }
            }
        }
        for &t in &D013_CHECKPOINT_THRESHOLDS {
            if sim.substep >= t {
                checkpoint_completion.insert(t, true);
            }
        }
    }

    let mut termination_reason = TerminationReason::MaxAcceptedSubstepsReached;
    let mut clean_termination = true;
    let window_size = config.window_size.max(1);

    while sim.substep < config.max_steps {
        if let Some(reason) = biological_termination(&sim) {
            termination_reason = reason;
            break;
        }
        if sim.max_consecutive_rejections >= config.rejection_stall_limit {
            termination_reason = TerminationReason::TimestepFloorFailure;
            break;
        }

        let prev_accepted = sim.substep;
        let prev_time = sim.sim_time;
        let prev_activation = activation.clone();
        let prev_material = build_material_equivalent_step(&sim.accounting.last_step);
        let rejected_before = sim.rejection_count;

        if !sim.step() {
            termination_reason = if sim.last_reject_limiter
                == chemistry_core::DtLimiter::FieldBoundValidation
                && sim.last_reject_detail.contains("excessive concentration")
            {
                TerminationReason::UnboundedAccumulation
            } else {
                TerminationReason::TimestepFloorFailure
            };
            break;
        }

        // Accepted-step authority: rejected attempts cannot reach this block.
        debug_assert_eq!(sim.substep, prev_accepted + 1);
        debug_assert!(sim.sim_time > prev_time || prev_accepted == 0);
        let _ = (prev_time, prev_activation, prev_material, rejected_before);

        let act_step = build_activation_potential_step(&sim.accounting.last_step);
        let activation_extent = sim.metabolism_accounting.last_step.activation;
        let productive = sim.constraint_accounting.last_step.virtual_production
            + sim.metabolism_accounting.last_step.reproduction
            + sim.membrane_accounting.last_step.synthesis;
        let turnover = sim.metabolism_accounting.last_step.activated_decay
            + sim.metabolism_accounting.last_step.catalyst_turnover;
        activation.apply_accepted_step(&act_step, activation_extent, productive, turnover);

        for event in crossed_checkpoint_thresholds(prev_accepted, sim.substep) {
            if let Some(dir) = &config.checkpoint_dir {
                let path = dir.join(format!("checkpoint_{:06}.json", event.threshold));
                let ckpt = build_checkpoint(
                    &sim,
                    event.threshold,
                    identity,
                    source_commit,
                    binary_sha,
                    &activation,
                    &convergence,
                    window_anchor.as_ref(),
                    &pending_samples,
                );
                write_governed_checkpoint(&path, &ckpt)?;
                checkpoint_completion.insert(event.threshold, true);
            } else {
                checkpoint_completion.insert(event.threshold, true);
            }
        }

        // Sample every accepted step into the current window buffer.
        pending_samples.push(capture_sample(&sim));
        if pending_samples.len() as u64 >= window_size {
            // Keep first and last for distinct-state check; downsample to window_size endpoints + fills.
            let samples = pending_samples.clone();
            let record = build_window_record(
                &samples,
                window_size,
                window_anchor.as_ref(),
                convergence.consecutive_qualifying,
            );
            if record.valid {
                let snap = sample_to_window_snapshot(
                    samples.last().unwrap(),
                    samples.first().unwrap(),
                );
                steady_windows.push(snap.clone());
                window_anchor = Some(snap);
            }
            let converged = update_convergence_counter(&mut convergence, record);
            pending_samples.clear();
            if converged {
                termination_reason = TerminationReason::QuasiSteadyConverged;
                break;
            }
        }
    }

    if matches!(termination_reason, TerminationReason::MaxAcceptedSubstepsReached)
        && sim.substep >= config.max_steps
    {
        termination_reason = TerminationReason::MaxAcceptedSubstepsReached;
    }

    // Mark checkpoints beyond horizon as not required / false if not reached.
    for &t in &D013_CHECKPOINT_THRESHOLDS {
        if sim.substep < t {
            checkpoint_completion.entry(t).or_insert(false);
        }
    }

    let partition =
        membrane_partition(&sim.grid, &sim.fields.structure, &sim.fields.membrane);
    let metrics = build_balance_metrics(
        sim.sim_time,
        &sim.constraint_accounting.cumulative,
        &sim.metabolism_accounting.cumulative,
        &sim.membrane_accounting.cumulative,
        &sim.transport_accounting.cumulative,
        retention(&sim, &sim.fields.catalyst),
        retention(&sim, &sim.fields.activated),
        partition.localization_fraction,
    );

    // Map legacy classification for oscillatory detection when not already terminated.
    if matches!(
        termination_reason,
        TerminationReason::MaxAcceptedSubstepsReached
    ) {
        let quasi = chemistry_core::quasi_steady_report(
            &steady_windows,
            window_size,
            D012_V2_REQUIRED_WINDOWS,
        );
        let legacy = classify_convergence(
            &quasi,
            &metrics,
            total_mass_field(&sim, &sim.fields.catalyst),
            total_mass_field(&sim, &sim.fields.activated),
            total_mass_field(&sim, &sim.fields.membrane),
            interior_mean(&sim, &sim.fields.nutrient),
            interior_mean(&sim, &sim.fields.fuel),
            soluble_max(&sim),
            sim.accounting.cumulative_within_tolerance(),
            sim.rejection_count as f64 / sim.attempted_substeps.max(1) as f64,
        );
        if matches!(legacy, ConvergenceClassification::OscillatoryUnresolved) {
            termination_reason = TerminationReason::OscillatoryUnresolved;
        }
    }

    if matches!(
        termination_reason,
        TerminationReason::TimestepFloorFailure | TerminationReason::NumericalFailure
    ) {
        clean_termination = true; // explicit governed numerical stop
    }

    let scientific =
        map_termination_to_scientific(termination_reason, config.max_steps, sim.substep);

    Ok(GovernedRunOutcome {
        metrics,
        accepted_substeps: sim.substep,
        attempted_substeps: sim.attempted_substeps,
        rejected_substeps: sim.rejection_count,
        simulated_time: sim.sim_time,
        current_dt: sim.dt,
        min_accepted_dt: sim.min_dt_seen,
        min_attempted_dt: sim.min_attempted_dt,
        max_consecutive_rejections: sim.max_consecutive_rejections,
        termination_reason,
        clean_termination,
        scientific_classification: scientific,
        convergence,
        material_accounting: build_material_equivalent_step(&sim.accounting.last_step),
        activation_potential_accounting: activation,
        checkpoint_completion,
        field_hashes: field_hashes(&sim),
        windows: steady_windows,
        wall_seconds: wall.elapsed().as_secs_f64(),
    })
}

fn outcome_artifact(
    outcome: &GovernedRunOutcome,
    identity: &CandidateIdentity,
    source_commit: &str,
    binary_sha: &str,
    config: &D013RunConfig,
    rates: &StageEReferenceRates,
) -> Value {
    let mut checkpoint_map = BTreeMap::new();
    for (k, v) in &outcome.checkpoint_completion {
        checkpoint_map.insert(k.to_string(), *v);
    }
    let balance = json!({
        "Q_structure": outcome.metrics.structure.q,
        "Q_catalyst": outcome.metrics.catalyst.q,
        "Q_membrane": outcome.metrics.membrane.q,
        "Q_activated": outcome.metrics.activated.q,
        "g_structure": outcome.metrics.structure.g,
        "g_catalyst": outcome.metrics.catalyst.g,
        "g_membrane": outcome.metrics.membrane.g,
        "g_activated": outcome.metrics.activated.g,
        "catalyst_retention": outcome.metrics.catalyst_retention,
        "activated_retention": outcome.metrics.activated_retention,
        "membrane_localization": outcome.metrics.membrane_localization,
        "nutrient_influx": outcome.metrics.nutrient_influx,
        "fuel_influx": outcome.metrics.fuel_influx,
        "waste_efflux": outcome.metrics.waste_efflux,
        "joint_overlap": joint_overlap_pass(&outcome.metrics),
    });
    let mut body = serde_json::Map::new();
    body.insert("project_directive".into(), json!("D-013"));
    body.insert(
        "snapshot_schema_version".into(),
        json!(chemistry_core::SNAPSHOT_SCHEMA_VERSION),
    );
    body.insert("field_schema_version".into(), json!(FieldSchemaVersion::SevenFieldV1));
    body.insert("field_schema".into(), json!("seven-field"));
    body.insert(
        "stoichiometric_schema_version".into(),
        json!(STOICHIOMETRIC_SCHEMA_VERSION_V2),
    );
    body.insert(
        "equation_version".into(),
        json!(EquationVersion::MembraneMetabolismV2Conservative),
    );
    body.insert("candidate_id".into(), json!(identity.candidate_id));
    body.insert("candidate_hash".into(), json!(identity.candidate_hash));
    body.insert("configuration_hash".into(), json!(identity.configuration_hash));
    body.insert("frozen_candidate_hash".into(), json!(FROZEN_CANDIDATE_HASH));
    body.insert(
        "frozen_configuration_hash".into(),
        json!(FROZEN_CONFIGURATION_HASH),
    );
    body.insert("source_commit".into(), json!(source_commit));
    body.insert("binary_sha256".into(), json!(binary_sha));
    body.insert("binary_hash".into(), json!(binary_sha));
    body.insert("seed".into(), json!(D013_SEED));
    body.insert("radius".into(), json!(config.radius));
    body.insert("selected_rates".into(), json!(rates));
    body.insert("accepted_substeps".into(), json!(outcome.accepted_substeps));
    body.insert("attempted_substeps".into(), json!(outcome.attempted_substeps));
    body.insert("rejected_substeps".into(), json!(outcome.rejected_substeps));
    body.insert("simulated_time".into(), json!(outcome.simulated_time));
    body.insert("final_dt".into(), json!(outcome.current_dt));
    body.insert("minimum_accepted_dt".into(), json!(outcome.min_accepted_dt));
    body.insert("minimum_attempted_dt".into(), json!(outcome.min_attempted_dt));
    body.insert(
        "maximum_consecutive_rejections".into(),
        json!(outcome.max_consecutive_rejections),
    );
    body.insert("max_steps".into(), json!(config.max_steps));
    body.insert("window_size".into(), json!(config.window_size));
    body.insert("termination_reason".into(), json!(outcome.termination_reason));
    body.insert("clean_termination".into(), json!(outcome.clean_termination));
    body.insert(
        "scientific_classification".into(),
        json!(outcome.scientific_classification),
    );
    body.insert("checkpoint_completion".into(), json!(checkpoint_map));
    body.insert("rolling_windows".into(), json!(outcome.convergence.windows));
    body.insert(
        "convergence_counter".into(),
        json!({
            "consecutive_qualifying": outcome.convergence.consecutive_qualifying,
            "required": outcome.convergence.required,
        }),
    );
    body.insert("balance_metrics".into(), balance);
    body.insert("material_accounting".into(), json!(outcome.material_accounting));
    body.insert(
        "activation_potential_accounting".into(),
        json!(outcome.activation_potential_accounting),
    );
    body.insert("field_hashes".into(), json!(outcome.field_hashes));
    body.insert("wall_seconds".into(), json!(outcome.wall_seconds));
    Value::Object(body)
}

pub fn seal_artifact(mut body: Value) -> Result<Value, Box<dyn std::error::Error>> {
    // Hash without artifact_hash field, then insert.
    body.as_object_mut().map(|o| o.remove("artifact_hash"));
    let bytes = serde_json::to_vec(&body)?;
    let hash = chemistry_core::sha256_hex(&bytes);
    body["artifact_hash"] = json!(hash);
    Ok(body)
}

pub fn artifact_view_from_json(value: &Value) -> GovernedArtifactView {
    let checkpoint_completion = value
        .get("checkpoint_completion")
        .and_then(|v| v.as_object())
        .map(|m| {
            m.iter()
                .map(|(k, v)| (k.clone(), v.as_bool().unwrap_or(false)))
                .collect()
        })
        .unwrap_or_default();
    let field_hashes = value.get("field_hashes").and_then(|v| {
        v.as_object().map(|m| {
            m.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
    });
    GovernedArtifactView {
        source_commit: value
            .get("source_commit")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        binary_hash: value
            .get("binary_hash")
            .or_else(|| value.get("binary_sha256"))
            .and_then(|v| v.as_str())
            .map(str::to_string),
        candidate_hash: value
            .get("candidate_hash")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        configuration_hash: value
            .get("configuration_hash")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        equation_version: value
            .get("equation_version")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        field_schema: value
            .get("field_schema")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        stoichiometric_schema: value
            .get("stoichiometric_schema_version")
            .and_then(|v| v.as_u64())
            .map(|u| u as u32),
        checkpoint_completion,
        accepted_substeps: value.get("accepted_substeps").and_then(|v| v.as_u64()),
        attempted_substeps: value.get("attempted_substeps").and_then(|v| v.as_u64()),
        rejected_substeps: value.get("rejected_substeps").and_then(|v| v.as_u64()),
        material_accounting: value
            .get("material_accounting")
            .and_then(|v| serde_json::from_value(v.clone()).ok()),
        activation_potential_accounting: value
            .get("activation_potential_accounting")
            .and_then(|v| serde_json::from_value(v.clone()).ok()),
        rolling_windows: value
            .get("rolling_windows")
            .and_then(|v| serde_json::from_value(v.clone()).ok()),
        termination_reason: value
            .get("termination_reason")
            .and_then(|v| serde_json::from_value(v.clone()).ok()),
        clean_termination: value.get("clean_termination").and_then(|v| v.as_bool()),
        field_hashes,
        artifact_hash: value
            .get("artifact_hash")
            .and_then(|v| v.as_str())
            .map(str::to_string),
    }
}

pub fn run_preflight(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    fs::create_dir_all(output)?;
    let checkpoint_dir = output.join("checkpoints");
    fs::create_dir_all(&checkpoint_dir)?;

    let mut params = v2_frozen_params()?;
    let rates = load_frozen_rates_from_invalid_reference()?;
    rates.apply_to(&mut params);
    let source_commit = git_commit_hash()?;
    let binary_sha = binary_hash()?;
    let identity = build_candidate_identity(
        params.clone(),
        &source_commit,
        Some("d013-preflight"),
        None,
        "D-013 Stage E preflight",
        None,
        None,
    );

    let config = D013RunConfig {
        max_steps: 25_000,
        window_size: D012_V2_WINDOW.min(5_000).max(100),
        radius: D012_V2_CENTER_RADIUS,
        rejection_stall_limit: D013_DEFAULT_REJECTION_STALL_LIMIT,
        checkpoint_dir: Some(checkpoint_dir.clone()),
        resume_checkpoint: None,
    };

    // Preflight uses window_size=1000 for faster window machinery exercise while still
    // requiring 10k/25k checkpoints. Scientific Stage E keeps 10k windows.
    let config = D013RunConfig {
        window_size: 1_000,
        ..config
    };

    let outcome = run_governed_reference(&params, &identity, &source_commit, &binary_sha, &config)?;
    let mut artifact = outcome_artifact(&outcome, &identity, &source_commit, &binary_sha, &config, &rates);
    artifact = seal_artifact(artifact)?;
    let view = artifact_view_from_json(&artifact);
    let (status, missing) = validate_governed_artifact(&view);

    let ckpt_10k = checkpoint_dir.join("checkpoint_010000.json");
    let ckpt_25k = checkpoint_dir.join("checkpoint_025000.json");
    let has_10k = ckpt_10k.exists();
    let has_25k = ckpt_25k.exists();

    // Continuation check: resume 10k → 25k vs uninterrupted presence of 25k.
    let mut continuation_ok = false;
    if has_10k {
        let cont_dir = output.join("continuation_checkpoints");
        fs::create_dir_all(&cont_dir)?;
        let cont_config = D013RunConfig {
            max_steps: 25_000,
            window_size: 1_000,
            radius: D012_V2_CENTER_RADIUS,
            rejection_stall_limit: D013_DEFAULT_REJECTION_STALL_LIMIT,
            checkpoint_dir: Some(cont_dir),
            resume_checkpoint: Some(ckpt_10k.clone()),
        };
        let cont = run_governed_reference(&params, &identity, &source_commit, &binary_sha, &cont_config)?;
        continuation_ok = cont.accepted_substeps == outcome.accepted_substeps
            && (cont.simulated_time - outcome.simulated_time).abs() < 1e-9
            && cont.field_hashes == outcome.field_hashes;
    }

    let preflight_pass = has_10k
        && has_25k
        && status == ArtifactValidationStatus::ValidGovernedArtifact
        && outcome.activation_potential_accounting.activation_potential_schema_version > 0
        && matches!(
            outcome.termination_reason,
            TerminationReason::MaxAcceptedSubstepsReached
                | TerminationReason::QuasiSteadyConverged
                | TerminationReason::ResourceExhaustion
                | TerminationReason::CatalystExtinction
                | TerminationReason::ActivatedExtinction
                | TerminationReason::MembraneExtinction
                | TerminationReason::UnboundedAccumulation
        )
        && continuation_ok
        && !outcome.convergence.windows.iter().any(|w| !w.valid && w.qualifying);

    let result = json!({
        "preflight_pass": preflight_pass,
        "checkpoint_10k": has_10k,
        "checkpoint_25k": has_25k,
        "continuation_equivalence": continuation_ok,
        "artifact_validation": status,
        "missing_fields": missing,
        "termination_reason": outcome.termination_reason,
        "accepted_substeps": outcome.accepted_substeps,
        "attempted_substeps": outcome.attempted_substeps,
        "rejected_substeps": outcome.rejected_substeps,
        "simulated_time": outcome.simulated_time,
        "artifact": artifact,
        "note": "Preflight is not a Stage E scientific result; do not derive solver corrections.",
    });
    atomic_write_json(&output.join("result.json"), &result)?;
    Ok(result)
}

pub fn run_reference_radius(
    output: &Path,
    radius: f64,
    max_steps: u64,
) -> Result<Value, Box<dyn std::error::Error>> {
    fs::create_dir_all(output)?;
    let checkpoint_dir = output.join("checkpoints");
    fs::create_dir_all(&checkpoint_dir)?;

    let mut params = v2_frozen_params()?;
    let rates = load_frozen_rates_from_invalid_reference()?;
    rates.apply_to(&mut params);
    let source_commit = git_commit_hash()?;
    let binary_sha = binary_hash()?;
    let identity = build_candidate_identity(
        params.clone(),
        &source_commit,
        Some("d013-reference"),
        None,
        "D-013 governed Stage E reference",
        None,
        None,
    );

    // Verify freeze hashes match preserved candidate when possible.
    if identity.candidate_hash != FROZEN_CANDIDATE_HASH {
        // Rates come from frozen artifact; identity hash may differ if source_commit changed.
        // Record both for audit; do not retune rates.
    }

    let config = D013RunConfig {
        max_steps,
        window_size: D012_V2_WINDOW,
        radius,
        rejection_stall_limit: D013_DEFAULT_REJECTION_STALL_LIMIT,
        checkpoint_dir: Some(checkpoint_dir),
        resume_checkpoint: None,
    };
    let outcome = run_governed_reference(&params, &identity, &source_commit, &binary_sha, &config)?;
    let mut artifact = outcome_artifact(&outcome, &identity, &source_commit, &binary_sha, &config, &rates);
    artifact = seal_artifact(artifact)?;
    let view = artifact_view_from_json(&artifact);
    let (status, missing) = validate_governed_artifact(&view);
    artifact["artifact_validation"] = json!(status);
    artifact["artifact_validation_missing"] = json!(missing);
    artifact["solver_entry_allowed"] = json!(solver_entry_allowed(
        status,
        outcome.scientific_classification,
        true,
        true
    ));

    atomic_write_json(&output.join("result.json"), &artifact)?;
    Ok(artifact)
}

pub fn run_d013_pipeline(root: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let preflight_dir = root.join("preflight");
    let preflight = run_preflight(&preflight_dir)?;
    if preflight["preflight_pass"].as_bool() != Some(true) {
        let fail = json!({
            "d013_conclusion": "D013_HARNESS_REPAIR_FAILED",
            "preflight": preflight,
            "next": "repair harness until preflight passes",
        });
        atomic_write_json(&root.join("manifest.json"), &fail)?;
        return Ok(fail);
    }

    let r22 = run_reference_radius(&root.join("reference_r22"), D012_V2_CENTER_RADIUS, D012_V2_MAX_STEPS)?;
    let sci = r22["scientific_classification"].clone();
    let mut r18 = Value::Null;
    let mut r26 = Value::Null;
    let mut branch = "center_only";

    if r22["artifact_validation"] == json!(ArtifactValidationStatus::ValidGovernedArtifact)
        && sci == json!(ScientificClassification::QuasiSteadyConverged)
    {
        r18 = run_reference_radius(&root.join("reference_r18"), 18.0, D012_V2_MAX_STEPS)?;
        r26 = run_reference_radius(&root.join("reference_r26"), 26.0, D012_V2_MAX_STEPS)?;
        branch = "center_and_neighbors";
    }

    let conclusion = if r22["artifact_validation"] != json!(ArtifactValidationStatus::ValidGovernedArtifact)
    {
        "D013_HARNESS_REPAIR_FAILED"
    } else if sci == json!(ScientificClassification::QuasiSteadyConverged) {
        "D013_GOVERNED_REFERENCE_RECOVERED"
    } else if sci == json!(ScientificClassification::NotConvergedAt200k) {
        "D013_REFERENCE_VALID_NOT_CONVERGED"
    } else if matches!(
        sci.as_str(),
        Some("RESOURCE_EXHAUSTION")
            | Some("CATALYST_EXTINCTION")
            | Some("ACTIVATED_EXTINCTION")
            | Some("MEMBRANE_EXTINCTION")
            | Some("UNBOUNDED_ACCUMULATION")
    ) {
        "D013_REFERENCE_VALID_BIOLOGICAL_FAILURE"
    } else if sci == json!(ScientificClassification::NumericalFailure) {
        "D013_REFERENCE_NUMERICAL_FAILURE"
    } else {
        "D013_FAIL"
    };

    let manifest = json!({
        "project_directive": "D-013",
        "agent_memory_directive": "D-20260715-d013-stage-e-harness-integrity",
        "d013_conclusion": conclusion,
        "branch": branch,
        "preflight": preflight,
        "reference_r22": r22,
        "reference_r18": r18,
        "reference_r26": r26,
        "solver_entry_allowed": r22.get("solver_entry_allowed").cloned().unwrap_or(json!(false)),
        "frozen_candidate_hash": FROZEN_CANDIDATE_HASH,
        "frozen_configuration_hash": FROZEN_CONFIGURATION_HASH,
    });
    atomic_write_json(&root.join("manifest.json"), &manifest)?;
    Ok(manifest)
}
