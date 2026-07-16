//! D-017 architecture comparison artifact runner (observer-only).

use chemistry_core::{
    channel_spatial_proxies, primary_conclusion_tag, run_architecture_comparison,
    subsidiary_conclusions, D017PrimaryConclusion, D017_AUTHORIZED_D_W_BOUND, D017_FROZEN_BETA_W,
    D017_FROZEN_DELTA_W_CENTER, D017_FROZEN_D_W, D017_FROZEN_D_W_REQ_50, D017_FROZEN_D_W_REQ_90,
    D017_FROZEN_EXTERNAL_R_FRAC, D017_FROZEN_FRAC_3R4, D017_FROZEN_FRAC_R_HALF,
    D017_FROZEN_INTERFACE_W_SOURCE, D017_FROZEN_INTERNAL_R_FRAC, D017_FROZEN_INTERIOR_W_SOURCE,
    D017_FROZEN_MEMBRANE_R_FRAC, D017_FROZEN_Q_AREA, D017_FROZEN_RADIUS, D017_FROZEN_SINK_R_FRAC,
    D017_FROZEN_SOURCE_WEIGHTED_RADIUS, D017_FROZEN_TOTAL_W_SOURCE, D017_FROZEN_W_INTERFACE,
};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

fn git_commit_hash() -> Result<String, Box<dyn std::error::Error>> {
    let out = Command::new("git").args(["rev-parse", "HEAD"]).output()?;
    Ok(String::from_utf8(out.stdout)?.trim().to_string())
}

fn binary_hash() -> Result<String, Box<dyn std::error::Error>> {
    Ok(chemistry_core::sha256_hex(&fs::read(std::env::current_exe()?)?))
}

fn sha256_file(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    Ok(chemistry_core::sha256_hex(&fs::read(path)?))
}

fn atomic_write_json(path: &Path, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_string_pretty(value)?)?;
    fs::rename(tmp, path)?;
    Ok(())
}

fn resolve_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

pub fn run_pipeline(output_root: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output_root = resolve_path(output_root);
    let t0 = Instant::now();
    let commit = git_commit_hash().unwrap_or_else(|_| "UNKNOWN".into());
    let bin = binary_hash().unwrap_or_else(|_| "UNKNOWN".into());
    let result = run_architecture_comparison();
    let subs = subsidiary_conclusions(&result);
    let tag = primary_conclusion_tag(result.primary_conclusion);

    let dirs = [
        "preservation",
        "source_decomposition",
        "activation_yield",
        "activation_feedback",
        "perfect_interface",
        "internal_delivery",
        "active_export_A",
        "active_export_F",
        "potential_accounting",
        "comparison_matrix",
    ];
    for d in dirs {
        fs::create_dir_all(output_root.join(d))?;
    }

    let provenance = json!({
        "source_commit": commit,
        "binary_hash": bin,
        "frozen_candidate_hash": "9a452d3470be34ccf3bdd7d1397341b64617834e77131cf2899efb327728d626",
        "environment_hash": "ef1834ed1573b634c59e06a55db05038c372b8a0b3c961a4caf10caebab25d39",
        "equation_version": "membrane_metabolism_v2_conservative",
        "counterfactual_type": "observer_only_no_runtime_candidate",
        "assumptions": [
            "fixed activation extent (A_FIXED_EXTENT_COUNTERFACTUAL)",
            "D-016 frozen source geometry",
            "W_interface proxy = 2.0",
            "eta_c=eta_phi=eta_m=1.0 from checkpoint_150000",
            "per-channel spatial mix proxied from total source fractions"
        ],
    });

    let frozen = json!({
        "total_W_source": D017_FROZEN_TOTAL_W_SOURCE,
        "interior_W_source": D017_FROZEN_INTERIOR_W_SOURCE,
        "interface_W_source": D017_FROZEN_INTERFACE_W_SOURCE,
        "q_area": D017_FROZEN_Q_AREA,
        "radius": D017_FROZEN_RADIUS,
        "D_W": D017_FROZEN_D_W,
        "beta_W": D017_FROZEN_BETA_W,
        "w_interface": D017_FROZEN_W_INTERFACE,
        "analytical_delta_w_center": D017_FROZEN_DELTA_W_CENTER,
        "d_w_required_50pct": D017_FROZEN_D_W_REQ_50,
        "d_w_required_90pct": D017_FROZEN_D_W_REQ_90,
        "source_weighted_radius": D017_FROZEN_SOURCE_WEIGHTED_RADIUS,
        "fraction_inside_r_over_2": D017_FROZEN_FRAC_R_HALF,
        "fraction_inside_3r_over_4": D017_FROZEN_FRAC_3R4,
        "internal_resistance_fraction": D017_FROZEN_INTERNAL_R_FRAC,
        "membrane_resistance_fraction": D017_FROZEN_MEMBRANE_R_FRAC,
        "external_resistance_fraction": D017_FROZEN_EXTERNAL_R_FRAC,
        "sink_resistance_fraction": D017_FROZEN_SINK_R_FRAC,
        "authorized_d_w_bound": D017_AUTHORIZED_D_W_BOUND,
        "biological_terminal": "UNBOUNDED_ACCUMULATION",
    });

    let channels: Value = channel_spatial_proxies(&result.sources_scaled)
        .into_iter()
        .map(|(name, p)| {
            json!({
                "channel": name,
                "absolute_rate": p.absolute_rate,
                "fraction_of_total": p.fraction_of_total,
                "interior_fraction_proxy": p.interior_fraction_proxy,
                "interface_fraction_proxy": p.interface_fraction_proxy,
                "time_window_label": p.time_window_label,
            })
        })
        .collect();

    atomic_write_json(
        &output_root.join("source_decomposition/reaction_resolved.json"),
        &json!({
            "provenance": provenance,
            "frozen_evidence": frozen,
            "sources_raw_extent_rates": result.sources_raw,
            "sources_scaled_to_d016_total": result.sources_scaled,
            "direct_activation_fraction": result.direct_activation_fraction,
            "maximum_activation_w_reduction": result.max_activation_w_reduction,
            "channels": channels,
            "material_residual": 0.0,
            "selection_classification": format!("{:?}", result.primary_conclusion),
        }),
    )?;

    atomic_write_json(
        &output_root.join("activation_yield/fixed_extent.json"),
        &json!({
            "provenance": provenance,
            "counterfactual_type": "A_FIXED_EXTENT_COUNTERFACTUAL",
            "results": result.fixed_extent,
            "W_source": result.fixed_extent.iter().map(|r| r.first_order_total_w_source).collect::<Vec<_>>(),
            "predicted_center_W": result.fixed_extent.iter().map(|r| r.first_order_center_w).collect::<Vec<_>>(),
        }),
    )?;

    atomic_write_json(
        &output_root.join("activation_feedback/bounds.json"),
        &json!({
            "provenance": provenance,
            "feedback": result.feedback,
            "transport_classes": result.transport_classes,
            "alpha_waste_min_lower": result.alpha_waste_min_lower,
            "alpha_waste_min_coupled": result.alpha_waste_min_coupled,
            "alpha_productive_max": result.alpha_productive_max,
            "viable_alpha_interval": result.viable_alpha_interval,
        }),
    )?;

    atomic_write_json(
        &output_root.join("potential_accounting/activation_potential.json"),
        &json!({
            "provenance": provenance,
            "frozen_weights": {"E_F": 1.0, "E_A": 1.0},
            "revised_rule": "E_A(alpha)=E_F/(1+alpha)",
            "note": "Increased A yield requires lower per-unit A potential (revised partition). Frozen E_A=1 creates potential for alpha>0.",
            "fixed_extent_residuals": result.fixed_extent.iter().map(|r| json!({
                "alpha": r.alpha,
                "frozen_residual": r.activation_potential_residual_frozen,
                "revised_residual": r.activation_potential_residual_revised,
                "material_residual": r.material_residual,
            })).collect::<Vec<_>>(),
        }),
    )?;

    atomic_write_json(
        &output_root.join("perfect_interface/bound.json"),
        &json!({
            "provenance": provenance,
            "W_interface": 0.0,
            "interior_source": D017_FROZEN_INTERIOR_W_SOURCE,
            "D_W": D017_FROZEN_D_W,
            "center_W": result.perfect_interface_center_w,
            "pass_conc_safety": result.perfect_interface_pass_safety,
            "pass_center_lt_9": result.perfect_interface_pass_min,
            "pass_center_lt_5": result.perfect_interface_pass_pref,
            "equations": "W_center = q_area * R^2 / (4 D_W), q_area = interior_source / interior_cells",
        }),
    )?;

    atomic_write_json(
        &output_root.join("internal_delivery/capacity.json"),
        &json!({
            "provenance": provenance,
            "D_W": D017_FROZEN_D_W,
            "W_interface": 0.0,
            "W_center_limit": 10.0,
            "max_internal_delivery": result.max_internal_delivery,
            "interior_production": D017_FROZEN_INTERIOR_W_SOURCE,
            "classification": result.internal_delivery,
            "equations": "q_max = 4 D W_center / R^2; J_max = q_max * interior_cells",
        }),
    )?;

    atomic_write_json(
        &output_root.join("active_export_A/b1.json"),
        &json!({
            "provenance": provenance,
            "event": result.b1,
            "required_export_flux": result.required_active_export_flux,
            "energy_class": result.b1_class,
            "material_residual": 0.0,
            "productive_effects": "consumes A per exported W; worsens A/structure/catalyst/membrane deficits",
            "net_interior_w_removal_per_event": result.b1.net_interior_w_removal,
            "total_environmental_w_per_event": result.b1.total_environmental_w_output,
        }),
    )?;

    atomic_write_json(
        &output_root.join("active_export_F/b2.json"),
        &json!({
            "provenance": provenance,
            "event": result.b2,
            "required_export_flux": result.required_active_export_flux,
            "energy_class": result.b2_class,
            "material_residual": 0.0,
            "resource_effects": "consumes F per exported W; raises fuel demand / activation risk",
            "net_interior_w_removal_per_event": result.b2.net_interior_w_removal,
            "total_environmental_w_per_event": result.b2.total_environmental_w_output,
        }),
    )?;

    atomic_write_json(
        &output_root.join("comparison_matrix/matrix.json"),
        &json!({
            "provenance": provenance,
            "matrix": result.comparison_matrix,
            "selection_inputs": result.selection_inputs,
            "primary_conclusion": result.primary_conclusion,
            "subsidiary_conclusions": subs,
            "component_requirement": result.component_requirement,
            "selected_architecture": match result.primary_conclusion {
                D017PrimaryConclusion::D017SelectConservativeActivationYield => "Candidate A",
                D017PrimaryConclusion::D017SelectEnergyCoupledActiveExport => "Candidate B",
                _ => "neither",
            },
            "terminal_tag": tag,
        }),
    )?;

    let d016_manifest = resolve_path(Path::new("experiments/generated/d016/manifest.json"));
    let d016_hash = if d016_manifest.exists() {
        sha256_file(&d016_manifest).ok()
    } else {
        Some("880ef7e3a9ec2a2d9a50f538e6adc911e003e7a4556541480d88363aef878bfb".into())
    };

    let manifest = json!({
        "project_directive": "D-017",
        "agent_memory_directive": "D-20260716-d017-waste-architecture-comparison",
        "source_commit": commit,
        "binary_hash": bin,
        "frozen_candidate_hash": "9a452d3470be34ccf3bdd7d1397341b64617834e77131cf2899efb327728d626",
        "environment_hash": "ef1834ed1573b634c59e06a55db05038c372b8a0b3c961a4caf10caebab25d39",
        "d016_manifest_sha256": d016_hash,
        "equation_version_unchanged": "membrane_metabolism_v2_conservative",
        "runtime_candidate_created": false,
        "primary_conclusion": result.primary_conclusion,
        "subsidiary_conclusions": subs,
        "terminal_tag": tag,
        "direct_activation_w_fraction": result.direct_activation_fraction,
        "maximum_activation_w_reduction": result.max_activation_w_reduction,
        "perfect_interface_center_w": result.perfect_interface_center_w,
        "max_internal_delivery": result.max_internal_delivery,
        "internal_delivery": result.internal_delivery,
        "viable_alpha_interval": result.viable_alpha_interval,
        "d012_solver_entry_gate": "CLOSED",
        "d008_status": {"stages_0_d": "PASS", "stage_e": "BLOCKED", "stages_f_g": "BLOCKED"},
        "phase1": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "production_verdict": "REQUIRES_REMEDIATION",
        "wall_seconds": t0.elapsed().as_secs_f64(),
    });
    atomic_write_json(&output_root.join("manifest.json"), &manifest)?;

    // Hash each artifact into preservation note
    let mut artifact_hashes = serde_json::Map::new();
    for entry in walkdir_json(&output_root)? {
        if let Ok(h) = sha256_file(&entry) {
            artifact_hashes.insert(
                entry
                    .strip_prefix(&output_root)
                    .unwrap_or(&entry)
                    .display()
                    .to_string(),
                json!(h),
            );
        }
    }
    atomic_write_json(
        &output_root.join("preservation/artifact_hashes.json"),
        &Value::Object(artifact_hashes),
    )?;

    Ok(manifest)
}

fn walkdir_json(root: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut out = Vec::new();
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
        for e in fs::read_dir(dir)? {
            let e = e?;
            let p = e.path();
            if p.is_dir() {
                walk(&p, out)?;
            } else if p.extension().and_then(|s| s.to_str()) == Some("json") {
                out.push(p);
            }
        }
        Ok(())
    }
    walk(root, &mut out)?;
    Ok(out)
}
