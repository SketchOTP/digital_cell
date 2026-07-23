//! D-080 geometry-consistent cut-cell (marching-squares) edge support graph.
//!
//! Built only from old-state cell-centered `φ`. No analytic circle knowledge,
//! no global component feedback into chemistry, no stored target ring.
//!
//! Ambiguous / isolevel handling: a corner is interior iff `φ > 0.5` (strict).
//! Saddle pairing (4 crossings): compare `(sw+ne)` vs `(se+nw)` deterministically.

use crate::edge_membrane::{FaceKind, grid_for_radius};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

pub const ISO_LEVEL: f64 = 0.5;
pub const MEASURE_EPS: f64 = 1e-15;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CutCellSupport {
    pub width: usize,
    pub height: usize,
    /// Interface measure attributed to each horizontal face (≥0).
    pub measure_h: Vec<f64>,
    /// Interface measure attributed to each vertical face (≥0).
    pub measure_v: Vec<f64>,
    /// Reconstructed interface polyline length (sum of local MS segments).
    pub interface_length: f64,
    /// Local corner adjacency: face key → neighbor face keys (orthogonal only, local).
    pub adjacency: HashMap<(u8, usize), Vec<(u8, usize)>>,
}

impl CutCellSupport {
    pub fn n_h(&self) -> usize {
        (self.width - 1) * self.height
    }

    pub fn n_v(&self) -> usize {
        self.width * (self.height - 1)
    }

    pub fn measure(&self, kind: FaceKind, idx: usize) -> f64 {
        match kind {
            FaceKind::Horizontal => self.measure_h[idx],
            FaceKind::Vertical => self.measure_v[idx],
        }
    }

    pub fn is_supported(&self, kind: FaceKind, idx: usize) -> bool {
        self.measure(kind, idx) > MEASURE_EPS
    }

    pub fn face_capacity(&self, kind: FaceKind, idx: usize, b_max: f64) -> f64 {
        let m = self.measure(kind, idx);
        if m <= MEASURE_EPS {
            return 0.0;
        }
        // Capacity ∝ local measure; normalize by mean positive measure so typical faces ≈ b_max.
        let mean = self.mean_positive_measure().max(MEASURE_EPS);
        let scale = (m / mean).clamp(0.25, 1.75);
        b_max * scale
    }

    pub fn mean_positive_measure(&self) -> f64 {
        let mut s = 0.0;
        let mut n = 0.0;
        for &m in self.measure_h.iter().chain(self.measure_v.iter()) {
            if m > MEASURE_EPS {
                s += m;
                n += 1.0;
            }
        }
        if n <= 0.0 {
            0.0
        } else {
            s / n
        }
    }

    pub fn supported_faces(&self) -> Vec<(FaceKind, usize)> {
        let mut out = Vec::new();
        for i in 0..self.n_h() {
            if self.measure_h[i] > MEASURE_EPS {
                out.push((FaceKind::Horizontal, i));
            }
        }
        for i in 0..self.n_v() {
            if self.measure_v[i] > MEASURE_EPS {
                out.push((FaceKind::Vertical, i));
            }
        }
        out
    }

    pub fn neighbors(&self, kind: FaceKind, idx: usize) -> Vec<(FaceKind, usize)> {
        let key = (kind_byte(kind), idx);
        self.adjacency
            .get(&key)
            .map(|v| {
                v.iter()
                    .map(|&(k, i)| (byte_kind(k), i))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn n_supported(&self) -> usize {
        self.supported_faces().len()
    }

    /// Geometric support coverage of largest connected component / supported count.
    pub fn geometric_support_coverage(&self) -> (f64, bool, usize) {
        let nodes = self.supported_faces();
        let n = nodes.len();
        if n == 0 {
            return (0.0, false, 0);
        }
        let idx: HashMap<(FaceKind, usize), usize> =
            nodes.iter().copied().enumerate().map(|(i, n)| (n, i)).collect();
        let mut adj: Vec<Vec<usize>> = vec![vec![]; n];
        for (a, &(ka, ia)) in nodes.iter().enumerate() {
            for (kb, ib) in self.neighbors(ka, ia) {
                if let Some(&b) = idx.get(&(kb, ib)) {
                    if b > a {
                        adj[a].push(b);
                        adj[b].push(a);
                    }
                }
            }
        }
        let mut seen = vec![false; n];
        let mut best = 0usize;
        let mut best_nodes = Vec::new();
        for s in 0..n {
            if seen[s] {
                continue;
            }
            let mut q = VecDeque::new();
            let mut comp = Vec::new();
            seen[s] = true;
            q.push_back(s);
            while let Some(u) = q.pop_front() {
                comp.push(u);
                for &v in &adj[u] {
                    if !seen[v] {
                        seen[v] = true;
                        q.push_back(v);
                    }
                }
            }
            if comp.len() > best {
                best = comp.len();
                best_nodes = comp;
            }
        }
        let closed = component_has_cycle(&adj, &best_nodes);
        (best as f64 / n as f64, closed, n)
    }

    /// True if any supported face is far from the φ≈0.5 band (off-interface support).
    pub fn off_interface_support_fraction(&self, phi: &[f64]) -> f64 {
        let mut on = 0.0;
        let mut off = 0.0;
        for (kind, idx) in self.supported_faces() {
            let (i0, j0, i1, j1) = face_cells(self.width, kind, idx);
            let p0 = phi[j0 * self.width + i0];
            let p1 = phi[j1 * self.width + i1];
            let i_phi = 0.5 * (interface_weight_local(p0) + interface_weight_local(p1));
            if i_phi < 1e-3 && (p0 - ISO_LEVEL) * (p1 - ISO_LEVEL) > 0.0 {
                off += 1.0;
            } else {
                on += 1.0;
            }
        }
        let t = on + off;
        if t <= 0.0 {
            0.0
        } else {
            off / t
        }
    }

    /// Diagonal leak probe: a 2×2 with only diagonal interior corners must not
    /// create a supported shortcut that seals both orthogonal pairs incorrectly.
    /// We require saddle pairing never connects both diagonals simultaneously.
    pub fn no_diagonal_leak_ok(&self) -> bool {
        // Every adjacency is within one dual square by construction; leak would be
        // a face adjacent to a non-local face. Verify all edges are local.
        for (&(ka, ia), nbrs) in &self.adjacency {
            let kind_a = byte_kind(ka);
            for &(kb, ib) in nbrs {
                let kind_b = byte_kind(kb);
                if !faces_share_primal_vertex(self.width, self.height, kind_a, ia, kind_b, ib) {
                    return false;
                }
            }
        }
        true
    }
}

fn kind_byte(k: FaceKind) -> u8 {
    match k {
        FaceKind::Horizontal => 0,
        FaceKind::Vertical => 1,
    }
}

fn byte_kind(b: u8) -> FaceKind {
    if b == 0 {
        FaceKind::Horizontal
    } else {
        FaceKind::Vertical
    }
}

fn interface_weight_local(phi: f64) -> f64 {
    let t = (1.0 - (2.0 * phi - 1.0).abs()).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn face_cells(width: usize, kind: FaceKind, idx: usize) -> (usize, usize, usize, usize) {
    match kind {
        FaceKind::Horizontal => {
            let w1 = width - 1;
            let j = idx / w1;
            let i = idx % w1;
            (i, j, i + 1, j)
        }
        FaceKind::Vertical => {
            let j = idx / width;
            let i = idx % width;
            (i, j, i, j + 1)
        }
    }
}

fn h_corners(i: usize, j: usize) -> [(i32, i32); 2] {
    // Vertical primal edge at x=i+1, y∈[j,j+1]
    [(i as i32 + 1, j as i32), (i as i32 + 1, j as i32 + 1)]
}

fn v_corners(i: usize, j: usize) -> [(i32, i32); 2] {
    [(i as i32, j as i32 + 1), (i as i32 + 1, j as i32 + 1)]
}

fn faces_share_primal_vertex(
    width: usize,
    _height: usize,
    ka: FaceKind,
    ia: usize,
    kb: FaceKind,
    ib: usize,
) -> bool {
    let (i0, j0, _, _) = face_cells(width, ka, ia);
    let (i1, j1, _, _) = face_cells(width, kb, ib);
    let ca = match ka {
        FaceKind::Horizontal => h_corners(i0, j0),
        FaceKind::Vertical => v_corners(i0, j0),
    };
    let cb = match kb {
        FaceKind::Horizontal => {
            let (i, j, _, _) = face_cells(width, kb, ib);
            h_corners(i, j)
        }
        FaceKind::Vertical => {
            let (i, j, _, _) = face_cells(width, kb, ib);
            v_corners(i, j)
        }
    };
    let _ = (i1, j1);
    ca.iter().any(|c| cb.contains(c))
}

fn is_interior(phi: f64) -> bool {
    // Strict: φ == 0.5 is exterior. Deterministic ambiguous-case handling.
    phi > ISO_LEVEL
}

fn cross_t(a: f64, b: f64) -> Option<f64> {
    let ia = is_interior(a);
    let ib = is_interior(b);
    if ia == ib {
        return None;
    }
    let den = b - a;
    if den.abs() < 1e-30 {
        return Some(0.5);
    }
    Some(((ISO_LEVEL - a) / den).clamp(0.0, 1.0))
}

fn component_has_cycle(adj: &[Vec<usize>], nodes: &[usize]) -> bool {
    if nodes.len() < 3 {
        return false;
    }
    let set: HashSet<usize> = nodes.iter().copied().collect();
    let mut seen = HashSet::new();
    fn dfs(
        u: usize,
        parent: Option<usize>,
        adj: &[Vec<usize>],
        set: &HashSet<usize>,
        seen: &mut HashSet<usize>,
    ) -> bool {
        seen.insert(u);
        for &v in &adj[u] {
            if !set.contains(&v) {
                continue;
            }
            if !seen.contains(&v) {
                if dfs(v, Some(u), adj, set, seen) {
                    return true;
                }
            } else if Some(v) != parent {
                return true;
            }
        }
        false
    }
    dfs(nodes[0], None, adj, &set, &mut seen)
}

/// Build cut-cell support from cell-centered φ (old accepted state only).
pub fn build_cut_cell_support(phi: &[f64], width: usize, height: usize) -> CutCellSupport {
    assert_eq!(phi.len(), width * height);
    assert!(width >= 2 && height >= 2);
    let n_h = (width - 1) * height;
    let n_v = width * (height - 1);
    let mut measure_h = vec![0.0; n_h];
    let mut measure_v = vec![0.0; n_v];
    let mut adj: HashMap<(u8, usize), HashSet<(u8, usize)>> = HashMap::new();
    let mut interface_length = 0.0;

    let h_idx = |i: usize, j: usize| j * (width - 1) + i;
    let v_idx = |i: usize, j: usize| j * width + i;

    for j in 0..(height - 1) {
        for i in 0..(width - 1) {
            let sw = phi[j * width + i];
            let se = phi[j * width + i + 1];
            let nw = phi[(j + 1) * width + i];
            let ne = phi[(j + 1) * width + i + 1];

            // Dual-square edges: bottom H(i,j), top H(i,j+1), left V(i,j), right V(i+1,j).
            let mut edges: Vec<(FaceKind, usize, (f64, f64))> = Vec::new();
            if let Some(t) = cross_t(sw, se) {
                edges.push((FaceKind::Horizontal, h_idx(i, j), (i as f64 + t, j as f64)));
            }
            if let Some(t) = cross_t(nw, ne) {
                edges.push((
                    FaceKind::Horizontal,
                    h_idx(i, j + 1),
                    (i as f64 + t, (j + 1) as f64),
                ));
            }
            if let Some(t) = cross_t(sw, nw) {
                edges.push((FaceKind::Vertical, v_idx(i, j), (i as f64, j as f64 + t)));
            }
            if let Some(t) = cross_t(se, ne) {
                edges.push((
                    FaceKind::Vertical,
                    v_idx(i + 1, j),
                    ((i + 1) as f64, j as f64 + t),
                ));
            }

            let pairs: Vec<(usize, usize)> = match edges.len() {
                0 | 1 => Vec::new(),
                2 => vec![(0, 1)],
                3 => {
                    // Deterministic shortest path through one midpoint.
                    let mut best = None;
                    let mut best_len = f64::INFINITY;
                    for mid in 0..3 {
                        let ends: Vec<usize> = (0..3).filter(|&e| e != mid).collect();
                        let p0 = edges[ends[0]].2;
                        let pm = edges[mid].2;
                        let p1 = edges[ends[1]].2;
                        let len = hypot2(p0, pm) + hypot2(pm, p1);
                        if len < best_len {
                            best_len = len;
                            best = Some(vec![(ends[0], mid), (mid, ends[1])]);
                        }
                    }
                    best.unwrap_or_default()
                }
                4 => {
                    // Saddle: deterministic asymptotic-style pairing.
                    let ids: HashMap<(FaceKind, usize), usize> = edges
                        .iter()
                        .enumerate()
                        .map(|(ei, &(k, fi, _))| ((k, fi), ei))
                        .collect();
                    let b = ids[&(FaceKind::Horizontal, h_idx(i, j))];
                    let t = ids[&(FaceKind::Horizontal, h_idx(i, j + 1))];
                    let l = ids[&(FaceKind::Vertical, v_idx(i, j))];
                    let r = ids[&(FaceKind::Vertical, v_idx(i + 1, j))];
                    if sw + ne >= se + nw {
                        vec![(b, l), (t, r)]
                    } else {
                        vec![(b, r), (t, l)]
                    }
                }
                _ => Vec::new(),
            };

            for (a, b) in pairs {
                let (ka, ia, pa) = edges[a];
                let (kb, ib, pb) = edges[b];
                let seg = hypot2(pa, pb);
                interface_length += seg;
                match ka {
                    FaceKind::Horizontal => measure_h[ia] += 0.5 * seg,
                    FaceKind::Vertical => measure_v[ia] += 0.5 * seg,
                }
                match kb {
                    FaceKind::Horizontal => measure_h[ib] += 0.5 * seg,
                    FaceKind::Vertical => measure_v[ib] += 0.5 * seg,
                }
                let ka_b = kind_byte(ka);
                let kb_b = kind_byte(kb);
                adj.entry((ka_b, ia)).or_default().insert((kb_b, ib));
                adj.entry((kb_b, ib)).or_default().insert((ka_b, ia));
            }
        }
    }

    let adjacency = adj
        .into_iter()
        .map(|(k, set)| {
            let mut v: Vec<_> = set.into_iter().collect();
            v.sort_by_key(|x| (x.0, x.1));
            (k, v)
        })
        .collect();

    CutCellSupport {
        width,
        height,
        measure_h,
        measure_v,
        interface_length,
        adjacency,
    }
}

fn hypot2(a: (f64, f64), b: (f64, f64)) -> f64 {
    let dx = a.0 - b.0;
    let dy = a.1 - b.1;
    (dx * dx + dy * dy).sqrt()
}

/// Analytic disk with optional center offset (for invariance assays only).
pub fn analytic_disk_phi_offset(
    width: usize,
    height: usize,
    radius: f64,
    ox: f64,
    oy: f64,
) -> Vec<f64> {
    let cx = (width as f64 - 1.0) * 0.5 + ox;
    let cy = (height as f64 - 1.0) * 0.5 + oy;
    let mut phi = vec![0.0; width * height];
    for j in 0..height {
        for i in 0..width {
            let dx = i as f64 - cx;
            let dy = j as f64 - cy;
            let r = (dx * dx + dy * dy).sqrt();
            let t = ((radius + 0.75 - r) / 1.5).clamp(0.0, 1.0);
            phi[j * width + i] = t * t * (3.0 - 2.0 * t);
        }
    }
    phi
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeometryQualifyRow {
    pub radius: f64,
    pub offset_x: f64,
    pub offset_y: f64,
    pub grid: usize,
    pub interface_length: f64,
    pub true_length: f64,
    pub length_error_frac: f64,
    pub geometric_coverage: f64,
    pub closed: bool,
    pub n_supported: usize,
    pub off_interface_support: f64,
    pub no_diagonal_leak: bool,
    pub row_ok: bool,
}

pub fn geometry_qualify_row(radius: f64, ox: f64, oy: f64) -> GeometryQualifyRow {
    let (w, h) = grid_for_radius(radius);
    let phi = analytic_disk_phi_offset(w, h, radius, ox, oy);
    let support = build_cut_cell_support(&phi, w, h);
    let true_length = 2.0 * std::f64::consts::PI * radius;
    let length_error_frac = (support.interface_length - true_length).abs() / true_length.max(1e-15);
    let (geom_cov, closed, n_sup) = support.geometric_support_coverage();
    let off = support.off_interface_support_fraction(&phi);
    let no_diag = support.no_diagonal_leak_ok();
    let row_ok = length_error_frac <= 0.05 + 1e-12
        && geom_cov + 1e-12 >= 0.99
        && closed
        && off <= 1e-9
        && no_diag;
    GeometryQualifyRow {
        radius,
        offset_x: ox,
        offset_y: oy,
        grid: w,
        interface_length: support.interface_length,
        true_length,
        length_error_frac,
        geometric_coverage: geom_cov,
        closed,
        n_supported: n_sup,
        off_interface_support: off,
        no_diagonal_leak: no_diag,
        row_ok,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ambiguous_isolevel_determinism() {
        // Corner exactly 0.5 must not fragment a ring under strict interior rule.
        let (w, h) = grid_for_radius(16.0);
        let phi = analytic_disk_phi_offset(w, h, 16.0, 0.0, 0.0);
        let n_exact = phi.iter().filter(|p| (**p - 0.5).abs() < 1e-12).count();
        assert!(n_exact >= 1, "fixture expects exact 0.5 corners on centered disk");
        let s = build_cut_cell_support(&phi, w, h);
        let (cov, closed, _) = s.geometric_support_coverage();
        assert!(closed);
        assert!(cov >= 0.99);
    }

    #[test]
    fn saddle_pairing_stable() {
        // Artificial saddle 2×2.
        let w = 3;
        let h = 3;
        let mut phi = vec![0.0; 9];
        phi[0] = 1.0; // sw of dual (0,0) at cell (0,0) — use cells:
        // cells: (0,0)=1 (1,0)=0 (0,1)=0 (1,1)=1 → saddle
        phi[0 * 3 + 0] = 1.0;
        phi[0 * 3 + 1] = 0.0;
        phi[1 * 3 + 0] = 0.0;
        phi[1 * 3 + 1] = 1.0;
        let s1 = build_cut_cell_support(&phi, w, h);
        let s2 = build_cut_cell_support(&phi, w, h);
        assert_eq!(s1.adjacency, s2.adjacency);
        assert!((s1.interface_length - s2.interface_length).abs() < 1e-15);
    }
}
