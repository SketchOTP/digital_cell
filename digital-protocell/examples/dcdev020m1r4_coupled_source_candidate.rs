//! DC-DEV-020-M1-R4 versioned finite-resource coupled-source candidate.
//!
//! This assay keeps ConservativeV3 chemistry and the finite V1 boundary fixed.
//! The candidate transforms only same-step paired N/F returned by the V1
//! boundary into A+W. It does not select a production configuration.

#[path = "dcdev020m1r1_capacity_decomp.rs"]
mod m1r1;

use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_contracts::snapshot;
use chemistry_core::mesh_mechanics::MechParams;
use chemistry_core::mesh_reactions::{reactions_step, ReactionLedger, ReactionParams};
use chemistry_core::mesh_transport::TransportParams;
use regulatory_core::{
    stable_json_hash, CoupledFiniteSpatialResourceRegionV1, FiniteSpatialResourceRegionV1,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-020-M1-R4-COUPLED-SOURCE-CANDIDATE-001";
const STARTING_HEAD: &str = "17226fb7484eb50079c1c30ce9fd8039b3f23c60";
const SCHEMA: &str = "FINITE_SPATIAL_RESOURCE_COUPLED_ACTIVATION_V1";
const HORIZON_STEPS: usize = 480;
const DT: f64 = 0.02;
const RESOURCE_CENTER: [f64; 2] = [4.8, 0.0];
const RESOURCE_RADIUS: f64 = 1.5;
const RESOURCE_N: f64 = 14.588954880632265;
const RESOURCE_F: f64 = 14.588954880632265;
const TOL: f64 = 1e-8;

#[derive(Debug, Clone, Serialize)]
struct State {
    step: usize,
    area: f64,
    n: f64,
    f: f64,
    a: f64,
    c: f64,
    structural_m: f64,
    free_l: f64,
    bound_b: f64,
    waste: f64,
    organized_material: f64,
    strict_material: f64,
    observer_viable: bool,
    closed_intact: bool,
    ruptured_edges: usize,
}

fn state(mesh: &MaterialMesh, step: usize) -> State {
    let s = snapshot(mesh);
    State {
        step,
        area: mesh.area(),
        n: s.n,
        f: s.f,
        a: s.a,
        c: s.c,
        structural_m: s.structural_m,
        free_l: s.free_l,
        bound_b: s.bound_b,
        waste: s.waste,
        organized_material: s.organized_material(),
        strict_material: s.strict_material_equivalent(),
        observer_viable: mesh.observer_viable(),
        closed_intact: mesh.closed_intact(),
        ruptured_edges: mesh.edges.iter().filter(|edge| edge.ruptured).count(),
    }
}

#[derive(Debug, Clone, Default, Serialize)]
struct ReactionTotals {
    a_produced_ordinary: f64,
    m_production: f64,
    m_turnover: f64,
    c_production: f64,
    c_turnover: f64,
    membrane_production: f64,
    membrane_turnover: f64,
}

impl ReactionTotals {
    fn absorb(&mut self, ledger: &ReactionLedger) {
        self.a_produced_ordinary += ledger.a_produced;
        self.m_production += ledger.m_produced;
        self.m_turnover += ledger.m_to_w;
        self.c_production += ledger.c_produced;
        self.c_turnover += ledger.c_turned;
        self.membrane_production += ledger.l_produced;
        self.membrane_turnover += ledger.bind_extent + ledger.unbind_extent;
    }
}

#[derive(Debug, Clone, Default, Serialize)]
struct SourceTotals {
    n_world_loss: f64,
    f_world_loss: f64,
    n_delivered: f64,
    f_delivered: f64,
    paired_activated: f64,
    n_deposited_unpaired: f64,
    f_deposited_unpaired: f64,
    a_produced_coupled: f64,
    w_produced_coupled: f64,
    max_conservation_residual: f64,
    max_v1_conservation_error: f64,
}

#[derive(Debug, Clone, Serialize)]
struct ArmResult {
    arm: String,
    source_schema: String,
    initial: State,
    final_state: State,
    organized_material_delta: f64,
    world_to_organism_closure_residual: f64,
    source: SourceTotals,
    ordinary_reactions: ReactionTotals,
}

#[derive(Debug, Clone, Serialize)]
struct ControlResult {
    name: String,
    schema: String,
    n_delivered: f64,
    f_delivered: f64,
    paired_activated: f64,
    coupled_a_delta: f64,
    coupled_w_delta: f64,
    preexisting_nf_preserved: bool,
    pass: bool,
}

#[derive(Debug, Clone, Copy)]
enum Arm {
    V1Baseline,
    SourceCapacityReference,
    CoupledCandidate,
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= TOL * (1.0 + a.abs().max(b.abs()))
}

fn reaction_params() -> ReactionParams {
    ReactionParams::conservative_v3()
}

fn paired_source_upper_bound(mesh: &mut MaterialMesh) -> f64 {
    let area = mesh.area().max(1e-6);
    let paired = mesh.interior.n.min(mesh.interior.f).max(0.0) * area;
    mesh.interior.n -= paired / area;
    mesh.interior.f -= paired / area;
    mesh.interior.a += paired / area;
    mesh.interior.w += paired / area;
    paired
}

fn run_arm(initial: &MaterialMesh, mechanics: &MechParams, arm: Arm) -> ArmResult {
    let mut mesh = initial.clone();
    let initial_state = state(&mesh, 0);
    let transport = TransportParams::default();
    let params = reaction_params();
    let mut v1_region = FiniteSpatialResourceRegionV1::new(
        RESOURCE_CENTER,
        RESOURCE_RADIUS,
        RESOURCE_N,
        RESOURCE_F,
    );
    let mut coupled_region = CoupledFiniteSpatialResourceRegionV1::new(
        RESOURCE_CENTER,
        RESOURCE_RADIUS,
        RESOURCE_N,
        RESOURCE_F,
    );
    let mut source = SourceTotals::default();
    let mut ordinary_reactions = ReactionTotals::default();

    for _step in 1..=HORIZON_STEPS {
        match arm {
            Arm::V1Baseline => {
                let ledger = v1_region.uptake(&mut mesh, &transport, mechanics.dt);
                source.n_world_loss += ledger.n_world_loss;
                source.f_world_loss += ledger.f_world_loss;
                source.n_delivered += ledger.n_delivered;
                source.f_delivered += ledger.f_delivered;
                source.max_v1_conservation_error = source
                    .max_v1_conservation_error
                    .max(ledger.conservation_error);
            }
            Arm::SourceCapacityReference => {
                let ledger = v1_region.uptake(&mut mesh, &transport, mechanics.dt);
                source.n_world_loss += ledger.n_world_loss;
                source.f_world_loss += ledger.f_world_loss;
                source.n_delivered += ledger.n_delivered;
                source.f_delivered += ledger.f_delivered;
                source.paired_activated += paired_source_upper_bound(&mut mesh);
                source.max_v1_conservation_error = source
                    .max_v1_conservation_error
                    .max(ledger.conservation_error);
            }
            Arm::CoupledCandidate => {
                let ledger = coupled_region.uptake(&mut mesh, &transport, mechanics.dt);
                assert!(ledger.paired_activated <= ledger.n_delivered + TOL);
                assert!(ledger.paired_activated <= ledger.f_delivered + TOL);
                assert!(close(
                    ledger.n_deposited_unpaired + ledger.paired_activated,
                    ledger.n_delivered
                ));
                assert!(close(
                    ledger.f_deposited_unpaired + ledger.paired_activated,
                    ledger.f_delivered
                ));
                assert!(close(ledger.a_produced_coupled, ledger.paired_activated));
                assert!(close(ledger.w_produced_coupled, ledger.paired_activated));
                source.n_world_loss += ledger.n_world_loss;
                source.f_world_loss += ledger.f_world_loss;
                source.n_delivered += ledger.n_delivered;
                source.f_delivered += ledger.f_delivered;
                source.paired_activated += ledger.paired_activated;
                source.n_deposited_unpaired += ledger.n_deposited_unpaired;
                source.f_deposited_unpaired += ledger.f_deposited_unpaired;
                source.a_produced_coupled += ledger.a_produced_coupled;
                source.w_produced_coupled += ledger.w_produced_coupled;
                source.max_conservation_residual = source
                    .max_conservation_residual
                    .max(ledger.conservation_residual);
                source.max_v1_conservation_error = source
                    .max_v1_conservation_error
                    .max(ledger.v1_ledger.conservation_error);
            }
        }
        let reaction = reactions_step(&mut mesh, &params, mechanics.dt, true, true);
        ordinary_reactions.absorb(&reaction);
    }

    let final_state = state(&mesh, HORIZON_STEPS);
    let organized_material_delta =
        final_state.organized_material - initial_state.organized_material;
    let closure = (final_state.strict_material
        - initial_state.strict_material
        - source.n_world_loss
        - source.f_world_loss)
        .abs();
    ArmResult {
        arm: match arm {
            Arm::V1Baseline => "V1_BASELINE_V3".into(),
            Arm::SourceCapacityReference => "ACCEPTED_SOURCE_CAPACITY_REFERENCE".into(),
            Arm::CoupledCandidate => "COUPLED_SOURCE_CANDIDATE".into(),
        },
        source_schema: match arm {
            Arm::CoupledCandidate => SCHEMA.into(),
            _ => "dcdev008_finite_static_nf_region_v1".into(),
        },
        initial: initial_state,
        final_state,
        organized_material_delta,
        world_to_organism_closure_residual: closure,
        source,
        ordinary_reactions,
    }
}

fn v1_replay_pass(initial: &MaterialMesh, mechanics: &MechParams) -> bool {
    let transport = TransportParams::default();
    let mut first_mesh = initial.clone();
    let mut second_mesh = initial.clone();
    let mut first = FiniteSpatialResourceRegionV1::new(
        RESOURCE_CENTER,
        RESOURCE_RADIUS,
        RESOURCE_N,
        RESOURCE_F,
    );
    let mut second = first.clone();
    for _ in 0..HORIZON_STEPS {
        let a = first.uptake(&mut first_mesh, &transport, mechanics.dt);
        let b = second.uptake(&mut second_mesh, &transport, mechanics.dt);
        if a != b {
            return false;
        }
    }
    stable_json_hash(&first_mesh).ok() == stable_json_hash(&second_mesh).ok() && first == second
}

fn run_controls(initial: &MaterialMesh, mechanics: &MechParams) -> Vec<ControlResult> {
    let transport = TransportParams::default();
    let cases = [
        ("NO_CONTACT", [30.0, 30.0], RESOURCE_N, RESOURCE_F, false),
        ("N_ONLY", RESOURCE_CENTER, RESOURCE_N, 0.0, false),
        ("F_ONLY", RESOURCE_CENTER, 0.0, RESOURCE_F, false),
        ("DEPLETED", RESOURCE_CENTER, 0.0, 0.0, false),
    ];
    let mut results = cases
        .into_iter()
        .map(|(name, center, n, f, _)| {
            let mut mesh = initial.clone();
            let before = mesh.interior;
            let mut region =
                CoupledFiniteSpatialResourceRegionV1::new(center, RESOURCE_RADIUS, n, f);
            let ledger = region.uptake(&mut mesh, &transport, mechanics.dt);
            let a_delta = mesh.interior.a - before.a;
            let w_delta = mesh.interior.w - before.w;
            let pass = if name == "NO_CONTACT" {
                ledger.n_delivered == 0.0
                    && ledger.f_delivered == 0.0
                    && ledger.paired_activated == 0.0
            } else if name == "N_ONLY" || name == "F_ONLY" || name == "DEPLETED" {
                ledger.paired_activated == 0.0 && a_delta == 0.0 && w_delta == 0.0
            } else {
                false
            };
            ControlResult {
                name: name.into(),
                schema: SCHEMA.into(),
                n_delivered: ledger.n_delivered,
                f_delivered: ledger.f_delivered,
                paired_activated: ledger.paired_activated,
                coupled_a_delta: a_delta,
                coupled_w_delta: w_delta,
                preexisting_nf_preserved: true,
                pass,
            }
        })
        .collect::<Vec<_>>();

    let mut ruptured_mesh = initial.clone();
    for edge in &mut ruptured_mesh.edges {
        edge.ruptured = true;
    }
    let before = ruptured_mesh.interior;
    let mut ruptured = CoupledFiniteSpatialResourceRegionV1::new(
        RESOURCE_CENTER,
        RESOURCE_RADIUS,
        RESOURCE_N,
        RESOURCE_F,
    );
    let ledger = ruptured.uptake(&mut ruptured_mesh, &transport, mechanics.dt);
    results.push(ControlResult {
        name: "ALL_EXPOSED_EDGES_RUPTURED".into(),
        schema: SCHEMA.into(),
        n_delivered: ledger.n_delivered,
        f_delivered: ledger.f_delivered,
        paired_activated: ledger.paired_activated,
        coupled_a_delta: ruptured_mesh.interior.a - before.a,
        coupled_w_delta: ruptured_mesh.interior.w - before.w,
        preexisting_nf_preserved: true,
        pass: ledger.n_delivered == 0.0
            && ledger.f_delivered == 0.0
            && ledger.paired_activated == 0.0,
    });

    let mut preexisting = initial.clone();
    preexisting.interior.n = 0.001;
    preexisting.interior.f = 0.001;
    let mut baseline = preexisting.clone();
    let mut v1 = FiniteSpatialResourceRegionV1::new(
        RESOURCE_CENTER,
        RESOURCE_RADIUS,
        RESOURCE_N,
        RESOURCE_F,
    );
    let v1_ledger = v1.uptake(&mut baseline, &transport, mechanics.dt);
    let mut candidate = CoupledFiniteSpatialResourceRegionV1::new(
        RESOURCE_CENTER,
        RESOURCE_RADIUS,
        RESOURCE_N,
        RESOURCE_F,
    );
    let candidate_ledger = candidate.uptake(&mut preexisting, &transport, mechanics.dt);
    let area = preexisting.area();
    let nf_preserved = close(
        preexisting.interior.n,
        baseline.interior.n - candidate_ledger.paired_activated / area,
    ) && close(
        preexisting.interior.f,
        baseline.interior.f - candidate_ledger.paired_activated / area,
    ) && close(v1_ledger.n_delivered, candidate_ledger.n_delivered)
        && close(v1_ledger.f_delivered, candidate_ledger.f_delivered);
    results.push(ControlResult {
        name: "PRE_EXISTING_INTERNAL_NF".into(),
        schema: SCHEMA.into(),
        n_delivered: candidate_ledger.n_delivered,
        f_delivered: candidate_ledger.f_delivered,
        paired_activated: candidate_ledger.paired_activated,
        coupled_a_delta: candidate_ledger.a_produced_coupled / area,
        coupled_w_delta: candidate_ledger.w_produced_coupled / area,
        preexisting_nf_preserved: nf_preserved,
        pass: nf_preserved,
    });
    results
}

fn read_report(path: &Path) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_else(|| json!({"status": "deferred_to_exact_workflow"}))
}

fn d087_pass(report: &Value, contract: &str) -> bool {
    report["mesh_contract"] == contract
        && report["reserve_enabled"] == false
        && (0..8).all(|i| report[format!("gate{i}")]["pass"] == true)
        && report["primary_conclusion"] == "D087_PHASE1_AUTOPOIETIC_PROTOCELL_CERTIFIED"
}

fn write_json(path: &Path, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::env::var_os("DCDEV020M1R4_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020m1r4"));
    let (entry, mechanics) = m1r1::m1r1_entry_state();
    assert!(close(mechanics.dt, DT));
    let entry_state = state(&entry, 0);
    assert!(close(entry_state.a, 19.69467805250676));
    assert!(close(entry_state.c, 55.87794642665143));
    assert!(close(entry_state.organized_material, 131.80639622655494));

    let v1_pass = v1_replay_pass(&entry, &mechanics);
    let controls = run_controls(&entry, &mechanics);
    let controls_pass = controls.iter().all(|control| control.pass);
    let baseline = run_arm(&entry, &mechanics, Arm::V1Baseline);
    let reference = run_arm(&entry, &mechanics, Arm::SourceCapacityReference);
    let candidate = run_arm(&entry, &mechanics, Arm::CoupledCandidate);
    assert!(v1_pass && controls_pass);
    assert!(candidate.source.max_conservation_residual <= TOL);
    assert!(candidate.world_to_organism_closure_residual <= TOL);

    let v3_d087 = read_report(&out.join("v3_d087/certification/report.json"));
    let v2_d087 = read_report(&out.join("v2_d087/certification/report.json"));
    let v3_d087_pass = d087_pass(&v3_d087, "ConservativeV3");
    let v2_d087_pass = d087_pass(&v2_d087, "ConservativeV2");
    let candidate_pass = candidate.organized_material_delta >= -TOL;
    let classification =
        if v1_pass && controls_pass && candidate_pass && v3_d087_pass && v2_d087_pass {
            "M1_COUPLED_SOURCE_CANDIDATE_QUALIFIED"
        } else if !candidate.source.max_conservation_residual.is_finite()
            || candidate.world_to_organism_closure_residual > TOL
        {
            "M1_COUPLED_SOURCE_CONSERVATION_FAILURE"
        } else if !v1_pass || !controls_pass || !v3_d087_pass || !v2_d087_pass {
            "M1_COUPLED_SOURCE_CANDIDATE_INVALID"
        } else {
            "M1_COUPLED_SOURCE_CAPACITY_SHORTFALL"
        };

    let protocol = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "coupled_source_schema": SCHEMA,
        "chemistry": "ConservativeV3",
        "reserve_enabled": false,
        "selected_production": {"mesh_contract": "ConservativeV2", "reserve_enabled": false},
        "horizon_steps": HORIZON_STEPS,
        "dt": DT,
        "resource": {"center": RESOURCE_CENTER, "radius": RESOURCE_RADIUS, "n": RESOURCE_N, "f": RESOURCE_F},
        "law": "paired=min(newly_delivered_N,newly_delivered_F); paired -> A+W; unmatched remains N/F",
        "arms": ["V1_BASELINE_V3", "ACCEPTED_SOURCE_CAPACITY_REFERENCE", "COUPLED_SOURCE_CANDIDATE"],
        "forbidden_changes": ["V1 transport", "V3 chemistry", "k_act", "k_a_decay", "finite inventory", "resource geometry", "permeability", "rupture", "death", "D-091", "D-087", "mechanics", "remesh", "controller", "recycling", "salvage", "M2", "DC-DEV-021"]
    });
    let results = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "v1_semantics_unchanged": v1_pass,
        "physical_controls": controls,
        "v1_baseline": baseline,
        "source_capacity_reference": reference,
        "coupled_candidate": candidate,
        "d087": {"v3": v3_d087, "v2": v2_d087},
        "classification": classification,
        "v1_chemistry_changed": false,
        "v3_chemistry_changed": false,
        "selected_production_changed": false,
        "parameter_search": false,
        "controller_added": false,
        "recycling": false,
        "m2_authorized": false,
        "dc_dev_021_authorized": false,
        "next_execution_started": false
    });
    let qualification = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "e0_authority": true,
        "e1_physical_law": controls_pass && v1_pass,
        "e2_causal_three_arm_experiment": true,
        "e3_nonnegative_organized_material_delta": candidate_pass,
        "e4_candidate_closure_and_d087": candidate.source.max_conservation_residual <= TOL && candidate.world_to_organism_closure_residual <= TOL && v3_d087_pass && v2_d087_pass,
        "e5_exact_head_remote_ci_required": true,
        "classification": classification,
        "selected_production_changed": false,
        "next_execution_started": false
    });
    let manifest = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "artifact_files": ["protocol.json", "results.json", "qualification.json", "artifact_manifest.json"],
        "authoritative_result": "results.json",
        "dense_ledgers_committed": false,
        "evidence_hash": stable_json_hash(&results)?,
        "next_execution_started": false
    });
    write_json(&out.join("protocol.json"), &protocol)?;
    write_json(&out.join("results.json"), &results)?;
    write_json(&out.join("qualification.json"), &qualification)?;
    write_json(&out.join("artifact_manifest.json"), &manifest)?;
    println!("DCDEV020M1R4_COUPLED_SOURCE_CANDIDATE_COMPLETE");
    println!("classification={classification}");
    println!("v1_organized_delta={}", baseline.organized_material_delta);
    println!(
        "source_capacity_reference_organized_delta={}",
        reference.organized_material_delta
    );
    println!(
        "coupled_candidate_organized_delta={}",
        candidate.organized_material_delta
    );
    println!("paired_activated={}", candidate.source.paired_activated);
    println!("v3_d087_pass={v3_d087_pass} v2_d087_pass={v2_d087_pass}");
    Ok(())
}
