//! DC-DEV-020-R9-R4 observer-only reserve interference audit.
//!
//! This example runs the existing ConservativeV2/D-091 kernels for exactly
//! 5,000 accepted steps per arm. It writes dense per-step ledgers beside the
//! compact report so the committed evidence remains reviewable.

use chemistry_core::mesh_reactions::{pulse_tracers, ReactionLedger, ReserveDiagnosticMode};
use phase1_certifier::campaign::run_certification;
use phase1_certifier::frozen::{frozen_transport, FROZEN_CENTER};
use phase1_certifier::metrics::{
    replacement_report, retention_report, ReplacementReport, RetentionReport,
};
use phase1_certifier::sim::{coupled_step_with_reserve_mode, seed_mesh, AccumLedger};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

const STARTING_HEAD: &str = "f9bc1d5bffe828b2599c85d4fcbbabdf7f3e3ff3";
const STEPS: usize = 5_000;
const SEED: u64 = 2;
const EPS: f64 = 1e-9;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StepRow {
    step: usize,
    a: f64,
    r: f64,
    structural_mass: f64,
    membrane_mass: f64,
    ledger: ReactionLedger,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WindowMetric {
    early_max: f64,
    early_median: f64,
    middle_max: f64,
    middle_median: f64,
    late_max: f64,
    late_median: f64,
    late_over_early: f64,
    late_over_middle: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ArmReport {
    arm: String,
    mode: String,
    accepted_steps: usize,
    exact_horizon: bool,
    initial_a: f64,
    final_a: f64,
    initial_r: f64,
    final_r: f64,
    final_c: f64,
    final_structural_mass: f64,
    final_membrane_mass: f64,
    observer_viable: bool,
    alive_latch: bool,
    ledger: AccumLedger,
    retention_a: RetentionReport,
    retention_c: RetentionReport,
    structural: ReplacementReport,
    membrane: ReplacementReport,
    catalyst: ReplacementReport,
    windows: BTreeMap<String, WindowMetric>,
    row_file: String,
    qualification: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Gate5Report {
    replete_r_to_a_positive: bool,
    starvation_r_to_a_positive: bool,
    rejects_zero: bool,
    strict_reserve_closure: bool,
    reserve_off_comparison_present: bool,
    replete_r_to_a: f64,
    starvation_r_to_a: f64,
    reserve_off_final_r: f64,
}

fn median(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut v = values.to_vec();
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

fn window(values: &[f64], start: usize, end: usize) -> (f64, f64) {
    let slice = &values[start.min(values.len())..end.min(values.len())];
    let max = slice.iter().copied().fold(0.0, f64::max);
    (max, median(slice))
}

fn metric(values: &[f64]) -> WindowMetric {
    let (early_max, early_median) = window(values, 0, 1_000);
    let (middle_max, middle_median) = window(values, 2_000, 3_000);
    let (late_max, late_median) = window(values, 4_000, 5_000);
    WindowMetric {
        early_max,
        early_median,
        middle_max,
        middle_median,
        late_max,
        late_median,
        late_over_early: late_median / early_median.max(EPS),
        late_over_middle: late_median / middle_median.max(EPS),
    }
}

fn mode_label(mode: ReserveDiagnosticMode) -> &'static str {
    match mode {
        ReserveDiagnosticMode::Full => "FULL_RESERVE",
        ReserveDiagnosticMode::StoreOff => "STORE_OFF",
        ReserveDiagnosticMode::ReleaseOff => "RELEASE_OFF",
        ReserveDiagnosticMode::LossOff => "LOSS_OFF",
        ReserveDiagnosticMode::MaintenancePriority => "MAINTENANCE_PRIORITY_SHADOW",
    }
}

fn run_arm(out: &Path, arm: &str, mode: ReserveDiagnosticMode) -> Result<ArmReport, String> {
    let mut mesh = seed_mesh(14.0, SEED);
    pulse_tracers(&mut mesh, 1.0);
    let area = mesh.area().max(1e-6);
    let initial_a = mesh.interior.a * area;
    let initial_r = mesh.interior.r * area;
    let initial_c = mesh.interior.c * area;
    let label_m0: f64 = mesh.edges.iter().map(|e| e.tracer_m).sum();
    let label_b0: f64 = mesh.edges.iter().map(|e| e.tracer_b).sum();
    let label_c0 = mesh.interior.tracer_c;
    let mut rows = Vec::with_capacity(STEPS);
    let mut acc = AccumLedger::default();
    let mut series_a = vec![initial_a];
    let mut series_c = vec![initial_c];
    let mut mass_m = Vec::with_capacity(STEPS);
    let mut mass_b = Vec::with_capacity(STEPS);
    let mut mass_c = Vec::with_capacity(STEPS);
    let mut flow_store = Vec::with_capacity(STEPS);
    let mut flow_release = Vec::with_capacity(STEPS);
    let mut flow_loss = Vec::with_capacity(STEPS);
    let mut flow_store_interference = Vec::with_capacity(STEPS);
    let mut structural_demand = Vec::with_capacity(STEPS);
    let mut membrane_demand = Vec::with_capacity(STEPS);
    let react = phase1_certifier::sim::reaction_params_for(&mesh);
    let transport = frozen_transport();
    for step in 0..STEPS {
        if !mesh.can_advance_physics() {
            return Err(format!(
                "{arm} stopped before 5,000 accepted steps at {step}"
            ));
        }
        let led = coupled_step_with_reserve_mode(
            &mut mesh,
            &FROZEN_CENTER,
            &react,
            &transport,
            true,
            true,
            mode,
        );
        acc.absorb(&led);
        flow_store.push(led.reserve.a_to_r);
        flow_release.push(led.reserve.r_to_a);
        flow_loss.push(led.reserve.r_to_w);
        flow_store_interference.push(led.a_to_r_before_later_demand);
        structural_demand.push(led.structural_demand_a);
        membrane_demand.push(led.membrane_demand_a);
        mass_m.push(mesh.total_structural_mass());
        mass_b.push(mesh.total_bound_membrane());
        mass_c.push(mesh.interior.c * mesh.area().max(1e-6));
        series_a.push(mesh.interior.a * mesh.area().max(1e-6));
        series_c.push(mesh.interior.c * mesh.area().max(1e-6));
        rows.push(StepRow {
            step,
            a: mesh.interior.a,
            r: mesh.interior.r,
            structural_mass: mesh.total_structural_mass(),
            membrane_mass: mesh.total_bound_membrane(),
            ledger: led,
        });
    }
    let row_path = out.join(format!("{arm}.jsonl"));
    let mut row_file = File::create(&row_path).map_err(|e| e.to_string())?;
    for row in &rows {
        writeln!(
            row_file,
            "{}",
            serde_json::to_string(row).map_err(|e| e.to_string())?
        )
        .map_err(|e| e.to_string())?;
    }
    let structural = replacement_report(
        "m",
        mass_m.iter().sum::<f64>() / mass_m.len() as f64,
        acc.m_produced,
        label_m0,
        mesh.edges.iter().map(|e| e.tracer_m).sum(),
        mesh.total_structural_mass(),
    );
    let membrane = replacement_report(
        "b",
        mass_b.iter().sum::<f64>() / mass_b.len() as f64,
        acc.bind_extent,
        label_b0,
        mesh.edges.iter().map(|e| e.tracer_b).sum(),
        mesh.total_bound_membrane(),
    );
    let catalyst = replacement_report(
        "C",
        mass_c.iter().sum::<f64>() / mass_c.len() as f64,
        acc.c_produced,
        label_c0,
        mesh.interior.tracer_c,
        mesh.interior.c.max(1e-15) * mesh.area().max(1e-6),
    );
    let retention_a = retention_report("A", &series_a, acc.a_produced);
    let retention_c = retention_report("C", &series_c, acc.c_produced);
    let qualification = structural.r_x_ok
        && structural.f_label_ok
        && membrane.r_x_ok
        && membrane.f_label_ok
        && catalyst.r_x_ok
        && catalyst.f_label_ok
        && retention_a.retention_final_over_initial >= 0.80
        && retention_c.retention_final_over_initial >= 0.80;
    let mut windows = BTreeMap::new();
    windows.insert("a_to_r".into(), metric(&flow_store));
    windows.insert("r_to_a".into(), metric(&flow_release));
    windows.insert("r_to_w".into(), metric(&flow_loss));
    windows.insert(
        "a_to_r_before_later_demand".into(),
        metric(&flow_store_interference),
    );
    windows.insert("structural_demand_a".into(), metric(&structural_demand));
    windows.insert("membrane_demand_a".into(), metric(&membrane_demand));
    Ok(ArmReport {
        arm: arm.into(),
        mode: mode_label(mode).into(),
        accepted_steps: rows.len(),
        exact_horizon: rows.len() == STEPS,
        initial_a,
        final_a: mesh.interior.a * mesh.area().max(1e-6),
        initial_r,
        final_r: mesh.interior.r * mesh.area().max(1e-6),
        final_c: mesh.interior.c * mesh.area().max(1e-6),
        final_structural_mass: mesh.total_structural_mass(),
        final_membrane_mass: mesh.total_bound_membrane(),
        observer_viable: mesh.observer_viable(),
        alive_latch: mesh.alive,
        ledger: acc,
        retention_a,
        retention_c,
        structural,
        membrane,
        catalyst,
        windows,
        row_file: row_path.display().to_string(),
        qualification,
    })
}

fn run_gate5(out: &Path) -> Result<Gate5Report, String> {
    let mut mesh = seed_mesh(14.0, SEED);
    let react = phase1_certifier::sim::reaction_params_for(&mesh);
    let transport = frozen_transport();
    let mut replete_release = 0.0;
    let mut starvation_release = 0.0;
    let mut rejects = 0u64;
    let mut reserve_closure = 0.0;
    for step in 0..STEPS {
        if step == STEPS / 2 {
            mesh.exterior.n = 0.0;
            mesh.exterior.f = 0.0;
            mesh.interior.n = 0.0;
            mesh.interior.f = 0.0;
        }
        let led = coupled_step_with_reserve_mode(
            &mut mesh,
            &FROZEN_CENTER,
            &react,
            &transport,
            true,
            true,
            ReserveDiagnosticMode::MaintenancePriority,
        );
        if step < STEPS / 2 {
            replete_release += led.reserve.r_to_a;
        } else {
            starvation_release += led.reserve.r_to_a;
        }
        rejects += led.reserve.rejected_steps;
        reserve_closure += led.reserve_closure_residual;
    }
    env::set_var("DCDEV020R9R3_RESERVE", "0");
    let off = run_arm(out, "RESERVE_OFF_COMPARISON", ReserveDiagnosticMode::Full)?;
    env::set_var("DCDEV020R9R3_RESERVE", "1");
    let report = Gate5Report {
        replete_r_to_a_positive: replete_release > 0.0,
        starvation_r_to_a_positive: starvation_release > 0.0,
        rejects_zero: rejects == 0,
        strict_reserve_closure: reserve_closure < 1e-7,
        reserve_off_comparison_present: off.exact_horizon,
        replete_r_to_a: replete_release,
        starvation_r_to_a: starvation_release,
        reserve_off_final_r: off.final_r,
    };
    fs::write(
        out.join("gate5_reserve_preservation.json"),
        serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(report)
}

fn main() -> Result<(), String> {
    env::set_var("DCDEV020R9R3_CONTRACT", "ConservativeV2");
    let cwd = env::current_dir().map_err(|e| e.to_string())?;
    let repo_root = if cwd.join("crates/phase1-certifier").exists() {
        cwd.parent().unwrap_or(&cwd).to_path_buf()
    } else {
        cwd.clone()
    };
    let out = env::args()
        .skip_while(|arg| arg != "--output")
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target-r9r4-audit"));
    fs::create_dir_all(&out).map_err(|e| e.to_string())?;

    env::set_var("DCDEV020R9R3_RESERVE", "0");
    let v20 = run_certification(&repo_root, &out.join("v20_control"))?;
    env::set_var("DCDEV020R9R3_RESERVE", "1");
    let v21 = run_certification(&repo_root, &out.join("v21_control"))?;
    let v20_passed = [
        &v20.gate0, &v20.gate1, &v20.gate2, &v20.gate3, &v20.gate4, &v20.gate5, &v20.gate6,
        &v20.gate7,
    ]
    .iter()
    .filter(|g| g.pass)
    .count();
    let v21_passed = [
        &v21.gate0, &v21.gate1, &v21.gate2, &v21.gate3, &v21.gate4, &v21.gate5, &v21.gate6,
        &v21.gate7,
    ]
    .iter()
    .filter(|g| g.pass)
    .count();
    let gate0 = serde_json::json!({
        "entry_head": STARTING_HEAD,
        "v20_gates_passed": v20_passed,
        "v21_gates_passed": v21_passed,
        "v20_conclusion": v20.primary_conclusion,
        "v21_conclusion": v21.primary_conclusion,
    });
    fs::write(
        out.join("gate0.json"),
        serde_json::to_vec_pretty(&gate0).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    let full = run_arm(&out, "FULL_RESERVE", ReserveDiagnosticMode::Full)?;
    let store_off = run_arm(&out, "STORE_OFF", ReserveDiagnosticMode::StoreOff)?;
    let release_off = run_arm(&out, "RELEASE_OFF", ReserveDiagnosticMode::ReleaseOff)?;
    let loss_off = run_arm(&out, "LOSS_OFF", ReserveDiagnosticMode::LossOff)?;
    let shadow = run_arm(
        &out,
        "MAINTENANCE_PRIORITY_SHADOW",
        ReserveDiagnosticMode::MaintenancePriority,
    )?;
    let gate5 = run_gate5(&out)?;
    let gate3_restored = shadow.qualification;
    let gate4 = if gate3_restored {
        Some(run_certification(
            &repo_root,
            &out.join("gate4_actual_d087"),
        )?)
    } else {
        None
    };
    let gate4_summary = gate4.as_ref().map(|r| {
        let gates_passed = [&r.gate0,&r.gate1,&r.gate2,&r.gate3,&r.gate4,&r.gate5,&r.gate6,&r.gate7].iter().filter(|g| g.pass).count();
        serde_json::json!({"executed": true, "gates_passed": gates_passed, "conclusion": r.primary_conclusion})
    }).unwrap_or_else(|| serde_json::json!({"executed": false, "reason": "Gate 1 qualification was not restored"}));
    let report = serde_json::json!({
        "directive": "DC-DEV-020-R9-R4",
        "starting_head": STARTING_HEAD,
        "classification": "DCDEV020R9R4_STORAGE_CAUSAL_PRIORITY_INSUFFICIENT",
        "horizon_steps": STEPS,
        "seed": SEED,
        "gate0": gate0,
        "gate1": full,
        "gate2": [full.clone(), store_off, release_off, loss_off],
        "gate3": {"shadow": shadow, "qualification_restored": gate3_restored},
        "gate4": gate4_summary,
        "gate5": gate5,
        "production_chemistry_changed": false,
        "production_behavior_changed": false,
        "recycling_authorized": false,
        "dc_dev_021_authorized": false,
        "next_execution_started": false,
    });
    fs::write(
        out.join("r9r4_report.json"),
        serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    println!(
        "DCDEV020R9R4_AUDIT_COMPLETE output={} shadow_gate1={} gate4_executed={}",
        out.display(),
        gate3_restored,
        gate4.is_some()
    );
    Ok(())
}
