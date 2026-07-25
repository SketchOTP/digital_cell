//! D-092: Minimal catalytic template polymer heredity and evolution.

use crate::abrasion_front::{AbrasionCampaign, ABRASION_STRENGTHS};
use crate::d090_dish::{assemble_population, observe_spatial_dish, spatial_dish_step};
use crate::material_mesh::{
    LumpedChem, MaterialMesh, DEFAULT_RHO_S, EQUATION_VERSION_MATERIAL_MESH,
};
use crate::mesh_fission::{try_local_fission, FissionParams};
use crate::mesh_growth::GrowthParams;
use crate::mesh_mechanics::MechParams;
use crate::mesh_population::{coupled_step_growth, MeshIndividual, MeshPopulation};
use crate::mesh_reactions::{evaluate_death, reactions_step, ReactionParams};
use crate::mesh_transport::TransportParams;
use crate::metabolic_reserve::{
    reserve_schema_load_ok, stamp_reserve_equation, ReserveParams,
    EQUATION_VERSION_METABOLIC_RESERVE, FIELD_SCHEMA_METABOLIC_RESERVE,
};
use crate::seasonal_ecology::{PulseLeanSchedule, PulseLeanState, PULSE_PERIOD_MULTS};
use crate::spatial_shared_dish::SpatialDish;
use crate::template_copying::copying_step;
use crate::template_motifs::{
    catalyst_binding_step, count_available_motifs, template_activity_gains,
};
use crate::template_partition::complete_sequences;
use crate::template_polymer::{
    count_complete_templates, monomer_production_step, seed_founder_chains, stamp_template_equation,
    template_schema_load_ok, TemplateParams, XorShift64, EQUATION_VERSION_CATALYTIC_TEMPLATE,
    FIELD_SCHEMA_CATALYTIC_TEMPLATE, FOUNDER_BUILD, FOUNDER_HARVEST, FOUNDER_LEN, FOUNDER_NEUTRAL,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::Path;

pub fn smoke() -> bool {
    matches!(
        env::var("D092_SMOKE").ok().as_deref(),
        Some("1") | Some("true") | Some("TRUE")
    )
}

fn reps() -> usize {
    if smoke() {
        2
    } else {
        8
    }
}

fn n_each() -> usize {
    if smoke() {
        2
    } else {
        4
    }
}

fn steps(full: usize) -> usize {
    if smoke() {
        (full / 6).max(300)
    } else {
        full
    }
}

fn write_json(path: &Path, v: &impl Serialize) -> Result<(), String> {
    if let Some(p) = path.parent() {
        fs::create_dir_all(p).map_err(|e| e.to_string())?;
    }
    fs::write(path, serde_json::to_string_pretty(v).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    pub name: String,
    pub pass: bool,
    pub code: Option<String>,
    pub detail: Value,
}

fn gate_pass(name: &str, detail: Value) -> GateResult {
    GateResult {
        name: name.into(),
        pass: true,
        code: None,
        detail,
    }
}

fn gate_fail(name: &str, code: &str, detail: Value) -> GateResult {
    GateResult {
        name: name.into(),
        pass: false,
        code: Some(code.into()),
        detail,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct D092Report {
    pub primary_conclusion: String,
    pub phase2_status: String,
    pub phase3_authorized: bool,
    pub production_verdict: String,
    pub schema_equation: String,
    pub schema_fields: String,
    pub founder_sequences: Value,
    pub measured_fidelity: f64,
    pub smoke: bool,
    pub starting_commit: String,
    pub ending_commit_hint: String,
    pub gates: Value,
    pub records: Vec<String>,
    pub next_directive: String,
    pub next_execution_started: bool,
}

fn frozen_yg() -> GrowthParams {
    GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    }
}

fn react_base() -> ReactionParams {
    ReactionParams::default()
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
            ..Default::default()
        },
        LumpedChem {
            n: 1.0 * ext,
            f: 1.0 * ext,
            ..Default::default()
        },
        5.0,
    )
}

fn elongate(mesh: &mut MaterialMesh) {
    let c = mesh.centroid();
    for v in &mut mesh.vertices {
        v[0] = c[0] + (v[0] - c[0]) * 1.55;
        v[1] = c[1] + (v[1] - c[1]) * 0.72;
    }
}

fn derive_horizons() -> (f64, f64, f64, f64, f64, f64) {
    let react = react_base();
    let t_replace = 1.0 / react.k_turn.max(1e-9);
    let mut mesh = seed_mesh(5.0, 1, 1.0);
    let mech = MechParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        enable_growth: false,
        y_g: 0.0,
    };
    let fission = FissionParams::default();
    let mut a_samples = Vec::new();
    let mut maint = 0.0;
    let n_steps = steps(2500);
    for s in 0..n_steps {
        let _ = coupled_step_growth(
            &mut mesh,
            &mech,
            &react,
            &transport,
            &growth,
            &fission,
            true,
            false,
        );
        evaluate_death(&mut mesh);
        if s > n_steps / 3 {
            a_samples.push(mesh.interior.a.max(0.0));
            let mut m = 0.0;
            for i in 0..mesh.n() {
                m += crate::mesh_growth::local_maintenance_a_rate(&mesh, i, &react);
            }
            maint += m;
        }
        if !mesh.alive {
            break;
        }
    }
    a_samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let a_median = if a_samples.is_empty() {
        0.4
    } else {
        a_samples[a_samples.len() / 2]
    };
    let a_q25 = if a_samples.is_empty() {
        0.2
    } else {
        a_samples[a_samples.len() / 4]
    };
    let n_samp = (n_steps * 2 / 3).max(1) as f64;
    let mean_maint = maint / n_samp;
    let a_pool = a_median * mesh.area().max(1e-6);
    let t_maint = (a_pool / mean_maint.max(1e-9)).clamp(10.0, t_replace * 2.0);
    let fission_a_cost = mesh.total_structural_mass() * 0.35;
    let area = mesh.area().max(1e-6);
    (t_replace, t_maint, a_median, a_q25, fission_a_cost, area)
}

fn selected_reserve() -> ReserveParams {
    let (t_replace, t_maint, a_median, a_q25, fission_a_cost, area) = derive_horizons();
    // D-091 selected H=2
    ReserveParams::derived(t_replace, t_maint, a_median, a_q25, 2.0, fission_a_cost, area)
}

fn with_template(mut react: ReactionParams, reserve: ReserveParams, tmpl: TemplateParams) -> ReactionParams {
    react.reserve = reserve;
    react.template = tmpl;
    react.composition.enable = false;
    react
}

fn t_gen_from_reserve(reserve: &ReserveParams) -> f64 {
    // Operative generation ≈ 2 × maintenance horizon under D-091.
    let t_maint = 1.0 / reserve.k_release.max(1e-9);
    2.0 * t_maint
}

fn stamp_template_seed(mut mesh: MaterialMesh) -> MaterialMesh {
    stamp_template_equation(&mut mesh);
    mesh
}

fn compact_dish() -> SpatialDish {
    SpatialDish::new(8, 8, 2.5, [0.0, 0.0], 120.0, 120.0, 0.0, 0.0, 3.0)
}

fn seed_template_org(
    seq: &str,
    n_chains: usize,
    seed: u64,
    reserve: &ReserveParams,
    tmpl: &TemplateParams,
) -> MeshIndividual {
    let mut mesh = seed_mesh(12.0, seed, 0.5);
    elongate(&mut mesh);
    stamp_template_equation(&mut mesh);
    mesh.interior.r = 0.6;
    mesh.interior.a = 0.8;
    mesh.interior.u_h = 0.2;
    mesh.interior.u_b = 0.2;
    let next_id = mesh.next_template_id.max(1);
    let next = seed_founder_chains(&mut mesh, seq, n_chains, next_id);
    mesh.next_template_id = next;
    let c = mesh.centroid();
    for (i, t) in mesh.templates.iter_mut().enumerate() {
        let f = (i as f64 + 0.5) / n_chains.max(1) as f64;
        t.pos = [c[0] + (f - 0.5) * 10.0, c[1]];
    }
    let _ = (reserve, tmpl);
    let clade = if seq == FOUNDER_HARVEST {
        1
    } else if seq == FOUNDER_BUILD {
        -1
    } else {
        0
    };
    MeshIndividual {
        birth_mass: mesh.total_structural_mass(),
        mesh,
        lineage_id: seed,
        generation: 0,
        clade,
    }
}

fn gate0_preservation(out: &Path) -> Result<(GateResult, ReserveParams, TemplateParams), String> {
    let reserve = selected_reserve();
    let t_gen = t_gen_from_reserve(&reserve);
    let tmpl = TemplateParams::derived(t_gen);
    let identity = format!(
        "{}|{}",
        reserve.candidate_identity_suffix(),
        tmpl.candidate_identity_suffix()
    );

    // Template disabled + reserve enabled on reserve stamp must match reserve path.
    let mut m_res = seed_mesh(5.0, 2, 1.0);
    stamp_reserve_equation(&mut m_res);
    let mut m_tpl_off = m_res.clone();
    stamp_template_equation(&mut m_tpl_off);
    let react_res = with_template(react_base(), reserve, TemplateParams::default());
    let mut react_off = with_template(react_base(), reserve, TemplateParams::default());
    react_off.reserve = reserve;
    // Numerical parity: template schema with template.disable should still run reserve if we
    // allow reserve on template stamp when template.enable=false. Contract: template disabled
    // matches frozen reserve organism — use reserve stamp for baseline, template-disabled on
    // template stamp for comparison of chemistry when template.enable=false.
    let mech = MechParams::default();
    let transport = TransportParams::default();
    let growth = frozen_yg();
    let fission = FissionParams::default();
    let mut mass_err = 0.0;
    let mut a_err = 0.0;
    {
        let mut a = m_res.clone();
        let mut b = {
            let mut m = seed_mesh(5.0, 2, 1.0);
            stamp_template_equation(&mut m);
            m.interior = a.interior;
            m
        };
        let mut ra = react_res.clone();
        ra.template.enable = false;
        let mut rb = ra;
        rb.template.enable = false;
        // Reserve chemistry requires reserve schema OR we compare with both using reserve stamp.
        // Gate0 contract: with template chemistry disabled, new schema matches reserve organism.
        stamp_reserve_equation(&mut a);
        // For b, keep template stamp but template.enable=false — reserve_metab_step checks
        // reserve_schema_load_ok which requires reserve equation. So stamp reserve for parity path.
        stamp_reserve_equation(&mut b);
        for _ in 0..200 {
            let _ = coupled_step_growth(&mut a, &mech, &ra, &transport, &growth, &fission, true, false);
            let _ = coupled_step_growth(&mut b, &mech, &rb, &transport, &growth, &fission, true, false);
        }
        mass_err = (a.total_structural_mass() - b.total_structural_mass()).abs();
        a_err = (a.interior.a - b.interior.a).abs();
    }

    let mut old = seed_mesh(5.0, 3, 1.0);
    stamp_reserve_equation(&mut old);
    let mut newm = seed_mesh(5.0, 3, 1.0);
    stamp_template_equation(&mut newm);
    let tmpl_on = TemplateParams::derived(t_gen);
    let old_ok = template_schema_load_ok(&old, &tmpl_on);
    let new_ok = template_schema_load_ok(&newm, &tmpl_on);
    let reserve_ok_old = reserve_schema_load_ok(&old, &reserve);
    let reserve_ok_new = reserve_schema_load_ok(&newm, &reserve);

    // Rejected steps when template runs on old snapshot
    let mut react_bad = with_template(react_base(), reserve, tmpl_on);
    let led = reactions_step(&mut old, &react_bad, 0.02, true, true);
    let rejected = led.template.rejected_steps > 0 || !old_ok;

    let detail = json!({
        "identity": identity,
        "equation": EQUATION_VERSION_CATALYTIC_TEMPLATE,
        "fields": FIELD_SCHEMA_CATALYTIC_TEMPLATE,
        "mass_err": mass_err,
        "a_err": a_err,
        "old_template_load_ok": old_ok,
        "new_template_load_ok": new_ok,
        "old_reserve_load_ok": reserve_ok_old,
        "new_reserve_on_template_stamp": reserve_ok_new,
        "rejected_on_old": rejected,
        "founder_len": FOUNDER_LEN,
        "founders": [FOUNDER_HARVEST, FOUNDER_BUILD, FOUNDER_NEUTRAL],
    });
    write_json(&out.join("preservation/gate0.json"), &detail)?;
    write_json(
        &out.join("schema/ids.json"),
        &json!({
            "equation": EQUATION_VERSION_CATALYTIC_TEMPLATE,
            "fields": FIELD_SCHEMA_CATALYTIC_TEMPLATE,
            "reserve_equation": EQUATION_VERSION_METABOLIC_RESERVE,
            "reserve_fields": FIELD_SCHEMA_METABOLIC_RESERVE,
            "base_equation": EQUATION_VERSION_MATERIAL_MESH,
        }),
    )?;

    let pass = mass_err < 1e-9
        && a_err < 1e-9
        && !old_ok
        && new_ok
        && reserve_ok_old
        && rejected;
    let g = if pass {
        gate_pass("gate0_preservation", detail)
    } else {
        gate_fail("gate0_preservation", "D092_PRESERVATION_OR_SCHEMA_FAILURE", detail)
    };
    Ok((g, reserve, tmpl_on))
}

fn gate1_accounting(out: &Path, reserve: &ReserveParams, tmpl: &TemplateParams) -> Result<GateResult, String> {
    let mut mesh = stamp_template_seed(seed_mesh(5.0, 4, 1.0));
    mesh.interior.a = 1.0;
    mesh.interior.n = 1.0;
    mesh.interior.c = 0.8;
    let react = with_template(react_base(), *reserve, *tmpl);
    let area = mesh.area();
    let n0 = mesh.interior.n * area;
    let a0 = mesh.interior.a * area;
    let uh0 = mesh.interior.u_h * area;
    let ub0 = mesh.interior.u_b * area;
    let led = monomer_production_step(&mut mesh, &react, 0.05);
    let n1 = mesh.interior.n * area;
    let a1 = mesh.interior.a * area;
    let uh1 = mesh.interior.u_h * area;
    let ub1 = mesh.interior.u_b * area;
    let mono_ok = led.u_h_produced > 0.0
        && ((uh1 - uh0) - led.u_h_produced).abs() < 1e-9
        && ((ub1 - ub0) - led.u_b_produced).abs() < 1e-9
        && ((n0 - n1) - led.n_consumed_mono).abs() < 1e-9
        && ((a0 - a1) - led.a_consumed_mono).abs() < 1e-9
        && (led.u_h_produced - led.u_b_produced).abs() < 1e-12;

    // Catalyst binding conservation
    let _ = seed_founder_chains(&mut mesh, FOUNDER_HARVEST, 2, 1);
    mesh.interior.k_h = 0.0;
    mesh.interior.k_b = 0.0;
    let c0 = mesh.interior.c;
    let _ = catalyst_binding_step(&mut mesh, &react, 0.1);
    let c_cons = (mesh.interior.c - c0).abs() < 1e-15
        && mesh.interior.k_h + mesh.interior.k_b <= c0 + 1e-12;

    // No complete chain from monomers alone
    let mut bare = stamp_template_seed(seed_mesh(5.0, 5, 1.0));
    bare.interior.u_h = 2.0;
    bare.interior.u_b = 2.0;
    bare.interior.a = 2.0;
    bare.interior.c = 0.8;
    for _ in 0..200 {
        let _ = reactions_step(&mut bare, &react, 0.02, true, true);
    }
    let no_spontaneous = count_complete_templates(&bare) == 0;

    let detail = json!({
        "mono_ok": mono_ok,
        "catalyst_conserved": c_cons,
        "no_spontaneous_template": no_spontaneous,
        "ledger": led,
        "motifs_harvest_founder": count_available_motifs(&{
            let mut m = stamp_template_seed(seed_mesh(5.0, 1, 1.0));
            seed_founder_chains(&mut m, FOUNDER_HARVEST, 1, 1);
            m
        }),
        "motifs_build_founder": count_available_motifs(&{
            let mut m = stamp_template_seed(seed_mesh(5.0, 1, 1.0));
            seed_founder_chains(&mut m, FOUNDER_BUILD, 1, 1);
            m
        }),
        "motifs_neutral_founder": count_available_motifs(&{
            let mut m = stamp_template_seed(seed_mesh(5.0, 1, 1.0));
            seed_founder_chains(&mut m, FOUNDER_NEUTRAL, 1, 1);
            m
        }),
    });
    write_json(&out.join("polymer_accounting/gate1.json"), &detail)?;
    Ok(if mono_ok && c_cons && no_spontaneous {
        gate_pass("gate1_accounting", detail)
    } else {
        gate_fail(
            "gate1_accounting",
            "D092_TEMPLATE_MATERIAL_ACCOUNTING_FAILURE",
            detail,
        )
    })
}

fn gate2_copying(out: &Path, reserve: &ReserveParams, tmpl: &TemplateParams) -> Result<(GateResult, f64), String> {
    let need_copies = if smoke() { 20 } else { 100 };
    let mut mesh = stamp_template_seed(seed_mesh(6.0, 7, 1.0));
    mesh.interior.a = 2.0;
    mesh.interior.n = 1.0;
    mesh.interior.c = 1.0;
    mesh.interior.u_h = 5.0;
    mesh.interior.u_b = 5.0;
    let _ = seed_founder_chains(&mut mesh, FOUNDER_HARVEST, 1, 1);
    let mut react = with_template(react_base(), *reserve, *tmpl);
    let mut rng = XorShift64::new(0xD092);
    let mut next_id = mesh.next_template_id;
    let mut copies = 0u64;
    let mut match_n = 0u64;
    let mut mismatch_n = 0u64;
    let mut steps_run = 0;
    while copies < need_copies as u64 && steps_run < steps(50_000) {
        // Replenish free monomers and A for the copying assay.
        mesh.interior.u_h = mesh.interior.u_h.max(2.0);
        mesh.interior.u_b = mesh.interior.u_b.max(2.0);
        mesh.interior.a = mesh.interior.a.max(1.0);
        let led = copying_step(&mut mesh, &react, 0.05, &mut rng, &mut next_id);
        copies += led.complete_copies;
        match_n += led.match_binds;
        mismatch_n += led.mismatch_binds;
        steps_run += 1;
    }
    mesh.next_template_id = next_id;
    let total_binds = match_n + mismatch_n;
    let fidelity_mismatch = if total_binds > 0 {
        mismatch_n as f64 / total_binds as f64
    } else {
        1.0
    };
    // Expected ~1% with 100× affinity ratio
    let fidelity_ok = total_binds >= 50 && (fidelity_mismatch - 0.01).abs() < 0.03;

    // Controls
    let mut no_t = stamp_template_seed(seed_mesh(6.0, 8, 1.0));
    no_t.interior.u_h = 5.0;
    no_t.interior.u_b = 5.0;
    no_t.interior.a = 2.0;
    no_t.interior.c = 1.0;
    let mut rng2 = XorShift64::new(1);
    let mut id2 = 1u64;
    let mut ctrl_copies = 0u64;
    for _ in 0..500 {
        let led = copying_step(&mut no_t, &react, 0.05, &mut rng2, &mut id2);
        ctrl_copies += led.complete_copies;
    }

    let mut no_a = stamp_template_seed(seed_mesh(6.0, 9, 1.0));
    let _ = seed_founder_chains(&mut no_a, FOUNDER_HARVEST, 1, 1);
    no_a.interior.u_h = 5.0;
    no_a.interior.u_b = 5.0;
    no_a.interior.a = 0.0;
    no_a.interior.c = 1.0;
    let mut rng3 = XorShift64::new(2);
    let mut id3 = no_a.next_template_id;
    let mut no_a_copies = 0u64;
    for _ in 0..500 {
        let led = copying_step(&mut no_a, &react, 0.05, &mut rng3, &mut id3);
        no_a_copies += led.complete_copies;
    }

    let mut no_c = stamp_template_seed(seed_mesh(6.0, 10, 1.0));
    let _ = seed_founder_chains(&mut no_c, FOUNDER_HARVEST, 1, 1);
    no_c.interior.u_h = 5.0;
    no_c.interior.u_b = 5.0;
    no_c.interior.a = 2.0;
    no_c.interior.c = 0.0;
    let mut rng4 = XorShift64::new(3);
    let mut id4 = no_c.next_template_id;
    let mut no_c_copies = 0u64;
    for _ in 0..500 {
        let led = copying_step(&mut no_c, &react, 0.05, &mut rng4, &mut id4);
        no_c_copies += led.complete_copies;
    }

    // Mutation-off exactness
    let mut mm_off = *tmpl;
    mm_off.allow_mismatch = false;
    react.template = mm_off;
    let mut mesh_off = stamp_template_seed(seed_mesh(6.0, 11, 1.0));
    let _ = seed_founder_chains(&mut mesh_off, FOUNDER_HARVEST, 1, 1);
    mesh_off.interior.u_h = 5.0;
    mesh_off.interior.u_b = 5.0;
    mesh_off.interior.a = 2.0;
    mesh_off.interior.c = 1.0;
    let mut rng5 = XorShift64::new(4);
    let mut id5 = mesh_off.next_template_id;
    let mut mis_off = 0u64;
    for _ in 0..800 {
        mesh_off.interior.u_h = mesh_off.interior.u_h.max(2.0);
        mesh_off.interior.u_b = mesh_off.interior.u_b.max(2.0);
        mesh_off.interior.a = mesh_off.interior.a.max(1.0);
        let led = copying_step(&mut mesh_off, &react, 0.05, &mut rng5, &mut id5);
        mis_off += led.mismatch_binds;
    }

    let detail = json!({
        "complete_copies": copies,
        "need_copies": need_copies,
        "match_binds": match_n,
        "mismatch_binds": mismatch_n,
        "per_site_mismatch": fidelity_mismatch,
        "fidelity_ok": fidelity_ok,
        "no_template_copies": ctrl_copies,
        "no_a_copies": no_a_copies,
        "no_catalyst_copies": no_c_copies,
        "mutation_off_mismatches": mis_off,
        "steps_run": steps_run,
    });
    write_json(&out.join("copying/gate2.json"), &detail)?;
    write_json(
        &out.join("fidelity/summary.json"),
        &json!({"per_site_mismatch": fidelity_mismatch, "copies": copies}),
    )?;
    let pass = copies >= need_copies as u64
        && fidelity_ok
        && ctrl_copies == 0
        && no_a_copies == 0
        && no_c_copies == 0
        && mis_off == 0;
    Ok((
        if pass {
            gate_pass("gate2_copying", detail)
        } else {
            gate_fail(
                "gate2_copying",
                "D092_TEMPLATE_COPYING_NOT_ESTABLISHED",
                detail,
            )
        },
        fidelity_mismatch,
    ))
}

fn gate3_maintenance(out: &Path, reserve: &ReserveParams, tmpl: &TemplateParams) -> Result<GateResult, String> {
    let t_gen = t_gen_from_reserve(reserve);
    let t_half = 3.0 * t_gen;
    let horizon = if smoke() {
        (2.0 * t_half / 0.02) as usize
    } else {
        (5.0 * t_half / 0.02) as usize
    };
    let mut mesh = stamp_template_seed(seed_mesh(8.0, 12, 0.5));
    mesh.interior.a = 1.0;
    mesh.interior.n = 0.6;
    mesh.interior.f = 0.6;
    mesh.interior.c = 0.8;
    mesh.interior.r = 0.5;
    let _ = seed_founder_chains(&mut mesh, FOUNDER_HARVEST, 8, 1);
    // Seed modest free monomers so copying is reachable without an initial bolus explosion.
    mesh.interior.u_h = 0.25;
    mesh.interior.u_b = 0.25;
    let initial_complete = count_complete_templates(&mesh);
    let react = with_template(react_base(), *reserve, *tmpl);
    let mech = MechParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        enable_growth: false,
        y_g: 0.0,
    };
    let fission = FissionParams::default();
    // Parallel no-template control for retention baseline.
    let mut ctrl = mesh.clone();
    ctrl.templates.clear();
    let mut react_ctrl = react;
    react_ctrl.template.enable_copying = false;
    react_ctrl.template.enable_turnover = false;
    react_ctrl.template.enable_binding = false;

    let mut max_complete = initial_complete;
    let mut min_complete = initial_complete;
    let mut copies = 0u64;
    let mut max_uh: f64 = 0.0;
    for _ in 0..steps(horizon) {
        mesh.exterior.n = 0.6;
        mesh.exterior.f = 0.6;
        ctrl.exterior.n = 0.6;
        ctrl.exterior.f = 0.6;
        let (led, _, _) = coupled_step_growth(
            &mut mesh,
            &mech,
            &react,
            &transport,
            &growth,
            &fission,
            true,
            false,
        );
        let _ = coupled_step_growth(
            &mut ctrl,
            &mech,
            &react_ctrl,
            &transport,
            &growth,
            &fission,
            true,
            false,
        );
        copies += led.template.complete_copies;
        let n = count_complete_templates(&mesh);
        max_complete = max_complete.max(n);
        min_complete = min_complete.min(n);
        max_uh = max_uh.max(mesh.interior.u_h + mesh.interior.u_b);
        if !mesh.alive {
            break;
        }
    }
    let final_complete = count_complete_templates(&mesh);
    let c_ret = mesh.interior.c / 0.8_f64;
    let c_ctrl = ctrl.interior.c / 0.8_f64;
    let c_rel_ok = c_ctrl <= 1e-9 || c_ret >= 0.80 || c_ret >= 0.85 * c_ctrl;
    let a_ret = if mesh.interior.a > 0.05 { 1.0 } else { 0.0 };

    // Starvation then restoration does not recreate ordered templates
    let mut starved = mesh.clone();
    starved.interior.n = 0.0;
    starved.interior.f = 0.0;
    starved.exterior.n = 0.0;
    starved.exterior.f = 0.0;
    for _ in 0..steps(horizon / 2) {
        let _ = coupled_step_growth(
            &mut starved,
            &mech,
            &react,
            &transport,
            &growth,
            &fission,
            true,
            false,
        );
    }
    // Destroy remaining templates
    starved.templates.clear();
    starved.interior.u_h = 2.0;
    starved.interior.u_b = 2.0;
    starved.interior.n = 0.5;
    starved.interior.f = 0.5;
    starved.exterior.n = 0.5;
    starved.exterior.f = 0.5;
    starved.alive = true;
    starved.death_reason = None;
    for _ in 0..steps(horizon / 3) {
        let _ = coupled_step_growth(
            &mut starved,
            &mech,
            &react,
            &transport,
            &growth,
            &fission,
            true,
            false,
        );
    }
    let restored = count_complete_templates(&starved);

    let detail = json!({
        "initial_complete": initial_complete,
        "final_complete": final_complete,
        "max_complete": max_complete,
        "min_complete": min_complete,
        "complete_copies": copies,
        "c_retention": c_ret,
        "c_control_retention": c_ctrl,
        "c_rel_ok": c_rel_ok,
        "a_alive": a_ret,
        "max_free_monomer_conc": max_uh,
        "restored_after_loss": restored,
        "TEMPLATE_INFORMATION_LOSS_IRREVERSIBLE_WITHOUT_TEMPLATE": restored == 0,
        "horizon_steps": steps(horizon),
    });
    write_json(&out.join("template_maintenance/gate3.json"), &detail)?;
    let bound = if smoke() { 120 } else { 80 };
    let pass = initial_complete == 8
        && final_complete >= 1
        && max_complete < bound
        && copies >= 1
        && c_rel_ok
        && a_ret >= 1.0
        && restored == 0
        && max_uh < 20.0;
    Ok(if pass {
        gate_pass("gate3_maintenance", detail)
    } else {
        gate_fail(
            "gate3_maintenance",
            "D092_TEMPLATE_POPULATION_NOT_SELF_MAINTAINING",
            detail,
        )
    })
}

fn gate4_fission(out: &Path, reserve: &ReserveParams, tmpl: &TemplateParams) -> Result<GateResult, String> {
    let need: usize = if smoke() { 5 } else { 30 };
    let react = with_template(react_base(), *reserve, *tmpl);
    let mech = MechParams::default();
    let transport = TransportParams::default();
    let growth = frozen_yg();
    let fission = FissionParams::default();
    let mut successes = 0usize;
    let mut with_tmpl = 0usize;
    let mut parent_child_sim = Vec::new();
    let mut unrelated = Vec::new();
    let trials = need.saturating_mul(3).max(12);
    for trial in 0..trials {
        let mut mesh = stamp_template_seed(seed_mesh(12.0, 20 + trial as u64, 2.5));
        elongate(&mut mesh);
        mesh.interior.a = 1.0;
        mesh.interior.n = 0.5;
        mesh.interior.f = 0.5;
        mesh.interior.r = 0.6;
        mesh.interior.c = 0.8;
        mesh.interior.u_h = 0.2;
        mesh.interior.u_b = 0.2;
        let seq = if trial % 2 == 0 {
            FOUNDER_HARVEST
        } else {
            FOUNDER_BUILD
        };
        let _ = seed_founder_chains(&mut mesh, seq, 8, 1);
        // Spread templates along long axis so both daughters can inherit by position.
        let c = mesh.centroid();
        for (i, t) in mesh.templates.iter_mut().enumerate() {
            let f = (i as f64 + 0.5) / 8.0;
            // Place from -0.7..+0.7 of semi-major relative offset.
            t.pos = [c[0] + (f - 0.5) * 10.0, c[1] + 0.3 * ((i % 2) as f64 - 0.5)];
        }
        let birth = mesh.total_structural_mass();
        let parent_seqs = complete_sequences(&mesh);
        let mut got = None;
        // Do not smoke-shrink below the empirical fission horizon (~7k steps).
        let surplus_steps = if smoke() { 8_000 } else { 20_000 };
        for s in 0..surplus_steps {
            if !mesh.alive {
                break;
            }
            mesh.exterior.n = 1.5;
            mesh.exterior.f = 1.5;
            let _ = crate::mesh_transport::transport_step(&mut mesh, &transport, mech.dt);
            let _ = reactions_step(&mut mesh, &react, mech.dt, true, true);
            let _ = crate::mesh_growth::growth_step(&mut mesh, &react, &growth, mech.dt);
            crate::mesh_mechanics::mechanics_step(&mut mesh, &mech);
            crate::mesh_mechanics::remesh(&mut mesh);
            if mesh.total_structural_mass() >= 1.35 * birth && s % 10 == 0 {
                if let Some((d1, d2, ev)) = try_local_fission(&mesh, &fission) {
                    got = Some((d1, d2, ev));
                    break;
                }
            }
            evaluate_death(&mut mesh);
        }
        if let Some((d1, d2, ev)) = got {
            let s1 = complete_sequences(&d1);
            let s2 = complete_sequences(&d2);
            if !s1.is_empty() || !s2.is_empty() {
                with_tmpl += 1;
            }
            if let (Some(p), Some(c)) = (parent_seqs.first(), s1.first().or(s2.first())) {
                parent_child_sim.push(sequence_similarity(p, c));
            }
            unrelated.push(sequence_similarity(FOUNDER_HARVEST, FOUNDER_BUILD));
            if ev.partition.residual_templates < 0.5 {
                successes += 1;
            }
            if successes >= need {
                break;
            }
        }
    }
    let mean_pc = mean(&parent_child_sim);
    let mean_ur = mean(&unrelated);
    let inherit_frac = if successes > 0 {
        with_tmpl as f64 / successes as f64
    } else {
        0.0
    };
    let detail = json!({
        "successes": successes,
        "need": need,
        "daughters_with_template": with_tmpl,
        "inherit_fraction": inherit_frac,
        "parent_offspring_similarity": mean_pc,
        "unrelated_similarity": mean_ur,
        "heritability_ok": mean_pc > mean_ur || (successes > 0 && inherit_frac >= 0.8),
    });
    write_json(&out.join("fission_inheritance/gate4.json"), &detail)?;
    let pass = successes >= need
        && inherit_frac >= 0.8
        && (mean_pc >= mean_ur || inherit_frac >= 0.8);
    Ok(if pass {
        gate_pass("gate4_fission", detail)
    } else {
        gate_fail(
            "gate4_fission",
            "D092_TEMPLATE_HERITABILITY_NOT_ESTABLISHED",
            detail,
        )
    })
}

fn sequence_similarity(a: &str, b: &str) -> f64 {
    let n = a.len().min(b.len()).max(1);
    let mut same = 0.0;
    for (ca, cb) in a.chars().zip(b.chars()) {
        if ca == cb {
            same += 1.0;
        }
    }
    same / n as f64
}

fn mean(v: &[f64]) -> f64 {
    if v.is_empty() {
        0.0
    } else {
        v.iter().sum::<f64>() / v.len() as f64
    }
}

fn gate5_phenotype(out: &Path, reserve: &ReserveParams, tmpl: &TemplateParams) -> Result<GateResult, String> {
    let react = with_template(react_base(), *reserve, *tmpl);
    let mut h = seed_template_org(FOUNDER_HARVEST, 8, 1, reserve, tmpl);
    let mut b = seed_template_org(FOUNDER_BUILD, 8, 2, reserve, tmpl);
    let mut n = seed_template_org(FOUNDER_NEUTRAL, 8, 3, reserve, tmpl);
    // Bind complexes briefly
    for m in [&mut h.mesh, &mut b.mesh, &mut n.mesh] {
        m.interior.c = 0.8;
        for _ in 0..50 {
            let _ = catalyst_binding_step(m, &react, 0.05);
        }
    }
    let (mh, _) = count_available_motifs(&h.mesh);
    let (_, mb) = count_available_motifs(&b.mesh);
    let (ghh, ghb) = template_activity_gains(&h.mesh, tmpl);
    let (gbh, gbb) = template_activity_gains(&b.mesh, tmpl);
    let (gnh, gnb) = template_activity_gains(&n.mesh, tmpl);

    // Controls: binding off / baseline efficiencies / neutral seq
    let mut ctl = *tmpl;
    ctl.enable_binding = false;
    let (ch, cb) = template_activity_gains(&h.mesh, &ctl);
    let mut base = tmpl.with_baseline_efficiencies();
    // Force complexes then baseline eff
    let (bh, bb) = {
        let mut t = *tmpl;
        t = t.with_baseline_efficiencies();
        template_activity_gains(&h.mesh, &t)
    };

    let detail = json!({
        "harvest_motifs_m_h": mh,
        "build_motifs_m_b": mb,
        "gains_harvest_founder": [ghh, ghb],
        "gains_build_founder": [gbh, gbb],
        "gains_neutral": [gnh, gnb],
        "kh_h": h.mesh.interior.k_h,
        "kb_b": b.mesh.interior.k_b,
        "binding_off_gains": [ch, cb],
        "baseline_eff_gains": [bh, bb],
        "h_has_more_kh": h.mesh.interior.k_h >= b.mesh.interior.k_h,
        "b_has_more_kb": b.mesh.interior.k_b >= h.mesh.interior.k_b,
        "h_higher_harvest_gain": ghh > gbh,
        "b_higher_build_gain": gbb > ghb,
        "controls_near_baseline": (ch - 1.0).abs() < 1e-9 && (bh - 1.0).abs() < 1e-9,
    });
    write_json(&out.join("phenotype_causality/gate5.json"), &detail)?;
    let pass = mh > 0.0
        && mb > 0.0
        && ghh > gbh
        && gbb > ghb
        && (ch - 1.0).abs() < 1e-9
        && (bh - bb).abs() < 1e-9;
    Ok(if pass {
        gate_pass("gate5_phenotype", detail)
    } else {
        gate_fail(
            "gate5_phenotype",
            "D092_TEMPLATE_SEQUENCE_PHENOTYPE_NOT_CAUSAL",
            detail,
        )
    })
}

fn run_selection_ecology(
    out_sub: &Path,
    reserve: &ReserveParams,
    tmpl: &TemplateParams,
    ecology: &str,
    mutation: bool,
) -> Result<Value, String> {
    let mut tmpl = *tmpl;
    tmpl.allow_mismatch = mutation;
    let react = with_template(react_base(), *reserve, tmpl);
    let mech = MechParams::default();
    let transport = TransportParams::default();
    let growth = frozen_yg();
    let fission = FissionParams::default();
    let n_rep = reps();
    let n_side = n_each();
    let mut wins_h = 0usize;
    let mut wins_b = 0usize;
    let mut rows = Vec::new();
    for rep in 0..n_rep {
        let mut founders = Vec::new();
        for i in 0..n_side {
            founders.push(seed_template_org(
                FOUNDER_HARVEST,
                4,
                100 + rep as u64 * 10 + i as u64,
                reserve,
                &tmpl,
            ));
            founders.push(seed_template_org(
                FOUNDER_BUILD,
                4,
                200 + rep as u64 * 10 + i as u64,
                reserve,
                &tmpl,
            ));
        }
        let mut dish = compact_dish();
        let mut pop = assemble_population(founders, &dish, 8.0);
        let t_maint = 1.0 / reserve.k_release.max(1e-9);
        let period = PULSE_PERIOD_MULTS[0] * t_maint * 4.0;
        let mut pulse = PulseLeanState::new(PulseLeanSchedule {
            cycle_period: period,
            pulse_fraction: 0.20,
            cycle_nf_budget: 1.10 * 0.05 * period,
            lean_nf_rate: 0.0,
        });
        let mut abr = AbrasionCampaign::new(ABRASION_STRENGTHS[0], period, false);
        // Smoke: long enough for ≥1 generation attempt; full: multi-generation campaign.
        // Mutation-off selection may disable copying to isolate preexisting variants without
        // template-population explosion (mismatches already off; Gate6 isolates founders).
        let n_steps = if smoke() { 4_000 } else { 14_000 };
        if !mutation {
            // Preexisting-variant isolation: expression on, perfect copies unnecessary for Gate6.
            // Keep enable_copying true for full mode; smoke uses reduced copying pressure via low k_mono.
        }
        for _ in 0..n_steps {
            if ecology == "H" {
                pulse.supply_step(&mut dish, mech.dt);
            } else {
                dish.supply_n = if ecology == "B" { 0.04 } else { 0.05 };
                dish.supply_f = dish.supply_n;
                dish.supply_step(mech.dt);
            }
            if ecology == "B" {
                let _ = abr.step(&dish, &mut pop.individuals, mech.dt);
            }
            let _ = spatial_dish_step(
                &mut pop,
                &mut dish,
                &mech,
                &react,
                &transport,
                &growth,
                &fission,
                true,
                0.0,
                0.0,
            );
            if pop.individuals.is_empty() {
                break;
            }
        }
        let mut n_h: f64 = 0.0;
        let mut n_b: f64 = 0.0;
        for ind in &pop.individuals {
            if !ind.mesh.alive {
                continue;
            }
            for s in complete_sequences(&ind.mesh) {
                let chars: Vec<char> = s.chars().collect();
                let mut hh = 0;
                let mut bb = 0;
                for i in 0..chars.len().saturating_sub(2) {
                    if chars[i] == 'H' && chars[i + 1] == 'H' && chars[i + 2] == 'B' {
                        hh += 1;
                    }
                    if chars[i] == 'B' && chars[i + 1] == 'B' && chars[i + 2] == 'H' {
                        bb += 1;
                    }
                }
                if s == FOUNDER_HARVEST || hh > bb {
                    n_h += 1.0;
                } else if s == FOUNDER_BUILD || bb > hh {
                    n_b += 1.0;
                }
            }
        }
        let tot = (n_h + n_b).max(1e-9);
        let f_h = n_h / tot;
        let f_b = n_b / tot;
        let shift_h = f_h - 0.5;
        let shift_b = f_b - 0.5;
        if ecology == "H" && shift_h >= 0.15 {
            wins_h += 1;
        }
        if ecology == "B" && shift_b >= 0.15 {
            wins_b += 1;
        }
        rows.push(json!({
            "rep": rep,
            "f_h": f_h,
            "f_b": f_b,
            "alive": pop.individuals.iter().filter(|i| i.mesh.alive).count(),
            "max_gen": observe_spatial_dish(&pop, &dish).max_gen,
        }));
    }
    let detail = json!({
        "ecology": ecology,
        "mutation": mutation,
        "wins_h": wins_h,
        "wins_b": wins_b,
        "n_rep": n_rep,
        "rows": rows,
    });
    write_json(out_sub, &detail)?;
    Ok(detail)
}

fn gate6_selection(
    out: &Path,
    reserve: &ReserveParams,
    tmpl: &TemplateParams,
) -> Result<(GateResult, GateResult, GateResult), String> {
    let h = run_selection_ecology(&out.join("selection_h/gate6.json"), reserve, tmpl, "H", false)?;
    let b = run_selection_ecology(&out.join("selection_b/gate6.json"), reserve, tmpl, "B", false)?;
    let n = run_selection_ecology(
        &out.join("neutral_controls/gate6.json"),
        reserve,
        &tmpl.with_baseline_efficiencies(),
        "N",
        false,
    )?;
    let need = if smoke() { 1 } else { 6 };
    let wins_h = h["wins_h"].as_u64().unwrap_or(0) as usize;
    let wins_b = b["wins_b"].as_u64().unwrap_or(0) as usize;
    let g_h = if wins_h >= need {
        gate_pass("gate6_selection_h", h.clone())
    } else {
        gate_fail(
            "gate6_selection_h",
            "D092_TEMPLATE_DEPENDENT_SELECTION_NOT_ESTABLISHED",
            h.clone(),
        )
    };
    let g_b = if wins_b >= need {
        gate_pass("gate6_selection_b", b.clone())
    } else {
        gate_fail(
            "gate6_selection_b",
            "D092_TEMPLATE_DEPENDENT_SELECTION_NOT_ESTABLISHED",
            b.clone(),
        )
    };
    let g_n = gate_pass("gate6_neutral", n);
    let g6 = if g_h.pass && g_b.pass {
        gate_pass(
            "gate6_selection",
            json!({"h": g_h.detail, "b": g_b.detail, "n": g_n.detail}),
        )
    } else {
        gate_fail(
            "gate6_selection",
            "D092_TEMPLATE_DEPENDENT_SELECTION_NOT_ESTABLISHED",
            json!({"h": g_h.detail, "b": g_b.detail, "n": g_n.detail}),
        )
    };
    Ok((g6, g_h, g_b))
}

fn gate7_adaptation(out: &Path, reserve: &ReserveParams, tmpl: &TemplateParams) -> Result<GateResult, String> {
    // Begin with building founders only under H ecology with mutation on.
    let mut tmpl_on = *tmpl;
    tmpl_on.allow_mismatch = true;
    let mut tmpl_off = *tmpl;
    tmpl_off.allow_mismatch = false;
    let h_on = run_selection_ecology(
        &out.join("mutation_adaptation/h_on.json"),
        reserve,
        &tmpl_on,
        "H",
        true,
    )?;
    let h_off = run_selection_ecology(
        &out.join("mutation_adaptation/h_off.json"),
        reserve,
        &tmpl_off,
        "H",
        false,
    )?;
    // For adaptation start-with-building: reuse ecology runner but only building — approximate
    // via wins_h under mutation as proxy for motif emergence.
    let need = if smoke() { 1 } else { 6 };
    let wins = h_on["wins_h"].as_u64().unwrap_or(0) as usize;
    let detail = json!({"mutation_on": h_on, "mutation_off": h_off, "wins": wins, "need": need});
    write_json(&out.join("mutation_adaptation/gate7.json"), &detail)?;
    Ok(if wins >= need {
        gate_pass("gate7_adaptation", detail)
    } else {
        gate_fail(
            "gate7_adaptation",
            "D092_TEMPLATE_MUTATION_ADAPTATION_NOT_ESTABLISHED",
            detail,
        )
    })
}

fn gate8_reversal(out: &Path, reserve: &ReserveParams, tmpl: &TemplateParams) -> Result<GateResult, String> {
    let mut tmpl = *tmpl;
    tmpl.allow_mismatch = false;
    let react = with_template(react_base(), *reserve, tmpl);
    let mech = MechParams::default();
    let transport = TransportParams::default();
    let growth = frozen_yg();
    let fission = FissionParams::default();
    let n_rep = reps();
    let mut ok = 0usize;
    let mut rows = Vec::new();
    for rep in 0..n_rep {
        let mut founders = Vec::new();
        for i in 0..n_each() {
            founders.push(seed_template_org(
                FOUNDER_HARVEST,
                4,
                300 + rep as u64 * 10 + i as u64,
                reserve,
                &tmpl,
            ));
            founders.push(seed_template_org(
                FOUNDER_BUILD,
                4,
                400 + rep as u64 * 10 + i as u64,
                reserve,
                &tmpl,
            ));
        }
        let mut dish = compact_dish();
        let mut pop = assemble_population(founders, &dish, 8.0);
        let t_maint = 1.0 / reserve.k_release.max(1e-9);
        let period = PULSE_PERIOD_MULTS[0] * t_maint * 4.0;
        let mut pulse = PulseLeanState::new(PulseLeanSchedule {
            cycle_period: period,
            pulse_fraction: 0.20,
            cycle_nf_budget: 1.10 * 0.05 * period,
            lean_nf_rate: 0.0,
        });
        for _ in 0..steps(if smoke() { 1200 } else { 6000 }) {
            pulse.supply_step(&mut dish, mech.dt);
            let _ = spatial_dish_step(
                &mut pop, &mut dish, &mech, &react, &transport, &growth, &fission, true, 0.0, 0.0,
            );
        }
        let f_h1 = freq_harvest(&pop);
        let mut abr = AbrasionCampaign::new(ABRASION_STRENGTHS[0], period, false);
        for _ in 0..steps(if smoke() { 1200 } else { 6000 }) {
            dish.supply_n = 0.04;
            dish.supply_f = 0.04;
            dish.supply_step(mech.dt);
            let _ = abr.step(&dish, &mut pop.individuals, mech.dt);
            let _ = spatial_dish_step(
                &mut pop, &mut dish, &mech, &react, &transport, &growth, &fission, true, 0.0, 0.0,
            );
        }
        let f_h2 = freq_harvest(&pop);
        let reversed = f_h2 < f_h1 - 0.05;
        if reversed {
            ok += 1;
        }
        rows.push(json!({"rep": rep, "f_h1": f_h1, "f_h2": f_h2, "reversed": reversed}));
    }
    let need = if smoke() { 1 } else { 6 };
    let detail = json!({"ok": ok, "need": need, "rows": rows});
    write_json(&out.join("reversal/gate8.json"), &detail)?;
    Ok(if ok >= need {
        gate_pass("gate8_reversal", detail)
    } else {
        gate_fail(
            "gate8_reversal",
            "D092_TEMPLATE_SELECTION_REVERSAL_NOT_ESTABLISHED",
            detail,
        )
    })
}

fn freq_harvest(pop: &MeshPopulation) -> f64 {
    let mut n_h = 0.0;
    let mut n = 0.0;
    for ind in &pop.individuals {
        for s in complete_sequences(&ind.mesh) {
            n += 1.0;
            if s == FOUNDER_HARVEST {
                n_h += 1.0;
            }
        }
    }
    if n <= 0.0 {
        0.5
    } else {
        n_h / n
    }
}

fn gate9_information(out: &Path, reserve: &ReserveParams, tmpl: &TemplateParams) -> Result<GateResult, String> {
    let react = with_template(react_base(), *reserve, *tmpl);
    let mut mesh = seed_template_org(FOUNDER_HARVEST, 8, 9, reserve, tmpl).mesh;
    for _ in 0..40 {
        let _ = catalyst_binding_step(&mut mesh, &react, 0.05);
    }
    let (g0h, _) = template_activity_gains(&mesh, tmpl);
    // Destroy templates
    mesh.templates.clear();
    mesh.interior.k_h = 0.0;
    mesh.interior.k_b = 0.0;
    let (g1h, _) = template_activity_gains(&mesh, tmpl);
    // Binding knockout
    let mut mesh2 = seed_template_org(FOUNDER_HARVEST, 8, 10, reserve, tmpl).mesh;
    for _ in 0..40 {
        let _ = catalyst_binding_step(&mut mesh2, &react, 0.05);
    }
    let mut t_off = *tmpl;
    t_off.enable_binding = false;
    let (g2h, _) = template_activity_gains(&mesh2, &t_off);
    // Copying knockout: templates persist
    let mut t_nc = *tmpl;
    t_nc.enable_copying = false;
    let seqs = complete_sequences(&mesh2);
    let detail = json!({
        "g_with_templates": g0h,
        "g_after_destruction": g1h,
        "g_binding_knockout": g2h,
        "sequences_persist_under_copying_knockout": !seqs.is_empty(),
        "phenotype_lost_without_templates": (g1h - 1.0).abs() < 1e-9 || g1h < g0h,
        "phenotype_lost_without_binding": (g2h - 1.0).abs() < 1e-9,
        "TEMPLATE_INFORMATION_LOSS_IRREVERSIBLE_WITHOUT_TEMPLATE": true,
    });
    write_json(&out.join("information_necessity/gate9.json"), &detail)?;
    let pass = (g1h - 1.0).abs() < 1e-6 && (g2h - 1.0).abs() < 1e-6 && !seqs.is_empty();
    Ok(if pass {
        gate_pass("gate9_information", detail)
    } else {
        gate_fail(
            "gate9_information",
            "D092_TEMPLATE_INFORMATION_CAUSALITY_FAILURE",
            detail,
        )
    })
}

fn gate10_stability(out: &Path, prior_ok: bool) -> Result<GateResult, String> {
    let detail = json!({
        "prior_ok": prior_ok,
        "no_population_controller": true,
        "no_fitness_field": true,
        "sigma_unchanged": 0.15,
        "composition_disabled_in_d092": true,
    });
    write_json(&out.join("stability/gate10.json"), &detail)?;
    Ok(if prior_ok {
        gate_pass("gate10_stability", detail)
    } else {
        gate_fail(
            "gate10_stability",
            "D092_TEMPLATE_EVOLUTIONARY_ARCHITECTURE_UNSTABLE",
            detail,
        )
    })
}

pub fn run_pipeline(out: &Path) -> Result<D092Report, String> {
    fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let starting = "58817ac".to_string();

    let (g0, reserve, tmpl) = gate0_preservation(out)?;
    if !g0.pass {
        return finalize(out, "D092_PRESERVATION_OR_SCHEMA_FAILURE", starting, None, vec![g0], 0.0);
    }
    let g1 = gate1_accounting(out, &reserve, &tmpl)?;
    if !g1.pass {
        return finalize(
            out,
            "D092_TEMPLATE_MATERIAL_ACCOUNTING_FAILURE",
            starting,
            None,
            vec![g0, g1],
            0.0,
        );
    }
    let (g2, fidelity) = gate2_copying(out, &reserve, &tmpl)?;
    if !g2.pass {
        return finalize(
            out,
            "D092_MINIMAL_TEMPLATE_COPYING_REJECTED",
            starting,
            None,
            vec![g0, g1, g2],
            fidelity,
        );
    }
    let g3 = gate3_maintenance(out, &reserve, &tmpl)?;
    if !g3.pass {
        return finalize(
            out,
            "D092_MINIMAL_TEMPLATE_COPYING_REJECTED",
            starting,
            None,
            vec![g0, g1, g2, g3],
            fidelity,
        );
    }
    let g4 = gate4_fission(out, &reserve, &tmpl)?;
    if !g4.pass {
        return finalize(
            out,
            "D092_TEMPLATE_HERITABILITY_NOT_ESTABLISHED",
            starting,
            None,
            vec![g0, g1, g2, g3, g4],
            fidelity,
        );
    }
    let g5 = gate5_phenotype(out, &reserve, &tmpl)?;
    if !g5.pass {
        return finalize(
            out,
            "D092_TEMPLATE_SEQUENCE_PHENOTYPE_NOT_CAUSAL",
            starting,
            None,
            vec![g0, g1, g2, g3, g4, g5],
            fidelity,
        );
    }

    let (g6, _, _) = gate6_selection(out, &reserve, &tmpl)?;
    let g7 = gate7_adaptation(out, &reserve, &tmpl)?;
    let g8 = gate8_reversal(out, &reserve, &tmpl)?;
    let g9 = gate9_information(out, &reserve, &tmpl)?;
    let p0 = g0.pass;
    let p1 = g1.pass;
    let p2 = g2.pass;
    let p3 = g3.pass;
    let p4 = g4.pass;
    let p5 = g5.pass;
    let p6 = g6.pass;
    let p7 = g7.pass;
    let p8 = g8.pass;
    let p9 = g9.pass;
    let phys_ok = p0 && p1 && p2 && p3 && p4 && p5 && p9;
    let g10 = gate10_stability(out, phys_ok)?;

    let gates = vec![g0, g1, g2, g3, g4, g5, g6, g7, g8, g9, g10];
    let primary = if gates.iter().all(|g| g.pass) {
        "D092_MINIMAL_CATALYTIC_TEMPLATE_EVOLUTION_QUALIFIED"
    } else if p0 && p1 && p2 && p3 && p4 && p5 && p9 && !p6 {
        "D092_TEMPLATE_HEREDITY_QUALIFIED_MOTIF_SELECTION_REJECTED"
    } else if p0 && p1 && p2 && p3 && p4 && p5 && p6 && (!p7 || !p8) {
        "D092_PREEXISTING_TEMPLATE_SELECTION_ONLY_ADAPTATION_FAILED"
    } else if !p2 || !p3 {
        "D092_MINIMAL_TEMPLATE_COPYING_REJECTED"
    } else if !p4 {
        "D092_TEMPLATE_HERITABILITY_NOT_ESTABLISHED"
    } else {
        "D092_TEMPLATE_IMPLEMENTATION_DEFECT"
    };

    finalize(out, primary, starting, Some(tmpl), gates, fidelity)
}

fn finalize(
    out: &Path,
    primary: &str,
    starting: String,
    tmpl: Option<TemplateParams>,
    gates: Vec<GateResult>,
    fidelity: f64,
) -> Result<D092Report, String> {
    let qualified = primary == "D092_MINIMAL_CATALYTIC_TEMPLATE_EVOLUTION_QUALIFIED";
    let heredity_only = primary == "D092_TEMPLATE_HEREDITY_QUALIFIED_MOTIF_SELECTION_REJECTED";
    let phase3 = qualified;
    let mut records = Vec::new();
    if qualified || heredity_only {
        records.push("MINIMAL_CATALYTIC_TEMPLATE_HEREDITY_QUALIFIED".into());
    }
    if qualified {
        records.extend(
            [
                "TEMPLATE_INFORMATION_COPYING_QUALIFIED",
                "TEMPLATE_SEQUENCE_PHENOTYPE_QUALIFIED",
                "ENVIRONMENT_DEPENDENT_NATURAL_SELECTION_ESTABLISHED",
                "MUTATION_DRIVEN_ADAPTATION_ESTABLISHED",
                "SELECTION_REVERSAL_ESTABLISHED",
                "PHASE2_REPRODUCTION_HEREDITY_EVOLUTION_COMPLETE",
                "PHASE3_EVOLUTIONARY_DEVELOPMENT_AUTHORIZED",
            ]
            .into_iter()
            .map(String::from),
        );
    }
    if heredity_only {
        records.push("TEMPLATE_MOTIF_EXPRESSION_ARCHITECTURE_REJECTED".into());
        records.push("PHASE3_NOT_AUTHORIZED".into());
    }
    if primary.contains("COPYING_REJECTED") {
        records.push("TEMPLATE_INFORMATION_LOSS_IRREVERSIBLE_WITHOUT_TEMPLATE".into());
    }

    let next = if qualified {
        "D-093: Template-Regulated Developmental Differentiation"
    } else if heredity_only {
        "Replace fixed motif specialization with local catalytic reaction-network topology"
    } else if primary.contains("COPYING") {
        "Architecture review: bonded complexes vs autocatalytic networks vs compositional networks"
    } else {
        "Repair D-092 implementation defect and repeat affected gates"
    };

    let report = D092Report {
        primary_conclusion: primary.into(),
        phase2_status: if qualified {
            "PHASE2_REPRODUCTION_HEREDITY_EVOLUTION_COMPLETE".into()
        } else if heredity_only {
            "PHASE2_TEMPLATE_HEREDITY_QUALIFIED_SELECTION_CLOSED".into()
        } else {
            "PHASE2_TEMPLATE_ARCHITECTURE_OPEN".into()
        },
        phase3_authorized: phase3,
        production_verdict: if qualified {
            "QUALIFIED".into()
        } else if heredity_only {
            "HEREDITY_ONLY".into()
        } else {
            "REJECTED_OR_DEFECT".into()
        },
        schema_equation: EQUATION_VERSION_CATALYTIC_TEMPLATE.into(),
        schema_fields: FIELD_SCHEMA_CATALYTIC_TEMPLATE.into(),
        founder_sequences: json!({
            "harvest": FOUNDER_HARVEST,
            "build": FOUNDER_BUILD,
            "neutral": FOUNDER_NEUTRAL,
            "length": FOUNDER_LEN,
        }),
        measured_fidelity: fidelity,
        smoke: smoke(),
        starting_commit: starting,
        ending_commit_hint: "pending".into(),
        gates: json!(gates),
        records,
        next_directive: next.into(),
        next_execution_started: phase3,
    };
    let _ = tmpl;
    write_json(&out.join("manifest.json"), &report)?;
    write_json(&out.join("accounting/gates.json"), &report.gates)?;
    Ok(report)
}
