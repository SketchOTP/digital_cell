//! One deterministic computational qualification campaign. JSON stdout only.
#[cfg(feature = "chemical-reference-port")]
use chemistry_core::chemical_reference_port::{ReferencePortConfig, ReferencePortState, INITIAL};
#[cfg(feature = "chemical-reference-port")]
use serde_json::{json, Value};

#[cfg(feature = "chemical-reference-port")]
fn run(config: ReferencePortConfig, dt: f64, horizon: f64) -> Result<Value, String> {
    let count = (horizon / dt).round();
    if !dt.is_finite()
        || !horizon.is_finite()
        || dt <= 0.0
        || horizon <= 0.0
        || (count * dt - horizon).abs() > 1e-12
        || count > 100_000.0
    {
        return Err("bounded integer-step horizon required".into());
    }
    let mut state = ReferencePortState::cold(config)?;
    let initial = state.diagnostics()?;
    let mut trajectory = Vec::new();
    let mut max_energy_increase: f64 = 0.0;
    let mut max_abs_step_defect: f64 = 0.0;
    let mut peak_us: f64 = 0.0;
    let mut net_coupling_work = 0.0;
    let mut source_conservation_error: f64 = 0.0;
    let mut max_gradient_identity_error: f64 = initial.gradient_identity_error.abs();
    let mut restart_equivalent = true;
    for i in 0..count as usize {
        let old = state.diagnostics()?;
        let (next, r) = state.try_step(dt)?;
        max_energy_increase =
            max_energy_increase.max((r.diagnostics.total_g - old.total_g).max(0.0));
        max_abs_step_defect = max_abs_step_defect.max(r.step_defect.abs());
        peak_us = peak_us.max(r.diagnostics.us);
        max_gradient_identity_error =
            max_gradient_identity_error.max(r.diagnostics.gradient_identity_error.abs());
        net_coupling_work += r.coupling_work;
        let z = r.diagnostics.species;
        source_conservation_error = source_conservation_error
            .max((z[0] + z[1] + z[2] - 0.2).abs())
            .max((z[4] + z[5] + z[1] + 2.0 * z[2] + 2.0 * z[3] - 1.27).abs());
        if i == count as usize / 2 {
            let restored =
                ReferencePortState::from_restart_json(&state.restart_json()?, state.config())?;
            let (replayed, _) = restored.try_step(dt)?;
            restart_equivalent &= replayed.restart_json()? == next.restart_json()?;
        }
        state = next;
        trajectory.push(r);
    }
    let final_diagnostics = state.diagnostics()?;
    let last = trajectory.last().ok_or("empty trajectory")?;
    Ok(
        json!({"dt":dt,"horizon":horizon,"initial":initial,"final":final_diagnostics,
        "summary":{"accepted_steps":trajectory.len(),"max_energy_increase":max_energy_increase,
            "max_abs_step_defect":max_abs_step_defect,"signed_defect":last.signed_defect,
            "absolute_defect":last.absolute_defect,"integrated_dchem":last.integrated_dchem,
            "integrated_dmech":last.integrated_dmech,"peak_us":peak_us,
            "net_coupling_work":net_coupling_work,"source_conservation_error":source_conservation_error,
            "max_gradient_identity_error":max_gradient_identity_error,"restart_equivalent":restart_equivalent},
        "trajectory":trajectory}),
    )
}

#[cfg(feature = "chemical-reference-port")]
fn probes(
    state: &ReferencePortState,
    strains: &[f64],
    offsets: &[f64],
) -> Result<Vec<Value>, String> {
    let mut rows = Vec::new();
    for &strain in strains {
        for &da in offsets {
            let d = state.probe(strain, da)?;
            let i = state.config().selected_edge;
            let j = (i + 1) % 6;
            let measured_tension: f64 = (0..2)
                .map(|axis| {
                    (d.combined_forces[i][axis] - d.native_forces[i][axis])
                        * (d.vertices[j][axis] - d.vertices[i][axis])
                        / d.lengths[i]
                })
                .sum();
            rows.push(
                json!({"strain":strain,"da":da,"P":d.p,"L0":d.rest_lengths[i],
            "preparation":"externally_prepared_diagnostic_clone_not_trajectory",
            "coupling_tension":measured_tension,"g":d.g,"ec":d.ec,"us":d.us,
            "bare_forward":d.bare_forward,"bare_reverse":d.bare_reverse,
            "forward":d.forward,"reverse":d.reverse,"native_forces":d.native_forces,
            "combined_forces":d.combined_forces}),
            );
        }
    }
    Ok(rows)
}

#[cfg(feature = "chemical-reference-port")]
fn campaign() -> Result<Value, String> {
    let mut dt = 0.002;
    let mut horizon = 2.0;
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() % 2 != 0 {
        return Err("arguments: --dt VALUE --horizon VALUE".into());
    }
    for pair in args.chunks(2) {
        match pair[0].as_str() {
            "--dt" => dt = pair[1].parse().map_err(|_| "invalid dt")?,
            "--horizon" => horizon = pair[1].parse().map_err(|_| "invalid horizon")?,
            _ => return Err("unknown argument".into()),
        }
    }
    let config = ReferencePortConfig::default();
    let state = ReferencePortState::cold(config.clone())?;
    let initial = state.diagnostics()?;
    let mut zero_lambda = config.clone();
    zero_lambda.lambda = 0.0;
    let mut zero_chemistry = config.clone();
    zero_chemistry.rate_scale = 0.0;
    let saved = state.restart_json()?;
    let mut malformed: Value = serde_json::from_str(&saved).map_err(|e| e.to_string())?;
    malformed["species"][0] = json!(0.0);
    let dead_source_rejected =
        ReferencePortState::from_restart_json(&malformed.to_string(), &config).is_err();
    let invalid_step_rejected = state.try_step(config.maximum_dt * 2.0).is_err();
    let old_unchanged = saved == state.restart_json()?;
    Ok(
        json!({"scope":"feature_opt_in_actual_native_mesh_mechanics_fixed_volume_reference_chamber",
        "not_qualified":["physical_parameter_calibration","constant_epsilon_A","native_cubic_replacement",
            "R3_R4_biology","full_organism_integration","contact_growth_remesh_fission"],
        "design_truth":{"K":config.k,"lambda":config.lambda,"theta":config.theta,
            "ks":config.k_s,"gamma":config.gamma,"rate_scale":config.rate_scale,
            "meaning":"designed_computational_reference_inputs_not_empirical_calibration"},
        "initial_native_mesh":{"vertices":initial.vertices,"rest_lengths":initial.rest_lengths,
            "material_total":state.mesh().total_structural_mass(),
            "source_id":"Avanzini_Falasco_Esposito_2020_closed_full_six_species_all_rates_one"},
        "initial_source":INITIAL,
        "training_probes":probes(&state,&[-0.15,-0.05,0.1,0.2],&[-0.04,0.04])?,
        "heldout_probes":probes(&state,&[-0.11,0.04,0.17],&[-0.025,0.025])?,
        "controls":{"dead_source_rejected":dead_source_rejected,"invalid_step_rejected":invalid_step_rejected,
            "rejected_state_unchanged":old_unchanged},
        "runs":{"coarse":run(config.clone(),dt,horizon)?,"fine":run(config,dt/2.0,horizon)?,
            "zero_lambda":run(zero_lambda,dt,horizon)?,"zero_chemistry":run(zero_chemistry,dt,horizon)?}}),
    )
}

#[cfg(feature = "chemical-reference-port")]
fn main() -> Result<(), String> {
    println!(
        "{}",
        serde_json::to_string(&campaign()?).map_err(|e| e.to_string())?
    );
    Ok(())
}

#[cfg(not(feature = "chemical-reference-port"))]
fn main() {
    eprintln!("enable the chemical-reference-port feature");
    std::process::exit(2);
}
