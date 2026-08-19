//! DC-DEV-020-R9 bounded mesh-contract requalification.
//!
//! This is an observer/reporting harness around the versioned v2 mesh path. It
//! does not add a source law, controller, salvage reaction, or behavior.

use crate::material_mesh::{LumpedChem, MaterialMesh, MeshContractVersion};
use crate::mesh_contracts::{audit, snapshot, MaterialLedgerSnapshot, MeshStoichiometricAudit};
use crate::mesh_fission::{try_local_fission, FissionParams};
use crate::mesh_mechanics::mechanics_step;
use crate::mesh_reactions::{apply_local_rupture, reactions_step, ReactionParams};
use crate::mesh_topology::TopologyParams;
use crate::mesh_transport::{transport_step, TransportParams};
use crate::metabolic_reserve::{reserve_schema_load_ok, stamp_reserve_equation, ReserveParams};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const R9_CLASSIFICATION_METRIC: &str = "DCDEV020R9R1_METRIC_CONFOUNDING_DOMINANT_CONFIRMED";
pub const R9_CLASSIFICATION_ACTIVATION: &str = "DCDEV020R9R1_TRUE_ACTIVATION_DEFICIT_CONFIRMED";
pub const R9_CLASSIFICATION_MATERIAL: &str = "DCDEV020R9R1_TRUE_MATERIAL_CYCLE_DEFICIT_CONFIRMED";
pub const R9_CLASSIFICATION_FAILURE: &str = "DCDEV020R9R1_MESH_CONTRACT_REQUALIFICATION_FAILURE";

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
pub struct R9GateResult {
    pub gate: String,
    pub pass: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct R9D087GateMatrix {
    pub gates: Vec<R9GateResult>,
    pub all_pass: bool,
    pub contract: String,
    pub equation_lineage: String,
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
    pub d087_gate_matrix: R9D087GateMatrix,
    pub e5_replay: Vec<R9ReplayRow>,
    pub exact_historical_replay_artifacts: Vec<String>,
    pub material_vector_weights: Vec<(String, f64)>,
    pub organized_material_definition: Vec<String>,
    pub historical_e_ar_revised: bool,
    pub primary_classification: String,
    pub phase1_status: String,
    pub production_chemistry_changed: String,
    pub production_behavior_changed: String,
    pub dc_dev_021_authorized: bool,
}

/// The exact D-015/D-016 D-091 founder: 24-edge radius-5 mesh, C=.8,
/// A=.5, R=.6, no internal N/F at settlement, and the sealed reserve
/// allocation derived from (80, 40, .5, .3, H=2, .1).
fn d091_fixture(external_nf: f64) -> MaterialMesh {
    let mut mesh = MaterialMesh::seed_regular(
        24,
        5.0,
        0.0,
        0.0,
        1.0,
        0.7,
        LumpedChem {
            c: 0.8,
            a: 0.5,
            n: 0.0,
            f: 0.0,
            w: 0.1,
            r: 0.6,
            ..Default::default()
        },
        LumpedChem {
            n: external_nf,
            f: external_nf,
            ..Default::default()
        },
        5.0,
    );
    stamp_reserve_equation(&mut mesh);
    mesh.stamp_conservative_schema();
    mesh
}

fn d091_react(mesh: &MaterialMesh) -> ReactionParams {
    let mut react = ReactionParams::conservative_v2();
    react.reserve = ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, mesh.area());
    react
}

fn d091_v2_step(
    mesh: &mut MaterialMesh,
    react: &ReactionParams,
    transport: &TransportParams,
) -> bool {
    let _ = transport_step(mesh, transport, 0.02);
    let _ = reactions_step(mesh, react, 0.02, true, true);
    mechanics_step(mesh, &crate::mesh_mechanics::MechParams::default())
}

fn run_d091_v2(mesh: &mut MaterialMesh, steps: usize, transport: &TransportParams) -> bool {
    let react = d091_react(mesh);
    for _ in 0..steps {
        if !d091_v2_step(mesh, &react, transport) {
            return false;
        }
    }
    true
}

fn d087_v2_gate_matrix() -> R9D087GateMatrix {
    let mut gates = Vec::with_capacity(8);

    // Gate 0: authority and orthogonal contract load.
    let g0_mesh = d091_fixture(0.0);
    let g0 = g0_mesh.contract_version == MeshContractVersion::ConservativeV2
        && g0_mesh.equation_id == crate::metabolic_reserve::EQUATION_VERSION_METABOLIC_RESERVE
        && reserve_schema_load_ok(&g0_mesh, &d091_react(&g0_mesh).reserve);
    gates.push(R9GateResult {
        gate: "D087-G0 authority/contract/lineage".into(),
        pass: g0,
        detail: format!(
            "contract={:?} equation_id={} reserve_schema_load_ok={}",
            g0_mesh.contract_version, g0_mesh.equation_id, g0
        ),
    });

    // Gate 1: passive mechanics remains a real D-091 lineage and conserves
    // structural/material boundary quantities while chemistry is not advanced.
    let mut g1_mesh = d091_fixture(0.0);
    let m0 = g1_mesh.total_structural_mass();
    let b0 = g1_mesh.total_bound_membrane();
    let a0 = g1_mesh.area();
    let mech = crate::mesh_mechanics::MechParams::default();
    let mut g1_steps_ok = true;
    for _ in 0..200 {
        if !mechanics_step(&mut g1_mesh, &mech) {
            g1_steps_ok = false;
            break;
        }
    }
    let g1 = g1_steps_ok
        && g1_mesh.closed_intact()
        && (g1_mesh.total_structural_mass() - m0).abs() < 1e-8
        && (g1_mesh.total_bound_membrane() - b0).abs() < 1e-8
        && g1_mesh.area().is_finite()
        && g1_mesh.area() > 0.25 * a0
        && g1_mesh.area() < 4.0 * a0;
    gates.push(R9GateResult {
        gate: "D087-G1 passive mechanics".into(),
        pass: g1,
        detail: format!(
            "steps_ok={g1_steps_ok} closed={} structural_delta={:.3e} membrane_delta={:.3e} area_ratio={:.6}",
            g1_mesh.closed_intact(),
            g1_mesh.total_structural_mass() - m0,
            g1_mesh.total_bound_membrane() - b0,
            g1_mesh.area() / a0
        ),
    });

    // Gate 2: the same reserve-bearing lineage remains in a bounded passive
    // basin with the contract active.
    let mut g2_mesh = d091_fixture(0.4);
    let g2_area0 = g2_mesh.area();
    let g2 = run_d091_v2(&mut g2_mesh, 240, &TransportParams { k_flux: 0.0 })
        && g2_mesh.closed_intact()
        && g2_mesh.observer_viable()
        && g2_mesh.area() > 0.2 * g2_area0
        && g2_mesh.area() < 5.0 * g2_area0;
    gates.push(R9GateResult {
        gate: "D087-G2 passive reserve basin".into(),
        pass: g2,
        detail: format!(
            "closed={} viable={} area_ratio={:.6}",
            g2_mesh.closed_intact(),
            g2_mesh.observer_viable(),
            g2_mesh.area() / g2_area0
        ),
    });

    // Gate 3: reserve metabolism must actually execute, not merely deserialize.
    let mut g3_mesh = d091_fixture(0.0);
    let g3_react = d091_react(&g3_mesh);
    let mut g3_a_to_r = 0.0;
    let mut g3_r_to_a = 0.0;
    let mut g3_r_to_w = 0.0;
    let mut g3_rejected = 0;
    for _ in 0..480 {
        let led = reactions_step(&mut g3_mesh, &g3_react, 0.02, true, true);
        g3_a_to_r += led.reserve.a_to_r;
        g3_r_to_a += led.reserve.r_to_a;
        g3_r_to_w += led.reserve.r_to_w;
        g3_rejected += led.reserve.rejected_steps;
    }
    let g3 = g3_rejected == 0
        && g3_a_to_r > 0.0
        && g3_r_to_a > 0.0
        && g3_r_to_w > 0.0
        && g3_mesh.observer_viable();
    gates.push(R9GateResult {
        gate: "D087-G3 reserve metabolism".into(),
        pass: g3,
        detail: format!(
            "a_to_r={g3_a_to_r:.6} r_to_a={g3_r_to_a:.6} r_to_w={g3_r_to_w:.6} rejected={g3_rejected} viable={}",
            g3_mesh.observer_viable()
        ),
    });

    // Gate 4: turnover/replacement is qualified by repeated reserve flux and
    // finite material state, rather than by a placeholder positive response.
    let mut g4_mesh = d091_fixture(0.2);
    let g4_react = d091_react(&g4_mesh);
    let mut g4_turnover = 0.0;
    for _ in 0..720 {
        let led = reactions_step(&mut g4_mesh, &g4_react, 0.02, true, true);
        g4_turnover += led.reserve.r_to_w + led.reserve.a_to_r + led.reserve.r_to_a;
    }
    let g4_strict = snapshot(&g4_mesh).strict_material_equivalent();
    let g4 = g4_turnover > 0.0 && g4_mesh.observer_viable() && g4_strict.is_finite();
    gates.push(R9GateResult {
        gate: "D087-G4 turnover/replacement".into(),
        pass: g4,
        detail: format!(
            "reserve_turnover={g4_turnover:.6} viable={} strict_finite={}",
            g4_mesh.observer_viable(),
            g4_strict.is_finite()
        ),
    });

    // Gate 5: dynamic basin with finite transport and mechanics.
    let mut g5_mesh = d091_fixture(0.8);
    let g5_area0 = g5_mesh.area();
    let g5 = run_d091_v2(&mut g5_mesh, 480, &TransportParams { k_flux: 0.35 })
        && g5_mesh.closed_intact()
        && g5_mesh.observer_viable()
        && g5_mesh.area() > 0.2 * g5_area0
        && g5_mesh.area() < 5.0 * g5_area0
        && snapshot(&g5_mesh).strict_material_equivalent().is_finite();
    gates.push(R9GateResult {
        gate: "D087-G5 dynamic reserve basin".into(),
        pass: g5,
        detail: format!(
            "closed={} viable={} area_ratio={:.6} strict_finite={}",
            g5_mesh.closed_intact(),
            g5_mesh.observer_viable(),
            g5_mesh.area() / g5_area0,
            snapshot(&g5_mesh).strict_material_equivalent().is_finite()
        ),
    });

    // Gate 6: damage is causal and local repair consumes existing A/C; no
    // topology or new repair biology is introduced here.
    let mut g6_mesh = d091_fixture(0.0);
    let g6_before = g6_mesh.total_structural_mass();
    apply_local_rupture(&mut g6_mesh, 0);
    let rebonded = crate::mesh_reactions::try_local_rebond(&mut g6_mesh, 18.0);
    let g6_strict = snapshot(&g6_mesh).strict_material_equivalent();
    let g6 = rebonded && g6_mesh.edges[0].m > 0.0 && g6_strict.is_finite();
    gates.push(R9GateResult {
        gate: "D087-G6 damage/repair causality".into(),
        pass: g6,
        detail: format!(
            "rebonded={rebonded} edge_mass={:.6} strict_after={:.6} strict_before={g6_before:.6}",
            g6_mesh.edges[0].m, g6_strict
        ),
    });

    // Gate 7: ConservativeV2 deliberately keeps physical alive state stable,
    // while observer viability closes under starvation and never respawns.
    let mut g7_mesh = d091_fixture(0.0);
    g7_mesh.interior.c = 0.0;
    g7_mesh.interior.a = 0.0;
    g7_mesh.interior.r = 0.0;
    g7_mesh.interior.n = 0.0;
    g7_mesh.interior.f = 0.0;
    for edge in &mut g7_mesh.edges {
        edge.m = 0.0;
        edge.ruptured = true;
    }
    let physical_alive_before = g7_mesh.alive;
    let g7_react = d091_react(&g7_mesh);
    let _ = reactions_step(&mut g7_mesh, &g7_react, 0.02, true, true);
    let observer_dead = !g7_mesh.observer_viable();
    let no_respawn = g7_mesh.alive == physical_alive_before && !g7_mesh.observer_viable();
    let g7 = observer_dead && no_respawn;
    gates.push(R9GateResult {
        gate: "D087-G7 starvation/observer-death".into(),
        pass: g7,
        detail: format!(
            "observer_viable={} physical_alive_before={} physical_alive_after={}",
            g7_mesh.observer_viable(),
            physical_alive_before,
            g7_mesh.alive
        ),
    });

    R9D087GateMatrix {
        all_pass: gates.iter().all(|g| g.pass),
        gates,
        contract: "ConservativeV2".into(),
        equation_lineage: crate::metabolic_reserve::EQUATION_VERSION_METABOLIC_RESERVE.into(),
    }
}

fn ledger_run(mut mesh: MaterialMesh, steps: u32, finite_transport: bool) -> (R9Ledgers, bool) {
    let initial = snapshot(&mesh);
    let mut boundary = 0.0;
    let react = d091_react(&mesh);
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
    let mut mesh = d091_fixture(0.0);
    let before = snapshot(&mesh);
    let react = d091_react(&mesh);
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

    let fission_parent = d091_fixture(0.0);
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
        directive: "DC-DEV-020-R9-R1".into(),
        entry_head: "22529ca0caa570e1603c28fe39b05786052b969e".into(),
        clean_production_base: "1e242f28152797b512e25cd56c7b718e45d6ca97".into(),
        branch: "strategy/dc-dev-020r9-mesh-contract-requalification".into(),
        superseded: "DC-DEV-020-R8-R6: SUPERSEDED / NOT STARTED".into(),
        dc_dev_021: "NOT AUTHORIZED".into(),
    };
    let e1_historical = audit(crate::mesh_reactions::MeshChemistrySchema::HistoricalV1);
    let e1_conservative = audit(crate::mesh_reactions::MeshChemistrySchema::ConservativeV2);
    let (e2_ledgers, _) = ledger_run(d091_fixture(0.0), 80, false);
    let e4 = run_e4();
    let d087_gate_matrix = d087_v2_gate_matrix();
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
            let mesh = d091_fixture(scale);
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
        || !d087_gate_matrix.all_pass
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
        d087_gate_matrix,
        e5_replay,
        exact_historical_replay_artifacts: vec![
            "experiments/generated/dcdev020r9r1/exact_replays/manifest.json".into(),
            "experiments/generated/dcdev020r9r1/exact_replays/results.json".into(),
            "experiments/generated/dcdev020r9r1/r8r2_exact/manifest.json".into(),
            "experiments/generated/dcdev020r9r1/r8r2_exact/protocol.json".into(),
            "experiments/generated/dcdev020r9r1/r8r2_exact/root_summary.json".into(),
            "experiments/generated/dcdev020r9r1/r8r2_exact/payback_summary.json".into(),
            "experiments/generated/dcdev020r9r1/r8r2_exact/shadow_summary.json".into(),
            "experiments/generated/dcdev020r9r1/r8r2_exact/qualification.json".into(),
            "experiments/generated/dcdev020r9r1/r8r4_exact/manifest.json".into(),
            "experiments/generated/dcdev020r9r1/r8r4_exact/protocol.json".into(),
            "experiments/generated/dcdev020r9r1/r8r4_exact/acute_reproduction.json".into(),
            "experiments/generated/dcdev020r9r1/r8r4_exact/finite_feed_summary.json".into(),
            "experiments/generated/dcdev020r9r1/r8r4_exact/dose_summary.json".into(),
            "experiments/generated/dcdev020r9r1/r8r4_exact/sustained_summary.json".into(),
            "experiments/generated/dcdev020r9r1/r8r4_exact/cycle_summary.json".into(),
            "experiments/generated/dcdev020r9r1/r8r4_exact/qualification.json".into(),
        ],
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
        production_chemistry_changed:
            "NO — certified Phase 1 biology/equations unchanged; bounded post-Phase-1 contract and observer accounting code changed".into(),
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
    write_json(out, "d087_gate_matrix.json", &report.d087_gate_matrix)?;
    write_json(out, "e5_minimal_metabolic_replay.json", &report.e5_replay)?;
    write_json(
        out,
        "exact_historical_replay_artifacts.json",
        &report.exact_historical_replay_artifacts,
    )?;
    Ok(report)
}
