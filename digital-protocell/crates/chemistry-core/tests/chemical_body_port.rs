#![cfg(feature = "chemical-reference-port")]
use chemistry_core::chemical_body_port::{BodyConfig, ChemicalBodyState, SourcePreparation};
fn close(a: f64, b: f64, t: f64) {
    assert!((a - b).abs() < t, "{a} != {b}");
}
fn cold() -> ChemicalBodyState {
    ChemicalBodyState::cold(BodyConfig::default()).unwrap()
}
fn evolved() -> ChemicalBodyState {
    let mut s = cold();
    for _ in 0..100 {
        s = s.try_step(0.002).unwrap().0;
    }
    s
}

#[test]
fn cold_preparation_has_finite_balanced_bath_and_zero_preload() {
    let s = cold();
    let d = s.diagnostics().unwrap();
    assert_eq!(d.vertices.len(), 24);
    close(d.volume, 1., 1e-15);
    close(d.outside_volume, 9., 1e-15);
    close(d.outside_amount / 9., d.amounts.iter().sum(), 1e-14);
    assert!(d.ec < 1e-25 && d.us < 1e-25);
    assert!(d.osmotic_forces.iter().flatten().all(|f| f.abs() < 1e-14));
    for (got, want) in d
        .amount_derivative
        .iter()
        .zip([0.089, 0.005, -0.094, 0.049, -0.9, 0.985])
    {
        close(*got, want, 1e-13);
    }
}
#[test]
fn all_storage_gradient_includes_osmotic_and_native_forces() {
    let s = evolved();
    let d = s.diagnostics().unwrap();
    let h = 1e-6;
    for vertex in [0, 1, 5, 12] {
        for axis in 0..2 {
            let up = s.geometry_probe(vertex, axis, h).unwrap();
            let dn = s.geometry_probe(vertex, axis, -h).unwrap();
            close(
                (up.total_g - dn.total_g) / (2. * h),
                -d.combined_forces[vertex][axis],
                1e-7,
            );
        }
    }
    assert!(d.gradient_identity_error.abs() < 1e-10);
    assert!(d.omitted_osmotic_identity_error.abs() > 1e-10);
}
#[test]
fn native_midpoint_transaction_preserves_source_storage_and_pullback() {
    let old = evolved();
    let saved = old.restart_json().unwrap();
    let (new, r) = old.try_remesh_split().unwrap();
    assert_eq!(r.native_splits, 24);
    assert_eq!(new.mesh().n(), 48);
    assert_eq!(old.mesh().n(), 24);
    assert_eq!(r.before.amounts, r.after.amounts);
    assert_eq!(r.before.p, r.after.p);
    assert_eq!(r.before.a0, r.after.a0);
    assert!(
        r.storage_delta.abs() < 1e-10
            && r.area_delta.abs() < 1e-12
            && r.force_pullback_error < 1e-10
    );
    close(r.before.arc_length, r.after.arc_length, 1e-12);
    close(r.before.arc_rest_length, r.after.arc_rest_length, 1e-12);
    assert_eq!(new.diagnostics().unwrap().arc_indices, vec![0, 1]);
    assert_eq!(saved, old.restart_json().unwrap());
    assert!(new.try_remesh_split().is_err());
}
#[test]
fn continued_native_body_after_split_owns_all_material_and_rates() {
    let mut s = evolved().try_remesh_split().unwrap().0;
    for _ in 0..20 {
        let (next, r) = s.try_step(0.001).unwrap();
        assert!(r.diagnostics.gradient_identity_error.abs() < 1e-10);
        s = next;
    }
    let d = s.diagnostics().unwrap();
    assert_eq!(d.vertices.len(), 48);
    close(
        d.structural_masses.iter().sum(),
        cold().diagnostics().unwrap().structural_masses.iter().sum(),
        1e-12,
    );
    assert!(d.us > 0. && d.volume != 1.);
    close(d.amounts[0] + d.amounts[1] + d.amounts[2], 0.2, 1e-12);
}
#[test]
fn exact_restart_replays_before_and_after_native_remesh() {
    let a = evolved();
    let b = a.try_remesh_split().unwrap().0;
    for state in [a, b] {
        let restored =
            ChemicalBodyState::from_restart_json(&state.restart_json().unwrap(), state.config())
                .unwrap();
        let next = state.try_step(0.001).unwrap().0;
        let replay = restored.try_step(0.001).unwrap().0;
        assert_eq!(next.restart_json().unwrap(), replay.restart_json().unwrap());
    }
}
#[test]
fn trusted_restart_rejects_source_material_ancestry_bath_and_ledger_changes() {
    let s = evolved().try_remesh_split().unwrap().0;
    let base: serde_json::Value = serde_json::from_str(&s.restart_json().unwrap()).unwrap();
    for path in [
        "config", "amount", "ancestry", "material", "ledger", "unknown",
    ] {
        let mut v = base.clone();
        match path {
            "config" => v["data"]["config"]["k"] = serde_json::json!(3.),
            "amount" => v["data"]["amounts"][0] = serde_json::json!(0.),
            "ancestry" => v["data"]["ancestry"][0]["end"] = serde_json::json!(0.7),
            "material" => v["data"]["mesh"]["edges"][0]["m_young"] = serde_json::json!(0.01),
            "ledger" => v["data"]["remesh_defect"] = serde_json::json!(1.),
            _ => v["data"]["outside_amount"] = serde_json::json!(0.),
        }
        assert!(
            ChemicalBodyState::from_restart_json(&v.to_string(), s.config()).is_err(),
            "{path}"
        );
    }
}
#[test]
fn rejected_full_trial_preserves_prior_nonzero_energy_ledger() {
    let mut cfg = BodyConfig::default();
    cfg.rate_scale = 1e5;
    let (state, prior) = ChemicalBodyState::cold(cfg)
        .unwrap()
        .try_step(1e-9)
        .unwrap();
    assert!(prior.integrated_dchem > 0.);
    let saved = state.restart_json().unwrap();
    assert!(state.try_step(0.002).is_err());
    assert_eq!(saved, state.restart_json().unwrap());
}
#[test]
fn invalid_steps_configs_and_remesh_are_not_silently_projected() {
    let s = cold();
    assert!(s.try_remesh_split().is_err());
    for dt in [f64::NAN, 0., -1., 0.02] {
        assert!(s.try_step(dt).is_err());
    }
    let mut c = BodyConfig::default();
    c.theta = 0.;
    assert!(ChemicalBodyState::cold(c).is_err());
    assert!(s.geometry_probe(99, 0, 0.1).is_err());
    assert!(s.geometry_probe(0, 0, 10.).is_err());
}
#[test]
fn simple_concavity_is_allowed_but_crossings_and_duplicate_vertices_are_not() {
    let s = cold();
    assert!(s.geometry_probe(0, 0, -0.08).is_ok());
    let evolved = evolved();
    let base: serde_json::Value = serde_json::from_str(&evolved.restart_json().unwrap()).unwrap();
    let mut duplicate = base.clone();
    duplicate["data"]["mesh"]["vertices"][3] = duplicate["data"]["mesh"]["vertices"][0].clone();
    let duplicate_error =
        ChemicalBodyState::from_restart_json(&duplicate.to_string(), evolved.config()).unwrap_err();
    // Reusing vertex0 also makes an earlier nonadjacent edge touch vertex0;
    // either geometry guard correctly rejects the same invalid boundary.
    assert!(
        duplicate_error == "duplicate/contact vertices"
            || duplicate_error == "nonadjacent boundary crossing/touch",
        "{duplicate_error}"
    );
    let mut crossing = base;
    let vertices = crossing["data"]["mesh"]["vertices"].as_array_mut().unwrap();
    // Nearby swapped vertices create crossing diagonals without first
    // exceeding the independent native length/reference domain.
    vertices.swap(2, 3);
    let error =
        ChemicalBodyState::from_restart_json(&crossing.to_string(), evolved.config()).unwrap_err();
    assert!(error.contains("crossing/touch"), "{error}");
}
#[test]
fn equilibrium_preparation_is_independent_of_reaction_trajectory() {
    let mut c = BodyConfig::default();
    c.preparation = SourcePreparation::ClosedEquilibrium;
    let mut s = ChemicalBodyState::cold(c).unwrap();
    let initial = s.diagnostics().unwrap();
    assert!(initial.currents.iter().all(|j| j.abs() < 1e-14));
    for _ in 0..100 {
        s = s.try_step(0.002).unwrap().0;
    }
    s = s.try_remesh_split().unwrap().0;
    for _ in 0..10 {
        s = s.try_step(0.001).unwrap().0;
    }
    assert!(s.diagnostics().unwrap().us < 1e-20);
}
#[test]
fn zero_chemistry_control_has_no_preloaded_mechanical_energy() {
    let mut c = BodyConfig::default();
    c.rate_scale = 0.;
    let mut s = ChemicalBodyState::cold(c).unwrap();
    for _ in 0..100 {
        s = s.try_step(0.002).unwrap().0;
    }
    s = s.try_remesh_split().unwrap().0;
    for _ in 0..10 {
        s = s.try_step(0.001).unwrap().0;
    }
    assert!(s.diagnostics().unwrap().us < 1e-20);
}
