//! Bounded computational port, not native organism chemistry or physical calibration.
//!
//! The six-species fixed-volume chamber is a new source owner. Native
//! `mesh_mechanics` owns every vertex update. All six native stretch receivers
//! are counted; transport, growth, pressure, bending and lifecycle are excluded.
use crate::material_mesh::{LumpedChem, MaterialMesh};
use crate::mesh_mechanics::{self, MechParams, MAX_EXTERNAL_FORCE_PER_VERTEX};
use serde::{Deserialize, Serialize};

pub const INITIAL: [f64; 6] = [0.1, 0.05, 0.05, 0.01, 1.0, 0.1];
pub const NU_A: [f64; 4] = [0.0, 1.0, 0.0, -1.0];
const NU: [[f64; 4]; 6] = [
    [0.0, -1.0, 0.0, 1.0],
    [0.0, 1.0, -1.0, 0.0],
    [0.0, 0.0, 1.0, -1.0],
    [0.0, 0.0, 0.0, 1.0],
    [-1.0, 0.0, 0.0, 0.0],
    [1.0, -1.0, -1.0, 0.0],
];

/// Every numerical value is an illustrative design input, never a fitted native parameter.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ReferencePortConfig {
    pub k: f64,
    pub lambda: f64,
    pub theta: f64,
    pub k_s: f64,
    pub gamma: f64,
    pub rate_scale: f64,
    pub selected_edge: usize,
    pub energy_tolerance: f64,
    pub invariant_tolerance: f64,
    pub maximum_dt: f64,
}

impl Default for ReferencePortConfig {
    fn default() -> Self {
        Self {
            k: 2.0,
            lambda: 0.75,
            theta: 1.0,
            k_s: 1.0,
            gamma: 1.0,
            rate_scale: 1.0,
            selected_edge: 0,
            energy_tolerance: 1e-12,
            invariant_tolerance: 1e-10,
            maximum_dt: 0.01,
        }
    }
}

impl ReferencePortConfig {
    fn validate(&self) -> Result<(), String> {
        let all = [
            self.k,
            self.lambda,
            self.theta,
            self.k_s,
            self.gamma,
            self.rate_scale,
            self.energy_tolerance,
            self.invariant_tolerance,
            self.maximum_dt,
        ];
        if all.iter().any(|x| !x.is_finite())
            || self.k < 0.0
            || self.lambda < 0.0
            || self.rate_scale < 0.0
            || self.theta <= 0.0
            || self.k_s <= 0.0
            || self.gamma < 1e-15
            || self.selected_edge >= 6
            || self.energy_tolerance < 0.0
            || self.energy_tolerance > 1e-9
            || self.invariant_tolerance <= 0.0
            || self.invariant_tolerance > 1e-8
            || self.maximum_dt <= 0.0
            || self.maximum_dt > 0.01
        {
            return Err("invalid frozen configuration/domain".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ReferencePortState {
    config: ReferencePortConfig,
    mesh: MaterialMesh,
    species: [f64; 6],
    p: f64,
    a0: f64,
    initial_total_g: f64,
    time: f64,
    accepted_steps: u64,
    integrated_dchem: f64,
    integrated_dmech: f64,
    signed_defect: f64,
    absolute_defect: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Restart {
    config: ReferencePortConfig,
    mesh: MaterialMesh,
    species: [f64; 6],
    p: f64,
    a0: f64,
    initial_total_g: f64,
    time: f64,
    accepted_steps: u64,
    integrated_dchem: f64,
    integrated_dmech: f64,
    signed_defect: f64,
    absolute_defect: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortDiagnostics {
    pub species: [f64; 6],
    pub vertices: Vec<[f64; 2]>,
    pub lengths: Vec<f64>,
    pub rest_lengths: Vec<f64>,
    pub p: f64,
    pub a0: f64,
    pub gchem: f64,
    pub ec: f64,
    pub us: f64,
    pub total_g: f64,
    pub coupling_tension: f64,
    pub g: f64,
    pub bare_forward: [f64; 4],
    pub bare_reverse: [f64; 4],
    pub forward: [f64; 4],
    pub reverse: [f64; 4],
    pub affinities: [f64; 4],
    pub currents: [f64; 4],
    pub species_derivative: [f64; 6],
    pub native_forces: Vec<[f64; 2]>,
    pub coupling_forces: Vec<[f64; 2]>,
    pub combined_forces: Vec<[f64; 2]>,
    pub velocities: Vec<[f64; 2]>,
    pub dchem: f64,
    pub dmech: f64,
    pub gradient_dot_ode: f64,
    pub gradient_identity_error: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepRecord {
    pub time: f64,
    pub dt: f64,
    pub diagnostics: PortDiagnostics,
    pub integrated_dchem: f64,
    pub integrated_dmech: f64,
    pub step_defect: f64,
    pub signed_defect: f64,
    pub absolute_defect: f64,
    pub coupling_work: f64,
    pub native_receiver_work: f64,
    pub native_material_total: f64,
}

fn chamber_totals(z: &[f64; 6]) -> [f64; 2] {
    [
        z[0] + z[1] + z[2],
        z[4] + z[5] + z[1] + 2.0 * z[2] + 2.0 * z[3],
    ]
}

fn cold_mesh() -> MaterialMesh {
    let mut mesh = MaterialMesh::seed_regular(
        6,
        1.0,
        0.0,
        0.0,
        1.0,
        0.7,
        LumpedChem::default(),
        LumpedChem::default(),
        0.0,
    );
    mesh.stamp_maturation_coupled_schema();
    mesh
}

impl ReferencePortState {
    pub fn cold(config: ReferencePortConfig) -> Result<Self, String> {
        config.validate()?;
        let mut state = Self {
            config,
            mesh: cold_mesh(),
            species: INITIAL,
            p: chamber_totals(&INITIAL)[0],
            a0: INITIAL[1] + INITIAL[2],
            initial_total_g: 0.0,
            time: 0.0,
            accepted_steps: 0,
            integrated_dchem: 0.0,
            integrated_dmech: 0.0,
            signed_defect: 0.0,
            absolute_defect: 0.0,
        };
        state.initial_total_g = state.diagnostics()?.total_g;
        state.validate()?;
        Ok(state)
    }

    pub fn config(&self) -> &ReferencePortConfig {
        &self.config
    }
    pub fn mesh(&self) -> &MaterialMesh {
        &self.mesh
    }
    pub fn species(&self) -> [f64; 6] {
        self.species
    }
    pub fn time(&self) -> f64 {
        self.time
    }

    fn mech(&self, dt: f64) -> MechParams {
        MechParams {
            gamma: self.config.gamma,
            k_s: self.config.k_s,
            kappa_b: 0.0,
            k_pi: 0.0,
            dt,
        }
    }

    fn validate_domain(&self) -> Result<(), String> {
        self.config.validate()?;
        if self.species.iter().any(|z| !z.is_finite() || *z <= 0.0)
            || self.mesh.n() != 6
            || self.mesh.edges.len() != 6
            || !self.mesh.physical_runtime_valid()
            || !self.mesh.alive
            || self.mesh.area() <= 1e-12
            || !self.mesh.area().is_finite()
        {
            return Err("invalid source or native geometry".into());
        }
        // Every non-geometry native field must equal the cold fixed-material owner.
        let mut anchor = cold_mesh();
        anchor.vertices = self.mesh.vertices.clone();
        if serde_json::to_value(&anchor).map_err(|e| e.to_string())?
            != serde_json::to_value(&self.mesh).map_err(|e| e.to_string())?
        {
            return Err("native material/contract/unsupported physics changed".into());
        }
        for i in 0..6 {
            let l0 = self.mesh.rest_length(i);
            let a = self.mesh.vertices[i];
            let b = self.mesh.vertices[(i + 1) % 6];
            let length = (b[0] - a[0]).hypot(b[1] - a[1]);
            if l0 < 1e-3 || !length.is_finite() || length <= 1e-15 || length > 4.0 * l0 {
                return Err("native reference/clamp domain exceeded".into());
            }
            // Every other vertex must lie strictly inside this oriented edge's
            // half-plane. Local turn tests alone admit self-crossing star rings.
            for v in 0..6 {
                if v == i || v == (i + 1) % 6 {
                    continue;
                }
                let c = self.mesh.vertices[v];
                if (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]) <= 1e-12 {
                    return Err("non-convex/contact geometry outside port".into());
                }
            }
        }
        if self.p != chamber_totals(&INITIAL)[0] || self.a0 != INITIAL[1] + INITIAL[2] {
            return Err("frozen chamber ownership changed".into());
        }
        let actual = chamber_totals(&self.species);
        let expected = chamber_totals(&INITIAL);
        if actual
            .iter()
            .zip(expected)
            .any(|(a, b)| (a - b).abs() > self.config.invariant_tolerance)
        {
            return Err("chamber conservation defect".into());
        }
        Ok(())
    }

    fn validate(&self) -> Result<(), String> {
        self.validate_domain()?;
        let scalars = [
            self.initial_total_g,
            self.time,
            self.integrated_dchem,
            self.integrated_dmech,
            self.signed_defect,
            self.absolute_defect,
        ];
        if scalars.iter().any(|x| !x.is_finite())
            || self.time < 0.0
            || self.integrated_dchem < 0.0
            || self.integrated_dmech < 0.0
            || self.absolute_defect < self.signed_defect.abs() - 1e-12
        {
            return Err("invalid cumulative ledger".into());
        }
        let initial_g = self.config.theta * INITIAL.iter().map(|z| z * (z.ln() - 1.0)).sum::<f64>();
        if (self.initial_total_g - initial_g).abs() > 1e-12
            || (self.diagnostics()?.total_g - self.initial_total_g
                + self.integrated_dchem
                + self.integrated_dmech
                - self.signed_defect)
                .abs()
                > self.config.invariant_tolerance
        {
            return Err("restart/free-energy ledger inconsistent".into());
        }
        if (self.accepted_steps == 0) != (self.time == 0.0) {
            return Err("invalid restart time/step ownership".into());
        }
        if self.accepted_steps == 0
            && (self.species != INITIAL
                || self.mesh.vertices != cold_mesh().vertices
                || self.integrated_dchem != 0.0
                || self.integrated_dmech != 0.0
                || self.signed_defect != 0.0
                || self.absolute_defect != 0.0)
        {
            return Err("non-cold entry is not a restart".into());
        }
        Ok(())
    }

    pub fn diagnostics(&self) -> Result<PortDiagnostics, String> {
        self.validate_domain()?;
        let z = self.species;
        let lengths: Vec<f64> = (0..6).map(|i| self.mesh.edge_length(i)).collect();
        let rest_lengths: Vec<f64> = (0..6).map(|i| self.mesh.rest_length(i)).collect();
        let edge = self.config.selected_edge;
        let l0 = rest_lengths[edge];
        let extension =
            (lengths[edge] - l0) / l0 - self.config.lambda * (z[1] + z[2] - self.a0) / self.p;
        let ec = self.config.k * self.p * extension * extension / 2.0;
        let g = -self.config.k * self.config.lambda * extension;
        let tc = self.config.k * self.p * extension / l0;
        let us = lengths
            .iter()
            .zip(&rest_lengths)
            .map(|(l, r)| self.config.k_s * (l - r).powi(2) / (2.0 * r))
            .sum();
        let gchem = self.config.theta * z.iter().map(|v| v * (v.ln() - 1.0)).sum::<f64>();
        let bare_forward = [z[4], z[5] * z[0], z[5] * z[1], z[2]];
        let bare_reverse = [z[5], z[1], z[2], z[0] * z[3]];
        let mut forward = [0.0; 4];
        let mut reverse = [0.0; 4];
        let mut affinities = [0.0; 4];
        let mut currents = [0.0; 4];
        for i in 0..4 {
            affinities[i] =
                bare_forward[i].ln() - bare_reverse[i].ln() - NU_A[i] * g / self.config.theta;
            forward[i] = self.config.rate_scale
                * bare_forward[i]
                * (-NU_A[i] * g / (2.0 * self.config.theta)).exp();
            reverse[i] = self.config.rate_scale
                * bare_reverse[i]
                * (NU_A[i] * g / (2.0 * self.config.theta)).exp();
            currents[i] = forward[i] - reverse[i];
        }
        let mut species_derivative = [0.0; 6];
        for i in 0..6 {
            for j in 0..4 {
                species_derivative[i] += NU[i][j] * currents[j];
            }
        }
        let native_forces = mesh_mechanics::compute_forces(&self.mesh, &self.mech(0.0));
        let mut coupling_forces = vec![[0.0; 2]; 6];
        let next = (edge + 1) % 6;
        for axis in 0..2 {
            let f = tc * (self.mesh.vertices[next][axis] - self.mesh.vertices[edge][axis])
                / lengths[edge];
            coupling_forces[edge][axis] = f;
            coupling_forces[next][axis] = -f;
        }
        let combined_forces: Vec<[f64; 2]> = native_forces
            .iter()
            .zip(&coupling_forces)
            .map(|(n, c)| [n[0] + c[0], n[1] + c[1]])
            .collect();
        let velocities: Vec<[f64; 2]> = combined_forces
            .iter()
            .map(|f| [f[0] / self.config.gamma, f[1] / self.config.gamma])
            .collect();
        let dchem = self.config.theta
            * currents
                .iter()
                .zip(affinities)
                .map(|(j, a)| j * a)
                .sum::<f64>();
        let dmech = combined_forces
            .iter()
            .map(|f| (f[0] * f[0] + f[1] * f[1]) / self.config.gamma)
            .sum::<f64>();
        let chemical_gradient_dot = species_derivative
            .iter()
            .enumerate()
            .map(|(i, d)| {
                d * (self.config.theta * z[i].ln() + if i == 1 || i == 2 { g } else { 0.0 })
            })
            .sum::<f64>();
        // Differentiate owned storage separately from native force evaluation.
        let mut receiver_gradient = vec![[0.0; 2]; 6];
        for i in 0..6 {
            let j = (i + 1) % 6;
            let tension = self.config.k_s * (lengths[i] - rest_lengths[i]) / rest_lengths[i];
            for axis in 0..2 {
                let component = tension
                    * (self.mesh.vertices[j][axis] - self.mesh.vertices[i][axis])
                    / lengths[i];
                receiver_gradient[i][axis] -= component;
                receiver_gradient[j][axis] += component;
            }
        }
        let spatial_gradient_dot: f64 = (0..6)
            .map(|i| {
                (0..2)
                    .map(|a| (receiver_gradient[i][a] - coupling_forces[i][a]) * velocities[i][a])
                    .sum::<f64>()
            })
            .sum();
        let gradient_dot_ode = chemical_gradient_dot + spatial_gradient_dot;
        let gradient_identity_error = gradient_dot_ode + dchem + dmech;
        let result = PortDiagnostics {
            species: z,
            vertices: self.mesh.vertices.clone(),
            lengths,
            rest_lengths,
            p: self.p,
            a0: self.a0,
            gchem,
            ec,
            us,
            total_g: gchem + ec + us,
            coupling_tension: tc,
            g,
            bare_forward,
            bare_reverse,
            forward,
            reverse,
            affinities,
            currents,
            species_derivative,
            native_forces,
            coupling_forces,
            combined_forces,
            velocities,
            dchem,
            dmech,
            gradient_dot_ode,
            gradient_identity_error,
        };
        let numbers = [
            result.total_g,
            result.ec,
            result.us,
            result.gchem,
            tc,
            g,
            dchem,
            dmech,
            gradient_identity_error,
        ];
        if numbers.iter().any(|v| !v.is_finite())
            || dchem < -1e-12
            || dmech < 0.0
            || forward
                .iter()
                .chain(reverse.iter())
                .any(|v| !v.is_finite() || *v < 0.0 || (self.config.rate_scale > 0.0 && *v == 0.0))
            || result.species_derivative.iter().any(|v| !v.is_finite())
            || result
                .combined_forces
                .iter()
                .flatten()
                .any(|v| !v.is_finite())
        {
            return Err("nonfinite/underflow/negative dissipation".into());
        }
        Ok(result)
    }

    pub fn try_step(&self, dt: f64) -> Result<(Self, StepRecord), String> {
        self.validate()?;
        if !dt.is_finite() || dt <= 0.0 || dt > self.config.maximum_dt {
            return Err("step outside frozen finite bound".into());
        }
        let old = self.diagnostics()?;
        if old
            .coupling_forces
            .iter()
            .any(|f| f[0].hypot(f[1]) > MAX_EXTERNAL_FORCE_PER_VERTEX)
        {
            return Err("native external-force cap exceeded".into());
        }
        let mut trial = self.clone();
        for i in 0..6 {
            trial.species[i] += dt * old.species_derivative[i];
        }
        if !mesh_mechanics::mechanics_step_with_external_forces(
            &mut trial.mesh,
            &self.mech(dt),
            &old.coupling_forces,
        ) {
            return Err("native mechanics rejected; entire trial discarded".into());
        }
        let new = trial.diagnostics()?;
        if new
            .coupling_forces
            .iter()
            .any(|f| f[0].hypot(f[1]) > MAX_EXTERNAL_FORCE_PER_VERTEX)
        {
            return Err("post-step port force outside native cap; entire trial discarded".into());
        }
        if new.total_g - old.total_g > self.config.energy_tolerance {
            return Err("joint free energy increased; entire trial discarded".into());
        }
        let dchem = dt * old.dchem;
        let dmech = dt * old.dmech;
        let defect = new.total_g - old.total_g + dchem + dmech;
        trial.integrated_dchem += dchem;
        trial.integrated_dmech += dmech;
        trial.signed_defect += defect;
        trial.absolute_defect += defect.abs();
        trial.time += dt;
        trial.accepted_steps = trial.accepted_steps.checked_add(1).ok_or("step overflow")?;
        trial.validate()?;
        let work = |forces: &Vec<[f64; 2]>| -> f64 {
            (0..6)
                .map(|i| {
                    (0..2)
                        .map(|a| {
                            forces[i][a] * (trial.mesh.vertices[i][a] - self.mesh.vertices[i][a])
                        })
                        .sum::<f64>()
                })
                .sum()
        };
        let record = StepRecord {
            time: trial.time,
            dt,
            diagnostics: new,
            integrated_dchem: trial.integrated_dchem,
            integrated_dmech: trial.integrated_dmech,
            step_defect: defect,
            signed_defect: trial.signed_defect,
            absolute_defect: trial.absolute_defect,
            coupling_work: work(&old.coupling_forces),
            native_receiver_work: work(&old.native_forces),
            native_material_total: trial.mesh.total_structural_mass(),
        };
        Ok((trial, record))
    }

    pub fn restart_json(&self) -> Result<String, String> {
        self.validate()?;
        serde_json::to_string(self).map_err(|e| e.to_string())
    }

    /// Validates mathematical ownership/ledgers; archive identity is checked by the caller.
    pub fn from_restart_json(
        json: &str,
        expected_config: &ReferencePortConfig,
    ) -> Result<Self, String> {
        expected_config.validate()?;
        let raw: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let r: Restart = serde_json::from_value(raw.clone()).map_err(|e| e.to_string())?;
        if &r.config != expected_config {
            return Err("restart differs from trusted birth configuration".into());
        }
        let state = Self {
            config: r.config,
            mesh: r.mesh,
            species: r.species,
            p: r.p,
            a0: r.a0,
            initial_total_g: r.initial_total_g,
            time: r.time,
            accepted_steps: r.accepted_steps,
            integrated_dchem: r.integrated_dchem,
            integrated_dmech: r.integrated_dmech,
            signed_defect: r.signed_defect,
            absolute_defect: r.absolute_defect,
        };
        if serde_json::to_value(&state).map_err(|e| e.to_string())? != raw {
            return Err("restart contains unknown/defaulted/stale fields".into());
        }
        state.validate()?;
        Ok(state)
    }

    /// Externally prepared diagnostic clone ONLY. Never a trajectory/restart entry.
    pub fn probe(&self, strain: f64, da: f64) -> Result<PortDiagnostics, String> {
        self.validate()?;
        if !strain.is_finite() || !da.is_finite() || self.accepted_steps != 0 {
            return Err("cold finite probe only".into());
        }
        let mut trial = self.clone();
        trial.species[0] -= da;
        trial.species[1] += da;
        trial.species[4] -= da;
        let i = self.config.selected_edge;
        let j = (i + 1) % 6;
        let length = trial.mesh.edge_length(i);
        for axis in 0..2 {
            let delta =
                (trial.mesh.vertices[j][axis] - trial.mesh.vertices[i][axis]) * strain / 2.0;
            trial.mesh.vertices[i][axis] -= delta;
            trial.mesh.vertices[j][axis] += delta;
        }
        let d = trial.diagnostics()?;
        if d.coupling_forces
            .iter()
            .any(|f| f[0].hypot(f[1]) > MAX_EXTERNAL_FORCE_PER_VERTEX)
        {
            return Err("diagnostic preparation outside native port cap".into());
        }
        if (d.lengths[i] / length - 1.0 - strain).abs() > 1e-12 {
            return Err("probe geometry mismatch".into());
        }
        Ok(d)
    }
}
