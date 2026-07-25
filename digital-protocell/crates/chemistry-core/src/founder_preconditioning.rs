//! D-090 founder preconditioning and inherited-reserve audit (observer/ecology only).

use crate::catalyst_composition::set_composition_from_z;
use crate::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use crate::mesh_fission::FissionParams;
use crate::mesh_growth::{growth_step, GrowthParams};
use crate::mesh_mechanics::{mechanics_step, remesh, MechParams};
use crate::mesh_population::MeshIndividual;
use crate::mesh_reactions::{evaluate_death, reactions_step, ReactionParams};
use crate::mesh_transport::TransportParams;
use crate::spatial_shared_dish::SpatialDish;
use serde::{Deserialize, Serialize};

/// Catalyst turnover e-folding time ≈ 1/k_c_turn (default k_c_turn=0.01 → 100).
pub fn catalyst_turnover_horizon(react: &ReactionParams) -> f64 {
    1.0 / react.k_c_turn.max(1e-9)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FounderAudit {
    pub structural_mass: f64,
    pub perimeter: f64,
    pub bound_membrane: f64,
    pub free_membrane: f64,
    pub membrane_reserve_total: f64,
    pub total_catalyst: f64,
    pub c_h: f64,
    pub c_b: f64,
    pub z: f64,
    pub a_mass: f64,
    pub interior_n: f64,
    pub interior_f: f64,
    pub birth_mass: f64,
    pub age_since_fission: f64,
    pub predicted_reserve_growth: f64,
    pub fission_growth_needed: f64,
    pub reserve_fraction_of_fission: f64,
}

pub fn audit_founder(ind: &MeshIndividual, age: f64, y_g: f64) -> FounderAudit {
    let m = &ind.mesh;
    let area = m.area().max(1e-9);
    let structural = m.total_structural_mass();
    let a_mass = m.interior.a.max(0.0) * area;
    // Upper bound: all inherited A converted to structural mass at yield y_g.
    let predicted = a_mass * y_g;
    let needed = (1.35 * ind.birth_mass - structural).max(0.0);
    FounderAudit {
        structural_mass: structural,
        perimeter: m.perimeter(),
        bound_membrane: m.total_bound_membrane(),
        free_membrane: m.free_l.max(0.0),
        membrane_reserve_total: m.total_membrane(),
        total_catalyst: m.interior.c.max(0.0) * area,
        c_h: m.interior.c_h.max(0.0) * area,
        c_b: m.interior.c_b.max(0.0) * area,
        z: crate::catalyst_composition::composition_z(m.interior.c_h, m.interior.c_b),
        a_mass,
        interior_n: m.interior.n.max(0.0) * area,
        interior_f: m.interior.f.max(0.0) * area,
        birth_mass: ind.birth_mass,
        age_since_fission: age,
        predicted_reserve_growth: predicted,
        fission_growth_needed: needed,
        reserve_fraction_of_fission: if needed > 1e-12 {
            predicted / needed
        } else {
            0.0
        },
    }
}

/// Measure actual structural growth with external N/F uptake disabled (inherited reserves only).
pub fn measure_reserve_funded_growth(
    mesh0: &MaterialMesh,
    react: &ReactionParams,
    growth: &GrowthParams,
    mech: &MechParams,
    max_steps: usize,
) -> f64 {
    let mut mesh = mesh0.clone();
    // Starve exterior so transport cannot fund new A from environment.
    mesh.exterior.n = 0.0;
    mesh.exterior.f = 0.0;
    let m0 = mesh.total_structural_mass();
    for _ in 0..max_steps {
        if !mesh.alive {
            break;
        }
        // No transport — only consume inherited A/N/F.
        let _ = reactions_step(&mut mesh, react, mech.dt, true, true);
        let _ = growth_step(&mut mesh, react, growth, mech.dt);
        mechanics_step(&mut mesh, mech);
        remesh(&mut mesh);
        evaluate_death(&mut mesh);
        if mesh.interior.a < 1e-6 {
            break;
        }
    }
    (mesh.total_structural_mass() - m0).max(0.0)
}

fn matched_within(a: f64, b: f64, tol: f64) -> bool {
    let den = a.abs().max(b.abs()).max(1e-9);
    (a - b).abs() / den <= tol
}

pub fn founders_matched(audits: &[FounderAudit], tol: f64) -> bool {
    if audits.len() < 2 {
        return true;
    }
    // Match physical endowment only — composition (z / C_H / C_B) may differ by design.
    let keys: &[(fn(&FounderAudit) -> f64, &str)] = &[
        (|a| a.structural_mass, "structural_mass"),
        (|a| a.perimeter, "perimeter"),
        (|a| a.total_catalyst, "total_catalyst"),
        (|a| a.a_mass, "a_mass"),
        (|a| a.membrane_reserve_total, "membrane_reserve"),
        (|a| a.bound_membrane, "bound_membrane"),
        (|a| a.interior_n, "interior_n"),
        (|a| a.interior_f, "interior_f"),
        (|a| a.age_since_fission, "age"),
    ];
    for (get, _) in keys {
        let vals: Vec<f64> = audits.iter().map(|a| get(a)).collect();
        let mean = vals.iter().sum::<f64>() / vals.len() as f64;
        if vals.iter().any(|v| !matched_within(*v, mean, tol)) {
            return false;
        }
    }
    true
}

fn bipolar_elongate(mesh: &mut MaterialMesh, scale: f64) {
    let c = mesh.centroid();
    for v in &mut mesh.vertices {
        let dx = v[0] - c[0];
        v[0] = c[0] + dx * scale;
    }
}

fn seed_raw(radius: f64, _seed: u64, z: f64, ext_n: f64, ext_f: f64) -> MaterialMesh {
    // Fixed vertex count so founder endowment matching is not seed-topology confounded.
    let n = 24;
    let mut interior = LumpedChem {
        c: 0.8,
        a: 0.5,
        n: 0.4,
        f: 0.4,
        w: 0.1,
        tracer_c: 0.0,
        c_h: 0.0,
        c_b: 0.0,
        r: 0.0,
            u_h: 0.0,
            u_b: 0.0,
            k_h: 0.0,
            k_b: 0.0,
            q_k: 0.0,
            q_e: 0.0,
            k_a: 0.0,
            k_r: 0.0,
            k_node_b: 0.0,
        };
    set_composition_from_z(&mut interior, z);
    let exterior = LumpedChem {
        c: 0.0,
        a: 0.0,
        n: ext_n,
        f: ext_f,
        w: 0.0,
        tracer_c: 0.0,
        c_h: 0.0,
        c_b: 0.0,
        r: 0.0,
            u_h: 0.0,
            u_b: 0.0,
            k_h: 0.0,
            k_b: 0.0,
            q_k: 0.0,
            q_e: 0.0,
            k_a: 0.0,
            k_r: 0.0,
            k_node_b: 0.0,
        };
    let mut mesh = MaterialMesh::seed_regular(
        n,
        radius,
        40.0,
        40.0,
        DEFAULT_RHO_S,
        0.7,
        interior,
        exterior,
        5.0,
    );
    bipolar_elongate(&mut mesh, 1.25);
    mesh
}

/// Common maintenance environment: moderate shared dish, mutation off, no growth surplus target.
pub fn maintenance_dish(n0: f64, f0: f64, supply: f64) -> SpatialDish {
    SpatialDish::new(8, 8, 2.5, [0.0, 0.0], n0, f0, supply, supply, 3.0)
}

/// Precondition a founder under maintenance ecology until reserves < 10% of fission growth need
/// and at least one catalyst-turnover horizon has elapsed. Mutation must be off (`react.composition.mu=0`
/// or caller passes mu=0 params). Does not overwrite composition fields.
pub fn precondition_founder(
    z: f64,
    seed: u64,
    clade: i8,
    react: &ReactionParams,
    growth: &GrowthParams,
    mech: &MechParams,
    transport: &TransportParams,
    fission: &FissionParams,
) -> (MeshIndividual, f64, FounderAudit) {
    let _ = fission;
    let mut dish = maintenance_dish(600.0, 600.0, 28.0);
    let mut mesh = seed_raw(10.0, seed, z, dish.conc_at(40.0, 40.0).0, dish.conc_at(40.0, 40.0).1);
    // Place near dish center.
    let c = mesh.centroid();
    let tx = dish.origin[0] + dish.nx as f64 * dish.dx * 0.5 - c[0];
    let ty = dish.origin[1] + dish.ny as f64 * dish.dx * 0.5 - c[1];
    // Identical placement at dish center — no jitter (local uptake gradients would
    // break 5% endowment matching across otherwise identical founders).
    for v in &mut mesh.vertices {
        v[0] += tx;
        v[1] += ty;
    }
    let horizon = catalyst_turnover_horizon(react);
    let mut age = 0.0;
    let target_age = horizon * 1.25; // ≥1 turnover horizon; avoid over-starvation
    let max_steps = ((target_age / mech.dt).ceil() as usize).max(500);
    let mut steps = 0usize;

    while steps < max_steps {
        dish.tick += 1;
        dish.supply_step(mech.dt);
        dish.diffuse(mech.dt);
        let _ = dish.transport_organism(&mut mesh, transport, mech.dt);
        let _ = reactions_step(&mut mesh, react, mech.dt, true, true);
        let _ = growth_step(&mut mesh, react, growth, mech.dt);
        mechanics_step(&mut mesh, mech);
        remesh(&mut mesh);
        evaluate_death(&mut mesh);
        age += mech.dt;
        steps += 1;
        if !mesh.alive {
            break;
        }
    }
    // Longer A spend-down to drive predicted reserve fraction below 10%.
    for _ in 0..2500 {
        if !mesh.alive || mesh.interior.c < 0.2 {
            break;
        }
        mesh.exterior.n = 0.05;
        mesh.exterior.f = 0.05;
        let _ = reactions_step(&mut mesh, react, mech.dt, true, true);
        let _ = growth_step(&mut mesh, react, growth, mech.dt);
        evaluate_death(&mut mesh);
        age += mech.dt;
        let birth_now = mesh.total_structural_mass();
        let predicted = mesh.interior.a.max(0.0) * mesh.area() * growth.y_g;
        let needed = (0.35 * birth_now).max(1e-12);
        if predicted / needed < 0.10 {
            break;
        }
    }
    let birth_now = mesh.total_structural_mass();
    let ind = MeshIndividual {
        mesh,
        lineage_id: 0,
        generation: 0,
        birth_mass: birth_now,
        clade,
    };
    let audit = audit_founder(&ind, age, growth.y_g);
    (ind, age, audit)
}
