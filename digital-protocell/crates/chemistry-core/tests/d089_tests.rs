//! D-089 unit checks: copying conservation, mutation-off exactness, tradeoff bounds.

use chemistry_core::catalyst_composition::{
    composition_z, copy_production_fluxes, derive_mutation_rate, g_build, g_harvest,
    set_composition_from_z, SIGMA_TRADEOFF,
};
use chemistry_core::material_mesh::LumpedChem;

#[test]
fn copy_fluxes_conserve_total() {
    for mu in [0.0, 1e-5, 1e-2, 0.5] {
        for (ch, cb, jc) in [(1.0, 0.0, 0.7), (0.0, 2.0, 0.4), (0.3, 0.7, 1.2)] {
            let (jh, jb) = copy_production_fluxes(jc, ch, cb, mu);
            assert!((jh + jb - jc).abs() < 1e-12, "mu={mu} jh+jb={}", jh + jb);
        }
    }
}

#[test]
fn empty_catalyst_no_copy() {
    let (jh, jb) = copy_production_fluxes(1.0, 0.0, 0.0, 0.01);
    assert_eq!(jh + jb, 0.0);
}

#[test]
fn mutation_off_preserves_type() {
    let (jh, jb) = copy_production_fluxes(1.0, 1.0, 0.0, 0.0);
    assert!((jh - 1.0).abs() < 1e-12);
    assert!(jb.abs() < 1e-12);
}

#[test]
fn tradeoff_bounds() {
    for z in [-1.0, -0.6, 0.0, 0.6, 1.0] {
        let gh = g_harvest(z, SIGMA_TRADEOFF);
        let gb = g_build(z, SIGMA_TRADEOFF);
        assert!((0.85..=1.15).contains(&gh));
        assert!((0.85..=1.15).contains(&gb));
        assert!((gh + gb - 2.0).abs() < 1e-12);
    }
}

#[test]
fn mu_derivation_clamped() {
    assert!((derive_mutation_rate(1.0) - 1e-2).abs() < 1e-15);
    assert!((derive_mutation_rate(1e6) - 1e-5).abs() < 1e-15);
    assert!((derive_mutation_rate(400.0) - 0.005).abs() < 1e-12);
}

#[test]
fn composition_z_from_set() {
    let mut chem = LumpedChem {
        c: 1.0,
        ..Default::default()
    };
    set_composition_from_z(&mut chem, 0.6);
    let z = composition_z(chem.c_h, chem.c_b);
    assert!((z - 0.6).abs() < 1e-12);
    assert!((chem.c_h + chem.c_b - 1.0).abs() < 1e-12);
}
