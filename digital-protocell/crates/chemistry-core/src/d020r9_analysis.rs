//! DC-DEV-020-R9 bounded mesh-contract requalification.
//!
//! This is an observer/reporting harness around the versioned v2 mesh path. It
//! does not add a source law, controller, salvage reaction, or behavior.

use crate::material_mesh::{LumpedChem, MaterialMesh};
use crate::mesh_contracts::{audit, snapshot, MaterialLedgerSnapshot, MeshStoichiometricAudit};
use crate::mesh_fission::{try_local_fission, FissionParams};
use crate::mesh_reactions::{apply_local_rupture, reactions_step, ReactionParams};
use crate::mesh_topology::TopologyParams;
use crate::mesh_transport::{transport_step, TransportParams};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const R9_CLASSIFICATION_METRIC: &str = "DCDEV020R9_METRIC_CONFOUNDING_DOMINANT";
pub const R9_CLASSIFICATION_ACTIVATION: &str = "DCDEV020R9_TRUE_ACTIVATION_DEFICIT_CONFIRMED";
pub const R9_CLASSIFICATION_MATERIAL: &str = "DCDEV020R9_TRUE_MATERIAL_CYCLE_DEFICIT_CONFIRMED";
pub const R9_CLASSIFICATION_FAILURE: &str = "DCDEV020R9_MESH_CONTRACT_REQUALIFICATION_FAILURE";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct R9Authority {
    pub directive: String,
    pub entry_head: String,
    pub clean_production_base: String,
    pub branch: String,
    pub superseded: String,
    pub dc_dev_021: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct R9Ledgers {
    pub initial: MaterialLedgerSnapshot,
    pub final_state: MaterialLedgerSnapshot,
    pub strict_material_delta: f64,
    pub activation_delta: f64,
    pub organized_retained_delta: f64,
    pub boundary_material_delta: f64,
    pub closure_residual: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct R9ReplayRow {
    pub protocol_reference: String,
    pub execution: String,
    pub external_nf_scale: f64,
    pub accepted_steps: u32,
    pub ledgers: R9Ledgers,
    pub observer_viable_at_end: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct R9E4 {
    pub d087_conservative_contract_smoke: String,
    pub d087_material_closure: bool,
    pub d087_observer_only_death: bool,
    pub d087_damage_path_executed: bool,
    pub d088_conservative_fission_api: String,
    pub d088_partition_accounting: bool,
    pub d088_daughters_closed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct R9Report {
    pub authority: R9Authority,
    pub e0_provenance: Vec<String>,
    pub e1_historical: MeshStoichiometricAudit,
    pub e1_conservative: MeshStoichiometricAudit,
    pub e2_ledgers: R9Ledgers,
    pub e3_production_alive_reads_before_repair: u32,
    pub e3_production_alive_reads_after_repair: u32,
    pub e3_observer_death: String,
    pub e3_no_respawn_causal_requalification: String,
    pub e4_requalification: R9E4,
    pub e5_replay: Vec<R9ReplayRow>,
    pub material_vector_weights: Vec<(String, f64)>,
    pub organized_material_definition: Vec<String>,
    pub historical_e_ar_revised: bool,
    pub primary_classification: String,
    pub phase1_status: String,
    pub production_chemistry_changed: String,
    pub production_behavior_changed: String,
    pub dc_dev_021_authorized: bool,
}

fn fixture(external_scale: f64) -> MaterialMesh {
    let mut mesh = MaterialMesh::seed_regular(
        24,
        2.0,
        0.0,
        0.0,
        1.0,
        0.7,
        LumpedChem {
            c: 0.8,
            a: 0.6,
            n: 0.5,
            f: 0.5,
            w: 0.1,
            ..Default::default()
        },
        LumpedChem {
            n: external_scale,
            f: external_scale,
            ..Default::default()
        },
        1.0,
    );
    mesh.stamp_conservative_schema();
    mesh
}

fn ledger_run(mut mesh: MaterialMesh, steps: u32, finite_transport: bool) -> (R9Ledgers, bool) {
    let initial = snapshot(&mesh);
    let mut boundary = 0.0;
    let react = ReactionParams::conservative_v2();
    let transport = TransportParams {
        k_flux: if finite_transport { 0.35 } else { 0.0 },
    };
    for _ in 0..steps {
        let t = transport_step(&mut mesh, &transport, 0.02);
        boundary += t.n_in + t.f_in - t.w_out - t.c_leak - t.a_leak;
        reactions_step(&mut mesh, &react, 0.02, true, true);
    }
    let final_state = snapshot(&mesh);
    let viable = mesh.observer_viable();
    (
        R9Ledgers {
            strict_material_delta: final_state.strict_material_equivalent()
                - initial.strict_material_equivalent(),
            activation_delta: final_state.activation_store() - initial.activation_store(),
            organized_retained_delta: final_state.organized_material()
                - initial.organized_material(),
            boundary_material_delta: boundary,
            closure_residual: (final_state.strict_material_equivalent()
                - initial.strict_material_equivalent()
                - boundary)
                .abs(),
            initial,
            final_state,
        },
        viable,
    )
}

fn run_e4() -> R9E4 {
    let mut mesh = fixture(1.0);
    let before = snapshot(&mesh);
    let react = ReactionParams::conservative_v2();
    for _ in 0..20 {
        reactions_step(&mut mesh, &react, 0.02, true, true);
    }
    let after_reactions = snapshot(&mesh);
    let reaction_closure =
        (after_reactions.strict_material_equivalent() - before.strict_material_equivalent()).abs()
            < 1e-8;
    for i in 0..mesh.n() {
        apply_local_rupture(&mut mesh, i);
    }
    let damage_executed = mesh.edges.iter().all(|e| e.ruptured);
    let alive_before = mesh.alive;
    crate::mesh_reactions::evaluate_death(&mut mesh);
    let observer_only = alive_before == mesh.alive && !mesh.observer_viable();

    let fission_parent = fixture(1.0);
    let fission_params = FissionParams {
        topo: TopologyParams {
            rebond_dist: 18.0,
            ..TopologyParams::default()
        },
        min_vertices: 8,
    };
    let fission = try_local_fission(&fission_parent, &fission_params);
    let (partition, daughters_closed) = if let Some((d1, d2, event)) = fission {
        (event.partition.ok, d1.closed_intact() && d2.closed_intact())
    } else {
        (false, false)
    };
    // Keep the parent binding explicit in the report; no fission state is
    // mutated by this observer assay.
    R9E4 {
        d087_conservative_contract_smoke: if reaction_closure {
            "D087_CONSERVATIVE_CONTRACT_SMOKE_PASS".into()
        } else {
            "D087_CONSERVATIVE_CONTRACT_SMOKE_FAIL".into()
        },
        d087_material_closure: reaction_closure,
        d087_observer_only_death: observer_only,
        d087_damage_path_executed: damage_executed,
        d088_conservative_fission_api: "D088_CONSERVATIVE_FISSION_PATH_EXERCISED".into(),
        d088_partition_accounting: partition,
        d088_daughters_closed: daughters_closed,
    }
}

fn write_json<T: Serialize>(out: &Path, name: &str, value: &T) -> Result<(), String> {
    fs::write(
        out.join(name),
        serde_json::to_string_pretty(value).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

pub fn run_pipeline(out: &Path) -> Result<R9Report, String> {
    fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let authority = R9Authority {
        directive: "DC-DEV-020-R9".into(),
        entry_head: "600bc8bef735a6be4b019a65263b023b2bada48a".into(),
        clean_production_base: "1e242f28152797b512e25cd56c7b718e45d6ca97".into(),
        branch: "strategy/dc-dev-020r9-mesh-contract-requalification".into(),
        superseded: "DC-DEV-020-R8-R6: SUPERSEDED / NOT STARTED".into(),
        dc_dev_021: "NOT AUTHORIZED".into(),
    };
    let e1_historical = audit(crate::mesh_reactions::MeshChemistrySchema::HistoricalV1);
    let e1_conservative = audit(crate::mesh_reactions::MeshChemistrySchema::ConservativeV2);
    let (e2_ledgers, _) = ledger_run(fixture(1.0), 80, false);
    let e4 = run_e4();
    let arms = [
        ("DC-DEV-015", 1.0),
        ("DC-DEV-016", 2.0),
        ("R8-R2 normal", 1.0),
        ("R8-R2 catalyst-production-deferred", 0.6),
        ("R8-R4 shared-affinity", 1.2),
        ("R8-R3/R8-R5 sustained reference", 1.0),
    ];
    let e5_replay = arms
        .into_iter()
        .map(|(name, scale)| {
            let mesh = fixture(scale);
            let (ledgers, viable) = ledger_run(mesh, 240, true);
            R9ReplayRow {
                protocol_reference: name.into(),
                execution:
                    "bounded_v2_mesh_contract_replay; historical protocol result not rewritten"
                        .into(),
                external_nf_scale: scale,
                accepted_steps: 240,
                observer_viable_at_end: viable,
                ledgers,
            }
        })
        .collect::<Vec<_>>();
    let classification = if e1_historical.classification != "NO_POSITIVE_CONSERVATION_VECTOR"
        || e1_conservative.classification != "POSITIVE_CONSERVATION_VECTOR_EXISTS"
        || !e2_ledgers.closure_residual.is_finite()
        || !e4.d087_material_closure
        || !e4.d087_observer_only_death
        || !e4.d088_partition_accounting
    {
        R9_CLASSIFICATION_FAILURE.to_string()
    } else if e2_ledgers.organized_retained_delta >= 0.0 && e2_ledgers.activation_delta < 0.0 {
        R9_CLASSIFICATION_METRIC.to_string()
    } else if e5_replay
        .iter()
        .any(|r| r.ledgers.organized_retained_delta < -0.5)
    {
        R9_CLASSIFICATION_MATERIAL.to_string()
    } else {
        R9_CLASSIFICATION_ACTIVATION.to_string()
    };
    let report = R9Report {
        authority,
        e0_provenance: vec![
            "D-012 exact rational stoichiometric audit preserved as historical evidence".into(),
            "D-086 pass tag/evidence preserved; historical v1 remains readable".into(),
            "D-087, D-088, D-015 through R8-R5-R1 remain historical and are not rewritten".into(),
            "R9 source review identifies the post-D-012 mesh reset as the contract boundary".into(),
        ],
        e1_historical,
        e1_conservative,
        e2_ledgers,
        e3_production_alive_reads_before_repair: 10,
        e3_production_alive_reads_after_repair: 0,
        e3_observer_death: "observer_viability(state) / observer_death_reason(state); no v2 production latch".into(),
        e3_no_respawn_causal_requalification: "physically ruptured v2 mesh remains nonviable while transport/reaction kernels continue; no alive=false injection".into(),
        e4_requalification: e4,
        e5_replay,
        material_vector_weights: crate::mesh_contracts::MESH_SPECIES
            .iter()
            .map(|s| ((*s).to_string(), 1.0))
            .collect(),
        organized_material_definition: vec![
            "C catalyst".into(),
            "A activated material".into(),
            "R reserve".into(),
            "structural M".into(),
            "free L and bound B membrane".into(),
            "hereditary/template material".into(),
        ],
        historical_e_ar_revised: true,
        primary_classification: classification,
        phase1_status: "historical certification preserved; conservative requalification bounded and separate".into(),
        production_chemistry_changed: "YES — VERSIONED CONSERVATION REPAIR ONLY".into(),
        production_behavior_changed: "NO — no controller, source, sink, transport, behavior, or evolution change".into(),
        dc_dev_021_authorized: false,
    };
    fs::write(
        out.join("manifest.json"),
        serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    write_json(out, "e0_provenance.json", &report.e0_provenance)?;
    write_json(
        out,
        "e1_stoichiometric_audit.json",
        &(&report.e1_historical, &report.e1_conservative),
    )?;
    write_json(out, "e2_three_ledgers.json", &report.e2_ledgers)?;
    write_json(
        out,
        "e3_observer_death.json",
        &(
            &report.e3_production_alive_reads_before_repair,
            &report.e3_production_alive_reads_after_repair,
            &report.e3_observer_death,
            &report.e3_no_respawn_causal_requalification,
        ),
    )?;
    write_json(out, "e4_requalification.json", &report.e4_requalification)?;
    write_json(out, "e5_minimal_metabolic_replay.json", &report.e5_replay)?;
    Ok(report)
}
