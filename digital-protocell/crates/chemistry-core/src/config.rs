//! Simulation configuration and parameter loading.

use serde::{Deserialize, Serialize};

pub const GRID_WIDTH: usize = 192;
pub const GRID_HEIGHT: usize = 192;
pub const DX: f64 = 1.0;
pub const DISH_RADIUS: f64 = 88.0;
pub const RESERVOIR_WIDTH: f64 = 5.0;
pub const MAX_DT: f64 = 0.0025;
pub const NEG_CLAMP: f64 = -1e-6;
pub const CONC_SAFETY_LIMIT: f64 = 10.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimParams {
    pub a: f64,
    pub kappa: f64,
    pub mobility_m: f64,
    pub d_c_inside: f64,
    pub d_c_outside: f64,
    pub d_n: f64,
    pub d_f: f64,
    pub d_w: f64,
    pub k_rep: f64,
    pub k_structure: f64,
    pub k_structure_decay: f64,
    pub k_catalyst_decay_inside: f64,
    pub k_catalyst_decay_outside: f64,
    pub k_waste_decay: f64,
    pub c_max: f64,
    pub n_reservoir: f64,
    pub f_reservoir: f64,
    pub w_reservoir: f64,
    pub reservoir_rate: f64,
    pub alpha_n_rep: f64,
    pub alpha_n_structure: f64,
    pub alpha_f_rep: f64,
    pub alpha_f_structure: f64,
    pub alpha_w_rep: f64,
    pub alpha_w_structure: f64,
    pub structure_extinction_threshold: f64,
    pub catalyst_extinction_threshold: f64,
    pub extinction_hold_time: u64,
    pub minimum_viable_duration: u64,
    pub seed_r0: f64,
    pub seed_interface_width: f64,
    pub seed_catalyst_scale: f64,
    pub noise_amplitude: f64,
    pub random_seed: u64,
    pub reactions_enabled: bool,
    pub phase_separation_enabled: bool,
    pub diffusion_enabled: bool,
}

impl Default for SimParams {
    fn default() -> Self {
        Self {
            a: 1.0,
            kappa: 1.5,
            mobility_m: 0.10,
            d_c_inside: 0.004,
            d_c_outside: 0.040,
            d_n: 0.18,
            d_f: 0.18,
            d_w: 0.25,
            k_rep: 0.012,
            k_structure: 0.030,
            k_structure_decay: 0.025,
            k_catalyst_decay_inside: 0.005,
            k_catalyst_decay_outside: 0.050,
            k_waste_decay: 0.010,
            c_max: 1.0,
            n_reservoir: 1.0,
            f_reservoir: 1.0,
            w_reservoir: 0.0,
            reservoir_rate: 0.5,
            alpha_n_rep: 1.0,
            alpha_n_structure: 1.0,
            alpha_f_rep: 1.0,
            alpha_f_structure: 1.0,
            alpha_w_rep: 1.0,
            alpha_w_structure: 1.0,
            structure_extinction_threshold: 5.0,
            catalyst_extinction_threshold: 0.05,
            extinction_hold_time: 5000,
            minimum_viable_duration: 250_000,
            seed_r0: 24.0,
            seed_interface_width: 3.0,
            seed_catalyst_scale: 0.35,
            noise_amplitude: 0.005,
            random_seed: 1,
            reactions_enabled: true,
            phase_separation_enabled: true,
            diffusion_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    pub name: String,
    #[serde(default = "default_seed")]
    pub seed: u64,
    #[serde(default = "default_substeps")]
    pub substeps: u64,
    #[serde(default)]
    pub params: SimParams,
    #[serde(default)]
    pub interventions: Vec<InterventionSpec>,
}

fn default_seed() -> u64 {
    1
}

fn default_substeps() -> u64 {
    250_000
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InterventionSpec {
    AtSubstep {
        substep: u64,
        action: InterventionAction,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterventionAction {
    RemoveNutrient,
    RemoveFuel,
    DisableCatalystReproduction,
    RestoreReservoir,
    PunctureRepair,
    CatastrophicDamage,
    DisableAllReactions,
}

impl SimParams {
    pub fn from_toml_str(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    pub fn scaled(&self, key: &str, factor: f64) -> Self {
        let mut p = self.clone();
        match key {
            "k_rep" => p.k_rep *= factor,
            "k_structure" => p.k_structure *= factor,
            "k_structure_decay" => p.k_structure_decay *= factor,
            "k_catalyst_decay_inside" => p.k_catalyst_decay_inside *= factor,
            "d_c_inside" => p.d_c_inside *= factor,
            "mobility_m" => p.mobility_m *= factor,
            "kappa" => p.kappa *= factor,
            _ => {}
        }
        p
    }
}

pub fn baseline_params() -> SimParams {
    SimParams::default()
}

pub fn passive_phase_params() -> SimParams {
    let mut p = SimParams::default();
    p.reactions_enabled = false;
    p
}

pub fn static_control_params() -> SimParams {
    let mut p = SimParams::default();
    p.k_rep = 0.0;
    p.k_structure = 0.0;
    p
}
