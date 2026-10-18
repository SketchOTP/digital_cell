#![cfg(feature = "chemical-reference-port")]
use chemistry_core::chemical_reference_port::{ReferencePortConfig, ReferencePortState, INITIAL};
use chemistry_core::mesh_mechanics::{
    mechanics_step, mechanics_step_with_external_forces, MechParams,
};

fn cold() -> ReferencePortState {
    ReferencePortState::cold(ReferencePortConfig::default()).unwrap()
}
fn close(a: f64, b: f64, t: f64) {
    assert!((a - b).abs() < t, "{a} != {b}");
}

#[test]
fn independent_source_initial_vector_and_native_cold_storage() {
    let d = cold().diagnostics().unwrap();
    assert_eq!(d.species, INITIAL);
    for (got, want) in d
        .species_derivative
        .iter()
        .zip([0.089, 0.005, -0.094, 0.049, -0.9, 0.985])
    {
        close(*got, want, 1e-14);
    }
    close(d.ec, 0.0, 1e-25);
    close(d.us, 0.0, 1e-25);
    assert_eq!(d.vertices.len(), 6);
}

#[test]
fn actual_native_receiver_gradient_matches_force() {
    let state = cold();
    let strain = 0.13;
    let h = 1e-6;
    // Independent energy differences along the actual two-endpoint preparation path.
    let d = state.probe(strain, 0.02).unwrap();
    let up = state.probe(strain + h, 0.02).unwrap().us;
    let down = state.probe(strain - h, 0.02).unwrap().us;
    let base = state.diagnostics().unwrap();
    let r = [
        base.vertices[1][0] - base.vertices[0][0],
        base.vertices[1][1] - base.vertices[0][1],
    ];
    let projected: f64 = (0..2)
        .map(|a| (d.native_forces[0][a] - d.native_forces[1][a]) * r[a] / 2.0)
        .sum();
    close((up - down) / (2.0 * h), projected, 1e-8);
}

#[test]
fn both_signed_coupling_forces_and_energy_gradient() {
    let s = cold();
    for strain in [-0.12, 0.16] {
        let d = s.probe(strain, 0.0).unwrap();
        let h = 1e-6;
        let plus = s.probe(strain + h, 0.0).unwrap();
        let minus = s.probe(strain - h, 0.0).unwrap();
        close(
            (plus.ec - minus.ec) / (2.0 * h) / d.rest_lengths[0],
            d.coupling_tension,
            1e-9,
        );
        assert_eq!(d.coupling_tension.is_sign_positive(), strain > 0.0);
        for a in 0..2 {
            close(
                d.coupling_forces[0][a] + d.coupling_forces[1][a],
                0.0,
                1e-15,
            );
        }
    }
}

#[test]
fn channel_bias_reciprocal_and_theta_not_assumed_one() {
    let mut cfg = ReferencePortConfig::default();
    cfg.theta = 2.5;
    let d = ReferencePortState::cold(cfg)
        .unwrap()
        .probe(0.15, -0.02)
        .unwrap();
    for (i, nu) in [0.0, 1.0, 0.0, -1.0].iter().enumerate() {
        close(
            (d.forward[i] / d.reverse[i]).ln() - (d.bare_forward[i] / d.bare_reverse[i]).ln(),
            -nu * d.g / 2.5,
            1e-13,
        );
    }
    close(d.gradient_identity_error, 0.0, 1e-12);
}

#[test]
fn actual_native_zero_external_path_preserves_legacy_mechanics() {
    let state = cold();
    let mut a = state.mesh().clone();
    let mut b = a.clone();
    a.vertices[0][0] += 0.01;
    b.vertices[0][0] += 0.01;
    let p = MechParams {
        gamma: 1.0,
        k_s: 1.0,
        kappa_b: 0.0,
        k_pi: 0.0,
        dt: 0.001,
    };
    assert!(mechanics_step(&mut a, &p));
    assert!(mechanics_step_with_external_forces(
        &mut b,
        &p,
        &[[0.0; 2]; 6]
    ));
    assert_eq!(
        serde_json::to_value(a).unwrap(),
        serde_json::to_value(b).unwrap()
    );
}

#[test]
fn native_joint_trajectory_moves_all_receivers_and_preserves_source() {
    let mut s = cold();
    let first = s.diagnostics().unwrap();
    let mut work = 0.0;
    for _ in 0..300 {
        let (next, r) = s.try_step(0.001).unwrap();
        work += r.coupling_work;
        s = next;
    }
    let last = s.diagnostics().unwrap();
    assert_ne!(last.vertices, first.vertices);
    assert!(last.us > 0.0);
    assert!(work > 0.0);
    assert!(last.total_g < first.total_g);
    close(
        last.species[0] + last.species[1] + last.species[2],
        0.2,
        1e-12,
    );
    close(
        last.species[4]
            + last.species[5]
            + last.species[1]
            + 2.0 * last.species[2]
            + 2.0 * last.species[3],
        1.27,
        1e-12,
    );
}

#[test]
fn restart_exact_next_step_and_trusted_config_binding() {
    let (s, _) = cold().try_step(0.001).unwrap();
    let json = s.restart_json().unwrap();
    let r = ReferencePortState::from_restart_json(&json, s.config()).unwrap();
    let (a, _) = s.try_step(0.001).unwrap();
    let (b, _) = r.try_step(0.001).unwrap();
    assert_eq!(a.restart_json().unwrap(), b.restart_json().unwrap());
    let mut tamper: serde_json::Value = serde_json::from_str(&json).unwrap();
    tamper["config"]["k"] = serde_json::json!(3.0);
    assert!(ReferencePortState::from_restart_json(&tamper.to_string(), s.config()).is_err());
}

#[test]
fn restart_rejects_a0_material_and_energy_ledger_tampering() {
    let (s, _) = cold().try_step(0.001).unwrap();
    let v: serde_json::Value = serde_json::from_str(&s.restart_json().unwrap()).unwrap();
    for key in ["a0", "signed_defect", "initial_total_g"] {
        let mut t = v.clone();
        t[key] = serde_json::json!(123.0);
        assert!(ReferencePortState::from_restart_json(&t.to_string(), s.config()).is_err());
    }
    let mut t = v;
    t["mesh"]["edges"][0]["m_young"] = serde_json::json!(0.01);
    assert!(ReferencePortState::from_restart_json(&t.to_string(), s.config()).is_err());
}

#[test]
fn crossing_ring_with_positive_local_turns_cannot_enter_by_restart() {
    let (s, _) = cold().try_step(0.001).unwrap();
    let mut value: serde_json::Value = serde_json::from_str(&s.restart_json().unwrap()).unwrap();
    let vertices = value["mesh"]["vertices"].as_array().unwrap().clone();
    value["mesh"]["vertices"] = serde_json::json!([
        vertices[0],
        vertices[2],
        vertices[4],
        vertices[1],
        vertices[3],
        vertices[5]
    ]);
    let error = ReferencePortState::from_restart_json(&value.to_string(), s.config()).unwrap_err();
    assert!(error.contains("non-convex"), "{error}");
}

#[test]
fn rejected_steps_keep_nonzero_old_ledgers_and_mesh_exact() {
    let (s, _) = cold().try_step(0.001).unwrap();
    let before = s.restart_json().unwrap();
    for dt in [f64::NAN, 0.0, -0.1, 0.02] {
        assert!(s.try_step(dt).is_err());
        assert_eq!(before, s.restart_json().unwrap());
    }
    let mut cfg = ReferencePortConfig::default();
    cfg.rate_scale = 1e8;
    let fast = ReferencePortState::cold(cfg).unwrap();
    let saved = fast.restart_json().unwrap();
    assert!(fast.try_step(0.001).is_err());
    assert_eq!(saved, fast.restart_json().unwrap());
}

#[test]
fn late_native_joint_failure_discards_trial_with_previous_nonzero_ledgers() {
    // A small genuine seed step loads chemical/defect ledgers while native
    // cold roundoff remains negligible. Its nonzero reciprocal tension then
    // makes the larger, still admitted dt fold the low-drag native ring.
    let mut cfg = ReferencePortConfig::default();
    cfg.gamma = 1e-10;
    let (state, first) = ReferencePortState::cold(cfg)
        .unwrap()
        .try_step(1e-6)
        .unwrap();
    assert!(first.integrated_dchem > 0.0);
    assert!(first.absolute_defect > 0.0);
    let force = first.diagnostics.coupling_forces[0];
    assert!(force[0].hypot(force[1]) > 1e-8);
    assert!(force[0].hypot(force[1]) < 0.5);
    let before = state.restart_json().unwrap();
    let error = state.try_step(0.001).unwrap_err();
    // These are after the actual native vertex trial, not a pre-step cap or
    // source failure. No state, source or prior ledger is published on reject.
    assert!(
        error.contains("native mechanics rejected")
            || error.contains("reference/clamp domain")
            || error.contains("non-convex/contact"),
        "{error}"
    );
    assert_eq!(before, state.restart_json().unwrap());
}

#[test]
fn unsupported_source_material_domain_and_cap_are_rejected() {
    let s = cold();
    assert!(s.probe(0.0, 0.2).is_err());
    assert!(s.probe(4.0, 0.0).is_err());
    let mut bad = ReferencePortConfig::default();
    bad.gamma = 0.0;
    assert!(ReferencePortState::cold(bad).is_err());
    let mut cap = ReferencePortConfig::default();
    cap.k = 10000.0;
    let s = ReferencePortState::cold(cap).unwrap();
    let before = s.restart_json().unwrap();
    assert!(s.try_step(0.001).is_err());
    assert_eq!(before, s.restart_json().unwrap());
}

#[test]
fn controls_leave_native_ring_at_rest() {
    for zero_lambda in [true, false] {
        let mut cfg = ReferencePortConfig::default();
        if zero_lambda {
            cfg.lambda = 0.0;
        } else {
            cfg.rate_scale = 0.0;
        }
        let mut s = ReferencePortState::cold(cfg).unwrap();
        let vertices = s.mesh().vertices.clone();
        for _ in 0..10 {
            s = s.try_step(0.001).unwrap().0;
        }
        let maximum_displacement = s
            .mesh()
            .vertices
            .iter()
            .zip(&vertices)
            .flat_map(|(a, b)| (0..2).map(move |axis| (a[axis] - b[axis]).abs()))
            .fold(0.0, f64::max);
        assert!(maximum_displacement <= 16.0 * f64::EPSILON);
        assert!(s.diagnostics().unwrap().us <= 1e-20);
    }
}
