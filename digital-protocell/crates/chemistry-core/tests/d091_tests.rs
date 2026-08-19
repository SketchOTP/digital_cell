//! D-091 metabolic reserve unit tests.

use chemistry_core::material_mesh::{
    LumpedChem, MaterialMesh, MeshContractVersion, DEFAULT_RHO_S,
    EQUATION_VERSION_MATERIAL_MESH,
};
use chemistry_core::mesh_growth::{growth_step, GrowthParams};
use chemistry_core::mesh_reactions::{q_catalyst, ReactionParams};
use chemistry_core::metabolic_reserve::{
    j_release, j_r_loss, j_store, reserve_metab_step, reserve_schema_load_ok, stamp_reserve_equation,
    ReserveParams, EQUATION_VERSION_METABOLIC_RESERVE,
};

fn tiny_mesh() -> MaterialMesh {
    MaterialMesh::seed_regular(
        12,
        5.0,
        0.0,
        0.0,
        DEFAULT_RHO_S,
        0.7,
        LumpedChem {
            c: 0.8,
            a: 1.0,
            n: 0.4,
            f: 0.4,
            w: 0.0,
            ..Default::default()
        },
        LumpedChem::default(),
        2.0,
    )
}

fn reserve_params() -> ReserveParams {
    ReserveParams {
        enable: true,
        k_store: 0.2,
        k_release: 0.1,
        k_r_loss: 0.01,
        k_store_half: 0.4,
        k_low: 0.2,
        k_r: 0.2,
        k_growth: 0.3,
        r_max: 2.0,
        store_horizon_mult: 4.0,
    }
}

#[test]
fn schema_ids() {
    assert_eq!(
        EQUATION_VERSION_METABOLIC_RESERVE,
        "autopoietic_material_mesh_metabolic_reserve_v1"
    );
}

#[test]
fn old_snapshot_rejected_for_reserve() {
    let mesh = tiny_mesh();
    assert_eq!(mesh.equation_id, EQUATION_VERSION_MATERIAL_MESH);
    let p = reserve_params();
    assert!(!reserve_schema_load_ok(&mesh, &p));
    let mut stamped = mesh;
    stamp_reserve_equation(&mut stamped);
    assert!(reserve_schema_load_ok(&stamped, &p));
}

#[test]
fn store_conserves_a_to_r() {
    let mut mesh = tiny_mesh();
    stamp_reserve_equation(&mut mesh);
    let mut react = ReactionParams::default();
    react.reserve = reserve_params();
    let area = mesh.area();
    let a0 = mesh.interior.a * area;
    let r0 = mesh.interior.r * area;
    let led = reserve_metab_step(&mut mesh, &react, 0.05);
    let a1 = mesh.interior.a * area;
    let r1 = mesh.interior.r * area;
    assert!(led.a_to_r > 0.0);
    assert!(((a0 - a1) - (r1 - r0)).abs() < 1e-9 * (1.0 + a0));
}

#[test]
fn release_stronger_when_a_low() {
    let p = reserve_params();
    let qc = 0.5;
    assert!(j_release(0.01, 0.5, qc, &p) > j_release(1.0, 0.5, qc, &p));
}

#[test]
fn loss_to_w() {
    let p = reserve_params();
    assert!(j_r_loss(1.0, &p) > 0.0);
    assert!(j_store(1.0, 0.0, q_catalyst(0.8, 0.3), &p) > 0.0);
}

#[test]
fn no_growth_without_r() {
    let mut mesh = tiny_mesh();
    stamp_reserve_equation(&mut mesh);
    mesh.interior.r = 0.0;
    mesh.interior.a = 2.0;
    let mut react = ReactionParams::default();
    react.reserve = reserve_params();
    let g = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let m0 = mesh.total_structural_mass();
    let _ = growth_step(&mut mesh, &react, &g, 0.2);
    assert!((mesh.total_structural_mass() - m0).abs() < 1e-12);
}

#[test]
fn reserve_disabled_matches_default_path_identity() {
    let p = ReserveParams::default();
    assert!(!p.enable);
    assert_eq!(p.k_store, 0.0);
}

#[test]
fn conservative_contract_composes_with_d091_lineage_and_runs_fluxes() {
    let mut mesh = tiny_mesh();
    mesh.interior.r = 0.2;
    stamp_reserve_equation(&mut mesh);
    mesh.stamp_conservative_schema();
    let equation = mesh.equation_id.clone();
    let mut react = ReactionParams::conservative_v2();
    react.reserve = reserve_params();

    assert_eq!(mesh.contract_version, MeshContractVersion::ConservativeV2);
    assert_eq!(mesh.equation_id, EQUATION_VERSION_METABOLIC_RESERVE);
    assert!(reserve_schema_load_ok(&mesh, &react.reserve));

    let mut a_to_r = 0.0;
    let mut r_to_a = 0.0;
    let mut r_to_w = 0.0;
    for _ in 0..200 {
        let led = reserve_metab_step(&mut mesh, &react, 0.02);
        a_to_r += led.a_to_r;
        r_to_a += led.r_to_a;
        r_to_w += led.r_to_w;
        assert_eq!(led.rejected_steps, 0);
    }
    assert!(a_to_r > 0.0);
    assert!(r_to_a > 0.0);
    assert!(r_to_w > 0.0);
    assert_eq!(mesh.equation_id, equation);
    assert_eq!(mesh.contract_version, MeshContractVersion::ConservativeV2);
}
