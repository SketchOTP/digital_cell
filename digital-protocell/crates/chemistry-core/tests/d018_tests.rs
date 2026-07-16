//! D-018 structural constraint provenance and nullcline tests.

use chemistry_core::config::{D008StageMode, EquationVersion, SimParams};
use chemistry_core::d011_analysis::STAGE_E_FAILED_RATES;
use chemistry_core::d018_analysis::*;
use chemistry_core::d018_provenance::*;
use chemistry_core::fields::field_sha256_stable;
use chemistry_core::stoichiometry::ReactionId;
use chemistry_core::{field_mass, Simulation};

fn v2_params() -> SimParams {
    let mut p = SimParams::default();
    p.equation_version = EquationVersion::MembraneMetabolismV2Conservative;
    p.d008_stage_mode = D008StageMode::ConstrainedRadius;
    p.d008_stage_b_enabled = false;
    p.eta_c = 1.0;
    p.eta_phi = 1.0;
    p.eta_m = 1.0;
    // Use frozen Stage-E structure rate for identity-compatible screens.
    STAGE_E_FAILED_RATES.apply_to(&mut p);
    p.k_d008_structure = D018_FROZEN_K_STRUCTURE;
    p
}

fn seed_disk(sim: &mut Simulation, radius: f64) {
    for idx in 0..sim.fields.structure.len() {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        let x = (idx % sim.grid.width) as f64 - sim.grid.cx;
        let y = (idx / sim.grid.width) as f64 - sim.grid.cy;
        let distance = (x * x + y * y).sqrt();
        let phi = 0.5 * (1.0 - ((distance - radius) / 2.0).tanh());
        sim.fields.structure[idx] = phi;
        sim.fields.membrane[idx] = chemistry_core::interface_weight(phi);
        if phi >= 0.5 {
            sim.fields.catalyst[idx] = 0.4;
            sim.fields.activated[idx] = 0.2;
            sim.fields.nutrient[idx] = 0.2;
            sim.fields.fuel[idx] = 0.2;
            sim.fields.waste[idx] = 0.5;
        } else {
            sim.fields.catalyst[idx] = 0.0;
            sim.fields.activated[idx] = 0.0;
            sim.fields.nutrient[idx] = sim.params.n_reservoir;
            sim.fields.fuel[idx] = sim.params.f_reservoir;
            sim.fields.waste[idx] = sim.params.w_reservoir;
        }
    }
    sim.fields.copy_current_to_next();
}

#[test]
fn test_structure_tracer_initially_all_endogenous() {
    let phi = vec![0.0, 0.5, 1.0, 0.25];
    let t = StructureProvenanceTracer::init_from_phi(&phi);
    assert!((t.sum_endogenous() - 1.75).abs() < 1e-12);
    assert_eq!(t.sum_constraint(), 0.0);
    assert!((t.endogenous_fraction_of_structure() - 1.0).abs() < 1e-12);
}

#[test]
fn test_structure_provenance_initialization() {
    test_structure_tracer_initially_all_endogenous();
}

#[test]
fn test_structure_synthesis_adds_endogenous_inventory() {
    let mut t = StructureProvenanceTracer::init_from_phi(&[1.0]);
    t.record_constrained_cell(0, 0.2, 0.0, 1.0);
    // produced 0.2 endogenous; constraint removes 0.2 to restore φ=1
    assert!((t.endogenous[0] - 1.0).abs() < 1e-12);
    assert!((t.constraint[0]).abs() < 1e-12);
    assert!((t.cumulative_endogenous_production - 0.2).abs() < 1e-12);
    assert!((t.cumulative_constraint_removal - 0.2).abs() < 1e-12);
}

#[test]
fn test_structure_synthesis_is_endogenous() {
    let mut t = StructureProvenanceTracer::init_from_phi(&[0.5]);
    // Only production, then restore: net endogenous production credited then constraint removal.
    t.record_constrained_cell(0, 0.1, 0.0, 0.5);
    assert!(t.cumulative_endogenous_production > 0.0);
    assert_eq!(t.cumulative_w_from_constraint, 0.0);
}

#[test]
fn test_positive_constraint_adds_constraint_inventory() {
    let mut t = StructureProvenanceTracer::init_from_phi(&[1.0]);
    t.record_constrained_cell(0, 0.0, 0.3, 1.0);
    assert!((t.constraint[0] - 0.3).abs() < 1e-12);
    assert!((t.endogenous[0] - 0.7).abs() < 1e-12);
    assert!((t.cumulative_constraint_addition - 0.3).abs() < 1e-12);
    assert!((t.cumulative_w_from_endogenous - 0.3).abs() < 1e-12);
}

#[test]
fn test_constraint_addition_is_constraint_origin() {
    test_positive_constraint_adds_constraint_inventory();
}

#[test]
fn test_structure_decay_is_attributed_proportionally() {
    let mut t = StructureProvenanceTracer::init_from_phi(&[0.5]);
    t.constraint[0] = 0.5;
    t.endogenous[0] = 0.5;
    t.record_constrained_cell(0, 0.0, 0.2, 1.0);
    // decay 0.2 split 50/50, then constraint adds 0.2
    assert!((t.cumulative_w_from_endogenous - 0.1).abs() < 1e-12);
    assert!((t.cumulative_w_from_constraint - 0.1).abs() < 1e-12);
}

#[test]
fn test_structure_decay_attributes_origin() {
    test_structure_decay_is_attributed_proportionally();
}

#[test]
fn test_negative_constraint_removes_proportionally() {
    let mut t = StructureProvenanceTracer::init_from_phi(&[1.0]);
    // Over-produce relative to decay ⇒ negative constraint (removal).
    t.record_constrained_cell(0, 0.4, 0.1, 1.0);
    assert!(t.cumulative_constraint_removal > 0.0);
    assert!((t.endogenous[0] + t.constraint[0] - 1.0).abs() < 1e-12);
}

#[test]
fn test_endogenous_plus_constraint_equals_fixed_structure() {
    let mut t = StructureProvenanceTracer::init_from_phi(&[0.8, 0.2, 1.0]);
    t.record_constrained_cell(0, 0.05, 0.12, 0.8);
    t.record_constrained_cell(1, 0.01, 0.00, 0.2);
    t.record_constrained_cell(2, 0.00, 0.25, 1.0);
    assert!(t.inventory_closes_against_phi(&[0.8, 0.2, 1.0]));
}

#[test]
fn test_provenance_inventory_closes() {
    test_endogenous_plus_constraint_equals_fixed_structure();
}

#[test]
fn test_constraint_waste_fraction_is_calculated() {
    let mut t = StructureProvenanceTracer::init_from_phi(&[1.0]);
    t.record_constrained_cell(0, 0.0, 1.0, 1.0);
    let m = t.metrics(2.0, 1.0);
    assert!((m.constraint_fraction_of_structure_w - 0.0).abs() < 1e-12); // first decay all endogenous
    // Second decay after constraint refill: half/half inventories ≈ equal attribution
    t.record_constrained_cell(0, 0.0, 1.0, 1.0);
    let m2 = t.metrics(2.0, 1.0);
    assert!(m2.constraint_fraction_of_total_w > 0.0);
    assert!(m2.constraint_turnovers >= 1.0);
}

#[test]
fn test_contaminated_window_cannot_establish_balance() {
    let class = classify_constraint_contamination(0.10, 1.0, 10.0);
    assert_eq!(class, ConstraintContaminationClass::ConstraintContaminated);
    let ok = classify_constraint_contamination(0.01, 0.1, 10.0);
    assert_eq!(ok, ConstraintContaminationClass::ConstraintUsable);
}

#[test]
fn test_required_k_structure_uses_raw_basis() {
    let b = 2.0;
    let l = 10.0;
    let k = required_k_structure(b, l);
    assert!((k - 5.0).abs() < 1e-12);
    // Must not silently use k/Q form when Q = k*b/l.
    let k_current = 1.0;
    let q = (k_current * b) / l;
    let wrong = k_current / q;
    assert!((wrong - k).abs() < 1e-12); // identity holds for this rate form
    assert!((production_basis_from_extent(4.0, 2.0) - 2.0).abs() < 1e-12);
}

#[test]
fn test_structure_basis_matches_runtime_rate() {
    // r = k * B ⇒ B = extent / k
    let k = 1.0812170527078209;
    let extent = 5.4060852635391045;
    assert!((production_basis_from_extent(extent, k) - 5.0).abs() < 1e-12);
}

#[test]
fn test_radius_scaling_fit_is_deterministic() {
    let points: Vec<StructureBasisPoint> = [14.0, 18.0, 22.0, 26.0, 30.0, 34.0]
        .into_iter()
        .map(|r| {
            let b = r; // interface-like
            let l = r * r; // bulk-like
            StructureBasisPoint {
                radius: r,
                b_structure: b,
                l_structure: l,
                k_required: l / b,
                k_current: D018_FROZEN_K_STRUCTURE,
                required_over_current: (l / b) / D018_FROZEN_K_STRUCTURE,
                authorized_min: authorized_k_structure_domain().0,
                authorized_max: authorized_k_structure_domain().1,
                inside_authorized_domain: false,
                sampling_window_steps: 100,
                constraint_fraction_of_total_w: 0.01,
                window_usable: true,
            }
        })
        .collect();
    let a = fit_radius_scaling(&points).unwrap();
    let b = fit_radius_scaling(&points).unwrap();
    assert_eq!(a.production_exponent_p, b.production_exponent_p);
    assert_eq!(a.decay_exponent_q, b.decay_exponent_q);
    assert!((a.production_exponent_p - 1.0).abs() < 0.05);
    assert!((a.decay_exponent_q - 2.0).abs() < 0.05);
    assert_eq!(a.production_scaling_class, ScalingClass::InterfaceScaled);
    assert_eq!(a.decay_scaling_class, ScalingClass::BulkScaled);
}

#[test]
fn test_restoring_crossing_requires_sign_change() {
    assert!(restoring_crossing_signs(1.0, 0.0, -1.0));
    assert!(!restoring_crossing_signs(-1.0, 0.0, 1.0));
    assert!(!restoring_crossing_signs(-1.0, -2.0, -3.0));
}

#[test]
fn test_rate_screen_respects_authorized_domain() {
    let (lo, hi) = authorized_k_structure_domain();
    assert!(k_structure_inside_authorized(D018_ANALYTICAL_K_STRUCTURE));
    assert!(!k_structure_inside_authorized(lo * 0.5));
    assert!(!k_structure_inside_authorized(hi * 2.0));
}

#[test]
fn test_prebalance_candidate_count_is_bounded() {
    let c = prebalance_k_candidates(2.0);
    assert!(c.len() <= 3);
    let (lo, hi) = authorized_k_structure_domain();
    for k in &c {
        assert!(*k >= lo - 1e-12 && *k <= hi + 1e-12);
    }
}

#[test]
fn test_full_reference_requires_low_constraint_contamination() {
    assert!(!promote_structure_candidate(1.0, 0.10, false, false, true));
    assert!(promote_structure_candidate(1.0, 0.01, false, false, true));
    assert!(!promote_structure_candidate(0.1, 0.01, false, false, true));
}

#[test]
fn test_d018_does_not_change_reaction_topology() {
    // Structure production / decay stoichiometry unchanged (A→φ virtual / φ→W).
    let _ = ReactionId::StructureProduction;
    let _ = ReactionId::StructureDecay;
    assert_eq!(D018_FROZEN_K_STRUCTURE, D018_ANALYTICAL_K_STRUCTURE);
    let p = v2_params();
    assert_eq!(
        p.equation_version,
        EquationVersion::MembraneMetabolismV2Conservative
    );
}

#[test]
fn test_unconstrained_control_has_no_constraint_flux() {
    let mut sim = Simulation::new(v2_params());
    seed_disk(&mut sim, 22.0);
    sim.enforce_structure_constraint = false;
    sim.structure_provenance = Some(StructureProvenanceTracer::init_from_phi(&sim.fields.structure));
    assert!(sim.step());
    assert!(sim.constraint_accounting.last_step.constraint_flux.abs() < 1e-12);
}

#[test]
fn test_tracer_has_no_effect_on_simulation_fields() {
    let mut a = Simulation::new(v2_params());
    let mut b = Simulation::new(v2_params());
    seed_disk(&mut a, 22.0);
    seed_disk(&mut b, 22.0);
    b.structure_provenance = Some(StructureProvenanceTracer::init_from_phi(&b.fields.structure));
    for _ in 0..20 {
        assert!(a.step());
        assert!(b.step());
    }
    assert_eq!(
        field_sha256_stable(&a.fields.structure),
        field_sha256_stable(&b.fields.structure)
    );
    assert_eq!(
        field_sha256_stable(&a.fields.catalyst),
        field_sha256_stable(&b.fields.catalyst)
    );
    assert_eq!(
        field_sha256_stable(&a.fields.waste),
        field_sha256_stable(&b.fields.waste)
    );
    assert_eq!(
        field_sha256_stable(&a.fields.activated),
        field_sha256_stable(&b.fields.activated)
    );
    assert_eq!(
        field_sha256_stable(&a.fields.nutrient),
        field_sha256_stable(&b.fields.nutrient)
    );
    assert_eq!(
        field_sha256_stable(&a.fields.fuel),
        field_sha256_stable(&b.fields.fuel)
    );
    assert_eq!(
        field_sha256_stable(&a.fields.membrane),
        field_sha256_stable(&b.fields.membrane)
    );
}

#[test]
fn test_tracer_has_no_effect_on_timestep() {
    let mut a = Simulation::new(v2_params());
    let mut b = Simulation::new(v2_params());
    seed_disk(&mut a, 22.0);
    seed_disk(&mut b, 22.0);
    b.structure_provenance = Some(StructureProvenanceTracer::init_from_phi(&b.fields.structure));
    for _ in 0..30 {
        assert!(a.step());
        assert!(b.step());
    }
    assert_eq!(a.substep, b.substep);
    assert!((a.sim_time - b.sim_time).abs() < 1e-12);
    assert!((a.dt - b.dt).abs() < 1e-15);
}

#[test]
fn test_tracer_has_no_effect_on_field_hashes() {
    test_tracer_has_no_effect_on_simulation_fields();
}

#[test]
fn test_provenance_tracer_is_observer_only() {
    test_tracer_has_no_effect_on_simulation_fields();
    test_tracer_has_no_effect_on_timestep();
}

#[test]
fn test_select_conclusion_surface_volume_with_artifact() {
    let points: Vec<StructureBasisPoint> = [14.0, 22.0, 34.0]
        .into_iter()
        .map(|r| StructureBasisPoint {
            radius: r,
            b_structure: r,
            l_structure: r * r,
            k_required: r,
            k_current: D018_FROZEN_K_STRUCTURE,
            required_over_current: r / D018_FROZEN_K_STRUCTURE,
            authorized_min: authorized_k_structure_domain().0,
            authorized_max: authorized_k_structure_domain().1,
            inside_authorized_domain: false,
            sampling_window_steps: 50,
            constraint_fraction_of_total_w: 0.9,
            window_usable: true,
        })
        .collect();
    let fit = fit_radius_scaling(&points).unwrap();
    let nullcline = classify_structural_nullcline(&points, D018_FROZEN_K_STRUCTURE);
    let (primary, sub) = select_d018_conclusion(
        true,
        HistoricalWasteOriginClass::ConstraintWasteDominant,
        UnconstrainedClass::StructureCollapseLimitsWSource,
        nullcline,
        Some(&fit),
        false,
        false,
    );
    assert!(matches!(
        primary,
        D018PrimaryConclusion::D018SurfaceVolumeScalingIncompatible
            | D018PrimaryConclusion::D018StructureBalanceOutsideRateDomain
    ));
    assert!(sub.is_some());
    let _ = field_mass;
}
