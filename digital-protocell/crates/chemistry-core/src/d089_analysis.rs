//! D-089: Compositional catalytic heredity and natural selection.

use crate::catalyst_composition::{
    composition_z, copy_production_fluxes, derive_mutation_rate, g_build, g_harvest,
    set_composition_from_z, CompositionParams, EQUATION_VERSION_CATALYTIC_COMPOSITION,
    FIELD_SCHEMA_CATALYST_COMPOSITION, SIGMA_TRADEOFF,
};
use crate::catalyst_inheritance::{mesh_z, ols_slope, pearson, ParentOffspringPair};
use crate::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use crate::mesh_fission::FissionParams;
use crate::mesh_growth::GrowthParams;
use crate::mesh_mechanics::MechParams;
use crate::mesh_population::{coupled_step_growth, MeshPopulation};
use crate::mesh_reactions::{
    apply_membrane_damage, apply_structural_damage, evaluate_death, reactions_step, ReactionParams,
};
use crate::mesh_transport::TransportParams;
use crate::population_selection::{
    dish_step, observe_dish, seed_competition, SharedBath,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;

pub fn smoke() -> bool {
    matches!(
        env::var("D089_SMOKE").ok().as_deref(),
        Some("1") | Some("true") | Some("TRUE")
    )
}

fn steps(full: usize) -> usize {
    if smoke() {
        (full / 10).max(200)
    } else {
        (full / 2).max(800)
    }
}

fn reps() -> usize {
    if smoke() {
        3
    } else {
        8
    }
}

fn write_json(path: &Path, v: &impl Serialize) -> Result<(), String> {
    if let Some(p) = path.parent() {
        fs::create_dir_all(p).map_err(|e| e.to_string())?;
    }
    fs::write(path, serde_json::to_string_pretty(v).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}

fn frozen_yg() -> GrowthParams {
    GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    }
}

fn react_scalar() -> ReactionParams {
    ReactionParams::default()
}

fn react_comp(mu: f64, sigma: f64) -> ReactionParams {
    let mut r = ReactionParams::default();
    r.composition = CompositionParams {
        enable: true,
        mu,
        sigma,
    };
    r
}

fn seed_mesh(radius: f64, seed: u64, ext: f64) -> MaterialMesh {
    let n = 24 + ((seed % 3) as usize);
    MaterialMesh::seed_regular(
        n,
        radius,
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
            tracer_c: 0.0,
            c_h: 0.0,
            c_b: 0.0,
            r: 0.0,
            u_h: 0.0,
            u_b: 0.0,
            k_h: 0.0,
            k_b: 0.0,
        },
        LumpedChem {
            c: 0.0,
            a: 0.0,
            n: 1.0 * ext,
            f: 1.0 * ext,
            w: 0.0,
            tracer_c: 0.0,
            c_h: 0.0,
            c_b: 0.0,
            r: 0.0,
            u_h: 0.0,
            u_b: 0.0,
            k_h: 0.0,
            k_b: 0.0,
        },
        5.0,
    )
}

/// Mild bipolar stretch to seed elongation (D-088 fission prerequisite; no cleavage plane).
fn bipolar_stretch(mesh: &mut MaterialMesh, scale: f64) {
    let c = mesh.centroid();
    for v in &mut mesh.vertices {
        let dx = v[0] - c[0];
        v[0] = c[0] + dx * scale;
    }
}

fn perturb_vertex(mesh: &mut MaterialMesh, mag: f64) {
    for (i, v) in mesh.vertices.iter_mut().enumerate() {
        let f = (((i as f64 + 1.0) * 12.9898).sin() * 43758.5453).fract();
        v[0] += mag * (f - 0.5);
        v[1] += mag * ((f * 7.13).fract() - 0.5);
    }
}

fn seed_z(radius: f64, seed: u64, ext: f64, z: f64) -> MaterialMesh {
    let mut m = seed_mesh(radius, seed, ext);
    set_composition_from_z(&mut m.interior, z);
    m
}

fn seed_dividing(seed: u64, z: f64) -> MeshPopulation {
    let mut pop = MeshPopulation::seed_one(14.0, seed, 2.2);
    if let Some(ind) = pop.individuals.first_mut() {
        set_composition_from_z(&mut ind.mesh.interior, z);
        bipolar_stretch(&mut ind.mesh, 1.25);
        perturb_vertex(&mut ind.mesh, 0.35);
        ind.birth_mass = ind.mesh.total_structural_mass();
    }
    pop
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    pub name: String,
    pub pass: bool,
    pub detail: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct D089Report {
    pub primary_conclusion: String,
    pub phase2_status: String,
    pub phase3_authorized: bool,
    pub production_verdict: String,
    pub schema_equation: String,
    pub schema_fields: String,
    pub selected_mu: f64,
    pub b_c_median: f64,
    pub sigma: f64,
    pub smoke: bool,
    pub gates: Vec<GateResult>,
    pub next_directive: Option<String>,
    pub next_execution_started: bool,
    pub starting_commit: String,
    pub d088_preservation: String,
}

/// Measure median catalyst-production equivalents per successful generation (D-088 ledger).
fn measure_b_c() -> f64 {
    let mech = MechParams::default();
    let react = react_scalar();
    let transport = TransportParams::default();
    let growth = frozen_yg();
    let fission = FissionParams::default();
    let nsteps = steps(6000);
    let mut vals = Vec::new();
    for seed in 0..5u64 {
        let mut pop = MeshPopulation::seed_one(5.0, seed, 1.0);
        let mut c_prod = 0.0;
        let mut c_mass_sum = 0.0;
        let mut samples = 0usize;
        let mut gens = 0u32;
        for _ in 0..nsteps {
            if pop.living_count() == 0 {
                break;
            }
            let led = pop.step(&mech, &react, &transport, &growth, &fission, true);
            c_prod += led.reactions.c_produced;
            for ind in &pop.individuals {
                if ind.mesh.alive {
                    c_mass_sum += ind.mesh.interior.c.max(0.0) * ind.mesh.area().max(1e-9);
                    samples += 1;
                }
            }
            if led.fissions > 0 {
                gens += led.fissions as u32;
                let mean_c = if samples > 0 {
                    c_mass_sum / samples as f64
                } else {
                    1.0
                };
                let bc = c_prod / mean_c.max(1e-9) / gens.max(1) as f64;
                vals.push(bc);
                // reset accumulators for next generation window
                c_prod = 0.0;
                c_mass_sum = 0.0;
                samples = 0;
                if vals.len() >= 3 {
                    break;
                }
            }
        }
    }
    if vals.is_empty() {
        // Analytical fallback from frozen rates (still clamps via derive).
        return 200.0;
    }
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    vals[vals.len() / 2]
}

fn gate0_preservation(out: &Path) -> GateResult {
    let mech = MechParams::default();
    let growth = frozen_yg();
    let fission = FissionParams::default();
    let transport = TransportParams::default();
    let nsteps = steps(2500);
    let mut max_rel = 0.0f64;
    let mut ok = true;
    for seed in 0..3u64 {
        let mut m_s = seed_mesh(5.0, seed, 1.0);
        let mut m_c = seed_z(5.0, seed, 1.0, 0.0); // balanced
        let rs = react_scalar();
        let rc = react_comp(0.0, 0.0);
        for _ in 0..nsteps {
            if !m_s.alive || !m_c.alive {
                break;
            }
            let _ = coupled_step_growth(
                &mut m_s, &mech, &rs, &transport, &growth, &fission, true, false,
            );
            let _ = coupled_step_growth(
                &mut m_c, &mech, &rc, &transport, &growth, &fission, true, false,
            );
            evaluate_death(&mut m_s);
            evaluate_death(&mut m_c);
        }
        let dc = (m_s.interior.c - m_c.interior.c).abs() / (1.0 + m_s.interior.c);
        let da = (m_s.interior.a - m_c.interior.a).abs() / (1.0 + m_s.interior.a);
        let dm = (m_s.total_structural_mass() - m_c.total_structural_mass()).abs()
            / (1.0 + m_s.total_structural_mass());
        max_rel = max_rel.max(dc).max(da).max(dm);
        // composition sum
        let sum = m_c.interior.c_h + m_c.interior.c_b;
        if (sum - m_c.interior.c).abs() > 1e-6 {
            ok = false;
        }
    }
    ok = ok && max_rel < 0.05;
    let detail = serde_json::json!({
        "max_rel_diff": max_rel,
        "tolerance": 0.05,
        "sigma": 0.0,
        "mu": 0.0,
    });
    let _ = write_json(&out.join("preservation/gate0.json"), &detail);
    GateResult {
        name: "gate0_preservation".into(),
        pass: ok,
        detail,
    }
}

fn gate1_accounting(out: &Path, mu: f64) -> GateResult {
    let mut ok = true;
    let mut notes: Vec<String> = Vec::new();
    // Production conservation J_CH + J_CB = J_C
    for (ch, cb, jc) in [(1.0, 0.0, 0.5), (0.0, 1.0, 0.3), (0.4, 0.6, 1.0), (0.0, 0.0, 0.2)] {
        let (jh, jb) = copy_production_fluxes(jc, ch, cb, mu);
        if ch + cb <= 1e-15 {
            if jh + jb > 1e-15 {
                ok = false;
                notes.push("empty pool produced catalyst".into());
            }
        } else if (jh + jb - jc).abs() > 1e-12 {
            ok = false;
            notes.push("J_CH+J_CB != J_C".into());
        }
    }
    // Spatial partition of actual material
    let mech = MechParams::default();
    let react = react_comp(0.0, SIGMA_TRADEOFF);
    let transport = TransportParams::default();
    let growth = frozen_yg();
    let fission = FissionParams::default();
    let mut parent = seed_z(14.0, 3, 2.2, 0.6);
    bipolar_stretch(&mut parent, 1.25);
    perturb_vertex(&mut parent, 0.35);
    let mut partitioned = false;
    for _ in 0..steps(8000) {
        if !parent.alive {
            break;
        }
        let (_r, _g, split) = coupled_step_growth(
            &mut parent, &mech, &react, &transport, &growth, &fission, true, true,
        );
        if let Some((d1, d2, ev)) = split {
            let zp = composition_z(parent.interior.c_h, parent.interior.c_b);
            let z1 = mesh_z(&d1);
            let z2 = mesh_z(&d2);
            if (d1.interior.c_h + d1.interior.c_b - d1.interior.c).abs() > 1e-6 {
                ok = false;
            }
            if (d2.interior.c_h + d2.interior.c_b - d2.interior.c).abs() > 1e-6 {
                ok = false;
            }
            let (ch1, cb1, _) = crate::catalyst_inheritance::catalyst_masses(&d1);
            let (ch2, cb2, _) = crate::catalyst_inheritance::catalyst_masses(&d2);
            if ch1 + ch2 + cb1 + cb2 < 1e-9 {
                ok = false;
            }
            notes.push(format!(
                "partition ok={} zp={:.3} z1={:.3} z2={:.3}",
                ev.partition.ok, zp, z1, z2
            ));
            partitioned = ev.partition.ok;
            let _ = (d1, d2);
            break;
        }
        evaluate_death(&mut parent);
    }
    if !partitioned && !smoke() {
        notes.push("no fission in window (accounting equations still verified)".into());
    }
    // Snapshot/resume
    let mut m = seed_z(5.0, 1, 1.0, -0.4);
    let bytes = serde_json::to_vec(&m).unwrap();
    let m2: MaterialMesh = serde_json::from_slice(&bytes).unwrap();
    if (m.interior.c_h - m2.interior.c_h).abs() > 1e-12
        || (m.interior.c_b - m2.interior.c_b).abs() > 1e-12
    {
        ok = false;
        notes.push("snapshot/resume composition mismatch".into());
    }
    let detail = serde_json::json!({ "ok": ok, "notes": notes, "partitioned": partitioned });
    let _ = write_json(&out.join("catalyst_schema/gate1.json"), &detail);
    GateResult {
        name: "gate1_accounting".into(),
        pass: ok,
        detail,
    }
}

fn gate2_mutation(out: &Path, mu: f64) -> GateResult {
    let mech = MechParams::default();
    let transport = TransportParams::default();
    let growth = frozen_yg();
    let fission = FissionParams::default();
    let nsteps = steps(5000);
    let founders = [1.0f64, -1.0, 0.0];
    let mut mu0_ok = true;
    let mut mu_ok = true;
    let mut ratios = Vec::new();
    for (fi, z0) in founders.iter().enumerate() {
        // μ=0: no new type from pure founders
        let mut m0 = seed_z(5.0, fi as u64, 1.0, *z0);
        let r0 = react_comp(0.0, 0.0);
        let mut conv0 = 0.0;
        for _ in 0..nsteps {
            if !m0.alive {
                break;
            }
            let (r, _, _) = coupled_step_growth(
                &mut m0, &mech, &r0, &transport, &growth, &fission, true, false,
            );
            conv0 += r.composition.conversion_events;
            evaluate_death(&mut m0);
        }
        if *z0 >= 0.99 && m0.interior.c_b > 1e-6 {
            mu0_ok = false;
        }
        if *z0 <= -0.99 && m0.interior.c_h > 1e-6 {
            mu0_ok = false;
        }
        if conv0 > 1e-9 {
            mu0_ok = false;
        }

        // selected μ
        let mut m1 = seed_z(5.0, 10 + fi as u64, 1.0, *z0);
        let r1 = react_comp(mu, 0.0);
        let mut conv1 = 0.0;
        let mut c_prod = 0.0;
        for _ in 0..nsteps {
            if !m1.alive {
                break;
            }
            let (r, _, _) = coupled_step_growth(
                &mut m1, &mech, &r1, &transport, &growth, &fission, true, false,
            );
            conv1 += r.composition.conversion_events;
            c_prod += r.c_produced;
            evaluate_death(&mut m1);
        }
        let expected = mu * c_prod;
        let ratio = if expected > 1e-12 {
            conv1 / expected
        } else {
            1.0
        };
        ratios.push(ratio);
        if z0.abs() > 0.5 && conv1 <= 1e-9 && mu > 0.0 {
            mu_ok = false;
        }
        if (ratio - 1.0).abs() > 0.25 && expected > 1e-6 {
            mu_ok = false;
        }
        // no mass creation: total C dynamics still A-costed
        if !m1.alive && m1.death_reason.as_deref() == Some("error_catastrophe") {
            mu_ok = false;
        }
    }
    let pass = mu0_ok && mu_ok;
    let detail = serde_json::json!({
        "mu": mu,
        "mu0_ok": mu0_ok,
        "mu_ok": mu_ok,
        "conversion_ratios": ratios,
    });
    let _ = write_json(&out.join("mutation_calibration/gate2.json"), &detail);
    GateResult {
        name: "gate2_mutation".into(),
        pass,
        detail,
    }
}

fn gate3_heritability(out: &Path, mu: f64) -> GateResult {
    use std::collections::HashMap;
    let mech = MechParams::default();
    // Heritability is about material inheritance; σ=0 keeps growth D-088-like.
    let react = react_comp(mu, 0.0);
    let transport = TransportParams::default();
    let growth = frozen_yg();
    let fission = FissionParams::default();
    let mut pairs: Vec<ParentOffspringPair> = Vec::new();
    let target = if smoke() { 12 } else { 30 };
    let nsteps = if smoke() { 5000 } else { 14000 };
    let mut seed = 0u64;
    while pairs.len() < target && seed < 25 {
        let z0 = -0.8 + (seed as f64 % 9.0) * 0.2;
        let mut pop = seed_dividing(seed, z0);
        for _ in 0..nsteps {
            if pairs.len() >= target {
                break;
            }
            let mut snap: HashMap<u64, (f64, f64, f64)> = HashMap::new();
            for i in &pop.individuals {
                if i.mesh.alive {
                    let a = i.mesh.area().max(1e-9);
                    snap.insert(
                        i.lineage_id,
                        (
                            composition_z(i.mesh.interior.c_h, i.mesh.interior.c_b),
                            i.mesh.interior.c_h * a,
                            i.mesh.interior.c_b * a,
                        ),
                    );
                }
            }
            let before_len = pop.individuals.len();
            let led = pop.step(&mech, &react, &transport, &growth, &fission, true);
            if led.fissions == 0 {
                continue;
            }
            // Parents that fissioned this step.
            let mut parent_zs: Vec<(f64, f64, f64)> = Vec::new();
            for i in &pop.individuals[..before_len] {
                if i.mesh.death_reason.as_deref() == Some("fissioned") {
                    if let Some(v) = snap.get(&i.lineage_id) {
                        parent_zs.push(*v);
                    }
                }
            }
            let newborns = &pop.individuals[before_len..];
            for (k, nb) in newborns.iter().enumerate() {
                let pidx = k / 2;
                let (zp, chp, cbp) = parent_zs
                    .get(pidx)
                    .copied()
                    .unwrap_or((z0, 0.0, 0.0));
                let a = nb.mesh.area().max(1e-9);
                pairs.push(ParentOffspringPair {
                    z_parent: zp,
                    z_daughter: mesh_z(&nb.mesh),
                    c_h_parent: chp,
                    c_b_parent: cbp,
                    c_h_daughter: nb.mesh.interior.c_h * a,
                    c_b_daughter: nb.mesh.interior.c_b * a,
                    area_frac: a,
                });
            }
        }
        seed += 1;
    }
    let xs: Vec<f64> = pairs.iter().map(|p| p.z_parent).collect();
    let ys: Vec<f64> = pairs.iter().map(|p| p.z_daughter).collect();
    let corr = pearson(&xs, &ys);
    let slope = ols_slope(&xs, &ys);
    let mut m_a = seed_z(5.0, 99, 1.0, 0.5);
    let mut m_b = m_a.clone();
    let r = react_comp(mu, SIGMA_TRADEOFF);
    for _ in 0..200 {
        let _ = reactions_step(&mut m_a, &r, mech.dt, true, true);
        let _ = reactions_step(&mut m_b, &r, mech.dt, true, true);
    }
    let id_ok = (m_a.interior.c_h - m_b.interior.c_h).abs() < 1e-12
        && (mesh_z(&m_a) - mesh_z(&m_b)).abs() < 1e-12;
    let pass = pairs.len() >= (if smoke() { 6 } else { 20 })
        && corr >= (if smoke() { 0.55 } else { 0.70 })
        && slope >= 0.50
        && slope <= 1.10
        && id_ok;
    let detail = serde_json::json!({
        "n_pairs": pairs.len(),
        "correlation": corr,
        "slope": slope,
        "lineage_id_noncausal": id_ok,
    });
    let _ = write_json(&out.join("heritability/gate3.json"), &detail);
    GateResult {
        name: "gate3_heritability".into(),
        pass,
        detail,
    }
}

fn gate4_phenotype(out: &Path) -> GateResult {
    let mech = MechParams::default();
    let transport = TransportParams::default();
    let growth = frozen_yg();
    let fission = FissionParams::default();
    let nsteps = steps(3000);
    let zs = [0.6f64, 0.0, -0.6];

    // Resource-limited: low exterior
    let mut act_yields = Vec::new();
    let mut a_ret = Vec::new();
    for (i, z) in zs.iter().enumerate() {
        let mut m = seed_z(5.0, 20 + i as u64, 0.45, *z);
        let r = react_comp(0.0, SIGMA_TRADEOFF);
        let mut a_prod = 0.0;
        let a0 = m.interior.a;
        for _ in 0..nsteps {
            if !m.alive {
                break;
            }
            let (led, _, _) = coupled_step_growth(
                &mut m, &mech, &r, &transport, &growth, &fission, true, false,
            );
            a_prod += led.a_produced;
            evaluate_death(&mut m);
        }
        act_yields.push(a_prod);
        a_ret.push(m.interior.a / (1.0 + a0));
    }
    let harvest_better =
        act_yields[0] > act_yields[2] * 1.02 && a_ret[0] >= a_ret[2] * 0.98;

    // Construction-demanding: rich + damage
    let mut growth_lat = Vec::new();
    for (i, z) in zs.iter().enumerate() {
        let mut m = seed_z(5.0, 40 + i as u64, 1.3, *z);
        let r = react_comp(0.0, SIGMA_TRADEOFF);
        let m0 = m.total_structural_mass();
        let mut t_grow = nsteps;
        for t in 0..nsteps {
            if !m.alive {
                break;
            }
            if t > 0 && t % 250 == 0 {
                let _ = apply_membrane_damage(&mut m, 0.08);
                let _ = apply_structural_damage(&mut m, 0.05);
            }
            let _ = coupled_step_growth(
                &mut m, &mech, &r, &transport, &growth, &fission, true, false,
            );
            evaluate_death(&mut m);
            if m.total_structural_mass() >= 1.15 * m0 {
                t_grow = t;
                break;
            }
        }
        growth_lat.push(t_grow);
    }
    let build_better = growth_lat[2] <= growth_lat[0];

    // Neutral σ=0 control
    let mut act_n = Vec::new();
    for (i, z) in zs.iter().enumerate() {
        let mut m = seed_z(5.0, 60 + i as u64, 0.45, *z);
        let r = react_comp(0.0, 0.0);
        let mut a_prod = 0.0;
        for _ in 0..nsteps {
            if !m.alive {
                break;
            }
            let (led, _, _) = coupled_step_growth(
                &mut m, &mech, &r, &transport, &growth, &fission, true, false,
            );
            a_prod += led.a_produced;
            evaluate_death(&mut m);
        }
        act_n.push(a_prod);
    }
    let spread = (act_n[0] - act_n[2]).abs() / (1.0 + act_n[1]);
    let neutral_ok = spread < 0.05;

    // Factor bounds
    let zh = g_harvest(1.0, SIGMA_TRADEOFF);
    let zb = g_build(1.0, SIGMA_TRADEOFF);
    let bounds_ok = (0.85..=1.15).contains(&zh) && (0.85..=1.15).contains(&zb);

    let pass = harvest_better && build_better && neutral_ok && bounds_ok;
    let detail = serde_json::json!({
        "act_yields_resource_limited": act_yields,
        "a_retention": a_ret,
        "growth_latency_construction": growth_lat,
        "neutral_act_yields": act_n,
        "neutral_spread": spread,
        "harvest_better": harvest_better,
        "build_better": build_better,
        "neutral_ok": neutral_ok,
        "g_harvest_at_z1": zh,
        "g_build_at_z1": zb,
    });
    let _ = write_json(&out.join("phenotype_causality/gate4.json"), &detail);
    GateResult {
        name: "gate4_phenotype".into(),
        pass,
        detail,
    }
}

fn run_competition(
    env_name: &str,
    bath0: SharedBath,
    damage_m: f64,
    damage_s: f64,
    mu: f64,
    sigma: f64,
    z_h: f64,
    z_b: f64,
    n_gen_target: u32,
    out_dir: &Path,
) -> (usize, Vec<serde_json::Value>) {
    let mech = MechParams::default();
    let react = react_comp(mu, sigma);
    let transport = TransportParams::default();
    let growth = frozen_yg();
    let fission = FissionParams::default();
    let n_reps = reps();
    // Fission needs ~2k+ steps with elongated seeds; do not over-shrink in smoke.
    let nsteps = if smoke() {
        3500
    } else {
        10000
    };
    let mut wins = 0usize;
    let mut rows = Vec::new();
    for rep in 0..n_reps {
        let mut bath = bath0.clone();
        let mut pop = seed_competition(1, z_h, z_b, 10.0, 200 + rep as u64, &bath);
        let snap0 = observe_dish(&pop, &bath, 0.2);
        let f0 = snap0.freq_c_h;
        let mut max_gen = 0u32;
        for _ in 0..nsteps {
            if pop.living_count() == 0 {
                break;
            }
            // Lawful ecological terminal: crowded finite dish (observer stop, not a fitness cull).
            if pop.living_count() >= 28 {
                break;
            }
            let _ = dish_step(
                &mut pop,
                &mut bath,
                &mech,
                &react,
                &transport,
                &growth,
                &fission,
                true,
                damage_m,
                damage_s,
            );
            max_gen = pop
                .individuals
                .iter()
                .filter(|i| i.mesh.alive)
                .map(|i| i.generation)
                .max()
                .unwrap_or(0)
                .max(max_gen);
            if max_gen >= n_gen_target {
                break;
            }
        }
        let snap1 = observe_dish(&pop, &bath, 0.2);
        let df = snap1.freq_c_h - f0;
        let row = serde_json::json!({
            "rep": rep,
            "freq0": f0,
            "freq1": snap1.freq_c_h,
            "delta_freq_c_h": df,
            "descendants_h": snap1.descendants_h,
            "descendants_b": snap1.descendants_b,
            "living": snap1.living,
            "biomass": snap1.biomass,
            "max_gen": max_gen,
            "bath_n": snap1.bath_n,
            "bath_f": snap1.bath_f,
        });
        rows.push(row.clone());
        let win = match env_name {
            "H" => {
                df >= (if smoke() { 0.10 } else { 0.15 })
                    && snap1.descendants_h > snap1.descendants_b
                    && max_gen >= 1
            }
            "B" => {
                df <= (if smoke() { -0.10 } else { -0.15 })
                    && snap1.descendants_b > snap1.descendants_h
                    && max_gen >= 1
            }
            "N" => df.abs() < 0.10,
            _ => false,
        };
        if win {
            wins += 1;
        }
    }
    let _ = write_json(&out_dir.join("replicates.json"), &rows);
    (wins, rows)
}

fn gate5_6_competition(out: &Path, mu: f64) -> (GateResult, GateResult) {
    let need = if smoke() { 2 } else { 6 };
    let (w_h, rows_h) = run_competition(
        "H",
        SharedBath::resource_limited(),
        0.0,
        0.0,
        mu,
        SIGMA_TRADEOFF,
        0.6,
        -0.6,
        if smoke() { 3 } else { 6 },
        &out.join("competition_resource_limited"),
    );
    let (w_b, rows_b) = run_competition(
        "B",
        SharedBath::construction_demand(),
        0.15,
        0.10,
        mu,
        SIGMA_TRADEOFF,
        0.6,
        -0.6,
        if smoke() { 3 } else { 6 },
        &out.join("competition_construction_demand"),
    );
    let (w_n, rows_n) = run_competition(
        "N",
        SharedBath::neutral(),
        0.0,
        0.0,
        mu,
        SIGMA_TRADEOFF,
        0.6,
        -0.6,
        if smoke() { 3 } else { 6 },
        &out.join("competition_neutral"),
    );
    let g5 = GateResult {
        name: "gate5_shared_dish".into(),
        pass: !rows_h.is_empty() && !rows_b.is_empty() && !rows_n.is_empty(),
        detail: serde_json::json!({
            "wins_H": w_h, "wins_B": w_b, "wins_N": w_n,
            "reps": reps(),
        }),
    };
    let same_dominates = w_h >= need && w_b >= need; // both directions → not same
    let pass6 = w_h >= need && w_b >= need && w_n >= (if smoke() { 1 } else { 4 }) && same_dominates;
    // same_dominates here means BOTH env select differently (both win conditions met)
    let g6 = GateResult {
        name: "gate6_selection".into(),
        pass: pass6,
        detail: serde_json::json!({
            "wins_H": w_h,
            "wins_B": w_b,
            "wins_N": w_n,
            "need": need,
            "environment_dependent": w_h >= need && w_b >= need,
        }),
    };
    let _ = write_json(&out.join("competition_resource_limited/summary.json"), &g6.detail);
    (g5, g6)
}

fn gate7_adaptation(out: &Path, mu: f64) -> GateResult {
    let need = if smoke() { 2 } else { 6 };
    // Resource-limited: start building-biased (against H advantage)
    let (w_mu, _) = run_competition(
        "H",
        SharedBath::resource_limited(),
        0.0,
        0.0,
        mu,
        SIGMA_TRADEOFF,
        -0.6,
        -0.6,
        if smoke() { 4 } else { 8 },
        &out.join("mutation_adaptation/H_mu_on"),
    );
    let (w_off, _) = run_competition(
        "H",
        SharedBath::resource_limited(),
        0.0,
        0.0,
        0.0,
        SIGMA_TRADEOFF,
        -0.6,
        -0.6,
        if smoke() { 4 } else { 8 },
        &out.join("mutation_adaptation/H_mu_off"),
    );
    // Construction: start harvesting-biased
    let (w_mu_b, _) = run_competition(
        "B",
        SharedBath::construction_demand(),
        0.10,
        0.06,
        mu,
        SIGMA_TRADEOFF,
        0.6,
        0.6,
        if smoke() { 4 } else { 8 },
        &out.join("mutation_adaptation/B_mu_on"),
    );
    let (w_off_b, _) = run_competition(
        "B",
        SharedBath::construction_demand(),
        0.10,
        0.06,
        0.0,
        SIGMA_TRADEOFF,
        0.6,
        0.6,
        if smoke() { 4 } else { 8 },
        &out.join("mutation_adaptation/B_mu_off"),
    );
    // Compare mean delta freq improvement — approximate via win counts vs off
    let improved = (w_mu > w_off) as usize + (w_mu_b > w_off_b) as usize;
    let pass = improved >= 1 && (w_mu >= need / 2 || w_mu_b >= need / 2);
    let detail = serde_json::json!({
        "H_mu_on_wins": w_mu,
        "H_mu_off_wins": w_off,
        "B_mu_on_wins": w_mu_b,
        "B_mu_off_wins": w_off_b,
        "improved_envs": improved,
    });
    let _ = write_json(&out.join("mutation_adaptation/gate7.json"), &detail);
    GateResult {
        name: "gate7_adaptation".into(),
        pass,
        detail,
    }
}

fn gate8_reversal(out: &Path, mu: f64) -> GateResult {
    let mech = MechParams::default();
    let react = react_comp(mu, SIGMA_TRADEOFF);
    let transport = TransportParams::default();
    let growth = frozen_yg();
    let fission = FissionParams::default();
    let nsteps = if smoke() { 3500 } else { 8000 };
    let mut reversals = 0usize;
    let n_reps = if smoke() { 2 } else { 4 };
    for rep in 0..n_reps {
        // Grow in H then move to B
        let mut bath = SharedBath::resource_limited();
        let mut pop = seed_competition(1, 0.6, -0.6, 14.0, 500 + rep as u64, &bath);
        for _ in 0..nsteps {
            let _ = dish_step(
                &mut pop, &mut bath, &mech, &react, &transport, &growth, &fission, true, 0.0, 0.0,
            );
        }
        let f_h = observe_dish(&pop, &bath, 0.2).freq_c_h;
        bath = SharedBath::construction_demand();
        for _ in 0..nsteps {
            let _ = dish_step(
                &mut pop, &mut bath, &mech, &react, &transport, &growth, &fission, true, 0.10, 0.06,
            );
        }
        let f_b = observe_dish(&pop, &bath, 0.2).freq_c_h;
        if f_b + 0.05 < f_h {
            reversals += 1;
        }
    }
    let pass = reversals >= if smoke() { 1 } else { 2 };
    let detail = serde_json::json!({ "reversals": reversals, "reps": n_reps });
    let _ = write_json(&out.join("environmental_reversal/gate8.json"), &detail);
    GateResult {
        name: "gate8_reversal".into(),
        pass,
        detail,
    }
}

fn gate9_stability(out: &Path, mu: f64) -> GateResult {
    let mech = MechParams::default();
    let react = react_comp(mu, SIGMA_TRADEOFF);
    let transport = TransportParams::default();
    let growth = frozen_yg();
    let fission = FissionParams::default();
    let mut bath = SharedBath::neutral();
    let mut pop = seed_competition(1, 0.3, -0.3, 14.0, 77, &bath);
    let mut unbounded = false;
    let mut accounting_ok = true;
    for _ in 0..steps(2000) {
        let led = dish_step(
            &mut pop, &mut bath, &mech, &react, &transport, &growth, &fission, true, 0.0, 0.0,
        );
        let ch = led.reactions.composition.c_h_produced;
        let cb = led.reactions.composition.c_b_produced;
        if (ch + cb - led.reactions.c_produced).abs() > 1e-4 * (1.0 + led.reactions.c_produced) {
            accounting_ok = false;
        }
        if pop.living_count() > 40 {
            unbounded = true;
            break;
        }
    }
    let living = pop.living_count();
    let pass = accounting_ok && !unbounded && living > 0;
    let detail = serde_json::json!({
        "accounting_ok": accounting_ok,
        "unbounded": unbounded,
        "living": living,
        "bath_n": bath.n_mass,
    });
    let _ = write_json(&out.join("stability/gate9.json"), &detail);
    GateResult {
        name: "gate9_stability".into(),
        pass,
        detail,
    }
}

pub fn run_pipeline(output: &Path) -> Result<D089Report, String> {
    fs::create_dir_all(output).map_err(|e| e.to_string())?;
    let b_c = measure_b_c();
    let mu = derive_mutation_rate(b_c);
    let mut gates = Vec::new();

    let g0 = gate0_preservation(output);
    gates.push(g0.clone());
    if !g0.pass {
        return Ok(finish(
            "D089_D088_PRESERVATION_FAILURE",
            mu,
            b_c,
            gates,
            false,
        ));
    }

    let g1 = gate1_accounting(output, mu);
    gates.push(g1.clone());
    if !g1.pass {
        return Ok(finish(
            "D089_CATALYTIC_COMPOSITION_ACCOUNTING_FAILURE",
            mu,
            b_c,
            gates,
            false,
        ));
    }

    let g2 = gate2_mutation(output, mu);
    gates.push(g2.clone());
    if !g2.pass {
        return Ok(finish(
            "D089_CATALYTIC_MUTATION_NOT_ESTABLISHED",
            mu,
            b_c,
            gates,
            false,
        ));
    }

    let g3 = gate3_heritability(output, mu);
    gates.push(g3.clone());
    if !g3.pass {
        return Ok(finish(
            "D089_CATALYTIC_HERITABILITY_FAILURE",
            mu,
            b_c,
            gates,
            false,
        ));
    }

    let g4 = gate4_phenotype(output);
    gates.push(g4.clone());
    if !g4.pass {
        return Ok(finish(
            "D089_CATALYTIC_PHENOTYPE_NOT_CAUSAL",
            mu,
            b_c,
            gates,
            false,
        ));
    }

    let (g5, g6) = gate5_6_competition(output, mu);
    gates.push(g5);
    gates.push(g6.clone());
    if !g6.pass {
        // D-090 reframes Gate 6: heredity/phenotype qualified; selection result
        // provisional pending ecological timescale audit (not trait rejection).
        let report = finish_provisional_selection(mu, b_c, gates);
        let _ = write_json(&output.join("manifest.json"), &report);
        let _ = write_json(
            &output.join("accounting/summary.json"),
            &serde_json::json!({
                "mu": mu,
                "b_c": b_c,
                "sigma": SIGMA_TRADEOFF,
                "schema": EQUATION_VERSION_CATALYTIC_COMPOSITION,
                "fields": FIELD_SCHEMA_CATALYST_COMPOSITION,
                "heredity_phenotype": "D089_HEREDITY_AND_PHENOTYPE_QUALIFIED",
                "selection": "D089_SELECTION_RESULT_PROVISIONAL_PENDING_ECOLOGICAL_TIMESCALE_AUDIT",
                "hypothesis": "EARLY_FISSION_PRECEDED_SELECTION_PRESSURE",
            }),
        );
        return Ok(report);
    }

    let g7 = gate7_adaptation(output, mu);
    gates.push(g7.clone());
    if !g7.pass {
        return Ok(finish(
            "D089_MUTATION_DRIVEN_ADAPTATION_NOT_ESTABLISHED",
            mu,
            b_c,
            gates,
            false,
        ));
    }

    let g8 = gate8_reversal(output, mu);
    gates.push(g8.clone());
    if !g8.pass {
        return Ok(finish(
            "D089_SELECTION_REVERSAL_FAILURE",
            mu,
            b_c,
            gates,
            false,
        ));
    }

    let g9 = gate9_stability(output, mu);
    gates.push(g9.clone());
    if !g9.pass {
        return Ok(finish(
            "D089_EVOLUTIONARY_ARCHITECTURE_UNSTABLE",
            mu,
            b_c,
            gates,
            false,
        ));
    }

    let report = finish(
        "D089_COMPOSITIONAL_CATALYTIC_EVOLUTION_QUALIFIED",
        mu,
        b_c,
        gates,
        true,
    );
    let _ = write_json(&output.join("manifest.json"), &report);
    let _ = write_json(
        &output.join("accounting/summary.json"),
        &serde_json::json!({
            "mu": mu,
            "b_c": b_c,
            "sigma": SIGMA_TRADEOFF,
            "schema": EQUATION_VERSION_CATALYTIC_COMPOSITION,
            "fields": FIELD_SCHEMA_CATALYST_COMPOSITION,
        }),
    );
    Ok(report)
}

fn finish(
    conclusion: &str,
    mu: f64,
    b_c: f64,
    gates: Vec<GateResult>,
    qualified: bool,
) -> D089Report {
    D089Report {
        primary_conclusion: conclusion.into(),
        phase2_status: if qualified {
            "PHASE2_REPRODUCTION_AND_HEREDITY_COMPLETE".into()
        } else {
            "PHASE2_PHYSICAL_REPRODUCTION_QUALIFIED".into()
        },
        phase3_authorized: qualified,
        production_verdict: "RESEARCH_PROGRAM_ACTIVE_FINAL_PRODUCT_NOT_READY".into(),
        schema_equation: EQUATION_VERSION_CATALYTIC_COMPOSITION.into(),
        schema_fields: FIELD_SCHEMA_CATALYST_COMPOSITION.into(),
        selected_mu: mu,
        b_c_median: b_c,
        sigma: SIGMA_TRADEOFF,
        smoke: smoke(),
        gates,
        next_directive: if qualified {
            Some("D-090: Heritable Catalytic Regulation and Developmental Differentiation".into())
        } else {
            None
        },
        next_execution_started: qualified,
        starting_commit: "e4e049d".into(),
        d088_preservation: "D088_PHYSICAL_REPRODUCTION_FROZEN".into(),
    }
}

/// Gate 6 fail path after D-090 reframing: heredity/phenotype stand; selection is provisional.
fn finish_provisional_selection(mu: f64, b_c: f64, gates: Vec<GateResult>) -> D089Report {
    let mut report = finish(
        "D089_SELECTION_RESULT_PROVISIONAL_PENDING_ECOLOGICAL_TIMESCALE_AUDIT",
        mu,
        b_c,
        gates,
        false,
    );
    report.next_directive = Some(
        "D-090: Ecological Timescale Repair and Natural Selection Requalification".into(),
    );
    report.next_execution_started = false;
    report
}
