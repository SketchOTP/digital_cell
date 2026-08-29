//! DC-DEV-020-R9-R5 observer-only decomposition of reserve charging and use.
//!
//! The runner keeps the frozen ConservativeV2/D-091 kernels and exposes only
//! two counterfactuals: surplus-capped A→R storage and an instantaneous 1:1 R
//! substitute for existing A-dependent M/L maintenance. Dense rows are local
//! audit output; the committed report is compact.

use chemistry_core::mesh_reactions::{pulse_tracers, ReactionLedger, ReserveDiagnosticMode};
use phase1_certifier::campaign::{run_certification, CertificationReport};
use phase1_certifier::frozen::{frozen_transport, FROZEN_CENTER};
use phase1_certifier::metrics::{
    replacement_report, retention_report, ReplacementReport, RetentionReport,
};
use phase1_certifier::sim::{coupled_step_with_reserve_mode, seed_mesh, AccumLedger};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

const STARTING_HEAD: &str = "08e1c45b11892e0b5533b11c74f175ee84d243ed";
const R1_STARTING_HEAD: &str = "f1fad5c65859f3a314102d3ec5a0751822a2f5ea";
const STEPS: usize = 5_000;
const SEED: u64 = 2;
const EPS: f64 = 1e-8;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StepRow {
    step: usize,
    a: f64,
    r: f64,
    ledger: ReactionLedger,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReserveFunction {
    replete_a_to_r: f64,
    starvation_r_to_a: f64,
    rejected_steps: u64,
    strict_closure: bool,
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
    final_structural_mass: f64,
    final_membrane_mass: f64,
    ledger: AccumLedger,
    retention_a: RetentionReport,
    structural: ReplacementReport,
    reserve_function: ReserveFunction,
    store_cap_respected: bool,
    ordinary_r_to_a: f64,
    diagnostic_liquid_r_used: f64,
    diagnostic_liquid_r_available: f64,
    diagnostic_liquid_r_used_for_m: f64,
    diagnostic_liquid_r_used_for_l: f64,
    row_file: String,
}

fn r1_enabled() -> bool {
    matches!(
        env::var("DCDEV020R9R5_R1").ok().as_deref(),
        Some("1") | Some("true") | Some("TRUE")
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ActualD087Summary {
    mode: String,
    gates: Vec<bool>,
    all_pass: bool,
    primary_conclusion: String,
    artifact_root: String,
    gate1_detail: String,
    gate2_detail: String,
    gate3_detail: String,
    gate4_detail: String,
}

fn mode_label(mode: ReserveDiagnosticMode) -> &'static str {
    match mode {
        ReserveDiagnosticMode::Full => "FULL",
        ReserveDiagnosticMode::StoreOff => "STORE_OFF",
        ReserveDiagnosticMode::ReleaseOff => "RELEASE_OFF",
        ReserveDiagnosticMode::LossOff => "LOSS_OFF",
        ReserveDiagnosticMode::MaintenancePriority => "MAINTENANCE_PRIORITY",
        ReserveDiagnosticMode::SurplusOnlyStore => "SURPLUS_ONLY_STORE",
        ReserveDiagnosticMode::LiquidReserveUpperBound => "LIQUID_RESERVE_UB",
        ReserveDiagnosticMode::LiquidReservePreThrottleUpperBound => {
            "LIQUID_RESERVE_PRETHROTTLE_UB"
        }
        ReserveDiagnosticMode::SurplusOnlyStoreLiquidReserveUpperBound => {
            "SURPLUS_ONLY_STORE_LIQUID_RESERVE_UB"
        }
        ReserveDiagnosticMode::SurplusOnlyStoreLiquidReservePreThrottleUpperBound => {
            "SURPLUS_ONLY_STORE_LIQUID_RESERVE_PRETHROTTLE_UB"
        }
        ReserveDiagnosticMode::MobilizeFirstStoreLast => "MOBILIZE_FIRST_STORE_LAST",
    }
}

fn run_arm(out: &Path, arm: &str, mode: ReserveDiagnosticMode) -> Result<ArmReport, String> {
    let mut mesh = seed_mesh(14.0, SEED);
    pulse_tracers(&mut mesh, 1.0);
    let area = mesh.area().max(1e-6);
    let initial_a = mesh.interior.a * area;
    let initial_r = mesh.interior.r * area;
    let label_m0: f64 = mesh.edges.iter().map(|e| e.tracer_m).sum();
    let mut rows = Vec::with_capacity(STEPS);
    let mut acc = AccumLedger::default();
    let mut series_a = vec![initial_a];
    let mut mass_m = Vec::with_capacity(STEPS);
    let mut rejects = 0;
    let mut closure = 0.0;
    let mut store_cap_respected = true;
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
        if matches!(
            mode,
            ReserveDiagnosticMode::SurplusOnlyStore
                | ReserveDiagnosticMode::SurplusOnlyStoreLiquidReserveUpperBound
                | ReserveDiagnosticMode::SurplusOnlyStoreLiquidReservePreThrottleUpperBound
        ) && led.reserve.a_to_r > led.reserve_store_potential.min(led.new_a_surplus) + EPS
        {
            store_cap_respected = false;
        }
        rejects += led.reserve.rejected_steps;
        closure += led.activation_equivalent_closure_residual;
        acc.absorb(&led);
        series_a.push(mesh.interior.a * mesh.area().max(1e-6));
        mass_m.push(mesh.total_structural_mass());
        rows.push(StepRow {
            step,
            a: mesh.interior.a,
            r: mesh.interior.r,
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
    let reserve_function = ReserveFunction {
        replete_a_to_r: run_reserve_function(mode, false),
        starvation_r_to_a: run_reserve_function(mode, true),
        rejected_steps: rejects,
        strict_closure: closure <= 1e-6,
    };
    Ok(ArmReport {
        arm: arm.into(),
        mode: mode_label(mode).into(),
        accepted_steps: rows.len(),
        exact_horizon: rows.len() == STEPS,
        initial_a,
        final_a: mesh.interior.a * mesh.area().max(1e-6),
        initial_r,
        final_r: mesh.interior.r * mesh.area().max(1e-6),
        final_structural_mass: mesh.total_structural_mass(),
        final_membrane_mass: mesh.total_bound_membrane(),
        ledger: acc.clone(),
        retention_a: retention_report("A", &series_a, acc.a_produced),
        structural,
        reserve_function,
        store_cap_respected,
        ordinary_r_to_a: acc.reserve_r_to_a,
        diagnostic_liquid_r_used: acc.diagnostic_liquid_r_used,
        diagnostic_liquid_r_available: acc.diagnostic_liquid_r_available,
        diagnostic_liquid_r_used_for_m: acc.diagnostic_liquid_r_used_for_m,
        diagnostic_liquid_r_used_for_l: acc.diagnostic_liquid_r_used_for_l,
        row_file: row_path.display().to_string(),
    })
}

fn run_reserve_function(mode: ReserveDiagnosticMode, starvation: bool) -> f64 {
    let mut mesh = seed_mesh(14.0, SEED);
    let react = phase1_certifier::sim::reaction_params_for(&mesh);
    let transport = frozen_transport();
    let mut total = 0.0;
    for step in 0..1_000 {
        if starvation && step == 500 {
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
            mode,
        );
        if starvation {
            if step >= 500 {
                total += led.reserve.r_to_a;
            }
        } else {
            total += led.reserve.a_to_r;
        }
    }
    total
}

fn actual_d087(
    repo_root: &Path,
    root: &Path,
    mode: Option<&str>,
) -> Result<ActualD087Summary, String> {
    env::set_var("DCDEV020R9R3_CONTRACT", "ConservativeV2");
    env::set_var("DCDEV020R9R3_RESERVE", "1");
    if let Some(mode) = mode {
        env::set_var("DCDEV020R9R5_MODE", mode);
    } else {
        env::remove_var("DCDEV020R9R5_MODE");
    }
    let report: CertificationReport = run_certification(repo_root, root)?;
    let gates = vec![
        report.gate0.pass,
        report.gate1.pass,
        report.gate2.pass,
        report.gate3.pass,
        report.gate4.pass,
        report.gate5.pass,
        report.gate6.pass,
        report.gate7.pass,
    ];
    Ok(ActualD087Summary {
        mode: mode.unwrap_or("FULL").into(),
        all_pass: gates.iter().all(|pass| *pass),
        gates,
        primary_conclusion: report.primary_conclusion,
        artifact_root: report.artifact_root,
        gate1_detail: report.gate1.detail,
        gate2_detail: report.gate2.detail,
        gate3_detail: report.gate3.detail,
        gate4_detail: report.gate4.detail,
    })
}

fn main() -> Result<(), String> {
    let r1 = r1_enabled();
    let cwd = env::current_dir().map_err(|e| e.to_string())?;
    let repo_root = if cwd.join("crates/phase1-certifier").exists() {
        cwd.parent().unwrap_or(&cwd).to_path_buf()
    } else {
        cwd.join("digital-protocell")
    };
    let out = env::args()
        .skip_while(|arg| arg != "--output")
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020r9r5"));
    fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    env::set_var("DCDEV020R9R3_CONTRACT", "ConservativeV2");

    // Gate 0: exact controls and V20 positive reproduction.
    env::set_var("DCDEV020R9R3_RESERVE", "0");
    env::remove_var("DCDEV020R9R5_MODE");
    let v20 = run_certification(&repo_root, &out.join("v20_control"))?;
    let v20_gates = vec![
        v20.gate0.pass,
        v20.gate1.pass,
        v20.gate2.pass,
        v20.gate3.pass,
        v20.gate4.pass,
        v20.gate5.pass,
        v20.gate6.pass,
        v20.gate7.pass,
    ];
    let v20_all_pass = v20_gates.iter().all(|v| *v);
    env::set_var("DCDEV020R9R3_RESERVE", "1");
    let full = run_arm(&out, "full", ReserveDiagnosticMode::Full)?;
    let store_off = run_arm(&out, "store_off", ReserveDiagnosticMode::StoreOff)?;
    let full_d087 = actual_d087(&repo_root, &out.join("full_control"), None)?;

    let surplus = run_arm(
        &out,
        "surplus_only_store",
        ReserveDiagnosticMode::SurplusOnlyStore,
    )?;
    let liquid_mode = if r1 {
        ReserveDiagnosticMode::LiquidReservePreThrottleUpperBound
    } else {
        ReserveDiagnosticMode::LiquidReserveUpperBound
    };
    let liquid = run_arm(&out, "liquid_reserve_ub", liquid_mode)?;
    let surplus_d087 = actual_d087(
        &repo_root,
        &out.join("surplus_only_store_d087"),
        Some("SURPLUS_ONLY_STORE"),
    )?;
    let liquid_d087 = actual_d087(
        &repo_root,
        &out.join("liquid_reserve_ub_d087"),
        Some(if r1 {
            "LIQUID_RESERVE_PRETHROTTLE_UB"
        } else {
            "LIQUID_RESERVE_UB"
        }),
    )?;
    let run_combined = !r1 && !surplus_d087.all_pass && !liquid_d087.all_pass;
    let combined = if run_combined {
        Some(run_arm(
            &out,
            "surplus_only_store_liquid_reserve_ub",
            ReserveDiagnosticMode::SurplusOnlyStoreLiquidReserveUpperBound,
        )?)
    } else {
        None
    };
    let combined_d087 = if run_combined {
        Some(actual_d087(
            &repo_root,
            &out.join("combined_d087"),
            Some("SURPLUS_ONLY_STORE_LIQUID_RESERVE_UB"),
        )?)
    } else {
        None
    };
    env::remove_var("DCDEV020R9R5_MODE");

    let gate0 = serde_json::json!({
        "entry_head": if r1 { R1_STARTING_HEAD } else { STARTING_HEAD },
        "v20_gates": v20_gates,
        "v20_all_pass": v20_all_pass,
        "full_r_m": full.structural.r_x,
        "store_off_r_m": store_off.structural.r_x,
        "full_d087": full_d087,
    });
    let surplus_restored = surplus_d087.all_pass;
    let liquid_restored = liquid_d087.all_pass;
    let combined_restored = combined_d087
        .as_ref()
        .map(|summary| summary.all_pass)
        .unwrap_or(false);
    let classification = if r1 {
        if !v20_all_pass {
            "DCDEV020R9R5R1_V20_CONTROL_NOT_REPRODUCED"
        } else if liquid_restored {
            "DCDEV020R9R5R1_RESERVE_LIQUIDITY_DEFECT_CONFIRMED"
        } else if liquid.structural.r_x > full.structural.r_x + 1e-9 {
            "DCDEV020R9R5R1_RESERVE_LIQUIDITY_CONTRIBUTORY_NOT_SUFFICIENT"
        } else {
            "DCDEV020R9R5R1_RESERVE_LIQUIDITY_NOT_SUFFICIENT"
        }
    } else {
        match (surplus_restored, liquid_restored, combined_restored) {
            (true, true, _) => "DCDEV020R9R5_OVERCHARGE_AND_LIQUIDITY_DUAL_CAPACITY",
            (true, false, _) => "DCDEV020R9R5_STANDING_STOCK_OVERCHARGE_CONFIRMED",
            (false, true, _) => "DCDEV020R9R5_RESERVE_LIQUIDITY_DEFICIT_CONFIRMED",
            (false, false, true) => "DCDEV020R9R5_COUPLED_OVERCHARGE_LIQUIDITY_INTERACTION",
            (false, false, false) => {
                "DCDEV020R9R5_RESERVE_DEFECT_OUTSIDE_CHARGE_LIQUIDITY_FACTORIZATION"
            }
        }
    };
    let report = serde_json::json!({
        "directive": if r1 { "DC-DEV-020-R9-R5-R1" } else { "DC-DEV-020-R9-R5" },
        "starting_head": if r1 { R1_STARTING_HEAD } else { STARTING_HEAD },
        "horizon_steps": STEPS,
        "seed": SEED,
        "gate0": gate0,
        "gate1": {"full": full, "store_off": store_off},
        "gate2_surplus_only_store": {"arm": surplus, "actual_d087": surplus_d087, "restored": surplus_restored},
        "gate3_liquid_reserve_ub": {"arm": liquid, "actual_d087": liquid_d087, "restored": liquid_restored},
        "gate4_combined": {"arm": combined, "actual_d087": combined_d087, "restored": combined_restored, "executed": run_combined},
        "classification": classification,
        "r9r5_architect_status": if r1 { "REPLAN_NOT_ACCEPTED" } else { "PENDING" },
        "r9r5_outside_factorization_classification_retired": r1,
        "valid_liquid_counterfactual_definition": if r1 { "Existing frozen structural M and membrane L demand equations receive only the currently available activation-equivalent R as a pre-throttle shadow availability; A is consumed first, diagnostic R supplies only the unmet M/L amount, and no unrelated chemistry sees the substitution." } else { "not_applicable" },
        "counterfactual_acts_before_a_limited_m_l_rate_calculation": r1,
        "diagnostic_r_usage": {
            "full_available": full.diagnostic_liquid_r_available,
            "full_used": full.diagnostic_liquid_r_used,
            "liquid_available": liquid.diagnostic_liquid_r_available,
            "liquid_used": liquid.diagnostic_liquid_r_used,
            "liquid_used_for_m": liquid.diagnostic_liquid_r_used_for_m,
            "liquid_used_for_l": liquid.diagnostic_liquid_r_used_for_l
        },
        "reserve_function": {
            "surplus": surplus.reserve_function,
            "liquid": liquid.reserve_function,
            "combined": combined.as_ref().map(|arm| &arm.reserve_function),
        },
        "ordinary_r_to_a": {"full": full.ordinary_r_to_a, "surplus": surplus.ordinary_r_to_a, "liquid": liquid.ordinary_r_to_a, "combined": combined.as_ref().map(|arm| arm.ordinary_r_to_a)},
        "diagnostic_liquid_r_used": {"full": full.diagnostic_liquid_r_used, "surplus": surplus.diagnostic_liquid_r_used, "liquid": liquid.diagnostic_liquid_r_used, "combined": combined.as_ref().map(|arm| arm.diagnostic_liquid_r_used)},
        "strict_closure": surplus.ledger.activation_equivalent_closure_residual <= 1e-6 && liquid.ledger.activation_equivalent_closure_residual <= 1e-6 && combined.as_ref().map(|arm| arm.ledger.activation_equivalent_closure_residual <= 1e-6).unwrap_or(true),
        "production_changes": false,
        "recycling_authorized": false,
        "dc_dev_021_authorized": false,
        "next_execution_started": false,
    });
    fs::write(
        out.join("r9r5_report.json"),
        serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    println!(
        "DCDEV020R9R5_AUDIT_COMPLETE output={} classification={}",
        out.display(),
        classification
    );
    Ok(())
}
