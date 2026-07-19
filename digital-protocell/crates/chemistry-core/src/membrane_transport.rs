//! D-008 fixed-membrane, conservative soluble transport.

use crate::config::{SimParams, DX};
use crate::grid::Grid;
use crate::membrane_accounting::SpeciesTransportAccounting;
use crate::reactions::{catalyst_diffusivity, interface_weight};
use crate::surface_density::{reconstruct_gamma, theta_gamma};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportSpecies {
    Catalyst,
    Activated,
    Nutrient,
    Fuel,
    Waste,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FaceGeometry {
    pub interface: f64,
    pub membrane: f64,
}

pub fn face_geometry(phi_i: f64, phi_j: f64, membrane_i: f64, membrane_j: f64) -> FaceGeometry {
    FaceGeometry {
        interface: (0.5 * (interface_weight(phi_i) + interface_weight(phi_j))).clamp(0.0, 1.0),
        membrane: (0.5 * (membrane_i + membrane_j)).max(0.0),
    }
}

pub fn permeability(species: TransportSpecies, geometry: FaceGeometry, params: &SimParams) -> f64 {
    let beta = match species {
        TransportSpecies::Catalyst => params.beta_c,
        TransportSpecies::Activated => params.beta_a,
        TransportSpecies::Nutrient => params.beta_n,
        TransportSpecies::Fuel => params.beta_f,
        TransportSpecies::Waste => params.beta_w,
    };
    (-beta * geometry.membrane * geometry.interface).exp()
}

/// D-024 v7: interface-crossing faces attenuate by exp(−β·θΓ); co-phase faces pass through.
///
/// D-041 schema 3: for Activated only, multiply by frozen species constant `ρ_A`
/// on φ-crossing faces: Π_A = ρ_A exp(−β_A θ_S). Historical default is ρ_A = 1.
pub fn permeability_surface_occupancy(
    species: TransportSpecies,
    phi_i: f64,
    phi_j: f64,
    s_i: f64,
    s_j: f64,
    params: &SimParams,
) -> f64 {
    let i_inside = phi_i >= 0.5;
    let j_inside = phi_j >= 0.5;
    if i_inside == j_inside {
        return 1.0;
    }
    let beta = match species {
        TransportSpecies::Catalyst => params.beta_c,
        TransportSpecies::Activated => params.beta_a,
        TransportSpecies::Nutrient => params.beta_n,
        TransportSpecies::Fuel => params.beta_f,
        TransportSpecies::Waste => params.beta_w,
    };
    // ponytail: single-cell |∇H| proxy when full geometry is unavailable at transport faces.
    let delta_i = cell_delta_estimate(phi_i, params.delta_floor);
    let delta_j = cell_delta_estimate(phi_j, params.delta_floor);
    let gamma_i = reconstruct_gamma(s_i, delta_i, params.delta_floor);
    let gamma_j = reconstruct_gamma(s_j, delta_j, params.delta_floor);
    let theta = 0.5
        * (theta_gamma(gamma_i, params.gamma_reference)
            + theta_gamma(gamma_j, params.gamma_reference));
    let mature = (-beta * theta).exp();
    if species == TransportSpecies::Activated
        && params.transport_schema_version == crate::config::TRANSPORT_SCHEMA_VERSION_V3
    {
        params.rho_a.max(0.0) * mature
    } else {
        mature
    }
}

/// Mature-membrane A permeability at occupancy θ (schema-independent factor exp(−β_A θ)).
#[inline]
pub fn mature_a_permeability(theta: f64, beta_a: f64) -> f64 {
    (-beta_a * theta.max(0.0)).exp()
}

/// Structural A retention factor on a φ-crossing face under schema 3.
#[inline]
pub fn structural_a_retention_factor(params: &SimParams, theta: f64) -> f64 {
    if params.transport_schema_version != crate::config::TRANSPORT_SCHEMA_VERSION_V3 {
        return 1.0;
    }
    params.rho_a.max(0.0) * mature_a_permeability(theta, params.beta_a)
}

#[inline]
fn cell_delta_estimate(phi: f64, delta_floor: f64) -> f64 {
    let p = phi.clamp(0.0, 1.0);
    let dh_dphi = 6.0 * p * (1.0 - p);
    (dh_dphi / DX).max(delta_floor)
}

fn base_diffusivity(species: TransportSpecies, phi: f64, params: &SimParams) -> f64 {
    match species {
        TransportSpecies::Catalyst => catalyst_diffusivity(phi, params),
        TransportSpecies::Activated => params.d_a,
        TransportSpecies::Nutrient => params.d_n,
        TransportSpecies::Fuel => params.d_f,
        TransportSpecies::Waste => params.d_w,
    }
}

pub fn face_diffusivity(
    species: TransportSpecies,
    phi_i: f64,
    phi_j: f64,
    membrane_i: f64,
    membrane_j: f64,
    params: &SimParams,
) -> f64 {
    let geometry = face_geometry(phi_i, phi_j, membrane_i, membrane_j);
    let base =
        0.5 * (base_diffusivity(species, phi_i, params) + base_diffusivity(species, phi_j, params));
    let perm = if params.equation_version.is_surface_density() {
        permeability_surface_occupancy(species, phi_i, phi_j, membrane_i, membrane_j, params)
    } else {
        permeability(species, geometry, params)
    };
    base * perm
}

/// Signed flux from cell i to cell j across one face.
pub fn face_flux(
    species: TransportSpecies,
    concentration_i: f64,
    concentration_j: f64,
    phi_i: f64,
    phi_j: f64,
    membrane_i: f64,
    membrane_j: f64,
    params: &SimParams,
) -> f64 {
    face_diffusivity(species, phi_i, phi_j, membrane_i, membrane_j, params)
        * (concentration_i - concentration_j)
        / (DX * DX)
}

/// No-flux dish transport. Each +x/+y face is processed once and contributes
/// equal and opposite rates to its two cells.
pub fn transport_field(
    grid: &Grid,
    species: TransportSpecies,
    field: &[f64],
    phi: &[f64],
    membrane: &[f64],
    params: &SimParams,
    out_rate: &mut [f64],
) -> SpeciesTransportAccounting {
    let size = grid.width * grid.height;
    assert_eq!(field.len(), size);
    assert_eq!(phi.len(), size);
    assert_eq!(membrane.len(), size);
    assert_eq!(out_rate.len(), size);
    out_rate.fill(0.0);

    let mut absolute_crossed_face_flux = 0.0;
    let mut interior_net_flux_rate = 0.0;
    for j in 0..grid.height {
        for i in 0..grid.width {
            let idx = Grid::index(grid.width, i, j);
            if !grid.in_dish(idx) {
                continue;
            }
            if i + 1 < grid.width {
                let neighbor = Grid::index(grid.width, i + 1, j);
                if grid.in_dish(neighbor) {
                    apply_face(
                        species,
                        idx,
                        neighbor,
                        field,
                        phi,
                        membrane,
                        params,
                        out_rate,
                        &mut absolute_crossed_face_flux,
                        &mut interior_net_flux_rate,
                    );
                }
            }
            if j + 1 < grid.height {
                let neighbor = Grid::index(grid.width, i, j + 1);
                if grid.in_dish(neighbor) {
                    apply_face(
                        species,
                        idx,
                        neighbor,
                        field,
                        phi,
                        membrane,
                        params,
                        out_rate,
                        &mut absolute_crossed_face_flux,
                        &mut interior_net_flux_rate,
                    );
                }
            }
        }
    }

    let net_change_rate = grid
        .dish_mask
        .iter()
        .zip(out_rate.iter())
        .filter(|(inside, _)| **inside)
        .map(|(_, rate)| *rate)
        .sum();
    SpeciesTransportAccounting {
        net_change_rate,
        absolute_crossed_face_flux,
        interior_net_flux_rate,
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_face(
    species: TransportSpecies,
    i: usize,
    j: usize,
    field: &[f64],
    phi: &[f64],
    membrane: &[f64],
    params: &SimParams,
    out_rate: &mut [f64],
    absolute_crossed_face_flux: &mut f64,
    interior_net_flux_rate: &mut f64,
) {
    let flux = face_flux(
        species,
        field[i],
        field[j],
        phi[i],
        phi[j],
        membrane[i],
        membrane[j],
        params,
    );
    out_rate[i] -= flux;
    out_rate[j] += flux;
    *absolute_crossed_face_flux += flux.abs();
    let i_inside = phi[i] >= 0.5;
    let j_inside = phi[j] >= 0.5;
    if i_inside != j_inside {
        *interior_net_flux_rate += if i_inside { -flux } else { flux };
    }
}
