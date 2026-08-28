//! DC-DEV-020-M1-REPLAN-002-R1 versioned production candidate.
//!
//! This harness is the only R1 scientific runner.  The prior REPLAN-002
//! example remains the immutable observer shadow oracle; this runner exercises
//! the new physical MaturationCoupledV4 contract.

#[path = "dcdev020m1r1_capacity_decomp.rs"]
mod r5_entry;

use chemistry_core::material_mesh::{MaterialMesh, MeshContractVersion};
use chemistry_core::mesh_contracts::snapshot;
use chemistry_core::mesh_mechanics::{mechanics_step, remesh, MechParams};
use chemistry_core::mesh_reactions::{reactions_step, try_local_rebond, ReactionParams};
use chemistry_core::mesh_transport::TransportParams;
use regulatory_core::{stable_json_hash, FiniteSpatialBackingReservoirV1};
use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

const DIRECTIVE: &str =
    "DC-DEV-020-M1-REPLAN-002-R1-MATURATION-COUPLED-PRODUCTION-CANDIDATE-QUALIFICATION-001";
const STARTING_HEAD: &str = "4becff4fff7d096c70468b759ace09f747c4eb56";
const DT: f64 = 0.02;
const CENTER: [f64; 2] = [4.8, 0.0];
const RADIUS: f64 = 1.5;
const RESOURCE_MASS: f64 = 243.14924801053778;
const BOUNDARY_CONCENTRATION: f64 = 2.063914918930895;
const FED_STEPS: usize = 8_000;
const DEPRIVATION_STEPS: usize = 480;
const STARVATION_BOUND: usize = 150_000;
const TOLERANCE: f64 = 1e-8;
const ORACLE_DELTA: f64 = 1.33231221701902;
const CHECKPOINTS: [usize; 7] = [0, 480, 1_000, 2_000, 4_000, 6_000, 8_000];

#[derive(Debug, Clone, Serialize)]
struct Checkpoint {
    step: usize,
    area: f64,
    perimeter: f64,
    structural_m: f64,
    young_m: f64,
    mature_m: f64,
    organized_material: f64,
    strict_material: f64,
    a: f64,
    c: f64,
    free_l: f64,
    bound_b: f64,
    n: f64,
    f: f64,
    n_delivered: f64,
    f_delivered: f64,
    n_remaining: f64,
    f_remaining: f64,
    observer_viable: bool,
    closed_intact: bool,
    physical_runtime_valid: bool,
}

#[derive(Debug, Clone, Serialize)]
struct Arm {
    name: String,
    fed: bool,
    initial: Checkpoint,
    final_state: Checkpoint,
    checkpoints: Vec<Checkpoint>,
    n_delivered: f64,
    f_delivered: f64,
    n_remaining: f64,
    f_remaining: f64,
    replenishment_events: u64,
    max_closure_residual: f64,
    m_produced: f64,
    m_matured: f64,
    m_turnover: f64,
    c_produced: f64,
    c_turnover: f64,
    a_decayed: f64,
    l_produced: f64,
    trajectory_hash: String,
    final_mesh_hash: String,
}

#[derive(Debug, Clone, Serialize)]
struct SourceStep {
    step: usize,
    n: f64,
    f: f64,
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= TOLERANCE * (1.0 + a.abs().max(b.abs()))
}

fn reservoir(n: f64, f: f64) -> FiniteSpatialBackingReservoirV1 {
    FiniteSpatialBackingReservoirV1::new(
        CENTER,
        RADIUS,
        n,
        f,
        BOUNDARY_CONCENTRATION,
        BOUNDARY_CONCENTRATION,
    )
}

fn checkpoint(
    mesh: &MaterialMesh,
    step: usize,
    n_delivered: f64,
    f_delivered: f64,
    n_remaining: f64,
    f_remaining: f64,
) -> Checkpoint {
    let s = snapshot(mesh);
    Checkpoint {
        step,
        area: mesh.area(),
        perimeter: mesh.perimeter(),
        structural_m: mesh.total_structural_mass(),
        young_m: mesh.total_young_structural_mass(),
        mature_m: (0..mesh.n()).map(|i| mesh.mature_structural_mass(i)).sum(),
        organized_material: s.organized_material(),
        strict_material: s.strict_material_equivalent(),
        a: s.a,
        c: s.c,
        free_l: s.free_l,
        bound_b: s.bound_b,
        n: s.n,
        f: s.f,
        n_delivered,
        f_delivered,
        n_remaining,
        f_remaining,
        observer_viable: mesh.observer_viable(),
        closed_intact: mesh.closed_intact(),
        physical_runtime_valid: mesh.physical_runtime_valid(),
    }
}

fn generate_sealed_source_schedule(entry: &MaterialMesh) -> Result<Vec<SourceStep>, String> {
    let mut mesh = entry.clone();
    let mut world = reservoir(RESOURCE_MASS, RESOURCE_MASS);
    let transport = TransportParams::default();
    let reaction = ReactionParams::conservative_v2();
    let mut schedule = Vec::with_capacity(FED_STEPS);
    for step in 1..=FED_STEPS {
        let uptake = world.uptake(&mut mesh, &transport, DT);
        if uptake.conservation_error > TOLERANCE {
            return Err(format!(
                "sealed source schedule closure failed at step {step}"
            ));
        }
        schedule.push(SourceStep {
            step,
            n: uptake.n_delivered,
            f: uptake.f_delivered,
        });
        reactions_step(&mut mesh, &reaction, DT, true, true);
    }
    Ok(schedule)
}

fn run_arm(
    initial: &MaterialMesh,
    schedule: Option<&[SourceStep]>,
    steps: usize,
    name: &str,
) -> Result<(Arm, MaterialMesh), String> {
    let mut mesh = initial.clone();
    if mesh.contract_version != MeshContractVersion::MaturationCoupledV4 {
        return Err("R1 runner received a non-V4 mesh".into());
    }
    let reaction = ReactionParams::conservative_v2();
    let mechanics = MechParams::default();
    let initial_n = schedule.map_or(0.0, |_| RESOURCE_MASS);
    let initial = checkpoint(&mesh, 0, 0.0, 0.0, initial_n, initial_n);
    let mut checkpoints = vec![initial.clone()];
    let mut trajectory = vec![stable_json_hash(&initial).map_err(|e| e.to_string())?];
    let mut n_delivered = 0.0;
    let mut f_delivered = 0.0;
    let mut max_closure_residual: f64 = 0.0;
    let mut m_produced = 0.0;
    let mut m_matured = 0.0;
    let mut m_turnover = 0.0;
    let mut c_produced = 0.0;
    let mut c_turnover = 0.0;
    let mut a_decayed = 0.0;
    let mut l_produced = 0.0;
    let mut dense = std::env::var_os("DCDEV020M1REPLAN002R1_DENSE_OUTPUT")
        .map(PathBuf::from)
        .map(|root| -> Result<BufWriter<fs::File>, String> {
            fs::create_dir_all(&root).map_err(|e| e.to_string())?;
            let path = root.join(format!("{name}.jsonl"));
            fs::File::create(path)
                .map(BufWriter::new)
                .map_err(|e| e.to_string())
        })
        .transpose()?;

    for step in 1..=steps {
        let before = snapshot(&mesh).strict_material_equivalent();
        let (dn, df) = schedule
            .and_then(|items| items.get(step - 1))
            .map_or((0.0, 0.0), |item| (item.n, item.f));
        let area = mesh.area();
        if !area.is_finite() || area <= 0.0 {
            return Err(format!(
                "invalid area before source delivery at step {step}"
            ));
        }
        mesh.interior.n += dn / area;
        mesh.interior.f += df / area;
        n_delivered += dn;
        f_delivered += df;
        let reaction_ledger = reactions_step(&mut mesh, &reaction, DT, true, true);
        m_produced += reaction_ledger.m_produced;
        m_matured += reaction_ledger.m_matured;
        m_turnover += reaction_ledger.m_to_w;
        c_produced += reaction_ledger.c_produced;
        c_turnover += reaction_ledger.c_turned;
        a_decayed += reaction_ledger.a_decayed;
        l_produced += reaction_ledger.l_produced;
        if !mechanics_step(&mut mesh, &mechanics) {
            return Err(format!("mechanics failed at step {step}"));
        }
        let _ = remesh(&mut mesh);
        let _ = try_local_rebond(
            &mut mesh,
            chemistry_core::material_mesh::DEFAULT_REBOND_DIST,
        );
        if !mesh.lifecycle_invariants_hold() {
            return Err(format!("V4 lifecycle invariant failed at step {step}"));
        }
        let after = snapshot(&mesh).strict_material_equivalent();
        let residual = (after - before - dn - df).abs();
        max_closure_residual = max_closure_residual.max(residual);
        let n_remaining = initial_n - n_delivered;
        let f_remaining = initial_n - f_delivered;
        let row = checkpoint(
            &mesh,
            step,
            n_delivered,
            f_delivered,
            n_remaining,
            f_remaining,
        );
        trajectory.push(stable_json_hash(&row).map_err(|e| e.to_string())?);
        if let Some(file) = dense.as_mut() {
            serde_json::to_writer(&mut *file, &row).map_err(|e| e.to_string())?;
            file.write_all(b"\n").map_err(|e| e.to_string())?;
        }
        if CHECKPOINTS.contains(&step) || step == steps {
            checkpoints.push(row);
        }
    }
    if let Some(file) = dense.as_mut() {
        file.flush().map_err(|e| e.to_string())?;
    }
    let final_state = checkpoints.last().cloned().unwrap_or_else(|| {
        checkpoint(
            &mesh,
            steps,
            n_delivered,
            f_delivered,
            initial_n - n_delivered,
            initial_n - f_delivered,
        )
    });
    let arm = Arm {
        name: name.into(),
        fed: schedule.is_some(),
        initial,
        final_state,
        checkpoints,
        n_delivered,
        f_delivered,
        n_remaining: initial_n - n_delivered,
        f_remaining: initial_n - f_delivered,
        replenishment_events: 0,
        max_closure_residual,
        m_produced,
        m_matured,
        m_turnover,
        c_produced,
        c_turnover,
        a_decayed,
        l_produced,
        trajectory_hash: stable_json_hash(&trajectory).map_err(|e| e.to_string())?,
        final_mesh_hash: stable_json_hash(&mesh).map_err(|e| e.to_string())?,
    };
    Ok((arm, mesh))
}

fn read_report(path: &Path) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| json!({"status": "missing"}))
}

fn d087_pass(report: &Value, contract: &str) -> bool {
    report["mesh_contract"] == contract
        && report["reserve_enabled"] == false
        && (0..8).all(|i| report[format!("gate{i}")]["pass"] == true)
        && report["primary_conclusion"] == "D087_PHASE1_AUTOPOIETIC_PROTOCELL_CERTIFIED"
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

fn main() -> Result<(), String> {
    let out = std::env::var_os("DCDEV020M1REPLAN002R1_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020m1replan002r1"));
    fs::create_dir_all(&out).map_err(|e| e.to_string())?;

    let (mut entry, mechanics) = r5_entry::m1r1_entry_state();
    if !close(mechanics.dt, DT) {
        return Err("entry dt differs from frozen authority".into());
    }
    let schedule = generate_sealed_source_schedule(&entry)?;
    entry.stamp_maturation_coupled_schema();
    let (fed, _) = run_arm(&entry, Some(&schedule), FED_STEPS, "V4_FED_8000")?;
    let (deprived, deprived_mesh) = run_arm(&entry, None, DEPRIVATION_STEPS, "V4_DEPRIVED_480")?;
    let (recovered, _) = run_arm(
        &deprived_mesh,
        Some(&schedule),
        FED_STEPS,
        "V4_RECOVERY_8000",
    )?;
    let (starvation, _) = run_arm(&entry, None, STARVATION_BOUND, "V4_STARVATION_150000")?;

    let fed_homeostasis = fed.final_state.organized_material + TOLERANCE
        >= fed.initial.organized_material
        && fed.final_state.observer_viable
        && fed.final_state.closed_intact;
    let recovery = recovered.final_state.organized_material
        > deprived.final_state.organized_material
        && (recovered.final_state.organized_material - fed.initial.organized_material).abs()
            < (deprived.final_state.organized_material - fed.initial.organized_material).abs();
    let starvation_decline = starvation.final_state.structural_m < starvation.initial.structural_m
        && starvation.final_state.organized_material < starvation.initial.organized_material;
    let shadow_parity = close(
        fed.final_state.organized_material - fed.initial.organized_material,
        ORACLE_DELTA,
    );
    let report_root = if out.ends_with("ci") {
        out.clone()
    } else {
        out.join("ci")
    };
    let v2 = read_report(&report_root.join("v2_d087/certification/report.json"));
    let v3 = read_report(&report_root.join("v3_d087/certification/report.json"));
    let v4 = read_report(&report_root.join("v4_d087/certification/report.json"));
    let d087_v2 = d087_pass(&v2, "ConservativeV2");
    let d087_v3 = d087_pass(&v3, "ConservativeV3");
    let d087_v4 = d087_pass(&v4, "MaturationCoupledV4");
    let preservation = d087_v2 && d087_v3 && d087_v4;
    let classification = if !shadow_parity {
        "M1_MATURATION_COUPLED_PRODUCTION_SHADOW_PARITY_FAILURE"
    } else if !fed_homeostasis || !recovery {
        "M1_MATURATION_COUPLED_PRODUCTION_HOMEOSTASIS_REGRESSION"
    } else if !starvation_decline || !preservation || fed.max_closure_residual > TOLERANCE {
        "M1_MATURATION_COUPLED_PRODUCTION_PRESERVATION_REGRESSION"
    } else {
        "M1_MATURATION_COUPLED_PRODUCTION_CANDIDATE_QUALIFIED"
    };
    let protocol = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "contract": "MaturationCoupledV4",
        "material_state": "edge.m = total physical material; edge.m_young = newly synthesized material; mature = max(m - young, 0)",
        "maturation": "min(m_young, k_turn * m_young * dt)",
        "load_bearing_reference": "mature / rho_s with existing 1e-15 numerical guard",
        "runtime": {"chemistry": "ConservativeV3", "reserve": "OFF", "dt": DT, "production_default": "ConservativeV2 / reserve OFF"},
        "shadow_oracle": {"organized_delta": ORACLE_DELTA, "source": "accepted immutable REPLAN-002 shadow"},
        "arms": ["V4_FED_8000", "V4_DEPRIVED_480", "V4_RECOVERY_8000", "V4_STARVATION_150000"],
        "observer_only_qualification": true,
        "next_execution_started": false
    });
    let results = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "fed": fed,
        "deprived": deprived,
        "recovered": recovered,
        "starvation": starvation,
        "checks": {"shadow_parity": shadow_parity, "fed_homeostasis": fed_homeostasis, "recovery": recovery, "starvation_decline": starvation_decline, "preservation": preservation, "d087_v2": d087_v2, "d087_v3": d087_v3, "d087_v4": d087_v4},
        "classification": classification,
        "production_default_changed": false,
        "new_parameter": false,
        "next_execution_started": false
    });
    let qualification = json!({
        "directive": DIRECTIVE,
        "classification": classification,
        "shadow_parity": shadow_parity,
        "fed_homeostasis": fed_homeostasis,
        "recovery": recovery,
        "starvation_decline": starvation_decline,
        "material_closure": fed.max_closure_residual <= TOLERANCE,
        "d087_v2": d087_v2,
        "d087_v3": d087_v3,
        "d087_v4": d087_v4,
        "next_execution_started": false
    });
    let preservation = json!({
        "v2_d087": d087_v2,
        "v3_d087": d087_v3,
        "v4_d087": d087_v4,
        "v4_lifecycle": fed.final_state.young_m >= 0.0 && fed.final_state.mature_m >= 0.0,
        "production_default": "ConservativeV2 / reserve OFF"
    });
    write_json(&out.join("protocol.json"), &protocol)?;
    write_json(&out.join("results.json"), &results)?;
    write_json(&out.join("qualification.json"), &qualification)?;
    write_json(&out.join("preservation.json"), &preservation)?;
    write_json(
        &out.join("artifact_manifest.json"),
        &json!({
            "schema": "dcdev020m1replan002r1_manifest_v1",
            "directive": DIRECTIVE,
            "starting_head": STARTING_HEAD,
            "files": ["protocol.json", "results.json", "qualification.json", "preservation.json", "artifact_manifest.json"],
            "dense_output": "/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/dcdev020m1replan002r1/dense",
            "shared_drive_required": true,
            "sha256": "computed-by-workflow"
        }),
    )?;
    println!("DCDEV020M1REPLAN002R1_PRODUCTION_CANDIDATE_QUALIFICATION_COMPLETE");
    println!("classification={classification}");
    println!("shadow_parity={shadow_parity}");
    println!("fed_homeostasis={fed_homeostasis}");
    println!("recovery={recovery}");
    println!("starvation_decline={starvation_decline}");
    println!("next_execution_started=false");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chemistry_core::autocatalytic_copying::{
        founder_h_edges, redistribute_edges_along_axis, seed_founder_edges,
    };
    use chemistry_core::autocatalytic_nodes::{stamp_autocatalytic_equation, AutocatalyticParams};
    use chemistry_core::material_mesh::{LumpedChem, DEFAULT_RHO_S};
    use chemistry_core::mesh_fission::{try_local_fission, FissionParams};
    use chemistry_core::mesh_growth::{growth_step, GrowthParams};
    use chemistry_core::mesh_reactions::apply_structural_damage;
    use chemistry_core::mesh_topology::find_local_pinch;
    use chemistry_core::mesh_transport::transport_step;
    use chemistry_core::metabolic_reserve::ReserveParams;

    #[test]
    fn v4_starts_existing_material_mature() {
        let (mut mesh, _) = r5_entry::m1r1_entry_state();
        mesh.stamp_maturation_coupled_schema();
        assert_eq!(mesh.total_young_structural_mass(), 0.0);
        assert!(mesh.lifecycle_invariants_hold());
    }

    #[test]
    fn v4_replay_is_deterministic() {
        let (mut mesh, _) = r5_entry::m1r1_entry_state();
        mesh.stamp_maturation_coupled_schema();
        let a = run_arm(&mesh, None, 4, "a").expect("first run").0;
        let b = run_arm(&mesh, None, 4, "b").expect("second run").0;
        assert_eq!(a.trajectory_hash, b.trajectory_hash);
        assert_eq!(a.final_mesh_hash, b.final_mesh_hash);
    }

    #[test]
    fn v4_serialization_preserves_young_state_and_historical_decode_is_mature() {
        let (mut mesh, _) = r5_entry::m1r1_entry_state();
        mesh.stamp_maturation_coupled_schema();
        mesh.edges[0].m_young = mesh.edges[0].m * 0.25;
        let encoded = serde_json::to_string(&mesh).expect("encode");
        let decoded: MaterialMesh = serde_json::from_str(&encoded).expect("decode");
        assert_eq!(decoded.edges[0].m_young, mesh.edges[0].m_young);
        let mut legacy = serde_json::to_value(mesh).expect("value");
        legacy.as_object_mut().unwrap().remove("contract_version");
        let old: MaterialMesh = serde_json::from_value(legacy).expect("legacy decode");
        assert_eq!(old.contract_version, MeshContractVersion::HistoricalV1);
    }

    #[test]
    fn v4_damage_removes_total_material_without_shielding_young_fraction() {
        let mut mesh = MaterialMesh::seed_regular(
            12,
            4.0,
            0.0,
            0.0,
            DEFAULT_RHO_S,
            0.7,
            LumpedChem {
                a: 1.0,
                c: 1.0,
                ..Default::default()
            },
            LumpedChem::default(),
            5.0,
        );
        mesh.stamp_maturation_coupled_schema();
        for edge in &mut mesh.edges {
            edge.m_young = edge.m * 0.4;
        }
        let before_total = mesh.total_structural_mass();
        let before_young = mesh.total_young_structural_mass();
        let removed = apply_structural_damage(&mut mesh, 0.5);
        assert!(removed > 0.0);
        assert!(mesh.total_structural_mass() < before_total);
        let after_total = mesh.total_structural_mass();
        let after_young = mesh.total_young_structural_mass();
        assert!((before_young / before_total - after_young / after_total).abs() < 1e-12);
        assert!(mesh.lifecycle_invariants_hold());
    }

    #[test]
    fn v4_fission_preserves_lifecycle_lineage_and_marks_new_cross_bond_young() {
        let mut mesh = MaterialMesh::seed_regular(
            24,
            12.0,
            40.0,
            40.0,
            DEFAULT_RHO_S,
            0.7,
            LumpedChem {
                a: 1.0,
                c: 0.8,
                n: 0.5,
                f: 0.5,
                r: 0.7,
                q_k: 0.5,
                q_e: 0.5,
                k_a: 0.15,
                k_r: 0.15,
                k_node_b: 0.15,
                ..Default::default()
            },
            LumpedChem {
                n: 1.5,
                f: 1.5,
                ..Default::default()
            },
            5.0,
        );
        let center = mesh.centroid();
        for vertex in &mut mesh.vertices {
            vertex[0] = center[0] + (vertex[0] - center[0]) * 1.45;
        }
        stamp_autocatalytic_equation(&mut mesh);
        seed_founder_edges(&mut mesh, &founder_h_edges());
        let mut reaction = ReactionParams::default();
        reaction.reserve = ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, mesh.area());
        reaction.reserve.enable = true;
        reaction.autocatalytic = AutocatalyticParams::derived(40.0).with_mutation_off();
        reaction.autocatalytic.k_edge_loss = 0.0;
        let growth = GrowthParams {
            y_g: 0.9,
            enable_growth: true,
        };
        let mechanics = MechParams::default();
        let transport = TransportParams::default();
        let fission = FissionParams::default();
        let birth_total = mesh.total_structural_mass();
        let mut ready_parent = None;
        for step in 0..12_000 {
            mesh.exterior.n = 1.5;
            mesh.exterior.f = 1.5;
            let _ = transport_step(&mut mesh, &transport, mechanics.dt);
            let _ = reactions_step(&mut mesh, &reaction, mechanics.dt, true, true);
            let _ = growth_step(&mut mesh, &reaction, &growth, mechanics.dt);
            assert!(mechanics_step(&mut mesh, &mechanics));
            remesh(&mut mesh);
            if step % 50 == 0 {
                redistribute_edges_along_axis(&mut mesh);
            }
            if mesh.total_structural_mass() >= 1.35 * birth_total && step % 10 == 0 {
                redistribute_edges_along_axis(&mut mesh);
                if find_local_pinch(&mesh, &fission.topo).is_some() {
                    ready_parent = Some(mesh.clone());
                    break;
                }
            }
        }
        let mut parent = ready_parent.expect("elongated fixture should reach a lawful local pinch");
        parent.stamp_maturation_coupled_schema();
        for edge in &mut parent.edges {
            edge.m_young = edge.m * 0.2;
        }
        let parent_young = parent.total_young_structural_mass();
        let parent_total = parent.total_structural_mass();
        let (d1, d2, event) = try_local_fission(&parent, &fission)
            .expect("V4 parent should produce a lawful local fission");
        assert!(event.partition.ok);
        assert_eq!(
            d1.contract_version,
            MeshContractVersion::MaturationCoupledV4
        );
        assert_eq!(
            d2.contract_version,
            MeshContractVersion::MaturationCoupledV4
        );
        let daughter_young = d1.total_young_structural_mass() + d2.total_young_structural_mass();
        let daughter_total = d1.total_structural_mass() + d2.total_structural_mass();
        let new_cross_bond = daughter_total - parent_total;
        assert!(new_cross_bond > 0.0);
        assert!((daughter_young - parent_young - new_cross_bond).abs() < 1e-9);
        assert!(d1.lifecycle_invariants_hold() && d2.lifecycle_invariants_hold());
    }
}
