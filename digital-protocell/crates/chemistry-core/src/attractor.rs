//! Active-attractor and transient analysis (D-004).

use crate::bottleneck::BalanceDiagnostics;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AttractorClassification {
    ConvergentActiveAttractor,
    StateDependentAttractors,
    NoActiveAttractor,
    ContinuedDrift,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryPoint {
    pub substep: u64,
    pub sim_time: f64,
    pub m_phi: f64,
    pub m_c: f64,
    pub q_phi: f64,
    pub q_c: f64,
    pub slope_phi: f64,
    pub slope_c: f64,
    pub mean_n_inside: f64,
    pub mean_f_inside: f64,
    pub retention: f64,
    pub equivalent_radius: f64,
    pub compactness: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransientAnalysis {
    pub t_settle: Option<f64>,
    pub first_qualifying_window_start: Option<f64>,
    pub qualifying_duration: f64,
    pub lost_qualifying_behavior: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSummary {
    pub state_class: String,
    pub seed: u64,
    pub final_m_phi: f64,
    pub final_m_c: f64,
    pub final_q_phi: f64,
    pub final_q_c: f64,
    pub final_retention: f64,
    pub final_radius: f64,
    pub classification: AttractorClassification,
    pub transient: TransientAnalysis,
}

pub fn window_qualifies(b: &BalanceDiagnostics) -> bool {
    (0.95..=1.05).contains(&b.q_phi)
        && (0.95..=1.05).contains(&b.q_c)
        && b.slope_phi.abs() <= 1e-4
        && b.slope_catalyst.abs() <= 1e-4
}

pub fn analyze_transient(
    windows: &[(f64, f64, BalanceDiagnostics)],
) -> TransientAnalysis {
    let mut consecutive = 0u32;
    let mut t_settle = None;
    let mut first_start = None;
    let mut qualifying_duration = 0.0;
    let mut lost = false;
    let mut in_qualifying = false;

    for (start, end, b) in windows {
        if window_qualifies(b) {
            if first_start.is_none() {
                first_start = Some(*start);
            }
            consecutive += 1;
            qualifying_duration += end - start;
            in_qualifying = true;
            if consecutive >= 3 && t_settle.is_none() {
                t_settle = Some(*start);
            }
        } else {
            if in_qualifying && t_settle.is_some() {
                lost = true;
            }
            consecutive = 0;
            in_qualifying = false;
        }
    }

    TransientAnalysis {
        t_settle,
        first_qualifying_window_start: first_start,
        qualifying_duration,
        lost_qualifying_behavior: lost,
    }
}

pub fn classify_cross_state_convergence(summaries: &[RunSummary]) -> AttractorClassification {
    if summaries.is_empty() {
        return AttractorClassification::NoActiveAttractor;
    }
    let classes: std::collections::HashSet<_> = summaries.iter().map(|s| s.state_class.as_str()).collect();
    if classes.len() < 2 {
        return AttractorClassification::ContinuedDrift;
    }

    let mean = |f: fn(&RunSummary) -> f64| -> f64 {
        summaries.iter().map(f).sum::<f64>() / summaries.len() as f64
    };
    let m_phi = mean(|s| s.final_m_phi);
    let m_c = mean(|s| s.final_m_c);
    let q_phi = mean(|s| s.final_q_phi);
    let q_c = mean(|s| s.final_q_c);
    let ret = mean(|s| s.final_retention);
    let rad = mean(|s| s.final_radius);

    let within = |vals: &[f64], mean: f64, tol_frac: f64| -> bool {
        vals.iter().all(|v| (v - mean).abs() / mean.max(1e-12) <= tol_frac)
    };

    let m_phis: Vec<_> = summaries.iter().map(|s| s.final_m_phi).collect();
    let m_cs: Vec<_> = summaries.iter().map(|s| s.final_m_c).collect();
    let rads: Vec<_> = summaries.iter().map(|s| s.final_radius).collect();
    let rets: Vec<_> = summaries.iter().map(|s| s.final_retention).collect();
    let q_phis: Vec<_> = summaries.iter().map(|s| s.final_q_phi).collect();
    let q_cs: Vec<_> = summaries.iter().map(|s| s.final_q_c).collect();

    let convergent = within(&m_phis, m_phi, 0.10)
        && within(&m_cs, m_c, 0.10)
        && within(&rads, rad, 0.10)
        && rets.iter().all(|r| (r - ret).abs() <= 0.05)
        && q_phis.iter().all(|q| (q - q_phi).abs() <= 0.05)
        && q_cs.iter().all(|q| (q - q_c).abs() <= 0.05)
        && (0.95..=1.05).contains(&q_phi)
        && (0.95..=1.05).contains(&q_c);

    if convergent {
        AttractorClassification::ConvergentActiveAttractor
    } else if summaries.iter().any(|s| {
        matches!(
            s.classification,
            AttractorClassification::NoActiveAttractor | AttractorClassification::ContinuedDrift
        )
    }) {
        AttractorClassification::StateDependentAttractors
    } else {
        AttractorClassification::StateDependentAttractors
    }
}
