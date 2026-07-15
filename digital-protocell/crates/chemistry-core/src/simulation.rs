//! Main simulation engine with adaptive timestep and mass accounting.

use crate::accounting::{
    build_field_ledger, field_mass, sum_clamp_correction, AccountingState, ReactionStepTotals,
    StepAccounting,
};
use crate::activated_metabolism::{
    activated_metabolism_rates, ActivatedMetabolismAccountingState,
    ActivatedMetabolismStepAccounting,
};
use crate::config::{
    D008StageMode, EquationVersion, ExperimentConfig, InterventionSpec, SimParams, MAX_DT,
    NEG_CLAMP,
};
use crate::constraint_accounting::{build_constraint_step, StructureConstraintAccounting};
use crate::diagnostics::{CellDetector, DiagnosticsSnapshot};
use crate::fields::{
    clamp_small_negative, initialize_seed, validate_soluble_field, validate_structure_field,
    FieldBuffers,
};
use crate::grid::Grid;
use crate::interventions::apply_intervention;
use crate::membrane::{evolve_fixed_membrane, MembraneEvolutionTotals};
use crate::membrane_accounting::{
    MembraneAccountingState, MembraneStepAccounting, TransportAccountingState,
    TransportStepAccounting,
};
use crate::membrane_transport::{transport_field, TransportSpecies};
use crate::operators::{diffuse_constant, diffuse_variable, laplacian};
use crate::phase_field::{chemical_potential_local, compute_interior_weights, structure_rate};
use crate::reactions::{catalyst_diffusivity, compute_all_reactions, interface_weight, ReactionScratch};
use crate::reservoir::apply_reservoir;
use crate::snapshot::FieldSnapshot;
use crate::time_audit::DtTelemetry;
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
    pub transport_accounting: TransportAccountingState,
    pub membrane_accounting: MembraneAccountingState,
    pub metabolism_accounting: ActivatedMetabolismAccountingState,
    pub constraint_accounting: StructureConstraintAccounting,
    pub substep: u64,
    pub sim_time: f64,
    pub dt: f64,
    pub min_dt_seen: f64,
    pub rejection_count: u64,
    /// Accept-or-reject adaptive attempts (not biological time).
    pub attempted_substeps: u64,
    pub max_consecutive_rejections: u64,
    pub min_attempted_dt: f64,
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
            transport_accounting: TransportAccountingState::default(),
            membrane_accounting: MembraneAccountingState::default(),
            metabolism_accounting: ActivatedMetabolismAccountingState::default(),
            constraint_accounting: StructureConstraintAccounting::default(),
            substep: 0,
            sim_time: 0.0,
            dt: MAX_DT,
            min_dt_seen: MAX_DT,
            rejection_count: 0,
            attempted_substeps: 0,
            max_consecutive_rejections: 0,
            min_attempted_dt: MAX_DT,
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
        let mut consecutive_rejections = 0u64;

        for _ in 0..max_attempts {
            let result = match self.params.equation_version {
                EquationVersion::D001BulkV1
                | EquationVersion::D003CrowdingV1
                | EquationVersion::SurfaceTurnoverV1 => self.try_legacy_substep(attempt_dt),
                EquationVersion::MembraneMetabolismV1 | EquationVersion::MembraneMetabolismV2Conservative
                    if self.params.d008_stage_b_enabled =>
                {
                    self.try_d008_stage_b(attempt_dt)
                }
                EquationVersion::MembraneMetabolismV1
                | EquationVersion::MembraneMetabolismV2Conservative => {
                    match self.params.d008_stage_mode {
                        D008StageMode::Transport => {
                            self.try_membrane_metabolism_v1_transport(attempt_dt)
                        }
                        D008StageMode::ActivatedMetabolism => self.try_d008_stage_c(attempt_dt),
                        D008StageMode::FixedCompartment => self.try_d008_stage_d(attempt_dt),
                        D008StageMode::ConstrainedRadius => {
                            self.try_d008_constrained_radius(attempt_dt)
                        }
                    }
                }
            };
            self.attempted_substeps += 1;
            self.min_attempted_dt = self.min_attempted_dt.min(attempt_dt);
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
                    consecutive_rejections += 1;
                    self.max_consecutive_rejections = self
                        .max_consecutive_rejections
                        .max(consecutive_rejections);
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

        laplacian(
            grid,
            &self.fields.scratch_mu,
            &mut self.fields.scratch_lap_mu,
        );

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
        diffuse_variable(
            grid,
            c,
            &self.fields.scratch_h,
            &mut self.fields.scratch_flux_x,
        );

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
            diffuse_constant(
                grid,
                w,
                params.d_w,
                &mut self.fields.scratch_lap,
                waste_diff,
            );
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

        let clamp_total =
            sum_clamp_correction(&self.fields.structure, &self.fields.structure_next, grid)
                + sum_clamp_correction(&self.fields.catalyst, &self.fields.catalyst_next, grid)
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

    fn try_membrane_metabolism_v1_transport(&mut self, dt: f64) -> SubstepResult {
        self.fields.copy_current_to_next();

        let mut transport = TransportStepAccounting::default();
        if self.params.diffusion_enabled {
            transport.set(
                TransportSpecies::Catalyst,
                transport_field(
                    &self.grid,
                    TransportSpecies::Catalyst,
                    &self.fields.catalyst,
                    &self.fields.structure,
                    &self.fields.membrane,
                    &self.params,
                    &mut self.fields.scratch_transport_c,
                ),
            );
            transport.set(
                TransportSpecies::Activated,
                transport_field(
                    &self.grid,
                    TransportSpecies::Activated,
                    &self.fields.activated,
                    &self.fields.structure,
                    &self.fields.membrane,
                    &self.params,
                    &mut self.fields.scratch_transport_a,
                ),
            );
            transport.set(
                TransportSpecies::Nutrient,
                transport_field(
                    &self.grid,
                    TransportSpecies::Nutrient,
                    &self.fields.nutrient,
                    &self.fields.structure,
                    &self.fields.membrane,
                    &self.params,
                    &mut self.fields.scratch_transport_n,
                ),
            );
            transport.set(
                TransportSpecies::Fuel,
                transport_field(
                    &self.grid,
                    TransportSpecies::Fuel,
                    &self.fields.fuel,
                    &self.fields.structure,
                    &self.fields.membrane,
                    &self.params,
                    &mut self.fields.scratch_transport_f,
                ),
            );
            transport.set(
                TransportSpecies::Waste,
                transport_field(
                    &self.grid,
                    TransportSpecies::Waste,
                    &self.fields.waste,
                    &self.fields.structure,
                    &self.fields.membrane,
                    &self.params,
                    &mut self.fields.scratch_transport_w,
                ),
            );
        } else {
            self.fields.scratch_transport_c.fill(0.0);
            self.fields.scratch_transport_a.fill(0.0);
            self.fields.scratch_transport_n.fill(0.0);
            self.fields.scratch_transport_f.fill(0.0);
            self.fields.scratch_transport_w.fill(0.0);
        }

        for idx in 0..self.grid.width * self.grid.height {
            if !self.grid.in_dish(idx) {
                continue;
            }
            self.fields.catalyst_next[idx] =
                self.fields.catalyst[idx] + dt * self.fields.scratch_transport_c[idx];
            self.fields.activated_next[idx] =
                self.fields.activated[idx] + dt * self.fields.scratch_transport_a[idx];
            self.fields.nutrient_next[idx] =
                self.fields.nutrient[idx] + dt * self.fields.scratch_transport_n[idx];
            self.fields.fuel_next[idx] =
                self.fields.fuel[idx] + dt * self.fields.scratch_transport_f[idx];
            self.fields.waste_next[idx] =
                self.fields.waste[idx] + dt * self.fields.scratch_transport_w[idx];
        }

        let mass_phi_before = field_mass(&self.grid, &self.fields.structure);
        let mass_c_before = field_mass(&self.grid, &self.fields.catalyst);
        let mass_a_before = field_mass(&self.grid, &self.fields.activated);
        let mass_n_before = field_mass(&self.grid, &self.fields.nutrient);
        let mass_f_before = field_mass(&self.grid, &self.fields.fuel);
        let mass_w_before = field_mass(&self.grid, &self.fields.waste);
        let mass_m_before = field_mass(&self.grid, &self.fields.membrane);

        let pre_clamp_c = field_mass(&self.grid, &self.fields.catalyst_next);
        let pre_clamp_a = field_mass(&self.grid, &self.fields.activated_next);
        let pre_clamp_n = field_mass(&self.grid, &self.fields.nutrient_next);
        let pre_clamp_f = field_mass(&self.grid, &self.fields.fuel_next);
        let pre_clamp_w = field_mass(&self.grid, &self.fields.waste_next);

        for field in [
            &mut self.fields.catalyst_next,
            &mut self.fields.activated_next,
            &mut self.fields.nutrient_next,
            &mut self.fields.fuel_next,
            &mut self.fields.waste_next,
        ] {
            for (idx, value) in field.iter_mut().enumerate() {
                if self.grid.in_dish(idx) {
                    *value = clamp_small_negative(*value);
                }
            }
        }

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

        let mass_phi_after = field_mass(&self.grid, &self.fields.structure_next);
        let mass_c_after = field_mass(&self.grid, &self.fields.catalyst_next);
        let mass_a_after = field_mass(&self.grid, &self.fields.activated_next);
        let mass_n_after = field_mass(&self.grid, &self.fields.nutrient_next);
        let mass_f_after = field_mass(&self.grid, &self.fields.fuel_next);
        let mass_w_after = field_mass(&self.grid, &self.fields.waste_next);
        let mass_m_after = field_mass(&self.grid, &self.fields.membrane_next);

        let clamp_total = (mass_c_after - pre_clamp_c)
            + (mass_a_after - pre_clamp_a)
            + (mass_n_after - pre_clamp_n)
            + (mass_f_after - pre_clamp_f)
            + (mass_w_after - pre_clamp_w);

        let step_accounting = StepAccounting {
            structure: build_field_ledger(
                mass_phi_before,
                0.0,
                0.0,
                0.0,
                mass_phi_after,
                mass_phi_after,
            ),
            catalyst: build_field_ledger(
                mass_c_before,
                0.0,
                transport.catalyst.net_change_rate * dt,
                0.0,
                pre_clamp_c,
                mass_c_after,
            ),
            nutrient: build_field_ledger(
                mass_n_before,
                0.0,
                transport.nutrient.net_change_rate * dt,
                0.0,
                pre_clamp_n,
                mass_n_after,
            ),
            fuel: build_field_ledger(
                mass_f_before,
                0.0,
                transport.fuel.net_change_rate * dt,
                0.0,
                pre_clamp_f,
                mass_f_after,
            ),
            waste: build_field_ledger(
                mass_w_before,
                0.0,
                transport.waste.net_change_rate * dt,
                0.0,
                pre_clamp_w,
                mass_w_after,
            ),
            activated: build_field_ledger(
                mass_a_before,
                0.0,
                transport.activated.net_change_rate * dt,
                0.0,
                pre_clamp_a,
                mass_a_after,
            ),
            membrane: build_field_ledger(mass_m_before, 0.0, 0.0, 0.0, mass_m_after, mass_m_after),
        };
        self.accounting
            .record_step(step_accounting, &ReactionStepTotals::default(), clamp_total);
        self.transport_accounting.record_accepted(transport, dt);
        self.fields.swap();
        SubstepResult::Ok
    }

    fn try_d008_stage_b(&mut self, dt: f64) -> SubstepResult {
        if self
            .fields
            .membrane
            .iter()
            .enumerate()
            .any(|(idx, &value)| {
                self.grid.in_dish(idx)
                    && (!value.is_finite() || value < 0.0 || value > self.params.m_max)
            })
        {
            return SubstepResult::Reject;
        }

        let mass_phi = field_mass(&self.grid, &self.fields.structure);
        let mass_c = field_mass(&self.grid, &self.fields.catalyst);
        let mass_n = field_mass(&self.grid, &self.fields.nutrient);
        let mass_f = field_mass(&self.grid, &self.fields.fuel);
        let mass_w = field_mass(&self.grid, &self.fields.waste);
        let mass_a = field_mass(&self.grid, &self.fields.activated);
        let mass_m_before = field_mass(&self.grid, &self.fields.membrane);

        self.fields.copy_current_to_next();
        let evolution = evolve_fixed_membrane(
            &self.grid,
            &self.fields.structure,
            &self.fields.catalyst,
            &self.fields.activated,
            &self.fields.membrane,
            &self.params,
            dt,
            &mut self.fields.scratch_lap,
            &mut self.fields.scratch_transport_c,
            &mut self.fields.membrane_next,
            Some(&mut self.fields.activated_next),
            Some(&mut self.fields.waste_next),
        );
        let pre_clamp_m = field_mass(&self.grid, &self.fields.membrane_next);
        for (idx, value) in self.fields.membrane_next.iter_mut().enumerate() {
            if !self.grid.in_dish(idx) {
                continue;
            }
            if !value.is_finite() || *value < NEG_CLAMP {
                return SubstepResult::Reject;
            }
            *value = value.max(0.0).min(self.params.m_max);
        }

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

        let mass_m_after = field_mass(&self.grid, &self.fields.membrane_next);
        let mass_w_after = field_mass(&self.grid, &self.fields.waste_next);
        let mass_a_after = field_mass(&self.grid, &self.fields.activated_next);
        let membrane_step =
            build_membrane_step(mass_m_before, pre_clamp_m, mass_m_after, evolution, &self.params);
        let membrane_reaction = evolution.membrane_mass_reaction_delta(&self.params);
        let step_accounting = StepAccounting {
            structure: build_field_ledger(mass_phi, 0.0, 0.0, 0.0, mass_phi, mass_phi),
            catalyst: build_field_ledger(mass_c, 0.0, 0.0, 0.0, mass_c, mass_c),
            nutrient: build_field_ledger(mass_n, 0.0, 0.0, 0.0, mass_n, mass_n),
            fuel: build_field_ledger(mass_f, 0.0, 0.0, 0.0, mass_f, mass_f),
            waste: build_field_ledger(
                mass_w,
                evolution.waste_reaction_delta,
                0.0,
                0.0,
                mass_w_after,
                mass_w_after,
            ),
            activated: build_field_ledger(
                mass_a,
                evolution.activated_reaction_delta,
                0.0,
                0.0,
                mass_a_after,
                mass_a_after,
            ),
            membrane: build_field_ledger(
                mass_m_before,
                membrane_reaction,
                evolution.diffusion_delta,
                0.0,
                pre_clamp_m,
                mass_m_after,
            ),
        };
        self.accounting.record_step(
            step_accounting,
            &ReactionStepTotals::default(),
            membrane_step.clamp_correction,
        );
        self.membrane_accounting.record_accepted(membrane_step);
        self.fields.swap();
        SubstepResult::Ok
    }

    fn try_d008_stage_c(&mut self, dt: f64) -> SubstepResult {
        if validate_structure_field(&self.fields.structure, &self.grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.catalyst, &self.grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.nutrient, &self.grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.fuel, &self.grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.waste, &self.grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.activated, &self.grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.membrane, &self.grid.dish_mask).is_err()
            || self
                .fields
                .catalyst
                .iter()
                .enumerate()
                .any(|(idx, &value)| self.grid.in_dish(idx) && value > self.params.d008_c_max)
            || self
                .fields
                .activated
                .iter()
                .enumerate()
                .any(|(idx, &value)| self.grid.in_dish(idx) && value > self.params.d008_a_max)
        {
            return SubstepResult::Reject;
        }

        let mass_phi = field_mass(&self.grid, &self.fields.structure);
        let mass_c_before = field_mass(&self.grid, &self.fields.catalyst);
        let mass_n_before = field_mass(&self.grid, &self.fields.nutrient);
        let mass_f_before = field_mass(&self.grid, &self.fields.fuel);
        let mass_w_before = field_mass(&self.grid, &self.fields.waste);
        let mass_a_before = field_mass(&self.grid, &self.fields.activated);
        let mass_m = field_mass(&self.grid, &self.fields.membrane);
        self.fields.copy_current_to_next();

        let mut activation = 0.0;
        let mut reproduction = 0.0;
        let mut activated_decay = 0.0;
        let mut catalyst_turnover = 0.0;
        let mut react_c = 0.0;
        let mut react_n = 0.0;
        let mut react_f = 0.0;
        let mut react_a = 0.0;
        let mut react_w = 0.0;

        for idx in 0..self.grid.width * self.grid.height {
            if !self.grid.in_dish(idx) {
                continue;
            }
            let rates = activated_metabolism_rates(
                self.fields.catalyst[idx],
                self.fields.nutrient[idx],
                self.fields.fuel[idx],
                self.fields.activated[idx],
                &self.params,
            );
            activation += rates.activation * dt;
            reproduction += rates.reproduction * dt;
            activated_decay += rates.activated_decay * dt;
            catalyst_turnover += rates.catalyst_turnover * dt;
            react_c += rates.d_catalyst * dt;
            react_n += rates.d_nutrient * dt;
            react_f += rates.d_fuel * dt;
            react_a += rates.d_activated * dt;
            react_w += rates.d_waste * dt;
            self.fields.catalyst_next[idx] += rates.d_catalyst * dt;
            self.fields.nutrient_next[idx] += rates.d_nutrient * dt;
            self.fields.fuel_next[idx] += rates.d_fuel * dt;
            self.fields.activated_next[idx] += rates.d_activated * dt;
            self.fields.waste_next[idx] += rates.d_waste * dt;
        }

        let pre_clamp_c = field_mass(&self.grid, &self.fields.catalyst_next);
        let pre_clamp_n = field_mass(&self.grid, &self.fields.nutrient_next);
        let pre_clamp_f = field_mass(&self.grid, &self.fields.fuel_next);
        let pre_clamp_w = field_mass(&self.grid, &self.fields.waste_next);
        let pre_clamp_a = field_mass(&self.grid, &self.fields.activated_next);

        for idx in 0..self.grid.width * self.grid.height {
            if !self.grid.in_dish(idx) {
                continue;
            }
            for value in [
                self.fields.catalyst_next[idx],
                self.fields.nutrient_next[idx],
                self.fields.fuel_next[idx],
                self.fields.waste_next[idx],
                self.fields.activated_next[idx],
            ] {
                if !value.is_finite() || value < NEG_CLAMP {
                    return SubstepResult::Reject;
                }
            }
            self.fields.catalyst_next[idx] = self.fields.catalyst_next[idx]
                .max(0.0)
                .min(self.params.d008_c_max);
            self.fields.activated_next[idx] = self.fields.activated_next[idx]
                .max(0.0)
                .min(self.params.d008_a_max);
            self.fields.nutrient_next[idx] = clamp_small_negative(self.fields.nutrient_next[idx]);
            self.fields.fuel_next[idx] = clamp_small_negative(self.fields.fuel_next[idx]);
            self.fields.waste_next[idx] = clamp_small_negative(self.fields.waste_next[idx]);
        }

        if validate_soluble_field(&self.fields.catalyst_next, &self.grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.nutrient_next, &self.grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.fuel_next, &self.grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.waste_next, &self.grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.activated_next, &self.grid.dish_mask).is_err()
        {
            return SubstepResult::Reject;
        }

        let mass_c_after = field_mass(&self.grid, &self.fields.catalyst_next);
        let mass_n_after = field_mass(&self.grid, &self.fields.nutrient_next);
        let mass_f_after = field_mass(&self.grid, &self.fields.fuel_next);
        let mass_w_after = field_mass(&self.grid, &self.fields.waste_next);
        let mass_a_after = field_mass(&self.grid, &self.fields.activated_next);
        let catalyst =
            build_field_ledger(mass_c_before, react_c, 0.0, 0.0, pre_clamp_c, mass_c_after);
        let nutrient =
            build_field_ledger(mass_n_before, react_n, 0.0, 0.0, pre_clamp_n, mass_n_after);
        let fuel = build_field_ledger(mass_f_before, react_f, 0.0, 0.0, pre_clamp_f, mass_f_after);
        let waste = build_field_ledger(mass_w_before, react_w, 0.0, 0.0, pre_clamp_w, mass_w_after);
        let activated =
            build_field_ledger(mass_a_before, react_a, 0.0, 0.0, pre_clamp_a, mass_a_after);
        let metabolism_step = ActivatedMetabolismStepAccounting {
            activation,
            reproduction,
            activated_decay,
            catalyst_turnover,
            catalyst: catalyst.clone(),
            nutrient: nutrient.clone(),
            fuel: fuel.clone(),
            activated: activated.clone(),
            waste: waste.clone(),
        };
        let step_accounting = StepAccounting {
            structure: build_field_ledger(mass_phi, 0.0, 0.0, 0.0, mass_phi, mass_phi),
            catalyst,
            nutrient,
            fuel,
            waste,
            activated,
            membrane: build_field_ledger(mass_m, 0.0, 0.0, 0.0, mass_m, mass_m),
        };
        let reaction_totals = ReactionStepTotals {
            catalyst_reproduction: reproduction,
            catalyst_decay: catalyst_turnover,
            nutrient_consumed_r1: activation,
            fuel_consumed_r1: activation,
            waste_from_r1: activation,
            waste_from_r2: reproduction,
            waste_from_decay: activated_decay + catalyst_turnover,
            ..ReactionStepTotals::default()
        };
        let clamp_total = step_accounting.catalyst.numerical_correction_delta
            + step_accounting.nutrient.numerical_correction_delta
            + step_accounting.fuel.numerical_correction_delta
            + step_accounting.waste.numerical_correction_delta
            + step_accounting.activated.numerical_correction_delta;
        self.accounting
            .record_step(step_accounting, &reaction_totals, clamp_total);
        self.metabolism_accounting.record_accepted(metabolism_step);
        self.fields.swap();
        SubstepResult::Ok
    }

    fn try_d008_stage_d(&mut self, dt: f64) -> SubstepResult {
        if validate_structure_field(&self.fields.structure, &self.grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.catalyst, &self.grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.nutrient, &self.grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.fuel, &self.grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.waste, &self.grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.activated, &self.grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.membrane, &self.grid.dish_mask).is_err()
            || self
                .fields
                .catalyst
                .iter()
                .enumerate()
                .any(|(idx, &value)| self.grid.in_dish(idx) && value > self.params.d008_c_max)
            || self
                .fields
                .activated
                .iter()
                .enumerate()
                .any(|(idx, &value)| self.grid.in_dish(idx) && value > self.params.d008_a_max)
        {
            return SubstepResult::Reject;
        }

        let mass_phi = field_mass(&self.grid, &self.fields.structure);
        let mass_c_before = field_mass(&self.grid, &self.fields.catalyst);
        let mass_n_before = field_mass(&self.grid, &self.fields.nutrient);
        let mass_f_before = field_mass(&self.grid, &self.fields.fuel);
        let mass_w_before = field_mass(&self.grid, &self.fields.waste);
        let mass_a_before = field_mass(&self.grid, &self.fields.activated);
        let mass_m = field_mass(&self.grid, &self.fields.membrane);

        self.fields.copy_current_to_working(&mut self.working);
        apply_reservoir(
            &self.grid,
            &mut self.working.nutrient,
            &mut self.working.fuel,
            &mut self.working.waste,
            dt,
            &self.params,
        );
        let n_reservoir_delta = field_mass(&self.grid, &self.working.nutrient) - mass_n_before;
        let f_reservoir_delta = field_mass(&self.grid, &self.working.fuel) - mass_f_before;
        let w_reservoir_delta = field_mass(&self.grid, &self.working.waste) - mass_w_before;

        let mut transport = TransportStepAccounting::default();
        if self.params.diffusion_enabled {
            transport.set(
                TransportSpecies::Catalyst,
                transport_field(
                    &self.grid,
                    TransportSpecies::Catalyst,
                    &self.working.catalyst,
                    &self.working.structure,
                    &self.working.membrane,
                    &self.params,
                    &mut self.fields.scratch_transport_c,
                ),
            );
            transport.set(
                TransportSpecies::Activated,
                transport_field(
                    &self.grid,
                    TransportSpecies::Activated,
                    &self.working.activated,
                    &self.working.structure,
                    &self.working.membrane,
                    &self.params,
                    &mut self.fields.scratch_transport_a,
                ),
            );
            transport.set(
                TransportSpecies::Nutrient,
                transport_field(
                    &self.grid,
                    TransportSpecies::Nutrient,
                    &self.working.nutrient,
                    &self.working.structure,
                    &self.working.membrane,
                    &self.params,
                    &mut self.fields.scratch_transport_n,
                ),
            );
            transport.set(
                TransportSpecies::Fuel,
                transport_field(
                    &self.grid,
                    TransportSpecies::Fuel,
                    &self.working.fuel,
                    &self.working.structure,
                    &self.working.membrane,
                    &self.params,
                    &mut self.fields.scratch_transport_f,
                ),
            );
            transport.set(
                TransportSpecies::Waste,
                transport_field(
                    &self.grid,
                    TransportSpecies::Waste,
                    &self.working.waste,
                    &self.working.structure,
                    &self.working.membrane,
                    &self.params,
                    &mut self.fields.scratch_transport_w,
                ),
            );
        } else {
            self.fields.scratch_transport_c.fill(0.0);
            self.fields.scratch_transport_a.fill(0.0);
            self.fields.scratch_transport_n.fill(0.0);
            self.fields.scratch_transport_f.fill(0.0);
            self.fields.scratch_transport_w.fill(0.0);
        }

        self.fields.copy_current_to_next();
        let mut activation = 0.0;
        let mut reproduction = 0.0;
        let mut activated_decay = 0.0;
        let mut catalyst_turnover = 0.0;
        let mut react_c = 0.0;
        let mut react_n = 0.0;
        let mut react_f = 0.0;
        let mut react_a = 0.0;
        let mut react_w = 0.0;
        for idx in 0..self.grid.width * self.grid.height {
            if !self.grid.in_dish(idx) {
                continue;
            }
            let rates = activated_metabolism_rates(
                self.working.catalyst[idx],
                self.working.nutrient[idx],
                self.working.fuel[idx],
                self.working.activated[idx],
                &self.params,
            );
            activation += rates.activation * dt;
            reproduction += rates.reproduction * dt;
            activated_decay += rates.activated_decay * dt;
            catalyst_turnover += rates.catalyst_turnover * dt;
            react_c += rates.d_catalyst * dt;
            react_n += rates.d_nutrient * dt;
            react_f += rates.d_fuel * dt;
            react_a += rates.d_activated * dt;
            react_w += rates.d_waste * dt;
            self.fields.catalyst_next[idx] = self.working.catalyst[idx]
                + dt * (self.fields.scratch_transport_c[idx] + rates.d_catalyst);
            self.fields.nutrient_next[idx] = self.working.nutrient[idx]
                + dt * (self.fields.scratch_transport_n[idx] + rates.d_nutrient);
            self.fields.fuel_next[idx] =
                self.working.fuel[idx] + dt * (self.fields.scratch_transport_f[idx] + rates.d_fuel);
            self.fields.waste_next[idx] = self.working.waste[idx]
                + dt * (self.fields.scratch_transport_w[idx] + rates.d_waste);
            self.fields.activated_next[idx] = self.working.activated[idx]
                + dt * (self.fields.scratch_transport_a[idx] + rates.d_activated);
        }

        let pre_clamp_c = field_mass(&self.grid, &self.fields.catalyst_next);
        let pre_clamp_n = field_mass(&self.grid, &self.fields.nutrient_next);
        let pre_clamp_f = field_mass(&self.grid, &self.fields.fuel_next);
        let pre_clamp_w = field_mass(&self.grid, &self.fields.waste_next);
        let pre_clamp_a = field_mass(&self.grid, &self.fields.activated_next);
        for idx in 0..self.grid.width * self.grid.height {
            if !self.grid.in_dish(idx) {
                continue;
            }
            for value in [
                self.fields.catalyst_next[idx],
                self.fields.nutrient_next[idx],
                self.fields.fuel_next[idx],
                self.fields.waste_next[idx],
                self.fields.activated_next[idx],
            ] {
                if !value.is_finite() || value < NEG_CLAMP {
                    return SubstepResult::Reject;
                }
            }
            self.fields.catalyst_next[idx] = self.fields.catalyst_next[idx]
                .max(0.0)
                .min(self.params.d008_c_max);
            self.fields.activated_next[idx] = self.fields.activated_next[idx]
                .max(0.0)
                .min(self.params.d008_a_max);
            self.fields.nutrient_next[idx] = clamp_small_negative(self.fields.nutrient_next[idx]);
            self.fields.fuel_next[idx] = clamp_small_negative(self.fields.fuel_next[idx]);
            self.fields.waste_next[idx] = clamp_small_negative(self.fields.waste_next[idx]);
        }
        if validate_soluble_field(&self.fields.catalyst_next, &self.grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.nutrient_next, &self.grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.fuel_next, &self.grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.waste_next, &self.grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.activated_next, &self.grid.dish_mask).is_err()
        {
            return SubstepResult::Reject;
        }

        let mass_c_after = field_mass(&self.grid, &self.fields.catalyst_next);
        let mass_n_after = field_mass(&self.grid, &self.fields.nutrient_next);
        let mass_f_after = field_mass(&self.grid, &self.fields.fuel_next);
        let mass_w_after = field_mass(&self.grid, &self.fields.waste_next);
        let mass_a_after = field_mass(&self.grid, &self.fields.activated_next);
        let catalyst = build_field_ledger(
            mass_c_before,
            react_c,
            transport.catalyst.net_change_rate * dt,
            0.0,
            pre_clamp_c,
            mass_c_after,
        );
        let nutrient = build_field_ledger(
            mass_n_before,
            react_n,
            transport.nutrient.net_change_rate * dt,
            n_reservoir_delta,
            pre_clamp_n,
            mass_n_after,
        );
        let fuel = build_field_ledger(
            mass_f_before,
            react_f,
            transport.fuel.net_change_rate * dt,
            f_reservoir_delta,
            pre_clamp_f,
            mass_f_after,
        );
        let waste = build_field_ledger(
            mass_w_before,
            react_w,
            transport.waste.net_change_rate * dt,
            w_reservoir_delta,
            pre_clamp_w,
            mass_w_after,
        );
        let activated = build_field_ledger(
            mass_a_before,
            react_a,
            transport.activated.net_change_rate * dt,
            0.0,
            pre_clamp_a,
            mass_a_after,
        );
        let metabolism_step = ActivatedMetabolismStepAccounting {
            activation,
            reproduction,
            activated_decay,
            catalyst_turnover,
            catalyst: catalyst.clone(),
            nutrient: nutrient.clone(),
            fuel: fuel.clone(),
            activated: activated.clone(),
            waste: waste.clone(),
        };
        let step_accounting = StepAccounting {
            structure: build_field_ledger(mass_phi, 0.0, 0.0, 0.0, mass_phi, mass_phi),
            catalyst,
            nutrient,
            fuel,
            waste,
            activated,
            membrane: build_field_ledger(mass_m, 0.0, 0.0, 0.0, mass_m, mass_m),
        };
        let reaction_totals = ReactionStepTotals {
            catalyst_reproduction: reproduction,
            catalyst_decay: catalyst_turnover,
            nutrient_consumed_r1: activation,
            fuel_consumed_r1: activation,
            waste_from_r1: activation,
            waste_from_r2: reproduction,
            waste_from_decay: activated_decay + catalyst_turnover,
            ..ReactionStepTotals::default()
        };
        let clamp_total = step_accounting.catalyst.numerical_correction_delta
            + step_accounting.nutrient.numerical_correction_delta
            + step_accounting.fuel.numerical_correction_delta
            + step_accounting.waste.numerical_correction_delta
            + step_accounting.activated.numerical_correction_delta;
        self.accounting
            .record_step(step_accounting, &reaction_totals, clamp_total);
        self.transport_accounting.record_accepted(transport, dt);
        self.metabolism_accounting.record_accepted(metabolism_step);
        self.fields.swap();
        SubstepResult::Ok
    }

    fn try_d008_constrained_radius(&mut self, dt: f64) -> SubstepResult {
        if validate_structure_field(&self.fields.structure, &self.grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.catalyst, &self.grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.nutrient, &self.grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.fuel, &self.grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.waste, &self.grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.activated, &self.grid.dish_mask).is_err()
            || validate_soluble_field(&self.fields.membrane, &self.grid.dish_mask).is_err()
            || self
                .fields
                .catalyst
                .iter()
                .enumerate()
                .any(|(idx, &value)| self.grid.in_dish(idx) && value > self.params.d008_c_max)
            || self
                .fields
                .activated
                .iter()
                .enumerate()
                .any(|(idx, &value)| self.grid.in_dish(idx) && value > self.params.d008_a_max)
        {
            return SubstepResult::Reject;
        }

        let mass_phi = field_mass(&self.grid, &self.fields.structure);
        let mass_c_before = field_mass(&self.grid, &self.fields.catalyst);
        let mass_n_before = field_mass(&self.grid, &self.fields.nutrient);
        let mass_f_before = field_mass(&self.grid, &self.fields.fuel);
        let mass_w_before = field_mass(&self.grid, &self.fields.waste);
        let mass_a_before = field_mass(&self.grid, &self.fields.activated);
        let mass_m_before = field_mass(&self.grid, &self.fields.membrane);

        self.fields.copy_current_to_working(&mut self.working);
        apply_reservoir(
            &self.grid,
            &mut self.working.nutrient,
            &mut self.working.fuel,
            &mut self.working.waste,
            dt,
            &self.params,
        );
        let n_reservoir_delta = field_mass(&self.grid, &self.working.nutrient) - mass_n_before;
        let f_reservoir_delta = field_mass(&self.grid, &self.working.fuel) - mass_f_before;
        let w_reservoir_delta = field_mass(&self.grid, &self.working.waste) - mass_w_before;

        let mut transport = TransportStepAccounting::default();
        if self.params.diffusion_enabled {
            transport.set(
                TransportSpecies::Catalyst,
                transport_field(
                    &self.grid,
                    TransportSpecies::Catalyst,
                    &self.working.catalyst,
                    &self.working.structure,
                    &self.working.membrane,
                    &self.params,
                    &mut self.fields.scratch_transport_c,
                ),
            );
            transport.set(
                TransportSpecies::Activated,
                transport_field(
                    &self.grid,
                    TransportSpecies::Activated,
                    &self.working.activated,
                    &self.working.structure,
                    &self.working.membrane,
                    &self.params,
                    &mut self.fields.scratch_transport_a,
                ),
            );
            transport.set(
                TransportSpecies::Nutrient,
                transport_field(
                    &self.grid,
                    TransportSpecies::Nutrient,
                    &self.working.nutrient,
                    &self.working.structure,
                    &self.working.membrane,
                    &self.params,
                    &mut self.fields.scratch_transport_n,
                ),
            );
            transport.set(
                TransportSpecies::Fuel,
                transport_field(
                    &self.grid,
                    TransportSpecies::Fuel,
                    &self.working.fuel,
                    &self.working.structure,
                    &self.working.membrane,
                    &self.params,
                    &mut self.fields.scratch_transport_f,
                ),
            );
            transport.set(
                TransportSpecies::Waste,
                transport_field(
                    &self.grid,
                    TransportSpecies::Waste,
                    &self.working.waste,
                    &self.working.structure,
                    &self.working.membrane,
                    &self.params,
                    &mut self.fields.scratch_transport_w,
                ),
            );
        } else {
            self.fields.scratch_transport_c.fill(0.0);
            self.fields.scratch_transport_a.fill(0.0);
            self.fields.scratch_transport_n.fill(0.0);
            self.fields.scratch_transport_f.fill(0.0);
            self.fields.scratch_transport_w.fill(0.0);
        }

        self.fields.copy_current_to_next();
        let mut activation = 0.0;
        let mut reproduction = 0.0;
        let mut activated_decay = 0.0;
        let mut catalyst_turnover = 0.0;
        let mut react_c = 0.0;
        let mut react_n = 0.0;
        let mut react_f = 0.0;
        let mut react_a = 0.0;
        let mut react_w = 0.0;
        let mut virtual_production = 0.0;
        let mut virtual_decay = 0.0;
        let v2 = self.params.equation_version == EquationVersion::MembraneMetabolismV2Conservative;
        let eta_phi = if v2 { self.params.eta_phi } else { 1.0 };
        for idx in 0..self.grid.width * self.grid.height {
            if !self.grid.in_dish(idx) {
                continue;
            }
            let phi = self.working.structure[idx];
            let i_face = interface_weight(phi);
            let r_structure = self.params.k_d008_structure * self.working.activated[idx] * i_face;
            let r_structure_decay = self.params.k_structure_decay * phi;
            virtual_production += if v2 {
                eta_phi * r_structure * dt
            } else {
                r_structure * dt
            };
            virtual_decay += r_structure_decay * dt;
            let d_a_structure = -r_structure * dt;
            let d_w_structure = if v2 {
                (1.0 - eta_phi) * r_structure * dt + r_structure_decay * dt
            } else {
                r_structure_decay * dt
            };
            react_a += d_a_structure;
            react_w += d_w_structure;

            let rates = activated_metabolism_rates(
                self.working.catalyst[idx],
                self.working.nutrient[idx],
                self.working.fuel[idx],
                self.working.activated[idx],
                &self.params,
            );
            activation += rates.activation * dt;
            reproduction += rates.reproduction * dt;
            activated_decay += rates.activated_decay * dt;
            catalyst_turnover += rates.catalyst_turnover * dt;
            react_c += rates.d_catalyst * dt;
            react_n += rates.d_nutrient * dt;
            react_f += rates.d_fuel * dt;
            react_a += rates.d_activated * dt;
            react_w += rates.d_waste * dt;
            self.fields.catalyst_next[idx] = self.working.catalyst[idx]
                + dt * (self.fields.scratch_transport_c[idx] + rates.d_catalyst);
            self.fields.nutrient_next[idx] = self.working.nutrient[idx]
                + dt * (self.fields.scratch_transport_n[idx] + rates.d_nutrient);
            self.fields.fuel_next[idx] =
                self.working.fuel[idx] + dt * (self.fields.scratch_transport_f[idx] + rates.d_fuel);
            self.fields.waste_next[idx] = self.working.waste[idx]
                + dt * (self.fields.scratch_transport_w[idx] + rates.d_waste)
                + d_w_structure;
            self.fields.activated_next[idx] = self.working.activated[idx]
                + dt * (self.fields.scratch_transport_a[idx] + rates.d_activated)
                + d_a_structure;
        }

        let evolution = evolve_fixed_membrane(
            &self.grid,
            &self.fields.structure,
            &self.fields.catalyst,
            &self.fields.activated,
            &self.fields.membrane,
            &self.params,
            dt,
            &mut self.fields.scratch_lap,
            &mut self.fields.scratch_transport_c,
            &mut self.fields.membrane_next,
            Some(&mut self.fields.activated_next),
            Some(&mut self.fields.waste_next),
        );
        react_a += evolution.activated_reaction_delta;
        react_w += evolution.waste_reaction_delta;

        let pre_clamp_c = field_mass(&self.grid, &self.fields.catalyst_next);
        let pre_clamp_n = field_mass(&self.grid, &self.fields.nutrient_next);
        let pre_clamp_f = field_mass(&self.grid, &self.fields.fuel_next);
        let pre_clamp_w = field_mass(&self.grid, &self.fields.waste_next);
        let pre_clamp_a = field_mass(&self.grid, &self.fields.activated_next);
        let pre_clamp_m = field_mass(&self.grid, &self.fields.membrane_next);
        for idx in 0..self.grid.width * self.grid.height {
            if !self.grid.in_dish(idx) {
                continue;
            }
            for value in [
                self.fields.catalyst_next[idx],
                self.fields.nutrient_next[idx],
                self.fields.fuel_next[idx],
                self.fields.waste_next[idx],
                self.fields.activated_next[idx],
            ] {
                if !value.is_finite() || value < NEG_CLAMP {
                    return SubstepResult::Reject;
                }
            }
            self.fields.catalyst_next[idx] = self.fields.catalyst_next[idx]
                .max(0.0)
                .min(self.params.d008_c_max);
            self.fields.activated_next[idx] = self.fields.activated_next[idx]
                .max(0.0)
                .min(self.params.d008_a_max);
            self.fields.nutrient_next[idx] = clamp_small_negative(self.fields.nutrient_next[idx]);
            self.fields.fuel_next[idx] = clamp_small_negative(self.fields.fuel_next[idx]);
            self.fields.waste_next[idx] = clamp_small_negative(self.fields.waste_next[idx]);
        }
        for (idx, value) in self.fields.membrane_next.iter_mut().enumerate() {
            if !self.grid.in_dish(idx) {
                continue;
            }
            if !value.is_finite() || *value < NEG_CLAMP {
                return SubstepResult::Reject;
            }
            *value = value.max(0.0).min(self.params.m_max);
        }

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

        let mass_c_after = field_mass(&self.grid, &self.fields.catalyst_next);
        let mass_n_after = field_mass(&self.grid, &self.fields.nutrient_next);
        let mass_f_after = field_mass(&self.grid, &self.fields.fuel_next);
        let mass_w_after = field_mass(&self.grid, &self.fields.waste_next);
        let mass_a_after = field_mass(&self.grid, &self.fields.activated_next);
        let mass_m_after = field_mass(&self.grid, &self.fields.membrane_next);
        let catalyst = build_field_ledger(
            mass_c_before,
            react_c,
            transport.catalyst.net_change_rate * dt,
            0.0,
            pre_clamp_c,
            mass_c_after,
        );
        let nutrient = build_field_ledger(
            mass_n_before,
            react_n,
            transport.nutrient.net_change_rate * dt,
            n_reservoir_delta,
            pre_clamp_n,
            mass_n_after,
        );
        let fuel = build_field_ledger(
            mass_f_before,
            react_f,
            transport.fuel.net_change_rate * dt,
            f_reservoir_delta,
            pre_clamp_f,
            mass_f_after,
        );
        let waste = build_field_ledger(
            mass_w_before,
            react_w,
            transport.waste.net_change_rate * dt,
            w_reservoir_delta,
            pre_clamp_w,
            mass_w_after,
        );
        let activated = build_field_ledger(
            mass_a_before,
            react_a,
            transport.activated.net_change_rate * dt,
            0.0,
            pre_clamp_a,
            mass_a_after,
        );
        let metabolism_step = ActivatedMetabolismStepAccounting {
            activation,
            reproduction,
            activated_decay,
            catalyst_turnover,
            catalyst: catalyst.clone(),
            nutrient: nutrient.clone(),
            fuel: fuel.clone(),
            activated: activated.clone(),
            waste: waste.clone(),
        };
        let membrane_step =
            build_membrane_step(mass_m_before, pre_clamp_m, mass_m_after, evolution, &self.params);
        let constraint_step = build_constraint_step(virtual_production, virtual_decay);
        let step_accounting = StepAccounting {
            structure: build_field_ledger(mass_phi, 0.0, 0.0, 0.0, mass_phi, mass_phi),
            catalyst,
            nutrient,
            fuel,
            waste,
            activated,
            membrane: build_field_ledger(
                mass_m_before,
                evolution.membrane_mass_reaction_delta(&self.params),
                evolution.diffusion_delta,
                0.0,
                pre_clamp_m,
                mass_m_after,
            ),
        };
        let reaction_totals = ReactionStepTotals {
            catalyst_reproduction: reproduction,
            catalyst_decay: catalyst_turnover,
            nutrient_consumed_r1: activation,
            nutrient_consumed_r2: 0.0,
            fuel_consumed_r1: activation,
            fuel_consumed_r2: 0.0,
            waste_from_r1: activation,
            waste_from_r2: reproduction,
            waste_from_decay: activated_decay + catalyst_turnover,
            structural_synthesis: virtual_production,
            structural_decay: virtual_decay,
        };
        let clamp_total = step_accounting.catalyst.numerical_correction_delta
            + step_accounting.nutrient.numerical_correction_delta
            + step_accounting.fuel.numerical_correction_delta
            + step_accounting.waste.numerical_correction_delta
            + step_accounting.activated.numerical_correction_delta
            + step_accounting.membrane.numerical_correction_delta;
        self.accounting
            .record_step(step_accounting, &reaction_totals, clamp_total);
        self.transport_accounting.record_accepted(transport, dt);
        self.metabolism_accounting.record_accepted(metabolism_step);
        self.membrane_accounting.record_accepted(membrane_step);
        self.constraint_accounting.record_accepted(constraint_step);
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

    /// Compatibility wrapper. Prefer [`Self::try_restore_snapshot`] at trust boundaries.
    pub fn restore_snapshot(&mut self, snap: &FieldSnapshot) {
        self.try_restore_snapshot(snap)
            .expect("snapshot restore failed schema or length validation");
    }

    pub fn try_restore_snapshot(&mut self, snap: &FieldSnapshot) -> Result<(), String> {
        snap.try_restore_fields(&mut self.fields)?;
        self.params = snap.params.clone();
        self.substep = snap.substep;
        self.sim_time = snap.sim_time;
        self.detector.turnover = snap.turnover.clone();
        self.detector.last_classification = snap.classification;
        Ok(())
    }

    /// Restore fields and timing only; candidate params remain from `Simulation::new`.
    pub fn restore_snapshot_fields_only(&mut self, snap: &FieldSnapshot) {
        self.try_restore_snapshot_fields_only(snap)
            .expect("snapshot equation and field schema must match target simulation");
    }

    pub fn try_restore_snapshot_fields_only(&mut self, snap: &FieldSnapshot) -> Result<(), String> {
        if self.params.equation_version != snap.equation_version {
            return Err(format!(
                "snapshot equation {} cannot be restored under {}",
                snap.equation_version, self.params.equation_version
            ));
        }
        snap.try_restore_fields(&mut self.fields)?;
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
            EquationVersion::MembraneMetabolismV1 | EquationVersion::MembraneMetabolismV2Conservative => {
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

    /// Stable cross-toolchain field digest for reproducibility checks.
    pub fn stable_field_digest(&self) -> String {
        use crate::candidate_identity::sha256_hex;
        let mut bytes = Vec::new();
        append_field_bits(&mut bytes, &self.fields.structure);
        append_field_bits(&mut bytes, &self.fields.catalyst);
        append_field_bits(&mut bytes, &self.fields.nutrient);
        append_field_bits(&mut bytes, &self.fields.fuel);
        append_field_bits(&mut bytes, &self.fields.waste);
        match self.params.equation_version {
            EquationVersion::MembraneMetabolismV1 | EquationVersion::MembraneMetabolismV2Conservative => {
                append_field_bits(&mut bytes, &self.fields.activated);
                append_field_bits(&mut bytes, &self.fields.membrane);
            }
            EquationVersion::D001BulkV1
            | EquationVersion::D003CrowdingV1
            | EquationVersion::SurfaceTurnoverV1 => {}
        }
        sha256_hex(&bytes)
    }
}

fn append_field_bits(out: &mut Vec<u8>, field: &[f64]) {
    for v in field {
        out.extend_from_slice(&v.to_bits().to_le_bytes());
    }
}

enum SubstepResult {
    Ok,
    Reject,
}

fn build_membrane_step(
    mass_before: f64,
    pre_clamp_mass: f64,
    mass_after: f64,
    evolution: MembraneEvolutionTotals,
    params: &SimParams,
) -> MembraneStepAccounting {
    let clamp_correction = mass_after - pre_clamp_mass;
    let reaction_delta = evolution.membrane_mass_reaction_delta(params);
    let residual = mass_after
        - (mass_before + reaction_delta + evolution.diffusion_delta + clamp_correction);
    MembraneStepAccounting {
        mass_before,
        synthesis: evolution.synthesis_delta,
        decay: evolution.decay_delta,
        detachment: evolution.detachment_delta,
        diffusion_net: evolution.diffusion_delta,
        pre_clamp_mass,
        clamp_correction,
        mass_after,
        residual,
    }
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
