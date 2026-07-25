//! D-091 abrasion fronts — dish-coordinate damage, identity-blind.

use crate::mesh_population::MeshIndividual;
use crate::mesh_reactions::{apply_membrane_damage, apply_structural_damage};
use crate::spatial_shared_dish::SpatialDish;
use serde::{Deserialize, Serialize};

/// Candidate front strengths (fraction of contacted mesh material damaged).
pub const ABRASION_STRENGTHS: [f64; 3] = [0.05, 0.075, 0.10];

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AbrasionFront {
    /// Front line: point + unit normal in dish coordinates.
    pub origin: [f64; 2],
    pub normal: [f64; 2],
    /// Half-width of the damage band (length units).
    pub half_width: f64,
    /// Structural/membrane damage fraction applied to contacted edges.
    pub strength: f64,
    /// Travel speed along the normal (dish units / time).
    pub speed: f64,
}

impl AbrasionFront {
    pub fn from_dish(dish: &SpatialDish, strength: f64, reverse: bool) -> Self {
        let w = dish.nx as f64 * dish.dx;
        let h = dish.ny as f64 * dish.dx;
        let origin = if reverse {
            [dish.origin[0] + w, dish.origin[1] + h * 0.5]
        } else {
            [dish.origin[0], dish.origin[1] + h * 0.5]
        };
        let normal = if reverse { [-1.0, 0.0] } else { [1.0, 0.0] };
        Self {
            origin,
            normal,
            half_width: dish.dx * 1.5,
            strength: strength.clamp(0.0, 1.0),
            speed: (w / 40.0).max(0.05),
        }
    }

    pub fn advance(&mut self, dt: f64) {
        self.origin[0] += self.normal[0] * self.speed * dt;
        self.origin[1] += self.normal[1] * self.speed * dt;
    }

    /// True if a world point lies inside the moving abrasion band.
    pub fn contacts_point(&self, x: f64, y: f64) -> bool {
        let dx = x - self.origin[0];
        let dy = y - self.origin[1];
        let dist = (dx * self.normal[0] + dy * self.normal[1]).abs();
        dist <= self.half_width
    }

    /// Apply local damage to any mesh whose centroid or vertices contact the front.
    /// Does not read lineage, clade, catalyst type, or labels.
    pub fn apply_to_individual(&self, ind: &mut MeshIndividual) -> f64 {
        if !ind.mesh.alive {
            return 0.0;
        }
        let c = ind.mesh.centroid();
        let hit = self.contacts_point(c[0], c[1])
            || ind
                .mesh
                .vertices
                .iter()
                .any(|v| self.contacts_point(v[0], v[1]));
        if !hit {
            return 0.0;
        }
        let s = apply_structural_damage(&mut ind.mesh, self.strength);
        let m = apply_membrane_damage(&mut ind.mesh, self.strength * 0.5);
        s + m
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbrasionCampaign {
    pub fronts_per_cycle: u32,
    pub strength: f64,
    pub reverse: bool,
    pub cycle_period: f64,
    pub t: f64,
    pub fronts_fired: u32,
    pub active: Option<AbrasionFront>,
}

impl AbrasionCampaign {
    pub fn new(strength: f64, cycle_period: f64, reverse: bool) -> Self {
        Self {
            fronts_per_cycle: 2,
            strength,
            reverse,
            cycle_period: cycle_period.max(1.0),
            t: 0.0,
            fronts_fired: 0,
            active: None,
        }
    }

    pub fn step(
        &mut self,
        dish: &SpatialDish,
        inds: &mut [MeshIndividual],
        dt: f64,
    ) -> f64 {
        let mut damaged = 0.0;
        let p = self.cycle_period;
        let phase = self.t.rem_euclid(p);
        // Two fronts per cycle at 25% and 75% phase.
        let fire_slots = [0.25 * p, 0.75 * p];
        for &slot in &fire_slots {
            if phase < slot && phase + dt >= slot {
                let mut front = AbrasionFront::from_dish(dish, self.strength, self.reverse);
                // Alternate direction within cycle for second front.
                if self.fronts_fired % 2 == 1 {
                    front.normal[0] *= -1.0;
                    front.normal[1] *= -1.0;
                }
                self.active = Some(front);
                self.fronts_fired += 1;
            }
        }
        if let Some(front) = self.active.as_mut() {
            for ind in inds.iter_mut() {
                damaged += front.apply_to_individual(ind);
            }
            front.advance(dt);
            // Retire after crossing the dish.
            let w = dish.nx as f64 * dish.dx;
            if front.origin[0] < dish.origin[0] - w || front.origin[0] > dish.origin[0] + 2.0 * w {
                self.active = None;
            }
        }
        self.t += dt;
        damaged
    }
}
