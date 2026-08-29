//! D-086 conservative transport between mesh boundary and Cartesian-style reservoirs.
//!
//! Permeability depends on bound-membrane occupancy θ = b / b_max.
//! Targets (D-079/D-083 lineage): C,A ≤0.05; N,F ∈[0.20,0.50]; W ≥0.70.

use crate::material_mesh::{MaterialMesh, MeshContractVersion};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TransportParams {
    pub k_flux: f64,
}

impl Default for TransportParams {
    fn default() -> Self {
        // Sized so perimeter influx can fund interior N+F→A against build demand
        // (Gate 3 failure mode was activation-starved A collapse).
        Self { k_flux: 1.1 }
    }
}

/// Occupancy → permeability for species class.
pub fn permeability(theta: f64, species: &str) -> f64 {
    let t = theta.clamp(0.0, 1.0);
    // Higher binding → lower C/A leak, moderate N/F, high W.
    match species {
        "C" | "A" => 0.05 * (1.0 - 0.9 * t),
        "N" | "F" => 0.20 + 0.30 * (1.0 - t),
        "W" => 0.70 + 0.25 * t,
        _ => 0.1,
    }
}

pub fn permeability_in_targets(theta: f64) -> bool {
    let pc = permeability(theta, "C");
    let pa = permeability(theta, "A");
    let pn = permeability(theta, "N");
    let pf = permeability(theta, "F");
    let pw = permeability(theta, "W");
    pc <= 0.05 + 1e-9
        && pa <= 0.05 + 1e-9
        && (0.20 - 1e-9..=0.50 + 1e-9).contains(&pn)
        && (0.20 - 1e-9..=0.50 + 1e-9).contains(&pf)
        && pw + 1e-9 >= 0.70
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransportLedger {
    pub n_in: f64,
    #[serde(default)]
    pub n_out: f64,
    pub f_in: f64,
    #[serde(default)]
    pub f_out: f64,
    pub w_out: f64,
    #[serde(default)]
    pub w_in: f64,
    pub c_leak: f64,
    #[serde(default)]
    pub c_in: f64,
    pub a_leak: f64,
    #[serde(default)]
    pub a_in: f64,
}

/// Mean occupancy across intact edges.
pub fn mean_occupancy(mesh: &MaterialMesh) -> f64 {
    let mut s = 0.0;
    let mut n = 0.0;
    for i in 0..mesh.n() {
        if mesh.edges[i].ruptured {
            continue;
        }
        s += mesh.occupancy(i);
        n += 1.0;
    }
    if n <= 0.0 {
        0.0
    } else {
        s / n
    }
}

/// Conservative exchange: interior ⇄ exterior across perimeter, using mean θ.
/// Ruptured edges increase leak (higher effective permeability).
pub fn transport_step(mesh: &mut MaterialMesh, p: &TransportParams, dt: f64) -> TransportLedger {
    match mesh.contract_version {
        MeshContractVersion::HistoricalV1 | MeshContractVersion::ConservativeV2 => {
            transport_step_historical(mesh, p, dt)
        }
        MeshContractVersion::GeometryConservativeV3 | MeshContractVersion::MaturationCoupledV4 => {
            transport_step_actual_area(mesh, p, dt)
        }
    }
}

/// Historical transport path.  Keep the floor-based concentration arithmetic
/// intact for the V1/V2 contracts; their serialized behavior is frozen.
fn transport_step_historical(
    mesh: &mut MaterialMesh,
    p: &TransportParams,
    dt: f64,
) -> TransportLedger {
    let mut led = TransportLedger::default();
    if !mesh.can_advance_physics() {
        return led;
    }
    let area = mesh.area().max(1e-6);
    let peri = mesh.perimeter().max(1e-6);
    let theta = mean_occupancy(mesh);
    let ruptured_frac =
        mesh.edges.iter().filter(|e| e.ruptured).count() as f64 / mesh.n().max(1) as f64;
    let leak_boost = 1.0 + 4.0 * ruptured_frac;

    let flux = |species: &str, c_in: f64, c_out: f64| -> f64 {
        let perm = permeability(theta, species) * leak_boost;
        p.k_flux * perm * (c_out - c_in) * peri * dt / area
    };

    // N, F enter from exterior reservoir.
    let dn = flux("N", mesh.interior.n, mesh.exterior.n);
    let df = flux("F", mesh.interior.f, mesh.exterior.f);
    mesh.interior.n = (mesh.interior.n + dn).max(0.0);
    mesh.interior.f = (mesh.interior.f + df).max(0.0);
    if dn > 0.0 {
        led.n_in += dn * area;
    }
    if df > 0.0 {
        led.f_in += df * area;
    }

    // W exits.
    let dw = flux("W", mesh.interior.w, mesh.exterior.w);
    mesh.interior.w = (mesh.interior.w + dw).max(0.0);
    if dw < 0.0 {
        led.w_out += (-dw) * area;
    }

    // C, A leak (should be small when bound membrane high).
    let c_before = mesh.interior.c.max(0.0);
    let dc = flux("C", mesh.interior.c, mesh.exterior.c);
    let da = flux("A", mesh.interior.a, mesh.exterior.a);
    if dc < 0.0 && c_before > 1e-15 {
        let frac = ((-dc) / c_before).clamp(0.0, 1.0);
        mesh.interior.tracer_c = (mesh.interior.tracer_c * (1.0 - frac)).max(0.0);
        // Compositional catalyst leaks proportionally (material, not ratio copy).
        let parts = mesh.interior.c_h.max(0.0) + mesh.interior.c_b.max(0.0);
        if parts > 1e-15 {
            mesh.interior.c_h = (mesh.interior.c_h * (1.0 - frac)).max(0.0);
            mesh.interior.c_b = (mesh.interior.c_b * (1.0 - frac)).max(0.0);
        }
    }
    mesh.interior.c = (mesh.interior.c + dc).max(0.0);
    if mesh.interior.c_h + mesh.interior.c_b > 1e-15 {
        // Keep total aligned with parts after leak; influx of C (rare) stays unlabeled scalar.
        if dc >= 0.0 {
            mesh.interior.c = mesh.interior.c_h + mesh.interior.c_b + dc;
            // Influx has no type — attribute proportionally to existing composition.
            let parts = mesh.interior.c_h.max(0.0) + mesh.interior.c_b.max(0.0);
            if parts > 1e-15 && dc > 0.0 {
                let ph = mesh.interior.c_h.max(0.0) / parts;
                mesh.interior.c_h += dc * ph;
                mesh.interior.c_b += dc * (1.0 - ph);
            }
        } else {
            mesh.interior.c = mesh.interior.c_h + mesh.interior.c_b;
        }
    }
    mesh.interior.a = (mesh.interior.a + da).max(0.0);
    if dc < 0.0 {
        led.c_leak += (-dc) * area;
    }
    if da < 0.0 {
        led.a_leak += (-da) * area;
    }

    led
}

/// Versioned material-conservative transport for the V3/V4 contracts.
///
/// The transport law is unchanged; only its representation is changed from a
/// concentration increment divided by a historical area floor to a signed
/// absolute amount applied against the actual positive physical area.  This
/// prevents a sub-floor body from exporting more material than it contains.
fn transport_step_actual_area(
    mesh: &mut MaterialMesh,
    p: &TransportParams,
    dt: f64,
) -> TransportLedger {
    let mut led = TransportLedger::default();
    if !mesh.can_advance_physics() {
        return led;
    }
    let actual_area = mesh.area();
    if !actual_area.is_finite() || actual_area <= 0.0 {
        return led;
    }
    let peri = mesh.perimeter().max(1e-6);
    let theta = mean_occupancy(mesh);
    let ruptured_frac =
        mesh.edges.iter().filter(|e| e.ruptured).count() as f64 / mesh.n().max(1) as f64;
    let leak_boost = 1.0 + 4.0 * ruptured_frac;

    let requested_amount = |species: &str, c_in: f64, c_out: f64| -> f64 {
        let perm = permeability(theta, species) * leak_boost;
        p.k_flux * perm * (c_out - c_in) * peri * dt
    };

    let apply_amount = |concentration: &mut f64, requested: f64| -> f64 {
        let amount_before = concentration.max(0.0) * actual_area;
        let applied = if requested < 0.0 {
            -(-requested).min(amount_before)
        } else {
            requested
        };
        let amount_after = (amount_before + applied).max(0.0);
        *concentration = amount_after / actual_area;
        applied
    };

    let n_before = mesh.interior.n;
    let f_before = mesh.interior.f;
    let w_before = mesh.interior.w;
    let dn_requested = requested_amount("N", n_before, mesh.exterior.n);
    let df_requested = requested_amount("F", f_before, mesh.exterior.f);
    let dw_requested = requested_amount("W", w_before, mesh.exterior.w);
    let dn = apply_amount(&mut mesh.interior.n, dn_requested);
    let df = apply_amount(&mut mesh.interior.f, df_requested);
    let dw = apply_amount(&mut mesh.interior.w, dw_requested);

    if dn >= 0.0 {
        led.n_in = dn;
    } else {
        led.n_out = -dn;
    }
    if df >= 0.0 {
        led.f_in = df;
    } else {
        led.f_out = -df;
    }
    if dw >= 0.0 {
        led.w_in = dw;
    } else {
        led.w_out = -dw;
    }

    let c_before = mesh.interior.c.max(0.0);
    let c_parts_before = mesh.interior.c_h.max(0.0) + mesh.interior.c_b.max(0.0);
    let dc_requested = requested_amount("C", c_before, mesh.exterior.c);
    let a_before = mesh.interior.a;
    let da_requested = requested_amount("A", a_before, mesh.exterior.a);
    let dc = apply_amount(&mut mesh.interior.c, dc_requested);
    let da = apply_amount(&mut mesh.interior.a, da_requested);

    if dc < 0.0 {
        led.c_leak = -dc;
        if c_before > 1e-15 {
            let fraction = (-dc / (c_before * actual_area)).clamp(0.0, 1.0);
            mesh.interior.tracer_c = (mesh.interior.tracer_c * (1.0 - fraction)).max(0.0);
            mesh.interior.c_h = (mesh.interior.c_h * (1.0 - fraction)).max(0.0);
            mesh.interior.c_b = (mesh.interior.c_b * (1.0 - fraction)).max(0.0);
        }
    } else if dc > 0.0 && c_parts_before > 1e-15 {
        let total_parts = c_parts_before * actual_area;
        let ph = mesh.interior.c_h.max(0.0) * actual_area / total_parts;
        mesh.interior.c_h += (dc / actual_area) * ph;
        mesh.interior.c_b += (dc / actual_area) * (1.0 - ph);
        led.c_in = dc;
    } else if dc > 0.0 {
        led.c_in = dc;
    }
    if da < 0.0 {
        led.a_leak = -da;
    } else {
        led.a_in = da;
    }

    led
}

/// Approximate retention over a window: final/initial for C and A.
pub fn retention(c0: f64, c1: f64) -> f64 {
    if c0 <= 1e-15 {
        1.0
    } else {
        (c1 / c0).clamp(0.0, 10.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material_mesh::{LumpedChem, MeshContractVersion};

    fn fixture(contract: MeshContractVersion, area: f64) -> MaterialMesh {
        let radius = (area / std::f64::consts::PI).sqrt();
        let mut mesh = MaterialMesh::seed_regular(
            12,
            radius,
            0.0,
            0.0,
            1.0,
            0.5,
            LumpedChem {
                n: 0.25,
                f: 0.25,
                a: 0.5,
                c: 1.0,
                w: 1.0,
                tracer_c: 0.4,
                c_h: 0.4,
                c_b: 0.6,
                ..Default::default()
            },
            LumpedChem {
                n: 0.0,
                f: 0.0,
                a: 0.0,
                c: 0.0,
                w: 0.0,
                ..Default::default()
            },
            0.0,
        );
        match contract {
            MeshContractVersion::HistoricalV1 => {}
            MeshContractVersion::ConservativeV2 => mesh.stamp_conservative_schema(),
            MeshContractVersion::GeometryConservativeV3 => {
                mesh.stamp_geometry_conservative_schema()
            }
            MeshContractVersion::MaturationCoupledV4 => mesh.stamp_maturation_coupled_schema(),
        }
        mesh
    }

    fn total_transport_amount(mesh: &MaterialMesh) -> f64 {
        let area = mesh.area();
        area * (mesh.interior.n + mesh.interior.f + mesh.interior.a + mesh.interior.c)
            + area * mesh.interior.w
    }

    fn ledger_net(ledger: &TransportLedger) -> f64 {
        ledger.n_in - ledger.n_out + ledger.f_in - ledger.f_out + ledger.w_in - ledger.w_out
            + ledger.c_in
            - ledger.c_leak
            + ledger.a_in
            - ledger.a_leak
    }

    #[test]
    fn v3_and_v4_sub_floor_transport_caps_outbound_material_and_closes() {
        for contract in [
            MeshContractVersion::GeometryConservativeV3,
            MeshContractVersion::MaturationCoupledV4,
        ] {
            let mut mesh = fixture(contract, 1.0e-12);
            let before = total_transport_amount(&mesh);
            let ledger = transport_step(&mut mesh, &TransportParams { k_flux: 1.1 }, 1.0);
            let after = total_transport_amount(&mesh);
            let residual = (after - before - ledger_net(&ledger)).abs();
            assert!(
                residual <= 1.0e-24,
                "contract={contract:?} residual={residual}"
            );
            assert!(mesh.interior.c >= 0.0);
            assert!(mesh.interior.a >= 0.0);
            assert!(mesh.interior.w >= 0.0);
            assert!(ledger.c_leak <= 1.0e-12 + 1.0e-24);
            assert!(ledger.w_out <= 1.0e-12 + 1.0e-24);
            assert!(ledger.c_leak > 0.0);
            assert!(ledger.w_out > 0.0);
            assert!((mesh.interior.tracer_c - 0.0).abs() <= 1.0e-12);
        }
    }

    #[test]
    fn v3_and_v4_sub_floor_transport_preserves_each_species_amount() {
        for area in [1.0e-7, 1.0e-9, 1.0e-12] {
            for contract in [
                MeshContractVersion::GeometryConservativeV3,
                MeshContractVersion::MaturationCoupledV4,
            ] {
                let mut mesh = fixture(contract, area);
                let actual_area = mesh.area();
                let before = [
                    mesh.interior.n * actual_area,
                    mesh.interior.f * actual_area,
                    mesh.interior.a * actual_area,
                    mesh.interior.c * actual_area,
                    mesh.interior.w * actual_area,
                ];
                let ledger = transport_step(&mut mesh, &TransportParams::default(), 0.02);
                let after = [
                    mesh.interior.n * actual_area,
                    mesh.interior.f * actual_area,
                    mesh.interior.a * actual_area,
                    mesh.interior.c * actual_area,
                    mesh.interior.w * actual_area,
                ];
                let expected = [
                    ledger.n_in - ledger.n_out,
                    ledger.f_in - ledger.f_out,
                    ledger.a_in - ledger.a_leak,
                    ledger.c_in - ledger.c_leak,
                    ledger.w_in - ledger.w_out,
                ];
                for i in 0..before.len() {
                    let residual = (after[i] - before[i] - expected[i]).abs();
                    assert!(
                        residual <= 1.0e-20,
                        "area={area} contract={contract:?} species={i} residual={residual}"
                    );
                }
            }
        }
    }

    #[test]
    fn v3_above_floor_matches_frozen_v2_transport() {
        let mut v2 = fixture(MeshContractVersion::ConservativeV2, 1.0);
        let mut v3 = v2.clone();
        v3.stamp_geometry_conservative_schema();
        let p = TransportParams::default();
        let v2_ledger = transport_step(&mut v2, &p, 0.02);
        let v3_ledger = transport_step(&mut v3, &p, 0.02);
        for (actual, expected) in [
            (v3.interior.n, v2.interior.n),
            (v3.interior.f, v2.interior.f),
            (v3.interior.w, v2.interior.w),
            (v3.interior.c, v2.interior.c),
            (v3.interior.a, v2.interior.a),
            (v3.interior.tracer_c, v2.interior.tracer_c),
            (v3.interior.c_h, v2.interior.c_h),
            (v3.interior.c_b, v2.interior.c_b),
            (v3_ledger.n_in, v2_ledger.n_in),
            (v3_ledger.f_in, v2_ledger.f_in),
            (v3_ledger.w_out, v2_ledger.w_out),
            (v3_ledger.c_leak, v2_ledger.c_leak),
            (v3_ledger.a_leak, v2_ledger.a_leak),
        ] {
            assert!((actual - expected).abs() <= 1.0e-12);
        }
    }
}
