#![cfg(feature = "chemical-reference-port")]
use chemistry_core::chemical_composed_port::{
    pair_diagnostics, regularized_stretch, ComposedConfig, ComposedWorld, Event,
};
fn close(a: f64, b: f64, t: f64) {
    assert!((a - b).abs() < t, "{a} != {b}");
}
fn prepared() -> ComposedWorld {
    ComposedWorld::prepared(ComposedConfig::default()).unwrap()
}
fn evolved() -> ComposedWorld {
    let mut w = prepared();
    for _ in 0..10 {
        w = w.try_step(0.001).unwrap().0;
    }
    w
}
fn split() -> ComposedWorld {
    let mut w = evolved();
    w = w.try_event(Event::Remesh { cell: 0 }).unwrap().0;
    w.try_event(Event::Fission {
        cell: 0,
        i: 0,
        j: 6,
    })
    .unwrap()
    .0
}
#[test]
fn exact_regularized_native_primitive_derivative_and_newborn_limit() {
    let c = ComposedConfig::default();
    for (h, y, l) in [
        (0., 2., 20.),
        (0.00005, 0.001, 0.02),
        (0.001, 0.002, 0.2),
        (1., 0.1, 4.),
        (1., 0.1, 50.),
    ] {
        let dh = 1e-6 * l;
        let derivative = (regularized_stretch(l + dh, h, y, &c).unwrap().0
            - regularized_stretch(l - dh, h, y, &c).unwrap().0)
            / (2. * dh);
        close(
            derivative,
            regularized_stretch(l, h, y, &c).unwrap().1,
            1e-7,
        );
    }
    assert_eq!(regularized_stretch(20., 0., 2., &c).unwrap(), (0., 0.));
}
#[test]
fn published_source_amount_derivative_and_native_movement_share_one_world() {
    let w = prepared();
    let d = w.diagnostics().unwrap();
    for (got, want) in d.cells[0]
        .ndot
        .iter()
        .zip([-0.901, 0.5, 0.401, 0.049, 9., -10.4])
    {
        close(*got, want, 1e-12);
    }
    let (next, r) = w.try_step(0.001).unwrap();
    assert_ne!(
        r.before.cells[0].mesh.vertices,
        r.after.cells[0].mesh.vertices
    );
    assert_ne!(
        w.diagnostics().unwrap().cells[0].n,
        next.diagnostics().unwrap().cells[0].n
    );
    assert!(r.after.cells[0].gradient_identity_error.abs() < 1e-9);
    close(
        r.delta_G + r.before.cells[0].Dchem * r.dt + r.before.cells[0].Dmech * r.dt,
        r.continuous_defect,
        1e-12,
    );
}
#[test]
fn evolving_transport_growth_maturation_use_actual_previous_pools() {
    let w = evolved();
    let old = w.diagnostics().unwrap();
    let (a, ra) = w
        .try_event(Event::Transport {
            cell: 0,
            species: "Sm".into(),
            direction: "in".into(),
            amount: 0.2,
        })
        .unwrap();
    close(ra.after.cells[0].n[5] - old.cells[0].n[5], 0.2, 1e-12);
    close(ra.before.outside.Sm - ra.after.outside.Sm, 0.2, 1e-12);
    let (b, rb) = a
        .try_event(Event::Growth {
            cell: 0,
            edge: 0,
            extent: 0.2,
        })
        .unwrap();
    close(rb.before.cells[0].n[5] - rb.after.cells[0].n[5], 0.2, 1e-12);
    close(
        rb.after.cells[0].mesh.edges[0].m - rb.before.cells[0].mesh.edges[0].m,
        0.18,
        1e-12,
    );
    let (_, rc) = b
        .try_event(Event::Maturation {
            cell: 0,
            edge: 0,
            extent: 0.04,
        })
        .unwrap();
    close(
        rc.before.cells[0].mesh.edges[0].m,
        rc.after.cells[0].mesh.edges[0].m,
        1e-12,
    );
    close(
        rc.before.cells[0].mesh.edges[0].m_young - rc.after.cells[0].mesh.edges[0].m_young,
        0.04,
        1e-12,
    );
}
#[test]
fn actual_native_remesh_preserves_nonzero_storage_pullback_and_source() {
    let w = evolved();
    let d = w.diagnostics().unwrap();
    assert!(d.cells[0].U_native > 0.);
    let (next, r) = w.try_event(Event::Remesh { cell: 0 }).unwrap();
    assert_eq!(next.diagnostics().unwrap().cells[0].mesh.n(), 12);
    assert_eq!(r.mapping.len(), 12);
    close(r.delta_G, 0., 1e-8);
    assert!(r.remesh_pullback_error.unwrap() < 1e-8);
    assert_eq!(r.before.cells[0].n, r.after.cells[0].n);
    assert_eq!(r.before.cells[0].R_ref, r.after.cells[0].R_ref);
    assert!(r.pair.is_none());
}
#[test]
fn prescribed_fission_pays_evolved_closure_and_both_daughters_continue() {
    let w = evolved().try_event(Event::Remesh { cell: 0 }).unwrap().0;
    let (new, r) = w
        .try_event(Event::Fission {
            cell: 0,
            i: 0,
            j: 6,
        })
        .unwrap();
    assert_eq!(r.after.cells.len(), 2);
    let p = &r.before.cells[0];
    let distance = (p.mesh.vertices[6][0] - p.mesh.vertices[0][0])
        .hypot(p.mesh.vertices[6][1] - p.mesh.vertices[0][1]);
    let debit = 2. * w.config().rho * distance / w.config().growth_yield;
    close(
        p.n[5] - r.after.cells.iter().map(|c| c.n[5]).sum::<f64>(),
        debit,
        1e-10,
    );
    for c in &r.after.cells {
        assert_eq!(c.mesh.n(), 7);
        assert_eq!(c.mesh.edges[6].m_young, c.mesh.edges[6].m);
        close(c.mesh.mature_structural_mass(6), 0., 1e-15);
    }
    close(
        r.after.global_invariants.substrate,
        r.before.global_invariants.substrate,
        1e-9,
    );
    close(
        r.after.global_invariants.R_ref,
        r.before.global_invariants.R_ref,
        1e-9,
    );
    assert_eq!(r.before.outside.S, r.after.outside.S);
    assert_eq!(r.before.root_area, r.after.root_area);
    let (next, step) = new.try_step(0.0005).unwrap();
    assert_eq!(next.diagnostics().unwrap().cells.len(), 2);
    for c in &step.after.cells {
        assert!(c.gradient_identity_error.abs() < 1e-9);
    }
}
#[test]
fn newborn_maturation_records_all_regularized_storage_not_heat() {
    let w = split();
    let (next, r) = w
        .try_event(Event::Maturation {
            cell: 1,
            edge: 6,
            extent: 0.01,
        })
        .unwrap();
    assert!(r.energy_changes.native_elastic > 0.);
    close(
        r.energy_changes.bulk_ideal
            + r.energy_changes.solid_standard
            + r.energy_changes.native_elastic
            + r.energy_changes.new_coupling,
        r.delta_G,
        1e-9,
    );
    let pair = r.pair.unwrap();
    close(pair.forward.ln() - pair.reverse.ln(), pair.log_ratio, 1e-10);
    assert!(pair.ensemble_pair_entropy >= 0.);
    assert!(next.try_step(0.0005).is_ok());
}
#[test]
fn global_ledger_separates_finite_jumps_from_continuous_defects() {
    let initial = prepared().diagnostics().unwrap().G_total;
    let mut w = evolved();
    w = w
        .try_event(Event::Growth {
            cell: 0,
            edge: 0,
            extent: 0.2,
        })
        .unwrap()
        .0;
    w = w.try_event(Event::Remesh { cell: 0 }).unwrap().0;
    w = w
        .try_event(Event::Fission {
            cell: 0,
            i: 0,
            j: 6,
        })
        .unwrap()
        .0;
    for _ in 0..5 {
        w = w.try_step(0.0005).unwrap().0;
    }
    let d = w.diagnostics().unwrap();
    let l = d.ledger;
    close(
        d.G_total - initial + l.integrated_Dchem + l.integrated_Dmech
            - l.continuous_signed_defect
            - l.event_delta_G_sum,
        0.,
        1e-8,
    );
}
#[test]
fn prepared_and_evolved_depleted_donors_leave_complete_world_unchanged() {
    for w in [prepared(), evolved()] {
        let before = w.restart_json().unwrap();
        for e in [
            Event::Growth {
                cell: 0,
                edge: 0,
                extent: 1000.,
            },
            Event::Transport {
                cell: 0,
                species: "Sm".into(),
                direction: "in".into(),
                amount: 1000.,
            },
        ] {
            assert!(w.try_event(e).is_err());
            assert_eq!(w.restart_json().unwrap(), before);
        }
    }
}
#[test]
fn pureyoung_floor_remesh_and_late_geometry_failure_are_atomic() {
    let w = split();
    let saved = w.restart_json().unwrap();
    let error = w.try_event(Event::Remesh { cell: 1 }).unwrap_err();
    assert!(error.contains("floor"));
    assert_eq!(saved, w.restart_json().unwrap());
    let mut c = ComposedConfig::default();
    c.gamma = 1e-10;
    let (seed, _) = ComposedWorld::prepared(c).unwrap().try_step(1e-13).unwrap();
    let saved = seed.restart_json().unwrap();
    assert!(seed.try_step(0.001).is_err());
    assert_eq!(saved, seed.restart_json().unwrap());
}
#[test]
fn exact_restart_before_and_after_fission_and_reject_config_ledger_tamper() {
    for w in [evolved(), split()] {
        let raw = w.restart_json().unwrap();
        let restored = ComposedWorld::from_restart_json(&raw, w.config()).unwrap();
        assert_eq!(
            w.try_step(0.0005).unwrap().0.restart_json().unwrap(),
            restored.try_step(0.0005).unwrap().0.restart_json().unwrap()
        );
        let mut changed: serde_json::Value = serde_json::from_str(&raw).unwrap();
        changed["data"]["config"]["k"] = serde_json::json!(9.);
        assert!(ComposedWorld::from_restart_json(&changed.to_string(), w.config()).is_err());
        let mut changed: serde_json::Value = serde_json::from_str(&raw).unwrap();
        changed["data"]["ledger"]["event_delta_G_sum"] = serde_json::json!(9.);
        assert!(ComposedWorld::from_restart_json(&changed.to_string(), w.config()).is_err());
    }
}
#[test]
fn pair_preference_reverses_with_solid_standard_potential_not_schedule() {
    let mut forward = ComposedConfig::default();
    let mut reversed = forward.clone();
    reversed.mu_m = 0.5;
    let mut probabilities = Vec::new();
    for cfg in [forward.clone(), reversed] {
        let w = ComposedWorld::prepared(cfg).unwrap();
        let (_, r) = w
            .try_event(Event::Maturation {
                cell: 0,
                edge: 0,
                extent: 0.04,
            })
            .unwrap();
        probabilities.push(r.pair.unwrap().forward);
    }
    assert!(probabilities[0] > 0.5 && probabilities[1] < 0.5);
    forward.kappa = 1e-320;
    assert!(pair_diagnostics(100., &forward).is_err());
}
#[test]
fn zero_source_and_zero_coupling_are_not_static_world_controls() {
    for zero_source in [true, false] {
        let mut c = ComposedConfig::default();
        if zero_source {
            c.source_rate_scale = 0.;
        } else {
            c.k = 0.;
        }
        let w = ComposedWorld::prepared(c).unwrap();
        let d = w.diagnostics().unwrap();
        let (next, _) = w.try_step(0.001).unwrap();
        assert_ne!(
            next.diagnostics().unwrap().cells[0].mesh.vertices,
            d.cells[0].mesh.vertices
        );
        if zero_source {
            assert_eq!(next.diagnostics().unwrap().cells[0].n, d.cells[0].n);
        }
    }
}
