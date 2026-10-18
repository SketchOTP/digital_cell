//! Explicit new computational architecture, not whole-organism biology.
//! Native kernels own vertex integration and midpoint splitting. Full chemical
//! amounts and a finite inert outer compartment own the area conjugate work.
use crate::chemical_reference_port::{INITIAL, NU_A};
use crate::material_mesh::{LumpedChem, MaterialMesh};
use crate::mesh_mechanics::{self, MechParams, MAX_EXTERNAL_FORCE_PER_VERTEX};
use serde::{Deserialize, Serialize};

const NU: [[f64; 4]; 6] = [
    [0., -1., 0., 1.],
    [0., 1., -1., 0.],
    [0., 0., 1., -1.],
    [0., 0., 0., 1.],
    [-1., 0., 0., 0.],
    [1., -1., -1., 0.],
];

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SourcePreparation {
    Published,
    ClosedEquilibrium,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BodyConfig {
    pub k: f64,
    pub lambda: f64,
    pub theta: f64,
    pub k_s: f64,
    pub gamma: f64,
    pub rate_scale: f64,
    pub preparation: SourcePreparation,
}
impl Default for BodyConfig {
    fn default() -> Self {
        Self {
            k: 2.,
            lambda: 0.75,
            theta: 1.,
            k_s: 1.,
            gamma: 1.,
            rate_scale: 1.,
            preparation: SourcePreparation::Published,
        }
    }
}
impl BodyConfig {
    fn validate(&self) -> Result<(), String> {
        if [
            self.k,
            self.lambda,
            self.theta,
            self.k_s,
            self.gamma,
            self.rate_scale,
        ]
        .iter()
        .any(|v| !v.is_finite())
            || self.k < 0.
            || self.lambda < 0.
            || self.theta <= 0.
            || self.k_s <= 0.
            || self.gamma < 1e-15
            || self.rate_scale < 0.
        {
            return Err("invalid fixed body configuration".into());
        }
        Ok(())
    }
}

/// Immutable birth-material interval, exactly 0/1 or binary halves in this gate.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Ancestry {
    pub birth_edge: usize,
    pub start: f64,
    pub end: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Owned {
    config: BodyConfig,
    mesh: MaterialMesh,
    amounts: [f64; 6],
    ancestry: Vec<Ancestry>,
    remeshed: bool,
    time: f64,
    steps: u64,
    integrated_dchem: f64,
    integrated_dmech: f64,
    signed_step_defect: f64,
    absolute_step_defect: f64,
    remesh_defect: f64,
}
#[derive(Debug, Clone, Serialize)]
pub struct ChemicalBodyState {
    data: Owned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyDiagnostics {
    pub amounts: [f64; 6],
    pub concentrations: [f64; 6],
    pub volume: f64,
    pub outside_volume: f64,
    pub outside_amount: f64,
    pub birth_area: f64,
    pub p: f64,
    pub a0: f64,
    pub vertices: Vec<[f64; 2]>,
    pub rest_lengths: Vec<f64>,
    pub lengths: Vec<f64>,
    pub structural_masses: Vec<f64>,
    pub ancestry: Vec<Ancestry>,
    pub arc_indices: Vec<usize>,
    pub arc_length: f64,
    pub arc_rest_length: f64,
    pub gchem: f64,
    pub gout: f64,
    pub ec: f64,
    pub us: f64,
    pub total_g: f64,
    pub g: f64,
    pub coupling_tension: f64,
    pub bare_forward: [f64; 4],
    pub bare_reverse: [f64; 4],
    pub forward: [f64; 4],
    pub reverse: [f64; 4],
    pub affinities: [f64; 4],
    pub currents: [f64; 4],
    pub amount_derivative: [f64; 6],
    pub native_forces: Vec<[f64; 2]>,
    pub coupling_forces: Vec<[f64; 2]>,
    pub osmotic_forces: Vec<[f64; 2]>,
    pub external_forces: Vec<[f64; 2]>,
    pub combined_forces: Vec<[f64; 2]>,
    pub velocities: Vec<[f64; 2]>,
    pub dchem: f64,
    pub dmech: f64,
    pub gradient_dot_ode: f64,
    pub gradient_identity_error: f64,
    pub omitted_osmotic_identity_error: f64,
}
#[derive(Debug, Clone, Serialize)]
pub struct BodyStep {
    pub time: f64,
    pub dt: f64,
    pub diagnostics: BodyDiagnostics,
    pub integrated_dchem: f64,
    pub integrated_dmech: f64,
    pub step_defect: f64,
    pub signed_step_defect: f64,
    pub absolute_step_defect: f64,
    pub remesh_defect: f64,
    pub coupling_work: f64,
    pub osmotic_work: f64,
    pub native_receiver_work: f64,
}
#[derive(Debug, Clone, Serialize)]
pub struct RemeshRecord {
    pub time: f64,
    pub before: BodyDiagnostics,
    pub after: BodyDiagnostics,
    pub ancestry: Vec<Ancestry>,
    pub storage_delta: f64,
    pub area_delta: f64,
    pub force_pullback_error: f64,
    pub native_splits: usize,
}

fn birth_mesh() -> MaterialMesh {
    let mut m = MaterialMesh::seed_regular(
        24,
        1.,
        0.,
        0.,
        1.,
        0.7,
        LumpedChem::default(),
        LumpedChem::default(),
        0.,
    );
    m.stamp_maturation_coupled_schema();
    m.l_max = 0.2;
    m.l_min = 0.05;
    m
}
fn birth_source(prep: SourcePreparation) -> [f64; 6] {
    if prep == SourcePreparation::Published {
        return INITIAL;
    }
    let mut lo: f64 = 0.;
    let mut hi: f64 = 1.;
    for _ in 0..100 {
        let s = (lo + hi) / 2.;
        let polynomial = 200. * s.powi(4) + 400. * s.powi(3) + 313. * s * s + 93. * s - 127.;
        if polynomial > 0. {
            hi = s;
        } else {
            lo = s;
        }
    }
    let s = (lo + hi) / 2.;
    let e = 0.2 / (1. + s + s * s);
    [e, s * e, s * s * e, s * s, s, s]
}
fn totals(n: &[f64; 6]) -> [f64; 2] {
    [
        n[0] + n[1] + n[2],
        n[4] + n[5] + n[1] + 2. * n[2] + 2. * n[3],
    ]
}
fn intervals(split: bool) -> Vec<Ancestry> {
    let mut a = Vec::new();
    for i in 0..24 {
        if split {
            a.push(Ancestry {
                birth_edge: i,
                start: 0.,
                end: 0.5,
            });
            a.push(Ancestry {
                birth_edge: i,
                start: 0.5,
                end: 1.,
            });
        } else {
            a.push(Ancestry {
                birth_edge: i,
                start: 0.,
                end: 1.,
            });
        }
    }
    a
}
fn cross(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}
fn segments_contact(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2]) -> bool {
    let o = [
        cross(a, b, c),
        cross(a, b, d),
        cross(c, d, a),
        cross(c, d, b),
    ];
    let sign = |v: f64| {
        if v > 1e-12 {
            1
        } else if v < -1e-12 {
            -1
        } else {
            0
        }
    };
    if sign(o[0]) * sign(o[1]) < 0 && sign(o[2]) * sign(o[3]) < 0 {
        return true;
    }
    let on = |p: [f64; 2], q: [f64; 2], r: [f64; 2], area: f64| {
        area.abs() <= 1e-12
            && (0..2).all(|k| r[k] >= p[k].min(q[k]) - 1e-12 && r[k] <= p[k].max(q[k]) + 1e-12)
    };
    on(a, b, c, o[0]) || on(a, b, d, o[1]) || on(c, d, a, o[2]) || on(c, d, b, o[3])
}

impl ChemicalBodyState {
    pub fn cold(config: BodyConfig) -> Result<Self, String> {
        config.validate()?;
        let n = birth_source(config.preparation);
        let s = Self {
            data: Owned {
                config,
                mesh: birth_mesh(),
                amounts: n,
                ancestry: intervals(false),
                remeshed: false,
                time: 0.,
                steps: 0,
                integrated_dchem: 0.,
                integrated_dmech: 0.,
                signed_step_defect: 0.,
                absolute_step_defect: 0.,
                remesh_defect: 0.,
            },
        };
        s.validate()?;
        Ok(s)
    }
    pub fn config(&self) -> &BodyConfig {
        &self.data.config
    }
    pub fn mesh(&self) -> &MaterialMesh {
        &self.data.mesh
    }
    pub fn time(&self) -> f64 {
        self.data.time
    }
    fn mech(&self, dt: f64) -> MechParams {
        MechParams {
            gamma: self.data.config.gamma,
            k_s: self.data.config.k_s,
            kappa_b: 0.,
            k_pi: 0.,
            dt,
        }
    }

    fn domain(&self) -> Result<(), String> {
        let d = &self.data;
        d.config.validate()?;
        if d.amounts.iter().any(|v| !v.is_finite() || *v <= 0.) {
            return Err("nonpositive/nonfinite source".into());
        }
        if d.ancestry != intervals(d.remeshed)
            || d.mesh.n() != if d.remeshed { 48 } else { 24 }
            || d.mesh.edges.len() != d.mesh.n()
        {
            return Err("material ancestry/topology mismatch".into());
        }
        let mut expected = birth_mesh();
        if d.remeshed {
            mesh_mechanics::remesh_split(&mut expected);
        }
        expected.vertices = d.mesh.vertices.clone();
        if serde_json::to_value(&expected).map_err(|e| e.to_string())?
            != serde_json::to_value(&d.mesh).map_err(|e| e.to_string())?
        {
            return Err("native material/unsupported physics mutated".into());
        }
        let m = &d.mesh;
        let count = m.n();
        if !m.physical_runtime_valid()
            || !m.alive
            || !m.signed_area().is_finite()
            || m.signed_area() <= 1e-12
        {
            return Err("invalid native geometry".into());
        }
        for i in 0..count {
            let p = m.vertices[i];
            let q = m.vertices[(i + 1) % count];
            let l = (q[0] - p[0]).hypot(q[1] - p[1]);
            if !l.is_finite() || l <= 1e-12 || m.rest_length(i) < 0.001 || l > 4. * m.rest_length(i)
            {
                return Err("inactive-clamp geometry domain exceeded".into());
            }
            for j in 0..count {
                if j != i && (p[0] - m.vertices[j][0]).hypot(p[1] - m.vertices[j][1]) <= 1e-12 {
                    return Err("duplicate/contact vertices".into());
                }
                if j <= i || j == (i + 1) % count || i == (j + 1) % count {
                    continue;
                }
                if segments_contact(p, q, m.vertices[j], m.vertices[(j + 1) % count]) {
                    return Err("nonadjacent boundary crossing/touch".into());
                }
            }
        }
        let volume = m.area() / birth_mesh().area();
        if volume <= 0. || volume >= 10. {
            return Err("finite bath exhausted/domain exceeded".into());
        }
        let original = totals(&birth_source(d.config.preparation));
        if totals(&d.amounts)
            .iter()
            .zip(original)
            .any(|(a, b)| (a - b).abs() > 1e-10)
        {
            return Err("source conserved-moiety defect".into());
        }
        Ok(())
    }
    fn validate(&self) -> Result<(), String> {
        self.domain()?;
        let d = &self.data;
        if [
            d.time,
            d.integrated_dchem,
            d.integrated_dmech,
            d.signed_step_defect,
            d.absolute_step_defect,
            d.remesh_defect,
        ]
        .iter()
        .any(|x| !x.is_finite())
            || d.time < 0.
            || d.integrated_dchem < 0.
            || d.integrated_dmech < 0.
            || d.absolute_step_defect < d.signed_step_defect.abs() - 1e-12
            || (d.steps == 0) != (d.time == 0.)
        {
            return Err("invalid cumulative body ledger".into());
        }
        let initial = birth_source(d.config.preparation);
        let no = 9. * initial.iter().sum::<f64>();
        let initial_g = d.config.theta
            * (initial.iter().map(|n| n * (n.ln() - 1.)).sum::<f64>() + no * ((no / 9.).ln() - 1.));
        if (self.diagnostics()?.total_g - initial_g + d.integrated_dchem + d.integrated_dmech
            - d.signed_step_defect
            - d.remesh_defect)
            .abs()
            > 1e-10
        {
            return Err("body energy/restart ledger defect".into());
        }
        if d.steps == 0
            && (d.amounts != initial
                || d.mesh.vertices != birth_mesh().vertices
                || d.remeshed
                || d.integrated_dchem != 0.
                || d.integrated_dmech != 0.
                || d.signed_step_defect != 0.
                || d.absolute_step_defect != 0.
                || d.remesh_defect != 0.)
        {
            return Err("noncold unsupported state entry".into());
        }
        Ok(())
    }
    pub fn diagnostics(&self) -> Result<BodyDiagnostics, String> {
        self.domain()?;
        let d = &self.data;
        let c = &d.config;
        let m = &d.mesh;
        let count = m.n();
        let birth = birth_source(c.preparation);
        let birth_area = birth_mesh().area();
        let p = totals(&birth)[0];
        let a0 = birth[1] + birth[2];
        let volume = m.area() / birth_area;
        let outside_volume = 10. - volume;
        let outside_amount = 9. * birth.iter().sum::<f64>();
        let mut z = d.amounts;
        for v in &mut z {
            *v /= volume;
        }
        let lengths: Vec<f64> = (0..count).map(|i| m.edge_length(i)).collect();
        let rest_lengths: Vec<f64> = (0..count).map(|i| m.rest_length(i)).collect();
        let arc_indices: Vec<usize> = d
            .ancestry
            .iter()
            .enumerate()
            .filter(|(_, a)| a.birth_edge == 0)
            .map(|(i, _)| i)
            .collect();
        let arc_length = arc_indices.iter().map(|i| lengths[*i]).sum::<f64>();
        let arc_rest_length = arc_indices.iter().map(|i| rest_lengths[*i]).sum::<f64>();
        let extension = (arc_length - arc_rest_length) / arc_rest_length
            - c.lambda * (d.amounts[1] + d.amounts[2] - a0) / p;
        let ec = c.k * p * extension * extension / 2.;
        let g = -c.k * c.lambda * extension;
        let tc = c.k * p * extension / arc_rest_length;
        let us = lengths
            .iter()
            .zip(&rest_lengths)
            .map(|(l, r)| c.k_s * (l - r).powi(2) / (2. * r))
            .sum::<f64>();
        let gchem = c.theta
            * d.amounts
                .iter()
                .zip(z)
                .map(|(n, conc)| n * (conc.ln() - 1.))
                .sum::<f64>();
        let gout = c.theta * outside_amount * ((outside_amount / outside_volume).ln() - 1.);
        let bare_forward = [z[4], z[5] * z[0], z[5] * z[1], z[2]];
        let bare_reverse = [z[5], z[1], z[2], z[0] * z[3]];
        let mut forward = [0.; 4];
        let mut reverse = [0.; 4];
        let mut affinities = [0.; 4];
        let mut currents = [0.; 4];
        for j in 0..4 {
            affinities[j] = bare_forward[j].ln() - bare_reverse[j].ln() - NU_A[j] * g / c.theta;
            forward[j] =
                volume * c.rate_scale * bare_forward[j] * (-NU_A[j] * g / (2. * c.theta)).exp();
            reverse[j] =
                volume * c.rate_scale * bare_reverse[j] * (NU_A[j] * g / (2. * c.theta)).exp();
            currents[j] = forward[j] - reverse[j];
        }
        let mut amount_derivative = [0.; 6];
        for i in 0..6 {
            for j in 0..4 {
                amount_derivative[i] += NU[i][j] * currents[j];
            }
        }
        let native_forces = mesh_mechanics::compute_forces(m, &self.mech(0.));
        let mut coupling_forces = vec![[0.; 2]; count];
        let mut osmotic_forces = vec![[0.; 2]; count];
        let mut storage_gradient = vec![[0.; 2]; count];
        for i in 0..count {
            let j = (i + 1) % count;
            let prev = (i + count - 1) % count;
            for axis in 0..2 {
                let unit = (m.vertices[j][axis] - m.vertices[i][axis]) / lengths[i];
                let receiver = c.k_s * (lengths[i] - rest_lengths[i]) / rest_lengths[i] * unit;
                storage_gradient[i][axis] -= receiver;
                storage_gradient[j][axis] += receiver;
                if d.ancestry[i].birth_edge == 0 {
                    coupling_forces[i][axis] += tc * unit;
                    coupling_forces[j][axis] -= tc * unit;
                }
            }
            let grad_area = [
                (m.vertices[j][1] - m.vertices[prev][1]) / 2.,
                (m.vertices[prev][0] - m.vertices[j][0]) / 2.,
            ];
            let pressure = c.theta
                * (d.amounts.iter().sum::<f64>() / volume - outside_amount / outside_volume);
            for axis in 0..2 {
                osmotic_forces[i][axis] = pressure * grad_area[axis] / birth_area;
            }
        }
        let external_forces: Vec<[f64; 2]> = (0..count)
            .map(|i| {
                [
                    coupling_forces[i][0] + osmotic_forces[i][0],
                    coupling_forces[i][1] + osmotic_forces[i][1],
                ]
            })
            .collect();
        let combined_forces: Vec<[f64; 2]> = (0..count)
            .map(|i| {
                [
                    native_forces[i][0] + external_forces[i][0],
                    native_forces[i][1] + external_forces[i][1],
                ]
            })
            .collect();
        let velocities: Vec<[f64; 2]> = combined_forces
            .iter()
            .map(|f| [f[0] / c.gamma, f[1] / c.gamma])
            .collect();
        let dchem = c.theta
            * currents
                .iter()
                .zip(affinities)
                .map(|(j, a)| j * a)
                .sum::<f64>();
        let dmech = c.gamma * velocities.iter().flatten().map(|v| v * v).sum::<f64>();
        let chem_dot = amount_derivative
            .iter()
            .enumerate()
            .map(|(i, j)| j * (c.theta * z[i].ln() + if i == 1 || i == 2 { g } else { 0. }))
            .sum::<f64>();
        let spatial_dot = (0..count)
            .map(|i| {
                (0..2)
                    .map(|a| {
                        (storage_gradient[i][a] - coupling_forces[i][a] - osmotic_forces[i][a])
                            * velocities[i][a]
                    })
                    .sum::<f64>()
            })
            .sum::<f64>();
        let gradient_dot_ode = chem_dot + spatial_dot;
        let gradient_identity_error = gradient_dot_ode + dchem + dmech;
        // Recompute both mechanics and drag after deliberately omitting the
        // conjugate force; merely deleting its term from the same RHS is not a control.
        let omitted_spatial = (0..count)
            .map(|i| {
                (0..2)
                    .map(|a| {
                        let v = (native_forces[i][a] + coupling_forces[i][a]) / c.gamma;
                        (storage_gradient[i][a] - coupling_forces[i][a] - osmotic_forces[i][a]) * v
                            + c.gamma * v * v
                    })
                    .sum::<f64>()
            })
            .sum::<f64>();
        let omitted_osmotic_identity_error = chem_dot + dchem + omitted_spatial;
        let result = BodyDiagnostics {
            amounts: d.amounts,
            concentrations: z,
            volume,
            outside_volume,
            outside_amount,
            birth_area,
            p,
            a0,
            vertices: m.vertices.clone(),
            rest_lengths,
            lengths,
            structural_masses: m.edges.iter().map(|e| e.m).collect(),
            ancestry: d.ancestry.clone(),
            arc_indices,
            arc_length,
            arc_rest_length,
            gchem,
            gout,
            ec,
            us,
            total_g: gchem + gout + ec + us,
            g,
            coupling_tension: tc,
            bare_forward,
            bare_reverse,
            forward,
            reverse,
            affinities,
            currents,
            amount_derivative,
            native_forces,
            coupling_forces,
            osmotic_forces,
            external_forces,
            combined_forces,
            velocities,
            dchem,
            dmech,
            gradient_dot_ode,
            gradient_identity_error,
            omitted_osmotic_identity_error,
        };
        if [
            result.total_g,
            g,
            tc,
            dchem,
            dmech,
            gradient_identity_error,
            omitted_osmotic_identity_error,
        ]
        .iter()
        .any(|x| !x.is_finite())
            || dchem < -1e-12
            || dmech < 0.
            || forward
                .iter()
                .chain(&reverse)
                .any(|v| !v.is_finite() || *v < 0. || (c.rate_scale > 0. && *v == 0.))
            || result
                .combined_forces
                .iter()
                .flatten()
                .chain(result.amount_derivative.iter())
                .any(|v| !v.is_finite())
        {
            return Err("nonfinite rate/force/dissipation".into());
        }
        Ok(result)
    }
    /// External diagnostic preparation only; never returns an executable state.
    pub fn geometry_probe(
        &self,
        vertex: usize,
        axis: usize,
        delta: f64,
    ) -> Result<BodyDiagnostics, String> {
        self.validate()?;
        if vertex >= self.mesh().n() || axis > 1 || !delta.is_finite() {
            return Err("invalid diagnostic coordinate".into());
        }
        let mut trial = self.clone();
        trial.data.mesh.vertices[vertex][axis] += delta;
        trial.diagnostics()
    }
    pub fn try_step(&self, dt: f64) -> Result<(Self, BodyStep), String> {
        self.validate()?;
        if !dt.is_finite() || dt <= 0. || dt > 0.01 {
            return Err("finite bounded step required".into());
        }
        let old = self.diagnostics()?;
        Self::force_cap(&old)?;
        let mut t = self.clone();
        for i in 0..6 {
            t.data.amounts[i] += dt * old.amount_derivative[i];
        }
        if !mesh_mechanics::mechanics_step_with_external_forces(
            &mut t.data.mesh,
            &self.mech(dt),
            &old.external_forces,
        ) {
            return Err("native body trial rejected".into());
        }
        let new = t.diagnostics()?;
        Self::force_cap(&new)?;
        if new.total_g - old.total_g > 1e-12 {
            return Err("joint body free energy increased".into());
        }
        let defect = new.total_g - old.total_g + dt * (old.dchem + old.dmech);
        t.data.integrated_dchem += dt * old.dchem;
        t.data.integrated_dmech += dt * old.dmech;
        t.data.signed_step_defect += defect;
        t.data.absolute_step_defect += defect.abs();
        t.data.time += dt;
        t.data.steps = t.data.steps.checked_add(1).ok_or("step overflow")?;
        t.validate()?;
        let work = |f: &Vec<[f64; 2]>| -> f64 {
            (0..f.len())
                .map(|i| {
                    (0..2)
                        .map(|a| {
                            f[i][a] * (t.data.mesh.vertices[i][a] - self.data.mesh.vertices[i][a])
                        })
                        .sum::<f64>()
                })
                .sum()
        };
        let r = BodyStep {
            time: t.data.time,
            dt,
            diagnostics: new,
            integrated_dchem: t.data.integrated_dchem,
            integrated_dmech: t.data.integrated_dmech,
            step_defect: defect,
            signed_step_defect: t.data.signed_step_defect,
            absolute_step_defect: t.data.absolute_step_defect,
            remesh_defect: t.data.remesh_defect,
            coupling_work: work(&old.coupling_forces),
            osmotic_work: work(&old.osmotic_forces),
            native_receiver_work: work(&old.native_forces),
        };
        Ok((t, r))
    }
    fn force_cap(d: &BodyDiagnostics) -> Result<(), String> {
        if d.external_forces
            .iter()
            .any(|f| f[0].hypot(f[1]) > MAX_EXTERNAL_FORCE_PER_VERTEX)
        {
            return Err("native combined external-force cap".into());
        }
        Ok(())
    }
    pub fn try_remesh_split(&self) -> Result<(Self, RemeshRecord), String> {
        self.validate()?;
        if self.data.remeshed || self.data.steps == 0 {
            return Err("only one evolved native 24-to-48 event admitted".into());
        }
        let before = self.diagnostics()?;
        if before.lengths.iter().any(|l| *l <= 0.2 || *l > 0.4) {
            return Err("not an all-edge single-midpoint native event".into());
        }
        let mut t = self.clone();
        let splits = mesh_mechanics::remesh_split(&mut t.data.mesh);
        if splits != 24 || t.data.mesh.n() != 48 {
            return Err("native remesh event mismatch".into());
        }
        for i in 0..24 {
            for axis in 0..2 {
                if t.data.mesh.vertices[2 * i][axis] != self.data.mesh.vertices[i][axis]
                    || t.data.mesh.vertices[2 * i + 1][axis]
                        != 0.5
                            * (self.data.mesh.vertices[i][axis]
                                + self.data.mesh.vertices[(i + 1) % 24][axis])
                {
                    return Err("native midpoint correspondence mismatch".into());
                }
            }
        }
        t.data.remeshed = true;
        t.data.ancestry = intervals(true);
        let after = t.diagnostics()?;
        Self::force_cap(&after)?;
        let storage_delta = after.total_g - before.total_g;
        let area_delta = t.data.mesh.area() - self.data.mesh.area();
        let mut error: f64 = 0.;
        for i in 0..24 {
            for a in 0..2 {
                let pullback = after.combined_forces[2 * i][a]
                    + 0.5 * after.combined_forces[(2 * i + 47) % 48][a]
                    + 0.5 * after.combined_forces[2 * i + 1][a];
                error = error.max((pullback - before.combined_forces[i][a]).abs());
            }
        }
        if storage_delta.abs() > 1e-10 || area_delta.abs() > 1e-12 || error > 1e-10 {
            return Err("remesh storage/area/force pullback defect".into());
        }
        t.data.remesh_defect += storage_delta;
        t.validate()?;
        let r = RemeshRecord {
            time: t.time(),
            before,
            after,
            ancestry: t.data.ancestry.clone(),
            storage_delta,
            area_delta,
            force_pullback_error: error,
            native_splits: splits,
        };
        Ok((t, r))
    }
    pub fn restart_json(&self) -> Result<String, String> {
        self.validate()?;
        serde_json::to_string(self).map_err(|e| e.to_string())
    }
    pub fn from_restart_json(json: &str, expected_config: &BodyConfig) -> Result<Self, String> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            data: Owned,
        }
        expected_config.validate()?;
        let raw: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let wire: Wire = serde_json::from_value(raw.clone()).map_err(|e| e.to_string())?;
        let s = Self { data: wire.data };
        if &s.data.config != expected_config
            || serde_json::to_value(&s).map_err(|e| e.to_string())? != raw
        {
            return Err("restart provenance/shape mismatch".into());
        }
        s.validate()?;
        Ok(s)
    }
}
