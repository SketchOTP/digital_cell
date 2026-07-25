//! D-093 template network unit tests.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use chemistry_core::mesh_reactions::ReactionParams;
use chemistry_core::metabolic_reserve::{stamp_reserve_equation, ReserveParams};
use chemistry_core::template_network::{
    count_pair_channels, derive_k_site, network_schema_load_ok, stamp_network_equation,
    NetworkParams, EQUATION_VERSION_TEMPLATE_NETWORK, FIELD_SCHEMA_TEMPLATE_NETWORK, RHO_NETWORK,
};
use chemistry_core::template_network_binding::{
    network_binding_step, occupancy_invariant_ok, response_vector, sum_channel_masses,
};
use chemistry_core::template_network_founders::enumerate_topology_class;
use chemistry_core::template_polymer::{
    seed_founder_chains, TemplateParams, FOUNDER_LEN,
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
            c: 0.9,
            a: 0.5,
            n: 0.8,
            f: 0.8,
            r: 0.4,
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
        EQUATION_VERSION_TEMPLATE_NETWORK,
        "autopoietic_material_mesh_template_network_v1"
    );
    assert_eq!(
        FIELD_SCHEMA_TEMPLATE_NETWORK,
        "mesh_vertices_edges_reserve_template_network_v1"
    );
    assert!((RHO_NETWORK - 1.5).abs() < 1e-12);
    assert_eq!(FOUNDER_LEN, 12);
}

#[test]
fn topology_class_equal_channels() {
    let class = enumerate_topology_class();
    assert!(
        class.len() >= 3,
        "need ≥3 non-equivalent class members, got {}",
        class.len()
    );
    for s in &class {
        let (hh, hb, bh, bb) = count_pair_channels(s);
        assert_eq!(hh, hb);
        assert_eq!(hb, bh);
        assert_eq!(bh, bb);
        assert_eq!(hh, 3, "circular L=12 should yield 3 of each pair type");
        assert_eq!(s.len(), 12);
    }
}

#[test]
fn schema_isolation() {
    let mut old = tiny_mesh();
    stamp_reserve_equation(&mut old);
    let net = NetworkParams::derived(&ReserveParams::default(), 40.0, 0.2, 0.1);
    assert!(!network_schema_load_ok(&old, &net));
    let mut neu = tiny_mesh();
    stamp_network_equation(&mut neu);
    assert!(network_schema_load_ok(&neu, &net));
}

#[test]
fn binding_conserves_catalyst_and_occupancy() {
    let mut mesh = tiny_mesh();
    stamp_network_equation(&mut mesh);
    seed_founder_chains(&mut mesh, "HBHBHBHBHBHB", 2, 1);
    let area = mesh.area();
    let k_site = derive_k_site(0.9, area, 2);
    let reserve = ReserveParams::default();
    let mut net = NetworkParams::derived(&reserve, 30.0, 0.2, k_site);
    net.enable = true;
    let mut react = ReactionParams::default();
    react.template = TemplateParams::derived(40.0);
    react.network = net;
    let c0 = mesh.interior.c;
    for _ in 0..40 {
        let _ = network_binding_step(&mut mesh, &react, 0.05);
    }
    assert!((mesh.interior.c - c0).abs() < 1e-12);
    assert!(occupancy_invariant_ok(&mesh, k_site));
    let (hh, hb, bh, bb) = sum_channel_masses(&mesh);
    assert!(hh + hb + bh + bb > 0.0);
    let v = response_vector(&mesh);
    assert!(v.iter().any(|x| *x > 0.0));
}

#[test]
fn binding_off_releases_complexes() {
    let mut mesh = tiny_mesh();
    stamp_network_equation(&mut mesh);
    seed_founder_chains(&mut mesh, "HHBBHHBBHHBB", 1, 1);
    let mut react = ReactionParams::default();
    react.template = TemplateParams::derived(40.0);
    react.network = NetworkParams::derived(&ReserveParams::default(), 30.0, 0.2, 0.05);
    for _ in 0..20 {
        let _ = network_binding_step(&mut mesh, &react, 0.05);
    }
    let bound_before = sum_channel_masses(&mesh);
    assert!(bound_before.0 + bound_before.1 + bound_before.2 + bound_before.3 > 0.0);
    react.network.k_on = 0.0;
    let _ = network_binding_step(&mut mesh, &react, 0.05);
    let bound_after = sum_channel_masses(&mesh);
    assert!(bound_after.0 + bound_after.1 + bound_after.2 + bound_after.3 < 1e-12);
}
