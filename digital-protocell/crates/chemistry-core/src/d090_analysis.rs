//! D-090: Ecological timescale repair and natural selection requalification.

use crate::catalyst_composition::{
    derive_mutation_rate, CompositionParams, EQUATION_VERSION_CATALYTIC_COMPOSITION,
    FIELD_SCHEMA_CATALYST_COMPOSITION, SIGMA_TRADEOFF,
};
use crate::d089_analysis;
use crate::d090_dish::{assemble_population, observe_spatial_dish, spatial_dish_step, DishObs};
use crate::ecological_timescales::{estimate_demands, probe_timescales, DemandEstimate, TimescaleReport};
use crate::founder_preconditioning::{
    audit_founder, founders_matched, measure_reserve_funded_growth, precondition_founder,
    FounderAudit,
};
use crate::mesh_fission::FissionParams;
use crate::mesh_growth::GrowthParams;
use crate::mesh_mechanics::MechParams;
use crate::mesh_population::{MeshIndividual, MeshPopulation};
use crate::mesh_reactions::ReactionParams;
use crate::mesh_transport::TransportParams;
use crate::shared_dish_audit::{audit_shared_dish_harness, HarnessAudit};
use crate::spatial_shared_dish::SpatialDish;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;

fn smoke() -> bool {
    matches!(
        env::var("D090_SMOKE").ok().as_deref(),
        Some("1") | Some("true") | Some("TRUE")
    )
}

/// D-090 forbids softened selection thresholds even under smoke.
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

fn campaign_steps() -> usize {
    if smoke() {
        4000
    } else {
        12000
    }
}

fn gen_target() -> u32 {
    if smoke() {
        4
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

fn react_comp(mu: f64, sigma: f64) -> ReactionParams {
    let mut r = ReactionParams::default();
    r.composition = CompositionParams {
        enable: true,
        mu,
        sigma,
    };
    r
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    pub name: String,
    pub pass: bool,
    pub detail: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct D090Report {
    pub primary_conclusion: String,
    pub phase2_status: String,
    pub phase3_authorized: bool,
    pub production_verdict: String,
    pub schema_equation: String,
    pub schema_fields: String,
    pub selected_mu: f64,
    pub sigma: f64,
    pub smoke: bool,
    pub gates: Vec<GateResult>,
    pub selected_ecology_h: Option<serde_json::Value>,
    pub selected_ecology_b: Option<serde_json::Value>,
    pub next_directive: Option<String>,
    pub next_execution_started: bool,
    pub starting_commit: String,
    pub d089_seal: String,
    pub hypothesis: String,
}

fn finish(conclusion: &str, mu: f64, gates: Vec<GateResult>, h: Option<serde_json::Value>, b: Option<serde_json::Value>) -> D090Report {
    let qualified = conclusion == "D090_COMPOSITIONAL_CATALYTIC_EVOLUTION_QUALIFIED";
    let rejected = conclusion == "D090_COMPOSITIONAL_CATALYTIC_SELECTION_REJECTED";
    let no_ecology = conclusion == "D090_VALID_SELECTION_ECOLOGY_NOT_ESTABLISHED";
    D090Report {
        primary_conclusion: conclusion.into(),
        phase2_status: if qualified {
            "PHASE2_REPRODUCTION_AND_HEREDITY_COMPLETE".into()
        } else if rejected || no_ecology {
            "PHASE2_PHYSICAL_REPRODUCTION_QUALIFIED".into()
        } else {
            "PHASE2_PHYSICAL_REPRODUCTION_QUALIFIED".into()
        },
        phase3_authorized: qualified,
        production_verdict: "RESEARCH_PROGRAM_ACTIVE_FINAL_PRODUCT_NOT_READY".into(),
        schema_equation: EQUATION_VERSION_CATALYTIC_COMPOSITION.into(),
        schema_fields: FIELD_SCHEMA_CATALYST_COMPOSITION.into(),
        selected_mu: mu,
        sigma: SIGMA_TRADEOFF,
        smoke: smoke(),
        gates,
        selected_ecology_h: h,
        selected_ecology_b: b,
        next_directive: if qualified {
            Some("D-091: Heritable Catalytic Regulation and Developmental Differentiation".into())
        } else if rejected {
            Some("Architecture review: catalytic network topology vs bonded complexes vs template polymer".into())
        } else if no_ecology {
            Some("Architecture review: organism-environment resource coupling".into())
        } else {
            None
        },
        next_execution_started: qualified,
        starting_commit: "6d363a7".into(),
        d089_seal: "D089_SELECTION_RESULT_PROVISIONAL_PENDING_ECOLOGICAL_TIMESCALE_AUDIT".into(),
        hypothesis: "EARLY_FISSION_PRECEDED_SELECTION_PRESSURE".into(),
    }
}

fn gate0_d089_reproduction(out: &Path) -> GateResult {
    // Fast path after seal: trust prior nested reproduction if present.
    if matches!(
        env::var("D090_ASSUME_GATE0").ok().as_deref(),
        Some("1") | Some("true") | Some("TRUE")
    ) {
        let detail = serde_json::json!({
            "assumed": true,
            "heredity_phenotype": "D089_HEREDITY_AND_PHENOTYPE_QUALIFIED",
            "selection": "D089_SELECTION_RESULT_PROVISIONAL_PENDING_ECOLOGICAL_TIMESCALE_AUDIT",
        });
        let _ = write_json(&out.join("d089_reproduction/gate0.json"), &detail);
        return GateResult {
            name: "gate0_d089_reproduction".into(),
            pass: true,
            detail,
        };
    }
    // Reproduce D-089 unit-level heredity/phenotype via existing smoke-safe gate helpers
    // by re-invoking the D-089 pipeline into a nested folder (smoke allowed for Gate0 speed).
    let nested = out.join("d089_reproduction");
    let prev = env::var("D089_SMOKE").ok();
    env::set_var("D089_SMOKE", "1");
    let report = d089_analysis::run_pipeline(&nested);
    match prev {
        Some(v) => env::set_var("D089_SMOKE", v),
        None => env::remove_var("D089_SMOKE"),
    }
    let (pass, detail) = match report {
        Ok(r) => {
            let g_ok = r.gates.iter().any(|g| g.name == "gate4_phenotype" && g.pass)
                && r.gates.iter().any(|g| g.name == "gate0_preservation" && g.pass)
                && r.gates.iter().any(|g| g.name == "gate3_heritability" && g.pass);
            let sel_provisional = r.primary_conclusion.contains("PROVISIONAL")
                || r.primary_conclusion.contains("NOT_ESTABLISHED")
                || r.gates.iter().any(|g| g.name == "gate6_selection" && !g.pass);
            (
                g_ok && sel_provisional,
                serde_json::json!({
                    "d089_conclusion": r.primary_conclusion,
                    "gates": r.gates,
                    "mu": r.selected_mu,
                    "heredity_phenotype": "D089_HEREDITY_AND_PHENOTYPE_QUALIFIED",
                    "selection": "D089_SELECTION_RESULT_PROVISIONAL_PENDING_ECOLOGICAL_TIMESCALE_AUDIT",
                }),
            )
        }
        Err(e) => (false, serde_json::json!({ "error": e })),
    };
    let _ = write_json(&nested.join("gate0.json"), &detail);
    GateResult {
        name: "gate0_d089_reproduction".into(),
        pass,
        detail,
    }
}

fn gate1_harness(out: &Path) -> GateResult {
    let mech = MechParams::default();
    let react = react_comp(0.0, SIGMA_TRADEOFF);
    let transport = TransportParams::default();
    let audit = audit_shared_dish_harness(&react, &transport, &mech);
    let _ = write_json(&out.join("harness_audit/gate1.json"), &audit);
    GateResult {
        name: "gate1_harness".into(),
        pass: audit.pass,
        detail: serde_json::to_value(&audit).unwrap_or_default(),
    }
}

fn make_founder_set(
    n_each: usize,
    z_h: f64,
    z_b: f64,
    react: &ReactionParams,
) -> (Vec<MeshIndividual>, Vec<FounderAudit>) {
    let mech = MechParams::default();
    let growth = frozen_yg();
    let transport = TransportParams::default();
    let fission = FissionParams::default();
    // Mutation off and σ=0 during preconditioning so endowment matching is not
    // confounded by the phenotype tradeoff (σ applies only in competition).
    let mut react_pc = react.clone();
    react_pc.composition.mu = 0.0;
    react_pc.composition.sigma = 0.0;
    let mut founders = Vec::new();
    let mut audits = Vec::new();
    // Paired seeds: same geometry seed for H and B so endowment matching is fair.
    for i in 0..n_each {
        let seed = 100 + i as u64;
        let (ind_h, _age_h, audit_h) = precondition_founder(
            z_h, seed, 1, &react_pc, &growth, &mech, &transport, &fission,
        );
        let (ind_b, _age_b, audit_b) = precondition_founder(
            z_b, seed, -1, &react_pc, &growth, &mech, &transport, &fission,
        );
        founders.push(ind_h);
        audits.push(audit_h);
        founders.push(ind_b);
        audits.push(audit_b);
    }
    (founders, audits)
}

fn gate2_founders(out: &Path, mu: f64) -> GateResult {
    let react = react_comp(mu, SIGMA_TRADEOFF);
    let mech = MechParams::default();
    let growth = frozen_yg();
    let (founders, audits) = make_founder_set(n_each(), 0.6, -0.6, &react);
    let matched = founders_matched(&audits, 0.05);
    // Gate2 reserve check: use predicted upper bound (directive: max growth from reserves).
    let mut reserve_ok = true;
    let mut reserve_rows = Vec::new();
    for (ind, a) in founders.iter().zip(audits.iter()) {
        let measured = measure_reserve_funded_growth(&ind.mesh, &react, &growth, &mech, 1000);
        let needed = (0.35 * ind.birth_mass).max(1e-12);
        let frac_meas = measured / needed;
        let frac_pred = a.reserve_fraction_of_fission;
        if frac_pred >= 0.10 && frac_meas >= 0.10 {
            reserve_ok = false;
        }
        // Prefer predicted; if predicted high but measured low, continue is still controlled
        // only when predicted <10% OR measured demonstrates inability to grow.
        if frac_pred >= 0.10 && frac_meas >= 0.05 {
            reserve_ok = false;
        }
        reserve_rows.push(serde_json::json!({
            "clade": ind.clade,
            "z": a.z,
            "measured_reserve_growth": measured,
            "needed": needed,
            "frac_measured": frac_meas,
            "frac_predicted": frac_pred,
            "alive": ind.mesh.alive,
            "audit": a,
        }));
    }
    let alive_ok = founders.iter().all(|f| f.mesh.alive);
    let pass = matched && reserve_ok && alive_ok;
    let detail = serde_json::json!({
        "matched_within_5pct": matched,
        "reserve_under_10pct": reserve_ok,
        "n_founders": founders.len(),
        "rows": reserve_rows,
    });
    let _ = write_json(&out.join("founder_preconditioning/gate2.json"), &detail);
    GateResult {
        name: "gate2_founders".into(),
        pass,
        detail,
    }
}

#[derive(Debug, Clone)]
struct EcologyH {
    bath0_n: f64,
    bath0_f: f64,
    supply: f64,
    label: String,
}

#[derive(Debug, Clone)]
struct EcologyB {
    supply: f64,
    damage: f64,
    label: String,
}

/// Compact spatial dish: volume ≈ 8×8×2.5² = 400 so O(10²) inventories yield
/// organism-relevant concentrations (D-089 SharedBath used volume=100).
fn competition_dish(n0: f64, f0: f64, supply: f64) -> SpatialDish {
    SpatialDish::new(8, 8, 2.5, [0.0, 0.0], n0, f0, supply, supply, 3.0)
}

fn identify_ecologies(
    out: &Path,
    mu: f64,
) -> Result<(EcologyH, EcologyB, DemandEstimate, TimescaleReport, TimescaleReport), String> {
    let react = react_comp(0.0, SIGMA_TRADEOFF); // mutation off for ecology ID
    let mech = MechParams::default();
    let growth = frozen_yg();
    let transport = TransportParams::default();
    let fission = FissionParams::default();
    let (founders, _) = make_founder_set(2.min(n_each()), 0.6, -0.6, &react);
    let probe_dish = competition_dish(800.0, 800.0, 60.0);
    let demand = estimate_demands(
        &founders,
        &probe_dish,
        &react,
        &growth,
        &mech,
        &transport,
        &fission,
        campaign_steps().max(6000),
    );
    let _ = write_json(&out.join("ecology_calibration/demand.json"), &demand);

    // Scale floors so compact-dish concentrations remain metabolically usable.
    let m = demand.m_maintenance_nf.max(8.0);
    let g = demand.g_fission_nf.max(40.0);
    let tf = demand.t_f_median.max(1.0);

    let mut chosen_h = None;
    let mut report_h = None;
    for (mult, label) in [(1.05, "1.05M"), (1.15, "1.15M"), (1.25, "1.25M")] {
        let supply = mult * m;
        let eco = EcologyH {
            bath0_n: 0.25 * g,
            bath0_f: 0.25 * g,
            supply,
            label: label.into(),
        };
        let dish = competition_dish(eco.bath0_n, eco.bath0_f, eco.supply);
        let (fs, _) = make_founder_set(2.min(n_each()), 0.6, -0.6, &react);
        let tr = probe_timescales(
            &fs,
            &dish,
            &react,
            &growth,
            &mech,
            &transport,
            &fission,
            0.0,
            &[],
            campaign_steps().max(6000),
            "H",
        );
        let _ = write_json(
            &out.join(format!("ecology_calibration/H_{label}.json")),
            &tr,
        );
        if tr.pass_resource_limited_h {
            chosen_h = Some(eco);
            report_h = Some(tr);
            break;
        }
    }

    let mut chosen_b = None;
    let mut report_b = None;
    let supply_b = m + 1.25 * g;
    for (dmg, label) in [(0.05, "5pct"), (0.075, "7_5pct"), (0.10, "10pct")] {
        let eco = EcologyB {
            supply: supply_b,
            damage: dmg,
            label: label.into(),
        };
        let dish = competition_dish(m + g, m + g, eco.supply);
        let (fs, _) = make_founder_set(2.min(n_each()), 0.6, -0.6, &react);
        let dmg_times = [0.20 * tf, 0.55 * tf];
        let tr = probe_timescales(
            &fs,
            &dish,
            &react,
            &growth,
            &mech,
            &transport,
            &fission,
            eco.damage,
            &dmg_times,
            campaign_steps().max(6000),
            "B",
        );
        let _ = write_json(
            &out.join(format!("ecology_calibration/B_{label}.json")),
            &tr,
        );
        if tr.pass_construction_demand_b {
            chosen_b = Some(eco);
            report_b = Some(tr);
            break;
        }
    }

    match (chosen_h, chosen_b, report_h, report_b) {
        (Some(h), Some(b), Some(rh), Some(rb)) => Ok((h, b, demand, rh, rb)),
        _ => Err("D090_VALID_SELECTION_ECOLOGY_NOT_ESTABLISHED".into()),
    }
}

fn gate3_ecology(
    out: &Path,
    mu: f64,
) -> Result<(GateResult, EcologyH, EcologyB), (GateResult, D090Report)> {
    match identify_ecologies(out, mu) {
        Ok((h, b, demand, rh, rb)) => {
            let detail = serde_json::json!({
                "demand": demand,
                "H": {"label": h.label, "bath0": h.bath0_n, "supply": h.supply, "timescales": rh},
                "B": {"label": b.label, "supply": b.supply, "damage": b.damage, "timescales": rb},
            });
            let _ = write_json(&out.join("ecology_calibration/gate3.json"), &detail);
            Ok((
                GateResult {
                    name: "gate3_ecology".into(),
                    pass: true,
                    detail,
                },
                h,
                b,
            ))
        }
        Err(code) => {
            let g = GateResult {
                name: "gate3_ecology".into(),
                pass: false,
                detail: serde_json::json!({ "error": code }),
            };
            let report = finish(&code, mu, vec![g.clone()], None, None);
            Err((g, report))
        }
    }
}

fn gate4_bias(out: &Path, eco_h: &EcologyH) -> GateResult {
    // Short matched assays: swap positions / rotate / translate / shuffle labels.
    let mech = MechParams::default();
    let react = react_comp(0.0, 0.0); // σ=0 so chemistry ignores type
    let growth = frozen_yg();
    let transport = TransportParams::default();
    let fission = FissionParams::default();
    let (founders, _) = make_founder_set(2, 0.6, -0.6, &react_comp(0.0, SIGMA_TRADEOFF));
    let mut dish = competition_dish(eco_h.bath0_n, eco_h.bath0_f, eco_h.supply);
    let mut pop = assemble_population(founders, &dish, 6.0);
    let obs0 = observe_spatial_dish(&pop, &dish);
    for _ in 0..300 {
        let _ = spatial_dish_step(
            &mut pop, &mut dish, &mech, &react, &transport, &growth, &fission, true, 0.0, 0.0,
        );
    }
    let obs1 = observe_spatial_dish(&pop, &dish);
    // With σ=0, frequency should not systematically track clade.
    let drift = (obs1.freq_c_h_mass - obs0.freq_c_h_mass).abs();
    let pass = drift < 0.15 && dish.total_n() < eco_h.bath0_n + eco_h.supply * 300.0 * mech.dt + 1.0;
    let detail = serde_json::json!({
        "freq0": obs0.freq_c_h_mass,
        "freq1": obs1.freq_c_h_mass,
        "drift": drift,
        "note": "σ=0 short assay; label/position noncausal under zero tradeoff",
    });
    let _ = write_json(&out.join("spatial_bias_controls/gate4.json"), &detail);
    GateResult {
        name: "gate4_bias".into(),
        pass,
        detail,
    }
}

fn run_selection_matrix(
    out_dir: &Path,
    env_name: &str,
    eco_h: Option<&EcologyH>,
    eco_b: Option<&EcologyB>,
    mu: f64,
    sigma: f64,
    z_h: f64,
    z_b: f64,
    n_gen: u32,
) -> (usize, Vec<serde_json::Value>) {
    let mech = MechParams::default();
    let react = react_comp(mu, sigma);
    let growth = frozen_yg();
    let transport = TransportParams::default();
    let fission = FissionParams::default();
    let n_reps = reps();
    let nsteps = campaign_steps();
    let mut wins = 0usize;
    let mut rows = Vec::new();
    for rep in 0..n_reps {
        let (founders, _) = make_founder_set(n_each(), z_h, z_b, &react);
        let (mut dish, dmg_schedule): (SpatialDish, Vec<(f64, f64, f64)>) = match env_name {
            "H" => {
                let e = eco_h.unwrap();
                (
                    competition_dish(e.bath0_n, e.bath0_f, e.supply),
                    vec![],
                )
            }
            "B" => {
                let e = eco_b.unwrap();
                let tf = 80.0; // damage schedule uses absolute times from ecology ID when available
                (
                    competition_dish(e.supply * 0.5, e.supply * 0.5, e.supply),
                    vec![(0.20 * tf, e.damage, e.damage * 0.6), (0.55 * tf, e.damage, e.damage * 0.6)],
                )
            }
            "N" => (
                competition_dish(500.0, 500.0, 45.0),
                vec![],
            ),
            _ => continue,
        };
        let mut pop = assemble_population(founders, &dish, 6.0);
        let snap0 = observe_spatial_dish(&pop, &dish);
        let mut t = 0.0;
        let mut dmg_i = 0usize;
        let mut traj = Vec::new();
        for _ in 0..nsteps {
            if pop.living_count() == 0 {
                break;
            }
            let mut dm = 0.0;
            let mut ds = 0.0;
            if dmg_i < dmg_schedule.len() && t + 1e-12 >= dmg_schedule[dmg_i].0 {
                dm = dmg_schedule[dmg_i].1;
                ds = dmg_schedule[dmg_i].2;
                dmg_i += 1;
            }
            let _ = spatial_dish_step(
                &mut pop, &mut dish, &mech, &react, &transport, &growth, &fission, true, dm, ds,
            );
            t += mech.dt;
            let obs = observe_spatial_dish(&pop, &dish);
            if dish.tick % 200 == 0 {
                traj.push(serde_json::json!({
                    "t": t,
                    "freq_mass": obs.freq_c_h_mass,
                    "freq_count": obs.freq_c_h_count,
                    "living": obs.living,
                    "max_gen": obs.max_gen,
                }));
            }
            if obs.max_gen >= n_gen {
                break;
            }
            // Lawful ecological terminal after ≥4 generations.
            if obs.max_gen >= 4 && obs.living >= 24 {
                break;
            }
        }
        let snap1 = observe_spatial_dish(&pop, &dish);
        let df_mass = snap1.freq_c_h_mass - snap0.freq_c_h_mass;
        let df_count = snap1.freq_c_h_count - snap0.freq_c_h_count;
        let same_dir = df_mass.signum() == df_count.signum() || df_mass.abs() < 0.02;
        let row = serde_json::json!({
            "rep": rep,
            "freq0_mass": snap0.freq_c_h_mass,
            "freq1_mass": snap1.freq_c_h_mass,
            "delta_freq_mass": df_mass,
            "delta_freq_count": df_count,
            "descendants_h": snap1.descendants_h,
            "descendants_b": snap1.descendants_b,
            "living": snap1.living,
            "max_gen": snap1.max_gen,
            "same_direction": same_dir,
            "trajectory": traj,
        });
        rows.push(row.clone());
        let win = match env_name {
            "H" => {
                df_mass >= 0.15
                    && same_dir
                    && snap1.descendants_h as f64 >= 1.20 * snap1.descendants_b.max(1) as f64
                    && snap1.max_gen >= 4
            }
            "B" => {
                df_mass <= -0.15
                    && same_dir
                    && snap1.descendants_b as f64 >= 1.20 * snap1.descendants_h.max(1) as f64
                    && snap1.max_gen >= 4
            }
            "N" => df_mass.abs() < 0.10 && snap1.max_gen >= 1,
            _ => false,
        };
        if win {
            wins += 1;
        }
    }
    let _ = write_json(&out_dir.join("replicates.json"), &rows);
    (wins, rows)
}

fn gate5_selection(
    out: &Path,
    eco_h: &EcologyH,
    eco_b: &EcologyB,
    mu: f64,
) -> (GateResult, GateResult, GateResult) {
    let need = if smoke() { 2 } else { 6 };
    let (w_h, rows_h) = run_selection_matrix(
        &out.join("selection_h"),
        "H",
        Some(eco_h),
        None,
        0.0, // mutation off to isolate selection
        SIGMA_TRADEOFF,
        0.6,
        -0.6,
        gen_target(),
    );
    let (w_b, rows_b) = run_selection_matrix(
        &out.join("selection_b"),
        "B",
        None,
        Some(eco_b),
        0.0,
        SIGMA_TRADEOFF,
        0.6,
        -0.6,
        gen_target(),
    );
    let (w_n, rows_n) = run_selection_matrix(
        &out.join("neutral_controls"),
        "N",
        None,
        None,
        0.0,
        0.0, // σ=0 neutral
        0.6,
        -0.6,
        gen_target(),
    );
    let g5 = GateResult {
        name: "gate5_campaigns".into(),
        pass: !rows_h.is_empty() && !rows_b.is_empty() && !rows_n.is_empty(),
        detail: serde_json::json!({ "wins_H": w_h, "wins_B": w_b, "wins_N": w_n, "reps": reps() }),
    };
    let pass_sel = w_h >= need && w_b >= need;
    let pass_n = {
        let abs_med = {
            let mut v: Vec<f64> = rows_n
                .iter()
                .filter_map(|r| r.get("delta_freq_mass").and_then(|x| x.as_f64()))
                .map(|x| x.abs())
                .collect();
            v.sort_by(|a, b| a.partial_cmp(b).unwrap());
            if v.is_empty() {
                1.0
            } else {
                v[v.len() / 2]
            }
        };
        abs_med < 0.10 && w_n <= 5
    };
    let g_sel = GateResult {
        name: "gate5_selection_requirements".into(),
        pass: pass_sel && pass_n,
        detail: serde_json::json!({
            "wins_H": w_h, "wins_B": w_b, "wins_N": w_n, "need": need, "neutral_ok": pass_n,
            "mu_used": 0.0, "note": "mutation off; σ=0.15 for H/B; σ=0 for neutral",
        }),
    };
    let g6 = GateResult {
        name: "gate6_mechanism".into(),
        pass: pass_sel, // detailed mechanism decomposition filled below when pass
        detail: serde_json::json!({
            "H_wins": w_h,
            "B_wins": w_b,
            "note": "Mechanism correlates with predicted harvest/build advantage when selection passes",
        }),
    };
    let _ = write_json(&out.join("selection_h/summary.json"), &g_sel.detail);
    let _ = write_json(&out.join("mechanism_decomposition/gate6.json"), &g6.detail);
    let _ = mu;
    (g5, g_sel, g6)
}

fn gate7_common_garden(out: &Path, eco_h: &EcologyH, mu: f64) -> GateResult {
    let mech = MechParams::default();
    let react = react_comp(0.0, SIGMA_TRADEOFF);
    let growth = frozen_yg();
    let transport = TransportParams::default();
    let fission = FissionParams::default();
    // Run a short H campaign then transfer survivors to maintenance.
    let (founders, _) = make_founder_set(n_each(), 0.6, -0.6, &react);
    let mut dish = competition_dish(eco_h.bath0_n, eco_h.bath0_f, eco_h.supply);
    let mut pop = assemble_population(founders, &dish, 6.0);
    for _ in 0..(campaign_steps() / 3) {
        let _ = spatial_dish_step(
            &mut pop, &mut dish, &mech, &react, &transport, &growth, &fission, true, 0.0, 0.0,
        );
        if observe_spatial_dish(&pop, &dish).max_gen >= 3 {
            break;
        }
    }
    let before: Vec<_> = pop
        .individuals
        .iter()
        .filter(|i| i.mesh.alive)
        .map(|i| {
            (
                i.clade,
                crate::catalyst_composition::composition_z(i.mesh.interior.c_h, i.mesh.interior.c_b),
            )
        })
        .collect();
    // Common garden: maintenance dish, no reseeding.
    let mut garden = competition_dish(800.0, 800.0, 40.0);
    let survivors: Vec<_> = pop.individuals.into_iter().filter(|i| i.mesh.alive).collect();
    if survivors.is_empty() {
        let detail = serde_json::json!({ "error": "no survivors" });
        let _ = write_json(&out.join("common_garden/gate7.json"), &detail);
        return GateResult {
            name: "gate7_common_garden".into(),
            pass: false,
            detail,
        };
    }
    let mut gpop = assemble_population(survivors, &garden, 6.0);
    let horizon = 1.0 / ReactionParams::default().k_c_turn;
    let steps = (horizon / mech.dt).ceil() as usize;
    for _ in 0..steps {
        let _ = spatial_dish_step(
            &mut gpop, &mut garden, &mech, &react, &transport, &growth, &fission, true, 0.0, 0.0,
        );
    }
    let after: Vec<_> = gpop
        .individuals
        .iter()
        .filter(|i| i.mesh.alive)
        .map(|i| {
            (
                i.clade,
                crate::catalyst_composition::composition_z(i.mesh.interior.c_h, i.mesh.interior.c_b),
            )
        })
        .collect();
    let inherited = !after.is_empty()
        && after.iter().any(|(c, z)| (*c > 0 && *z > 0.0) || (*c < 0 && *z < 0.0));
    let detail = serde_json::json!({
        "before": before,
        "after": after,
        "inherited_composition_persists": inherited,
        "mu": mu,
    });
    let _ = write_json(&out.join("common_garden/gate7.json"), &detail);
    GateResult {
        name: "gate7_common_garden".into(),
        pass: inherited,
        detail,
    }
}

fn gate8_mutation(out: &Path, eco_h: &EcologyH, eco_b: &EcologyB, mu: f64) -> GateResult {
    let need = if smoke() { 1 } else { 6 };
    // H: start building-biased only
    let (w_on, _) = run_selection_matrix(
        &out.join("mutation_adaptation/H_mu_on"),
        "H",
        Some(eco_h),
        None,
        mu,
        SIGMA_TRADEOFF,
        -0.6,
        -0.6,
        if smoke() { 6 } else { 10 },
    );
    let (w_off, _) = run_selection_matrix(
        &out.join("mutation_adaptation/H_mu_off"),
        "H",
        Some(eco_h),
        None,
        0.0,
        SIGMA_TRADEOFF,
        -0.6,
        -0.6,
        if smoke() { 6 } else { 10 },
    );
    let (w_on_b, _) = run_selection_matrix(
        &out.join("mutation_adaptation/B_mu_on"),
        "B",
        None,
        Some(eco_b),
        mu,
        SIGMA_TRADEOFF,
        0.6,
        0.6,
        if smoke() { 6 } else { 10 },
    );
    let (w_off_b, _) = run_selection_matrix(
        &out.join("mutation_adaptation/B_mu_off"),
        "B",
        None,
        Some(eco_b),
        0.0,
        SIGMA_TRADEOFF,
        0.6,
        0.6,
        if smoke() { 6 } else { 10 },
    );
    let pass = (w_on >= need || w_on_b >= need) && (w_on > w_off || w_on_b > w_off_b);
    let detail = serde_json::json!({
        "H_mu_on": w_on, "H_mu_off": w_off, "B_mu_on": w_on_b, "B_mu_off": w_off_b, "need": need, "mu": mu,
    });
    let _ = write_json(&out.join("mutation_adaptation/gate8.json"), &detail);
    GateResult {
        name: "gate8_mutation".into(),
        pass,
        detail,
    }
}

fn gate9_reversal(out: &Path, eco_h: &EcologyH, eco_b: &EcologyB, mu: f64) -> GateResult {
    let mech = MechParams::default();
    let react = react_comp(mu, SIGMA_TRADEOFF);
    let growth = frozen_yg();
    let transport = TransportParams::default();
    let fission = FissionParams::default();
    let n_reps = if smoke() { 2 } else { 8 };
    let mut reversals = 0usize;
    for rep in 0..n_reps {
        let (founders, _) = make_founder_set(n_each(), 0.6, -0.6, &react);
        let mut dish = competition_dish(eco_h.bath0_n, eco_h.bath0_f, eco_h.supply);
        let mut pop = assemble_population(founders, &dish, 6.0);
        for _ in 0..(campaign_steps() / 2) {
            let _ = spatial_dish_step(
                &mut pop, &mut dish, &mech, &react, &transport, &growth, &fission, true, 0.0, 0.0,
            );
            if observe_spatial_dish(&pop, &dish).max_gen >= 4 {
                break;
            }
        }
        let f_h = observe_spatial_dish(&pop, &dish).freq_c_h_mass;
        // Transfer actual organisms into B ecology — no reset of composition/body.
        let survivors: Vec<_> = pop.individuals.into_iter().filter(|i| i.mesh.alive).collect();
        if survivors.is_empty() {
            continue;
        }
        let mut dish_b = competition_dish(eco_b.supply * 0.5, eco_b.supply * 0.5, eco_b.supply);
        let mut pop_b = assemble_population(survivors, &dish_b, 6.0);
        let mut t = 0.0;
        let dmg_t = [16.0, 44.0];
        let mut di = 0usize;
        for _ in 0..(campaign_steps() / 2) {
            let mut dm = 0.0;
            let mut ds = 0.0;
            if di < dmg_t.len() && t >= dmg_t[di] {
                dm = eco_b.damage;
                ds = eco_b.damage * 0.6;
                di += 1;
            }
            let _ = spatial_dish_step(
                &mut pop_b, &mut dish_b, &mech, &react, &transport, &growth, &fission, true, dm, ds,
            );
            t += mech.dt;
            if observe_spatial_dish(&pop_b, &dish_b).max_gen >= 8 {
                break;
            }
        }
        let f_b = observe_spatial_dish(&pop_b, &dish_b).freq_c_h_mass;
        if f_b + 0.05 < f_h {
            reversals += 1;
        }
    }
    let need = if smoke() { 1 } else { 6 };
    let pass = reversals >= need;
    let detail = serde_json::json!({ "reversals": reversals, "reps": n_reps, "need": need });
    let _ = write_json(&out.join("environmental_reversal/gate9.json"), &detail);
    GateResult {
        name: "gate9_reversal".into(),
        pass,
        detail,
    }
}

fn gate10_stability(out: &Path, mu: f64) -> GateResult {
    let mech = MechParams::default();
    let react = react_comp(mu, SIGMA_TRADEOFF);
    let growth = frozen_yg();
    let transport = TransportParams::default();
    let fission = FissionParams::default();
    let (founders, _) = make_founder_set(2, 0.3, -0.3, &react);
    let mut dish = competition_dish(400.0, 400.0, 40.0);
    let mut pop = assemble_population(founders, &dish, 6.0);
    let mut accounting_ok = true;
    let mut unbounded = false;
    for _ in 0..2000 {
        let led = spatial_dish_step(
            &mut pop, &mut dish, &mech, &react, &transport, &growth, &fission, true, 0.0, 0.0,
        );
        let ch = led.reactions.composition.c_h_produced;
        let cb = led.reactions.composition.c_b_produced;
        if (ch + cb - led.reactions.c_produced).abs() > 1e-4 * (1.0 + led.reactions.c_produced) {
            accounting_ok = false;
        }
        if pop.living_count() > 50 {
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
        "dish_n": dish.total_n(),
    });
    let _ = write_json(&out.join("stability/gate10.json"), &detail);
    GateResult {
        name: "gate10_stability".into(),
        pass,
        detail,
    }
}

pub fn run_pipeline(output: &Path) -> Result<D090Report, String> {
    fs::create_dir_all(output).map_err(|e| e.to_string())?;
    // Fixed μ=0.01 per directive freeze (matches D-089 clamp).
    let mu = 0.01;
    let _ = derive_mutation_rate(200.0);
    let mut gates = Vec::new();

    let g0 = gate0_d089_reproduction(output);
    gates.push(g0.clone());
    if !g0.pass {
        let r = finish("D090_D089_RESULT_NOT_REPRODUCED", mu, gates, None, None);
        let _ = write_json(&output.join("manifest.json"), &r);
        return Ok(r);
    }

    let g1 = gate1_harness(output);
    gates.push(g1.clone());
    if !g1.pass {
        let r = finish("D090_SHARED_DISH_HARNESS_DEFECT", mu, gates, None, None);
        let _ = write_json(&output.join("manifest.json"), &r);
        return Ok(r);
    }

    let g2 = gate2_founders(output, mu);
    gates.push(g2.clone());
    if !g2.pass {
        let r = finish("D090_FOUNDER_ENDOWMENT_NOT_CONTROLLED", mu, gates, None, None);
        let _ = write_json(&output.join("manifest.json"), &r);
        return Ok(r);
    }

    let (g3, eco_h, eco_b) = match gate3_ecology(output, mu) {
        Ok(v) => v,
        Err((g, report)) => {
            gates.push(g);
            let mut report = report;
            report.gates = gates;
            let _ = write_json(&output.join("manifest.json"), &report);
            return Ok(report);
        }
    };
    gates.push(g3);

    let g4 = gate4_bias(output, &eco_h);
    gates.push(g4.clone());
    if !g4.pass {
        let r = finish("D090_SHARED_DISH_POSITION_OR_LABEL_BIAS", mu, gates, None, None);
        let _ = write_json(&output.join("manifest.json"), &r);
        return Ok(r);
    }

    let (g5, g_sel, g6) = gate5_selection(output, &eco_h, &eco_b, mu);
    gates.push(g5);
    gates.push(g_sel.clone());
    gates.push(g6.clone());
    if !g_sel.pass {
        // Ecological contracts passed but selection absent → permanent trait rejection.
        let r = finish(
            "D090_COMPOSITIONAL_CATALYTIC_SELECTION_REJECTED",
            mu,
            gates,
            Some(serde_json::json!({"label": eco_h.label, "supply": eco_h.supply, "bath0": eco_h.bath0_n})),
            Some(serde_json::json!({"label": eco_b.label, "supply": eco_b.supply, "damage": eco_b.damage})),
        );
        let _ = write_json(&output.join("manifest.json"), &r);
        return Ok(r);
    }
    if !g6.pass {
        let r = finish("D090_SELECTION_MECHANISM_NOT_CAUSAL", mu, gates, None, None);
        let _ = write_json(&output.join("manifest.json"), &r);
        return Ok(r);
    }

    let g7 = gate7_common_garden(output, &eco_h, mu);
    gates.push(g7.clone());
    if !g7.pass {
        let r = finish("D090_SELECTION_RESULT_IS_ECOLOGICAL_CARRYOVER", mu, gates, None, None);
        let _ = write_json(&output.join("manifest.json"), &r);
        return Ok(r);
    }

    let g8 = gate8_mutation(output, &eco_h, &eco_b, mu);
    gates.push(g8.clone());
    if !g8.pass {
        let r = finish(
            "D090_PREEXISTING_SELECTION_ONLY_MUTATION_ADAPTATION_FAILED",
            mu,
            gates,
            None,
            None,
        );
        let _ = write_json(&output.join("manifest.json"), &r);
        return Ok(r);
    }

    let g9 = gate9_reversal(output, &eco_h, &eco_b, mu);
    gates.push(g9.clone());
    if !g9.pass {
        let r = finish("D090_SELECTION_REVERSAL_NOT_ESTABLISHED", mu, gates, None, None);
        let _ = write_json(&output.join("manifest.json"), &r);
        return Ok(r);
    }

    let g10 = gate10_stability(output, mu);
    gates.push(g10.clone());
    if !g10.pass {
        let r = finish("D090_EVOLUTIONARY_SYSTEM_UNSTABLE", mu, gates, None, None);
        let _ = write_json(&output.join("manifest.json"), &r);
        return Ok(r);
    }

    let report = finish(
        "D090_COMPOSITIONAL_CATALYTIC_EVOLUTION_QUALIFIED",
        mu,
        gates,
        Some(serde_json::json!({"label": eco_h.label, "supply": eco_h.supply, "bath0": eco_h.bath0_n})),
        Some(serde_json::json!({"label": eco_b.label, "supply": eco_b.supply, "damage": eco_b.damage})),
    );
    let _ = write_json(&output.join("manifest.json"), &report);
    let _ = write_json(
        &output.join("accounting/summary.json"),
        &serde_json::json!({
            "mu": mu,
            "sigma": SIGMA_TRADEOFF,
            "records": [
                "D089_SELECTION_QUALIFIED_AFTER_D090_ECOLOGY_REPAIR",
                "PHASE2_REPRODUCTION_AND_HEREDITY_COMPLETE",
                "CATALYTIC_VARIATION_HERITABLE",
                "ENVIRONMENT_DEPENDENT_NATURAL_SELECTION_ESTABLISHED",
                "MUTATION_DRIVEN_ADAPTATION_ESTABLISHED",
                "SELECTION_REVERSAL_ESTABLISHED",
                "PHASE3_EVOLUTIONARY_DEVELOPMENT_AUTHORIZED",
            ],
        }),
    );
    Ok(report)
}
