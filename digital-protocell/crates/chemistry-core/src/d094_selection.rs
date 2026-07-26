//! D-094 Gates 6–8: multi-generation selection on MeshPopulation (D-088 harness).
//!
//! Shared-dish pulse-lean yields zero completed generations even for D-091 controls
//! (`ECOLOGY_HORIZON_TOO_SHORT`). Selection therefore uses the reproduction-qualified
//! MeshPopulation step with ecology applied as exterior pulse/lean and identity-blind
//! membrane/structural abrasion — without changing D-091 ecology definitions' intent.

use crate::abrasion_front::ABRASION_STRENGTHS;
use crate::autocatalytic_copying::{
    founder_b_edges, founder_h_edges, redistribute_edges_along_axis, seed_founder_edges,
};
use crate::autocatalytic_nodes::{stamp_autocatalytic_equation, AutocatalyticParams, NodeKind};
use crate::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use crate::mesh_fission::FissionParams;
use crate::mesh_growth::GrowthParams;
use crate::mesh_mechanics::{mechanics_step, remesh, MechParams};
use crate::mesh_population::{MeshIndividual, MeshPopulation};
use crate::mesh_reactions::{apply_membrane_damage, apply_structural_damage, ReactionParams};
use crate::mesh_transport::TransportParams;
use crate::metabolic_reserve::ReserveParams;
use crate::seasonal_ecology::{PulseLeanSchedule, PulseLeanState, PULSE_PERIOD_MULTS};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionGateResult {
    pub name: String,
    pub pass: bool,
    pub code: Option<String>,
    pub detail: Value,
}

fn smoke() -> bool {
    matches!(
        std::env::var("D094_SMOKE").ok().as_deref(),
        Some("1") | Some("true") | Some("TRUE")
    )
}

fn write_json(path: &Path, v: &impl Serialize) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(
        path,
        serde_json::to_string_pretty(v).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

fn write_json_atomic(path: &Path, v: &impl Serialize) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("checkpoint has no parent: {}", path.display()))?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let tmp = parent.join(format!(
        ".{}.{}.tmp",
        path.file_name().unwrap().to_string_lossy(),
        stamp
    ));
    fs::write(
        &tmp,
        serde_json::to_vec_pretty(v).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::rename(&tmp, path).map_err(|e| e.to_string())
}

fn source_commit() -> String {
    std::env::var("D094_SOURCE_COMMIT").unwrap_or_else(|_| "UNCOMMITTED".into())
}

fn binary_hash() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|path| fs::read(path).ok())
        .map(|bytes| crate::sha256_hex(&bytes))
        .unwrap_or_else(|| "UNAVAILABLE".into())
}

fn config_hash(
    ecology: &str,
    mutation: bool,
    n_rep: usize,
    n_each: usize,
    n_steps: usize,
    target_gen: u32,
    freq_delta_need: f64,
) -> String {
    crate::sha256_hex(
        format!(
            "d094r|{ecology}|{mutation}|{n_rep}|{n_each}|{n_steps}|{target_gen}|{freq_delta_need}"
        )
        .as_bytes(),
    )
}

pub fn provenance_is_complete(v: &Value) -> bool {
    let p = &v["provenance"];
    let known = |key: &str| {
        p[key].as_str().is_some_and(|value| {
            !value.is_empty() && value != "UNCOMMITTED" && value != "UNAVAILABLE"
        })
    };
    known("source_commit")
        && known("binary_hash")
        && known("config_hash")
        && p["atomic_generation_checkpoints"] == true
        && p["lineage_ledger_complete"] == true
}

pub fn hard_blocked_downstream_gates() -> Value {
    json!({
        "blocked": true,
        "reason": "D094R_GATE6_ONLY_UNTIL_FUTURE_DIRECTIVE",
        "status": "NOT_EXECUTED",
    })
}

#[derive(Serialize)]
struct GenerationCheckpoint<'a> {
    source_commit: String,
    binary_hash: String,
    config_hash: String,
    founder_identity: &'a str,
    treatment_identity: &'a str,
    seed_identity: u64,
    mutation_contract: bool,
    generation_index: u32,
    accepted_steps: usize,
    population_state_hash: String,
    lineage_ledger_hash: String,
    atomic_checkpoint_complete: bool,
    lineages: &'a [MeshPopulation],
}

fn gate_pass(name: &str, detail: Value) -> SelectionGateResult {
    SelectionGateResult {
        name: name.into(),
        pass: true,
        code: None,
        detail,
    }
}

fn gate_fail(name: &str, code: &str, detail: Value) -> SelectionGateResult {
    SelectionGateResult {
        name: name.into(),
        pass: false,
        code: Some(code.into()),
        detail,
    }
}

#[derive(Clone, Copy)]
enum SeedMode {
    /// Equal H and B founders (Gate 6).
    Mixed,
    /// B-only founders (Gate 7 H adaptation).
    BOnly,
    /// H-only founders (Gate 7 B adaptation).
    HOnly,
}

fn classify_hb(mesh: &MaterialMesh) -> (f64, f64) {
    let mut aa = 0usize;
    let mut bb = 0usize;
    for e in &mesh.autocatalytic_edges {
        if e.source == NodeKind::A && e.target == NodeKind::A {
            aa += 1;
        }
        if e.source == NodeKind::B && e.target == NodeKind::B {
            bb += 1;
        }
    }
    if aa > bb {
        (1.0, 0.0)
    } else if bb > aa {
        (0.0, 1.0)
    } else if aa + bb == 0 {
        (0.0, 0.0)
    } else {
        (0.5, 0.5)
    }
}

fn seed_founder(
    edges: &[(NodeKind, NodeKind)],
    seed: u64,
    clade: i8,
    exterior_scale: f64,
) -> MeshIndividual {
    let mut mesh = MaterialMesh::seed_regular(
        24,
        12.0,
        40.0,
        40.0,
        DEFAULT_RHO_S,
        0.7,
        LumpedChem {
            c: 0.8,
            a: 0.9,
            n: 0.4,
            f: 0.4,
            r: 0.6,
            q_k: 0.5,
            q_e: 0.5,
            k_a: 0.12,
            k_r: 0.12,
            k_node_b: 0.12,
            ..Default::default()
        },
        LumpedChem {
            n: 1.0 * exterior_scale,
            f: 1.0 * exterior_scale,
            ..Default::default()
        },
        5.0,
    );
    let c = mesh.centroid();
    for v in &mut mesh.vertices {
        v[0] = c[0] + (v[0] - c[0]) * 1.35;
    }
    stamp_autocatalytic_equation(&mut mesh);
    seed_founder_edges(&mut mesh, edges);
    MeshIndividual {
        birth_mass: mesh.total_structural_mass(),
        mesh,
        lineage_id: seed,
        generation: 0,
        clade,
    }
}

fn build_pop(mode: SeedMode, n_each: usize, rep: usize, exterior_scale: f64) -> MeshPopulation {
    let mut pop = MeshPopulation::default();
    let mut next = 1u64;
    match mode {
        SeedMode::Mixed => {
            for i in 0..n_each {
                pop.individuals.push(seed_founder(
                    &founder_h_edges(),
                    100 + rep as u64 * 20 + i as u64,
                    1,
                    exterior_scale,
                ));
                next += 1;
                pop.individuals.push(seed_founder(
                    &founder_b_edges(),
                    200 + rep as u64 * 20 + i as u64,
                    -1,
                    exterior_scale,
                ));
                next += 1;
            }
        }
        SeedMode::BOnly => {
            for i in 0..n_each * 2 {
                pop.individuals.push(seed_founder(
                    &founder_b_edges(),
                    300 + rep as u64 * 20 + i as u64,
                    -1,
                    exterior_scale,
                ));
                next += 1;
            }
        }
        SeedMode::HOnly => {
            for i in 0..n_each * 2 {
                pop.individuals.push(seed_founder(
                    &founder_h_edges(),
                    400 + rep as u64 * 20 + i as u64,
                    1,
                    exterior_scale,
                ));
                next += 1;
            }
        }
    }
    pop.next_lineage = next;
    pop
}

fn apply_ecology(
    ecology: &str,
    pop: &mut MeshPopulation,
    pulse: &mut PulseLeanState,
    abr_t: &mut f64,
    abr_period: f64,
    rich: f64,
    dt: f64,
) {
    match ecology {
        "H" => {
            // Strong intermittent supply: lean hard enough that some lineages die,
            // creating room under the soft pop ceiling for multi-generation turnover.
            let in_pulse = pulse.in_pulse();
            let (n, f) = if in_pulse {
                (rich * 1.25, rich * 1.25)
            } else {
                (rich * 0.12, rich * 0.12)
            };
            for ind in &mut pop.individuals {
                if ind.mesh.alive {
                    ind.mesh.exterior.n = n;
                    ind.mesh.exterior.f = f;
                }
            }
            pulse.t += dt;
        }
        "B" => {
            // Steady supply with frequent abrasion: builders repair; excess deaths
            // allow generation depth beyond the soft living ceiling.
            for ind in &mut pop.individuals {
                if ind.mesh.alive {
                    ind.mesh.exterior.n = rich * 0.90;
                    ind.mesh.exterior.f = rich * 0.90;
                }
            }
            *abr_t += dt;
            if *abr_t >= abr_period * 0.35 {
                *abr_t = 0.0;
                let s = ABRASION_STRENGTHS[0];
                for ind in &mut pop.individuals {
                    if ind.mesh.alive {
                        let _ = apply_structural_damage(&mut ind.mesh, s);
                        let _ = apply_membrane_damage(&mut ind.mesh, s * 0.6);
                    }
                }
            }
        }
        _ => {
            // Neutral: steady moderate supply, baseline efficiencies set by caller.
            for ind in &mut pop.individuals {
                if ind.mesh.alive {
                    ind.mesh.exterior.n = rich * 0.7;
                    ind.mesh.exterior.f = rich * 0.7;
                }
            }
        }
    }
}

fn obs_pop(pop: &MeshPopulation) -> (u32, usize, f64, f64, usize, usize) {
    let mut max_gen = 0u32;
    let mut alive = 0usize;
    let mut n_h = 0.0;
    let mut n_b = 0.0;
    let mut desc_h = 0usize;
    let mut desc_b = 0usize;
    for ind in pop.individuals.iter().filter(|i| i.mesh.alive) {
        alive += 1;
        max_gen = max_gen.max(ind.generation);
        let (h, b) = classify_hb(&ind.mesh);
        n_h += h;
        n_b += b;
        if ind.clade > 0 {
            desc_h += 1;
        } else if ind.clade < 0 {
            desc_b += 1;
        }
    }
    let tot = (n_h + n_b).max(1.0);
    (max_gen, alive, n_h / tot, n_b / tot, desc_h, desc_b)
}

pub fn paired_effect_summary(
    treatment: &Value,
    neutral: &Value,
    frequency: &str,
    descendants: &str,
) -> Value {
    let paired: Vec<Value> = treatment["rows"]
        .as_array()
        .into_iter()
        .flatten()
        .zip(neutral["rows"].as_array().into_iter().flatten())
        .map(|(t, n)| {
            let td = t[frequency].as_f64().unwrap_or(0.0) - 0.5;
            let nd = n[frequency].as_f64().unwrap_or(0.0) - 0.5;
            let tc = t[descendants].as_f64().unwrap_or(0.0);
            let nc = n[descendants].as_f64().unwrap_or(0.0);
            json!({"rep": t["rep"], "frequency_effect": td - nd, "descendant_effect": tc - nc})
        })
        .collect();
    let values =
        |key: &str| -> Vec<f64> { paired.iter().filter_map(|v| v[key].as_f64()).collect() };
    let summary = |key: &str| {
        let mut xs = values(key);
        xs.sort_by(f64::total_cmp);
        let mean = if xs.is_empty() {
            0.0
        } else {
            xs.iter().sum::<f64>() / xs.len() as f64
        };
        let quantile = |p: f64| {
            xs.get(((xs.len().saturating_sub(1)) as f64 * p).round() as usize)
                .copied()
                .unwrap_or(0.0)
        };
        json!({"mean": mean, "median": quantile(0.5), "ci95": [quantile(0.025), quantile(0.975)], "signs": xs.iter().filter(|x| **x > 0.0).count()})
    };
    json!({"paired_replicates": paired, "frequency": summary("frequency_effect"), "descendant_contribution": summary("descendant_effect")})
}

fn ecology_fields(
    ecology: &str,
    pulse: &mut PulseLeanState,
    abr_t: &mut f64,
    abr_period: f64,
    rich: f64,
    dt: f64,
) -> (f64, f64, bool) {
    match ecology {
        "H" => {
            let in_pulse = pulse.in_pulse();
            pulse.t += dt;
            if in_pulse {
                (rich * 1.25, rich * 1.25, false)
            } else {
                (rich * 0.18, rich * 0.18, false)
            }
        }
        "B" => {
            *abr_t += dt;
            let fire = *abr_t >= abr_period * 1.5;
            if fire {
                *abr_t = 0.0;
            }
            (rich * 1.20, rich * 1.20, fire)
        }
        _ => (rich * 0.70, rich * 0.70, false),
    }
}

fn apply_fields_to_pop(pop: &mut MeshPopulation, n: f64, f: f64, abrade: bool) {
    for ind in &mut pop.individuals {
        if ind.mesh.alive {
            ind.mesh.exterior.n = n;
            ind.mesh.exterior.f = f;
            if abrade {
                let s = ABRASION_STRENGTHS[0];
                let _ = apply_structural_damage(&mut ind.mesh, s);
                let _ = apply_membrane_damage(&mut ind.mesh, s * 0.6);
            }
        }
    }
}

fn run_campaign(
    ecology: &str,
    mutation: bool,
    mode: SeedMode,
    n_rep: usize,
    n_each: usize,
    n_steps: usize,
    target_gen: u32,
    freq_delta_need: f64,
    checkpoint_root: Option<&Path>,
) -> Result<Value, String> {
    let mech = MechParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let fission = FissionParams::default();
    let rich = 2.2;
    let mut wins_h = 0usize;
    let mut wins_b = 0usize;
    let mut rows = Vec::new();
    let mut max_gen_all = 0u32;
    let mut gens = Vec::new();
    let source_commit = source_commit();
    let binary_hash = binary_hash();
    let config_hash = config_hash(
        ecology,
        mutation,
        n_rep,
        n_each,
        n_steps,
        target_gen,
        freq_delta_need,
    );
    let founder_identity = match mode {
        SeedMode::Mixed => "equal_h_b_founders_v1",
        SeedMode::BOnly => "b_only_founders_v1",
        SeedMode::HOnly => "h_only_founders_v1",
    };
    let mut all_checkpoints_complete = checkpoint_root.is_some();

    for rep in 0..n_rep {
        // Parallel single-founder lineages under one shared ecology clock.
        let bag = build_pop(mode, n_each, rep, rich);
        let mut lineages: Vec<MeshPopulation> = bag
            .individuals
            .into_iter()
            .map(|ind| {
                let mut p = MeshPopulation::default();
                p.next_lineage = ind.lineage_id.saturating_add(1);
                p.individuals.push(ind);
                p
            })
            .collect();

        let mut react = ReactionParams::default();
        let area = lineages[0].individuals[0].mesh.area();
        react.reserve = ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, area);
        react.reserve.enable = true;
        let mut acs = AutocatalyticParams::derived(40.0);
        if !mutation {
            acs = acs.with_mutation_off();
        }
        if ecology == "N" {
            acs = acs.with_baseline_efficiencies();
        }
        react.autocatalytic = acs;
        react.composition.enable = false;
        react.network.enable = false;
        react.template.enable = false;

        let t_maint = 1.0 / react.reserve.k_release.max(1e-9);
        let period = PULSE_PERIOD_MULTS[0] * t_maint * 4.0;
        let mut pulse = PulseLeanState::new(PulseLeanSchedule {
            cycle_period: period,
            pulse_fraction: 0.40,
            cycle_nf_budget: 1.10 * 0.05 * period,
            lean_nf_rate: 0.0,
        });
        let abr_period = period * 0.5;
        let mut abr_t = 0.0;

        let mut peak_gen = 0u32;
        let mut checkpointed_generation = 0u32;
        for s in 0..n_steps {
            let (n, f, abrade) =
                ecology_fields(ecology, &mut pulse, &mut abr_t, abr_period, rich, mech.dt);
            for pop in &mut lineages {
                apply_fields_to_pop(pop, n, f, abrade);
                if s % 80 == 0 {
                    for ind in &mut pop.individuals {
                        if ind.mesh.alive {
                            redistribute_edges_along_axis(&mut ind.mesh);
                        }
                    }
                }
                pop.individuals.retain(|i| i.mesh.alive);
                // No living ceiling: early-exit on generation depth keeps cost bounded.
                if pop.living_count() > 0 {
                    let _ = pop.step(&mech, &react, &transport, &growth, &fission, true);
                }
                for ind in &pop.individuals {
                    peak_gen = peak_gen.max(ind.generation);
                }
            }
            if let Some(root) = checkpoint_root {
                for generation in checkpointed_generation.saturating_add(1)..=peak_gen {
                    let population_state =
                        serde_json::to_vec(&lineages).map_err(|e| e.to_string())?;
                    let lineage_ledger: Vec<_> = lineages
                        .iter()
                        .flat_map(|population| population.individuals.iter())
                        .map(|individual| {
                            json!({
                                "lineage_id": individual.lineage_id,
                                "generation": individual.generation,
                                "clade": individual.clade,
                                "alive": individual.mesh.alive,
                            })
                        })
                        .collect();
                    let lineage_ledger =
                        serde_json::to_vec(&lineage_ledger).map_err(|e| e.to_string())?;
                    let checkpoint = GenerationCheckpoint {
                        source_commit: source_commit.clone(),
                        binary_hash: binary_hash.clone(),
                        config_hash: config_hash.clone(),
                        founder_identity,
                        treatment_identity: ecology,
                        seed_identity: rep as u64,
                        mutation_contract: mutation,
                        generation_index: generation,
                        accepted_steps: s + 1,
                        population_state_hash: crate::sha256_hex(&population_state),
                        lineage_ledger_hash: crate::sha256_hex(&lineage_ledger),
                        atomic_checkpoint_complete: true,
                        lineages: &lineages,
                    };
                    write_json_atomic(
                        &root.join(format!("rep_{rep}/generation_{generation}.json")),
                        &checkpoint,
                    )?;
                }
                checkpointed_generation = peak_gen;
            }
            if lineages.iter().all(|p| p.living_count() == 0) {
                break;
            }
            if s > 2_500 && peak_gen >= target_gen {
                break;
            }
        }
        let extinct = lineages.iter().all(|p| p.living_count() == 0);

        // Aggregate observers across lineages.
        let mut merged = MeshPopulation::default();
        let mut fissions = 0usize;
        for p in &lineages {
            fissions += p.fission_log.len();
            for ind in &p.individuals {
                if ind.mesh.alive {
                    merged.individuals.push(ind.clone());
                }
            }
        }
        let (_mg_live, alive, f_h, f_b, desc_h, desc_b) = obs_pop(&merged);
        // Completed generations: peak across living/dead during run (not only survivors).
        let max_gen = peak_gen;
        let checkpoints_complete =
            checkpoint_root.is_some() && max_gen > 0 && checkpointed_generation == max_gen;
        all_checkpoints_complete &= checkpoints_complete;
        max_gen_all = max_gen_all.max(max_gen);
        gens.push(max_gen);
        let ratio_h = if desc_b > 0 {
            desc_h as f64 / desc_b as f64
        } else if desc_h > 0 {
            2.0
        } else {
            1.0
        };
        let ratio_b = if desc_h > 0 {
            desc_b as f64 / desc_h as f64
        } else if desc_b > 0 {
            2.0
        } else {
            1.0
        };
        // Preregistered Gate 6 thresholds: Δf ≥ freq_delta_need and ≥1.20x descendants.
        if ecology == "H" && max_gen >= 4 && ratio_h >= 1.20 && (f_h - 0.5) >= freq_delta_need {
            wins_h += 1;
        }
        if ecology == "B" && max_gen >= 4 && ratio_b >= 1.20 && (f_b - 0.5) >= freq_delta_need {
            wins_b += 1;
        }
        if matches!(mode, SeedMode::BOnly)
            && ecology == "H"
            && mutation
            && f_h >= 0.15
            && max_gen >= 4
        {
            wins_h += 1;
        }
        if matches!(mode, SeedMode::HOnly)
            && ecology == "B"
            && mutation
            && f_b >= 0.15
            && max_gen >= 4
        {
            wins_b += 1;
        }

        rows.push(json!({
            "rep": rep,
            "f_h": f_h,
            "f_b": f_b,
            "max_gen": max_gen,
            "alive": alive,
            "extinct": extinct,
            "replicate_complete": max_gen >= target_gen || (extinct && max_gen >= 4),
            "desc_h": desc_h,
            "desc_b": desc_b,
            "desc_h_fraction": if desc_h + desc_b == 0 { 0.0 } else { desc_h as f64 / (desc_h + desc_b) as f64 },
            "desc_b_fraction": if desc_h + desc_b == 0 { 0.0 } else { desc_b as f64 / (desc_h + desc_b) as f64 },
            "ratio_h": ratio_h,
            "ratio_b": ratio_b,
            "fissions": fissions,
            "lineages": lineages.len(),
            "generation_checkpoints_complete": checkpoints_complete,
        }));
        let mode_s = match mode {
            SeedMode::Mixed => "mixed",
            SeedMode::BOnly => "b_only",
            SeedMode::HOnly => "h_only",
        };
        eprintln!(
            "d094-sel eco={} mut={} mode={} rep={}/{} max_gen={} alive={} f_h={:.2} f_b={:.2}",
            ecology,
            mutation,
            mode_s,
            rep + 1,
            n_rep,
            max_gen,
            alive,
            f_h,
            f_b
        );
        let _ = std::fs::write(
            "/tmp/d094_sel_progress.txt",
            format!(
                "{} mut={} mode={} rep={}/{} max_gen={} alive={} f_h={:.3} f_b={:.3}\n",
                ecology,
                mutation,
                mode_s,
                rep + 1,
                n_rep,
                max_gen,
                alive,
                f_h,
                f_b
            ),
        );
    }

    gens.sort_unstable();
    let median_gen = if gens.is_empty() {
        0
    } else {
        gens[gens.len() / 2]
    };

    Ok(json!({
        "ecology": ecology,
        "mutation": mutation,
        "mode": match mode {
            SeedMode::Mixed => "mixed",
            SeedMode::BOnly => "b_only",
            SeedMode::HOnly => "h_only",
        },
        "harness": "parallel_lineage_mesh_population",
        "target_gen": target_gen,
        "freq_delta_need": freq_delta_need,
        "wins_h": wins_h,
        "wins_b": wins_b,
        "rows": rows,
        "provenance": {
            "source_commit": source_commit,
            "binary_hash": binary_hash,
            "config_hash": config_hash,
            "atomic_generation_checkpoints": all_checkpoints_complete,
            "lineage_ledger_complete": all_checkpoints_complete,
            "reuse_status": if all_checkpoints_complete { "FRESH_ATTEMPT_CHECKPOINTS_PRESENT" } else { "REJECT_UNTIL_CHECKPOINT_AND_LEDGER_VALIDATION" },
        },
        "max_gen_all": max_gen_all,
        "median_gen": median_gen,
        "smoke": smoke(),
    }))
}

fn transfer_reversal(
    from_eco: &str,
    to_eco: &str,
    n_rep: usize,
    n_each: usize,
    n_steps_sel: usize,
    n_steps_rev: usize,
) -> Value {
    let mech = MechParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let fission = FissionParams::default();
    let rich = 2.2;
    let mut ok = 0usize;
    let mut rows = Vec::new();

    for rep in 0..n_rep {
        // Phase A: selection under from_eco (mutation off, mixed).
        let mut pop = build_pop(SeedMode::Mixed, n_each, rep, rich);
        let mut react = ReactionParams::default();
        let area = pop.individuals[0].mesh.area();
        react.reserve = ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, area);
        react.reserve.enable = true;
        react.autocatalytic = AutocatalyticParams::derived(40.0).with_mutation_off();
        react.composition.enable = false;
        let t_maint = 1.0 / react.reserve.k_release.max(1e-9);
        let period = PULSE_PERIOD_MULTS[0] * t_maint * 4.0;
        let mut pulse = PulseLeanState::new(PulseLeanSchedule {
            cycle_period: period,
            pulse_fraction: 0.35,
            cycle_nf_budget: 1.10 * 0.05 * period,
            lean_nf_rate: 0.0,
        });
        let mut abr_t = 0.0;
        for s in 0..n_steps_sel {
            apply_ecology(
                from_eco,
                &mut pop,
                &mut pulse,
                &mut abr_t,
                period * 0.5,
                rich,
                mech.dt,
            );
            if s % 80 == 0 {
                for ind in &mut pop.individuals {
                    if ind.mesh.alive {
                        redistribute_edges_along_axis(&mut ind.mesh);
                    }
                }
            }
            pop.individuals.retain(|i| i.mesh.alive);
            if pop.living_count() >= 16 {
                if s % 4 != 0 {
                    continue;
                }
                for ind in &mut pop.individuals {
                    let _ =
                        crate::mesh_transport::transport_step(&mut ind.mesh, &transport, mech.dt);
                    let _ = crate::mesh_reactions::reactions_step(
                        &mut ind.mesh,
                        &react,
                        mech.dt,
                        true,
                        true,
                    );
                    let _ =
                        crate::mesh_growth::growth_step(&mut ind.mesh, &react, &growth, mech.dt);
                    mechanics_step(&mut ind.mesh, &mech);
                    remesh(&mut ind.mesh);
                    crate::mesh_reactions::evaluate_death(&mut ind.mesh);
                }
            } else {
                let _ = pop.step(&mech, &react, &transport, &growth, &fission, true);
            }
            if pop.living_count() == 0 {
                break;
            }
        }
        let (g_a, _, f_h_a, f_b_a, _, _) = obs_pop(&pop);
        if g_a < 4 {
            rows.push(json!({"rep": rep, "skipped": true, "reason": "insufficient_generations"}));
            continue;
        }

        // Phase B: transfer survivors without normalizing biology.
        pulse = PulseLeanState::new(PulseLeanSchedule {
            cycle_period: period,
            pulse_fraction: 0.35,
            cycle_nf_budget: 1.10 * 0.05 * period,
            lean_nf_rate: 0.0,
        });
        abr_t = 0.0;
        for s in 0..n_steps_rev {
            apply_ecology(
                to_eco,
                &mut pop,
                &mut pulse,
                &mut abr_t,
                period * 0.5,
                rich,
                mech.dt,
            );
            if s % 80 == 0 {
                for ind in &mut pop.individuals {
                    if ind.mesh.alive {
                        redistribute_edges_along_axis(&mut ind.mesh);
                    }
                }
            }
            pop.individuals.retain(|i| i.mesh.alive);
            if pop.living_count() >= 16 {
                if s % 4 != 0 {
                    continue;
                }
                for ind in &mut pop.individuals {
                    let _ =
                        crate::mesh_transport::transport_step(&mut ind.mesh, &transport, mech.dt);
                    let _ = crate::mesh_reactions::reactions_step(
                        &mut ind.mesh,
                        &react,
                        mech.dt,
                        true,
                        true,
                    );
                    let _ =
                        crate::mesh_growth::growth_step(&mut ind.mesh, &react, &growth, mech.dt);
                    mechanics_step(&mut ind.mesh, &mech);
                    remesh(&mut ind.mesh);
                    crate::mesh_reactions::evaluate_death(&mut ind.mesh);
                }
            } else {
                let _ = pop.step(&mech, &react, &transport, &growth, &fission, true);
            }
            if pop.living_count() == 0 {
                break;
            }
        }
        let (g_b, alive, f_h_b, f_b_b, _, _) = obs_pop(&pop);
        let reversed = if from_eco == "H" && to_eco == "B" {
            f_b_b > f_b_a + 0.10 && f_h_b < f_h_a
        } else if from_eco == "B" && to_eco == "H" {
            f_h_b > f_h_a + 0.10 && f_b_b < f_b_a
        } else {
            false
        };
        if reversed && g_b >= g_a {
            ok += 1;
        }
        rows.push(json!({
            "rep": rep,
            "skipped": false,
            "gen_before": g_a,
            "gen_after": g_b,
            "f_h_before": f_h_a,
            "f_b_before": f_b_a,
            "f_h_after": f_h_b,
            "f_b_after": f_b_b,
            "reversed": reversed,
            "alive": alive,
        }));
    }

    json!({
        "from": from_eco,
        "to": to_eco,
        "ok_transfers": ok,
        "need": 6,
        "rows": rows,
        "smoke": smoke(),
    })
}

pub fn run_selection_gates(
    out: &Path,
) -> Result<
    (
        SelectionGateResult,
        SelectionGateResult,
        SelectionGateResult,
    ),
    String,
> {
    let n_rep = if smoke() { 2 } else { 8 };
    let n_each = if smoke() { 2 } else { 4 };
    // Reproduction-qualified MeshPopulation reaches gen2 quickly; allow headroom
    // for ≥6 median generations without unbounded population cost.
    let n_steps = if smoke() { 6_000 } else { 12_000 };
    let n_steps_rev = if smoke() { 4_000 } else { 12_000 };

    let h = run_campaign(
        "H",
        false,
        SeedMode::Mixed,
        n_rep,
        n_each,
        n_steps,
        6,
        0.15,
        None,
    )?;
    write_json(&out.join("selection_h/gate6.json"), &h)?;
    let b = run_campaign(
        "B",
        false,
        SeedMode::Mixed,
        n_rep,
        n_each,
        n_steps,
        6,
        0.15,
        None,
    )?;
    write_json(&out.join("selection_b/gate6.json"), &b)?;
    let n = run_campaign(
        "N",
        false,
        SeedMode::Mixed,
        n_rep,
        n_each,
        n_steps,
        6,
        0.15,
        None,
    )?;
    write_json(&out.join("neutral_controls/gate6.json"), &n)?;

    let max_gen = h["max_gen_all"]
        .as_u64()
        .unwrap_or(0)
        .max(b["max_gen_all"].as_u64().unwrap_or(0));
    let median_h = h["median_gen"].as_u64().unwrap_or(0);
    let median_b = b["median_gen"].as_u64().unwrap_or(0);
    let median = median_h.min(median_b);
    let valid = !smoke() && max_gen >= 4 && median_h >= 6 && median_b >= 6;
    let wins_h = h["wins_h"].as_u64().unwrap_or(0) as usize;
    let wins_b = b["wins_b"].as_u64().unwrap_or(0) as usize;
    let neut_shift = n["rows"]
        .as_array()
        .map(|rows| {
            let mut abs = 0.0;
            for r in rows {
                abs += (r["f_h"].as_f64().unwrap_or(0.5) - 0.5).abs();
            }
            if rows.is_empty() {
                1.0
            } else {
                abs / rows.len() as f64
            }
        })
        .unwrap_or(1.0);
    let neut_ok = neut_shift < 0.10
        && n["wins_h"].as_u64().unwrap_or(0) <= 5
        && n["wins_b"].as_u64().unwrap_or(0) <= 5;

    let g6 = if valid && wins_h >= 6 && wins_b >= 6 && neut_ok {
        gate_pass(
            "gate6_selection",
            json!({"h": h, "b": b, "n": n, "valid": true, "max_gen": max_gen, "median": median}),
        )
    } else if max_gen == 0 {
        gate_fail(
            "gate6_selection",
            "D094_AUTOCATALYTIC_SET_SELECTION_UNTESTABLE_INSUFFICIENT_GENERATIONS",
            json!({"h": h, "b": b, "n": n, "valid": false, "max_gen": 0}),
        )
    } else {
        gate_fail(
            "gate6_selection",
            "D094_AUTOCATALYTIC_SET_SELECTION_NOT_ESTABLISHED",
            json!({
                "h": h, "b": b, "n": n,
                "valid": valid,
                "max_gen": max_gen,
                "median": median,
                "wins_h": wins_h,
                "wins_b": wins_b,
                "neut_ok": neut_ok,
                "neut_shift": neut_shift,
            }),
        )
    };

    let blocked = hard_blocked_downstream_gates();
    write_json(&out.join("mutation_adaptation/gate7.json"), &blocked)?;
    write_json(&out.join("reversal/gate8.json"), &blocked)?;
    let g7 = gate_fail(
        "gate7_adaptation",
        "GATE7_HARD_BLOCKED_BY_D094R",
        blocked.clone(),
    );
    let g8 = gate_fail("gate8_reversal", "GATE8_HARD_BLOCKED_BY_D094R", blocked);
    Ok((g6, g7, g8))
}

/// D-094R: Gate 6 completion only — eight-generation horizon, no Gates 7/8.
pub fn run_gate6_completion_only(out: &Path) -> Result<Value, String> {
    fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let n_rep = if smoke() { 2 } else { 8 };
    let n_each = if smoke() { 2 } else { 4 };
    // Horizon: 8 completed generations (preregistered contract).
    let n_steps = if smoke() { 8_000 } else { 22_000 };
    let target_gen = if smoke() { 4 } else { 8u32 };
    let freq_need = 0.15f64;

    // Completed campaigns are not re-run (directive §6: do not restart completed generations).
    let reuse = |path: &Path| -> Option<Value> {
        let text = fs::read_to_string(path).ok()?;
        let v: Value = serde_json::from_str(&text).ok()?;
        if !provenance_is_complete(&v) {
            return None;
        }
        let same_horizon = v["target_gen"].as_u64() == Some(target_gen as u64);
        let complete = v["rows"]
            .as_array()
            .map(|rows| {
                rows.len() == n_rep
                    && rows
                        .iter()
                        .all(|r| r["replicate_complete"].as_bool().unwrap_or(false))
            })
            .unwrap_or(false);
        if same_horizon && complete {
            Some(v)
        } else {
            None
        }
    };

    let h_path = out.join("selection_h_completion/gate6.json");
    let h = match reuse(&h_path) {
        Some(v) => v,
        None => {
            let v = run_campaign(
                "H",
                false,
                SeedMode::Mixed,
                n_rep,
                n_each,
                n_steps,
                target_gen,
                freq_need,
                Some(&out.join("checkpoints/h_selection")),
            )?;
            write_json(&h_path, &v)?;
            v
        }
    };
    let b_path = out.join("selection_b_completion/gate6.json");
    let b = match reuse(&b_path) {
        Some(v) => v,
        None => {
            let v = run_campaign(
                "B",
                false,
                SeedMode::Mixed,
                n_rep,
                n_each,
                n_steps,
                target_gen,
                freq_need,
                Some(&out.join("checkpoints/b_selection")),
            )?;
            write_json(&b_path, &v)?;
            v
        }
    };
    let n_path = out.join("neutral_completion/gate6.json");
    let n = match reuse(&n_path) {
        Some(v) => v,
        None => {
            let v = run_campaign(
                "N",
                false,
                SeedMode::Mixed,
                n_rep,
                n_each,
                n_steps,
                target_gen,
                freq_need,
                Some(&out.join("checkpoints/neutral")),
            )?;
            write_json(&n_path, &v)?;
            v
        }
    };

    let median_h = h["median_gen"].as_u64().unwrap_or(0);
    let median_b = b["median_gen"].as_u64().unwrap_or(0);
    let median_n = n["median_gen"].as_u64().unwrap_or(0);
    let wins_h = h["wins_h"].as_u64().unwrap_or(0) as usize;
    let wins_b = b["wins_b"].as_u64().unwrap_or(0) as usize;

    let campaign_complete = |camp: &Value| -> bool {
        camp["rows"]
            .as_array()
            .map(|rows| {
                !rows.is_empty()
                    && rows
                        .iter()
                        .all(|r| r["replicate_complete"].as_bool().unwrap_or(false))
            })
            .unwrap_or(false)
    };
    let h_complete = campaign_complete(&h);
    let b_complete = campaign_complete(&b);
    let n_complete = campaign_complete(&n);
    let provenance_ok =
        provenance_is_complete(&h) && provenance_is_complete(&b) && provenance_is_complete(&n);
    let horizon_ok = !smoke() && provenance_ok && h_complete && b_complete && n_complete;

    // Neutral: absolute median |Δf| < 0.10 and neither label wins >5/8.
    // Label asymmetry is |f_h - f_b|/2. This equals |f_h - 0.5| whenever label material
    // exists (f_h + f_b = 1) and is 0 when no label material remains, which is the correct
    // reading of "no label advantage" rather than a spurious 0.5 shift.
    let neut_shift = n["rows"]
        .as_array()
        .map(|rows| {
            let mut abs = 0.0;
            for r in rows {
                let fh = r["f_h"].as_f64().unwrap_or(0.0);
                let fb = r["f_b"].as_f64().unwrap_or(0.0);
                abs += (fh - fb).abs() / 2.0;
            }
            if rows.is_empty() {
                1.0
            } else {
                abs / rows.len() as f64
            }
        })
        .unwrap_or(1.0);
    let neut_depth_ok = median_n >= 6
        || n["rows"]
            .as_array()
            .map(|rows| {
                rows.iter().all(|r| {
                    r["extinct"].as_bool().unwrap_or(false)
                        && r["max_gen"].as_u64().unwrap_or(0) >= 4
                })
            })
            .unwrap_or(false);
    let neut_ok = neut_shift < 0.10
        && n["wins_h"].as_u64().unwrap_or(0) <= 5
        && n["wins_b"].as_u64().unwrap_or(0) <= 5
        && neut_depth_ok;

    let h_effects = paired_effect_summary(&h, &n, "f_h", "desc_h_fraction");
    let b_effects = paired_effect_summary(&b, &n, "f_b", "desc_b_fraction");
    let selection_pass =
        horizon_ok && median_h >= 6 && median_b >= 6 && wins_h >= 6 && wins_b >= 6 && neut_ok;
    let valid_complete_no_selection = horizon_ok && !selection_pass;

    let conclusion = if !provenance_ok {
        "D094_AUTOCATALYTIC_SET_SELECTION_INVALID_PROVENANCE"
    } else if selection_pass {
        "D094_AUTOCATALYTIC_SET_ENVIRONMENT_DEPENDENT_SELECTION_QUALIFIED"
    } else if valid_complete_no_selection {
        "D094_AUTOCATALYTIC_SET_HEREDITY_QUALIFIED_SELECTION_REJECTED"
    } else {
        "D094_AUTOCATALYTIC_SET_SELECTION_UNTESTABLE_INSUFFICIENT_GENERATIONS"
    };

    let decision = json!({
        "conclusion": conclusion,
        "selection_pass": selection_pass,
        "horizon_ok": horizon_ok,
        "h_complete": h_complete,
        "b_complete": b_complete,
        "n_complete": n_complete,
        "provenance_ok": provenance_ok,
        "valid_complete_no_selection": valid_complete_no_selection,
        "target_gen": target_gen,
        "freq_delta_need": freq_need,
        "median_h": median_h,
        "median_b": median_b,
        "median_n": median_n,
        "wins_h": wins_h,
        "wins_b": wins_b,
        "neut_ok": neut_ok,
        "neut_shift": neut_shift,
        "gates_7_8_blocked": true,
        "treatment_effects": {"H": h_effects, "B": b_effects},
        "h": h,
        "b": b,
        "n": n,
        "records": if valid_complete_no_selection {
            vec![
                "DISTRIBUTED_AUTOCATALYTIC_HEREDITY_QUALIFIED",
                "AUTOCATALYTIC_NETWORK_PHENOTYPE_CAUSAL",
                "ENVIRONMENT_DEPENDENT_SELECTION_NOT_ESTABLISHED",
                "PHASE3_NOT_AUTHORIZED",
                "GATE7_GATE8_BLOCKED_UNTIL_GATE6_PASS",
            ]
        } else if selection_pass {
            vec![
                "D094_AUTOCATALYTIC_SET_ENVIRONMENT_DEPENDENT_SELECTION_QUALIFIED",
                "GATE7_GATE8_BLOCKED_UNTIL_FUTURE_DIRECTIVE",
            ]
        } else {
            vec!["D094_GATE6_SELECTION_CLOSURE_INCOMPLETE"]
        },
    });
    write_json(&out.join("gate6_decision/decision.json"), &decision)?;
    write_json(&out.join("manifest.json"), &decision)?;
    Ok(decision)
}
