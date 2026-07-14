//! Main simulation engine with adaptive timestep and mass accounting.

use crate::accounting::{
    build_field_ledger, field_mass, sum_clamp_correction, AccountingState, ReactionStepTotals,
    StepAccounting,
};
use crate::config::{
    EquationVersion, ExperimentConfig, InterventionSpec, SimParams, MAX_DT, NEG_CLAMP,
};
use crate::diagnostics::{CellDetector, DiagnosticsSnapshot};
use crate::fields::{
    clamp_small_negative, initialize_seed, validate_structure_field, validate_soluble_field,
    FieldBuffers,
};
use crate::time_audit::DtTelemetry;
use crate::grid::Grid;
use crate::interventions::apply_intervention;
use crate::operators::{diffuse_constant, diffuse_variable, laplacian};
use crate::phase_field::{chemical_potential_local, compute_interior_weights, structure_rate};
use crate::reactions::{catalyst_diffusivity, compute_all_reactions, ReactionScratch};
use crate::reservoir::apply_reservoir;
use crate::snapshot::FieldSnapshot;
use std::time::Instant;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TimingTelemetry {
    pub wall_seconds: f64,
    pub substeps_per_second: f64,
    pub phase_field_seconds: f64,
    pub diffusion_seconds: f64,
    pub reactions_seconds: f64,
    pub diagnostics_seconds: f64,
    pub serialization_seconds: f64,
}

#[derive(Debug, Clone)]
pub struct Simulation {
    pub grid: Grid,
    pub params: SimParams,
    pub fields: FieldBuffers,
    pub working: FieldBuffers,
    pub reaction_scratch: ReactionScratch,
    pub detector: CellDetector,
    pub accounting: AccountingState,
    pub substep: u64,
    pub sim_time: f64,
    pub dt: f64,
    pub min_dt_seen: f64,
    pub rejection_count: u64,
    pub history: Vec<DiagnosticsSnapshot>,
    pub interventions_applied: Vec<(u64, String)>,
    pub observer_enabled: bool,
    pub morphology_sample_interval: u64,
    pub timing: TimingTelemetry,
    pub dt_telemetry: DtTelemetry,
    run_start: Option<Instant>,
    prev_attempt_dt: f64,
}

impl Simulation {
    pub fn new(params: SimParams) -> Self {
        let grid = Grid::new();
        let size = grid.width * grid.height;
        let mut fields = FieldBuffers::for_grid(&grid);
        initialize_seed(&grid, &params, &mut fields);
        let mut detector = CellDetector::default();
        detector.capture_initial_masses(&grid, &fields);
        Self {
            grid,
            params,
            fields,
            working: FieldBuffers::new(size),
            reaction_scratch: ReactionScratch::new(size),
            detector,
            accounting: AccountingState::default(),
            substep: 0,
            sim_time: 0.0,
            dt: MAX_DT,
            min_dt_seen: MAX_DT,
            rejection_count: 0,
            history: Vec::new(),
            interventions_applied: Vec::new(),
            observer_enabled: true,
            morphology_sample_interval: 100,
            timing: TimingTelemetry::default(),
            dt_telemetry: DtTelemetry::default(),
            run_start: None,
            prev_attempt_dt: MAX_DT,
        }
    }

    pub fn from_config(config: &ExperimentConfig) -> Self {
        let mut sim = Self::new(config.params.clone());
        sim.params.random_seed = config.seed;
        initialize_seed(&sim.grid, &sim.params, &mut sim.fields);
        sim.detector.capture_initial_masses(&sim.grid, &sim.fields);
        sim
    }

    pub fn reset(&mut self) {
        let params = self.params.clone();
        *self = Self::new(params);
    }

    pub fn begin_timing(&mut self) {
        self.run_start = Some(Instant::now());
    }

    pub fn finish_timing(&mut self, substeps: u64) {
        if let Some(start) = self.run_start.take() {
            let elapsed = start.elapsed().as_secs_f64();
            self.timing.wall_seconds = elapsed;
            if elapsed > 0.0 {
                self.timing.substeps_per_second = substeps as f64 / elapsed;
            }
        }
    }

    pub fn run_substeps(&mut self, n: u64, record_every: u64) {
        self.begin_timing();
        for _ in 0..n {
            if !self.step() {
                break;
            }
            if record_every > 0 && self.substep % record_every == 0 {
                let snap = self.current_diagnostics();
                self.history.push(snap);
            }
        }
        self.finish_timing(self.substep);
    }

    pub fn step(&mut self) -> bool {
        let mut attempt_dt = self.dt;
        let max_attempts = 20;
        let dt_before_attempt = attempt_dt;

        for _ in 0..max_attempts {
            let result = match self.params.equation_version {
                EquationVersion::D001BulkV1
                | EquationVersion::D003CrowdingV1
                | EquationVersion::SurfaceTurnoverV1 => self.try_legacy_substep(attempt_dt),
                EquationVersion::MembraneMetabolismV1 => {
                    self.try_membrane_metabolism_v1_scaffold()
                }
            };
            match result {
                SubstepResult::Ok => {
                    self.dt_telemetry.record_accept(attempt_dt);
                    self.dt_telemetry
                        .record_recovery(self.prev_attempt_dt, attempt_dt);
                    self.prev_attempt_dt = attempt_dt;
                    self.sim_time += attempt_dt;
                    self.substep += 1;
                    self.dt = attempt_dt.min(MAX_DT);
                    return true;
                }
                SubstepResult::Reject => {
                    self.rejection_count += 1;
                    self.accounting.cumulative.rejected_steps += 1;
                    self.dt_telemetry.record_reduction();
                    attempt_dt *= 0.5;
                    self.min_dt_seen = self.min_dt_seen.min(attempt_dt);
                    if attempt_dt < 1e-8 {
                        return false;
                    }
                }
            }
        }
        let _ = dt_before_attempt;
        false
    }

    fn try_legacy_substep(&mut self, dt: f64) -> SubstepResult {
        let grid = &self.grid;
        let params = &self.params;
        let t0 = Instant::now();

        let mass_phi_before = field_mass(grid, &self.fields.structure);
        let mass_c_before = field_mass(grid, &self.fields.catalyst);
        let mass_n_before = field_mass(grid, &self.fields.nutrient);
        let mass_f_before = field_mass(grid, &self.fields.fuel);
        let mass_w_before = field_mass(grid, &self.fields.waste);

        self.fields.copy_current_to_working(&mut self.working);
        self.fields
            .activated_next
            .copy_from_slice(&self.fields.activated);
        self.fields
            .membrane_next
            .copy_from_slice(&self.fields.membrane);

        let n_before_res = field_mass(grid, &self.working.nutrient);
        let f_before_res = field_mass(grid, &self.working.fuel);
        let w_before_res = field_mass(grid, &self.working.waste);

        apply_reservoir(
            grid,
            &mut self.working.nutrient,
            &mut self.working.fuel,
            &mut self.working.waste,
            dt,
            params,
        );

        let n_reservoir_delta = field_mass(grid, &self.working.nutrient) - n_before_res;
        let f_reservoir_delta = field_mass(grid, &self.working.fuel) - f_before_res;
        let w_reservoir_delta = field_mass(grid, &self.working.waste) - w_before_res;

        let phi = &self.working.structure;
        let c = &self.working.catalyst;
        let n = &self.working.nutrient;
        let f = &self.working.fuel;
        let w = &self.working.waste;

        compute_interior_weights(phi, &mut self.fields.scratch_h);
        laplacian(grid, phi, &mut self.fields.scratch_lap);

        for idx in 0..grid.width * grid.height {
            if grid.in_dish(idx) {
                self.fields.scratch_mu[idx] =
                    chemical_potential_local(phi[idx], self.fields.scratch_lap[idx], params);
            } else {
                self.fields.scratch_mu[idx] = 0.0;
            }
        }

        laplacian(grid, &self.fields.scratch_mu, &mut self.fields.scratch_lap_mu);

        let t_react = Instant::now();
        compute_all_reactions(
            phi,
            c,
            n,
            f,
            w,
            params,
            params.reactions_enabled,
            &mut self.reaction_scratch,
        );
        self.timing.reactions_seconds += t_react.elapsed().as_secs_f64();

        for idx in 0..grid.width * grid.height {
            self.fields.scratch_h[idx] = catalyst_diffusivity(phi[idx], params);
        }

        let t_diff = Instant::now();
        diffuse_variable(grid, c, &self.fields.scratch_h, &mut self.fields.scratch_flux_x);

        if params.diffusion_enabled {
            diffuse_constant(
                grid,
                n,
                params.d_n,
                &mut self.fields.scratch_lap,
                &mut self.fields.scratch_flux_y,
            );
        } else {
            self.fields.scratch_flux_y.fill(0.0);
        }
        let nutrient_diff = &self.fields.scratch_flux_y;

        let mut fuel_diff = &mut self.fields.scratch_fuel_diff;
        let mut waste_diff = &mut self.fields.scratch_waste_diff;
        if params.diffusion_enabled {
            diffuse_constant(grid, f, params.d_f, &mut self.fields.scratch_lap, fuel_diff);
            diffuse_constant(grid, w, params.d_w, &mut self.fields.scratch_lap, waste_diff);
        } else {
            fuel_diff.fill(0.0);
            waste_diff.fill(0.0);
        }
        self.timing.diffusion_seconds += t_diff.elapsed().as_secs_f64();

        let mut react_phi = 0.0;
        let mut react_c = 0.0;
        let mut react_n = 0.0;
        let mut react_f = 0.0;
        let mut react_w = 0.0;
        let mut diff_phi = 0.0;
        let mut diff_c = 0.0;
        let mut diff_n = 0.0;
        let mut diff_f = 0.0;
        let mut diff_w = 0.0;
        let mut rx_totals = ReactionStepTotals::default();

        let t_phase = Instant::now();
        for idx in 0..grid.width * grid.height {
            if !grid.in_dish(idx) {
                self.fields.structure_next[idx] = 0.0;
                self.fields.catalyst_next[idx] = 0.0;
                self.fields.nutrient_next[idx] = 0.0;
                self.fields.fuel_next[idx] = 0.0;
                self.fields.waste_next[idx] = 0.0;
                continue;
            }

            let r = &self.reaction_scratch.rates[idx];
            let dphi = structure_rate(
                self.fields.scratch_lap_mu[idx],
                r.r_phi,
                params,
                params.phase_separation_enabled,
            );

            let dphi_dt = dphi * dt;
            let dc_dt = (r.r_c + self.fields.scratch_flux_x[idx]) * dt;
            let dn_dt = (r.r_n + nutrient_diff[idx]) * dt;
            let df_dt = (r.r_f + fuel_diff[idx]) * dt;
            let dw_dt = (r.r_w + waste_diff[idx]) * dt;

            react_phi += r.r_phi * dt;
            react_c += r.r_c * dt;
            react_n += r.r_n * dt;
            react_f += r.r_f * dt;
            react_w += r.r_w * dt;
            diff_phi += (dphi - r.r_phi) * dt;
            diff_c += self.fields.scratch_flux_x[idx] * dt;
            diff_n += nutrient_diff[idx] * dt;
            diff_f += fuel_diff[idx] * dt;
            diff_w += waste_diff[idx] * dt;

            rx_totals.structural_synthesis += r.r_structure * dt;
            rx_totals.structural_decay += r.r_structure_decay * dt;
            rx_totals.catalyst_reproduction += r.r_rep * dt;
            rx_totals.catalyst_decay += r.r_catalyst_decay * dt;
            rx_totals.nutrient_consumed_r1 += params.alpha_n_rep * r.r_rep * dt;
            rx_totals.nutrient_consumed_r2 += params.alpha_n_structure * r.r_structure * dt;
            rx_totals.fuel_consumed_r1 += params.alpha_f_rep * r.r_rep * dt;
            rx_totals.fuel_consumed_r2 += params.alpha_f_structure * r.r_structure * dt;
            rx_totals.waste_from_r1 += params.alpha_w_rep * r.r_rep * dt;
            rx_totals.waste_from_r2 += params.alpha_w_structure * r.r_structure * dt;
            rx_totals.waste_from_decay += (r.r_structure_decay + r.r_catalyst_decay) * dt;

            self.fields.structure_next[idx] = phi[idx] + dphi_dt;
            self.fields.catalyst_next[idx] = c[idx] + dc_dt;
            self.fields.nutrient_next[idx] = n[idx] + dn_dt;
            self.fields.fuel_next[idx] = f[idx] + df_dt;
            self.fields.waste_next[idx] = w[idx] + dw_dt;
        }
        self.timing.phase_field_seconds += t_phase.elapsed().as_secs_f64();

        let pre_clamp_phi = field_mass(grid, &self.fields.structure_next);
        let pre_clamp_c = field_mass(grid, &self.fields.catalyst_next);
        let pre_clamp_n = field_mass(grid, &self.fields.nutrient_next);
        let pre_clamp_f = field_mass(grid, &self.fields.fuel_next);
        let pre_clamp_w = field_mass(grid, &self.fields.waste_next);

        for (idx, v) in self.fields.structure_next.iter_mut().enumerate() {
            if !grid.in_dish(idx) {
                continue;
            }
            if !v.is_finite() {
                return SubstepResult::Reject;
            }
            // ponytail: defer PHI_HARD_MAX to validate_structure_field so dt can adapt
            if *v < -1e-6 {
                return SubstepResult::Reject;
            }
            if *v < 0.0 && *v >= NEG_CLAMP {
                *v = 0.0;
            }
        }

        for field in [
            &mut self.fields.catalyst_next,
            &mut self.fields.nutrient_next,
            &mut self.fields.fuel_next,
            &mut self.fields.waste_next,
        ] {
            for (idx, v) in field.iter_mut().enumerate() {
                if !grid.in_dish(idx) {
                    continue;
                }
                if !v.is_finite() {
                    return SubstepResult::Reject;
                }
                if *v < -1e-6 {
                    return SubstepResult::Reject;
                }
                if *v > 10.0 {
                    return SubstepResult::Reject;
                }
                *v = clamp_small_negative(*v);
            }
        }

        if validate_structure_field(&self.fields.structure_next, &grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.catalyst_next, &grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.nutrient_next, &grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.fuel_next, &grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.waste_next, &grid.dish_mask).is_err()
        {
            return SubstepResult::Reject;
        }

        let mass_phi_after = field_mass(grid, &self.fields.structure_next);
        let mass_c_after = field_mass(grid, &self.fields.catalyst_next);
        let mass_n_after = field_mass(grid, &self.fields.nutrient_next);
        let mass_f_after = field_mass(grid, &self.fields.fuel_next);
        let mass_w_after = field_mass(grid, &self.fields.waste_next);

        let clamp_total = sum_clamp_correction(
            &self.fields.structure,
            &self.fields.structure_next,
            grid,
        ) + sum_clamp_correction(&self.fields.catalyst, &self.fields.catalyst_next, grid)
            + sum_clamp_correction(&self.fields.nutrient, &self.fields.nutrient_next, grid)
            + sum_clamp_correction(&self.fields.fuel, &self.fields.fuel_next, grid)
            + sum_clamp_correction(&self.fields.waste, &self.fields.waste_next, grid);

        let step_accounting = StepAccounting {
            structure: build_field_ledger(
                mass_phi_before,
                react_phi,
                diff_phi,
                0.0,
                pre_clamp_phi,
                mass_phi_after,
            ),
            catalyst: build_field_ledger(
                mass_c_before,
                react_c,
                diff_c,
                0.0,
                pre_clamp_c,
                mass_c_after,
            ),
            nutrient: build_field_ledger(
                mass_n_before,
                react_n,
                diff_n,
                n_reservoir_delta,
                pre_clamp_n,
                mass_n_after,
            ),
            fuel: build_field_ledger(
                mass_f_before,
                react_f,
                diff_f,
                f_reservoir_delta,
                pre_clamp_f,
                mass_f_after,
            ),
            waste: build_field_ledger(
                mass_w_before,
                react_w,
                diff_w,
                w_reservoir_delta,
                pre_clamp_w,
                mass_w_after,
            ),
            activated: Default::default(),
            membrane: Default::default(),
        };
        self.accounting
            .record_step(step_accounting, &rx_totals, clamp_total);

        self.fields.swap();

        if self.observer_enabled {
            let t_diag = Instant::now();
            let sample_morphology = self.substep % self.morphology_sample_interval == 0;
            let _ = self.detector.observe(
                grid,
                &self.fields,
                params,
                self.substep + 1,
                self.sim_time + dt,
                dt,
                &self.reaction_scratch,
                &self.accounting,
                sample_morphology,
            );
            self.timing.diagnostics_seconds += t_diag.elapsed().as_secs_f64();
        }

        let _ = t0;
        SubstepResult::Ok
    }

    fn try_membrane_metabolism_v1_scaffold(&mut self) -> SubstepResult {
        self.fields.copy_current_to_next();
        if validate_structure_field(&self.fields.structure_next, &self.grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.catalyst_next, &self.grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.nutrient_next, &self.grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.fuel_next, &self.grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.waste_next, &self.grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.activated_next, &self.grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.membrane_next, &self.grid.dish_mask).is_err()
        {
            return SubstepResult::Reject;
        }

        let ledger = |before: &[f64], after: &[f64]| {
            let mass_before = field_mass(&self.grid, before);
            let mass_after = field_mass(&self.grid, after);
            build_field_ledger(
                mass_before,
                0.0,
                0.0,
                0.0,
                mass_after,
                mass_after,
            )
        };
        let step_accounting = StepAccounting {
            structure: ledger(&self.fields.structure, &self.fields.structure_next),
            catalyst: ledger(&self.fields.catalyst, &self.fields.catalyst_next),
            nutrient: ledger(&self.fields.nutrient, &self.fields.nutrient_next),
            fuel: ledger(&self.fields.fuel, &self.fields.fuel_next),
            waste: ledger(&self.fields.waste, &self.fields.waste_next),
            activated: ledger(&self.fields.activated, &self.fields.activated_next),
            membrane: ledger(&self.fields.membrane, &self.fields.membrane_next),
        };
        self.accounting
            .record_step(step_accounting, &ReactionStepTotals::default(), 0.0);
        self.fields.swap();
        SubstepResult::Ok
    }

    pub fn current_diagnostics(&mut self) -> DiagnosticsSnapshot {
        compute_all_reactions(
            &self.fields.structure,
            &self.fields.catalyst,
            &self.fields.nutrient,
            &self.fields.fuel,
            &self.fields.waste,
            &self.params,
            self.params.reactions_enabled,
            &mut self.reaction_scratch,
        );
        self.detector.observe(
            &self.grid,
            &self.fields,
            &self.params,
            self.substep,
            self.sim_time,
            self.dt,
            &self.reaction_scratch,
            &self.accounting,
            true,
        )
    }

    pub fn apply_scheduled_interventions(&mut self, specs: &[InterventionSpec]) {
        for spec in specs {
            if let InterventionSpec::AtSubstep { substep, action } = spec {
                if self.substep == *substep {
                    let name = format!("{action:?}");
                    apply_intervention(&self.grid, &mut self.fields, action, &mut self.params);
                    self.interventions_applied.push((self.substep, name));
                }
            }
        }
    }

    pub fn snapshot(&self) -> FieldSnapshot {
        FieldSnapshot::from_sim(
            &self.fields,
            &self.params,
            self.substep,
            self.sim_time,
            &self.detector,
        )
    }

    pub fn restore_snapshot(&mut self, snap: &FieldSnapshot) {
        snap.restore_fields(&mut self.fields);
        self.params = snap.params.clone();
        self.substep = snap.substep;
        self.sim_time = snap.sim_time;
        self.detector.turnover = snap.turnover.clone();
        self.detector.last_classification = snap.classification;
    }

    /// Restore fields and timing only; candidate params remain from `Simulation::new`.
    pub fn restore_snapshot_fields_only(&mut self, snap: &FieldSnapshot) {
        self.try_restore_snapshot_fields_only(snap)
            .expect("snapshot equation and field schema must match target simulation");
    }

    pub fn try_restore_snapshot_fields_only(
        &mut self,
        snap: &FieldSnapshot,
    ) -> Result<(), String> {
        if self.params.equation_version != snap.equation_version {
            return Err(format!(
                "snapshot equation {} cannot be restored under {}",
                snap.equation_version, self.params.equation_version
            ));
        }
        snap.restore_fields(&mut self.fields);
        self.substep = snap.substep;
        self.sim_time = snap.sim_time;
        self.detector.turnover = snap.turnover.clone();
        self.detector.last_classification = snap.classification;
        Ok(())
    }

    pub fn field_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        for v in &self.fields.structure {
            v.to_bits().hash(&mut hasher);
        }
        for v in &self.fields.catalyst {
            v.to_bits().hash(&mut hasher);
        }
        for v in &self.fields.nutrient {
            v.to_bits().hash(&mut hasher);
        }
        for v in &self.fields.fuel {
            v.to_bits().hash(&mut hasher);
        }
        for v in &self.fields.waste {
            v.to_bits().hash(&mut hasher);
        }
        match self.params.equation_version {
            EquationVersion::MembraneMetabolismV1 => {
                for v in &self.fields.activated {
                    v.to_bits().hash(&mut hasher);
                }
                for v in &self.fields.membrane {
                    v.to_bits().hash(&mut hasher);
                }
            }
            EquationVersion::D001BulkV1
            | EquationVersion::D003CrowdingV1
            | EquationVersion::SurfaceTurnoverV1 => {}
        }
        hasher.finish()
    }
}

enum SubstepResult {
    Ok,
    Reject,
}

/// Run experiment with interventions, return final simulation state.
pub fn run_experiment(config: &ExperimentConfig, record_every: u64) -> Simulation {
    let mut sim = Simulation::from_config(config);
    sim.begin_timing();
    let total = config.substeps;
    for s in 0..total {
        sim.apply_scheduled_interventions(&config.interventions);
        if !sim.step() {
            break;
        }
        if record_every > 0 && sim.substep % record_every == 0 {
            let diag = sim.current_diagnostics();
            sim.history.push(diag);
        }
        let _ = s;
    }
    sim.finish_timing(sim.substep);
    sim
}

/// Run with observer disabled for causal-influence comparison.
pub fn run_experiment_no_observer(config: &ExperimentConfig) -> Simulation {
    let mut sim = Simulation::from_config(config);
    sim.observer_enabled = false;
    let total = config.substeps;
    for _ in 0..total {
        if !sim.step() {
            break;
        }
    }
    sim
}
