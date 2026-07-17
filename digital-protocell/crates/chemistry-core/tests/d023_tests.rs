//! D-023 membrane-precursor interface assembly tests (Gate 0 schema + Gate 1 chemistry).

use chemistry_core::config::{
    EquationVersion, SimParams, EIGHT_FIELD_COUNT, GRID_HEIGHT, GRID_WIDTH,
    MEMBRANE_TRANSPORT_SCHEMA_VERSION_V1, PRECURSOR_SCHEMA_VERSION_V1,
};
use chemistry_core::fields::{interior_weight, FieldBuffers, FIELD_NAMES_V6};
use chemistry_core::grid::Grid;
use chemistry_core::membrane::{
    evolve_precursor_assembly, membrane_catalyst_saturation, precursor_assembly_rate,
    precursor_decay_rate, precursor_synthesis_rate,
};
use chemistry_core::operators::total_mass;
use chemistry_core::reactions::interface_weight;
use chemistry_core::snapshot::FieldSchemaVersion;
use chemistry_core::{build_candidate_identity, Simulation};

fn v6_params() -> SimParams {
    let mut p = SimParams::default();
    p.equation_version = EquationVersion::MembraneMetabolismV6PrecursorAssembly;
    p.k_precursor = 0.2;
    p.k_assembly = 0.3;
    p
}

// === Gate 0: schema and preservation ===

#[test]
fn test_eight_field_allocation_and_swap() {
    let n = GRID_WIDTH * GRID_HEIGHT;
    let mut f = FieldBuffers::new(n);
    assert_eq!(f.precursor.len(), n);
    assert_eq!(f.precursor_next.len(), n);
    assert_eq!(EIGHT_FIELD_COUNT, 8);
    assert_eq!(FIELD_NAMES_V6.len(), 8);
    assert!(FIELD_NAMES_V6.contains(&"precursor"));

    // copy_current_to_next mirrors precursor.
    f.precursor[10] = 0.7;
    f.copy_current_to_next();
    assert_eq!(f.precursor_next[10], 0.7);

    // swap exchanges current/next for precursor.
    f.precursor[10] = 1.0;
    f.precursor_next[10] = 2.0;
    f.swap();
    assert_eq!(f.precursor[10], 2.0);
    assert_eq!(f.precursor_next[10], 1.0);
}

#[test]
fn test_v6_schema_versions() {
    let v6 = EquationVersion::MembraneMetabolismV6PrecursorAssembly;
    assert!(v6.is_precursor_assembly());
    assert!(v6.is_eight_field());
    assert_eq!(v6.precursor_schema_version(), PRECURSOR_SCHEMA_VERSION_V1);
    // Frozen transport: χ_M = 0 → diffusion-only M transport (schema v1).
    assert_eq!(
        v6.membrane_transport_schema_version(),
        MEMBRANE_TRANSPORT_SCHEMA_VERSION_V1
    );
    // Earlier versions are not eight-field.
    assert!(!EquationVersion::MembraneMetabolismV5InterfaceAffinity.is_eight_field());
}

#[test]
fn test_eight_field_snapshot_roundtrip() {
    let mut sim = Simulation::new(v6_params());
    sim.fields.precursor[500] = 0.42;
    sim.fields.membrane[500] = 0.11;
    let snap = sim.snapshot();
    assert_eq!(snap.field_schema_version, FieldSchemaVersion::EightFieldV1);
    assert!(snap.validate().is_ok());
    assert!((snap.fields.precursor().expect("P payload")[500] - 0.42).abs() < 1e-15);

    let mut restored = FieldBuffers::new(GRID_WIDTH * GRID_HEIGHT);
    snap.try_restore_fields(&mut restored).expect("restore");
    assert!((restored.precursor[500] - 0.42).abs() < 1e-15);
    assert!((restored.membrane[500] - 0.11).abs() < 1e-15);
}

#[test]
fn test_seven_field_snapshot_cannot_resume_as_v6() {
    // A seven-field (v5) snapshot must be rejected when the destination is v6.
    let mut v5 = SimParams::default();
    v5.equation_version = EquationVersion::MembraneMetabolismV5InterfaceAffinity;
    let sim5 = Simulation::new(v5);
    let snap5 = sim5.snapshot();
    assert_eq!(snap5.field_schema_version, FieldSchemaVersion::SevenFieldV1);
    assert!(
        snap5.can_resume_into(&v6_params()).is_err(),
        "seven-field snapshot must not resume as v6"
    );

    // An eight-field snapshot must be rejected when the destination is v5.
    let sim6 = Simulation::new(v6_params());
    let snap6 = sim6.snapshot();
    let mut v5b = SimParams::default();
    v5b.equation_version = EquationVersion::MembraneMetabolismV5InterfaceAffinity;
    assert!(
        snap6.can_resume_into(&v5b).is_err(),
        "eight-field snapshot must not resume as v5"
    );
    // ...but is accepted by its own version.
    assert!(snap6.can_resume_into(&v6_params()).is_ok());
    assert!(snap6.validate().is_ok());
}

#[test]
fn test_candidate_hash_includes_precursor_params() {
    let base = build_candidate_identity(v6_params(), "t", Some("v6"), None, "v6", None, None);

    let mut p_ka = v6_params();
    p_ka.k_assembly *= 2.0;
    let ka = build_candidate_identity(p_ka, "t", Some("v6"), None, "v6", None, None);
    assert_ne!(base.candidate_hash, ka.candidate_hash, "k_assembly ignored");

    let mut p_kp = v6_params();
    p_kp.k_precursor *= 2.0;
    let kp = build_candidate_identity(p_kp, "t", Some("v6"), None, "v6", None, None);
    assert_ne!(base.candidate_hash, kp.candidate_hash, "k_precursor ignored");

    let mut p_dp = v6_params();
    p_dp.d_p *= 2.0;
    let dp = build_candidate_identity(p_dp, "t", Some("v6"), None, "v6", None, None);
    assert_ne!(base.candidate_hash, dp.candidate_hash, "d_p ignored");
}

// === Gate 1: conservation and causal chemistry ===

#[test]
fn test_precursor_requires_activated_and_catalyst() {
    let p = v6_params();
    let phi = 0.8;
    // Full drivers → positive synthesis.
    assert!(precursor_synthesis_rate(phi, 0.4, 0.5, &p) > 0.0);
    // No A → no P.
    assert_eq!(precursor_synthesis_rate(phi, 0.4, 0.0, &p), 0.0);
    // No C → q(C) = 0 → no P.
    assert_eq!(membrane_catalyst_saturation(0.0, &p), 0.0);
    assert_eq!(precursor_synthesis_rate(phi, 0.0, 0.5, &p), 0.0);
}

#[test]
fn test_membrane_requires_precursor() {
    let p = v6_params();
    // Assembly needs P and interface presence.
    assert!(precursor_assembly_rate(0.5, 0.3, 0.0, &p) > 0.0);
    assert_eq!(precursor_assembly_rate(0.5, 0.0, 0.0, &p), 0.0);
    // Interface weight zero away from interface (φ≈1 interior, φ≈0 exterior).
    assert!(interface_weight(0.5) > 0.0);
}

#[test]
fn test_direct_a_to_m_disabled_no_m_without_precursor() {
    // With P = 0 everywhere, no membrane may be synthesized (only P → M path exists).
    let grid = Grid::new();
    let n = GRID_WIDTH * GRID_HEIGHT;
    let p = v6_params();
    let (phi, catalyst, activated, precursor, membrane) = seeded_state(&grid, n, 0.0, 0.2);
    let mut scratch = vec![0.0; n];
    let mut diff = vec![0.0; n];
    let mut m_next = membrane.clone();
    let mut a_next = activated.clone();
    let mut p_next = precursor.clone();
    let mut w_next = vec![0.0; n];

    let totals = evolve_precursor_assembly(
        &grid, &phi, &catalyst, &activated, &precursor, &membrane, &p, 0.01, &mut scratch,
        &mut diff, &mut m_next, &mut a_next, &mut p_next, &mut w_next,
    );
    // Precursor is produced from A, but no assembly can happen (P starts at 0 this step).
    assert!(totals.synthesis_delta > 0.0, "A→P must be active");
    assert_eq!(totals.assembly_delta, 0.0, "no P → no assembly");
    assert!(totals.membrane_reaction_delta <= 0.0, "M cannot grow without P");
}

#[test]
fn test_conservative_precursor_assembly_and_turnover() {
    let grid = Grid::new();
    let n = GRID_WIDTH * GRID_HEIGHT;
    let p = v6_params();
    // Seed with both A and P present so all reactions are active.
    let (phi, catalyst, activated, precursor, membrane) = seeded_state(&grid, n, 0.3, 0.2);
    let mut scratch = vec![0.0; n];
    let mut diff = vec![0.0; n];
    let mut m_next = membrane.clone();
    let mut a_next = activated.clone();
    let mut p_next = precursor.clone();
    let mut w_next = vec![0.0; n];

    let a0 = total_mass(&grid, &activated);
    let p0 = total_mass(&grid, &precursor);
    let m0 = total_mass(&grid, &membrane);

    let totals = evolve_precursor_assembly(
        &grid, &phi, &catalyst, &activated, &precursor, &membrane, &p, 0.01, &mut scratch,
        &mut diff, &mut m_next, &mut a_next, &mut p_next, &mut w_next,
    );

    // All three reactions active.
    assert!(totals.synthesis_delta > 0.0);
    assert!(totals.assembly_delta > 0.0);
    assert!(totals.precursor_decay_delta > 0.0);

    // Reaction deltas close exactly (materially conservative, diffusion aside).
    let reaction_sum = totals.activated_reaction_delta
        + totals.precursor_reaction_delta
        + totals.membrane_reaction_delta
        + totals.waste_reaction_delta;
    assert!(reaction_sum.abs() < 1e-12, "reaction sum={reaction_sum}");

    // Turnover + loss go to W.
    assert!(
        (totals.waste_reaction_delta
            - (totals.precursor_decay_delta + totals.membrane_loss_delta))
            .abs()
            < 1e-15
    );

    // Total A+P+M+W conserved up to M diffusion (which conserves over the dish).
    let a1 = total_mass(&grid, &a_next);
    let p1 = total_mass(&grid, &p_next);
    let m1 = total_mass(&grid, &m_next);
    let w1 = total_mass(&grid, &w_next);
    let net = (a1 + p1 + m1 + w1) - (a0 + p0 + m0);
    assert!(net.abs() < 1e-9, "material not conserved: net={net}");
}

#[test]
fn test_precursor_transport_conserves_mass() {
    use chemistry_core::operators::diffuse_constant;
    let grid = Grid::new();
    let n = GRID_WIDTH * GRID_HEIGHT;
    let mut precursor = vec![0.0; n];
    for idx in 0..n {
        if !grid.in_dish(idx) {
            continue;
        }
        let x = (idx % GRID_WIDTH) as f64;
        precursor[idx] = 0.3 + 0.1 * (x * 0.05).sin();
    }
    let mut scratch = vec![0.0; n];
    let mut out = vec![0.0; n];
    diffuse_constant(&grid, &precursor, v6_params().d_p, &mut scratch, &mut out);
    let sum: f64 = out
        .iter()
        .enumerate()
        .filter(|(i, _)| grid.in_dish(*i))
        .map(|(_, v)| *v)
        .sum();
    assert!(sum.abs() < 1e-9, "P transport mass rate sum={sum}");
}

#[test]
fn test_v6_short_sim_bounded() {
    let mut sim = Simulation::new(v6_params());
    for _ in 0..40 {
        if !sim.step() {
            break;
        }
    }
    assert!(sim.substep > 0);
    assert!(sim
        .fields
        .precursor
        .iter()
        .all(|&v| v.is_finite() && v >= -1e-9));
    assert!(sim
        .fields
        .membrane
        .iter()
        .all(|&v| v.is_finite() && v >= -1e-9 && v <= sim.params.m_max));
}

#[test]
fn test_precursor_decay_proportional() {
    let p = v6_params();
    assert_eq!(precursor_decay_rate(0.0, &p), 0.0);
    let a = precursor_decay_rate(0.5, &p);
    let b = precursor_decay_rate(1.0, &p);
    assert!((b - 2.0 * a).abs() < 1e-15);
}

/// Circular φ interior with uniform C, A(=activated_level), P(=precursor_level), M.
fn seeded_state(
    grid: &Grid,
    n: usize,
    precursor_level: f64,
    membrane_level: f64,
) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut phi = vec![0.0; n];
    let mut catalyst = vec![0.0; n];
    let mut activated = vec![0.0; n];
    let mut precursor = vec![0.0; n];
    let mut membrane = vec![0.0; n];
    let cx = (GRID_WIDTH / 2) as f64;
    let cy = (GRID_HEIGHT / 2) as f64;
    for idx in 0..n {
        if !grid.in_dish(idx) {
            continue;
        }
        let x = (idx % GRID_WIDTH) as f64;
        let y = (idx / GRID_WIDTH) as f64;
        let r = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
        let f = 0.5 * (1.0 - ((r - 22.0) / 2.0).tanh());
        phi[idx] = f;
        catalyst[idx] = 0.4 * interior_weight(f);
        activated[idx] = 0.3 * interior_weight(f);
        precursor[idx] = precursor_level * interior_weight(f);
        membrane[idx] = membrane_level * interface_weight(f);
    }
    (phi, catalyst, activated, precursor, membrane)
}
