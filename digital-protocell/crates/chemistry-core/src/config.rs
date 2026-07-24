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
/// D-041 structural-interface A retention: Π_A = ρ_A exp(−β_A θ_S) on φ-crossing faces.
pub const TRANSPORT_SCHEMA_VERSION_V3: u32 = 3;
/// Governed name for transport schema 3 (D-041).
pub const MEMBRANE_TRANSPORT_SCHEMA_3_STRUCTURAL_A_RETENTION: &str =
    "membrane_transport_schema_3_structural_a_retention";
/// D-023 eight-field soluble-precursor + interface-assembly schema.
pub const PRECURSOR_SCHEMA_VERSION_V1: u32 = 1;
/// D-024 conserved interfacial surface-density schema (S = δΓ).
pub const SURFACE_DENSITY_SCHEMA_VERSION_V1: u32 = 1;
/// D-029 irreversible adsorption (v7) vs reversible exchange (v8).
pub const SURFACE_EXCHANGE_SCHEMA_VERSION_V1: u32 = 1;
pub const SURFACE_EXCHANGE_SCHEMA_VERSION_V2: u32 = 2;
/// D-032 activated surface assembly on top of reversible exchange (v9).
pub const SURFACE_EXCHANGE_SCHEMA_VERSION_V3: u32 = 3;
/// D-032 active-assembly reaction schema (P+A→S+W).
pub const ACTIVE_ASSEMBLY_SCHEMA_VERSION_V1: u32 = 1;
/// D-033 activated-intermediate schema (P+A→X+W, X→S, X→P).
pub const ACTIVATED_INTERMEDIATE_SCHEMA_VERSION_V1: u32 = 1;
/// D-033 surface-exchange schema tag for v10.
pub const SURFACE_EXCHANGE_SCHEMA_VERSION_V4: u32 = 4;
/// D-034 surface-exchange schema tag for v11 dual-surface maturation.
pub const SURFACE_EXCHANGE_SCHEMA_VERSION_V5: u32 = 5;
/// D-034 immature/mature surface maturation schema (v11).
pub const SURFACE_MATURATION_SCHEMA_VERSION_V1: u32 = 1;
/// D-034 dual-surface (U + S) schema (v11).
pub const DUAL_SURFACE_SCHEMA_VERSION_V1: u32 = 1;
/// D-023 field schema tag: seven current + seven next + P/P_next.
pub const EIGHT_FIELD_COUNT: usize = 8;
/// D-033 nine-field count: eight + activated intermediate X.
pub const NINE_FIELD_COUNT: usize = 9;
/// D-024 membrane transport: surface-occupancy permeability (θΓ).
pub const MEMBRANE_TRANSPORT_SCHEMA_VERSION_V3: u32 = 3;

/// Explicit structure-evolution execution mode (D-061).
///
/// FixedGeometry: assays may freeze φ; structural ledgers are counterfactual.
/// DynamicStructure: accepted structural chemistry mutates φ (organism biology).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StructureEvolutionMode {
    /// Accepted structural production/decay do not mutate φ.
    #[default]
    FixedGeometry,
    /// Accepted structural production/decay mutate φ.
    DynamicStructure,
}

impl StructureEvolutionMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FixedGeometry => "FixedGeometry",
            Self::DynamicStructure => "DynamicStructure",
        }
    }

    /// Whether accepted structural extents update the φ field.
    pub const fn apply_phi(self) -> bool {
        matches!(self, Self::DynamicStructure)
    }

    /// Legacy boolean: true iff FixedGeometry.
    pub const fn enforce_constraint(self) -> bool {
        matches!(self, Self::FixedGeometry)
    }

    pub const fn from_enforce_constraint(enforce: bool) -> Self {
        if enforce {
            Self::FixedGeometry
        } else {
            Self::DynamicStructure
        }
    }

    pub const fn is_counterfactual_structure_ledger(self) -> bool {
        matches!(self, Self::FixedGeometry)
    }
}

impl fmt::Display for StructureEvolutionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

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
    /// D-032 metabolically activated surface assembly on the validated v8 interfacial architecture.
    /// Same stored fields φ,C,N,F,W,A,P,S; adds powered P+A→S+W while retaining passive P↔S exchange.
    #[serde(rename = "membrane_metabolism_v9_activated_surface_assembly")]
    MembraneMetabolismV9ActivatedSurfaceAssembly,
    /// D-033 two-stage activated membrane intermediate on the validated v8 interfacial architecture.
    /// Stored fields φ,C,N,F,W,A,P,X,S; replaces direct P+A→S+W with P+A→X+W, X→S, X→P.
    #[serde(rename = "membrane_metabolism_v10_activated_intermediate")]
    MembraneMetabolismV10ActivatedIntermediate,
    /// D-034 immature/mature dual-surface maturation on the validated v8 interfacial architecture.
    /// Stored fields φ,C,N,F,W,A,P,U,S (U = δΓ_U immature surface density); maturation U→S stub path.
    #[serde(rename = "membrane_metabolism_v11_surface_maturation")]
    MembraneMetabolismV11SurfaceMaturation,
    /// D-035 mature-membrane-catalyzed assembly on the v11 dual-surface architecture.
    /// Same stored fields φ,C,N,F,W,A,P,U,S; replaces linear U→S maturation with saturating
    /// mature-catalyzed U+A→S+W: J = q f_A f_U (k0 Γ_max + k_cat Γ_S).
    #[serde(rename = "membrane_metabolism_v12_membrane_catalytic_assembly")]
    MembraneMetabolismV12MembraneCatalyticAssembly,
    /// D-050 catalyst-saturating volume activation on the validated v8 interfacial architecture.
    /// Same stored fields φ,C,N,F,W,A,P,S as v8; activation schema 2 handled in activated_metabolism.
    #[serde(rename = "membrane_metabolism_v13_catalyst_saturating_activation")]
    MembraneMetabolismV13CatalystSaturatingActivation,
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
            Self::MembraneMetabolismV9ActivatedSurfaceAssembly => {
                "membrane_metabolism_v9_activated_surface_assembly"
            }
            Self::MembraneMetabolismV10ActivatedIntermediate => {
                "membrane_metabolism_v10_activated_intermediate"
            }
            Self::MembraneMetabolismV11SurfaceMaturation => {
                "membrane_metabolism_v11_surface_maturation"
            }
            Self::MembraneMetabolismV12MembraneCatalyticAssembly => {
                "membrane_metabolism_v12_membrane_catalytic_assembly"
            }
            Self::MembraneMetabolismV13CatalystSaturatingActivation => {
                "membrane_metabolism_v13_catalyst_saturating_activation"
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
                | Self::MembraneMetabolismV9ActivatedSurfaceAssembly
                | Self::MembraneMetabolismV10ActivatedIntermediate
                | Self::MembraneMetabolismV11SurfaceMaturation | Self::MembraneMetabolismV12MembraneCatalyticAssembly
                | Self::MembraneMetabolismV13CatalystSaturatingActivation
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
                | Self::MembraneMetabolismV9ActivatedSurfaceAssembly
                | Self::MembraneMetabolismV10ActivatedIntermediate
                | Self::MembraneMetabolismV11SurfaceMaturation | Self::MembraneMetabolismV12MembraneCatalyticAssembly
                | Self::MembraneMetabolismV13CatalystSaturatingActivation
        )
    }

    pub const fn is_interface_protected_membrane(self) -> bool {
        matches!(
            self,
            Self::MembraneMetabolismV4InterfaceProtected
                | Self::MembraneMetabolismV5InterfaceAffinity
                | Self::MembraneMetabolismV6PrecursorAssembly
                // v7/v8/v9/v10 surface turnover uses k_gamma_decay on Γ; retention path retained chemically.
                | Self::MembraneMetabolismV7SurfaceDensity
                | Self::MembraneMetabolismV8ReversibleSurfaceExchange
                | Self::MembraneMetabolismV9ActivatedSurfaceAssembly
                | Self::MembraneMetabolismV10ActivatedIntermediate
                | Self::MembraneMetabolismV11SurfaceMaturation | Self::MembraneMetabolismV12MembraneCatalyticAssembly
                | Self::MembraneMetabolismV13CatalystSaturatingActivation
        )
    }

    pub const fn is_interface_affinity_membrane(self) -> bool {
        matches!(self, Self::MembraneMetabolismV5InterfaceAffinity)
    }

    /// D-023 eight-field soluble-precursor architecture (bulk M assembly).
    pub const fn is_precursor_assembly(self) -> bool {
        matches!(self, Self::MembraneMetabolismV6PrecursorAssembly)
    }

    /// D-024/D-029/D-032/D-033 interfacial surface-density architecture (S = δΓ).
    pub const fn is_surface_density(self) -> bool {
        matches!(
            self,
            Self::MembraneMetabolismV7SurfaceDensity
                | Self::MembraneMetabolismV8ReversibleSurfaceExchange
                | Self::MembraneMetabolismV9ActivatedSurfaceAssembly
                | Self::MembraneMetabolismV10ActivatedIntermediate
                | Self::MembraneMetabolismV11SurfaceMaturation | Self::MembraneMetabolismV12MembraneCatalyticAssembly
                | Self::MembraneMetabolismV13CatalystSaturatingActivation
        )
    }

    /// D-029/D-032/D-033 reversible bulk–surface exchange (v8+ retain passive P↔S).
    pub const fn is_reversible_surface_exchange(self) -> bool {
        matches!(
            self,
            Self::MembraneMetabolismV8ReversibleSurfaceExchange
                | Self::MembraneMetabolismV9ActivatedSurfaceAssembly
                | Self::MembraneMetabolismV10ActivatedIntermediate
                | Self::MembraneMetabolismV11SurfaceMaturation | Self::MembraneMetabolismV12MembraneCatalyticAssembly
                | Self::MembraneMetabolismV13CatalystSaturatingActivation
        )
    }

    /// D-032 metabolically activated surface assembly (v9 only; not v10).
    pub const fn is_activated_surface_assembly(self) -> bool {
        matches!(self, Self::MembraneMetabolismV9ActivatedSurfaceAssembly)
    }

    /// True when the field schema carries the eight-field (P + membrane/S) payload (v6–v9).
    pub const fn is_eight_field(self) -> bool {
        self.is_precursor_assembly()
            || matches!(
                self,
                Self::MembraneMetabolismV7SurfaceDensity
                    | Self::MembraneMetabolismV8ReversibleSurfaceExchange
                    | Self::MembraneMetabolismV9ActivatedSurfaceAssembly
                    | Self::MembraneMetabolismV13CatalystSaturatingActivation
            )
    }

    /// D-050 catalyst-saturating volume activation (v13 only).
    pub const fn is_catalyst_saturating_activation(self) -> bool {
        matches!(self, Self::MembraneMetabolismV13CatalystSaturatingActivation)
    }

    /// D-033 two-stage activated membrane intermediate (v10 only).
    pub const fn is_activated_intermediate(self) -> bool {
        matches!(self, Self::MembraneMetabolismV10ActivatedIntermediate)
    }

    /// D-034/D-035 immature/mature dual-surface maturation (v11 linear or v12 catalytic).
    pub const fn is_surface_maturation(self) -> bool {
        matches!(
            self,
            Self::MembraneMetabolismV11SurfaceMaturation
                | Self::MembraneMetabolismV12MembraneCatalyticAssembly
        )
    }

    /// D-035 mature-membrane-catalyzed assembly (v12 only).
    pub const fn is_membrane_catalytic_assembly(self) -> bool {
        matches!(self, Self::MembraneMetabolismV12MembraneCatalyticAssembly)
    }

    /// True when the field schema carries the nine-field (P + X/U + S) payload (v10/v11).
    pub const fn is_nine_field(self) -> bool {
        self.is_activated_intermediate() || self.is_surface_maturation()
    }

    pub const fn stoichiometric_schema_version(self) -> u32 {
        match self {
            Self::MembraneMetabolismV2Conservative
            | Self::MembraneMetabolismV3StructuralScaling
            | Self::MembraneMetabolismV4InterfaceProtected
            | Self::MembraneMetabolismV5InterfaceAffinity
            | Self::MembraneMetabolismV6PrecursorAssembly
            | Self::MembraneMetabolismV7SurfaceDensity
            | Self::MembraneMetabolismV8ReversibleSurfaceExchange
            | Self::MembraneMetabolismV9ActivatedSurfaceAssembly
            | Self::MembraneMetabolismV10ActivatedIntermediate
            | Self::MembraneMetabolismV11SurfaceMaturation | Self::MembraneMetabolismV12MembraneCatalyticAssembly
            | Self::MembraneMetabolismV13CatalystSaturatingActivation => STOICHIOMETRIC_SCHEMA_VERSION_V2,
            Self::MembraneMetabolismV1 => STOICHIOMETRIC_SCHEMA_VERSION_V1,
            Self::D001BulkV1 | Self::D003CrowdingV1 | Self::SurfaceTurnoverV1 => 0,
        }
    }

    pub const fn membrane_schema_version(self) -> u32 {
        match self {
            Self::MembraneMetabolismV4InterfaceProtected
            | Self::MembraneMetabolismV5InterfaceAffinity
            | Self::MembraneMetabolismV6PrecursorAssembly => MEMBRANE_SCHEMA_VERSION_V2,
            // v7/v8/v9/v10: surface-density schema supersedes bulk membrane schema numbering.
            Self::MembraneMetabolismV7SurfaceDensity
            | Self::MembraneMetabolismV8ReversibleSurfaceExchange
            | Self::MembraneMetabolismV9ActivatedSurfaceAssembly
            | Self::MembraneMetabolismV10ActivatedIntermediate
            | Self::MembraneMetabolismV11SurfaceMaturation | Self::MembraneMetabolismV12MembraneCatalyticAssembly
            | Self::MembraneMetabolismV13CatalystSaturatingActivation => SURFACE_DENSITY_SCHEMA_VERSION_V1,
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
            | Self::MembraneMetabolismV8ReversibleSurfaceExchange
            | Self::MembraneMetabolismV9ActivatedSurfaceAssembly
            | Self::MembraneMetabolismV10ActivatedIntermediate
            | Self::MembraneMetabolismV11SurfaceMaturation | Self::MembraneMetabolismV12MembraneCatalyticAssembly
            | Self::MembraneMetabolismV13CatalystSaturatingActivation => {
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

    /// D-023 precursor-assembly schema version (0 for non-v6+ versions).
    pub const fn precursor_schema_version(self) -> u32 {
        match self {
            Self::MembraneMetabolismV6PrecursorAssembly
            | Self::MembraneMetabolismV7SurfaceDensity
            | Self::MembraneMetabolismV8ReversibleSurfaceExchange
            | Self::MembraneMetabolismV9ActivatedSurfaceAssembly
            | Self::MembraneMetabolismV10ActivatedIntermediate
            | Self::MembraneMetabolismV11SurfaceMaturation | Self::MembraneMetabolismV12MembraneCatalyticAssembly
            | Self::MembraneMetabolismV13CatalystSaturatingActivation => PRECURSOR_SCHEMA_VERSION_V1,
            _ => 0,
        }
    }

    /// D-024 surface-density schema version (0 for non-surface-density versions).
    pub const fn surface_density_schema_version(self) -> u32 {
        match self {
            Self::MembraneMetabolismV7SurfaceDensity
            | Self::MembraneMetabolismV8ReversibleSurfaceExchange
            | Self::MembraneMetabolismV9ActivatedSurfaceAssembly
            | Self::MembraneMetabolismV10ActivatedIntermediate
            | Self::MembraneMetabolismV11SurfaceMaturation | Self::MembraneMetabolismV12MembraneCatalyticAssembly
            | Self::MembraneMetabolismV13CatalystSaturatingActivation => SURFACE_DENSITY_SCHEMA_VERSION_V1,
            _ => 0,
        }
    }

    /// Surface-exchange schema: 1=v7, 2=v8, 3=v9, 4=v10 activated intermediate.
    pub const fn surface_exchange_schema_version(self) -> u32 {
        match self {
            Self::MembraneMetabolismV7SurfaceDensity => 1,
            Self::MembraneMetabolismV8ReversibleSurfaceExchange
            | Self::MembraneMetabolismV13CatalystSaturatingActivation => 2,
            Self::MembraneMetabolismV9ActivatedSurfaceAssembly => 3,
            Self::MembraneMetabolismV10ActivatedIntermediate => 4,
            Self::MembraneMetabolismV11SurfaceMaturation | Self::MembraneMetabolismV12MembraneCatalyticAssembly => SURFACE_EXCHANGE_SCHEMA_VERSION_V5,
            _ => 0,
        }
    }

    /// D-034 surface-maturation schema version (0 unless v11).
    pub const fn surface_maturation_schema_version(self) -> u32 {
        match self {
            Self::MembraneMetabolismV11SurfaceMaturation | Self::MembraneMetabolismV12MembraneCatalyticAssembly => SURFACE_MATURATION_SCHEMA_VERSION_V1,
            _ => 0,
        }
    }

    /// D-034 dual-surface schema version (0 unless v11).
    pub const fn dual_surface_schema_version(self) -> u32 {
        match self {
            Self::MembraneMetabolismV11SurfaceMaturation | Self::MembraneMetabolismV12MembraneCatalyticAssembly => DUAL_SURFACE_SCHEMA_VERSION_V1,
            _ => 0,
        }
    }

    /// D-032 active-assembly schema version (0 unless v9).
    pub const fn active_assembly_schema_version(self) -> u32 {
        match self {
            Self::MembraneMetabolismV9ActivatedSurfaceAssembly => ACTIVE_ASSEMBLY_SCHEMA_VERSION_V1,
            _ => 0,
        }
    }

    /// D-033 activated-intermediate schema version (0 unless v10).
    pub const fn activated_intermediate_schema_version(self) -> u32 {
        match self {
            Self::MembraneMetabolismV10ActivatedIntermediate => {
                ACTIVATED_INTERMEDIATE_SCHEMA_VERSION_V1
            }
            _ => 0,
        }
    }
}

/// Numerical integrator for local reversible P↔S exchange (equation law unchanged).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceExchangeIntegrator {
    /// D-029/D-030 explicit Euler (can overshoot capacity).
    ExplicitEulerV1,
    /// D-031 invariant-domain backward Euler + Strang turnover.
    #[default]
    InvariantDomainV2,
}

impl SurfaceExchangeIntegrator {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExplicitEulerV1 => "surface_exchange_integrator_v1_explicit_euler",
            Self::InvariantDomainV2 => "surface_exchange_integrator_v2_invariant_domain",
        }
    }
}

/// Surface membrane turnover representation (D-024 onward).
///
/// Schema 1 (historical default): `J = k_Γ S` with `k_Γ = k_membrane_decay`.
/// Schema 2 (D-038): `J = k_M S [ε_M + (1 − I(φ))]` — exact D-021 protection on embedded S.
/// Schema 3 (D-039 experimental): no constitutive mature-membrane `S→W`; loss only via declared damage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceTurnoverSchema {
    /// Historical D-024..D-037 uniform surface loss (omits interface protection).
    #[default]
    #[serde(rename = "surface_turnover_schema_1_historical_uniform")]
    HistoricalUniform,
    /// D-038 corrected transfer: D-021 protection law on embedded surface density.
    #[serde(rename = "surface_turnover_schema_2_d021_equivalent")]
    D021Equivalent,
    /// D-039 exchange+damage-only: normal biological `S→W` turnover is zero.
    #[serde(rename = "surface_turnover_schema_3_exchange_damage_only")]
    ExchangeDamageOnly,
}

impl SurfaceTurnoverSchema {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::HistoricalUniform => "surface_turnover_schema_1_historical_uniform",
            Self::D021Equivalent => "surface_turnover_schema_2_d021_equivalent",
            Self::ExchangeDamageOnly => "surface_turnover_schema_3_exchange_damage_only",
        }
    }

    pub const fn is_d021_equivalent(self) -> bool {
        matches!(self, Self::D021Equivalent)
    }

    /// True when constitutive mature-membrane first-order `S→W` is enabled.
    pub const fn allows_constitutive_turnover(self) -> bool {
        matches!(self, Self::HistoricalUniform | Self::D021Equivalent)
    }

    pub const fn is_exchange_damage_only(self) -> bool {
        matches!(self, Self::ExchangeDamageOnly)
    }
}

impl fmt::Display for SurfaceTurnoverSchema {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
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
    /// D-016/D-041 transport schema version (1 = baseline; 2 = calibrated W; 3 = structural A retention).
    #[serde(default = "default_transport_schema_version")]
    pub transport_schema_version: u32,
    /// D-041 primitive structural-interface A permeability ρ_A.
    /// Historical behavior is ρ_A = 1. Applied only under transport schema 3 on φ-crossing faces.
    #[serde(default = "default_rho_a")]
    pub rho_a: f64,
    /// D-019 analysis-only mechanism override. `None` uses equation-version defaults.
    /// Must remain `None` for governed v2 identity hashes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub d019_mechanism_probe: Option<StructuralScalingMechanism>,
    /// D-084: when true, structural loss uses `k φ [η+(1−η)I]` instead of legacy `ε+I` floor.
    #[serde(default)]
    pub use_mixed_structure_turnover: bool,
    /// D-084 bulk mix η ∈ [0,1]. Meaningful only when `use_mixed_structure_turnover`.
    #[serde(default)]
    pub structure_turnover_eta: f64,
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
    /// D-071 opt-in constitutive precursor scale `m_P` (default 1 = frozen law).
    #[serde(default = "default_precursor_m_p")]
    pub precursor_m_p: f64,
    /// D-071 opt-in product inhibition `K_I`. `0` disables inhibition (frozen law).
    #[serde(default = "default_precursor_product_inhibition_ki")]
    pub precursor_product_inhibition_ki: f64,
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
    /// D-032 activated-resource activity reference (a = A / A_reference).
    #[serde(default = "default_a_reference")]
    pub a_reference: f64,
    /// D-032 active surface-assembly rate: J_active = k_active q(C) a p max(0,1−θ).
    #[serde(default = "default_k_active")]
    pub k_active: f64,
    /// D-033 precursor charging rate: r_charge = k_charge H(φ) q(C) P A.
    #[serde(default = "default_k_charge")]
    pub k_charge: f64,
    /// D-033 surface insertion rate: r_insert = k_insert δ X max(0,1−θ).
    #[serde(default = "default_k_insert")]
    pub k_insert: f64,
    /// D-033 intermediate relaxation rate: r_relax = k_relax X.
    #[serde(default = "default_k_relax")]
    pub k_relax: f64,
    /// D-033 activated-intermediate diffusivity (frozen initially to D_P).
    #[serde(default = "default_d_x")]
    pub d_x: f64,
    /// D-034 immature-surface maturation rate coefficient (U→S).
    #[serde(default = "default_k_mature")]
    pub k_mature: f64,
    /// D-035 basal nucleation coefficient k_0 in catalytic maturation (v12).
    #[serde(default = "default_k_mature_basal")]
    pub k_mature_basal: f64,
    /// D-035 mature-membrane catalytic coefficient k_cat (v12).
    #[serde(default = "default_k_mature_cat")]
    pub k_mature_cat: f64,
    /// D-035 half-saturation for activated resource f_A = a/(K_A+a).
    #[serde(default = "default_k_a_half", rename = "K_A")]
    pub k_a_half: f64,
    /// D-035 half-saturation for immature surface f_U = Γ_U/(K_U+Γ_U).
    #[serde(default = "default_k_u_half", rename = "K_U")]
    pub k_u_half: f64,
    /// D-034 immature-surface tangential diffusivity D_U (defaults to D_Γ).
    #[serde(default = "default_d_u")]
    pub d_u: f64,
    /// D-024 surface membrane turnover (Γ → W): J_loss = k_Γ_decay Γ under schema 1.
    /// Under schema 2: J_loss = k_Γ_decay · S · [ε_M + (1 − I(φ))] with S = δΓ.
    #[serde(default = "default_k_gamma_decay")]
    pub k_gamma_decay: f64,
    /// D-038 surface-turnover representation. Default schema 1 preserves historical equations.
    #[serde(default)]
    pub surface_turnover_schema: SurfaceTurnoverSchema,
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
    /// D-031 local reversible-exchange numerical integrator (law unchanged).
    #[serde(default)]
    pub surface_exchange_integrator: SurfaceExchangeIntegrator,
    /// D-050 activation rate schema (1 = historical mass-action; 2 = catalyst_saturating_volume).
    #[serde(default = "default_activation_schema")]
    pub activation_schema: u32,
    /// D-050 catalyst half-saturation K_C for activation schema 2.
    #[serde(default = "default_k_c_activation")]
    pub k_c_activation: f64,
    /// D-050 nutrient reference for activation schema 2.
    #[serde(default = "default_n_ref_activation")]
    pub n_ref_activation: f64,
    /// D-050 fuel reference for activation schema 2.
    #[serde(default = "default_f_ref_activation")]
    pub f_ref_activation: f64,
    /// D-053 exterior N/F conductance multiplier (≥1). Applies only to extracellular–extracellular faces.
    #[serde(default = "default_m_ext")]
    pub m_ext: f64,
    /// D-053 membrane N/F attenuation multiplier (0 < m_β ≤ 1). Scales β_N and β_F together.
    #[serde(default = "default_m_beta")]
    pub m_beta: f64,
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

fn default_rho_a() -> f64 {
    1.0
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

fn default_precursor_m_p() -> f64 {
    1.0
}

fn default_precursor_product_inhibition_ki() -> f64 {
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

fn default_a_reference() -> f64 {
    1.0
}

fn default_k_active() -> f64 {
    0.0
}

fn default_k_charge() -> f64 {
    0.0
}

fn default_k_insert() -> f64 {
    0.0
}

fn default_k_relax() -> f64 {
    0.0
}

fn default_d_x() -> f64 {
    default_d_p()
}

fn default_k_mature() -> f64 {
    0.0
}

fn default_k_mature_basal() -> f64 {
    0.0
}

fn default_k_mature_cat() -> f64 {
    0.0
}

fn default_k_a_half() -> f64 {
    0.45
}

fn default_k_u_half() -> f64 {
    0.22
}

fn default_d_u() -> f64 {
    default_d_gamma()
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

fn default_activation_schema() -> u32 {
    1
}

fn default_k_c_activation() -> f64 {
    0.10
}

fn default_n_ref_activation() -> f64 {
    1.0
}

fn default_f_ref_activation() -> f64 {
    1.0
}

fn default_m_ext() -> f64 {
    1.0
}

fn default_m_beta() -> f64 {
    1.0
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
            rho_a: default_rho_a(),
            d019_mechanism_probe: None,
            use_mixed_structure_turnover: false,
            structure_turnover_eta: 0.0,
            eps_m: default_eps_m(),
            chi_m: default_chi_m(),
            k_precursor: default_k_precursor(),
            k_assembly: default_k_assembly(),
            k_precursor_decay: default_k_precursor_decay(),
            precursor_m_p: default_precursor_m_p(),
            precursor_product_inhibition_ki: default_precursor_product_inhibition_ki(),
            d_p: default_d_p(),
            k_ads: default_k_ads(),
            k_exchange: default_k_exchange(),
            k_exchange_eq: default_k_exchange_eq(),
            p_reference: default_p_reference(),
            a_reference: default_a_reference(),
            k_active: default_k_active(),
            k_charge: default_k_charge(),
            k_insert: default_k_insert(),
            k_relax: default_k_relax(),
            d_x: default_d_x(),
            k_mature: default_k_mature(),
            k_mature_basal: default_k_mature_basal(),
            k_mature_cat: default_k_mature_cat(),
            k_a_half: default_k_a_half(),
            k_u_half: default_k_u_half(),
            d_u: default_d_u(),
            k_gamma_decay: default_k_gamma_decay(),
            surface_turnover_schema: SurfaceTurnoverSchema::HistoricalUniform,
            d_gamma: default_d_gamma(),
            gamma_max: default_gamma_max(),
            gamma_reference: default_gamma_reference(),
            eta_n: default_eta_n(),
            delta_floor: default_delta_floor(),
            delta_face_eps: default_delta_face_eps(),
            eta_v: default_eta_v(),
            interface_grad_min: default_interface_grad_min(),
            surface_exchange_integrator: SurfaceExchangeIntegrator::InvariantDomainV2,
            activation_schema: default_activation_schema(),
            k_c_activation: default_k_c_activation(),
            n_ref_activation: default_n_ref_activation(),
            f_ref_activation: default_f_ref_activation(),
            m_ext: default_m_ext(),
            m_beta: default_m_beta(),
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
