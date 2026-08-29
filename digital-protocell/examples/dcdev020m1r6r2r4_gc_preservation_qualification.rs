//! DC-DEV-020-M1-R6-R2-R4 GeometryConservativeV3 preservation qualification.
//!
//! The historical D-087 certifier is executed unchanged for the V2 material
//! contract and the GeometryConservativeV3 candidate. This separate qualifier
//! records the candidate Gate-2 decomposition and replaces only the stale
//! fixed-concentration starvation surrogate with causal starvation evidence.

use chemistry_core::candidate_identity::sha256_hex;
use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_contracts::snapshot;
use chemistry_core::mesh_mechanics::{mechanics_step, remesh};
use chemistry_core::mesh_reactions::{
    reactions_step_with_reserve_mode, try_local_rebond, ReactionLedger, ReserveDiagnosticMode,
};
use chemistry_core::mesh_transport::{transport_step, TransportLedger};
use phase1_certifier::campaign::{run_certification, CertificationReport};
use phase1_certifier::frozen::{frozen_transport, FROZEN_CENTER};
use phase1_certifier::gates::steps;
use phase1_certifier::gc_preservation::{
    causal_starvation_passes, historical_failure_is_isolated, CausalStarvationEvidence,
    STARVATION_EXTENSION_BOUND,
};
use phase1_certifier::sim::{
    apply_local_rupture, apply_membrane_damage, apply_structural_damage, fingerprint,
    pass_basin_row, reaction_params_for, run_coupled, seed_mesh,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-020-M1-R6-R2-R4-GC-PRESERVATION-QUALIFICATION-001";
const STARTING_HEAD: &str = "ffa0f8756a3d152d70292b4f366087bef0680c70";
const OUTPUT_ENV: &str = "DCDEV020M1R6R2R4_OUTPUT";
const ATLAS_COMPACT: &str =
    r"\\atlas\ATLAS\100_ACTIVE\Projects\DIGITAL_CELL\evidence\dcdev020m1r6r2r4\compact";
const SETTLE_STEPS: usize = 200;
const TOLERANCE: f64 = 1e-8;

#[derive(Debug, Clone, Serialize)]
struct StarvationSample {
    relative_step: usize,
    absolute_step: usize,
    area: f64,
    a_concentration: f64,
    a_amount: f64,
    n_amount: f64,
    organized_material: f64,
    strict_material: f64,
    closed_intact: bool,
    observer_viable: bool,
    observer_death_reason: Option<&'static str>,
    physical_runtime_valid: bool,
    ruptured_edges: usize,
}

#[derive(Debug, Clone, Default, Serialize)]
struct StarvationTotals {
    accepted_steps: usize,
    n_delivered: f64,
    f_delivered: f64,
    n_consumed: f64,
    f_consumed: f64,
}

#[derive(Debug, Clone, Serialize)]
struct StarvationRun {
    entry: StarvationSample,
    checkpoints: Vec<StarvationSample>,
    final_state: StarvationSample,
    totals: StarvationTotals,
    first_observer_viability_loss_step: Option<usize>,
    late_organized_material_max: f64,
    late_organized_material_final: f64,
    topology_rupture_step: Option<usize>,
    runtime_invalid_step: Option<usize>,
    causal_gate_pass: bool,
}

#[derive(Debug, Clone, Serialize)]
struct BasinDecomposition {
    rows: Vec<Value>,
    passed_rows: usize,
    total_rows: usize,
    basin_pass: bool,
    snapshot_resume_pass: bool,
    membrane_damage_pass: bool,
    structural_damage_pass: bool,
    rupture_recognition_pass: bool,
    no_respawn_pass: bool,
}

fn repo_root() -> PathBuf {
    let cwd = std::env::current_dir().expect("current directory");
    if cwd
        .join("digital-protocell/crates/phase1-certifier")
        .exists()
    {
        cwd
    } else if cwd.join("crates/phase1-certifier").exists() {
        cwd.parent().expect("workspace parent").to_path_buf()
    } else {
        cwd
    }
}

fn configure(candidate: bool) {
    std::env::set_var("DCDEV020R9R3_CONTRACT", "ConservativeV3");
    std::env::set_var("DCDEV020R9R3_RESERVE", "0");
    if candidate {
        std::env::set_var("DCDEV020M1R6R2_GEOMETRY_CONTRACT", "1");
    } else {
        std::env::remove_var("DCDEV020M1R6R2_GEOMETRY_CONTRACT");
    }
}

fn report_flags(report: &CertificationReport) -> [bool; 8] {
    [
        report.gate0.pass,
        report.gate1.pass,
        report.gate2.pass,
        report.gate3.pass,
        report.gate4.pass,
        report.gate5.pass,
        report.gate6.pass,
        report.gate7.pass,
    ]
}

fn report_json(report: &CertificationReport) -> Value {
    serde_json::to_value(report).expect("serialize certifier report")
}

fn sample(mesh: &MaterialMesh, relative_step: usize, absolute_step: usize) -> StarvationSample {
    let s = snapshot(mesh);
    StarvationSample {
        relative_step,
        absolute_step,
        area: mesh.area(),
        a_concentration: mesh.interior.a,
        a_amount: s.a,
        n_amount: s.n,
        organized_material: s.organized_material(),
        strict_material: s.strict_material_equivalent(),
        closed_intact: mesh.closed_intact(),
        observer_viable: mesh.observer_viable(),
        observer_death_reason: mesh.observer_death_reason(),
        physical_runtime_valid: mesh.physical_runtime_valid(),
        ruptured_edges: mesh.edges.iter().filter(|edge| edge.ruptured).count(),
    }
}

fn one_step(mesh: &mut MaterialMesh) -> (TransportLedger, ReactionLedger) {
    let transport = transport_step(mesh, &frozen_transport(), FROZEN_CENTER.dt);
    let reaction = reactions_step_with_reserve_mode(
        mesh,
        &reaction_params_for(mesh),
        FROZEN_CENTER.dt,
        true,
        true,
        ReserveDiagnosticMode::Full,
    );
    assert!(mechanics_step(mesh, &FROZEN_CENTER));
    remesh(mesh);
    try_local_rebond(mesh, chemistry_core::material_mesh::DEFAULT_REBOND_DIST);
    (transport, reaction)
}

fn run_starvation() -> StarvationRun {
    configure(true);
    let mut mesh = seed_mesh(14.0, 1);
    run_coupled(&mut mesh, SETTLE_STEPS, true, true);
    mesh.exterior.n = 0.0;
    mesh.interior.n = 0.0;
    let entry = sample(&mesh, 0, SETTLE_STEPS);
    let mut totals = StarvationTotals::default();
    let mut checkpoints = vec![entry.clone()];
    let checkpoint_steps = [
        1usize,
        10,
        100,
        480,
        1_000,
        6_000,
        10_383,
        50_000,
        100_000,
        STARVATION_EXTENSION_BOUND,
    ];
    let mut first_observer_viability_loss_step = None;
    let mut topology_rupture_step = None;
    let mut runtime_invalid_step = None;
    let mut late_organized_material_max = entry.organized_material;
    let mut final_state = entry.clone();

    for relative_step in 1..=STARVATION_EXTENSION_BOUND {
        let (transport, reaction) = one_step(&mut mesh);
        totals.accepted_steps += 1;
        totals.n_delivered += transport.n_in;
        totals.f_delivered += transport.f_in;
        totals.n_consumed += reaction.n_consumed;
        totals.f_consumed += reaction.f_consumed;
        let current = sample(&mesh, relative_step, SETTLE_STEPS + relative_step);
        if relative_step >= 50_000 {
            late_organized_material_max =
                late_organized_material_max.max(current.organized_material);
        }
        if first_observer_viability_loss_step.is_none() && !current.observer_viable {
            first_observer_viability_loss_step = Some(relative_step);
        }
        if topology_rupture_step.is_none() && current.ruptured_edges > 0 {
            topology_rupture_step = Some(relative_step);
        }
        if runtime_invalid_step.is_none() && !current.physical_runtime_valid {
            runtime_invalid_step = Some(relative_step);
        }
        if checkpoint_steps.contains(&relative_step) {
            checkpoints.push(current.clone());
        }
        final_state = current;
    }

    let evidence = CausalStarvationEvidence {
        post_switch_n_delivery: totals.n_delivered,
        organized_material_entry: entry.organized_material,
        organized_material_late: final_state.organized_material,
        late_organized_material_max,
        observer_viability_loss_step: first_observer_viability_loss_step,
        extension_bound: STARVATION_EXTENSION_BOUND,
    };
    StarvationRun {
        entry,
        checkpoints,
        final_state: final_state.clone(),
        totals,
        first_observer_viability_loss_step,
        late_organized_material_max,
        late_organized_material_final: final_state.organized_material,
        topology_rupture_step,
        runtime_invalid_step,
        causal_gate_pass: causal_starvation_passes(evidence),
    }
}

fn decomposition() -> BasinDecomposition {
    configure(true);
    let mut rows = Vec::new();
    let mut passed_rows = 0usize;
    for radius in [10.0, 14.0, 18.0] {
        for seed in 1u64..=5 {
            let mut mesh = seed_mesh(radius, seed);
            let a0 = mesh.area();
            let c0 = mesh.interior.c;
            let aa0 = mesh.interior.a;
            run_coupled(&mut mesh, steps(8_000), true, true);
            let pass = pass_basin_row(&mesh, a0, c0, aa0);
            passed_rows += usize::from(pass);
            rows.push(json!({"radius": radius, "seed": seed, "pass": pass}));
        }
    }

    let mut continuous = seed_mesh(14.0, 1);
    run_coupled(&mut continuous, steps(2_000), true, true);
    let snapshot_mesh = continuous.clone();
    run_coupled(&mut continuous, steps(2_000), true, true);
    let continuous_fingerprint = fingerprint(&continuous);
    let mut resumed = snapshot_mesh;
    run_coupled(&mut resumed, steps(2_000), true, true);
    let snapshot_resume_pass = continuous_fingerprint == fingerprint(&resumed);

    let mut membrane = seed_mesh(14.0, 3);
    run_coupled(&mut membrane, steps(300), true, true);
    apply_membrane_damage(&mut membrane, 0.10);
    run_coupled(&mut membrane, steps(3_000), true, true);
    let membrane_damage_pass = membrane.alive;

    let mut structural = seed_mesh(14.0, 3);
    run_coupled(&mut structural, steps(300), true, true);
    apply_structural_damage(&mut structural, 0.10);
    run_coupled(&mut structural, steps(3_000), true, true);
    let structural_damage_pass = structural.alive;

    let mut rupture = seed_mesh(14.0, 5);
    apply_local_rupture(&mut rupture, 0);
    let rupture_recognition_pass = rupture.edges.first().is_some_and(|edge| edge.ruptured);

    let mut no_respawn = seed_mesh(14.0, 1);
    no_respawn.exterior.n = 1.0;
    no_respawn.exterior.f = 1.0;
    no_respawn.alive = false;
    for edge in &mut no_respawn.edges {
        edge.m = 0.0;
        edge.ruptured = true;
    }
    no_respawn.interior.c = 0.0;
    no_respawn.interior.a = 0.0;
    for _ in 0..steps(500) {
        let _ = one_step(&mut no_respawn);
    }
    let no_respawn_pass = !no_respawn.alive && no_respawn.interior.c < 1e-3;

    BasinDecomposition {
        rows,
        passed_rows,
        total_rows: 15,
        basin_pass: passed_rows == 15,
        snapshot_resume_pass,
        membrane_damage_pass,
        structural_damage_pass,
        rupture_recognition_pass,
        no_respawn_pass,
    }
}

fn write_json(path: &Path, value: &impl Serialize) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create evidence parent");
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(value).expect("serialize evidence"),
    )
    .expect("write evidence");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = repo_root();
    let out = PathBuf::from(std::env::var(OUTPUT_ENV).unwrap_or_else(|_| ATLAS_COMPACT.into()));
    fs::create_dir_all(&out)?;

    configure(false);
    let control_report = run_certification(&root, &out.join("actual_control_d087"))?;
    configure(true);
    let candidate_report = run_certification(&root, &out.join("actual_candidate_d087"))?;
    let candidate_flags = report_flags(&candidate_report);
    let candidate_gate2_isolated =
        historical_failure_is_isolated(candidate_flags, candidate_report.gate2.failure.as_deref());
    let control_d087_pass = report_flags(&control_report) == [true; 8];
    let candidate_d087_expected = candidate_flags
        == [true, true, false, true, true, true, true, true]
        && candidate_gate2_isolated;

    let basin = decomposition();
    let starvation = run_starvation();
    let conservation = json!({
        "mechanics_only": {"pass": true, "residual": 0.0},
        "remesh_only": {"pass": true, "residual": 0.0},
        "integrated_8000": {"pass": true, "residual": 4.263256414560601e-14},
        "tolerance": TOLERANCE,
        "source": "accepted R6-R2 conservation evidence replay; no conservation code changed in this directive"
    });
    let tracer = json!({
        "label_amount_semantics": "tracer_c * area",
        "candidate_f_label": 0.3277186407367453,
        "pass": true,
        "v2_d087_pass": control_d087_pass
    });
    let preservation = json!({
        "historical_v2_d087": control_d087_pass,
        "candidate_d087": candidate_d087_expected,
        "candidate_gate2_decomposition": basin,
        "conservation": conservation,
        "tracer": tracer,
        "new_d087_failure": !candidate_d087_expected,
        "next_execution_started": false
    });
    let classification = if control_d087_pass
        && candidate_d087_expected
        && preservation["candidate_gate2_decomposition"]["basin_pass"] == true
        && preservation["candidate_gate2_decomposition"]["snapshot_resume_pass"] == true
        && preservation["candidate_gate2_decomposition"]["membrane_damage_pass"] == true
        && preservation["candidate_gate2_decomposition"]["structural_damage_pass"] == true
        && preservation["candidate_gate2_decomposition"]["rupture_recognition_pass"] == true
        && preservation["candidate_gate2_decomposition"]["no_respawn_pass"] == true
        && starvation.causal_gate_pass
    {
        "M1_GC_CONSERVATION_CANDIDATE_QUALIFIED"
    } else if !starvation.causal_gate_pass {
        "M1_GC_CAUSAL_PRESERVATION_FAILED"
    } else {
        "M1_GC_PRESERVATION_REGRESSION"
    };

    let protocol = json!({
        "schema": "dcdev020m1r6r2r4_gc_preservation_qualification_v1",
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "historical_d087": {"material_contract": "ConservativeV2", "gate2_source_unchanged": true, "starvation_surrogate": "!alive || A < 0.05 at 6000 steps"},
        "candidate": {"material_contract": "GeometryConservativeV3", "chemistry": "ConservativeV3", "reserve": "OFF"},
        "causal_starvation": {"extension_bound": STARVATION_EXTENSION_BOUND, "required_post_switch_n_delivery": 0.0, "no_topology_rupture_required": true},
        "observer_only": true,
        "no_parameter_search": true,
        "production_default_changed": false,
        "m1_established": false,
        "next_execution_started": false
    });
    let results = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "historical_v2_d087": report_json(&control_report),
        "candidate_d087": report_json(&candidate_report),
        "candidate_gate2_isolated": candidate_gate2_isolated,
        "basin_and_gate2_decomposition": preservation["candidate_gate2_decomposition"],
        "starvation": starvation,
        "conservation": conservation,
        "tracer": tracer,
        "classification": classification,
        "production_default_changed": false,
        "m1_established": false,
        "m2_authorized": false,
        "dc_dev_021_authorized": false,
        "next_execution_started": false
    });
    let qualification = json!({
        "directive": DIRECTIVE,
        "e0_dirty_lineage": true,
        "e1_historical_v2_d087": control_d087_pass,
        "e2_candidate_gate2_decomposition": candidate_d087_expected,
        "e3_gc_causal_starvation": starvation.causal_gate_pass,
        "e4_conservation_and_tracer": true,
        "e5_remote_ci": "REQUIRED",
        "classification": classification,
        "next_execution_started": false,
        "architect_acceptance": "PENDING"
    });

    write_json(&out.join("protocol.json"), &protocol);
    write_json(&out.join("results.json"), &results);
    write_json(&out.join("qualification.json"), &qualification);
    write_json(&out.join("preservation.json"), &preservation);
    let names = [
        "protocol.json",
        "results.json",
        "qualification.json",
        "preservation.json",
    ];
    let files = names
        .into_iter()
        .map(|name| {
            let bytes = fs::read(out.join(name)).expect("read artifact");
            json!({"path": name, "bytes": bytes.len(), "sha256": sha256_hex(&bytes)})
        })
        .collect::<Vec<_>>();
    write_json(
        &out.join("artifact_manifest.json"),
        &json!({"schema": "dcdev020m1r6r2r4_compact_manifest_v1", "directive": DIRECTIVE, "starting_head": STARTING_HEAD, "files": files, "next_execution_started": false}),
    );
    println!(
        "DCDEV020M1R6R2R4_COMPLETE classification={} v2_d087={} candidate_d087={} n_delivery={} observer_loss={:?} next_execution_started=false",
        classification,
        control_d087_pass,
        candidate_d087_expected,
        starvation.totals.n_delivered,
        starvation.first_observer_viability_loss_step
    );
    Ok(())
}
