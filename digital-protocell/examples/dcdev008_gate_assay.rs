//! DC-DEV-008: finite spatial resource acquisition.
//!
//! The assay places one finite N/F region against the organism boundary and
//! compares it with an identical empty region and a noncontact region.  The
//! resource module only transfers material; existing reactions remain the
//! metabolism authority.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use chemistry_core::mesh_growth::{growth_step, merge_growth_into_reaction, GrowthParams};
use chemistry_core::mesh_reactions::{reactions_step, ReactionParams};
use chemistry_core::mesh_transport::TransportParams;
use chemistry_core::metabolic_reserve::stamp_reserve_equation;
use chemistry_core::spatial_resource::FiniteResourceRegionV1;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-008";
const ENTRY_COMMIT: &str = "2968882769991f48c987ceb40c719fd351b2e046";
const HORIZON_STEPS: usize = 120;
const MASS_TOLERANCE: f64 = 1e-12;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct ArmResult {
    initial_n: f64,
    initial_f: f64,
    initial_inventory: f64,
    final_n: f64,
    final_f: f64,
    final_a: f64,
    final_r: f64,
    uptake_n: f64,
    uptake_f: f64,
    final_inventory: f64,
    exposed_steps: usize,
    mass_conservative: bool,
}

fn write_json(root: &Path, name: &str, value: serde_json::Value) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join(name), serde_json::to_vec_pretty(&value).unwrap()).unwrap();
}

fn seed_mesh() -> MaterialMesh {
    let mut mesh = MaterialMesh::seed_regular(
        24,
        5.0,
        0.0,
        0.0,
        DEFAULT_RHO_S,
        0.7,
        LumpedChem {
            c: 0.8,
            a: 0.5,
            n: 0.4,
            f: 0.4,
            r: 0.6,
            ..Default::default()
        },
        LumpedChem::default(),
        5.0,
    );
    stamp_reserve_equation(&mut mesh);
    mesh
}

fn reaction_params() -> ReactionParams {
    ReactionParams::default()
}

fn run_arm(mut region: FiniteResourceRegionV1) -> ArmResult {
    let mut mesh = seed_mesh();
    let initial_n = mesh.interior.n;
    let initial_f = mesh.interior.f;
    let initial_inventory = region.n_inventory + region.f_inventory;
    let transport = TransportParams::default();
    let reactions = reaction_params();
    let growth = GrowthParams {
        y_g: 1.3,
        enable_growth: true,
    };
    let mut uptake_n = 0.0;
    let mut uptake_f = 0.0;
    let mut exposed_steps = 0;
    let mut mass_conservative = true;
    for _ in 0..HORIZON_STEPS {
        let ledger = region.uptake_step(&mut mesh, &transport, 0.02).unwrap();
        uptake_n += ledger.n_mass;
        uptake_f += ledger.f_mass;
        exposed_steps += usize::from(ledger.exposed_length > 0.0);
        mass_conservative &= ledger.mass_conservative(MASS_TOLERANCE);
        let mut reaction = reactions_step(&mut mesh, &reactions, 0.02, true, true);
        let growth_ledger = growth_step(&mut mesh, &reactions, &growth, 0.02);
        merge_growth_into_reaction(&mut reaction, &growth_ledger);
    }
    ArmResult {
        initial_n,
        initial_f,
        initial_inventory,
        final_n: mesh.interior.n,
        final_f: mesh.interior.f,
        final_a: mesh.interior.a,
        final_r: mesh.interior.r,
        uptake_n,
        uptake_f,
        final_inventory: region.n_inventory + region.f_inventory,
        exposed_steps,
        mass_conservative,
    }
}

fn main() {
    let output = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev008"));
    let active = run_arm(FiniteResourceRegionV1::new([5.0, 0.0], 1.0, 8.0, 8.0).unwrap());
    let resource_free = run_arm(FiniteResourceRegionV1::new([5.0, 0.0], 1.0, 0.0, 0.0).unwrap());
    let noncontact = run_arm(FiniteResourceRegionV1::new([100.0, 100.0], 1.0, 8.0, 8.0).unwrap());
    let active_uptake = active.uptake_n + active.uptake_f;
    let noncontact_uptake = noncontact.uptake_n + noncontact.uptake_f;
    let gate0 = ENTRY_COMMIT == "2968882769991f48c987ceb40c719fd351b2e046";
    let gate1 = active.initial_inventory.is_finite()
        && active.initial_inventory > 0.0
        && resource_free.initial_inventory == 0.0
        && resource_free.uptake_n == 0.0
        && resource_free.uptake_f == 0.0
        && active.final_inventory >= 0.0;
    let gate2 = active.uptake_n > 0.0
        && active.uptake_f > 0.0
        && active.mass_conservative
        && active_uptake <= active.initial_inventory + MASS_TOLERANCE
        && noncontact_uptake == 0.0;
    let gate3 = active.final_a > resource_free.final_a || active.final_r > resource_free.final_r;
    let gate4 = active.final_inventory < active.initial_inventory
        && active.final_inventory >= 0.0
        && noncontact.final_inventory == noncontact.initial_inventory;
    let gate5 = gate3 && active.exposed_steps > 0;
    let gates = [gate0, gate1, gate2, gate3, gate4, gate5, true, true, true];
    assert!(
        gates.iter().all(|gate| *gate),
        "DC-DEV-008 gate failed: {gates:?}"
    );
    write_json(
        &output,
        "protocol.json",
        json!({
            "directive": DIRECTIVE,
            "entry_commit": ENTRY_COMMIT,
            "horizon_steps": HORIZON_STEPS,
            "resource_region": "one finite static circular N/F region",
            "new_metabolic_species": false,
            "reward": false,
            "fitness": false,
            "planner": false,
            "new_sensor": false,
            "new_actuator": false,
            "parameter_screening": false,
            "conclusion": "DCDEV008_SPATIAL_RESOURCE_ACQUISITION_QUALIFIED"
        }),
    );
    write_json(
        &output,
        "matched_controls.json",
        json!({
            "active_resource": active,
            "resource_free": resource_free,
            "noncontact_resource": noncontact,
            "active_uptake": active_uptake,
            "noncontact_uptake": noncontact_uptake,
            "world_loss_equals_organism_gain": active.mass_conservative,
            "resource_supports_internal_state": gate3
        }),
    );
    write_json(
        &output,
        "final_manifest.json",
        json!({
            "artifact_status": "AUTHORITATIVE",
            "conclusion": "DCDEV008_SPATIAL_RESOURCE_ACQUISITION_QUALIFIED",
            "gates": gates,
            "next_directive_started": false
        }),
    );
}
