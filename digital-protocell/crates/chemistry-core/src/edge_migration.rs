//! D-083 conservative local cut-cell edge-membrane migration.
//!
//! When the support graph changes with φ, bound material must follow through
//! local geometric continuity only — no global remapping, analytic circle, or
//! target-ring reconstruction. Kinetics (bind/unbind/lateral) are unchanged.

use crate::edge_membrane::{EdgeMembraneParams, EdgeMembraneState, FaceKind};
use crate::edge_support::{build_cut_cell_support, CutCellSupport, MEASURE_EPS};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct MigrationLedger {
    pub retained_on_overlap: f64,
    pub transferred_local: f64,
    pub returned_to_l: f64,
    pub capacity_excess_to_l: f64,
    pub orphan_cleared_to_l: f64,
    pub m_before: f64,
    pub m_after: f64,
    pub conservation_ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportTransitionAudit {
    pub n_old: usize,
    pub n_new: usize,
    pub n_overlap: usize,
    pub n_disappear: usize,
    pub n_appear: usize,
    pub b_on_disappear: f64,
    pub b_on_overlap: f64,
    pub classification_note: String,
}

fn face_key(kind: FaceKind, idx: usize) -> (u8, usize) {
    (
        match kind {
            FaceKind::Horizontal => 0,
            FaceKind::Vertical => 1,
        },
        idx,
    )
}

fn cells_of(state: &EdgeMembraneState, kind: FaceKind, idx: usize) -> [(usize, usize); 2] {
    let (i0, j0, i1, j1) = state.face_cells(kind, idx);
    [(i0, j0), (i1, j1)]
}

fn halo_cells(
    state: &EdgeMembraneState,
    kind: FaceKind,
    idx: usize,
) -> HashSet<(usize, usize)> {
    let mut cells = HashSet::new();
    let mut frontier: Vec<(usize, usize)> = cells_of(state, kind, idx).to_vec();
    for c in &frontier {
        cells.insert(*c);
    }
    // Two-cell geometric halo: enough for subcell–few-cell interface sweeps.
    for _ in 0..2 {
        let mut next = Vec::new();
        for &(i, j) in &frontier {
            let neigh = [
                (i.wrapping_sub(1), j),
                (i + 1, j),
                (i, j.wrapping_sub(1)),
                (i, j + 1),
            ];
            for (ni, nj) in neigh {
                if ni < state.width && nj < state.height && cells.insert((ni, nj)) {
                    next.push((ni, nj));
                }
            }
        }
        frontier = next;
    }
    cells
}

fn touches_halo(
    state: &EdgeMembraneState,
    kind: FaceKind,
    idx: usize,
    halo: &HashSet<(usize, usize)>,
) -> bool {
    cells_of(state, kind, idx)
        .iter()
        .any(|c| halo.contains(c))
}

/// New-supported faces within a bounded local hop neighborhood of a disappearing face.
/// Hops traverse shared-cell / support-adjacency links on the old∪new face set only.
pub fn local_new_targets(
    state: &EdgeMembraneState,
    kind: FaceKind,
    idx: usize,
    old_support: &CutCellSupport,
    new_support: &CutCellSupport,
) -> Vec<(FaceKind, usize)> {
    const MAX_HOPS: usize = 4;
    let mut universe: HashSet<(u8, usize)> = HashSet::new();
    for &(k, i) in &old_support.supported_faces() {
        universe.insert(face_key(k, i));
    }
    for &(k, i) in &new_support.supported_faces() {
        universe.insert(face_key(k, i));
    }

    let decode = |kb: u8, i: usize| -> (FaceKind, usize) {
        (
            if kb == 0 {
                FaceKind::Horizontal
            } else {
                FaceKind::Vertical
            },
            i,
        )
    };

    // Build local adjacency: support neighbors ∪ shared-cell neighbors within universe.
    let mut adj: HashMap<(u8, usize), Vec<(u8, usize)>> = HashMap::new();
    let mut faces: Vec<(u8, usize)> = universe.iter().copied().collect();
    faces.sort_unstable();
    for &(ka, ia) in &faces {
        let (kinda, _) = decode(ka, ia);
        let mut nbrs: HashSet<(u8, usize)> = HashSet::new();
        for (nk, ni) in old_support.neighbors(kinda, ia) {
            let key = face_key(nk, ni);
            if universe.contains(&key) {
                nbrs.insert(key);
            }
        }
        for (nk, ni) in new_support.neighbors(kinda, ia) {
            let key = face_key(nk, ni);
            if universe.contains(&key) {
                nbrs.insert(key);
            }
        }
        let halo = halo_cells(state, kinda, ia);
        for &(kb, ib) in &faces {
            if (kb, ib) == (ka, ia) {
                continue;
            }
            let (kindb, _) = decode(kb, ib);
            if touches_halo(state, kindb, ib, &halo) {
                nbrs.insert((kb, ib));
            }
        }
        let mut nbr_vec: Vec<_> = nbrs.into_iter().collect();
        nbr_vec.sort_unstable();
        adj.insert((ka, ia), nbr_vec);
    }

    let start = face_key(kind, idx);
    let mut seen: HashSet<(u8, usize)> = HashSet::new();
    let mut queue: VecDeque<((u8, usize), usize)> = VecDeque::new();
    queue.push_back((start, 0));
    seen.insert(start);
    let mut keys: HashSet<(u8, usize)> = HashSet::new();
    while let Some((node, dist)) = queue.pop_front() {
        let (nk, ni) = decode(node.0, node.1);
        if new_support.is_supported(nk, ni) && node != start {
            keys.insert(node);
        }
        if dist >= MAX_HOPS {
            continue;
        }
        if let Some(nbrs) = adj.get(&node) {
            for &n in nbrs {
                if seen.insert(n) {
                    queue.push_back((n, dist + 1));
                }
            }
        }
    }

    let mut out: Vec<(FaceKind, usize)> = keys
        .into_iter()
        .map(|(kb, i)| decode(kb, i))
        .collect();
    out.sort_by_key(|&(k, i)| (matches!(k, FaceKind::Vertical), i));
    out
}

pub fn audit_support_transition(
    state: &EdgeMembraneState,
    old: &CutCellSupport,
    new: &CutCellSupport,
) -> SupportTransitionAudit {
    let old_set: HashSet<_> = old
        .supported_faces()
        .into_iter()
        .map(|(k, i)| face_key(k, i))
        .collect();
    let new_set: HashSet<_> = new
        .supported_faces()
        .into_iter()
        .map(|(k, i)| face_key(k, i))
        .collect();
    let mut n_overlap = 0;
    let mut n_disappear = 0;
    let mut b_on_disappear = 0.0;
    let mut b_on_overlap = 0.0;
    for &(k, i) in &old.supported_faces() {
        let key = face_key(k, i);
        let b = state.bound_ref(k)[i];
        if new_set.contains(&key) {
            n_overlap += 1;
            b_on_overlap += b;
        } else {
            n_disappear += 1;
            b_on_disappear += b;
        }
    }
    let n_appear = new_set.difference(&old_set).count();
    SupportTransitionAudit {
        n_old: old_set.len(),
        n_new: new_set.len(),
        n_overlap,
        n_disappear,
        n_appear,
        b_on_disappear,
        b_on_overlap,
        classification_note: if b_on_disappear > 1e-9 {
            "B on disappearing support will strand without migration".into()
        } else {
            "no B on disappearing fragments".into()
        },
    }
}

fn give_to_cells(state: &mut EdgeMembraneState, kind: FaceKind, idx: usize, amount: f64) {
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

/// Atomic conservative support-transition operator.
///
/// 1. Retain B on overlapping old/new faces (cap excess → L).
/// 2. Transfer B from disappearing faces to locally adjacent new faces.
/// 3. Return unmatched remainder to nearby L.
/// 4. Clear residual unsupported B → L.
///
/// Does not use analytic circles, global coverage, or nonlocal nearest-edge projection.
pub fn migrate_bound_across_support(
    state: &mut EdgeMembraneState,
    old: &CutCellSupport,
    new: &CutCellSupport,
    params: &EdgeMembraneParams,
) -> MigrationLedger {
    let m_before = state.total_membrane();
    let mut led = MigrationLedger {
        m_before,
        ..Default::default()
    };

    let new_set: HashSet<_> = new
        .supported_faces()
        .into_iter()
        .map(|(k, i)| face_key(k, i))
        .collect();

    // Pass 1: overlapping faces — retain up to new capacity; local excess → appear neighbors, else L.
    let old_set: HashSet<_> = old
        .supported_faces()
        .into_iter()
        .map(|(k, i)| face_key(k, i))
        .collect();

    for &(k, i) in &old.supported_faces() {
        if !new_set.contains(&face_key(k, i)) {
            continue;
        }
        let b = state.bound_ref(k)[i];
        if b <= 0.0 {
            continue;
        }
        let cap = new.face_capacity(k, i, params.b_max);
        if b > cap + MEASURE_EPS {
            let mut excess = b - cap;
            state.bound_mut(k)[i] = cap;
            led.retained_on_overlap += cap;
            // Prefer spilling excess onto locally appearing neighbors (same continuity rule).
            let spill_targets: Vec<_> = local_new_targets(state, k, i, old, new)
                .into_iter()
                .filter(|&(tk, ti)| !old_set.contains(&face_key(tk, ti)))
                .collect();
            if !spill_targets.is_empty() {
                let rooms: Vec<f64> = spill_targets
                    .iter()
                    .map(|&(tk, ti)| {
                        (new.face_capacity(tk, ti, params.b_max) - state.bound_ref(tk)[ti])
                            .max(0.0)
                    })
                    .collect();
                let room_sum: f64 = rooms.iter().sum();
                if room_sum > MEASURE_EPS {
                    let take = excess.min(room_sum);
                    for (j, &(tk, ti)) in spill_targets.iter().enumerate() {
                        if rooms[j] <= 0.0 {
                            continue;
                        }
                        let share = take * (rooms[j] / room_sum);
                        state.bound_mut(tk)[ti] += share;
                        excess -= share;
                        led.transferred_local += share;
                    }
                }
            }
            if excess > MEASURE_EPS {
                give_to_cells(state, k, i, excess);
                led.capacity_excess_to_l += excess;
            }
        } else {
            led.retained_on_overlap += b;
        }
    }

    // Pass 2: disappearing faces — local transfer then return remainder to L.
    for &(k, i) in &old.supported_faces() {
        if new_set.contains(&face_key(k, i)) {
            continue;
        }
        let mut rem = state.bound_ref(k)[i];
        if rem <= MEASURE_EPS {
            state.bound_mut(k)[i] = 0.0;
            continue;
        }
        state.bound_mut(k)[i] = 0.0;

        let targets = local_new_targets(state, k, i, old, new);
        // Prefer newly appearing faces so material follows the swept interface.
        let (appear, retain): (Vec<_>, Vec<_>) = targets
            .into_iter()
            .partition(|&(tk, ti)| !old_set.contains(&face_key(tk, ti)));
        for group in [appear, retain] {
            if rem <= MEASURE_EPS || group.is_empty() {
                continue;
            }
            let rooms: Vec<f64> = group
                .iter()
                .map(|&(tk, ti)| {
                    (new.face_capacity(tk, ti, params.b_max) - state.bound_ref(tk)[ti]).max(0.0)
                })
                .collect();
            let room_sum: f64 = rooms.iter().sum();
            if room_sum <= MEASURE_EPS {
                continue;
            }
            let take = rem.min(room_sum);
            for (j, &(tk, ti)) in group.iter().enumerate() {
                if rooms[j] <= 0.0 {
                    continue;
                }
                let share = take * (rooms[j] / room_sum);
                state.bound_mut(tk)[ti] += share;
                rem -= share;
                led.transferred_local += share;
            }
        }
        if rem > MEASURE_EPS {
            // Deposit unmatched L near a local new target when possible (not stranded interior).
            if let Some(&(tk, ti)) = local_new_targets(state, k, i, old, new).first() {
                give_to_cells(state, tk, ti, rem);
            } else {
                give_to_cells(state, k, i, rem);
            }
            led.returned_to_l += rem;
        }
    }

    // Pass 2.5: local continuity fill — appear faces pull surplus B from nearby retained faces.
    let thr = params.occupied_theta * params.b_max;
    let appear_faces: Vec<(FaceKind, usize)> = new
        .supported_faces()
        .into_iter()
        .filter(|&(k, i)| !old_set.contains(&face_key(k, i)))
        .collect();
    for &(ak, ai) in &appear_faces {
        let mut room = (new.face_capacity(ak, ai, params.b_max) - state.bound_ref(ak)[ai]).max(0.0);
        if room <= MEASURE_EPS {
            continue;
        }
        // Sources: local retained faces with surplus above occupancy threshold.
        let mut sources = local_new_targets(state, ak, ai, old, new)
            .into_iter()
            .filter(|&(sk, si)| {
                old_set.contains(&face_key(sk, si)) && new_set.contains(&face_key(sk, si))
            })
            .collect::<Vec<_>>();
        sources.sort_by_key(|&(k, i)| (matches!(k, FaceKind::Vertical), i));
        let mut surplus: Vec<f64> = sources
            .iter()
            // Leave a thin occupied floor so the source stays in the connected component.
            .map(|&(sk, si)| (state.bound_ref(sk)[si] - 0.5 * thr).max(0.0))
            .collect();
        let surplus_sum: f64 = surplus.iter().sum();
        if surplus_sum <= MEASURE_EPS {
            continue;
        }
        let take = room.min(surplus_sum);
        for (j, &(sk, si)) in sources.iter().enumerate() {
            if surplus[j] <= 0.0 {
                continue;
            }
            let share = take * (surplus[j] / surplus_sum);
            state.bound_mut(sk)[si] -= share;
            state.bound_mut(ak)[ai] += share;
            surplus[j] -= share;
            room -= share;
            led.transferred_local += share;
        }
    }

    // Pass 3: any remaining unsupported B (should be ~0) → L.
    for i in 0..state.n_h() {
        if !new.is_supported(FaceKind::Horizontal, i) {
            let b = state.bound_h[i];
            if b > MEASURE_EPS {
                state.bound_h[i] = 0.0;
                give_to_cells(state, FaceKind::Horizontal, i, b);
                led.orphan_cleared_to_l += b;
            } else {
                state.bound_h[i] = 0.0;
            }
        }
    }
    for i in 0..state.n_v() {
        if !new.is_supported(FaceKind::Vertical, i) {
            let b = state.bound_v[i];
            if b > MEASURE_EPS {
                state.bound_v[i] = 0.0;
                give_to_cells(state, FaceKind::Vertical, i, b);
                led.orphan_cleared_to_l += b;
            } else {
                state.bound_v[i] = 0.0;
            }
        }
    }

    led.m_after = state.total_membrane();
    led.conservation_ok = (led.m_after - led.m_before).abs() < 1e-9 * (1.0 + led.m_before.abs());
    led
}

/// Rebuild support from new φ and migrate in one accepted transition.
pub fn apply_support_transition(
    state: &mut EdgeMembraneState,
    old_support: &CutCellSupport,
    new_phi: &[f64],
    params: &EdgeMembraneParams,
) -> (CutCellSupport, MigrationLedger) {
    let new_support = build_cut_cell_support(new_phi, state.width, state.height);
    let led = migrate_bound_across_support(state, old_support, &new_support, params);
    (new_support, led)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::d079_analysis::SEED_DENSITY;
    use crate::edge_membrane::{
        analytic_disk_phi, diagnostic_fill_support, grid_for_radius, off_support_bound_fraction,
        seed_free_near_support,
    };

    #[test]
    fn migration_conserves_on_radius_step() {
        let params = EdgeMembraneParams::default();
        let (w, h) = grid_for_radius(22.0);
        let phi0 = analytic_disk_phi(w, h, 20.0);
        let old = build_cut_cell_support(&phi0, w, h);
        let mut state = EdgeMembraneState::new(w, h);
        state.catalyst = 1.0;
        seed_free_near_support(&mut state, &old, SEED_DENSITY);
        diagnostic_fill_support(&mut state, &old, params.b_max);
        let m0 = state.total_membrane();
        let phi1 = analytic_disk_phi(w, h, 22.0);
        let new = build_cut_cell_support(&phi1, w, h);
        let led = migrate_bound_across_support(&mut state, &old, &new, &params);
        assert!(led.conservation_ok, "{led:?}");
        assert!((state.total_membrane() - m0).abs() < 1e-9 * (1.0 + m0));
        assert!(off_support_bound_fraction(&state, &new) < 1e-12);
    }

    #[test]
    fn no_duplication_mass_identity() {
        let params = EdgeMembraneParams::default();
        let (w, h) = grid_for_radius(18.0);
        let phi0 = analytic_disk_phi(w, h, 16.0);
        let old = build_cut_cell_support(&phi0, w, h);
        let mut state = EdgeMembraneState::new(w, h);
        diagnostic_fill_support(&mut state, &old, params.b_max);
        let m0 = state.total_membrane();
        let phi1 = analytic_disk_phi(w, h, 18.0);
        let new = build_cut_cell_support(&phi1, w, h);
        let _ = migrate_bound_across_support(&mut state, &old, &new, &params);
        assert!((state.total_membrane() - m0).abs() < 1e-9 * (1.0 + m0));
    }

    #[test]
    fn diagnose_single_radius_step_coverage() {
        use crate::d079_analysis::{ASSEMBLY_DT, SEED_DENSITY};
        use crate::d080_analysis::frozen_d079_params;
        use crate::edge_membrane::{
            accepted_step_supported, connected_closed_support_observer, off_support_bound_fraction,
            seed_free_near_support, support_coverage,
        };

        let params = frozen_d079_params();
        let (w, h) = grid_for_radius(22.0);
        let mut state = EdgeMembraneState::new(w, h);
        state.catalyst = 1.0;
        let phi0 = analytic_disk_phi(w, h, 18.0);
        let old = build_cut_cell_support(&phi0, w, h);
        seed_free_near_support(&mut state, &old, SEED_DENSITY);
        for _ in 0..800 {
            let _ = accepted_step_supported(
                &mut state, &phi0, &old, &params, ASSEMBLY_DT, false, 1.0,
            );
        }
        let (c0, _, _) = connected_closed_support_observer(&state, &old, &params);
        let phi1 = analytic_disk_phi(w, h, 20.0);
        let new = build_cut_cell_support(&phi1, w, h);
        let audit = audit_support_transition(&state, &old, &new);
        let led = migrate_bound_across_support(&mut state, &old, &new, &params);
        let (c1, cl1, _) = connected_closed_support_observer(&state, &new, &params);
        eprintln!(
            "diag c0={c0:.3} audit dis={} app={} b_dis={:.3} led xfer={:.3} toL={:.3} ret={:.3} imm_cov={c1:.3} closed={cl1} cov={} off={:.4} B={:.3} L={:.3}",
            audit.n_disappear,
            audit.n_appear,
            audit.b_on_disappear,
            led.transferred_local,
            led.returned_to_l,
            led.retained_on_overlap,
            support_coverage(&state, &new, &params),
            off_support_bound_fraction(&state, &new),
            state.total_b(),
            state.total_l(),
        );
        assert!(led.conservation_ok);
        // Immediate post-migration occupancy should retain a connected majority.
        assert!(
            c1 >= 0.50,
            "immediate coverage too low after migration: {c1}"
        );
    }
}
