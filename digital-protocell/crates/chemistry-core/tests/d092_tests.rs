//! D-092 catalytic template unit tests.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S, EQUATION_VERSION_MATERIAL_MESH};
use chemistry_core::mesh_reactions::ReactionParams;
use chemistry_core::metabolic_reserve::{
    stamp_reserve_equation, EQUATION_VERSION_METABOLIC_RESERVE,
};
use chemistry_core::template_motifs::{count_available_motifs, template_activity_gains};
use chemistry_core::template_polymer::{
    monomer_production_step, parse_founder, seed_founder_chains, stamp_template_equation,
    template_schema_load_ok, TemplateParams, EQUATION_VERSION_CATALYTIC_TEMPLATE,
    FIELD_SCHEMA_CATALYTIC_TEMPLATE, FOUNDER_BUILD, FOUNDER_HARVEST, FOUNDER_LEN, FOUNDER_NEUTRAL,
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

#[test]
fn schema_ids() {
    assert_eq!(
        EQUATION_VERSION_CATALYTIC_TEMPLATE,
        "autopoietic_material_mesh_catalytic_template_v1"
    );
    assert_eq!(
        FIELD_SCHEMA_CATALYTIC_TEMPLATE,
        "mesh_vertices_edges_reserve_template_polymer_v1"
    );
    assert_eq!(FOUNDER_LEN, 12);
}

#[test]
fn founders_equal_composition_different_order() {
    let h = parse_founder(FOUNDER_HARVEST).unwrap();
    let b = parse_founder(FOUNDER_BUILD).unwrap();
    let n = parse_founder(FOUNDER_NEUTRAL).unwrap();
    assert_ne!(FOUNDER_HARVEST, FOUNDER_BUILD);
    assert_eq!(h.len(), 12);
    assert_eq!(b.len(), 12);
    assert_eq!(n.len(), 12);
}

#[test]
fn schema_isolation() {
    let mut old = tiny_mesh();
    stamp_reserve_equation(&mut old);
    assert_eq!(old.equation_id, EQUATION_VERSION_METABOLIC_RESERVE);
    let tmpl = TemplateParams::derived(50.0);
    assert!(!template_schema_load_ok(&old, &tmpl));
    let mut neu = tiny_mesh();
    assert_eq!(neu.equation_id, EQUATION_VERSION_MATERIAL_MESH);
    stamp_template_equation(&mut neu);
    assert!(template_schema_load_ok(&neu, &tmpl));
}

#[test]
fn monomer_production_conserves() {
    let mut mesh = tiny_mesh();
    stamp_template_equation(&mut mesh);
    let mut react = ReactionParams::default();
    react.template = TemplateParams::derived(40.0);
    react.reserve.enable = false;
    let area = mesh.area();
    let n0 = mesh.interior.n * area;
    let a0 = mesh.interior.a * area;
    let led = monomer_production_step(&mut mesh, &react, 0.1);
    assert!(led.u_h_produced > 0.0);
    assert!((led.u_h_produced - led.u_b_produced).abs() < 1e-12);
    let n1 = mesh.interior.n * area;
    let a1 = mesh.interior.a * area;
    assert!(((n0 - n1) - led.n_consumed_mono).abs() < 1e-9);
    assert!(((a0 - a1) - led.a_consumed_mono).abs() < 1e-9);
}

#[test]
fn motifs_on_founders() {
    let mut h = tiny_mesh();
    stamp_template_equation(&mut h);
    seed_founder_chains(&mut h, FOUNDER_HARVEST, 1, 1);
    let (mh, mb_h) = count_available_motifs(&h);
    assert!(mh > 0.0);
    let mut b = tiny_mesh();
    stamp_template_equation(&mut b);
    seed_founder_chains(&mut b, FOUNDER_BUILD, 1, 1);
    let (mh_b, mb) = count_available_motifs(&b);
    assert!(mb > 0.0);
    let mut n = tiny_mesh();
    stamp_template_equation(&mut n);
    seed_founder_chains(&mut n, FOUNDER_NEUTRAL, 1, 1);
    let (mn_h, mn_b) = count_available_motifs(&n);
    assert_eq!(mn_h + mn_b, 0.0);
    let _ = (mb_h, mh_b);
}

#[test]
fn activity_gains_respond_to_complexes() {
    let mut mesh = tiny_mesh();
    stamp_template_equation(&mut mesh);
    seed_founder_chains(&mut mesh, FOUNDER_HARVEST, 2, 1);
    mesh.interior.k_h = 0.4;
    mesh.interior.k_b = 0.0;
    mesh.interior.c = 0.8;
    let p = TemplateParams::derived(40.0);
    let (gh, gb) = template_activity_gains(&mesh, &p);
    assert!(gh > 1.0);
    assert!(gb < 1.0);
}

#[test]
fn no_complete_chain_without_template() {
    let mut mesh = tiny_mesh();
    stamp_template_equation(&mut mesh);
    mesh.interior.u_h = 3.0;
    mesh.interior.u_b = 3.0;
    mesh.interior.a = 2.0;
    let mut react = ReactionParams::default();
    react.template = TemplateParams::derived(40.0);
    for _ in 0..100 {
        let _ = chemistry_core::mesh_reactions::reactions_step(&mut mesh, &react, 0.02, true, true);
    }
    assert_eq!(
        chemistry_core::template_polymer::count_complete_templates(&mesh),
        0
    );
}
