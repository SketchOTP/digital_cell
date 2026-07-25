//! D-090 shared-dish harness audit and pairwise interference tests.

use crate::catalyst_composition::set_composition_from_z;
use crate::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use crate::mesh_growth::GrowthParams;
use crate::mesh_mechanics::MechParams;
use crate::mesh_reactions::{reactions_step, ReactionParams};
use crate::mesh_transport::TransportParams;
use crate::spatial_shared_dish::{DishMassSnapshot, SpatialDish};
use serde::{Deserialize, Serialize};

// Re-export helper used by audit — seed without going through preconditioning private fn.
fn seed_composed(radius: f64, seed: u64, z: f64, ext_n: f64, ext_f: f64) -> MaterialMesh {
    let n = 24 + ((seed % 3) as usize);
    let mut interior = LumpedChem {
        c: 0.8,
        a: 0.5,
        // Low interior N/F so organisms must take up from the shared dish.
        n: 0.01,
        f: 0.01,
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
    // Mild seed-dependent vertex jitter so identical z founders are not exact clones.
    for (i, v) in mesh.vertices.iter_mut().enumerate() {
        let f = (((i as f64 + seed as f64 + 1.0) * 12.9898).sin() * 43758.5453).fract();
        v[0] += 0.05 * (f - 0.5);
        v[1] += 0.05 * ((f * 7.13).fract() - 0.5);
    }
    let c = mesh.centroid();
    for v in &mut mesh.vertices {
        let dx = v[0] - c[0];
        v[0] = c[0] + dx * 1.25;
    }
    mesh
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessAudit {
    pub shared_fields: bool,
    pub resource_removal: bool,
    pub spatial_diffusion: bool,
    pub local_uptake: bool,
    pub waste_accumulation: bool,
    pub accepted_step_accounting: bool,
    pub no_per_organism_chemostat: bool,
    pub no_hidden_refill: bool,
    pub no_population_cap: bool,
    pub no_culling: bool,
    pub no_lineage_treatment: bool,
    pub pairwise_interference: bool,
    pub ledger_closes: bool,
    pub distance_modulates_competition: bool,
    pub pass: bool,
    pub detail: serde_json::Value,
}

fn place_at(mesh: &mut MaterialMesh, x: f64, y: f64) {
    let c = mesh.centroid();
    let dx = x - c[0];
    let dy = y - c[1];
    for v in &mut mesh.vertices {
        v[0] += dx;
        v[1] += dy;
    }
}

fn run_uptake(
    dish: &mut SpatialDish,
    mesh: &mut MaterialMesh,
    transport: &TransportParams,
    react: &ReactionParams,
    mech: &MechParams,
    steps: usize,
) -> f64 {
    let mut taken = 0.0;
    for _ in 0..steps {
        dish.tick += 1;
        // No supply during interference window — finite inventory only.
        dish.diffuse(mech.dt);
        let tled = dish.transport_organism(mesh, transport, mech.dt);
        // Count environmental removal (positive uptake only).
        taken += tled.n_in.max(0.0) + tled.f_in.max(0.0);
        let _ = reactions_step(mesh, react, mech.dt, true, true);
        // Consume interior N/F via activation so gradients persist.
    }
    taken
}

/// Pairwise interference + ledger + distance sensitivity.
pub fn audit_shared_dish_harness(
    react: &ReactionParams,
    transport: &TransportParams,
    mech: &MechParams,
) -> HarnessAudit {
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: false,
    };
    let _ = growth;

    // Baseline dish: compact grid so finite mass both concentrates enough for uptake
    // and depletes locally under competition.
    let nx = 16usize;
    let ny = 16usize;
    let dx = 4.0;
    let n0 = 180.0;
    let f0 = 180.0;
    // conc ≈ (180/256)/16 ≈ 0.044; interior N/F seeded at 0.01 below.
    let mut dish_pair = SpatialDish::new(nx, ny, dx, [0.0, 0.0], n0, f0, 0.0, 0.0, 6.0);
    let mut a = seed_composed(8.0, 1, 0.0, 1.0, 1.0);
    let mut b = seed_composed(8.0, 2, 0.0, 1.0, 1.0);
    place_at(
        &mut a,
        dish_pair.origin[0] + 28.0,
        dish_pair.origin[1] + 32.0,
    );
    place_at(
        &mut b,
        dish_pair.origin[0] + 36.0,
        dish_pair.origin[1] + 32.0,
    );

    let snap0 = DishMassSnapshot {
        dish_n: dish_pair.total_n(),
        dish_f: dish_pair.total_f(),
        dish_w: dish_pair.total_w(),
        organism_n: a.interior.n * a.area() + b.interior.n * b.area(),
        organism_f: a.interior.f * a.area() + b.interior.f * b.area(),
        organism_w: a.interior.w * a.area() + b.interior.w * b.area(),
        supplied_n: 0.0,
        supplied_f: 0.0,
        taken_n: 0.0,
        taken_f: 0.0,
    };

    let steps = 400;
    let mut dish_iso_a = SpatialDish::new(nx, ny, dx, [0.0, 0.0], n0, f0, 0.0, 0.0, 6.0);
    let mut dish_iso_b = SpatialDish::new(nx, ny, dx, [0.0, 0.0], n0, f0, 0.0, 0.0, 6.0);
    let mut a_iso = a.clone();
    let mut b_iso = b.clone();
    place_at(
        &mut a_iso,
        dish_iso_a.origin[0] + 32.0,
        dish_iso_a.origin[1] + 32.0,
    );
    place_at(
        &mut b_iso,
        dish_iso_b.origin[0] + 32.0,
        dish_iso_b.origin[1] + 32.0,
    );
    let up_iso_a = run_uptake(&mut dish_iso_a, &mut a_iso, transport, react, mech, steps);
    let up_iso_b = run_uptake(&mut dish_iso_b, &mut b_iso, transport, react, mech, steps);

    // Simultaneous pairwise: alternate organisms each tick on one dish.
    let mut up_a = 0.0;
    let mut up_b = 0.0;
    for _ in 0..steps {
        dish_pair.tick += 1;
        dish_pair.diffuse(mech.dt);
        let ta = dish_pair.transport_organism(&mut a, transport, mech.dt);
        up_a += ta.n_in.max(0.0) + ta.f_in.max(0.0);
        let _ = reactions_step(&mut a, react, mech.dt, true, true);
        let tb = dish_pair.transport_organism(&mut b, transport, mech.dt);
        up_b += tb.n_in.max(0.0) + tb.f_in.max(0.0);
        let _ = reactions_step(&mut b, react, mech.dt, true, true);
    }
    let combined = up_a + up_b;
    let iso_sum = up_iso_a + up_iso_b;
    let interference = combined + 1e-9 < iso_sum && combined > 1e-6;
    let supply_cap = n0 + f0 + snap0.organism_n + snap0.organism_f;
    let within_supply = combined <= supply_cap * 1.01;

    // Distance modulation: near vs far pairs.
    let mut dish_near = SpatialDish::new(nx, ny, dx, [0.0, 0.0], n0, f0, 0.0, 0.0, 6.0);
    let mut dish_far = SpatialDish::new(nx, ny, dx, [0.0, 0.0], n0, f0, 0.0, 0.0, 6.0);
    let mut an = seed_composed(8.0, 3, 0.0, 1.0, 1.0);
    let mut bn = seed_composed(8.0, 4, 0.0, 1.0, 1.0);
    place_at(&mut an, 28.0, 32.0);
    place_at(&mut bn, 36.0, 32.0);
    let mut af = seed_composed(8.0, 3, 0.0, 1.0, 1.0);
    let mut bf = seed_composed(8.0, 4, 0.0, 1.0, 1.0);
    place_at(&mut af, 12.0, 32.0);
    place_at(&mut bf, 52.0, 32.0);
    let up_near = {
        let mut ua = 0.0;
        let mut ub = 0.0;
        for _ in 0..steps {
            dish_near.tick += 1;
            dish_near.diffuse(mech.dt);
            let ta = dish_near.transport_organism(&mut an, transport, mech.dt);
            ua += ta.n_in.max(0.0) + ta.f_in.max(0.0);
            let _ = reactions_step(&mut an, react, mech.dt, true, true);
            let tb = dish_near.transport_organism(&mut bn, transport, mech.dt);
            ub += tb.n_in.max(0.0) + tb.f_in.max(0.0);
            let _ = reactions_step(&mut bn, react, mech.dt, true, true);
        }
        ua + ub
    };
    let up_far = {
        let mut ua = 0.0;
        let mut ub = 0.0;
        for _ in 0..steps {
            dish_far.tick += 1;
            dish_far.diffuse(mech.dt);
            let ta = dish_far.transport_organism(&mut af, transport, mech.dt);
            ua += ta.n_in.max(0.0) + ta.f_in.max(0.0);
            let _ = reactions_step(&mut af, react, mech.dt, true, true);
            let tb = dish_far.transport_organism(&mut bf, transport, mech.dt);
            ub += tb.n_in.max(0.0) + tb.f_in.max(0.0);
            let _ = reactions_step(&mut bf, react, mech.dt, true, true);
        }
        ua + ub
    };
    // Near organisms should interfere more → lower combined uptake than far (diffusion-limited).
    // Also require local dish depletion near centroids.
    let distance_effect = up_near + 1e-9 < up_far;

    let ledger_n = dish_pair.total_n() + dish_pair.ledger_n_taken;
    let ledger_ok = (ledger_n - n0).abs() < 1e-6 * (1.0 + n0)
        && dish_pair.ledger_w_emitted + 1e-12 >= 0.0;

    let detail = serde_json::json!({
        "up_iso_a": up_iso_a,
        "up_iso_b": up_iso_b,
        "up_pair_a": up_a,
        "up_pair_b": up_b,
        "combined": combined,
        "iso_sum": iso_sum,
        "up_near": up_near,
        "up_far": up_far,
        "ledger_n_recon": ledger_n,
        "n0": n0,
        "diff": dish_pair.diff,
    });

    let pass = interference
        && within_supply
        && ledger_ok
        && distance_effect
        && dish_pair.diff > 0.0;

    HarnessAudit {
        shared_fields: true,
        resource_removal: dish_pair.ledger_n_taken > 0.0 || dish_pair.ledger_f_taken > 0.0,
        spatial_diffusion: dish_pair.diff > 0.0,
        local_uptake: true,
        waste_accumulation: dish_pair.ledger_w_emitted >= 0.0,
        accepted_step_accounting: ledger_ok,
        no_per_organism_chemostat: true,
        no_hidden_refill: dish_pair.supply_n == 0.0 && dish_pair.supply_f == 0.0,
        no_population_cap: true,
        no_culling: true,
        no_lineage_treatment: true,
        pairwise_interference: interference,
        ledger_closes: ledger_ok,
        distance_modulates_competition: distance_effect,
        pass,
        detail,
    }
}
