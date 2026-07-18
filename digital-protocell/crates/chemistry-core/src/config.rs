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

/// Governed stoichiometric schema for membrane metabolism v1 (nonconservative productive chemistry).
pub const STOICHIOMETRIC_SCHEMA_VERSION_V1: u32 = 1;
/// Governed stoichiometric schema for membrane_metabolism_v2_conservative.
pub const STOICHIOMETRIC_SCHEMA_VERSION_V2: u32 = 2;
/// Uniform membrane decay (v1–v3).
pub const MEMBRANE_SCHEMA_VERSION_V1: u32 = 1;
/// Interface-protected membrane decay (v4): ε_M + (1 − I(φ)).
pub const MEMBRANE_SCHEMA_VERSION_V2: u32 = 2;
/// Plain membrane diffusion (v1–v4).
pub const MEMBRANE_TRANSPORT_SCHEMA_VERSION_V1: u32 = 1;
/// Interface-affinity membrane transport (v5): J += χ_M · mean(M) · ΔI.
pub const MEMBRANE_TRANSPORT_SCHEMA_VERSION_V2: u32 = 2;
/// Baseline selective-boundary transport schema (D-008/D-015).
pub const TRANSPORT_SCHEMA_VERSION_V1: u32 = 1;
/// D-016 calibrated passive waste transport schema (D_W / β_W repair).
pub const TRANSPORT_SCHEMA_VERSION_V2: u32 = 2;
/// D-023 eight-field soluble-precursor + interface-assembly schema.
pub const PRECURSOR_SCHEMA_VERSION_V1: u32 = 1;
/// D-024 conserved interfacial surface-density schema (S = δΓ).
pub const SURFACE_DENSITY_SCHEMA_VERSION_V1: u32 = 1;
/// D-029 irreversible adsorption (v7) vs reversible exchange (v8).
pub const SURFACE_EXCHANGE_SCHEMA_VERSION_V1: u32 = 1;
pub const SURFACE_EXCHANGE_SCHEMA_VERSION_V2: u32 = 2;
/// D-023 field schema tag: seven current + seven next + P/P_next.
pub const EIGHT_FIELD_COUNT: usize = 8;
/// D-024 membrane transport: surface-occupancy permeability (θΓ).
pub const MEMBRANE_TRANSPORT_SCHEMA_VERSION_V3: u32 = 3;

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
    /// D-012 conservative seven-field network; not comparable to v1 candidate hashes.
    #[serde(rename = "membrane_metabolism_v2_conservative")]
    MembraneMetabolismV2Conservative,
    /// D-019 structural scaling repair; inherits v2 stoichiometry / yields / transport.
    #[serde(rename = "membrane_metabolism_v3_structural_scaling")]
    MembraneMetabolismV3StructuralScaling,
    /// D-021 interface-protected membrane turnover; inherits v3 structure + v2 stoichiometry.
    #[serde(rename = "membrane_metabolism_v4_interface_protected")]
    MembraneMetabolismV4InterfaceProtected,
    /// D-022 interface-affinity M transport; inherits v4 decay + v3 structure.
    #[serde(rename = "membrane_metabolism_v5_interface_affinity")]
    MembraneMetabolismV5InterfaceAffinity,
    /// D-023 eight-field soluble precursor + interface assembly.
    /// Adds P (soluble membrane precursor); disables direct A→M synthesis.
    #[serde(rename = "membrane_metabolism_v6_precursor_assembly")]
    MembraneMetabolismV6PrecursorAssembly,
    /// D-024 conserved interfacial membrane surface density.
    /// Replaces bulk M with S = δΓ; retains soluble precursor P.
    #[serde(rename = "membrane_metabolism_v7_surface_density")]
    MembraneMetabolismV7SurfaceDensity,
    /// D-029 reversible thermodynamic bulk–surface exchange.
    /// Same stored fields as v7; replaces irreversible P→S adsorption with P↔S exchange.
    #[serde(rename = "membrane_metabolism_v8_reversible_surface_exchange")]
    MembraneMetabolismV8ReversibleSurfaceExchange,
}

impl EquationVersion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::D001BulkV1 => "d001-bulk-v1",
            Self::D003CrowdingV1 => "d003-crowding-v1",
            Self::SurfaceTurnoverV1 => "surface_turnover_v1",
            Self::MembraneMetabolismV1 => "membrane_metabolism_v1",
            Self::MembraneMetabolismV2Conservative => "membrane_metabolism_v2_conservative",
            Self::MembraneMetabolismV3StructuralScaling => "membrane_metabolism_v3_structural_scaling",
            Self::MembraneMetabolismV4InterfaceProtected => {
                "membrane_metabolism_v4_interface_protected"
            }
            Self::MembraneMetabolismV5InterfaceAffinity => {
                "membrane_metabolism_v5_interface_affinity"
            }
            Self::MembraneMetabolismV6PrecursorAssembly => {
                "membrane_metabolism_v6_precursor_assembly"
            }
            Self::MembraneMetabolismV7SurfaceDensity => "membrane_metabolism_v7_surface_density",
            Self::MembraneMetabolismV8ReversibleSurfaceExchange => {
                "membrane_metabolism_v8_reversible_surface_exchange"
            }
        }
    }

    pub const fn is_membrane_metabolism(self) -> bool {
        matches!(
            self,
            Self::MembraneMetabolismV1
                | Self::MembraneMetabolismV2Conservative
                | Self::MembraneMetabolismV3StructuralScaling
                | Self::MembraneMetabolismV4InterfaceProtected
                | Self::MembraneMetabolismV5InterfaceAffinity
                | Self::MembraneMetabolismV6PrecursorAssembly
                | Self::MembraneMetabolismV7SurfaceDensity
                | Self::MembraneMetabolismV8ReversibleSurfaceExchange
        )
    }

    pub const fn is_conservative_membrane_metabolism(self) -> bool {
        matches!(
            self,
            Self::MembraneMetabolismV2Conservative
                | Self::MembraneMetabolismV3StructuralScaling
                | Self::MembraneMetabolismV4InterfaceProtected
                | Self::MembraneMetabolismV5InterfaceAffinity
                | Self::MembraneMetabolismV6PrecursorAssembly
                | Self::MembraneMetabolismV7SurfaceDensity
                | Self::MembraneMetabolismV8ReversibleSurfaceExchange
        )
    }

    pub const fn is_interface_protected_membrane(self) -> bool {
        matches!(
            self,
            Self::MembraneMetabolismV4InterfaceProtected
                | Self::MembraneMetabolismV5InterfaceAffinity
                | Self::MembraneMetabolismV6PrecursorAssembly
                // v7/v8 surface turnover uses k_gamma_decay on Γ; retention path retained chemically.
                | Self::MembraneMetabolismV7SurfaceDensity
                | Self::MembraneMetabolismV8ReversibleSurfaceExchange
        )
    }

    pub const fn is_interface_affinity_membrane(self) -> bool {
        matches!(self, Self::MembraneMetabolismV5InterfaceAffinity)
    }

    /// D-023 eight-field soluble-precursor architecture (bulk M assembly).
    pub const fn is_precursor_assembly(self) -> bool {
        matches!(self, Self::MembraneMetabolismV6PrecursorAssembly)
    }

    /// D-024/D-029 interfacial surface-density architecture (S = δΓ).
    pub const fn is_surface_density(self) -> bool {
        matches!(
            self,
            Self::MembraneMetabolismV7SurfaceDensity
                | Self::MembraneMetabolismV8ReversibleSurfaceExchange
        )
    }

    /// D-029 reversible bulk–surface exchange (v8 only).
    pub const fn is_reversible_surface_exchange(self) -> bool {
        matches!(self, Self::MembraneMetabolismV8ReversibleSurfaceExchange)
    }

    /// True when the field schema carries the eight-field (P + membrane/S) payload.
    pub const fn is_eight_field(self) -> bool {
        self.is_precursor_assembly() || self.is_surface_density()
    }

    pub const fn stoichiometric_schema_version(self) -> u32 {
        match self {
            Self::MembraneMetabolismV2Conservative
            | Self::MembraneMetabolismV3StructuralScaling
            | Self::MembraneMetabolismV4InterfaceProtected
            | Self::MembraneMetabolismV5InterfaceAffinity
            | Self::MembraneMetabolismV6PrecursorAssembly
            | Self::MembraneMetabolismV7SurfaceDensity
            | Self::MembraneMetabolismV8ReversibleSurfaceExchange => STOICHIOMETRIC_SCHEMA_VERSION_V2,
            Self::MembraneMetabolismV1 => STOICHIOMETRIC_SCHEMA_VERSION_V1,
            Self::D001BulkV1 | Self::D003CrowdingV1 | Self::SurfaceTurnoverV1 => 0,
        }
    }

    pub const fn membrane_schema_version(self) -> u32 {
        match self {
            Self::MembraneMetabolismV4InterfaceProtected
            | Self::MembraneMetabolismV5InterfaceAffinity
            | Self::MembraneMetabolismV6PrecursorAssembly => MEMBRANE_SCHEMA_VERSION_V2,
            // v7/v8: surface-density schema supersedes bulk membrane schema numbering.
            Self::MembraneMetabolismV7SurfaceDensity
            | Self::MembraneMetabolismV8ReversibleSurfaceExchange => SURFACE_DENSITY_SCHEMA_VERSION_V1,
            Self::MembraneMetabolismV1
            | Self::MembraneMetabolismV2Conservative
            | Self::MembraneMetabolismV3StructuralScaling => MEMBRANE_SCHEMA_VERSION_V1,
            Self::D001BulkV1 | Self::D003CrowdingV1 | Self::SurfaceTurnoverV1 => 0,
        }
    }

    pub const fn membrane_transport_schema_version(self) -> u32 {
        match self {
            Self::MembraneMetabolismV5InterfaceAffinity => MEMBRANE_TRANSPORT_SCHEMA_VERSION_V2,
            Self::MembraneMetabolismV7SurfaceDensity
            | Self::MembraneMetabolismV8ReversibleSurfaceExchange => {
                MEMBRANE_TRANSPORT_SCHEMA_VERSION_V3
            }
            // v6 keeps interface-protected M turnover with χ_M = 0 (diffusion-only M transport).
            Self::MembraneMetabolismV1
            | Self::MembraneMetabolismV2Conservative
            | Self::MembraneMetabolismV3StructuralScaling
            | Self::MembraneMetabolismV4InterfaceProtected
            | Self::MembraneMetabolismV6PrecursorAssembly => MEMBRANE_TRANSPORT_SCHEMA_VERSION_V1,
            Self::D001BulkV1 | Self::D003CrowdingV1 | Self::SurfaceTurnoverV1 => 0,
        }
    }

    /// D-023 precursor-assembly schema version (0 for non-v6/v7/v8 versions).
    pub const fn precursor_schema_version(self) -> u32 {
        match self {
            Self::MembraneMetabolismV6PrecursorAssembly
            | Self::MembraneMetabolismV7SurfaceDensity
            | Self::MembraneMetabolismV8ReversibleSurfaceExchange => PRECURSOR_SCHEMA_VERSION_V1,
            _ => 0,
        }
    }

    /// D-024 surface-density schema version (0 for non-surface-density versions).
    pub const fn surface_density_schema_version(self) -> u32 {
        match self {
            Self::MembraneMetabolismV7SurfaceDensity
            | Self::MembraneMetabolismV8ReversibleSurfaceExchange => SURFACE_DENSITY_SCHEMA_VERSION_V1,
            _ => 0,
        }
    }

    /// D-029 surface-exchange schema: 1 = irreversible adsorption (v7), 2 = reversible (v8).
    pub const fn surface_exchange_schema_version(self) -> u32 {
        match self {
            Self::MembraneMetabolismV7SurfaceDensity => 1,
            Self::MembraneMetabolismV8ReversibleSurfaceExchange => 2,
            _ => 0,
        }
    }
}

impl fmt::Display for EquationVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// D-019 structural scaling mechanism probe / selection identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StructuralScalingMechanism {
    PhaseVolumeSynthesis,
    InterfaceLimitedTurnover,
    LocalCurvatureMaintenance,
}

impl StructuralScalingMechanism {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PhaseVolumeSynthesis => "phase_volume_synthesis",
            Self::InterfaceLimitedTurnover => "interface_limited_turnover",
            Self::LocalCurvatureMaintenance => "local_curvature_maintenance",
        }
    }

    pub const fn selection_tag(self) -> &'static str {
        match self {
            Self::PhaseVolumeSynthesis => "D019_SELECT_PHASE_VOLUME_SYNTHESIS",
            Self::InterfaceLimitedTurnover => "D019_SELECT_INTERFACE_LIMITED_TURNOVER",
            Self::LocalCurvatureMaintenance => "D019_SELECT_LOCAL_CURVATURE_MAINTENANCE",
        }
    }
}

impl fmt::Display for StructuralScalingMechanism {
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
    ConstrainedRadius,
}

impl D008StageMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Transport => "transport",
            Self::ActivatedMetabolism => "activated_metabolism",
            Self::FixedCompartment => "fixed_compartment",
            Self::ConstrainedRadius => "constrained_radius",
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
    /// D-015 W clearance inner boundary; N/F supply still uses `reservoir_mask` only.
    #[serde(default = "default_waste_sink_inner_radius")]
    pub waste_sink_inner_radius: f64,
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
    /// D-008 Stage E interface structure production from activated resource.
    #[serde(default = "default_k_d008_structure")]
    pub k_d008_structure: f64,
    #[serde(default = "default_d008_a_max")]
    pub d008_a_max: f64,
    #[serde(default = "default_d008_c_max")]
    pub d008_c_max: f64,
    /// D-012 v2 catalyst yield η_C ∈ (0, 1].
    #[serde(default = "default_eta_c")]
    pub eta_c: f64,
    /// D-012 v2 structure yield η_φ ∈ (0, 1].
    #[serde(default = "default_eta_phi")]
    pub eta_phi: f64,
    /// D-012 v2 membrane yield η_M ∈ (0, 1].
    #[serde(default = "default_eta_m")]
    pub eta_m: f64,
    /// D-016 transport schema version (1 = baseline; 2 = calibrated W transport).
    #[serde(default = "default_transport_schema_version")]
    pub transport_schema_version: u32,
    /// D-019 analysis-only mechanism override. `None` uses equation-version defaults.
    /// Must remain `None` for governed v2 identity hashes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub d019_mechanism_probe: Option<StructuralScalingMechanism>,
    /// D-021 membrane interface-protection floor ε_M ∈ (0, 1]. Used under v4/v5.
    #[serde(default = "default_eps_m")]
    pub eps_m: f64,
    /// D-022 membrane interface-affinity coefficient χ_M. Used under v5; 0 ⇒ v4 transport.
    #[serde(default = "default_chi_m")]
    pub chi_m: f64,
    /// D-023 precursor synthesis rate coefficient (A → P). Used under v6.
    #[serde(default = "default_k_precursor")]
    pub k_precursor: f64,
    /// D-023 interface assembly rate coefficient (P → M). Only new screened parameter.
    #[serde(default = "default_k_assembly")]
    pub k_assembly: f64,
    /// D-023 precursor turnover rate coefficient (P → W). Frozen to k_A_decay.
    #[serde(default = "default_k_precursor_decay")]
    pub k_precursor_decay: f64,
    /// D-023 precursor diffusivity. Frozen to D_A for the initial bounded experiment.
    #[serde(default = "default_d_p")]
    pub d_p: f64,
    /// D-024 adsorption rate coefficient (P → Γ): J_ads = k_ads P q(C) (1−Γ/Γ_max).
    /// Retained for v7; unused by v8 reversible exchange.
    #[serde(default = "default_k_ads")]
    pub k_ads: f64,
    /// D-029 exchange mobility coefficient: m_exchange = k_exchange × q(C) × Γ_max.
    #[serde(default = "default_k_exchange")]
    pub k_exchange: f64,
    /// D-029 exchange equilibrium constant K_exchange in a_forward = K p (1−θ).
    #[serde(default = "default_k_exchange_eq", rename = "K_exchange")]
    pub k_exchange_eq: f64,
    /// D-029 precursor activity reference (p = P / P_reference). Fixed at 1 unless governed otherwise.
    #[serde(default = "default_p_reference")]
    pub p_reference: f64,
    /// D-024 surface membrane turnover (Γ → W): J_loss = k_Γ_decay Γ.
    #[serde(default = "default_k_gamma_decay")]
    pub k_gamma_decay: f64,
    /// D-024 tangential surface diffusivity D_Γ.
    #[serde(default = "default_d_gamma")]
    pub d_gamma: f64,
    /// D-024 saturation ceiling Γ_max for adsorption.
    #[serde(default = "default_gamma_max")]
    pub gamma_max: f64,
    /// D-024 permeability occupancy reference Γ_reference (θΓ = Γ/Γ_ref).
    #[serde(default = "default_gamma_reference")]
    pub gamma_reference: f64,
    /// D-024 normal regularization η_n.
    #[serde(default = "default_eta_n")]
    pub eta_n: f64,
    /// D-024 Γ reconstruction floor δ_floor.
    #[serde(default = "default_delta_floor")]
    pub delta_floor: f64,
    /// D-024 face interface support threshold for surface fluxes.
    #[serde(default = "default_delta_face_eps")]
    pub delta_face_eps: f64,
    /// D-025 velocity estimator regularization η_v in v_n = −∂tφ / sqrt(|∇φ|² + η_v²).
    #[serde(default = "default_eta_v")]
    pub eta_v: f64,
    /// D-025 minimum |∇φ| for valid interface-band velocity (weak-gradient exclusion).
    #[serde(default = "default_interface_grad_min")]
    pub interface_grad_min: f64,
}

/// Validate governed v2 yields: 0 < η ≤ 1.
pub fn validate_v2_yields(eta_c: f64, eta_phi: f64, eta_m: f64) -> Result<(), String> {
    for (name, eta) in [("eta_c", eta_c), ("eta_phi", eta_phi), ("eta_m", eta_m)] {
        if !eta.is_finite() || eta <= 0.0 || eta > 1.0 {
            return Err(format!("{name} must satisfy 0 < η ≤ 1; got {eta}"));
        }
    }
    Ok(())
}

fn default_waste_sink_inner_radius() -> f64 {
    DISH_RADIUS - RESERVOIR_WIDTH
}

fn default_eta_c() -> f64 {
    1.0
}

fn default_eta_phi() -> f64 {
    1.0
}

fn default_eta_m() -> f64 {
    1.0
}

fn default_transport_schema_version() -> u32 {
    TRANSPORT_SCHEMA_VERSION_V1
}

fn default_eps_m() -> f64 {
    0.05
}

fn default_chi_m() -> f64 {
    0.0
}

fn default_k_precursor() -> f64 {
    // Mirrors the historical membrane synthesis coefficient scale.
    0.2
}

fn default_k_assembly() -> f64 {
    0.0
}

fn default_k_precursor_decay() -> f64 {
    // Frozen: k_precursor_decay = k_A_decay (D-008 activated decay).
    default_k_d008_activated_decay()
}

fn default_d_p() -> f64 {
    // Frozen: D_P = D_A.
    default_d_a()
}

fn default_k_ads() -> f64 {
    0.0
}

fn default_k_exchange() -> f64 {
    0.0
}

fn default_k_exchange_eq() -> f64 {
    1.0
}

fn default_p_reference() -> f64 {
    1.0
}

fn default_k_gamma_decay() -> f64 {
    // Mirror historical membrane decay scale.
    default_k_membrane_decay()
}

fn default_d_gamma() -> f64 {
    0.02
}

fn default_gamma_max() -> f64 {
    1.0
}

fn default_gamma_reference() -> f64 {
    1.0
}

fn default_eta_n() -> f64 {
    1e-6
}

fn default_delta_floor() -> f64 {
    1e-12
}

fn default_delta_face_eps() -> f64 {
    1e-14
}

fn default_eta_v() -> f64 {
    1e-6
}

fn default_interface_grad_min() -> f64 {
    1e-3
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

fn default_k_d008_structure() -> f64 {
    0.030
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
            waste_sink_inner_radius: default_waste_sink_inner_radius(),
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
            k_d008_structure: default_k_d008_structure(),
            d008_a_max: default_d008_a_max(),
            d008_c_max: default_d008_c_max(),
            eta_c: default_eta_c(),
            eta_phi: default_eta_phi(),
            eta_m: default_eta_m(),
            transport_schema_version: default_transport_schema_version(),
            d019_mechanism_probe: None,
            eps_m: default_eps_m(),
            chi_m: default_chi_m(),
            k_precursor: default_k_precursor(),
            k_assembly: default_k_assembly(),
            k_precursor_decay: default_k_precursor_decay(),
            d_p: default_d_p(),
            k_ads: default_k_ads(),
            k_exchange: default_k_exchange(),
            k_exchange_eq: default_k_exchange_eq(),
            p_reference: default_p_reference(),
            k_gamma_decay: default_k_gamma_decay(),
            d_gamma: default_d_gamma(),
            gamma_max: default_gamma_max(),
            gamma_reference: default_gamma_reference(),
            eta_n: default_eta_n(),
            delta_floor: default_delta_floor(),
            delta_face_eps: default_delta_face_eps(),
            eta_v: default_eta_v(),
            interface_grad_min: default_interface_grad_min(),
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

    pub fn validate_equation_config(&self) -> Result<(), String> {
        if self.equation_version.is_conservative_membrane_metabolism() {
            validate_v2_yields(self.eta_c, self.eta_phi, self.eta_m)?;
        }
        Ok(())
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
