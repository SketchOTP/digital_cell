//! DC-DEV-020-R9-R1 exact D-015/D-016 replay under the orthogonal v2 contract.
//!
//! This is an observer-only qualification runner. It reuses the frozen D-015/
//! D-016 founder, reserve parameters, geometry, dt, settlement, deprivation,
//! finite-region, and 480-step delivery semantics; it does not alter biology.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use chemistry_core::mesh_contracts::snapshot;
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use chemistry_core::mesh_reactions::{reactions_step, ReactionParams};
use chemistry_core::mesh_transport::TransportParams;
use chemistry_core::metabolic_reserve::{stamp_reserve_equation, ReserveParams};
use regulatory_core::FiniteSpatialResourceRegionV1;
use serde::Serialize;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

const SETTLEMENT_STEPS: usize = 5_000;
const METABOLIC_STEPS: usize = 480;
const DT: f64 = 0.02;
const CENTER: [f64; 2] = [4.8, 0.0];
const RESOURCE_RADIUS: f64 = 1.5;
const CURRENT_INVENTORY: f64 = 3.0;
const CHALLENGE_INVENTORY: f64 = 14.588954880632265;

#[derive(Debug, Clone, Serialize)]
struct LedgerRow {
    n_delivered: f64,
    f_delivered: f64,
    n_world_loss: f64,
    f_world_loss: f64,
    a_to_r: f64,
    r_to_a: f64,
    r_to_w: f64,
    reserve_rejected_steps: u64,
    strict_material_delta: f64,
    activation_delta: f64,
    organized_retained_delta: f64,
    boundary_material_delta: f64,
    closure_residual: f64,
    observer_viable_at_end: bool,
}

#[derive(Debug, Clone, Serialize)]
struct ReplayRow {
    protocol: String,
    arm: String,
    inventory_n: f64,
    inventory_f: f64,
    settlement_steps: usize,
    deprivation_steps: usize,
    feed_steps: usize,
    dt: f64,
    resource_center: [f64; 2],
    resource_radius: f64,
    reserve_enabled: bool,
    equation_id: String,
    contract_version: String,
    result: LedgerRow,
}

fn founder() -> MaterialMesh {
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
            r: 0.6,
            ..Default::default()
        },
        LumpedChem::default(),
        5.0,
    );
    stamp_reserve_equation(&mut mesh);
    mesh.stamp_conservative_schema();
    mesh
}

fn reaction_params(mesh: &MaterialMesh) -> ReactionParams {
    let mut p = ReactionParams::conservative_v2();
    p.reserve = ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, mesh.area());
    p
}

fn settle(mut mesh: MaterialMesh, mechanics: &MechParams) -> MaterialMesh {
    for _ in 0..SETTLEMENT_STEPS {
        assert!(mechanics_step(&mut mesh, mechanics));
    }
    mesh
}

fn replay(
    protocol: &str,
    arm: &str,
    deprived: &MaterialMesh,
    inventory: f64,
    reactions: bool,
    mechanics: &MechParams,
) -> ReplayRow {
    let mut mesh = deprived.clone();
    let initial = snapshot(&mesh);
    let params = reaction_params(&mesh);
    let transport = TransportParams::default();
    let mut region =
        FiniteSpatialResourceRegionV1::new(CENTER, RESOURCE_RADIUS, inventory, inventory);
    let mut n_delivered = 0.0;
    let mut f_delivered = 0.0;
    let mut n_world_loss = 0.0;
    let mut f_world_loss = 0.0;
    let mut a_to_r = 0.0;
    let mut r_to_a = 0.0;
    let mut r_to_w = 0.0;
    let mut rejected_steps = 0;
    let mut max_resource_error: f64 = 0.0;
    for _ in 0..METABOLIC_STEPS {
        let uptake = region.uptake(&mut mesh, &transport, mechanics.dt);
        n_delivered += uptake.n_delivered;
        f_delivered += uptake.f_delivered;
        n_world_loss += uptake.n_world_loss;
        f_world_loss += uptake.f_world_loss;
        max_resource_error = max_resource_error.max(uptake.conservation_error);
        if reactions {
            let led = reactions_step(&mut mesh, &params, mechanics.dt, true, true);
            a_to_r += led.reserve.a_to_r;
            r_to_a += led.reserve.r_to_a;
            r_to_w += led.reserve.r_to_w;
            rejected_steps += led.reserve.rejected_steps;
        }
    }
    let final_state = snapshot(&mesh);
    let strict_delta =
        final_state.strict_material_equivalent() - initial.strict_material_equivalent();
    // World loss is depletion of the finite environment, not an organism
    // outflux. The organism-side boundary term is delivered N+F only.
    let boundary = n_delivered + f_delivered;
    ReplayRow {
        protocol: protocol.into(),
        arm: arm.into(),
        inventory_n: inventory,
        inventory_f: inventory,
        settlement_steps: SETTLEMENT_STEPS,
        deprivation_steps: METABOLIC_STEPS,
        feed_steps: METABOLIC_STEPS,
        dt: DT,
        resource_center: CENTER,
        resource_radius: RESOURCE_RADIUS,
        reserve_enabled: params.reserve.enable,
        equation_id: mesh.equation_id.clone(),
        contract_version: format!("{:?}", mesh.contract_version),
        result: LedgerRow {
            n_delivered,
            f_delivered,
            n_world_loss,
            f_world_loss,
            a_to_r,
            r_to_a,
            r_to_w,
            reserve_rejected_steps: rejected_steps,
            strict_material_delta: strict_delta,
            activation_delta: final_state.activation_store() - initial.activation_store(),
            organized_retained_delta: final_state.organized_material()
                - initial.organized_material(),
            boundary_material_delta: boundary,
            closure_residual: (strict_delta - boundary).abs().max(max_resource_error),
            observer_viable_at_end: mesh.observer_viable(),
        },
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::env::var_os("DCDEV020R9R1_REPLAY_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020r9r1/exact_replays"));
    fs::create_dir_all(&out)?;
    let mechanics = MechParams::default();
    assert!((mechanics.dt - DT).abs() < 1e-12);
    let settled = settle(founder(), &mechanics);
    let p = reaction_params(&settled);
    let mut deprived = settled.clone();
    for _ in 0..METABOLIC_STEPS {
        let _ = reactions_step(&mut deprived, &p, mechanics.dt, true, true);
    }
    let rows = vec![
        replay("D-015", "no_delivery", &deprived, 0.0, true, &mechanics),
        replay(
            "D-015",
            "current_resource_feed",
            &deprived,
            CURRENT_INVENTORY,
            true,
            &mechanics,
        ),
        replay(
            "D-015",
            "uptake_only",
            &deprived,
            CURRENT_INVENTORY,
            false,
            &mechanics,
        ),
        replay("D-016", "no_delivery", &deprived, 0.0, true, &mechanics),
        replay(
            "D-016",
            "current_resource_reference",
            &deprived,
            CURRENT_INVENTORY,
            true,
            &mechanics,
        ),
        replay(
            "D-016",
            "derived_break_even_resource",
            &deprived,
            CHALLENGE_INVENTORY,
            true,
            &mechanics,
        ),
        replay(
            "D-016",
            "derived_resource_uptake_only",
            &deprived,
            CHALLENGE_INVENTORY,
            false,
            &mechanics,
        ),
    ];
    let manifest = json!({
        "directive": "DC-DEV-020-R9-R1",
        "source_protocols": ["experiments/generated/dcdev015/protocol.json", "experiments/generated/dcdev016/protocol.json"],
        "exact_correspondence": {
            "settlement_steps": SETTLEMENT_STEPS,
            "deprivation_steps": METABOLIC_STEPS,
            "feed_steps": METABOLIC_STEPS,
            "dt": DT,
            "resource_center": CENTER,
            "resource_radius": RESOURCE_RADIUS,
            "d015_current_inventory": CURRENT_INVENTORY,
            "d016_challenge_inventory": CHALLENGE_INVENTORY,
            "reserve_parameters": "ReserveParams::derived(80,40,0.5,0.3,2,0.1,area)",
            "contract": "MeshContractVersion::ConservativeV2",
            "equation_lineage": "autopoietic_material_mesh_metabolic_reserve_v1"
        },
        "rows": rows,
        "historical_artifacts_overwritten": false,
        "observer_only": true
    });
    fs::write(
        out.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    fs::write(out.join("results.json"), serde_json::to_vec_pretty(&rows)?)?;
    println!("DCDEV020R9R1_EXACT_D015_D016_REPLAYS_COMPLETE");
    println!("{}", out.display());
    Ok(())
}
