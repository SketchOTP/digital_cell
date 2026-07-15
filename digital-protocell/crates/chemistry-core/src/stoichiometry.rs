//! D-012 compile-time stoichiometric descriptors and exact rational conservation analysis.
//!
//! Shared fixed descriptors are the sole stoichiometric source of truth for matrix
//! construction, ledger expectations, runtime-delta verification, and audit docs.

use serde::{Deserialize, Serialize};
use std::fmt;

pub const SEVEN_FIELD_COUNT: usize = 7;

/// Governed species row order: φ, C, N, F, W, A, M.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum SpeciesId {
    Phi = 0,
    C = 1,
    N = 2,
    F = 3,
    W = 4,
    A = 5,
    M = 6,
}

impl SpeciesId {
    pub const ALL: [SpeciesId; SEVEN_FIELD_COUNT] = [
        SpeciesId::Phi,
        SpeciesId::C,
        SpeciesId::N,
        SpeciesId::F,
        SpeciesId::W,
        SpeciesId::A,
        SpeciesId::M,
    ];

    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Phi => "phi",
            Self::C => "C",
            Self::N => "N",
            Self::F => "F",
            Self::W => "W",
            Self::A => "A",
            Self::M => "M",
        }
    }
}

/// Governed internal-reaction column order (transport/reservoir/clearance excluded).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ReactionId {
    Activation = 0,
    CatalystProduction = 1,
    StructureProduction = 2,
    MembraneProduction = 3,
    StructureDecay = 4,
    CatalystDecay = 5,
    ActivatedDecay = 6,
    MembraneDecay = 7,
    MembraneDetachment = 8,
}

impl ReactionId {
    pub const INTERNAL_COUNT: usize = 9;

    pub const ALL: [ReactionId; Self::INTERNAL_COUNT] = [
        ReactionId::Activation,
        ReactionId::CatalystProduction,
        ReactionId::StructureProduction,
        ReactionId::MembraneProduction,
        ReactionId::StructureDecay,
        ReactionId::CatalystDecay,
        ReactionId::ActivatedDecay,
        ReactionId::MembraneDecay,
        ReactionId::MembraneDetachment,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Activation => "activation",
            Self::CatalystProduction => "catalyst_production",
            Self::StructureProduction => "structure_production",
            Self::MembraneProduction => "membrane_production",
            Self::StructureDecay => "structure_decay",
            Self::CatalystDecay => "catalyst_decay",
            Self::ActivatedDecay => "activated_decay",
            Self::MembraneDecay => "membrane_decay",
            Self::MembraneDetachment => "membrane_detachment",
        }
    }
}

/// Reduced exact rational (i64 / i64, GCD-normalized, den > 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rational {
    pub num: i64,
    pub den: i64,
}

impl Rational {
    pub const ZERO: Self = Self { num: 0, den: 1 };
    pub const ONE: Self = Self { num: 1, den: 1 };

    pub const fn new(num: i64, den: i64) -> Self {
        assert!(den != 0, "rational denominator must be non-zero");
        let mut n = num;
        let mut d = den;
        if d < 0 {
            n = -n;
            d = -d;
        }
        let g = gcd_i64_const(n.abs(), d);
        Self {
            num: n / g,
            den: d / g,
        }
    }

    pub const fn from_i64(v: i64) -> Self {
        Self::new(v, 1)
    }

    pub fn is_zero(self) -> bool {
        self.num == 0
    }

    pub fn is_positive(self) -> bool {
        self.num > 0
    }

    pub fn is_negative(self) -> bool {
        self.num < 0
    }

    pub fn neg(self) -> Self {
        Self::new(-self.num, self.den)
    }

    pub fn add(self, other: Self) -> Self {
        Self::new(
            self.num * other.den + other.num * self.den,
            self.den * other.den,
        )
    }

    pub fn sub(self, other: Self) -> Self {
        self.add(other.neg())
    }

    pub fn mul(self, other: Self) -> Self {
        Self::new(self.num * other.num, self.den * other.den)
    }

    pub fn div(self, other: Self) -> Self {
        assert!(!other.is_zero(), "division by zero rational");
        Self::new(self.num * other.den, self.den * other.num)
    }

    pub fn dot(self, other: Self) -> Self {
        self.mul(other)
    }
}

impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.den == 1 {
            write!(f, "{}", self.num)
        } else {
            write!(f, "{}/{}", self.num, self.den)
        }
    }
}

fn gcd_i64(a: i64, b: i64) -> i64 {
    gcd_i64_const(a, b)
}

const fn gcd_i64_const(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    if a == 0 {
        1
    } else {
        a
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReactionStoichiometry {
    pub reaction: ReactionId,
    pub delta: [Rational; SEVEN_FIELD_COUNT],
}

impl ReactionStoichiometry {
    pub const fn new(reaction: ReactionId, delta: [Rational; SEVEN_FIELD_COUNT]) -> Self {
        Self { reaction, delta }
    }
}

macro_rules! stoich {
    ($rx:expr; $($sp:ident $c:expr),* $(,)?) => {{
        let mut d = [Rational::ZERO; SEVEN_FIELD_COUNT];
        $(d[SpeciesId::$sp.index()] = Rational::from_i64($c);)*
        ReactionStoichiometry::new($rx, d)
    }};
}

/// V1 internal reactions encoded from actual runtime deltas (2026-07-15 audit).
///
/// Documented comments in `activated_metabolism.rs` mention C+N+F→C+A+W and C+A→2C+W,
/// but isolated unit-extent deltas show:
/// - Activation: N+F→A+W (C modulates rate only; not consumed)
/// - Catalyst production: A→C+W (creates net material)
/// - Structure production: A→φ (no W on productive step; constrained-radius path)
/// - Membrane production: ∅→M (synthesis adds M without A/W consumption)
/// - Membrane decay/detachment: M→∅ (mass leaves M without entering W)
static V1_INTERNAL: [ReactionStoichiometry; ReactionId::INTERNAL_COUNT] = [
    // Activation — activated_metabolism.rs d_n/d_f/d_a/d_w
    stoich!(ReactionId::Activation; N -1, F -1, A 1, W 1),
    // Catalyst reproduction — d_c=+rep, d_a=-rep, d_w=+rep
    stoich!(ReactionId::CatalystProduction; C 1, A -1, W 1),
    // Structure production — simulation constrained-radius d_a=-r, virtual φ+=r, no W
    stoich!(ReactionId::StructureProduction; Phi 1, A -1),
    // Membrane synthesis — membrane.rs evolve_fixed_membrane synthesis_delta only +M
    stoich!(ReactionId::MembraneProduction; M 1),
    // Structure decay — k_structure_decay * phi → W
    stoich!(ReactionId::StructureDecay; Phi -1, W 1),
    // Catalyst turnover — C→W
    stoich!(ReactionId::CatalystDecay; C -1, W 1),
    // Activated decay — A→W
    stoich!(ReactionId::ActivatedDecay; A -1, W 1),
    // Membrane decay — M removed, not routed to W in v1
    stoich!(ReactionId::MembraneDecay; M -1),
    // Membrane detachment — M removed off-interface
    stoich!(ReactionId::MembraneDetachment; M -1),
];

pub fn v1_internal_reactions() -> &'static [ReactionStoichiometry] {
    &V1_INTERNAL
}

/// Conservative v2 unit-yield descriptors (Task 6+ runtime target).
pub fn v2_internal_reactions(
    eta_c: Rational,
    eta_phi: Rational,
    eta_m: Rational,
) -> [ReactionStoichiometry; ReactionId::INTERNAL_COUNT] {
    let one = Rational::ONE;
    let w_c = one.sub(eta_c);
    let w_phi = one.sub(eta_phi);
    let w_m = one.sub(eta_m);

    let mut d = [Rational::ZERO; SEVEN_FIELD_COUNT];
    d[SpeciesId::N.index()] = Rational::from_i64(-1);
    d[SpeciesId::F.index()] = Rational::from_i64(-1);
    d[SpeciesId::A.index()] = Rational::from_i64(1);
    d[SpeciesId::W.index()] = Rational::from_i64(1);
    let activation = ReactionStoichiometry::new(ReactionId::Activation, d);

    let mut d = [Rational::ZERO; SEVEN_FIELD_COUNT];
    d[SpeciesId::C.index()] = eta_c;
    d[SpeciesId::A.index()] = Rational::from_i64(-1);
    d[SpeciesId::W.index()] = w_c;
    let catalyst = ReactionStoichiometry::new(ReactionId::CatalystProduction, d);

    let mut d = [Rational::ZERO; SEVEN_FIELD_COUNT];
    d[SpeciesId::Phi.index()] = eta_phi;
    d[SpeciesId::A.index()] = Rational::from_i64(-1);
    d[SpeciesId::W.index()] = w_phi;
    let structure = ReactionStoichiometry::new(ReactionId::StructureProduction, d);

    let mut d = [Rational::ZERO; SEVEN_FIELD_COUNT];
    d[SpeciesId::M.index()] = eta_m;
    d[SpeciesId::A.index()] = Rational::from_i64(-1);
    d[SpeciesId::W.index()] = w_m;
    let membrane = ReactionStoichiometry::new(ReactionId::MembraneProduction, d);

    [
        activation,
        catalyst,
        structure,
        membrane,
        stoich!(ReactionId::StructureDecay; Phi -1, W 1),
        stoich!(ReactionId::CatalystDecay; C -1, W 1),
        stoich!(ReactionId::ActivatedDecay; A -1, W 1),
        stoich!(ReactionId::MembraneDecay; M -1, W 1),
        stoich!(ReactionId::MembraneDetachment; M -1, W 1),
    ]
}

/// Per-unit-extent field delta for a governed v1 reaction (descriptor source of truth).
pub fn v1_unit_extent_field_delta(reaction: ReactionId) -> [Rational; SEVEN_FIELD_COUNT] {
    v1_internal_reactions()[reaction as usize].delta
}

/// Floating-point isolated deltas mirroring v1 runtime paths for later cross-checks.
pub fn v1_runtime_isolated_delta(reaction: ReactionId) -> [f64; SEVEN_FIELD_COUNT] {
    v1_unit_extent_field_delta(reaction)
        .map(|r| r.num as f64 / r.den as f64)
}

/// Stage-C activation extent=1 isolated delta from `activated_metabolism_rates`.
pub fn v1_runtime_activation_delta(extent: f64) -> [f64; SEVEN_FIELD_COUNT] {
    let mut d = [0.0; SEVEN_FIELD_COUNT];
    d[SpeciesId::N.index()] = -extent;
    d[SpeciesId::F.index()] = -extent;
    d[SpeciesId::A.index()] = extent;
    d[SpeciesId::W.index()] = extent;
    d
}

/// Stage-C reproduction extent=1 isolated delta from `activated_metabolism_rates`.
pub fn v1_runtime_catalyst_production_delta(extent: f64) -> [f64; SEVEN_FIELD_COUNT] {
    let mut d = [0.0; SEVEN_FIELD_COUNT];
    d[SpeciesId::C.index()] = extent;
    d[SpeciesId::A.index()] = -extent;
    d[SpeciesId::W.index()] = extent;
    d
}

/// Constrained-radius structure production extent=1 (`try_d008_constrained_radius`).
pub fn v1_runtime_structure_production_delta(extent: f64) -> [f64; SEVEN_FIELD_COUNT] {
    let mut d = [0.0; SEVEN_FIELD_COUNT];
    d[SpeciesId::Phi.index()] = extent;
    d[SpeciesId::A.index()] = -extent;
    d
}

/// Membrane synthesis extent=1 from `evolve_fixed_membrane` (M only).
pub fn v1_runtime_membrane_production_delta(extent: f64) -> [f64; SEVEN_FIELD_COUNT] {
    let mut d = [0.0; SEVEN_FIELD_COUNT];
    d[SpeciesId::M.index()] = extent;
    d
}
