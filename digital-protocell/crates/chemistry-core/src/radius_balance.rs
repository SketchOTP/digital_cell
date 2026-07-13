//! Radius-dependent balance analysis (D-004).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RadiusBalanceClass {
    StableFixedRadius,
    UnstableFixedRadius,
    NoFixedRadius,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadiusBalancePoint {
    pub equivalent_radius: f64,
    pub interior_area: f64,
    pub interface_length: f64,
    pub structural_production_per_area: f64,
    pub structural_production_per_interface: f64,
    pub structural_decay_per_area: f64,
    pub resource_influx_per_interface: f64,
    pub net_structural_flux: f64,
}

pub fn classify_radius_balance(points: &[RadiusBalancePoint]) -> RadiusBalanceClass {
    if points.len() < 2 {
        return RadiusBalanceClass::NoFixedRadius;
    }
    let mut sign_changes = 0i32;
    let mut prev_sign = points[0].net_structural_flux.signum() as i32;
    for p in &points[1..] {
        let s = p.net_structural_flux.signum() as i32;
        if s != 0 && prev_sign != 0 && s != prev_sign {
            sign_changes += 1;
        }
        if s != 0 {
            prev_sign = s;
        }
    }
    if sign_changes == 0 {
        RadiusBalanceClass::NoFixedRadius
    } else if points[0].net_structural_flux > 0.0 {
        RadiusBalanceClass::StableFixedRadius
    } else {
        RadiusBalanceClass::UnstableFixedRadius
    }
}
