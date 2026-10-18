//! Native operator audits only. No identification of generic A with reference Sm.
use chemistry_core::material_mesh::{LumpedChem, MaterialMesh};
use chemistry_core::mesh_fission::{try_local_segment_fission, FissionParams};
use chemistry_core::mesh_growth::{growth_step, GrowthParams};
use chemistry_core::mesh_mechanics::{compute_forces, MechParams};
use chemistry_core::mesh_reactions::{reactions_step, ReactionParams};
use chemistry_core::mesh_transport::{transport_step, TransportParams};
use serde_json::{json, Value};

fn native_fixture() -> MaterialMesh {
    let mut m = MaterialMesh::seed_regular(
        6,
        1.,
        0.,
        0.,
        1.,
        0.7,
        LumpedChem {
            c: 1.,
            a: 10.,
            n: 10.,
            f: 10.,
            w: 0.1,
            ..Default::default()
        },
        LumpedChem {
            c: 0.05,
            a: 0.1,
            n: 2.,
            f: 2.,
            w: 0.,
            ..Default::default()
        },
        0.2,
    );
    m.stamp_maturation_coupled_schema();
    m
}

fn force_samples() -> Vec<Value> {
    let mut samples = Vec::new();
    let params = MechParams {
        gamma: 1.,
        k_s: 1.,
        kappa_b: 0.,
        k_pi: 0.,
        dt: 0.01,
    };
    for (h, y) in [(0., 1.), (0.0005, 0.001), (0.01, 0.002), (1., 0.2)] {
        for length in [0.00025, 0.002, 0.01, 0.5, 2., 8.] {
            let mut m = MaterialMesh::seed_regular(
                6,
                length,
                0.,
                0.,
                1.,
                0.7,
                LumpedChem::default(),
                LumpedChem::default(),
                0.,
            );
            m.stamp_maturation_coupled_schema();
            m.edges[0].m = h + y;
            m.edges[0].m_young = y;
            let forces = compute_forces(&m, &params);
            // Difference of two actual native force evaluations isolates the
            // selected edge while pressure/bending are exactly disabled.
            let mut without = m.clone();
            without.edges[0].ruptured = true;
            let removed = compute_forces(&without, &params);
            let actual_length = m.edge_length(0);
            let unit = [
                (m.vertices[1][0] - m.vertices[0][0]) / actual_length,
                (m.vertices[1][1] - m.vertices[0][1]) / actual_length,
            ];
            let measured =
                (forces[0][0] - removed[0][0]) * unit[0] + (forces[0][1] - removed[0][1]) * unit[1];
            samples.push(json!({"requested_length":length,"L":actual_length,"mature":m.mature_structural_mass(0),
                "young":m.young_structural_mass(0),"rho":m.rho_s,"k_s":params.k_s,"r_native":m.rest_length(0),
                "phi_native":m.mature_structural_fraction(0),"measured_tension":measured,
                "native_forces":forces,"without_selected_edge_forces":removed,"mesh":m,
                "kind":"externally_prepared_force_difference_audit_not_organism_trajectory"}));
        }
    }
    samples
}

fn native_operators() -> Value {
    let mut transport = native_fixture();
    let transport_before = transport.clone();
    let transport_params = TransportParams::default();
    let transport_ledger = transport_step(&mut transport, &transport_params, 0.01);
    let react = ReactionParams::default();
    let growth_params = GrowthParams::default();
    let mut growth = native_fixture();
    let growth_before = growth.clone();
    let growth_ledger = growth_step(&mut growth, &react, &growth_params, 0.01);
    let mut maturation = native_fixture();
    for e in &mut maturation.edges {
        e.m_young = 0.2 * e.m;
    }
    let maturation_before = maturation.clone();
    let reaction_ledger = reactions_step(&mut maturation, &react, 0.01, true, true);
    let points = vec![
        [-3., -2.],
        [-1., -2.],
        [-0.5, -0.2],
        [0.5, -0.2],
        [1., -2.],
        [3., -2.],
        [3., 2.],
        [1., 2.],
        [0.5, 0.2],
        [-0.5, 0.2],
        [-1., 2.],
        [-3., 2.],
    ];
    let mut parent = MaterialMesh::seed_regular(
        points.len(),
        3.,
        0.,
        0.,
        1.,
        0.5,
        LumpedChem {
            a: 100.,
            c: 1.,
            ..Default::default()
        },
        LumpedChem::default(),
        0.,
    );
    parent.stamp_maturation_coupled_schema();
    parent.vertices = points;
    for i in 0..parent.n() {
        // Explicit external compression preparation, not a chemical-work
        // claim. V4 signed load is 1/1.3 - 1, strictly beyond the unchanged
        // 0.15 stress threshold; the native proximity rule also remains intact.
        parent.edges[i].m = 1.3 * parent.rho_s * parent.edge_length(i);
    }
    let preparation = json!({"kind":"externally_prepared_compressed_native_V4_dumbbell_audit",
        "mature_rest_length_ratio":1.3,"young_fraction":0.,"not_new_owner_hexagon_cut":true,
        "load_bearing_strains":(0..parent.n()).map(|i|parent.load_bearing_strain(i)).collect::<Vec<_>>(),
        "boundary":"external prestrain preparation; no finite chemical work ownership or measured biological mechanism claim"});
    let fission_params = FissionParams::default();
    let fission = try_local_segment_fission(&parent, &fission_params);
    let fission_audit = match fission {
        Some((a, b, event)) => {
            json!({"success":true,"before":parent,"daughters":[a,b],"event":event,"params":fission_params,"preparation":preparation})
        }
        None => {
            json!({"success":false,"before":parent,"params":fission_params,"preparation":preparation,"reason":"no lawful native segment proposal"})
        }
    };
    json!({"native_transport":{"before":transport_before,"after":transport,"ledger":transport_ledger,"params":transport_params,"dt":0.01,
        "boundary":"generic N/F/C/A/W, exterior not depleted; no finite-reference transport integration claim"},
        "native_growth":{"before":growth_before,"after":growth,"ledger":growth_ledger,"reaction_params":react,"growth_params":growth_params,"dt":0.01,
        "boundary":"generic native A/yield/young-material proposal; no identification of A with Sm"},
        "native_maturation":{"before":maturation_before,"after":maturation,"ledger":reaction_ledger,"params":react,"dt":0.01,
        "boundary":"actual full native reaction call includes maturation, turnover and other enabled channels; not isolated kinetics"},
        "native_fission":fission_audit})
}

fn main() -> Result<(), String> {
    if std::env::args().len() != 1 {
        return Err("native audit accepts no arguments".into());
    }
    let result = json!({"scope":"actual native force and four-operator audit probes only",
        "new_owner_integrated_biology":false,"physical_fusion":false,
        "force_samples":force_samples(),"operators":native_operators()});
    println!(
        "{}",
        serde_json::to_string(&result).map_err(|e| e.to_string())?
    );
    Ok(())
}
