//! DC-DEV-020-M1-R3 versioned ordinary-decay candidate qualification.
//!
//! ConservativeV3 is an experimental chemistry schema.  It keeps the
//! ConservativeV2 material contract and every kinetic parameter, but does not
//! apply the starvation-specific fourfold multiplier to activated-material
//! decay.  This example is observer/qualification machinery only; it does not
//! select V3 as production.

#[path = "dcdev020m1r1_capacity_decomp.rs"]
mod m1r1;

use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_contracts::snapshot;
use chemistry_core::mesh_mechanics::MechParams;
use chemistry_core::mesh_reactions::{
    reactions_step, MeshChemistrySchema, ReactionLedger, ReactionParams,
};
use chemistry_core::mesh_transport::TransportParams;
use regulatory_core::{stable_json_hash, FiniteSpatialResourceRegionV1};
use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-020-M1-R3-ORDINARY-DECAY-CANDIDATE-001";
const STARTING_HEAD: &str = "1622b664a4a37b8a0ac4ea51fbc97ca71f9d853c";
const HORIZON_STEPS: usize = 480;
const RUPTURE_SEARCH_STEPS: usize = 1_000_000;
const REFEED_STEPS: usize = 5_000;
const DT: f64 = 0.02;
const K_A_DECAY: f64 = 0.008;
const ORDINARY_REFERENCE_K_A_DECAY: f64 = 0.002;
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
    waste: f64,
    r: f64,
    structural_m: f64,
    organized_material: f64,
    free_l: f64,
    bound_b: f64,
    strict_material: f64,
    observer_viable: bool,
    observer_death_reason: Option<&'static str>,
    closed_intact: bool,
    physical_runtime_valid: bool,
    ruptured_edges: usize,
    min_edge_m: f64,
}

fn state(mesh: &MaterialMesh, step: usize) -> State {
    let s = snapshot(mesh);
    let min_edge_m = mesh
        .edges
        .iter()
        .map(|edge| edge.m)
        .min_by(|a, b| a.total_cmp(b))
        .unwrap_or(0.0);
    State {
        step,
        area: mesh.area(),
        n: s.n,
        f: s.f,
        a: s.a,
        c: s.c,
        waste: s.waste,
        r: s.r,
        structural_m: s.structural_m,
        organized_material: s.organized_material(),
        free_l: s.free_l,
        bound_b: s.bound_b,
        strict_material: s.strict_material_equivalent(),
        observer_viable: mesh.observer_viable(),
        observer_death_reason: mesh.observer_death_reason(),
        closed_intact: mesh.closed_intact(),
        physical_runtime_valid: mesh.physical_runtime_valid(),
        ruptured_edges: mesh.edges.iter().filter(|edge| edge.ruptured).count(),
        min_edge_m,
    }
}

#[derive(Debug, Clone, Default, Serialize)]
struct CapacityTotals {
    n_delivered: f64,
    f_delivered: f64,
    a_produced: f64,
    a_decayed: f64,
    structural_production: f64,
    structural_turnover: f64,
    catalyst_production: f64,
    catalyst_turnover: f64,
    membrane_production: f64,
    membrane_turnover: f64,
}

impl CapacityTotals {
    fn absorb(&mut self, ledger: &ReactionLedger) {
        self.a_produced += ledger.a_produced;
        self.a_decayed += ledger.a_decayed;
        self.structural_production += ledger.m_produced;
        self.structural_turnover += ledger.m_to_w;
        self.catalyst_production += ledger.c_produced;
        self.catalyst_turnover += ledger.c_turned;
        self.membrane_production += ledger.l_produced;
        self.membrane_turnover += ledger.bind_extent + ledger.unbind_extent;
    }
}

#[derive(Debug, Clone, Serialize)]
struct CapacityResult {
    schema: MeshChemistrySchema,
    k_a_decay: f64,
    initial: State,
    final_state: State,
    totals: CapacityTotals,
    organized_material_delta: f64,
    strict_material_delta: f64,
    world_to_organism_closure_residual: f64,
    max_resource_conservation_error: f64,
}

#[derive(Debug, Clone, Serialize)]
struct RefeedResult {
    mode: String,
    accepted_steps: usize,
    n_delivered: f64,
    f_delivered: f64,
    closed_intact_before: bool,
    closed_intact_after: bool,
    ruptured_edges_before: usize,
    ruptured_edges_after: usize,
    world_to_organism_closure_residual: f64,
    final_state: State,
}

#[derive(Debug, Clone, Serialize)]
struct StarvationEquivalence {
    v2_reference_k_a_decay: f64,
    v3_k_a_decay: f64,
    v2_observer_collapse_step: Option<usize>,
    v3_observer_collapse_step: Option<usize>,
    v2_first_rupture_step: Option<usize>,
    v3_first_rupture_step: Option<usize>,
    v2_480: State,
    v3_480: State,
    v2_endpoint_20480: State,
    v3_endpoint_20480: State,
    max_state_abs_difference: f64,
    exact_rupture_step_match: bool,
    pass: bool,
}

#[derive(Debug, Clone, Serialize)]
struct NonstarvedParity {
    steps: usize,
    n_times_f_positive: bool,
    max_state_abs_difference: f64,
    exact_mesh_and_ledger_replay: bool,
    pass: bool,
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= TOL * (1.0 + a.abs().max(b.abs()))
}

fn json_close(a: &Value, b: &Value, tol: f64) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => match (x.as_f64(), y.as_f64()) {
            (Some(x), Some(y)) => (x - y).abs() <= tol * (1.0 + x.abs().max(y.abs())),
            _ => x == y,
        },
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(a, b)| json_close(a, b, tol))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len()
                && x.iter().all(|(key, value)| {
                    y.get(key)
                        .is_some_and(|other| json_close(value, other, tol))
                })
        }
        _ => a == b,
    }
}

fn state_distance(a: &State, b: &State) -> f64 {
    [
        (a.area - b.area).abs(),
        (a.n - b.n).abs(),
        (a.f - b.f).abs(),
        (a.a - b.a).abs(),
        (a.c - b.c).abs(),
        (a.waste - b.waste).abs(),
        (a.r - b.r).abs(),
        (a.structural_m - b.structural_m).abs(),
        (a.organized_material - b.organized_material).abs(),
        (a.free_l - b.free_l).abs(),
        (a.bound_b - b.bound_b).abs(),
        (a.strict_material - b.strict_material).abs(),
        (a.min_edge_m - b.min_edge_m).abs(),
    ]
    .into_iter()
    .fold(0.0, f64::max)
}

fn reaction_params(schema: MeshChemistrySchema, k_a_decay: f64) -> ReactionParams {
    let mut params = match schema {
        MeshChemistrySchema::HistoricalV1 => ReactionParams::default(),
        MeshChemistrySchema::ConservativeV2 => ReactionParams::conservative_v2(),
        MeshChemistrySchema::ConservativeV3 => ReactionParams::conservative_v3(),
    };
    params.k_a_decay = k_a_decay;
    params
}

fn paired_source_upper_bound(mesh: &mut MaterialMesh) {
    let area = mesh.area().max(1e-6);
    let paired = mesh.interior.n.min(mesh.interior.f).max(0.0) * area;
    mesh.interior.n = (mesh.interior.n - paired / area).max(0.0);
    mesh.interior.f = (mesh.interior.f - paired / area).max(0.0);
    mesh.interior.a += paired / area;
    mesh.interior.w += paired / area;
}

fn parameter_isolation_pass() -> bool {
    let v2 = reaction_params(MeshChemistrySchema::ConservativeV2, K_A_DECAY);
    let v3 = reaction_params(MeshChemistrySchema::ConservativeV3, K_A_DECAY);
    let mut v2_json = serde_json::to_value(v2).expect("V2 serializes");
    let mut v3_json = serde_json::to_value(v3).expect("V3 serializes");
    v2_json
        .as_object_mut()
        .expect("V2 object")
        .remove("mesh_schema");
    v3_json
        .as_object_mut()
        .expect("V3 object")
        .remove("mesh_schema");
    v2_json == v3_json
        && v2.mesh_schema == MeshChemistrySchema::ConservativeV2
        && v3.mesh_schema == MeshChemistrySchema::ConservativeV3
        && close(v2.k_a_decay, K_A_DECAY)
        && close(v3.k_a_decay, K_A_DECAY)
}

fn run_nonstarved_parity(initial: &MaterialMesh, mechanics: &MechParams) -> NonstarvedParity {
    let mut v2 = initial.clone();
    let mut v3 = initial.clone();
    v2.interior.n = 100.0;
    v2.interior.f = 100.0;
    v3.interior.n = 100.0;
    v3.interior.f = 100.0;
    let p2 = reaction_params(MeshChemistrySchema::ConservativeV2, K_A_DECAY);
    let p3 = reaction_params(MeshChemistrySchema::ConservativeV3, K_A_DECAY);
    let mut max_difference: f64 = 0.0;
    let mut exact = true;
    let mut n_times_f_positive = true;
    for _step in 1..=HORIZON_STEPS {
        let l2 = reactions_step(&mut v2, &p2, mechanics.dt, true, true);
        let l3 = reactions_step(&mut v3, &p3, mechanics.dt, true, true);
        n_times_f_positive &=
            v2.interior.n * v2.interior.f >= 1e-8 && v3.interior.n * v3.interior.f >= 1e-8;
        let s2 = state(&v2, _step);
        let s3 = state(&v3, _step);
        max_difference = max_difference.max(state_distance(&s2, &s3));
        exact &= serde_json::to_value(&v2).expect("mesh serializes")
            == serde_json::to_value(&v3).expect("mesh serializes")
            && serde_json::to_value(&l2).expect("ledger serializes")
                == serde_json::to_value(&l3).expect("ledger serializes");
        assert!(json_close(
            &serde_json::to_value(&v2).expect("mesh serializes"),
            &serde_json::to_value(&v3).expect("mesh serializes"),
            TOL
        ));
        assert!(json_close(
            &serde_json::to_value(&l2).expect("ledger serializes"),
            &serde_json::to_value(&l3).expect("ledger serializes"),
            TOL
        ));
    }
    NonstarvedParity {
        steps: HORIZON_STEPS,
        n_times_f_positive,
        max_state_abs_difference: max_difference,
        exact_mesh_and_ledger_replay: exact,
        pass: n_times_f_positive && max_difference <= TOL,
    }
}

fn run_starvation_equivalence(
    initial: &MaterialMesh,
    mechanics: &MechParams,
) -> (StarvationEquivalence, MaterialMesh) {
    let mut v2 = initial.clone();
    let mut v3 = initial.clone();
    let p2 = reaction_params(
        MeshChemistrySchema::ConservativeV2,
        ORDINARY_REFERENCE_K_A_DECAY,
    );
    let p3 = reaction_params(MeshChemistrySchema::ConservativeV3, K_A_DECAY);
    let mut v2_collapse = None;
    let mut v3_collapse = None;
    let mut v2_rupture = None;
    let mut v3_rupture = None;
    let mut v3_ruptured_mesh = None;
    let mut max_difference: f64 = 0.0;
    let mut v2_480 = None;
    let mut v3_480 = None;
    let mut v2_endpoint = None;
    let mut v3_endpoint = None;

    for step in 1..=RUPTURE_SEARCH_STEPS {
        reactions_step(&mut v2, &p2, mechanics.dt, true, true);
        reactions_step(&mut v3, &p3, mechanics.dt, true, true);
        let s2 = state(&v2, step);
        let s3 = state(&v3, step);
        max_difference = max_difference.max(state_distance(&s2, &s3));
        if v2_collapse.is_none() && !s2.observer_viable {
            v2_collapse = Some(step);
        }
        if v3_collapse.is_none() && !s3.observer_viable {
            v3_collapse = Some(step);
        }
        if v2_rupture.is_none() && s2.ruptured_edges > 0 {
            v2_rupture = Some(step);
        }
        if v3_rupture.is_none() && s3.ruptured_edges > 0 {
            v3_rupture = Some(step);
            v3_ruptured_mesh = Some(v3.clone());
        }
        if step == HORIZON_STEPS {
            v2_480 = Some(s2.clone());
            v3_480 = Some(s3.clone());
        }
        if step == 20_480 {
            v2_endpoint = Some(s2.clone());
            v3_endpoint = Some(s3.clone());
        }
        if v2_rupture.is_some() && v3_rupture.is_some() {
            break;
        }
    }

    let v2_480 = v2_480.expect("480-step ordinary reference state");
    let v3_480 = v3_480.expect("480-step V3 state");
    let v2_endpoint = v2_endpoint.expect("20,480-step ordinary reference state");
    let v3_endpoint = v3_endpoint.expect("20,480-step V3 state");
    let pass = v2_collapse == v3_collapse
        && v2_rupture == v3_rupture
        && max_difference <= 1e-7
        && v3_rupture.is_some();
    let evidence = StarvationEquivalence {
        v2_reference_k_a_decay: ORDINARY_REFERENCE_K_A_DECAY,
        v3_k_a_decay: K_A_DECAY,
        v2_observer_collapse_step: v2_collapse,
        v3_observer_collapse_step: v3_collapse,
        v2_first_rupture_step: v2_rupture,
        v3_first_rupture_step: v3_rupture,
        v2_480,
        v3_480,
        v2_endpoint_20480: v2_endpoint,
        v3_endpoint_20480: v3_endpoint,
        max_state_abs_difference: max_difference,
        exact_rupture_step_match: v2_rupture == v3_rupture,
        pass,
    };
    (
        evidence,
        v3_ruptured_mesh.expect("V3 reaches topology rupture"),
    )
}

fn run_capacity(
    initial: &MaterialMesh,
    mechanics: &MechParams,
    schema: MeshChemistrySchema,
    k_a_decay: f64,
) -> CapacityResult {
    let mut mesh = initial.clone();
    let initial_state = state(&mesh, 0);
    let params = reaction_params(schema, k_a_decay);
    let transport = TransportParams::default();
    let mut region = FiniteSpatialResourceRegionV1::new(
        RESOURCE_CENTER,
        RESOURCE_RADIUS,
        RESOURCE_N,
        RESOURCE_F,
    );
    let mut totals = CapacityTotals::default();
    let mut max_resource_conservation_error: f64 = 0.0;
    for _step in 1..=HORIZON_STEPS {
        let uptake = region.uptake(&mut mesh, &transport, mechanics.dt);
        assert!(uptake.conservation_error <= TOL);
        max_resource_conservation_error =
            max_resource_conservation_error.max(uptake.conservation_error);
        paired_source_upper_bound(&mut mesh);
        let ledger = reactions_step(&mut mesh, &params, mechanics.dt, true, true);
        totals.n_delivered += uptake.n_delivered;
        totals.f_delivered += uptake.f_delivered;
        totals.absorb(&ledger);
    }
    let final_state = state(&mesh, HORIZON_STEPS);
    let strict_delta = final_state.strict_material - initial_state.strict_material;
    let world_to_organism_closure_residual =
        (strict_delta - totals.n_delivered - totals.f_delivered).abs();
    CapacityResult {
        schema,
        k_a_decay,
        initial: initial_state.clone(),
        final_state: final_state.clone(),
        totals,
        organized_material_delta: final_state.organized_material - initial_state.organized_material,
        strict_material_delta: strict_delta,
        world_to_organism_closure_residual,
        max_resource_conservation_error,
    }
}

fn run_refeed(
    rupture_mesh: &MaterialMesh,
    mechanics: &MechParams,
    params: &ReactionParams,
    source_capacity_shadow: bool,
) -> RefeedResult {
    let mut mesh = rupture_mesh.clone();
    let initial = state(&mesh, 0);
    let mut region = FiniteSpatialResourceRegionV1::new(
        RESOURCE_CENTER,
        RESOURCE_RADIUS,
        RESOURCE_N,
        RESOURCE_F,
    );
    let transport = TransportParams::default();
    let mut n_delivered = 0.0;
    let mut f_delivered = 0.0;
    let mut max_resource_conservation_error: f64 = 0.0;
    for _step in 1..=REFEED_STEPS {
        let uptake = region.uptake(&mut mesh, &transport, mechanics.dt);
        assert!(uptake.conservation_error <= TOL);
        max_resource_conservation_error =
            max_resource_conservation_error.max(uptake.conservation_error);
        n_delivered += uptake.n_delivered;
        f_delivered += uptake.f_delivered;
        if source_capacity_shadow {
            paired_source_upper_bound(&mut mesh);
        }
        reactions_step(&mut mesh, params, mechanics.dt, true, true);
    }
    let final_state = state(&mesh, REFEED_STEPS);
    let strict_delta = final_state.strict_material - initial.strict_material;
    assert!(max_resource_conservation_error <= TOL);
    RefeedResult {
        mode: if source_capacity_shadow {
            "SOURCE_CAPACITY_UPPER_BOUND".into()
        } else {
            "ORDINARY_FINITE_RESOURCE".into()
        },
        accepted_steps: REFEED_STEPS,
        n_delivered,
        f_delivered,
        closed_intact_before: initial.closed_intact,
        closed_intact_after: final_state.closed_intact,
        ruptured_edges_before: initial.ruptured_edges,
        ruptured_edges_after: final_state.ruptured_edges,
        world_to_organism_closure_residual: (strict_delta - n_delivered - f_delivered).abs(),
        final_state,
    }
}

fn read_report(path: &Path) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_else(
            || json!({"status": "deferred_to_exact_workflow", "path": path.display().to_string()}),
        )
}

fn d087_pass(report: &Value, contract: &str) -> bool {
    report["mesh_contract"] == contract
        && report["reserve_enabled"] == false
        && [
            "gate0", "gate1", "gate2", "gate3", "gate4", "gate5", "gate6", "gate7",
        ]
        .iter()
        .all(|gate| report[*gate]["pass"] == true)
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
    let out = std::env::var_os("DCDEV020M1R3_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020m1r3"));
    let (entry, mechanics) = m1r1::m1r1_entry_state();
    assert!(close(mechanics.dt, DT));
    let entry_state = state(&entry, 0);
    let v2 = reaction_params(MeshChemistrySchema::ConservativeV2, K_A_DECAY);
    let v3 = reaction_params(MeshChemistrySchema::ConservativeV3, K_A_DECAY);
    let e0 = close(entry_state.a, 19.69467805250676)
        && close(entry_state.c, 55.87794642665143)
        && close(entry_state.organized_material, 131.80639622655494)
        && v2.reserve.enable == false
        && v2.mesh_schema == MeshChemistrySchema::ConservativeV2;
    assert!(
        e0,
        "entry authority or V2 selection mismatch: {entry_state:?}"
    );

    let e1 = parameter_isolation_pass();
    assert!(e1, "V2/V3 parameter isolation failed");
    let parity = run_nonstarved_parity(&entry, &mechanics);
    assert!(parity.pass, "non-starved V2/V3 parity failed: {parity:?}");
    let (starvation, v3_rupture_mesh) = run_starvation_equivalence(&entry, &mechanics);
    assert!(
        starvation.pass,
        "ordinary starvation equivalence failed: {starvation:?}"
    );

    let v2_capacity = run_capacity(
        &entry,
        &mechanics,
        MeshChemistrySchema::ConservativeV2,
        ORDINARY_REFERENCE_K_A_DECAY,
    );
    let v3_capacity = run_capacity(
        &entry,
        &mechanics,
        MeshChemistrySchema::ConservativeV3,
        K_A_DECAY,
    );
    let mut v2_capacity_physical = serde_json::to_value(&v2_capacity).expect("capacity serializes");
    let mut v3_capacity_physical = serde_json::to_value(&v3_capacity).expect("capacity serializes");
    for value in [&mut v2_capacity_physical, &mut v3_capacity_physical] {
        value
            .as_object_mut()
            .expect("capacity result object")
            .remove("schema");
        value
            .as_object_mut()
            .expect("capacity result object")
            .remove("k_a_decay");
    }
    assert!(json_close(
        &v2_capacity_physical,
        &v3_capacity_physical,
        1e-7
    ));
    assert!(close(
        v3_capacity.organized_material_delta,
        1.25718049040759
    ));
    assert!(v3_capacity.world_to_organism_closure_residual <= TOL);

    let v3_params = reaction_params(MeshChemistrySchema::ConservativeV3, K_A_DECAY);
    let ordinary_refeed = run_refeed(&v3_rupture_mesh, &mechanics, &v3_params, false);
    let source_capacity_refeed = run_refeed(&v3_rupture_mesh, &mechanics, &v3_params, true);
    assert!(!ordinary_refeed.closed_intact_after && !source_capacity_refeed.closed_intact_after);

    let v3_d087_path = out.join("v3_d087/certification/report.json");
    let v2_d087_path = out.join("v2_d087/certification/report.json");
    let v3_d087 = read_report(&v3_d087_path);
    let v2_d087 = read_report(&v2_d087_path);
    let v3_d087_pass = d087_pass(&v3_d087, "ConservativeV3");
    let v2_d087_pass = d087_pass(&v2_d087, "ConservativeV2");
    let local_assay_pass = e0
        && e1
        && parity.pass
        && starvation.pass
        && close(v3_capacity.organized_material_delta, 1.25718049040759)
        && v3_capacity.world_to_organism_closure_residual <= TOL
        && !ordinary_refeed.closed_intact_after
        && !source_capacity_refeed.closed_intact_after;
    let classification = if local_assay_pass && v3_d087_pass && v2_d087_pass {
        "M1_ORDINARY_DECAY_CANDIDATE_QUALIFIED"
    } else if !v3_d087_pass || !v2_d087_pass {
        "M1_ORDINARY_DECAY_CANDIDATE_D087_REGRESSION"
    } else {
        "M1_ORDINARY_DECAY_CANDIDATE_INVALID"
    };
    let protocol = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "selected_production": {"mesh_contract": "ConservativeV2", "reserve_enabled": false},
        "candidate": {"mesh_chemistry_schema": "ConservativeV3", "physical_mesh_contract": "ConservativeV2", "k_a_decay": K_A_DECAY, "starvation_multiplier": 1.0},
        "dt": DT,
        "nonstarved_steps": HORIZON_STEPS,
        "ordinary_starvation_reference_k_a_decay": ORDINARY_REFERENCE_K_A_DECAY,
        "rupture_search_steps": RUPTURE_SEARCH_STEPS,
        "refeed_steps": REFEED_STEPS,
        "source_capacity_steps": HORIZON_STEPS,
        "source_capacity_rule": "accepted paired internal N/F to A upper bound, unchanged",
        "resource": {"center": RESOURCE_CENTER, "radius": RESOURCE_RADIUS, "n": RESOURCE_N, "f": RESOURCE_F},
        "intentional_difference": "only starvation-specific activated-material decay multiplier: V2=4.0 when N*F<1e-8, V3=1.0",
        "forbidden_changes": ["ConservativeV2 semantics", "selected production", "N/F→A source law", "uptake", "transport", "D-091", "D-087 criteria", "mechanics", "remesh", "recycling", "salvage", "M2", "DC-DEV-021"]
    });
    let results = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "entry": entry_state,
        "nonstarved_parity": parity,
        "ordinary_starvation_equivalence": starvation,
        "source_capacity_v2_reference": v2_capacity,
        "source_capacity_v3": v3_capacity,
        "v3_refeed_ordinary": ordinary_refeed,
        "v3_refeed_source_capacity": source_capacity_refeed,
        "d087": {"v3": v3_d087, "v2": v2_d087},
        "classification": classification,
        "conservative_v2_semantics_changed": false,
        "selected_production_changed": false,
        "production_source_law_changed": false,
        "d091_changed": false,
        "parameter_search": false,
        "recycling": false,
        "m2_authorized": false,
        "dc_dev_021_authorized": false,
        "next_execution_started": false
    });
    let qualification = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "e0_authority_and_v2_preservation": e0,
        "e1_version_isolation": e1,
        "e2_causal_ordinary_starvation_equivalence": starvation.pass,
        "e3_source_capacity_without_decay_compensation": close(v3_capacity.organized_material_delta, 1.25718049040759),
        "e4_v3_d087": v3_d087_pass,
        "e4_v2_d087_preservation": v2_d087_pass,
        "e4_internal_material_closure": v3_capacity.world_to_organism_closure_residual <= TOL,
        "e5_exact_head_remote_ci_required": true,
        "observer_only": true,
        "selected_production_changed": false,
        "production_source_law_changed": false,
        "parameter_search": false,
        "recycling": false,
        "m2_authorized": false,
        "dc_dev_021_authorized": false,
        "next_execution_started": false,
        "classification": classification
    });
    let manifest = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "artifact_files": ["protocol.json", "results.json", "qualification.json", "artifact_manifest.json"],
        "authoritative_result": "results.json",
        "dense_ledgers_committed": false,
        "observer_only": true,
        "next_execution_started": false,
        "evidence_hash": stable_json_hash(&results)?
    });
    write_json(&out.join("protocol.json"), &protocol)?;
    write_json(&out.join("results.json"), &results)?;
    write_json(&out.join("qualification.json"), &qualification)?;
    write_json(&out.join("artifact_manifest.json"), &manifest)?;
    println!("DCDEV020M1R3_ORDINARY_DECAY_CANDIDATE_COMPLETE");
    println!("classification={classification}");
    println!(
        "v3_observer_collapse_step={:?}",
        starvation.v3_observer_collapse_step
    );
    println!(
        "v3_topology_rupture_step={:?}",
        starvation.v3_first_rupture_step
    );
    println!(
        "source_capacity_organized_delta={}",
        v3_capacity.organized_material_delta
    );
    println!("v3_d087_pass={v3_d087_pass} v2_d087_pass={v2_d087_pass}");
    println!("{}", out.display());
    Ok(())
}
