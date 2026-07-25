//! Spatial partition of template chains at physical fission.
//!
//! Each complete chain remains at its position and enters the daughter whose
//! closed mesh contains it. No sequence or count is copied.

use crate::material_mesh::{MaterialMesh, MonomerKind, TemplateChain};
use crate::template_polymer::MONOMER_MASS;

/// Assign each parent template to d1 or d2 by point-in-polygon; conservative
/// geometric assignment if on the cleavage region (centroid distance).
pub fn partition_templates(
    parent: &MaterialMesh,
    d1: &mut MaterialMesh,
    d2: &mut MaterialMesh,
) -> (usize, usize, f64) {
    d1.templates.clear();
    d2.templates.clear();
    let mut n1 = 0usize;
    let mut n2 = 0usize;
    let mut residual = 0.0;
    let c1 = d1.centroid();
    let c2 = d2.centroid();
    for chain in &parent.templates {
        let p = chain.pos;
        let in1 = d1.point_inside(p[0], p[1]);
        let in2 = d2.point_inside(p[0], p[1]);
        let go_d1 = match (in1, in2) {
            (true, false) => true,
            (false, true) => false,
            (true, true) | (false, false) => {
                // Conservative geometric assignment by nearer daughter centroid.
                let d_a = (p[0] - c1[0]).hypot(p[1] - c1[1]);
                let d_b = (p[0] - c2[0]).hypot(p[1] - c2[1]);
                d_a <= d_b
            }
        };
        let mut child = chain.clone();
        // Clear nascent pairing state across fission (physical separation of complexes).
        // Bound monomers return to free pool of the receiving daughter via accounting below.
        let mut released_h = 0.0;
        let mut released_b = 0.0;
        for slot in child.paired.iter_mut() {
            if let Some(m) = slot.take() {
                    match m {
                        MonomerKind::H => released_h += 1.0,
                        MonomerKind::B => released_b += 1.0,
                    }
            }
        }
        for b in child.nascent_backbone.iter_mut() {
            *b = false;
        }
        if go_d1 {
            let a = d1.area().max(1e-9);
            d1.interior.u_h += released_h / a;
            d1.interior.u_b += released_b / a;
            d1.templates.push(child);
            n1 += 1;
        } else {
            let a = d2.area().max(1e-9);
            d2.interior.u_h += released_h / a;
            d2.interior.u_b += released_b / a;
            d2.templates.push(child);
            n2 += 1;
        }
    }
    // Free monomers already partitioned by area fraction in set_conc; residual checks
    // chain count conservation.
    residual = ((n1 + n2) as f64 - parent.templates.len() as f64).abs();
    // Propagate next_template_id.
    d1.next_template_id = parent.next_template_id;
    d2.next_template_id = parent.next_template_id;
    (n1, n2, residual)
}

pub fn complete_sequences(mesh: &MaterialMesh) -> Vec<String> {
    mesh.templates
        .iter()
        .filter(|t| t.is_complete_template())
        .map(TemplateChain::sequence_string)
        .collect()
}

/// Bounded internal diffusion of chain positions (observer-scale jitter).
pub fn diffuse_templates(mesh: &mut MaterialMesh, dt: f64, d_coeff: f64) {
    if d_coeff <= 0.0 || !mesh.alive {
        return;
    }
    let c = mesh.centroid();
    let scale = (2.0 * d_coeff * dt).sqrt();
    let n = mesh.templates.len();
    for i in 0..n {
        let id = mesh.templates[i].id;
        let pos = mesh.templates[i].pos;
        let h = (id.wrapping_mul(6364136223846793005).wrapping_add(i as u64)) as f64;
        let ang = (h % 1000.0) / 1000.0 * std::f64::consts::TAU;
        let step = scale * 0.05;
        let nx = pos[0] + step * ang.cos();
        let ny = pos[1] + step * ang.sin();
        if mesh.point_inside(nx, ny) {
            mesh.templates[i].pos = [nx, ny];
        } else {
            mesh.templates[i].pos = [
                0.9 * pos[0] + 0.1 * c[0],
                0.9 * pos[1] + 0.1 * c[1],
            ];
        }
    }
}
