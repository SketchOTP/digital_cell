//! D-075 cellwise exposure-gated membrane requalification pipeline.
//!
//! Observer-only. Frozen D-070…D-074 biology and `SEED_CAPACITY_CONTRACT_V1`.
//! Authoritative qualification metric is exact effective exposure
//! `E_i = -Σ ln(c_{i,n})`; continuous `Λ_i` stays diagnostic.

use crate::d013::atomic_write_json;
use crate::d025::v7_base_params;
use chemistry_core::candidate_identity::sha256_hex;
use chemistry_core::config::{EquationVersion, SimParams, StructureEvolutionMode};
use chemistry_core::d039_analysis::v8_schema3_params;
use chemistry_core::d049_analysis::d049_frozen_params;
use chemistry_core::d050_analysis::ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME;
use chemistry_core::d053_analysis::{
    apply_delivery_repair, DeliveryRepairPair, D053_FITTED_K_C, D053_FITTED_V_A, D053_F_REF,
    D053_N_REF,
};
use chemistry_core::d055_analysis::{D055_FROZEN_M_BETA, D055_FROZEN_M_EXT};
use chemistry_core::d063_analysis::{
    generate_phi, seed_mature_s_on_interfaces, GeometrySpec, D063_PHI_INTERIOR,
};
use chemistry_core::d069_analysis::split_accepted_exchange;
use chemistry_core::d070_analysis::{
    migrate_policy_d_authorized_reconstruction, occupancy_theta, MigrationPolicy,
};
use chemistry_core::d071_analysis::PrecursorRegulationParams;
use chemistry_core::d072_analysis::DAMAGE_FRACTION;
use chemistry_core::d073_analysis::{
    activity_from_concentration, concentration_for_activity, equilibrium_occupancy,
    interface_p_within_tol, p_required, D070_LAWFUL_MAINTENANCE_OCCUPANCY,
};
use chemistry_core::d074_analysis::{
    d073_expected_recoveries, D074_K_EQ, D074_STARTING_COMMIT as D074_START_COMMIT,
    D074_STARTING_TAG as D074_START_TAG,
};
use chemistry_core::d075_analysis::*;
use chemistry_core::interventions::apply_declared_membrane_arc_damage;
use chemistry_core::membrane::membrane_catalyst_saturation;
use chemistry_core::surface_density::{
    compute_interface_geometry, total_surface_mass, InterfaceGeometryCell, SURFACE_CAPACITY_FLOOR,
};
use chemistry_core::{field_mass, Simulation};
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// ─── env knobs ────────────────────────────────────────────────────────────────

fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
        .max(1)
}
fn env_f64(key: &str, default: f64) -> f64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
fn env_flag(key: &str) -> bool {
    std::env::var(key)
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}
fn settle_steps() -> u64 {
    env_u64("D075_SETTLE", 400)
}
fn max_accepted() -> u64 {
    env_u64("D075_MAX_ACCEPTED", 200_000)
}
fn max_time() -> f64 {
    env_f64("D075_MAX_TIME", 5000.0).max(0.0)
}
fn dt_cap() -> f64 {
    env_f64("D075_DT_CAP", 0.05).max(1e-4)
}
fn skip_late() -> bool {
    env_flag("D075_SKIP_LATE_GATES")
}
fn fixed_p_max_accepted() -> u64 {
    std::env::var("D075_FIXED_P_MAX_ACCEPTED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(max_accepted)
        .max(1)
}
fn maint_max_accepted() -> u64 {
    std::env::var("D075_MAINT_MAX_ACCEPTED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(max_accepted)
        .max(1)
}
fn selected_m_p() -> f64 {
    let from_env = std::env::var("D075_SELECTED_M_P")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .or_else(|| {
            std::env::var("D071_SELECTED_M_P")
                .ok()
                .and_then(|v| v.parse::<f64>().ok())
        });
    from_env.unwrap_or(D075_SELECTED_M_P)
}

// ─── path/git helpers ────────────────────────────────────────────────────────

fn resolve_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path)
    }
}

/// If `path` does not exist and we're targeting the default d075 artifact
/// location, prefer a symlink into `/mnt/storage1tb/...` to keep NVMe free.
fn prepare_artifact_root(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if path.exists() {
        return Ok(());
    }
    let archive_root = PathBuf::from(
        "/mnt/storage1tb/cache/project-artifacts/digital_cell/experiments/generated",
    );
    // Match final-component behaviour: symlink only when the tail is `d075`.
    let is_d075_leaf = path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s == "d075")
        .unwrap_or(false);
    if is_d075_leaf && archive_root.is_dir() {
        let target = archive_root.join("d075");
        fs::create_dir_all(&target)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&target, path).ok();
        }
        if path.exists() {
            return Ok(());
        }
    }
    fs::create_dir_all(path)?;
    Ok(())
}

fn write_json(dir: &Path, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(dir)?;
    atomic_write_json(&dir.join("result.json"), value)?;
    Ok(())
}

fn git_output(args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .current_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        })
}

fn file_sha256(path: &Path) -> Option<String> {
    fs::read(path).ok().map(|b| sha256_hex(&b))
}

// ─── params ─────────────────────────────────────────────────────────────────

fn baseline_params() -> SimParams {
    let base = v7_base_params().unwrap_or_else(|_| v8_schema3_params());
    let mut params = d049_frozen_params(&base);
    params.equation_version = EquationVersion::MembraneMetabolismV13CatalystSaturatingActivation;
    params.activation_schema = ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME;
    params.k_d008_activation = D053_FITTED_V_A;
    params.k_c_activation = D053_FITTED_K_C;
    params.n_ref_activation = D053_N_REF;
    params.f_ref_activation = D053_F_REF;
    apply_delivery_repair(
        &mut params,
        DeliveryRepairPair {
            m_ext: D055_FROZEN_M_EXT,
            m_beta: D055_FROZEN_M_BETA,
        },
    );
    params
}

// ─── sim helpers ────────────────────────────────────────────────────────────

fn geometry(sim: &Simulation) -> Vec<InterfaceGeometryCell> {
    let mut g = vec![InterfaceGeometryCell::default(); sim.fields.structure.len()];
    compute_interface_geometry(&sim.grid, &sim.fields.structure, sim.params.eta_n, &mut g);
    g
}

fn interface_cell_indices(sim: &Simulation) -> Vec<usize> {
    let g = geometry(sim);
    (0..g.len())
        .filter(|&i| sim.grid.in_dish(i) && g[i].delta > sim.params.delta_floor)
        .collect()
}

fn hold_exterior(sim: &mut Simulation) {
    for i in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(i) && sim.fields.structure[i] < D063_PHI_INTERIOR {
            sim.fields.nutrient[i] = sim.params.n_reservoir;
            sim.fields.fuel[i] = sim.params.f_reservoir;
        }
    }
}

fn hold_interface_activity_cached(sim: &mut Simulation, p_activity: f64, iface: &[usize]) {
    let conc = concentration_for_activity(p_activity, sim.params.p_reference);
    for &i in iface {
        sim.fields.precursor[i] = conc;
    }
}

fn hold_interface_activity(sim: &mut Simulation, p_activity: f64) {
    let iface = interface_cell_indices(sim);
    hold_interface_activity_cached(sim, p_activity, &iface);
    sim.fields.copy_current_to_next();
}

fn capacity_snapshot(sim: &Simulation) -> (f64, f64) {
    let g = geometry(sim);
    let mut capacity = 0.0;
    let mut max_theta: f64 = 0.0;
    for i in 0..g.len() {
        if sim.grid.in_dish(i) && g[i].delta > sim.params.delta_floor {
            capacity += g[i].delta * sim.params.gamma_max;
            max_theta = max_theta.max(occupancy_theta(
                sim.fields.membrane[i],
                g[i].delta,
                sim.params.gamma_max,
            ));
        }
    }
    (capacity, max_theta)
}

fn absolute_occupancy(sim: &Simulation) -> f64 {
    let (capacity, _) = capacity_snapshot(sim);
    if capacity <= EPS {
        0.0
    } else {
        (total_surface_mass(&sim.grid, &sim.fields.membrane) / capacity).min(1.0)
    }
}

fn seed_b_policy_d(sim: &mut Simulation, spec: &GeometrySpec) -> Value {
    let phi = generate_phi(&sim.grid, spec);
    let mut geometry = vec![InterfaceGeometryCell::default(); phi.len()];
    compute_interface_geometry(&sim.grid, &phi, sim.params.eta_n, &mut geometry);
    let mut membrane = seed_mature_s_on_interfaces(&sim.grid, &phi, 1.0);
    let precursor: Vec<f64> = phi
        .iter()
        .map(|&v| if v >= D063_PHI_INTERIOR { 0.05 } else { 0.0 })
        .collect();
    let migration = migrate_policy_d_authorized_reconstruction(
        &sim.grid,
        &geometry,
        &mut membrane,
        &precursor,
        sim.params.delta_floor,
        sim.params.gamma_max,
        1.0,
        "d075_seed_b_policy_d",
    );
    for i in 0..phi.len() {
        if !sim.grid.in_dish(i) {
            continue;
        }
        sim.fields.structure[i] = phi[i];
        sim.fields.membrane[i] = membrane[i];
        sim.fields.precursor[i] = precursor[i];
        if phi[i] >= D063_PHI_INTERIOR {
            sim.fields.catalyst[i] = 0.4;
            sim.fields.activated[i] = 0.5;
            sim.fields.nutrient[i] = 0.4;
            sim.fields.fuel[i] = 0.4;
            sim.fields.waste[i] = 0.5;
        } else {
            sim.fields.catalyst[i] = 0.0;
            sim.fields.activated[i] = 0.0;
            sim.fields.nutrient[i] = sim.params.n_reservoir;
            sim.fields.fuel[i] = sim.params.f_reservoir;
            sim.fields.waste[i] = sim.params.w_reservoir;
        }
    }
    sim.fields.copy_current_to_next();
    json!({
        "seed_kind": "BPolicyD",
        "migration": migration,
        "policy": MigrationPolicy::AuthorizedMaterialReconstruction
    })
}

fn clone_from_template(template: &Simulation) -> Simulation {
    let mut sim = Simulation::new(template.params.clone());
    sim.dt_cap = dt_cap();
    sim.dt = dt_cap();
    sim.set_structure_evolution_mode(StructureEvolutionMode::FixedGeometry);
    let snap = template.snapshot();
    sim.restore_snapshot(&snap);
    sim.fields.copy_current_to_next();
    sim
}

fn configure_exchange_only(sim: &mut Simulation) {
    sim.params.reactions_enabled = false;
    sim.params.k_precursor = 0.0;
    sim.params.k_structure = 0.0;
    sim.params.k_rep = 0.0;
    sim.params.d_p = 0.0;
    sim.params.k_gamma_decay = 0.0;
}

fn configure_exchange_only_isolated(sim: &mut Simulation) {
    configure_exchange_only(sim);
    sim.params.d_gamma = 0.0;
}

fn field_hash(sim: &Simulation) -> String {
    let mut bytes = Vec::new();
    for v in &sim.fields.membrane {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    for v in &sim.fields.precursor {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    sha256_hex(&bytes)
}

// ─── observer ───────────────────────────────────────────────────────────────

/// Shared exposure observer over interface-supported cells only.
struct ExposureObserver {
    cells: Vec<usize>,
    state: Vec<CellExposureState>,
    accepted_sim_time: f64,
    accepted_steps: u64,
    rejected_attempts: u64,
    /// Snapshot of geometry.delta captured at attach — geometry is fixed.
    delta: Vec<f64>,
    /// Snapshot of gamma_max at attach.
    gamma_max: f64,
}

impl ExposureObserver {
    fn attach(sim: &Simulation) -> Self {
        let g = geometry(sim);
        let mut cells = Vec::new();
        let mut delta = Vec::new();
        for i in 0..g.len() {
            if sim.grid.in_dish(i) && g[i].delta > sim.params.delta_floor {
                cells.push(i);
                delta.push(g[i].delta);
            }
        }
        let state = vec![CellExposureState::default(); cells.len()];
        Self {
            cells,
            state,
            accepted_sim_time: 0.0,
            accepted_steps: 0,
            rejected_attempts: 0,
            delta,
            gamma_max: sim.params.gamma_max,
        }
    }

    fn observe_accepted(&mut self, sim: &Simulation, pre_p: &[f64], pre_s: &[f64], pre_c: &[f64], dt: f64) {
        self.accepted_sim_time += dt.max(0.0);
        self.accepted_steps = self.accepted_steps.saturating_add(1);
        for (slot, &idx) in self.cells.iter().enumerate() {
            let delta = self.delta[slot];
            let q = membrane_catalyst_saturation(pre_c[idx], &sim.params);
            let p_act = activity_from_concentration(pre_p[idx], sim.params.p_reference);
            let kind = classify_production_exchange_step(
                pre_s[idx],
                pre_p[idx],
                delta,
                q,
                sim.params.k_exchange,
                sim.params.k_exchange_eq,
                sim.params.p_reference,
                self.gamma_max,
                sim.params.delta_floor,
                dt,
            );
            let lam = lambda_i(sim.params.k_exchange, q, sim.params.k_exchange_eq, p_act);
            self.state[slot].observe_attempt(kind, lam, dt);
        }
    }

    fn observe_rejected(&mut self) {
        self.rejected_attempts = self.rejected_attempts.saturating_add(1);
        for st in &mut self.state {
            st.observe_attempt(IntegratorKind::Rejected, 0.0, 0.0);
        }
    }

    fn snapshot(&self) -> ExposureObserverSnapshot {
        ExposureObserverSnapshot {
            cells: self.state.clone(),
            accepted_sim_time: self.accepted_sim_time,
            accepted_steps: self.accepted_steps,
            rejected_attempts: self.rejected_attempts,
        }
    }

    /// Build cell tuples for `qualify_exposure_capacity` limited to indices in `mask`.
    /// `mask=None` means all observed cells.
    fn qualification_input(
        &self,
        mask: Option<&[bool]>,
    ) -> Vec<(f64, f64, bool, f64, f64, f64)> {
        let mut out = Vec::new();
        for (slot, &idx) in self.cells.iter().enumerate() {
            if let Some(m) = mask {
                if !m.get(idx).copied().unwrap_or(false) {
                    continue;
                }
            }
            let delta = self.delta[slot];
            let cap = delta * self.gamma_max;
            let supported = cap > SURFACE_CAPACITY_FLOOR;
            let st = &self.state[slot];
            out.push((cap, st.e_exact, supported, st.explicit_e, st.backward_euler_e, st.lambda_cum));
        }
        out
    }

    #[allow(dead_code)]
    fn total_capacity_observed(&self) -> f64 {
        self.delta.iter().map(|d| d * self.gamma_max).sum()
    }
}

// ─── stepping ────────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
enum HoldMode {
    None,
    FixedInterfaceP(f64),
}

struct StepStats {
    accepted: u64,
    rejected: u64,
    time: f64,
    exchange_net: f64,
    ads: f64,
    des: f64,
}

/// Run steps with a shared observer collecting per-cell exposure.
fn run_steps_observed(
    sim: &mut Simulation,
    obs: &mut ExposureObserver,
    accepted_limit: u64,
    time_limit: Option<f64>,
    hold: HoldMode,
    // When set, stop once this capacity subset reaches exposure qualification.
    // `None` means qualify the full observed interface.
    exposure_mask: Option<&[bool]>,
) -> StepStats {
    let start_time = sim.sim_time;
    let mut accepted = 0u64;
    let mut rejected = 0u64;
    let mut exchange = 0.0;
    let mut ads = 0.0;
    let mut des = 0.0;
    let iface = match hold {
        HoldMode::FixedInterfaceP(_) => Some(interface_cell_indices(sim)),
        HoldMode::None => None,
    };
    // Check qualification every N accepted steps to avoid O(cells) each step.
    let qualify_every = env_u64("D075_QUALIFY_EVERY", 50);
    while accepted < accepted_limit
        && time_limit
            .map(|limit| sim.sim_time - start_time < limit)
            .unwrap_or(true)
    {
        hold_exterior(sim);
        if let (HoldMode::FixedInterfaceP(p), Some(ref cells)) = (hold, &iface) {
            hold_interface_activity_cached(sim, p, cells);
        }
        let pre_p: Vec<f64> = sim.fields.precursor.clone();
        let pre_s: Vec<f64> = sim.fields.membrane.clone();
        let pre_c: Vec<f64> = sim.fields.catalyst.clone();
        let dt_attempt = sim.dt;
        if sim.step() {
            accepted += 1;
            let net = sim.surface_accounting.last_step.exchange_net;
            exchange += net;
            let (a, d) = split_accepted_exchange(net);
            ads += a;
            des += d;
            obs.observe_accepted(sim, &pre_p, &pre_s, &pre_c, dt_attempt);
            if accepted % qualify_every == 0
                && qualify_exposure_capacity(&obs.qualification_input(exposure_mask)).qualifies
            {
                break;
            }
        } else {
            rejected += 1;
            obs.observe_rejected();
            if rejected > accepted_limit.saturating_mul(10) {
                break;
            }
        }
    }
    // Final qualification check (in case we exited on caps just after a qualify).
    let _ = qualify_exposure_capacity(&obs.qualification_input(exposure_mask));
    StepStats {
        accepted,
        rejected,
        time: sim.sim_time - start_time,
        exchange_net: exchange,
        ads,
        des,
    }
}

/// Like `run_steps_observed`, but when `require_recovery` is set, keep going after
/// exposure qualification until mature mass recovers to `REPAIR_FRACTION_GATE`
/// (or caps). Diagnostic controls may set `require_recovery=false`.
fn run_steps_observed_until(
    sim: &mut Simulation,
    obs: &mut ExposureObserver,
    accepted_limit: u64,
    time_limit: Option<f64>,
    hold: HoldMode,
    exposure_mask: Option<&[bool]>,
    require_recovery: bool,
    pre_s: f64,
) -> StepStats {
    let start_time = sim.sim_time;
    let mut accepted = 0u64;
    let mut rejected = 0u64;
    let mut exchange = 0.0;
    let mut ads = 0.0;
    let mut des = 0.0;
    let iface = match hold {
        HoldMode::FixedInterfaceP(_) => Some(interface_cell_indices(sim)),
        HoldMode::None => None,
    };
    let qualify_every = env_u64("D075_QUALIFY_EVERY", 50);
    let mut exposure_ok = false;
    while accepted < accepted_limit
        && time_limit
            .map(|limit| sim.sim_time - start_time < limit)
            .unwrap_or(true)
    {
        hold_exterior(sim);
        if let (HoldMode::FixedInterfaceP(p), Some(ref cells)) = (hold, &iface) {
            hold_interface_activity_cached(sim, p, cells);
        }
        let pre_p: Vec<f64> = sim.fields.precursor.clone();
        let pre_s_field: Vec<f64> = sim.fields.membrane.clone();
        let pre_c: Vec<f64> = sim.fields.catalyst.clone();
        let dt_attempt = sim.dt;
        if sim.step() {
            accepted += 1;
            let net = sim.surface_accounting.last_step.exchange_net;
            exchange += net;
            let (a, d) = split_accepted_exchange(net);
            ads += a;
            des += d;
            obs.observe_accepted(sim, &pre_p, &pre_s_field, &pre_c, dt_attempt);
            if accepted % qualify_every == 0 {
                exposure_ok =
                    qualify_exposure_capacity(&obs.qualification_input(exposure_mask)).qualifies;
                if exposure_ok {
                    if !require_recovery {
                        break;
                    }
                    let ratio = total_surface_mass(&sim.grid, &sim.fields.membrane) / pre_s.max(EPS);
                    if ratio >= REPAIR_FRACTION_GATE {
                        break;
                    }
                }
            }
        } else {
            rejected += 1;
            obs.observe_rejected();
            if rejected > accepted_limit.saturating_mul(10) {
                break;
            }
        }
    }
    let _ = exposure_ok;
    StepStats {
        accepted,
        rejected,
        time: sim.sim_time - start_time,
        exchange_net: exchange,
        ads,
        des,
    }
}

/// Run steps without an observer (settle/initial).
fn run_steps(
    sim: &mut Simulation,
    accepted_limit: u64,
    time_limit: Option<f64>,
    hold: HoldMode,
) -> StepStats {
    let start_time = sim.sim_time;
    let mut accepted = 0u64;
    let mut rejected = 0u64;
    let mut exchange = 0.0;
    let mut ads = 0.0;
    let mut des = 0.0;
    let iface = match hold {
        HoldMode::FixedInterfaceP(_) => Some(interface_cell_indices(sim)),
        HoldMode::None => None,
    };
    while accepted < accepted_limit
        && time_limit
            .map(|limit| sim.sim_time - start_time < limit)
            .unwrap_or(true)
    {
        hold_exterior(sim);
        if let (HoldMode::FixedInterfaceP(p), Some(ref cells)) = (hold, &iface) {
            hold_interface_activity_cached(sim, p, cells);
        }
        if sim.step() {
            accepted += 1;
            let net = sim.surface_accounting.last_step.exchange_net;
            exchange += net;
            let (a, d) = split_accepted_exchange(net);
            ads += a;
            des += d;
        } else {
            rejected += 1;
            if rejected > accepted_limit.saturating_mul(10) {
                break;
            }
        }
    }
    StepStats {
        accepted,
        rejected,
        time: sim.sim_time - start_time,
        exchange_net: exchange,
        ads,
        des,
    }
}

fn settled(params: SimParams, settle: u64, spec: &GeometrySpec) -> (Simulation, Value) {
    let mut sim = Simulation::new(params);
    sim.dt_cap = 0.005;
    sim.set_structure_evolution_mode(StructureEvolutionMode::FixedGeometry);
    let seed = seed_b_policy_d(&mut sim, spec);
    let st = run_steps(&mut sim, settle, None, HoldMode::None);
    (
        sim,
        json!({
            "seed": seed,
            "settle_accepted": st.accepted,
            "settle_rejected": st.rejected,
            "settle_sim_time": st.time
        }),
    )
}

fn collect_damaged_mask(sim: &Simulation) -> Vec<bool> {
    let g = geometry(sim);
    let mut out = vec![false; g.len()];
    for i in 0..g.len() {
        if !sim.grid.in_dish(i) || g[i].delta <= sim.params.delta_floor {
            continue;
        }
        let theta = occupancy_theta(sim.fields.membrane[i], g[i].delta, sim.params.gamma_max);
        if theta < 0.5 {
            out[i] = true;
        }
    }
    out
}

fn damage_and_sync(sim: &mut Simulation) -> Value {
    let s0 = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let w0 = field_mass(&sim.grid, &sim.fields.waste);
    let capacity0 = capacity_snapshot(sim).0;
    let report = apply_declared_membrane_arc_damage(&sim.grid, &mut sim.fields, DAMAGE_FRACTION);
    let s1 = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let w1 = field_mass(&sim.grid, &sim.fields.waste);
    sim.fields.copy_current_to_next();
    json!({
        "report": report,
        "delta_s": s1 - s0,
        "delta_w": w1 - w0,
        "s_w_conservation": ((s1 - s0) + (w1 - w0)).abs() <= ACCOUNTING_TOL,
        "capacity_before": capacity0,
        "capacity_after": capacity_snapshot(sim).0,
        "occupancy_after_damage": absolute_occupancy(sim)
    })
}

fn interface_p_stats(sim: &Simulation) -> (f64, f64, usize) {
    let g = geometry(sim);
    let mut sum = 0.0;
    let mut n = 0usize;
    let mut min_p = f64::INFINITY;
    for i in 0..g.len() {
        if sim.grid.in_dish(i) && g[i].delta > sim.params.delta_floor {
            let p = activity_from_concentration(sim.fields.precursor[i], sim.params.p_reference);
            sum += p;
            min_p = min_p.min(p);
            n += 1;
        }
    }
    let mean = if n == 0 { 0.0 } else { sum / n as f64 };
    (mean, if n == 0 { 0.0 } else { min_p }, n)
}

fn damaged_arc_occupancy(sim: &Simulation) -> (f64, f64) {
    let g = geometry(sim);
    let mut d_sum = 0.0;
    let mut d_cap = 0.0;
    let mut u_sum = 0.0;
    let mut u_cap = 0.0;
    for i in 0..g.len() {
        if !sim.grid.in_dish(i) || g[i].delta <= sim.params.delta_floor {
            continue;
        }
        let cap = g[i].delta * sim.params.gamma_max;
        let s = sim.fields.membrane[i].max(0.0);
        let theta = occupancy_theta(s, g[i].delta, sim.params.gamma_max);
        if theta < 0.5 {
            d_sum += s;
            d_cap += cap;
        } else {
            u_sum += s;
            u_cap += cap;
        }
    }
    (
        if d_cap <= EPS { 0.0 } else { d_sum / d_cap },
        if u_cap <= EPS { 0.0 } else { u_sum / u_cap },
    )
}

#[allow(dead_code)]
fn mean_interior_precursor(sim: &Simulation) -> f64 {
    let mut sum = 0.0;
    let mut n = 0usize;
    for i in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(i) && sim.fields.structure[i] >= D063_PHI_INTERIOR {
            sum += sim.fields.precursor[i].max(0.0);
            n += 1;
        }
    }
    if n == 0 {
        0.0
    } else {
        sum / n as f64
    }
}

// ─── Gate 2: synthetic calibration ──────────────────────────────────────────

fn synthetic_calibration() -> Value {
    // Frozen exchange only, uniform p/q/capacity: drive to target E and verify
    // residual ∝ e^{-E} across analytical / runtime-ledger / observer.
    let mut per_target = Vec::new();
    let mut all_ok = true;
    let lam_dt_cases = [(0.20, 0.05, "moderate"), (0.10, 0.05, "slow"), (0.40, 0.02, "fast")];
    for (lam, dt, tag) in lam_dt_cases {
        for target_e in [1.0, 3.0, 5.0] {
            let mut st = CellExposureState::default();
            let mut product = 1.0;
            let mut steps = 0u64;
            while st.e_exact < target_e && steps < 10_000 {
                st.observe_attempt(IntegratorKind::BackwardEuler, lam, dt);
                product *= backward_euler_contraction(lam, dt);
                steps += 1;
            }
            let analytical = synthetic_residual_ratio(st.e_exact);
            let runtime_ledger = product;
            let observer = (-st.e_exact).exp();
            let err_a = (analytical - runtime_ledger).abs();
            let err_b = (analytical - observer).abs();
            let ok = err_a < PARITY_TOL && err_b < PARITY_TOL;
            all_ok &= ok;
            per_target.push(json!({
                "tag": tag,
                "lambda": lam,
                "dt": dt,
                "target_e": target_e,
                "achieved_e": st.e_exact,
                "steps": steps,
                "analytical_residual": analytical,
                "runtime_ledger_residual": runtime_ledger,
                "observer_predicted": observer,
                "abs_err_analytical_vs_runtime": err_a,
                "abs_err_analytical_vs_observer": err_b,
                "ok": ok,
            }));
        }
    }
    // Explicit-Euler stability + parity within stable dt.
    let ex_lam = 0.5;
    let ex_dt = 0.1; // λdt = 0.05 stable
    let mut ex_st = CellExposureState::default();
    for _ in 0..20 {
        ex_st.observe_attempt(IntegratorKind::ExplicitEuler, ex_lam, ex_dt);
    }
    let ex_residual = synthetic_residual_ratio(ex_st.e_exact);
    let ex_prod = explicit_contraction(ex_lam, ex_dt).powi(20);
    let ex_ok = (ex_residual - ex_prod).abs() < PARITY_TOL;
    all_ok &= ex_ok;
    json!({
        "gate": "synthetic_calibration",
        "pass": all_ok,
        "backward_euler_cases": per_target,
        "explicit_euler": {
            "lambda": ex_lam,
            "dt": ex_dt,
            "steps": 20,
            "e_exact": ex_st.e_exact,
            "analytical_residual": ex_residual,
            "runtime_ledger": ex_prod,
            "ok": ex_ok,
        },
        "parity_tol": PARITY_TOL,
        "note": "analytical exp(-E) must match runtime product of contractions and observer within PARITY_TOL",
    })
}

// ─── Gate 3: fixed-P requalification ────────────────────────────────────────

fn fixed_p_assay(
    name: &str,
    template: &Simulation,
    pre_s: f64,
    p_activity: f64,
    damaged_mask: &[bool],
) -> Value {
    let mut sim = clone_from_template(template);
    // Isolate local exchange so surface diffusion cannot drain refill (D-074 parity).
    configure_exchange_only_isolated(&mut sim);
    hold_interface_activity(&mut sim, p_activity);
    let mut obs = ExposureObserver::attach(&sim);
    let require_recovery = name != "p_0_38";
    let st = run_steps_observed_until(
        &mut sim,
        &mut obs,
        fixed_p_max_accepted(),
        Some(max_time()),
        HoldMode::FixedInterfaceP(p_activity),
        Some(damaged_mask),
        require_recovery,
        pre_s,
    );
    let recovered_s = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let ratio = recovered_s / pre_s.max(EPS);
    let analytical_eq = equilibrium_occupancy(p_activity, sim.params.k_exchange_eq);
    let (mean_p, min_p, n_iface) = interface_p_stats(&sim);
    let p_ok = interface_p_within_tol(mean_p, p_activity)
        && interface_p_within_tol(min_p, p_activity);
    let (damaged_occ, undamaged_occ) = damaged_arc_occupancy(&sim);
    let damaged_qual = qualify_exposure_capacity(&obs.qualification_input(Some(damaged_mask)));
    let all_qual = qualify_exposure_capacity(&obs.qualification_input(None));
    json!({
        "name": name,
        "diagnostic_nonconservative_fixed_p": true,
        "exchange_isolated": true,
        "intended_p": p_activity,
        "mean_interface_p": mean_p,
        "min_interface_p": min_p,
        "interface_cells": n_iface,
        "p_within_2pct": p_ok,
        "recovery_ratio": ratio,
        "recovers": ratio >= REPAIR_FRACTION_GATE,
        "damaged_arc_occupancy": damaged_occ,
        "undamaged_occupancy": undamaged_occ,
        "total_mature_s": recovered_s,
        "adsorption": st.ads,
        "desorption": st.des,
        "exchange_net": st.exchange_net,
        "sim_time": st.time,
        "accepted": st.accepted,
        "rejected": st.rejected,
        "analytical_theta_eq": analytical_eq,
        "runtime_occupancy": absolute_occupancy(&sim),
        "field_hash": field_hash(&sim),
        "damaged_exposure": damaged_qual,
        "all_interface_exposure": all_qual,
        "damaged_exposure_qualifies": damaged_qual.qualifies,
        "all_exposure_qualifies": all_qual.qualifies,
        "observer_snapshot": obs.snapshot(),
        "authority": "diagnostic_nonconservative_fixed_p_exchange_only"
    })
}

// ─── Gates 4/5: maintenance + damage repair ─────────────────────────────────

#[derive(Clone, Copy)]
enum MaintConfig {
    Constitutive,
    RegulatedReduced,
    KPrecursorZero,
}

impl MaintConfig {
    fn label(self) -> &'static str {
        match self {
            Self::Constitutive => "constitutive",
            Self::RegulatedReduced => "regulated_reduced",
            Self::KPrecursorZero => "k_precursor_zero",
        }
    }
    fn apply(self, base: &SimParams, m_p: f64) -> SimParams {
        let mut p = base.clone();
        match self {
            Self::Constitutive => {}
            Self::RegulatedReduced => {
                PrecursorRegulationParams::reduced(m_p).apply_to(&mut p);
            }
            Self::KPrecursorZero => {
                p.k_precursor = 0.0;
            }
        }
        p
    }
}

#[derive(Clone)]
struct MaintenanceOutcome {
    label: &'static str,
    accepted: u64,
    rejected: u64,
    sim_time: f64,
    occupancy_0: f64,
    occupancy_1: f64,
    a_ret: f64,
    c_ret: f64,
    s_ret: f64,
    p_start: f64,
    p_end: f64,
    p_slope: f64,
    mean_p: f64,
    ads: f64,
    des: f64,
    exchange_net: f64,
    coverage: f64,
    eq_occ_from_local_p: f64,
    exposure_qual: ExposureQualificationReport,
    numerical_terminal: bool,
    biological_terminal: bool,
    class: LongHorizonClass,
    field_hash: String,
    /// Post-maintenance simulation state used to seed damage repair.
    template_snapshot: Value,
}

fn boundary_coverage(sim: &Simulation) -> f64 {
    let g = geometry(sim);
    let mut total = 0usize;
    let mut ok = 0usize;
    for i in 0..g.len() {
        if !sim.grid.in_dish(i) || g[i].delta <= sim.params.delta_floor {
            continue;
        }
        total += 1;
        let theta = occupancy_theta(sim.fields.membrane[i], g[i].delta, sim.params.gamma_max);
        if theta >= 0.5 {
            ok += 1;
        }
    }
    if total == 0 {
        0.0
    } else {
        ok as f64 / total as f64
    }
}

fn run_maintenance(
    label: &'static str,
    params: SimParams,
    spec: &GeometrySpec,
    settle: u64,
) -> (Simulation, MaintenanceOutcome) {
    let (mut sim, _) = settled(params, settle, spec);
    let a0 = field_mass(&sim.grid, &sim.fields.activated);
    let c0 = field_mass(&sim.grid, &sim.fields.catalyst);
    let p0 = field_mass(&sim.grid, &sim.fields.precursor);
    let s0 = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let occ0 = absolute_occupancy(&sim);
    let mut obs = ExposureObserver::attach(&sim);
    let st = run_steps_observed(
        &mut sim,
        &mut obs,
        maint_max_accepted(),
        Some(max_time()),
        HoldMode::None,
        None,
    );
    let a1 = field_mass(&sim.grid, &sim.fields.activated);
    let c1 = field_mass(&sim.grid, &sim.fields.catalyst);
    let p1 = field_mass(&sim.grid, &sim.fields.precursor);
    let s1 = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let occ1 = absolute_occupancy(&sim);
    let (mean_p, _min_p, _n) = interface_p_stats(&sim);
    let eq_occ = equilibrium_occupancy(mean_p, sim.params.k_exchange_eq);
    let coverage = boundary_coverage(&sim);
    let qual = qualify_exposure_capacity(&obs.qualification_input(None));
    let a_ret = if a0 <= EPS { 1.0 } else { a1 / a0 };
    let c_ret = if c0 <= EPS { 1.0 } else { c1 / c0 };
    let s_ret = if s0 <= EPS { 1.0 } else { s1 / s0 };
    let p_slope = if p0 <= EPS || st.accepted == 0 {
        0.0
    } else {
        (p1 - p0) / (p0 * st.accepted as f64)
    };
    let biological_terminal = a1 < 0.05 * a0.max(EPS);
    let numerical_terminal = st.accepted == 0 && st.rejected > 0;
    let evidence = MaintenanceEvidence {
        exposure_qualified: qual.qualifies,
        numerical_terminal,
        biological_terminal,
        mature_occupancy: occ1,
        a_retention: a_ret,
        c_retention: c_ret,
        p_bounded: p_slope.abs() <= 1e-3,
        zero_exposure_fraction: qual.zero_exposure_fraction,
        catalytic_exposure_failure: qual.zero_exposure_fraction > ZERO_EXPOSURE_CAP_FRAC_MAX,
        eq_occ_from_local_p: eq_occ,
        s_retention: s_ret,
    };
    let class = classify_long_horizon(evidence);
    let template_snapshot = json!({
        "occ": occ1,
        "s1": s1,
        "field_hash": field_hash(&sim),
        "class": class.as_str(),
    });
    let outcome = MaintenanceOutcome {
        label,
        accepted: st.accepted,
        rejected: st.rejected,
        sim_time: st.time,
        occupancy_0: occ0,
        occupancy_1: occ1,
        a_ret,
        c_ret,
        s_ret,
        p_start: p0,
        p_end: p1,
        p_slope,
        mean_p,
        ads: st.ads,
        des: st.des,
        exchange_net: st.exchange_net,
        coverage,
        eq_occ_from_local_p: eq_occ,
        exposure_qual: qual,
        numerical_terminal,
        biological_terminal,
        class,
        field_hash: field_hash(&sim),
        template_snapshot,
    };
    (sim, outcome)
}

fn maintenance_to_json(m: &MaintenanceOutcome) -> Value {
    json!({
        "label": m.label,
        "accepted": m.accepted,
        "rejected": m.rejected,
        "sim_time": m.sim_time,
        "occupancy_0": m.occupancy_0,
        "occupancy_1": m.occupancy_1,
        "a_retention": m.a_ret,
        "c_retention": m.c_ret,
        "s_retention": m.s_ret,
        "p_start": m.p_start,
        "p_end": m.p_end,
        "p_slope": m.p_slope,
        "mean_interface_p": m.mean_p,
        "adsorption": m.ads,
        "desorption": m.des,
        "exchange_net": m.exchange_net,
        "coverage": m.coverage,
        "eq_occupancy_from_local_p": m.eq_occ_from_local_p,
        "exposure": m.exposure_qual,
        "numerical_terminal": m.numerical_terminal,
        "biological_terminal": m.biological_terminal,
        "class": m.class.as_str(),
        "field_hash": m.field_hash,
        "template_snapshot": m.template_snapshot,
    })
}

/// Damage the given post-maintenance sim and run recovery under the same config.
fn run_damage_repair(
    config: MaintConfig,
    template: &Simulation,
    pre_s: f64,
    pre_occ: f64,
) -> Value {
    let mut sim = clone_from_template(template);
    let damage = damage_and_sync(&mut sim);
    let damaged_mask = collect_damaged_mask(&sim);
    let mut obs = ExposureObserver::attach(&sim);
    let st = run_steps_observed(
        &mut sim,
        &mut obs,
        maint_max_accepted(),
        Some(max_time()),
        HoldMode::None,
        Some(&damaged_mask),
    );
    let recovered_s = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let ratio = recovered_s / pre_s.max(EPS);
    let (damaged_occ, undamaged_occ) = damaged_arc_occupancy(&sim);
    let coverage = boundary_coverage(&sim);
    let damaged_qual = qualify_exposure_capacity(&obs.qualification_input(Some(&damaged_mask)));
    let all_qual = qualify_exposure_capacity(&obs.qualification_input(None));
    let mature_fraction_of_pre = recovered_s / pre_s.max(EPS);
    let accounting_closed = damage["s_w_conservation"].as_bool().unwrap_or(false);
    let repairs = damaged_qual.qualifies
        && mature_fraction_of_pre >= REPAIR_FRACTION_GATE
        && damaged_occ >= REPAIR_FRACTION_GATE
        && coverage >= 0.95
        && accounting_closed;
    json!({
        "config": config.label(),
        "pre_damage_s": pre_s,
        "pre_damage_occupancy": pre_occ,
        "damage": damage,
        "recovered_s": recovered_s,
        "recovery_ratio": ratio,
        "damaged_arc_occupancy": damaged_occ,
        "undamaged_occupancy": undamaged_occ,
        "coverage": coverage,
        "accepted": st.accepted,
        "rejected": st.rejected,
        "sim_time": st.time,
        "damaged_exposure": damaged_qual,
        "all_exposure": all_qual,
        "damaged_exposure_qualifies": damaged_qual.qualifies,
        "repairs": repairs,
        "mature_fraction_of_pre": mature_fraction_of_pre,
    })
}

// ─── Gate 7: radius portability ─────────────────────────────────────────────

fn radius_row(
    config: MaintConfig,
    base: &SimParams,
    radius: f64,
    settle: u64,
) -> Value {
    let params = config.apply(base, selected_m_p());
    let (sim, m) = run_maintenance("radius", params, &GeometrySpec::smooth(radius), settle);
    let (mean_p, _, _) = interface_p_stats(&sim);
    let eq_occ = equilibrium_occupancy(mean_p, sim.params.k_exchange_eq);
    let a_ok = m.a_ret >= A_RETENTION_GATE;
    let c_ok = m.c_ret >= C_RETENTION_GATE;
    let occ_ok = m.occupancy_1 >= OCC_GATE;
    let p_bounded = m.p_slope.abs() <= 1e-3;
    let exposure_ok = m.exposure_qual.qualifies;
    let capacity_ok = m.exposure_qual.capacity_unsupported / m.exposure_qual.relevant_lawful_capacity.max(EPS) < 0.05;
    let row_ok = a_ok && c_ok && occ_ok && p_bounded && exposure_ok && capacity_ok;
    json!({
        "radius": radius,
        "maintenance": maintenance_to_json(&m),
        "eq_occupancy_from_local_p": eq_occ,
        "a_ok": a_ok,
        "c_ok": c_ok,
        "occ_ok": occ_ok,
        "p_bounded": p_bounded,
        "exposure_ok": exposure_ok,
        "capacity_ok": capacity_ok,
        "row_ok": row_ok,
    })
}

// ─── main pipeline ──────────────────────────────────────────────────────────

pub fn run_pipeline(out: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let out = resolve_path(out);
    prepare_artifact_root(&out)?;
    for d in [
        "preservation",
        "d074_reproduction",
        "exposure_observer",
        "synthetic_calibration",
        "fixed_p_controls",
        "undamaged_maintenance",
        "damage_repair",
        "regulation_decision",
        "radius_portability",
        "stage_e_screen",
        "accounting",
    ] {
        fs::create_dir_all(out.join(d))?;
    }

    let base = baseline_params();
    let frozen = frozen_kinetics_unchanged(base.k_exchange_eq, base.k_exchange, base.gamma_max);
    let defaults_ok = (base.precursor_m_p - 1.0).abs() < 1e-15
        && base.precursor_product_inhibition_ki.abs() < 1e-15;

    // ── Gate 0 (part 1): preservation of D-074 tag/commit + artifact hashes ─
    let d074_dir = resolve_path(Path::new("experiments/generated/d074"));
    let d074_result_hash = file_sha256(&d074_dir.join("result.json"));
    let d074_manifest_hash = file_sha256(&d074_dir.join("manifest.json"));
    let preservation = json!({
        "gate": "preservation",
        "pass": frozen && defaults_ok,
        "frozen_kinetics_unchanged": frozen,
        "production_defaults_unchanged": defaults_ok,
        "seed_contract": SEED_CONTRACT,
        "starting_commit": D075_STARTING_COMMIT,
        "starting_tag": D075_STARTING_TAG,
        "d074_conclusion_preserved": D074_CONCLUSION == "D074_EXCHANGE_TIMESCALE_CLASSIFICATION_DEFECT",
        "d074_starting_commit": D074_START_COMMIT,
        "d074_starting_tag": D074_START_TAG,
        "d074_result_sha256": d074_result_hash,
        "d074_manifest_sha256": d074_manifest_hash,
        "source_commit": git_output(&["rev-parse", "HEAD"]),
        "branch": git_output(&["branch", "--show-current"]),
        "d070_d074_tags_present": {
            "D-070": git_output(&["rev-parse", "D-070-mature-membrane-seed-capacity-repair"]).is_some(),
            "D-071": git_output(&["rev-parse", "D-071-precursor-demand-regulation-fail"]).is_some(),
            "D-072": git_output(&["rev-parse", "D-072-membrane-damage-refill-audit"]).is_some(),
            "D-073": git_output(&["rev-parse", "D-073-mature-membrane-equilibrium-audit"]).is_some(),
            "D-074": git_output(&["rev-parse", D074_START_TAG]).is_some(),
        }
    });
    write_json(&out.join("preservation"), &preservation)?;

    // ── Gate 0 (part 2): D-074 reproduction (fixed-P anchors + defect check) ─
    let mut d074_reproduced = true;
    let mut anchor_rows = Map::new();
    let d074_sufficient = d074_dir.join("d073_reproduction/result.json");
    if let Ok(text) = fs::read_to_string(&d074_sufficient) {
        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            for (label, p_exp, rec_exp) in d073_expected_recoveries() {
                let obs = v
                    .pointer(&format!("/artifact_rows/{label}/recovery_observed"))
                    .and_then(|x| x.as_f64())
                    .or_else(|| {
                        v.pointer(&format!("/live_exchange_only/{label}/assay/recovery_ratio"))
                            .and_then(|x| x.as_f64())
                    });
                let ok = obs
                    .map(|o| (o - *rec_exp).abs() < 0.02)
                    .unwrap_or(false);
                d074_reproduced &= ok;
                anchor_rows.insert(
                    (*label).into(),
                    json!({
                        "expected_p": p_exp,
                        "expected_recovery": rec_exp,
                        "observed_recovery": obs,
                        "ok": ok,
                    }),
                );
            }
        } else {
            d074_reproduced = false;
        }
    } else {
        d074_reproduced = false;
    }
    // Verify D-074 exposure_audit shows fraction_E_ge5≈0 for the mean-τ horizon
    // (which is what D-074 concluded is a classification defect). If not present,
    // we run a short live check below.
    let mut prior_horizon_defect = false;
    let d074_exposure = d074_dir.join("exposure_audit/result.json");
    if let Ok(text) = fs::read_to_string(&d074_exposure) {
        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            if let Some(camps) = v.get("campaigns").and_then(|c| c.as_object()) {
                for (_lab, camp) in camps {
                    if let Some(cov) = camp.get("exposure_coverage") {
                        let five_frac = cov
                            .get("fraction_five_timescale")
                            .and_then(|x| x.as_f64())
                            .unwrap_or(0.0);
                        let qualifies = cov
                            .get("qualifies_five_timescale")
                            .and_then(|x| x.as_bool())
                            .unwrap_or(false);
                        if five_frac < 0.5 && !qualifies {
                            prior_horizon_defect = true;
                        }
                    }
                }
            }
        }
    }

    // Live short exchange-isolated exposure sanity: does a mean-τ-length campaign
    // fail to exposure-qualify? Cheap: settle, damage, isolate exchange at 5·τ_mean.
    let mut live_defect_row = Value::Null;
    if !skip_late() {
        let (mut template, _) = settled(base.clone(), settle_steps(), &GeometrySpec::smooth(22.0));
        let pre_s = total_surface_mass(&template.grid, &template.fields.membrane);
        let _dmg = damage_and_sync(&mut template);
        let damaged_mask = collect_damaged_mask(&template);
        let p_hold = p_required(0.95, D074_K_EQ);
        let mut sim = clone_from_template(&template);
        configure_exchange_only_isolated(&mut sim);
        hold_interface_activity(&mut sim, p_hold);
        let mut obs = ExposureObserver::attach(&sim);
        // Short horizon representing prior "mean-τ" gate scale.
        let horizon = (5.0 * 1.0 / (D075_K_EXCHANGE * 0.4 * (1.0 + D075_K_EQ * p_hold))).min(50.0);
        let st = run_steps_observed(
            &mut sim,
            &mut obs,
            fixed_p_max_accepted().min(4000),
            Some(horizon),
            HoldMode::FixedInterfaceP(p_hold),
            Some(&damaged_mask),
        );
        let dmg_qual = qualify_exposure_capacity(&obs.qualification_input(Some(&damaged_mask)));
        if !dmg_qual.qualifies {
            prior_horizon_defect = true;
        }
        live_defect_row = json!({
            "intended_p": p_hold,
            "horizon_sim_time": horizon,
            "accepted": st.accepted,
            "rejected": st.rejected,
            "recovery_ratio": total_surface_mass(&sim.grid, &sim.fields.membrane) / pre_s.max(EPS),
            "damaged_exposure": dmg_qual,
            "prior_horizon_defect_observed": !dmg_qual.qualifies,
        });
    }
    let d074_reproduction = json!({
        "gate": "d074_reproduction",
        "pass": d074_reproduced && prior_horizon_defect,
        "d073_anchor_rows": anchor_rows,
        "d074_conclusion": D074_CONCLUSION,
        "prior_horizon_defect_confirmed": prior_horizon_defect,
        "live_short_exchange_isolated": live_defect_row,
        "d074_result_sha256": d074_result_hash,
        "note": "D-075 requires D-074 conclusions to be reproducible; if not, early-stop with StopD074",
    });
    write_json(&out.join("d074_reproduction"), &d074_reproduction)?;

    // Early stop if Gate 0 fails.
    if !d074_reproduction["pass"].as_bool().unwrap_or(false) {
        let route = D075Route::StopD074;
        return finalize_early(&out, route, &preservation, &d074_reproduction, "D-074 reproduction failed");
    }

    // ── Gate 1: shared exposure observer (documentation of the operator) ────
    let g1 = json!({
        "gate": "exposure_observer",
        "pass": true,
        "operator": "classify_production_exchange_step → observe_attempt",
        "authoritative_metric": "E_i = -Σ ln(c_{i,n})",
        "diagnostic_metric": "Λ_i = Σ λ_{i,n} Δt_n",
        "rejected_steps_zero_exposure": true,
        "capacity_gate": EXPOSURE_GATE,
        "capacity_coverage_gate": EXPOSURE_COVERAGE_GATE,
        "zero_exposure_capacity_max": ZERO_EXPOSURE_CAP_FRAC_MAX,
        "notes": [
            "No simulation feedback: observer never gates the sim.",
            "Snapshot/resume payload is ExposureObserverSnapshot.",
            "Explicit and BE contributions tracked separately per cell."
        ]
    });
    write_json(&out.join("exposure_observer"), &g1)?;

    // ── Gate 2: synthetic calibration ───────────────────────────────────────
    let g2 = synthetic_calibration();
    write_json(&out.join("synthetic_calibration"), &g2)?;
    let synthetic_ok = g2["pass"].as_bool().unwrap_or(false);
    if !synthetic_ok {
        let route = D075Route::StopObserver;
        return finalize_early(&out, route, &preservation, &g2, "synthetic calibration failed");
    }

    // ── Gate 3: fixed-P requalification (damaged-region exposure gating) ────
    let (mut template, _setup) = settled(base.clone(), settle_steps(), &GeometrySpec::smooth(22.0));
    let pre_s = total_surface_mass(&template.grid, &template.fields.membrane);
    let pre_occ = absolute_occupancy(&template);
    let damage_meta = damage_and_sync(&mut template);
    let damaged_mask = collect_damaged_mask(&template);

    let fixed_p_targets = [
        ("p_0_38", p_required(0.95, D074_K_EQ)),
        ("p_0_418", 1.1 * p_required(0.95, D074_K_EQ)),
        ("p_2_48", p_required(D070_LAWFUL_MAINTENANCE_OCCUPANCY, D074_K_EQ)),
    ];
    let mut fixed_p_rows = Map::new();
    let mut fixed_p_ok = true;
    let mut fixed_p_repairs = true;
    for (name, p) in fixed_p_targets {
        eprintln!("D-075 Gate3 fixed_p {name} p={p}");
        let assay = fixed_p_assay(name, &template, pre_s, p, &damaged_mask);
        let recovers = assay["recovers"].as_bool().unwrap_or(false);
        let p_ok = assay["p_within_2pct"].as_bool().unwrap_or(false);
        let exp_ok = assay["damaged_exposure_qualifies"]
            .as_bool()
            .unwrap_or(false);
        // All controls must reach damaged-region exposure qualification.
        if !exp_ok || !p_ok {
            fixed_p_ok = false;
        }
        // Recovery gate: 0.38 is diagnostic-only; 0.418 and 2.48 must recover.
        if name != "p_0_38" && !(recovers && exp_ok) {
            fixed_p_repairs = false;
        }
        fixed_p_rows.insert(name.into(), assay);
    }
    let g3 = json!({
        "gate": "fixed_p_controls",
        "pass": fixed_p_ok && fixed_p_repairs,
        "pre_damage_s": pre_s,
        "pre_damage_occupancy": pre_occ,
        "damage": damage_meta,
        "targets": fixed_p_rows,
        "note": "diagnostic non-promotable controls; damaged-region exposure gate is authoritative",
    });
    write_json(&out.join("fixed_p_controls"), &g3)?;
    if !g3["pass"].as_bool().unwrap_or(false) {
        let route = D075Route::StopFixedP;
        return finalize_early(&out, route, &preservation, &g3, "fixed-P exposure parity failed");
    }

    // ── Gates 4/5/6/7/8 (late) ──────────────────────────────────────────────
    if skip_late() {
        let skipped = json!({"gate": "skipped", "pass": false, "reason": "D075_SKIP_LATE_GATES=1"});
        write_json(&out.join("undamaged_maintenance"), &skipped)?;
        write_json(&out.join("damage_repair"), &skipped)?;
        write_json(&out.join("regulation_decision"), &skipped)?;
        write_json(&out.join("radius_portability"), &skipped)?;
        write_json(&out.join("stage_e_screen"), &skipped)?;
        let evidence = RouteEvidence075 {
            d074_reproduced: true,
            observer_ok: true,
            synthetic_calibration_ok: true,
            fixed_p_ok: true,
            fixed_p_repairs,
            accounting_ok: true,
            ..RouteEvidence075::default()
        };
        return finalize(
            &out,
            evidence,
            preservation,
            d074_reproduction,
            g1,
            g2,
            g3,
            skipped.clone(),
            skipped.clone(),
            skipped.clone(),
            skipped.clone(),
            skipped,
            damage_meta,
        );
    }

    // Gate 4: maintenance across the three configs. Constitutive is the only
    // one required to seed damage repair (Gate 5) directly.
    let mut maint_rows = Map::new();
    let (const_sim, const_maint) =
        run_maintenance(MaintConfig::Constitutive.label(), MaintConfig::Constitutive.apply(&base, selected_m_p()), &GeometrySpec::smooth(22.0), settle_steps());
    let const_pre_s = total_surface_mass(&const_sim.grid, &const_sim.fields.membrane);
    let const_pre_occ = absolute_occupancy(&const_sim);
    let (reg_sim, reg_maint) = run_maintenance(
        MaintConfig::RegulatedReduced.label(),
        MaintConfig::RegulatedReduced.apply(&base, selected_m_p()),
        &GeometrySpec::smooth(22.0),
        settle_steps(),
    );
    let reg_pre_s = total_surface_mass(&reg_sim.grid, &reg_sim.fields.membrane);
    let reg_pre_occ = absolute_occupancy(&reg_sim);
    let (_nop_sim, nop_maint) = run_maintenance(
        MaintConfig::KPrecursorZero.label(),
        MaintConfig::KPrecursorZero.apply(&base, selected_m_p()),
        &GeometrySpec::smooth(22.0),
        settle_steps(),
    );
    maint_rows.insert("constitutive".into(), maintenance_to_json(&const_maint));
    maint_rows.insert("regulated_reduced".into(), maintenance_to_json(&reg_maint));
    maint_rows.insert("k_precursor_zero".into(), maintenance_to_json(&nop_maint));
    let const_maint_qual = const_maint.exposure_qual.qualifies
        && const_maint.occupancy_1 >= OCC_GATE
        && const_maint.a_ret >= A_RETENTION_GATE
        && const_maint.c_ret >= C_RETENTION_GATE;
    let reg_maint_qual = reg_maint.exposure_qual.qualifies
        && reg_maint.occupancy_1 >= OCC_GATE
        && reg_maint.a_ret >= A_RETENTION_GATE
        && reg_maint.c_ret >= C_RETENTION_GATE
        && reg_maint.p_slope.abs() <= 1e-3;
    let g4 = json!({
        "gate": "undamaged_maintenance",
        "pass": const_maint_qual || reg_maint_qual,
        "rows": maint_rows,
        "constitutive_qualifies": const_maint_qual,
        "regulated_qualifies": reg_maint_qual,
        "selected_m_p": selected_m_p(),
        "note": "TRUE_LONG_HORIZON_MAINTENANCE requires exposure_qualified + occ≥0.95 + A/C≥0.80",
    });
    write_json(&out.join("undamaged_maintenance"), &g4)?;

    // Gate 5: damage repair. Only run from maintenance-passed templates.
    let mut repair_rows = Map::new();
    let mut const_repair_ok = false;
    let mut reg_repair_ok = false;
    if const_maint_qual {
        eprintln!("D-075 Gate5 damage_repair constitutive");
        let r = run_damage_repair(MaintConfig::Constitutive, &const_sim, const_pre_s, const_pre_occ);
        const_repair_ok = r["repairs"].as_bool().unwrap_or(false);
        repair_rows.insert("constitutive".into(), r);
    } else {
        repair_rows.insert(
            "constitutive".into(),
            json!({"skipped": true, "reason": "constitutive maintenance did not qualify"}),
        );
    }
    if reg_maint_qual {
        eprintln!("D-075 Gate5 damage_repair regulated_reduced");
        let r = run_damage_repair(MaintConfig::RegulatedReduced, &reg_sim, reg_pre_s, reg_pre_occ);
        reg_repair_ok = r["repairs"].as_bool().unwrap_or(false);
        repair_rows.insert("regulated_reduced".into(), r);
    } else {
        repair_rows.insert(
            "regulated_reduced".into(),
            json!({"skipped": true, "reason": "regulated maintenance did not qualify"}),
        );
    }
    // Controls: no-precursor (Gate5 must fail); no-A (starved, must fail).
    let (nop_full_sim, _) = run_maintenance(
        "k_precursor_zero_ctrl",
        MaintConfig::KPrecursorZero.apply(&base, selected_m_p()),
        &GeometrySpec::smooth(22.0),
        settle_steps(),
    );
    let nop_pre_s = total_surface_mass(&nop_full_sim.grid, &nop_full_sim.fields.membrane);
    let nop_pre_occ = absolute_occupancy(&nop_full_sim);
    let nop_repair = run_damage_repair(MaintConfig::KPrecursorZero, &nop_full_sim, nop_pre_s, nop_pre_occ);
    let nop_repair_ok = nop_repair["repairs"].as_bool().unwrap_or(false);
    repair_rows.insert("k_precursor_zero_control".into(), nop_repair);
    let g5 = json!({
        "gate": "damage_repair",
        "pass": (const_repair_ok || reg_repair_ok) && !nop_repair_ok,
        "constitutive_repairs": const_repair_ok,
        "regulated_repairs": reg_repair_ok,
        "no_precursor_fails": !nop_repair_ok,
        "rows": repair_rows,
        "damage_fraction": DAMAGE_FRACTION,
    });
    write_json(&out.join("damage_repair"), &g5)?;

    // Gate 6: regulation decision (Route R vs Route T).
    let regulated_a_ok = reg_maint.a_ret >= A_RETENTION_GATE;
    let regulated_p_bounded = reg_maint.p_slope.abs() <= 1e-3;
    let regulated_maintains = reg_maint_qual;
    let regulated_repairs = reg_repair_ok;
    let regulation_route = if regulated_maintains && regulated_repairs && regulated_a_ok && regulated_p_bounded {
        "route_r_qualified"
    } else if regulated_maintains && regulated_a_ok && regulated_p_bounded && !regulated_repairs {
        "route_t_tradeoff"
    } else if const_maint_qual && const_repair_ok {
        "route_q_constitutive"
    } else {
        "route_f_or_h"
    };
    let g6 = json!({
        "gate": "regulation_decision",
        "pass": true,
        "regulated_maintains": regulated_maintains,
        "regulated_repairs": regulated_repairs,
        "regulated_a_ok": regulated_a_ok,
        "regulated_p_bounded": regulated_p_bounded,
        "constitutive_maintains": const_maint_qual,
        "constitutive_repairs": const_repair_ok,
        "provisional_route": regulation_route,
    });
    write_json(&out.join("regulation_decision"), &g6)?;

    // Gate 7: radius portability R16/R22/R32 under the same parameter set.
    let config_for_radius = if regulated_maintains && regulated_repairs {
        MaintConfig::RegulatedReduced
    } else {
        MaintConfig::Constitutive
    };
    let mut radius_rows = Map::new();
    let mut all_portable = true;
    for r in [16.0f64, 22.0, 32.0] {
        eprintln!("D-075 Gate7 radius R{}", r as i32);
        let row = radius_row(config_for_radius, &base, r, settle_steps());
        if !row["row_ok"].as_bool().unwrap_or(false) {
            all_portable = false;
        }
        radius_rows.insert(format!("R{}", r as i32), row);
    }
    let g7 = json!({
        "gate": "radius_portability",
        "pass": all_portable,
        "config": config_for_radius.label(),
        "rows": radius_rows,
    });
    write_json(&out.join("radius_portability"), &g7)?;

    // Gate 8: Stage E screen R18/R22/R26 — same discipline as d071 gate 8, but
    // horizon qualification replaced by exposure qualification.
    let mut stage_e_rows = Map::new();
    let mut stage_e_ok = true;
    if all_portable && (const_repair_ok || reg_repair_ok) {
        for r in [18.0f64, 22.0, 26.0] {
            eprintln!("D-075 Gate8 stage_e R{}", r as i32);
            let row = radius_row(config_for_radius, &base, r, settle_steps());
            if !row["row_ok"].as_bool().unwrap_or(false) {
                stage_e_ok = false;
            }
            stage_e_rows.insert(format!("R{}", r as i32), row);
        }
    } else {
        stage_e_ok = false;
        stage_e_rows.insert(
            "skipped".into(),
            json!({"reason": "gates 0..7 not fully passing; Stage E not attempted"}),
        );
    }
    let g8 = json!({
        "gate": "stage_e_screen",
        "pass": stage_e_ok,
        "config": config_for_radius.label(),
        "rows": stage_e_rows,
        "note": "Stage E chemistry/rates/thresholds unchanged; only horizon replaced with exposure gate",
    });
    write_json(&out.join("stage_e_screen"), &g8)?;

    // ── Accounting + route selection ────────────────────────────────────────
    let accounting_ok = damage_meta["s_w_conservation"].as_bool().unwrap_or(false)
        && (damage_meta["capacity_before"].as_f64().unwrap_or(0.0)
            - damage_meta["capacity_after"].as_f64().unwrap_or(0.0))
        .abs()
            <= 1e-6;
    let accounting = json!({
        "gate": "accounting",
        "pass": accounting_ok,
        "damage_s_w_conservation": damage_meta["s_w_conservation"],
        "no_capacity_change_on_damage": (damage_meta["capacity_before"].as_f64().unwrap_or(0.0)
            - damage_meta["capacity_after"].as_f64().unwrap_or(0.0)).abs()
            <= 1e-6,
        "rejected_steps_zero_exchange": true,
    });
    write_json(&out.join("accounting"), &accounting)?;

    let evidence = RouteEvidence075 {
        accounting_ok,
        numerical_ok: !const_maint.numerical_terminal,
        d074_reproduced: true,
        observer_ok: true,
        synthetic_calibration_ok: true,
        fixed_p_ok: true,
        fixed_p_repairs,
        constitutive_maintains: const_maint_qual,
        constitutive_repairs: const_repair_ok,
        regulated_maintains,
        regulated_repairs,
        regulated_a_ok,
        regulated_p_bounded,
        radius_portable: all_portable,
        catalytic_exposure_limit: const_maint.exposure_qual.zero_exposure_fraction
            > ZERO_EXPOSURE_CAP_FRAC_MAX
            && !fixed_p_repairs,
        horizon_unqualifiable: false,
        endogenous_p_insufficient: !const_maint_qual && fixed_p_repairs,
        stage_e_ok,
    };

    finalize(
        &out,
        evidence,
        preservation,
        d074_reproduction,
        g1,
        g2,
        g3,
        g4,
        g5,
        g6,
        g7,
        g8,
        damage_meta,
    )
}

// ─── finalize / manifest writers ────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn finalize(
    out: &Path,
    ev: RouteEvidence075,
    preservation: Value,
    d074_reproduction: Value,
    exposure_observer: Value,
    synthetic_calibration: Value,
    fixed_p_controls: Value,
    undamaged_maintenance: Value,
    damage_repair: Value,
    regulation_decision: Value,
    radius_portability: Value,
    stage_e_screen: Value,
    damage_meta: Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    let route = select_route(ev);
    let conclusion = route.conclusion();
    let manifest_extra = json!({
        "damage_meta": damage_meta,
    });
    write_manifest_and_result(
        out,
        route,
        conclusion,
        ev,
        &preservation,
        &d074_reproduction,
        &exposure_observer,
        &synthetic_calibration,
        &fixed_p_controls,
        &undamaged_maintenance,
        &damage_repair,
        &regulation_decision,
        &radius_portability,
        &stage_e_screen,
        manifest_extra,
    )
}

fn finalize_early(
    out: &Path,
    route: D075Route,
    a: &Value,
    b: &Value,
    reason: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let conclusion = route.conclusion();
    let skipped = json!({"gate": "skipped", "pass": false, "reason": reason});
    let ev = RouteEvidence075 {
        d074_reproduced: !matches!(route, D075Route::StopD074),
        observer_ok: !matches!(route, D075Route::StopObserver),
        synthetic_calibration_ok: !matches!(route, D075Route::StopObserver),
        fixed_p_ok: !matches!(route, D075Route::StopFixedP),
        ..RouteEvidence075::default()
    };
    write_manifest_and_result(
        out,
        route,
        conclusion,
        ev,
        a,
        b,
        &skipped,
        &skipped,
        &skipped,
        &skipped,
        &skipped,
        &skipped,
        &skipped,
        &skipped,
        json!({"early_stop": reason}),
    )
}

#[allow(clippy::too_many_arguments)]
fn write_manifest_and_result(
    out: &Path,
    route: D075Route,
    conclusion: D075PrimaryConclusion,
    ev: RouteEvidence075,
    preservation: &Value,
    d074_reproduction: &Value,
    exposure_observer: &Value,
    synthetic_calibration: &Value,
    fixed_p_controls: &Value,
    undamaged_maintenance: &Value,
    damage_repair: &Value,
    regulation_decision: &Value,
    radius_portability: &Value,
    stage_e_screen: &Value,
    extra: Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    let d008 = if matches!(
        conclusion,
        D075PrimaryConclusion::StageERecovered
            | D075PrimaryConclusion::ExposureGatedMembraneRequalified
            | D075PrimaryConclusion::PrecursorRegulationQualified
    ) {
        if matches!(conclusion, D075PrimaryConclusion::StageERecovered) {
            "STAGE_E_RECOVERED"
        } else {
            "REQUIRES_STAGE_E_RESCREEN"
        }
    } else {
        "BLOCKED_NOT_RECOVERED"
    };
    let phase1 = if matches!(conclusion, D075PrimaryConclusion::StageERecovered) {
        "PHASE1_SELF_MAINTENANCE_QUALIFIED"
    } else {
        "PHASE1_SELF_MAINTENANCE_PARTIAL"
    };
    let production_verdict = match conclusion {
        D075PrimaryConclusion::StageERecovered => "READY_FOR_STAGE_F",
        D075PrimaryConclusion::ExposureGatedMembraneRequalified
        | D075PrimaryConclusion::PrecursorRegulationQualified => "REQUIRES_STAGE_E_RESCREEN",
        _ => "REQUIRES_REMEDIATION",
    };
    let manifest = json!({
        "project_directive": D075_PROJECT_ID,
        "agent_memory_directive": D075_AGENT_MEMORY_ID,
        "starting_commit": D075_STARTING_COMMIT,
        "starting_tag": D075_STARTING_TAG,
        "source_commit": git_output(&["rev-parse", "HEAD"]),
        "branch": git_output(&["branch", "--show-current"]),
        "primary_conclusion": conclusion.as_str(),
        "route": route.as_str(),
        "d008_status": d008,
        "phase1_status": phase1,
        "production_verdict": production_verdict,
        "d074_conclusion": D074_CONCLUSION,
        "evidence": {
            "accounting_ok": ev.accounting_ok,
            "numerical_ok": ev.numerical_ok,
            "d074_reproduced": ev.d074_reproduced,
            "observer_ok": ev.observer_ok,
            "synthetic_calibration_ok": ev.synthetic_calibration_ok,
            "fixed_p_ok": ev.fixed_p_ok,
            "fixed_p_repairs": ev.fixed_p_repairs,
            "constitutive_maintains": ev.constitutive_maintains,
            "constitutive_repairs": ev.constitutive_repairs,
            "regulated_maintains": ev.regulated_maintains,
            "regulated_repairs": ev.regulated_repairs,
            "regulated_a_ok": ev.regulated_a_ok,
            "regulated_p_bounded": ev.regulated_p_bounded,
            "radius_portable": ev.radius_portable,
            "catalytic_exposure_limit": ev.catalytic_exposure_limit,
            "horizon_unqualifiable": ev.horizon_unqualifiable,
            "endogenous_p_insufficient": ev.endogenous_p_insufficient,
            "stage_e_ok": ev.stage_e_ok,
        },
        "artifacts": [
            "preservation",
            "d074_reproduction",
            "exposure_observer",
            "synthetic_calibration",
            "fixed_p_controls",
            "undamaged_maintenance",
            "damage_repair",
            "regulation_decision",
            "radius_portability",
            "stage_e_screen",
            "accounting",
            "result.json"
        ]
    });
    atomic_write_json(&out.join("manifest.json"), &manifest)?;
    let result = json!({
        "project_directive": D075_PROJECT_ID,
        "agent_memory_directive": D075_AGENT_MEMORY_ID,
        "primary_conclusion": conclusion.as_str(),
        "route": route.as_str(),
        "d008_status": d008,
        "phase1_status": phase1,
        "production_verdict": production_verdict,
        "manifest": manifest,
        "gates": {
            "preservation": preservation,
            "d074_reproduction": d074_reproduction,
            "exposure_observer": exposure_observer,
            "synthetic_calibration": synthetic_calibration,
            "fixed_p_controls": fixed_p_controls,
            "undamaged_maintenance": undamaged_maintenance,
            "damage_repair": damage_repair,
            "regulation_decision": regulation_decision,
            "radius_portability": radius_portability,
            "stage_e_screen": stage_e_screen,
        },
        "extra": extra,
    });
    atomic_write_json(&out.join("result.json"), &result)?;
    Ok(result)
}

#[cfg(test)]
mod smoke {
    use super::*;

    #[test]
    fn env_defaults_sane() {
        assert!(dt_cap() > 0.0);
        assert!(max_time() >= 0.0);
        assert!(max_accepted() >= 1);
    }

    #[test]
    fn selected_m_p_defaults_to_d075() {
        std::env::remove_var("D075_SELECTED_M_P");
        std::env::remove_var("D071_SELECTED_M_P");
        assert!((selected_m_p() - D075_SELECTED_M_P).abs() < 1e-18);
    }
}

