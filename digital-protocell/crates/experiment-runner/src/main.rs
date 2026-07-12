//! Headless experiment runner for Phase 1 scientific acceptance.

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
