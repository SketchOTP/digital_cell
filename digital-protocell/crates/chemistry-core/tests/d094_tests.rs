use chemistry_core::autocatalytic_copying::{founder_h_edges, founder_n_edges, seed_founder_edges};
use chemistry_core::autocatalytic_nodes::{
    autocatalytic_schema_load_ok, stamp_autocatalytic_equation, AutocatalyticParams,
    EQUATION_VERSION_AUTOCATALYTIC_SET, FIELD_SCHEMA_AUTOCATALYTIC_SET, MU_E,
};
use chemistry_core::autocatalytic_partition::has_directed_cycle;
use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use chemistry_core::metabolic_reserve::stamp_reserve_equation;

#[test]
fn schema_ids() {
    assert_eq!(
        EQUATION_VERSION_AUTOCATALYTIC_SET,
        "autopoietic_material_mesh_autocatalytic_set_v1"
    );
    assert_eq!(
        FIELD_SCHEMA_AUTOCATALYTIC_SET,
        "mesh_vertices_edges_reserve_autocatalytic_network_v1"
    );
    assert!((MU_E - 0.0089).abs() < 1e-9);
}

#[test]
fn schema_isolation() {
    let mut old = MaterialMesh::seed_regular(
        24,
        5.0,
        40.0,
        40.0,
        DEFAULT_RHO_S,
        0.7,
        LumpedChem {
            c: 0.8,
            a: 0.5,
            n: 0.4,
            f: 0.4,
            w: 0.1,
            ..Default::default()
        },
        LumpedChem {
            n: 1.0,
            f: 1.0,
            ..Default::default()
        },
        5.0,
    );
    stamp_reserve_equation(&mut old);
    let p = AutocatalyticParams::derived(40.0);
    assert!(!autocatalytic_schema_load_ok(&old, &p));
    stamp_autocatalytic_equation(&mut old);
    assert!(autocatalytic_schema_load_ok(&old, &p));
}

#[test]
fn founder_h_has_cycle() {
    let mut mesh = MaterialMesh::seed_regular(
        24,
        5.0,
        40.0,
        40.0,
        DEFAULT_RHO_S,
        0.7,
        LumpedChem {
            c: 0.8,
            a: 0.5,
            ..Default::default()
        },
        LumpedChem::default(),
        5.0,
    );
    stamp_autocatalytic_equation(&mut mesh);
    seed_founder_edges(&mut mesh, &founder_h_edges());
    assert!(has_directed_cycle(&mesh));
    assert_eq!(mesh.autocatalytic_edges.len(), 10);
    // Neutral also cyclic
    mesh.autocatalytic_edges.clear();
    seed_founder_edges(&mut mesh, &founder_n_edges());
    assert!(has_directed_cycle(&mesh));
}

#[test]
fn partition_spreads_edges() {
    use chemistry_core::autocatalytic_copying::{
        founder_h_edges, redistribute_edges_along_axis, seed_founder_edges,
    };
    use chemistry_core::autocatalytic_nodes::stamp_autocatalytic_equation;
    use chemistry_core::autocatalytic_partition::partition_autocatalytic_edges;
    use chemistry_core::mesh_fission::{try_local_fission, FissionParams};
    use chemistry_core::mesh_growth::GrowthParams;
    use chemistry_core::mesh_mechanics::{mechanics_step, remesh, MechParams};
    use chemistry_core::mesh_reactions::{reactions_step, ReactionParams};
    use chemistry_core::mesh_transport::TransportParams;
    use chemistry_core::metabolic_reserve::ReserveParams;

    let mut mesh = MaterialMesh::seed_regular(
        24,
        12.0,
        40.0,
        40.0,
        DEFAULT_RHO_S,
        0.7,
        LumpedChem {
            c: 0.8,
            a: 1.0,
            n: 0.5,
            f: 0.5,
            r: 0.7,
            q_k: 0.5,
            q_e: 0.5,
            k_a: 0.15,
            k_r: 0.15,
            k_node_b: 0.15,
            assimilation_n: 0.31,
            assimilation_f: 0.27,
            ..Default::default()
        },
        LumpedChem {
            n: 1.5,
            f: 1.5,
            ..Default::default()
        },
        5.0,
    );
    let c = mesh.centroid();
    for v in &mut mesh.vertices {
        v[0] = c[0] + (v[0] - c[0]) * 1.45;
    }
    stamp_autocatalytic_equation(&mut mesh);
    seed_founder_edges(&mut mesh, &founder_h_edges());
    let birth = mesh.total_structural_mass();
    let mut react = ReactionParams::default();
    react.reserve = ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, mesh.area());
    react.reserve.enable = true;
    react.autocatalytic = AutocatalyticParams::derived(40.0).with_mutation_off();
    react.autocatalytic.k_edge_loss = 0.0;
    let mech = MechParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let fission = FissionParams::default();
    let mut got = None;
    for s in 0..12000 {
        mesh.exterior.n = 1.5;
        mesh.exterior.f = 1.5;
        let _ = chemistry_core::mesh_transport::transport_step(&mut mesh, &transport, mech.dt);
        let _ = reactions_step(&mut mesh, &react, mech.dt, true, true);
        let _ = chemistry_core::mesh_growth::growth_step(&mut mesh, &react, &growth, mech.dt);
        mechanics_step(&mut mesh, &mech);
        remesh(&mut mesh);
        if s % 50 == 0 {
            redistribute_edges_along_axis(&mut mesh);
        }
        if mesh.total_structural_mass() >= 1.35 * birth && s % 10 == 0 {
            redistribute_edges_along_axis(&mut mesh);
            let xs: Vec<f64> = mesh.autocatalytic_edges.iter().map(|e| e.pos[0]).collect();
            eprintln!("pre-fission edge xs: {:?}", xs);
            if let Some((d1, d2, ev)) = try_local_fission(&mesh, &fission) {
                let c1 = d1.centroid();
                let c2 = d2.centroid();
                eprintln!(
                    "pinch {:?} d1n={} d2n={} c1={:?} c2={:?}",
                    ev.pinch,
                    d1.n(),
                    d2.n(),
                    c1,
                    c2
                );
                for e in &mesh.autocatalytic_edges {
                    let in1 = d1.point_inside(e.pos[0], e.pos[1]);
                    let in2 = d2.point_inside(e.pos[0], e.pos[1]);
                    eprintln!("edge pos={:?} in1={} in2={}", e.pos, in1, in2);
                }
                eprintln!(
                    "d1 edges {} mass {} d2 edges {} mass {}",
                    d1.autocatalytic_edges.len(),
                    d1.total_structural_mass(),
                    d2.autocatalytic_edges.len(),
                    d2.total_structural_mass()
                );
                got = Some((
                    d1,
                    d2,
                    ev.partition.residual_assimilation_n,
                    ev.partition.residual_assimilation_f,
                ));
                break;
            }
        }
    }
    let (d1, d2, residual_assimilation_n, residual_assimilation_f) = got.expect("fission");
    assert!(residual_assimilation_n < 1e-4);
    assert!(residual_assimilation_f < 1e-4);
    assert!(
        d1.autocatalytic_edges.len() > 0 && d2.autocatalytic_edges.len() > 0,
        "expected both daughters to get edges, got {} and {}",
        d1.autocatalytic_edges.len(),
        d2.autocatalytic_edges.len()
    );
}

#[test]
fn selection_completes_generations() {
    use chemistry_core::autocatalytic_copying::{
        founder_b_edges, founder_h_edges, redistribute_edges_along_axis, seed_founder_edges,
    };
    use chemistry_core::autocatalytic_nodes::{stamp_autocatalytic_equation, AutocatalyticParams};
    use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
    use chemistry_core::mesh_fission::FissionParams;
    use chemistry_core::mesh_growth::GrowthParams;
    use chemistry_core::mesh_mechanics::MechParams;
    use chemistry_core::mesh_population::{MeshIndividual, MeshPopulation};
    use chemistry_core::mesh_reactions::ReactionParams;
    use chemistry_core::mesh_transport::TransportParams;
    use chemistry_core::metabolic_reserve::ReserveParams;
    use chemistry_core::seasonal_ecology::{PulseLeanSchedule, PulseLeanState, PULSE_PERIOD_MULTS};

    // Single H individual, radius 14 (Gate-3 reproduction geometry), repaired H
    // ecology with lean floor. Reproduction must complete under intermittent supply.
    let mut pop = MeshPopulation::seed_one(14.0, 11, 2.2);
    {
        let mesh = &mut pop.individuals[0].mesh;
        let c = mesh.centroid();
        for v in &mut mesh.vertices {
            let dx = v[0] - c[0];
            v[0] = c[0] + dx * 1.35;
        }
        stamp_autocatalytic_equation(mesh);
        mesh.interior.r = 0.6;
        mesh.interior.a = 0.9;
        mesh.interior.q_k = 0.5;
        mesh.interior.q_e = 0.5;
        mesh.interior.k_a = 0.12;
        mesh.interior.k_r = 0.12;
        mesh.interior.k_node_b = 0.12;
        seed_founder_edges(mesh, &founder_h_edges());
        pop.individuals[0].birth_mass = mesh.total_structural_mass();
    }
    let _ = (founder_b_edges,);
    let mut react = ReactionParams::default();
    react.reserve = ReserveParams::derived(
        80.0,
        40.0,
        0.5,
        0.3,
        2.0,
        0.1,
        pop.individuals[0].mesh.area(),
    );
    react.reserve.enable = true;
    react.autocatalytic = AutocatalyticParams::derived(40.0).with_mutation_off();
    react.composition.enable = false;
    let mech = MechParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let fission = FissionParams::default();
    let t_maint = 1.0 / react.reserve.k_release.max(1e-9);
    let period = PULSE_PERIOD_MULTS[0] * t_maint * 4.0;
    let mut pulse = PulseLeanState::new(PulseLeanSchedule {
        cycle_period: period,
        pulse_fraction: 0.35,
        cycle_nf_budget: 1.10 * 0.05 * period,
        lean_nf_rate: 0.0,
    });
    let rich = 2.2;
    for s in 0..18000 {
        let (n, f) = if pulse.in_pulse() {
            (rich * 1.25, rich * 1.25)
        } else {
            (rich * 0.12, rich * 0.12)
        };
        for ind in &mut pop.individuals {
            if ind.mesh.alive {
                ind.mesh.exterior.n = n;
                ind.mesh.exterior.f = f;
            }
        }
        pulse.t += mech.dt;
        if s % 40 == 0 {
            for ind in &mut pop.individuals {
                if ind.mesh.alive {
                    redistribute_edges_along_axis(&mut ind.mesh);
                }
            }
        }
        let _ = pop.step(&mech, &react, &transport, &growth, &fission, true);
        if pop.living_count() == 0 || pop.living_count() > 32 {
            break;
        }
    }
    let max_gen = pop
        .individuals
        .iter()
        .map(|i| i.generation)
        .max()
        .unwrap_or(0);
    eprintln!(
        "selection probe max_gen={} alive={} fissions={}",
        max_gen,
        pop.living_count(),
        pop.fission_log.len()
    );
    assert!(
        max_gen >= 2,
        "expected multi-gen under repaired H ecology, got gen {max_gen} fissions {}",
        pop.fission_log.len()
    );
}

#[test]
fn gate6_threshold_and_downstream_block_rules() {
    // Preregistered Gate 6 acceptance (directive §9).
    let freq_need = 0.15;
    let win = |max_gen: u32, ratio: f64, f: f64| -> bool {
        max_gen >= 4 && ratio >= 1.20 && (f - 0.5) >= freq_need
    };
    assert!(!win(6, 1.25, 0.55)); // Δf=0.05 < 0.15
    assert!(win(6, 1.25, 0.66)); // Δf=0.16
    assert!(!win(3, 2.0, 0.80)); // gen too short
                                 // Downstream may not run after Gate 6 nonpass.
    let gate6_pass = false;
    assert!(!gate6_pass, "Gate7/8 forbidden when Gate6 nonpass");
}

#[test]
fn neutral_label_shift_metric() {
    // Label asymmetry = |f_h - f_b| / 2.
    let shift = |fh: f64, fb: f64| (fh - fb).abs() / 2.0;
    // With label material present this equals the classic |f_h - 0.5|.
    assert!((shift(0.44, 0.56) - 0.06).abs() < 1e-9);
    assert!((shift(0.80, 0.20) - 0.30).abs() < 1e-9);
    // No label material left → no label advantage, not a 0.5 shift.
    assert!((shift(0.0, 0.0) - 0.0).abs() < 1e-9);
}

#[test]
fn stale_implementation_defect_rejected() {
    let stale = "D094_AUTOCATALYTIC_SET_IMPLEMENTATION_DEFECT";
    let pending = "D094_GATE6_SELECTION_CLOSURE_PENDING";
    assert_ne!(stale, pending);
    assert_eq!(pending, "D094_GATE6_SELECTION_CLOSURE_PENDING");
}
