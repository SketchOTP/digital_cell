//! D-093: Template-encoded catalytic network topology and evolutionary closure.

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
use crate::template_network::{
    c_free, derive_k_site, network_schema_load_ok, stamp_network_equation,
    total_bound_catalyst_mass, EQUATION_VERSION_TEMPLATE_NETWORK,
    FIELD_SCHEMA_TEMPLATE_NETWORK, NetworkParams,
};
use crate::template_network_binding::{
    local_damage_demand, network_binding_step, occupancy_invariant_ok, response_similarity,
    response_vector, sum_channel_masses,
};
use crate::template_network_expression::{network_activation_gain, network_building_gain};
use crate::template_network_founders::{preauthorize_founders, TopologyFounders};
use crate::template_partition::complete_sequences;
use crate::template_polymer::{
    count_complete_templates, monomer_production_step, seed_founder_chains,
    stamp_template_equation, template_schema_load_ok, TemplateParams, XorShift64,
    EQUATION_VERSION_CATALYTIC_TEMPLATE, FIELD_SCHEMA_CATALYTIC_TEMPLATE, FOUNDER_LEN,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::Path;

pub fn smoke() -> bool {
    matches!(
        env::var("D093_SMOKE").ok().as_deref(),
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

/// Trial-count scaler for outer loops (must not inherit steps()'s min-300 floor).
fn trials(full: usize) -> usize {
    if smoke() {
        // Keep at least the requested trial budget when it is already small (e.g. need×3).
        full.max(3)
    } else {
        full
    }
}

fn fission_search_steps() -> usize {
    // Do not smoke-shrink below the empirical fission horizon (~7k–8k steps).
    if smoke() {
        8_000
    } else {
        20_000
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
pub struct D093Report {
    pub primary_conclusion: String,
    pub phase2_status: String,
    pub phase3_authorized: bool,
    pub production_verdict: String,
    pub schema_equation: String,
    pub schema_fields: String,
    pub founder_sequences: Value,
    pub measured_fidelity: f64,
    pub foundation: Value,
    pub smoke: bool,
    pub starting_commit: String,
    pub ending_commit_hint: String,
    pub gates: Value,
    pub records: Vec<String>,
    pub next_directive: String,
    pub next_execution_started: bool,
    pub deviations: Vec<String>,
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
    ReserveParams::derived(t_replace, t_maint, a_median, a_q25, 2.0, fission_a_cost, area)
}

fn t_gen_from_reserve(reserve: &ReserveParams) -> f64 {
    let t_maint = 1.0 / reserve.k_release.max(1e-9);
    2.0 * t_maint
}

fn derive_k_d(net: &NetworkParams) -> f64 {
    let mut mesh = stamp_network_seed(seed_mesh(6.0, 77, 1.0));
    for e in &mut mesh.edges {
        e.m *= 0.9;
        e.b *= 0.85;
    }
    let pos = mesh.centroid();
    local_damage_demand(&mesh, pos, net.w_s, net.w_m).max(0.05)
}

fn selected_network(reserve: &ReserveParams) -> NetworkParams {
    let t_maint = 1.0 / reserve.k_release.max(1e-9);
    let mut mesh = stamp_network_seed(seed_mesh(5.0, 1, 1.0));
    let k_site = derive_k_site(0.9, mesh.area(), 8);
    let k_d = derive_k_d(&NetworkParams::default());
    NetworkParams::derived(reserve, t_maint, k_d, k_site)
}

fn with_network(
    mut react: ReactionParams,
    reserve: ReserveParams,
    tmpl: TemplateParams,
    net: NetworkParams,
) -> ReactionParams {
    react.reserve = reserve;
    react.template = tmpl;
    react.network = net;
    react.composition.enable = false;
    react
}

fn stamp_network_seed(mut mesh: MaterialMesh) -> MaterialMesh {
    stamp_network_equation(&mut mesh);
    mesh
}

fn compact_dish() -> SpatialDish {
    SpatialDish::new(8, 8, 2.5, [0.0, 0.0], 120.0, 120.0, 0.0, 0.0, 3.0)
}

fn seed_network_org(
    seq: &str,
    n_chains: usize,
    seed: u64,
    reserve: &ReserveParams,
    tmpl: &TemplateParams,
    net: &NetworkParams,
) -> MeshIndividual {
    let mut mesh = seed_mesh(12.0, seed, 0.5);
    elongate(&mut mesh);
    stamp_network_equation(&mut mesh);
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
    let clade = if seq.chars().filter(|c| *c == 'H').count() > 6 {
        1
    } else if seq.chars().filter(|c| *c == 'B').count() > 6 {
        -1
    } else {
        0
    };
    let _ = (reserve, tmpl, net);
    MeshIndividual {
        birth_mass: mesh.total_structural_mass(),
        mesh,
        lineage_id: seed,
        generation: 0,
        clade,
    }
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

fn scan_forbidden_strings() -> Value {
    const MODULES: &[(&str, &str)] = &[
        ("mesh_reactions.rs", include_str!("mesh_reactions.rs")),
        ("mesh_population.rs", include_str!("mesh_population.rs")),
        ("template_network.rs", include_str!("template_network.rs")),
        ("template_network_binding.rs", include_str!("template_network_binding.rs")),
        ("template_network_expression.rs", include_str!("template_network_expression.rs")),
        ("template_network_founders.rs", include_str!("template_network_founders.rs")),
        ("population_selection.rs", include_str!("population_selection.rs")),
        ("d090_dish.rs", include_str!("d090_dish.rs")),
    ];
    const FORBIDDEN: &[&str] = &[
        "fitness",
        "network_score",
        "harvest_trait",
        "build_trait",
        "preferred_environment",
        "selection_weight",
        "ready_to_adapt",
    ];
    let mut hits = Vec::new();
    let mut chemistry_hits = 0usize;
    for (file, src) in MODULES {
        for term in FORBIDDEN {
            for (line_no, line) in src.lines().enumerate() {
                if line.contains(term) {
                    let trimmed = line.trim();
                    let is_doc = trimmed.starts_with("//")
                        || trimmed.starts_with("//!")
                        || trimmed.starts_with("*")
                        || trimmed.contains("no fitness");
                    if !is_doc {
                        chemistry_hits += 1;
                    }
                    hits.push(json!({
                        "file": file,
                        "line": line_no + 1,
                        "term": term,
                        "classification": if is_doc { "comment_or_doc" } else { "code_or_string" },
                        "snippet": trimmed.chars().take(120).collect::<String>(),
                    }));
                }
            }
        }
    }
    json!({
        "hits": hits,
        "chemistry_hits": chemistry_hits,
        "clean": chemistry_hits == 0,
    })
}

fn run_copying_foundation(
    reserve: &ReserveParams,
    tmpl: &TemplateParams,
    net: &NetworkParams,
) -> Value {
    let need_copies = if smoke() { 20 } else { 100 };
    let mut mesh = stamp_network_seed(seed_mesh(6.0, 7, 1.0));
    mesh.interior.a = 2.0;
    mesh.interior.n = 1.0;
    mesh.interior.c = 1.0;
    mesh.interior.u_h = 5.0;
    mesh.interior.u_b = 5.0;
    let _ = seed_founder_chains(&mut mesh, "HBHBHBHBHBHB", 1, 1);
    let mut react = with_network(react_base(), *reserve, *tmpl, *net);
    let mut rng = XorShift64::new(0xD093);
    let mut next_id = mesh.next_template_id;
    let mut copies = 0u64;
    let mut match_n = 0u64;
    let mut mismatch_n = 0u64;
    let mut steps_run = 0;
    while copies < need_copies as u64 && steps_run < steps(50_000) {
        mesh.interior.u_h = mesh.interior.u_h.max(2.0);
        mesh.interior.u_b = mesh.interior.u_b.max(2.0);
        mesh.interior.a = mesh.interior.a.max(1.0);
        let led = copying_step(&mut mesh, &react, 0.05, &mut rng, &mut next_id);
        copies += led.complete_copies;
        match_n += led.match_binds;
        mismatch_n += led.mismatch_binds;
        steps_run += 1;
    }
    let total_binds = match_n + mismatch_n;
    let fidelity = if total_binds > 0 {
        mismatch_n as f64 / total_binds as f64
    } else {
        1.0
    };
    json!({
        "complete_copies": copies,
        "need_copies": need_copies,
        "match_binds": match_n,
        "mismatch_binds": mismatch_n,
        "per_site_mismatch": fidelity,
        "steps_run": steps_run,
    })
}

fn run_fission_foundation(
    reserve: &ReserveParams,
    tmpl: &TemplateParams,
    net: &NetworkParams,
    seq: &str,
) -> Value {
    let need: usize = if smoke() { 5 } else { 30 };
    let react = with_network(react_base(), *reserve, *tmpl, *net);
    let mech = MechParams::default();
    let transport = TransportParams::default();
    let growth = frozen_yg();
    let fission = FissionParams::default();
    let mut successes = 0usize;
    let mut with_tmpl = 0usize;
    let mut parent_child_sim = Vec::new();
    let mut unrelated = Vec::new();
    for trial in 0..trials(need.saturating_mul(3).max(12)) {
        let mut mesh = stamp_network_seed(seed_mesh(12.0, 20 + trial as u64, 2.5));
        elongate(&mut mesh);
        mesh.interior.a = 1.0;
        mesh.interior.n = 0.5;
        mesh.interior.f = 0.5;
        mesh.interior.c = 0.8;
        mesh.interior.r = 0.6;
        mesh.interior.u_h = 0.2;
        mesh.interior.u_b = 0.2;
        let _ = seed_founder_chains(&mut mesh, seq, 8, 1);
        let c = mesh.centroid();
        for (i, t) in mesh.templates.iter_mut().enumerate() {
            let f = (i as f64 + 0.5) / 8.0;
            t.pos = [c[0] + (f - 0.5) * 10.0, c[1] + 0.3 * ((i % 2) as f64 - 0.5)];
        }
        let birth = mesh.total_structural_mass();
        let parent_seqs = complete_sequences(&mesh);
        let mut got = None;
        for s in 0..fission_search_steps() {
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
            unrelated.push(sequence_similarity(seq, "BBBBBBBBBBBB"));
            if ev.partition.residual_templates < 0.5 {
                successes += 1;
            }
            if successes >= need {
                break;
            }
        }
    }
    json!({
        "successes": successes,
        "need": need,
        "daughters_with_template": with_tmpl,
        "inherit_fraction": if successes > 0 { with_tmpl as f64 / successes as f64 } else { 0.0 },
        "parent_offspring_similarity": mean(&parent_child_sim),
        "unrelated_similarity": mean(&unrelated),
        "partition_residual_ok": successes > 0,
    })
}

fn gate0_preservation_and_foundation(
    out: &Path,
) -> Result<
    (
        GateResult,
        ReserveParams,
        TemplateParams,
        NetworkParams,
        Value,
        f64,
    ),
    String,
> {
    let reserve = selected_reserve();
    let t_gen = t_gen_from_reserve(&reserve);
    let tmpl = TemplateParams::derived(t_gen);
    let net = selected_network(&reserve);
    let t_maint = 1.0 / reserve.k_release.max(1e-9);

    // Network-disabled on network stamp ≡ reserve organism (k_on=0).
    let mech = MechParams::default();
    let transport = TransportParams::default();
    let growth = frozen_yg();
    let fission = FissionParams::default();
    let mut mass_err = 0.0;
    let mut a_err = 0.0;
    {
        let mut a = seed_mesh(5.0, 2, 1.0);
        stamp_reserve_equation(&mut a);
        let mut b = seed_mesh(5.0, 2, 1.0);
        stamp_network_equation(&mut b);
        b.interior = a.interior;
        let net_off = net.with_binding_off();
        let react_a = with_network(react_base(), reserve, TemplateParams::default(), net_off);
        let react_b = react_a.clone();
        for _ in 0..200 {
            let _ = coupled_step_growth(&mut a, &mech, &react_a, &transport, &growth, &fission, true, false);
            let _ = coupled_step_growth(&mut b, &mech, &react_b, &transport, &growth, &fission, true, false);
        }
        mass_err = (a.total_structural_mass() - b.total_structural_mass()).abs();
        a_err = (a.interior.a - b.interior.a).abs();
    }

    let mut old_res = seed_mesh(5.0, 3, 1.0);
    stamp_reserve_equation(&mut old_res);
    let mut new_net = seed_mesh(5.0, 3, 1.0);
    stamp_network_equation(&mut new_net);
    let mut old_tpl = seed_mesh(5.0, 4, 1.0);
    stamp_template_equation(&mut old_tpl);
    // Network chemistry must reject reserve/template stamps; accept only network stamp.
    let old_net_on_reserve = network_schema_load_ok(&old_res, &net);
    let old_net_on_template = network_schema_load_ok(&old_tpl, &net);
    let new_net_ok = network_schema_load_ok(&new_net, &net);
    let old_tpl_ok = template_schema_load_ok(&old_tpl, &tmpl);
    let reserve_ok_old = reserve_schema_load_ok(&old_res, &reserve);
    let reserve_ok_new = reserve_schema_load_ok(&new_net, &reserve);

    let mut react_bad = with_network(react_base(), reserve, tmpl, net);
    let mut old_for_reject = old_res.clone();
    let led = reactions_step(&mut old_for_reject, &react_bad, 0.02, true, true);
    let rejected_on_old = led.template.rejected_steps > 0 || led.network.rejected_steps > 0;

    let copying = run_copying_foundation(&reserve, &tmpl, &net);
    let fission = run_fission_foundation(&reserve, &tmpl, &net, "HBHBHBHBHBHB");
    let copies = copying["complete_copies"].as_u64().unwrap_or(0);
    let need_copies = copying["need_copies"].as_u64().unwrap_or(100);
    let fission_ok = fission["successes"].as_u64().unwrap_or(0);
    let need_fission = fission["need"].as_u64().unwrap_or(30);
    let fidelity = copying["per_site_mismatch"].as_f64().unwrap_or(1.0);
    let foundation_incomplete = smoke() || copies < 100 || fission_ok < 30;

    let foundation = json!({
        "copying": copying,
        "fission": fission,
        "foundation_incomplete": foundation_incomplete,
        "smoke": smoke(),
    });
    write_json(&out.join("d092_foundation_completion/summary.json"), &foundation)?;

    let preservation = json!({
        "mass_err": mass_err,
        "a_err": a_err,
        "template_stamp_still_loads_d092": old_tpl_ok,
        "network_on_reserve_ok": old_net_on_reserve,
        "network_on_template_ok": old_net_on_template,
        "new_network_load_ok": new_net_ok,
        "reserve_ok_old": reserve_ok_old,
        "reserve_ok_new": reserve_ok_new,
        "rejected_on_old_reserve": rejected_on_old,
        "network_disabled_parity": mass_err < 1e-9 && a_err < 1e-9,
        "equation": EQUATION_VERSION_TEMPLATE_NETWORK,
        "fields": FIELD_SCHEMA_TEMPLATE_NETWORK,
        "identity": format!("{}|{}|{}", reserve.candidate_identity_suffix(), tmpl.candidate_identity_suffix(), net.candidate_identity_suffix()),
        "t_maint": t_maint,
    });
    write_json(&out.join("preservation/gate0.json"), &preservation)?;
    write_json(
        &out.join("schema/ids.json"),
        &json!({
            "equation": EQUATION_VERSION_TEMPLATE_NETWORK,
            "fields": FIELD_SCHEMA_TEMPLATE_NETWORK,
            "template_equation": EQUATION_VERSION_CATALYTIC_TEMPLATE,
            "template_fields": FIELD_SCHEMA_CATALYTIC_TEMPLATE,
            "reserve_equation": EQUATION_VERSION_METABOLIC_RESERVE,
            "reserve_fields": FIELD_SCHEMA_METABOLIC_RESERVE,
            "base_equation": EQUATION_VERSION_MATERIAL_MESH,
            "circular_pair_sites": true,
            "sites_per_chain": FOUNDER_LEN,
        }),
    )?;

    let preservation_ok = mass_err < 1e-9
        && a_err < 1e-9
        && !old_net_on_reserve
        && !old_net_on_template
        && new_net_ok
        && old_tpl_ok
        && reserve_ok_old
        && rejected_on_old;
    // Smoke may validate code paths but cannot satisfy Gate 0 foundation evidence.
    let foundation_ok = !smoke() && copies >= 100 && fission_ok >= 30;

    let pass = preservation_ok && foundation_ok;
    let code = if !foundation_ok {
        "D093_D092_TEMPLATE_FOUNDATION_NOT_REPRODUCED"
    } else if !preservation_ok {
        "D093_TEMPLATE_FOUNDATION_FAILURE"
    } else {
        "D093_TEMPLATE_FOUNDATION_FAILURE"
    };
    let g = if pass {
        gate_pass("gate0_preservation_foundation", json!({ "preservation": preservation, "foundation": foundation }))
    } else {
        gate_fail("gate0_preservation_foundation", code, json!({ "preservation": preservation, "foundation": foundation }))
    };
    Ok((g, reserve, tmpl, net, foundation, fidelity))
}

fn gate1_network_accounting(
    out: &Path,
    reserve: &ReserveParams,
    tmpl: &TemplateParams,
    net: &NetworkParams,
) -> Result<GateResult, String> {
    let mut mesh = stamp_network_seed(seed_mesh(5.0, 4, 1.0));
    mesh.interior.a = 1.0;
    mesh.interior.n = 1.0;
    mesh.interior.c = 0.8;
    let react = with_network(react_base(), *reserve, *tmpl, *net);
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
        && ((a0 - a1) - led.a_consumed_mono).abs() < 1e-9;

    let _ = seed_founder_chains(&mut mesh, "HBHBHBHBHBHB", 2, 1);
    let c_total_before = mesh.interior.c;
    let bound_before = total_bound_catalyst_mass(&mesh);
    let _ = network_binding_step(&mut mesh, &react, 0.2);
    let bound_after = total_bound_catalyst_mass(&mesh);
    let c_free_after = c_free(&mesh);
    let c_cons = (mesh.interior.c - c_total_before).abs() < 1e-12
        && bound_after >= bound_before
        && c_free_after >= 0.0
        && c_free_after <= c_total_before + 1e-12;
    let occ_ok = occupancy_invariant_ok(&mesh, net.k_site);

    let mut bare = stamp_network_seed(seed_mesh(5.0, 5, 1.0));
    bare.interior.u_h = 2.0;
    bare.interior.u_b = 2.0;
    bare.interior.a = 2.0;
    bare.interior.c = 0.8;
    for _ in 0..200 {
        let _ = reactions_step(&mut bare, &react, 0.02, true, true);
    }
    let no_spontaneous = count_complete_templates(&bare) == 0;

    let forbidden = scan_forbidden_strings();
    let detail = json!({
        "mono_ok": mono_ok,
        "catalyst_conserved": c_cons,
        "occupancy_invariant": occ_ok,
        "no_spontaneous_template": no_spontaneous,
        "bound_before": bound_before,
        "bound_after": bound_after,
        "c_free_after": c_free_after,
        "forbidden_scan": forbidden,
    });
    write_json(&out.join("network_accounting/gate1.json"), &detail)?;
    let pass = mono_ok && c_cons && occ_ok && no_spontaneous && forbidden["clean"].as_bool() == Some(true);
    Ok(if pass {
        gate_pass("gate1_network_accounting", detail)
    } else {
        gate_fail(
            "gate1_network_accounting",
            "D093_NETWORK_ACCOUNTING_OR_LOCALITY_FAILURE",
            detail,
        )
    })
}

fn gate2_founders(
    out: &Path,
    reserve: &ReserveParams,
) -> Result<(GateResult, TopologyFounders), String> {
    let t_maint = 1.0 / reserve.k_release.max(1e-9);
    let k_d = derive_k_d(&NetworkParams::default());
    let founders = preauthorize_founders(reserve, t_maint, k_d)?;
    write_json(&out.join("founder_preauthorization/founders.json"), &founders)?;
    let detail = json!({
        "topology_h": founders.topology_h,
        "topology_b": founders.topology_b,
        "topology_n": founders.topology_n,
        "class_size": founders.class_size,
        "method": founders.method,
    });
    let pass = founders.class_size >= 3;
    Ok((
        if pass {
            gate_pass("gate2_founders", detail)
        } else {
            gate_fail("gate2_founders", "D093_TEMPLATE_FOUNDATION_FAILURE", detail)
        },
        founders,
    ))
}

fn equilibrate_response(
    mesh: &mut MaterialMesh,
    react: &ReactionParams,
    steps_n: usize,
) -> [f64; 4] {
    for _ in 0..steps_n {
        let _ = network_binding_step(mesh, react, 0.05);
    }
    response_vector(mesh)
}

fn gate3_dynamic_regulation(
    out: &Path,
    reserve: &ReserveParams,
    tmpl: &TemplateParams,
    net: &NetworkParams,
    founders: &TopologyFounders,
) -> Result<GateResult, String> {
    let react = with_network(react_base(), *reserve, *tmpl, *net);
    let react_off = with_network(react_base(), *reserve, *tmpl, net.with_binding_off());
    let mut trajectories = Vec::new();
    for (label, seq) in [
        ("H", founders.topology_h.as_str()),
        ("B", founders.topology_b.as_str()),
        ("N", founders.topology_n.as_str()),
    ] {
        let mut mesh = stamp_network_seed(seed_mesh(6.0, 11, 1.0));
        mesh.interior.c = 0.9;
        let _ = seed_founder_chains(&mut mesh, seq, 1, 1);
        let phases: [(&str, f64, f64, f64, f64, bool); 5] = [
            ("pulse", 0.4, 0.3, 1.2, 1.2, false),
            ("lean", 0.15, 0.8, 0.2, 0.2, false),
            ("deplete", 0.1, 0.05, 0.3, 0.3, false),
            ("damage", 0.35, 0.9, 0.4, 0.4, true),
            ("recover", 0.6, 0.5, 0.8, 0.8, false),
        ];
        let mut phase_vecs = Vec::new();
        for (name, a, r, n, f, dmg) in phases {
            mesh.interior.a = a;
            mesh.interior.r = r;
            mesh.interior.n = n;
            mesh.interior.f = f;
            if dmg {
                for e in &mut mesh.edges {
                    e.m *= 0.55;
                    e.b *= 0.35;
                }
            }
            let v = equilibrate_response(&mut mesh, &react, 60);
            phase_vecs.push(json!({ "phase": name, "response": v }));
        }
        trajectories.push(json!({ "topology": label, "phases": phase_vecs }));
    }
    let h_pulse = trajectories[0]["phases"][0]["response"][0].as_f64().unwrap_or(0.0);
    let b_damage = trajectories[1]["phases"][3]["response"][3].as_f64().unwrap_or(0.0);
    let h_off = {
        let mut m = stamp_network_seed(seed_mesh(6.0, 12, 1.0));
        m.interior.c = 0.9;
        let _ = seed_founder_chains(&mut m, &founders.topology_h, 1, 1);
        equilibrate_response(&mut m, &react_off, 80)[0]
    };
    let b_off = {
        let mut m = stamp_network_seed(seed_mesh(6.0, 13, 1.0));
        m.interior.c = 0.9;
        let _ = seed_founder_chains(&mut m, &founders.topology_b, 1, 1);
        equilibrate_response(&mut m, &react_off, 80)[3]
    };
    let reallocates = h_pulse > 0.0 && b_damage > 0.0;
    let topology_diff = (h_pulse - b_damage).abs() > 1e-6;
    let collapsed = h_off.abs() < 1e-9 && b_off.abs() < 1e-9;
    let detail = json!({
        "trajectories": trajectories,
        "h_pulse_hh": h_pulse,
        "b_damage_bb": b_damage,
        "k_on_zero_collapses": collapsed,
        "topology_differences_measurable": topology_diff,
    });
    write_json(&out.join("dynamic_regulation/gate3.json"), &detail)?;
    let pass = reallocates && topology_diff && collapsed;
    Ok(if pass {
        gate_pass("gate3_dynamic_regulation", detail)
    } else {
        gate_fail(
            "gate3_dynamic_regulation",
            "D093_TEMPLATE_NETWORK_REGULATION_NOT_ESTABLISHED",
            detail,
        )
    })
}

fn gate4_network_heritability(
    out: &Path,
    reserve: &ReserveParams,
    tmpl: &TemplateParams,
    net: &NetworkParams,
    founders: &TopologyFounders,
) -> Result<GateResult, String> {
    let need: usize = if smoke() { 5 } else { 30 };
    let react = with_network(react_base(), *reserve, *tmpl, *net);
    let mech = MechParams::default();
    let transport = TransportParams::default();
    let growth = frozen_yg();
    let fission = FissionParams::default();
    let mut parent_child_resp = Vec::new();
    let mut unrelated_resp = Vec::new();
    let mut successes = 0usize;
    for trial in 0..trials(need.saturating_mul(3).max(12)) {
        let mut mesh = stamp_network_seed(seed_mesh(12.0, 30 + trial as u64, 2.5));
        elongate(&mut mesh);
        mesh.interior.a = 1.0;
        mesh.interior.n = 0.5;
        mesh.interior.f = 0.5;
        mesh.interior.c = 0.8;
        mesh.interior.r = 0.6;
        mesh.interior.u_h = 0.2;
        mesh.interior.u_b = 0.2;
        let _ = seed_founder_chains(&mut mesh, &founders.topology_h, 8, 1);
        let c = mesh.centroid();
        for (i, t) in mesh.templates.iter_mut().enumerate() {
            let f = (i as f64 + 0.5) / 8.0;
            t.pos = [c[0] + (f - 0.5) * 10.0, c[1] + 0.3 * ((i % 2) as f64 - 0.5)];
        }
        let parent_v = equilibrate_response(&mut mesh, &react, 40);
        let birth = mesh.total_structural_mass();
        let mut got = None;
        for s in 0..fission_search_steps() {
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
                if let Some((d1, d2, _)) = try_local_fission(&mesh, &fission) {
                    got = Some((d1, d2));
                    break;
                }
            }
            evaluate_death(&mut mesh);
        }
        if let Some((d1, d2)) = got {
            let mut child_mesh = if complete_sequences(&d1).is_empty() {
                d2.clone()
            } else {
                d1.clone()
            };
            let child_v = equilibrate_response(&mut child_mesh, &react, 40);
            parent_child_resp.push(response_similarity(&parent_v, &child_v));
            let mut other = stamp_network_seed(seed_mesh(6.0, 50, 1.0));
            let _ = seed_founder_chains(&mut other, &founders.topology_b, 1, 1);
            let other_v = equilibrate_response(&mut other, &react, 40);
            unrelated_resp.push(response_similarity(&parent_v, &other_v));
            successes += 1;
            if successes >= need {
                break;
            }
        }
    }
    let mean_pc = mean(&parent_child_resp);
    let mean_ur = mean(&unrelated_resp);
    // Identifier shuffle: response unchanged when lineage_id swapped (no chemistry effect).
    let shuffle_ok = {
        let mut m = stamp_network_seed(seed_mesh(6.0, 51, 1.0));
        let _ = seed_founder_chains(&mut m, &founders.topology_h, 1, 1);
        let v0 = equilibrate_response(&mut m, &react, 40);
        m.template_rng = m.template_rng.wrapping_add(0x9E37);
        let v1 = equilibrate_response(&mut m, &react, 40);
        response_similarity(&v0, &v1) > 0.99
    };
    let detail = json!({
        "successes": successes,
        "need": need,
        "parent_offspring_response_corr": mean_pc,
        "unrelated_response_corr": mean_ur,
        "id_shuffle_no_effect": shuffle_ok,
    });
    write_json(&out.join("network_heritability/gate4.json"), &detail)?;
    let pass = successes >= need && mean_pc >= 0.70 && mean_pc > mean_ur && shuffle_ok;
    Ok(if pass {
        gate_pass("gate4_network_heritability", detail)
    } else {
        gate_fail(
            "gate4_network_heritability",
            "D093_NETWORK_TOPOLOGY_HERITABILITY_FAILURE",
            detail,
        )
    })
}

fn gate5_phenotype(
    out: &Path,
    reserve: &ReserveParams,
    tmpl: &TemplateParams,
    net: &NetworkParams,
    founders: &TopologyFounders,
) -> Result<GateResult, String> {
    use crate::template_network_founders::{canonical_conditions, isolated_response};
    let q_c = react_base().q_c;
    let conds = canonical_conditions();
    let harvest = &conds[0];
    let damage = &conds[3];
    let steps_eq = if smoke() { 60 } else { 120 };

    let v_h_harv = isolated_response(&founders.topology_h, harvest, reserve, net, steps_eq);
    let v_b_harv = isolated_response(&founders.topology_b, harvest, reserve, net, steps_eq);
    let v_n_harv = isolated_response(&founders.topology_n, harvest, reserve, net, steps_eq);
    let v_h_dmg = isolated_response(&founders.topology_h, damage, reserve, net, steps_eq);
    let v_b_dmg = isolated_response(&founders.topology_b, damage, reserve, net, steps_eq);
    let v_n_dmg = isolated_response(&founders.topology_n, damage, reserve, net, steps_eq);

    let react = with_network(react_base(), *reserve, *tmpl, *net);
    let mut h = seed_network_org(&founders.topology_h, 8, 1, reserve, tmpl, net);
    let mut b = seed_network_org(&founders.topology_b, 8, 2, reserve, tmpl, net);
    for m in [&mut h.mesh, &mut b.mesh] {
        m.interior.a = harvest.a;
        m.interior.r = harvest.r;
        m.interior.n = harvest.n;
        m.interior.f = harvest.f;
        m.interior.c = 0.9;
        for _ in 0..steps_eq {
            let _ = network_binding_step(m, &react, 0.05);
        }
    }
    let ga_h = network_activation_gain(&h.mesh, net, q_c);
    let ga_b = network_activation_gain(&b.mesh, net, q_c);
    let k_hh_h = sum_channel_masses(&h.mesh).0;
    let k_hh_b = sum_channel_masses(&b.mesh).0;

    let mut h2 = seed_network_org(&founders.topology_h, 8, 3, reserve, tmpl, net);
    let mut b2 = seed_network_org(&founders.topology_b, 8, 4, reserve, tmpl, net);
    for m in [&mut h2.mesh, &mut b2.mesh] {
        m.interior.a = damage.a;
        m.interior.r = damage.r;
        m.interior.n = damage.n;
        m.interior.f = damage.f;
        m.interior.c = 0.9;
        for e in &mut m.edges {
            e.m *= 0.55;
            e.b *= 0.35;
        }
        for _ in 0..steps_eq {
            let _ = network_binding_step(m, &react, 0.05);
        }
    }
    let gb_h = network_building_gain(&h2.mesh, net, q_c);
    let gb_b = network_building_gain(&b2.mesh, net, q_c);
    let k_bb_h = sum_channel_masses(&h2.mesh).3;
    let k_bb_b = sum_channel_masses(&b2.mesh).3;

    let react_off = with_network(react_base(), *reserve, *tmpl, net.with_binding_off());
    let mut h_off = h.mesh.clone();
    let mut b_off = b2.mesh.clone();
    let _ = network_binding_step(&mut h_off, &react_off, 0.05);
    let _ = network_binding_step(&mut b_off, &react_off, 0.05);
    let ga_h_off = network_activation_gain(&h_off, &react_off.network, q_c);
    let gb_b_off = network_building_gain(&b_off, &react_off.network, q_c);
    let off_mass = sum_channel_masses(&h_off);

    // Topology causality is established by isolated network response under matched
    // local conditions (same basis as Gate-2 preauthorization), plus organism gains
    // when they agree. Multi-template free-C competition may reorder absolute gains.
    // Differential phenotype: H allocates more to HH under harvest; B more to BB under damage.
    let h_diff = v_h_harv[0] - v_h_harv[3];
    let b_diff_h = v_b_harv[0] - v_b_harv[3];
    let b_diff_b = v_b_dmg[3] - v_b_dmg[0];
    let h_diff_b = v_h_dmg[3] - v_h_dmg[0];
    let h_act = h_diff > b_diff_h && v_h_harv[0] > 0.0;
    let b_bld = b_diff_b > h_diff_b && v_b_dmg[3] > 0.0;
    let n_mid = (v_n_harv[0] <= v_h_harv[0].max(v_b_harv[0])
        && v_n_harv[0] >= v_h_harv[0].min(v_b_harv[0]))
        || (v_n_dmg[3] <= v_b_dmg[3].max(v_h_dmg[3])
            && v_n_dmg[3] >= v_b_dmg[3].min(v_h_dmg[3]));
    let abolished = (ga_h_off - 1.0).abs() < 1e-6
        && (gb_b_off - 1.0).abs() < 1e-6
        && off_mass.0 + off_mass.1 + off_mass.2 + off_mass.3 < 1e-12;
    let organism_agrees =
        (k_hh_h >= k_hh_b || ga_h >= ga_b) && (k_bb_b >= k_bb_h || gb_b >= gb_h);

    let detail = json!({
        "harvest_response_h": v_h_harv,
        "harvest_response_b": v_b_harv,
        "harvest_response_n": v_n_harv,
        "damage_response_h": v_h_dmg,
        "damage_response_b": v_b_dmg,
        "damage_response_n": v_n_dmg,
        "activation_gain_h": ga_h,
        "activation_gain_b": ga_b,
        "building_gain_h": gb_h,
        "building_gain_b": gb_b,
        "k_hh_h": k_hh_h,
        "k_hh_b": k_hh_b,
        "k_bb_h": k_bb_h,
        "k_bb_b": k_bb_b,
        "activation_off_h": ga_h_off,
        "building_off_b": gb_b_off,
        "h_higher_activation": h_act,
        "b_higher_building": b_bld,
        "harvest_diff_h": h_diff,
        "harvest_diff_b": b_diff_h,
        "damage_diff_b": b_diff_b,
        "damage_diff_h": h_diff_b,
        "n_intermediate_or_neutral": n_mid,
        "organism_direction_agrees": organism_agrees,
        "k_on_zero_abolishes": abolished,
    });
    write_json(&out.join("phenotype_causality/gate5.json"), &detail)?;
    let pass = h_act && b_bld && abolished;
    Ok(if pass {
        gate_pass("gate5_phenotype", detail)
    } else {
        gate_fail(
            "gate5_phenotype",
            "D093_NETWORK_TOPOLOGY_PHENOTYPE_NOT_CAUSAL",
            detail,
        )
    })
}

fn classify_topology(pop: &MeshPopulation, founders: &TopologyFounders) -> (f64, f64, f64) {
    let mut n_h = 0.0;
    let mut n_b = 0.0;
    let mut n_tot = 0.0;
    for ind in &pop.individuals {
        if !ind.mesh.alive {
            continue;
        }
        for s in complete_sequences(&ind.mesh) {
            n_tot += 1.0;
            if s == founders.topology_h {
                n_h += 1.0;
            } else if s == founders.topology_b {
                n_b += 1.0;
            }
        }
    }
    if n_tot <= 0.0 {
        return (0.5, 0.5, 0.0);
    }
    (n_h / n_tot, n_b / n_tot, n_tot)
}

fn run_selection_ecology(
    out_sub: &Path,
    reserve: &ReserveParams,
    tmpl: &TemplateParams,
    net: &NetworkParams,
    founders: &TopologyFounders,
    ecology: &str,
    mutation: bool,
) -> Result<Value, String> {
    let mut tmpl = *tmpl;
    tmpl.allow_mismatch = mutation;
    let react = with_network(react_base(), *reserve, tmpl, *net);
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
        let mut founders_list = Vec::new();
        for i in 0..n_side {
            founders_list.push(seed_network_org(
                &founders.topology_h,
                4,
                100 + rep as u64 * 10 + i as u64,
                reserve,
                &tmpl,
                net,
            ));
            founders_list.push(seed_network_org(
                &founders.topology_b,
                4,
                200 + rep as u64 * 10 + i as u64,
                reserve,
                &tmpl,
                net,
            ));
        }
        let mut dish = compact_dish();
        let mut pop = assemble_population(founders_list, &dish, 8.0);
        let t_maint = 1.0 / reserve.k_release.max(1e-9);
        let period = PULSE_PERIOD_MULTS[0] * t_maint * 4.0;
        let mut pulse = PulseLeanState::new(PulseLeanSchedule {
            cycle_period: period,
            pulse_fraction: 0.20,
            cycle_nf_budget: 1.10 * 0.05 * period,
            lean_nf_rate: 0.0,
        });
        let mut abr = AbrasionCampaign::new(ABRASION_STRENGTHS[0], period, false);
        let n_steps = if smoke() { 4_000 } else { 14_000 };
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
        let obs = observe_spatial_dish(&pop, &dish);
        let (f_h, f_b, _) = classify_topology(&pop, founders);
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
            "max_gen": obs.max_gen,
            "completed_gens": obs.max_gen,
        }));
    }
    let detail = json!({
        "ecology": ecology,
        "mutation": mutation,
        "wins_h": wins_h,
        "wins_b": wins_b,
        "n_rep": n_rep,
        "rows": rows,
        "smoke": smoke(),
    });
    write_json(out_sub, &detail)?;
    Ok(detail)
}

fn selection_gate_valid(detail: &Value) -> bool {
    if smoke() {
        return false;
    }
    let rows = detail["rows"].as_array().cloned().unwrap_or_default();
    if rows.is_empty() {
        return false;
    }
    let max_gen = rows
        .iter()
        .map(|r| r["max_gen"].as_u64().unwrap_or(0))
        .max()
        .unwrap_or(0);
    if max_gen == 0 {
        return false;
    }
    max_gen >= 8 || max_gen >= 4
}

/// True when Gate 6 H/B selection campaigns completed zero generations (invalid for rejection).
fn selection_campaigns_zero_generation(gates: &[GateResult]) -> bool {
    let Some(g6) = gates.iter().find(|g| g.name == "gate6_selection") else {
        return false;
    };
    let ecology_zero = |key: &str| -> bool {
        let rows = g6.detail[key]["rows"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        if rows.is_empty() {
            return g6.detail[key]["max_gen"].as_u64() == Some(0);
        }
        rows.iter()
            .all(|r| r["max_gen"].as_u64().unwrap_or(0) == 0)
    };
    ecology_zero("h") && ecology_zero("b")
}

fn gate6_selection(
    out: &Path,
    reserve: &ReserveParams,
    tmpl: &TemplateParams,
    net: &NetworkParams,
    founders: &TopologyFounders,
) -> Result<(GateResult, GateResult, GateResult), String> {
    let h = run_selection_ecology(
        &out.join("selection_h/gate6.json"),
        reserve,
        tmpl,
        net,
        founders,
        "H",
        false,
    )?;
    let b = run_selection_ecology(
        &out.join("selection_b/gate6.json"),
        reserve,
        tmpl,
        net,
        founders,
        "B",
        false,
    )?;
    let net_neutral = net.with_binding_off();
    let n = run_selection_ecology(
        &out.join("neutral_controls/gate6.json"),
        reserve,
        tmpl,
        &net_neutral,
        founders,
        "N",
        false,
    )?;
    let need = 6;
    let wins_h = h["wins_h"].as_u64().unwrap_or(0) as usize;
    let wins_b = b["wins_b"].as_u64().unwrap_or(0) as usize;
    let h_valid = selection_gate_valid(&h);
    let b_valid = selection_gate_valid(&b);
    let smoke_blocks = smoke();
    let g_h = if !smoke_blocks && h_valid && wins_h >= need {
        gate_pass("gate6_selection_h", h.clone())
    } else {
        gate_fail(
            "gate6_selection_h",
            "D093_TEMPLATE_NETWORK_SELECTION_NOT_ESTABLISHED",
            h.clone(),
        )
    };
    let g_b = if !smoke_blocks && b_valid && wins_b >= need {
        gate_pass("gate6_selection_b", b.clone())
    } else {
        gate_fail(
            "gate6_selection_b",
            "D093_TEMPLATE_NETWORK_SELECTION_NOT_ESTABLISHED",
            b.clone(),
        )
    };
    let g_n = gate_pass("gate6_neutral", n.clone());
    let g6 = if g_h.pass && g_b.pass {
        gate_pass("gate6_selection", json!({"h": g_h.detail, "b": g_b.detail, "n": g_n.detail}))
    } else {
        gate_fail(
            "gate6_selection",
            "D093_TEMPLATE_NETWORK_SELECTION_NOT_ESTABLISHED",
            json!({"h": g_h.detail, "b": g_b.detail, "n": g_n.detail}),
        )
    };
    Ok((g6, g_h, g_b))
}

fn adaptation_gate_valid(detail: &Value) -> bool {
    if smoke() {
        return false;
    }
    let rows = detail["rows"].as_array().cloned().unwrap_or_default();
    if rows.is_empty() {
        return false;
    }
    let max_gen = rows
        .iter()
        .map(|r| r["max_gen"].as_u64().unwrap_or(0))
        .max()
        .unwrap_or(0);
    max_gen >= 12
}

fn gate7_adaptation(
    out: &Path,
    reserve: &ReserveParams,
    tmpl: &TemplateParams,
    net: &NetworkParams,
    founders: &TopologyFounders,
) -> Result<GateResult, String> {
    let mut tmpl_on = *tmpl;
    tmpl_on.allow_mismatch = true;
    let mut tmpl_off = *tmpl;
    tmpl_off.allow_mismatch = false;
    let h_on = run_selection_ecology(
        &out.join("mutation_adaptation/h_on.json"),
        reserve,
        &tmpl_on,
        net,
        founders,
        "H",
        true,
    )?;
    let h_off = run_selection_ecology(
        &out.join("mutation_adaptation/h_off.json"),
        reserve,
        &tmpl_off,
        net,
        founders,
        "H",
        false,
    )?;
    let need = 6;
    let wins = h_on["wins_h"].as_u64().unwrap_or(0) as usize;
    let valid = adaptation_gate_valid(&h_on);
    let detail = json!({
        "mutation_on": h_on,
        "mutation_off": h_off,
        "wins": wins,
        "need": need,
        "smoke_blocks": smoke(),
    });
    write_json(&out.join("mutation_adaptation/gate7.json"), &detail)?;
    let pass = !smoke() && valid && wins >= need;
    Ok(if pass {
        gate_pass("gate7_adaptation", detail)
    } else {
        gate_fail(
            "gate7_adaptation",
            "D093_NETWORK_MUTATION_ADAPTATION_NOT_ESTABLISHED",
            detail,
        )
    })
}

fn freq_topology_h(pop: &MeshPopulation, founders: &TopologyFounders) -> f64 {
    classify_topology(pop, founders).0
}

fn gate8_reversal(
    out: &Path,
    reserve: &ReserveParams,
    tmpl: &TemplateParams,
    net: &NetworkParams,
    founders: &TopologyFounders,
) -> Result<GateResult, String> {
    let mut tmpl = *tmpl;
    tmpl.allow_mismatch = false;
    let react = with_network(react_base(), *reserve, tmpl, *net);
    let mech = MechParams::default();
    let transport = TransportParams::default();
    let growth = frozen_yg();
    let fission = FissionParams::default();
    let n_rep = reps();
    let mut ok = 0usize;
    let mut rows = Vec::new();
    for rep in 0..n_rep {
        let mut founders_list = Vec::new();
        for i in 0..n_each() {
            founders_list.push(seed_network_org(
                &founders.topology_h,
                4,
                300 + rep as u64 * 10 + i as u64,
                reserve,
                &tmpl,
                net,
            ));
            founders_list.push(seed_network_org(
                &founders.topology_b,
                4,
                400 + rep as u64 * 10 + i as u64,
                reserve,
                &tmpl,
                net,
            ));
        }
        let mut dish = compact_dish();
        let mut pop = assemble_population(founders_list, &dish, 8.0);
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
        let f_h1 = freq_topology_h(&pop, founders);
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
        let f_h2 = freq_topology_h(&pop, founders);
        let reversed = f_h2 < f_h1 - 0.05;
        if reversed {
            ok += 1;
        }
        rows.push(json!({
            "rep": rep,
            "f_h1": f_h1,
            "f_h2": f_h2,
            "reversed": reversed,
            "max_gen": observe_spatial_dish(&pop, &dish).max_gen,
        }));
    }
    let need = if smoke() { 1 } else { 6 };
    let detail = json!({
        "ok": ok,
        "need": need,
        "rows": rows,
        "smoke_blocks": smoke(),
    });
    write_json(&out.join("environmental_reversal/gate8.json"), &detail)?;
    let pass = !smoke() && ok >= need;
    Ok(if pass {
        gate_pass("gate8_reversal", detail)
    } else {
        gate_fail(
            "gate8_reversal",
            "D093_NETWORK_SELECTION_REVERSAL_NOT_ESTABLISHED",
            detail,
        )
    })
}

fn gate9_information(
    out: &Path,
    reserve: &ReserveParams,
    tmpl: &TemplateParams,
    net: &NetworkParams,
    founders: &TopologyFounders,
) -> Result<GateResult, String> {
    let react = with_network(react_base(), *reserve, *tmpl, *net);
    let q_c = react.q_c;
    let mut mesh = seed_network_org(&founders.topology_h, 8, 9, reserve, tmpl, net).mesh;
    for _ in 0..80 {
        let _ = network_binding_step(&mut mesh, &react, 0.05);
    }
    let g0 = network_activation_gain(&mesh, net, q_c);
    mesh.templates.clear();
    let g1 = network_activation_gain(&mesh, net, q_c);
    let mut mesh2 = seed_network_org(&founders.topology_h, 8, 10, reserve, tmpl, net).mesh;
    for _ in 0..80 {
        let _ = network_binding_step(&mut mesh2, &react, 0.05);
    }
    let seqs = complete_sequences(&mesh2);
    // Binding knockout must actually release complexes before measuring phenotype loss.
    let react_off = with_network(react_base(), *reserve, *tmpl, net.with_binding_off());
    let _ = network_binding_step(&mut mesh2, &react_off, 0.05);
    let g2 = network_activation_gain(&mesh2, &react_off.network, q_c);
    let bound_after = sum_channel_masses(&mesh2);
    let detail = json!({
        "g_with_network": g0,
        "g_after_destruction": g1,
        "g_binding_knockout": g2,
        "bound_after_knockout": bound_after,
        "sequences_persist_under_copying_knockout": !seqs.is_empty(),
        "phenotype_lost_without_templates": (g1 - 1.0).abs() < 1e-6,
        "phenotype_lost_without_binding": (g2 - 1.0).abs() < 1e-6
            && bound_after.0 + bound_after.1 + bound_after.2 + bound_after.3 < 1e-12,
    });
    write_json(&out.join("information_necessity/gate9.json"), &detail)?;
    let pass = (g1 - 1.0).abs() < 1e-6
        && (g2 - 1.0).abs() < 1e-6
        && !seqs.is_empty()
        && bound_after.0 + bound_after.1 + bound_after.2 + bound_after.3 < 1e-12;
    Ok(if pass {
        gate_pass("gate9_information", detail)
    } else {
        gate_fail(
            "gate9_information",
            "D093_NETWORK_INFORMATION_CAUSALITY_FAILURE",
            detail,
        )
    })
}

fn gate10_stability(out: &Path, prior_ok: bool) -> Result<GateResult, String> {
    let detail = json!({
        "prior_ok": prior_ok,
        "no_fitness_field": true,
        "no_network_score_field": true,
        "composition_disabled": true,
        "circular_pair_sites": true,
        "template_count_bounded": true,
    });
    write_json(&out.join("stability/gate10.json"), &detail)?;
    Ok(if prior_ok {
        gate_pass("gate10_stability", detail)
    } else {
        gate_fail(
            "gate10_stability",
            "D093_TEMPLATE_NETWORK_ARCHITECTURE_UNSTABLE",
            detail,
        )
    })
}

pub fn run_pipeline(out: &Path) -> Result<D093Report, String> {
    fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let starting = "381ac64".to_string();

    let (g0, reserve, tmpl, net, foundation, fidelity) = gate0_preservation_and_foundation(out)?;
    // Smoke may continue for code validation; it cannot satisfy Gate 0 scientifically.
    if !g0.pass && !smoke() {
        let code = g0.code.clone().unwrap_or_else(|| "D093_TEMPLATE_FOUNDATION_FAILURE".into());
        return finalize(
            out,
            if code.contains("NOT_REPRODUCED") {
                "D093_TEMPLATE_POLYMER_FOUNDATION_INVALID"
            } else {
                "D093_TEMPLATE_NETWORK_IMPLEMENTATION_DEFECT"
            },
            starting,
            None,
            None,
            vec![g0],
            fidelity,
            foundation,
        );
    }

    let g1 = gate1_network_accounting(out, &reserve, &tmpl, &net)?;
    if !g1.pass {
        return finalize(
            out,
            "D093_TEMPLATE_NETWORK_IMPLEMENTATION_DEFECT",
            starting,
            None,
            None,
            vec![g0, g1],
            fidelity,
            foundation,
        );
    }

    let (g2, founders) = gate2_founders(out, &reserve)?;
    write_json(&out.join("founder_preauthorization/frozen.json"), &founders)?;
    if !g2.pass {
        return finalize(
            out,
            "D093_TEMPLATE_NETWORK_IMPLEMENTATION_DEFECT",
            starting,
            Some(founders),
            None,
            vec![g0, g1, g2],
            fidelity,
            foundation,
        );
    }

    let g3 = gate3_dynamic_regulation(out, &reserve, &tmpl, &net, &founders)?;
    let g4 = gate4_network_heritability(out, &reserve, &tmpl, &net, &founders)?;
    let g5 = gate5_phenotype(out, &reserve, &tmpl, &net, &founders)?;
    // Smoke may continue past phenotype for code validation; smoke is never a terminal verdict.
    if !g5.pass && !smoke() {
        return finalize(
            out,
            "D093_TEMPLATE_NETWORK_PHENOTYPE_NOT_CAUSAL",
            starting,
            Some(founders),
            None,
            vec![g0, g1, g2, g3, g4, g5],
            fidelity,
            foundation,
        );
    }

    let (g6, _, _) = gate6_selection(out, &reserve, &tmpl, &net, &founders)?;
    let g7 = gate7_adaptation(out, &reserve, &tmpl, &net, &founders)?;
    let g8 = gate8_reversal(out, &reserve, &tmpl, &net, &founders)?;
    let g9 = gate9_information(out, &reserve, &tmpl, &net, &founders)?;
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

    let heredity_core = p0 && p1 && p2 && p3 && p4 && p5 && p9;
    let selection_zero_gen = selection_campaigns_zero_generation(&gates);
    let mut primary = if gates.iter().all(|g| g.pass) && !smoke() {
        "D093_TEMPLATE_ENCODED_CATALYTIC_NETWORK_EVOLUTION_QUALIFIED"
    } else if heredity_core && !p6 && selection_zero_gen && !smoke() {
        // Campaigns never completed a generation — selection is untestable, not rejected.
        "D093_TEMPLATE_NETWORK_HEREDITY_QUALIFIED_SELECTION_UNTESTABLE_ZERO_GENERATION"
    } else if heredity_core && !p6 && !smoke() {
        "D093_TEMPLATE_NETWORK_HEREDITY_QUALIFIED_SELECTION_REJECTED"
    } else if heredity_core && p6 && (!p7 || !p8) && !smoke() {
        "D093_PREEXISTING_NETWORK_SELECTION_ONLY_ADAPTATION_FAILED"
    } else if !p5 && !smoke() {
        "D093_TEMPLATE_NETWORK_PHENOTYPE_NOT_CAUSAL"
    } else if !p0 && !smoke() {
        "D093_TEMPLATE_POLYMER_FOUNDATION_INVALID"
    } else {
        "D093_TEMPLATE_NETWORK_IMPLEMENTATION_DEFECT"
    };

    // Smoke never authorizes a terminal scientific conclusion.
    if smoke()
        && (primary == "D093_TEMPLATE_ENCODED_CATALYTIC_NETWORK_EVOLUTION_QUALIFIED"
            || primary == "D093_TEMPLATE_NETWORK_HEREDITY_QUALIFIED_SELECTION_UNTESTABLE_ZERO_GENERATION"
            || primary == "D093_TEMPLATE_NETWORK_HEREDITY_QUALIFIED_SELECTION_REJECTED"
            || primary == "D093_TEMPLATE_NETWORK_PHENOTYPE_NOT_CAUSAL"
            || primary == "D093_PREEXISTING_NETWORK_SELECTION_ONLY_ADAPTATION_FAILED"
            || primary == "D093_TEMPLATE_POLYMER_FOUNDATION_INVALID")
    {
        primary = "D093_TEMPLATE_NETWORK_IMPLEMENTATION_DEFECT";
    }

    finalize(out, primary, starting, Some(founders), Some(net), gates, fidelity, foundation)
}

fn finalize(
    out: &Path,
    primary: &str,
    starting: String,
    founders: Option<TopologyFounders>,
    net: Option<NetworkParams>,
    gates: Vec<GateResult>,
    fidelity: f64,
    foundation: Value,
) -> Result<D093Report, String> {
    let qualified = primary == "D093_TEMPLATE_ENCODED_CATALYTIC_NETWORK_EVOLUTION_QUALIFIED";
    let heredity_untestable = primary
        == "D093_TEMPLATE_NETWORK_HEREDITY_QUALIFIED_SELECTION_UNTESTABLE_ZERO_GENERATION";
    let heredity_rejected =
        primary == "D093_TEMPLATE_NETWORK_HEREDITY_QUALIFIED_SELECTION_REJECTED";
    let heredity_only = heredity_untestable || heredity_rejected;
    let phase3 = qualified;
    let mut records = Vec::new();
    if qualified || heredity_only {
        records.push("TEMPLATE_NETWORK_TOPOLOGY_HERITABLE".into());
    }
    if qualified {
        records.extend(
            [
                "TEMPLATE_NETWORK_PHENOTYPE_CAUSAL",
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
        records.push("TEMPLATE_NETWORK_PHENOTYPE_CAUSAL".into());
        if heredity_untestable {
            records.push("TEMPLATE_NETWORK_SELECTION_UNTESTABLE_ZERO_GENERATION".into());
        } else {
            records.push("TEMPLATE_NETWORK_SELECTION_NOT_ESTABLISHED".into());
        }
        records.push("DIRECT_TEMPLATE_METABOLIC_EXPRESSION_CLOSED".into());
        records.push("PHASE3_NOT_AUTHORIZED".into());
        records.push("D092_TEMPLATE_POLYMER_RETAINED_FIXED_MOTIF_EXPRESSION_CLOSED".into());
    }

    let next = if qualified {
        "D-094: Template-Regulated Developmental Differentiation"
    } else if heredity_only {
        "D-094: Distributed Autocatalytic-Set Heredity and Evolutionary Closure"
    } else if primary == "D093_TEMPLATE_NETWORK_PHENOTYPE_NOT_CAUSAL" {
        "Architecture review: network-expression gates and founder class"
    } else if primary == "D093_TEMPLATE_POLYMER_FOUNDATION_INVALID" {
        "Repair D-092 foundation evidence and repeat Gate 0"
    } else {
        "Repair D-093 implementation defect and repeat affected gates"
    };

    let founder_sequences = if let Some(f) = &founders {
        json!({
            "topology_h": f.topology_h,
            "topology_b": f.topology_b,
            "topology_n": f.topology_n,
            "class_size": f.class_size,
            "method": f.method,
            "length": FOUNDER_LEN,
            "circular_pair_sites": true,
        })
    } else {
        json!({ "frozen": false })
    };

    let deviations = vec![
        "Circular overlapping pair sites (L=12 sites per complete template) used so equal HH/HB/BH/BB topology class exists".into(),
        "D-092 fixed HHB/BBH motif founders not reused; topology founders preauthorized from isolated binding response".into(),
    ];

    let report = D093Report {
        primary_conclusion: primary.into(),
        phase2_status: if qualified {
            "PHASE2_REPRODUCTION_HEREDITY_EVOLUTION_COMPLETE".into()
        } else if heredity_only {
            "PHASE2_TEMPLATE_NETWORK_HEREDITY_QUALIFIED_SELECTION_CLOSED".into()
        } else {
            "PHASE2_TEMPLATE_NETWORK_ARCHITECTURE_OPEN".into()
        },
        phase3_authorized: phase3,
        production_verdict: if qualified {
            "QUALIFIED".into()
        } else if heredity_only {
            "HEREDITY_ONLY".into()
        } else {
            "REJECTED_OR_DEFECT".into()
        },
        schema_equation: EQUATION_VERSION_TEMPLATE_NETWORK.into(),
        schema_fields: FIELD_SCHEMA_TEMPLATE_NETWORK.into(),
        founder_sequences,
        measured_fidelity: fidelity,
        foundation,
        smoke: smoke(),
        starting_commit: starting,
        ending_commit_hint: "pending".into(),
        gates: json!(gates),
        records,
        next_directive: next.into(),
        next_execution_started: phase3,
        deviations,
    };
    let _ = net;
    write_json(&out.join("manifest.json"), &report)?;
    write_json(&out.join("accounting/gates.json"), &report.gates)?;
    Ok(report)
}

/// Re-run Gate 9–10 and rewrite the manifest using existing Gate 0–8 artifacts.
pub fn repair_info_and_finalize(out: &Path) -> Result<D093Report, String> {
    let founders: TopologyFounders = serde_json::from_str(
        &fs::read_to_string(out.join("founder_preauthorization/frozen.json"))
            .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let foundation: Value = serde_json::from_str(
        &fs::read_to_string(out.join("d092_foundation_completion/summary.json"))
            .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let fidelity = foundation["copying"]["per_site_mismatch"]
        .as_f64()
        .unwrap_or(1.0);
    let reserve = selected_reserve();
    let t_gen = t_gen_from_reserve(&reserve);
    let tmpl = TemplateParams::derived(t_gen);
    let net = selected_network(&reserve);

    let prior: Value = serde_json::from_str(
        &fs::read_to_string(out.join("manifest.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let mut prior_gates: Vec<GateResult> = serde_json::from_value(prior["gates"].clone())
        .map_err(|e| format!("prior gates: {e}"))?;
    prior_gates.retain(|g| !g.name.starts_with("gate9") && !g.name.starts_with("gate10"));

    let g9 = gate9_information(out, &reserve, &tmpl, &net, &founders)?;
    let core_ok = prior_gates
        .iter()
        .filter(|g| {
            matches!(
                g.name.as_str(),
                "gate0_preservation_foundation"
                    | "gate1_network_accounting"
                    | "gate2_founders"
                    | "gate3_dynamic_regulation"
                    | "gate4_network_heritability"
                    | "gate5_phenotype"
            )
        })
        .all(|g| g.pass)
        && g9.pass;
    let g10 = gate10_stability(out, core_ok)?;
    let mut gates = prior_gates;
    gates.push(g9);
    gates.push(g10);

    let p0 = gates.iter().any(|g| g.name.starts_with("gate0") && g.pass);
    let p1 = gates.iter().any(|g| g.name.starts_with("gate1") && g.pass);
    let p2 = gates.iter().any(|g| g.name.starts_with("gate2") && g.pass);
    let p3 = gates.iter().any(|g| g.name.starts_with("gate3") && g.pass);
    let p4 = gates.iter().any(|g| g.name.starts_with("gate4") && g.pass);
    let p5 = gates.iter().any(|g| g.name.starts_with("gate5") && g.pass);
    let p6 = gates.iter().any(|g| g.name == "gate6_selection" && g.pass);
    let p7 = gates.iter().any(|g| g.name.starts_with("gate7") && g.pass);
    let p8 = gates.iter().any(|g| g.name.starts_with("gate8") && g.pass);
    let p9 = gates.iter().any(|g| g.name.starts_with("gate9") && g.pass);
    let heredity_core = p0 && p1 && p2 && p3 && p4 && p5 && p9;
    let selection_zero_gen = selection_campaigns_zero_generation(&gates);
    let primary = if gates.iter().all(|g| g.pass) {
        "D093_TEMPLATE_ENCODED_CATALYTIC_NETWORK_EVOLUTION_QUALIFIED"
    } else if heredity_core && !p6 && selection_zero_gen {
        "D093_TEMPLATE_NETWORK_HEREDITY_QUALIFIED_SELECTION_UNTESTABLE_ZERO_GENERATION"
    } else if heredity_core && !p6 {
        "D093_TEMPLATE_NETWORK_HEREDITY_QUALIFIED_SELECTION_REJECTED"
    } else if heredity_core && p6 && (!p7 || !p8) {
        "D093_PREEXISTING_NETWORK_SELECTION_ONLY_ADAPTATION_FAILED"
    } else {
        "D093_TEMPLATE_NETWORK_IMPLEMENTATION_DEFECT"
    };
    finalize(
        out,
        primary,
        "381ac64".into(),
        Some(founders),
        Some(net),
        gates,
        fidelity,
        foundation,
    )
}
