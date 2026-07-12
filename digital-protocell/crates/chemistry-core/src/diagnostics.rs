//! Observer diagnostics and viability classification.

use crate::accounting::AccountingState;
use crate::config::SimParams;
use crate::fields::{interior_weight, FieldBuffers};
use crate::grid::Grid;
use crate::reactions::ReactionScratch;
use serde::{Deserialize, Serialize};

pub const VIABILITY_WINDOW: u64 = 25_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ViabilityClass {
    Seeding,
    Transient,
    Viable,
    Degraded,
    Collapsed,
    Dead,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TurnoverTotals {
    pub structural_synthesis: f64,
    pub structural_decay: f64,
    pub catalyst_reproduction: f64,
    pub catalyst_decay: f64,
    pub nutrient_consumption: f64,
    pub fuel_consumption: f64,
    pub waste_production: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TurnoverRatios {
    pub structural_replacement: f64,
    pub structural_synthesis: f64,
    pub catalyst_replacement: f64,
    pub catalyst_reproduction: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsSnapshot {
    pub substep: u64,
    pub sim_time: f64,
    pub dt: f64,
    pub structural_mass: f64,
    pub catalyst_mass: f64,
    pub catalyst_retention: f64,
    pub protocell_area: u64,
    pub largest_component: u64,
    pub compactness: f64,
    pub nutrient_consumption_rate: f64,
    pub fuel_consumption_rate: f64,
    pub waste_production_rate: f64,
    pub classification: ViabilityClass,
    pub turnover_ratios: TurnoverRatios,
    pub structure_cv: f64,
    pub catalyst_cv: f64,
    pub consecutive_viable_windows: u32,
}

#[derive(Debug, Clone, Default)]
struct WindowSample {
    structural_mass: f64,
    catalyst_mass: f64,
    catalyst_retention: f64,
    largest_component: u64,
    protocell_area: u64,
    structural_synthesis: f64,
    structural_decay: f64,
    catalyst_reproduction: f64,
    catalyst_decay: f64,
    nutrient_consumption: f64,
    fuel_consumption: f64,
    waste_production: f64,
}

#[derive(Debug, Clone)]
pub struct CellDetector {
    pub extinction_counter: u64,
    pub last_classification: ViabilityClass,
    pub turnover: TurnoverTotals,
    pub rolling_structure: Vec<f64>,
    pub rolling_catalyst: Vec<f64>,
    pub pre_damage_structure_mean: f64,
    pub pre_damage_retention: f64,
    pub initial_structural_mass: f64,
    pub initial_catalyst_mass: f64,
    pub window_samples: Vec<WindowSample>,
    pub consecutive_viable_windows: u32,
    pub morphology_interval: u64,
}

impl Default for CellDetector {
    fn default() -> Self {
        Self {
            extinction_counter: 0,
            last_classification: ViabilityClass::Seeding,
            turnover: TurnoverTotals::default(),
            rolling_structure: Vec::new(),
            rolling_catalyst: Vec::new(),
            pre_damage_structure_mean: 0.0,
            pre_damage_retention: 0.0,
            initial_structural_mass: 0.0,
            initial_catalyst_mass: 0.0,
            window_samples: Vec::new(),
            consecutive_viable_windows: 0,
            morphology_interval: 100,
        }
    }
}

impl CellDetector {
    pub fn capture_initial_masses(&mut self, grid: &Grid, fields: &FieldBuffers) {
        self.initial_structural_mass = mass_sum(grid, &fields.structure);
        self.initial_catalyst_mass = mass_sum(grid, &fields.catalyst);
    }

    pub fn turnover_ratios(&self) -> TurnoverRatios {
        let s0 = self.initial_structural_mass.max(1e-12);
        let c0 = self.initial_catalyst_mass.max(1e-12);
        TurnoverRatios {
            structural_replacement: self.turnover.structural_decay / s0,
            structural_synthesis: self.turnover.structural_synthesis / s0,
            catalyst_replacement: self.turnover.catalyst_decay / c0,
            catalyst_reproduction: self.turnover.catalyst_reproduction / c0,
        }
    }

    pub fn observe(
        &mut self,
        grid: &Grid,
        fields: &FieldBuffers,
        params: &SimParams,
        substep: u64,
        sim_time: f64,
        dt: f64,
        reaction_scratch: &ReactionScratch,
        accounting: &AccountingState,
        sample_morphology: bool,
    ) -> DiagnosticsSnapshot {
        let structural_mass = mass_sum(grid, &fields.structure);
        let catalyst_mass = mass_sum(grid, &fields.catalyst);

        let mut retained = 0.0;
        for idx in 0..grid.width * grid.height {
            if grid.in_dish(idx) {
                retained += fields.catalyst[idx] * interior_weight(fields.structure[idx]);
            }
        }
        let catalyst_retention = retained / catalyst_mass.max(1e-12);

        let (area, largest, compactness) = if sample_morphology {
            structure_morphology(grid, &fields.structure)
        } else {
            (0, 0, 0.0)
        };

        let mut n_cons = 0.0;
        let mut f_cons = 0.0;
        let mut w_prod = 0.0;
        let mut syn = 0.0;
        let mut sdec = 0.0;
        let mut crep = 0.0;
        let mut cdec = 0.0;

        for idx in 0..grid.width * grid.height {
            if !grid.in_dish(idx) {
                continue;
            }
            let r = &reaction_scratch.rates[idx];
            n_cons += (-r.r_n).max(0.0);
            f_cons += (-r.r_f).max(0.0);
            w_prod += r.r_w.max(0.0);
            syn += r.r_structure;
            sdec += r.r_structure_decay;
            crep += r.r_rep;
            cdec += r.r_catalyst_decay;
        }

        let scale = dt;
        self.turnover.structural_synthesis += syn * scale;
        self.turnover.structural_decay += sdec * scale;
        self.turnover.catalyst_reproduction += crep * scale;
        self.turnover.catalyst_decay += cdec * scale;
        self.turnover.nutrient_consumption += n_cons * scale;
        self.turnover.fuel_consumption += f_cons * scale;
        self.turnover.waste_production += w_prod * scale;

        if self.rolling_structure.len() >= VIABILITY_WINDOW as usize {
            self.rolling_structure.remove(0);
            self.rolling_catalyst.remove(0);
        }
        self.rolling_structure.push(structural_mass);
        self.rolling_catalyst.push(catalyst_mass);

        self.window_samples.push(WindowSample {
            structural_mass,
            catalyst_mass,
            catalyst_retention,
            largest_component: largest,
            protocell_area: area,
            structural_synthesis: syn * scale,
            structural_decay: sdec * scale,
            catalyst_reproduction: crep * scale,
            catalyst_decay: cdec * scale,
            nutrient_consumption: n_cons * scale,
            fuel_consumption: f_cons * scale,
            waste_production: w_prod * scale,
        });
        if self.window_samples.len() > VIABILITY_WINDOW as usize {
            self.window_samples.remove(0);
        }

        let structure_cv = coefficient_of_variation(&self.rolling_structure);
        let catalyst_cv = coefficient_of_variation(&self.rolling_catalyst);

        let window_ok = self.window_qualifies(params, accounting);
        if window_ok && self.window_samples.len() >= VIABILITY_WINDOW as usize {
            self.consecutive_viable_windows += 1;
        } else if self.window_samples.len() >= VIABILITY_WINDOW as usize {
            self.consecutive_viable_windows = 0;
        }

        let classification = self.classify(
            structural_mass,
            catalyst_mass,
            catalyst_retention,
            substep,
            params,
            largest,
            area,
            structure_cv,
            catalyst_cv,
            accounting,
        );

        DiagnosticsSnapshot {
            substep,
            sim_time,
            dt,
            structural_mass,
            catalyst_mass,
            catalyst_retention,
            protocell_area: area,
            largest_component: largest,
            compactness,
            nutrient_consumption_rate: n_cons,
            fuel_consumption_rate: f_cons,
            waste_production_rate: w_prod,
            classification,
            turnover_ratios: self.turnover_ratios(),
            structure_cv,
            catalyst_cv,
            consecutive_viable_windows: self.consecutive_viable_windows,
        }
    }

    fn window_qualifies(&self, params: &SimParams, accounting: &AccountingState) -> bool {
        if self.window_samples.len() < VIABILITY_WINDOW as usize {
            return false;
        }
        let area_threshold = self
            .window_samples
            .iter()
            .map(|s| s.protocell_area)
            .filter(|&a| a > 0)
            .max()
            .unwrap_or(1)
            .max(1);
        let mut syn = 0.0;
        let mut sdec = 0.0;
        let mut crep = 0.0;
        let mut cdec = 0.0;
        let mut n_cons = 0.0;
        let mut f_cons = 0.0;
        let mut w_prod = 0.0;
        let mut min_retention = f64::MAX;
        let mut min_largest_frac = f64::MAX;

        for s in &self.window_samples {
            syn += s.structural_synthesis;
            sdec += s.structural_decay;
            crep += s.catalyst_reproduction;
            cdec += s.catalyst_decay;
            n_cons += s.nutrient_consumption;
            f_cons += s.fuel_consumption;
            w_prod += s.waste_production;
            min_retention = min_retention.min(s.catalyst_retention);
            if s.protocell_area > 0 {
                min_largest_frac = min_largest_frac.min(s.largest_component as f64 / s.protocell_area as f64);
            }
        }

        let structure_cv = coefficient_of_variation(
            &self
                .window_samples
                .iter()
                .map(|s| s.structural_mass)
                .collect::<Vec<_>>(),
        );
        let catalyst_cv = coefficient_of_variation(
            &self
                .window_samples
                .iter()
                .map(|s| s.catalyst_mass)
                .collect::<Vec<_>>(),
        );

        min_largest_frac >= 0.95
            && min_retention >= 0.75
            && structure_cv <= 0.20
            && catalyst_cv <= 0.20
            && syn > 0.0
            && sdec > 0.0
            && crep > 0.0
            && cdec > 0.0
            && n_cons > 0.0
            && f_cons > 0.0
            && w_prod > 0.0
            && accounting.cumulative_within_tolerance()
            && area_threshold > params.structure_extinction_threshold as u64
    }

    fn classify(
        &mut self,
        m_structure: f64,
        m_catalyst: f64,
        retention: f64,
        substep: u64,
        params: &SimParams,
        largest: u64,
        area: u64,
        structure_cv: f64,
        catalyst_cv: f64,
        accounting: &AccountingState,
    ) -> ViabilityClass {
        if m_structure < params.structure_extinction_threshold
            && m_catalyst < params.catalyst_extinction_threshold
        {
            self.extinction_counter += 1;
        } else {
            self.extinction_counter = 0;
        }

        let class = if self.extinction_counter >= params.extinction_hold_time {
            ViabilityClass::Dead
        } else if substep < 5000 {
            ViabilityClass::Seeding
        } else if m_structure < params.structure_extinction_threshold * 2.0
            || m_catalyst < params.catalyst_extinction_threshold * 5.0
        {
            ViabilityClass::Collapsed
        } else if m_structure < params.structure_extinction_threshold * 5.0 {
            ViabilityClass::Degraded
        } else if self.consecutive_viable_windows >= 2
            && retention >= 0.75
            && structure_cv <= 0.25
            && catalyst_cv <= 0.25
            && accounting.cumulative_within_tolerance()
            && (area == 0 || largest as f64 / area as f64 >= 0.90)
        {
            ViabilityClass::Viable
        } else if substep < 50_000 {
            ViabilityClass::Transient
        } else {
            ViabilityClass::Transient
        };

        self.last_classification = class;
        class
    }

    pub fn rolling_mean_structure(&self) -> f64 {
        if self.rolling_structure.is_empty() {
            return 0.0;
        }
        self.rolling_structure.iter().sum::<f64>() / self.rolling_structure.len() as f64
    }
}

fn coefficient_of_variation(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    if mean.abs() < 1e-12 {
        return 0.0;
    }
    let var = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
    var.sqrt() / mean.abs()
}

fn mass_sum(grid: &Grid, field: &[f64]) -> f64 {
    grid.dish_mask
        .iter()
        .zip(field.iter())
        .filter(|(&m, _)| m)
        .map(|(_, &v)| v)
        .sum()
}

fn structure_morphology(grid: &Grid, structure: &[f64]) -> (u64, u64, f64) {
    let w = grid.width;
    let h = grid.height;
    let threshold = 0.5;
    let mut area = 0u64;
    let mut perimeter = 0u64;
    let mut labels = vec![0i32; w * h];
    let mut current_label = 0i32;
    let mut component_sizes: Vec<u64> = Vec::new();

    for j in 0..h {
        for i in 0..w {
            let idx = Grid::index(w, i, j);
            if !grid.in_dish(idx) || structure[idx] < threshold {
                continue;
            }
            area += 1;
            for (ni, nj) in [(i.wrapping_sub(1), j), (i + 1, j), (i, j.wrapping_sub(1)), (i, j + 1)] {
                if ni >= w || nj >= h {
                    perimeter += 1;
                    continue;
                }
                let nidx = Grid::index(w, ni, nj);
                if !grid.in_dish(nidx) || structure[nidx] < threshold {
                    perimeter += 1;
                }
            }
        }
    }

    for j in 0..h {
        for i in 0..w {
            let idx = Grid::index(w, i, j);
            if !grid.in_dish(idx) || structure[idx] < threshold || labels[idx] != 0 {
                continue;
            }
            current_label += 1;
            let mut size = 0u64;
            let mut stack = vec![idx];
            labels[idx] = current_label;
            while let Some(cur) = stack.pop() {
                size += 1;
                let ci = cur % w;
                let cj = cur / w;
                for (ni, nj) in [
                    (ci.wrapping_sub(1), cj),
                    (ci + 1, cj),
                    (ci, cj.wrapping_sub(1)),
                    (ci, cj + 1),
                ] {
                    if ni >= w || nj >= h {
                        continue;
                    }
                    let nidx = Grid::index(w, ni, nj);
                    if grid.in_dish(nidx) && structure[nidx] >= threshold && labels[nidx] == 0 {
                        labels[nidx] = current_label;
                        stack.push(nidx);
                    }
                }
            }
            component_sizes.push(size);
        }
    }

    let largest = component_sizes.iter().copied().max().unwrap_or(0);
    let compactness = if perimeter > 0 {
        4.0 * std::f64::consts::PI * area as f64 / (perimeter as f64).powi(2)
    } else {
        0.0
    };

    (area, largest, compactness)
}
