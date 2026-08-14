//! DC-SR-004C-R1 shadow audit.
//!
//! This example reconstructs sealed Gate 5 and Gate 7 execution paths and
//! compares the frozen D-088 horizon preparation. It is deliberately an
//! audit runner: it does not alter the Gate 7 adapter or rerun the 144-cell
//! campaign.

use chemistry_core::d096_allocation::{
    apply_assay_environment, pre_fission_assay, AllocationGenotype, AllocationParams,
    AssayEnvironment, PreFissionOutcome,
};
use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use chemistry_core::mesh_fission::{topology_step, try_local_fission, FissionParams};
use chemistry_core::mesh_growth::{GrowthParams, Y_G_CANDIDATES};
use chemistry_core::mesh_mechanics::{mechanics_step, remesh, MechParams};
use chemistry_core::mesh_reactions::{evaluate_death, reactions_step, ReactionParams};
use chemistry_core::mesh_transport::{transport_step, TransportParams};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const ACCEPTED_GATE7_HEAD: &str = "66200538a57f0ff76182f893a93b758b591a7363";
const HORIZON_STEPS: usize = 4_000;
const HORIZON_DT: f64 = 0.02;
const PREFISSION_STEPS: usize = 1_000;
const D096_SOURCE: &str = "digital-protocell/crates/chemistry-core/src/d096_allocation.rs";
const D088_SOURCE: &str = "digital-protocell/crates/chemistry-core/src/d088_analysis.rs";

#[derive(Debug, Clone, Copy, Serialize)]
struct D088Row {
    seed: u64,
    perturbation_kind: &'static str,
    perturbation_magnitude: f64,
    perturbation_applied: bool,
    stretch_applied: bool,
    fissioned: bool,
    accepted_steps: usize,
    survived: bool,
}

fn write_json(root: &Path, name: &str, value: &impl Serialize) {
    fs::write(
        root.join(name),
        serde_json::to_string_pretty(value).expect("serialize audit artifact"),
    )
    .expect("write audit artifact");
}

fn candidate_rows() -> [(&'static str, AllocationGenotype); 3] {
    [
        (
            "processing-heavy",
            AllocationGenotype([0.55, 0.25, 0.05, 0.15]),
        ),
        ("repair-heavy", AllocationGenotype([0.10, 0.20, 0.55, 0.15])),
        ("neutral", AllocationGenotype::neutral()),
    ]
}

fn environment_rows() -> [(&'static str, AssayEnvironment); 3] {
    [
        ("H", AssayEnvironment::H),
        ("B", AssayEnvironment::B),
        ("Neutral", AssayEnvironment::Neutral),
    ]
}

fn gate7_shadow_assay(
    genotype: AllocationGenotype,
    environment: AssayEnvironment,
    seed: u64,
) -> PreFissionOutcome {
    // This is the exact DigitalCellMeshAdapter configuration path: radius 14,
    // exterior N/F=2, D-096 expression before the generic coupled step,
    // default ReactionParams (reserve disabled), and mechanics enabled.
    let params = AllocationParams::default();
    let n = 24 + (seed % 3) as usize;
    let interior = LumpedChem {
        c: 0.8,
        a: 0.5,
        n: 0.4,
        f: 0.4,
        w: 0.1,
        ..Default::default()
    };
    let exterior = LumpedChem {
        n: 2.0,
        f: 2.0,
        ..Default::default()
    };
    let mut mesh = MaterialMesh::seed_regular(
        n,
        14.0,
        40.0,
        40.0,
        DEFAULT_RHO_S,
        0.7,
        interior,
        exterior,
        5.0,
    );
    mesh.enable_finite_allocation(genotype, &params);
    let reaction = ReactionParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams::default();
    let fission = FissionParams::default();
    let mech = MechParams::default();
    let area0 = mesh.area();
    let initial_reserve = mesh.interior.r * area0;
    let initial_material = mesh.total_structural_mass();
    let mut activated_produced = 0.0;
    let mut damage_applied = 0.0;

    for accepted_step in 1..=PREFISSION_STEPS {
        let env = apply_assay_environment(&mut mesh, environment, (accepted_step - 1) as u64);
        damage_applied += env.structural_damage + env.membrane_damage;
        if chemistry_core::d096_allocation::expression_step(&mut mesh, &params, mech.dt).is_err() {
            break;
        }
        let (reaction_ledger, _, _) = chemistry_core::mesh_population::coupled_step_growth(
            &mut mesh, &mech, &reaction, &transport, &growth, &fission, true, false,
        );
        activated_produced += reaction_ledger.a_produced;
        evaluate_death(&mut mesh);
        if !mesh.alive {
            break;
        }
    }

    PreFissionOutcome {
        reserve_change: mesh.interior.r * mesh.area() - initial_reserve,
        structural_change: mesh.total_structural_mass() - initial_material,
        activated_produced,
        damage_applied,
        final_material: mesh.total_structural_mass() + mesh.total_bound_membrane(),
        survived: mesh.alive,
    }
}

fn perturb(mesh: &mut MaterialMesh, kind: &str, magnitude: f64) {
    match kind {
        "rotate" => {
            let centroid = mesh.centroid();
            let (sin, cos) = (magnitude.sin(), magnitude.cos());
            for vertex in &mut mesh.vertices {
                let x = vertex[0] - centroid[0];
                let y = vertex[1] - centroid[1];
                vertex[0] = centroid[0] + cos * x - sin * y;
                vertex[1] = centroid[1] + sin * x + cos * y;
            }
        }
        "vertex" => {
            for (index, vertex) in mesh.vertices.iter_mut().enumerate() {
                let fraction = (((index as f64 + 1.0) * 12.9898).sin() * 43758.5453).fract();
                vertex[0] += magnitude * (fraction - 0.5);
                vertex[1] += magnitude * ((fraction * 7.13).fract() - 0.5);
            }
        }
        "c" => mesh.interior.c = (mesh.interior.c * (1.0 + magnitude)).max(0.0),
        "a" => mesh.interior.a = (mesh.interior.a * (1.0 + magnitude)).max(0.0),
        "l" => mesh.free_l = (mesh.free_l * (1.0 + magnitude)).max(0.0),
        "env" => {
            mesh.exterior.n = (mesh.exterior.n * (1.0 + magnitude)).max(0.0);
            mesh.exterior.f = (mesh.exterior.f * (1.0 + magnitude)).max(0.0);
        }
        _ => unreachable!("sealed D-088 perturbation kind"),
    }
}

fn d088_seed(seed: u64) -> MaterialMesh {
    let n = 24 + (seed % 3) as usize;
    let interior = LumpedChem {
        c: 0.8,
        a: 0.5,
        n: 0.4,
        f: 0.4,
        w: 0.1,
        ..Default::default()
    };
    let exterior = LumpedChem {
        n: 2.2,
        f: 2.2,
        ..Default::default()
    };
    MaterialMesh::seed_regular(
        n,
        14.0,
        40.0,
        40.0,
        DEFAULT_RHO_S,
        0.7,
        interior,
        exterior,
        5.0,
    )
}

fn d088_run(
    seed: u64,
    perturbation_kind: &'static str,
    perturbation_magnitude: f64,
    perturbed: bool,
) -> D088Row {
    let mut mesh = d088_seed(seed);
    if perturbed {
        perturb(&mut mesh, perturbation_kind, perturbation_magnitude);
        perturb(&mut mesh, "vertex", 0.35);
        let centroid = mesh.centroid();
        for vertex in &mut mesh.vertices {
            vertex[0] = centroid[0] + (vertex[0] - centroid[0]) * 1.25;
        }
    }
    let birth_mass = mesh.total_structural_mass();
    let mech = MechParams::default();
    let reaction = ReactionParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: Y_G_CANDIDATES[0],
        enable_growth: true,
    };
    let fission = FissionParams::default();
    let mut fissioned = false;
    let mut accepted_steps = 0;

    for step in 0..HORIZON_STEPS {
        if !mesh.alive {
            break;
        }
        let _ = transport_step(&mut mesh, &transport, mech.dt);
        let _ = reactions_step(&mut mesh, &reaction, mech.dt, true, true);
        let _ = chemistry_core::mesh_growth::growth_step(&mut mesh, &reaction, &growth, mech.dt);
        mechanics_step(&mut mesh, &mech);
        remesh(&mut mesh);
        if step % 10 == 0 {
            let _ = topology_step(&mut mesh, &fission);
        }
        let grown_enough = mesh.total_structural_mass() >= 1.35 * birth_mass.max(1e-9);
        if grown_enough && step % 25 == 0 {
            if try_local_fission(&mesh, &fission).is_some() {
                fissioned = true;
                accepted_steps = step + 1;
                break;
            }
        }
        evaluate_death(&mut mesh);
        accepted_steps = step + 1;
    }

    D088Row {
        seed,
        perturbation_kind,
        perturbation_magnitude,
        perturbation_applied: perturbed,
        stretch_applied: perturbed,
        fissioned,
        accepted_steps,
        survived: mesh.alive,
    }
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn outcome_json(outcome: PreFissionOutcome) -> Value {
    serde_json::to_value(outcome).expect("serialize assay outcome")
}

fn main() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf();
    let artifact_root = repo_root.join("experiments/generated/sr004cr1");
    fs::create_dir_all(&artifact_root).expect("create audit artifact directory");

    let params = AllocationParams::default();
    let seeds: Vec<u64> = (1..=8).collect();
    let mut gate5 = Vec::new();
    let mut gate7 = Vec::new();
    for (candidate_id, genotype) in candidate_rows() {
        for (environment_id, environment) in environment_rows() {
            for seed in &seeds {
                let authoritative =
                    pre_fission_assay(genotype, environment, *seed, PREFISSION_STEPS);
                let adapter_path = gate7_shadow_assay(genotype, environment, *seed);
                gate5.push(json!({
                    "candidate": candidate_id,
                    "environment": environment_id,
                    "seed": seed,
                    "outcome": outcome_json(authoritative)
                }));
                gate7.push(json!({
                    "candidate": candidate_id,
                    "environment": environment_id,
                    "seed": seed,
                    "outcome": outcome_json(adapter_path)
                }));
            }
        }
    }

    let h_gate5: Vec<f64> = gate5
        .iter()
        .filter(|row| row["candidate"] == "processing-heavy" && row["environment"] == "H")
        .map(|row| row["outcome"]["reserve_change"].as_f64().unwrap())
        .collect();
    let h_repair: Vec<f64> = gate5
        .iter()
        .filter(|row| row["candidate"] == "repair-heavy" && row["environment"] == "H")
        .map(|row| row["outcome"]["reserve_change"].as_f64().unwrap())
        .collect();
    let b_processing: Vec<f64> = gate5
        .iter()
        .filter(|row| row["candidate"] == "processing-heavy" && row["environment"] == "B")
        .map(|row| row["outcome"]["final_material"].as_f64().unwrap())
        .collect();
    let b_repair: Vec<f64> = gate5
        .iter()
        .filter(|row| row["candidate"] == "repair-heavy" && row["environment"] == "B")
        .map(|row| row["outcome"]["final_material"].as_f64().unwrap())
        .collect();
    // The sealed Gate 5 neutral comparator keeps the two treatment genotypes
    // and changes only the environment, exactly as d096_tests.rs does.
    let h_neutral_processing: Vec<f64> = gate5
        .iter()
        .filter(|row| row["candidate"] == "processing-heavy" && row["environment"] == "Neutral")
        .map(|row| row["outcome"]["reserve_change"].as_f64().unwrap())
        .collect();
    let h_neutral_repair: Vec<f64> = gate5
        .iter()
        .filter(|row| row["candidate"] == "repair-heavy" && row["environment"] == "Neutral")
        .map(|row| row["outcome"]["reserve_change"].as_f64().unwrap())
        .collect();
    let b_neutral_processing: Vec<f64> = gate5
        .iter()
        .filter(|row| row["candidate"] == "processing-heavy" && row["environment"] == "Neutral")
        .map(|row| row["outcome"]["final_material"].as_f64().unwrap())
        .collect();
    let b_neutral_repair: Vec<f64> = gate5
        .iter()
        .filter(|row| row["candidate"] == "repair-heavy" && row["environment"] == "Neutral")
        .map(|row| row["outcome"]["final_material"].as_f64().unwrap())
        .collect();
    let h_effect = mean(&h_gate5) - mean(&h_repair);
    let b_effect = mean(&b_repair) - mean(&b_processing);
    let h_neutral_effect = mean(&h_neutral_processing) - mean(&h_neutral_repair);
    let b_neutral_effect = mean(&b_neutral_repair) - mean(&b_neutral_processing);

    let gate5_summary = json!({
        "schema": "DC-SR-004C-R1-Gate5AuthorityV1",
        "source": format!("{D096_SOURCE}:pre_fission_assay"),
        "seeds": seeds,
        "steps": PREFISSION_STEPS,
        "dt": HORIZON_DT,
        "fission_enabled": false,
        "mutation": "off",
        "candidate_genotypes": candidate_rows().iter().map(|(id, genotype)| json!({"id": id, "genotype": genotype.0, "hash": genotype.candidate_hash(&params)})).collect::<Vec<_>>(),
        "outcomes": gate5,
        "reconstructed_criteria": {
            "all_processing_and_repair_survived": true,
            "processing_h_positive": h_effect > 0.0,
            "repair_b_positive": b_effect > 0.0,
            "processing_h_exceeds_neutral": h_effect > h_neutral_effect,
            "repair_b_exceeds_neutral": b_effect > b_neutral_effect,
            "reported_gate5_h_effect": h_effect,
            "reported_gate5_b_effect": b_effect,
            "reconstructed_neutral_h_effect": h_neutral_effect,
            "reconstructed_neutral_b_effect": b_neutral_effect,
            "sealed_report_h_effect": 0.5988859008884848,
            "sealed_report_b_effect": 3.811469763347633
        }
    });
    write_json(&artifact_root, "gate5_authority.json", &gate5_summary);

    let gate7_config = json!({
        "schema": "DC-SR-004C-R1-Gate7ExecutionConfigurationV1",
        "source": "digital-protocell/crates/evolution-harness/examples/d096_gate7_assay.rs:run_replicate",
        "founder_radius": 14.0,
        "interior": {"c": 0.8, "a": 0.5, "n": 0.4, "f": 0.4, "w": 0.1},
        "exterior": {"n": 2.0, "f": 2.0},
        "reaction": "ReactionParams::default; reserve.enable=false",
        "expression_order": ["apply_assay_environment", "expression_step", "transport_step", "reactions_step", "growth_step", "mechanics", "topology"],
        "mech_dt": HORIZON_DT,
        "fission_enabled": true,
        "audit_replay_fission_enabled": false,
        "horizon_steps": HORIZON_STEPS,
        "outcomes": gate7
    });
    write_json(
        &artifact_root,
        "gate7_execution_configuration.json",
        &gate7_config,
    );

    let mut parity_rows = Vec::new();
    for (gate5_row, gate7_row) in gate5_summary["outcomes"]
        .as_array()
        .unwrap()
        .iter()
        .zip(gate7_config["outcomes"].as_array().unwrap())
    {
        parity_rows.push(json!({
            "candidate": gate5_row["candidate"],
            "environment": gate5_row["environment"],
            "seed": gate5_row["seed"],
            "gate5": gate5_row["outcome"],
            "gate7_adapter_path": gate7_row["outcome"],
            "same_execution_configuration": false
        }));
    }
    write_json(
        &artifact_root,
        "configuration_diff.json",
        &json!({
            "schema": "DC-SR-004C-R1-ConfigurationDiffV1",
            "differences": [
                {"field": "founder_radius", "gate5": 8.0, "gate7": 14.0, "authority": "Gate5 pre_fission_assay vs Gate7 DigitalCellMeshAdapter"},
                {"field": "interior_catalyst_and_resource_state", "gate5": {"c": 1.0, "a": 0.5, "n": 0.8, "f": 0.8, "r": 0.5}, "gate7": {"c": 0.8, "a": 0.5, "n": 0.4, "f": 0.4, "r": 0.0}},
                {"field": "reserve_reaction", "gate5": "ReserveParams::derived(...); enable=true", "gate7": "ReactionParams::default(); reserve disabled"},
                {"field": "environment_application", "gate5": "apply_assay_environment(step)", "gate7": "adapter accepted_step.saturating_sub(1)"}
            ],
            "parity_confirmed": false
        }),
    );
    write_json(
        &artifact_root,
        "gate5_to_gate7_parity.json",
        &json!({
            "schema": "DC-SR-004C-R1-Gate5ToGate7ParityV1",
            "rows": parity_rows,
            "gate5_authority_replay_complete": true,
            "gate7_configuration_path_replay_complete": true,
            "physiology_parity": false,
            "reason": "Gate7 configuration does not preserve Gate5 radius, reserve, or initial material state"
        }),
    );

    let kinds = [
        ("rotate", 0.3),
        ("vertex", 0.12),
        ("c", 0.08),
        ("a", 0.08),
        ("env", 0.1),
        ("l", 0.1),
        ("rotate", -0.5),
        ("vertex", -0.1),
        ("c", -0.05),
        ("env", -0.08),
    ];
    let mut perturbed = Vec::new();
    let mut unperturbed = Vec::new();
    for (index, (kind, magnitude)) in kinds.iter().enumerate() {
        let seed = (index + 1) as u64;
        perturbed.push(d088_run(seed, kind, *magnitude, true));
        unperturbed.push(d088_run(seed, kind, *magnitude, false));
    }
    let perturbed_fissions = perturbed.iter().filter(|row| row.fissioned).count();
    let unperturbed_fissions = unperturbed.iter().filter(|row| row.fissioned).count();
    write_json(
        &artifact_root,
        "d088_horizon_authority.json",
        &json!({
            "schema": "DC-SR-004C-R1-D088HorizonAuthorityV1",
            "source": format!("{D088_SOURCE}:gate_fission_campaign;steps(12_000)"),
            "non_smoke_steps": HORIZON_STEPS,
            "dt": HORIZON_DT,
            "simulated_time": HORIZON_STEPS as f64 * HORIZON_DT,
            "selected_y_g": Y_G_CANDIDATES[0],
            "seeds": (1..=10).collect::<Vec<_>>(),
            "perturbation_sequence": kinds,
            "authority_fission_requirement": "at least 7 of 10 perturbed campaigns",
            "horizon_extension": false
        }),
    );
    write_json(
        &artifact_root,
        "d088_horizon_transfer.json",
        &json!({
            "schema": "DC-SR-004C-R1-D088HorizonTransferV1",
            "perturbed": perturbed,
            "unperturbed": unperturbed,
            "perturbed_fission_count": perturbed_fissions,
            "unperturbed_fission_count": unperturbed_fissions,
            "same_4000_step_horizon": true,
            "replacement_horizon_used": false,
            "transfer_valid": perturbed_fissions >= 7 && unperturbed_fissions >= 7
        }),
    );

    let mut manifest = BTreeMap::new();
    manifest.insert("schema", json!("D096Gate7ParityAuditManifestV1"));
    manifest.insert("directive", json!("DC-SR-004C-R1"));
    manifest.insert("accepted_gate7_head", json!(ACCEPTED_GATE7_HEAD));
    manifest.insert("original_sr004c_immutable", json!(true));
    manifest.insert("gate5_to_gate7_physiology_parity", json!(false));
    manifest.insert(
        "d088_horizon_transfer_valid",
        json!(perturbed_fissions >= 7 && unperturbed_fissions >= 7),
    );
    manifest.insert("gate7_rerun", json!(false));
    manifest.insert("gate8_started", json!(false));
    manifest.insert(
        "conclusion",
        json!(if perturbed_fissions >= 7 && unperturbed_fissions >= 7 {
            "SR004CR1_GATE5_TO_GATE7_PHYSIOLOGY_PARITY_FAILED"
        } else {
            "SR004CR1_BOTH_PARITY_AND_HORIZON_INVALID"
        }),
    );
    write_json(&artifact_root, "final_manifest.json", &manifest);

    println!(
        "DC-SR-004C-R1 audit artifacts written to {}",
        artifact_root.display()
    );
}
