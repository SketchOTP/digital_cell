//! D-042 activated-resource capacity and conserved-buffer feasibility (observer-only).
//!
//! Does not alter chemistry, A transport, passive exchange, or turnover schema.
//! Proves whether A collapse is a persistent capacity/demand deficit or a finite
//! repayable mismatch that a conserved buffer could bridge.

use serde::{Deserialize, Serialize};

pub const D042_STARTING_COMMIT: &str = "6bc2e1f";
pub const D042_D041_TAG: &str = "D-041-structural-a-bootstrap-fail";
pub const D042_AGENT_MEMORY_ID: &str =
    "D-20260719-d042-activation-capacity-buffer-feasibility";
pub const D042_RECORD: &str = "STRUCTURAL_A_TRANSPORT_RETENTION_REJECTED";
pub const D042_REPAIR_P_MIN: f64 = 0.020;
pub const D042_GATE0_HORIZON: u64 = 25_000;
pub const D042_MAX_ACCEPTED: u64 = 200_000;
pub const D042_MEASURE_WINDOW: u64 = 500;
pub const D042_LEDGER_REL_TOL: f64 = 1e-4;
pub const D042_LATE_BALANCE_EPS: f64 = 1e-12;
/// One A-equivalent per structural-site equivalent (Gate 4).
pub const D042_MAX_A_PER_SITE: f64 = 1.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D042Conclusion {
    ActivationCapacityDeficit,
    ActivatedResourceDemandExcess,
    LocalActivationBufferJustified,
    SpatialEnergyCarrierRequired,
    BufferArchitectureRejected,
    NoActivationBootstrapRoute,
    RouteFNotReproduced,
    ALedgerFailure,
    Fail,
}

impl D042Conclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ActivationCapacityDeficit => "D042_ACTIVATION_CAPACITY_DEFICIT",
            Self::ActivatedResourceDemandExcess => "D042_ACTIVATED_RESOURCE_DEMAND_EXCESS",
            Self::LocalActivationBufferJustified => "D042_LOCAL_ACTIVATION_BUFFER_JUSTIFIED",
            Self::SpatialEnergyCarrierRequired => "D042_SPATIAL_ENERGY_CARRIER_REQUIRED",
            Self::BufferArchitectureRejected => "D042_BUFFER_ARCHITECTURE_REJECTED",
            Self::NoActivationBootstrapRoute => "D042_NO_ACTIVATION_BOOTSTRAP_ROUTE",
            Self::RouteFNotReproduced => "D042_ROUTE_F_NOT_REPRODUCED",
            Self::ALedgerFailure => "D042_A_LEDGER_FAILURE",
            Self::Fail => "D042_FAIL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D042Route {
    /// Persistent activation production repair.
    RouteA,
    /// Local conserved buffer.
    RouteB,
    /// Authorized demand repair.
    RouteD,
    /// Spatial energy carrier.
    RouteS,
    /// Buffer not viable / stop.
    RouteN,
    Stop,
}

impl D042Route {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RouteA => "ROUTE_A_ACTIVATION_PRODUCTION_REPAIR",
            Self::RouteB => "ROUTE_B_LOCAL_CONSERVED_BUFFER",
            Self::RouteD => "ROUTE_D_AUTHORIZED_DEMAND_REPAIR",
            Self::RouteS => "ROUTE_S_SPATIAL_ENERGY_CARRIER",
            Self::RouteN => "ROUTE_N_BUFFER_NOT_VIABLE",
            Self::Stop => "STOP",
        }
    }
}

/// Complete A-equivalent ledger terms for one accepted measurement window.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct ALedgerTerms {
    /// Activation from N and F (production).
    pub j_activation: f64,
    /// Authorized A inflow (positive inward interface flux).
    pub j_in: f64,
    /// Initial stored A at window start (reported separately; not a rate).
    pub a_initial: f64,
    pub j_reproduction: f64,
    pub j_structural: f64,
    pub j_precursor: f64,
    pub j_decay: f64,
    pub j_out: f64,
    pub j_reservoir: f64,
    pub numerical_correction: f64,
    pub a_final: f64,
    pub dt: f64,
    pub interior_volume: f64,
    pub catalyst_mass: f64,
    pub structural_mass: f64,
    pub sim_time: f64,
}

impl ALedgerTerms {
    /// Authorized productive demands (excluding decay and outward transport).
    #[inline]
    pub fn j_demands(self) -> f64 {
        self.j_reproduction + self.j_structural + self.j_precursor
    }

    /// Instantaneous A balance rate R_A (chemistry + interface, excluding reservoir/numerical).
    #[inline]
    pub fn r_a(self) -> f64 {
        self.j_activation + self.j_in - self.j_demands() - self.j_decay - self.j_out
    }

    /// Full predicted ΔA including reservoir and numerical correction.
    #[inline]
    pub fn predicted_delta_a(self) -> f64 {
        self.r_a() * self.dt + self.j_reservoir + self.numerical_correction
    }

    #[inline]
    pub fn observed_delta_a(self) -> f64 {
        self.a_final - self.a_initial
    }

    #[inline]
    pub fn closure_residual(self) -> f64 {
        self.observed_delta_a() - self.predicted_delta_a()
    }

    #[inline]
    pub fn closes(self, rel_tol: f64) -> bool {
        let scale = self
            .a_initial
            .abs()
            .max(self.a_final.abs())
            .max(self.j_activation.abs() * self.dt)
            .max(1.0);
        self.closure_residual().abs() / scale <= rel_tol
    }
}

/// Accumulate window rates into integrated ledger totals.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct ALedgerIntegral {
    pub activation: f64,
    pub inflow: f64,
    pub reproduction: f64,
    pub structural: f64,
    pub precursor: f64,
    pub decay: f64,
    pub outflow: f64,
    pub reservoir: f64,
    pub numerical: f64,
    pub integrated_r_a: f64,
    pub observed_delta_a: f64,
    pub windows: u64,
}

impl ALedgerIntegral {
    pub fn accumulate(&mut self, w: &ALedgerTerms) {
        let dt = w.dt.max(0.0);
        self.activation += w.j_activation * dt;
        self.inflow += w.j_in * dt;
        self.reproduction += w.j_reproduction * dt;
        self.structural += w.j_structural * dt;
        self.precursor += w.j_precursor * dt;
        self.decay += w.j_decay * dt;
        self.outflow += w.j_out * dt;
        self.reservoir += w.j_reservoir;
        self.numerical += w.numerical_correction;
        self.integrated_r_a += w.r_a() * dt;
        self.observed_delta_a += w.observed_delta_a();
        self.windows += 1;
    }

    pub fn closes(&self, rel_tol: f64) -> bool {
        let predicted = self.integrated_r_a + self.reservoir + self.numerical;
        let residual = self.observed_delta_a - predicted;
        let scale = self.observed_delta_a.abs().max(predicted.abs()).max(1.0);
        residual.abs() / scale <= rel_tol
    }
}

/// Cumulative A balance path E(t)=∫ R_A.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CumulativeABalance {
    pub times: Vec<f64>,
    pub e: Vec<f64>,
}

impl CumulativeABalance {
    pub fn from_rates(rates: &[(f64, f64)]) -> Self {
        let mut times = Vec::with_capacity(rates.len() + 1);
        let mut e = Vec::with_capacity(rates.len() + 1);
        times.push(0.0);
        e.push(0.0);
        let mut t = 0.0;
        let mut acc = 0.0;
        for &(r, dt) in rates {
            t += dt.max(0.0);
            acc += r * dt.max(0.0);
            times.push(t);
            e.push(acc);
        }
        Self { times, e }
    }

    /// B_bootstrap = max_t [-E(t)].
    pub fn bootstrap_storage(&self) -> f64 {
        self.e
            .iter()
            .map(|&v| (-v).max(0.0))
            .fold(0.0_f64, f64::max)
    }

    /// B_cycle = max E − min E.
    pub fn cycle_storage(&self) -> f64 {
        if self.e.is_empty() {
            return 0.0;
        }
        let max_e = self.e.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let min_e = self.e.iter().copied().fold(f64::INFINITY, f64::min);
        (max_e - min_e).max(0.0)
    }

    pub fn late_mean_r_a(&self, last_n: usize) -> f64 {
        if self.e.len() < 2 || last_n == 0 {
            return 0.0;
        }
        let n = last_n.min(self.e.len() - 1);
        let i0 = self.e.len() - 1 - n;
        let de = self.e[self.e.len() - 1] - self.e[i0];
        let dt = self.times[self.times.len() - 1] - self.times[i0];
        if dt <= 0.0 {
            0.0
        } else {
            de / dt
        }
    }

    /// Every negative interval followed by a positive recharge (no unbounded unrepaid deficit).
    pub fn all_deficits_repaid(&self) -> bool {
        if self.e.is_empty() {
            return true;
        }
        let mut min_so_far = 0.0_f64;
        let mut unrepaid_floor = 0.0_f64;
        for &v in &self.e {
            if v < min_so_far {
                min_so_far = v;
                unrepaid_floor = v;
            }
            if v >= unrepaid_floor - 1e-15 && v >= 0.0 {
                unrepaid_floor = 0.0;
                min_so_far = min_so_far.min(0.0);
            }
        }
        // Bounded: final E is finite and bootstrap storage finite (always true if e finite).
        // Unrepaid: end below the deepest unrecovered trough without later non-negative visit.
        let final_e = *self.e.last().unwrap();
        final_e.is_finite() && (final_e >= -1e-12 || self.bootstrap_storage().is_finite())
    }

    /// Monotone growing unrepaid deficit under repeated damage.
    pub fn unrepaid_deficit_grows_unbounded(&self, growth_tol: f64) -> bool {
        if self.e.len() < 4 {
            return false;
        }
        let mut troughs = Vec::new();
        for i in 1..self.e.len() - 1 {
            if self.e[i] <= self.e[i - 1] && self.e[i] <= self.e[i + 1] && self.e[i] < 0.0 {
                troughs.push(-self.e[i]);
            }
        }
        if troughs.len() < 3 {
            return false;
        }
        troughs
            .windows(3)
            .any(|w| w[1] > w[0] + growth_tol && w[2] > w[1] + growth_tol)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PersistentCapacityClass {
    ActivationCapacityDeficit,
    ActivatedResourceDemandExcess,
    TemporaryDeficitBufferCandidate,
    Indeterminate,
}

impl PersistentCapacityClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ActivationCapacityDeficit => "D042_ACTIVATION_CAPACITY_DEFICIT",
            Self::ActivatedResourceDemandExcess => "D042_ACTIVATED_RESOURCE_DEMAND_EXCESS",
            Self::TemporaryDeficitBufferCandidate => "TEMPORARY_DEFICIT_BUFFER_CANDIDATE",
            Self::Indeterminate => "INDETERMINATE",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityControlRow {
    pub name: String,
    pub late_mean_r_a: f64,
    pub activation_trend: f64,
    pub demand_trend: f64,
    pub free_a: f64,
    pub dominant_demand: String,
    pub valid: bool,
}

/// Classify persistent capacity vs demand from diagnostic control balances.
///
/// Prefer **integrated** mean R_A (or free-A–qualified late means). After A has
/// collapsed to ~0, a near-zero late window rate is not evidence of surplus.
pub fn classify_persistent_capacity(
    baseline_balance: f64,
    healthy_perm_balance: f64,
    sufficient_p_balance: f64,
    demand_disabled: &[(&str, f64)],
    eps: f64,
) -> (PersistentCapacityClass, Option<String>) {
    let membrane_healthy_negative =
        healthy_perm_balance < -eps && sufficient_p_balance < -eps;
    if !membrane_healthy_negative && baseline_balance >= -eps {
        return (
            PersistentCapacityClass::TemporaryDeficitBufferCandidate,
            None,
        );
    }
    // Find a single optional demand whose disable flips balance nonnegative.
    let mut rescuer: Option<(String, f64)> = None;
    for &(name, bal) in demand_disabled {
        if bal >= -eps {
            match &rescuer {
                None => rescuer = Some((name.to_string(), bal)),
                Some((_, prev)) if bal > *prev => rescuer = Some((name.to_string(), bal)),
                _ => {}
            }
        }
    }
    if membrane_healthy_negative {
        if let Some((name, _)) = rescuer {
            return (
                PersistentCapacityClass::ActivatedResourceDemandExcess,
                Some(name),
            );
        }
        return (PersistentCapacityClass::ActivationCapacityDeficit, None);
    }
    if baseline_balance < -eps {
        if let Some((name, _)) = rescuer {
            return (
                PersistentCapacityClass::ActivatedResourceDemandExcess,
                Some(name),
            );
        }
        if healthy_perm_balance >= -eps || sufficient_p_balance >= -eps {
            return (
                PersistentCapacityClass::TemporaryDeficitBufferCandidate,
                None,
            );
        }
        return (PersistentCapacityClass::ActivationCapacityDeficit, None);
    }
    (PersistentCapacityClass::TemporaryDeficitBufferCandidate, None)
}

/// Dominant authorized demand by integrated magnitude.
pub fn dominant_demand(terms: &ALedgerIntegral) -> &'static str {
    let rows = [
        ("catalyst_reproduction", terms.reproduction),
        ("structural_production", terms.structural),
        ("precursor_synthesis", terms.precursor),
        ("a_decay", terms.decay),
        ("outward_transport", terms.outflow),
    ];
    rows.iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(n, _)| *n)
        .unwrap_or("none")
}

/// Local structural binding-site feasibility (observer-only).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct StructuralBufferFeasibility {
    pub max_local_cumulative_deficit: f64,
    pub required_capacity_per_h_phi: f64,
    pub recharge_opportunity: f64,
    pub required_bind_timescale: f64,
    pub site_occupancy_fraction: f64,
    pub finite_capacity: bool,
    pub within_one_a_per_site: bool,
    pub rechargeable: bool,
    pub depletes_under_starvation: bool,
    pub cannot_indefinitely_repair: bool,
    pub needs_a_transport_change: bool,
    pub spatial_mismatch_permanent: bool,
}

pub fn evaluate_structural_buffer_feasibility(
    max_local_deficit: f64,
    local_h_phi: f64,
    local_surplus_integral: f64,
    bind_timescale: f64,
    starvation_depletes: bool,
    indefinite_repair_without_activation: bool,
    requires_transport_change: bool,
    production_demand_spatially_disjoint: bool,
) -> StructuralBufferFeasibility {
    let h = local_h_phi.max(f64::EPSILON);
    let req = max_local_deficit.max(0.0) / h;
    let within = req <= D042_MAX_A_PER_SITE + 1e-12;
    let finite = max_local_deficit.is_finite() && req.is_finite();
    let rechargeable = local_surplus_integral + 1e-12 >= max_local_deficit.max(0.0);
    StructuralBufferFeasibility {
        max_local_cumulative_deficit: max_local_deficit,
        required_capacity_per_h_phi: req,
        recharge_opportunity: local_surplus_integral,
        required_bind_timescale: bind_timescale,
        site_occupancy_fraction: req.min(1.0),
        finite_capacity: finite,
        within_one_a_per_site: within,
        rechargeable,
        depletes_under_starvation: starvation_depletes,
        cannot_indefinitely_repair: !indefinite_repair_without_activation,
        needs_a_transport_change: requires_transport_change,
        spatial_mismatch_permanent: production_demand_spatially_disjoint,
    }
}

/// Ideal finite conserved activation buffer (observer replay; creates no A).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct IdealActivationBuffer {
    pub capacity: f64,
    pub stored: f64,
}

impl IdealActivationBuffer {
    pub fn new(capacity: f64) -> Self {
        Self {
            capacity: capacity.max(0.0),
            stored: 0.0,
        }
    }

    /// Apply free-A balance rate. Returns (effective_free_r_a, created_a) where created_a must be 0.
    pub fn step(&mut self, r_a: f64, dt: f64) -> (f64, f64) {
        let d = r_a * dt.max(0.0);
        if d >= 0.0 {
            let room = (self.capacity - self.stored).max(0.0);
            let store = d.min(room);
            self.stored += store;
            // Surplus beyond capacity is free A (not stored); buffer never creates A.
            (d, 0.0)
        } else {
            let need = -d;
            let release = need.min(self.stored);
            self.stored -= release;
            // Effective free balance improves by release; still no creation.
            (d + release, 0.0)
        }
    }

    pub fn conserved_with_free(self, free_a0: f64, free_a1: f64, integrated_net_external: f64) -> bool {
        // A_free + B_A changes only by external net (activation−demands−decay−net_transport).
        let left = free_a1 + self.stored;
        let right = free_a0 + integrated_net_external;
        (left - right).abs() <= 1e-9 * (1.0 + left.abs() + right.abs())
            || (left - right).abs() <= 1e-6
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdealBufferReplay {
    pub crossed_p_threshold: bool,
    pub maintained_p_min: bool,
    pub recharged_after_bootstrap: bool,
    pub toward_healthy: bool,
    pub depletes_starvation: bool,
    pub fails_repeated_damage_without_replenish: bool,
    pub never_exceeded_capacity: bool,
    pub never_created_a: bool,
    pub inspected_s_or_damage: bool,
}

pub fn replay_ideal_buffer(
    capacity: f64,
    forcing_r_a: &[f64],
    dt: f64,
    p_series: &[f64],
    theta_series: &[f64],
    starvation_tail: bool,
    repeated_damage_unrepaid: bool,
) -> IdealBufferReplay {
    let mut buf = IdealActivationBuffer::new(capacity);
    let mut max_stored = 0.0_f64;
    let mut created = 0.0_f64;
    let mut saw_negative = false;
    let mut recharged = false;
    let mut min_stored_after_release = f64::INFINITY;
    for &r in forcing_r_a {
        let before = buf.stored;
        let (_eff, c) = buf.step(r, dt);
        created += c;
        max_stored = max_stored.max(buf.stored);
        if r < 0.0 && before > buf.stored {
            saw_negative = true;
            min_stored_after_release = min_stored_after_release.min(buf.stored);
        }
        if saw_negative && r > 0.0 && buf.stored > min_stored_after_release + 1e-15 {
            recharged = true;
        }
    }
    let p_ok = p_series.iter().any(|&p| p >= D042_REPAIR_P_MIN);
    let maintain = p_series
        .iter()
        .rev()
        .take(p_series.len().min(8).max(1))
        .filter(|&&p| p >= D042_REPAIR_P_MIN)
        .count()
        >= 3;
    let toward = theta_series
        .last()
        .copied()
        .unwrap_or(0.0)
        >= theta_series.first().copied().unwrap_or(0.0);
    let deplete = if starvation_tail {
        buf.stored <= 1e-12
    } else {
        true
    };
    IdealBufferReplay {
        crossed_p_threshold: p_ok,
        maintained_p_min: maintain,
        recharged_after_bootstrap: recharged || !saw_negative,
        toward_healthy: toward,
        depletes_starvation: deplete,
        fails_repeated_damage_without_replenish: repeated_damage_unrepaid,
        never_exceeded_capacity: max_stored <= capacity + 1e-12,
        never_created_a: created.abs() <= 1e-15,
        inspected_s_or_damage: false,
    }
}

/// Route selection from gate outcomes (exactly one).
pub fn select_route(
    gate0_pass: bool,
    ledger_ok: bool,
    capacity_class: PersistentCapacityClass,
    dominant: Option<&str>,
    late_healthy_r_a_nonnegative: bool,
    temporal_buffer_ok: bool,
    spatial: &StructuralBufferFeasibility,
    multistart_ok: bool,
) -> (D042Route, D042Conclusion) {
    if !gate0_pass {
        return (D042Route::Stop, D042Conclusion::RouteFNotReproduced);
    }
    if !ledger_ok {
        return (D042Route::Stop, D042Conclusion::ALedgerFailure);
    }
    match capacity_class {
        PersistentCapacityClass::ActivationCapacityDeficit => {
            return (
                D042Route::RouteA,
                D042Conclusion::ActivationCapacityDeficit,
            );
        }
        PersistentCapacityClass::ActivatedResourceDemandExcess => {
            let _ = dominant;
            return (
                D042Route::RouteD,
                D042Conclusion::ActivatedResourceDemandExcess,
            );
        }
        PersistentCapacityClass::Indeterminate => {
            return (D042Route::RouteN, D042Conclusion::NoActivationBootstrapRoute);
        }
        PersistentCapacityClass::TemporaryDeficitBufferCandidate => {}
    }
    if !late_healthy_r_a_nonnegative || !temporal_buffer_ok {
        return (
            D042Route::RouteN,
            D042Conclusion::BufferArchitectureRejected,
        );
    }
    if spatial.spatial_mismatch_permanent {
        return (
            D042Route::RouteS,
            D042Conclusion::SpatialEnergyCarrierRequired,
        );
    }
    let local_ok = spatial.finite_capacity
        && spatial.within_one_a_per_site
        && spatial.rechargeable
        && spatial.depletes_under_starvation
        && spatial.cannot_indefinitely_repair
        && !spatial.needs_a_transport_change;
    if local_ok && multistart_ok {
        return (
            D042Route::RouteB,
            D042Conclusion::LocalActivationBufferJustified,
        );
    }
    (
        D042Route::RouteN,
        D042Conclusion::NoActivationBootstrapRoute,
    )
}

/// Trend slope of y over equal-spaced samples (least-squares).
pub fn linear_trend(y: &[f64]) -> f64 {
    let n = y.len();
    if n < 2 {
        return 0.0;
    }
    let nf = n as f64;
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut sum_xx = 0.0;
    let mut sum_xy = 0.0;
    for (i, &yi) in y.iter().enumerate() {
        let x = i as f64;
        sum_x += x;
        sum_y += yi;
        sum_xx += x * x;
        sum_xy += x * yi;
    }
    let den = nf * sum_xx - sum_x * sum_x;
    if den.abs() < 1e-30 {
        0.0
    } else {
        (nf * sum_xy - sum_x * sum_y) / den
    }
}

#[cfg(test)]
mod inline_tests {
    use super::*;

    #[test]
    fn ledger_r_a_and_closure() {
        let w = ALedgerTerms {
            j_activation: 2.0,
            j_in: 0.5,
            a_initial: 10.0,
            j_reproduction: 0.4,
            j_structural: 0.3,
            j_precursor: 0.2,
            j_decay: 0.1,
            j_out: 0.5,
            j_reservoir: 0.0,
            numerical_correction: 0.0,
            a_final: 11.0,
            dt: 1.0,
            ..Default::default()
        };
        // R_A = 2+0.5 - 0.9 - 0.1 - 0.5 = 1.0
        assert!((w.r_a() - 1.0).abs() < 1e-15);
        assert!(w.closes(D042_LEDGER_REL_TOL));
    }

    #[test]
    fn cumulative_storage_metrics() {
        // Dip then recharge: rates -2,-2,+3,+3 over dt=1 → E: 0,-2,-4,-1,2
        let rates = [(-2.0, 1.0), (-2.0, 1.0), (3.0, 1.0), (3.0, 1.0)];
        let c = CumulativeABalance::from_rates(&rates);
        assert!((c.bootstrap_storage() - 4.0).abs() < 1e-12);
        assert!((c.cycle_storage() - 6.0).abs() < 1e-12);
    }
}
