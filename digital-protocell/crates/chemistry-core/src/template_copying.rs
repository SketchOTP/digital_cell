//! Local template-directed copying: sitewise association + backbone ligation.
//!
//! No `copy_template()` whole-chain clone. Daughter sequences come from bound monomers.

use crate::mesh_reactions::{q_catalyst, ReactionParams};
use crate::material_mesh::{MaterialMesh, MonomerKind, TemplateChain};
use crate::template_polymer::{
    template_schema_load_ok, RngLike, TemplateLedger, FOUNDER_LEN, MONOMER_MASS,
};

const EPS: f64 = 1e-15;

fn is_match(site: MonomerKind, mono: MonomerKind) -> bool {
    site == mono
}

/// Sitewise association / dissociation and adjacent ligation.
pub fn copying_step(
    mesh: &mut MaterialMesh,
    react: &ReactionParams,
    dt: f64,
    rng: &mut impl RngLike,
    next_id: &mut u64,
) -> TemplateLedger {
    let mut led = TemplateLedger::default();
    let p = &react.template;
    if !p.enable || !p.enable_copying || !template_schema_load_ok(mesh, p) {
        if p.enable && !template_schema_load_ok(mesh, p) {
            led.rejected_steps += 1;
        }
        return led;
    }
    let area = mesh.area().max(EPS);
    let qc = q_catalyst(mesh.interior.c, react.q_c);

    // --- Association / dissociation on complete templates only ---
    let n_chains = mesh.templates.len();
    for ci in 0..n_chains {
        if !mesh.templates[ci].is_complete_template() {
            continue;
        }
        let len = mesh.templates[ci].monomers.len();
        for si in 0..len {
            let site = mesh.templates[ci].monomers[si];
            if mesh.templates[ci].paired[si].is_some() {
                // Dissociation
                let p_off = 1.0 - (-p.k_off * dt).exp();
                if rng.unit() < p_off {
                    if let Some(m) = mesh.templates[ci].paired[si].take() {
                        match m {
                            MonomerKind::H => mesh.interior.u_h += MONOMER_MASS / area,
                            MonomerKind::B => mesh.interior.u_b += MONOMER_MASS / area,
                        }
                        // Clear adjacent nascent bonds
                        if si > 0 {
                            mesh.templates[ci].nascent_backbone[si - 1] = false;
                        }
                        if si < len - 1 {
                            mesh.templates[ci].nascent_backbone[si] = false;
                        }
                        led.dissociations += 1;
                    }
                }
                continue;
            }
            // Association attempts from free pool (stochastic mass-action proxy).
            let uh = mesh.interior.u_h.max(0.0);
            let ub = mesh.interior.u_b.max(0.0);
            let k_mis = p.k_on_match / p.match_mismatch_ratio.max(1.0);
            let (k_h, k_b) = match site {
                MonomerKind::H => (p.k_on_match, if p.allow_mismatch { k_mis } else { 0.0 }),
                MonomerKind::B => (if p.allow_mismatch { k_mis } else { 0.0 }, p.k_on_match),
            };
            let rate_h = k_h * uh;
            let rate_b = k_b * ub;
            let rate = rate_h + rate_b;
            if rate <= 0.0 {
                continue;
            }
            let p_bind = 1.0 - (-rate * dt).exp();
            if rng.unit() >= p_bind {
                continue;
            }
            let pick_h = rate_h / rate.max(EPS);
            let mono = if rng.unit() < pick_h {
                MonomerKind::H
            } else {
                MonomerKind::B
            };
            let conc = match mono {
                MonomerKind::H => mesh.interior.u_h,
                MonomerKind::B => mesh.interior.u_b,
            };
            if conc * area < MONOMER_MASS - 1e-12 {
                led.rejected_steps += 1;
                continue;
            }
            match mono {
                MonomerKind::H => mesh.interior.u_h = (mesh.interior.u_h - MONOMER_MASS / area).max(0.0),
                MonomerKind::B => mesh.interior.u_b = (mesh.interior.u_b - MONOMER_MASS / area).max(0.0),
            }
            mesh.templates[ci].paired[si] = Some(mono);
            if is_match(site, mono) {
                led.match_binds += 1;
            } else {
                led.mismatch_binds += 1;
            }
        }
    }

    // --- Ligation of adjacent paired monomers (requires A + catalyst) ---
    if qc > 0.0 && p.k_ligate > 0.0 {
        for ci in 0..mesh.templates.len() {
            if !mesh.templates[ci].is_complete_template() {
                continue;
            }
            let len = mesh.templates[ci].monomers.len();
            if len < 2 {
                continue;
            }
            for bi in 0..len - 1 {
                if mesh.templates[ci].nascent_backbone[bi] {
                    continue;
                }
                if mesh.templates[ci].paired[bi].is_none()
                    || mesh.templates[ci].paired[bi + 1].is_none()
                {
                    continue;
                }
                let p_lig = 1.0 - (-p.k_ligate * qc * dt).exp();
                if rng.unit() >= p_lig {
                    continue;
                }
                let a_have = mesh.interior.a.max(0.0) * area;
                if a_have < MONOMER_MASS - 1e-12 {
                    led.rejected_steps += 1;
                    continue;
                }
                mesh.interior.a = (mesh.interior.a - MONOMER_MASS / area).max(0.0);
                mesh.interior.w += MONOMER_MASS / area;
                mesh.templates[ci].nascent_backbone[bi] = true;
                led.ligations += 1;
                led.a_consumed_ligation += MONOMER_MASS;
                led.w_produced += MONOMER_MASS;
            }
        }
    }

    // --- Release complete daughter chains when all sites paired and ligated ---
    let mut new_chains: Vec<TemplateChain> = Vec::new();
    for ci in 0..mesh.templates.len() {
        if !mesh.templates[ci].is_complete_template() {
            continue;
        }
        let len = mesh.templates[ci].monomers.len();
        if len != FOUNDER_LEN {
            continue;
        }
        if mesh.templates[ci].paired.iter().any(|p| p.is_none()) {
            continue;
        }
        if mesh.templates[ci].nascent_backbone.iter().any(|b| !*b) {
            continue;
        }
        // Assemble daughter from actual bound monomers.
        let monomers: Vec<MonomerKind> = mesh.templates[ci]
            .paired
            .iter()
            .map(|p| p.unwrap())
            .collect();
        let parent_id = mesh.templates[ci].id;
        let pos = mesh.templates[ci].pos;
        let orient = mesh.templates[ci].orientation + 0.15;
        // Clear pairing on parent (daughter separates via pairing dissociation).
        for slot in mesh.templates[ci].paired.iter_mut() {
            *slot = None;
        }
        for b in mesh.templates[ci].nascent_backbone.iter_mut() {
            *b = false;
        }
        let mut daughter = TemplateChain {
            id: *next_id,
            parent_id: Some(parent_id),
            pos: [pos[0] + 0.05 * orient.cos(), pos[1] + 0.05 * orient.sin()],
            orientation: orient,
            monomers,
            backbone: vec![true; FOUNDER_LEN - 1],
            paired: vec![None; FOUNDER_LEN],
            nascent_backbone: vec![false; FOUNDER_LEN - 1],
            complete: true,
        };
        daughter.refresh_complete();
        *next_id += 1;
        new_chains.push(daughter);
        led.complete_copies += 1;
    }
    mesh.templates.extend(new_chains);
    led
}
