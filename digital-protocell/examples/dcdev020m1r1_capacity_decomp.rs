//! DC-DEV-020-M1-R1 observer-only capacity decomposition.
//!
//! Four matched shadows are run from the exact accepted M1-R0 high-inventory
//! deprived state. The source upper bound and catalyst-investment deferral are
//! accounting/capacity shadows only; they do not alter production chemistry.

#[path = "dcdev020m1r0_requalification.rs"]
mod m1r0;

use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_contracts::snapshot;
use chemistry_core::mesh_mechanics::MechParams;
use chemistry_core::mesh_reactions::{reactions_step, ReactionLedger, ReactionParams};
use chemistry_core::mesh_transport::TransportParams;
use regulatory_core::{stable_json_hash, FiniteSpatialResourceRegionV1};
use serde::Serialize;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

const STARTING_HEAD: &str = "3cab12551072ad1eafaece72615f448d8efb9bea";
const M1R0_ENTRY: &str = "4895135deee7dbd782446dbfe25662181951afe0";
const SETTLEMENT_STEPS: usize = 5_000;
const DEPRIVATION_STEPS: usize = 480;
const HORIZON_STEPS: usize = 480;
const HIGH_INVENTORY: f64 = 14.588954880632265;
const RESOURCE_CENTER: [f64; 2] = [4.8, 0.0];
const RESOURCE_RADIUS: f64 = 1.5;
const DT: f64 = 0.02;
const TOL: f64 = 1e-8;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Snap {
    step: usize,
    area: f64,
    n: f64,
    f: f64,
    a: f64,
    r: f64,
    c: f64,
    waste: f64,
    structural_m: f64,
    free_l: f64,
    bound_b: f64,
    organized_material: f64,
    strict_material: f64,
    observer_viable: bool,
}

fn snap(mesh: &MaterialMesh, step: usize) -> Snap {
    let s = snapshot(mesh);
    Snap {
        step,
        area: mesh.area(),
        n: s.n,
        f: s.f,
        a: s.a,
        r: s.r,
        c: s.c,
        waste: s.waste,
        structural_m: s.structural_m,
        free_l: s.free_l,
        bound_b: s.bound_b,
        organized_material: s.organized_material(),
        strict_material: s.strict_material_equivalent(),
        observer_viable: mesh.observer_viable(),
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Ledger {
    ordinary_n_consumed: f64,
    ordinary_f_consumed: f64,
    ordinary_a_produced: f64,
    diagnostic_n_consumed: f64,
    diagnostic_f_consumed: f64,
    diagnostic_a_produced: f64,
    diagnostic_waste_produced: f64,
    structural_production: f64,
    structural_turnover: f64,
    catalyst_production: f64,
    catalyst_turnover: f64,
    activated_decay: f64,
    membrane_production: f64,
    membrane_turnover: f64,
    a_to_m: f64,
    a_to_l: f64,
    n_delivered: f64,
    f_delivered: f64,
    n_world_loss: f64,
    f_world_loss: f64,
    max_resource_conservation_error: f64,
}

impl Ledger {
    fn absorb(&mut self, led: &ReactionLedger) {
        self.ordinary_n_consumed += led.n_consumed;
        self.ordinary_f_consumed += led.f_consumed;
        self.ordinary_a_produced += led.a_produced;
        self.structural_production += led.m_produced;
        self.structural_turnover += led.m_to_w;
        self.catalyst_production += led.c_produced;
        self.catalyst_turnover += led.c_turned;
        self.activated_decay += led.a_decayed;
        self.membrane_production += led.l_produced;
        self.membrane_turnover += led.bind_extent + led.unbind_extent;
        self.a_to_m += led.a_to_m;
        self.a_to_l += led.a_to_l;
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum Shadow {
    Base,
    SourceCapacityUpperBound,
    CatalystInvestmentOff,
    Combined,
}

impl Shadow {
    fn id(self) -> &'static str {
        match self {
            Self::Base => "BASE",
            Self::SourceCapacityUpperBound => "SOURCE_CAPACITY_UB",
            Self::CatalystInvestmentOff => "CATALYST_INVESTMENT_OFF",
            Self::Combined => "COMBINED",
        }
    }

    fn source_upper_bound(self) -> bool {
        matches!(self, Self::SourceCapacityUpperBound | Self::Combined)
    }

    fn catalyst_investment_off(self) -> bool {
        matches!(self, Self::CatalystInvestmentOff | Self::Combined)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ArmResult {
    pub id: String,
    pub shadow_definition: String,
    pub initial: Snap,
    pub final_state: Snap,
    pub checkpoints: Vec<Snap>,
    pub ledger: Ledger,
    pub n_remaining: f64,
    pub f_remaining: f64,
    pub organized_material_delta: f64,
    pub strict_material_delta: f64,
    pub world_to_organism_closure_residual: f64,
    pub internal_material_closure_residual: f64,
    pub trajectory_hash: String,
    pub final_mesh_hash: String,
    pub viable_at_end: bool,
}

/// Per-step observer evidence for the frozen starvation predicate and its
/// selected A-decay multiplier. This records existing production decisions;
/// it does not provide a production override.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct DecayObservation {
    pub step: usize,
    pub post_uptake_n: f64,
    pub post_uptake_f: f64,
    pub post_source_ub_n: f64,
    pub post_source_ub_f: f64,
    pub starvation_predicate: bool,
    pub selected_multiplier: f64,
    pub declared_k_a_decay: f64,
    pub effective_k_a_decay: f64,
    pub a_decay: f64,
}

#[derive(Debug, Clone, Serialize)]
struct Qualification {
    directive: String,
    starting_head: String,
    m1r0_entry: String,
    base_reproduction: bool,
    source_ub_stoichiometry: String,
    source_ub_material_closure: bool,
    world_organism_closure: bool,
    internal_material_closure: bool,
    observer_only: bool,
    production_biology_changed: bool,
    chemistry_changed: bool,
    d091_changed: bool,
    uptake_changed: bool,
    recycling_implemented: bool,
    parameter_search: bool,
    m1_production_change_authorized: bool,
    m2_authorized: bool,
    recycling_authorized: bool,
    dc_dev_021_authorized: bool,
    next_execution_started: bool,
    capacity_classification: String,
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= TOL
}

pub fn reaction_params() -> ReactionParams {
    let mut p = ReactionParams::conservative_v2();
    p.reserve = chemistry_core::metabolic_reserve::ReserveParams::default();
    assert!(!p.reserve.enable);
    p
}

fn source_capacity_upper_bound(mesh: &mut MaterialMesh, ledger: &mut Ledger) {
    let area = mesh.area().max(1e-6);
    let paired = (mesh.interior.n.min(mesh.interior.f)).max(0.0) * area;
    mesh.interior.n = (mesh.interior.n - paired / area).max(0.0);
    mesh.interior.f = (mesh.interior.f - paired / area).max(0.0);
    mesh.interior.a += paired / area;
    mesh.interior.w += paired / area;
    ledger.diagnostic_n_consumed += paired;
    ledger.diagnostic_f_consumed += paired;
    ledger.diagnostic_a_produced += paired;
    ledger.diagnostic_waste_produced += paired;
}

pub fn run_arm(initial_mesh: &MaterialMesh, shadow: Shadow, mechanics: &MechParams) -> ArmResult {
    run_arm_with_options(initial_mesh, shadow, mechanics, None, false).0
}

pub fn m1r1_entry_state() -> (MaterialMesh, MechParams) {
    m1r0::m1r1_entry_state()
}

/// Run the exact M1-R1 arm, optionally with the one authorized diagnostic
/// decay coefficient and optional per-step starvation provenance capture.
/// `decay_override` is intentionally an example-local observer shadow input;
/// the chemistry-core production function remains unchanged.
pub fn run_arm_with_options(
    initial_mesh: &MaterialMesh,
    shadow: Shadow,
    mechanics: &MechParams,
    decay_override: Option<f64>,
    capture_decay: bool,
) -> (ArmResult, Vec<DecayObservation>) {
    let mut mesh = initial_mesh.clone();
    let mut params = reaction_params();
    if shadow.catalyst_investment_off() {
        // This is the accepted R8-R2 observer concept: suppress only the
        // current step's A→new C investment. Existing C and C turnover stay.
        params.k_c_prod = 0.0;
    }
    if let Some(k_a_decay) = decay_override {
        params.k_a_decay = k_a_decay;
    }
    let transport = TransportParams::default();
    let mut region = FiniteSpatialResourceRegionV1::new(
        RESOURCE_CENTER,
        RESOURCE_RADIUS,
        HIGH_INVENTORY,
        HIGH_INVENTORY,
    );
    let initial = snap(&mesh, 0);
    let mut current = initial;
    let mut ledger = Ledger::default();
    let mut checkpoints = Vec::new();
    let mut trajectory = vec![stable_json_hash(&initial).unwrap()];
    let mut decay_observations = Vec::new();

    for step in 0..HORIZON_STEPS {
        let uptake = region.uptake(&mut mesh, &transport, mechanics.dt);
        ledger.n_delivered += uptake.n_delivered;
        ledger.f_delivered += uptake.f_delivered;
        ledger.n_world_loss += uptake.n_world_loss;
        ledger.f_world_loss += uptake.f_world_loss;
        ledger.max_resource_conservation_error = ledger
            .max_resource_conservation_error
            .max(uptake.conservation_error);
        assert!(uptake.conservation_error <= TOL);

        let post_uptake_n = mesh.interior.n.max(0.0);
        let post_uptake_f = mesh.interior.f.max(0.0);
        if shadow.source_upper_bound() {
            source_capacity_upper_bound(&mut mesh, &mut ledger);
        }
        let starvation_predicate = mesh.interior.n.max(0.0) * mesh.interior.f.max(0.0) < 1e-8;
        let selected_multiplier = if starvation_predicate { 4.0 } else { 1.0 };
        let declared_k_a_decay = params.k_a_decay;
        let effective_k_a_decay = declared_k_a_decay * selected_multiplier;
        let reaction = reactions_step(&mut mesh, &params, mechanics.dt, true, true);
        ledger.absorb(&reaction);
        if capture_decay {
            decay_observations.push(DecayObservation {
                step: step + 1,
                post_uptake_n,
                post_uptake_f,
                post_source_ub_n: mesh.interior.n.max(0.0),
                post_source_ub_f: mesh.interior.f.max(0.0),
                starvation_predicate,
                selected_multiplier,
                declared_k_a_decay,
                effective_k_a_decay,
                a_decay: reaction.a_decayed,
            });
        }
        current = snap(&mesh, step + 1);
        if [1, 120, 240, 360, 480].contains(&current.step) {
            checkpoints.push(current);
        }
        trajectory.push(stable_json_hash(&current).unwrap());
    }

    let strict_delta = current.strict_material - initial.strict_material;
    let boundary = ledger.n_delivered + ledger.f_delivered;
    let internal_residual = (strict_delta - boundary).abs();
    let result = ArmResult {
        id: shadow.id().into(),
        shadow_definition: match shadow {
            Shadow::Base => "exact accepted M1-R0 high-inventory trajectory".into(),
            Shadow::SourceCapacityUpperBound => {
                "ordinary finite uptake plus conservative paired N/F→A upper bound".into()
            }
            Shadow::CatalystInvestmentOff => {
                "exact source law with only new A→C investment deferred".into()
            }
            Shadow::Combined => {
                "source capacity upper bound plus only new A→C investment deferred".into()
            }
        },
        initial,
        final_state: current,
        checkpoints,
        ledger,
        n_remaining: region.n_mass,
        f_remaining: region.f_mass,
        organized_material_delta: current.organized_material - initial.organized_material,
        strict_material_delta: strict_delta,
        world_to_organism_closure_residual: (strict_delta - boundary).abs(),
        internal_material_closure_residual: internal_residual,
        trajectory_hash: stable_json_hash(&trajectory).unwrap(),
        final_mesh_hash: stable_json_hash(&mesh).unwrap(),
        viable_at_end: mesh.observer_viable(),
    };
    assert!(result.world_to_organism_closure_residual <= TOL);
    assert!(result.internal_material_closure_residual <= TOL);
    (result, decay_observations)
}

fn classify(results: &[ArmResult; 4]) -> &'static str {
    let source = results[1].organized_material_delta >= -TOL;
    let allocation = results[2].organized_material_delta >= -TOL;
    let combined = results[3].organized_material_delta >= -TOL;
    match (source, allocation, combined) {
        (true, false, true) => "M1_SOURCE_CAPACITY_SUFFICIENT",
        (false, true, true) => "M1_CATALYST_INVESTMENT_SUFFICIENT",
        (true, true, true) => "M1_SOURCE_AND_ALLOCATION_COUPLED",
        _ => "M1_SOURCE_AND_ALLOCATION_INSUFFICIENT",
    }
}

fn write_json(path: &Path, value: &serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::env::var_os("DCDEV020M1R1_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020m1r1"));
    let (deprived, mechanics) = m1r0::m1r1_entry_state();
    assert!((mechanics.dt - DT).abs() <= 1e-12);

    let results = [
        run_arm(&deprived, Shadow::Base, &mechanics),
        run_arm(&deprived, Shadow::SourceCapacityUpperBound, &mechanics),
        run_arm(&deprived, Shadow::CatalystInvestmentOff, &mechanics),
        run_arm(&deprived, Shadow::Combined, &mechanics),
    ];
    let base = &results[0];
    let base_reproduction = close(base.initial.a, 19.69467805250676)
        && close(base.initial.c, 55.87794642665143)
        && close(base.initial.organized_material, 131.80639622655494)
        && close(base.final_state.a, 14.417573565583695)
        && close(base.final_state.n, 10.493951473277624)
        && close(base.final_state.c, 53.506132055982654)
        && close(base.final_state.organized_material, 122.60541779905688)
        && close(base.organized_material_delta, -9.200978427498057)
        && close(base.ledger.ordinary_a_produced, 0.9457376370749997)
        && close(base.ledger.structural_production, 1.0820395428619851)
        && close(base.ledger.catalyst_production, 2.884382497630763);
    assert!(base_reproduction);
    assert!(results
        .iter()
        .all(|r| r.world_to_organism_closure_residual <= TOL
            && r.internal_material_closure_residual <= TOL));

    let source_stoichiometry = json!({
        "reaction": "N + F -> A + W",
        "coefficients": {"N": -1.0, "F": -1.0, "A": 1.0, "W": 1.0},
        "source": "chemistry-core/src/mesh_contracts.rs::MeshReaction::Activation",
        "diagnostic_rule": "paired=min(N,F), applied only to immediately available internal material"
    });
    let classification = classify(&results);
    let qualification = Qualification {
        directive: "DC-DEV-020-M1-R1-CAPACITY-DECOMP-001".into(),
        starting_head: STARTING_HEAD.into(),
        m1r0_entry: M1R0_ENTRY.into(),
        base_reproduction,
        source_ub_stoichiometry: "PASS: exact ConservativeV2 activation coefficients".into(),
        source_ub_material_closure: results[1].internal_material_closure_residual <= TOL,
        world_organism_closure: results
            .iter()
            .all(|r| r.world_to_organism_closure_residual <= TOL),
        internal_material_closure: results
            .iter()
            .all(|r| r.internal_material_closure_residual <= TOL),
        observer_only: true,
        production_biology_changed: false,
        chemistry_changed: false,
        d091_changed: false,
        uptake_changed: false,
        recycling_implemented: false,
        parameter_search: false,
        m1_production_change_authorized: false,
        m2_authorized: false,
        recycling_authorized: false,
        dc_dev_021_authorized: false,
        next_execution_started: false,
        capacity_classification: classification.into(),
    };
    let protocol = json!({
        "directive": "DC-DEV-020-M1-R1-CAPACITY-DECOMP-001",
        "starting_head": STARTING_HEAD,
        "m1r0_entry": M1R0_ENTRY,
        "selected_production": {"mesh_contract": "ConservativeV2", "reserve_enabled": false},
        "settlement_steps": SETTLEMENT_STEPS,
        "deprivation_steps": DEPRIVATION_STEPS,
        "horizon_steps": HORIZON_STEPS,
        "dt": DT,
        "resource_center": RESOURCE_CENTER,
        "resource_radius": RESOURCE_RADIUS,
        "inventory_n": HIGH_INVENTORY,
        "inventory_f": HIGH_INVENTORY,
        "stoichiometry": source_stoichiometry,
        "prior_constraints": {
            "R5": "d215cfc00ce70517e25fa7c3b51b13d85d9ce521: local N/F source coordinate sufficient on audited manifold",
            "R8-R2": "9fdd292bbd13f62ef9c88d08e8d887f15326d242: catalyst investment is an acute recovery burden",
            "R8-R4": "shared-affinity production did not establish sustained homeostasis",
            "R8-R5-R1": "reversible allocation had local capacity in the prior state; this is not current-baseline proof"
        },
        "arms": [
            {"id": "BASE", "intervention": "none"},
            {"id": "SOURCE_CAPACITY_UB", "intervention": "paired internal N/F to A upper bound"},
            {"id": "CATALYST_INVESTMENT_OFF", "intervention": "k_c_prod=0 observer shadow only"},
            {"id": "COMBINED", "intervention": "SOURCE_CAPACITY_UB plus CATALYST_INVESTMENT_OFF"}
        ],
        "forbidden_changes": ["production chemistry", "chemistry-core", "ConservativeV2", "D-091", "uptake", "transport", "resource quantity", "degradation", "recycling", "M2", "DC-DEV-021"]
    });
    let results_json = json!({
        "directive": "DC-DEV-020-M1-R1-CAPACITY-DECOMP-001",
        "starting_head": STARTING_HEAD,
        "base_reproduction": base_reproduction,
        "source_ub_stoichiometry": source_stoichiometry,
        "arms": results,
        "classification": classification,
        "sustained_decline_warning": results.iter().map(|r| json!({
            "arm": r.id,
            "organized_material_delta_is_not_sustained_homeostasis": r.organized_material_delta >= -TOL,
            "final_c": r.final_state.c,
            "final_a": r.final_state.a,
            "final_structural_m": r.final_state.structural_m,
            "final_free_l_plus_bound_b": r.final_state.free_l + r.final_state.bound_b
        })).collect::<Vec<_>>(),
        "classification_note": "A nonnegative 480-step shadow is an acute capacity result, not sustained M1 homeostasis.",
        "production_biology_changed": false,
        "chemistry_changed": false,
        "d091_changed": false,
        "uptake_changed": false,
        "recycling_implemented": false,
        "parameter_search": false,
        "next_execution_started": false
    });
    let manifest = json!({
        "directive": "DC-DEV-020-M1-R1-CAPACITY-DECOMP-001",
        "starting_head": STARTING_HEAD,
        "artifact_files": ["protocol.json", "results.json", "qualification.json", "artifact_manifest.json"],
        "authoritative_result": "results.json",
        "qualification": "qualification.json",
        "dense_ledgers_committed": false,
        "observer_only": true,
        "next_execution_started": false
    });
    write_json(&out.join("protocol.json"), &protocol)?;
    write_json(&out.join("results.json"), &results_json)?;
    write_json(
        &out.join("qualification.json"),
        &serde_json::to_value(&qualification)?,
    )?;
    write_json(&out.join("artifact_manifest.json"), &manifest)?;
    println!("DCDEV020M1R1_CAPACITY_DECOMPOSITION_COMPLETE");
    println!("classification={classification}");
    println!("{}", out.display());
    Ok(())
}
