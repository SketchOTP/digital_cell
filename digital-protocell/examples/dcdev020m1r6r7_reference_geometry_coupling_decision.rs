//! DC-DEV-020-M1-R6-R7 observer-only reference-geometry coupling decision.
//!
//! The diagnostic owns a per-edge reference-length vector.  Production mesh
//! material, chemistry, mechanics defaults, and serialized organism state are
//! not changed by this example.

#[path = "dcdev020m1r1_capacity_decomp.rs"]
mod r5_entry;

use chemistry_core::material_mesh::{
    conserve_interior_amount_across_area_change, MaterialMesh, MeshEdge,
};
use chemistry_core::mesh_contracts::snapshot;
use chemistry_core::mesh_mechanics::{
    mechanics_step, mechanics_step_with_reference_lengths, remesh, MechParams,
};
use chemistry_core::mesh_reactions::{
    reactions_step, reactions_step_with_reference_lengths, try_local_rebond, ReactionLedger,
    ReactionParams,
};
use chemistry_core::mesh_transport::TransportParams;
use phase1_certifier::frozen::FROZEN_CENTER;
use regulatory_core::{stable_json_hash, FiniteSpatialBackingReservoirV1};
use serde::Serialize;
use serde_json::{json, Value};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-020-M1-R6-R7-REFERENCE-GEOMETRY-COUPLING-DECISION-001";
const STARTING_HEAD: &str = "821f6a85c1d4825715090c8ccb3482ceddccbde5";
const DT: f64 = 0.02;
const CENTER: [f64; 2] = [4.8, 0.0];
const RADIUS: f64 = 1.5;
const RESOURCE_MASS: f64 = 243.14924801053778;
const CONCENTRATION: f64 = 2.063914918930895;
const HORIZON: usize = 8_000;
const DEPRIVATION: usize = 480;
const TOLERANCE: f64 = 1e-8;
const CHECKPOINTS: [usize; 7] = [0, 480, 1_000, 2_000, 4_000, 6_000, 8_000];
const ATLAS_DENSE_ROOT: &str =
    r"\\atlas\ATLAS\100_ACTIVE\Projects\DIGITAL_CELL\evidence\dcdev020m1r6r7";

#[derive(Debug, Clone, Serialize)]
struct SourceStep {
    step: usize,
    n: f64,
    f: f64,
}

#[derive(Debug, Clone, Serialize)]
struct State {
    step: usize,
    organized_material: f64,
    strict_material: f64,
    a: f64,
    c: f64,
    structural_m: f64,
    free_l: f64,
    bound_b: f64,
    waste: f64,
    area: f64,
    perimeter: f64,
    reference_perimeter: Option<f64>,
    vertex_count: usize,
    mean_strain: f64,
    max_positive_strain: f64,
    mean_turn_scale: f64,
    resource_n_remaining: f64,
    resource_f_remaining: f64,
    n_delivered: f64,
    f_delivered: f64,
    closed_intact: bool,
    observer_viable: bool,
    physical_runtime_valid: bool,
}

#[derive(Debug, Clone, Serialize, Default)]
struct Totals {
    activation: f64,
    a_decay: f64,
    catalyst_turnover: f64,
    structural_production: f64,
    structural_turnover: f64,
    membrane_production: f64,
    membrane_turnover: f64,
}

impl Totals {
    fn absorb(&mut self, ledger: &ReactionLedger) {
        self.activation += ledger.a_produced;
        self.a_decay += ledger.a_decayed;
        self.catalyst_turnover += ledger.c_turned;
        self.structural_production += ledger.m_produced;
        self.structural_turnover += ledger.m_to_w;
        self.membrane_production += ledger.l_produced;
        self.membrane_turnover += ledger.bind_extent + ledger.unbind_extent;
    }
}

#[derive(Debug, Clone, Serialize, Default)]
struct Closure {
    max_strict_residual: f64,
    max_mechanics_residual: f64,
    max_remesh_residual: f64,
    max_rebond_residual: f64,
}

impl Closure {
    fn pass(&self) -> bool {
        self.max_strict_residual <= TOLERANCE
            && self.max_mechanics_residual <= TOLERANCE
            && self.max_remesh_residual <= TOLERANCE
            && self.max_rebond_residual <= TOLERANCE
    }
}

#[derive(Debug, Clone, Serialize)]
struct StepRecord {
    step: usize,
    state: State,
    source: SourceStep,
    totals: Totals,
    strict_residual: f64,
    mechanics_residual: f64,
    remesh_residual: f64,
    rebond_residual: f64,
    reference_mapping_ok: bool,
}

#[derive(Debug, Clone, Serialize)]
struct ArmResult {
    arm: String,
    mode: String,
    initial: State,
    final_state: State,
    checkpoints: Vec<State>,
    totals: Totals,
    closure: Closure,
    n_delivered: f64,
    f_delivered: f64,
    source_schedule_hash: String,
    trajectory_hash: String,
    final_mesh_hash: String,
    remesh_splits: usize,
    remesh_merges: usize,
    reference_mapping_pass: bool,
    mechanically_bounded: bool,
    organized_material_delta: f64,
}

#[derive(Debug, Clone, Copy)]
enum Mode {
    Static,
    Current,
    Reference,
}

impl Mode {
    fn id(self) -> &'static str {
        match self {
            Self::Static => "GEOMETRY_FROZEN_STATIC",
            Self::Current => "MOVING_CURRENT_PRODUCTION",
            Self::Reference => "MOVING_REFERENCE_DECOUPLED_SHADOW",
        }
    }

    fn moving(self) -> bool {
        !matches!(self, Self::Static)
    }
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= TOLERANCE * (1.0 + a.abs().max(b.abs()))
}

fn reservoir() -> FiniteSpatialBackingReservoirV1 {
    FiniteSpatialBackingReservoirV1::new(
        CENTER,
        RADIUS,
        RESOURCE_MASS,
        RESOURCE_MASS,
        CONCENTRATION,
        CONCENTRATION,
    )
}

fn apply_schedule(
    mesh: &mut MaterialMesh,
    n: f64,
    f: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    let area = mesh.area();
    if !area.is_finite() || area <= 0.0 {
        return Err("cannot apply material schedule to non-positive GC area".into());
    }
    mesh.interior.n += n / area;
    mesh.interior.f += f / area;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_lineage_split_and_merge_preserves_mapping_and_material() {
        let (mut split_mesh, _) = r5_entry::m1r1_entry_state();
        let mut split_refs: Vec<f64> = (0..split_mesh.n())
            .map(|i| split_mesh.rest_length(i))
            .collect();
        let split_reference_sum: f64 = split_refs.iter().sum();
        let split_material = snapshot(&split_mesh).strict_material_equivalent();
        split_mesh.l_max = 1.0;
        let splits = diagnostic_split(&mut split_mesh, &mut split_refs);
        assert!(splits > 0, "fixture must exercise at least one split");
        assert_eq!(split_refs.len(), split_mesh.n());
        assert!(split_refs
            .iter()
            .all(|value| value.is_finite() && *value > 0.0));
        assert!(close(split_refs.iter().sum(), split_reference_sum));
        assert!(close(
            snapshot(&split_mesh).strict_material_equivalent(),
            split_material
        ));

        let (mut merge_mesh, _) = r5_entry::m1r1_entry_state();
        let mut merge_refs: Vec<f64> = (0..merge_mesh.n())
            .map(|i| merge_mesh.rest_length(i))
            .collect();
        let merge_reference_sum: f64 = merge_refs.iter().sum();
        merge_mesh.l_min = 10.0;
        let merges = diagnostic_merge(&mut merge_mesh, &mut merge_refs);
        assert!(merges > 0, "fixture must exercise at least one merge");
        assert_eq!(merge_refs.len(), merge_mesh.n());
        assert!(merge_refs
            .iter()
            .all(|value| value.is_finite() && *value > 0.0));
        assert!(close(merge_refs.iter().sum(), merge_reference_sum));
    }
}

fn strain_for(mesh: &MaterialMesh, i: usize, references: Option<&[f64]>) -> f64 {
    let l0 = references
        .and_then(|values| values.get(i).copied())
        .unwrap_or_else(|| mesh.rest_length(i));
    (mesh.edge_length(i) - l0) / l0
}

fn state(
    mesh: &MaterialMesh,
    step: usize,
    refs: Option<&[f64]>,
    n_remaining: f64,
    f_remaining: f64,
    dn: f64,
    df: f64,
) -> State {
    let s = snapshot(mesh);
    let mut strain_sum = 0.0;
    let mut max_positive: f64 = 0.0;
    let mut turn_sum = 0.0;
    let mut count = 0.0;
    for i in 0..mesh.n() {
        if mesh.edges[i].ruptured {
            continue;
        }
        let strain = strain_for(mesh, i, refs);
        strain_sum += strain;
        max_positive = max_positive.max(strain.max(0.0));
        turn_sum += 1.0 / (1.0 + 2.0 * strain.max(0.0));
        count += 1.0;
    }
    State {
        step,
        organized_material: s.organized_material(),
        strict_material: s.strict_material_equivalent(),
        a: s.a,
        c: s.c,
        structural_m: s.structural_m,
        free_l: s.free_l,
        bound_b: s.bound_b,
        waste: s.waste,
        area: mesh.area(),
        perimeter: mesh.perimeter(),
        reference_perimeter: refs.map(|values| values.iter().sum()),
        vertex_count: mesh.n(),
        mean_strain: if count > 0.0 { strain_sum / count } else { 0.0 },
        max_positive_strain: max_positive,
        mean_turn_scale: if count > 0.0 { turn_sum / count } else { 0.0 },
        resource_n_remaining: n_remaining,
        resource_f_remaining: f_remaining,
        n_delivered: dn,
        f_delivered: df,
        closed_intact: mesh.closed_intact(),
        observer_viable: mesh.observer_viable(),
        physical_runtime_valid: mesh.physical_runtime_valid(),
    }
}

fn diagnostic_split(mesh: &mut MaterialMesh, refs: &mut Vec<f64>) -> usize {
    let mut splits = 0;
    let mut i = 0;
    let limit = mesh.n().saturating_mul(4).max(16);
    while i < mesh.n() && splits < limit {
        if mesh.edges[i].ruptured || mesh.edge_length(i) <= mesh.l_max {
            i += 1;
            continue;
        }
        let j = (i + 1) % mesh.n();
        let a = mesh.vertices[i];
        let b = mesh.vertices[j];
        let edge = mesh.edges[i];
        let reference = refs[i] * 0.5;
        mesh.vertices
            .insert(i + 1, [0.5 * (a[0] + b[0]), 0.5 * (a[1] + b[1])]);
        mesh.edges[i] = MeshEdge {
            m: edge.m * 0.5,
            b: edge.b * 0.5,
            tracer_m: edge.tracer_m * 0.5,
            tracer_b: edge.tracer_b * 0.5,
            ruptured: false,
        };
        mesh.edges.insert(
            i + 1,
            MeshEdge {
                m: edge.m * 0.5,
                b: edge.b * 0.5,
                tracer_m: edge.tracer_m * 0.5,
                tracer_b: edge.tracer_b * 0.5,
                ruptured: false,
            },
        );
        refs[i] = reference;
        refs.insert(i + 1, reference);
        splits += 1;
        i += 2;
    }
    splits
}

fn diagnostic_merge(mesh: &mut MaterialMesh, refs: &mut Vec<f64>) -> usize {
    let mut merges = 0;
    let limit = mesh.n().saturating_mul(2).max(8);
    while merges < limit && mesh.n() > 6 {
        let n = mesh.n();
        let mut pick = None;
        for i in 0..n {
            let j = (i + 1) % n;
            if !mesh.edges[i].ruptured
                && !mesh.edges[j].ruptured
                && mesh.edge_length(i) < mesh.l_min
            {
                pick = Some(i);
                break;
            }
        }
        let Some(i) = pick else { break };
        let j = (i + 1) % n;
        if j == 0 {
            mesh.vertices.rotate_left(1);
            mesh.edges.rotate_left(1);
            refs.rotate_left(1);
            let i2 = mesh.n() - 2;
            mesh.edges[i2] = MeshEdge {
                m: mesh.edges[i2].m + mesh.edges[i2 + 1].m,
                b: mesh.edges[i2].b + mesh.edges[i2 + 1].b,
                tracer_m: mesh.edges[i2].tracer_m + mesh.edges[i2 + 1].tracer_m,
                tracer_b: mesh.edges[i2].tracer_b + mesh.edges[i2 + 1].tracer_b,
                ruptured: false,
            };
            refs[i2] += refs[i2 + 1];
            mesh.edges.pop();
            mesh.vertices.pop();
            refs.pop();
        } else {
            mesh.edges[i] = MeshEdge {
                m: mesh.edges[i].m + mesh.edges[j].m,
                b: mesh.edges[i].b + mesh.edges[j].b,
                tracer_m: mesh.edges[i].tracer_m + mesh.edges[j].tracer_m,
                tracer_b: mesh.edges[i].tracer_b + mesh.edges[j].tracer_b,
                ruptured: false,
            };
            refs[i] += refs[j];
            mesh.edges.remove(j);
            mesh.vertices.remove(j);
            refs.remove(j);
        }
        merges += 1;
    }
    merges
}

fn diagnostic_remesh(mesh: &mut MaterialMesh, refs: &mut Vec<f64>) -> (usize, usize, bool) {
    if refs.len() != mesh.n() {
        return (0, 0, false);
    }
    let before_material = snapshot(mesh).strict_material_equivalent();
    let before_refs = refs.iter().sum::<f64>();
    let area_before = mesh.area();
    let splits = diagnostic_split(mesh, refs);
    let merges = diagnostic_merge(mesh, refs);
    let area_after = mesh.area();
    let area_conserved = conserve_interior_amount_across_area_change(mesh, area_before, area_after);
    let mapping_ok = refs.len() == mesh.n()
        && refs.iter().all(|value| value.is_finite() && *value > 0.0)
        && close(refs.iter().sum(), before_refs)
        && area_conserved
        && close(snapshot(mesh).strict_material_equivalent(), before_material);
    (splits, merges, mapping_ok)
}

fn run_arm(
    initial: &MaterialMesh,
    initial_refs: Option<&[f64]>,
    mode: Mode,
    schedule: Option<&[SourceStep]>,
    steps: usize,
    name: &str,
    dense_root: Option<&Path>,
) -> Result<(ArmResult, MaterialMesh, Option<Vec<f64>>, Vec<SourceStep>), Box<dyn std::error::Error>>
{
    let mut mesh = initial.clone();
    let mut refs = if matches!(mode, Mode::Reference) {
        Some(initial_refs.map_or_else(
            || (0..mesh.n()).map(|i| mesh.rest_length(i)).collect(),
            ToOwned::to_owned,
        ))
    } else {
        None
    };
    if refs
        .as_ref()
        .is_some_and(|values: &Vec<f64>| values.len() != mesh.n())
    {
        return Err("reference state does not match mesh topology".into());
    }
    let mechanics = MechParams::default();
    assert_eq!(mechanics.dt, DT);
    let reaction = ReactionParams::conservative_v3();
    assert!(!reaction.reserve.enable);
    let transport = TransportParams::default();
    let mut world = if schedule.is_none() {
        Some(reservoir())
    } else {
        None
    };
    let mut remaining_n = if schedule.is_some_and(|sealed| sealed.is_empty()) {
        0.0
    } else {
        RESOURCE_MASS
    };
    let mut remaining_f = remaining_n;
    let initial_state = state(
        &mesh,
        0,
        refs.as_deref(),
        remaining_n,
        remaining_f,
        0.0,
        0.0,
    );
    let mut checkpoints = vec![initial_state.clone()];
    let mut trajectory = vec![stable_json_hash(&initial_state)?];
    let mut generated_schedule = Vec::with_capacity(steps);
    let mut totals = Totals::default();
    let mut closure = Closure::default();
    let mut n_delivered = 0.0;
    let mut f_delivered = 0.0;
    let mut mapping_pass = true;
    let mut bounded = true;
    let mut split_count = 0;
    let mut merge_count = 0;
    let mut dense = dense_root
        .map(|root| File::create(root.join(format!("{name}.jsonl"))).map(BufWriter::new))
        .transpose()?;

    for step in 1..=steps {
        let (dn, df) = if let Some(sealed) = schedule {
            if sealed.is_empty() {
                (0.0, 0.0)
            } else {
                let item = sealed.get(step - 1).ok_or("sealed schedule too short")?;
                remaining_n -= item.n;
                remaining_f -= item.f;
                if remaining_n < -TOLERANCE || remaining_f < -TOLERANCE {
                    return Err("sealed source schedule exceeded inventory".into());
                }
                apply_schedule(&mut mesh, item.n, item.f)?;
                (item.n, item.f)
            }
        } else {
            let world = world.as_mut().expect("spatial source world");
            let ledger = world.uptake(&mut mesh, &transport, mechanics.dt);
            remaining_n = world.region.n_mass;
            remaining_f = world.region.f_mass;
            if ledger.conservation_error > TOLERANCE {
                return Err("world transport conservation failed".into());
            }
            (ledger.n_delivered, ledger.f_delivered)
        };
        generated_schedule.push(SourceStep { step, n: dn, f: df });
        n_delivered += dn;
        f_delivered += df;
        let before = snapshot(&mesh).strict_material_equivalent();
        let ledger = if let Some(values) = refs.as_deref() {
            reactions_step_with_reference_lengths(&mut mesh, &reaction, DT, true, true, values)
        } else {
            reactions_step(&mut mesh, &reaction, DT, true, true)
        };
        totals.absorb(&ledger);
        let after_reaction = snapshot(&mesh).strict_material_equivalent();
        let mut mechanics_residual = 0.0;
        let mut remesh_residual = 0.0;
        let mut rebond_residual = 0.0;
        if mode.moving() {
            let before_mechanics = snapshot(&mesh).strict_material_equivalent();
            let ok = if let Some(values) = refs.as_deref() {
                mechanics_step_with_reference_lengths(&mut mesh, &mechanics, values)
            } else {
                mechanics_step(&mut mesh, &mechanics)
            };
            if !ok {
                return Err(format!("mechanics rejected at step {step}").into());
            }
            mechanics_residual = snapshot(&mesh).strict_material_equivalent() - before_mechanics;
            let before_remesh = snapshot(&mesh).strict_material_equivalent();
            if let Some(values) = refs.as_mut() {
                let (splits, merges, ok) = diagnostic_remesh(&mut mesh, values);
                split_count += splits;
                merge_count += merges;
                mapping_pass &= ok;
            } else {
                let (splits, merges) = remesh(&mut mesh);
                split_count += splits;
                merge_count += merges;
            }
            remesh_residual = snapshot(&mesh).strict_material_equivalent() - before_remesh;
            let before_rebond = snapshot(&mesh).strict_material_equivalent();
            let _ = try_local_rebond(
                &mut mesh,
                chemistry_core::material_mesh::DEFAULT_REBOND_DIST,
            );
            rebond_residual = snapshot(&mesh).strict_material_equivalent() - before_rebond;
        }
        let strict_residual = after_reaction - before;
        closure.max_strict_residual = closure.max_strict_residual.max(strict_residual.abs());
        closure.max_mechanics_residual =
            closure.max_mechanics_residual.max(mechanics_residual.abs());
        closure.max_remesh_residual = closure.max_remesh_residual.max(remesh_residual.abs());
        closure.max_rebond_residual = closure.max_rebond_residual.max(rebond_residual.abs());
        bounded &= mesh.physical_runtime_valid()
            && mesh.area().is_finite()
            && mesh.perimeter().is_finite()
            && mesh.area() < 10_000.0
            && mesh.n() < 10_000;
        let post = state(
            &mesh,
            step,
            refs.as_deref(),
            remaining_n,
            remaining_f,
            dn,
            df,
        );
        if CHECKPOINTS.contains(&step) {
            checkpoints.push(post.clone());
        }
        let record = StepRecord {
            step,
            state: post.clone(),
            source: SourceStep { step, n: dn, f: df },
            totals: totals.clone(),
            strict_residual,
            mechanics_residual,
            remesh_residual,
            rebond_residual,
            reference_mapping_ok: mapping_pass,
        };
        trajectory.push(stable_json_hash(&post)?);
        if let Some(writer) = dense.as_mut() {
            serde_json::to_writer(&mut *writer, &record)?;
            writer.write_all(b"\n")?;
        }
    }
    if let Some(writer) = dense.as_mut() {
        writer.flush()?;
    }
    let final_state = checkpoints.last().cloned().unwrap_or_else(|| {
        state(
            &mesh,
            steps,
            refs.as_deref(),
            remaining_n,
            remaining_f,
            0.0,
            0.0,
        )
    });
    let source_hash = stable_json_hash(&generated_schedule)?;
    let result = ArmResult {
        arm: name.to_string(),
        mode: mode.id().to_string(),
        initial: initial_state.clone(),
        final_state: final_state.clone(),
        checkpoints,
        totals,
        closure,
        n_delivered,
        f_delivered,
        source_schedule_hash: source_hash,
        trajectory_hash: stable_json_hash(&trajectory)?,
        final_mesh_hash: stable_json_hash(&mesh)?,
        remesh_splits: split_count,
        remesh_merges: merge_count,
        reference_mapping_pass: mapping_pass,
        mechanically_bounded: bounded,
        organized_material_delta: final_state.organized_material - initial_state.organized_material,
    };
    Ok((result, mesh, refs, generated_schedule))
}

fn read_report(path: &Path) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_else(|| json!({"status": "missing"}))
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
    let out = std::env::var_os("DCDEV020M1R6R7_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020m1r6r7"));
    let dense = std::env::var_os("DCDEV020M1R6R7_DENSE_OUTPUT")
        .map(PathBuf::from)
        .or_else(|| Some(PathBuf::from(ATLAS_DENSE_ROOT)));
    fs::create_dir_all(&out)?;
    if let Some(root) = dense.as_ref() {
        fs::create_dir_all(root)?;
    }
    let (mut entry, mechanics) = r5_entry::m1r1_entry_state();
    entry.stamp_geometry_conservative_schema();
    assert_eq!(mechanics.dt, FROZEN_CENTER.dt);

    let (static_arm, _, _, sealed_schedule) = run_arm(
        &entry,
        None,
        Mode::Static,
        None,
        HORIZON,
        "arm_a_frozen_geometry_reference",
        dense.as_deref(),
    )?;
    let (current_arm, _, _, _) = run_arm(
        &entry,
        None,
        Mode::Current,
        Some(&sealed_schedule),
        HORIZON,
        "arm_b_current_moving_production",
        dense.as_deref(),
    )?;
    let initial_refs: Vec<f64> = (0..entry.n()).map(|i| entry.rest_length(i)).collect();
    let (reference_arm, _, _, _) = run_arm(
        &entry,
        Some(&initial_refs),
        Mode::Reference,
        Some(&sealed_schedule),
        HORIZON,
        "arm_c_reference_decoupled_moving_shadow",
        dense.as_deref(),
    )?;

    let (deprived_b, deprived_b_mesh, _, _) = run_arm(
        &entry,
        None,
        Mode::Current,
        Some(&[]),
        DEPRIVATION,
        "arm_b_deprivation",
        dense.as_deref(),
    )?;
    let (recovery_b, _, _, _) = run_arm(
        &deprived_b_mesh,
        None,
        Mode::Current,
        Some(&sealed_schedule),
        HORIZON,
        "arm_b_recovery",
        dense.as_deref(),
    )?;
    let (deprived_c, deprived_c_mesh, deprived_c_refs, _) = run_arm(
        &entry,
        Some(&initial_refs),
        Mode::Reference,
        Some(&[]),
        DEPRIVATION,
        "arm_c_deprivation",
        dense.as_deref(),
    )?;
    let (recovery_c, _, _, _) = run_arm(
        &deprived_c_mesh,
        deprived_c_refs.as_deref(),
        Mode::Reference,
        Some(&sealed_schedule),
        HORIZON,
        "arm_c_recovery",
        dense.as_deref(),
    )?;

    let d087_v2 = read_report(&out.join("v2_d087/certification/report.json"));
    let d087_v3 = read_report(&out.join("v3_d087/certification/report.json"));
    let d087 = d087_pass(&d087_v2, "ConservativeV2") && d087_pass(&d087_v3, "ConservativeV3");
    let current_restoration = recovery_b.final_state.organized_material
        > deprived_b.final_state.organized_material
        && (recovery_b.final_state.organized_material - static_arm.initial.organized_material)
            .abs()
            < (deprived_b.final_state.organized_material - static_arm.initial.organized_material)
                .abs();
    let reference_restoration = recovery_c.final_state.organized_material
        > deprived_c.final_state.organized_material
        && (recovery_c.final_state.organized_material - static_arm.initial.organized_material)
            .abs()
            < (deprived_c.final_state.organized_material - static_arm.initial.organized_material)
                .abs();
    let sustained_homeostasis = reference_arm.organized_material_delta >= -TOLERANCE;
    let current_reproduced = close(static_arm.organized_material_delta, 0.3421406768903523)
        && close(current_arm.organized_material_delta, -9.954959206543037);
    let causal_cycle = reference_arm.closure.pass()
        && reference_arm.reference_mapping_pass
        && reference_arm.mechanically_bounded
        && reference_arm.organized_material_delta >= -TOLERANCE
        && reference_arm.totals.structural_turnover < current_arm.totals.structural_turnover;
    let classification = if current_reproduced && causal_cycle && reference_restoration {
        "M1_REFERENCE_GEOMETRY_COUPLING_CAUSALLY_CONFIRMED"
    } else if current_reproduced && causal_cycle {
        "M1_REFERENCE_GEOMETRY_COUPLING_HOMEOSTASIS_ONLY"
    } else if current_reproduced {
        "M1_REFERENCE_GEOMETRY_COUPLING_NOT_SUFFICIENT"
    } else {
        "M1_REFERENCE_GEOMETRY_COUPLING_INVALID"
    };

    let protocol = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "runtime": {"material": "GeometryConservativeV3", "chemistry": "ConservativeV3", "reserve": "OFF", "dt": DT},
        "source_schedule": "sealed R6-R6 successful frozen-geometry schedule generated from the accepted entry state",
        "horizon": HORIZON,
        "deprivation": DEPRIVATION,
        "arms": ["A_frozen_geometry", "B_current_moving", "C_reference_decoupled_moving", "B_deprivation_recovery", "C_deprivation_recovery"],
        "observer_only": true,
        "reference_state": "per-edge reference_length initialized to production rest_length; split halves and merge sums; zero material authority",
        "forbidden_changes": ["production biology", "coefficients", "mechanics defaults", "transport", "resource schedule", "target geometry", "controller", "recycling", "salvage", "M2", "R6-R8"],
        "next_execution_started": false
    });
    let results = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "arms": {"A": static_arm, "B": current_arm, "C": reference_arm},
        "restoration": {"B_deprived": deprived_b, "B_refed": recovery_b, "C_deprived": deprived_c, "C_refed": recovery_c, "B_pass": current_restoration, "C_pass": reference_restoration},
        "causal_decomposition": {
            "current_structural_production": current_arm.totals.structural_production,
            "current_structural_turnover": current_arm.totals.structural_turnover,
            "reference_structural_production": reference_arm.totals.structural_production,
            "reference_structural_turnover": reference_arm.totals.structural_turnover,
            "structural_turnover_reduction": current_arm.totals.structural_turnover - reference_arm.totals.structural_turnover,
            "current_final_m": current_arm.final_state.structural_m,
            "reference_final_m": reference_arm.final_state.structural_m,
            "current_final_area": current_arm.final_state.area,
            "reference_final_area": reference_arm.final_state.area,
            "current_final_perimeter": current_arm.final_state.perimeter,
            "reference_final_perimeter": reference_arm.final_state.perimeter,
            "reference_final_reference_perimeter": reference_arm.final_state.reference_perimeter,
            "static_organized_delta": static_arm.organized_material_delta,
            "current_organized_delta": current_arm.organized_material_delta,
            "reference_organized_delta": reference_arm.organized_material_delta
        },
        "checks": {"r6_r6_reproduction": current_reproduced, "reference_sustained_homeostasis": sustained_homeostasis, "current_restoration": current_restoration, "reference_restoration": reference_restoration, "reference_bounded": reference_arm.mechanically_bounded, "material_closure": static_arm.closure.pass() && current_arm.closure.pass() && reference_arm.closure.pass(), "remesh_reference_lineage": reference_arm.reference_mapping_pass, "d087": d087},
        "classification": classification,
        "production_scientific_code_changed": false,
        "target_size_or_shape_controller_added": false,
        "parameter_search": false,
        "source_schedule_changed": false,
        "m1": "NOT ESTABLISHED",
        "m2_authorized": false,
        "next_execution_started": false
    });
    let qualification = json!({
        "directive": DIRECTIVE,
        "e0_authority": true,
        "e1_reference_length_caller_audit": true,
        "e2_isolated_shadow_correctness": reference_arm.reference_mapping_pass && reference_arm.closure.pass(),
        "e3_causal_full_runtime": current_reproduced && causal_cycle,
        "e4_restoration_boundedness": reference_restoration && reference_arm.mechanically_bounded,
        "e5_remote_ci": "required",
        "observer_only": true,
        "classification": classification,
        "next_execution_started": false
    });
    let preservation = json!({
        "r6_r6_reproduction": current_reproduced,
        "geometry_material_conservation": static_arm.closure.pass() && current_arm.closure.pass() && reference_arm.closure.pass(),
        "historical_v2_d087": d087_pass(&d087_v2, "ConservativeV2"),
        "candidate_v3_d087": d087_pass(&d087_v3, "ConservativeV3"),
        "focused_remesh_reference_lineage": reference_arm.reference_mapping_pass,
        "tier": "R6-R6, GC conservation, V2/V3 D087, and focused reference-lineage preservation"
    });
    write_json(&out.join("protocol.json"), &protocol)?;
    write_json(&out.join("results.json"), &results)?;
    write_json(&out.join("qualification.json"), &qualification)?;
    write_json(&out.join("preservation.json"), &preservation)?;
    write_json(
        &out.join("artifact_manifest.json"),
        &json!({"schema": "dcdev020m1r6r7_manifest_v1", "directive": DIRECTIVE, "starting_head": STARTING_HEAD, "files": ["protocol.json", "results.json", "qualification.json", "preservation.json", "artifact_manifest.json"], "dense_output": ATLAS_DENSE_ROOT, "shared_drive_required": true, "sha256": "computed-by-workflow"}),
    )?;
    println!("DCDEV020M1R6R7_REFERENCE_GEOMETRY_COUPLING_DECISION_COMPLETE");
    println!("classification={classification}");
    println!(
        "static_organized_delta={}",
        static_arm.organized_material_delta
    );
    println!(
        "current_moving_organized_delta={}",
        current_arm.organized_material_delta
    );
    println!(
        "reference_decoupled_organized_delta={}",
        reference_arm.organized_material_delta
    );
    println!("reference_restoration={reference_restoration}");
    println!("next_execution_started=false");
    Ok(())
}
