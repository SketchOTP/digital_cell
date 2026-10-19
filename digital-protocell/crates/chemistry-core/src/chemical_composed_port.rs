//! New opt-in hybrid reference: continuous published chemistry/native drift,
//! interleaved with EXTERNALLY PRESCRIBED finite proposals. Pair rates are
//! diagnostics, not sampled hazards. No native biology or physical fusion claim.
use crate::material_mesh::{LumpedChem, MaterialMesh};
use crate::mesh_mechanics::{self, MechParams, MAX_EXTERNAL_FORCE_PER_VERTEX};
use crate::mesh_self_contact::polygon_simple;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const BIRTH_N: [f64; 6] = [0.1, 0.05, 0.05, 0.01, 1.0, 10.0];
const NU_A: [f64; 4] = [0.0, 1.0, 0.0, -1.0];
const NU: [[f64; 4]; 6] = [
    [0., -1., 0., 1.],
    [0., 1., -1., 0.],
    [0., 0., 1., -1.],
    [0., 0., 0., 1.],
    [-1., 0., 0., 0.],
    [1., -1., -1., 0.],
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ComposedConfig {
    pub owner: String,
    pub radius: f64,
    pub rho: f64,
    pub theta: f64,
    pub k_s: f64,
    pub gamma: f64,
    pub k: f64,
    pub lambda: f64,
    pub mu_y: f64,
    pub mu_m: f64,
    pub growth_yield: f64,
    pub kappa: f64,
    pub source_rate_scale: f64,
    pub total_volume: f64,
    pub outside_s: f64,
    pub outside_sm: f64,
    pub outside_inert: f64,
}
impl Default for ComposedConfig {
    fn default() -> Self {
        Self {
            owner: "dc-m4-composed-reference".into(),
            radius: 10.,
            rho: 0.1,
            theta: 1.,
            k_s: 1.,
            gamma: 1.,
            k: 0.25,
            lambda: 0.1,
            mu_y: -0.25,
            mu_m: -0.5,
            growth_yield: 0.9,
            kappa: 1.,
            source_rate_scale: 1.,
            total_volume: 10.,
            outside_s: 10.,
            outside_sm: 10.,
            outside_inert: 10.,
        }
    }
}
impl ComposedConfig {
    fn validate(&self) -> Result<(), String> {
        let values = [
            self.radius,
            self.rho,
            self.theta,
            self.k_s,
            self.gamma,
            self.k,
            self.lambda,
            self.mu_y,
            self.mu_m,
            self.growth_yield,
            self.kappa,
            self.source_rate_scale,
            self.total_volume,
            self.outside_s,
            self.outside_sm,
            self.outside_inert,
        ];
        if self.owner.is_empty()
            || values.iter().any(|v| !v.is_finite())
            || self.radius <= 0.
            || self.rho < 1e-15
            || self.theta <= 0.
            || self.k_s <= 0.
            || self.gamma < 1e-15
            || self.k < 0.
            || self.lambda < 0.
            || self.growth_yield <= 0.
            || self.growth_yield > 1.
            || self.kappa <= 0.
            || self.source_rate_scale < 0.
            || self.total_volume <= 1.
            || self.outside_s < 0.
            || self.outside_sm < 0.
            || self.outside_inert < 0.
        {
            return Err("invalid immutable composed configuration".into());
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialTag {
    pub root: String,
    pub fraction: f64,
    pub young: f64,
    pub mature: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OwnedCell {
    id: u64,
    mesh: MaterialMesh,
    n: [f64; 6],
    p: f64,
    a0: f64,
    r_ref: f64,
    tags: Vec<Vec<MaterialTag>>,
    remeshed: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Bath {
    s: f64,
    sm: f64,
    inert: f64,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(non_snake_case)]
pub struct Ledger {
    pub integrated_Dchem: f64,
    pub integrated_Dmech: f64,
    pub continuous_signed_defect: f64,
    pub continuous_abs_defect: f64,
    pub event_delta_G_sum: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OwnedWorld {
    config: ComposedConfig,
    root_area: f64,
    cells: Vec<OwnedCell>,
    bath: Bath,
    time: f64,
    steps: u64,
    revision: u64,
    initial_g: f64,
    ledger: Ledger,
}
#[derive(Debug, Clone, Serialize)]
pub struct ComposedWorld {
    data: OwnedWorld,
}
#[derive(Debug, Clone, Serialize)]
pub struct ForceComponents {
    pub native: Vec<[f64; 2]>,
    pub coupling: Vec<[f64; 2]>,
    pub osmotic: Vec<[f64; 2]>,
    pub external: Vec<[f64; 2]>,
    pub total: Vec<[f64; 2]>,
}
#[derive(Debug, Clone, Serialize)]
#[allow(non_snake_case)]
pub struct CellDiagnostics {
    pub id: u64,
    pub mesh: MaterialMesh,
    pub n: [f64; 6],
    pub P: f64,
    pub a0: f64,
    pub R_ref: f64,
    pub material_tags: Vec<Vec<MaterialTag>>,
    pub V: f64,
    pub lengths: Vec<f64>,
    pub rest_lengths: Vec<f64>,
    pub force_components: ForceComponents,
    pub velocity: Vec<[f64; 2]>,
    pub forward: [f64; 4],
    pub reverse: [f64; 4],
    pub currents: [f64; 4],
    pub affinities: [f64; 4],
    pub ndot: [f64; 6],
    pub G_bulk: f64,
    pub G_solid: f64,
    pub U_native: f64,
    pub Ec: f64,
    pub Dchem: f64,
    pub Dmech: f64,
    pub gradient_dot_ode: f64,
    pub gradient_identity_error: f64,
    pub g: f64,
    pub tension: f64,
}
#[derive(Debug, Clone, Serialize)]
#[allow(non_snake_case)]
pub struct OutsideDiagnostics {
    pub S: f64,
    pub Sm: f64,
    pub inert: f64,
    pub V: f64,
}
#[derive(Debug, Clone, Serialize)]
#[allow(non_snake_case)]
pub struct GlobalInvariants {
    pub enzyme: f64,
    pub substrate: f64,
    pub a0: f64,
    pub R_ref: f64,
    pub membrane: f64,
}
#[derive(Debug, Clone, Serialize)]
#[allow(non_snake_case)]
pub struct WorldDiagnostics {
    pub time: f64,
    pub steps: u64,
    pub revision: u64,
    pub root_area: f64,
    pub cells: Vec<CellDiagnostics>,
    pub outside: OutsideDiagnostics,
    pub G_total: f64,
    pub global_invariants: GlobalInvariants,
    pub ledger: Ledger,
}
#[derive(Debug, Clone, Default, Serialize)]
pub struct WorkComponents {
    pub native: f64,
    pub coupling: f64,
    pub osmotic: f64,
    pub drag: f64,
}
#[derive(Debug, Clone, Serialize)]
#[allow(non_snake_case)]
pub struct StepRecord {
    pub dt: f64,
    pub before: WorldDiagnostics,
    pub after: WorldDiagnostics,
    pub delta_G: f64,
    pub continuous_defect: f64,
    pub work_components: WorkComponents,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Event {
    Transport {
        cell: u64,
        species: String,
        direction: String,
        amount: f64,
    },
    Growth {
        cell: u64,
        edge: usize,
        extent: f64,
    },
    Maturation {
        cell: u64,
        edge: usize,
        extent: f64,
    },
    Remesh {
        cell: u64,
    },
    Fission {
        cell: u64,
        i: usize,
        j: usize,
    },
}
#[derive(Debug, Clone, Serialize)]
pub struct EdgeMapping {
    pub new_cell: u64,
    pub new_edge: usize,
    pub parent_cell: Option<u64>,
    pub parent_edge: Option<usize>,
    pub start: f64,
    pub end: f64,
    pub creation_tag: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
pub struct EnergyChanges {
    pub bulk_ideal: f64,
    pub solid_standard: f64,
    pub native_elastic: f64,
    pub new_coupling: f64,
}
#[derive(Debug, Clone, Serialize)]
pub struct PairDiagnostics {
    pub forward: f64,
    pub reverse: f64,
    pub log_ratio: f64,
    pub reference_occupancies: [f64; 2],
    pub ensemble_forward_flux: f64,
    pub ensemble_reverse_flux: f64,
    pub ensemble_pair_entropy: f64,
}
#[derive(Debug, Clone, Serialize)]
#[allow(non_snake_case)]
pub struct EventRecord {
    pub event: Event,
    pub before: WorldDiagnostics,
    pub after: WorldDiagnostics,
    pub delta_G: f64,
    pub energy_changes: EnergyChanges,
    pub pair: Option<PairDiagnostics>,
    pub mapping: Vec<EdgeMapping>,
    pub remesh_pullback_error: Option<f64>,
}

fn birth_mesh(c: &ComposedConfig) -> MaterialMesh {
    let mut m = MaterialMesh::seed_regular(
        6,
        c.radius,
        0.,
        0.,
        c.rho,
        0.7,
        LumpedChem::default(),
        LumpedChem::default(),
        0.,
    );
    m.stamp_maturation_coupled_schema();
    m.l_max = 7.;
    m.l_min = 0.05;
    for e in &mut m.edges {
        e.m += 0.1;
        e.m_young = 0.1;
    }
    m
}
fn blank(c: ComposedConfig) -> ComposedWorld {
    let m = birth_mesh(&c);
    let root_area = m.area();
    let tags = (0..m.n())
        .map(|i| {
            vec![MaterialTag {
                root: format!("birth:{i}"),
                fraction: 1.,
                young: m.young_structural_mass(i),
                mature: m.mature_structural_mass(i),
            }]
        })
        .collect();
    let r_ref = (0..m.n()).map(|i| m.rest_length(i)).sum();
    let cell = OwnedCell {
        id: 0,
        mesh: m,
        n: BIRTH_N,
        p: 0.2,
        a0: 0.1,
        r_ref,
        tags,
        remeshed: false,
    };
    ComposedWorld {
        data: OwnedWorld {
            bath: Bath {
                s: c.outside_s,
                sm: c.outside_sm,
                inert: c.outside_inert,
            },
            config: c,
            root_area,
            cells: vec![cell],
            time: 0.,
            steps: 0,
            revision: 0,
            initial_g: 0.,
            ledger: Ledger::default(),
        },
    }
}
fn ideal(n: &[f64], v: f64, theta: f64) -> f64 {
    theta
        * n.iter()
            .map(|x| if *x > 0. { x * ((x / v).ln() - 1.) } else { 0. })
            .sum::<f64>()
}
/// Exact native regularized V4 stretch primitive, including a wholly young edge.
pub fn regularized_stretch(
    length: f64,
    mature: f64,
    young: f64,
    config: &ComposedConfig,
) -> Result<(f64, f64), String> {
    config.validate()?;
    if [length, mature, young].iter().any(|v| !v.is_finite())
        || length <= 0.
        || mature < 0.
        || young < 0.
    {
        return Err("invalid native stretch state".into());
    }
    let r = (mature / config.rho.max(1e-15)).max(1e-15);
    let reference = r.max(0.001);
    let phi = if mature + young > 0. {
        mature / (mature + young)
    } else {
        0.
    };
    let factor = if length < r { 1. } else { phi };
    let primitive = if length <= 4. * reference {
        (length - r).powi(2) / (2. * reference)
    } else {
        (4. * reference - r).powi(2) / (2. * reference) + 4. * (length - 4. * reference)
            - 4. * r * (length / (4. * reference)).ln()
    };
    Ok((
        config.k_s * factor * primitive,
        config.k_s * factor * (length - r) / r.max(length / 4.).max(0.001),
    ))
}
fn source_q(n: &[f64; 6]) -> f64 {
    n[1] + 2. * n[2] + 2. * n[3] + n[4] + n[5]
}
fn sum_invariants(d: &OwnedWorld) -> GlobalInvariants {
    GlobalInvariants {
        enzyme: d.cells.iter().map(|c| c.n[..3].iter().sum::<f64>()).sum(),
        substrate: d.bath.s
            + d.bath.sm
            + d.cells
                .iter()
                .map(|c| source_q(&c.n) + c.mesh.total_structural_mass())
                .sum::<f64>(),
        a0: d.cells.iter().map(|c| c.a0).sum(),
        R_ref: d.cells.iter().map(|c| c.r_ref).sum(),
        membrane: d.cells.iter().map(|c| c.mesh.total_bound_membrane()).sum(),
    }
}
impl ComposedWorld {
    pub fn prepared(config: ComposedConfig) -> Result<Self, String> {
        config.validate()?;
        let mut w = blank(config);
        w.data.initial_g = w.diagnostics()?.G_total;
        w.validate()?;
        Ok(w)
    }
    pub fn config(&self) -> &ComposedConfig {
        &self.data.config
    }
    pub fn time(&self) -> f64 {
        self.data.time
    }
    fn mech(&self, dt: f64) -> MechParams {
        MechParams {
            gamma: self.config().gamma,
            k_s: self.config().k_s,
            kappa_b: 0.,
            k_pi: 0.,
            dt,
        }
    }
    fn domain(&self) -> Result<(), String> {
        let d = &self.data;
        let cfg = &d.config;
        cfg.validate()?;
        if !d.root_area.is_finite()
            || (d.root_area - birth_mesh(cfg).area()).abs() > 1e-10
            || d.cells.is_empty()
            || d.cells.len() > 2
            || [d.bath.s, d.bath.sm, d.bath.inert]
                .iter()
                .any(|v| !v.is_finite() || *v < 0.)
            || d.bath.inert != cfg.outside_inert
        {
            return Err("invalid root/bath ownership".into());
        }
        let mut ids = Vec::new();
        let mut origins: BTreeMap<String, f64> = BTreeMap::new();
        let mut total_v = 0.;
        for cell in &d.cells {
            if ids.contains(&cell.id)
                || cell.tags.len() != cell.mesh.n()
                || cell.mesh.edges.len() != cell.mesh.n()
            {
                return Err("duplicate cell/tag dimensions".into());
            }
            ids.push(cell.id);
            let m = &cell.mesh;
            let n = m.n();
            if (cell.id == 0 && ((!cell.remeshed && n != 6) || (cell.remeshed && n != 12)))
                || (cell.id != 0 && (!cell.remeshed || n != 7))
            {
                return Err("unsupported native hybrid topology/lineage".into());
            }
            if !m.physical_runtime_valid()
                || !m.alive
                || m.signed_area() <= 1e-12
                || !polygon_simple(&m.vertices)
                || cell.n.iter().any(|x| !x.is_finite() || *x <= 0.)
                || ![cell.p, cell.a0, cell.r_ref].iter().all(|x| x.is_finite())
                || cell.p <= 0.
                || cell.a0 < 0.
                || cell.a0 > cell.p
                || cell.r_ref <= 0.
                || (cell.n[..3].iter().sum::<f64>() - cell.p).abs() > 1e-10
            {
                return Err("invalid source/material/simple cell geometry".into());
            }
            let mut expected = birth_mesh(cfg);
            expected.vertices = m.vertices.clone();
            expected.edges = m.edges.clone();
            if serde_json::to_value(expected).map_err(|e| e.to_string())?
                != serde_json::to_value(m).map_err(|e| e.to_string())?
            {
                return Err("unsupported native metadata/chemistry changed".into());
            }
            for i in 0..n {
                if cell.tags[i].is_empty()
                    || m.edges[i].ruptured
                    || m.edges[i].tracer_m != 0.
                    || m.edges[i].tracer_b != 0.
                    || m.edge_length(i) <= 1e-12
                    || !m.edge_length(i).is_finite()
                {
                    return Err("invalid native edge/unsupported rupture".into());
                }
                for j in i + 1..n {
                    if (m.vertices[i][0] - m.vertices[j][0])
                        .hypot(m.vertices[i][1] - m.vertices[j][1])
                        <= 1e-12
                    {
                        return Err("duplicate native vertices".into());
                    }
                }
                let mut young = 0.;
                let mut mature = 0.;
                for tag in &cell.tags[i] {
                    if tag.root.is_empty()
                        || [tag.fraction, tag.young, tag.mature]
                            .iter()
                            .any(|x| !x.is_finite())
                        || tag.fraction <= 0.
                        || tag.fraction > 1.
                        || tag.young < 0.
                        || tag.mature < 0.
                    {
                        return Err("invalid material origin".into());
                    }
                    *origins.entry(tag.root.clone()).or_default() += tag.fraction;
                    young += tag.young;
                    mature += tag.mature;
                }
                if (young - m.edges[i].m_young).abs() > 1e-10
                    || (young + mature - m.edges[i].m).abs() > 1e-10
                {
                    return Err("native material/tag mismatch".into());
                }
            }
            total_v += m.area() / d.root_area;
        }
        if ids != vec![0] && ids != vec![1, 2] {
            return Err("unowned cell identity/fission ancestry".into());
        }
        if origins.values().any(|x| (x - 1.).abs() > 1e-12) || total_v >= cfg.total_volume {
            return Err("lost ancestry/finite bath volume exhausted".into());
        }
        let expected = blank(cfg.clone());
        let a = sum_invariants(d);
        let b = sum_invariants(&expected.data);
        if [
            a.enzyme - b.enzyme,
            a.substrate - b.substrate,
            a.a0 - b.a0,
            a.R_ref - b.R_ref,
            a.membrane - b.membrane,
        ]
        .iter()
        .any(|x| !x.is_finite() || x.abs() > 1e-9)
        {
            return Err("global source/material/anchor conservation defect".into());
        }
        Ok(())
    }
    fn validate(&self) -> Result<(), String> {
        self.domain()?;
        let d = &self.data;
        let l = &d.ledger;
        if [
            d.time,
            d.initial_g,
            l.integrated_Dchem,
            l.integrated_Dmech,
            l.continuous_signed_defect,
            l.continuous_abs_defect,
            l.event_delta_G_sum,
        ]
        .iter()
        .any(|x| !x.is_finite())
            || d.time < 0.
            || l.integrated_Dchem < 0.
            || l.integrated_Dmech < 0.
            || l.continuous_abs_defect < l.continuous_signed_defect.abs() - 1e-12
            || (d.steps == 0) != (d.time == 0.)
            || d.revision < d.steps
        {
            return Err("invalid hybrid cumulative ledger".into());
        }
        let initial = blank(d.config.clone()).diagnostics()?.G_total;
        if (d.initial_g - initial).abs() > 1e-10
            || (self.diagnostics()?.G_total - initial + l.integrated_Dchem + l.integrated_Dmech
                - l.continuous_signed_defect
                - l.event_delta_G_sum)
                .abs()
                > 1e-8
        {
            return Err("global hybrid energy/restart ledger mismatch".into());
        }
        if d.revision == 0 {
            let mut expected = blank(d.config.clone());
            expected.data.initial_g = initial;
            if serde_json::to_value(&self).map_err(|e| e.to_string())?
                != serde_json::to_value(expected).map_err(|e| e.to_string())?
            {
                return Err("unsupported nonprepared initial state".into());
            }
        }
        Ok(())
    }
    pub fn diagnostics(&self) -> Result<WorldDiagnostics, String> {
        self.domain()?;
        let d = &self.data;
        let p = &d.config;
        let outside_v = p.total_volume
            - d.cells
                .iter()
                .map(|c| c.mesh.area() / d.root_area)
                .sum::<f64>();
        let outside_density = (d.bath.s + d.bath.sm + d.bath.inert) / outside_v;
        let mut cells = Vec::new();
        let outside_g = ideal(&[d.bath.s, d.bath.sm, d.bath.inert], outside_v, p.theta);
        let mut total = outside_g;
        for c in &d.cells {
            let m = &c.mesh;
            let count = m.n();
            let volume = m.area() / d.root_area;
            let mut z = c.n;
            for x in &mut z {
                *x /= volume;
            }
            let lengths: Vec<f64> = (0..count).map(|i| m.edge_length(i)).collect();
            let rest_lengths: Vec<f64> = (0..count).map(|i| m.rest_length(i)).collect();
            let extension = (lengths.iter().sum::<f64>() - rest_lengths.iter().sum::<f64>())
                / c.r_ref
                - p.lambda * (c.n[1] + c.n[2] - c.a0) / c.p;
            let ec = p.k * c.p * extension * extension / 2.;
            let g = -p.k * p.lambda * extension;
            let tension = p.k * c.p * extension / c.r_ref;
            let mut native_u = 0.;
            let mut solid = 0.;
            let mut storage_grad = vec![[0.; 2]; count];
            let mut coupling = vec![[0.; 2]; count];
            let mut osmotic = vec![[0.; 2]; count];
            let pressure = p.theta * (c.n.iter().sum::<f64>() / volume - outside_density);
            for i in 0..count {
                let (u, t) = regularized_stretch(
                    lengths[i],
                    m.mature_structural_mass(i),
                    m.young_structural_mass(i),
                    p,
                )?;
                native_u += u;
                solid += p.mu_y * m.young_structural_mass(i) + p.mu_m * m.mature_structural_mass(i);
                let j = (i + 1) % count;
                let prev = (i + count - 1) % count;
                for axis in 0..2 {
                    let unit = (m.vertices[j][axis] - m.vertices[i][axis]) / lengths[i];
                    storage_grad[i][axis] -= t * unit;
                    storage_grad[j][axis] += t * unit;
                    coupling[i][axis] += tension * unit;
                    coupling[j][axis] -= tension * unit;
                }
                let grad_area = [
                    (m.vertices[j][1] - m.vertices[prev][1]) / 2.,
                    (m.vertices[prev][0] - m.vertices[j][0]) / 2.,
                ];
                for axis in 0..2 {
                    osmotic[i][axis] = pressure * grad_area[axis] / d.root_area;
                }
            }
            let native = mesh_mechanics::compute_forces(m, &self.mech(0.));
            let external: Vec<[f64; 2]> = (0..count)
                .map(|i| {
                    [
                        coupling[i][0] + osmotic[i][0],
                        coupling[i][1] + osmotic[i][1],
                    ]
                })
                .collect();
            let full: Vec<[f64; 2]> = (0..count)
                .map(|i| [native[i][0] + external[i][0], native[i][1] + external[i][1]])
                .collect();
            let velocity: Vec<[f64; 2]> = full
                .iter()
                .map(|f| [f[0] / p.gamma, f[1] / p.gamma])
                .collect();
            let bp = [z[4], z[5] * z[0], z[5] * z[1], z[2]];
            let bm = [z[5], z[1], z[2], z[0] * z[3]];
            if bp.iter().chain(&bm).any(|x| !x.is_finite() || *x <= 0.) {
                return Err("bare source rate overflow/underflow".into());
            }
            let mut forward = [0.; 4];
            let mut reverse = [0.; 4];
            let mut currents = [0.; 4];
            let mut affinities = [0.; 4];
            for j in 0..4 {
                affinities[j] = bp[j].ln() - bm[j].ln() - NU_A[j] * g / p.theta;
                forward[j] =
                    volume * p.source_rate_scale * bp[j] * (-NU_A[j] * g / (2. * p.theta)).exp();
                reverse[j] =
                    volume * p.source_rate_scale * bm[j] * (NU_A[j] * g / (2. * p.theta)).exp();
                currents[j] = forward[j] - reverse[j];
            }
            let mut ndot = [0.; 6];
            for i in 0..6 {
                for j in 0..4 {
                    ndot[i] += NU[i][j] * currents[j];
                }
            }
            let dchem = p.theta
                * currents
                    .iter()
                    .zip(affinities)
                    .map(|(j, a)| j * a)
                    .sum::<f64>();
            let dmech = p.gamma * velocity.iter().flatten().map(|v| v * v).sum::<f64>();
            let chem_dot = ndot
                .iter()
                .enumerate()
                .map(|(i, j)| j * (p.theta * z[i].ln() + if i == 1 || i == 2 { g } else { 0. }))
                .sum::<f64>();
            let spatial_dot = (0..count)
                .map(|i| {
                    (0..2)
                        .map(|a| {
                            (storage_grad[i][a] - coupling[i][a] - osmotic[i][a]) * velocity[i][a]
                        })
                        .sum::<f64>()
                })
                .sum::<f64>();
            let gradient_dot_ode = chem_dot + spatial_dot;
            let error = gradient_dot_ode + dchem + dmech;
            let bulk = ideal(&c.n, volume, p.theta);
            total += bulk + solid + native_u + ec;
            if [bulk, solid, native_u, ec, g, tension, dchem, dmech, error]
                .iter()
                .any(|x| !x.is_finite())
                || dchem < -1e-12
                || dmech < 0.
                || forward
                    .iter()
                    .chain(&reverse)
                    .any(|x| !x.is_finite() || *x < 0. || (p.source_rate_scale > 0. && *x == 0.))
                || velocity
                    .iter()
                    .flatten()
                    .chain(ndot.iter())
                    .any(|x| !x.is_finite())
            {
                return Err("nonfinite composed rate/energy/force".into());
            }
            cells.push(CellDiagnostics {
                id: c.id,
                mesh: m.clone(),
                n: c.n,
                P: c.p,
                a0: c.a0,
                R_ref: c.r_ref,
                material_tags: c.tags.clone(),
                V: volume,
                lengths,
                rest_lengths,
                force_components: ForceComponents {
                    native,
                    coupling,
                    osmotic,
                    external,
                    total: full,
                },
                velocity,
                forward,
                reverse,
                currents,
                affinities,
                ndot,
                G_bulk: bulk,
                G_solid: solid,
                U_native: native_u,
                Ec: ec,
                Dchem: dchem,
                Dmech: dmech,
                gradient_dot_ode,
                gradient_identity_error: error,
                g,
                tension,
            });
        }
        if !total.is_finite() {
            return Err("nonfinite total hybrid free energy".into());
        }
        Ok(WorldDiagnostics {
            time: d.time,
            steps: d.steps,
            revision: d.revision,
            root_area: d.root_area,
            cells,
            outside: OutsideDiagnostics {
                S: d.bath.s,
                Sm: d.bath.sm,
                inert: d.bath.inert,
                V: outside_v,
            },
            G_total: total,
            global_invariants: sum_invariants(d),
            ledger: d.ledger.clone(),
        })
    }
    fn cap(diag: &WorldDiagnostics) -> Result<(), String> {
        if diag
            .cells
            .iter()
            .flat_map(|c| &c.force_components.external)
            .any(|f| f[0].hypot(f[1]) > MAX_EXTERNAL_FORCE_PER_VERTEX)
        {
            return Err("native external-force bound exceeded".into());
        }
        Ok(())
    }
    pub fn try_step(&self, dt: f64) -> Result<(Self, StepRecord), String> {
        self.validate()?;
        if !dt.is_finite() || dt <= 0. || dt > 0.005 {
            return Err("finite bounded continuous step required".into());
        }
        let before = self.diagnostics()?;
        Self::cap(&before)?;
        let mut t = self.clone();
        for (cell, diag) in t.data.cells.iter_mut().zip(&before.cells) {
            for i in 0..6 {
                cell.n[i] += dt * diag.ndot[i];
            }
            if !mesh_mechanics::mechanics_step_with_external_forces(
                &mut cell.mesh,
                &self.mech(dt),
                &diag.force_components.external,
            ) {
                return Err("native vertex trial rejected atomically".into());
            }
        }
        let provisional = t.diagnostics()?;
        Self::cap(&provisional)?;
        let delta = provisional.G_total - before.G_total;
        if delta > 1e-12 {
            return Err("continuous hybrid free energy increased".into());
        }
        let dc = dt * before.cells.iter().map(|c| c.Dchem).sum::<f64>();
        let dm = dt * before.cells.iter().map(|c| c.Dmech).sum::<f64>();
        let defect = delta + dc + dm;
        t.data.ledger.integrated_Dchem += dc;
        t.data.ledger.integrated_Dmech += dm;
        t.data.ledger.continuous_signed_defect += defect;
        t.data.ledger.continuous_abs_defect += defect.abs();
        t.data.time += dt;
        t.data.steps = t.data.steps.checked_add(1).ok_or("step overflow")?;
        t.data.revision = t.data.revision.checked_add(1).ok_or("revision overflow")?;
        t.validate()?;
        let after = t.diagnostics()?;
        let mut work = WorkComponents::default();
        for (a, b) in before.cells.iter().zip(&after.cells) {
            for i in 0..a.mesh.n() {
                for axis in 0..2 {
                    let dx = b.mesh.vertices[i][axis] - a.mesh.vertices[i][axis];
                    work.native += a.force_components.native[i][axis] * dx;
                    work.coupling += a.force_components.coupling[i][axis] * dx;
                    work.osmotic += a.force_components.osmotic[i][axis] * dx;
                }
            }
        }
        work.drag = dm;
        Ok((
            t,
            StepRecord {
                dt,
                before,
                after,
                delta_G: delta,
                continuous_defect: defect,
                work_components: work,
            },
        ))
    }
    fn cell_index(&self, id: u64) -> Result<usize, String> {
        self.data
            .cells
            .iter()
            .position(|c| c.id == id)
            .ok_or("unknown cell owner".into())
    }
    fn sync_material(cell: &mut OwnedCell, edge: usize) {
        let young: f64 = cell.tags[edge].iter().map(|t| t.young).sum();
        let mature: f64 = cell.tags[edge].iter().map(|t| t.mature).sum();
        cell.mesh.edges[edge].m = young + mature;
        cell.mesh.edges[edge].m_young = young;
    }
    pub fn try_event(&self, event: Event) -> Result<(Self, EventRecord), String> {
        self.validate()?;
        let before = self.diagnostics()?;
        Self::cap(&before)?;
        let mut t = self.clone();
        let revision = t.data.revision.checked_add(1).ok_or("revision overflow")?;
        let mut mapping = Vec::new();
        let mut pullback = None;
        match &event {
            Event::Transport {
                cell,
                species,
                direction,
                amount,
            } => {
                if !amount.is_finite() || *amount <= 0. {
                    return Err("positive finite transport amount required".into());
                }
                let q = match direction.as_str() {
                    "in" => *amount,
                    "out" => -*amount,
                    _ => return Err("invalid transport direction".into()),
                };
                let index = t.cell_index(*cell)?;
                match species.as_str() {
                    "S" => {
                        t.data.cells[index].n[4] += q;
                        t.data.bath.s -= q;
                    }
                    "Sm" => {
                        t.data.cells[index].n[5] += q;
                        t.data.bath.sm -= q;
                    }
                    _ => return Err("unsupported transported species".into()),
                }
            }
            Event::Growth { cell, edge, extent } => {
                if !extent.is_finite() || *extent == 0. {
                    return Err("finite nonzero growth extent required".into());
                }
                let idx = t.cell_index(*cell)?;
                let c = &mut t.data.cells[idx];
                if *edge >= c.mesh.n() {
                    return Err("unknown growth edge".into());
                }
                let dm = t.data.config.growth_yield * extent;
                c.n[5] -= extent;
                c.n[4] += (1. - t.data.config.growth_yield) * extent;
                if dm >= 0. {
                    c.tags[*edge].push(MaterialTag {
                        root: format!("growth:{revision}:{cell}:{edge}"),
                        fraction: 1.,
                        young: dm,
                        mature: 0.,
                    });
                } else {
                    let donor: f64 = c.tags[*edge].iter().map(|x| x.young).sum();
                    if -dm > donor {
                        return Err("growth material donor exhausted".into());
                    }
                    let fraction = (donor + dm) / donor;
                    for tag in &mut c.tags[*edge] {
                        tag.young *= fraction;
                    }
                }
                Self::sync_material(c, *edge);
            }
            Event::Maturation { cell, edge, extent } => {
                if !extent.is_finite() || *extent == 0. {
                    return Err("finite nonzero maturation extent required".into());
                }
                let idx = t.cell_index(*cell)?;
                let c = &mut t.data.cells[idx];
                if *edge >= c.mesh.n() {
                    return Err("unknown maturation edge".into());
                }
                let donor: f64 = c.tags[*edge]
                    .iter()
                    .map(|x| if *extent >= 0. { x.young } else { x.mature })
                    .sum();
                if extent.abs() > donor {
                    return Err("maturation material donor exhausted".into());
                }
                for tag in &mut c.tags[*edge] {
                    let q = extent * (if *extent >= 0. { tag.young } else { tag.mature }) / donor;
                    tag.young -= q;
                    tag.mature += q;
                }
                Self::sync_material(c, *edge);
            }
            Event::Remesh { cell } => {
                let idx = t.cell_index(*cell)?;
                let c = &mut t.data.cells[idx];
                if (0..c.mesh.n()).any(|i| c.mesh.rest_length(i) / 2. < 0.001) {
                    return Err("native absolute-reference floor forbids neutral remesh".into());
                }
                if c.mesh.n() != 6
                    || c.remeshed
                    || (0..6).any(|i| c.mesh.edge_length(i) <= 7. || c.mesh.edge_length(i) > 14.)
                {
                    return Err("unsupported native single-midpoint remesh".into());
                }
                let old = c.clone();
                let splits = mesh_mechanics::remesh_split(&mut c.mesh);
                if splits != 6 || c.mesh.n() != 12 {
                    return Err("native midpoint split count mismatch".into());
                }
                c.tags.clear();
                for (i, tags) in old.tags.iter().enumerate() {
                    for part in 0..2 {
                        c.tags.push(
                            tags.iter()
                                .map(|x| MaterialTag {
                                    root: x.root.clone(),
                                    fraction: x.fraction / 2.,
                                    young: x.young / 2.,
                                    mature: x.mature / 2.,
                                })
                                .collect(),
                        );
                        mapping.push(EdgeMapping {
                            new_cell: *cell,
                            new_edge: 2 * i + part,
                            parent_cell: Some(*cell),
                            parent_edge: Some(i),
                            start: part as f64 / 2.,
                            end: (part + 1) as f64 / 2.,
                            creation_tag: None,
                        });
                    }
                }
                for i in 0..6 {
                    for axis in 0..2 {
                        if c.mesh.vertices[2 * i][axis] != old.mesh.vertices[i][axis]
                            || c.mesh.vertices[2 * i + 1][axis]
                                != 0.5
                                    * (old.mesh.vertices[i][axis]
                                        + old.mesh.vertices[(i + 1) % 6][axis])
                        {
                            return Err("native midpoint correspondence failure".into());
                        }
                    }
                }
                c.remeshed = true;
                let after = t.diagnostics()?;
                let b = &before.cells[idx];
                let a = &after.cells[idx];
                let mut error: f64 = 0.;
                for i in 0..6 {
                    for axis in 0..2 {
                        let force = a.force_components.total[2 * i][axis]
                            + 0.5 * a.force_components.total[(2 * i + 11) % 12][axis]
                            + 0.5 * a.force_components.total[2 * i + 1][axis];
                        error = error.max((force - b.force_components.total[i][axis]).abs());
                    }
                }
                if error > 1e-8
                    || (after.G_total - before.G_total).abs() > 1e-8
                    || (a.mesh.area() - b.mesh.area()).abs() > 1e-8
                {
                    return Err("native remesh storage/area/pullback defect".into());
                }
                pullback = Some(error);
            }
            Event::Fission { cell, i, j } => {
                let idx = t.cell_index(*cell)?;
                let parent = t.data.cells[idx].clone();
                let count = parent.mesh.n();
                if t.data.cells.len() != 1
                    || *cell != 0
                    || count != 12
                    || !parent.remeshed
                    || *i != 0
                    || *j != 6
                {
                    return Err("unsupported prescribed two-loop cut".into());
                }
                let length = (parent.mesh.vertices[*j][0] - parent.mesh.vertices[*i][0])
                    .hypot(parent.mesh.vertices[*j][1] - parent.mesh.vertices[*i][1]);
                let mass = t.config().rho * length;
                let debit = 2. * mass / t.config().growth_yield;
                let mut n = parent.n;
                n[5] -= debit;
                n[4] += (1. - t.config().growth_yield) * debit;
                let paths = [
                    (*i..=*j).collect::<Vec<_>>(),
                    (*j..count).chain(0..=*i).collect::<Vec<_>>(),
                ];
                let mut children = Vec::new();
                for (which, path) in paths.iter().enumerate() {
                    let id = which as u64 + 1;
                    let mut mesh = parent.mesh.clone();
                    mesh.vertices = path.iter().map(|k| parent.mesh.vertices[*k]).collect();
                    mesh.edges = path[..path.len() - 1]
                        .iter()
                        .map(|k| parent.mesh.edges[*k])
                        .collect();
                    let mut tags: Vec<Vec<MaterialTag>> = path[..path.len() - 1]
                        .iter()
                        .map(|k| parent.tags[*k].clone())
                        .collect();
                    for (edge, old_edge) in path[..path.len() - 1].iter().enumerate() {
                        mapping.push(EdgeMapping {
                            new_cell: id,
                            new_edge: edge,
                            parent_cell: Some(*cell),
                            parent_edge: Some(*old_edge),
                            start: 0.,
                            end: 1.,
                            creation_tag: None,
                        });
                    }
                    let root = format!("closure:{revision}:{id}");
                    let mut closure = crate::material_mesh::MeshEdge::default();
                    closure.m = mass;
                    closure.m_young = mass;
                    mesh.edges.push(closure);
                    tags.push(vec![MaterialTag {
                        root: root.clone(),
                        fraction: 1.,
                        young: mass,
                        mature: 0.,
                    }]);
                    mapping.push(EdgeMapping {
                        new_cell: id,
                        new_edge: mesh.n() - 1,
                        parent_cell: None,
                        parent_edge: None,
                        start: 0.,
                        end: 1.,
                        creation_tag: Some(root),
                    });
                    let fraction = mesh.area() / parent.mesh.area();
                    let mut child_n = n;
                    for v in &mut child_n {
                        *v *= fraction;
                    }
                    children.push(OwnedCell {
                        id,
                        mesh,
                        n: child_n,
                        p: parent.p * fraction,
                        a0: parent.a0 * fraction,
                        r_ref: parent.r_ref * fraction,
                        tags,
                        remeshed: parent.remeshed,
                    });
                }
                if (children.iter().map(|c| c.mesh.area()).sum::<f64>() - parent.mesh.area()).abs()
                    > 1e-8
                {
                    return Err("prescribed daughter area mismatch".into());
                }
                t.data.cells = children;
            }
        }
        t.data.revision = revision;
        let provisional = t.diagnostics()?;
        Self::cap(&provisional)?;
        let delta = provisional.G_total - before.G_total;
        let energy_changes = energy_changes(&before, &provisional);
        let pair = if matches!(&event, Event::Remesh { .. }) {
            None
        } else {
            Some(pair_diagnostics(delta, t.config())?)
        };
        t.data.ledger.event_delta_G_sum += delta;
        t.validate()?;
        let after = t.diagnostics()?;
        Ok((
            t,
            EventRecord {
                event,
                before,
                after,
                delta_G: delta,
                energy_changes,
                pair,
                mapping,
                remesh_pullback_error: pullback,
            },
        ))
    }
    pub fn restart_json(&self) -> Result<String, String> {
        self.validate()?;
        serde_json::to_string(self).map_err(|e| e.to_string())
    }
    pub fn from_restart_json(json: &str, expected: &ComposedConfig) -> Result<Self, String> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            data: OwnedWorld,
        }
        expected.validate()?;
        let raw: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let wire: Wire = serde_json::from_value(raw.clone()).map_err(|e| e.to_string())?;
        let w = Self { data: wire.data };
        if w.config() != expected || serde_json::to_value(&w).map_err(|e| e.to_string())? != raw {
            return Err("restart config/shape/birth owner mismatch".into());
        }
        w.validate()?;
        Ok(w)
    }
}
fn energy_changes(a: &WorldDiagnostics, b: &WorldDiagnostics) -> EnergyChanges {
    // Outside theta is not stored in diagnostics; recover its actual contribution
    // by subtracting every explicitly reported cell store from total G.
    let bulk = |d: &WorldDiagnostics| {
        d.G_total
            - d.cells
                .iter()
                .map(|c| c.G_solid + c.U_native + c.Ec)
                .sum::<f64>()
    };
    EnergyChanges {
        bulk_ideal: bulk(b) - bulk(a),
        solid_standard: b.cells.iter().map(|c| c.G_solid).sum::<f64>()
            - a.cells.iter().map(|c| c.G_solid).sum::<f64>(),
        native_elastic: b.cells.iter().map(|c| c.U_native).sum::<f64>()
            - a.cells.iter().map(|c| c.U_native).sum::<f64>(),
        new_coupling: b.cells.iter().map(|c| c.Ec).sum::<f64>()
            - a.cells.iter().map(|c| c.Ec).sum::<f64>(),
    }
}
pub fn pair_diagnostics(delta: f64, c: &ComposedConfig) -> Result<PairDiagnostics, String> {
    c.validate()?;
    let x = delta / c.theta;
    if !x.is_finite() {
        return Err("nonfinite finite-event affinity".into());
    }
    let z = (-x.abs()).exp();
    let low = c.kappa * z / (1. + z);
    let high = c.kappa / (1. + z);
    let (forward, reverse) = if x >= 0. { (low, high) } else { (high, low) };
    if !forward.is_finite() || !reverse.is_finite() || forward <= 0. || reverse <= 0. {
        return Err("finite-pair underflow/overflow".into());
    }
    Ok(PairDiagnostics {
        forward,
        reverse,
        log_ratio: -x,
        reference_occupancies: [0.5, 0.5],
        ensemble_forward_flux: 0.5 * forward,
        ensemble_reverse_flux: 0.5 * reverse,
        ensemble_pair_entropy: 0.5 * (forward - reverse) * (-x),
    })
}
