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

pub fn stoichiometric_matrix(reactions: &[ReactionStoichiometry]) -> Vec<Vec<Rational>> {
    let mut matrix = vec![vec![Rational::ZERO; reactions.len()]; SEVEN_FIELD_COUNT];
    for (col, rx) in reactions.iter().enumerate() {
        for (row, &coeff) in rx.delta.iter().enumerate() {
            matrix[row][col] = coeff;
        }
    }
    matrix
}

pub fn exact_rank(matrix: &[Vec<Rational>]) -> usize {
    if matrix.is_empty() {
        return 0;
    }
    let rows = matrix.len();
    let cols = matrix[0].len();
    let mut a = matrix.to_vec();
    let mut rank = 0usize;
    let mut pivot_col = 0usize;
    for row in 0..rows {
        if pivot_col >= cols {
            break;
        }
        let mut pivot_row = None;
        for r in row..rows {
            if !a[r][pivot_col].is_zero() {
                pivot_row = Some(r);
                break;
            }
        }
        let Some(pr) = pivot_row else {
            pivot_col += 1;
            continue;
        };
        a.swap(row, pr);
        let pivot = a[row][pivot_col];
        for c in pivot_col..cols {
            a[row][c] = a[row][c].div(pivot);
        }
        for r in 0..rows {
            if r == row || a[r][pivot_col].is_zero() {
                continue;
            }
            let factor = a[r][pivot_col];
            for c in pivot_col..cols {
                a[r][c] = a[r][c].sub(factor.mul(a[row][c]));
            }
        }
        rank += 1;
        pivot_col += 1;
    }
    rank
}

fn rref_pivot_columns(a: &mut [Vec<Rational>]) -> Vec<usize> {
    if a.is_empty() {
        return Vec::new();
    }
    let rows = a.len();
    let cols = a[0].len();
    let mut pivot_cols = Vec::new();
    let mut pivot_row = 0usize;
    let mut col = 0usize;
    while pivot_row < rows && col < cols {
        let mut swap_row = None;
        for r in pivot_row..rows {
            if !a[r][col].is_zero() {
                swap_row = Some(r);
                break;
            }
        }
        let Some(sr) = swap_row else {
            col += 1;
            continue;
        };
        a.swap(pivot_row, sr);
        let pivot = a[pivot_row][col];
        for c in col..cols {
            a[pivot_row][c] = a[pivot_row][c].div(pivot);
        }
        for r in 0..rows {
            if r == pivot_row || a[r][col].is_zero() {
                continue;
            }
            let factor = a[r][col];
            for c in col..cols {
                a[r][c] = a[r][c].sub(factor.mul(a[pivot_row][c]));
            }
        }
        pivot_cols.push(col);
        pivot_row += 1;
        col += 1;
    }
    pivot_cols
}

/// Basis for `{x | a * x = 0}` with `x` length = column count of `a`.
fn nullspace_columns(a: &[Vec<Rational>]) -> Vec<Vec<Rational>> {
    if a.is_empty() || a[0].is_empty() {
        return Vec::new();
    }
    let cols = a[0].len();
    let mut rref = a.to_vec();
    let pivot_cols = rref_pivot_columns(&mut rref);
    let pivot_set: std::collections::BTreeSet<usize> = pivot_cols.iter().copied().collect();
    let free_cols: Vec<usize> = (0..cols).filter(|c| !pivot_set.contains(c)).collect();
    let mut basis = Vec::new();
    for &free in &free_cols {
        let mut vec = vec![Rational::ZERO; cols];
        vec[free] = Rational::ONE;
        for (row, &pivot_col) in pivot_cols.iter().enumerate() {
            if row < rref.len() && free < rref[row].len() && !rref[row][free].is_zero() {
                vec[pivot_col] = rref[row][free].neg();
            }
        }
        basis.push(vec);
    }
    basis
}

pub fn left_nullspace(matrix: &[Vec<Rational>]) -> Vec<Vec<Rational>> {
    // m^T S = 0  <=>  S^T m = 0
    nullspace_columns(&transpose(matrix))
}

pub fn right_nullspace(matrix: &[Vec<Rational>]) -> Vec<Vec<Rational>> {
    nullspace_columns(matrix)
}

fn transpose(matrix: &[Vec<Rational>]) -> Vec<Vec<Rational>> {
    if matrix.is_empty() {
        return Vec::new();
    }
    let rows = matrix.len();
    let cols = matrix[0].len();
    let mut t = vec![vec![Rational::ZERO; rows]; cols];
    for r in 0..rows {
        for c in 0..cols {
            t[c][r] = matrix[r][c];
        }
    }
    t
}

pub fn verify_m_transpose_s_zero(m: &[Rational], s: &[Vec<Rational>]) -> bool {
    if s.is_empty() || s[0].is_empty() || m.len() != s.len() {
        return false;
    }
    let cols = s[0].len();
    for col in 0..cols {
        let mut sum = Rational::ZERO;
        for (row, &mr) in m.iter().enumerate() {
            sum = sum.add(mr.mul(s[row][col]));
        }
        if !sum.is_zero() {
            return false;
        }
    }
    true
}

fn reaction_material_residual(m: &[Rational], reaction: &ReactionStoichiometry) -> Rational {
    let mut sum = Rational::ZERO;
    for (i, &mi) in m.iter().enumerate() {
        sum = sum.add(mi.mul(reaction.delta[i]));
    }
    sum
}

fn normalize_vector(v: &[Rational]) -> Vec<Rational> {
    // ponytail: lcm scaling for display; homogeneous tests use primitive integer scaling
    let lcm_den = v
        .iter()
        .filter(|c| !c.is_zero())
        .map(|c| c.den)
        .fold(1i64, |acc, d| acc / gcd_i64(acc, d) * d);
    v.iter()
        .map(|c| Rational::new(c.num * (lcm_den / c.den), lcm_den))
        .collect()
}

fn is_strictly_positive(v: &[Rational]) -> bool {
    !v.is_empty() && v.iter().all(|c| c.is_positive())
}

fn is_nonnegative(v: &[Rational]) -> bool {
    v.iter().all(|c| c.is_zero() || c.is_positive())
}

fn is_nontrivial(v: &[Rational]) -> bool {
    v.iter().any(|c| !c.is_zero())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConservationClass {
    StrictlyConservative,
    PartiallyConservative,
    NoPositiveConservationVector,
    InconsistentStoichiometry,
}

impl fmt::Display for ConservationClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StrictlyConservative => write!(f, "STRICTLY_CONSERVATIVE"),
            Self::PartiallyConservative => write!(f, "PARTIALLY_CONSERVATIVE"),
            Self::NoPositiveConservationVector => write!(f, "NO_POSITIVE_CONSERVATION_VECTOR"),
            Self::InconsistentStoichiometry => write!(f, "INCONSISTENT_STOICHIOMETRY"),
        }
    }
}

pub struct ConservationAnalysis {
    pub class: ConservationClass,
    pub rank: usize,
    pub left_nullspace_dimension: usize,
    pub right_nullspace_dimension: usize,
    pub strictly_positive_vectors: Vec<Vec<Rational>>,
    pub nonnegative_vectors: Vec<Vec<Rational>>,
    pub nonconservative_reactions: Vec<ReactionId>,
}

pub fn classify_conservation(matrix: &[Vec<Rational>]) -> ConservationClass {
    classify_conservation_detailed(matrix).class
}

pub fn classify_conservation_detailed(matrix: &[Vec<Rational>]) -> ConservationAnalysis {
    let rank = exact_rank(matrix);
    let left = left_nullspace(matrix);
    let right = right_nullspace(matrix);

    let mut strictly_positive_vectors = Vec::new();
    let all_ones = vec![Rational::ONE; matrix.len()];
    if verify_m_transpose_s_zero(&all_ones, matrix) && is_strictly_positive(&all_ones) {
        strictly_positive_vectors.push(all_ones.clone());
    }
    for v in &left {
        let normalized = normalize_vector(v);
        if is_strictly_positive(&normalized)
            && verify_m_transpose_s_zero(&normalized, matrix)
            && !strictly_positive_vectors.iter().any(|existing| existing == &normalized)
        {
            strictly_positive_vectors.push(normalized);
        }
    }

    let mut nonnegative_vectors = Vec::new();
    for v in &left {
        let normalized = normalize_vector(v);
        if is_nonnegative(&normalized) && is_nontrivial(&normalized) && verify_m_transpose_s_zero(&normalized, matrix) {
            nonnegative_vectors.push(normalized);
        }
    }

    let reactions = v1_internal_reactions();
    let nonconservative_reactions: Vec<ReactionId> = reactions
        .iter()
        .filter(|rx| !reaction_material_residual(&all_ones, rx).is_zero())
        .map(|rx| rx.reaction)
        .collect();

    let class = if !strictly_positive_vectors.is_empty() {
        ConservationClass::StrictlyConservative
    } else if nonnegative_vectors.is_empty() {
        ConservationClass::NoPositiveConservationVector
    } else {
        ConservationClass::PartiallyConservative
    };

    ConservationAnalysis {
        class,
        rank,
        left_nullspace_dimension: left.len(),
        right_nullspace_dimension: right.len(),
        strictly_positive_vectors,
        nonnegative_vectors,
        nonconservative_reactions,
    }
}

pub fn positive_conservation_vectors(matrix: &[Vec<Rational>]) -> Vec<Vec<Rational>> {
    classify_conservation_detailed(matrix).strictly_positive_vectors
}

pub fn nonconservative_reactions_under_vector(
    m: &[Rational],
    reactions: &[ReactionStoichiometry],
) -> Vec<ReactionId> {
    reactions
        .iter()
        .filter(|rx| !reaction_material_residual(m, rx).is_zero())
        .map(|rx| rx.reaction)
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
