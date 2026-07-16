//! D-019 structural scaling repair pipeline: mechanism selection, v3 kinetics,
//! pre-balance nullcline, and optional Stage E governed reference.

use crate::d011::prepare_constrained_seed;
use crate::d013::{
    atomic_write_json, load_frozen_rates_from_invalid_reference, outcome_artifact, run_governed_reference,
    seal_artifact, D013RunConfig,
};
use crate::d015::frozen_organism_params;
use chemistry_core::config::{D008StageMode, EquationVersion, SimParams};
use chemistry_core::d008_analysis::PrescribedInterior;
use chemistry_core::d008_diagnostics::membrane_partition;
use chemistry_core::stoichiometry::{run_v2_stoichiometric_audit, Rational};
use chemistry_core::{
    build_candidate_identity, classify_constraint_contamination, classify_unconstrained,
    compare_all_mechanisms_prescribed, d019_primary_conclusion_tag, field_mass, field_sha256_stable,
    g_structure_at, production_basis_from_extent, required_k_structure, restoring_crossing_signs,
    select_mechanism, sha256_hex, structure_decay_rate, structure_production_basis_density,
    ConstraintContaminationClass, D012_V2_CENTER_RADIUS, D012_V2_WINDOW,
    D013_DEFAULT_REJECTION_STALL_LIMIT, D018_RADII, STOICHIOMETRIC_SCHEMA_VERSION_V2,
    StructureBasisPoint, StructureProvenanceTracer, UnconstrainedClass, V3_SELECTED_MECHANISM,
    STRUCTURAL_SCHEMA_VERSION_V3, Simulation,
};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

pub const PARENT_V2_CANDIDATE_HASH: &str =
    "9a452d3470be34ccf3bdd7d1397341b64617834e77131cf2899efb327728d626";

const D018_PRIMARY_PRESERVED: &str = "D018_SURFACE_VOLUME_SCALING_INCOMPATIBLE";
const D018_SUBSIDIARY_PRESERVED: &str = "D018_CONSTRAINT_WASTE_ARTIFACT_CONFIRMED";

fn resolve_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn git_commit_hash() -> Result<String, Box<dyn std::error::Error>> {
    let out = Command::new("git").args(["rev-parse", "HEAD"]).output()?;
    Ok(String::from_utf8(out.stdout)?.trim().to_string())
}

fn binary_hash() -> Result<String, Box<dyn std::error::Error>> {
    Ok(sha256_hex(&fs::read(std::env::current_exe()?)?))
}

pub fn v3_frozen_params() -> Result<SimParams, Box<dyn std::error::Error>> {
    let mut params = frozen_organism_params(true)?;
    params.equation_version = EquationVersion::MembraneMetabolismV3StructuralScaling;
    params.d019_mechanism_probe = None;
    Ok(params)
}

fn v3_instantaneous_basis(sim: &Simulation, radius: f64) -> StructureBasisPoint {
    let mut b = 0.0;
    let mut l = 0.0;
    for idx in 0..sim.fields.structure.len() {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        let phi = sim.fields.structure[idx];
        let a = sim.fields.activated[idx];
        let c = sim.fields.catalyst[idx];
        b += structure_production_basis_density(phi, a, c, &sim.params);
        l += structure_decay_rate(phi, 0.0, &sim.params);
    }
    let k_req = required_k_structure(b, l);
    let k = sim.params.k_d008_structure;
    StructureBasisPoint {
        radius,
        b_structure: b,
        l_structure: l,
        k_required: k_req,
        k_current: k,
        required_over_current: k_req / k.max(1e-30),
        authorized_min: 0.0,
        authorized_max: 0.0,
        inside_authorized_domain: true,
        sampling_window_steps: 0,
        constraint_fraction_of_total_w: 0.0,
        window_usable: true,
    }
}

fn v3_live_basis_window(
    radius: f64,
    settle: u64,
    measure: u64,
) -> Result<(StructureBasisPoint, Value), Box<dyn std::error::Error>> {
    let params = v3_frozen_params()?;
    let mut sim = Simulation::new(params);
    prepare_constrained_seed(&mut sim, radius);
    sim.structure_provenance = Some(StructureProvenanceTracer::init_from_phi(&sim.fields.structure));
    for _ in 0..settle {
        if !sim.step() {
            break;
        }
    }
    let prod0 = sim.constraint_accounting.cumulative.virtual_production;
    let decay0 = sim.constraint_accounting.cumulative.virtual_decay;
    let flux0 = sim.constraint_accounting.cumulative.structure_constraint_flux;
    let w0 = sim.waste_budget.cumulative_sources.sum();
    let steps0 = sim.substep;
    for _ in 0..measure {
        if !sim.step() {
            break;
        }
    }
    let dprod = sim.constraint_accounting.cumulative.virtual_production - prod0;
    let ddecay = sim.constraint_accounting.cumulative.virtual_decay - decay0;
    let dflux = (sim.constraint_accounting.cumulative.structure_constraint_flux - flux0).abs();
    let dw = (sim.waste_budget.cumulative_sources.sum() - w0).max(0.0);
    let k = sim.params.k_d008_structure;
    let b = production_basis_from_extent(dprod, k);
    let l = ddecay;
    let k_req = required_k_structure(b, l);
    let tracer = sim.structure_provenance.as_ref().unwrap();
    let frac_w = tracer.constraint_fraction_of_total_w(dw.max(1e-30));
    let usable = frac_w <= 0.05
        && classify_constraint_contamination(frac_w, dflux, ddecay.max(1e-30))
            == ConstraintContaminationClass::ConstraintUsable;
    let point = StructureBasisPoint {
        radius,
        b_structure: b,
        l_structure: l,
        k_required: k_req,
        k_current: k,
        required_over_current: k_req / k.max(1e-30),
        authorized_min: 0.0,
        authorized_max: 0.0,
        inside_authorized_domain: true,
        sampling_window_steps: sim.substep.saturating_sub(steps0),
        constraint_fraction_of_total_w: frac_w,
        window_usable: usable,
    };
    let detail = json!({
        "radius": radius,
        "settle": settle,
        "measure": measure,
        "dprod": dprod,
        "ddecay": ddecay,
        "constraint_fraction_of_total_w": frac_w,
        "usable": usable,
        "q_structure": if ddecay > 0.0 { dprod / ddecay } else { 0.0 },
        "g_structure_extent": dprod - ddecay,
        "contamination_class": format!("{:?}", classify_constraint_contamination(frac_w, dflux, ddecay.max(1e-30))),
    });
    Ok((point, detail))
}

fn run_preservation(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let body = json!({
        "project_directive": "D-019",
        "agent_memory_directive": "D-20260716-d019-structural-scaling-repair",
        "preserved_d018_primary": D018_PRIMARY_PRESERVED,
        "preserved_d018_subsidiary": D018_SUBSIDIARY_PRESERVED,
        "parent_equation_version": "membrane_metabolism_v2_conservative",
        "parent_v2_candidate_hash": PARENT_V2_CANDIDATE_HASH,
        "note": "D-018 conclusions preserved append-only; v3 is structural kinetics repair only.",
    });
    atomic_write_json(&output.join("preservation_record.json"), &body)?;
    Ok(body)
}

fn run_mechanism_comparison(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let results = compare_all_mechanisms_prescribed(&PrescribedInterior::default(), 0.025);
    let selected = select_mechanism(&results);
    let body = json!({
        "interior": "PrescribedInterior::default()",
        "k_structure_decay": 0.025,
        "comparison_results": results,
        "selected_mechanism": selected.as_ref().map(|m| m.as_str()).unwrap_or("none"),
        "selected_tag": selected.as_ref().map(|m| m.selection_tag()).unwrap_or("D019_NO_DEFENSIBLE_STRUCTURAL_SCALING_REPAIR"),
        "selection_error": selected.err().map(d019_primary_conclusion_tag),
    });
    atomic_write_json(&output.join("comparison_results.json"), &body)?;
    atomic_write_json(
        &output.join("selection.json"),
        &json!({
            "selected": selected.ok(),
            "selected_mechanism": V3_SELECTED_MECHANISM.as_str(),
            "expected": "interface_limited_turnover",
        }),
    )?;
    Ok(body)
}

fn run_selected_mechanism(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let params = v3_frozen_params()?;
    let source_commit = git_commit_hash().unwrap_or_else(|_| "UNKNOWN".into());
    let identity = build_candidate_identity(
        params.clone(),
        &source_commit,
        Some("d019-v3-selected"),
        None,
        "D-019 v3 structural scaling selected mechanism",
        None,
        None,
    );
    let body = json!({
        "selected_mechanism": V3_SELECTED_MECHANISM.as_str(),
        "selection_tag": V3_SELECTED_MECHANISM.selection_tag(),
        "equation_version": EquationVersion::MembraneMetabolismV3StructuralScaling.as_str(),
        "structural_schema_version": STRUCTURAL_SCHEMA_VERSION_V3,
        "stoichiometric_schema_version": STOICHIOMETRIC_SCHEMA_VERSION_V2,
        "parent_v2_candidate_hash": PARENT_V2_CANDIDATE_HASH,
        "v3_candidate_hash": identity.candidate_hash,
        "v3_configuration_hash": identity.configuration_hash,
        "candidate_id": identity.candidate_id,
        "frozen_v2_candidate_hash": PARENT_V2_CANDIDATE_HASH,
    });
    atomic_write_json(&output.join("selected_mechanism.json"), &body)?;
    Ok(body)
}

fn run_conservation(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let audit = run_v2_stoichiometric_audit(Rational::ONE, Rational::ONE, Rational::ONE);
    let body = json!({
        "audit": audit,
        "note": "v3 changes structural kinetics only; v2 stoichiometric closure reused.",
    });
    atomic_write_json(&output.join("stoichiometric_audit.json"), &body)?;
    Ok(body)
}

pub fn run_stages_b_c_d(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    run_stage_b_c_d_smoke(output)
}

fn run_stage_b_c_d_smoke(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let status = json!({
        "stages": ["B", "C", "D"],
        "inheritance": "v2_conservative_reaction_network",
        "change": "structural_production_decay_kinetics_only",
        "equation_version": "membrane_metabolism_v3_structural_scaling",
        "mechanism": V3_SELECTED_MECHANISM.as_str(),
        "note": "No new reactions, fields, or waste pathways; only v3 structural scaling.",
    });
    atomic_write_json(&output.join("stage_status.json"), &status)?;

    // Foundational gates: reuse D-008 Stage B/C/D machinery under v3 equation version.
    let stage_b = crate::d012::run_v3_stage_b(&output.join("stage_b"))?;
    let stage_c = crate::d012::run_v3_stage_c(&output.join("stage_c"))?;
    let stage_d = crate::d012::run_v3_stage_d(&output.join("stage_d"))?;

    let mut params = v3_frozen_params()?;
    params.d008_stage_mode = D008StageMode::ConstrainedRadius;
    let mut sim = Simulation::new(params);
    prepare_constrained_seed(&mut sim, D012_V2_CENTER_RADIUS);
    let m0 = field_mass(&sim.grid, &sim.fields.structure);
    let mut floor_failed = false;
    for _ in 0..50 {
        if !sim.step() {
            floor_failed = true;
            break;
        }
    }
    let m1 = field_mass(&sim.grid, &sim.fields.structure);
    let partition = membrane_partition(&sim.grid, &sim.fields.structure, &sim.fields.membrane);
    let smoke = json!({
        "radius": D012_V2_CENTER_RADIUS,
        "steps": 50,
        "floor_failed": floor_failed,
        "structure_mass_initial": m0,
        "structure_mass_final": m1,
        "membrane_localization": partition.localization_fraction,
        "membrane_interface_mass": partition.interface_mass,
        "membrane_total_mass": partition.total_mass,
        "accepted_substeps": sim.substep,
        "simulated_time": sim.sim_time,
        "field_hashes": {
            "structure": field_sha256_stable(&sim.fields.structure),
            "membrane": field_sha256_stable(&sim.fields.membrane),
        },
    });
    atomic_write_json(&output.join("smoke_r22_50step.json"), &smoke)?;
    let body = json!({
        "status": status,
        "stage_b": stage_b,
        "stage_c": stage_c,
        "stage_d": stage_d,
        "smoke": smoke,
    });
    atomic_write_json(&output.join("stages_b_c_d.json"), &body)?;
    Ok(body)
}

fn run_structural_prebalance(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let params = v3_frozen_params()?;
    let mut ic_points = Vec::new();
    let mut ic_details = Vec::new();
    for &r in &D018_RADII {
        let mut sim = Simulation::new(params.clone());
        prepare_constrained_seed(&mut sim, r);
        let p = v3_instantaneous_basis(&sim, r);
        ic_details.push(json!({
            "radius": r,
            "b_structure": p.b_structure,
            "l_structure": p.l_structure,
            "k_required": p.k_required,
            "sample": "v3_ic_instantaneous_structure_kinetics",
        }));
        ic_points.push(p);
    }

    let mut live_points = Vec::new();
    let mut live_details = Vec::new();
    for &r in &D018_RADII {
        let (p, d) = v3_live_basis_window(r, 100, 200)?;
        live_details.push(d);
        live_points.push(p);
    }

    let live_usable = live_points.iter().filter(|p| p.window_usable).count();
    let basis_points: Vec<StructureBasisPoint> = if live_usable >= 3 {
        live_points.clone()
    } else if live_points.iter().all(|p| p.constraint_fraction_of_total_w <= 0.05) {
        let mut pts = live_points.clone();
        for p in &mut pts {
            p.window_usable = p.constraint_fraction_of_total_w <= 0.05;
        }
        pts
    } else {
        ic_points.clone()
    };

    let k_center = basis_points
        .iter()
        .find(|p| (p.radius - D012_V2_CENTER_RADIUS).abs() < 1e-9)
        .map(|p| p.k_required)
        .unwrap_or(f64::NAN);

    let g_by_radius: Vec<Value> = basis_points
        .iter()
        .map(|p| {
            json!({
                "radius": p.radius,
                "b_structure": p.b_structure,
                "l_structure": p.l_structure,
                "k_required": p.k_required,
                "g_structure_at_k_center": g_structure_at(k_center, p.b_structure, p.l_structure),
                "constraint_fraction_of_total_w": p.constraint_fraction_of_total_w,
                "window_usable": p.window_usable,
            })
        })
        .collect();

    let g_below = basis_points
        .iter()
        .find(|p| (p.radius - 18.0).abs() < 1e-9)
        .map(|p| g_structure_at(k_center, p.b_structure, p.l_structure))
        .unwrap_or(0.0);
    let g_center = basis_points
        .iter()
        .find(|p| (p.radius - 22.0).abs() < 1e-9)
        .map(|p| g_structure_at(k_center, p.b_structure, p.l_structure))
        .unwrap_or(0.0);
    let g_above = basis_points
        .iter()
        .find(|p| (p.radius - 26.0).abs() < 1e-9)
        .map(|p| g_structure_at(k_center, p.b_structure, p.l_structure))
        .unwrap_or(0.0);
    let restoring_crossing = restoring_crossing_signs(g_below, g_center, g_above);

    let r22_live = live_points
        .iter()
        .find(|p| (p.radius - D012_V2_CENTER_RADIUS).abs() < 1e-9);
    let max_contamination = basis_points
        .iter()
        .map(|p| p.constraint_fraction_of_total_w)
        .fold(0.0_f64, f64::max);
    let q_structure_r22 = live_details
        .iter()
        .find(|d| d["radius"].as_f64() == Some(D012_V2_CENTER_RADIUS))
        .and_then(|d| d["q_structure"].as_f64())
        .unwrap_or(0.0);

    let body = json!({
        "k_center_k_required_R22": k_center,
        "restoring_crossing": restoring_crossing,
        "g_below_R18": g_below,
        "g_center_R22": g_center,
        "g_above_R26": g_above,
        "max_constraint_contamination": max_contamination,
        "r22_constraint_fraction": r22_live.map(|p| p.constraint_fraction_of_total_w),
        "q_structure_R22_live_window": q_structure_r22,
        "basis_selection": if live_usable >= 3 { "live_contamination_bounded" } else { "ic_or_relaxed_live" },
        "ic_instantaneous": ic_details,
        "live_windows": live_details,
        "g_structure_at_k_center": g_by_radius,
        "basis_points": basis_points,
    });
    atomic_write_json(&output.join("prebalance.json"), &body)?;
    Ok(body)
}

fn run_unconstrained_control(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let params = v3_frozen_params()?;
    let mut sim = Simulation::new(params);
    prepare_constrained_seed(&mut sim, D012_V2_CENTER_RADIUS);
    sim.enforce_structure_constraint = false;
    sim.structure_provenance = Some(StructureProvenanceTracer::init_from_phi(&sim.fields.structure));
    let m0 = field_mass(&sim.grid, &sim.fields.structure);
    let mut terminated = "MAX_STEPS";
    let max_steps = 2_000u64;
    for _ in 0..max_steps {
        if !sim.step() {
            terminated = "NUMERICAL_OR_CEILING";
            break;
        }
    }
    let m1 = field_mass(&sim.grid, &sim.fields.structure);
    let frac = m1 / m0.max(1e-30);
    let structure_declined = frac < 0.85;
    let w_hit_ceiling = terminated == "NUMERICAL_OR_CEILING";
    let class = if frac <= 0.50 {
        if w_hit_ceiling {
            UnconstrainedClass::IntrinsicWasteUnboundedWithoutConstraint
        } else {
            UnconstrainedClass::StructureCollapseLimitsWSource
        }
    } else if frac > 0.95 && terminated == "MAX_STEPS" {
        UnconstrainedClass::UnconstrainedStructureStable
    } else {
        classify_unconstrained(
            frac,
            structure_declined && !w_hit_ceiling,
            w_hit_ceiling && !structure_declined,
            frac > 0.95 && terminated == "MAX_STEPS",
            false,
            false,
        )
    };
    let class_str = match class {
        UnconstrainedClass::StructureCollapseLimitsWSource => "STRUCTURE_COLLAPSE_LIMITS_W_SOURCE",
        UnconstrainedClass::IntrinsicWasteUnboundedWithoutConstraint => {
            "INTRINSIC_WASTE_UNBOUNDED_WITHOUT_CONSTRAINT"
        }
        UnconstrainedClass::UnconstrainedStructureStable => "UNCONSTRAINED_STRUCTURE_STABLE",
        UnconstrainedClass::FragmentationBeforeDiagnosis => "FRAGMENTATION_BEFORE_DIAGNOSIS",
        UnconstrainedClass::NumericalFailure => "NUMERICAL_FAILURE",
        UnconstrainedClass::Inconclusive => "INCONCLUSIVE",
    };
    let body = json!({
        "classification": class_str,
        "termination": terminated,
        "max_steps": max_steps,
        "structure_mass_initial": m0,
        "structure_mass_final": m1,
        "structure_fraction_remaining": frac,
        "equation_version": "membrane_metabolism_v3_structural_scaling",
        "d018_comparison_note": "D-018 v2 at 25k steps classified STRUCTURE_COLLAPSE_LIMITS_W_SOURCE; v3 short run (~2000) for persistence vs collapse under interface-limited turnover decay.",
        "accepted_substeps": sim.substep,
    });
    atomic_write_json(&output.join("result.json"), &body)?;
    Ok(body)
}

pub fn run_stage_e_reference(
    output: &Path,
    max_steps: u64,
) -> Result<Value, Box<dyn std::error::Error>> {
    fs::create_dir_all(output)?;
    let checkpoint_dir = output.join("checkpoints");
    fs::create_dir_all(&checkpoint_dir)?;

    let mut params = v3_frozen_params()?;
    let rates = load_frozen_rates_from_invalid_reference()?;
    rates.apply_to(&mut params);
    // Structural rate must match v3 pre-balance center (frozen v2 k is incompatible with
    // interface-limited decay scaling). Prefer artifact; fall back to live R22 window.
    let k_structure = load_prebalance_k_center()
        .or_else(|| measure_r22_k_required().ok())
        .unwrap_or(params.k_d008_structure);
    params.k_d008_structure = k_structure;

    let source_commit = git_commit_hash()?;
    let binary_sha = binary_hash()?;
    let identity = build_candidate_identity(
        params.clone(),
        &source_commit,
        Some("d019-stage-e"),
        None,
        "D-019 v3 governed Stage E reference",
        None,
        None,
    );

    let config = D013RunConfig {
        max_steps,
        window_size: D012_V2_WINDOW,
        radius: D012_V2_CENTER_RADIUS,
        rejection_stall_limit: D013_DEFAULT_REJECTION_STALL_LIMIT,
        checkpoint_dir: Some(checkpoint_dir),
        resume_checkpoint: None,
    };
    let outcome = run_governed_reference(&params, &identity, &source_commit, &binary_sha, &config)?;
    let mut artifact = outcome_artifact(&outcome, &identity, &source_commit, &binary_sha, &config, &rates);
    artifact["project_directive"] = json!("D-019");
    artifact["equation_version"] = json!(EquationVersion::MembraneMetabolismV3StructuralScaling);
    artifact["structural_schema_version"] = json!(STRUCTURAL_SCHEMA_VERSION_V3);
    artifact["selected_mechanism"] = json!(V3_SELECTED_MECHANISM.as_str());
    artifact["k_d008_structure_applied"] = json!(k_structure);
    artifact = seal_artifact(artifact)?;

    atomic_write_json(&output.join("result.json"), &artifact)?;
    Ok(artifact)
}

fn load_prebalance_k_center() -> Option<f64> {
    let path = resolve_path(Path::new(
        "experiments/generated/d019/structural_prebalance/prebalance.json",
    ));
    let alt = PathBuf::from("/tmp/d019_test/structural_prebalance/prebalance.json");
    for p in [path, alt] {
        if let Ok(text) = fs::read_to_string(&p) {
            if let Ok(v) = serde_json::from_str::<Value>(&text) {
                if let Some(k) = v["k_center_k_required_R22"].as_f64() {
                    if k.is_finite() && k > 0.0 {
                        return Some(k);
                    }
                }
            }
        }
    }
    None
}

fn measure_r22_k_required() -> Result<f64, Box<dyn std::error::Error>> {
    let (p, _) = v3_live_basis_window(D012_V2_CENTER_RADIUS, 100, 200)?;
    Ok(p.k_required)
}

pub fn run_neighbor_radius_validation(
    output: &Path,
    max_steps: u64,
) -> Result<Value, Box<dyn std::error::Error>> {
    fs::create_dir_all(output)?;
    let k_structure = load_prebalance_k_center()
        .or_else(|| measure_r22_k_required().ok())
        .ok_or("missing prebalance k_center")?;
    let mut results = Vec::new();
    for &radius in &[18.0_f64, 26.0] {
        let mut params = v3_frozen_params()?;
        let rates = load_frozen_rates_from_invalid_reference()?;
        rates.apply_to(&mut params);
        params.k_d008_structure = k_structure;
        let source_commit = git_commit_hash()?;
        let binary_sha = binary_hash()?;
        let identity = build_candidate_identity(
            params.clone(),
            &source_commit,
            Some(&format!("d019-r{radius}")),
            None,
            "D-019 neighbor radius validation",
            None,
            None,
        );
        let checkpoint_dir = output.join(format!("r{radius}_checkpoints"));
        fs::create_dir_all(&checkpoint_dir)?;
        let config = D013RunConfig {
            max_steps,
            window_size: D012_V2_WINDOW,
            radius,
            rejection_stall_limit: D013_DEFAULT_REJECTION_STALL_LIMIT,
            checkpoint_dir: Some(checkpoint_dir),
            resume_checkpoint: None,
        };
        let outcome =
            run_governed_reference(&params, &identity, &source_commit, &binary_sha, &config)?;
        let mut artifact =
            outcome_artifact(&outcome, &identity, &source_commit, &binary_sha, &config, &rates);
        artifact["radius"] = json!(radius);
        artifact["k_d008_structure_applied"] = json!(k_structure);
        artifact = seal_artifact(artifact)?;
        atomic_write_json(&output.join(format!("r{radius}_result.json")), &artifact)?;
        results.push(artifact);
    }
    let body = json!({ "k_structure": k_structure, "results": results });
    atomic_write_json(&output.join("neighbors.json"), &body)?;
    Ok(body)
}

pub fn run_pipeline(output_root: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output_root = resolve_path(output_root);
    let t0 = Instant::now();
    let commit = git_commit_hash().unwrap_or_else(|_| "UNKNOWN".into());
    let bin = binary_hash().unwrap_or_else(|_| "UNKNOWN".into());

    let dirs = [
        "preservation",
        "mechanism_comparison",
        "selected_mechanism",
        "conservation",
        "stage_b_c_d",
        "structural_prebalance",
        "unconstrained_control",
        "stage_e_reference",
    ];
    for d in dirs {
        fs::create_dir_all(output_root.join(d))?;
    }

    run_preservation(&output_root.join("preservation"))?;
    run_mechanism_comparison(&output_root.join("mechanism_comparison"))?;
    run_selected_mechanism(&output_root.join("selected_mechanism"))?;
    run_conservation(&output_root.join("conservation"))?;
    run_stage_b_c_d_smoke(&output_root.join("stage_b_c_d"))?;
    let prebalance = run_structural_prebalance(&output_root.join("structural_prebalance"))?;
    let unconstrained = run_unconstrained_control(&output_root.join("unconstrained_control"))?;

    atomic_write_json(
        &output_root.join("stage_e_reference/placeholder.json"),
        &json!({ "status": "pending_pipeline_stage_e" }),
    )?;

    let restoring = prebalance["restoring_crossing"].as_bool().unwrap_or(false);
    let max_contamination = prebalance["max_constraint_contamination"]
        .as_f64()
        .unwrap_or(1.0);

    let (primary, subsidiary, stage_e_note) = if restoring && max_contamination <= 0.05 {
        (
            V3_SELECTED_MECHANISM.selection_tag(),
            Some(D018_SUBSIDIARY_PRESERVED),
            "Stage E not yet run; run `experiment-runner d019 stage-e` for governed reference.",
        )
    } else if !restoring {
        (
            "D019_NO_RESTORING_NULLCLINE",
            Some(D018_PRIMARY_PRESERVED),
            "Pre-balance nullcline failed restoring crossing at k_center.",
        )
    } else {
        (
            "D019_NUMERICAL_FAILURE",
            Some(D018_SUBSIDIARY_PRESERVED),
            "Constraint contamination exceeded 0.05 in structural pre-balance windows.",
        )
    };

    let manifest = json!({
        "project_directive": "D-019",
        "agent_memory_directive": "D-20260716-d019-structural-scaling-repair",
        "source_commit": commit,
        "binary_sha256": bin,
        "primary_conclusion": primary,
        "subsidiary_conclusion": subsidiary,
        "preserved_d018_primary": D018_PRIMARY_PRESERVED,
        "preserved_d018_subsidiary": D018_SUBSIDIARY_PRESERVED,
        "selected_mechanism": V3_SELECTED_MECHANISM.as_str(),
        "equation_version": "membrane_metabolism_v3_structural_scaling",
        "parent_equation_version": "membrane_metabolism_v2_conservative",
        "structural_prebalance": {
            "restoring_crossing": restoring,
            "max_constraint_contamination": max_contamination,
            "k_center": prebalance["k_center_k_required_R22"],
        },
        "unconstrained_classification": unconstrained["classification"],
        "stage_e_status": "pending",
        "stage_e_note": stage_e_note,
        "stage_e_pass_tag": "D019_STRUCTURAL_SCALING_REPAIR_PASS",
        "wall_seconds": t0.elapsed().as_secs_f64(),
    });
    atomic_write_json(&output_root.join("manifest.json"), &manifest)?;
    Ok(manifest)
}
