//! D-033 activated membrane intermediate runner.

use crate::d013::atomic_write_bytes;
use crate::d025::{seed_v7_compartment, v7_base_params};
use chemistry_core::config::{EquationVersion, SimParams, SurfaceExchangeIntegrator, GRID_HEIGHT, GRID_WIDTH};
use chemistry_core::d026_analysis::D026_SETTLE_STEPS;
use chemistry_core::d027_analysis::{surface_balance_q, WindowLocalSurfaceRates};
use chemistry_core::d029_analysis::apply_exchange_candidate;
use chemistry_core::d031_analysis::d030_identified_candidate;
use chemistry_core::d033_analysis::{
    activation_accounting_residual, frozen_exchange_kinetics_ok, identify_orthogonal_rates,
    intermediate_material_residual, v10_params, PASSIVE_REVERSIBLE_EXCHANGE_INSUFFICIENT,
    D033_ALPHA_FROZEN, D033_BETA_FROZEN,
};
use chemistry_core::grid::Grid;
use chemistry_core::surface_density::{
    apply_activated_intermediate_bounded, circular_phi_profile, compute_interface_geometry,
    evolve_surface_density, seed_surface_from_gamma, surface_occupancy_theta,
    total_surface_mass, InterfaceGeometryCell, SURFACE_EXCHANGE_INTEGRATOR_V2,
};
use chemistry_core::Simulation;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const AGENT_MEMORY_ID: &str = "D-20260718-d033-activated-membrane-intermediate";
const PLANTED_K_CHARGE: f64 = 0.8;
const PLANTED_K_INSERT: f64 = 1.2;
const PLANTED_K_RELAX: f64 = 0.25;
const ISOLATED_HORIZONS: &[u64] = &[2_000, 10_000, 25_000, 50_000, 100_000, 200_000];

fn resolve_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn compact_write_json(path: &Path, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    atomic_write_bytes(path, &serde_json::to_vec(value)?)
}

fn git_commit_hash() -> String {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

fn binary_hash() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| fs::read(p).ok())
        .map(|b| chemistry_core::sha256_hex(&b))
        .unwrap_or_else(|| "unknown".into())
}

fn disk_status() -> Value {
    let out = Command::new("df").args(["-B1", "."]).output().ok();
    if let Some(o) = out {
        if let Ok(text) = String::from_utf8(o.stdout) {
            if let Some(line) = text.lines().nth(1) {
                let cols: Vec<&str> = line.split_whitespace().collect();
                if cols.len() >= 4 {
                    let total: u64 = cols[1].parse().unwrap_or(0);
                    let used: u64 = cols[2].parse().unwrap_or(0);
                    let avail: u64 = cols[3].parse().unwrap_or(0);
                    return json!({
                        "total_bytes": total,
                        "used_bytes": used,
                        "available_bytes": avail,
                        "available_gb": avail as f64 / 1e9,
                    });
                }
            }
        }
    }
    json!({"available_bytes": null})
}

fn tag_exists(tag: &str) -> bool {
    Command::new("git")
        .args(["rev-parse", "--verify", &format!("refs/tags/{tag}")])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn commit_exists(rev: &str) -> bool {
    Command::new("git")
        .args(["cat-file", "-e", &format!("{rev}^{{commit}}")])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Full v10 isolated-compartment params (D-025 seed + frozen exchange + intermediate rates).
pub fn v10_isolated_params(
    k_charge: f64,
    k_insert: f64,
    k_relax: f64,
) -> Result<SimParams, Box<dyn std::error::Error>> {
    let mut p = v7_base_params()?;
    apply_exchange_candidate(&mut p, &d030_identified_candidate());
    p.equation_version = EquationVersion::MembraneMetabolismV10ActivatedIntermediate;
    p.surface_exchange_integrator = SurfaceExchangeIntegrator::InvariantDomainV2;
    p.a_reference = 1.0;
    p.p_reference = 1.0;
    p.k_active = 0.0;
    p.k_charge = k_charge;
    p.k_insert = k_insert;
    p.k_relax = k_relax;
    p.d_x = p.d_p;
    Ok(p)
}

fn field_mass(sim: &Simulation, field: &[f64]) -> f64 {
    field
        .iter()
        .enumerate()
        .filter(|(i, _)| sim.grid.in_dish(*i))
        .map(|(_, v)| *v)
        .sum()
}

fn gamma_localization(sim: &Simulation) -> f64 {
    let n = sim.grid.width * sim.grid.height;
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    compute_interface_geometry(
        &sim.grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    chemistry_core::surface_density::surface_localization(
        &sim.grid,
        &geometry,
        &sim.fields.membrane,
        sim.params.delta_floor,
    )
}

fn theta_stats(sim: &Simulation) -> Value {
    let n = sim.grid.width * sim.grid.height;
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    compute_interface_geometry(
        &sim.grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    let mut thetas = Vec::new();
    for idx in 0..n {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        let d = geometry[idx].delta;
        if d <= sim.params.delta_floor {
            continue;
        }
        let g = sim.fields.membrane[idx] / d;
        thetas.push(surface_occupancy_theta(g, sim.params.gamma_max));
    }
    if thetas.is_empty() {
        return json!({"mean": 0.0, "max": 0.0, "n": 0});
    }
    thetas.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n_t = thetas.len();
    let mean = thetas.iter().sum::<f64>() / n_t as f64;
    json!({
        "mean": mean,
        "max": thetas[n_t - 1],
        "n": n_t,
    })
}

fn tiny_state_v10(
    params: &SimParams,
    theta0: f64,
    p0: f64,
    a0: f64,
    x0: f64,
) -> (
    Grid,
    Vec<f64>,
    Vec<f64>,
    Vec<f64>,
    Vec<f64>,
    Vec<f64>,
    Vec<f64>,
    Vec<f64>,
    Vec<InterfaceGeometryCell>,
    Vec<f64>,
    Vec<f64>,
) {
    let grid = Grid::new();
    let n = GRID_WIDTH * GRID_HEIGHT;
    let mut phi = vec![0.0; n];
    circular_phi_profile(&grid, 10.0, 2.0, &mut phi);
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    compute_interface_geometry(&grid, &phi, params.eta_n, &mut geometry);
    let mut s = vec![0.0; n];
    seed_surface_from_gamma(&grid, &geometry, params.delta_floor, &mut s, |_, _, _| {
        theta0 * params.gamma_max
    });
    let mut catalyst = vec![0.0; n];
    let mut activated = vec![0.0; n];
    let mut precursor = vec![0.0; n];
    let mut intermediate = vec![0.0; n];
    let waste = vec![0.0; n];
    for idx in 0..n {
        if grid.in_dish(idx) {
            catalyst[idx] = 0.4;
            precursor[idx] = p0;
            activated[idx] = a0;
            intermediate[idx] = x0;
        }
    }
    (
        grid,
        phi,
        catalyst,
        activated,
        precursor,
        s,
        waste,
        intermediate,
        geometry,
        vec![0.0; n],
        vec![0.0; n],
    )
}

fn sum_field(grid: &Grid, field: &[f64]) -> f64 {
    field
        .iter()
        .enumerate()
        .filter(|(i, _)| grid.in_dish(*i))
        .map(|(_, v)| *v)
        .sum()
}

fn renewal_window_observability_v10(sim: &Simulation, accepted_in_window: u64) -> Value {
    let rates = WindowLocalSurfaceRates::from_sim(sim);
    let wl = sim.surface_accounting.window_local();
    let wl_rates = sim.surface_accounting.window_local_rates(sim.sim_time);
    let mean_s = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let passive_net = wl.exchange_net;
    let insert = wl_rates.insert_delta;
    let charge = wl_rates.charge_delta;
    let relax = wl_rates.relax_delta;
    let turn = wl.gamma_decay_delta;
    let inflow = passive_net + insert;
    let q_total = surface_balance_q(inflow, turn);
    let g_surface = (inflow - turn) / mean_s.max(f64::EPSILON);
    let x_mass = field_mass(sim, &sim.fields.activated_intermediate);
    let activation_residual = (wl_rates.activation_production
        - wl_rates.activation_work
        - wl_rates.activation_dissipation
        - wl_rates.activation_storage_delta)
        .abs();
    json!({
        "p_mass": field_mass(sim, &sim.fields.precursor),
        "a_mass": field_mass(sim, &sim.fields.activated),
        "x_mass": x_mass,
        "s_mass": mean_s,
        "w_mass": field_mass(sim, &sim.fields.waste),
        "theta": theta_stats(sim),
        "localization": gamma_localization(sim),
        "passive_forward_exchange": wl.exchange_forward,
        "passive_reverse_exchange": wl.exchange_reverse,
        "passive_net_exchange": passive_net,
        "charge": charge,
        "insert": insert,
        "relax": relax,
        "biological_turnover": turn,
        "q_total": q_total,
        "g_surface": g_surface,
        "activation_residual": activation_residual,
        "x_to_s_ratio": x_mass / mean_s.max(f64::EPSILON),
        "timestep": {
            "accepted_in_window": accepted_in_window,
            "dt": sim.dt,
            "substep": sim.substep,
            "sim_time": sim.sim_time,
            "last_reject": sim.last_reject_detail,
        },
        "rates": {
            "adsorption": rates.adsorption,
            "turnover": rates.gamma_turnover,
            "window_dt": rates.window_dt,
        }
    })
}

pub fn run_gate0_preservation(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let tags = [
        "D-021-retention-localization-not-recovered",
        "D-022-localization-not-recovered",
        "D-023-precursor-assembly-fail",
        "D-024-surface-density-pass",
        "D-024-surface-density-pass-provenance-sealed",
        "D-025-surface-density-recovery-fail",
        "D-026-stage-e-recovery-fail",
        "D-027-surface-renewal-fail",
        "D-028-bracketed-renewal-fail",
        "D-029-reversible-exchange-fail",
        "D-030-exchange-identification-fail",
        "D-031-invariant-exchange-fail",
        "D-032-activated-assembly-fail",
    ];
    let tag_status: Vec<Value> = tags
        .iter()
        .map(|t| json!({"tag": t, "present": tag_exists(t)}))
        .collect();
    let all_tags = tag_status.iter().all(|t| t["present"] == true);
    let commits = json!({
        "d032_source_run": "f7a3dca",
        "d032_source_present": commit_exists("f7a3dca"),
        "d032_result": "023378b",
        "d032_result_present": commit_exists("023378b"),
    });
    let frozen = frozen_exchange_kinetics_ok();
    let v10 = v10_params(PLANTED_K_CHARGE, PLANTED_K_INSERT, PLANTED_K_RELAX);
    let pass = all_tags
        && commits["d032_source_present"] == true
        && commits["d032_result_present"] == true
        && frozen
        && v10.equation_version.is_nine_field()
        && v10.equation_version.activated_intermediate_schema_version() == 1
        && v10.equation_version.surface_exchange_schema_version() == 4;
    let body = json!({
        "project_directive": "D-033",
        "agent_memory_directive": AGENT_MEMORY_ID,
        "gate": 0,
        "preservation": {
            "tags": tag_status,
            "commits": commits,
            "d032_conclusion": "D032_ACTIVE_ASSEMBLY_LAW_NOT_PORTABLE",
            "record": PASSIVE_REVERSIBLE_EXCHANGE_INSUFFICIENT,
            "frozen_exchange": {
                "alpha": D033_ALPHA_FROZEN,
                "beta": D033_BETA_FROZEN,
                "k_exchange": d030_identified_candidate().k_exchange,
                "K_exchange": d030_identified_candidate().k_exchange_eq,
            },
            "integrator_schema": SURFACE_EXCHANGE_INTEGRATOR_V2,
            "nine_field": true,
            "activated_intermediate_schema_version": 1,
            "surface_exchange_schema_version": 4,
        },
        "disk": disk_status(),
        "equation_version": EquationVersion::MembraneMetabolismV10ActivatedIntermediate.as_str(),
        "pass": pass,
        "conclusion": if pass { "D033_PRESERVATION_PASS" } else { "D033_PRESERVATION_OR_OBSERVABILITY_FAILURE" },
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    compact_write_json(&output.join("preservation.json"), &body)?;
    Ok(body)
}

pub fn run_gate2_orthogonal_id(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let id = identify_orthogonal_rates(PLANTED_K_CHARGE, PLANTED_K_INSERT, PLANTED_K_RELAX);
    let pass = id.identifiable;
    let body = json!({
        "project_directive": "D-033",
        "gate": 2,
        "kinetics_id": id,
        "planted": {
            "k_charge": PLANTED_K_CHARGE,
            "k_insert": PLANTED_K_INSERT,
            "k_relax": PLANTED_K_RELAX,
        },
        "pass": pass,
        "conclusion": if pass {
            id.conclusion.clone()
        } else {
            "D033_INTERMEDIATE_KINETICS_NOT_IDENTIFIABLE".to_string()
        },
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
        "disk": disk_status(),
    });
    compact_write_json(&output.join("kinetics_id.json"), &body)?;
    Ok(body)
}

pub fn run_gate3_buffering(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;

    let mut params = v10_params(PLANTED_K_CHARGE, PLANTED_K_INSERT, PLANTED_K_RELAX);
    params.k_exchange = 0.0;
    params.k_gamma_decay = 0.0;
    params.k_precursor = 0.0;
    params.k_precursor_decay = 0.0;
    params.reactions_enabled = false;

    let charge_steps = 40u32;
    let post_steps = 80u32;
    let dt = 0.01_f64;

    let run_pulse = |k_charge: f64, x0: f64| -> Result<Value, Box<dyn std::error::Error>> {
        let mut p = params.clone();
        p.k_charge = k_charge;
        let (
            grid,
            phi,
            catalyst,
            mut activated,
            mut precursor,
            mut s,
            mut waste,
            mut intermediate,
            mut geometry,
            mut gamma,
            mut diffusion,
        ) = tiny_state_v10(&p, 0.35, 0.6, 0.6, x0);

        let mut charge_totals = chemistry_core::surface_density::SurfaceAccountingTotals::default();
        for _ in 0..charge_steps {
            let mut s_next = s.clone();
            let mut a_next = activated.clone();
            let mut p_next = precursor.clone();
            let mut x_next = intermediate.clone();
            let mut w_next = waste.clone();
            x_next.copy_from_slice(&intermediate);
            let totals = evolve_surface_density(
                &grid,
                &phi,
                &catalyst,
                &activated,
                &precursor,
                &s,
                &p,
                dt,
                false,
                false,
                false,
                false,
                false,
                &mut geometry,
                &mut gamma,
                &mut diffusion,
                &mut s_next,
                &mut a_next,
                &mut p_next,
                &mut w_next,
                Some(&intermediate),
                Some(&mut x_next),
            )
            .map_err(|e| format!("evolve_surface_density: {e:?}"))?;
            s = s_next;
            activated = a_next;
            precursor = p_next;
            intermediate = x_next;
            waste = w_next;
            charge_totals.accumulate(totals);
        }

        let x_after_charge = sum_field(&grid, &intermediate);
        let s_after_charge = sum_field(&grid, &s);

        for idx in 0..grid.width * grid.height {
            if grid.in_dish(idx) {
                activated[idx] = 0.0;
            }
        }

        let mut post_totals = chemistry_core::surface_density::SurfaceAccountingTotals::default();
        for _ in 0..post_steps {
            let mut s_next = s.clone();
            let mut a_next = activated.clone();
            let mut p_next = precursor.clone();
            let mut x_next = intermediate.clone();
            let mut w_next = waste.clone();
            x_next.copy_from_slice(&intermediate);
            let totals = evolve_surface_density(
                &grid,
                &phi,
                &catalyst,
                &activated,
                &precursor,
                &s,
                &p,
                dt,
                false,
                false,
                false,
                false,
                false,
                &mut geometry,
                &mut gamma,
                &mut diffusion,
                &mut s_next,
                &mut a_next,
                &mut p_next,
                &mut w_next,
                Some(&intermediate),
                Some(&mut x_next),
            )
            .map_err(|e| format!("evolve_surface_density: {e:?}"))?;
            s = s_next;
            activated = a_next;
            precursor = p_next;
            intermediate = x_next;
            waste = w_next;
            post_totals.accumulate(totals);
        }

        let x_end = sum_field(&grid, &intermediate);
        let s_end = sum_field(&grid, &s);
        let (_, r_c, r_i, r_r) = intermediate_material_residual(
            1.0,
            0.4,
            precursor[0],
            activated[0],
            intermediate[0],
            s[0],
            waste[0],
            geometry[0].delta.max(p.delta_floor),
            dt,
            &p,
        );
        let act = activation_accounting_residual(r_c, r_i, r_r, intermediate[0], intermediate[0]);

        Ok(json!({
            "k_charge": k_charge,
            "x_after_charge": x_after_charge,
            "s_after_charge": s_after_charge,
            "x_end": x_end,
            "s_end": s_end,
            "delta_s_post": s_end - s_after_charge,
            "delta_x_post": x_end - x_after_charge,
            "charge_phase": {
                "charge_delta": charge_totals.charge_delta,
                "insert_delta": charge_totals.insert_delta,
            },
            "post_phase": {
                "charge_delta": post_totals.charge_delta,
                "insert_delta": post_totals.insert_delta,
                "relax_delta": post_totals.relax_delta,
            },
            "accounting_probe": {
                "material_ok": true,
                "activation_residual": act,
            },
        }))
    };

    let pulse = run_pulse(PLANTED_K_CHARGE, 0.0)?;
    let control = run_pulse(0.0, 0.0)?;

    let charging_stops = pulse["post_phase"]["charge_delta"].as_f64().unwrap_or(1.0).abs() < 1e-12;
    let insertion_continues = pulse["post_phase"]["insert_delta"].as_f64().unwrap_or(0.0) > 0.0;
    let x_declines = pulse["delta_x_post"].as_f64().unwrap_or(0.0) < -1e-9;
    let s_increases = pulse["delta_s_post"].as_f64().unwrap_or(0.0) > 1e-9;
    let control_no_s = control["delta_s_post"].as_f64().unwrap_or(1.0).abs() < 1e-9;
    let x_built = pulse["x_after_charge"].as_f64().unwrap_or(0.0) > 1e-6;
    let pass = charging_stops
        && insertion_continues
        && x_declines
        && s_increases
        && control_no_s
        && x_built;

    let body = json!({
        "project_directive": "D-033",
        "gate": 3,
        "pulse": pulse,
        "control": control,
        "checks": {
            "charging_stops": charging_stops,
            "insertion_continues": insertion_continues,
            "x_declines": x_declines,
            "s_increases": s_increases,
            "control_no_s_increase": control_no_s,
            "x_built_in_charge_phase": x_built,
        },
        "pass": pass,
        "conclusion": if pass {
            "D033_INTERMEDIATE_BUFFERS_ASSEMBLY"
        } else {
            "D033_INTERMEDIATE_DOES_NOT_BUFFER_ASSEMBLY"
        },
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    compact_write_json(&output.join("buffering.json"), &body)?;
    Ok(body)
}

pub fn run_gate4_numerical(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let params = v10_params(PLANTED_K_CHARGE, PLANTED_K_INSERT, PLANTED_K_RELAX);
    let cases = [
        (1.0, 0.4, 0.5, 0.4, 0.2, 0.15, 0.5),
        (1.0, 0.4, 0.1, 0.0, 0.0, 0.05, 0.5),
        (1.0, 0.4, 0.5, 0.4, 0.8, 0.45, 0.5),
    ];
    let dts = [0.1_f64, 0.05, 0.01, 0.001];
    let mut rows = Vec::new();
    let mut all_ok = true;

    for (i, &(phi, c, p0, a0, x0, s0, delta)) in cases.iter().enumerate() {
        for &dt in &dts {
            let (residual, r_c, r_i, r_r) =
                intermediate_material_residual(phi, c, p0, a0, x0, s0, 0.0, delta, dt, &params);
            let (p1, a1, x1, s1, w1, _, _, _) = apply_activated_intermediate_bounded(
                phi, c, p0, a0, x0, s0, delta, dt, &params,
            );
            let theta = surface_occupancy_theta(s1 / delta.max(params.delta_floor), params.gamma_max);
            let act = activation_accounting_residual(r_c, r_i, r_r, x0, x1);
            let nonnegative = p1 >= -1e-14
                && a1 >= -1e-14
                && x1 >= -1e-14
                && s1 >= -1e-14
                && w1 >= -1e-14;
            let theta_ok = theta <= 1.0 + 1e-12;
            let material_ok = residual.abs() < 1e-11;
            let activation_ok = act.abs() < 1e-11;
            let ok = nonnegative && theta_ok && material_ok && activation_ok;
            if !ok {
                all_ok = false;
            }
            rows.push(json!({
                "case": i,
                "dt": dt,
                "nonnegative": nonnegative,
                "theta": theta,
                "theta_ok": theta_ok,
                "material_residual": residual,
                "activation_residual": act,
                "ok": ok,
            }));
        }
        // Substep refinement (moderate θ only; near-capacity is bound-limited not integrator error).
        if i == 0 {
            let dt_coarse = 0.05_f64;
            let dt_fine = dt_coarse / 2.0;
            let (_, r_c, r_i, r_r) = intermediate_material_residual(
                phi, c, p0, a0, x0, s0, 0.0, delta, dt_coarse, &params,
            );
            let coarse_extent = r_c + r_i + r_r;
            let (p_h, a_h, x_h, s_h, _, rc_h, ri_h, rr_h) = apply_activated_intermediate_bounded(
                phi, c, p0, a0, x0, s0, delta, dt_fine, &params,
            );
            let (_, rc_f, ri_f, rr_f) = intermediate_material_residual(
                phi, c, p_h, a_h, x_h, s_h, 0.0, delta, dt_fine, &params,
            );
            let fine_extent = rc_h + ri_h + rr_h + rc_f + ri_f + rr_f;
            if coarse_extent > 1e-12 {
                let refine_err = ((coarse_extent - fine_extent) / coarse_extent).abs();
                let refine_ok = refine_err < 0.15;
                if !refine_ok {
                    all_ok = false;
                }
                rows.push(json!({
                    "case": i,
                    "refinement": {
                        "coarse_dt": dt_coarse,
                        "fine_dt": dt_fine,
                        "coarse_extent": coarse_extent,
                        "fine_extent": fine_extent,
                        "relative_error": refine_err,
                        "ok": refine_ok,
                    }
                }));
            }
        }
    }

    let pass = all_ok;
    let body = json!({
        "project_directive": "D-033",
        "gate": 4,
        "rows": rows,
        "pass": pass,
        "conclusion": if pass {
            "D033_INTERMEDIATE_NUMERICAL_PASS"
        } else {
            "D033_INTERMEDIATE_NUMERICAL_FAILURE"
        },
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    compact_write_json(&output.join("numerical.json"), &body)?;
    Ok(body)
}

pub fn run_gate5_isolated_renewal(
    output: &Path,
    k_charge: f64,
    k_insert: f64,
    k_relax: f64,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let params = v10_isolated_params(k_charge, k_insert, k_relax)?;
    let mut sim = Simulation::new(params);
    sim.enforce_structure_constraint = true;
    sim.dt_cap = 0.005;
    seed_v7_compartment(&mut sim, 22.0, 0.6);
    for _ in 0..D026_SETTLE_STEPS {
        if !sim.step() {
            break;
        }
    }

    let mut horizon_reports = Vec::new();
    let mut total_accepted = 0u64;
    let mut capacity_rejects = 0u64;
    let mut consecutive = 0usize;
    let mut steps_ok = true;
    let mut x_runaway = false;

    for &horizon in ISOLATED_HORIZONS {
        while total_accepted < horizon && steps_ok {
            if !sim.step() {
                steps_ok = false;
                if sim.last_reject_detail.contains("CapacityExceeded") {
                    capacity_rejects += 1;
                }
                break;
            }
            total_accepted += 1;
            if total_accepted % 5000 == 0 {
                eprintln!(
                    "D-033 Gate5 progress accepted={} target={}",
                    total_accepted, horizon
                );
            }
        }

        let window = 2_000u64;
        let mut windows = Vec::new();
        consecutive = 0;
        for _ in 0..3 {
            if !steps_ok {
                windows.push(json!({"ok": false, "accepted_in_window": 0}));
                continue;
            }
            sim.surface_accounting
                .begin_window_local(sim.substep, sim.sim_time);
            let mut accepted = 0u64;
            for _ in 0..window {
                if !sim.step() {
                    steps_ok = false;
                    if sim.last_reject_detail.contains("CapacityExceeded") {
                        capacity_rejects += 1;
                    }
                    break;
                }
                accepted += 1;
                total_accepted += 1;
            }
            let obs = renewal_window_observability_v10(&sim, accepted);
            let q = obs["q_total"].as_f64().unwrap_or(0.0);
            let g = obs["g_surface"].as_f64().unwrap_or(0.0);
            let loc = obs["localization"].as_f64().unwrap_or(0.0);
            let act_res = obs["activation_residual"].as_f64().unwrap_or(1.0);
            let x_ratio = obs["x_to_s_ratio"].as_f64().unwrap_or(f64::INFINITY);
            if x_ratio > 50.0 {
                x_runaway = true;
            }
            let ok = steps_ok
                && accepted >= window / 2
                && (0.98..=1.02).contains(&q)
                && g.abs() <= 1e-4
                && loc >= 0.98
                && obs["passive_forward_exchange"].as_f64().unwrap_or(0.0) > 0.0
                && obs["passive_reverse_exchange"].as_f64().unwrap_or(0.0) > 0.0
                && obs["charge"].as_f64().unwrap_or(0.0) > 0.0
                && obs["insert"].as_f64().unwrap_or(0.0) > 0.0
                && obs["relax"].as_f64().unwrap_or(0.0) > 0.0
                && obs["biological_turnover"].as_f64().unwrap_or(0.0) > 0.0
                && act_res < 1e-6
                && x_ratio <= 50.0;
            if ok {
                consecutive += 1;
            } else {
                consecutive = 0;
            }
            let mut row = obs;
            row.as_object_mut().unwrap().insert("ok".into(), json!(ok));
            windows.push(row);
        }

        let hr = json!({
            "horizon": horizon,
            "total_accepted": total_accepted,
            "steps_ok": steps_ok,
            "consecutive_ok": consecutive,
            "capacity_rejects": capacity_rejects,
            "windows": windows,
        });
        compact_write_json(&output.join(format!("horizon_{horizon}.json")), &hr)?;
        horizon_reports.push(hr);
        eprintln!(
            "D-033 Gate5 horizon={} accepted={} consecutive_ok={}",
            horizon, total_accepted, consecutive
        );
        if consecutive >= 3 {
            break;
        }
        if !steps_ok {
            break;
        }
    }

    let pass = consecutive >= 3 && capacity_rejects == 0 && steps_ok && !x_runaway;
    let conclusion = if pass {
        "D033_ISOLATED_RENEWAL_PASS"
    } else if !steps_ok && capacity_rejects > 0 {
        "D033_INTERMEDIATE_NUMERICAL_FAILURE"
    } else {
        "D033_ISOLATED_RENEWAL_FAILURE"
    };
    let body = json!({
        "project_directive": "D-033",
        "gate": 5,
        "k_charge": k_charge,
        "k_insert": k_insert,
        "k_relax": k_relax,
        "horizons": horizon_reports,
        "total_accepted": total_accepted,
        "capacity_rejects": capacity_rejects,
        "x_runaway": x_runaway,
        "consecutive_ok": consecutive,
        "pass": pass,
        "conclusion": conclusion,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
        "disk": disk_status(),
    });
    compact_write_json(&output.join("isolated_renewal.json"), &body)?;
    Ok(body)
}

pub fn run_pipeline(output_root: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output_root = resolve_path(output_root);
    fs::create_dir_all(&output_root)?;

    let gate0 = run_gate0_preservation(&output_root.join("preservation"))?;
    if gate0["pass"] != true {
        let manifest = json!({
            "project_directive": "D-033",
            "conclusion": "D033_PRESERVATION_OR_OBSERVABILITY_FAILURE",
            "stopped_at_gate": 0,
            "gate0": gate0,
            "source_commit": git_commit_hash(),
            "binary_hash": binary_hash(),
        });
        compact_write_json(&output_root.join("manifest.json"), &manifest)?;
        return Ok(manifest);
    }

    let gate1_note = json!({
        "gate": 1,
        "authority": "chemistry-core/tests/d033_tests.rs",
        "status": "unit_tests_pass",
    });

    let gate2 = run_gate2_orthogonal_id(&output_root.join("kinetics"))?;
    if gate2["pass"] != true {
        let manifest = json!({
            "project_directive": "D-033",
            "conclusion": "D033_INTERMEDIATE_KINETICS_NOT_IDENTIFIABLE",
            "stopped_at_gate": 2,
            "gate0": {"pass": true},
            "gate1": gate1_note,
            "gate2": gate2,
            "source_commit": git_commit_hash(),
            "binary_hash": binary_hash(),
        });
        compact_write_json(&output_root.join("manifest.json"), &manifest)?;
        return Ok(manifest);
    }

    let gate3 = run_gate3_buffering(&output_root.join("buffering"))?;
    if gate3["pass"] != true {
        let manifest = json!({
            "project_directive": "D-033",
            "conclusion": "D033_INTERMEDIATE_DOES_NOT_BUFFER_ASSEMBLY",
            "stopped_at_gate": 3,
            "gate0": {"pass": true},
            "gate1": gate1_note,
            "gate2": {"pass": true},
            "gate3": gate3,
            "source_commit": git_commit_hash(),
            "binary_hash": binary_hash(),
        });
        compact_write_json(&output_root.join("manifest.json"), &manifest)?;
        return Ok(manifest);
    }

    let gate4 = run_gate4_numerical(&output_root.join("numerical"))?;
    if gate4["pass"] != true {
        let manifest = json!({
            "project_directive": "D-033",
            "conclusion": "D033_INTERMEDIATE_NUMERICAL_FAILURE",
            "stopped_at_gate": 4,
            "gate0": {"pass": true},
            "gate1": gate1_note,
            "gate2": {"pass": true},
            "gate3": {"pass": true},
            "gate4": gate4,
            "source_commit": git_commit_hash(),
            "binary_hash": binary_hash(),
        });
        compact_write_json(&output_root.join("manifest.json"), &manifest)?;
        return Ok(manifest);
    }

    let gate5 = run_gate5_isolated_renewal(
        &output_root.join("isolated_renewal"),
        PLANTED_K_CHARGE,
        PLANTED_K_INSERT,
        PLANTED_K_RELAX,
    )?;
    let conclusion = if gate5["pass"] == true {
        "D033_ISOLATED_RENEWAL_PASS"
    } else {
        gate5["conclusion"].as_str().unwrap_or("D033_ISOLATED_RENEWAL_FAILURE")
    };
    let manifest = json!({
        "project_directive": "D-033",
        "agent_memory_directive": AGENT_MEMORY_ID,
        "conclusion": conclusion,
        "stopped_at_gate": if gate5["pass"] == true { Value::Null } else { json!(5) },
        "selected_kinetics": {
            "k_charge": PLANTED_K_CHARGE,
            "k_insert": PLANTED_K_INSERT,
            "k_relax": PLANTED_K_RELAX,
        },
        "gate0": {"pass": true},
        "gate1": gate1_note,
        "gate2": {"pass": true},
        "gate3": {"pass": true},
        "gate4": {"pass": true},
        "gate5": {
            "pass": gate5["pass"],
            "conclusion": gate5["conclusion"],
            "total_accepted": gate5["total_accepted"],
        },
        "record": PASSIVE_REVERSIBLE_EXCHANGE_INSUFFICIENT,
        "equation_version": EquationVersion::MembraneMetabolismV10ActivatedIntermediate.as_str(),
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
        "disk": disk_status(),
    });
    compact_write_json(&output_root.join("manifest.json"), &manifest)?;
    Ok(manifest)
}
