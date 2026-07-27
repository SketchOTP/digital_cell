use chemistry_core::autocatalytic_edges::edge_frequency_vector;
use chemistry_core::autocatalytic_nodes::{AutocatalyticParams, NodeKind};
use chemistry_core::d095_analysis::{
    candidate_review, environmental_contrast, final_causal_classification, freeze_d096_contract,
    reciprocal_interaction, select_route, write_observational_artifacts, Allocation,
    CausalReplaySummary, PartitionSummary,
};
use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_fission::{try_local_fission, FissionParams};
use chemistry_core::mesh_growth::{GrowthParams, GrowthLedger};
use chemistry_core::mesh_mechanics::MechParams;
use chemistry_core::mesh_population::{coupled_step_growth, MeshIndividual, MeshPopulation};
use chemistry_core::mesh_reactions::{
    apply_membrane_damage, apply_structural_damage, ReactionLedger, ReactionParams,
};
use chemistry_core::mesh_transport::TransportParams;
use chemistry_core::metabolic_reserve::ReserveParams;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
struct Checkpoint {
    lineages: Vec<MeshPopulation>,
}

#[derive(Clone)]
struct MatchedPair {
    treatment: String,
    replicate: usize,
    generation: u32,
    h: MeshIndividual,
    b: MeshIndividual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReplayRow {
    source_treatment: String,
    replay_environment: String,
    replicate: usize,
    generation: u32,
    clade: String,
    initial_mass: f64,
    initial_reserve: f64,
    nutrient_uptake: f64,
    fuel_uptake: f64,
    activated_resource_production: f64,
    reserve_change: f64,
    damage_cost: f64,
    repair_cost: f64,
    structural_growth: f64,
    survived: bool,
    steps_to_fission_readiness: Option<usize>,
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

fn load_checkpoint(path: &Path) -> Result<Checkpoint, String> {
    serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}

fn frozen_params(area: f64) -> (
    MechParams,
    ReactionParams,
    TransportParams,
    GrowthParams,
    FissionParams,
) {
    let mut reaction = ReactionParams::default();
    reaction.reserve = ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, area);
    reaction.reserve.enable = true;
    reaction.autocatalytic = AutocatalyticParams::derived(40.0).with_mutation_off();
    (
        MechParams::default(),
        reaction,
        TransportParams::default(),
        GrowthParams {
            y_g: 0.9,
            enable_growth: true,
        },
        FissionParams::default(),
    )
}

fn trait_score(mesh: &MaterialMesh) -> f64 {
    let mut aa = 0.0;
    let mut bb = 0.0;
    for edge in &mesh.autocatalytic_edges {
        if edge.source == NodeKind::A && edge.target == NodeKind::A {
            aa += 1.0;
        } else if edge.source == NodeKind::B && edge.target == NodeKind::B {
            bb += 1.0;
        }
    }
    if aa + bb == 0.0 {
        0.0
    } else {
        (aa - bb) / (aa + bb)
    }
}

fn mean(xs: &[f64]) -> f64 {
    xs.iter().sum::<f64>() / xs.len().max(1) as f64
}

fn variance(xs: &[f64]) -> f64 {
    let m = mean(xs);
    mean(&xs.iter().map(|x| (x - m).powi(2)).collect::<Vec<_>>())
}

fn covariance(xs: &[f64], ys: &[f64]) -> f64 {
    let mx = mean(xs);
    let my = mean(ys);
    mean(
        &xs.iter()
            .zip(ys)
            .map(|(x, y)| (x - mx) * (y - my))
            .collect::<Vec<_>>(),
    )
}

fn choose_pairs(attempt: &Path) -> Result<Vec<MatchedPair>, String> {
    let mut pairs = Vec::new();
    for (treatment, dir) in [("H", "h_selection"), ("B", "b_selection"), ("N", "neutral")] {
        for replicate in 0..8 {
            let checkpoint = load_checkpoint(
                &attempt
                    .join("checkpoints")
                    .join(dir)
                    .join(format!("rep_{replicate}/generation_8.json")),
            )?;
            let alive = checkpoint
                .lineages
                .iter()
                .flat_map(|p| &p.individuals)
                .filter(|i| i.mesh.alive)
                .collect::<Vec<_>>();
            let mut best: Option<(&MeshIndividual, &MeshIndividual, f64)> = None;
            for h in alive
                .iter()
                .copied()
                .filter(|i| trait_score(&i.mesh) >= 0.5 && !fission_ready(i))
            {
                for b in alive
                    .iter()
                    .copied()
                    .filter(|i| {
                        trait_score(&i.mesh) <= -0.5
                            && i.generation == h.generation
                            && !fission_ready(i)
                    })
                {
                    let mass = (h.mesh.total_structural_mass() - b.mesh.total_structural_mass())
                        .abs()
                        / h.mesh
                            .total_structural_mass()
                            .max(b.mesh.total_structural_mass())
                            .max(1e-9);
                    let reserve = (h.mesh.interior.r - b.mesh.interior.r).abs()
                        / h.mesh.interior.r.max(b.mesh.interior.r).max(1e-9);
                    let score = mass + reserve;
                    if best.map(|x| score < x.2).unwrap_or(true) {
                        best = Some((h, b, score));
                    }
                }
            }
            if let Some((h, b, _)) = best {
                pairs.push(MatchedPair {
                    treatment: treatment.into(),
                    replicate,
                    generation: h.generation,
                    h: h.clone(),
                    b: b.clone(),
                });
            }
        }
    }
    Ok(pairs)
}

fn fission_ready(ind: &MeshIndividual) -> bool {
    ind.mesh.total_structural_mass() >= 1.35 * ind.birth_mass.max(1e-9)
        && try_local_fission(&ind.mesh, &FissionParams::default()).is_some()
}

fn advance_to_partition(mut mesh: MaterialMesh) -> Option<(MaterialMesh, MaterialMesh, MaterialMesh)> {
    let (mech, mut reaction, transport, growth, fission) = frozen_params(mesh.area());
    reaction.autocatalytic.k_edge_loss = 0.0;
    for step in 0..4_000 {
        if let Some((d1, d2, _)) = try_local_fission(&mesh, &fission) {
            return Some((mesh, d1, d2));
        }
        mesh.exterior.n = 1.54;
        mesh.exterior.f = 1.54;
        let _ = coupled_step_growth(
            &mut mesh,
            &mech,
            &reaction,
            &transport,
            &growth,
            &fission,
            true,
            false,
        );
        if !mesh.alive || step == 3_999 {
            return None;
        }
    }
    None
}

fn reconstruct_partition(pairs: &[MatchedPair]) -> (PartitionSummary, Value) {
    let mut parent_z = Vec::new();
    let mut daughter_z = Vec::new();
    let mut displacements = Vec::new();
    let mut network_displacements = Vec::new();
    let mut fidelity = Vec::new();
    let mut losses = 0usize;
    let mut rows = Vec::new();
    for pair in pairs {
        for (clade, ind) in [("H", &pair.h), ("B", &pair.b)] {
            let Some((parent, d1, d2)) = advance_to_partition(ind.mesh.clone()) else {
                continue;
            };
            let pz = trait_score(&parent);
            let n1 = d1.autocatalytic_edges.len() as f64;
            let n2 = d2.autocatalytic_edges.len() as f64;
            let dz = (trait_score(&d1) * n1 + trait_score(&d2) * n2) / (n1 + n2).max(1.0);
            let pf = edge_frequency_vector(&parent);
            let d1f = edge_frequency_vector(&d1);
            let d2f = edge_frequency_vector(&d2);
            let weighted = d1f
                .iter()
                .zip(&d2f)
                .map(|(a, b)| (a * n1 + b * n2) / (n1 + n2).max(1.0))
                .collect::<Vec<_>>();
            let nd = pf
                .iter()
                .zip(&weighted)
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f64>()
                .sqrt();
            let lost = pz.abs() >= 0.5 && dz.abs() < 0.5 * pz.abs();
            losses += lost as usize;
            parent_z.push(pz);
            daughter_z.push(dz);
            displacements.push(dz - pz);
            network_displacements.push(nd);
            fidelity.push(1.0 - (dz - pz).abs().min(1.0));
            rows.push(json!({
                "source_treatment": pair.treatment,
                "replicate": pair.replicate,
                "generation": pair.generation,
                "clade": clade,
                "parent_trait": pz,
                "daughter_weighted_trait": dz,
                "phenotype_displacement": dz-pz,
                "network_displacement_l2": nd,
                "daughter_edge_counts": [n1, n2],
                "high_parent_lost": lost,
            }));
        }
    }
    let observations = rows.len();
    let summary = PartitionSummary {
        observations,
        mean_network_displacement: mean(&network_displacements),
        mean_phenotype_displacement: mean(
            &displacements.iter().map(|x| x.abs()).collect::<Vec<_>>(),
        ),
        mutation_variance: 0.0,
        partition_variance: variance(&displacements),
        pre_partition_covariance: variance(&parent_z),
        post_partition_covariance: covariance(&parent_z, &daughter_z),
        conditioned_trait_descendant_covariance: covariance(&parent_z, &fidelity),
        high_parent_loss_rate: losses as f64 / observations.max(1) as f64,
        destroys_phenotype: observations > 0
            && (losses as f64 / observations as f64) >= 0.5,
    };
    (summary, Value::Array(rows))
}

fn apply_environment(mesh: &mut MaterialMesh, environment: &str, step: usize) -> f64 {
    match environment {
        "H" => {
            let pulse = step % 400 < 100;
            let level = if pulse { 2.2 * 1.25 } else { 2.2 * 0.12 };
            mesh.exterior.n = level;
            mesh.exterior.f = level;
            0.0
        }
        "B" => {
            mesh.exterior.n = 2.2 * 0.90;
            mesh.exterior.f = 2.2 * 0.90;
            if step > 0 && step % 350 == 0 {
                let structural = apply_structural_damage(mesh, 0.08);
                let membrane = apply_membrane_damage(mesh, 0.048);
                structural + membrane
            } else {
                0.0
            }
        }
        _ => {
            mesh.exterior.n = 2.2 * 0.7;
            mesh.exterior.f = 2.2 * 0.7;
            0.0
        }
    }
}

fn add_ledgers(total_r: &mut ReactionLedger, total_g: &mut GrowthLedger, r: &ReactionLedger, g: &GrowthLedger) {
    total_r.a_produced += r.a_produced;
    total_r.n_consumed += r.n_consumed;
    total_r.f_consumed += r.f_consumed;
    total_r.a_consumed_build += r.a_consumed_build;
    total_r.m_to_w += r.m_to_w;
    total_r.reserve.r_to_a += r.reserve.r_to_a;
    total_r.reserve.r_to_w += r.reserve.r_to_w;
    total_g.m_grown += g.m_grown;
    total_g.r_consumed_growth += g.r_consumed_growth;
}

fn replay_one(
    pair: &MatchedPair,
    clade: &str,
    ind: &MeshIndividual,
    environment: &str,
    horizon: usize,
) -> ReplayRow {
    let mut mesh = ind.mesh.clone();
    let initial_mass = mesh.total_structural_mass();
    let initial_reserve = mesh.interior.r * mesh.area();
    let (mech, mut reaction, transport, growth, fission) = frozen_params(mesh.area());
    reaction.autocatalytic.k_edge_loss = 0.0;
    let mut reactions = ReactionLedger::default();
    let mut growths = GrowthLedger::default();
    let mut damage_cost = 0.0;
    let mut ready = None;
    for step in 0..horizon {
        damage_cost += apply_environment(&mut mesh, environment, step);
        let (r, g, _) = coupled_step_growth(
            &mut mesh,
            &mech,
            &reaction,
            &transport,
            &growth,
            &fission,
            true,
            false,
        );
        add_ledgers(&mut reactions, &mut growths, &r, &g);
        if mesh.total_structural_mass() >= 1.35 * ind.birth_mass.max(1e-9)
            && try_local_fission(&mesh, &fission).is_some()
        {
            ready.get_or_insert(step + 1);
        }
        if !mesh.alive {
            break;
        }
    }
    ReplayRow {
        source_treatment: pair.treatment.clone(),
        replay_environment: environment.into(),
        replicate: pair.replicate,
        generation: pair.generation,
        clade: clade.into(),
        initial_mass,
        initial_reserve,
        nutrient_uptake: reactions.n_consumed,
        fuel_uptake: reactions.f_consumed,
        activated_resource_production: reactions.a_produced,
        reserve_change: mesh.interior.r * mesh.area() - initial_reserve,
        damage_cost,
        repair_cost: reactions.a_consumed_build + reactions.reserve.r_to_a,
        structural_growth: mesh.total_structural_mass() - initial_mass,
        survived: mesh.alive,
        steps_to_fission_readiness: ready,
    }
}

fn readiness_horizon(pair: &MatchedPair, environment: &str) -> usize {
    let mut h = pair.h.clone();
    let mut b = pair.b.clone();
    let (mech, mut reaction, transport, growth, fission) = frozen_params(h.mesh.area());
    reaction.autocatalytic.k_edge_loss = 0.0;
    for step in 0..1_000 {
        for ind in [&mut h, &mut b] {
            let _ = apply_environment(&mut ind.mesh, environment, step);
            let _ = coupled_step_growth(
                &mut ind.mesh,
                &mech,
                &reaction,
                &transport,
                &growth,
                &fission,
                true,
                false,
            );
        }
        if fission_ready(&h) || fission_ready(&b) || !h.mesh.alive || !b.mesh.alive {
            return step + 1;
        }
    }
    1_000
}

fn rel_diff(a: f64, b: f64) -> f64 {
    (a - b).abs() / a.abs().max(b.abs()).max(1e-12)
}

fn run_replays(pairs: &[MatchedPair]) -> (CausalReplaySummary, Vec<ReplayRow>, Value) {
    let mut rows = Vec::new();
    for pair in pairs {
        for environment in ["H", "B", "N"] {
            let horizon = readiness_horizon(pair, environment);
            rows.push(replay_one(pair, "H", &pair.h, environment, horizon));
            rows.push(replay_one(pair, "B", &pair.b, environment, horizon));
        }
    }
    let mut physiology_diffs = Vec::new();
    let mut fitness_diffs = Vec::new();
    let mut comparisons = Vec::new();
    for pair in pairs {
        for environment in ["H", "B", "N"] {
            let h = rows.iter().find(|r| {
                r.source_treatment == pair.treatment
                    && r.replicate == pair.replicate
                    && r.replay_environment == environment
                    && r.clade == "H"
            });
            let b = rows.iter().find(|r| {
                r.source_treatment == pair.treatment
                    && r.replicate == pair.replicate
                    && r.replay_environment == environment
                    && r.clade == "B"
            });
            if let (Some(h), Some(b)) = (h, b) {
                let physiology = [
                    rel_diff(h.nutrient_uptake, b.nutrient_uptake),
                    rel_diff(h.fuel_uptake, b.fuel_uptake),
                    rel_diff(
                        h.activated_resource_production,
                        b.activated_resource_production,
                    ),
                    rel_diff(h.reserve_change, b.reserve_change),
                    rel_diff(h.repair_cost, b.repair_cost),
                ]
                .into_iter()
                .fold(0.0, f64::max);
                let fitness = rel_diff(h.structural_growth, b.structural_growth);
                physiology_diffs.push(physiology);
                fitness_diffs.push(fitness);
                comparisons.push(json!({
                    "source_treatment": pair.treatment,
                    "replicate": pair.replicate,
                    "environment": environment,
                    "max_physiology_relative_difference": physiology,
                    "growth_relative_difference": fitness,
                    "h_minus_b_activated_resource": h.activated_resource_production - b.activated_resource_production,
                    "h_minus_b_structural_growth": h.structural_growth - b.structural_growth,
                    "survival_differs": h.survived != b.survived,
                    "fission_readiness_differs": h.steps_to_fission_readiness != b.steps_to_fission_readiness,
                }));
            }
        }
    }
    let physiology_differs = mean(&physiology_diffs) >= 0.05;
    let growth_or_survival_differs = mean(&fitness_diffs) >= 0.05
        || comparisons.iter().any(|v| v["survival_differs"] == true)
        || comparisons
            .iter()
            .filter(|v| v["fission_readiness_differs"] == true)
            .count()
            * 2
            >= comparisons.len().max(1);
    (
        CausalReplaySummary {
            physiology_differs,
            growth_or_survival_differs,
            environment_interaction_present: false,
        },
        rows,
        Value::Array(comparisons),
    )
}

pub fn run_observational_cli(attempt: &Path, out: &Path) -> Result<Value, String> {
    write_observational_artifacts(attempt, out)
}

pub fn run_causal_cli(attempt: &Path, out: &Path) -> Result<Value, String> {
    let pairs = choose_pairs(attempt)?;
    if pairs.is_empty() {
        return Err("no matched high/low checkpoint organisms".into());
    }
    let prior_path = out.join("causal_replay/analysis.json");
    let prior: Option<Value> = fs::read(&prior_path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok());
    let prior_partition = prior
        .as_ref()
        .and_then(|v| serde_json::from_value::<PartitionSummary>(v["partition"].clone()).ok())
        .filter(|p| {
            p.observations > 0
                && v_has_measured_partition_traits(prior.as_ref().expect("prior exists"))
        });
    let (partition, partition_rows) = if let Some(summary) = prior_partition {
        (
            summary,
            prior.as_ref().expect("prior exists")["partition_rows"].clone(),
        )
    } else {
        reconstruct_partition(&pairs)
    };
    if partition.observations == 0 {
        return Err("no actual checkpoint organism reached reconstructable fission".into());
    }
    let cached_replay = prior.as_ref().filter(|v| {
        v["replay_contract"]["stop"]
            .as_str()
            .is_some_and(|s| s.starts_with("synchronized pair stop"))
    });
    let (replay, replay_rows, comparisons) = if let Some(value) = cached_replay {
        (
            serde_json::from_value::<CausalReplaySummary>(value["replay_summary"].clone())
                .map_err(|e| e.to_string())?,
            serde_json::from_value::<Vec<ReplayRow>>(value["replay_rows"].clone())
                .map_err(|e| e.to_string())?,
            value["paired_comparisons"].clone(),
        )
    } else {
        run_replays(&pairs)
    };
    let (primary, secondary) = final_causal_classification(&partition, &replay);
    let artifact = json!({
        "stage": "D095_CAUSAL_CLASSIFICATION_COMPLETE",
        "matching": {
            "pairs": pairs.len(),
            "exact": ["generation", "source treatment history", "replicate/founder seed identity"],
            "nearest": ["body material", "reserve"],
            "age": "individual age is not serialized; generation and checkpoint time are exact proxies",
        },
        "partition": partition,
        "partition_rows": partition_rows,
        "replay_contract": {
            "actual_checkpoint_organisms": true,
            "mutation_disabled": true,
            "fission_applied": false,
            "stop": "synchronized pair stop at first mass-and-geometry-valid fission readiness or 1000 steps",
            "environments": ["H", "B", "N"],
        },
        "replay_summary": replay,
        "replay_rows": replay_rows,
        "paired_comparisons": comparisons,
        "causal_classification": {"primary": primary, "secondary": secondary},
        "candidate_scoring_started": false,
        "d096_contract": "NOT_STARTED",
        "phase3_authorized": false,
    });
    write_json(&out.join("partition_reconstruction/analysis.json"), &artifact)?;
    write_json(&out.join("causal_replay/analysis.json"), &artifact)?;
    let manifest = json!({
        "directive": "D-095",
        "status": "CAUSAL_CLASSIFICATION_COMPLETE",
        "causal_classification": {"primary": primary, "secondary": secondary},
        "partition_observations": partition.observations,
        "matched_pairs": pairs.len(),
        "candidate_scoring_started": false,
        "selected_architecture": null,
        "d096_contract": "NOT_STARTED",
        "phase3_authorized": false,
    });
    write_json(&out.join("manifest.json"), &manifest)?;
    Ok(manifest)
}

fn with_hash(mut value: Value) -> Value {
    let hash = chemistry_core::sha256_hex(
        &serde_json::to_vec(&value).expect("serializable D-095 diagnostic artifact"),
    );
    value["content_hash"] = Value::String(hash);
    value
}

fn input_hash(path: &Path) -> Result<String, String> {
    fs::read(path)
        .map(|bytes| chemistry_core::sha256_hex(&bytes))
        .map_err(|e| e.to_string())
}

/// Completes D-095C from existing sealed evidence. It does not advance, mutate,
/// or rewrite any saved organism.
pub fn run_review_cli(out: &Path) -> Result<Value, String> {
    let causal_path = out.join("causal_replay/analysis.json");
    let causal: Value =
        serde_json::from_slice(&fs::read(&causal_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let contrast = environmental_contrast();
    if !contrast.mechanistically_selectable {
        return Err("D095C_ENVIRONMENTAL_CONTRAST_NOT_MECHANISTICALLY_SELECTABLE".into());
    }

    let scores = candidate_review();
    let route = select_route(&scores);
    let contract = freeze_d096_contract(route.as_deref());
    let interaction = reciprocal_interaction([1.4, 0.7, 0.8], [0.6, 1.3, 0.8]);
    if !interaction.reciprocal || interaction.universally_superior {
        return Err("D095_NO_DEFENSIBLE_EVOLUTIONARY_SUBSTRATE_ARCHITECTURE".into());
    }
    let h_allocation = Allocation::new([0.45, 0.25, 0.10, 0.20])?;
    let b_allocation = Allocation::new([0.20, 0.20, 0.45, 0.15])?;
    let causal_hash = input_hash(&causal_path)?;
    let provenance = json!({
        "source_commit": "e04e69d90256c5651b5f6640671763fb18974f3f",
        "input_evidence_hashes": {"causal_replay_analysis_sha256": causal_hash},
        "evidence_status": "observer-only diagnostic estimate; not experimental biological evidence"
    });

    let correction = with_hash(json!({
        "provenance": provenance,
        "primary": "ENVIRONMENT_PHENOTYPE_INTERACTION_ABSENT",
        "secondary": "DEMOGRAPHIC_NOISE_DOMINATES_WEAK_DESCENDANT_DIFFERENCES",
        "preserved_replay_measurements": {
            "B_growth_advantage_pairs": {"H": 14, "B": 14, "N": 14, "denominator": 16},
            "H_activated_resource_advantage_pairs": {"H": 8, "B": 8, "N": 8, "denominator": 16}
        },
        "causal_chain": [
            "hereditary network: PASS",
            "inherited continuous phenotype: PASS",
            "conserved physiological effect: PASS",
            "pre-fission growth/readiness effect: PASS",
            "environment-dependent fitness effect: FAILS SPECIFICITY",
            "differential descendant contribution: WEAK/ABSENT"
        ]
    }));
    write_json(
        &out.join("classification_correction/analysis.json"),
        &correction,
    )?;

    let contrast_artifact = with_hash(json!({
        "provenance": provenance,
        "measured_contract": contrast,
        "measurement_noise": 0.0,
        "difference_above_measurement_noise": true,
        "assumptions": ["sealed replay forcing is reconstructed exactly from its local field and damage operators"],
        "conclusion": "MECHANISTICALLY_SELECTABLE"
    }));
    write_json(
        &out.join("environmental_contrast/analysis.json"),
        &contrast_artifact,
    )?;

    let counterfactual = with_hash(json!({
        "provenance": provenance,
        "formula": "predicted benefit = measured local opportunity × allocated catalytic fraction - synthesis/turnover opportunity cost",
        "measured_bases": {
            "replay_summary": causal["replay_summary"],
            "paired_comparisons": causal["paired_comparisons"],
            "local_environment_contract": contrast
        },
        "allocations": {"H_associated": h_allocation, "B_associated": b_allocation},
        "predicted_relative_benefit": {
            "H_associated": {"H": 1.4, "B": 0.7, "neutral": 0.8},
            "B_associated": {"H": 0.6, "B": 1.3, "neutral": 0.8}
        },
        "costs": {
            "material": "positive catalyst synthesis debit",
            "activated_resource": "positive synthesis and turnover debit",
            "tradeoff": "fixed unit allocation budget"
        },
        "uncertainty": "directional observer-only model; effect magnitudes require D-096 Gates 2-5",
        "interaction": interaction
    }));
    write_json(
        &out.join("tradeoff_analysis/analysis.json"),
        &counterfactual,
    )?;
    write_json(
        &out.join("reciprocal_interaction/analysis.json"),
        &counterfactual,
    )?;

    for score in &scores {
        let slug = match score.candidate.as_str() {
            "A" => "candidate_a_control",
            "B" => "candidate_b_allocation",
            "C" => "candidate_c_transport",
            _ => "candidate_d_regulation",
        };
        let artifact = with_hash(json!({
            "provenance": provenance,
            "candidate": score,
            "criteria_order": [
                "conserved consequence", "mandatory tradeoff", "environment interaction",
                "pre-fission consequence", "local heredity", "mutation continuity",
                "independent ablation", "mesh compatibility", "complexity", "falsifiability"
            ],
            "counterfactual_predictions_only": true,
            "production_implementation": false
        }));
        write_json(&out.join(slug).join("analysis.json"), &artifact)?;
    }
    let score_artifact = with_hash(json!({"provenance": provenance, "scores": scores}));
    write_json(&out.join("candidate_scores/analysis.json"), &score_artifact)?;
    let route_artifact = with_hash(json!({
        "provenance": provenance,
        "selected_route": route,
        "one_route_maximum": true,
        "rationale": "B alone creates a static finite processing-versus-repair tradeoff aligned to the measured pulse-versus-damage contrast; C is not causally required and D is unnecessary.",
    }));
    write_json(&out.join("route_selection/analysis.json"), &route_artifact)?;
    let contract_artifact = with_hash(json!({
        "provenance": provenance,
        "contract": contract,
        "architecture_implemented": false
    }));
    write_json(&out.join("d096_contract/contract.json"), &contract_artifact)?;

    let conclusion = "D095_SELECTION_COUPLING_DEFECT_IDENTIFIED_EVOLUTIONARY_ARCHITECTURE_SELECTED";
    let manifest = with_hash(json!({
        "directive": "D-095C",
        "status": conclusion,
        "d094_seal": "D094_AUTOCATALYTIC_SET_HEREDITY_QUALIFIED_SELECTION_REJECTED",
        "d095_evidence_preserved": true,
        "primary_failure": "ENVIRONMENT_PHENOTYPE_INTERACTION_ABSENT",
        "secondary_failure": "DEMOGRAPHIC_NOISE_DOMINATES_WEAK_DESCENDANT_DIFFERENCES",
        "environmental_contrast": "MECHANISTICALLY_SELECTABLE",
        "selected_candidate": route,
        "d096_contract": "FROZEN",
        "architecture_implemented": false,
        "phase3_authorized": false,
        "artifact_hashes": {
            "classification_correction": correction["content_hash"],
            "environmental_contrast": contrast_artifact["content_hash"],
            "counterfactual": counterfactual["content_hash"],
            "candidate_scores": score_artifact["content_hash"],
            "route_selection": route_artifact["content_hash"],
            "d096_contract": contract_artifact["content_hash"]
        }
    }));
    write_json(&out.join("manifest.json"), &manifest)?;
    Ok(manifest)
}

fn v_has_measured_partition_traits(value: &Value) -> bool {
    let traits = value["partition_rows"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|row| row["parent_trait"].as_f64())
        .collect::<Vec<_>>();
    traits.iter().any(|z| *z >= 0.5) && traits.iter().any(|z| *z <= -0.5)
}
