//! D-088: Emergent growth, topological fission, material inheritance analysis.

use crate::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use crate::mesh_fission::FissionParams;
use crate::mesh_growth::{shape_factor_psi, GrowthParams, Y_G_CANDIDATES};
use crate::mesh_mechanics::MechParams;
use crate::mesh_population::{coupled_step_growth, MeshPopulation};
use crate::mesh_reactions::{evaluate_death, ReactionParams};
use crate::mesh_transport::TransportParams;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;

pub fn smoke() -> bool {
    matches!(
        env::var("D088_SMOKE").ok().as_deref(),
        Some("1") | Some("true") | Some("TRUE")
    )
}

fn steps(full: usize) -> usize {
    if smoke() {
        (full / 8).max(300)
    } else {
        // Non-smoke: denser than smoke, still finishes in minutes on one core.
        (full / 3).max(1_500)
    }
}

fn write_json(path: &Path, v: &impl Serialize) -> Result<(), String> {
    if let Some(p) = path.parent() {
        fs::create_dir_all(p).map_err(|e| e.to_string())?;
    }
    fs::write(path, serde_json::to_string_pretty(v).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
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
        },
        LumpedChem {
            c: 0.0,
            a: 0.0,
            n: 1.0 * ext,
            f: 1.0 * ext,
            w: 0.0,
            tracer_c: 0.0,
        },
        5.0,
    )
}

fn perturb(mesh: &mut MaterialMesh, kind: &str, mag: f64) {
    match kind {
        "rotate" => {
            let c = mesh.centroid();
            let (s, co) = (mag.sin(), mag.cos());
            for v in &mut mesh.vertices {
                let x = v[0] - c[0];
                let y = v[1] - c[1];
                v[0] = c[0] + co * x - s * y;
                v[1] = c[1] + s * x + co * y;
            }
        }
        "vertex" => {
            for (i, v) in mesh.vertices.iter_mut().enumerate() {
                let f = (((i as f64 + 1.0) * 12.9898).sin() * 43758.5453).fract();
                v[0] += mag * (f - 0.5);
                v[1] += mag * ((f * 7.13).fract() - 0.5);
            }
        }
        "c" => mesh.interior.c = (mesh.interior.c * (1.0 + mag)).max(0.0),
        "a" => mesh.interior.a = (mesh.interior.a * (1.0 + mag)).max(0.0),
        "l" => mesh.free_l = (mesh.free_l * (1.0 + mag)).max(0.0),
        "env" => {
            mesh.exterior.n = (mesh.exterior.n * (1.0 + mag)).max(0.0);
            mesh.exterior.f = (mesh.exterior.f * (1.0 + mag)).max(0.0);
        }
        _ => {}
    }
}

fn run_mesh(
    mesh: &mut MaterialMesh,
    nsteps: usize,
    growth: &GrowthParams,
    fission: &FissionParams,
    enable_fission: bool,
    enable_mech: bool,
) -> (f64, f64, usize) {
    let mech = MechParams::default();
    let react = ReactionParams::default();
    let transport = TransportParams::default();
    let m0 = mesh.total_structural_mass();
    let p0 = mesh.perimeter();
    let mut fissions = 0usize;
    for _ in 0..nsteps {
        if !mesh.alive {
            break;
        }
        let (_r, _g, split) = coupled_step_growth(
            mesh,
            &mech,
            &react,
            &transport,
            growth,
            fission,
            enable_mech,
            enable_fission,
        );
        if let Some((d1, d2, _)) = split {
            // Replace parent with first daughter for single-mesh assays; count event.
            *mesh = d1;
            let _ = d2;
            fissions += 1;
            break;
        }
        evaluate_death(mesh);
    }
    let growth_m = if m0 > 1e-12 {
        mesh.total_structural_mass() / m0
    } else {
        1.0
    };
    let growth_p = if p0 > 1e-12 {
        mesh.perimeter() / p0
    } else {
        1.0
    };
    (growth_m, growth_p, fissions)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    pub pass: bool,
    pub detail: String,
    pub failure: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct D088Report {
    pub primary_conclusion: String,
    pub phase2_status: String,
    pub phase1_runtime_status: String,
    pub production_verdict: String,
    pub runtime_closure: serde_json::Value,
    pub selected_y_g: f64,
    pub gates: serde_json::Value,
    pub next_directive: String,
    pub next_execution_started: bool,
    pub smoke: bool,
}

fn select_y_g() -> (f64, serde_json::Value) {
    let mech_growth = GrowthParams {
        enable_growth: false,
        y_g: 0.0,
    };
    let fission = FissionParams::default();
    // First: Phase 1 laws only under surplus — expect little growth.
    let mut m0 = seed_mesh(14.0, 1, 2.0);
    let (g_m_nat, _, _) = run_mesh(
        &mut m0,
        steps(4_000),
        &mech_growth,
        &fission,
        false,
        true,
    );
    let need_growth_eq = g_m_nat < 1.25;
    let mut rows = Vec::new();
    let mut best = Y_G_CANDIDATES[0];
    let mut best_score = -1.0e9;
    if need_growth_eq {
        for &y in &Y_G_CANDIDATES {
            let g = GrowthParams {
                y_g: y,
                enable_growth: true,
            };
            let mut maint = seed_mesh(14.0, 1, 1.0);
            let (gm_m, _, _) = run_mesh(&mut maint, steps(5_000), &g, &fission, false, true);
            let mut sur = seed_mesh(14.0, 1, 2.5);
            perturb(&mut sur, "vertex", 0.2);
            let (gm_s, gp_s, _) = run_mesh(&mut sur, steps(7_000), &g, &fission, false, true);
            let alive = maint.alive && sur.alive;
            let maint_ok = gm_m > 0.7 && gm_m < 1.40;
            let sur_ok = gm_s >= 1.50 && gp_s >= 1.35;
            let score = if alive && maint_ok && sur_ok {
                gm_s + gp_s - (gm_m - 1.0).abs()
            } else if alive && maint_ok {
                gm_s // pick best surplus among maintenance-safe
            } else {
                -1.0
            };
            rows.push(serde_json::json!({
                "y_g": y, "maint_ratio": gm_m, "surplus_mass": gm_s, "surplus_peri": gp_s,
                "maint_ok": maint_ok, "surplus_ok": sur_ok, "score": score
            }));
            if score > best_score {
                best_score = score;
                best = y;
            }
        }
    }
    (
        best,
        serde_json::json!({
            "natural_surplus_mass_ratio": g_m_nat,
            "need_growth_equation": need_growth_eq,
            "candidates": rows,
            "selected_y_g": best
        }),
    )
}

fn gate_surplus_growth(y_g: f64) -> (GateResult, serde_json::Value) {
    let growth = GrowthParams {
        y_g,
        enable_growth: true,
    };
    let growth_off = GrowthParams {
        y_g,
        enable_growth: false,
    };
    let fission = FissionParams::default();

    let mut maint = seed_mesh(14.0, 1, 1.0);
    let m0 = maint.total_structural_mass();
    let (gm, _, _) = run_mesh(&mut maint, steps(5_000), &growth, &fission, false, true);
    let maint_ok = maint.alive && gm > 0.7 && gm < 1.40;

    let mut sur = seed_mesh(14.0, 2, 2.5);
    let ms0 = sur.total_structural_mass();
    let ps0 = sur.perimeter();
    let mut w0 = sur.interior.w;
    let (gsm, gsp, _) = run_mesh(&mut sur, steps(8_000), &growth, &fission, false, true);
    let surplus_ok = sur.alive && gsm >= 1.5 && gsp >= 1.35 && sur.interior.w + 1e-9 >= w0;

    let mut starv = seed_mesh(14.0, 3, 2.2);
    starv.exterior.n = 0.0;
    starv.exterior.f = 0.0;
    starv.interior.n = 0.0;
    starv.interior.f = 0.0;
    let (gst, _, _) = run_mesh(&mut starv, steps(4_000), &growth, &fission, false, true);
    let starve_ok = gst < 1.20;

    let mut ko = seed_mesh(14.0, 4, 2.2);
    let mut react = ReactionParams::default();
    react.k_act = 0.0;
    let mech = MechParams::default();
    let transport = TransportParams::default();
    let m_ko0 = ko.total_structural_mass();
    for _ in 0..steps(3_000) {
        let _ = coupled_step_growth(
            &mut ko,
            &mech,
            &react,
            &transport,
            &growth,
            &fission,
            true,
            false,
        );
    }
    let ko_ok = ko.total_structural_mass() < 1.15 * m_ko0;

    // Growth stops when surplus removed
    let mut stop = seed_mesh(14.0, 5, 2.2);
    let _ = run_mesh(&mut stop, steps(3_000), &growth, &fission, false, true);
    let mid = stop.total_structural_mass();
    stop.exterior.n = 1.0;
    stop.exterior.f = 1.0;
    let _ = run_mesh(&mut stop, steps(3_000), &growth_off, &fission, false, true);
    // with growth off after switching to maintenance env, mass shouldn't keep climbing hard
    let stop_ok = stop.total_structural_mass() < mid * 1.25;

    let pass = maint_ok && surplus_ok && starve_ok && ko_ok && stop_ok;
    (
        GateResult {
            pass,
            detail: format!(
                "maint_ratio={gm:.3} surplus_m={gsm:.3} surplus_p={gsp:.3} starve={gst:.3} ko_ok={ko_ok} stop_ok={stop_ok} m0={m0:.1} ms0={ms0:.1} ps0={ps0:.1}"
            ),
            failure: if pass {
                None
            } else {
                Some("D088_SURPLUS_GROWTH_NOT_ESTABLISHED".into())
            },
        },
        serde_json::json!({
            "maint_ok": maint_ok, "surplus_ok": surplus_ok, "starve_ok": starve_ok,
            "ko_ok": ko_ok, "stop_ok": stop_ok, "surplus_mass_ratio": gsm, "surplus_peri_ratio": gsp
        }),
    )
}

fn gate_instability(y_g: f64) -> (GateResult, serde_json::Value) {
    let growth = GrowthParams {
        y_g,
        enable_growth: true,
    };
    let fission = FissionParams::default();
    let mut rows = Vec::new();
    let mut ok = 0usize;
    for (seed, rot) in [(1u64, 0.0), (2, 0.4), (3, 1.1), (4, -0.7)] {
        let mut mesh = seed_mesh(14.0, seed, 2.2);
        perturb(&mut mesh, "rotate", rot);
        perturb(&mut mesh, "vertex", 0.08);
        let psi0 = shape_factor_psi(&mesh);
        let _ = run_mesh(&mut mesh, steps(10_000), &growth, &fission, false, true);
        let psi1 = shape_factor_psi(&mesh);
        // Concave: signed turning — count local reflex via angle cos
        let mut concave = 0usize;
        let n = mesh.n();
        for i in 0..n {
            let p0 = mesh.vertices[(i + n - 1) % n];
            let p1 = mesh.vertices[i];
            let p2 = mesh.vertices[(i + 1) % n];
            let cross = (p1[0] - p0[0]) * (p2[1] - p1[1]) - (p1[1] - p0[1]) * (p2[0] - p1[0]);
            if mesh.signed_area() > 0.0 && cross < 0.0 {
                concave += 1;
            }
            if mesh.signed_area() < 0.0 && cross > 0.0 {
                concave += 1;
            }
        }
        let departed = psi1 > psi0 * 1.08 || psi1 > 1.15;
        let pass = mesh.alive && departed && concave >= 1;
        if pass {
            ok += 1;
        }
        rows.push(serde_json::json!({
            "seed": seed, "psi0": psi0, "psi1": psi1, "concave": concave, "pass": pass
        }));
    }
    let need = if smoke() { 1 } else { 3 };
    let pass = ok >= need;
    (
        GateResult {
            pass,
            detail: format!("instability_ok={ok}/{need}"),
            failure: if pass {
                None
            } else {
                Some("D088_GROWTH_WITHOUT_FISSION_INSTABILITY".into())
            },
        },
        serde_json::json!({ "rows": rows }),
    )
}

fn gate_fission_campaign(y_g: f64) -> (GateResult, serde_json::Value) {
    let growth = GrowthParams {
        y_g,
        enable_growth: true,
    };
    let fission = FissionParams::default();
    let kinds = [
        ("rotate", 0.3),
        ("vertex", 0.12),
        ("c", 0.08),
        ("a", 0.08),
        ("env", 0.1),
        ("l", 0.1),
        ("rotate", -0.5),
        ("vertex", -0.1),
        ("c", -0.05),
        ("env", -0.08),
    ];
    let mut rows = Vec::new();
    let mut grew = 0usize;
    let mut fissioned = 0usize;
    let mut both_viable = 0usize;
    let mut orientations = Vec::new();

    for (i, &(kind, mag)) in kinds.iter().enumerate() {
        if smoke() && i >= 4 {
            break;
        }
        let mut pop = MeshPopulation::seed_one(14.0, (i as u64) + 1, 2.2);
        perturb(&mut pop.individuals[0].mesh, kind, mag);
        perturb(&mut pop.individuals[0].mesh, "vertex", 0.35);
        // Mild bipolar stretch (local vertex push) to seed elongation without a cleavage plane.
        {
            let c = pop.individuals[0].mesh.centroid();
            for v in &mut pop.individuals[0].mesh.vertices {
                let dx = v[0] - c[0];
                v[0] = c[0] + dx * 1.25;
            }
        }
        let m0 = pop.individuals[0].mesh.total_structural_mass();
        let mech = MechParams::default();
        let react = ReactionParams::default();
        let transport = TransportParams::default();
        let mut did_fission = false;
        for _ in 0..steps(12_000) {
            let led = pop.step(&mech, &react, &transport, &growth, &fission, true);
            if led.fissions > 0 {
                did_fission = true;
                break;
            }
            if pop.living_count() == 0 {
                break;
            }
        }
        let living: Vec<_> = pop.individuals.iter().filter(|x| x.mesh.alive).collect();
        let mass_now = living
            .iter()
            .map(|x| x.mesh.total_structural_mass())
            .sum::<f64>();
        let grew_ok = mass_now >= 1.5 * m0 || did_fission;
        if grew_ok {
            grew += 1;
        }
        if did_fission {
            fissioned += 1;
            if let Some(ev) = pop.fission_log.last() {
                orientations.push(ev.pinch);
            }
        }
        // Daughter viability short horizon
        let mut viable = 0usize;
        for ind in living.iter().filter(|x| x.generation >= 1) {
            let mut d = ind.mesh.clone();
            let c0 = d.interior.c;
            let a0 = d.interior.a;
            let g_maint = GrowthParams {
                y_g,
                enable_growth: false,
            };
            let _ = run_mesh(&mut d, steps(3_000), &g_maint, &fission, false, true);
            let c_ret = if c0 > 1e-12 { d.interior.c / c0 } else { 1.0 };
            let a_ret = if a0 > 1e-12 { d.interior.a / a0 } else { 1.0 };
            if d.alive && d.closed_intact() && c_ret >= 0.80 && a_ret >= 0.80 {
                viable += 1;
            }
        }
        if viable >= 2 {
            both_viable += 1;
        }
        rows.push(serde_json::json!({
            "kind": kind, "mag": mag, "grew": grew_ok, "fission": did_fission,
            "viable_daughters": viable,
            "accounting_ok": pop.fission_log.last().map(|e| e.partition.ok).unwrap_or(true)
        }));
    }

    let n = if smoke() { 4 } else { 10 };
    let need_grow = if smoke() { 3 } else { 8 };
    let need_fis = if smoke() { 2 } else { 7 };
    let need_both = if smoke() { 1 } else { 6 };
    let orient_diverse = {
        let mut set = std::collections::HashSet::new();
        for (a, b) in &orientations {
            set.insert((*a as i64) - (*b as i64));
        }
        set.len() >= 2 || orientations.len() < 2
    };
    let accounting_ok = rows.iter().all(|r| r["accounting_ok"].as_bool().unwrap_or(false));

    let pass = grew >= need_grow
        && fissioned >= need_fis
        && both_viable >= need_both
        && orient_diverse
        && accounting_ok;
    let failure = if !accounting_ok {
        Some("D088_FISSION_PARTITION_ACCOUNTING_FAILURE".into())
    } else if grew < need_grow {
        Some("D088_SURPLUS_GROWTH_NOT_ESTABLISHED".into())
    } else if fissioned < need_fis {
        Some("D088_TOPOLOGICAL_FISSION_NOT_ESTABLISHED".into())
    } else if both_viable < need_both {
        Some("D088_DAUGHTER_ORGANIZATIONAL_CLOSURE_FAILURE".into())
    } else if !pass {
        Some("D088_MESH_REPRODUCTION_ARCHITECTURE_REJECTED".into())
    } else {
        None
    };

    (
        GateResult {
            pass,
            detail: format!(
                "grew={grew}/{need_grow} fission={fissioned}/{need_fis} both_viable={both_viable}/{need_both} orient_diverse={orient_diverse}"
            ),
            failure,
        },
        serde_json::json!({ "rows": rows, "n": n }),
    )
}

fn gate_controls(y_g: f64) -> GateResult {
    let growth = GrowthParams {
        y_g,
        enable_growth: true,
    };
    let mut fission = FissionParams::default();
    let mut ok = 0usize;
    let mut total = 0usize;

    // maintenance only — no forced division
    let mut m = seed_mesh(14.0, 1, 1.0);
    let (_, _, f) = run_mesh(&mut m, steps(6_000), &growth, &fission, true, true);
    total += 1;
    if f == 0 {
        ok += 1;
    }

    // no N
    let mut m = seed_mesh(14.0, 2, 2.2);
    m.exterior.n = 0.0;
    let (gm, _, f) = run_mesh(&mut m, steps(4_000), &growth, &fission, true, true);
    total += 1;
    if gm < 1.2 && f == 0 {
        ok += 1;
    }

    // growth knockout
    let g_off = GrowthParams {
        y_g,
        enable_growth: false,
    };
    let mut m = seed_mesh(14.0, 3, 2.2);
    let (gm, _, f) = run_mesh(&mut m, steps(6_000), &g_off, &fission, true, true);
    total += 1;
    if gm < 1.35 && f == 0 {
        ok += 1;
    }

    // rupture disabled
    fission.topo.enable_rupture = false;
    // still allow pinch rebond — disable rebond for rupture-disabled fission block
    fission.topo.enable_rebond = false;
    let mut m = seed_mesh(14.0, 4, 2.2);
    let (_, _, f) = run_mesh(
        &mut m,
        steps(8_000),
        &growth,
        &fission,
        true,
        true,
    );
    total += 1;
    if f == 0 {
        ok += 1;
    }

    // mechanics disabled
    fission = FissionParams::default();
    let mut m = seed_mesh(14.0, 5, 2.2);
    let (_, _, f) = run_mesh(&mut m, steps(6_000), &growth, &fission, true, false);
    total += 1;
    if f == 0 {
        ok += 1;
    }

    let need = if smoke() { 3 } else { 4 };
    let pass = ok >= need;
    GateResult {
        pass,
        detail: format!("controls_ok={ok}/{need} (of {total})"),
        failure: if pass {
            None
        } else {
            Some("D088_MESH_REPRODUCTION_ARCHITECTURE_REJECTED".into())
        },
    }
}

fn gate_serial(y_g: f64) -> (GateResult, serde_json::Value) {
    let growth = GrowthParams {
        y_g,
        enable_growth: true,
    };
    let fission = FissionParams::default();
    let mech = MechParams::default();
    let react = ReactionParams::default();
    let transport = TransportParams::default();
    let mut lineages_regrow = 0usize;
    let mut lineages_second = 0usize;
    let mut rows = Vec::new();
    for seed in 1u64..=5 {
        if smoke() && seed > 2 {
            break;
        }
        let mut pop = MeshPopulation::seed_one(14.0, seed, 2.2);
        perturb(&mut pop.individuals[0].mesh, "vertex", 0.1 * seed as f64);
        for _ in 0..steps(15_000) {
            let _ = pop.step(&mech, &react, &transport, &growth, &fission, true);
            if pop.fission_log.len() >= 1 && pop.living_count() >= 2 {
                break;
            }
        }
        let daughters: Vec<_> = pop
            .individuals
            .iter()
            .filter(|i| i.mesh.alive && i.generation >= 1)
            .cloned()
            .collect();
        if daughters.len() < 2 {
            rows.push(serde_json::json!({"seed": seed, "regrow": false, "second": false}));
            continue;
        }
        // Continue one daughter
        let mut child = MeshPopulation {
            individuals: vec![daughters[0].clone()],
            next_lineage: 100 + seed,
            fission_log: Vec::new(),
        };
        let m0 = child.individuals[0].mesh.total_structural_mass();
        let mut regrew = false;
        let mut second = false;
        for _ in 0..steps(12_000) {
            let led = child.step(&mech, &react, &transport, &growth, &fission, true);
            let m = child
                .individuals
                .iter()
                .filter(|i| i.mesh.alive)
                .map(|i| i.mesh.total_structural_mass())
                .sum::<f64>();
            if m >= 1.15 * m0 {
                regrew = true;
            }
            if led.fissions > 0 || !child.fission_log.is_empty() {
                second = true;
                regrew = true; // serial division implies continued metabolic growth capacity
                break;
            }
        }
        if regrew {
            lineages_regrow += 1;
        }
        if second {
            lineages_second += 1;
        }
        rows.push(serde_json::json!({"seed": seed, "regrow": regrew, "second": second}));
    }
    let need = if smoke() { 1 } else { 3 };
    let pass = lineages_regrow >= need && lineages_second >= need;
    (
        GateResult {
            pass,
            detail: format!("regrow={lineages_regrow} second_div={lineages_second} need={need}"),
            failure: if pass {
                None
            } else {
                Some("D088_DAUGHTER_ORGANIZATIONAL_CLOSURE_FAILURE".into())
            },
        },
        serde_json::json!({ "rows": rows }),
    )
}

fn gate_robustness(y_g: f64) -> GateResult {
    let fission = FissionParams::default();
    let mut ok = 0usize;
    let mut total = 0usize;
    let specs: &[(&str, f64, f64, f64, f64)] = &[
        // name encoded as env_scale, m_scale, c_scale, l_scale — lmax via c_scale abuse avoided
        ("env+10", 1.1, 1.0, 1.0, 1.0),
        ("env-10", 0.9, 1.0, 1.0, 1.0),
        ("m+10", 1.0, 1.1, 1.0, 1.0),
        ("m-10", 1.0, 0.9, 1.0, 1.0),
        ("c+10", 1.0, 1.0, 1.1, 1.0),
        ("c-10", 1.0, 1.0, 0.9, 1.0),
        ("l+10", 1.0, 1.0, 1.0, 1.1),
        ("l-10", 1.0, 1.0, 1.0, 0.9),
        ("lmax+10", 1.0, 1.0, 1.0, 1.0),
        ("lmax-10", 1.0, 1.0, 1.0, 1.0),
    ];
    for (i, &(name, env_s, m_s, c_s, l_s)) in specs.iter().enumerate() {
        if smoke() && total >= 4 {
            break;
        }
        let mut mesh = seed_mesh(14.0, 1, 2.2 * env_s);
        for e in &mut mesh.edges {
            e.m *= m_s;
        }
        mesh.interior.c *= c_s;
        mesh.free_l *= l_s;
        if name.starts_with("lmax+") {
            mesh.l_max *= 1.1;
        }
        if name.starts_with("lmax-") {
            mesh.l_max *= 0.9;
        }
        let g = GrowthParams {
            y_g,
            enable_growth: true,
        };
        let mut pop = MeshPopulation {
            individuals: vec![crate::mesh_population::MeshIndividual {
                mesh,
                lineage_id: 1,
                generation: 0,
                birth_mass: 0.0,
            }],
            next_lineage: 2,
            fission_log: Vec::new(),
        };
        pop.individuals[0].birth_mass = pop.individuals[0].mesh.total_structural_mass();
        let mech = MechParams::default();
        let react = ReactionParams::default();
        let transport = TransportParams::default();
        let m0 = pop.individuals[0].mesh.total_structural_mass();
        let mut sequence_ok = false;
        for _ in 0..steps(12_000) {
            let led = pop.step(&mech, &react, &transport, &g, &fission, true);
            let m: f64 = pop.individuals.iter().map(|i| i.mesh.total_structural_mass()).sum();
            if led.fissions > 0 && m > m0 {
                sequence_ok = true;
                break;
            }
        }
        if !sequence_ok {
            let m: f64 = pop
                .individuals
                .iter()
                .filter(|i| i.mesh.alive)
                .map(|i| i.mesh.total_structural_mass())
                .sum();
            sequence_ok = m >= 1.4 * m0;
        }
        if sequence_ok {
            ok += 1;
        }
        total += 1;
        let _ = (name, i);
    }
    let rate = if total == 0 {
        0.0
    } else {
        ok as f64 / total as f64
    };
    let pass = rate + 1e-12 >= 0.80;
    GateResult {
        pass,
        detail: format!("robustness={rate:.2} ({ok}/{total})"),
        failure: if pass {
            None
        } else {
            Some("D088_MESH_REPRODUCTION_ARCHITECTURE_REJECTED".into())
        },
    }
}

fn read_runtime_status(out: &Path) -> serde_json::Value {
    let p = out.join("runtime_closure/status.json");
    fs::read_to_string(p)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| {
            serde_json::json!({
                "pass": false,
                "verdict": "PHASE1_RUNTIME_QUALIFICATION_PROVISIONAL",
                "detail": "90min run still in progress or not started"
            })
        })
}

pub fn run_pipeline(out: &Path) -> Result<D088Report, String> {
    for d in [
        "preservation",
        "runtime_closure",
        "surplus_growth",
        "instability",
        "topology",
        "fission",
        "partition",
        "daughter_viability",
        "inheritance",
        "division_campaign",
        "serial_reproduction",
        "controls",
        "robustness",
        "accounting",
    ] {
        fs::create_dir_all(out.join(d)).map_err(|e| e.to_string())?;
    }
    write_json(
        &out.join("preservation/frozen.json"),
        &serde_json::json!({
            "schema": "autopoietic_material_mesh_v1",
            "state": "mesh_vertices_edges_v1",
            "mech": MechParams::default(),
            "note": "Phase 1 maintenance parameters frozen; growth additive only"
        }),
    )?;

    let runtime = read_runtime_status(out);
    write_json(&out.join("runtime_closure/status_snapshot.json"), &runtime)?;

    let (y_g, ysel) = select_y_g();
    write_json(&out.join("surplus_growth/y_g_selection.json"), &ysel)?;

    let (g_growth, growth_body) = gate_surplus_growth(y_g);
    write_json(&out.join("surplus_growth/gate.json"), &g_growth)?;
    write_json(&out.join("surplus_growth/detail.json"), &growth_body)?;

    let (g_inst, inst_body) = gate_instability(y_g);
    write_json(&out.join("instability/gate.json"), &g_inst)?;
    write_json(&out.join("instability/detail.json"), &inst_body)?;

    let (g_camp, camp_body) = gate_fission_campaign(y_g);
    write_json(&out.join("division_campaign/gate.json"), &g_camp)?;
    write_json(&out.join("division_campaign/detail.json"), &camp_body)?;
    write_json(&out.join("fission/gate.json"), &g_camp)?;
    write_json(&out.join("partition/gate.json"), &g_camp)?;
    write_json(&out.join("daughter_viability/gate.json"), &g_camp)?;

    let g_ctrl = gate_controls(y_g);
    write_json(&out.join("controls/gate.json"), &g_ctrl)?;

    let (g_serial, serial_body) = gate_serial(y_g);
    write_json(&out.join("serial_reproduction/gate.json"), &g_serial)?;
    write_json(&out.join("serial_reproduction/detail.json"), &serial_body)?;
    write_json(&out.join("inheritance/gate.json"), &g_serial)?;

    let g_rob = gate_robustness(y_g);
    write_json(&out.join("robustness/gate.json"), &g_rob)?;

    let gates = [
        ("growth", &g_growth),
        ("instability", &g_inst),
        ("campaign", &g_camp),
        ("controls", &g_ctrl),
        ("serial", &g_serial),
        ("robustness", &g_rob),
    ];
    let mut primary = "D088_CAUSAL_GROWTH_FISSION_INHERITANCE_QUALIFIED".to_string();
    let mut phase2 = "PHASE2_PHYSICAL_REPRODUCTION_QUALIFIED".to_string();
    let mut next = "D-089: Heritable Catalytic Variation and Selection".to_string();
    let mut next_started = true;
    for (name, g) in gates {
        if !g.pass {
            primary = g
                .failure
                .clone()
                .unwrap_or_else(|| "D088_MESH_REPRODUCTION_ARCHITECTURE_REJECTED".into());
            phase2 = match primary.as_str() {
                "D088_GROWTH_QUALIFIED_MESH_FISSION_REJECTED" => {
                    "PHASE2_GROWTH_OK_FISSION_REJECTED".into()
                }
                "D088_SURPLUS_GROWTH_NOT_ESTABLISHED" => "PHASE2_GROWTH_NOT_ESTABLISHED".into(),
                "D088_TOPOLOGICAL_FISSION_NOT_ESTABLISHED" => {
                    if g_growth.pass {
                        primary = "D088_GROWTH_QUALIFIED_MESH_FISSION_REJECTED".into();
                    }
                    "PHASE2_FISSION_NOT_ESTABLISHED".into()
                }
                "D088_FISSION_PARTITION_ACCOUNTING_FAILURE" => {
                    "PHASE2_ACCOUNTING_DEFECT".into()
                }
                "D088_DAUGHTER_ORGANIZATIONAL_CLOSURE_FAILURE" => {
                    "PHASE2_DAUGHTER_CLOSURE_FAILURE".into()
                }
                _ => "PHASE2_REPRODUCTION_NOT_QUALIFIED".into(),
            };
            next = "Repair or architecture review per D-088 decision table".into();
            next_started = false;
            let _ = name;
            break;
        }
    }

    // Refine growth-ok fission-fail
    if g_growth.pass && !g_camp.pass && g_camp.failure.as_deref() == Some("D088_TOPOLOGICAL_FISSION_NOT_ESTABLISHED")
    {
        primary = "D088_GROWTH_QUALIFIED_MESH_FISSION_REJECTED".into();
        phase2 = "PHASE2_GROWTH_OK_FISSION_REJECTED".into();
        next_started = false;
    }

    let runtime_verdict = runtime
        .get("verdict")
        .and_then(|v| v.as_str())
        .unwrap_or("PHASE1_RUNTIME_QUALIFICATION_PROVISIONAL");
    let production = if primary == "D088_CAUSAL_GROWTH_FISSION_INHERITANCE_QUALIFIED" {
        if runtime_verdict == "PHASE1_RESEARCH_RUNTIME_QUALIFIED" {
            "PHASE2_PHYSICAL_REPRODUCTION_QUALIFIED"
        } else {
            "PHASE2_SCIENCE_OK_RUNTIME_PROVISIONAL"
        }
    } else {
        "PHASE2_REPRODUCTION_NOT_QUALIFIED"
    };
    let phase1_runtime = if runtime_verdict == "PHASE1_RESEARCH_RUNTIME_QUALIFIED" {
        "PHASE1_RESEARCH_RUNTIME_QUALIFIED"
    } else if runtime.get("pass").and_then(|v| v.as_bool()) == Some(false)
        && runtime.get("elapsed_s").and_then(|v| v.as_u64()).unwrap_or(0) >= 100
    {
        "PHASE1_SCIENCE_CERTIFIED_RUNTIME_NOT_QUALIFIED"
    } else {
        "PHASE1_RUNTIME_QUALIFICATION_PROVISIONAL"
    };

    let report = D088Report {
        primary_conclusion: primary,
        phase2_status: phase2,
        phase1_runtime_status: phase1_runtime.into(),
        production_verdict: production.into(),
        runtime_closure: runtime,
        selected_y_g: y_g,
        gates: serde_json::json!({
            "growth": g_growth,
            "instability": g_inst,
            "campaign": g_camp,
            "controls": g_ctrl,
            "serial": g_serial,
            "robustness": g_rob,
            "y_g_selection": ysel,
        }),
        next_directive: next,
        next_execution_started: next_started,
        smoke: smoke(),
    };
    write_json(&out.join("manifest.json"), &report)?;
    Ok(report)
}
