//! D-079 conserved edge-network membrane substrate (experimental, schema-isolated).
//!
//! Membrane material lives as free units `L` at cell centers and bound units `B`
//! on horizontal/vertical grid faces. Ordinary bind/unbind/lateral transfer
//! conserve `L+B`. Damage transfers `B→W`. Production transfers `A→L`.
//!
//! Does **not** change production continuum defaults.

use crate::reactions::{catalyst_activation, interface_weight};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

pub const EQUATION_VERSION_EDGE_NETWORK: &str = "edge_network_membrane_v1";
pub const FIELD_SCHEMA_EDGE_NETWORK: &str = "edge_network_faces_v1";
pub const EDGE_NETWORK_SCHEMA_VERSION: u32 = 1;

pub const STAGE_A_C_PERM_MAX: f64 = 0.05;
pub const STAGE_A_A_PERM_MAX: f64 = 0.05;
pub const STAGE_A_NF_PERM_LO: f64 = 0.20;
pub const STAGE_A_NF_PERM_HI: f64 = 0.50;
pub const STAGE_A_W_PERM_MIN: f64 = 0.70;

/// Default Stage A β set (authoritative D-008 Stage A).
pub const BETA_C: f64 = 4.6;
pub const BETA_A: f64 = 4.6;
pub const BETA_N: f64 = 1.2;
pub const BETA_F: f64 = 1.2;
pub const BETA_W: f64 = 0.2;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EdgeMembraneParams {
    pub b_max: f64,
    pub k_bind: f64,
    pub k_unbind: f64,
    pub k_lateral: f64,
    pub k_produce: f64,
    pub yield_l_from_a: f64,
    pub k_c: f64,
    pub endpoint_unbind_boost: f64,
    pub occupied_theta: f64,
    pub beta_c: f64,
    pub beta_a: f64,
    pub beta_n: f64,
    pub beta_f: f64,
    pub beta_w: f64,
}

impl Default for EdgeMembraneParams {
    fn default() -> Self {
        Self {
            b_max: 1.0,
            k_bind: 8.0,
            k_unbind: 0.02,
            k_lateral: 1.5,
            k_produce: 0.0,
            yield_l_from_a: 1.0,
            k_c: 0.1,
            endpoint_unbind_boost: 4.0,
            occupied_theta: 0.35,
            beta_c: BETA_C,
            beta_a: BETA_A,
            beta_n: BETA_N,
            beta_f: BETA_F,
            beta_w: BETA_W,
        }
    }
}

impl EdgeMembraneParams {
    pub fn identity_parts(&self) -> Vec<(&str, f64)> {
        vec![
            ("b_max", self.b_max),
            ("k_bind", self.k_bind),
            ("k_unbind", self.k_unbind),
            ("k_lateral", self.k_lateral),
            ("k_produce", self.k_produce),
            ("yield_l_from_a", self.yield_l_from_a),
            ("k_c", self.k_c),
            ("endpoint_unbind_boost", self.endpoint_unbind_boost),
            ("occupied_theta", self.occupied_theta),
            ("beta_c", self.beta_c),
            ("beta_a", self.beta_a),
            ("beta_n", self.beta_n),
            ("beta_f", self.beta_f),
            ("beta_w", self.beta_w),
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FaceKind {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeMembraneState {
    pub width: usize,
    pub height: usize,
    pub free_l: Vec<f64>,
    pub bound_h: Vec<f64>,
    pub bound_v: Vec<f64>,
    pub waste: f64,
    pub activated: f64,
    pub catalyst: f64,
    pub accepted_steps: u64,
    pub rejected_steps: u64,
    pub equation_version: String,
    pub field_schema: String,
    pub schema_version: u32,
}

impl EdgeMembraneState {
    pub fn new(width: usize, height: usize) -> Self {
        assert!(width >= 2 && height >= 2);
        Self {
            width,
            height,
            free_l: vec![0.0; width * height],
            bound_h: vec![0.0; (width - 1) * height],
            bound_v: vec![0.0; width * (height - 1)],
            waste: 0.0,
            activated: 0.0,
            catalyst: 1.0,
            accepted_steps: 0,
            rejected_steps: 0,
            equation_version: EQUATION_VERSION_EDGE_NETWORK.into(),
            field_schema: FIELD_SCHEMA_EDGE_NETWORK.into(),
            schema_version: EDGE_NETWORK_SCHEMA_VERSION,
        }
    }

    pub fn n_h(&self) -> usize {
        (self.width - 1) * self.height
    }

    pub fn n_v(&self) -> usize {
        self.width * (self.height - 1)
    }

    pub fn cell_idx(&self, i: usize, j: usize) -> usize {
        j * self.width + i
    }

    pub fn h_idx(&self, i: usize, j: usize) -> usize {
        debug_assert!(i + 1 < self.width);
        j * (self.width - 1) + i
    }

    pub fn v_idx(&self, i: usize, j: usize) -> usize {
        debug_assert!(j + 1 < self.height);
        j * self.width + i
    }

    pub fn total_l(&self) -> f64 {
        self.free_l.iter().sum()
    }

    pub fn total_b(&self) -> f64 {
        self.bound_h.iter().sum::<f64>() + self.bound_v.iter().sum::<f64>()
    }

    pub fn total_membrane(&self) -> f64 {
        self.total_l() + self.total_b()
    }

    pub fn face_i_phi(&self, kind: FaceKind, idx: usize, phi: &[f64]) -> f64 {
        let (i0, j0, i1, j1) = self.face_cells(kind, idx);
        0.5 * (interface_weight(phi[self.cell_idx(i0, j0)])
            + interface_weight(phi[self.cell_idx(i1, j1)]))
    }

    pub fn face_cells(&self, kind: FaceKind, idx: usize) -> (usize, usize, usize, usize) {
        match kind {
            FaceKind::Horizontal => {
                let w1 = self.width - 1;
                let j = idx / w1;
                let i = idx % w1;
                (i, j, i + 1, j)
            }
            FaceKind::Vertical => {
                let j = idx / self.width;
                let i = idx % self.width;
                (i, j, i, j + 1)
            }
        }
    }

    pub fn face_free_l_mean(&self, kind: FaceKind, idx: usize) -> f64 {
        let (i0, j0, i1, j1) = self.face_cells(kind, idx);
        0.5 * (self.free_l[self.cell_idx(i0, j0)] + self.free_l[self.cell_idx(i1, j1)])
    }

    pub fn bound_ref(&self, kind: FaceKind) -> &[f64] {
        match kind {
            FaceKind::Horizontal => &self.bound_h,
            FaceKind::Vertical => &self.bound_v,
        }
    }

    pub fn bound_mut(&mut self, kind: FaceKind) -> &mut [f64] {
        match kind {
            FaceKind::Horizontal => &mut self.bound_h,
            FaceKind::Vertical => &mut self.bound_v,
        }
    }

    /// Local endpoint factor r_f from adjacent occupied faces (observer-local topology).
    pub fn endpoint_factor(&self, kind: FaceKind, idx: usize, params: &EdgeMembraneParams) -> f64 {
        let n = self.local_occupied_neighbors(kind, idx, params);
        // Endpoints (0–1 neighbors) unbind faster; interior chain (2+) slower.
        if n >= 2 {
            1.0
        } else {
            params.endpoint_unbind_boost
        }
    }

    fn local_occupied_neighbors(
        &self,
        kind: FaceKind,
        idx: usize,
        params: &EdgeMembraneParams,
    ) -> usize {
        let thr = params.occupied_theta * params.b_max;
        let (i0, j0, i1, j1) = self.face_cells(kind, idx);
        let mut n = 0;
        // Count other occupied faces incident to either endpoint vertex.
        for &(ii, jj) in &[(i0, j0), (i1, j1)] {
            // H faces touching cell (ii,jj)
            if ii > 0 {
                let h = self.h_idx(ii - 1, jj);
                if !(kind == FaceKind::Horizontal && h == idx) && self.bound_h[h] >= thr {
                    n += 1;
                }
            }
            if ii + 1 < self.width {
                let h = self.h_idx(ii, jj);
                if !(kind == FaceKind::Horizontal && h == idx) && self.bound_h[h] >= thr {
                    n += 1;
                }
            }
            // V faces
            if jj > 0 {
                let v = self.v_idx(ii, jj - 1);
                if !(kind == FaceKind::Vertical && v == idx) && self.bound_v[v] >= thr {
                    n += 1;
                }
            }
            if jj + 1 < self.height {
                let v = self.v_idx(ii, jj);
                if !(kind == FaceKind::Vertical && v == idx) && self.bound_v[v] >= thr {
                    n += 1;
                }
            }
        }
        n
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeSnapshot {
    pub equation_version: String,
    pub field_schema: String,
    pub schema_version: u32,
    pub width: usize,
    pub height: usize,
    pub free_l: Vec<f64>,
    pub bound_h: Vec<f64>,
    pub bound_v: Vec<f64>,
    pub waste: f64,
    pub activated: f64,
    pub catalyst: f64,
    pub params: EdgeMembraneParams,
}

impl EdgeSnapshot {
    pub fn from_state(state: &EdgeMembraneState, params: &EdgeMembraneParams) -> Self {
        Self {
            equation_version: state.equation_version.clone(),
            field_schema: state.field_schema.clone(),
            schema_version: state.schema_version,
            width: state.width,
            height: state.height,
            free_l: state.free_l.clone(),
            bound_h: state.bound_h.clone(),
            bound_v: state.bound_v.clone(),
            waste: state.waste,
            activated: state.activated,
            catalyst: state.catalyst,
            params: *params,
        }
    }

    pub fn can_resume_into(&self, target_schema: &str, target_eq: &str, target_ver: u32) -> bool {
        self.field_schema == target_schema
            && self.equation_version == target_eq
            && self.schema_version == target_ver
    }

    pub fn resume_into(&self, state: &mut EdgeMembraneState) -> Result<(), String> {
        if !self.can_resume_into(
            FIELD_SCHEMA_EDGE_NETWORK,
            EQUATION_VERSION_EDGE_NETWORK,
            EDGE_NETWORK_SCHEMA_VERSION,
        ) {
            return Err("legacy or mismatched snapshot cannot resume under edge-network schema".into());
        }
        if self.width != state.width || self.height != state.height {
            return Err("snapshot grid size mismatch".into());
        }
        state.free_l = self.free_l.clone();
        state.bound_h = self.bound_h.clone();
        state.bound_v = self.bound_v.clone();
        state.waste = self.waste;
        state.activated = self.activated;
        state.catalyst = self.catalyst;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct StepLedger {
    pub bind: f64,
    pub unbind: f64,
    pub lateral: f64,
    pub produce: f64,
    pub damage: f64,
}

/// Analytic disk φ for fixed-geometry assays (1 interior, 0 exterior).
pub fn analytic_disk_phi(width: usize, height: usize, radius: f64) -> Vec<f64> {
    let cx = (width as f64 - 1.0) * 0.5;
    let cy = (height as f64 - 1.0) * 0.5;
    let mut phi = vec![0.0; width * height];
    for j in 0..height {
        for i in 0..width {
            let dx = i as f64 - cx;
            let dy = j as f64 - cy;
            let r = (dx * dx + dy * dy).sqrt();
            // Smooth interface band ~1 cell.
            let t = ((radius + 0.75 - r) / 1.5).clamp(0.0, 1.0);
            phi[j * width + i] = t * t * (3.0 - 2.0 * t);
        }
    }
    phi
}

pub fn is_crossing_face(phi: &[f64], width: usize, kind: FaceKind, i0: usize, j0: usize, i1: usize, j1: usize) -> bool {
    let a = phi[j0 * width + i0] >= 0.5;
    let b = phi[j1 * width + i1] >= 0.5;
    a != b
}

pub fn crossing_face_indices(phi: &[f64], width: usize, height: usize) -> (Vec<usize>, Vec<usize>) {
    let mut hs = Vec::new();
    let mut vs = Vec::new();
    for j in 0..height {
        for i in 0..(width - 1) {
            if is_crossing_face(phi, width, FaceKind::Horizontal, i, j, i + 1, j) {
                hs.push(j * (width - 1) + i);
            }
        }
    }
    for j in 0..(height - 1) {
        for i in 0..width {
            if is_crossing_face(phi, width, FaceKind::Vertical, i, j, i, j + 1) {
                vs.push(j * width + i);
            }
        }
    }
    (hs, vs)
}

/// Seed free L near interface without completing a bound ring.
pub fn seed_free_near_interface(
    state: &mut EdgeMembraneState,
    phi: &[f64],
    density_per_crossing: f64,
) {
    let (hs, vs) = crossing_face_indices(phi, state.width, state.height);
    let n = (hs.len() + vs.len()).max(1) as f64;
    let total = density_per_crossing * n;
    let mut weights = vec![0.0; state.free_l.len()];
    let mut wsum = 0.0;
    for &idx in &hs {
        let (i0, j0, i1, j1) = state.face_cells(FaceKind::Horizontal, idx);
        for (i, j) in [(i0, j0), (i1, j1)] {
            let c = state.cell_idx(i, j);
            weights[c] += 1.0;
            wsum += 1.0;
        }
    }
    for &idx in &vs {
        let (i0, j0, i1, j1) = state.face_cells(FaceKind::Vertical, idx);
        for (i, j) in [(i0, j0), (i1, j1)] {
            let c = state.cell_idx(i, j);
            weights[c] += 1.0;
            wsum += 1.0;
        }
    }
    if wsum <= 0.0 {
        return;
    }
    for (c, w) in weights.iter().enumerate() {
        state.free_l[c] += total * (*w) / wsum;
    }
}

fn take_from_face_neighbors(state: &mut EdgeMembraneState, kind: FaceKind, idx: usize, amount: f64) {
    if amount <= 0.0 {
        return;
    }
    let (i0, j0, i1, j1) = state.face_cells(kind, idx);
    let c0 = state.cell_idx(i0, j0);
    let c1 = state.cell_idx(i1, j1);
    let a0 = state.free_l[c0].max(0.0);
    let a1 = state.free_l[c1].max(0.0);
    let s = a0 + a1;
    if s <= 1e-18 {
        return;
    }
    let t0 = amount * a0 / s;
    let t1 = amount - t0;
    state.free_l[c0] = (a0 - t0).max(0.0);
    state.free_l[c1] = (a1 - t1).max(0.0);
}

fn give_to_face_neighbors(state: &mut EdgeMembraneState, kind: FaceKind, idx: usize, amount: f64) {
    if amount <= 0.0 {
        return;
    }
    let (i0, j0, i1, j1) = state.face_cells(kind, idx);
    let c0 = state.cell_idx(i0, j0);
    let c1 = state.cell_idx(i1, j1);
    let half = 0.5 * amount;
    state.free_l[c0] += half;
    state.free_l[c1] += half;
}

/// One accepted edge-network kinetics step (bind/unbind/lateral/optional produce).
pub fn accepted_step(
    state: &mut EdgeMembraneState,
    phi: &[f64],
    params: &EdgeMembraneParams,
    dt: f64,
    allow_produce: bool,
) -> StepLedger {
    let mut ledger = StepLedger::default();
    // Snapshot for atomic accept — we apply in-place but clamp so invariants hold;
    // rejected path is separate (no mutation).
    let l_before = state.total_membrane();

    // Binding on faces with appreciable interface weight.
    for kind in [FaceKind::Horizontal, FaceKind::Vertical] {
        let n = match kind {
            FaceKind::Horizontal => state.n_h(),
            FaceKind::Vertical => state.n_v(),
        };
        for idx in 0..n {
            let i_phi = state.face_i_phi(kind, idx, phi);
            if i_phi < 1e-4 {
                continue;
            }
            let (i0, j0, i1, j1) = state.face_cells(kind, idx);
            let p0 = phi[state.cell_idx(i0, j0)];
            let p1 = phi[state.cell_idx(i1, j1)];
            // Local crossing support: faces straddling φ=0.5 bind preferentially.
            // This is local old-state geometry, not a prescribed ring.
            let cross = if (p0 - 0.5) * (p1 - 0.5) < 0.0 {
                1.0
            } else {
                0.0
            };
            if cross <= 0.0 {
                continue;
            }
            let l_face = state.face_free_l_mean(kind, idx);
            let b = state.bound_ref(kind)[idx];
            let room = (params.b_max - b).max(0.0);
            let q = catalyst_activation(state.catalyst, params.k_c);
            let j_bind =
                params.k_bind * q * i_phi * cross * l_face * (room / params.b_max.max(1e-15));
            let mut d = j_bind * dt;
            let avail = {
                let (i0, j0, i1, j1) = state.face_cells(kind, idx);
                state.free_l[state.cell_idx(i0, j0)].max(0.0)
                    + state.free_l[state.cell_idx(i1, j1)].max(0.0)
            };
            d = d.min(avail).min(room);
            if d > 0.0 {
                take_from_face_neighbors(state, kind, idx, d);
                state.bound_mut(kind)[idx] += d;
                ledger.bind += d;
            }
        }
    }

    // Unbinding only where mass exists.
    for kind in [FaceKind::Horizontal, FaceKind::Vertical] {
        let n = match kind {
            FaceKind::Horizontal => state.n_h(),
            FaceKind::Vertical => state.n_v(),
        };
        for idx in 0..n {
            let b = state.bound_ref(kind)[idx];
            if b < 1e-15 {
                continue;
            }
            let r = state.endpoint_factor(kind, idx, params);
            let mut d = params.k_unbind * b * r * dt;
            d = d.min(b);
            if d > 0.0 {
                state.bound_mut(kind)[idx] -= d;
                give_to_face_neighbors(state, kind, idx, d);
                ledger.unbind += d;
            }
        }
    }

    // Lateral transfer along shared vertices between occupied crossing faces (conserves B).
    lateral_transfer(state, params, dt, &mut ledger, phi);

    if allow_produce && params.k_produce > 0.0 && state.activated > 0.0 {
        let q = catalyst_activation(state.catalyst, params.k_c);
        let mut d_a = params.k_produce * q * state.activated * dt;
        d_a = d_a.min(state.activated);
        let d_l = d_a * params.yield_l_from_a;
        state.activated -= d_a;
        // Deposit produced L preferentially near interface cells.
        let mut wsum = 0.0;
        let mut w = vec![0.0; state.free_l.len()];
        for (c, p) in phi.iter().enumerate() {
            let iw = interface_weight(*p);
            w[c] = iw;
            wsum += iw;
        }
        if wsum > 0.0 {
            for c in 0..w.len() {
                state.free_l[c] += d_l * w[c] / wsum;
            }
        } else {
            let per = d_l / state.free_l.len() as f64;
            for v in &mut state.free_l {
                *v += per;
            }
        }
        ledger.produce += d_l;
    }

    // Enforce nonnegativity / capacity (no hidden renormalization of totals).
    for v in &mut state.free_l {
        *v = v.max(0.0);
    }
    for v in state.bound_h.iter_mut().chain(state.bound_v.iter_mut()) {
        *v = v.clamp(0.0, params.b_max);
    }
    state.activated = state.activated.max(0.0);
    state.catalyst = state.catalyst.max(0.0);

    let _ = l_before;
    state.accepted_steps += 1;
    ledger
}

/// Rejected step: increment reject counter only; no field mutation.
pub fn rejected_step(state: &mut EdgeMembraneState) {
    state.rejected_steps += 1;
}

fn lateral_transfer(
    state: &mut EdgeMembraneState,
    params: &EdgeMembraneParams,
    dt: f64,
    ledger: &mut StepLedger,
    phi: &[f64],
) {
    let thr = 1e-12;
    let (hs, vs) = crossing_face_indices(phi, state.width, state.height);
    let cross_h: std::collections::HashSet<usize> = hs.into_iter().collect();
    let cross_v: std::collections::HashSet<usize> = vs.into_iter().collect();
    let mut faces: Vec<(FaceKind, usize)> = Vec::new();
    for &i in &cross_h {
        if state.bound_h[i] > thr {
            faces.push((FaceKind::Horizontal, i));
        }
    }
    for &i in &cross_v {
        if state.bound_v[i] > thr {
            faces.push((FaceKind::Vertical, i));
        }
    }
    let snapshot_h = state.bound_h.clone();
    let snapshot_v = state.bound_v.clone();
    let get = |kind: FaceKind, idx: usize| match kind {
        FaceKind::Horizontal => snapshot_h[idx],
        FaceKind::Vertical => snapshot_v[idx],
    };
    for &(kind, idx) in &faces {
        let b = get(kind, idx);
        let neighbors = adjacent_faces(state, kind, idx);
        for (nk, ni) in neighbors {
            let neighbor_cross = match nk {
                FaceKind::Horizontal => cross_h.contains(&ni),
                FaceKind::Vertical => cross_v.contains(&ni),
            };
            if !neighbor_cross {
                continue;
            }
            let bn = get(nk, ni);
            let diff = b - bn;
            if diff <= 0.0 {
                continue;
            }
            let room = (params.b_max - bn).max(0.0);
            let mut flux = params.k_lateral * 0.5 * diff * dt;
            flux = flux.min(diff * 0.5).min(room).min(get(kind, idx));
            if flux > 0.0 {
                match kind {
                    FaceKind::Horizontal => state.bound_h[idx] -= flux,
                    FaceKind::Vertical => state.bound_v[idx] -= flux,
                }
                match nk {
                    FaceKind::Horizontal => state.bound_h[ni] += flux,
                    FaceKind::Vertical => state.bound_v[ni] += flux,
                }
                ledger.lateral += flux;
            }
        }
    }
}

fn adjacent_faces(state: &EdgeMembraneState, kind: FaceKind, idx: usize) -> Vec<(FaceKind, usize)> {
    let (i0, j0, i1, j1) = state.face_cells(kind, idx);
    let mut out = Vec::new();
    for &(ii, jj) in &[(i0, j0), (i1, j1)] {
        if ii > 0 {
            let h = state.h_idx(ii - 1, jj);
            if !(kind == FaceKind::Horizontal && h == idx) {
                out.push((FaceKind::Horizontal, h));
            }
        }
        if ii + 1 < state.width {
            let h = state.h_idx(ii, jj);
            if !(kind == FaceKind::Horizontal && h == idx) {
                out.push((FaceKind::Horizontal, h));
            }
        }
        if jj > 0 {
            let v = state.v_idx(ii, jj - 1);
            if !(kind == FaceKind::Vertical && v == idx) {
                out.push((FaceKind::Vertical, v));
            }
        }
        if jj + 1 < state.height {
            let v = state.v_idx(ii, jj);
            if !(kind == FaceKind::Vertical && v == idx) {
                out.push((FaceKind::Vertical, v));
            }
        }
    }
    out.sort_by_key(|x| (x.0 == FaceKind::Vertical, x.1));
    out.dedup();
    out
}

/// Declared damage: remove fraction of bound mass on selected crossing faces → W.
pub fn apply_damage(
    state: &mut EdgeMembraneState,
    phi: &[f64],
    fraction: f64,
    params: &EdgeMembraneParams,
) -> f64 {
    let (hs, vs) = crossing_face_indices(phi, state.width, state.height);
    let mut targets: Vec<(FaceKind, usize)> = hs
        .into_iter()
        .map(|i| (FaceKind::Horizontal, i))
        .chain(vs.into_iter().map(|i| (FaceKind::Vertical, i)))
        .collect();
    targets.retain(|(k, i)| state.bound_ref(*k)[*i] > params.occupied_theta * params.b_max * 0.1);
    if targets.is_empty() {
        return 0.0;
    }
    let n_damage = ((targets.len() as f64) * fraction).round().max(1.0) as usize;
    let n_damage = n_damage.min(targets.len());
    let mut removed = 0.0;
    for &(kind, idx) in targets.iter().take(n_damage) {
        let b = state.bound_ref(kind)[idx];
        state.bound_mut(kind)[idx] = 0.0;
        state.waste += b;
        removed += b;
    }
    removed
}

/// Permeability Π = exp(−β θ) with θ = B/B_max on a face.
pub fn face_permeability(theta: f64, beta: f64) -> f64 {
    (-beta * theta.clamp(0.0, 1.0)).exp()
}

pub fn species_beta(params: &EdgeMembraneParams, species: &str) -> f64 {
    match species {
        "C" => params.beta_c,
        "A" => params.beta_a,
        "N" => params.beta_n,
        "F" => params.beta_f,
        "W" => params.beta_w,
        _ => 0.0,
    }
}

/// Mean permeability on crossing faces for a species.
pub fn mean_crossing_permeability(
    state: &EdgeMembraneState,
    phi: &[f64],
    params: &EdgeMembraneParams,
    species: &str,
) -> f64 {
    let beta = species_beta(params, species);
    let (hs, vs) = crossing_face_indices(phi, state.width, state.height);
    let mut sum = 0.0;
    let mut n = 0.0;
    for idx in hs {
        let th = state.bound_h[idx] / params.b_max.max(1e-15);
        sum += face_permeability(th, beta);
        n += 1.0;
    }
    for idx in vs {
        let th = state.bound_v[idx] / params.b_max.max(1e-15);
        sum += face_permeability(th, beta);
        n += 1.0;
    }
    if n <= 0.0 {
        1.0
    } else {
        sum / n
    }
}

/// Occupied coverage of crossing faces (θ ≥ occupied_theta).
pub fn crossing_coverage(state: &EdgeMembraneState, phi: &[f64], params: &EdgeMembraneParams) -> f64 {
    let thr = params.occupied_theta * params.b_max;
    let (hs, vs) = crossing_face_indices(phi, state.width, state.height);
    let total = (hs.len() + vs.len()).max(1) as f64;
    let mut occ = 0.0;
    for idx in hs {
        if state.bound_h[idx] >= thr {
            occ += 1.0;
        }
    }
    for idx in vs {
        if state.bound_v[idx] >= thr {
            occ += 1.0;
        }
    }
    occ / total
}

/// Off-interface bound mass fraction.
pub fn off_interface_bound_fraction(
    state: &EdgeMembraneState,
    phi: &[f64],
) -> f64 {
    let (hs, vs) = crossing_face_indices(phi, state.width, state.height);
    let mut cross = std::collections::HashSet::new();
    for i in hs {
        cross.insert((FaceKind::Horizontal, i));
    }
    for i in vs {
        cross.insert((FaceKind::Vertical, i));
    }
    let mut on = 0.0;
    let mut off = 0.0;
    for i in 0..state.n_h() {
        let m = state.bound_h[i];
        if cross.contains(&(FaceKind::Horizontal, i)) {
            on += m;
        } else {
            off += m;
        }
    }
    for i in 0..state.n_v() {
        let m = state.bound_v[i];
        if cross.contains(&(FaceKind::Vertical, i)) {
            on += m;
        } else {
            off += m;
        }
    }
    let t = on + off;
    if t <= 1e-15 {
        0.0
    } else {
        off / t
    }
}

/// Observer-only: BFS on occupied crossing faces; return largest component size / crossing count
/// and whether a closed cycle exists in that component.
pub fn connected_closed_observer(
    state: &EdgeMembraneState,
    phi: &[f64],
    params: &EdgeMembraneParams,
) -> (f64, bool, usize) {
    let thr = params.occupied_theta * params.b_max;
    let (hs, vs) = crossing_face_indices(phi, state.width, state.height);
    let mut nodes: Vec<(FaceKind, usize)> = Vec::new();
    for &i in &hs {
        if state.bound_h[i] >= thr {
            nodes.push((FaceKind::Horizontal, i));
        }
    }
    for &i in &vs {
        if state.bound_v[i] >= thr {
            nodes.push((FaceKind::Vertical, i));
        }
    }
    let n_cross = hs.len() + vs.len();
    if nodes.is_empty() {
        return (0.0, false, n_cross);
    }
    // Build adjacency among occupied crossing faces.
    let mut adj: Vec<Vec<usize>> = vec![vec![]; nodes.len()];
    for a in 0..nodes.len() {
        let na = adjacent_faces(state, nodes[a].0, nodes[a].1);
        for b in (a + 1)..nodes.len() {
            if na.iter().any(|&(k, i)| k == nodes[b].0 && i == nodes[b].1) {
                adj[a].push(b);
                adj[b].push(a);
            }
        }
    }
    // Largest component.
    let mut seen = vec![false; nodes.len()];
    let mut best = 0usize;
    let mut best_nodes: Vec<usize> = Vec::new();
    for s in 0..nodes.len() {
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
    let coverage = best as f64 / n_cross.max(1) as f64;
    // Closed cycle: in largest component, ∃ node with DFS back-edge (undirected cycle).
    let closed = component_has_cycle(&adj, &best_nodes);
    (coverage, closed, n_cross)
}

fn component_has_cycle(adj: &[Vec<usize>], nodes: &[usize]) -> bool {
    if nodes.len() < 3 {
        return false;
    }
    let set: std::collections::HashSet<usize> = nodes.iter().copied().collect();
    let mut seen = std::collections::HashSet::new();
    fn dfs(
        u: usize,
        parent: Option<usize>,
        adj: &[Vec<usize>],
        set: &std::collections::HashSet<usize>,
        seen: &mut std::collections::HashSet<usize>,
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

/// Grid size helper for radius assays.
pub fn grid_for_radius(radius: f64) -> (usize, usize) {
    let s = ((radius * 2.0 + 8.0).ceil() as usize).max(24);
    // Prefer odd so center is a cell.
    let s = if s % 2 == 0 { s + 1 } else { s };
    (s, s)
}

// ─── D-080 cut-cell support-aware kinetics (legacy crossing API unchanged) ───

use crate::edge_support::CutCellSupport;

/// Seed free L near cut-cell supported faces (no completed bound ring).
pub fn seed_free_near_support(
    state: &mut EdgeMembraneState,
    support: &CutCellSupport,
    density_per_face: f64,
) {
    let faces = support.supported_faces();
    let n = faces.len().max(1) as f64;
    let total = density_per_face * n;
    let mut weights = vec![0.0; state.free_l.len()];
    let mut wsum = 0.0;
    for &(kind, idx) in &faces {
        let (i0, j0, i1, j1) = state.face_cells(kind, idx);
        for (i, j) in [(i0, j0), (i1, j1)] {
            let c = state.cell_idx(i, j);
            weights[c] += 1.0;
            wsum += 1.0;
        }
    }
    if wsum <= 0.0 {
        return;
    }
    for (c, w) in weights.iter().enumerate() {
        state.free_l[c] += total * (*w) / wsum;
    }
}

/// Accepted step using cut-cell support for bind eligibility, capacity, and lateral graph.
pub fn accepted_step_supported(
    state: &mut EdgeMembraneState,
    phi: &[f64],
    support: &CutCellSupport,
    params: &EdgeMembraneParams,
    dt: f64,
    allow_produce: bool,
    k_lateral_scale: f64,
) -> StepLedger {
    let mut ledger = StepLedger::default();
    let supported = support.supported_faces();

    for &(kind, idx) in &supported {
        let cap = support.face_capacity(kind, idx, params.b_max);
        let i_phi = state.face_i_phi(kind, idx, phi);
        if i_phi < 1e-4 {
            continue;
        }
        let l_face = state.face_free_l_mean(kind, idx);
        let b = state.bound_ref(kind)[idx];
        let room = (cap - b).max(0.0);
        let q = catalyst_activation(state.catalyst, params.k_c);
        let j_bind = params.k_bind * q * i_phi * l_face * (room / cap.max(1e-15));
        let mut d = j_bind * dt;
        let avail = {
            let (i0, j0, i1, j1) = state.face_cells(kind, idx);
            state.free_l[state.cell_idx(i0, j0)].max(0.0)
                + state.free_l[state.cell_idx(i1, j1)].max(0.0)
        };
        d = d.min(avail).min(room);
        if d > 0.0 {
            take_from_face_neighbors(state, kind, idx, d);
            state.bound_mut(kind)[idx] += d;
            ledger.bind += d;
        }
    }

    // Unbind wherever bound mass exists (including residual off-support).
    for kind in [FaceKind::Horizontal, FaceKind::Vertical] {
        let n = match kind {
            FaceKind::Horizontal => state.n_h(),
            FaceKind::Vertical => state.n_v(),
        };
        for idx in 0..n {
            let b = state.bound_ref(kind)[idx];
            if b < 1e-15 {
                continue;
            }
            let r = state.endpoint_factor(kind, idx, params);
            let mut d = params.k_unbind * b * r * dt;
            d = d.min(b);
            if d > 0.0 {
                state.bound_mut(kind)[idx] -= d;
                give_to_face_neighbors(state, kind, idx, d);
                ledger.unbind += d;
            }
        }
    }

    lateral_transfer_supported(state, support, params, dt, k_lateral_scale, &mut ledger);

    if allow_produce && params.k_produce > 0.0 && state.activated > 0.0 {
        let q = catalyst_activation(state.catalyst, params.k_c);
        let mut d_a = params.k_produce * q * state.activated * dt;
        d_a = d_a.min(state.activated);
        let d_l = d_a * params.yield_l_from_a;
        state.activated -= d_a;
        let mut wsum = 0.0;
        let mut w = vec![0.0; state.free_l.len()];
        for (c, p) in phi.iter().enumerate() {
            let iw = interface_weight(*p);
            w[c] = iw;
            wsum += iw;
        }
        if wsum > 0.0 {
            for c in 0..w.len() {
                state.free_l[c] += d_l * w[c] / wsum;
            }
        } else {
            let per = d_l / state.free_l.len() as f64;
            for v in &mut state.free_l {
                *v += per;
            }
        }
        ledger.produce += d_l;
    }

    for v in &mut state.free_l {
        *v = v.max(0.0);
    }
    for &(kind, idx) in &supported {
        let cap = support.face_capacity(kind, idx, params.b_max);
        let b = state.bound_ref(kind)[idx];
        if b > cap {
            let excess = b - cap;
            state.bound_mut(kind)[idx] = cap;
            give_to_face_neighbors(state, kind, idx, excess);
        } else if b < 0.0 {
            state.bound_mut(kind)[idx] = 0.0;
        }
    }
    // Off-support residual: return any mass to free L (should stay ~0).
    for kind in [FaceKind::Horizontal, FaceKind::Vertical] {
        let n = match kind {
            FaceKind::Horizontal => state.n_h(),
            FaceKind::Vertical => state.n_v(),
        };
        for idx in 0..n {
            if support.is_supported(kind, idx) {
                continue;
            }
            let b = state.bound_ref(kind)[idx];
            if b > 0.0 {
                state.bound_mut(kind)[idx] = 0.0;
                give_to_face_neighbors(state, kind, idx, b);
            }
        }
    }
    state.activated = state.activated.max(0.0);
    state.catalyst = state.catalyst.max(0.0);
    state.accepted_steps += 1;
    ledger
}

fn lateral_transfer_supported(
    state: &mut EdgeMembraneState,
    support: &CutCellSupport,
    params: &EdgeMembraneParams,
    dt: f64,
    k_lateral_scale: f64,
    ledger: &mut StepLedger,
) {
    let kind_ord = |k: FaceKind| match k {
        FaceKind::Horizontal => 0u8,
        FaceKind::Vertical => 1u8,
    };
    let thr = 1e-12;
    let faces = support.supported_faces();
    let k_lat = params.k_lateral * k_lateral_scale;
    let mean_m = support.mean_positive_measure().max(1e-15);
    // Pairwise transfers with live capacity checks (conserves B exactly).
    let mut pairs: Vec<((FaceKind, usize), (FaceKind, usize))> = Vec::new();
    for &(kind, idx) in &faces {
        for (nk, ni) in support.neighbors(kind, idx) {
            if !support.is_supported(nk, ni) {
                continue;
            }
            // Order pairs to avoid double-counting undirected edges.
            let a = (kind_ord(kind), idx);
            let b = (kind_ord(nk), ni);
            if a < b {
                pairs.push(((kind, idx), (nk, ni)));
            }
        }
    }
    for &((k0, i0), (k1, i1)) in &pairs {
        let b0 = state.bound_ref(k0)[i0];
        let b1 = state.bound_ref(k1)[i1];
        let diff = b0 - b1;
        if diff.abs() <= thr {
            continue;
        }
        let (src_k, src_i, dst_k, dst_i, dpos) = if diff > 0.0 {
            (k0, i0, k1, i1, diff)
        } else {
            (k1, i1, k0, i0, -diff)
        };
        let cap_dst = support.face_capacity(dst_k, dst_i, params.b_max);
        let room = (cap_dst - state.bound_ref(dst_k)[dst_i]).max(0.0);
        let m_scale =
            0.5 * (support.measure(src_k, src_i) + support.measure(dst_k, dst_i)) / mean_m;
        let mut flux = k_lat * m_scale * 0.5 * dpos * dt;
        flux = flux
            .min(dpos * 0.5)
            .min(room)
            .min(state.bound_ref(src_k)[src_i]);
        if flux > 0.0 {
            state.bound_mut(src_k)[src_i] -= flux;
            state.bound_mut(dst_k)[dst_i] += flux;
            ledger.lateral += flux;
        }
    }
}

pub fn support_coverage(
    state: &EdgeMembraneState,
    support: &CutCellSupport,
    params: &EdgeMembraneParams,
) -> f64 {
    let thr = params.occupied_theta * params.b_max;
    let faces = support.supported_faces();
    let total = faces.len().max(1) as f64;
    let mut occ = 0.0;
    for &(kind, idx) in &faces {
        if state.bound_ref(kind)[idx] >= thr {
            occ += 1.0;
        }
    }
    occ / total
}

pub fn off_support_bound_fraction(state: &EdgeMembraneState, support: &CutCellSupport) -> f64 {
    let mut on = 0.0;
    let mut off = 0.0;
    for i in 0..state.n_h() {
        let m = state.bound_h[i];
        if support.is_supported(FaceKind::Horizontal, i) {
            on += m;
        } else {
            off += m;
        }
    }
    for i in 0..state.n_v() {
        let m = state.bound_v[i];
        if support.is_supported(FaceKind::Vertical, i) {
            on += m;
        } else {
            off += m;
        }
    }
    let t = on + off;
    if t <= 1e-15 {
        0.0
    } else {
        off / t
    }
}

pub fn connected_closed_support_observer(
    state: &EdgeMembraneState,
    support: &CutCellSupport,
    params: &EdgeMembraneParams,
) -> (f64, bool, usize) {
    let thr = params.occupied_theta * params.b_max;
    let supported = support.supported_faces();
    let n_sup = supported.len();
    let mut nodes: Vec<(FaceKind, usize)> = Vec::new();
    for &(kind, idx) in &supported {
        if state.bound_ref(kind)[idx] >= thr {
            nodes.push((kind, idx));
        }
    }
    if nodes.is_empty() {
        return (0.0, false, n_sup);
    }
    let idx_map: std::collections::HashMap<(FaceKind, usize), usize> =
        nodes.iter().copied().enumerate().map(|(i, n)| (n, i)).collect();
    let mut adj: Vec<Vec<usize>> = vec![vec![]; nodes.len()];
    for a in 0..nodes.len() {
        for (nk, ni) in support.neighbors(nodes[a].0, nodes[a].1) {
            if let Some(&b) = idx_map.get(&(nk, ni)) {
                if b > a {
                    adj[a].push(b);
                    adj[b].push(a);
                }
            }
        }
    }
    let mut seen = vec![false; nodes.len()];
    let mut best = 0usize;
    let mut best_nodes = Vec::new();
    for s in 0..nodes.len() {
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
    let coverage = best as f64 / n_sup.max(1) as f64;
    let closed = component_has_cycle(&adj, &best_nodes);
    (coverage, closed, n_sup)
}

pub fn mean_support_permeability(
    state: &EdgeMembraneState,
    support: &CutCellSupport,
    params: &EdgeMembraneParams,
    species: &str,
) -> f64 {
    let beta = species_beta(params, species);
    let faces = support.supported_faces();
    let mut sum = 0.0;
    let mut n = 0.0;
    for &(kind, idx) in &faces {
        let cap = support.face_capacity(kind, idx, params.b_max).max(1e-15);
        let th = (state.bound_ref(kind)[idx] / cap).clamp(0.0, 1.0);
        sum += face_permeability(th, beta);
        n += 1.0;
    }
    if n <= 0.0 {
        1.0
    } else {
        sum / n
    }
}

pub fn apply_damage_supported(
    state: &mut EdgeMembraneState,
    support: &CutCellSupport,
    fraction: f64,
    params: &EdgeMembraneParams,
) -> f64 {
    let mut targets: Vec<(FaceKind, usize)> = support
        .supported_faces()
        .into_iter()
        .filter(|(k, i)| state.bound_ref(*k)[*i] > params.occupied_theta * params.b_max * 0.1)
        .collect();
    if targets.is_empty() {
        return 0.0;
    }
    let n_damage = ((targets.len() as f64) * fraction).round().max(1.0) as usize;
    let n_damage = n_damage.min(targets.len());
    let mut removed = 0.0;
    for &(kind, idx) in targets.iter().take(n_damage) {
        let b = state.bound_ref(kind)[idx];
        state.bound_mut(kind)[idx] = 0.0;
        state.waste += b;
        removed += b;
    }
    removed
}

/// Fill all supported faces to capacity (diagnostic geometry fill; not chemistry).
pub fn diagnostic_fill_support(
    state: &mut EdgeMembraneState,
    support: &CutCellSupport,
    b_max: f64,
) {
    for (kind, idx) in support.supported_faces() {
        let cap = support.face_capacity(kind, idx, b_max);
        state.bound_mut(kind)[idx] = cap;
    }
}
