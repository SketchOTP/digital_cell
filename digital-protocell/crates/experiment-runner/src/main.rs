//! Headless experiment runner and parameter sweep.

use chemistry_core::*;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run { config, output } => run_single(&config, &output)?,
        Commands::Sweep { config, output } => run_sweep(&config, &output)?,
        Commands::All { output } => run_all_experiments(&output)?,
    }
    Ok(())
}

fn run_single(config_path: &PathBuf, output_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_experiment_config(config_path)?;
    let out = output_dir.join(&config.name);
    fs::create_dir_all(&out)?;
    let sim = run_experiment(&config, 500);
    write_experiment_artifacts(&out, &config, &sim)?;
    println!("Experiment {} complete -> {}", config.name, out.display());
    Ok(())
}

fn run_all_experiments(output: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
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
        let sim = run_experiment(&config, 500);
        write_experiment_artifacts(&out, &config, &sim)?;
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
    }
}

fn puncture_repair() -> ExperimentConfig {
    ExperimentConfig {
        name: "puncture_repair".into(),
        seed: 1,
        substeps: substeps(),
        params: baseline_params(),
        interventions: vec![InterventionSpec::AtSubstep {
            substep: 8000,
            action: InterventionAction::PunctureRepair,
        }],
    }
}

fn catastrophic_damage() -> ExperimentConfig {
    ExperimentConfig {
        name: "catastrophic_damage".into(),
        seed: 1,
        substeps: substeps(),
        params: baseline_params(),
        interventions: vec![InterventionSpec::AtSubstep {
            substep: 5000,
            action: InterventionAction::CatastrophicDamage,
        }],
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
                substep: 5000,
                action: InterventionAction::RemoveNutrient,
            },
            InterventionSpec::AtSubstep {
                substep: 5000,
                action: InterventionAction::RemoveFuel,
            },
            InterventionSpec::AtSubstep {
                substep: substeps(),
                action: InterventionAction::RestoreReservoir,
            },
        ],
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
    }
}

fn load_experiment_config(path: &PathBuf) -> Result<ExperimentConfig, Box<dyn std::error::Error>> {
    let data = fs::read_to_string(path)?;
    let config: ExperimentConfig = toml::from_str(&data)?;
    Ok(config)
}

fn write_experiment_artifacts(
    out: &PathBuf,
    config: &ExperimentConfig,
    sim: &Simulation,
) -> Result<(), Box<dyn std::error::Error>> {
    let snap = sim.snapshot();
    save_snapshot(&out.join("snapshot.json"), &snap)?;
    fs::write(out.join("config.json"), serde_json::to_string_pretty(config)?)?;

    let mut csv = String::from("substep,sim_time,dt,structural_mass,catalyst_mass,retention,classification\n");
    for d in &sim.history {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{:?}\n",
            d.substep, d.sim_time, d.dt, d.structural_mass, d.catalyst_mass,
            d.catalyst_retention, d.classification
        ));
    }
    fs::write(out.join("diagnostics.csv"), csv)?;

    let summary = serde_json::json!({
        "experiment": config.name,
        "seed": config.seed,
        "substeps": sim.substep,
        "classification": format!("{:?}", sim.detector.last_classification),
        "structural_mass": total_mass(&sim.grid, &sim.fields.structure),
        "catalyst_mass": total_mass(&sim.grid, &sim.fields.catalyst),
        "rejection_count": sim.rejection_count,
        "min_dt": sim.min_dt_seen,
        "turnover": sim.detector.turnover,
        "version": SIM_VERSION,
    });
    fs::write(out.join("summary.json"), serde_json::to_string_pretty(&summary)?)?;

    render_field_png(&sim, &out.join("structure_final.png"), "structure")?;
    Ok(())
}

fn render_field_png(sim: &Simulation, path: &PathBuf, field: &str) -> Result<(), Box<dyn std::error::Error>> {
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

fn run_sweep(_config_path: &PathBuf, output: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
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
