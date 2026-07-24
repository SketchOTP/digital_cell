//! D-085 decisive structural closure campaign runner.

use crate::d013::atomic_write_json;
use crate::d025::{seed_v7_compartment, v7_base_params};
use chemistry_core::config::{EquationVersion, SimParams, StructureEvolutionMode, DX};
use chemistry_core::d039_analysis::v8_schema3_params;
use chemistry_core::d049_analysis::d049_frozen_params;
use chemistry_core::d050_analysis::ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME;
use chemistry_core::d053_analysis::*;
use chemistry_core::d055_analysis::{D055_FROZEN_M_BETA, D055_FROZEN_M_EXT};
use chemistry_core::d079_analysis::{ASSEMBLY_DT, DYNAMIC_COVERAGE_GATE, SEED_DENSITY};
use chemistry_core::d085_analysis::*;
use chemistry_core::edge_membrane::{
    accepted_step_supported, off_support_bound_fraction, seed_free_near_support, support_coverage,
    EdgeMembraneParams, EdgeMembraneState,
};
use chemistry_core::edge_migration::migrate_bound_across_support;
use chemistry_core::edge_support::{build_cut_cell_support, CutCellSupport};
use chemistry_core::field_mass;
use chemistry_core::structural_kinetics::apply_mixed_turnover_params;
use chemistry_core::Simulation;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const EDGE_SYNC_EVERY: u64 = 25;
const ASSEMBLY_STEPS: usize = 80;

fn resolve_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path)
    }
}

fn ensure_archive_symlink(path: &Path) {
    if path.exists() {
        return;
    }
    let archive_root = PathBuf::from(
        "/mnt/storage1tb/cache/project-artifacts/digital_cell/experiments/generated",
    );
    if path.file_name().map(|n| n == "d085").unwrap_or(false) {
        let target = archive_root.join("d085");
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::create_dir_all(&target);
        #[cfg(unix)]
        {
            let _ = std::os::unix::fs::symlink(&target, path);
        }
    }
}

fn git_commit_hash() -> String {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

fn write_gate(out: &Path, name: &str, body: &Value) -> Result<(), Box<dyn std::error::Error>> {
    let dir = out.join(name);
    fs::create_dir_all(&dir)?;
    atomic_write_json(&dir.join("result.json"), body)?;
    Ok(())
}

fn organism_params(noise_seed: u64, mechano: Option<&MechanoCandidate>) -> SimParams {
    let base = v7_base_params().unwrap_or_else(|_| v8_schema3_params());
    let mut params = d049_frozen_params(&base);
    params.equation_version = EquationVersion::MembraneMetabolismV13CatalystSaturatingActivation;
    params.activation_schema = ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME;
    params.k_d008_activation = D053_FITTED_V_A;
    params.k_c_activation = D053_FITTED_K_C;
    params.n_ref_activation = D053_N_REF;
    params.f_ref_activation = D053_F_REF;
    params.m_ext = D055_FROZEN_M_EXT;
    params.m_beta = D055_FROZEN_M_BETA;
    apply_delivery_repair(
        &mut params,
        DeliveryRepairPair {
            m_ext: D055_FROZEN_M_EXT,
            m_beta: D055_FROZEN_M_BETA,
        },
    );
    apply_mixed_turnover_params(&mut params, D085_D084_ETA, D085_D084_K_PHI_LOSS);
    clear_mechano_params(&mut params);
    if let Some(c) = mechano {
        apply_mechano_params(&mut params, c);
    }
    params.random_seed = noise_seed;
    // Production default remains mixed-off; this candidate run enables it explicitly.
    params
}

fn mass_equivalent_radius(sim: &Simulation) -> f64 {
    (field_mass(&sim.grid, &sim.fields.structure) / std::f64::consts::PI)
        .max(0.0)
        .sqrt()
}

fn hold_exterior(sim: &mut Simulation) {
    for idx in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] < 0.5 {
            sim.fields.nutrient[idx] = sim.params.n_reservoir;
            sim.fields.fuel[idx] = sim.params.f_reservoir;
        }
    }
}

fn dish_contact(sim: &Simulation) -> bool {
    let w = sim.grid.width;
    let h = sim.grid.height;
    for j in 0..h {
        for i in 0..w {
            let idx = j * w + i;
            if !sim.grid.in_dish(idx) {
                continue;
            }
            if sim.fields.structure[idx] >= 0.5
                && (i <= 1 || j <= 1 || i + 2 >= w || j + 2 >= h)
            {
                return true;
            }
        }
    }
    false
}

fn fragmented(sim: &Simulation) -> bool {
    // Simple 4-connected interior component count.
    let n = sim.fields.structure.len();
    let w = sim.grid.width;
    let mut seen = vec![false; n];
    let mut components = 0u32;
    for start in 0..n {
        if seen[start] || !sim.grid.in_dish(start) || sim.fields.structure[start] < 0.5 {
            continue;
        }
        components += 1;
        if components > 1 {
            return true;
        }
        let mut stack = vec![start];
        seen[start] = true;
        while let Some(idx) = stack.pop() {
            let i = idx % w;
            let j = idx / w;
            for (di, dj) in [(-1isize, 0), (1, 0), (0, -1), (0, 1)] {
                let ii = i as isize + di;
                let jj = j as isize + dj;
                if ii < 0 || jj < 0 {
                    continue;
                }
                let (ii, jj) = (ii as usize, jj as usize);
                if ii >= w || jj >= sim.grid.height {
                    continue;
                }
                let nidx = jj * w + ii;
                if seen[nidx] || !sim.grid.in_dish(nidx) || sim.fields.structure[nidx] < 0.5 {
                    continue;
                }
                seen[nidx] = true;
                stack.push(nidx);
            }
        }
    }
    false
}

fn init_edge(sim: &Simulation) -> (EdgeMembraneState, CutCellSupport, EdgeMembraneParams) {
    let w = sim.grid.width;
    let h = sim.grid.height;
    let params = EdgeMembraneParams::default();
    let mut state = EdgeMembraneState::new(w, h);
    state.catalyst = 1.0;
    let support = build_cut_cell_support(&sim.fields.structure, w, h);
    seed_free_near_support(&mut state, &support, SEED_DENSITY);
    for _ in 0..ASSEMBLY_STEPS {
        let _ = accepted_step_supported(
            &mut state,
            &sim.fields.structure,
            &support,
            &params,
            ASSEMBLY_DT,
            false,
            1.0,
        );
    }
    (state, support, params)
}

fn sync_edge(
    sim: &Simulation,
    state: &mut EdgeMembraneState,
    support: &mut CutCellSupport,
    params: &EdgeMembraneParams,
) {
    let new_support = build_cut_cell_support(&sim.fields.structure, sim.grid.width, sim.grid.height);
    let _ = migrate_bound_across_support(state, support, &new_support, params);
    *support = new_support;
    let _ = accepted_step_supported(
        state,
        &sim.fields.structure,
        support,
        params,
        ASSEMBLY_DT,
        false,
        1.0,
    );
}

/// Attribute local curvature/strain from cut-cell face measures into cell fields.
fn feed_mechano_from_support(
    sim: &mut Simulation,
    support: &CutCellSupport,
    prev_measures: &mut (Vec<f64>, Vec<f64>),
    dt: f64,
) {
    if !sim.params.use_mechanochemical_structure {
        return;
    }
    let w = sim.grid.width;
    let h = sim.grid.height;
    for v in sim.mechano_kappa.iter_mut() {
        *v = 0.0;
    }
    for v in sim.mechano_strain.iter_mut() {
        *v = 0.0;
    }
    // Local |κ| proxy: discrete |∇²φ| on interface band (field-local; no circle target).
    for idx in 0..w * h {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        let phi = sim.fields.structure[idx];
        if chemistry_core::reactions::interface_weight(phi) < 1e-6 {
            continue;
        }
        let lap = chemistry_core::structural_kinetics::local_abs_laplacian(
            &sim.fields.structure,
            w,
            h,
            idx,
        );
        sim.mechano_kappa[idx] = lap;
    }
    // Strain from accepted change in face measures attributed to adjacent cells.
    let dt = dt.max(1e-12);
    for (kind_i, (cur, prev)) in [
        (&support.measure_h, &mut prev_measures.0),
        (&support.measure_v, &mut prev_measures.1),
    ]
    .into_iter()
    .enumerate()
    {
        if prev.len() != cur.len() {
            *prev = cur.clone();
            continue;
        }
        for (fi, (&m_now, &m_prev)) in cur.iter().zip(prev.iter()).enumerate() {
            if m_prev <= 1e-15 && m_now <= 1e-15 {
                continue;
            }
            let s = (m_now - m_prev) / (m_prev.max(1e-9) * dt);
            // Map face index to two adjacent cells.
            let (c0, c1) = if kind_i == 0 {
                // horizontal face between (i,j) and (i+1,j)
                let i = fi % (w - 1);
                let j = fi / (w - 1);
                (j * w + i, j * w + i + 1)
            } else {
                let i = fi % w;
                let j = fi / w;
                (j * w + i, (j + 1) * w + i)
            };
            for c in [c0, c1] {
                if c < sim.mechano_strain.len() {
                    sim.mechano_strain[c] = s;
                }
            }
        }
        *prev = cur.clone();
    }
}

fn edge_metrics(
    state: &EdgeMembraneState,
    support: &CutCellSupport,
    params: &EdgeMembraneParams,
) -> (f64, f64, bool) {
    let cov = support_coverage(state, support, params);
    let ghost = off_support_bound_fraction(state, support);
    let trailing_ok = ghost <= 0.20 && cov + 1e-12 >= DYNAMIC_COVERAGE_GATE.min(D085_COVERAGE_MIN);
    (cov, ghost, trailing_ok)
}

pub fn run_dynamic_member(
    radius: f64,
    noise_seed: u64,
    mechano: Option<&MechanoCandidate>,
) -> DynamicRunRow {
    let max_h = if smoke_mode() {
        max_accepted().min(3_000)
    } else {
        max_accepted()
    };
    let window = if smoke_mode() {
        window_size().min(500)
    } else {
        window_size()
    };
    let mut params = organism_params(noise_seed, mechano);
    let mut sim = Simulation::new(params);
    sim.dt_cap = 0.005;
    sim.set_structure_evolution_mode(StructureEvolutionMode::DynamicStructure);
    seed_v7_compartment(&mut sim, radius, D053_THETA);
    hold_exterior(&mut sim);

    let (mut edge, mut support, edge_params) = init_edge(&sim);
    let mut prev_measures = (support.measure_h.clone(), support.measure_v.clone());

    let a0 = field_mass(&sim.grid, &sim.fields.activated).max(1e-18);
    let c0 = field_mass(&sim.grid, &sim.fields.catalyst).max(1e-18);
    let r0 = mass_equivalent_radius(&sim);

    let mut accepted = 0u64;
    let mut rejected = 0u64;
    let mut consecutive_rejected = 0u64;
    let mut steps_ok = true;
    let mut converged_windows = 0u64;
    let mut window_start_r = r0;
    let mut window_start_accepted = 0u64;
    let mut termination = TerminationKind::MaxHorizon;
    let mut last_cov = 0.0;
    let mut last_ghost = 1.0;
    let mut last_trailing = false;

    while accepted < max_h {
        hold_exterior(&mut sim);
        if !sim.step() {
            rejected += 1;
            consecutive_rejected += 1;
            if consecutive_rejected >= 80 || rejected > max_h {
                steps_ok = false;
                termination = TerminationKind::NumericalFailure;
                break;
            }
            continue;
        }
        accepted += 1;
        consecutive_rejected = 0;
        let dt = sim.dt;

        if accepted % EDGE_SYNC_EVERY == 0 || accepted == 1 {
            sync_edge(&sim, &mut edge, &mut support, &edge_params);
            let (cov, ghost, trailing) = edge_metrics(&edge, &support, &edge_params);
            last_cov = cov;
            last_ghost = ghost;
            last_trailing = trailing;
            feed_mechano_from_support(&mut sim, &support, &mut prev_measures, dt * EDGE_SYNC_EVERY as f64);
        }

        if accepted - window_start_accepted >= window {
            let r_now = mass_equivalent_radius(&sim);
            let v_r = (r_now - window_start_r) / window as f64;
            if v_r.abs() <= D085_RADIUS_VEL_CONV {
                converged_windows += 1;
            } else {
                converged_windows = 0;
            }
            window_start_r = r_now;
            window_start_accepted = accepted;
            if converged_windows >= D085_REQUIRED_WINDOWS {
                termination = TerminationKind::ThreeConvergedWindows;
                break;
            }
        }

        let r_now = mass_equivalent_radius(&sim);
        let a_now = field_mass(&sim.grid, &sim.fields.activated);
        let c_now = field_mass(&sim.grid, &sim.fields.catalyst);
        if r_now < 3.0 || a_now < 0.05 * a0 || c_now < 0.05 * c0 {
            termination = TerminationKind::BiologicalTerminal;
            break;
        }
        // Early fail: retention already below basin floor — no need to burn full horizon.
        if accepted >= window && (a_now / a0 < D085_RETENTION_MIN || c_now / c0 < D085_RETENTION_MIN)
        {
            termination = TerminationKind::BiologicalTerminal;
            break;
        }
        if dish_contact(&sim) {
            termination = TerminationKind::BiologicalTerminal;
            break;
        }
    }

    sync_edge(&sim, &mut edge, &mut support, &edge_params);
    let (cov, ghost, trailing) = edge_metrics(&edge, &support, &edge_params);
    last_cov = cov;
    last_ghost = ghost;
    last_trailing = trailing;

    let r_final = mass_equivalent_radius(&sim);
    let a1 = field_mass(&sim.grid, &sim.fields.activated);
    let c1 = field_mass(&sim.grid, &sim.fields.catalyst);
    let frag = fragmented(&sim);
    let contact = dish_contact(&sim);
    let exhausted = a1 < 0.05 * a0 || c1 < 0.05 * c0;
    let clipped = sim.fields.structure.iter().any(|p| !p.is_finite())
        || sim.fields.activated.iter().any(|p| !p.is_finite());

    // Mandatory static/dynamic parity: freeze geometry, recompute structural flow.
    let cell = DX * DX;
    let (g_frozen, l_frozen) = integrate_frozen_field_rates(
        &sim.fields.structure,
        &sim.fields.activated,
        &sim.fields.catalyst,
        sim.grid.width,
        sim.grid.height,
        |idx| sim.grid.in_dish(idx),
        &sim.params,
        cell,
    );
    let frozen_net = g_frozen - l_frozen;
    let dt_last = sim.dt.max(1e-12);
    // Runtime: last accepted step's structural reaction mass change rate.
    let runtime_net = sim.accounting.last_step.structure.reaction_delta / dt_last;
    let parity_ok = parity_direction_agrees(runtime_net, frozen_net, 1e-6)
        || (runtime_net.abs() < 1e-8 && frozen_net.abs() < 1e-8);
    let parity_ok = if runtime_net.abs() > 1e-6 && frozen_net.abs() > 1e-6 {
        parity_ok
            && (runtime_net - frozen_net).abs()
                <= 0.35 * runtime_net.abs().max(frozen_net.abs())
    } else {
        parity_ok
    };

    DynamicRunRow {
        radius_seed: radius,
        noise_seed,
        equivalent_radius: r_final,
        structural_mass: field_mass(&sim.grid, &sim.fields.structure),
        c_mass: c1,
        a_mass: a1,
        l_mass: edge.total_l(),
        b_mass: edge.total_b(),
        w_mass: field_mass(&sim.grid, &sim.fields.waste),
        structural_production: sim.accounting.cumulative.structural_synthesis,
        structural_loss: sim.accounting.cumulative.structural_decay,
        radius_velocity: (r_final - r0) / accepted.max(1) as f64,
        edge_coverage: last_cov,
        ghost_fraction: last_ghost,
        trailing_ok: last_trailing,
        c_retention: c1 / c0,
        a_retention: a1 / a0,
        accepted,
        accepted_time: sim.sim_time,
        termination,
        steps_ok: steps_ok && !clipped,
        accounting_ok: sim.accounting.cumulative_within_tolerance(),
        fragmented: frag,
        dish_contact: contact,
        exhausted,
        clipped,
        window_converged: matches!(termination, TerminationKind::ThreeConvergedWindows),
        runtime_structural_net: runtime_net,
        frozen_structural_net: frozen_net,
        parity_ok,
    }
}

fn run_basin_matrix(mechano: Option<&MechanoCandidate>) -> Vec<RadiusCohortResult> {
    let radii: Vec<f64> = if smoke_mode() {
        vec![22.0]
    } else {
        D085_BASIN_RADII.to_vec()
    };
    let seeds: Vec<u64> = if smoke_mode() {
        vec![1]
    } else {
        D085_NOISE_SEEDS.to_vec()
    };
    let mut cohorts = Vec::new();
    for &r in &radii {
        let mut rows = Vec::new();
        for &seed in &seeds {
            eprintln!("D-085 dynamic run R{r} seed={seed} mechano={:?}", mechano.map(|c| c.label));
            rows.push(run_dynamic_member(r, seed, mechano));
        }
        cohorts.push(classify_radius_cohort(r, &rows));
    }
    cohorts
}

fn measure_kappa_strain_scales() -> (f64, f64) {
    // Short probe at R22 seed1 to set K_κ, K_s from local ranges (not target radius).
    let mut params = organism_params(1, None);
    let mut sim = Simulation::new(params);
    sim.dt_cap = 0.005;
    sim.set_structure_evolution_mode(StructureEvolutionMode::DynamicStructure);
    seed_v7_compartment(&mut sim, 22.0, D053_THETA);
    let (_edge, support, _p) = init_edge(&sim);
    let mut kappa_max: f64 = 1e-6;
    let mut strain_max: f64 = 1e-6;
    let w = sim.grid.width;
    let h = sim.grid.height;
    for idx in 0..w * h {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        let lap = chemistry_core::structural_kinetics::local_abs_laplacian(
            &sim.fields.structure,
            w,
            h,
            idx,
        );
        kappa_max = kappa_max.max(lap);
    }
    for m in support.measure_h.iter().chain(support.measure_v.iter()) {
        strain_max = strain_max.max(*m);
    }
    // Strain scale: relative change over one sync interval at unit measure.
    (kappa_max, (strain_max.max(1e-3)).recip().min(10.0).max(1e-3))
}

fn parity_checkpoints(cohorts: &[RadiusCohortResult]) -> Value {
    let mut rows = Vec::new();
    let mut all_ok = true;
    for c in cohorts {
        for r in &c.rows {
            let ok = r.parity_ok
                || parity_direction_agrees(r.runtime_structural_net, r.frozen_structural_net, 1e-6);
            all_ok &= ok;
            rows.push(json!({
                "radius_seed": r.radius_seed,
                "noise_seed": r.noise_seed,
                "equivalent_radius": r.equivalent_radius,
                "runtime_structural_net": r.runtime_structural_net,
                "frozen_structural_net": r.frozen_structural_net,
                "agree": ok,
            }));
        }
    }
    json!({ "pass": all_ok, "rows": rows })
}

pub fn run_pipeline(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let out = resolve_path(output);
    ensure_archive_symlink(&out);
    let out = if out.exists() {
        out
    } else {
        let local = resolve_path(Path::new("experiments/generated/d085"));
        fs::create_dir_all(&local)?;
        local
    };
    for sub in [
        "preservation",
        "d084_full_basin",
        "parity",
        "failure_classification",
        "mechanochemical_candidates",
        "mechanochemical_basin",
        "energy_waste",
        "damage_controls",
        "stage_e",
        "robustness",
        "accounting",
    ] {
        fs::create_dir_all(out.join(sub))?;
    }

    let mixed_default_off = {
        let p = SimParams::default();
        !p.use_mixed_structure_turnover
    };
    let preservation = gate_preservation(mixed_default_off);
    write_gate(&out, "preservation", &serde_json::to_value(&preservation)?)?;
    if !preservation.pass {
        let route = select_conclusion(
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            DynamicFailureClass::StaticDynamicParityDefect,
        );
        let result = json!({
            "primary_conclusion": route.conclusion,
            "stopped_at": "preservation",
            "preservation": preservation,
            "route": route,
            "next_execution_started": false,
        });
        atomic_write_json(&out.join("result.json"), &result)?;
        return Ok(result);
    }

    // Phase A — complete missing D-084 basin (no SKIP).
    eprintln!("D-085 Phase A: full dynamic basin matrix");
    let phase_a = run_basin_matrix(None);
    let phase_a_pass = if smoke_mode() {
        // Smoke does not claim scientific basin pass.
        false
    } else {
        basin_matrix_passes(&phase_a)
    };
    write_gate(
        &out,
        "d084_full_basin",
        &json!({
            "smoke": smoke_mode(),
            "phase_a_pass": phase_a_pass,
            "cohorts": phase_a,
            "eta": D085_D084_ETA,
            "k_phi_loss": D085_D084_K_PHI_LOSS,
            "skip_late_gates": false,
        }),
    )?;

    let parity = parity_checkpoints(&phase_a);
    let parity_ok = parity["pass"].as_bool().unwrap_or(false);
    write_gate(&out, "parity", &parity)?;

    let failure = if phase_a_pass {
        DynamicFailureClass::None
    } else {
        classify_dynamic_failure(&phase_a, parity_ok)
    };
    write_gate(
        &out,
        "failure_classification",
        &json!({
            "class": failure.as_str(),
            "parity_ok": parity_ok,
            "phase_a_pass": phase_a_pass,
        }),
    )?;

    let mut used_mechano = false;
    let mut mechano_label: Option<String> = None;
    let mut phase_c_pass = false;
    let mut phase_c: Vec<RadiusCohortResult> = Vec::new();

    if !phase_a_pass && parity_ok && !smoke_mode() {
        // Phase B/C — one mechanochemical architecture, ≤3 strength candidates.
        let (k_scale, s_scale) = measure_kappa_strain_scales();
        let cands = mechano_candidates_from_scales(k_scale, s_scale);
        write_gate(
            &out,
            "mechanochemical_candidates",
            &json!({
                "kappa_scale": k_scale,
                "strain_scale": s_scale,
                "candidates": cands,
                "equations": {
                    "f_kappa": "|κ|/(K_κ+|κ|)",
                    "f_s": "tanh(s/K_s)",
                    "r_plus": "r_plus0 * (1 + g_κ f_κ - g_s f_s)",
                    "r_minus": "r_minus0 * (1 + g_s f_s)",
                    "clamp": "[0.5, 2.0]",
                },
            }),
        )?;
        for cand in &cands {
            eprintln!("D-085 Phase C candidate {}", cand.label);
            used_mechano = true;
            mechano_label = Some(cand.label.to_string());
            let cohorts = run_basin_matrix(Some(cand));
            let pass = basin_matrix_passes(&cohorts);
            write_gate(
                &out,
                "mechanochemical_basin",
                &json!({
                    "candidate": cand,
                    "pass": pass,
                    "cohorts": cohorts,
                }),
            )?;
            if pass {
                phase_c = cohorts;
                phase_c_pass = true;
                break;
            }
            phase_c = cohorts;
        }
    } else {
        write_gate(
            &out,
            "mechanochemical_candidates",
            &json!({ "skipped": true, "reason": "Phase A passed or parity failed or smoke" }),
        )?;
        write_gate(
            &out,
            "mechanochemical_basin",
            &json!({ "skipped": true }),
        )?;
    }

    // Phase D placeholders — full Stage E only if a basin qualifies.
    let basin_ok = phase_a_pass || phase_c_pass;
    let energy_ok;
    let puncture_ok;
    let stage_e_pass;
    if basin_ok && !smoke_mode() {
        // Energy/waste from basin rows.
        let rows: Vec<&DynamicRunRow> = if phase_a_pass {
            phase_a.iter().flat_map(|c| c.rows.iter()).collect()
        } else {
            phase_c.iter().flat_map(|c| c.rows.iter()).collect()
        };
        energy_ok = rows.iter().all(|r| {
            r.a_retention >= D085_RETENTION_MIN
                && r.c_retention >= D085_RETENTION_MIN
                && r.w_mass.is_finite()
                && r.l_mass.is_finite()
                && r.b_mass.is_finite()
        });
        write_gate(
            &out,
            "energy_waste",
            &json!({ "pass": energy_ok, "n_rows": rows.len() }),
        )?;

        // Structural puncture: lawful local damage then recovery under selected params.
        puncture_ok = run_puncture_controls(if used_mechano {
            mechano_label
                .as_deref()
                .and_then(|l| {
                    let (k, s) = measure_kappa_strain_scales();
                    mechano_candidates_from_scales(k, s)
                        .into_iter()
                        .find(|c| c.label == l)
                })
        } else {
            None
        })?;
        write_gate(
            &out,
            "damage_controls",
            &json!({ "pass": puncture_ok }),
        )?;

        // Stage E: joint assay across R18/22/26 — structure basin + energy + puncture.
        let active_pass = if phase_a_pass {
            basin_matrix_passes(&phase_a)
        } else {
            phase_c_pass
        };
        stage_e_pass = active_pass && energy_ok && puncture_ok;
        write_gate(
            &out,
            "stage_e",
            &json!({
                "pass": stage_e_pass,
                "structure_basin": active_pass,
                "energy": energy_ok,
                "puncture": puncture_ok,
                "note": "Joint Stage E requires structure+energy+puncture under D-085 basin criteria",
            }),
        )?;
        write_gate(
            &out,
            "robustness",
            &json!({
                "multi_seed": true,
                "seeds": D085_NOISE_SEEDS,
                "radii": D085_BASIN_RADII,
            }),
        )?;
        write_gate(
            &out,
            "accounting",
            &json!({
                "all_accounting_ok": rows.iter().all(|r| r.accounting_ok),
            }),
        )?;

        let mut route = select_conclusion(
            phase_a_pass,
            used_mechano,
            phase_c_pass,
            stage_e_pass,
            energy_ok,
            puncture_ok,
            parity_ok,
            failure,
        );
        if let Some(label) = &mechano_label {
            route.mechano_label = Some(label.clone());
        }
        // Fix conclusion naming when Phase A alone passes Stage E.
        if phase_a_pass && stage_e_pass && !used_mechano {
            route.conclusion = D085Conclusion::D084DynamicBasinQualified.as_str().into();
            route.d008_status = "PASS_AFTER_D085".into();
            route.scientific_conclusion =
                "D-084 candidate establishes dynamic basin and Stage E recovery without mechanochemical feedback."
                    .into();
            route.next_directive = "Proceed to D-008 Stage F.".into();
            // Directive: Stage E pass tag is EdgeMechanochemical only when mechano used;
            // when D-084 alone recovers Stage E, use D085_D084_DYNAMIC_BASIN_QUALIFIED and PASS_AFTER_D085.
        }

        let result = json!({
            "project_directive": D085_PROJECT_ID,
            "agent_memory_directive": D085_AGENT_MEMORY_ID,
            "starting_commit": D085_STARTING_COMMIT,
            "starting_tag": D085_STARTING_TAG,
            "ending_commit_at_run": git_commit_hash(),
            "pending_record": D084_FIXED_RADIUS_PENDING,
            "primary_conclusion": route.conclusion,
            "failure_class": route.failure_class,
            "phase_a_pass": phase_a_pass,
            "phase_c_pass": phase_c_pass,
            "used_mechanochemical": used_mechano,
            "mechano_label": mechano_label,
            "parity_ok": parity_ok,
            "stage_e_pass": stage_e_pass,
            "d008_status": route.d008_status,
            "phase1_status": route.phase1_status,
            "production_verdict": route.production_verdict,
            "scientific_conclusion": route.scientific_conclusion,
            "next_directive": route.next_directive,
            "next_execution_started": false,
            "smoke": smoke_mode(),
            "route": route,
        });
        atomic_write_json(&out.join("result.json"), &result)?;
        atomic_write_json(
            &out.join("manifest.json"),
            &json!({
                "directive": D085_PROJECT_ID,
                "conclusion": result["primary_conclusion"],
            }),
        )?;
        return Ok(result);
    }

    // No basin / smoke path.
    energy_ok = false;
    puncture_ok = false;
    stage_e_pass = false;
    write_gate(&out, "energy_waste", &json!({ "skipped": true }))?;
    write_gate(&out, "damage_controls", &json!({ "skipped": true }))?;
    write_gate(&out, "stage_e", &json!({ "skipped": true }))?;
    write_gate(&out, "robustness", &json!({ "skipped": !smoke_mode() }))?;
    write_gate(&out, "accounting", &json!({ "skipped": true }))?;

    let mut route = select_conclusion(
        phase_a_pass,
        used_mechano,
        phase_c_pass,
        stage_e_pass,
        energy_ok,
        puncture_ok,
        parity_ok,
        failure,
    );
    if let Some(label) = &mechano_label {
        route.mechano_label = Some(label.clone());
    }
    if smoke_mode() {
        route.conclusion = "D085_SMOKE_PARTIAL".into();
        route.scientific_conclusion = "Smoke mode only; scientific conclusion withheld.".into();
    }

    let result = json!({
        "project_directive": D085_PROJECT_ID,
        "agent_memory_directive": D085_AGENT_MEMORY_ID,
        "starting_commit": D085_STARTING_COMMIT,
        "starting_tag": D085_STARTING_TAG,
        "ending_commit_at_run": git_commit_hash(),
        "pending_record": D084_FIXED_RADIUS_PENDING,
        "primary_conclusion": route.conclusion,
        "failure_class": route.failure_class,
        "phase_a_pass": phase_a_pass,
        "phase_c_pass": phase_c_pass,
        "used_mechanochemical": used_mechano,
        "mechano_label": mechano_label,
        "parity_ok": parity_ok,
        "stage_e_pass": stage_e_pass,
        "d008_status": route.d008_status,
        "phase1_status": route.phase1_status,
        "production_verdict": route.production_verdict,
        "scientific_conclusion": route.scientific_conclusion,
        "next_directive": route.next_directive,
        "next_execution_started": false,
        "smoke": smoke_mode(),
        "route": route,
        "phase_a_cohorts": phase_a,
    });
    atomic_write_json(&out.join("result.json"), &result)?;
    atomic_write_json(
        &out.join("manifest.json"),
        &json!({
            "directive": D085_PROJECT_ID,
            "conclusion": result["primary_conclusion"],
        }),
    )?;
    Ok(result)
}

fn run_puncture_controls(
    mechano: Option<MechanoCandidate>,
) -> Result<bool, Box<dyn std::error::Error>> {
    // Lawful local structural damage: suppress φ in an arc, require recovery with A/N/F,
    // and require no-A / no-N / no-F / production-knockout controls to fail recovery.
    let mut base = run_damage_case(22.0, 2, mechano.as_ref(), DamageControl::None);
    let recover = base.recovery_frac >= 0.95;
    let no_a = run_damage_case(22.0, 2, mechano.as_ref(), DamageControl::NoA).recovery_frac < 0.50;
    let no_n = run_damage_case(22.0, 2, mechano.as_ref(), DamageControl::NoN).recovery_frac < 0.50;
    let no_f = run_damage_case(22.0, 2, mechano.as_ref(), DamageControl::NoF).recovery_frac < 0.50;
    let no_prod =
        run_damage_case(22.0, 2, mechano.as_ref(), DamageControl::NoProduction).recovery_frac < 0.50;
    let _ = &mut base;
    Ok(recover && no_a && no_n && no_f && no_prod)
}

#[derive(Clone, Copy)]
enum DamageControl {
    None,
    NoA,
    NoN,
    NoF,
    NoProduction,
}

struct DamageOutcome {
    recovery_frac: f64,
}

fn run_damage_case(
    radius: f64,
    seed: u64,
    mechano: Option<&MechanoCandidate>,
    control: DamageControl,
) -> DamageOutcome {
    let mut params = organism_params(seed, mechano);
    match control {
        DamageControl::NoA => {
            // Leave A but zero activation rate.
            params.k_d008_activation = 0.0;
        }
        DamageControl::NoN => params.n_reservoir = 0.0,
        DamageControl::NoF => params.f_reservoir = 0.0,
        DamageControl::NoProduction => params.k_d008_structure = 0.0,
        DamageControl::None => {}
    }
    let mut sim = Simulation::new(params);
    sim.dt_cap = 0.005;
    sim.set_structure_evolution_mode(StructureEvolutionMode::DynamicStructure);
    seed_v7_compartment(&mut sim, radius, D053_THETA);
    hold_exterior(&mut sim);
    // Settle briefly.
    for _ in 0..200 {
        hold_exterior(&mut sim);
        let _ = sim.step();
    }
    let mass0 = field_mass(&sim.grid, &sim.fields.structure);
    // Puncture: clear an angular sector of structure near the interface.
    let w = sim.grid.width;
    for idx in 0..sim.fields.structure.len() {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        let i = idx % w;
        let j = idx / w;
        let x = i as f64 - sim.grid.cx;
        let y = j as f64 - sim.grid.cy;
        let ang = y.atan2(x);
        let r = (x * x + y * y).sqrt();
        if ang.abs() < 0.35 && (r - radius).abs() < 3.0 {
            sim.fields.structure[idx] *= 0.05;
        }
    }
    sim.fields.copy_current_to_next();
    let mass_damaged = field_mass(&sim.grid, &sim.fields.structure);
    let lost = (mass0 - mass_damaged).max(0.0);
    for _ in 0..2_500 {
        hold_exterior(&mut sim);
        if matches!(control, DamageControl::NoA) {
            for a in sim.fields.activated.iter_mut() {
                *a = 0.0;
            }
        }
        let _ = sim.step();
    }
    let mass1 = field_mass(&sim.grid, &sim.fields.structure);
    let recovered = (mass1 - mass_damaged).max(0.0);
    let recovery_frac = if lost <= 1e-12 {
        1.0
    } else {
        (recovered / lost).clamp(0.0, 2.0)
    };
    DamageOutcome { recovery_frac }
}
