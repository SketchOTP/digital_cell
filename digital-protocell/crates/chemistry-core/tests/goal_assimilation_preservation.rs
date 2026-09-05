//! Goal-mode preservation checks for the opt-in environmental assimilation state.
//!
//! These tests do not qualify assimilation as the organism-world architecture.
//! They prove only that the already-added optional state has the same basic
//! validity, geometry, legacy-schema, and observer-death boundaries as the
//! other interior material fields.

use chemistry_core::material_mesh::{
    conserve_interior_amount_across_area_change, LumpedChem, MaterialMesh, MeshContractVersion,
    DEFAULT_RHO_S,
};

fn mesh() -> MaterialMesh {
    let mut mesh = MaterialMesh::seed_regular(
        12,
        5.0,
        0.0,
        0.0,
        DEFAULT_RHO_S,
        0.7,
        LumpedChem {
            c: 0.8,
            a: 0.1,
            assimilation_n: 0.31,
            assimilation_f: 0.27,
            ..Default::default()
        },
        LumpedChem::default(),
        5.0,
    );
    mesh.stamp_maturation_coupled_schema();
    mesh
}

fn amount_pair(mesh: &MaterialMesh) -> (f64, f64) {
    (
        mesh.interior.assimilation_n * mesh.area(),
        mesh.interior.assimilation_f * mesh.area(),
    )
}

#[test]
fn geometry_change_preserves_assimilation_amounts() {
    let mut mesh = mesh();
    let before = amount_pair(&mesh);
    let area_before = mesh.area();
    let centroid = mesh.centroid();
    let scale = 1.4_f64.sqrt();
    for vertex in &mut mesh.vertices {
        vertex[0] = centroid[0] + (vertex[0] - centroid[0]) * scale;
        vertex[1] = centroid[1] + (vertex[1] - centroid[1]) * scale;
    }
    let area_after = mesh.area();
    assert!(conserve_interior_amount_across_area_change(
        &mut mesh,
        area_before,
        area_after
    ));
    let after = amount_pair(&mesh);
    assert!((before.0 - after.0).abs() <= 1e-12);
    assert!((before.1 - after.1).abs() <= 1e-12);
}

#[test]
fn legacy_chemistry_defaults_assimilation_to_zero() {
    let chemistry: LumpedChem =
        serde_json::from_str(r#"{"c":1.0,"a":2.0,"n":3.0,"f":4.0,"w":5.0}"#)
            .expect("legacy chemistry remains readable");
    assert_eq!(chemistry.assimilation_n, 0.0);
    assert_eq!(chemistry.assimilation_f, 0.0);
}

#[test]
fn assimilation_fields_share_physical_validity_guard() {
    let mut mesh = mesh();
    assert!(mesh.physical_runtime_valid());
    mesh.interior.assimilation_n = f64::NAN;
    assert!(!mesh.physical_runtime_valid());
    mesh.interior.assimilation_n = 0.31;
    mesh.interior.assimilation_f = f64::INFINITY;
    assert!(!mesh.physical_runtime_valid());
}

#[test]
fn observer_death_semantics_see_assimilated_nutrient_without_new_latch() {
    let mut fed = mesh();
    fed.interior.a = 0.01;
    fed.interior.assimilation_n = 1.0;
    fed.interior.assimilation_f = 1.0;
    assert!(fed.observer_viable());

    let mut starved = fed.clone();
    starved.interior.assimilation_n = 0.0;
    starved.interior.assimilation_f = 0.0;
    assert!(!starved.observer_viable());
    assert_eq!(starved.observer_death_reason(), Some("starvation_collapse"));
}

#[test]
fn historical_contract_remains_distinct_from_opt_in_assimilation_state() {
    let mut mesh = mesh();
    mesh.contract_version = MeshContractVersion::ConservativeV2;
    assert_eq!(mesh.interior.assimilation_n, 0.31);
    assert_eq!(mesh.interior.assimilation_f, 0.27);
    assert!(mesh.physical_runtime_valid());
}
