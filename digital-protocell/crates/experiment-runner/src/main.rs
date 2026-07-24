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
mod d021;
mod d022;
mod d023;
mod d024;
mod d025;
mod d025_stage_e;
mod d026;
mod d027;
mod d028;
mod d029;
mod d030;
mod d031;
mod d032;
mod d033;
mod d034;
mod d035;
mod d036;
mod d037;
mod d038;
mod d039;
mod d040;
mod d041;
mod d042;
mod d043;
mod d044;
mod d045;
mod d046;
mod d047;
mod d048;
mod d049;
mod d050;
mod d051;
mod d052;
mod d053;
mod d055;
mod d056;
mod d057;
mod d058;
mod d059;
mod d060;
mod d061;
mod d062;
mod d063;
mod d064;
mod d065;
mod d066;
mod d067;
mod d068;
mod d069;
mod d070;
mod d071;
mod d072;
mod d073;
mod d074;
mod d075;
mod d076;
mod d077;
mod d078;
mod d079;
mod d080;
mod d081;
mod d082;
mod d083;
mod d084;
mod d085;

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
    D021 {
        #[command(subcommand)]
        action: D021Commands,
    },
    D022 {
        #[command(subcommand)]
        action: D022Commands,
    },
    D023 {
        #[command(subcommand)]
        action: D023Commands,
    },
    D024 {
        #[command(subcommand)]
        action: D024Commands,
    },
    D025 {
        #[command(subcommand)]
        action: D025Commands,
    },
    D026 {
        #[command(subcommand)]
        action: D026Commands,
    },
    D027 {
        #[command(subcommand)]
        action: D027Commands,
    },
    D028 {
        #[command(subcommand)]
        action: D028Commands,
    },
    D029 {
        #[command(subcommand)]
        action: D029Commands,
    },
    D030 {
        #[command(subcommand)]
        action: D030Commands,
    },
    D031 {
        #[command(subcommand)]
        action: D031Commands,
    },
    D032 {
        #[command(subcommand)]
        action: D032Commands,
    },
    D033 {
        #[command(subcommand)]
        action: D033Commands,
    },
    D034 {
        #[command(subcommand)]
        action: D034Commands,
    },
    D035 {
        #[command(subcommand)]
        action: D035Commands,
    },
    D036 {
        #[command(subcommand)]
        action: D036Commands,
    },
    D037 {
        #[command(subcommand)]
        action: D037Commands,
    },
    D038 {
        #[command(subcommand)]
        action: D038Commands,
    },
    D039 {
        #[command(subcommand)]
        action: D039Commands,
    },
    D040 {
        #[command(subcommand)]
        action: D040Commands,
    },
    D041 {
        #[command(subcommand)]
        action: D041Commands,
    },
    D042 {
        #[command(subcommand)]
        action: D042Commands,
    },
    D043 {
        #[command(subcommand)]
        action: D043Commands,
    },
    D044 {
        #[command(subcommand)]
        action: D044Commands,
    },
    D045 {
        #[command(subcommand)]
        action: D045Commands,
    },
    D046 {
        #[command(subcommand)]
        action: D046Commands,
    },
    D047 {
        #[command(subcommand)]
        action: D047Commands,
    },
    D048 {
        #[command(subcommand)]
        action: D048Commands,
    },
    D049 {
        #[command(subcommand)]
        action: D049Commands,
    },
    D050 {
        #[command(subcommand)]
        action: D050Commands,
    },
    D051 {
        #[command(subcommand)]
        action: D051Commands,
    },
    D052 {
        #[command(subcommand)]
        action: D052Commands,
    },
    D053 {
        #[command(subcommand)]
        action: D053Commands,
    },
    D055 {
        #[command(subcommand)]
        action: D055Commands,
    },
    D056 {
        #[command(subcommand)]
        action: D056Commands,
    },
    D057 {
        #[command(subcommand)]
        action: D057Commands,
    },
    D058 {
        #[command(subcommand)]
        action: D058Commands,
    },
    D059 {
        #[command(subcommand)]
        action: D059Commands,
    },
    D060 {
        #[command(subcommand)]
        action: D060Commands,
    },
    D061 {
        #[command(subcommand)]
        action: D061Commands,
    },
    D062 {
        #[command(subcommand)]
        action: D062Commands,
    },
    D063 {
        #[command(subcommand)]
        action: D063Commands,
    },
    D064 {
        #[command(subcommand)]
        action: D064Commands,
    },
    D065 {
        #[command(subcommand)]
        action: D065Commands,
    },
    D066 {
        #[command(subcommand)]
        action: D066Commands,
    },
    D067 {
        #[command(subcommand)]
        action: D067Commands,
    },
    D068 {
        #[command(subcommand)]
        action: D068Commands,
    },
    D069 {
        #[command(subcommand)]
        action: D069Commands,
    },
    D070 {
        #[command(subcommand)]
        action: D070Commands,
    },
    D071 {
        #[command(subcommand)]
        action: D071Commands,
    },
    D072 {
        #[command(subcommand)]
        action: D072Commands,
    },
    D073 {
        #[command(subcommand)]
        action: D073Commands,
    },
    D074 {
        #[command(subcommand)]
        action: D074Commands,
    },
    D075 {
        #[command(subcommand)]
        action: D075Commands,
    },
    D076 {
        #[command(subcommand)]
        action: D076Commands,
    },
    D077 {
        #[command(subcommand)]
        action: D077Commands,
    },
    D078 {
        #[command(subcommand)]
        action: D078Commands,
    },
    D079 {
        #[command(subcommand)]
        action: D079Commands,
    },
    D080 {
        #[command(subcommand)]
        action: D080Commands,
    },
    D081 {
        #[command(subcommand)]
        action: D081Commands,
    },
    D082 {
        #[command(subcommand)]
        action: D082Commands,
    },
    D083 {
        #[command(subcommand)]
        action: D083Commands,
    },
    D084 {
        #[command(subcommand)]
        action: D084Commands,
    },
    D085 {
        #[command(subcommand)]
        action: D085Commands,
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
        Commands::D021 { action } => run_d021(action)?,
        Commands::D022 { action } => run_d022(action)?,
        Commands::D023 { action } => run_d023(action)?,
        Commands::D024 { action } => run_d024(action)?,
        Commands::D025 { action } => run_d025(action)?,
        Commands::D026 { action } => run_d026(action)?,
        Commands::D027 { action } => run_d027(action)?,
        Commands::D028 { action } => run_d028(action)?,
        Commands::D029 { action } => run_d029(action)?,
        Commands::D030 { action } => run_d030(action)?,
        Commands::D031 { action } => run_d031(action)?,
        Commands::D032 { action } => run_d032(action)?,
        Commands::D033 { action } => run_d033(action)?,
        Commands::D034 { action } => run_d034(action)?,
        Commands::D035 { action } => run_d035(action)?,
        Commands::D036 { action } => run_d036(action)?,
        Commands::D037 { action } => run_d037(action)?,
        Commands::D038 { action } => run_d038(action)?,
        Commands::D039 { action } => run_d039(action)?,
        Commands::D040 { action } => run_d040(action)?,
        Commands::D041 { action } => run_d041(action)?,
        Commands::D042 { action } => run_d042(action)?,
        Commands::D043 { action } => run_d043(action)?,
        Commands::D044 { action } => run_d044(action)?,
        Commands::D045 { action } => run_d045(action)?,
        Commands::D046 { action } => run_d046(action)?,
        Commands::D047 { action } => run_d047(action)?,
        Commands::D048 { action } => run_d048(action)?,
        Commands::D049 { action } => run_d049(action)?,
        Commands::D050 { action } => run_d050(action)?,
        Commands::D051 { action } => run_d051(action)?,
        Commands::D052 { action } => run_d052(action)?,
        Commands::D053 { action } => run_d053(action)?,
        Commands::D055 { action } => run_d055(action)?,
        Commands::D056 { action } => run_d056(action)?,
        Commands::D057 { action } => run_d057(action)?,
        Commands::D058 { action } => run_d058(action)?,
        Commands::D059 { action } => run_d059(action)?,
        Commands::D060 { action } => run_d060(action)?,
        Commands::D061 { action } => run_d061(action)?,
        Commands::D062 { action } => run_d062(action)?,
        Commands::D063 { action } => run_d063(action)?,
        Commands::D064 { action } => run_d064(action)?,
        Commands::D065 { action } => run_d065(action)?,
        Commands::D066 { action } => run_d066(action)?,
        Commands::D067 { action } => run_d067(action)?,
        Commands::D068 { action } => run_d068(action)?,
        Commands::D069 { action } => run_d069(action)?,
        Commands::D070 { action } => run_d070(action)?,
        Commands::D071 { action } => run_d071(action)?,
        Commands::D072 { action } => run_d072(action)?,
        Commands::D073 { action } => run_d073(action)?,
        Commands::D074 { action } => run_d074(action)?,
        Commands::D075 { action } => run_d075(action)?,
        Commands::D076 { action } => run_d076(action)?,
        Commands::D077 { action } => run_d077(action)?,
        Commands::D078 { action } => run_d078(action)?,
        Commands::D079 { action } => run_d079(action)?,
        Commands::D080 { action } => run_d080(action)?,
        Commands::D081 { action } => run_d081(action)?,
        Commands::D082 { action } => run_d082(action)?,
        Commands::D083 { action } => run_d083(action)?,
        Commands::D084 { action } => run_d084(action)?,
        Commands::D085 { action } => run_d085(action)?,
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

#[derive(Subcommand)]
enum D021Commands {
    /// Full D-021 retention/localization repair pipeline (Gates 1–5).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d021")]
        output: PathBuf,
        #[arg(long, default_value = "200000")]
        max_steps: u64,
    },
    /// Gate 1 ε screen + Stage B localization.
    Gate1 {
        #[arg(long, default_value = "experiments/generated/d021/gate1")]
        output: PathBuf,
    },
    /// Gate 2 fixed-compartment Stage D for Gate-1 passers.
    Gate2 {
        #[arg(long, default_value = "experiments/generated/d021/gate2")]
        output: PathBuf,
        #[arg(long, default_value = "experiments/generated/d021/gate1/gate1_eps_screen.json")]
        gate1: PathBuf,
    },
    /// Gate 3 R22 pre-balance promotion.
    Gate3 {
        #[arg(long, default_value = "experiments/generated/d021/gate3")]
        output: PathBuf,
        #[arg(long, default_value = "experiments/generated/d021/gate2/gate2_fixed_compartment.json")]
        gate2: PathBuf,
    },
    /// Gate 4 bounded joint-rate recovery.
    Gate4 {
        #[arg(long, default_value = "experiments/generated/d021/gate4")]
        output: PathBuf,
        #[arg(long)]
        eps_m: f64,
    },
    /// Gate 5 full Stage E R22 + R18/R26.
    Gate5 {
        #[arg(long, default_value = "experiments/generated/d021/gate5")]
        output: PathBuf,
        #[arg(long)]
        eps_m: f64,
        #[arg(long, default_value = "200000")]
        max_steps: u64,
    },
}

#[derive(Subcommand)]
enum D022Commands {
    /// Full D-022 interface-affinity localization pipeline (Gates 1–4).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d022")]
        output: PathBuf,
        #[arg(long, default_value = "200000")]
        max_steps: u64,
    },
    /// Gate 1 transport integrity (unit-backed).
    Gate1 {
        #[arg(long, default_value = "experiments/generated/d022/gate1")]
        output: PathBuf,
    },
    /// Gate 2 Stage B + short R22 χ_M/D_M screen.
    Gate2 {
        #[arg(long, default_value = "experiments/generated/d022/gate2")]
        output: PathBuf,
    },
    /// Gate 3 fixed-compartment Stage D for promoted χ_M.
    Gate3 {
        #[arg(long, default_value = "experiments/generated/d022/gate3")]
        output: PathBuf,
        #[arg(long)]
        chi_m: f64,
    },
    /// Gate 4 bounded joint-rate Stage E recovery.
    Gate4 {
        #[arg(long, default_value = "experiments/generated/d022/gate4")]
        output: PathBuf,
        #[arg(long)]
        chi_m: f64,
        #[arg(long, default_value = "200000")]
        max_steps: u64,
    },
}

#[derive(Subcommand)]
enum D023Commands {
    /// Full D-023 precursor-assembly pipeline (Gates 0–2 decisive isolated).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d023")]
        output: PathBuf,
    },
    /// Gate 0 schema + preservation summary.
    Gate0 {
        #[arg(long, default_value = "experiments/generated/d023/gate0")]
        output: PathBuf,
    },
    /// Gate 1 conservation + causal chemistry summary.
    Gate1 {
        #[arg(long, default_value = "experiments/generated/d023/gate1")]
        output: PathBuf,
    },
    /// Gate 2 isolated assembly + localization (analytical k_assembly + 0.5/1/2× screen).
    Gate2 {
        #[arg(long, default_value = "experiments/generated/d023/gate2")]
        output: PathBuf,
    },
}

#[derive(Subcommand)]
enum D024Commands {
    /// Full D-024 interfacial surface-density pipeline (Gates 0–6).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d024")]
        output: PathBuf,
    },
    Gate0 {
        #[arg(long, default_value = "experiments/generated/d024/preservation")]
        output: PathBuf,
    },
    Gate1 {
        #[arg(long, default_value = "experiments/generated/d024/interface_measure")]
        output: PathBuf,
    },
    Gate2 {
        #[arg(long, default_value = "experiments/generated/d024/passive_surface")]
        output: PathBuf,
    },
    Gate3 {
        #[arg(long, default_value = "experiments/generated/d024/adsorption")]
        output: PathBuf,
    },
    Gate4 {
        #[arg(long, default_value = "experiments/generated/d024/selective_transport")]
        output: PathBuf,
    },
    Gate5 {
        #[arg(long, default_value = "experiments/generated/d024/moving_interface")]
        output: PathBuf,
    },
    Gate6 {
        #[arg(long, default_value = "experiments/generated/d024/R22_bootstrap")]
        output: PathBuf,
        #[arg(long, default_value_t = true)]
        assume_prior_gates_pass: bool,
    },
}

fn resolve_d024_artifact_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(path)
}

fn run_d024(action: D024Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D024Commands::Pipeline { output } => {
            let output = resolve_d024_artifact_path(&output);
            let result = d024::run_pipeline(&output)?;
            println!(
                "D-024 conclusion={} -> {}",
                result["primary_conclusion"],
                output.display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D024Commands::Gate0 { output } => {
            let out = resolve_d024_artifact_path(&output);
            let result = d024::run_gate0_preservation(&out)?;
            println!("D-024 Gate0 -> {}", out.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D024Commands::Gate1 { output } => {
            let out = resolve_d024_artifact_path(&output);
            let result = d024::run_gate1_interface_measure(&out)?;
            println!("D-024 Gate1 -> {}", out.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D024Commands::Gate2 { output } => {
            let out = resolve_d024_artifact_path(&output);
            let result = d024::run_gate2_passive_surface(&out)?;
            println!(
                "D-024 Gate2 pass={} -> {}",
                result["gate2_pass"],
                out.display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D024Commands::Gate3 { output } => {
            let out = resolve_d024_artifact_path(&output);
            let result = d024::run_gate3_adsorption(&out)?;
            println!(
                "D-024 Gate3 any_pass={} promoted_k_ads={} -> {}",
                result["any_pass"], result["promoted_k_ads"], out.display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D024Commands::Gate4 { output } => {
            let out = resolve_d024_artifact_path(&output);
            let result = d024::run_gate4_selective_transport(&out)?;
            println!("D-024 Gate4 pass={} -> {}", result["gate4_pass"], out.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D024Commands::Gate5 { output } => {
            let out = resolve_d024_artifact_path(&output);
            let result = d024::run_gate5_moving_interface(&out)?;
            println!("D-024 Gate5 pass={} -> {}", result["gate5_pass"], out.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D024Commands::Gate6 {
            output,
            assume_prior_gates_pass,
        } => {
            let out = resolve_d024_artifact_path(&output);
            let result = d024::run_gate6_r22_bootstrap(
                &out,
                assume_prior_gates_pass,
                None,
            )?;
            println!("D-024 Gate6 pass={} -> {}", result["gate6_pass"], out.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D025Commands {
    /// Gates 3–6 pipeline; stops at first failed gate.
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d025")]
        output: PathBuf,
    },
    Gate3 {
        #[arg(long, default_value = "experiments/generated/d025/growth_shrinkage")]
        output: PathBuf,
    },
    StageB {
        #[arg(long, default_value = "experiments/generated/d025/stage_b_regression")]
        output: PathBuf,
    },
    StageC {
        #[arg(long, default_value = "experiments/generated/d025/stage_c_regression")]
        output: PathBuf,
    },
    StageD {
        #[arg(long, default_value = "experiments/generated/d025/stage_d_fixed_compartment")]
        output: PathBuf,
    },
    DynamicR22 {
        #[arg(long, default_value = "experiments/generated/d025/dynamic_r22")]
        output: PathBuf,
    },
    StageEReference {
        #[arg(long, default_value = "experiments/generated/d025/stage_e_reference")]
        output: PathBuf,
    },
    StageEDiagnostic {
        #[arg(long, default_value = "experiments/generated/d025/stage_e_reference")]
        output: PathBuf,
    },
    StageESolve {
        #[arg(long, default_value = "experiments/generated/d025/stage_e_candidates")]
        output: PathBuf,
        #[arg(long, default_value = "experiments/generated/d025/stage_e_reference")]
        reference: PathBuf,
    },
}

#[derive(Subcommand)]
enum D026Commands {
    Gate0 {
        #[arg(long, default_value = "experiments/generated/d026/runner_parity")]
        output: PathBuf,
    },
    Gate1 {
        #[arg(long, default_value = "experiments/generated/d026")]
        output: PathBuf,
    },
    Gate2 {
        #[arg(long, default_value = "experiments/generated/d026")]
        output: PathBuf,
    },
    Gate5 {
        #[arg(long, default_value = "experiments/generated/d026")]
        output: PathBuf,
    },
    Classify {
        #[arg(long, default_value = "experiments/generated/d026")]
        output: PathBuf,
    },
}

fn resolve_d026_artifact_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn run_d026(action: D026Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D026Commands::Gate0 { output } => {
            let out = resolve_d026_artifact_path(&output);
            let result = d026::run_gate0_parity(&out)?;
            println!(
                "D-026 gate0 -> {} (pass={})",
                out.join("gate0_parity.json").display(),
                result["gate0_pass"]
            );
        }
        D026Commands::Gate1 { output } => {
            let out = resolve_d026_artifact_path(&output);
            let result = d026::run_gate1_observability_demo(&out)?;
            println!(
                "D-026 gate1 -> {} (samples={}/{})",
                out.join("gate1_observability.json").display(),
                result["surface_sample_count"],
                result["budget_sample_count"]
            );
        }
        D026Commands::Gate2 { output } => {
            let out = resolve_d026_artifact_path(&output);
            let result = d026::run_gate2_reference_history(&out)?;
            println!(
                "D-026 gate2 -> {} (divergence={})",
                out.join("reference_history/chronology.json").display(),
                result["earliest_divergence"]
            );
        }
        D026Commands::Gate5 { output } => {
            let out = resolve_d026_artifact_path(&output);
            let result = d026::run_gate5_causal_controls(&out)?;
            println!(
                "D-026 gate5 -> {} (controls={})",
                out.join("causal_controls/summary.json").display(),
                result["controls"].as_array().map(|a| a.len()).unwrap_or(0)
            );
        }
        D026Commands::Classify { output } => {
            let out = resolve_d026_artifact_path(&output);
            let result = d026::run_gate6_classification(&out)?;
            println!(
                "D-026 classify -> {} (mechanism={})",
                out.join("late_time_classification/classification.json").display(),
                result["gate6_mechanism"]
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D027Commands {
    /// Gates 0–4 early pipeline (ledger, basis, candidates, isolated surface).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d027")]
        output: PathBuf,
    },
    Gate0 {
        #[arg(long, default_value = "experiments/generated/d027/ledger_restore")]
        output: PathBuf,
    },
    Gate1 {
        #[arg(long, default_value = "experiments/generated/d027/adsorption_basis")]
        output: PathBuf,
    },
}

#[derive(Subcommand)]
enum D028Commands {
    /// Preservation + Gates 0–3 (bracket reproduce, root solve, robustness, portability).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d028")]
        output: PathBuf,
    },
    Gate0 {
        #[arg(long, default_value = "experiments/generated/d028/bracket_reproduction")]
        output: PathBuf,
    },
    Gate1 {
        #[arg(long, default_value = "experiments/generated/d028/root_iterations")]
        output: PathBuf,
    },
    /// Re-run Gate 3 portability using selected_k from prior root_solve / flag.
    Gate3 {
        #[arg(long, default_value = "experiments/generated/d028/portability")]
        output: PathBuf,
        #[arg(long)]
        k_ads: Option<f64>,
        #[arg(long, default_value = "experiments/generated/d028")]
        root: PathBuf,
    },
}

#[derive(Subcommand)]
enum D029Commands {
    /// Gate 0–6 pipeline (stop on first fail).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d029")]
        output: PathBuf,
    },
    Gate2 {
        #[arg(long, default_value = "experiments/generated/d029/parameter_identification")]
        output: PathBuf,
    },
}

#[derive(Subcommand)]
enum D030Commands {
    /// Gate 0–8 orthogonal identification pipeline (stop on first fail).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d030")]
        output: PathBuf,
    },
    Gate0 {
        #[arg(long, default_value = "experiments/generated/d030/preservation")]
        output: PathBuf,
    },
}

#[derive(Subcommand)]
enum D031Commands {
    /// Gate 0/3/4 invariant-domain exchange pipeline (stop on first fail).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d031")]
        output: PathBuf,
    },
    Gate0 {
        #[arg(long, default_value = "experiments/generated/d031")]
        output: PathBuf,
    },
    Gate4 {
        #[arg(long, default_value = "experiments/generated/d031/isolated_turnover")]
        output: PathBuf,
    },
    Gate4Diag {
        #[arg(long, default_value = "experiments/generated/d031/isolated_turnover")]
        output: PathBuf,
    },
}

#[derive(Subcommand)]
enum D032Commands {
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d032")]
        output: PathBuf,
    },
    Gate0 {
        #[arg(long, default_value = "experiments/generated/d032")]
        output: PathBuf,
    },
    Gate2 {
        #[arg(long, default_value = "experiments/generated/d032/active_basis")]
        output: PathBuf,
    },
    Gate5 {
        #[arg(long, default_value = "experiments/generated/d032/isolated_renewal")]
        output: PathBuf,
        #[arg(long)]
        k_active: f64,
    },
}

#[derive(Subcommand)]
enum D033Commands {
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d033")]
        output: PathBuf,
    },
    Gate0 {
        #[arg(long, default_value = "experiments/generated/d033")]
        output: PathBuf,
    },
    Gate2 {
        #[arg(long, default_value = "experiments/generated/d033/kinetics")]
        output: PathBuf,
    },
    Gate3 {
        #[arg(long, default_value = "experiments/generated/d033/buffering")]
        output: PathBuf,
    },
    Gate4 {
        #[arg(long, default_value = "experiments/generated/d033/numerical")]
        output: PathBuf,
    },
    Gate5 {
        #[arg(long, default_value = "experiments/generated/d033/isolated_renewal")]
        output: PathBuf,
        #[arg(long, default_value_t = 0.8)]
        k_charge: f64,
        #[arg(long, default_value_t = 1.2)]
        k_insert: f64,
        #[arg(long, default_value_t = 0.25)]
        k_relax: f64,
    },
}

fn resolve_d027_artifact_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn run_d027(action: D027Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D027Commands::Pipeline { output } => {
            let out = resolve_d027_artifact_path(&output);
            let result = d027::run_pipeline(&out)?;
            println!(
                "D-027 pipeline conclusion={} -> {}",
                result["conclusion"],
                out.join("manifest.json").display()
            );
        }
        D027Commands::Gate0 { output } => {
            let out = resolve_d027_artifact_path(&output);
            let result = d027::run_gate0_ledger_restore(&out)?;
            println!(
                "D-027 gate0 pass={} max_abs={} -> {}",
                result["pass"],
                result["max_abs_diff"],
                out.join("ledger_restore.json").display()
            );
        }
        D027Commands::Gate1 { output } => {
            let out = resolve_d027_artifact_path(&output);
            let result = d027::run_gate1_adsorption_basis(&out)?;
            println!(
                "D-027 gate1 portable={} span={} -> {}",
                result["portability"]["portable"],
                result["portability"]["span"],
                out.join("adsorption_basis.json").display()
            );
        }
    }
    Ok(())
}

fn resolve_d028_artifact_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn run_d028(action: D028Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D028Commands::Pipeline { output } => {
            let out = resolve_d028_artifact_path(&output);
            let result = d028::run_pipeline(&out)?;
            println!(
                "D-028 pipeline conclusion={} selected_k={:?} -> {}",
                result["conclusion"],
                result["selected_k_ads"],
                out.join("manifest.json").display()
            );
        }
        D028Commands::Gate0 { output } => {
            let out = resolve_d028_artifact_path(&output);
            let result = d028::run_gate0_bracket_reproduction(&out)?;
            println!(
                "D-028 gate0 pass={} conclusion={} -> {}",
                result["pass"],
                result["conclusion"],
                out.join("bracket_reproduction.json").display()
            );
        }
        D028Commands::Gate1 { output } => {
            let out = resolve_d028_artifact_path(&output);
            let gate0_path = out
                .parent()
                .unwrap_or(Path::new("."))
                .join("bracket_reproduction/bracket_reproduction.json");
            let gate0: serde_json::Value = if gate0_path.is_file() {
                serde_json::from_str(&std::fs::read_to_string(&gate0_path)?)?
            } else {
                serde_json::json!({
                    "bracket": {
                        "k_low": chemistry_core::d028_analysis::D028_K_ADS_1X,
                        "q_low": chemistry_core::d028_analysis::D028_Q_1X,
                        "k_high": chemistry_core::d028_analysis::D028_K_ADS_2X,
                        "q_high": chemistry_core::d028_analysis::D028_Q_2X,
                    }
                })
            };
            let result = d028::run_gate1_root_solve(&out, &gate0)?;
            println!(
                "D-028 gate1 pass={} conclusion={} k={:?} -> {}",
                result["pass"],
                result["conclusion"],
                result["selected_k_ads"],
                out.join("root_solve.json").display()
            );
        }
        D028Commands::Gate3 {
            output,
            k_ads,
            root,
        } => {
            let out = resolve_d028_artifact_path(&output);
            let root = resolve_d028_artifact_path(&root);
            let selected = if let Some(k) = k_ads {
                k
            } else {
                let solve_path = root.join("root_iterations/root_solve.json");
                let v: serde_json::Value =
                    serde_json::from_str(&std::fs::read_to_string(&solve_path)?)?;
                v["selected_k_ads"]
                    .as_f64()
                    .ok_or("missing selected_k_ads; pass --k-ads")?
            };
            let gate3 = d028::run_gate3_portability(&out, selected)?;
            // Update manifest in place if present.
            let man_path = root.join("manifest.json");
            if man_path.is_file() {
                let mut man: serde_json::Value =
                    serde_json::from_str(&std::fs::read_to_string(&man_path)?)?;
                man["gate3"] = gate3.clone();
                man["selected_k_ads"] = serde_json::json!(selected);
                if gate3["pass"].as_bool().unwrap_or(false) {
                    man["conclusion"] = serde_json::json!("D028_PARTIAL_GATES_0_3_PASS");
                    man["stopped_at_gate"] = serde_json::Value::Null;
                } else {
                    man["conclusion"] = serde_json::json!("D028_ROOT_NOT_PORTABLE");
                    man["stopped_at_gate"] = serde_json::json!(3);
                }
                std::fs::write(&man_path, serde_json::to_string_pretty(&man)?)?;
            }
            println!(
                "D-028 gate3 pass={} pass_count={} conclusion={} -> {}",
                gate3["pass"],
                gate3["pass_count"],
                gate3["conclusion"],
                out.join("portability.json").display()
            );
        }
    }
    Ok(())
}

fn resolve_d029_artifact_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn run_d029(action: D029Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D029Commands::Pipeline { output } => {
            let out = resolve_d029_artifact_path(&output);
            let result = d029::run_pipeline(&out)?;
            println!(
                "D-029 conclusion={} pass={} -> {}",
                result["conclusion"],
                result["pass"],
                out.display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D029Commands::Gate2 { output } => {
            let out = resolve_d029_artifact_path(&output);
            let result = d029::run_gate2_identification(&out)?;
            println!(
                "D-029 Gate2 pass={} conclusion={} -> {}",
                result["pass"],
                result["conclusion"],
                out.display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

fn resolve_d030_artifact_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn run_d030(action: D030Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D030Commands::Pipeline { output } => {
            let out = resolve_d030_artifact_path(&output);
            let result = d030::run_pipeline(&out)?;
            println!(
                "D-030 conclusion={} pass={} -> {}",
                result["conclusion"],
                result["pass"],
                out.display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D030Commands::Gate0 { output } => {
            let out = resolve_d030_artifact_path(&output);
            let result = d030::run_gate0_preservation(&out)?;
            println!(
                "D-030 Gate0 pass={} conclusion={} -> {}",
                result["pass"],
                result["conclusion"],
                out.display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

fn resolve_d031_artifact_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn run_d031(action: D031Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D031Commands::Pipeline { output } => {
            let out = resolve_d031_artifact_path(&output);
            let result = d031::run_pipeline(&out)?;
            println!(
                "D-031 conclusion={} -> {}",
                result["conclusion"],
                out.join("manifest.json").display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D031Commands::Gate0 { output } => {
            let out = resolve_d031_artifact_path(&output);
            let p = d031::run_gate0_preservation(&out.join("preservation"))?;
            let c = d031::run_gate0_capacity_failure(&out.join("capacity_failure"))?;
            println!(
                "D-031 Gate0 class={} -> {}",
                c["capacity_failure"]["classification"],
                out.display()
            );
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({"preservation": p, "capacity": c}))?
            );
        }
        D031Commands::Gate4 { output } => {
            let out = resolve_d031_artifact_path(&output);
            let result = d031::run_gate4_isolated_turnover(&out)?;
            println!(
                "D-031 Gate4 pass={} conclusion={} -> {}",
                result["pass"],
                result["conclusion"],
                out.display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D031Commands::Gate4Diag { output } => {
            let out = resolve_d031_artifact_path(&output);
            let result = d031::run_gate4_short_diagnostic(&out)?;
            println!(
                "D-031 Gate4Diag accepted={} q={} g={} -> {}",
                result["total_accepted"],
                result["q_renewal"],
                result["g_surface"],
                out.display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

fn resolve_d032_artifact_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn run_d032(action: D032Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D032Commands::Pipeline { output } => {
            let out = resolve_d032_artifact_path(&output);
            let result = d032::run_pipeline(&out)?;
            println!(
                "D-032 conclusion={} -> {}",
                result["conclusion"],
                out.join("manifest.json").display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D032Commands::Gate0 { output } => {
            let out = resolve_d032_artifact_path(&output);
            let result = d032::run_gate0_preservation(&out.join("preservation"))?;
            println!(
                "D-032 Gate0 pass={} -> {}",
                result["pass"],
                out.display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D032Commands::Gate2 { output } => {
            let out = resolve_d032_artifact_path(&output);
            let result = d032::run_gate2_active_basis(&out)?;
            println!(
                "D-032 Gate2 pass={} conclusion={} -> {}",
                result["pass"],
                result["conclusion"],
                out.display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D032Commands::Gate5 { output, k_active } => {
            let out = resolve_d032_artifact_path(&output);
            let result = d032::run_gate5_isolated_renewal(&out, k_active)?;
            println!(
                "D-032 Gate5 pass={} conclusion={} -> {}",
                result["pass"],
                result["conclusion"],
                out.display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

fn resolve_d033_artifact_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn run_d033(action: D033Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D033Commands::Pipeline { output } => {
            let out = resolve_d033_artifact_path(&output);
            let result = d033::run_pipeline(&out)?;
            println!(
                "D-033 conclusion={} -> {}",
                result["conclusion"],
                out.join("manifest.json").display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D033Commands::Gate0 { output } => {
            let out = resolve_d033_artifact_path(&output);
            let result = d033::run_gate0_preservation(&out.join("preservation"))?;
            println!(
                "D-033 Gate0 pass={} conclusion={} -> {}",
                result["pass"],
                result["conclusion"],
                out.display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D033Commands::Gate2 { output } => {
            let out = resolve_d033_artifact_path(&output);
            let result = d033::run_gate2_orthogonal_id(&out)?;
            println!(
                "D-033 Gate2 pass={} conclusion={} -> {}",
                result["pass"],
                result["conclusion"],
                out.display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D033Commands::Gate3 { output } => {
            let out = resolve_d033_artifact_path(&output);
            let result = d033::run_gate3_buffering(&out)?;
            println!(
                "D-033 Gate3 pass={} conclusion={} -> {}",
                result["pass"],
                result["conclusion"],
                out.display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D033Commands::Gate4 { output } => {
            let out = resolve_d033_artifact_path(&output);
            let result = d033::run_gate4_numerical(&out)?;
            println!(
                "D-033 Gate4 pass={} conclusion={} -> {}",
                result["pass"],
                result["conclusion"],
                out.display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D033Commands::Gate5 {
            output,
            k_charge,
            k_insert,
            k_relax,
        } => {
            let out = resolve_d033_artifact_path(&output);
            let result = d033::run_gate5_isolated_renewal(&out, k_charge, k_insert, k_relax)?;
            println!(
                "D-033 Gate5 pass={} conclusion={} -> {}",
                result["pass"],
                result["conclusion"],
                out.display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D034Commands {
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d034")]
        output: PathBuf,
    },
    Gate0 {
        #[arg(long, default_value = "experiments/generated/d034")]
        output: PathBuf,
    },
    Gate1 {
        #[arg(long, default_value = "experiments/generated/d034/unit_tests")]
        output: PathBuf,
    },
    Gate2 {
        #[arg(long, default_value = "experiments/generated/d034/passive_exchange_regression")]
        output: PathBuf,
    },
    Gate3 {
        #[arg(long, default_value = "experiments/generated/d034/transport_smoke")]
        output: PathBuf,
    },
    Gate4 {
        #[arg(long, default_value = "experiments/generated/d034/maturation_identification")]
        output: PathBuf,
    },
    Gate5 {
        #[arg(long, default_value = "experiments/generated/d034/maturation_smoke")]
        output: PathBuf,
    },
    Gate6 {
        #[arg(long, default_value = "experiments/generated/d034/rate_reconstruction")]
        output: PathBuf,
    },
    Gate7 {
        #[arg(long, default_value = "experiments/generated/d034/candidates")]
        output: PathBuf,
        #[arg(long)]
        median_k: Option<f64>,
    },
    Gate8 {
        #[arg(long, default_value = "experiments/generated/d034/isolated_renewal")]
        output: PathBuf,
        #[arg(long)]
        k_mature: Option<f64>,
    },
}

fn resolve_d034_artifact_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn run_d034(action: D034Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D034Commands::Pipeline { output } => {
            let out = resolve_d034_artifact_path(&output);
            let result = d034::run_pipeline(&out)?;
            println!(
                "D-034 pipeline conclusion={} -> {}",
                result["conclusion"],
                out.join("manifest.json").display()
            );
        }
        D034Commands::Gate0 { output } => {
            let out = resolve_d034_artifact_path(&output);
            let result = d034::run_gate0_preservation(&out.join("preservation"))?;
            println!(
                "D-034 gate0 pass={} -> {}",
                result["pass"],
                out.join("preservation/preservation.json").display()
            );
        }
        D034Commands::Gate1 { output } => {
            let out = resolve_d034_artifact_path(&output);
            let result = d034::run_gate1_unit_tests(&out)?;
            println!("D-034 gate1 pass={} authority={}", result["pass"], result["authority"]);
        }
        D034Commands::Gate2 { output } => {
            let out = resolve_d034_artifact_path(&output);
            let result = d034::run_gate2_passive_exchange(&out)?;
            println!(
                "D-034 gate2 pass={} conclusion={} -> {}",
                result["pass"],
                result["conclusion"],
                out.join("result.json").display()
            );
        }
        D034Commands::Gate3 { output } => {
            let out = resolve_d034_artifact_path(&output);
            let result = d034::run_gate3_transport_smoke(&out)?;
            println!("D-034 gate3 pass={} -> {}", result["pass"], out.join("result.json").display());
        }
        D034Commands::Gate4 { output } => {
            let out = resolve_d034_artifact_path(&output);
            let result = d034::run_gate4_maturation_id(&out)?;
            println!(
                "D-034 gate4 pass={} conclusion={} -> {}",
                result["pass"],
                result["conclusion"],
                out.join("result.json").display()
            );
        }
        D034Commands::Gate5 { output } => {
            let out = resolve_d034_artifact_path(&output);
            let result = d034::run_gate5_maturation_smoke(&out)?;
            println!("D-034 gate5 pass={} -> {}", result["pass"], out.join("result.json").display());
        }
        D034Commands::Gate6 { output } => {
            let out = resolve_d034_artifact_path(&output);
            let result = d034::run_gate6_rate_reconstruction(&out)?;
            println!(
                "D-034 gate6 pass={} conclusion={} -> {}",
                result["pass"],
                result["conclusion"],
                out.join("result.json").display()
            );
        }
        D034Commands::Gate7 { output, median_k } => {
            let out = resolve_d034_artifact_path(&output);
            let mk = median_k.unwrap_or(chemistry_core::d034_analysis::D034_ASSAY_K_MATURE);
            let result = d034::run_gate7_candidates(&out, mk)?;
            println!(
                "D-034 gate7 pass={} selected={:?} -> {}",
                result["pass"],
                result["selected"],
                out.join("result.json").display()
            );
        }
        D034Commands::Gate8 { output, k_mature } => {
            let out = resolve_d034_artifact_path(&output);
            let km = k_mature.unwrap_or(chemistry_core::d034_analysis::D034_ASSAY_K_MATURE);
            let result = d034::run_gate8_isolated_renewal(&out, km)?;
            println!(
                "D-034 gate8 pass={} conclusion={} -> {}",
                result["pass"],
                result["conclusion"],
                out.join("result.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D035Commands {
    /// Preservation + Gate 0–1 architecture/saturation screen (stop on fail; no chemistry change).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d035")]
        output: PathBuf,
    },
    Gate0 {
        #[arg(long, default_value = "experiments/generated/d035")]
        output: PathBuf,
    },
    ArchitectureReview {
        #[arg(long, default_value = "experiments/generated/d035/architecture_review")]
        output: PathBuf,
    },
    Gate1 {
        #[arg(long, default_value = "experiments/generated/d035/saturation_identification")]
        output: PathBuf,
    },
}

fn resolve_d035_artifact_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn run_d035(action: D035Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D035Commands::Pipeline { output } => {
            let out = resolve_d035_artifact_path(&output);
            let result = d035::run_pipeline(&out)?;
            println!(
                "D-035 pipeline conclusion={} pass={} -> {}",
                result["conclusion"],
                result["pass"],
                out.join("manifest.json").display()
            );
        }
        D035Commands::Gate0 { output } => {
            let out = resolve_d035_artifact_path(&output);
            let result = d035::run_pipeline(&out)?;
            println!(
                "D-035 gate0 conclusion={} pass={} -> {}",
                result["conclusion"],
                result["pass"],
                out.join("manifest.json").display()
            );
        }
        D035Commands::ArchitectureReview { output } => {
            let out = resolve_d035_artifact_path(&output);
            let result = d035::run_gate0_architecture_review(&out)?;
            println!(
                "D-035 architecture_review conclusion={} pass={} -> {}",
                result["conclusion"],
                result["pass"],
                out.join("architecture_review.json").display()
            );
        }
        D035Commands::Gate1 { output } => {
            let out = resolve_d035_artifact_path(&output);
            let result = d035::run_gate1_saturation_identification(&out)?;
            println!(
                "D-035 gate1 conclusion={} pass={} -> {}",
                result["conclusion"],
                result["pass"],
                out.join("saturation_identification.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D036Commands {
    /// Preservation + Gate 0 D-035 observer/runtime parity audit (stop before v13 on defect).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d036")]
        output: PathBuf,
    },
    Gate0 {
        #[arg(long, default_value = "experiments/generated/d036/d035_parity")]
        output: PathBuf,
        #[arg(long, default_value_t = 2500)]
        gate5_advance: u64,
    },
    Gate1 {
        #[arg(long, default_value = "experiments/generated/d036/architecture_review")]
        output: PathBuf,
    },
}

fn resolve_d036_artifact_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

#[derive(Subcommand)]
enum D037Commands {
    /// Full Gates 0–7 membrane-turnover / renewal-gate audit (no chemistry changes).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d037")]
        output: PathBuf,
    },
}

#[derive(Subcommand)]
enum D038Commands {
    /// Correct D-021 surface-turnover transfer and replay renewal architectures.
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d038")]
        output: PathBuf,
    },
}

#[derive(Subcommand)]
enum D039Commands {
    /// Qualify exchange+damage membrane maintenance without constitutive S→W.
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d039")]
        output: PathBuf,
    },
}

#[derive(Subcommand)]
enum D040Commands {
    /// Decompose schema-3 v8 exchange–precursor coupling failure (diagnostic only).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d040")]
        output: PathBuf,
    },
}

#[derive(Subcommand)]
enum D041Commands {
    /// Structural A-retention basin-accessibility qualification (Gates 0–10).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d041")]
        output: PathBuf,
    },
    /// Focused ρ_A zero-S / low-S bootstrap diagnostic (does not certify gates).
    DiagnoseRho {
        #[arg(long, default_value = "experiments/generated/d041")]
        output: PathBuf,
        #[arg(long, default_value_t = 15_000)]
        steps: u64,
    },
}

#[derive(Subcommand)]
enum D042Commands {
    /// Activated-resource capacity and conserved-buffer feasibility audit (Gates 0–5).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d042")]
        output: PathBuf,
    },
}

fn resolve_d037_artifact_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn resolve_d038_artifact_path(path: &Path) -> PathBuf {
    resolve_d037_artifact_path(path)
}

fn resolve_d039_artifact_path(path: &Path) -> PathBuf {
    resolve_d037_artifact_path(path)
}

fn resolve_d040_artifact_path(path: &Path) -> PathBuf {
    resolve_d037_artifact_path(path)
}

fn resolve_d041_artifact_path(path: &Path) -> PathBuf {
    resolve_d037_artifact_path(path)
}

fn resolve_d042_artifact_path(path: &Path) -> PathBuf {
    resolve_d037_artifact_path(path)
}

#[derive(Subcommand)]
enum D043Commands {
    /// Activation-reaction capacity repair audit (Gates 0–9).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d043")]
        output: PathBuf,
    },
}

fn resolve_d043_artifact_path(path: &Path) -> PathBuf {
    resolve_d042_artifact_path(path)
}

fn run_d043(action: D043Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D043Commands::Pipeline { output } => {
            let out = resolve_d043_artifact_path(&output);
            let result = d043::run_pipeline(&out)?;
            println!(
                "D-043 pipeline primary={} k={} -> {}",
                result["primary_conclusion"],
                result.get("selected_k_activation").unwrap_or(&serde_json::Value::Null),
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D044Commands {
    /// Activation-law architecture review (Gates 0–13).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d044")]
        output: PathBuf,
    },
}

fn resolve_d044_artifact_path(path: &Path) -> PathBuf {
    resolve_d043_artifact_path(path)
}

fn run_d044(action: D044Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D044Commands::Pipeline { output } => {
            let out = resolve_d044_artifact_path(&output);
            let result = d044::run_pipeline(&out)?;
            println!(
                "D-044 pipeline primary={} -> {}",
                result["primary_conclusion"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D045Commands {
    /// Fuel-charged catalyst activation review (Phase A stop-on-fail).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d045")]
        output: PathBuf,
    },
}

fn resolve_d045_artifact_path(path: &Path) -> PathBuf {
    resolve_d043_artifact_path(path)
}

fn run_d045(action: D045Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D045Commands::Pipeline { output } => {
            let out = resolve_d045_artifact_path(&output);
            let result = d045::run_pipeline(&out)?;
            println!(
                "D-045 pipeline primary={} -> {}",
                result["primary_conclusion"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D046Commands {
    /// Activated-resource demand topology audit (diagnostic only).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d046")]
        output: PathBuf,
    },
}

fn resolve_d046_artifact_path(path: &Path) -> PathBuf {
    resolve_d043_artifact_path(path)
}

fn run_d046(action: D046Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D046Commands::Pipeline { output } => {
            let out = resolve_d046_artifact_path(&output);
            let result = d046::run_pipeline(&out)?;
            println!(
                "D-046 pipeline primary={} route={} -> {}",
                result["primary_conclusion"],
                result["selected_route"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D047Commands {
    /// Shared activated-resource pool sufficiency audit (diagnostic only).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d047")]
        output: PathBuf,
    },
}

fn resolve_d047_artifact_path(path: &Path) -> PathBuf {
    resolve_d043_artifact_path(path)
}

fn run_d047(action: D047Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D047Commands::Pipeline { output } => {
            let out = resolve_d047_artifact_path(&output);
            let result = d047::run_pipeline(&out)?;
            println!(
                "D-047 pipeline primary={} route={} -> {}",
                result["primary_conclusion"],
                result["selected_route"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D048Commands {
    /// Frozen-biology membrane basin and repair qualification (Gates 0–10).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d048")]
        output: PathBuf,
    },
}

fn resolve_d048_artifact_path(path: &Path) -> PathBuf {
    resolve_d047_artifact_path(path)
}

fn run_d048(action: D048Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D048Commands::Pipeline { output } => {
            let out = resolve_d048_artifact_path(&output);
            let result = d048::run_pipeline(&out)?;
            println!(
                "D-048 pipeline primary={} route={} -> {}",
                result["primary_conclusion"],
                result["route"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D049Commands {
    /// Coupled A/P/S collapse feedback decomposition (Gates 0–11).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d049")]
        output: PathBuf,
    },
}

fn resolve_d049_artifact_path(path: &Path) -> PathBuf {
    resolve_d048_artifact_path(path)
}

fn run_d049(action: D049Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D049Commands::Pipeline { output } => {
            let out = resolve_d049_artifact_path(&output);
            let result = d049::run_pipeline(&out)?;
            println!(
                "D-049 pipeline primary={} route={} -> {}",
                result["primary_conclusion"],
                result["route"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D050Commands {
    /// Catalyst-saturating volume activation repair (Gates 0–13).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d050")]
        output: PathBuf,
    },
}

fn resolve_d050_artifact_path(path: &Path) -> PathBuf {
    resolve_d049_artifact_path(path)
}

fn run_d050(action: D050Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D050Commands::Pipeline { output } => {
            let out = resolve_d050_artifact_path(&output);
            let result = d050::run_pipeline(&out)?;
            println!(
                "D-050 pipeline primary={} stage_e={} -> {}",
                result["primary_conclusion"],
                result["stage_e_status"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D051Commands {
    /// Coupled activation throughput bottleneck audit (Gates −1–10).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d051")]
        output: PathBuf,
    },
}

fn resolve_d051_artifact_path(path: &Path) -> PathBuf {
    resolve_d050_artifact_path(path)
}

fn run_d051(action: D051Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D051Commands::Pipeline { output } => {
            let out = resolve_d051_artifact_path(&output);
            let result = d051::run_pipeline(&out)?;
            println!(
                "D-051 pipeline primary={} stage_e={} -> {}",
                result["primary_conclusion"],
                result["stage_e_status"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D052Commands {
    /// Nutrient/fuel delivery resistance decomposition (Gates 0–12).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d052")]
        output: PathBuf,
    },
}

fn resolve_d052_artifact_path(path: &Path) -> PathBuf {
    resolve_d051_artifact_path(path)
}

fn run_d052(action: D052Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D052Commands::Pipeline { output } => {
            let out = resolve_d052_artifact_path(&output);
            let result = d052::run_pipeline(&out)?;
            println!(
                "D-052 pipeline primary={} stage_e={} -> {}",
                result["primary_conclusion"],
                result["stage_e_status"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D053Commands {
    /// Bounded combined exterior + membrane N/F delivery repair (Gates 0–14).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d053")]
        output: PathBuf,
    },
}

fn resolve_d053_artifact_path(path: &Path) -> PathBuf {
    resolve_d052_artifact_path(path)
}

fn run_d053(action: D053Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D053Commands::Pipeline { output } => {
            let out = resolve_d053_artifact_path(&output);
            let result = d053::run_pipeline(&out)?;
            println!(
                "D-053 pipeline primary={} stage_e={} -> {}",
                result["primary_conclusion"],
                result["stage_e_status"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D055Commands {
    /// Strict D-053 gate repair + passive resource-architecture review.
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d055")]
        output: PathBuf,
    },
}

fn resolve_d055_artifact_path(path: &Path) -> PathBuf {
    resolve_d053_artifact_path(path)
}

fn run_d055(action: D055Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D055Commands::Pipeline { output } => {
            let out = resolve_d055_artifact_path(&output);
            let result = d055::run_pipeline(&out)?;
            println!(
                "D-055 pipeline primary={} route={} -> {}",
                result["primary_conclusion"],
                result["selected_route"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D056Commands {
    /// Waste-coupled resource carrier architecture review (Gates 0–5 Phase A; Phase B gated).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d056")]
        output: PathBuf,
    },
}

fn resolve_d056_artifact_path(path: &Path) -> PathBuf {
    resolve_d053_artifact_path(path)
}

fn run_d056(action: D056Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D056Commands::Pipeline { output } => {
            let out = resolve_d056_artifact_path(&output);
            let result = d056::run_pipeline(&out)?;
            println!(
                "D-056 pipeline primary={} phase_b={} -> {}",
                result["primary_conclusion"],
                result["phase_b_authorized"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D057Commands {
    /// Carrier geometry, normalization, and driving-force audit (observer-only).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d057")]
        output: PathBuf,
    },
}

fn resolve_d057_artifact_path(path: &Path) -> PathBuf {
    resolve_d053_artifact_path(path)
}

fn run_d057(action: D057Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D057Commands::Pipeline { output } => {
            let out = resolve_d057_artifact_path(&output);
            let result = d057::run_pipeline(&out)?;
            println!(
                "D-057 pipeline primary={} route={} -> {}",
                result["primary_conclusion"],
                result["route"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D058Commands {
    /// Corrected carrier face/timestep normalization and re-identification (observer/shadow-only).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d058")]
        output: PathBuf,
    },
}

fn resolve_d058_artifact_path(path: &Path) -> PathBuf {
    resolve_d053_artifact_path(path)
}

fn run_d058(action: D058Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D058Commands::Pipeline { output } => {
            let out = resolve_d058_artifact_path(&output);
            let result = d058::run_pipeline(&out)?;
            println!(
                "D-058 pipeline primary={} route={} -> {}",
                result["primary_conclusion"],
                result["route"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D059Commands {
    /// Viable-size basin and membrane-area architecture review (observer/shadow-only).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d059")]
        output: PathBuf,
    },
}

fn resolve_d059_artifact_path(path: &Path) -> PathBuf {
    resolve_d053_artifact_path(path)
}

fn run_d059(action: D059Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D059Commands::Pipeline { output } => {
            let out = resolve_d059_artifact_path(&output);
            let result = d059::run_pipeline(&out)?;
            println!(
                "D-059 pipeline primary={} route={} -> {}",
                result["primary_conclusion"],
                result["route"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D060Commands {
    /// Structural growth law and resource-coupled size feedback (observer/shadow-only).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d060")]
        output: PathBuf,
    },
}

fn resolve_d060_artifact_path(path: &Path) -> PathBuf {
    resolve_d059_artifact_path(path)
}

fn run_d060(action: D060Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D060Commands::Pipeline { output } => {
            let out = resolve_d060_artifact_path(&output);
            let result = d060::run_pipeline(&out)?;
            println!(
                "D-060 pipeline primary={} route={} -> {}",
                result["primary_conclusion"],
                result["route"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D061Commands {
    /// Structural-constraint execution repair and dynamic-size revalidation.
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d061")]
        output: PathBuf,
    },
}

fn run_d061(action: D061Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D061Commands::Pipeline { output } => {
            let out = resolve_d060_artifact_path(&output);
            let result = d061::run_pipeline(&out)?;
            println!(
                "D-061 pipeline primary={} route={} -> {}",
                result["primary_conclusion"],
                result["route"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D062Commands {
    /// Long-horizon structural maintenance and decay review.
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d062")]
        output: PathBuf,
    },
}

fn run_d062(action: D062Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D062Commands::Pipeline { output } => {
            let out = resolve_d060_artifact_path(&output);
            let result = d062::run_pipeline(&out)?;
            println!(
                "D-062 pipeline primary={} route={} -> {}",
                result["primary_conclusion"],
                result["route"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D063Commands {
    /// Environmentally connected membrane invagination architecture review.
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d063")]
        output: PathBuf,
    },
}

fn run_d063(action: D063Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D063Commands::Pipeline { output } => {
            let out = resolve_d060_artifact_path(&output);
            let result = d063::run_pipeline(&out)?;
            println!(
                "D-063 pipeline primary={} route={} -> {}",
                result["primary_conclusion"],
                result["route"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D064Commands {
    /// Connected-geometry coupled rejection and membrane-load decomposition.
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d064")]
        output: PathBuf,
    },
}

fn run_d064(action: D064Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D064Commands::Pipeline { output } => {
            let out = resolve_d060_artifact_path(&output);
            let result = d064::run_pipeline(&out)?;
            println!(
                "D-064 pipeline primary={} route={} -> {}",
                result["primary_conclusion"],
                result["route"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D065Commands {
    /// Canonical resource-sufficiency requalification and topology-necessity audit.
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d065")]
        output: PathBuf,
    },
}

fn run_d065(action: D065Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D065Commands::Pipeline { output } => {
            let out = resolve_d060_artifact_path(&output);
            let result = d065::run_pipeline(&out)?;
            println!(
                "D-065 pipeline primary={} route={} -> {}",
                result["primary_conclusion"],
                result["route"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D066Commands {
    /// Smooth-membrane activation utilization and local substrate-access audit.
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d066")]
        output: PathBuf,
    },
}

fn run_d066(action: D066Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D066Commands::Pipeline { output } => {
            let out = resolve_d060_artifact_path(&output);
            let result = d066::run_pipeline(&out)?;
            println!(
                "D-066 pipeline primary={} route={} -> {}",
                result["primary_conclusion"],
                result["route"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D067Commands {
    /// Shadow-only activation-capacity law identification (Gates -1–10).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d067")]
        output: PathBuf,
    },
}

fn run_d067(action: D067Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D067Commands::Pipeline { output } => {
            let out = resolve_d060_artifact_path(&output);
            let result = d067::run_pipeline(&out)?;
            println!(
                "D-067 pipeline primary={} route={} -> {}",
                result["primary_conclusion"],
                result["route"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D068Commands {
    /// Shadow-only precursor demand and membrane assembly audit (Gates -1–15).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d068")]
        output: PathBuf,
    },
}

fn run_d068(action: D068Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D068Commands::Pipeline { output } => {
            let out = resolve_d060_artifact_path(&output);
            let result = d068::run_pipeline(&out)?;
            println!(
                "D-068 pipeline primary={} route={} -> {}",
                result["primary_conclusion"],
                result["route"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D069Commands {
    /// Shadow-only mature P↔S exchange equilibrium and desorption audit (Gates -1–16).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d069")]
        output: PathBuf,
    },
}

fn run_d069(action: D069Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D069Commands::Pipeline { output } => {
            let out = resolve_d060_artifact_path(&output);
            let result = d069::run_pipeline(&out)?;
            println!(
                "D-069 pipeline primary={} route={} -> {}",
                result["primary_conclusion"],
                result["route"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D070Commands {
    /// Mature-membrane seed/capacity contract repair (Gates -1–12).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d070")]
        output: PathBuf,
    },
}

fn run_d070(action: D070Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D070Commands::Pipeline { output } => {
            let out = resolve_d060_artifact_path(&output);
            let result = d070::run_pipeline(&out)?;
            println!(
                "D-070 pipeline primary={} route={} -> {}",
                result["primary_conclusion"],
                result["route"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D071Commands {
    /// Capacity-bounded precursor demand regulation (Gates 0–8).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d071")]
        output: PathBuf,
    },
}

fn run_d071(action: D071Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D071Commands::Pipeline { output } => {
            let out = resolve_d060_artifact_path(&output);
            let result = d071::run_pipeline(&out)?;
            println!(
                "D-071 pipeline primary={} route={} -> {}",
                result["primary_conclusion"],
                result["route"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D072Commands {
    /// Mature-membrane damage refill causal audit (Gates 0–6).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d072")]
        output: PathBuf,
    },
}

fn run_d072(action: D072Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D072Commands::Pipeline { output } => {
            let out = resolve_d060_artifact_path(&output);
            let result = d072::run_pipeline(&out)?;
            println!(
                "D-072 pipeline primary={} route={} -> {}",
                result["primary_conclusion"],
                result["route"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D073Commands {
    /// Mature-membrane equilibrium sufficiency audit (Gates 0–7).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d073")]
        output: PathBuf,
    },
}

fn run_d073(action: D073Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D073Commands::Pipeline { output } => {
            let out = resolve_d060_artifact_path(&output);
            let result = d073::run_pipeline(&out)?;
            println!(
                "D-073 pipeline primary={} route={} -> {}",
                result["primary_conclusion"],
                result["route"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D074Commands {
    /// Cellwise exchange integration parity audit (Gates 0–7).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d074")]
        output: PathBuf,
    },
}

fn run_d074(action: D074Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D074Commands::Pipeline { output } => {
            let out = resolve_d060_artifact_path(&output);
            let result = d074::run_pipeline(&out)?;
            println!(
                "D-074 pipeline primary={} route={} -> {}",
                result["primary_conclusion"],
                result["route"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D075Commands {
    /// Cellwise exposure-gated membrane requalification (Gates 0–8).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d075")]
        output: PathBuf,
    },
}

fn run_d075(action: D075Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D075Commands::Pipeline { output } => {
            let out = resolve_d060_artifact_path(&output);
            let result = d075::run_pipeline(&out)?;
            println!(
                "D-075 pipeline primary={} route={} -> {}",
                result["primary_conclusion"],
                result["route"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D076Commands {
    /// Nonequilibrium surface-state cycle architecture review (Gates 0–6).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d076")]
        output: PathBuf,
    },
}

fn run_d076(action: D076Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D076Commands::Pipeline { output } => {
            let out = resolve_d060_artifact_path(&output);
            let result = d076::run_pipeline(&out)?;
            println!(
                "D-076 pipeline primary={} route={} -> {}",
                result["primary_conclusion"],
                result["route"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D077Commands {
    /// Cooperative surface condensation architecture review (Gates 0–7).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d077")]
        output: PathBuf,
    },
}

fn run_d077(action: D077Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D077Commands::Pipeline { output } => {
            let out = resolve_d060_artifact_path(&output);
            let result = d077::run_pipeline(&out)?;
            println!(
                "D-077 pipeline primary={} route={} chi={} -> {}",
                result["primary_conclusion"],
                result["route"],
                result["selected_chi"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D078Commands {
    /// Phase 1 boundary substrate redesign downselect (Gates 0–6).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d078")]
        output: PathBuf,
    },
}

fn run_d078(action: D078Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D078Commands::Pipeline { output } => {
            let out = resolve_d060_artifact_path(&output);
            let result = d078::run_pipeline(&out)?;
            println!(
                "D-078 pipeline primary={} route={} -> {}",
                result["primary_conclusion"],
                result["route"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D079Commands {
    /// Conserved edge-network membrane feasibility (Gates 0–8).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d079")]
        output: PathBuf,
    },
}

fn run_d079(action: D079Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D079Commands::Pipeline { output } => {
            let out = resolve_d060_artifact_path(&output);
            let result = d079::run_pipeline(&out)?;
            println!(
                "D-079 pipeline primary={} route={} stopped={} -> {}",
                result["primary_conclusion"],
                result["route"],
                result["stopped_at_gate"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D080Commands {
    /// Geometry-consistent edge-network repair and requalification (Gates 0–9).
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d080")]
        output: PathBuf,
    },
}

fn run_d080(action: D080Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D080Commands::Pipeline { output } => {
            let out = resolve_d060_artifact_path(&output);
            let result = d080::run_pipeline(&out)?;
            println!(
                "D-080 pipeline primary={} route={} stopped={} -> {}",
                result["primary_conclusion"],
                result["route"],
                result["stopped_at_gate"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D081Commands {
    /// Edge-membrane reserve provenance and replenishment causality audit.
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d081")]
        output: PathBuf,
    },
}

fn run_d081(action: D081Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D081Commands::Pipeline { output } => {
            let out = resolve_d060_artifact_path(&output);
            let result = d081::run_pipeline(&out)?;
            println!(
                "D-081 pipeline primary={} route={} stopped={} -> {}",
                result["primary_conclusion"],
                result["route"],
                result["stopped_at_gate"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D082Commands {
    /// Edge-membrane activation supply integration audit.
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d082")]
        output: PathBuf,
    },
}

fn run_d082(action: D082Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D082Commands::Pipeline { output } => {
            let out = resolve_d060_artifact_path(&output);
            let result = d082::run_pipeline(&out)?;
            println!(
                "D-082 pipeline primary={} route={} stopped={} -> {}",
                result["primary_conclusion"],
                result["route"],
                result["stopped_at_gate"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D083Commands {
    /// Conservative dynamic edge-membrane migration repair.
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d083")]
        output: PathBuf,
    },
}

fn run_d083(action: D083Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D083Commands::Pipeline { output } => {
            let out = resolve_d060_artifact_path(&output);
            let result = d083::run_pipeline(&out)?;
            println!(
                "D-083 pipeline primary={} structural={} stopped={} -> {}",
                result["primary_conclusion"],
                result["structural_direction"],
                result["stopped_at_gate"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D084Commands {
    /// Edge-boundary structural homeostasis via mixed bulk/interface turnover.
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d084")]
        output: PathBuf,
    },
}

fn run_d084(action: D084Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D084Commands::Pipeline { output } => {
            let out = resolve_d060_artifact_path(&output);
            let result = d084::run_pipeline(&out)?;
            println!(
                "D-084 pipeline primary={} stopped={} eta={:?} k={:?} -> {}",
                result["primary_conclusion"],
                result["stopped_at_gate"],
                result["selected_eta"],
                result["selected_k_phi_minus"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum D085Commands {
    /// Decisive structural closure: D-084 dynamic basin + optional mechanochemical fallback.
    Pipeline {
        #[arg(long, default_value = "experiments/generated/d085")]
        output: PathBuf,
    },
}

fn run_d085(action: D085Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D085Commands::Pipeline { output } => {
            let out = resolve_d060_artifact_path(&output);
            let result = d085::run_pipeline(&out)?;
            println!(
                "D-085 pipeline primary={} phase_a={} stage_e={} -> {}",
                result["primary_conclusion"],
                result["phase_a_pass"],
                result["stage_e_pass"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

fn run_d037(action: D037Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D037Commands::Pipeline { output } => {
            let out = resolve_d037_artifact_path(&output);
            let result = d037::run_pipeline(&out)?;
            println!(
                "D-037 pipeline primary={} route={} -> {}",
                result["primary_conclusion"],
                result["selected_route"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

fn run_d038(action: D038Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D038Commands::Pipeline { output } => {
            let out = resolve_d038_artifact_path(&output);
            let result = d038::run_pipeline(&out)?;
            println!(
                "D-038 pipeline primary={} arch={} -> {}",
                result["primary_conclusion"],
                result["selected_architecture"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

fn run_d039(action: D039Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D039Commands::Pipeline { output } => {
            let out = resolve_d039_artifact_path(&output);
            let result = d039::run_pipeline(&out)?;
            println!(
                "D-039 pipeline primary={} route={} -> {}",
                result["primary_conclusion"],
                result["route"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

fn run_d040(action: D040Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D040Commands::Pipeline { output } => {
            let out = resolve_d040_artifact_path(&output);
            let result = d040::run_pipeline(&out)?;
            println!(
                "D-040 pipeline primary={} route={} -> {}",
                result["primary_conclusion"],
                result["selected_route"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

fn run_d041(action: D041Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D041Commands::Pipeline { output } => {
            let out = resolve_d041_artifact_path(&output);
            let result = d041::run_pipeline(&out)?;
            println!(
                "D-041 pipeline primary={} route={} rho_a={} -> {}",
                result["primary_conclusion"],
                result["route"],
                result.get("selected_rho_a").unwrap_or(&serde_json::Value::Null),
                out.join("manifest.json").display()
            );
        }
        D041Commands::DiagnoseRho { output, steps } => {
            let out = resolve_d041_artifact_path(&output);
            let result = d041::diagnose_rho_bootstrap(&out, steps)?;
            println!(
                "D-041 diagnose-rho steps={} rows={} -> {}",
                result["steps"],
                result["rows"].as_array().map(|a| a.len()).unwrap_or(0),
                out.join("retention_candidates/bootstrap_diagnostic.json")
                    .display()
            );
        }
    }
    Ok(())
}

fn run_d042(action: D042Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D042Commands::Pipeline { output } => {
            let out = resolve_d042_artifact_path(&output);
            let result = d042::run_pipeline(&out)?;
            println!(
                "D-042 pipeline primary={} route={} -> {}",
                result["primary_conclusion"],
                result["selected_route"],
                out.join("manifest.json").display()
            );
        }
    }
    Ok(())
}

fn run_d036(action: D036Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D036Commands::Pipeline { output } => {
            let out = resolve_d036_artifact_path(&output);
            let result = d036::run_pipeline(&out)?;
            println!(
                "D-036 pipeline conclusion={} pass={} -> {}",
                result["conclusion"],
                result["pass"],
                out.join("manifest.json").display()
            );
        }
        D036Commands::Gate0 {
            output,
            gate5_advance,
        } => {
            let out = resolve_d036_artifact_path(&output);
            let result = d036::run_gate0_parity(&out, gate5_advance)?;
            println!(
                "D-036 gate0 conclusion={} pass={} -> {}",
                result["conclusion"],
                result["pass"],
                out.join("parity_summary.json").display()
            );
        }
        D036Commands::Gate1 { output } => {
            let out = resolve_d036_artifact_path(&output);
            let result = d036::run_gate1_architecture(&out)?;
            println!(
                "D-036 gate1 conclusion={} pass={} -> {}",
                result["conclusion"],
                result["pass"],
                out.join("architecture_review.json").display()
            );
        }
    }
    Ok(())
}

fn resolve_d025_artifact_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn run_d025(action: D025Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D025Commands::Pipeline { output } => {
            let out = resolve_d025_artifact_path(&output);
            let result = d025::run_gates_3_6(&out)?;
            println!(
                "D-025 stopped_at_gate={} conclusion={} -> {}",
                result["stopped_at_gate"],
                result["conclusion"],
                out.display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D025Commands::Gate3 { output } => {
            let out = resolve_d025_artifact_path(&output);
            let result = d025::run_gate3_growth_shrinkage(&out)?;
            println!("D-025 Gate3 pass={} -> {}", result["gate3_pass"], out.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D025Commands::StageB { output } => {
            let out = resolve_d025_artifact_path(&output);
            let result = d025::run_stage_b_regression(&out)?;
            println!("D-025 StageB pass={} -> {}", result["gate4_pass"], out.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D025Commands::StageC { output } => {
            let out = resolve_d025_artifact_path(&output);
            let result = d025::run_stage_c_regression(&out)?;
            println!("D-025 StageC pass={} -> {}", result["gate5_pass"], out.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D025Commands::StageD { output } => {
            let out = resolve_d025_artifact_path(&output);
            let result = d025::run_stage_d_regression(&out)?;
            println!("D-025 StageD pass={} -> {}", result["gate6_pass"], out.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D025Commands::DynamicR22 { output } => {
            let out = resolve_d025_artifact_path(&output);
            let result = d025::run_dynamic_r22(&out)?;
            println!("D-025 Gate7 pass={} -> {}", result["gate7_pass"], out.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D025Commands::StageEReference { output } => {
            let out = resolve_d025_artifact_path(&output);
            let result = d025_stage_e::run_stage_e_reference(&out, false)?;
            println!(
                "D-025 StageE recovered={} conclusion={} -> {}",
                result["stage_e_recovered"], result["conclusion"], out.display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D025Commands::StageEDiagnostic { output } => {
            let out = resolve_d025_artifact_path(&output);
            let result = d025_stage_e::run_stage_e_reference(&out, true)?;
            println!(
                "D-025 StageE diagnostic conclusion={} -> {}",
                result["conclusion"], out.display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D025Commands::StageESolve { output, reference } => {
            let out = resolve_d025_artifact_path(&output);
            let reference = resolve_d025_artifact_path(&reference);
            let result = d025_stage_e::run_stage_e_solve(&out, &reference)?;
            println!(
                "D-025 StageE solve conclusion={} -> {}",
                result["conclusion"], out.display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

fn run_d023(action: D023Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D023Commands::Pipeline { output } => {
            let result = d023::run_pipeline(&output)?;
            println!(
                "D-023 conclusion={} -> {}",
                result["primary_conclusion"], output.display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D023Commands::Gate0 { output } => {
            let result = d023::run_gate0_schema(&output)?;
            println!("D-023 Gate0 -> {}", output.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D023Commands::Gate1 { output } => {
            let result = d023::run_gate1_conservation(&output)?;
            println!("D-023 Gate1 -> {}", output.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D023Commands::Gate2 { output } => {
            let result = d023::run_gate2_isolated_assembly(&output)?;
            println!(
                "D-023 Gate2 any_pass={} promoted_k_assembly={} -> {}",
                result["any_pass"], result["promoted_k_assembly"], output.display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

fn resolve_d020_artifact_path(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        return path;
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(path)
}

fn resolve_d021_artifact_path(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        return path;
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(path)
}

fn resolve_d022_artifact_path(path: PathBuf) -> PathBuf {
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

fn run_d021(action: D021Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D021Commands::Pipeline { output, max_steps } => {
            let output = resolve_d021_artifact_path(output);
            let result = d021::run_pipeline(&output, max_steps)?;
            println!(
                "D-021 conclusion={} -> {}",
                result["primary_conclusion"],
                output.display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D021Commands::Gate1 { output } => {
            let output = resolve_d021_artifact_path(output);
            let result = d021::run_gate1_eps_screen(&output)?;
            println!("D-021 Gate1 -> {}", output.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D021Commands::Gate2 { output, gate1 } => {
            let output = resolve_d021_artifact_path(output);
            let gate1_path = resolve_d021_artifact_path(gate1);
            let gate1: serde_json::Value =
                serde_json::from_slice(&std::fs::read(gate1_path)?)?;
            let result = d021::run_gate2_fixed_compartment(&output, &gate1)?;
            println!("D-021 Gate2 -> {}", output.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D021Commands::Gate3 { output, gate2 } => {
            let output = resolve_d021_artifact_path(output);
            let gate2_path = resolve_d021_artifact_path(gate2);
            let gate2: serde_json::Value =
                serde_json::from_slice(&std::fs::read(gate2_path)?)?;
            let result = d021::run_gate3_prebalance(&output, &gate2)?;
            println!("D-021 Gate3 -> {}", output.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D021Commands::Gate4 { output, eps_m } => {
            let output = resolve_d021_artifact_path(output);
            let result = d021::run_gate4_joint_recovery(&output, eps_m)?;
            println!("D-021 Gate4 -> {}", output.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D021Commands::Gate5 {
            output,
            eps_m,
            max_steps,
        } => {
            let output = resolve_d021_artifact_path(output);
            let rates = chemistry_core::D021_ANALYTICAL_V4_RATES;
            let result = d021::run_gate5_stage_e(&output, eps_m, &rates, max_steps)?;
            println!("D-021 Gate5 -> {}", output.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}

fn run_d022(action: D022Commands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        D022Commands::Pipeline { output, max_steps } => {
            let output = resolve_d022_artifact_path(output);
            let result = d022::run_pipeline(&output, max_steps)?;
            println!(
                "D-022 conclusion={} -> {}",
                result["primary_conclusion"],
                output.display()
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D022Commands::Gate1 { output } => {
            let output = resolve_d022_artifact_path(output);
            let result = d022::run_gate1_transport_integrity(&output)?;
            println!("D-022 Gate1 -> {}", output.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D022Commands::Gate2 { output } => {
            let output = resolve_d022_artifact_path(output);
            let result = d022::run_gate2_localization(&output)?;
            println!("D-022 Gate2 -> {}", output.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D022Commands::Gate3 { output, chi_m } => {
            let output = resolve_d022_artifact_path(output);
            let result = d022::run_gate3_fixed_compartment(&output, chi_m)?;
            println!("D-022 Gate3 -> {}", output.display());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        D022Commands::Gate4 {
            output,
            chi_m,
            max_steps,
        } => {
            let output = resolve_d022_artifact_path(output);
            let result = d022::run_gate4_stage_e(&output, chi_m, max_steps)?;
            println!("D-022 Gate4 -> {}", output.display());
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
