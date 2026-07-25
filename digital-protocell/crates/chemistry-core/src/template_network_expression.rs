//! Network catalytic expression: channel-specific bases from free + bound catalyst.

use crate::material_mesh::MaterialMesh;
use crate::mesh_reactions::q_catalyst;
use crate::template_network::{c_free, NetworkParams, RHO_NETWORK};
use crate::template_network_binding::sum_channel_masses;

const EPS: f64 = 1e-15;

#[derive(Debug, Clone, Copy, Default)]
pub struct NetworkCatalyticBases {
    pub c_activation: f64,
    pub c_storage: f64,
    pub c_release: f64,
    pub c_building: f64,
}

/// C_channel = C_free + ρ K_channel (concentrations).
pub fn network_catalytic_bases(mesh: &MaterialMesh, p: &NetworkParams) -> NetworkCatalyticBases {
    if !p.enable {
        let c = mesh.interior.c.max(0.0);
        return NetworkCatalyticBases {
            c_activation: c,
            c_storage: c,
            c_release: c,
            c_building: c,
        };
    }
    let area = mesh.area().max(EPS);
    let cf = c_free(mesh);
    let rho = if p.rho > 0.0 { p.rho } else { RHO_NETWORK };
    let (hh, hb, bh, bb) = sum_channel_masses(mesh);
    NetworkCatalyticBases {
        c_activation: cf + rho * hh / area,
        c_storage: cf + rho * hb / area,
        c_release: cf + rho * bh / area,
        c_building: cf + rho * bb / area,
    }
}

/// Effective catalyst activity for a channel relative to baseline q(C_total).
/// Returns a multiplicative gain so stoichiometry stays frozen.
pub fn channel_gain(mesh: &MaterialMesh, p: &NetworkParams, q_c: f64, channel: char) -> f64 {
    if !p.enable {
        return 1.0;
    }
    let c_tot = mesh.interior.c.max(0.0);
    let q_base = q_catalyst(c_tot, q_c).max(EPS);
    let bases = network_catalytic_bases(mesh, p);
    let c_ch = match channel {
        'A' => bases.c_activation,
        'S' => bases.c_storage,
        'R' => bases.c_release,
        'B' => bases.c_building,
        _ => c_tot,
    };
    (q_catalyst(c_ch, q_c) / q_base).max(0.0)
}

/// Activation (harvest) gain — for N+F→A.
pub fn network_activation_gain(mesh: &MaterialMesh, p: &NetworkParams, q_c: f64) -> f64 {
    channel_gain(mesh, p, q_c, 'A')
}

/// Building / repair / growth gain.
pub fn network_building_gain(mesh: &MaterialMesh, p: &NetworkParams, q_c: f64) -> f64 {
    channel_gain(mesh, p, q_c, 'B')
}

/// Storage gain — for A→R.
pub fn network_storage_gain(mesh: &MaterialMesh, p: &NetworkParams, q_c: f64) -> f64 {
    channel_gain(mesh, p, q_c, 'S')
}

/// Release gain — for R→A.
pub fn network_release_gain(mesh: &MaterialMesh, p: &NetworkParams, q_c: f64) -> f64 {
    channel_gain(mesh, p, q_c, 'R')
}
