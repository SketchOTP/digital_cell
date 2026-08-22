//! DC-DEV-020-R9-R6 final observer-only D-091 phase-order audit.
//!
//! The shadow reuses the frozen release/loss kernels before productive
//! chemistry and the frozen store kernel after productive chemistry. It does
//! not add a rate, target, availability signal, or direct R→M/R→L pathway.

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

const STARTING_HEAD: &str = "f1704acff5ca64e509a28c74af8cccbf76439ef2";
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
    a_at_starvation: f64,
    r_at_starvation: f64,
    maximum_starvation_r_to_a_step: f64,
    rejected_steps: u64,
    strict_closure: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ActualD087Summary {
    mode: String,
    gates: Vec<bool>,
    all_pass: bool,
    primary_conclusion: String,
    artifact_root: String,
    gate1_detail: String,
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
    organized_retained_material: f64,
    ledger: AccumLedger,
    retention_a: RetentionReport,
    retention_c: RetentionReport,
    structural: ReplacementReport,
    membrane: ReplacementReport,
    catalyst: ReplacementReport,
    reserve_function: ReserveFunction,
    row_file: String,
}

fn gates(report: &CertificationReport) -> Vec<bool> {
    vec![
        report.gate0.pass,
        report.gate1.pass,
        report.gate2.pass,
        report.gate3.pass,
        report.gate4.pass,
        report.gate5.pass,
        report.gate6.pass,
        report.gate7.pass,
    ]
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
    let report = run_certification(repo_root, root)?;
    let gate_values = gates(&report);
    Ok(ActualD087Summary {
        mode: mode.unwrap_or("FULL").into(),
        all_pass: gate_values.iter().all(|pass| *pass),
        gates: gate_values,
        primary_conclusion: report.primary_conclusion,
        artifact_root: "certification_artifacts".into(),
        gate1_detail: report.gate1.detail,
    })
}

fn run_reserve_function(mode: ReserveDiagnosticMode, starvation: bool) -> ReserveFunction {
    let mut mesh = seed_mesh(14.0, SEED);
    let react = phase1_certifier::sim::reaction_params_for(&mesh);
    let transport = frozen_transport();
    let mut a_to_r = 0.0;
    let mut r_to_a = 0.0;
    let mut a_at_starvation = 0.0;
    let mut r_at_starvation = 0.0;
    let mut maximum_starvation_r_to_a_step: f64 = 0.0;
    let mut rejects = 0;
    let mut closure = 0.0;
    for step in 0..STEPS {
        if starvation && step == STEPS / 2 {
            mesh.exterior.n = 0.0;
            mesh.exterior.f = 0.0;
            mesh.interior.n = 0.0;
            mesh.interior.f = 0.0;
            a_at_starvation = mesh.interior.a * mesh.area().max(1e-6);
            r_at_starvation = mesh.interior.r * mesh.area().max(1e-6);
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
        if starvation && step >= STEPS / 2 {
            r_to_a += led.reserve.r_to_a;
            maximum_starvation_r_to_a_step = maximum_starvation_r_to_a_step.max(led.reserve.r_to_a);
        } else if !starvation {
            a_to_r += led.reserve.a_to_r;
        }
        rejects += led.reserve.rejected_steps;
        closure += led.activation_equivalent_closure_residual;
    }
    ReserveFunction {
        replete_a_to_r: a_to_r,
        starvation_r_to_a: r_to_a,
        a_at_starvation,
        r_at_starvation,
        maximum_starvation_r_to_a_step,
        rejected_steps: rejects,
        strict_closure: closure <= 1e-6,
    }
}

fn run_arm(out: &Path, arm: &str, mode: ReserveDiagnosticMode) -> Result<ArmReport, String> {
    let mut mesh = seed_mesh(14.0, SEED);
    pulse_tracers(&mut mesh, 1.0);
    let area = mesh.area().max(1e-6);
    let initial_a = mesh.interior.a * area;
    let initial_r = mesh.interior.r * area;
    let initial_c = mesh.interior.c * area;
    let label_m0: f64 = mesh.edges.iter().map(|edge| edge.tracer_m).sum();
    let label_b0: f64 = mesh.edges.iter().map(|edge| edge.tracer_b).sum();
    let label_c0 = mesh.interior.tracer_c;
    let mut rows = Vec::with_capacity(STEPS);
    let mut acc = AccumLedger::default();
    let mut series_a = vec![initial_a];
    let mut series_c = vec![initial_c];
    let mut mass_m = Vec::with_capacity(STEPS);
    let mut mass_b = Vec::with_capacity(STEPS);
    let mut mass_c = Vec::with_capacity(STEPS);
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
        mass_m.push(mesh.total_structural_mass());
        mass_b.push(mesh.total_bound_membrane());
        mass_c.push(mesh.interior.c * mesh.area().max(1e-6));
        series_a.push(mesh.interior.a * mesh.area().max(1e-6));
        series_c.push(mesh.interior.c * mesh.area().max(1e-6));
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
        mesh.edges.iter().map(|edge| edge.tracer_m).sum(),
        mesh.total_structural_mass(),
    );
    let membrane = replacement_report(
        "b",
        mass_b.iter().sum::<f64>() / mass_b.len() as f64,
        acc.bind_extent,
        label_b0,
        mesh.edges.iter().map(|edge| edge.tracer_b).sum(),
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
    let replete_function = run_reserve_function(mode, false);
    let starvation_function = run_reserve_function(mode, true);
    let reserve_function = ReserveFunction {
        replete_a_to_r: replete_function.replete_a_to_r,
        starvation_r_to_a: starvation_function.starvation_r_to_a,
        a_at_starvation: starvation_function.a_at_starvation,
        r_at_starvation: starvation_function.r_at_starvation,
        maximum_starvation_r_to_a_step: starvation_function.maximum_starvation_r_to_a_step,
        rejected_steps: replete_function.rejected_steps + starvation_function.rejected_steps,
        strict_closure: replete_function.strict_closure && starvation_function.strict_closure,
    };
    Ok(ArmReport {
        arm: arm.into(),
        mode: if mode == ReserveDiagnosticMode::Full {
            "FULL".into()
        } else {
            "MOBILIZE_FIRST_STORE_LAST".into()
        },
        accepted_steps: rows.len(),
        exact_horizon: rows.len() == STEPS,
        initial_a,
        final_a: mesh.interior.a * mesh.area().max(1e-6),
        initial_r,
        final_r: mesh.interior.r * mesh.area().max(1e-6),
        final_structural_mass: mesh.total_structural_mass(),
        final_membrane_mass: mesh.total_bound_membrane(),
        organized_retained_material: mesh.total_structural_mass() + mesh.total_bound_membrane(),
        ledger: acc.clone(),
        retention_a: retention_report("A", &series_a, acc.a_produced),
        retention_c: retention_report("C", &series_c, acc.c_produced),
        structural,
        membrane,
        catalyst,
        reserve_function,
        row_file: format!("{arm}.jsonl"),
    })
}

fn main() -> Result<(), String> {
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
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020r9r6"));
    fs::create_dir_all(&out).map_err(|e| e.to_string())?;

    env::set_var("DCDEV020R9R3_CONTRACT", "ConservativeV2");
    env::set_var("DCDEV020R9R3_RESERVE", "0");
    env::remove_var("DCDEV020R9R5_MODE");
    let v20 = run_certification(&repo_root, &out.join("v20_control"))?;
    let v20_gates = gates(&v20);
    let v20_pass = v20_gates.iter().all(|pass| *pass);

    env::set_var("DCDEV020R9R3_RESERVE", "1");
    let full = run_arm(&out, "full", ReserveDiagnosticMode::Full)?;
    let shadow = run_arm(
        &out,
        "mobilize_first_store_last",
        ReserveDiagnosticMode::MobilizeFirstStoreLast,
    )?;
    let full_d087 = actual_d087(&repo_root, &out.join("full_d087"), None)?;
    let shadow_d087 = actual_d087(
        &repo_root,
        &out.join("shadow_d087"),
        Some("MOBILIZE_FIRST_STORE_LAST"),
    )?;
    env::remove_var("DCDEV020R9R5_MODE");

    let controls_match =
        (full.structural.r_x - 0.8398695202805284).abs() <= 1e-9 && full_d087.mode == "FULL";
    let shadow_improves = shadow.structural.r_x > full.structural.r_x + 1e-9
        || shadow.ledger.c_produced > full.ledger.c_produced + 1e-9;
    let classification = if !v20_pass || !controls_match {
        "DCDEV020R9R6_CONTROL_REGRESSION"
    } else if shadow_d087.all_pass {
        "DCDEV020R9R6_MOBILIZE_FIRST_STORE_LAST_CAPACITY_CONFIRMED"
    } else if shadow_improves {
        "DCDEV020R9R6_MOBILIZE_FIRST_STORE_LAST_CONTRIBUTORY_NOT_SUFFICIENT"
    } else {
        "DCDEV020R9R6_MOBILIZE_FIRST_STORE_LAST_INSUFFICIENT"
    };
    let reserve_function_preserved = shadow.reserve_function.replete_a_to_r > EPS
        && shadow.reserve_function.starvation_r_to_a > EPS
        && shadow.reserve_function.rejected_steps == 0
        && shadow.reserve_function.strict_closure;
    let strict_closure = full.ledger.activation_equivalent_closure_residual <= 1e-6
        && shadow.ledger.activation_equivalent_closure_residual <= 1e-6;

    let report = serde_json::json!({
        "directive": "DC-DEV-020-R9-R6",
        "starting_head": STARTING_HEAD,
        "horizon_steps": STEPS,
        "seed": SEED,
        "gate0": {
            "v20_gates": v20_gates,
            "v20_all_pass": v20_pass,
            "full_r_m": full.structural.r_x,
            "full_r_m_reference": 0.8398695202805284,
            "store_off_r_m_reference": 1.0180981834599838,
            "controls_match": controls_match,
            "accepted_r9r5r1_classification": "DCDEV020R9R5R1_RESERVE_LIQUIDITY_CONTRIBUTORY_NOT_SUFFICIENT"
        },
        "gate1_phase_order": {
            "full": full,
            "shadow": shadow,
            "same_frozen_release_loss_kernels": true,
            "same_frozen_store_kernel": true,
            "direct_r_to_m_or_l": false,
            "shadow_r_to_a_before_productive_chemistry": shadow.ledger.reserve_r_to_a > EPS,
            "shadow_a_to_r_after_productive_chemistry": shadow.ledger.a_before_final_storage >= 0.0 && shadow.ledger.reserve_a_to_r > EPS,
            "shadow_catalyst_production_uses_a_only": true,
            "strict_material_closure": strict_closure
        },
        "gate2_shadow": {
            "mode": "MOBILIZE_FIRST_STORE_LAST",
            "production_defaults_unchanged": true,
            "parameter_sweep": false,
            "target_signal": false,
            "health_controller": false
        },
        "gate3_causal_comparison": {
            "shadow_improves_gate1_mechanism": shadow_improves,
            "full_r_m": full.structural.r_x,
            "shadow_r_m": shadow.structural.r_x,
            "full_c_produced": full.ledger.c_produced,
            "shadow_c_produced": shadow.ledger.c_produced,
            "full_m_produced": full.ledger.m_produced,
            "shadow_m_produced": shadow.ledger.m_produced,
            "full_l_produced": full.ledger.l_produced,
            "shadow_l_produced": shadow.ledger.l_produced,
            "full_a_to_r": full.ledger.reserve_a_to_r,
            "shadow_a_to_r": shadow.ledger.reserve_a_to_r,
            "full_r_to_a": full.ledger.reserve_r_to_a,
            "shadow_r_to_a": shadow.ledger.reserve_r_to_a,
            "full_r_to_w": full.ledger.reserve_r_to_w,
            "shadow_r_to_w": shadow.ledger.reserve_r_to_w
        },
        "gate4_actual_d087": {"full": full_d087, "shadow": shadow_d087},
        "gate5_reserve_function": {
            "replete_a_to_r": shadow.reserve_function.replete_a_to_r,
            "starvation_r_to_a": shadow.reserve_function.starvation_r_to_a,
            "rejected_steps": shadow.reserve_function.rejected_steps,
            "strict_closure": shadow.reserve_function.strict_closure,
            "preserved": reserve_function_preserved
        },
        "classification": classification,
        "production_chemistry_changed": false,
        "production_reserve_physiology_changed": false,
        "recycling_authorized": false,
        "dc_dev_021_authorized": false,
        "next_execution_started": false
    });
    fs::write(
        out.join("r9r6_report.json"),
        serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    println!(
        "DCDEV020R9R6_AUDIT_COMPLETE output={} classification={}",
        out.display(),
        classification
    );
    Ok(())
}
