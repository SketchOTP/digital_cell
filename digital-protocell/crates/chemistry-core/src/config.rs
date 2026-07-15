//! Simulation configuration and parameter loading.

use serde::{Deserialize, Serialize};
use std::fmt;

pub const GRID_WIDTH: usize = 192;
pub const GRID_HEIGHT: usize = 192;
pub const DX: f64 = 1.0;
pub const DISH_RADIUS: f64 = 88.0;
pub const RESERVOIR_WIDTH: f64 = 5.0;
pub const MAX_DT: f64 = 0.0025;
pub const NEG_CLAMP: f64 = -1e-6;
pub const PHI_HARD_MIN: f64 = -1e-4;
pub const PHI_HARD_MAX: f64 = 1.50;
pub const PHI_SOFT_MAX: f64 = 1.25;
pub const CONC_SAFETY_LIMIT: f64 = 10.0;
pub const M_MAX: f64 = 1.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EquationVersion {
    #[serde(rename = "d001-bulk-v1")]
    D001BulkV1,
    #[serde(rename = "d003-crowding-v1")]
    D003CrowdingV1,
    #[serde(rename = "surface_turnover_v1")]
    SurfaceTurnoverV1,
    #[serde(rename = "membrane_metabolism_v1")]
    MembraneMetabolismV1,
}

impl EquationVersion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::D001BulkV1 => "d001-bulk-v1",
            Self::D003CrowdingV1 => "d003-crowding-v1",
            Self::SurfaceTurnoverV1 => "surface_turnover_v1",
            Self::MembraneMetabolismV1 => "membrane_metabolism_v1",
        }
    }
}

impl fmt::Display for EquationVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum D008StageMode {
    #[default]
    Transport,
    ActivatedMetabolism,
    FixedCompartment,
}

impl D008StageMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Transport => "transport",
            Self::ActivatedMetabolism => "activated_metabolism",
            Self::FixedCompartment => "fixed_compartment",
        }
    }
}

impl fmt::Display for D008StageMode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

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
    /// D-003 crowding parameter K_phi
    #[serde(default = "default_k_phi")]
    pub k_phi: f64,
    /// When true, use max(0,1-φ) instead of crowding (D-002 control)
    #[serde(default)]
    pub use_legacy_structure_kinetics: bool,
    /// Immutable equation identifier.
    #[serde(default = "default_equation_version")]
    pub equation_version: EquationVersion,
    /// D-006 interface structural assembly rate
    #[serde(default = "default_k_structure_interface")]
    pub k_structure_interface: f64,
    /// D-006 catalyst half-saturation for structural assembly
    #[serde(default = "default_k_c_structure")]
    pub k_c_structure: f64,
    /// D-008 activated-resource base diffusivity.
    #[serde(default = "default_d_a")]
    pub d_a: f64,
    /// D-008 membrane attenuation coefficients.
    #[serde(default = "default_beta_c")]
    pub beta_c: f64,
    #[serde(default = "default_beta_a")]
    pub beta_a: f64,
    #[serde(default = "default_beta_n")]
    pub beta_n: f64,
    #[serde(default = "default_beta_f")]
    pub beta_f: f64,
    #[serde(default = "default_beta_w")]
    pub beta_w: f64,
    /// D-008 Stage B fixed-field membrane dynamics.
    #[serde(default = "default_m_max")]
    pub m_max: f64,
    #[serde(default = "default_d_m")]
    pub d_m: f64,
    #[serde(default = "default_k_membrane_decay")]
    pub k_membrane_decay: f64,
    #[serde(default = "default_k_membrane_detach")]
    pub k_membrane_detach: f64,
    #[serde(default = "default_k_c_membrane")]
    pub k_c_membrane: f64,
    #[serde(default)]
    pub k_membrane: f64,
    /// Isolated Stage B mode: fixed φ,C,A and solubles; only M advances.
    #[serde(default)]
    pub d008_stage_b_enabled: bool,
    /// Isolated typed D-008 stage dispatch. Stage C is a homogeneous local reactor;
    /// Stage D couples transport, metabolism, and reservoir exchange with fixed φ/M.
    #[serde(default)]
    pub d008_stage_mode: D008StageMode,
    /// D-008 Stage C reference rates. These conservative qualitative-gate values
    /// remain subject to the directive's later Stage E calibration.
    #[serde(default = "default_k_d008_activation")]
    pub k_d008_activation: f64,
    #[serde(default = "default_k_d008_reproduction")]
    pub k_d008_reproduction: f64,
    #[serde(default = "default_k_d008_activated_decay")]
    pub k_d008_activated_decay: f64,
    #[serde(default = "default_k_d008_catalyst_turnover")]
    pub k_d008_catalyst_turnover: f64,
    #[serde(default = "default_d008_a_max")]
    pub d008_a_max: f64,
    #[serde(default = "default_d008_c_max")]
    pub d008_c_max: f64,
}

fn default_k_phi() -> f64 {
    1.0
}

fn default_equation_version() -> EquationVersion {
    EquationVersion::D003CrowdingV1
}

fn default_k_structure_interface() -> f64 {
    0.0
}

fn default_k_c_structure() -> f64 {
    0.10
}

fn default_d_a() -> f64 {
    0.040
}

fn default_beta_c() -> f64 {
    4.6
}

fn default_beta_a() -> f64 {
    4.6
}

fn default_beta_n() -> f64 {
    1.2
}

fn default_beta_f() -> f64 {
    1.2
}

fn default_beta_w() -> f64 {
    0.2
}

fn default_m_max() -> f64 {
    M_MAX
}

fn default_d_m() -> f64 {
    0.001
}

fn default_k_membrane_decay() -> f64 {
    0.002
}

fn default_k_membrane_detach() -> f64 {
    0.020
}

fn default_k_c_membrane() -> f64 {
    0.10
}

fn default_k_d008_activation() -> f64 {
    0.020
}

fn default_k_d008_reproduction() -> f64 {
    0.040
}

fn default_k_d008_activated_decay() -> f64 {
    0.005
}

fn default_k_d008_catalyst_turnover() -> f64 {
    0.002
}

fn default_d008_a_max() -> f64 {
    1.0
}

fn default_d008_c_max() -> f64 {
    1.0
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
            k_phi: 1.0,
            use_legacy_structure_kinetics: false,
            equation_version: EquationVersion::D003CrowdingV1,
            k_structure_interface: 0.0,
            k_c_structure: 0.10,
            d_a: default_d_a(),
            beta_c: default_beta_c(),
            beta_a: default_beta_a(),
            beta_n: default_beta_n(),
            beta_f: default_beta_f(),
            beta_w: default_beta_w(),
            m_max: default_m_max(),
            d_m: default_d_m(),
            k_membrane_decay: default_k_membrane_decay(),
            k_membrane_detach: default_k_membrane_detach(),
            k_c_membrane: default_k_c_membrane(),
            k_membrane: 0.0,
            d008_stage_b_enabled: false,
            d008_stage_mode: D008StageMode::Transport,
            k_d008_activation: default_k_d008_activation(),
            k_d008_reproduction: default_k_d008_reproduction(),
            k_d008_activated_decay: default_k_d008_activated_decay(),
            k_d008_catalyst_turnover: default_k_d008_catalyst_turnover(),
            d008_a_max: default_d008_a_max(),
            d008_c_max: default_d008_c_max(),
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
    #[serde(default = "default_record_every")]
    pub record_every: u64,
}

fn default_record_every() -> u64 {
    1000
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
    DisableStructuralSynthesis,
    ShutdownReservoir,
    DamageFraction { fraction: f64 },
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
            "k_structure_interface" => p.k_structure_interface *= factor,
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

pub fn surface_turnover_params_from_calibrated_kphi1() -> SimParams {
    // Machine-extracted final K_phi=1.0 candidate non-structural parameters.
    let mut p = SimParams::default();
    p.equation_version = EquationVersion::SurfaceTurnoverV1;
    p.k_rep = 0.014489097664708522;
    p.k_structure = 0.0; // unused by surface_turnover_v1
    p.k_structure_decay = 0.025;
    p.k_structure_interface = 0.0; // set after planar derivation
    p.k_c_structure = 0.10;
    p.k_phi = 1.0; // retained unused for provenance; not used by surface kinetics
    p.k_catalyst_decay_inside = 0.005;
    p.k_catalyst_decay_outside = 0.05;
    p.c_max = 1.0;
    p.d_c_inside = 0.004;
    p.d_c_outside = 0.04;
    p.d_n = 0.18;
    p.d_f = 0.18;
    p.d_w = 0.25;
    p.a = 1.0;
    p.kappa = 1.5;
    p.mobility_m = 0.1;
    p.n_reservoir = 1.0;
    p.f_reservoir = 1.0;
    p.w_reservoir = 0.0;
    p.reservoir_rate = 0.5;
    p
}

pub fn legacy_d002_params() -> SimParams {
    let mut p = SimParams::default();
    p.use_legacy_structure_kinetics = true;
    p
}

pub fn d003_params() -> SimParams {
    SimParams::default()
}

pub fn baseline_params() -> SimParams {
    d003_params()
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
