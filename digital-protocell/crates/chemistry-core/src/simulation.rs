//! Main simulation engine with adaptive timestep.

use crate::config::{ExperimentConfig, InterventionSpec, MAX_DT, SimParams};
use crate::diagnostics::{CellDetector, DiagnosticsSnapshot};
use crate::fields::{clamp_small_negative, initialize_seed, validate_field, FieldBuffers};
use crate::grid::Grid;
use crate::interventions::apply_intervention;
use crate::operators::{diffuse_constant, diffuse_variable, laplacian};
use crate::phase_field::{chemical_potential_local, compute_interior_weights, structure_rate};
use crate::reactions::{catalyst_diffusivity, compute_all_reactions, ReactionScratch};
use crate::reservoir::apply_reservoir;
use crate::snapshot::FieldSnapshot;

#[derive(Debug, Clone)]
pub struct Simulation {
    pub grid: Grid,
    pub params: SimParams,
    pub fields: FieldBuffers,
    pub working: FieldBuffers,
    pub reaction_scratch: ReactionScratch,
    pub detector: CellDetector,
    pub substep: u64,
    pub sim_time: f64,
    pub dt: f64,
    pub min_dt_seen: f64,
    pub rejection_count: u64,
    pub history: Vec<DiagnosticsSnapshot>,
    pub interventions_applied: Vec<(u64, String)>,
}

impl Simulation {
    pub fn new(params: SimParams) -> Self {
        let grid = Grid::new();
        let size = grid.width * grid.height;
        let mut fields = FieldBuffers::for_grid(&grid);
        initialize_seed(&grid, &params, &mut fields);
        Self {
            grid,
            params,
            fields,
            working: FieldBuffers::new(size),
            reaction_scratch: ReactionScratch::new(size),
            detector: CellDetector::default(),
            substep: 0,
            sim_time: 0.0,
            dt: MAX_DT,
            min_dt_seen: MAX_DT,
            rejection_count: 0,
            history: Vec::new(),
            interventions_applied: Vec::new(),
        }
    }

    pub fn from_config(config: &ExperimentConfig) -> Self {
        let mut sim = Self::new(config.params.clone());
        sim.params.random_seed = config.seed;
        initialize_seed(&sim.grid, &sim.params, &mut sim.fields);
        sim
    }

    pub fn reset(&mut self) {
        let params = self.params.clone();
        *self = Self::new(params);
    }

    pub fn run_substeps(&mut self, n: u64, record_every: u64) {
        for _ in 0..n {
            if !self.step() {
                break;
            }
            if record_every > 0 && self.substep % record_every == 0 {
                let snap = self.current_diagnostics();
                self.history.push(snap);
            }
        }
    }

    pub fn step(&mut self) -> bool {
        let mut attempt_dt = self.dt;
        let max_attempts = 20;

        for _ in 0..max_attempts {
            match self.try_substep(attempt_dt) {
                SubstepResult::Ok => {
                    self.sim_time += attempt_dt;
                    self.substep += 1;
                    self.dt = attempt_dt.min(MAX_DT);
                    return true;
                }
                SubstepResult::Reject => {
                    self.rejection_count += 1;
                    attempt_dt *= 0.5;
                    self.min_dt_seen = self.min_dt_seen.min(attempt_dt);
                    if attempt_dt < 1e-8 {
                        return false;
                    }
                }
            }
        }
        false
    }

    fn try_substep(&mut self, dt: f64) -> SubstepResult {
        let grid = &self.grid;
        let params = &self.params;

        // 1. Copy current to working, apply reservoir
        self.fields.copy_current_to_working(&mut self.working);
        apply_reservoir(
            grid,
            &mut self.working.nutrient,
            &mut self.working.fuel,
            &mut self.working.waste,
            dt,
            params,
        );

        let phi = &self.working.structure;
        let c = &self.working.catalyst;
        let n = &self.working.nutrient;
        let f = &self.working.fuel;
        let w = &self.working.waste;

        // 2. h(phi)
        compute_interior_weights(phi, &mut self.fields.scratch_h);

        // 3. laplacian(phi)
        laplacian(grid, phi, &mut self.fields.scratch_lap);

        // 4. chemical potential
        for idx in 0..grid.width * grid.height {
            if grid.in_dish(idx) {
                self.fields.scratch_mu[idx] =
                    chemical_potential_local(phi[idx], self.fields.scratch_lap[idx], params);
            } else {
                self.fields.scratch_mu[idx] = 0.0;
            }
        }

        // 5. laplacian(mu)
        laplacian(grid, &self.fields.scratch_mu, &mut self.fields.scratch_lap_mu);

        // 6. reactions
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

        // 7. catalyst diffusion
        for idx in 0..grid.width * grid.height {
            self.fields.scratch_h[idx] = catalyst_diffusivity(phi[idx], params);
        }
        diffuse_variable(grid, c, &self.fields.scratch_h, &mut self.fields.scratch_flux_x);

        // 8-10. nutrient, fuel, waste diffusion
        if params.diffusion_enabled {
            diffuse_constant(grid, n, params.d_n, &mut self.fields.scratch_lap, &mut self.fields.scratch_flux_y);
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

        // 11. integrate into next buffers
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

            self.fields.structure_next[idx] = phi[idx] + dphi * dt;
            self.fields.catalyst_next[idx] = c[idx] + (r.r_c + self.fields.scratch_flux_x[idx]) * dt;
            self.fields.nutrient_next[idx] = n[idx] + (r.r_n + nutrient_diff[idx]) * dt;
            self.fields.fuel_next[idx] = f[idx] + (r.r_f + fuel_diff[idx]) * dt;
            self.fields.waste_next[idx] = w[idx] + (r.r_w + waste_diff[idx]) * dt;
        }

        // 12-14. validate and clamp small negatives
        for field in [
            &mut self.fields.structure_next,
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

        // validate all fields
        if validate_field(&self.fields.structure_next, &grid.dish_mask).is_err()
            || validate_field(&self.fields.catalyst_next, &grid.dish_mask).is_err()
            || validate_field(&self.fields.nutrient_next, &grid.dish_mask).is_err()
            || validate_field(&self.fields.fuel_next, &grid.dish_mask).is_err()
            || validate_field(&self.fields.waste_next, &grid.dish_mask).is_err()
        {
            return SubstepResult::Reject;
        }

        // 16. swap
        self.fields.swap();

        // 17. diagnostics (on current after swap)
        let _ = self.detector.observe(
            grid,
            &self.fields,
            params,
            self.substep + 1,
            self.sim_time + dt,
            dt,
            &self.reaction_scratch,
        );

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
}

enum SubstepResult {
    Ok,
    Reject,
}

/// Run experiment with interventions, return final diagnostics.
pub fn run_experiment(config: &ExperimentConfig, record_every: u64) -> Simulation {
    let mut sim = Simulation::from_config(config);
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
    sim
}
