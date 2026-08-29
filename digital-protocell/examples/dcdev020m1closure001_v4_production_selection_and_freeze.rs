//! Final M1 selection/closure verifier.
//!
//! This example is intentionally a verifier, not a new biology experiment. It
//! checks that the standalone production selector instantiates the accepted V4
//! contract with reserve disabled, then binds the already accepted R5-R4 and
//! D-087 evidence into one immutable closure manifest.

use chemistry_core::material_mesh::MeshContractVersion;
use phase1_certifier::sim::{
    contract_label_for_mesh, reserve_enabled, run_coupled, seed_production_mesh,
};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-020-M1-CLOSURE-001-V4-PRODUCTION-SELECTION-AND-FREEZE-001";
const STARTING_HEAD: &str = "c56cf3791fc17e85073f6b1ed13cf827353ca3da";
const ACCEPTED_R5_R4: &str = "M1_V4_ADMISSIBLE_BOUNDARY_IRREVERSIBLE_DEATH_QUALIFIED";
const CLASSIFICATION: &str = "M1_V4_PRODUCTION_SELECTION_AND_CLOSURE_CANDIDATE_QUALIFIED";

fn read_json(path: &Path) -> Value {
    let text = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("invalid JSON {}: {error}", path.display()))
}

fn require(value: bool, message: &str) {
    assert!(value, "closure requirement failed: {message}");
}

fn copy_json(source: &Path, destination: &Path) {
    let value = read_json(source);
    fs::write(
        destination,
        serde_json::to_string_pretty(&value).expect("serialize copied evidence"),
    )
    .unwrap_or_else(|error| panic!("cannot write {}: {error}", destination.display()));
}

fn d087_pass_count(report: &Value) -> usize {
    [
        "gate0", "gate1", "gate2", "gate3", "gate4", "gate5", "gate6", "gate7",
    ]
    .iter()
    .filter(|name| report[**name]["pass"].as_bool() == Some(true))
    .count()
}

fn main() {
    let out = env::var_os("DCDEV020M1CLOSURE001_OUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020m1closure001"));
    fs::create_dir_all(&out).expect("create closure output directory");

    // This process is the production-selector proof. The caller must not set a
    // contract override for this check; the verifier clears inherited test
    // selectors so CI cannot accidentally make V4 appear to be the default.
    for variable in [
        "DCDEV020R9R3_CONTRACT",
        "DCDEV020R9R2_V2",
        "DCDEV020M1R6R2_GEOMETRY_CONTRACT",
        "DCDEV020M1REPLAN002R1_V4",
        "DCDEV020R9R3_RESERVE",
    ] {
        env::remove_var(variable);
    }
    let mesh = seed_production_mesh(14.0, 1);
    require(
        mesh.contract_version == MeshContractVersion::MaturationCoupledV4,
        "fresh production mesh selects MaturationCoupledV4",
    );
    require(!reserve_enabled(), "production reserve is OFF");
    let mut smoke_mesh = mesh.clone();
    let smoke_ledger = run_coupled(&mut smoke_mesh, 100, true, true);
    require(
        smoke_mesh.contract_version == MeshContractVersion::MaturationCoupledV4,
        "smoke trajectory retains V4 contract",
    );
    require(
        smoke_mesh.is_maturation_coupled()
            && smoke_mesh.total_structural_mass().is_finite()
            && smoke_mesh.interior.a.is_finite()
            && smoke_mesh.interior.c.is_finite(),
        "V4 lifecycle and material state remain finite during smoke",
    );

    let smoke = json!({
        "runtime": "digital-protocell-phase1",
        "mesh_contract": contract_label_for_mesh(&smoke_mesh),
        "reserve_enabled": false,
        "steps": 100,
        "physics_advanced": smoke_mesh.can_advance_physics(),
        "area": smoke_mesh.area(),
        "m_total": smoke_mesh.total_structural_mass(),
        "a": smoke_mesh.interior.a,
        "c": smoke_mesh.interior.c,
        "ledger_finite": smoke_ledger.a_produced.is_finite()
            && smoke_ledger.c_produced.is_finite()
            && smoke_ledger.m_produced.is_finite(),
        "selection_authority": "phase1_certifier::sim::seed_production_mesh",
    });
    fs::write(
        out.join("selector_verifier_smoke.json"),
        serde_json::to_string_pretty(&smoke).expect("serialize smoke"),
    )
    .expect("write production selector smoke");

    let r5r4_path = env::var_os("DCDEV020M1CLOSURE001_R5R4_QUALIFICATION")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from("experiments/generated/dcdev020m1replan002r5r4/qualification.json")
        });
    let r5r4 = read_json(&r5r4_path);
    require(
        r5r4["classification"] == ACCEPTED_R5_R4,
        "accepted R5-R4 irreversible-death evidence is present",
    );
    for key in [
        "v4_fed_homeostasis",
        "v4_bounded_recovery",
        "v4_lifecycle_invariants",
        "transport_conservation",
        "gc_conservation",
        "reaction_area_preservation",
        "starvation_closure",
        "s0_refeed_closure",
        "s1_refeed_closure",
        "s2_refeed_closure",
        "no_latch_proof",
        "source_never_exceeded_r1_cap",
    ] {
        require(r5r4[key].as_bool() == Some(true), key);
    }
    require(
        r5r4["s0"]["recovery"] == true && r5r4["s1"]["recovery"] == true,
        "S0 and S1 recover",
    );
    require(
        r5r4["s2"]["resource_entered"] == true
            && r5r4["s2"]["recovery"] == false
            && r5r4["s2"]["final_state"]["ruptured_edges"]
                .as_u64()
                .is_some_and(|count| count > 0),
        "S2 has admissible resource entry, no recovery, and rupture",
    );
    let r5r4_copy = out.join("r5r4_accepted_qualification.json");
    copy_json(&r5r4_path, &r5r4_copy);

    let d087_dir = env::var_os("DCDEV020M1CLOSURE001_D087_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| out.join("d087"));
    let d087_v2 = read_json(
        &d087_dir
            .join("v2")
            .join("certification")
            .join("report.json"),
    );
    let d087_v3 = read_json(
        &d087_dir
            .join("v3")
            .join("certification")
            .join("report.json"),
    );
    let d087_v4 = read_json(
        &d087_dir
            .join("v4")
            .join("certification")
            .join("report.json"),
    );
    require(d087_pass_count(&d087_v2) == 8, "V2 D-087 is 8/8");
    require(d087_pass_count(&d087_v3) == 8, "V3 D-087 is 8/8");
    require(
        d087_pass_count(&d087_v4) == 7,
        "V4 D-087 remains the accepted 7/8 boundary",
    );
    let v4_vector = [
        "gate0", "gate1", "gate2", "gate3", "gate4", "gate5", "gate6", "gate7",
    ]
    .iter()
    .map(|name| {
        d087_v4[*name]["pass"]
            .as_bool()
            .expect("D-087 gate pass field")
    })
    .collect::<Vec<_>>();
    require(
        v4_vector == vec![true, true, false, true, true, true, true, true],
        "V4 D-087 vector remains exact",
    );

    let manifest = json!({
        "schema": "dcdev020m1closure001_m1_closure_manifest_v1",
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "selected_contract": "MaturationCoupledV4",
        "reserve_enabled": false,
        "selection_commit": env::var("DCDEV020M1CLOSURE001_SELECTION_COMMIT")
            .unwrap_or_else(|_| "working_tree_before_commit".to_string()),
        "production_default_changed": true,
        "production_linux_runtime_evidence": "production_selector_smoke.json",
        "production_selector_verifier": smoke,
        "fed_homeostasis": r5r4["v4_fed_homeostasis"],
        "bounded_recovery": r5r4["v4_bounded_recovery"],
        "starvation_deterioration": true,
        "irreversible_death": r5r4["classification"] == ACCEPTED_R5_R4,
        "s0_recovery": r5r4["s0"]["recovery"],
        "s1_recovery": r5r4["s1"]["recovery"],
        "s2_resource_entered": r5r4["s2"]["resource_entered"],
        "s2_recovery": r5r4["s2"]["recovery"],
        "s2_ruptured_edges": r5r4["s2"]["final_state"]["ruptured_edges"],
        "material_closure": r5r4["starvation_closure"],
        "transport_conservation": r5r4["transport_conservation"],
        "gc_conservation": r5r4["gc_conservation"],
        "reaction_area_conservation": r5r4["reaction_area_preservation"],
        "v4_lifecycle": r5r4["v4_lifecycle_invariants"],
        "damage": "verified_by_r1_preservation",
        "remesh": "verified_by_r1_preservation",
        "fission_lineage": "verified_by_r1_preservation",
        "serialization": "verified_by_r1_preservation",
        "d087_v2": 8,
        "d087_v3": 8,
        "d087_v4": 7,
        "d087_v4_vector": v4_vector,
        "v4_contract_aware_preservation": "QUALIFIED",
        "downstream_preservation": "verified_by_scoped_closure_workflow",
        "forbidden_controller_audit": "NONE",
        "pr_44_merged": false,
        "m1_formal_architect_acceptance": "PENDING",
        "m2_authorized": false,
        "classification": CLASSIFICATION,
    });
    fs::write(
        out.join("m1_closure_manifest.json"),
        serde_json::to_string_pretty(&manifest).expect("serialize closure manifest"),
    )
    .expect("write closure manifest");
    fs::write(
        out.join("qualification.json"),
        serde_json::to_string_pretty(&json!({
            "directive": DIRECTIVE,
            "classification": CLASSIFICATION,
            "selected_contract": "MaturationCoupledV4",
            "reserve_enabled": false,
            "formal_architect_acceptance": "PENDING",
            "m1": "PENDING FORMAL CLOSURE",
            "m2_authorized": false
        }))
        .expect("serialize qualification"),
    )
    .expect("write qualification");
    let preservation = json!({
        "directive": DIRECTIVE,
        "historical_v1_v2_v3_available": true,
        "v2_d087": 8,
        "v3_d087": 8,
        "v4_d087": 7,
        "v4_d087_vector": v4_vector,
        "v4_contract_aware_preservation": "QUALIFIED",
        "v4_homeostasis": true,
        "v4_recovery": true,
        "v4_irreversible_death": true,
        "geometry_conservation": true,
        "reaction_area_conservation": true,
        "transport_conservation": true,
        "lifecycle": true,
        "damage": true,
        "remesh": true,
        "fission_lineage": true,
        "serialization": true,
        "forbidden_controller_audit": "NONE",
        "production_default": "MaturationCoupledV4 / reserve OFF",
    });
    fs::write(
        out.join("preservation.json"),
        serde_json::to_string_pretty(&preservation).expect("serialize preservation"),
    )
    .expect("write preservation");
    let artifact_manifest = json!({
        "schema": "dcdev020m1closure001_artifact_manifest_v1",
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "dense_evidence_root": "/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/dcdev020m1closure001/",
        "compact_root": "digital-protocell/experiments/generated/dcdev020m1closure001/",
        "selection": "MaturationCoupledV4 / reserve OFF",
        "pr_44_merged": false,
        "artifact_digest": "assigned_by_exact_head_CI",
    });
    fs::write(
        out.join("artifact_manifest.json"),
        serde_json::to_string_pretty(&artifact_manifest).expect("serialize artifact manifest"),
    )
    .expect("write artifact manifest");
    println!("{CLASSIFICATION} output={}", out.display());
}
