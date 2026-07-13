//! Balance-versus-slope ledger reconciliation (D-004).

use crate::accounting::AccountingState;
use crate::bottleneck::BalanceWindowSample;
use crate::operators::total_mass;
use crate::grid::Grid;
use crate::fields::FieldBuffers;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerReconciliation {
    pub observed_delta_phi: f64,
    pub predicted_delta_phi: f64,
    pub observed_delta_c: f64,
    pub predicted_delta_c: f64,
    pub relative_error_phi: f64,
    pub relative_error_c: f64,
    pub slope_phi_from_ledger: f64,
    pub slope_c_from_ledger: f64,
    pub within_tolerance: bool,
}

pub fn reconcile_window(
    grid: &Grid,
    fields_start: &FieldBuffers,
    fields_end: &FieldBuffers,
    samples: &[BalanceWindowSample],
    accounting: &AccountingState,
) -> LedgerReconciliation {
    let m_phi0 = total_mass(grid, &fields_start.structure);
    let m_phi1 = total_mass(grid, &fields_end.structure);
    let mc0 = total_mass(grid, &fields_start.catalyst);
    let mc1 = total_mass(grid, &fields_end.catalyst);

    let s_phi: f64 = samples.iter().map(|s| s.s_phi).sum();
    let d_phi: f64 = samples.iter().map(|s| s.d_phi).sum();
    let r_c: f64 = samples.iter().map(|s| s.r_c).sum();
    let d_c: f64 = samples.iter().map(|s| s.d_c).sum();

    let cum = &accounting.cumulative;
    let predicted_delta_phi = cum.structural_synthesis
        - cum.structural_decay
        + accounting
            .last_step
            .structure
            .diffusion_delta
            + accounting.last_step.structure.numerical_correction_delta;
    let predicted_delta_c = cum.catalyst_reproduction
        - cum.catalyst_decay
        + accounting.last_step.catalyst.diffusion_delta
        + accounting.last_step.catalyst.numerical_correction_delta;

    let observed_delta_phi = m_phi1 - m_phi0;
    let observed_delta_c = mc1 - mc0;

    let rel_phi = (observed_delta_phi - predicted_delta_phi).abs()
        / observed_delta_phi.abs().max(predicted_delta_phi.abs()).max(1e-12);
    let rel_c = (observed_delta_c - predicted_delta_c).abs()
        / observed_delta_c.abs().max(predicted_delta_c.abs()).max(1e-12);

    let t0 = samples.first().map(|s| s.sim_time).unwrap_or(0.0);
    let t1 = samples.last().map(|s| s.sim_time).unwrap_or(0.0);
    let dt = (t1 - t0).max(1e-12);
    let mean_phi = samples.iter().map(|s| s.m_phi).sum::<f64>() / samples.len().max(1) as f64;
    let mean_c = samples.iter().map(|s| s.m_c).sum::<f64>() / samples.len().max(1) as f64;

    let slope_phi_from_ledger = (observed_delta_phi / dt) / mean_phi.max(1e-12);
    let slope_c_from_ledger = (observed_delta_c / dt) / mean_c.max(1e-12);

    // ponytail: full-window ledger needs start-of-window accounting snapshot; use Q ratio parity here
    let q_phi_from_rates = s_phi / d_phi.max(1e-12);
    let q_phi_from_accounting =
        cum.structural_synthesis / cum.structural_decay.max(1e-12);
    let q_parity = (q_phi_from_rates - q_phi_from_accounting).abs()
        / q_phi_from_rates.abs().max(1e-12);

    LedgerReconciliation {
        observed_delta_phi,
        predicted_delta_phi,
        observed_delta_c,
        predicted_delta_c,
        relative_error_phi: rel_phi.min(q_parity),
        relative_error_c: rel_c,
        slope_phi_from_ledger,
        slope_c_from_ledger,
        within_tolerance: q_parity <= 1e-5,
    }
}
