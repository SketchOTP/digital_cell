//! Motif detection and catalyst–template complex formation.
//!
//! Motifs: HHB (harvesting), BBH (building). A physical monomer participates in
//! at most one bound complex at a time. Efficiencies apply locally via complex
//! allocation — never as an organism genome parameter.

use crate::material_mesh::{MaterialMesh, MonomerKind};
use crate::mesh_reactions::ReactionParams;
use crate::template_polymer::{template_schema_load_ok, TemplateLedger, TemplateParams};
use serde::{Deserialize, Serialize};

const EPS: f64 = 1e-15;

pub const MOTIF_HARVEST: [MonomerKind; 3] = [MonomerKind::H, MonomerKind::H, MonomerKind::B];
pub const MOTIF_BUILD: [MonomerKind; 3] = [MonomerKind::B, MonomerKind::B, MonomerKind::H];

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct MotifOccupancy {
    /// Available unbound HHB motif instances (count).
    pub m_h_free: f64,
    /// Available unbound BBH motif instances (count).
    pub m_b_free: f64,
    /// Bound complexes (catalyst mass units = concentration·area).
    pub k_h: f64,
    pub k_b: f64,
}

/// Count motif starts on complete templates; overlapping detection allowed,
/// but each monomer index can be claimed by at most one binding.
pub fn count_motifs(mesh: &MaterialMesh) -> (usize, usize, Vec<(usize, usize, bool)>) {
    // returns (n_hhb, n_bbh, list of (chain_idx, start, is_harvest))
    let mut sites = Vec::new();
    let mut n_h = 0usize;
    let mut n_b = 0usize;
    for (ci, chain) in mesh.templates.iter().enumerate() {
        if !chain.is_complete_template() {
            continue;
        }
        let m = &chain.monomers;
        if m.len() < 3 {
            continue;
        }
        for i in 0..m.len() - 2 {
            if m[i] == MOTIF_HARVEST[0] && m[i + 1] == MOTIF_HARVEST[1] && m[i + 2] == MOTIF_HARVEST[2]
            {
                sites.push((ci, i, true));
                n_h += 1;
            }
            if m[i] == MOTIF_BUILD[0] && m[i + 1] == MOTIF_BUILD[1] && m[i + 2] == MOTIF_BUILD[2] {
                sites.push((ci, i, false));
                n_b += 1;
            }
        }
    }
    (n_h, n_b, sites)
}

/// Count non-overlapping motifs with greedy left-to-right claim (monomer exclusivity).
pub fn count_available_motifs(mesh: &MaterialMesh) -> (f64, f64) {
    let mut m_h = 0.0;
    let mut m_b = 0.0;
    for chain in &mesh.templates {
        if !chain.is_complete_template() {
            continue;
        }
        let m = &chain.monomers;
        let mut claimed = vec![false; m.len()];
        // Prefer alternating claim order by scanning once for HHB then BBH on unclaimed.
        for i in 0..m.len().saturating_sub(2) {
            if claimed[i] || claimed[i + 1] || claimed[i + 2] {
                continue;
            }
            if m[i] == MonomerKind::H && m[i + 1] == MonomerKind::H && m[i + 2] == MonomerKind::B {
                m_h += 1.0;
                claimed[i] = true;
                claimed[i + 1] = true;
                claimed[i + 2] = true;
            }
        }
        for i in 0..m.len().saturating_sub(2) {
            if claimed[i] || claimed[i + 1] || claimed[i + 2] {
                continue;
            }
            if m[i] == MonomerKind::B && m[i + 1] == MonomerKind::B && m[i + 2] == MonomerKind::H {
                m_b += 1.0;
                claimed[i] = true;
                claimed[i + 1] = true;
                claimed[i + 2] = true;
            }
        }
    }
    (m_h, m_b)
}

/// Competitive binding: C_free + M ⇌ K. Conserves C_total = C_free + K_H + K_B.
pub fn catalyst_binding_step(
    mesh: &mut MaterialMesh,
    react: &ReactionParams,
    dt: f64,
) -> TemplateLedger {
    let mut led = TemplateLedger::default();
    let p = &react.template;
    if !p.enable || !template_schema_load_ok(mesh, p) {
        if p.enable {
            led.rejected_steps += 1;
        }
        return led;
    }
    let area = mesh.area().max(EPS);
    let c_total = mesh.interior.c.max(0.0);
    let mut k_h = mesh.interior.k_h.max(0.0);
    let mut k_b = mesh.interior.k_b.max(0.0);
    // Clamp complexes into total catalyst.
    if k_h + k_b > c_total {
        let s = c_total / (k_h + k_b).max(EPS);
        k_h *= s;
        k_b *= s;
    }

    if !p.enable_binding {
        // Release all complexes to free catalyst.
        mesh.interior.k_h = 0.0;
        mesh.interior.k_b = 0.0;
        led.k_h_bound = 0.0;
        led.k_b_bound = 0.0;
        return led;
    }

    let (m_h, m_b) = count_available_motifs(mesh);
    // Motifs already occupied scale with current complexes (1 catalyst per motif).
    let occupied_h = (k_h * area).min(m_h);
    let occupied_b = (k_b * area).min(m_b);
    let m_h_free = (m_h - occupied_h).max(0.0);
    let m_b_free = (m_b - occupied_b).max(0.0);
    let c_free = (c_total - k_h - k_b).max(0.0);

    // Mass-action: bind free C to free motifs; unbind complexes.
    let bind_h = p.k_bind_motif * c_free * (m_h_free / area.max(EPS)) * dt;
    let bind_b = p.k_bind_motif * c_free * (m_b_free / area.max(EPS)) * dt;
    let unbind_h = p.k_unbind_motif * k_h * dt;
    let unbind_b = p.k_unbind_motif * k_b * dt;

    let mut dk_h = bind_h - unbind_h;
    let mut dk_b = bind_b - unbind_b;
    // Limit by free catalyst and free motifs.
    let max_bind = c_free;
    if dk_h.max(0.0) + dk_b.max(0.0) > max_bind {
        let s = max_bind / (dk_h.max(0.0) + dk_b.max(0.0)).max(EPS);
        if dk_h > 0.0 {
            dk_h *= s;
        }
        if dk_b > 0.0 {
            dk_b *= s;
        }
    }
    k_h = (k_h + dk_h).clamp(0.0, c_total);
    k_b = (k_b + dk_b).clamp(0.0, c_total);
    if k_h + k_b > c_total {
        let s = c_total / (k_h + k_b);
        k_h *= s;
        k_b *= s;
    }
    mesh.interior.k_h = k_h;
    mesh.interior.k_b = k_b;
    led.k_h_bound = k_h * area;
    led.k_b_bound = k_b * area;
    led
}

/// Harvest/build activity multipliers from complex allocation.
/// Free catalyst retains baseline 1.0; complexes use fixed efficiencies.
pub fn template_activity_gains(mesh: &MaterialMesh, p: &TemplateParams) -> (f64, f64) {
    if !p.enable || !p.enable_binding {
        return (1.0, 1.0);
    }
    let (m_h, m_b) = count_available_motifs(mesh);
    if m_h + m_b <= 0.0 {
        // No motifs ⇒ no lawful complexes; orphan K cannot express phenotype.
        return (1.0, 1.0);
    }
    let c = mesh.interior.c.max(0.0);
    if c <= EPS {
        return (1.0, 1.0);
    }
    let k_h = mesh.interior.k_h.clamp(0.0, c).min(m_h / mesh.area().max(EPS));
    let k_b = mesh.interior.k_b.clamp(0.0, c - k_h).min(m_b / mesh.area().max(EPS));
    let c_free = (c - k_h - k_b).max(0.0);
    let g_h = (c_free * 1.0 + k_h * p.eff_kh_harvest + k_b * p.eff_kb_harvest) / c;
    let g_b = (c_free * 1.0 + k_h * p.eff_kh_build + k_b * p.eff_kb_build) / c;
    (g_h.max(0.0), g_b.max(0.0))
}

pub fn motif_frequency_in_population<'a, I>(sequences: I) -> (f64, f64)
where
    I: IntoIterator<Item = &'a str>,
{
    let mut n_h = 0.0;
    let mut n_b = 0.0;
    let mut n = 0.0;
    for seq in sequences {
        n += 1.0;
        let chars: Vec<char> = seq.chars().collect();
        for i in 0..chars.len().saturating_sub(2) {
            if chars[i] == 'H' && chars[i + 1] == 'H' && chars[i + 2] == 'B' {
                n_h += 1.0;
            }
            if chars[i] == 'B' && chars[i + 1] == 'B' && chars[i + 2] == 'H' {
                n_b += 1.0;
            }
        }
    }
    if n <= 0.0 {
        return (0.0, 0.0);
    }
    (n_h / n, n_b / n)
}
