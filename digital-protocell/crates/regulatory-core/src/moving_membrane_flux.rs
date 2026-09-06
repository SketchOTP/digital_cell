//! Opt-in finite transport across the actual moving polygonal membrane.
//!
//! `SpatialMaterialFieldV1` and `SharedFiniteExtracellularMediumV1` form
//! requests from membrane-edge samples.  This substrate instead treats the
//! membrane as the transfer boundary: each edge contributes only the length
//! of its intersection with the finite extracellular control volume.  The
//! existing permeability, `k_flux`, concentration-jump law, and timestep are
//! unchanged.  N/F remain world-owned until the simultaneous debit/credit
//! pass completes.

use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_transport::{
    permeability, transport_step, TransportLedger, TransportParams,
};
use serde::{Deserialize, Serialize};

pub const MOVING_MEMBRANE_FINITE_FLUX_SCHEMA_V1: &str =
    "digital_cell_moving_membrane_finite_flux_v1";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MovingMembraneDeliveryV1 {
    pub organism_index: usize,
    pub interfaced_edges: usize,
    pub intersected_membrane_length: f64,
    pub n_requested: f64,
    pub f_requested: f64,
    pub n_delivered: f64,
    pub f_delivered: f64,
    pub n_world_loss: f64,
    pub f_world_loss: f64,
    pub allocation_scale: f64,
    pub conservation_error: f64,
    #[serde(default)]
    pub nonfeeding_transport: TransportLedger,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MovingMembraneFiniteFluxV1 {
    pub schema: String,
    pub center: [f64; 2],
    pub radius: f64,
    pub material_volume: f64,
    pub initial_n_mass: f64,
    pub initial_f_mass: f64,
    pub n_mass: f64,
    pub f_mass: f64,
    pub step: u64,
    pub transfer_enabled: bool,
    pub ledger_n_taken: f64,
    pub ledger_f_taken: f64,
}

#[derive(Debug, Clone, Copy)]
struct InterfaceRequest {
    organism_index: usize,
    membrane_length: f64,
    n: f64,
    f: f64,
}

impl MovingMembraneFiniteFluxV1 {
    pub fn new(center: [f64; 2], radius: f64, n_mass: f64, f_mass: f64) -> Result<Self, String> {
        if !radius.is_finite() || radius <= 0.0 {
            return Err("moving-membrane medium radius must be positive".into());
        }
        let n_mass = n_mass.max(0.0);
        let f_mass = f_mass.max(0.0);
        let material_volume = std::f64::consts::PI * radius * radius;
        Ok(Self {
            schema: MOVING_MEMBRANE_FINITE_FLUX_SCHEMA_V1.to_string(),
            center,
            radius,
            material_volume,
            initial_n_mass: n_mass,
            initial_f_mass: f_mass,
            n_mass,
            f_mass,
            step: 0,
            transfer_enabled: true,
            ledger_n_taken: 0.0,
            ledger_f_taken: 0.0,
        })
    }

    pub fn total_n_mass(&self) -> f64 {
        self.n_mass
    }

    pub fn total_f_mass(&self) -> f64 {
        self.f_mass
    }

    pub fn boundary_concentrations(&self) -> (f64, f64) {
        (
            self.n_mass / self.material_volume,
            self.f_mass / self.material_volume,
        )
    }

    /// Execute one simultaneous moving-interface exchange.
    ///
    /// The nonfeeding transport pass is the existing zero-N/F bath path.  It
    /// preserves W export and C/A leakage without creating a second positive
    /// N/F source.  Positive N/F requests are then computed from the same
    /// pre-step medium concentration and the exact membrane/control-volume
    /// intersection length.  One species-wise scale is applied after all
    /// requests are known, making finite depletion independent of iteration
    /// order.
    pub fn exchange(
        &mut self,
        meshes: &mut [MaterialMesh],
        transport: &TransportParams,
        dt: f64,
    ) -> Vec<MovingMembraneDeliveryV1> {
        let nonfeeding: Vec<TransportLedger> = meshes
            .iter_mut()
            .enumerate()
            .map(|(_organism_index, mesh)| {
                let exterior = mesh.exterior;
                mesh.exterior.n = 0.0;
                mesh.exterior.f = 0.0;
                let ledger = transport_step(mesh, transport, dt);
                mesh.exterior = exterior;
                ledger
            })
            .collect();

        let (boundary_n, boundary_f) = self.boundary_concentrations();
        let mut requests = Vec::new();
        let mut deliveries = vec![MovingMembraneDeliveryV1::default(); meshes.len()];
        for (organism_index, mesh) in meshes.iter().enumerate() {
            deliveries[organism_index].organism_index = organism_index;
            deliveries[organism_index].nonfeeding_transport =
                nonfeeding.get(organism_index).cloned().unwrap_or_default();
            if !mesh.can_advance_physics() || dt <= 0.0 {
                continue;
            }
            for edge in 0..mesh.n() {
                if mesh.edges[edge].ruptured {
                    continue;
                }
                let a = mesh.vertices[edge];
                let b = mesh.vertices[(edge + 1) % mesh.n()];
                let membrane_length =
                    segment_circle_intersection_length(a, b, self.center, self.radius);
                if membrane_length <= 1e-15 {
                    continue;
                }
                let theta = mesh.occupancy(edge);
                let n = boundary_source_request(
                    boundary_n,
                    permeability(theta, "N"),
                    transport.k_flux,
                    membrane_length,
                    dt,
                );
                let f = boundary_source_request(
                    boundary_f,
                    permeability(theta, "F"),
                    transport.k_flux,
                    membrane_length,
                    dt,
                );
                deliveries[organism_index].interfaced_edges += 1;
                deliveries[organism_index].intersected_membrane_length += membrane_length;
                requests.push(InterfaceRequest {
                    organism_index,
                    membrane_length,
                    n,
                    f,
                });
            }
        }

        let total_n_positive: f64 = requests.iter().map(|request| request.n.max(0.0)).sum();
        let total_f_positive: f64 = requests.iter().map(|request| request.f.max(0.0)).sum();
        let n_scale = if total_n_positive > 0.0 {
            (self.n_mass / total_n_positive).min(1.0)
        } else {
            1.0
        };
        let f_scale = if total_f_positive > 0.0 {
            (self.f_mass / total_f_positive).min(1.0)
        } else {
            1.0
        };
        let positive_scale = if self.transfer_enabled {
            n_scale.min(f_scale).max(0.0)
        } else {
            0.0
        };

        for request in requests {
            let n = request.n * positive_scale;
            let f = request.f * positive_scale;
            if let Some(mesh) = meshes.get_mut(request.organism_index) {
                let area = mesh.area();
                if area.is_finite() && area > 0.0 {
                    mesh.interior.n += n / area;
                    mesh.interior.f += f / area;
                }
            }
            let delivery = &mut deliveries[request.organism_index];
            delivery.n_requested += request.n.max(0.0);
            delivery.f_requested += request.f.max(0.0);
            delivery.n_delivered += n.max(0.0);
            delivery.f_delivered += f.max(0.0);
            // Keep the field available in the request for audit/debugging and
            // assert that it remains a physical nonnegative interface measure.
            debug_assert!(request.membrane_length.is_finite() && request.membrane_length >= 0.0);
        }

        let delivered_n: f64 = deliveries.iter().map(|delivery| delivery.n_delivered).sum();
        let delivered_f: f64 = deliveries.iter().map(|delivery| delivery.f_delivered).sum();
        self.n_mass = (self.n_mass - delivered_n).max(0.0);
        self.f_mass = (self.f_mass - delivered_f).max(0.0);
        self.ledger_n_taken += delivered_n;
        self.ledger_f_taken += delivered_f;
        for delivery in &mut deliveries {
            delivery.n_world_loss = delivery.n_delivered;
            delivery.f_world_loss = delivery.f_delivered;
            delivery.allocation_scale = if delivery.n_requested > 0.0 {
                (delivery.n_delivered / delivery.n_requested).min(if delivery.f_requested > 0.0 {
                    delivery.f_delivered / delivery.f_requested
                } else {
                    1.0
                })
            } else {
                1.0
            };
            delivery.conservation_error = (delivery.n_world_loss - delivery.n_delivered).abs()
                + (delivery.f_world_loss - delivery.f_delivered).abs();
        }
        self.step = self.step.saturating_add(1);
        deliveries
    }
}

fn boundary_source_request(
    boundary: f64,
    permeability: f64,
    k_flux: f64,
    segment_length: f64,
    dt: f64,
) -> f64 {
    (k_flux * permeability * boundary.max(0.0) * segment_length * dt).max(0.0)
}

/// Return the physical length of the portion of a segment inside a circular
/// finite control volume.  This is the moving-interface geometry contract:
/// midpoint membership is not used, and a partially intersecting edge gets
/// exactly its intersecting measure.
pub fn segment_circle_intersection_length(
    a: [f64; 2],
    b: [f64; 2],
    center: [f64; 2],
    radius: f64,
) -> f64 {
    if !radius.is_finite() || radius <= 0.0 {
        return 0.0;
    }
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let length = dx.hypot(dy);
    if !length.is_finite() || length <= 0.0 {
        return 0.0;
    }
    let ax = a[0] - center[0];
    let ay = a[1] - center[1];
    let qa = dx * dx + dy * dy;
    let qb = 2.0 * (ax * dx + ay * dy);
    let qc = ax * ax + ay * ay - radius * radius;
    let discriminant = qb * qb - 4.0 * qa * qc;
    if discriminant <= 0.0 {
        return if qc <= 0.0 { length } else { 0.0 };
    }
    let root = discriminant.sqrt();
    let lo = ((-qb - root) / (2.0 * qa)).max(0.0);
    let hi = ((-qb + root) / (2.0 * qa)).min(1.0);
    if hi <= lo {
        0.0
    } else {
        (hi - lo) * length
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chemistry_core::material_mesh::{
        LumpedChem, MaterialMesh, MeshContractVersion, DEFAULT_RHO_S,
    };

    fn mesh(center: [f64; 2]) -> MaterialMesh {
        let mut mesh = MaterialMesh::seed_regular(
            24,
            0.8,
            center[0],
            center[1],
            DEFAULT_RHO_S,
            0.7,
            LumpedChem {
                n: 0.0,
                f: 0.0,
                ..LumpedChem::default()
            },
            LumpedChem::default(),
            5.0,
        );
        mesh.contract_version = MeshContractVersion::MaturationCoupledV4;
        mesh
    }

    #[test]
    fn segment_intersection_uses_partial_measure_not_midpoint_membership() {
        let length = segment_circle_intersection_length([-2.0, 0.0], [2.0, 0.0], [0.0, 0.0], 1.0);
        assert!((length - 2.0).abs() < 1e-12);
    }

    #[test]
    fn uniform_boundary_matches_whole_membrane_first_step() {
        let radius = 3.0;
        let concentration = 2.0;
        let volume = std::f64::consts::PI * radius * radius;
        let mass = concentration * volume;
        let moving_mesh = mesh([0.0, 0.0]);
        let mut reference_mesh = moving_mesh.clone();
        reference_mesh.exterior.n = concentration;
        reference_mesh.exterior.f = concentration;
        let reference = transport_step(&mut reference_mesh, &TransportParams::default(), 0.02);
        let mut medium = MovingMembraneFiniteFluxV1::new([0.0, 0.0], radius, mass, mass).unwrap();
        let moving = medium.exchange(&mut [moving_mesh], &TransportParams::default(), 0.02);
        assert!((reference.n_in - moving[0].n_delivered).abs() < 1e-12);
        assert!((reference.f_in - moving[0].f_delivered).abs() < 1e-12);
    }

    #[test]
    fn finite_debit_credit_and_order_are_exact() {
        let mut first = MovingMembraneFiniteFluxV1::new([0.0, 0.0], 1.5, 0.01, 0.01).unwrap();
        let mut second = first.clone();
        let mut a = vec![mesh([0.0, 0.0]), mesh([0.0, 0.0])];
        let mut b = vec![a[1].clone(), a[0].clone()];
        let left = first.exchange(&mut a, &TransportParams::default(), 0.02);
        let right = second.exchange(&mut b, &TransportParams::default(), 0.02);
        assert!((first.n_mass - second.n_mass).abs() < 1e-12);
        assert!(
            (left.iter().map(|d| d.n_delivered).sum::<f64>()
                - right.iter().map(|d| d.n_delivered).sum::<f64>())
            .abs()
                < 1e-12
        );
        assert!((left[0].n_delivered - right[1].n_delivered).abs() < 1e-12);
        assert!(left.iter().all(|d| d.conservation_error == 0.0));
    }

    #[test]
    fn disabled_medium_has_no_positive_transfer() {
        let mut medium = MovingMembraneFiniteFluxV1::new([0.0, 0.0], 1.5, 10.0, 10.0).unwrap();
        medium.transfer_enabled = false;
        let mut bodies = vec![mesh([0.0, 0.0])];
        let before = bodies[0].interior;
        let deliveries = medium.exchange(&mut bodies, &TransportParams::default(), 0.02);
        assert_eq!(deliveries[0].n_delivered, 0.0);
        assert_eq!(deliveries[0].f_delivered, 0.0);
        assert_eq!(medium.n_mass, 10.0);
        assert_eq!(medium.f_mass, 10.0);
        assert!(bodies[0].interior.n <= before.n);
    }

    #[test]
    fn partial_interface_scales_by_intersected_membrane_length() {
        let mut medium = MovingMembraneFiniteFluxV1::new([0.75, 0.0], 0.55, 10.0, 10.0).unwrap();
        let mut bodies = vec![mesh([0.0, 0.0])];
        let deliveries = medium.exchange(&mut bodies, &TransportParams::default(), 0.02);
        assert!(deliveries[0].interfaced_edges > 0);
        assert!(deliveries[0].interfaced_edges < bodies[0].n());
        assert!(deliveries[0].intersected_membrane_length > 0.0);
        assert!(deliveries[0].intersected_membrane_length < bodies[0].perimeter());
        assert!(deliveries[0].n_delivered > 0.0);
    }

    #[test]
    fn short_horizon_uniform_parity_tracks_the_whole_membrane_law() {
        let radius = 1_000_000.0;
        let concentration = 2.0;
        let mass = concentration * std::f64::consts::PI * radius * radius;
        let mut moving_mesh = mesh([0.0, 0.0]);
        let mut reference_mesh = moving_mesh.clone();
        let mut medium = MovingMembraneFiniteFluxV1::new([0.0, 0.0], radius, mass, mass).unwrap();
        for _ in 0..16 {
            reference_mesh.exterior.n = concentration;
            reference_mesh.exterior.f = concentration;
            let _ = transport_step(&mut reference_mesh, &TransportParams::default(), 0.02);
            let _ = medium.exchange(
                std::slice::from_mut(&mut moving_mesh),
                &TransportParams::default(),
                0.02,
            );
        }
        assert!(
            (reference_mesh.interior.n - moving_mesh.interior.n).abs() < 1e-10,
            "N parity error: reference={} moving={} diff={}",
            reference_mesh.interior.n,
            moving_mesh.interior.n,
            reference_mesh.interior.n - moving_mesh.interior.n
        );
        assert!(
            (reference_mesh.interior.f - moving_mesh.interior.f).abs() < 1e-10,
            "F parity error: reference={} moving={} diff={}",
            reference_mesh.interior.f,
            moving_mesh.interior.f,
            reference_mesh.interior.f - moving_mesh.interior.f
        );
    }

    #[test]
    fn rotating_geometry_and_interface_rotates_without_changing_flux() {
        let center = [0.72, 0.11];
        let mut first_medium = MovingMembraneFiniteFluxV1::new(center, 0.9, 10.0, 10.0).unwrap();
        let mut second_medium =
            MovingMembraneFiniteFluxV1::new([-center[1], center[0]], 0.9, 10.0, 10.0).unwrap();
        let first_mesh = mesh([0.0, 0.0]);
        let mut second_mesh = first_mesh.clone();
        for point in &mut second_mesh.vertices {
            let [x, y] = *point;
            *point = [-y, x];
        }
        let first = first_medium.exchange(&mut [first_mesh], &TransportParams::default(), 0.02);
        let second = second_medium.exchange(&mut [second_mesh], &TransportParams::default(), 0.02);
        assert!((first[0].n_delivered - second[0].n_delivered).abs() < 1e-12);
        assert!((first[0].f_delivered - second[0].f_delivered).abs() < 1e-12);
        assert!(
            (first[0].intersected_membrane_length - second[0].intersected_membrane_length).abs()
                < 1e-12
        );
    }
}
