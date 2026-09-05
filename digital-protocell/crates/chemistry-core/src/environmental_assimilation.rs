//! Opt-in finite environmental assimilation material-flow composition.
//!
//! This is not part of the frozen production selector. It provides a real
//! organism-owned material boundary for goal-mode runtime experiments:
//! finite environmental N/F -> retained assimilation substrate -> existing
//! activated material pool -> existing structural-growth/fission gates.
//!
//! No new rate, gain, threshold, timer, or fission criterion is introduced.
//! The conversion uses the existing activation rate/catalyst term and the
//! existing structural yield and strain-local incorporation law.

use crate::material_mesh::MaterialMesh;
use crate::mesh_reactions::{q_catalyst, ReactionParams};
use serde::{Deserialize, Serialize};

pub const EQUATION_VERSION_ENVIRONMENTAL_ASSIMILATION: &str =
    "goal_material_flow_environmental_assimilation_v1";
pub const FIELD_SCHEMA_ENVIRONMENTAL_ASSIMILATION: &str =
    "mesh_interior_environmental_assimilation_n_f_v1";

/// Observer/accounting ledger for one assimilation and growth composition
/// step. Amounts are absolute material units, not concentrations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssimilationLedger {
    pub n_delivered: f64,
    pub f_delivered: f64,
    pub n_processed: f64,
    pub f_processed: f64,
    pub assimilation_a_produced: f64,
    pub w_from_processing: f64,
    pub closure_residual: f64,
}

/// Add finite environmental delivery to the organism-owned assimilation
/// compartment. The caller owns the world debit and passes exact delivered
/// amounts; this function owns only the organism-side material state.
pub fn receive(mesh: &mut MaterialMesh, n: f64, f: f64) {
    if !mesh.can_advance_physics() {
        return;
    }
    mesh.interior.assimilation_n += n.max(0.0) / mesh.area().max(1e-6);
    mesh.interior.assimilation_f += f.max(0.0) / mesh.area().max(1e-6);
}

/// Process the retained N/F substrate with the existing activation law. The
/// resulting activated material enters the existing physical A pool, which is
/// the same pool consumed by the accepted reaction, growth, and fission laws.
pub fn process(mesh: &mut MaterialMesh, react: &ReactionParams, dt: f64) -> AssimilationLedger {
    let mut led = AssimilationLedger::default();
    if !mesh.can_advance_physics() {
        return led;
    }
    let area = mesh.area().max(1e-6);
    let n0 = mesh.interior.assimilation_n.max(0.0) * area;
    let f0 = mesh.interior.assimilation_f.max(0.0) * area;
    let qc = q_catalyst(mesh.interior.c, react.q_c);
    let gh = if react.composition.enable {
        let z = crate::catalyst_composition::composition_z(mesh.interior.c_h, mesh.interior.c_b);
        crate::catalyst_composition::g_harvest(z, react.composition.sigma)
    } else if react.autocatalytic.enable {
        crate::autocatalytic_nodes::node_activation_gain(mesh, &react.autocatalytic, react.q_c)
    } else if react.network.enable {
        crate::template_network_expression::network_activation_gain(mesh, &react.network, react.q_c)
    } else if react.template.enable {
        crate::template_motifs::template_activity_gains(mesh, &react.template).0
    } else if mesh.finite_allocation.is_some() {
        let processing = crate::d096_allocation::function_gain(mesh, 0);
        let activation = crate::d096_allocation::function_gain(mesh, 1);
        (processing * activation).sqrt()
    } else {
        1.0
    };
    let extent = react.k_act.max(0.0)
        * qc
        * gh
        * mesh.interior.assimilation_n.max(0.0)
        * mesh.interior.assimilation_f.max(0.0)
        * dt.max(0.0)
        * area;
    let taken = extent.min(n0).min(f0).max(0.0);
    mesh.interior.assimilation_n = ((n0 - taken) / area).max(0.0);
    mesh.interior.assimilation_f = ((f0 - taken) / area).max(0.0);
    mesh.interior.a += taken / area;
    // The existing N+F activation contract produces one waste equivalent.
    mesh.interior.w += taken / area;
    led.n_processed = taken;
    led.f_processed = taken;
    led.assimilation_a_produced = taken;
    led.w_from_processing = taken;
    led.closure_residual = (n0 + f0 - mesh.interior.assimilation_n * area
        - mesh.interior.assimilation_f * area
        - 2.0 * taken)
        .abs();
    led
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh_population::MeshPopulation;

    #[test]
    fn finite_substrate_processing_closes_n_f_transfer() {
        let mut mesh = MeshPopulation::seed_one(8.0, 1, 0.0).individuals.remove(0).mesh;
        let delivered_n = 1.0;
        let delivered_f = 1.0;
        receive(&mut mesh, delivered_n, delivered_f);
        let ledger = process(&mut mesh, &ReactionParams::conservative_v3(), 0.02);
        assert!(ledger.n_processed > 0.0);
        assert!((ledger.n_processed - ledger.f_processed).abs() <= 1e-12);
        assert!((ledger.n_processed + mesh.interior.assimilation_n * mesh.area()
            - delivered_n)
            .abs()
            <= 1e-12);
        assert!((ledger.f_processed + mesh.interior.assimilation_f * mesh.area()
            - delivered_f)
            .abs()
            <= 1e-12);
        assert!(ledger.closure_residual <= 1e-12);
        assert!((ledger.assimilation_a_produced - ledger.w_from_processing).abs() <= 1e-12);
    }
}
