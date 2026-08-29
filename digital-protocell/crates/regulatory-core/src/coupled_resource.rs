//! DC-DEV-020-M1-R4: opt-in finite-resource coupled activation.
//!
//! The V1 finite spatial resource boundary remains the authority for exposure,
//! permeability, inventory, and delivery. This adapter applies the new
//! versioned law only after V1 returns its same-step delivery ledger:
//! newly delivered paired N/F becomes A+W, while unmatched material remains
//! internal N/F. Pre-existing internal N/F is never selected by this boundary.

use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_transport::TransportParams;
use serde::{Deserialize, Serialize};

use crate::spatial_resource::{FiniteSpatialResourceRegionV1, SpatialResourceStepLedgerV1};

pub const COUPLED_FINITE_SPATIAL_RESOURCE_SCHEMA_V1: &str =
    "FINITE_SPATIAL_RESOURCE_COUPLED_ACTIVATION_V1";
pub const COUPLED_SPATIAL_RESOURCE_STEP_LEDGER_SCHEMA_V1: &str =
    "dcdev020m1r4_coupled_spatial_resource_step_ledger_v1";

/// Versioned opt-in wrapper around the unchanged V1 finite resource region.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoupledFiniteSpatialResourceRegionV1 {
    pub schema: String,
    pub region: FiniteSpatialResourceRegionV1,
}

impl CoupledFiniteSpatialResourceRegionV1 {
    pub fn new(center: [f64; 2], radius: f64, n_mass: f64, f_mass: f64) -> Self {
        Self {
            schema: COUPLED_FINITE_SPATIAL_RESOURCE_SCHEMA_V1.to_string(),
            region: FiniteSpatialResourceRegionV1::new(center, radius, n_mass, f_mass),
        }
    }

    /// Construct the R4 adapter with a finite inventory whose boundary
    /// concentrations are fixed by an already-qualified world contract.
    /// This changes capacity only; V1 exposure, permeability, and uptake are
    /// still executed by the wrapped region.
    pub fn new_with_boundary_concentrations(
        center: [f64; 2],
        radius: f64,
        n_mass: f64,
        f_mass: f64,
        boundary_n_concentration: f64,
        boundary_f_concentration: f64,
    ) -> Self {
        let mut adapter = Self::new(center, radius, n_mass, f_mass);
        adapter.region.boundary_n_concentration = boundary_n_concentration.max(0.0);
        adapter.region.boundary_f_concentration = boundary_f_concentration.max(0.0);
        adapter
    }

    /// Run the exact V1 transport calculation, then transform only this step's
    /// newly delivered paired mass into the existing A+W material fields.
    pub fn uptake(
        &mut self,
        mesh: &mut MaterialMesh,
        transport: &TransportParams,
        dt: f64,
    ) -> CoupledSpatialResourceStepLedgerV1 {
        let v1 = self.region.uptake(mesh, transport, dt);
        apply_coupled_delivery(mesh, v1)
    }

    pub fn total_mass(&self) -> f64 {
        self.region.total_mass()
    }
}

/// Apply the R4 same-step paired-delivery law to an already-executed V1
/// ledger. This lets R5 reuse the exact R4 transformation without duplicating
/// transport semantics.
pub fn apply_coupled_delivery(
    mesh: &mut MaterialMesh,
    v1: SpatialResourceStepLedgerV1,
) -> CoupledSpatialResourceStepLedgerV1 {
    let area = mesh.area().max(1e-6);
    let paired = v1.n_delivered.min(v1.f_delivered).max(0.0);
    let n_unpaired = v1.n_delivered - paired;
    let f_unpaired = v1.f_delivered - paired;
    let paired_concentration = paired / area;

    mesh.interior.n -= paired_concentration;
    mesh.interior.f -= paired_concentration;
    mesh.interior.a += paired_concentration;
    mesh.interior.w += paired_concentration;

    let conservation_residual = (v1.n_world_loss - (n_unpaired + paired)).abs()
        + (v1.f_world_loss - (f_unpaired + paired)).abs()
        + (paired - v1.n_delivered.min(v1.f_delivered)).abs();

    CoupledSpatialResourceStepLedgerV1 {
        schema: COUPLED_SPATIAL_RESOURCE_STEP_LEDGER_SCHEMA_V1.to_string(),
        exposed_edges: v1.exposed_edges,
        n_world_loss: v1.n_world_loss,
        f_world_loss: v1.f_world_loss,
        n_delivered: v1.n_delivered,
        f_delivered: v1.f_delivered,
        paired_activated: paired,
        n_deposited_unpaired: n_unpaired,
        f_deposited_unpaired: f_unpaired,
        a_produced_coupled: paired,
        w_produced_coupled: paired,
        conservation_residual: conservation_residual + v1.conservation_error,
        v1_ledger: v1,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoupledSpatialResourceStepLedgerV1 {
    pub schema: String,
    pub exposed_edges: usize,
    pub n_world_loss: f64,
    pub f_world_loss: f64,
    pub n_delivered: f64,
    pub f_delivered: f64,
    pub paired_activated: f64,
    pub n_deposited_unpaired: f64,
    pub f_deposited_unpaired: f64,
    pub a_produced_coupled: f64,
    pub w_produced_coupled: f64,
    pub conservation_residual: f64,
    pub v1_ledger: SpatialResourceStepLedgerV1,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chemistry_core::material_mesh::{LumpedChem, DEFAULT_RHO_S};

    const CENTER: [f64; 2] = [4.8, 0.0];
    const RADIUS: f64 = 1.5;
    const N_MASS: f64 = 3.0;
    const F_MASS: f64 = 3.0;
    const DT: f64 = 0.02;

    fn mesh() -> MaterialMesh {
        MaterialMesh::seed_regular(
            24,
            5.0,
            0.0,
            0.0,
            DEFAULT_RHO_S,
            0.7,
            LumpedChem::default(),
            LumpedChem::default(),
            5.0,
        )
    }

    fn transport() -> TransportParams {
        TransportParams::default()
    }

    #[test]
    fn coupled_adapter_preserves_v1_transport_replay() {
        let mut coupled_mesh = mesh();
        let mut coupled = CoupledFiniteSpatialResourceRegionV1::new(CENTER, RADIUS, N_MASS, F_MASS);

        for _ in 0..120 {
            let mut v1_mesh = coupled_mesh.clone();
            let mut v1_region = coupled.region.clone();
            let v1 = v1_region.uptake(&mut v1_mesh, &transport(), DT);
            let led = coupled.uptake(&mut coupled_mesh, &transport(), DT);
            assert_eq!(v1.exposed_edges, led.exposed_edges);
            assert_eq!(v1.n_world_loss, led.n_world_loss);
            assert_eq!(v1.f_world_loss, led.f_world_loss);
            assert_eq!(v1.n_delivered, led.n_delivered);
            assert_eq!(v1.f_delivered, led.f_delivered);
            assert_eq!(v1.conservation_error, led.v1_ledger.conservation_error);
            assert_eq!(v1_region, coupled.region);
        }
    }

    #[test]
    fn same_step_pairing_and_controls_are_fail_closed() {
        let mut body = mesh();
        let mut region = CoupledFiniteSpatialResourceRegionV1::new(CENTER, RADIUS, N_MASS, F_MASS);
        let before = body.interior;
        let led = region.uptake(&mut body, &transport(), DT);
        assert!(led.paired_activated > 0.0);
        assert_eq!(led.a_produced_coupled, led.paired_activated);
        assert_eq!(led.w_produced_coupled, led.paired_activated);
        assert_eq!(
            led.n_deposited_unpaired + led.paired_activated,
            led.n_delivered
        );
        assert_eq!(
            led.f_deposited_unpaired + led.paired_activated,
            led.f_delivered
        );
        assert!(led.conservation_residual <= 1e-12);
        let area = body.area();
        assert!((body.interior.a - before.a - led.paired_activated / area).abs() <= 1e-12);

        for (n_mass, f_mass, center, expected_a) in [
            (N_MASS, F_MASS, [30.0, 30.0], 0.0),
            (N_MASS, 0.0, CENTER, 0.0),
            (0.0, F_MASS, CENTER, 0.0),
            (0.0, 0.0, CENTER, 0.0),
        ] {
            let mut control_mesh = mesh();
            let before_a = control_mesh.interior.a;
            let mut control =
                CoupledFiniteSpatialResourceRegionV1::new(center, RADIUS, n_mass, f_mass);
            let control_led = control.uptake(&mut control_mesh, &transport(), DT);
            assert_eq!(control_led.a_produced_coupled, expected_a);
            assert_eq!(control_mesh.interior.a, before_a);
        }

        let mut ruptured_mesh = mesh();
        for edge in &mut ruptured_mesh.edges {
            edge.ruptured = true;
        }
        let mut ruptured =
            CoupledFiniteSpatialResourceRegionV1::new(CENTER, RADIUS, N_MASS, F_MASS);
        let rupture_led = ruptured.uptake(&mut ruptured_mesh, &transport(), DT);
        assert_eq!(rupture_led.n_delivered, 0.0);
        assert_eq!(rupture_led.f_delivered, 0.0);
        assert_eq!(rupture_led.paired_activated, 0.0);
    }

    #[test]
    fn preexisting_internal_nf_is_not_selected_by_boundary_pairing() {
        let mut baseline_mesh = mesh();
        baseline_mesh.interior.n = 0.001;
        baseline_mesh.interior.f = 0.001;
        let mut coupled_mesh = baseline_mesh.clone();
        let mut baseline = FiniteSpatialResourceRegionV1::new(CENTER, RADIUS, N_MASS, F_MASS);
        let mut coupled = CoupledFiniteSpatialResourceRegionV1::new(CENTER, RADIUS, N_MASS, F_MASS);
        let v1 = baseline.uptake(&mut baseline_mesh, &transport(), DT);
        let led = coupled.uptake(&mut coupled_mesh, &transport(), DT);
        let area = coupled_mesh.area();
        assert!(
            (coupled_mesh.interior.n - (baseline_mesh.interior.n - led.paired_activated / area))
                .abs()
                <= 1e-12
        );
        assert!(
            (coupled_mesh.interior.f - (baseline_mesh.interior.f - led.paired_activated / area))
                .abs()
                <= 1e-12
        );
        assert_eq!(v1.n_delivered, led.n_delivered);
        assert_eq!(v1.f_delivered, led.f_delivered);
    }
}
