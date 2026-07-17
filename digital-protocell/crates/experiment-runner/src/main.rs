//! Headless experiment runner for Phase 1 scientific acceptance.

mod d003;
mod d004;
mod d005;
mod d006;
mod d007;
mod d008;
mod d011;
mod d012;
mod d012_stage_e;
mod d013;
mod d014;
mod d015;
mod d016;
mod d017;
mod d018;
mod d019;
mod d020;

use chemistry_core::*;
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[derive(Parser)]
#[command(name = "experiment-runner")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run {
        #[arg(long, default_value = "configs/baseline.toml")]
        config: PathBuf,
        #[arg(long, default_value = "experiments/generated")]
        output: PathBuf,
    },
    Baseline {
        #[arg(long, default_value = "configs/phase1_candidate.toml")]
        config: PathBuf,
        #[arg(long, default_value = "250000")]
        steps: u64,
        #[arg(long, default_value = "1")]
        seed: u64,
        #[arg(long)]
        output: PathBuf,
    },
    Acceptance {
        #[arg(long, default_value = "configs/phase1_candidate.toml")]
        config: PathBuf,
        #[arg(long, default_value = "experiments/generated/phase1_acceptance")]
        output: PathBuf,
    },
    Sweep {
        #[arg(long, default_value = "configs/parameter_sweep.toml")]
        config: PathBuf,
        #[arg(long, default_value = "experiments/generated/sweep")]
        output: PathBuf,
    },
    All {
        #[arg(long, default_value = "experiments/generated")]
        output: PathBuf,
    },
    D003 {
        #[command(subcommand)]
        action: D003Commands,
    },
    D004 {
        #[command(subcommand)]
        action: D004Commands,
    },
    D005 {
        #[command(subcommand)]
        action: D005Commands,
    },
    D006 {
        #[command(subcommand)]
        action: D006Commands,
    },
    D007 {
        #[command(subcommand)]
        action: D007Commands,
    },
    D008 {
        #[command(subcommand)]
        action: D008Commands,
    },
    D011 {
        #[command(subcommand)]
        action: D011Commands,
    },
    D012 {
        #[command(subcommand)]
        action: D012Commands,
    },
    D013 {
        #[command(subcommand)]
        action: D013Commands,
    },
    D014 {
        #[command(subcommand)]
        action: D014Commands,
    },
    D015 {
        #[command(subcommand)]
        action: D015Commands,
    },
    D016 {
        #[command(subcommand)]
        action: D016Commands,
    },
    D017 {
        #[command(subcommand)]
        action: D017Commands,
    },
    D018 {
        #[command(subcommand)]
        action: D018Commands,
    },
    D019 {
        #[command(subcommand)]
        action: D019Commands,
    },
    D020 {
        #[command(subcommand)]
        action: D020Commands,
    },
}

#[derive(Subcommand)]
enum D003Commands {
    Diagnose,
    Calibrate {
        #[arg(long, default_value = "1.0")]
        k_phi: f64,
    },
    Screen {
        #[arg(long, default_value = "20000")]
        steps: u64,
    },
    Pipeline,
}

#[derive(Subcommand)]
enum D004Commands {
    /// Full D-004 provenance and attractor audit
    Audit,
    /// Extract final calibrated configs to configs/d004/
    ExtractConfigs,
    /// SHA-256 manifest for D-003 artifacts
    Manifest,
}

#[derive(Subcommand)]
enum D005Commands {
    /// Full D-005 accessible-attractor pipeline
    Pipeline {
        #[arg(long, default_value = "250000")]
        continuation_target: u64,
        #[arg(long, default_value = "20000")]
        coarse_steps: u64,
    },
    /// Aggregate D-004 cross-state results only
    Aggregate,
    /// Continue fresh-state runs from D-004 snapshots
    Continuations {
        #[arg(long, default_value = "250000")]
        target_substeps: u64,
    },
    /// Coarse basin map for one k_phi
    CoarseBasin {
        #[arg(long, default_value = "1.0")]
        k_phi: f64,
        #[arg(long, default_value = "20000")]
        steps: u64,
    },
    /// Finalize flow/nullcline/manifest from completed artifacts (no long sims)
    Finalize,
}

#[derive(Subcommand)]
enum D006Commands {
    /// Planar interface calibration + five candidates + prescribed radius
    Bootstrap,
    /// Coupled radius screen for surviving candidates
    Screen {
        #[arg(long, default_value = "50000")]
        steps: u64,
    },
    /// Bootstrap then screen
    Pipeline {
        #[arg(long, default_value = "50000")]
        steps: u64,
    },
    /// Single coupled screen point (for parallel orchestration)
    RunOne {
        #[arg(long)]
        candidate_id: String,
        #[arg(long)]
        r0: f64,
        #[arg(long)]
        c0: f64,
        #[arg(long)]
        seed: u64,
        #[arg(long, default_value = "50000")]
        steps: u64,
    },
}

#[derive(Subcommand)]
enum D007Commands {
    /// Write reference config + ensure artifact dirs
    Init,
    /// Replay D-006 1.0× reference (expects v_R>0, v_C_inside<0)
    ReferenceReplay {
        #[arg(long, default_value = "10000")]
        steps: u64,
    },
    /// Write structural-bracket candidate for a factor
    WriteStructuralCandidate {
        #[arg(long)]
        factor: f64,
    },
    /// Strict-schema single run for an identity.json path
    RunOne {
        #[arg(long)]
        identity: PathBuf,
        #[arg(long)]
        r0: f64,
        #[arg(long)]
        c0: f64,
        #[arg(long)]
        seed: u64,
        #[arg(long, default_value = "30000")]
        steps: u64,
        #[arg(long)]
        output: PathBuf,
    },
    /// Write a joint candidate (structural_factor × k_rep)
    WriteJointCandidate {
        #[arg(long)]
        structural_factor: f64,
        #[arg(long)]
        k_rep: f64,
        #[arg(long, default_value = "1.0")]
        catalyst_factor: f64,
        #[arg(long, default_value = "d006-1.0x-reference")]
        parent: String,
    },
}

#[derive(Subcommand)]
enum D008Commands {
    /// Run the immutable Stage A planar selective-transport gate.
    StageA {
        #[arg(long)]
        output: PathBuf,
    },
    /// Run the immutable Stage B fixed-field membrane-localization gate.
    StageB {
        #[arg(long)]
        output: PathBuf,
    },
    /// Run the immutable Stage C zero-dimensional activated-metabolism gate.
    StageC {
        #[arg(long)]
        output: PathBuf,
    },
    /// Run the immutable Stage D fixed-compartment coupling gate.
    StageD {
        #[arg(long)]
        output: PathBuf,
    },
    /// Run the immutable Stage E prescribed-radius balance gate.
    StageE {
        #[arg(long)]
        output: PathBuf,
    },
}

#[derive(Subcommand)]
enum D011Commands {
    /// Run the D-011 transport-coupled constrained-radius balance assay.
    Run {
        #[arg(long, default_value = "experiments/generated/d011")]
        output: PathBuf,
        #[arg(long, default_value = "50000")]
        max_steps: u64,
        #[arg(long, default_value = "10000")]
        window_size: u64,
        #[arg(long, default_value = "false")]
        quick: bool,
    },
}

#[derive(Subcommand)]
enum D012Commands {
    /// Run conservative v2 Stage B membrane-localization gate.
    StageB {
        #[arg(long)]
        output: PathBuf,
    },
    /// Run conservative v2 Stage C zero-dimensional metabolism gate.
    StageC {
        #[arg(long)]
        output: PathBuf,
    },
    /// Run conservative v2 Stage D fixed-compartment gate.
    StageD {
        #[arg(long)]
        output: PathBuf,
    },
    /// Run conservative v2 Stage E transport-coupled reference assay.
    StageE {
        #[arg(long, default_value = "experiments/generated/d012/v2_stage_e_reference")]
        output: PathBuf,
        #[arg(long, default_value = "200000")]
        max_steps: u64,
        #[arg(long, default_value = "10000")]
        window_size: u64,
        #[arg(long, default_value = "false")]
        diagnostic: bool,
    },
    /// Run v2 bounded four-rate joint solver after reference sensitivity.
    StageESolver {
        #[arg(long, default_value = "experiments/generated/d012/v2_joint_candidates")]
        output: PathBuf,
        #[arg(long, default_value = "experiments/generated/d012/v2_stage_e_reference")]
        reference: PathBuf,
        #[arg(long, default_value = "200000")]
        max_steps: u64,
        #[arg(long, default_value = "10000")]
        window_size: u64,
        #[arg(long, default_value = "false")]
        diagnostic: bool,
    },
    /// Conditional v2 yield branch (one component at a time).
    StageEYield {
        #[arg(long, default_value = "experiments/generated/d012/v2_yield_candidates")]
        output: PathBuf,
        #[arg(long, default_value = "experiments/generated/d012/v2_stage_e_reference")]
        diagnosis: PathBuf,
        #[arg(long, default_value = "5000")]
        max_steps: u64,
        #[arg(long, default_value = "1000")]
        window_size: u64,
    },
    /// Robust overlap: ±2% rates and ±5% initial C/A/M.
    StageERobust {
        #[arg(long, default_value = "experiments/generated/d012/v2_robust_overlap")]
        output: PathBuf,
        #[arg(long, default_value = "experiments/generated/d012/v2_stage_e_reference")]
        candidate: PathBuf,
        #[arg(long, default_value = "200000")]
        max_steps: u64,
        #[arg(long, default_value = "10000")]
        window_size: u64,
        #[arg(long, default_value = "false")]
        diagnostic: bool,
    },
    /// Diagnostic short-horizon Stage E reference (rate estimation screens).
    StageEDiagnostic {
        #[arg(long, default_value = "experiments/generated/d012/v2_stage_e_reference")]
        output: PathBuf,
    },
}

const CHECKPOINT_STEPS: [u64; 7] = [0, 25_000, 50_000, 100_000, 150_000, 200_000, 250_000];
const RATIO_CHECKPOINTS: [u64; 6] = [25_000, 50_000, 100_000, 150_000, 200_000, 250_000];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run { config, output } => run_single(&config, &output)?,
        Commands::Baseline {
            config,
            steps,
            seed,
            output,
        } => run_baseline_acceptance(&config, steps, seed, &output)?,
        Commands::Acceptance { config, output } => run_full_acceptance(&config, &output)?,
        Commands::Sweep { config, output } => run_sweep(&config, &output)?,
        Commands::All { output } => run_all_experiments(&output)?,
        Commands::D003 { action } => run_d003(action)?,
        Commands::D004 { action } => run_d004(action)?,
        Commands::D005 { action } => run_d005(action)?,
        Commands::D006 { action } => run_d006(action)?,
        Commands::D007 { action } => run_d007(action)?,
        Commands::D008 { action } => run_d008(action)?,
        Commands::D011 { action } => run_d011(action)?,
        Commands::D012 { action } => run_d012(action)?,
        Commands::D013 { action } => run_d013(action)?,
        Commands::D014 { action } => run_d014(action)?,
        Commands::D015 { action } => run_d015(action)?,
        Commands::D016 { action } => run_d016(action)?,
        Commands::D017 { action } => run_d017(action)?,
        Commands::D018 { action } => run_d018(action)?,
        Commands::D019 { action } => run_d019(action)?,
        Commands::D020 { action } => run_d020(action)?,
    }
    Ok(())
}

fn run_d003(action: D003Commands) -> Result<(), Box<dyn std::error::Error>> {
    let root = d003::d003_output_root();
    match action {
        D003Commands::Diagnose => {
            d003::run_diagnosis(&root.join("diagnosis"))?;
            for k in [0.5, 1.0, 2.0] {
                let est = d003::analytical_estimates_from_d002(k);
                fs::create_dir_all(root.join("analytical_estimates"))?;
                fs::write(
                    root.join(format!("analytical_estimates/kphi_{k}.json")),
                    serde_json::to_string_pretty(&est)?,
                )?;
            }
            println!("D-003 diagnosis -> {}", root.join("diagnosis").display());
        }
        D003Commands::Calibrate { k_phi } => {
            let out = root.join(format!("calibration/kphi_{}", k_phi.to_string().replace('.', "_")));
            let result = d003::calibrate_kphi(k_phi, &out, 20_000)?;
            println!("Calibration k_phi={k_phi}: {result}");
        }
        D003Commands::Screen { steps } => {
            let params = d003::load_final_calibrated_params(1.0)
                .unwrap_or_else(|_| d003::params_from_analytical_estimate(1.0));
            let identity = build_candidate_identity(
                params,
                &git_commit_hash(),
                Some("kphi_1.0"),
                Some(5),
                "final calibrated K_phi=1.0 candidate",
                None,
                None,
            );
            let results = d003::short_screen(&identity, &[1, 2, 3], steps, &root.join("short_screen"))?;
            println!("Short screen: {results:?}");
        }
        D003Commands::Pipeline => {
            d003::run_diagnosis(&root.join("diagnosis"))?;
            let commit = git_commit_hash();
            for k in [0.5, 1.0, 2.0] {
                let est = d003::analytical_estimates_from_d002(k);
                fs::create_dir_all(root.join("analytical_estimates"))?;
                fs::write(
                    root.join(format!("analytical_estimates/kphi_{k}.json")),
                    serde_json::to_string_pretty(&est)?,
                )?;
                d003::calibrate_kphi(k, &root.join(format!("calibration/kphi_{k}")), 20_000)?;
            }
            let params = d003::load_final_calibrated_params(1.0)?;
            let identity = build_candidate_identity(
                params,
                &commit,
                Some("kphi_1.0"),
                Some(5),
                "final calibrated K_phi=1.0 candidate for Stage B",
                None,
                None,
            );
            d003::short_screen(&identity, &[1, 2, 3], 20_000, &root.join("short_screen"))?;
            println!("D-003 pipeline complete -> {}", root.display());
        }
    }
    Ok(())
}

fn run_d004(action: D004Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D004Commands::Audit => {
            let summary = d004::run_full_audit()?;
            println!("D-004 audit complete: {summary}");
        }
        D004Commands::ExtractConfigs => {
            let ids = d004::extract_final_configs()?;
            for id in &ids {
                println!("{} hash={}", id.candidate_id, id.candidate_hash);
            }
        }
        D004Commands::Manifest => {
            let m = d004::sha256_manifest(&d004::d003_root())?;
            fs::write(
                d004::d003_root().join("manifest.json"),
                serde_json::to_string_pretty(&m)?,
            )?;
            println!("D-003 manifest written");
        }
    }
    Ok(())
}

fn run_d005(action: D005Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D005Commands::Pipeline {
            continuation_target,
            coarse_steps,
        } => {
            let summary = d005::run_full_d005(continuation_target, coarse_steps)?;
            println!("D-005 pipeline complete: {summary}");
        }
        D005Commands::Aggregate => {
            let agg = d005::aggregate_d004_cross_state()?;
            println!("D-004 aggregate: {} runs", agg["run_count"]);
        }
        D005Commands::Continuations { target_substeps } => {
            let results = d005::run_all_continuations(target_substeps)?;
            println!("Continuations complete: {} runs", results.len());
        }
        D005Commands::CoarseBasin { k_phi, steps } => {
            let ids = d005::load_d004_identities()?;
            let id = ids
                .into_iter()
                .find(|i| (i.k_phi - k_phi).abs() < 1e-9)
                .ok_or("k_phi candidate not found")?;
            let n = d005::run_coarse_basin(&id, steps)?.len();
            println!("Coarse basin: {n} points for k_phi={k_phi}");
        }
        D005Commands::Finalize => {
            let summary = d005::finalize_from_artifacts()?;
            println!("D-005 finalize: {summary}");
        }
    }
    Ok(())
}

fn run_d006(action: D006Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D006Commands::Bootstrap => {
            let planar = d006::run_planar_calibration()?;
            let k0 = planar["k_structure_interface_initial"].as_f64().unwrap();
            let ids = d006::write_candidates(k0)?;
            for id in &ids {
                let pr = d006::run_prescribed_radius(id)?;
                println!(
                    "{} crossing={}",
                    id.candidate_id, pr["has_stable_crossing"]
                );
            }
            println!("D-006 bootstrap: {} candidates, k0={k0}", ids.len());
        }
        D006Commands::Screen { steps } => {
            let summary = d006::run_coupled_screen(steps)?;
            println!("D-006 screen: {summary}");
        }
        D006Commands::Pipeline { steps } => {
            let summary = d006::bootstrap_and_screen(steps)?;
            println!("D-006 pipeline: {summary}");
        }
        D006Commands::RunOne {
            candidate_id,
            r0,
            c0,
            seed,
            steps,
        } => {
            let id_path = d006::d006_root()
                .join("candidates")
                .join(&candidate_id)
                .join("identity.json");
            let id: chemistry_core::CandidateIdentity =
                serde_json::from_str(&fs::read_to_string(id_path)?)?;
            let out = d006::d006_root().join("candidate_screen").join(&candidate_id).join(
                format!("R{}_C{}_s{}", r0 as u32, (c0 * 1000.0) as u32, seed),
            );
            let rec = d006::run_one_public(&id, r0, c0, seed, steps, &out)?;
            println!("D-006 run-one: {}", rec["seed_recipe"]);
        }
    }
    Ok(())
}

fn run_d007(action: D007Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D007Commands::Init => {
            d007::ensure_artifact_dirs()?;
            let id = d007::write_reference_config()?;
            println!(
                "D-007 init: reference {} config_hash={}",
                id.candidate_id, id.configuration_hash
            );
        }
        D007Commands::ReferenceReplay { steps } => {
            let summary = d007::run_reference_replay(steps)?;
            println!("D-007 reference replay: {summary}");
        }
        D007Commands::WriteStructuralCandidate { factor } => {
            d007::ensure_artifact_dirs()?;
            let id = d007::write_structural_candidate(factor)?;
            println!(
                "D-007 structural candidate factor={factor} id={} k_iface={}",
                id.candidate_id, id.params.k_structure_interface
            );
        }
        D007Commands::RunOne {
            identity,
            r0,
            c0,
            seed,
            steps,
            output,
        } => {
            let id: chemistry_core::CandidateIdentity =
                serde_json::from_str(&fs::read_to_string(identity)?)?;
            let rec = d007::run_strict(&id, r0, c0, seed, steps, &output)?;
            println!(
                "D-007 run-one clean={} v_R={} v_C_inside={}",
                rec["clean_termination"], rec["v_R"], rec["v_C_inside"]
            );
        }
        D007Commands::WriteJointCandidate {
            structural_factor,
            k_rep,
            catalyst_factor,
            parent,
        } => {
            d007::ensure_artifact_dirs()?;
            let id = d007::make_joint_candidate(
                structural_factor,
                k_rep,
                &parent,
                "D-007 joint candidate",
            );
            let dir = d007::write_joint_candidate(&id, structural_factor, catalyst_factor, &parent)?;
            println!(
                "D-007 joint candidate {} -> {}",
                id.candidate_id,
                dir.display()
            );
        }
    }
    Ok(())
}

fn run_d008(action: D008Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D008Commands::StageA { output } => {
            let result = d008::run_stage_a(&output)?;
            println!(
                "D-008 Stage A: {} -> {}",
                result["stage_classification"],
                output.display()
            );
        }
        D008Commands::StageB { output } => {
            let result = d008::run_stage_b(&output)?;
            println!(
                "D-008 Stage B: {} -> {}",
                result["stage_classification"],
                output.display()
            );
        }
        D008Commands::StageC { output } => {
            let result = d008::run_stage_c(&output)?;
            println!(
                "D-008 Stage C: {} -> {}",
                result["stage_classification"],
                output.display()
            );
        }
        D008Commands::StageD { output } => {
            let result = d008::run_stage_d(&output)?;
            println!(
                "D-008 Stage D: {} -> {}",
                result["stage_classification"], result["attempt_directory"]
            );
        }
        D008Commands::StageE { output } => {
            let result = d008::run_stage_e(&output)?;
            println!(
                "D-008 Stage E: {} -> {}",
                result["stage_classification"], result["attempt_directory"]
            );
        }
    }
    Ok(())
}

fn run_d011(action: D011Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D011Commands::Run {
            output,
            max_steps,
            window_size,
            quick,
        } => {
            let config = d011::D011RunConfig {
                max_steps: if quick { 5_000 } else { max_steps },
                window_size: if quick { 1_000 } else { window_size },
                quick,
            };
            let result = d011::run_d011_protocol(&output, &config)?;
            println!(
                "D-011: {} -> {:?}",
                result["scientific_conclusion"], result["attempt_directory"]
            );
        }
    }
    Ok(())
}

fn resolve_d012_artifact_path(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        return path;
    }
    let canonical_root = d012::d012_generated_root();
    let rendered = path.to_string_lossy();
    if let Some(rest) = rendered.strip_prefix("experiments/generated/d012/") {
        return canonical_root.join(rest);
    }
    if rendered == "experiments/generated/d012" {
        return canonical_root;
    }
    // Default: resolve relative to chemistry workspace so cwd cannot leak artifacts.
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(path)
}

fn run_d012(action: D012Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D012Commands::StageB { output } => {
            let output = resolve_d012_artifact_path(output);
            let result = d012::run_v2_stage_b(&output)?;
            println!(
                "D-012 Stage B: {} -> {}",
                result["stage_classification"],
                output.display()
            );
        }
        D012Commands::StageC { output } => {
            let output = resolve_d012_artifact_path(output);
            let result = d012::run_v2_stage_c(&output)?;
            println!(
                "D-012 Stage C: {} -> {}",
                result["stage_classification"],
                output.display()
            );
        }
        D012Commands::StageD { output } => {
            let output = resolve_d012_artifact_path(output);
            let result = d012::run_v2_stage_d(&output)?;
            println!(
                "D-012 Stage D: {} -> {:?}",
                result["stage_classification"], result["attempt_directory"]
            );
        }
        D012Commands::StageE {
            output,
            max_steps,
            window_size,
            diagnostic,
        } => {
            let output = resolve_d012_artifact_path(output);
            let config = d012::D012StageEConfig {
                max_steps,
                window_size,
                diagnostic,
            };
            let result = d012::run_v2_stage_e_reference(&output, &config)?;
            println!(
                "D-012 Stage E: {} pass={} -> {}",
                result["stage_classification"],
                result["stage_e_pass"],
                output.display()
            );
        }
        D012Commands::StageEDiagnostic { output } => {
            let output = resolve_d012_artifact_path(output);
            let config = d012::D012StageEConfig::diagnostic();
            let result = d012::run_v2_stage_e_reference(&output, &config)?;
            println!(
                "D-012 Stage E diagnostic: {} -> {}",
                result["stage_classification"],
                output.display()
            );
        }
        D012Commands::StageESolver {
            output,
            reference,
            max_steps,
            window_size,
            diagnostic,
        } => {
            let output = resolve_d012_artifact_path(output);
            let reference = resolve_d012_artifact_path(reference);
            let config = d012::D012StageEConfig {
                max_steps,
                window_size,
                diagnostic,
            };
            let result = d012::run_v2_stage_e_solver(&output, &reference, &config)?;
            println!(
                "D-012 Stage E solver: any_pass={} -> {}",
                result["any_joint_overlap_pass"],
                output.display()
            );
        }
        D012Commands::StageEYield {
            output,
            diagnosis,
            max_steps,
            window_size,
        } => {
            let output = resolve_d012_artifact_path(output);
            let diagnosis = resolve_d012_artifact_path(diagnosis);
            let config = d012::D012StageEConfig {
                max_steps,
                window_size,
                diagnostic: true,
            };
            let result = d012::run_v2_stage_e_yield(&output, &diagnosis, &config)?;
            println!(
                "D-012 Stage E yield: skipped={} -> {}",
                result["skipped"],
                output.display()
            );
        }
        D012Commands::StageERobust {
            output,
            candidate,
            max_steps,
            window_size,
            diagnostic,
        } => {
            let output = resolve_d012_artifact_path(output);
            let candidate = resolve_d012_artifact_path(candidate);
            let config = d012::D012StageEConfig {
                max_steps,
                window_size,
                diagnostic,
            };
            let result = d012::run_v2_stage_e_robust(&output, &candidate, &config)?;
            println!(
                "D-012 Stage E robust: restoring={} rate_robust={} -> {}",
                result["restoring_radius_pass"],
                result["rate_robust_overlap"],
                output.display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D013Commands {
    /// Deterministic Stage E harness preflight (R=22, 25k accepted).
    Preflight {
        #[arg(long, default_value = "experiments/generated/d013/preflight")]
        output: PathBuf,
    },
    /// Governed center reference (R=22, up to 200k accepted).
    ReferenceR22 {
        #[arg(long, default_value = "experiments/generated/d013/reference_r22")]
        output: PathBuf,
        #[arg(long, default_value = "200000")]
        max_steps: u64,
    },
    /// Neighbor reference R=18 (only after valid converged R22).
    ReferenceR18 {
        #[arg(long, default_value = "experiments/generated/d013/reference_r18")]
        output: PathBuf,
        #[arg(long, default_value = "200000")]
        max_steps: u64,
    },
    /// Neighbor reference R=26 (only after valid converged R22).
    ReferenceR26 {
        #[arg(long, default_value = "experiments/generated/d013/reference_r26")]
        output: PathBuf,
        #[arg(long, default_value = "200000")]
        max_steps: u64,
    },
    /// Full D-013 pipeline: preflight then progressive reference.
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d013")]
        output: PathBuf,
    },
}

fn resolve_d013_artifact_path(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        return path;
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(path)
}

fn run_d013(action: D013Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D013Commands::Preflight { output } => {
            let output = resolve_d013_artifact_path(output);
            let result = d013::run_preflight(&output)?;
            println!(
                "D-013 preflight: pass={} -> {}",
                result["preflight_pass"],
                output.display()
            );
        }
        D013Commands::ReferenceR22 { output, max_steps } => {
            let output = resolve_d013_artifact_path(output);
            let result = d013::run_reference_radius(&output, 22.0, max_steps)?;
            println!(
                "D-013 R22: {:?} {:?} -> {}",
                result["termination_reason"],
                result["scientific_classification"],
                output.display()
            );
        }
        D013Commands::ReferenceR18 { output, max_steps } => {
            let output = resolve_d013_artifact_path(output);
            let result = d013::run_reference_radius(&output, 18.0, max_steps)?;
            println!(
                "D-013 R18: {:?} {:?} -> {}",
                result["termination_reason"],
                result["scientific_classification"],
                output.display()
            );
        }
        D013Commands::ReferenceR26 { output, max_steps } => {
            let output = resolve_d013_artifact_path(output);
            let result = d013::run_reference_radius(&output, 26.0, max_steps)?;
            println!(
                "D-013 R26: {:?} {:?} -> {}",
                result["termination_reason"],
                result["scientific_classification"],
                output.display()
            );
        }
        D013Commands::Pipeline { output } => {
            let output = resolve_d013_artifact_path(output);
            let result = d013::run_d013_pipeline(&output)?;
            println!(
                "D-013 pipeline: {} -> {}",
                result["d013_conclusion"],
                output.display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D014Commands {
    /// Reproduce D-013 TIMESTEP_FLOOR_FAILURE from the 150k checkpoint.
    FailureReplay {
        #[arg(long, default_value = "experiments/generated/d014/failure_replay")]
        output: PathBuf,
    },
    /// Diagnostic 150k→170k replay after numerical repair.
    DiagnosticReplay {
        #[arg(long, default_value = "experiments/generated/d014/diagnostic_checkpoint_replay")]
        output: PathBuf,
    },
    /// Fresh R22 preflight on repaired binary.
    Preflight {
        #[arg(long, default_value = "experiments/generated/d014/preflight")]
        output: PathBuf,
    },
    /// Fresh governed R22 reference after preflight.
    FreshR22 {
        #[arg(long, default_value = "experiments/generated/d014/fresh_reference_r22")]
        output: PathBuf,
    },
    /// Non-stiff 0–25k equivalence at 1×/0.5×/0.25× dt_cap.
    NonstiffEquivalence {
        #[arg(long, default_value = "experiments/generated/d014/nonstiff_equivalence")]
        output: PathBuf,
    },
}

fn resolve_d014_artifact_path(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        return path;
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(path)
}

fn run_d014(action: D014Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D014Commands::FailureReplay { output } => {
            let output = resolve_d014_artifact_path(output);
            let result = d014::run_failure_reproduction(&output)?;
            println!(
                "D-014 failure-replay: floor={} reproduced={} limiter={:?} -> {}",
                result["floor_failure"],
                result["reproduced_near_original"],
                result["terminal_limiter"],
                output.display()
            );
        }
        D014Commands::DiagnosticReplay { output } => {
            let output = resolve_d014_artifact_path(output);
            let result = d014::run_diagnostic_replay_170k(&output)?;
            println!(
                "D-014 diagnostic-replay: floor={} end_steps={} -> {}",
                result["floor_failure"],
                result["end_accepted_substeps"],
                output.display()
            );
        }
        D014Commands::Preflight { output } => {
            let output = resolve_d014_artifact_path(output);
            let result = d014::run_d014_preflight(&output)?;
            println!(
                "D-014 preflight: pass={} -> {}",
                result["preflight_pass"],
                output.display()
            );
        }
        D014Commands::FreshR22 { output } => {
            let output = resolve_d014_artifact_path(output);
            let result = d014::run_fresh_reference_r22(&output)?;
            println!(
                "D-014 fresh-r22: {:?} {:?} -> {}",
                result["termination_reason"],
                result["scientific_classification"],
                output.display()
            );
        }
        D014Commands::NonstiffEquivalence { output } => {
            let output = resolve_d014_artifact_path(output);
            let result = d014::run_nonstiff_equivalence(&output)?;
            println!(
                "D-014 nonstiff-equivalence: horizon={} -> {}",
                result["horizon_accepted"],
                output.display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D015Commands {
    Preserve {
        #[arg(long, default_value = "experiments/generated/d015/preservation")]
        output: PathBuf,
    },
    RegressionSummary {
        #[arg(long, default_value = "experiments/generated/d015/regressions")]
        output: PathBuf,
    },
    AnalyzeD014Checkpoint {
        #[arg(long, default_value = "experiments/generated/d015/d014_replay")]
        output: PathBuf,
    },
    Controls {
        #[arg(long, default_value = "experiments/generated/d015/controls")]
        output: PathBuf,
        #[arg(long, default_value_t = true)]
        repaired: bool,
    },
    Preflight {
        #[arg(long, default_value = "experiments/generated/d015/preflight")]
        output: PathBuf,
        #[arg(long, default_value_t = true)]
        repaired: bool,
    },
    FreshR22 {
        #[arg(long, default_value = "experiments/generated/d015/fresh_reference_r22")]
        output: PathBuf,
        #[arg(long, default_value_t = true)]
        repaired: bool,
    },
}

fn resolve_d015_artifact_path(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        return path;
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(path)
}

#[derive(Subcommand)]
enum D016Commands {
    Preserve {
        #[arg(long, default_value = "experiments/generated/d016/preservation")]
        output: PathBuf,
    },
    TransportAudit {
        #[arg(long, default_value = "experiments/generated/d016/transport_audit")]
        output: PathBuf,
    },
    SourceTimescales {
        #[arg(long, default_value = "experiments/generated/d016")]
        output: PathBuf,
    },
    FixedSourceCampaign {
        #[arg(long, default_value = "experiments/generated/d016")]
        output: PathBuf,
    },
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d016")]
        output: PathBuf,
    },
}

fn resolve_d016_artifact_path(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        return path;
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(path)
}

fn run_d016(action: D016Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D016Commands::Preserve { output } => {
            let output = resolve_d016_artifact_path(output);
            let result = d016::run_preserve(&output)?;
            println!("D-016 preserve -> {}", output.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D016Commands::TransportAudit { output } => {
            let output = resolve_d016_artifact_path(output);
            let result = d016::run_transport_audit(&output)?;
            println!("D-016 transport-audit -> {}", output.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D016Commands::SourceTimescales { output } => {
            let output = resolve_d016_artifact_path(output);
            let result = d016::run_source_and_timescales(&output)?;
            println!("D-016 source-timescales -> {}", output.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D016Commands::FixedSourceCampaign { output } => {
            let output = resolve_d016_artifact_path(output);
            let result = d016::run_fixed_source_campaign(&output)?;
            println!(
                "D-016 fixed-source-campaign conclusion={} -> {}",
                result["primary_conclusion"],
                output.display()
            );
        }
        D016Commands::Pipeline { output } => {
            let output = resolve_d016_artifact_path(output);
            let result = d016::run_pipeline(&output)?;
            println!(
                "D-016 pipeline conclusion={} -> {}",
                result["manifest"]["primary_conclusion"],
                output.display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D017Commands {
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d017")]
        output: PathBuf,
    },
}

fn resolve_d017_artifact_path(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        return path;
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(path)
}

fn run_d017(action: D017Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D017Commands::Pipeline { output } => {
            let output = resolve_d017_artifact_path(output);
            let result = d017::run_pipeline(&output)?;
            println!(
                "D-017 pipeline conclusion={:?} -> {}",
                result["primary_conclusion"],
                output.display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D018Commands {
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d018")]
        output: PathBuf,
    },
}

fn resolve_d018_artifact_path(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        return path;
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(path)
}

fn run_d018(action: D018Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D018Commands::Pipeline { output } => {
            let output = resolve_d018_artifact_path(output);
            let result = d018::run_pipeline(&output)?;
            println!(
                "D-018 pipeline conclusion={} tag={} -> {}",
                result["primary_conclusion"],
                result["terminal_tag"],
                output.display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D019Commands {
    /// D-019 structural scaling repair pipeline (mechanism selection + pre-balance).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d019")]
        output: PathBuf,
    },
    /// Governed Stage E reference at R=22 with v3 structural scaling equation.
    StageE {
        #[arg(long, default_value = "experiments/generated/d019/stage_e_reference")]
        output: PathBuf,
        #[arg(long, default_value = "200000")]
        max_steps: u64,
    },
    /// Neighbor radius validation at R=18 and R=26 after Stage E center.
    Neighbors {
        #[arg(long, default_value = "experiments/generated/d019/stage_e_reference/neighbors")]
        output: PathBuf,
        #[arg(long, default_value = "200000")]
        max_steps: u64,
    },
    /// Re-run foundational Stage B/C/D gates under v3 equation version.
    StagesBcd {
        #[arg(long, default_value = "experiments/generated/d019/stage_b_c_d")]
        output: PathBuf,
    },
}

fn resolve_d019_artifact_path(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        return path;
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(path)
}

#[derive(Subcommand)]
enum D020Commands {
    /// Full D-020 joint-rate Stage E recovery pipeline.
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d020")]
        output: PathBuf,
        #[arg(long, default_value = "200000")]
        max_steps: u64,
    },
    /// Stage A flow audit from D-019 R22 artifacts.
    StageA {
        #[arg(long, default_value = "experiments/generated/d020/stage_a_flow_audit")]
        output: PathBuf,
    },
    /// Stage B ±10% sensitivity and bounded candidates.
    StageB {
        #[arg(long, default_value = "experiments/generated/d020/stage_b_sensitivity")]
        output: PathBuf,
    },
    /// Stage C promote ≤2 candidates.
    StageC {
        #[arg(long, default_value = "experiments/generated/d020/stage_c_promotion")]
        output: PathBuf,
    },
    /// Stage D full R22 governed reference for promoted candidates.
    StageD {
        #[arg(long, default_value = "experiments/generated/d020/stage_d_full_r22")]
        output: PathBuf,
        #[arg(long, default_value = "200000")]
        max_steps: u64,
    },
    /// Stage E R18/R26 restoring confirmation.
    StageE {
        #[arg(long, default_value = "experiments/generated/d020/stage_e_neighbors")]
        output: PathBuf,
        #[arg(long, default_value = "200000")]
        max_steps: u64,
    },
    /// Stage A+B only (short local-response preconditioner).
    Precondition {
        #[arg(long, default_value = "experiments/generated/d020")]
        output: PathBuf,
    },
}

fn resolve_d020_artifact_path(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        return path;
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(path)
}

fn run_d020(action: D020Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D020Commands::Pipeline { output, max_steps } => {
            let output = resolve_d020_artifact_path(output);
            let result = d020::run_pipeline(&output, max_steps)?;
            println!(
                "D-020 conclusion={} -> {}",
                result["primary_conclusion"],
                output.display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D020Commands::StageA { output } => {
            let output = resolve_d020_artifact_path(output);
            let result = d020::run_stage_a_flow_audit(&output)?;
            println!("D-020 Stage A -> {}", output.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D020Commands::StageB { output } => {
            let output = resolve_d020_artifact_path(output);
            let result = d020::run_stage_b_sensitivity(&output)?;
            println!("D-020 Stage B -> {}", output.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D020Commands::StageC { output } => {
            let output = resolve_d020_artifact_path(output);
            let result = d020::run_stage_c_promote(&output)?;
            println!("D-020 Stage C -> {}", output.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D020Commands::StageD { output, max_steps } => {
            let output = resolve_d020_artifact_path(output);
            let result = d020::run_stage_d_full_r22(&output, max_steps)?;
            println!("D-020 Stage D -> {}", output.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D020Commands::StageE { output, max_steps } => {
            let output = resolve_d020_artifact_path(output);
            let result = d020::run_stage_e_neighbors(&output, max_steps)?;
            println!("D-020 Stage E -> {}", output.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D020Commands::Precondition { output } => {
            let output = resolve_d020_artifact_path(output);
            let result = d020::run_precondition_only(&output)?;
            println!("D-020 precondition -> {}", output.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

fn run_d019(action: D019Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D019Commands::Pipeline { output } => {
            let output = resolve_d019_artifact_path(output);
            let result = d019::run_pipeline(&output)?;
            println!(
                "D-019 pipeline conclusion={} -> {}",
                result["primary_conclusion"],
                output.display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D019Commands::StageE { output, max_steps } => {
            let output = resolve_d019_artifact_path(output);
            let result = d019::run_stage_e_reference(&output, max_steps)?;
            println!(
                "D-019 Stage E classification={} -> {}",
                result["scientific_classification"],
                output.display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D019Commands::Neighbors { output, max_steps } => {
            let output = resolve_d019_artifact_path(output);
            let result = d019::run_neighbor_radius_validation(&output, max_steps)?;
            println!(
                "D-019 neighbors -> {}",
                output.display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D019Commands::StagesBcd { output } => {
            let output = resolve_d019_artifact_path(output);
            let result = d019::run_stages_b_c_d(&output)?;
            println!("D-019 stages B/C/D -> {}", output.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

fn run_d015(action: D015Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D015Commands::Preserve { output } => {
            let output = resolve_d015_artifact_path(output);
            let result = d015::run_preserve(&output)?;
            println!("D-015 preserve -> {}", output.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D015Commands::RegressionSummary { output } => {
            let output = resolve_d015_artifact_path(output);
            let result = d015::run_regression_summary(&output)?;
            println!("D-015 regression-summary -> {}", output.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D015Commands::AnalyzeD014Checkpoint { output } => {
            let output = resolve_d015_artifact_path(output);
            let result = d015::run_analyze_d014_checkpoint(&output)?;
            println!("D-015 analyze-d014-checkpoint -> {}", output.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D015Commands::Controls { output, repaired } => {
            let output = resolve_d015_artifact_path(output);
            let result = d015::run_controls(&output, repaired)?;
            println!(
                "D-015 controls (repaired={repaired}) -> {}",
                output.display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D015Commands::Preflight { output, repaired } => {
            let output = resolve_d015_artifact_path(output);
            let result = d015::run_preflight(&output, repaired)?;
            println!(
                "D-015 preflight (repaired={repaired}) pass={} -> {}",
                result["preflight_pass"],
                output.display()
            );
        }
        D015Commands::FreshR22 { output, repaired } => {
            let output = resolve_d015_artifact_path(output);
            let result = d015::run_fresh_r22(&output, repaired)?;
            println!(
                "D-015 fresh-r22 (repaired={repaired}) {:?} -> {}",
                result["termination_reason"],
                output.display()
            );
        }
    }
    Ok(())
}

fn run_baseline_acceptance(
    config_path: &Path,
    steps: u64,
    seed: u64,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(output_dir)?;
    let mut config = load_experiment_config(config_path)?;
    config.seed = seed;
    config.params.random_seed = seed;
    config.substeps = steps;
    config.name = format!("baseline_seed_{seed}");

    let start = Instant::now();
    let started_at = unix_timestamp();
    let mut sim = Simulation::from_config(&config);
    sim.morphology_sample_interval = 100;

    let record_every = config.record_every.max(1);
    let mut ratio_log: Vec<serde_json::Value> = Vec::new();
    let initial_structure = total_mass(&sim.grid, &sim.fields.structure);
    let initial_catalyst = total_mass(&sim.grid, &sim.fields.catalyst);

    save_checkpoint(output_dir, 0, &sim, &config)?;

    for s in 0..steps {
        sim.apply_scheduled_interventions(&config.interventions);
        if !sim.step() {
            break;
        }
        if sim.substep % record_every == 0 {
            let diag = sim.current_diagnostics();
            sim.history.push(diag);
        }
        if RATIO_CHECKPOINTS.contains(&sim.substep) {
            ratio_log.push(serde_json::json!({
                "substep": sim.substep,
                "ratios": sim.detector.turnover_ratios(),
            }));
        }
        if CHECKPOINT_STEPS.contains(&sim.substep) && sim.substep > 0 {
            save_checkpoint(output_dir, sim.substep, &sim, &config)?;
        }
        let _ = s;
    }

    let wall = start.elapsed().as_secs_f64();
    sim.finish_timing(sim.substep);

    write_experiment_artifacts(output_dir, &config, &sim, AcceptanceMeta {
        commit_hash: git_commit_hash(),
        dirty: git_dirty(),
        started_at,
        ended_at: unix_timestamp(),
        wall_seconds: wall,
        initial_structure,
        initial_catalyst,
        ratio_log,
    })?;

    println!(
        "Baseline seed {seed}: {} substeps, {:?}, {:.1} s",
        sim.substep,
        sim.detector.last_classification,
        wall
    );
    Ok(())
}

fn run_full_acceptance(config_path: &Path, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(output)?;
    let mut seed_results = Vec::new();

    for seed in 1..=5 {
        let out = output.join(format!("baseline_seed_{seed}"));
        run_baseline_acceptance(config_path, 250_000, seed, &out)?;
        let summary: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(out.join("summary.json"))?)?;
        seed_results.push(summary);
    }

    let mut interventions = Vec::new();
    let intervention_configs = [
        ("starvation_nutrient", intervention_starvation_nutrient()),
        ("starvation_fuel", intervention_starvation_fuel()),
        ("catalyst_knockout", intervention_catalyst_knockout()),
        ("structure_knockout", intervention_structure_knockout()),
        ("reservoir_shutdown", intervention_reservoir_shutdown()),
    ];
    for (name, mut config) in intervention_configs {
        let out = output.join(name);
        fs::create_dir_all(&out)?;
        config.substeps = 250_000;
        let sim = run_experiment(&config, 1000);
        write_experiment_artifacts(
            &out,
            &config,
            &sim,
            AcceptanceMeta::minimal(),
        )?;
        interventions.push(serde_json::json!({
            "name": name,
            "classification": format!("{:?}", sim.detector.last_classification),
            "substeps": sim.substep,
        }));
    }

    for fraction in [10, 20, 30, 40, 50, 60, 70, 80] {
        let out = output.join(format!("damage_{fraction}pct"));
        fs::create_dir_all(&out)?;
        let config = damage_fraction_experiment(fraction);
        let sim = run_experiment(&config, 1000);
        write_experiment_artifacts(&out, &config, &sim, AcceptanceMeta::minimal())?;
    }

    let control_configs = [
        ("control_passive", control_passive()),
        ("control_no_catalyst_rep", control_no_catalyst_rep()),
        ("control_no_structure", control_no_structure()),
    ];
    for (name, config) in control_configs {
        let out = output.join(name);
        fs::create_dir_all(&out)?;
        let sim = run_experiment(&config, 1000);
        write_experiment_artifacts(&out, &config, &sim, AcceptanceMeta::minimal())?;
    }

    let manifest = build_manifest(output)?;
    fs::write(
        output.join("manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;

    let report = build_acceptance_report(&seed_results, &interventions);
    fs::write(
        PathBuf::from("docs/phase1_acceptance_report.md"),
        report,
    )?;

    println!("Full acceptance suite written to {}", output.display());
    Ok(())
}

struct AcceptanceMeta {
    commit_hash: String,
    dirty: bool,
    started_at: u64,
    ended_at: u64,
    wall_seconds: f64,
    initial_structure: f64,
    initial_catalyst: f64,
    ratio_log: Vec<serde_json::Value>,
}

impl AcceptanceMeta {
    fn minimal() -> Self {
        Self {
            commit_hash: git_commit_hash(),
            dirty: git_dirty(),
            started_at: unix_timestamp(),
            ended_at: unix_timestamp(),
            wall_seconds: 0.0,
            initial_structure: 0.0,
            initial_catalyst: 0.0,
            ratio_log: vec![],
        }
    }
}

fn save_checkpoint(
    out: &Path,
    step: u64,
    sim: &Simulation,
    config: &ExperimentConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = out.join(format!("checkpoint_{step:06}"));
    fs::create_dir_all(&dir)?;
    let snap = sim.snapshot();
    save_snapshot(&dir.join("snapshot.json"), &snap)?;
    for field in FIELD_NAMES {
        render_field_png(sim, &dir.join(format!("{field}.png")), field)?;
    }
    render_combined_png(sim, &dir.join("combined.png"))?;
    fs::write(dir.join("config.json"), serde_json::to_string_pretty(config)?)?;
    Ok(())
}

fn run_single(config_path: &Path, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_experiment_config(config_path)?;
    let out = output_dir.join(&config.name);
    fs::create_dir_all(&out)?;
    let sim = run_experiment(&config, config.record_every);
    write_experiment_artifacts(&out, &config, &sim, AcceptanceMeta::minimal())?;
    println!("Experiment {} complete -> {}", config.name, out.display());
    Ok(())
}

fn run_all_experiments(output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let experiments: Vec<(&str, ExperimentConfig)> = vec![
        ("baseline", baseline_experiment()),
        ("starvation_nutrient", starvation_nutrient()),
        ("starvation_fuel", starvation_fuel()),
        ("catalyst_knockout", catalyst_knockout()),
        ("structure_knockout", structure_knockout()),
        ("puncture_repair", puncture_repair()),
        ("catastrophic_damage", catastrophic_damage()),
        ("no_resurrection", no_resurrection()),
        ("static_control", static_control()),
    ];
    for (name, config) in experiments {
        let out = output.join(name);
        fs::create_dir_all(&out)?;
        let sim = run_experiment(&config, config.record_every);
        write_experiment_artifacts(&out, &config, &sim, AcceptanceMeta::minimal())?;
        println!("{name}: classification {:?}", sim.detector.last_classification);
    }
    Ok(())
}

fn substeps() -> u64 {
    #[cfg(feature = "long-experiments")]
    {
        250_000
    }
    #[cfg(not(feature = "long-experiments"))]
    {
        8_000
    }
}

fn baseline_experiment() -> ExperimentConfig {
    ExperimentConfig {
        name: "baseline".into(),
        seed: 1,
        substeps: substeps(),
        params: baseline_params(),
        interventions: vec![],
        record_every: 500,
    }
}

fn intervention_starvation_nutrient() -> ExperimentConfig {
    ExperimentConfig {
        name: "starvation_nutrient".into(),
        seed: 1,
        substeps: 250_000,
        params: baseline_params(),
        interventions: vec![InterventionSpec::AtSubstep {
            substep: 50_000,
            action: InterventionAction::RemoveNutrient,
        }],
        record_every: 1000,
    }
}

fn intervention_starvation_fuel() -> ExperimentConfig {
    ExperimentConfig {
        name: "starvation_fuel".into(),
        seed: 1,
        substeps: 250_000,
        params: baseline_params(),
        interventions: vec![InterventionSpec::AtSubstep {
            substep: 50_000,
            action: InterventionAction::RemoveFuel,
        }],
        record_every: 1000,
    }
}

fn intervention_catalyst_knockout() -> ExperimentConfig {
    ExperimentConfig {
        name: "catalyst_knockout".into(),
        seed: 1,
        substeps: 250_000,
        params: baseline_params(),
        interventions: vec![InterventionSpec::AtSubstep {
            substep: 50_000,
            action: InterventionAction::DisableCatalystReproduction,
        }],
        record_every: 1000,
    }
}

fn intervention_structure_knockout() -> ExperimentConfig {
    ExperimentConfig {
        name: "structure_knockout".into(),
        seed: 1,
        substeps: 250_000,
        params: baseline_params(),
        interventions: vec![InterventionSpec::AtSubstep {
            substep: 50_000,
            action: InterventionAction::DisableStructuralSynthesis,
        }],
        record_every: 1000,
    }
}

fn intervention_reservoir_shutdown() -> ExperimentConfig {
    ExperimentConfig {
        name: "reservoir_shutdown".into(),
        seed: 1,
        substeps: 250_000,
        params: baseline_params(),
        interventions: vec![InterventionSpec::AtSubstep {
            substep: 50_000,
            action: InterventionAction::ShutdownReservoir,
        }],
        record_every: 1000,
    }
}

fn damage_fraction_experiment(fraction: u32) -> ExperimentConfig {
    ExperimentConfig {
        name: format!("damage_{fraction}pct"),
        seed: 1,
        substeps: 250_000,
        params: baseline_params(),
        interventions: vec![
            InterventionSpec::AtSubstep {
                substep: 50_000,
                action: InterventionAction::DamageFraction {
                    fraction: fraction as f64 / 100.0,
                },
            },
        ],
        record_every: 1000,
    }
}

fn control_passive() -> ExperimentConfig {
    ExperimentConfig {
        name: "control_passive".into(),
        seed: 1,
        substeps: substeps(),
        params: passive_phase_params(),
        interventions: vec![InterventionSpec::AtSubstep {
            substep: 0,
            action: InterventionAction::DisableAllReactions,
        }],
        record_every: 1000,
    }
}

fn control_no_catalyst_rep() -> ExperimentConfig {
    ExperimentConfig {
        name: "control_no_catalyst_rep".into(),
        seed: 1,
        substeps: substeps(),
        params: {
            let mut p = baseline_params();
            p.k_rep = 0.0;
            p
        },
        interventions: vec![],
        record_every: 1000,
    }
}

fn control_no_structure() -> ExperimentConfig {
    ExperimentConfig {
        name: "control_no_structure".into(),
        seed: 1,
        substeps: substeps(),
        params: {
            let mut p = baseline_params();
            p.k_structure = 0.0;
            p
        },
        interventions: vec![],
        record_every: 1000,
    }
}

fn starvation_nutrient() -> ExperimentConfig {
    let mut params = baseline_params();
    params.n_reservoir = 0.0;
    ExperimentConfig {
        name: "starvation_nutrient".into(),
        seed: 1,
        substeps: substeps(),
        params,
        interventions: vec![],
        record_every: 500,
    }
}

fn starvation_fuel() -> ExperimentConfig {
    let mut params = baseline_params();
    params.f_reservoir = 0.0;
    ExperimentConfig {
        name: "starvation_fuel".into(),
        seed: 1,
        substeps: substeps(),
        params,
        interventions: vec![],
        record_every: 500,
    }
}

fn catalyst_knockout() -> ExperimentConfig {
    let mut params = baseline_params();
    params.k_rep = 0.0;
    ExperimentConfig {
        name: "catalyst_knockout".into(),
        seed: 1,
        substeps: substeps(),
        params,
        interventions: vec![],
        record_every: 500,
    }
}

fn structure_knockout() -> ExperimentConfig {
    let mut params = baseline_params();
    params.k_structure = 0.0;
    ExperimentConfig {
        name: "structure_knockout".into(),
        seed: 1,
        substeps: substeps(),
        params,
        interventions: vec![],
        record_every: 500,
    }
}

fn puncture_repair() -> ExperimentConfig {
    ExperimentConfig {
        name: "puncture_repair".into(),
        seed: 1,
        substeps: substeps(),
        params: baseline_params(),
        interventions: vec![InterventionSpec::AtSubstep {
            substep: 50_000,
            action: InterventionAction::PunctureRepair,
        }],
        record_every: 500,
    }
}

fn catastrophic_damage() -> ExperimentConfig {
    ExperimentConfig {
        name: "catastrophic_damage".into(),
        seed: 1,
        substeps: substeps(),
        params: baseline_params(),
        interventions: vec![InterventionSpec::AtSubstep {
            substep: 50_000,
            action: InterventionAction::CatastrophicDamage,
        }],
        record_every: 500,
    }
}

fn no_resurrection() -> ExperimentConfig {
    ExperimentConfig {
        name: "no_resurrection".into(),
        seed: 1,
        substeps: substeps() * 2,
        params: baseline_params(),
        interventions: vec![
            InterventionSpec::AtSubstep {
                substep: 50_000,
                action: InterventionAction::RemoveNutrient,
            },
            InterventionSpec::AtSubstep {
                substep: 50_000,
                action: InterventionAction::RemoveFuel,
            },
            InterventionSpec::AtSubstep {
                substep: substeps(),
                action: InterventionAction::RestoreReservoir,
            },
        ],
        record_every: 500,
    }
}

fn static_control() -> ExperimentConfig {
    ExperimentConfig {
        name: "static_control".into(),
        seed: 1,
        substeps: substeps(),
        params: static_control_params(),
        interventions: vec![InterventionSpec::AtSubstep {
            substep: 100,
            action: InterventionAction::DisableAllReactions,
        }],
        record_every: 500,
    }
}

fn load_experiment_config(path: &Path) -> Result<ExperimentConfig, Box<dyn std::error::Error>> {
    let data = fs::read_to_string(path)?;
    let config: ExperimentConfig = toml::from_str(&data)?;
    Ok(config)
}

fn write_experiment_artifacts(
    out: &Path,
    config: &ExperimentConfig,
    sim: &Simulation,
    meta: AcceptanceMeta,
) -> Result<(), Box<dyn std::error::Error>> {
    let snap = sim.snapshot();
    save_snapshot(&out.join("snapshot.json"), &snap)?;
    fs::write(out.join("config.json"), serde_json::to_string_pretty(config)?)?;

    let mut csv = String::from(
        "substep,sim_time,dt,structural_mass,catalyst_mass,retention,classification,struct_rep,cat_rep\n",
    );
    for d in &sim.history {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{:?},{},{}\n",
            d.substep,
            d.sim_time,
            d.dt,
            d.structural_mass,
            d.catalyst_mass,
            d.catalyst_retention,
            d.classification,
            d.turnover_ratios.structural_replacement,
            d.turnover_ratios.catalyst_replacement,
        ));
    }
    fs::write(out.join("diagnostics.csv"), csv)?;

    let ratios = sim.detector.turnover_ratios();
    let summary = serde_json::json!({
        "experiment": config.name,
        "seed": config.seed,
        "substeps": sim.substep,
        "classification": format!("{:?}", sim.detector.last_classification),
        "structural_mass": total_mass(&sim.grid, &sim.fields.structure),
        "catalyst_mass": total_mass(&sim.grid, &sim.fields.catalyst),
        "initial_structural_mass": meta.initial_structure,
        "initial_catalyst_mass": meta.initial_catalyst,
        "turnover_ratios": ratios,
        "ratio_checkpoints": meta.ratio_log,
        "rejection_count": sim.rejection_count,
        "min_dt": sim.min_dt_seen,
        "turnover": sim.detector.turnover,
        "accounting": sim.accounting,
        "timing": sim.timing,
        "version": SIM_VERSION,
        "commit_hash": meta.commit_hash,
        "dirty_working_tree": meta.dirty,
        "rustc_version": rustc_version(),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "started_at": meta.started_at,
        "ended_at": meta.ended_at,
        "wall_seconds": meta.wall_seconds,
        "interventions": sim.interventions_applied,
        "consecutive_viable_windows": sim.detector.consecutive_viable_windows,
        "accounting_within_tolerance": sim.accounting.cumulative_within_tolerance(),
    });
    fs::write(out.join("summary.json"), serde_json::to_string_pretty(&summary)?)?;

    for field in FIELD_NAMES {
        render_field_png(sim, &out.join(format!("{field}_final.png")), field)?;
    }
    render_combined_png(sim, &out.join("combined_final.png"))?;
    Ok(())
}

fn render_field_png(sim: &Simulation, path: &Path, field: &str) -> Result<(), Box<dyn std::error::Error>> {
    let data = field_slice(&sim.fields, field).unwrap_or(&sim.fields.structure);
    let w = sim.grid.width;
    let h = sim.grid.height;
    let mut img = image::RgbImage::new(w as u32, h as u32);
    let mut max_v = 1e-6f64;
    for &v in data {
        if v > max_v {
            max_v = v;
        }
    }
    for j in 0..h {
        for i in 0..w {
            let idx = Grid::index(w, i, j);
            let v = if sim.grid.in_dish(idx) {
                (data[idx] / max_v).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let c = (v * 255.0) as u8;
            img.put_pixel(i as u32, j as u32, image::Rgb([c, c / 2, c / 3]));
        }
    }
    img.save(path)?;
    Ok(())
}

fn render_combined_png(sim: &Simulation, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    render_field_png(sim, path, "structure")
}

fn build_manifest(base: &Path) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut files = HashMap::new();
    collect_hashes(base, base, &mut files)?;
    Ok(serde_json::json!({
        "generated_at": unix_timestamp(),
        "commit_hash": git_commit_hash(),
        "files": files,
    }))
}

fn collect_hashes(
    root: &Path,
    dir: &Path,
    out: &mut HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_hashes(root, &path, out)?;
        } else if path.file_name().and_then(|s| s.to_str()) != Some("manifest.json") {
            let rel = path.strip_prefix(root)?.to_string_lossy().to_string();
            out.insert(rel, file_hash(&path)?);
        }
    }
    Ok(())
}

fn file_hash(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let data = fs::read(path)?;
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    Ok(format!("{:016x}", hasher.finish()))
}

fn build_acceptance_report(
    seeds: &[serde_json::Value],
    interventions: &[serde_json::Value],
) -> String {
    let mut w = Vec::new();
    writeln!(w, "# Phase 1 Acceptance Report").ok();
    writeln!(w, "\nGenerated by experiment-runner acceptance suite.\n").ok();
    writeln!(w, "## Baseline seeds\n").ok();
    for s in seeds {
        writeln!(w, "- {s}").ok();
    }
    writeln!(w, "\n## Interventions\n").ok();
    for i in interventions {
        writeln!(w, "- {i}").ok();
    }
    String::from_utf8(w).unwrap_or_default()
}

fn git_commit_hash() -> String {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

fn git_dirty() -> bool {
    Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(true)
}

fn rustc_version() -> String {
    Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

const SWEEP_PARAMS: [&str; 7] = [
    "k_rep",
    "k_structure",
    "k_structure_decay",
    "k_catalyst_decay_inside",
    "d_c_inside",
    "mobility_m",
    "kappa",
];

const SWEEP_FACTORS: [f64; 5] = [0.5, 0.75, 1.0, 1.25, 1.5];

fn run_sweep(_config_path: &Path, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(output)?;
    let mut results = Vec::new();
    for param in SWEEP_PARAMS {
        for factor in SWEEP_FACTORS {
            let mut params = baseline_params();
            params = params.scaled(param, factor);
            let config = ExperimentConfig {
                name: format!("{param}_{factor}"),
                seed: 1,
                substeps: 10_000,
                params,
                interventions: vec![],
                record_every: 1000,
            };
            let sim = run_experiment(&config, 1000);
            let mass = total_mass(&sim.grid, &sim.fields.structure);
            let cat = total_mass(&sim.grid, &sim.fields.catalyst);
            let outcome = classify_sweep_outcome(&sim, mass, cat);
            results.push(serde_json::json!({
                "param": param,
                "factor": factor,
                "outcome": outcome,
                "structural_mass": mass,
                "catalyst_mass": cat,
                "classification": format!("{:?}", sim.detector.last_classification),
            }));
        }
    }
    fs::write(output.join("sweep_results.json"), serde_json::to_string_pretty(&results)?)?;
    Ok(())
}

fn classify_sweep_outcome(sim: &Simulation, mass: f64, cat: f64) -> &'static str {
    if sim.rejection_count > 100 {
        "numerical_instability"
    } else if mass < 5.0 && cat < 0.05 {
        "immediate_collapse"
    } else if mass > 8000.0 {
        "unbounded_growth"
    } else if cat < 0.1 && mass > 100.0 {
        "catalyst_escape"
    } else if sim.detector.turnover.nutrient_consumption < 1.0 {
        "static_non_turning_droplet"
    } else if mass > 50.0 && cat > 0.1 {
        "stable_active_protocell"
    } else {
        "slow_collapse"
    }
}
