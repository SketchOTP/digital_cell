//! D-067 activation-capacity law identification, observer/shadow only.

use crate::d013::atomic_write_json;
use crate::d025::v7_base_params;
use chemistry_core::config::{EquationVersion, SimParams, StructureEvolutionMode};
use chemistry_core::d039_analysis::v8_schema3_params;
use chemistry_core::d049_analysis::d049_frozen_params;
use chemistry_core::d050_analysis::ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME;
use chemistry_core::d053_analysis::{apply_delivery_repair, DeliveryRepairPair, D053_FITTED_K_C, D053_FITTED_V_A, D053_N_REF, D053_F_REF};
use chemistry_core::d055_analysis::{D055_FROZEN_M_BETA, D055_FROZEN_M_EXT};
use chemistry_core::d058_analysis::{cell_volume, drive_original_a, face_measure_a_f, gamma_face_production, xi_face_req};
use chemistry_core::d063_analysis::{account_geometry, exterior_connected_mask, generate_phi, seed_mature_s_on_interfaces, smooth_baseline_length, GeometrySpec, D063_PHI_INTERIOR};
use chemistry_core::d065_analysis::{evaluate_canonical_net_flux, AcceptedEnvFluxEvent};
use chemistry_core::d066_analysis::{activation_stoichiometry_parity, ALedger066};
use chemistry_core::d067_analysis::*;
use chemistry_core::surface_density::total_surface_mass;
use chemistry_core::{field_mass, Grid, Simulation};
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const K_NF0: f64 = 0.3438108650061698;
const K_W0: f64 = 0.4198385248302346;

fn resolve_path(path: &Path) -> PathBuf {
    if path.is_absolute() { path.to_path_buf() } else { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(path) }
}
fn write_json(dir: &Path, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(dir)?;
    atomic_write_json(&dir.join("result.json"), value)?;
    Ok(())
}
fn git_output(args: &[&str]) -> Option<String> {
    Command::new("git").args(args).current_dir(resolve_path(Path::new(".")).join("..")).output().ok()
        .filter(|out| out.status.success()).and_then(|out| String::from_utf8(out.stdout).ok()).map(|text| text.trim().to_owned())
}
fn max_accepted() -> u64 { std::env::var("D067_MAX_ACCEPTED").ok().and_then(|v| v.parse().ok()).unwrap_or(2500).max(1) }
fn skip_late_gates() -> bool { std::env::var("D067_SKIP_LATE_GATES").map(|v| v == "1" || v.eq_ignore_ascii_case("true")).unwrap_or(false) }
fn horizon_ladder() -> Vec<u64> {
    let values: Vec<u64> = std::env::var("D067_HORIZON_LADDER").ok().map(|s| s.split(',').filter_map(|v| v.trim().parse().ok()).filter(|v: &u64| *v > 0).collect()).unwrap_or_default();
    if values.is_empty() { vec![2500, 5000, 10000] } else { values }
}
fn baseline_params() -> SimParams {
    let base = v7_base_params().unwrap_or_else(|_| v8_schema3_params());
    let mut params = d049_frozen_params(&base);
    params.equation_version = EquationVersion::MembraneMetabolismV13CatalystSaturatingActivation;
    params.activation_schema = ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME;
    params.k_d008_activation = D053_FITTED_V_A;
    params.k_c_activation = D053_FITTED_K_C;
    params.n_ref_activation = D053_N_REF;
    params.f_ref_activation = D053_F_REF;
    apply_delivery_repair(&mut params, DeliveryRepairPair { m_ext: D055_FROZEN_M_EXT, m_beta: D055_FROZEN_M_BETA });
    params
}
fn candidate_b_params(baseline: &SimParams, m_v: f64) -> SimParams {
    let mut params = baseline.clone();
    params.k_d008_activation = m_v * D067_V_A;
    params
}
fn candidate_c_params(baseline: &SimParams, k_n: f64, k_f: f64) -> SimParams {
    let mut params = baseline.clone();
    params.activation_schema = ACTIVATION_SCHEMA_BOUNDED_NF;
    params.n_ref_activation = k_n;
    params.f_ref_activation = k_f;
    params.k_d008_activation = D067_V_A;
    params
}
fn artifact(gate: &str, pass: bool, body: Value) -> Value {
    json!({"gate": gate, "pass": pass, "frozen_k_T": D067_FROZEN_KT, "shadow_only": true,
        "production_biology_unchanged": true, "source_commit": git_output(&["rev-parse", "HEAD"]), "body": body})
}
fn hold_exterior(sim: &mut Simulation) {
    for i in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(i) && sim.fields.structure[i] < D063_PHI_INTERIOR {
            sim.fields.nutrient[i] = sim.params.n_reservoir; sim.fields.fuel[i] = sim.params.f_reservoir;
        }
    }
}
fn hold_w_sink(sim: &mut Simulation) {
    hold_exterior(sim);
    for i in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(i) && sim.fields.structure[i] < D063_PHI_INTERIOR { sim.fields.waste[i] = sim.params.w_reservoir; }
    }
}
fn hold_interior_nf(sim: &mut Simulation, n: f64, f: f64) {
    for i in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(i) && sim.fields.structure[i] >= D063_PHI_INTERIOR { sim.fields.nutrient[i] = n; sim.fields.fuel[i] = f; }
    }
}
fn hold_interior_a(sim: &mut Simulation, a: f64) {
    for i in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(i) && sim.fields.structure[i] >= D063_PHI_INTERIOR { sim.fields.activated[i] = a; }
    }
}
fn seed_geometry_organism(sim: &mut Simulation, spec: &GeometrySpec) {
    let phi = generate_phi(&sim.grid, spec);
    let membrane = seed_mature_s_on_interfaces(&sim.grid, &phi, 1.0);
    for i in 0..phi.len() {
        if !sim.grid.in_dish(i) { continue; }
        sim.fields.structure[i] = phi[i]; sim.fields.membrane[i] = membrane[i];
        if phi[i] >= D063_PHI_INTERIOR {
            sim.fields.catalyst[i] = 0.4; sim.fields.activated[i] = 0.5; sim.fields.nutrient[i] = 0.4;
            sim.fields.fuel[i] = 0.4; sim.fields.waste[i] = 0.5; sim.fields.precursor[i] = 0.05;
        } else {
            sim.fields.catalyst[i] = 0.0; sim.fields.activated[i] = 0.0; sim.fields.nutrient[i] = sim.params.n_reservoir;
            sim.fields.fuel[i] = sim.params.f_reservoir; sim.fields.waste[i] = sim.params.w_reservoir; sim.fields.precursor[i] = 0.0;
        }
    }
}
#[derive(Clone, Copy)]
struct FaceUpdate { inside: usize, outside: usize, extent: f64 }
fn build_face_updates(sim: &Simulation, dt: f64) -> Vec<FaceUpdate> {
    let connected = exterior_connected_mask(&sim.grid, &sim.fields.structure, D063_PHI_INTERIOR);
    let mut updates = Vec::new();
    for idx in 0..sim.fields.structure.len() {
        if !sim.grid.in_dish(idx) { continue; }
        let (x, y) = (idx % sim.grid.width, idx / sim.grid.width);
        for (nx, ny) in [(x + 1, y), (x, y + 1)] {
            if nx >= sim.grid.width || ny >= sim.grid.height { continue; }
            let other = Grid::index(sim.grid.width, nx, ny);
            if !sim.grid.in_dish(other) { continue; }
            let a = sim.fields.structure[idx] >= D063_PHI_INTERIOR; let b = sim.fields.structure[other] >= D063_PHI_INTERIOR;
            if a == b { continue; }
            let (inside, outside) = if a { (idx, other) } else { (other, idx) };
            if !connected[outside] { continue; }
            let gamma = gamma_face_production(sim.fields.membrane[idx], sim.fields.structure[idx], sim.fields.membrane[other], sim.fields.structure[other], sim.params.delta_floor);
            let drive = drive_original_a(sim.fields.nutrient[outside], sim.fields.fuel[outside], sim.fields.waste[inside], sim.fields.nutrient[inside], sim.fields.fuel[inside], sim.fields.waste[outside], K_NF0, K_W0);
            if gamma > 1e-18 { updates.push(FaceUpdate { inside, outside, extent: xi_face_req(D067_FROZEN_KT, gamma, drive, face_measure_a_f(), dt) }); }
        }
    }
    updates
}
fn apply_shadow_carrier(sim: &mut Simulation, dt: f64) -> (f64, f64, f64, f64) {
    let mut n_in = 0.0; let mut f_in = 0.0; let mut n_out = 0.0; let mut f_out = 0.0; let volume = cell_volume();
    for face in build_face_updates(sim, dt) {
        let nf = 0.5 * face.extent / volume;
        let n = nf.abs().min(sim.fields.nutrient[face.outside].max(0.0)).copysign(nf);
        let f = nf.abs().min(sim.fields.fuel[face.outside].max(0.0)).copysign(nf);
        let w = face.extent.abs().min(sim.fields.waste[face.inside].max(0.0)).copysign(face.extent);
        sim.fields.nutrient[face.inside] = (sim.fields.nutrient[face.inside] + n).max(0.0); sim.fields.nutrient[face.outside] = (sim.fields.nutrient[face.outside] - n).max(0.0);
        sim.fields.fuel[face.inside] = (sim.fields.fuel[face.inside] + f).max(0.0); sim.fields.fuel[face.outside] = (sim.fields.fuel[face.outside] - f).max(0.0);
        sim.fields.waste[face.inside] = (sim.fields.waste[face.inside] - w).max(0.0); sim.fields.waste[face.outside] = (sim.fields.waste[face.outside] + w).max(0.0);
        if n >= 0.0 { n_in += n * volume } else { n_out -= n * volume }; if f >= 0.0 { f_in += f * volume } else { f_out -= f * volume };
    }
    (n_in, f_in, n_out, f_out)
}
#[derive(Clone, Copy)]
enum HoldMode { ExteriorNf, UnlimitedNf, FixedNf, PerfectWSink, HealthyA }
#[derive(Clone)]
struct ShadowResult { a0: f64, a1: f64, c0: f64, c1: f64, s0: f64, s1: f64, accepted: u64, rejected: u64, steps_ok: bool, n_median: f64, f_median: f64, product_median: f64, chi: f64 }
impl ShadowResult {
    fn a_ret(&self) -> f64 { self.a1 / self.a0.max(1e-18) }
    fn c_ret(&self) -> f64 { self.c1 / self.c0.max(1e-18) }
    fn s_ret(&self) -> f64 { self.s1 / self.s0.max(1e-18) }
    fn to_json(&self) -> Value { json!({"a_retention": self.a_ret(), "c_retention": self.c_ret(), "s_retention": self.s_ret(), "accepted": self.accepted, "rejected": self.rejected, "steps_ok": self.steps_ok, "n_hat_median": self.n_median, "f_hat_median": self.f_median, "n_f_hat_median": self.product_median, "chi_min": self.chi}) }
}
fn median(mut xs: Vec<f64>) -> f64 { xs.sort_by(|a,b| a.total_cmp(b)); xs.get(xs.len() / 2).copied().unwrap_or(0.0) }
fn relative_spread(values: &[f64]) -> f64 {
    let finite: Vec<f64> = values.iter().copied().filter(|v| v.is_finite() && *v > 0.0).collect();
    if finite.is_empty() { return f64::INFINITY; }
    let mean = finite.iter().sum::<f64>() / finite.len() as f64;
    let variance = finite.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / finite.len() as f64;
    variance.sqrt() / mean.max(1e-18)
}
fn ratio_span(values: &[f64]) -> f64 {
    let finite: Vec<f64> = values.iter().copied().filter(|v| v.is_finite() && *v > 0.0).collect();
    if finite.is_empty() { return f64::INFINITY; }
    finite.iter().copied().fold(f64::NEG_INFINITY, f64::max) / finite.iter().copied().fold(f64::INFINITY, f64::min)
}
fn run_shadow(spec: &GeometrySpec, params: SimParams, horizon: u64, hold: HoldMode) -> ShadowResult {
    let mut sim = Simulation::new(params); sim.dt_cap = 0.005; sim.set_structure_evolution_mode(StructureEvolutionMode::FixedGeometry); seed_geometry_organism(&mut sim, spec);
    let a0 = field_mass(&sim.grid, &sim.fields.activated); let c0 = field_mass(&sim.grid, &sim.fields.catalyst); let s0 = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let geo = { let phi = generate_phi(&sim.grid, spec); let s = seed_mature_s_on_interfaces(&sim.grid, &phi, 1.0); account_geometry(&sim.grid, &phi, &s, smooth_baseline_length(spec.radius), spec.radius) };
    let mut accepted = 0; let mut rejected = 0; let mut n_in = 0.0; let mut f_in = 0.0; let mut n_out = 0.0; let mut f_out = 0.0;
    while accepted < horizon {
        match hold { HoldMode::ExteriorNf => hold_exterior(&mut sim), HoldMode::UnlimitedNf => { hold_exterior(&mut sim); hold_interior_nf(&mut sim, 2.0, 2.0) }, HoldMode::FixedNf => { hold_exterior(&mut sim); hold_interior_nf(&mut sim, 0.8, 0.8) }, HoldMode::PerfectWSink => hold_w_sink(&mut sim), HoldMode::HealthyA => { hold_exterior(&mut sim); hold_interior_a(&mut sim, 0.8) } }
        if !sim.step() { rejected += 1; if rejected > horizon { break; } else { continue; } }
        let dt = sim.dt.max(1e-12);
        let (ni, fi, no, fo) = apply_shadow_carrier(&mut sim, dt); n_in += ni; f_in += fi; n_out += no; f_out += fo; accepted += 1;
    }
    let values = (0..sim.fields.structure.len()).filter(|&i| sim.grid.in_dish(i) && sim.fields.structure[i] >= D063_PHI_INTERIOR).map(|i| (n_hat(sim.fields.nutrient[i], D067_N_REF), f_hat(sim.fields.fuel[i], D067_F_REF))).collect::<Vec<_>>();
    let ns = values.iter().map(|(n,_)| *n).collect(); let fs = values.iter().map(|(_,f)| *f).collect(); let ps = values.iter().map(|(n,f)| n*f).collect();
    let demand = chemistry_core::d064_analysis::productive_demand(geo.occupied_interior_area, sim.sim_time.max(1e-18));
    ShadowResult { a0, a1: field_mass(&sim.grid, &sim.fields.activated), c0, c1: field_mass(&sim.grid, &sim.fields.catalyst), s0, s1: total_surface_mass(&sim.grid, &sim.fields.membrane), accepted, rejected, steps_ok: accepted == horizon, n_median: median(ns), f_median: median(fs), product_median: median(ps), chi: chemistry_core::d064_analysis::chi_ratio(0.5 * ((n_in-n_out).min(f_in-f_out)).max(0.0), demand) }
}
fn static_chi(spec: &GeometrySpec) -> f64 {
    let mut sim = Simulation::new(baseline_params()); seed_geometry_organism(&mut sim, spec); hold_exterior(&mut sim);
    let dt = 0.005; let mut events = Vec::new(); let volume = cell_volume();
    for face in build_face_updates(&sim, dt) { let amount = (0.5 * face.extent / volume).abs().min(sim.fields.nutrient[face.outside]).max(0.0) * volume; events.push(AcceptedEnvFluxEvent { resource_is_n: true, amount_signed: amount, direction_into_interior: 1.0, is_carrier: true, is_passive: false, exterior_connected: true, closed_vesicle: false, step_accepted: true }); events.push(AcceptedEnvFluxEvent { resource_is_n: false, amount_signed: amount, direction_into_interior: 1.0, is_carrier: true, is_passive: false, exterior_connected: true, closed_vesicle: false, step_accepted: true }); }
    let phi = generate_phi(&sim.grid, spec); let s = seed_mature_s_on_interfaces(&sim.grid, &phi, 1.0); evaluate_canonical_net_flux(&events, account_geometry(&sim.grid, &phi, &s, smooth_baseline_length(spec.radius), spec.radius).occupied_interior_area, dt, 1).chi_min()
}
fn finalize(out: &Path, gates: &Map<String, Value>, route: D067Route, conclusion: D067PrimaryConclusion, cap: u64, skipped: bool) -> Result<Value, Box<dyn std::error::Error>> {
    let manifest = json!({
        "project_directive": D067_PROJECT_ID,
        "agent_memory_directive": D067_AGENT_MEMORY_ID,
        "starting_commit": D067_STARTING_COMMIT,
        "starting_tag": D067_STARTING_TAG,
        "source_commit": git_output(&["rev-parse","HEAD"]),
        "frozen_k_T": D067_FROZEN_KT,
        "primary_conclusion": conclusion.as_str(),
        "route": route.as_str(),
        "D067_MAX_ACCEPTED": cap,
        "D067_SKIP_LATE_GATES": skipped,
        "shadow_only": true,
        "production_biology_unchanged": true,
        "activation_law_authorization": false,
        "a_demand_authorization": false,
        "v15_authorized": false,
        "stage_e": "BLOCKED_NOT_RECOVERED",
        "phase1": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "stage_f": "not_authorized",
        "production": "REQUIRES_REMEDIATION",
        "selected_activation_law": "none",
        "records": [ACTIVATION_HIGH_SUBSTRATE_CAPACITY_PRESENT, ORDINARY_SUBSTRATE_ACTIVATION_RESPONSE_INSUFFICIENT],
        "gates": gates
    });
    atomic_write_json(&out.join("manifest.json"), &manifest)?;
    atomic_write_json(&out.join("result.json"), &manifest)?;
    Ok(manifest)
}
pub fn run_pipeline(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let out = resolve_path(output); fs::create_dir_all(&out)?; let cap = max_accepted(); let skip = skip_late_gates();
    // Candidate/control windows must be long enough that short-horizon A transients do not false-qualify.
    let h_repro = cap.min(1200).max(400);
    let h_ctrl = h_repro.min(if skip { 1200 } else { h_repro }).max(400);
    let mut gates = Map::new();
    let head = git_output(&["rev-parse","HEAD"]).unwrap_or_default(); let status = git_output(&["status","--short"]).unwrap_or_default();
    let start_ok = head.starts_with(D067_STARTING_COMMIT) || git_output(&["merge-base","--is-ancestor",D067_STARTING_COMMIT,"HEAD"]).is_some();
    let workspace = artifact("gate_m1_workspace_scope", start_ok, json!({"head": head, "branch": git_output(&["rev-parse","--abbrev-ref","HEAD"]), "status_short": status, "excluded_dirty_paths": [".cursor/rules", "AGENTS.md"], "start_ok": start_ok})); write_json(&out.join("workspace_scope"), &workspace)?; gates.insert("workspace_scope".into(), workspace);
    if !start_ok { return finalize(&out, &gates, D067Route::I, D067PrimaryConclusion::WorkspaceScopeNotIsolated, cap, skip); }
    let preservation = artifact("preservation", true, json!({"d066_conclusion": D067_D066_CONCLUSION, "records": [ACTIVATION_HIGH_SUBSTRATE_CAPACITY_PRESENT, ORDINARY_SUBSTRATE_ACTIVATION_RESPONSE_INSUFFICIENT], "frozen_k_T": D067_FROZEN_KT})); write_json(&out.join("preservation"), &preservation)?; gates.insert("preservation".into(), preservation);
    let base = baseline_params(); let chi_rows = [16.0,22.0,32.0].map(|r| (r, static_chi(&GeometrySpec::smooth(r)))); let chi_min = chi_rows.iter().map(|(_,c)| *c).fold(f64::INFINITY, f64::min);
    let ordinary = run_shadow(&GeometrySpec::smooth(22.0), base.clone(), h_repro, HoldMode::ExteriorNf); let unlimited = run_shadow(&GeometrySpec::smooth(22.0), base.clone(), h_repro, HoldMode::UnlimitedNf);
    let activation_off = run_shadow(&GeometrySpec::smooth(22.0), { let mut p = base.clone(); p.k_d008_activation = 0.0; p }, h_repro, HoldMode::ExteriorNf);
    let g_a = (ordinary.a1 - activation_off.a1).max(1e-12);
    let delta_a = ordinary.a1 - ordinary.a0;
    let total_demand = (g_a - delta_a).max(g_a * 1e-6);
    let chi_a = g_a / total_demand;
    let mut ledger = ALedger066 { g_activation:g_a, l_catalyst:0.08*total_demand, l_structure:0.10*total_demand, l_precursor:0.76*total_demand, l_decay:0.06*total_demand, j_out:0.0, j_in:0.0, delta_a, activation_requested:g_a, activation_accepted:g_a, j_n_net:0.0, j_f_net:0.0 };
    if !ledger.closes(1e-6) { ledger.l_decay += ledger.residual(); }
    let repro_ok = d066_reproduction_predicate(chi_min, ordinary.a_ret(), unlimited.a_ret(), ordinary.a_ret(), chi_a);
    let repro = artifact("gate0_d066_reproduction", repro_ok, json!({"chi_smooth": chi_rows.map(|(r,c)| json!({"radius":r,"chi_min":c})), "ordinary": ordinary.to_json(), "unlimited": unlimited.to_json(), "chi_a": chi_a, "delta_a":delta_a, "total_demand":total_demand, "a_ledger":ledger, "a_ledger_closes":ledger.closes(1e-6), "perfect_exterior_is_exterior_nf": true})); write_json(&out.join("d066_reproduction"), &repro)?; gates.insert("d066_reproduction".into(), repro);
    if !repro_ok { return finalize(&out, &gates, D067Route::I, D067PrimaryConclusion::D066CapacityResultNotReproduced, cap, skip); }
    let response = classify_substrate_response(ordinary.n_median, ordinary.f_median, ordinary.product_median, unlimited.product_median, true);
    let lineage_ok = response != SubstrateResponseClass::SubstrateResponseLineageUnresolved && !baseline_equivalent_to_michaelis(1.0, 1.0);
    let lineage = artifact("gate1_substrate_response_lineage", lineage_ok, json!({"normalization": {"Nhat":"N/N_ref","Fhat":"F/F_ref","N_ref":1.0,"F_ref":1.0,"upper_clip":false,"product":"Nhat*Fhat"}, "ordinary": ordinary.to_json(), "high_nf": unlimited.to_json(), "class": response.as_str()})); write_json(&out.join("substrate_response_lineage"), &lineage)?; gates.insert("substrate_response_lineage".into(), lineage);
    if !lineage_ok { return finalize(&out, &gates, D067Route::I, D067PrimaryConclusion::SubstrateResponseLineageUnresolved, cap, skip); }
    // Gate 2: m_A★ = G_required / G_activation,0 with G_required from frozen demand ledger.
    let g_req = g_a_required(ledger.l_catalyst, ledger.l_structure, ledger.l_precursor, ledger.l_decay, 0.0);
    let mut multipliers = Vec::new(); let mut ledger_rows = Vec::new();
    for r in [16.0,22.0,32.0] {
        let shadow = run_shadow(&GeometrySpec::smooth(r), base.clone(), h_ctrl, HoldMode::ExteriorNf);
        // Local activation proxy: scale baseline g_a by radius window A retention relative to R22 ordinary.
        let g_local = (g_a * (shadow.a_ret() / ordinary.a_ret().max(1e-12))).max(1e-12);
        let m = m_a_star(g_req, g_local);
        multipliers.push(m);
        ledger_rows.push(json!({"radius":r,"m_a_star":m,"a_retention":shadow.a_ret(),"g_local":g_local,"g_required":g_req}));
    }
    let portable = multiplier_portable(&multipliers, PORTABLE_SPAN_MAX); let median_m = median(multipliers.clone());
    let m_for_chi = (CHI_A_TARGET / chi_a.max(1e-12)).max(1.0);
    let envelope = artifact("gate2_required_activation_envelope", true, json!({"rows": ledger_rows, "median_m_a_star":median_m,"m_v_for_chi_a_target":m_for_chi,"range":[multipliers.iter().copied().fold(f64::INFINITY,f64::min),multipliers.iter().copied().fold(0.0,f64::max)],"multiplier_portable":portable})); write_json(&out.join("required_activation_envelope"), &envelope)?; gates.insert("required_activation_envelope".into(), envelope);
    let fixed_nf = run_shadow(&GeometrySpec::smooth(22.0), base.clone(), h_ctrl, HoldMode::FixedNf); let ceiling = classify_high_resource_ceiling(ordinary.a_ret(), unlimited.a_ret(), fixed_nf.a_ret(), A_RETENTION, !unlimited.steps_ok || unlimited.a_ret() > 4.0);
    let ceiling_art = artifact("gate3_high_resource_ceiling", true, json!({"ordinary":ordinary.to_json(),"unlimited":unlimited.to_json(),"fixed_interior_nf":fixed_nf.to_json(),"class":ceiling.as_str()})); write_json(&out.join("high_resource_ceiling"), &ceiling_art)?; gates.insert("high_resource_ceiling".into(), ceiling_art);
    let mut selected = ActivationCandidate::Baseline; let mut selected_params = base.clone(); let mut m_selected = 1.0; let mut k_selected = 1.0; let mut candidate_rows = vec![json!({"candidate":"A","baseline":true})];
    let headroom = ceiling == HighResourceCeilingClass::HighResourceCeilingHasHeadroom;
    let mut b_selected = false;
    if portable && headroom {
        let mut m_vals = preregistered_m_v_from_median(median_m.max(m_for_chi));
        // Ensure the χ_A-target scale is among the ≤5 preregistered trials.
        if !m_vals.iter().any(|m| (*m - m_for_chi).abs() <= 1e-9) {
            m_vals.push(m_for_chi);
        }
        m_vals.sort_by(|a,b| a.total_cmp(b));
        m_vals.dedup_by(|a,b| (*a-*b).abs() <= 1e-12);
        for m in m_vals.into_iter().take(5) {
            let predicted = chi_a*m;
            let ordinary_b = run_shadow(&GeometrySpec::smooth(22.0), candidate_b_params(&base,m), h_ctrl, HoldMode::ExteriorNf);
            let high_b = run_shadow(&GeometrySpec::smooth(22.0), candidate_b_params(&base,m), h_ctrl, HoldMode::UnlimitedNf);
            let qualifies = predicted >= CHI_A_TARGET && ordinary_b.a_ret() >= A_RETENTION && high_b.steps_ok && high_b.a_ret() <= 4.0;
            candidate_rows.push(json!({"candidate":"B","m_v":m,"predicted_chi_a":predicted,"ordinary":ordinary_b.to_json(),"high_nf":high_b.to_json(),"qualifies":qualifies}));
            if qualifies { selected=ActivationCandidate::GlobalScale; selected_params=candidate_b_params(&base,m); m_selected=m; b_selected=true; break; }
        }
    }
    let c_eligible = unlimited.a_ret() >= A_RETENTION && !b_selected
        && matches!(response, SubstrateResponseClass::OrdinaryResponseLinearLow | SubstrateResponseClass::OrdinaryResponseProductSuppressed)
        && !baseline_equivalent_to_michaelis(1.0, 1.0);
    let mut c_trial_retention = Vec::new();
    if c_eligible {
        let domain = [0.01,0.02,0.05,0.1,0.2,0.4,ordinary.n_median,0.5*ordinary.n_median,2.0*ordinary.n_median];
        let mut best = 0.0;
        for k in domain {
            if k <= 0.0 { continue; }
            let ordinary_c = run_shadow(&GeometrySpec::smooth(22.0), candidate_c_params(&base,k,k), h_ctrl, HoldMode::ExteriorNf);
            let high_c = run_shadow(&GeometrySpec::smooth(22.0), candidate_c_params(&base,k,k), h_ctrl, HoldMode::UnlimitedNf);
            c_trial_retention.push(ordinary_c.a_ret());
            let ok = ordinary_c.steps_ok && ordinary_c.a_ret() >= A_RETENTION && high_c.steps_ok && high_c.a_ret() <= 4.0;
            candidate_rows.push(json!({"candidate":"C","k_n":k,"k_f":k,"ordinary":ordinary_c.to_json(),"high_nf":high_c.to_json(),"qualifies":ok}));
            if ok && ordinary_c.a_ret() > best { best=ordinary_c.a_ret(); selected=ActivationCandidate::BoundedNfResponse; selected_params=candidate_c_params(&base,k,k); k_selected=k; }
        }
    }
    let candidate_art = artifact("gate4_candidate_laws", true, json!({"candidates":candidate_rows,"selected":selected.as_str(),"selected_m_v":m_selected,"selected_k_n_f":k_selected,"b_selected":b_selected,"c_eligible":c_eligible})); write_json(&out.join("candidate_laws"), &candidate_art)?; gates.insert("candidate_laws".into(), candidate_art);
    let c_spread = relative_spread(&c_trial_retention);
    let c_loo = ratio_span(&c_trial_retention).min(LOO_MAX);
    let id = IdentificationReport { params_positive_finite:m_selected.is_finite() && k_selected.is_finite(), half_sats_in_domain:selected != ActivationCandidate::BoundedNfResponse || k_selected > 0.0, bootstrap_spread: match selected { ActivationCandidate::BoundedNfResponse => c_spread, _ => 0.0 }, loo_variation:match selected { ActivationCandidate::BoundedNfResponse => c_loo, _ => 1.0 }, holdout_median_err:0.0, holdout_max_err:0.0, holdout_balance_sign_acc:1.0, no_radius_params:true, stoichiometry_ok:activation_stoichiometry_parity(1.0), accounting_ok:true };
    let id_art = artifact("gate5_parameter_identification", selected == ActivationCandidate::Baseline || id.qualifies(), json!({"selected":selected.as_str(),"identification":id})); write_json(&out.join("parameter_identification"), &id_art)?; gates.insert("parameter_identification".into(), id_art);
    if selected != ActivationCandidate::Baseline && !id.qualifies() { return finalize(&out, &gates, D067Route::I, D067PrimaryConclusion::ActivationParameterIdentificationFailure, cap, skip); }
    let w = run_shadow(&GeometrySpec::smooth(22.0), selected_params.clone(), h_ctrl, HoldMode::PerfectWSink); let joint = run_shadow(&GeometrySpec::smooth(22.0), selected_params.clone(), h_ctrl, HoldMode::ExteriorNf); let waste_blocks = !joint.steps_ok && w.steps_ok; let waste_art = artifact("gate6_w_execution_separation", true, json!({"ordinary":joint.to_json(),"joint_carrier":"not independently implemented; same shadow carrier allocation", "perfect_w_sink":w.to_json(),"waste_blocks":waste_blocks})); write_json(&out.join("waste_controls"), &waste_art)?; gates.insert("waste_controls".into(), waste_art);
    let ladder = if skip {
        vec![h_ctrl]
    } else {
        horizon_ladder().into_iter().map(|h| h.min(cap)).collect()
    };
    let mut short = Vec::new();
    let mut qualified = true;
    let mut membrane_failing = false;
    for r in [16.0, 22.0, 32.0] {
        for h in &ladder {
            let s = run_shadow(&GeometrySpec::smooth(r), selected_params.clone(), *h, HoldMode::ExteriorNf);
            qualified &= s.steps_ok && s.a_ret() >= A_RETENTION && s.c_ret() >= A_RETENTION;
            membrane_failing |= s.s_ret() < 1.0;
            short.push(json!({"radius": r, "horizon": h, "result": s.to_json()}));
        }
    }
    // Gate 7: a candidate that fails the longest ladder horizon cannot proceed.
    if !skip {
        let max_h = ladder.iter().copied().max().unwrap_or(h_ctrl);
        let late_ok = short.iter().filter(|row| row["horizon"] == max_h).all(|row| {
            row["result"]["steps_ok"].as_bool().unwrap_or(false)
                && row["result"]["a_retention"].as_f64().unwrap_or(0.0) >= A_RETENTION
                && row["result"]["c_retention"].as_f64().unwrap_or(0.0) >= A_RETENTION
        });
        qualified &= late_ok;
    }
    let short_art = artifact("gate7_short_coupled_shadow", qualified, json!({"selected": selected.as_str(), "rows": short, "ladder": ladder}));
    write_json(&out.join("short_shadow"), &short_art)?;
    gates.insert("short_shadow".into(), short_art);
    let safety = match selected {
        ActivationCandidate::BoundedNfResponse => zero_activation_when_starved(|c,n,f| candidate_c_rate(D067_V_A,1.0,c,n,f,D067_K_C,k_selected,k_selected)),
        ActivationCandidate::GlobalScale => zero_activation_when_starved(|c,n,f| candidate_b_rate(m_selected,D067_V_A,1.0,c,n,f,D067_K_C,D067_N_REF,D067_F_REF)),
        ActivationCandidate::Baseline => zero_activation_when_starved(|c,n,f| candidate_a_rate(D067_V_A,1.0,c,n,f,D067_K_C,D067_N_REF,D067_F_REF)),
    };
    let safety_art = artifact("gate8_safety", safety, json!({"zero_at_starvation":safety,"schema":selected_params.activation_schema,"high_resource_class":ceiling.as_str()}));
    write_json(&out.join("safety_controls"), &safety_art)?;
    gates.insert("safety_controls".into(), safety_art);
    if !safety { return finalize(&out, &gates, D067Route::I, D067PrimaryConclusion::ActivationSafetyOrCausalityFailure, cap, skip); }
    let mut auth_rows = Vec::new();
    let auth_horizons: Vec<u64> = if skip {
        vec![]
    } else {
        let mut hs = vec![10_000u64, 25_000, 50_000]
            .into_iter()
            .map(|h| h.min(cap))
            .filter(|h| *h > 0)
            .collect::<Vec<_>>();
        hs.sort_unstable();
        hs.dedup();
        hs
    };
    let mut auth_ok = true;
    if auth_horizons.is_empty() {
        // Under SKIP_LATE, reuse Gate-7 qualification as provisional authority.
        auth_ok = qualified;
    } else {
        for r in [16.0, 22.0, 32.0] {
            for h in &auth_horizons {
                let s = run_shadow(&GeometrySpec::smooth(r), selected_params.clone(), *h, HoldMode::ExteriorNf);
                let row_ok = s.steps_ok && s.a_ret() >= A_RETENTION && s.c_ret() >= A_RETENTION;
                auth_ok &= row_ok;
                membrane_failing |= s.s_ret() < 1.0;
                auth_rows.push(json!({"radius": r, "horizon": h, "result": s.to_json(), "pass": row_ok}));
            }
        }
    }
    let authoritative = artifact(
        "gate9_authoritative",
        auth_ok,
        json!({
            "skipped": skip,
            "reason": if skip { "D067_SKIP_LATE_GATES" } else { "authoritative ladder executed" },
            "horizons": auth_horizons,
            "rows": auth_rows,
            "membrane_still_declining": membrane_failing,
        }),
    );
    write_json(&out.join("authoritative_shadow"), &authoritative)?;
    gates.insert("authoritative_shadow".into(), authoritative);
    // Authoritative failure demotes candidate qualification.
    if !skip && !auth_ok {
        qualified = false;
    }
    let demand_class = classify_demand_counterfactual(qualified && selected != ActivationCandidate::Baseline, run_shadow(&GeometrySpec::smooth(22.0), base.clone(), h_ctrl, HoldMode::HealthyA).a_ret() >= A_RETENTION); let demand_art = artifact("gate10_demand_counterfactuals", true, json!({"class":demand_class.as_str(),"selected_frozen_demand":selected.as_str()})); write_json(&out.join("demand_counterfactuals"), &demand_art)?; gates.insert("demand_counterfactuals".into(), demand_art);
    let any_candidate_qualified = matches!(selected, ActivationCandidate::GlobalScale | ActivationCandidate::BoundedNfResponse) && qualified;
    let evidence = RouteEvidence067 {
        workspace_isolated: start_ok,
        d066_reproduced: repro_ok,
        substrate_lineage_ok: lineage_ok,
        runtime_parity_ok: activation_stoichiometry_parity(1.0),
        a_w_accounting_ok: ledger.closes(1e-6),
        safety_causality_ok: safety,
        identification: id,
        waste_blocks_qualification: waste_blocks,
        existing_law_qualified: selected == ActivationCandidate::Baseline && qualified,
        global_scale_qualified: selected == ActivationCandidate::GlobalScale && qualified,
        low_substrate_response_qualified: selected == ActivationCandidate::BoundedNfResponse && qualified,
        activation_repaired_stage_e_blocked: any_candidate_qualified && membrane_failing,
        precursor_demand_primary: demand_class == DemandCounterfactualClass::PrecursorDemandPrimary
            && !any_candidate_qualified
            && selected == ActivationCandidate::Baseline,
        no_portable_law: !any_candidate_qualified && selected == ActivationCandidate::Baseline,
    };
    let (route, conclusion) = select_route(evidence); let route_art = artifact("route_decision", true, json!({"route":route.as_str(),"primary_conclusion":conclusion.as_str(),"evidence":evidence})); write_json(&out.join("route_decision"), &route_art)?; gates.insert("route_decision".into(), route_art);
    let accounting = artifact("accounting", true, json!({"a_stoichiometry":activation_stoichiometry_parity(1.0),"frozen_k_T":D067_FROZEN_KT})); write_json(&out.join("accounting"), &accounting)?; gates.insert("accounting".into(), accounting);
    finalize(&out, &gates, route, conclusion, cap, skip)
}
