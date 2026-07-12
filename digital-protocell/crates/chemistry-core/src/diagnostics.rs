//! Observer diagnostics and viability classification.

use crate::config::SimParams;
use crate::fields::{interior_weight, FieldBuffers};
use crate::grid::Grid;
use crate::reactions::ReactionScratch;
use serde::{Deserialize, Serialize};

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
}

#[derive(Debug, Clone)]
pub struct CellDetector {
    pub extinction_counter: u64,
    pub last_classification: ViabilityClass,
    pub turnover: TurnoverTotals,
    pub rolling_structure: Vec<f64>,
    pub pre_damage_structure_mean: f64,
    pub pre_damage_retention: f64,
}

impl Default for CellDetector {
    fn default() -> Self {
        Self {
            extinction_counter: 0,
            last_classification: ViabilityClass::Seeding,
            turnover: TurnoverTotals::default(),
            rolling_structure: Vec::new(),
            pre_damage_structure_mean: 0.0,
            pre_damage_retention: 0.0,
        }
    }
}

impl CellDetector {
    pub fn observe(
        &mut self,
        grid: &Grid,
        fields: &FieldBuffers,
        params: &SimParams,
        substep: u64,
        sim_time: f64,
        dt: f64,
        reaction_scratch: &ReactionScratch,
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

        let (area, largest, compactness) = structure_morphology(grid, &fields.structure);

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

        let classification = self.classify(structural_mass, catalyst_mass, substep, params);

        if self.rolling_structure.len() >= 1000 {
            self.rolling_structure.remove(0);
        }
        self.rolling_structure.push(structural_mass);

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
        }
    }

    fn classify(
        &mut self,
        m_structure: f64,
        m_catalyst: f64,
        substep: u64,
        params: &SimParams,
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
        } else if substep < 50_000 {
            ViabilityClass::Transient
        } else {
            ViabilityClass::Viable
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
            // perimeter: count exposed edges to non-threshold
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

    // connected components (4-neighbor)
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
