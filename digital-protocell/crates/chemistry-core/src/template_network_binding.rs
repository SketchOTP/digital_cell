//! Local overlapping pair-site catalyst binding (D-093).
//!
//! Circular sites on complete templates. No global matching algorithm.
//! Adjacent sites compete via shared-monomer free capacity.

use crate::material_mesh::{MaterialMesh, MonomerKind};
use crate::mesh_reactions::ReactionParams;
use crate::template_network::{
    c_free, ensure_all_site_k, network_schema_load_ok, NetworkLedger, PairChannel,
};

const EPS: f64 = 1e-15;

/// Local damage demand from nearby edge strain and membrane occupancy only.
pub fn local_damage_demand(mesh: &MaterialMesh, pos: [f64; 2], w_s: f64, w_m: f64) -> f64 {
    let mut best = 0.0_f64;
    let n = mesh.n();
    // Radius-scale neighborhood so templates near the centroid still sense boundary damage.
    let reach = (mesh.perimeter() / (2.0 * std::f64::consts::PI)).max(4.0) * 1.5;
    for i in 0..n {
        if mesh.edges[i].ruptured {
            continue;
        }
        let a = mesh.vertices[i];
        let b = mesh.vertices[(i + 1) % n];
        let mx = 0.5 * (a[0] + b[0]);
        let my = 0.5 * (a[1] + b[1]);
        let d = (pos[0] - mx).hypot(pos[1] - my);
        if d > reach {
            continue;
        }
        let eps = mesh.strain(i).max(0.0);
        let theta = mesh.occupancy(i).clamp(0.0, 1.0);
        let d_local = w_s * eps + w_m * (1.0 - theta);
        let w = 1.0 / (1.0 + d / reach.max(1e-9));
        best = best.max(d_local * w);
    }
    best
}

/// State gate at a pair site using chemistry at the physical template position.
pub fn site_gate(
    mesh: &MaterialMesh,
    channel: PairChannel,
    pos: [f64; 2],
    react: &ReactionParams,
) -> f64 {
    let p = &react.network;
    let a = mesh.interior.a.max(0.0);
    let r = mesh.interior.r.max(0.0);
    let n = mesh.interior.n.max(0.0);
    let f = mesh.interior.f.max(0.0);
    match channel {
        PairChannel::Hh => {
            let qn = n / (0.15 + n);
            let qf = f / (0.15 + f);
            qn * qf * (p.k_low / (p.k_low + a + EPS))
        }
        PairChannel::Hb => {
            let sat = (a * a) / (p.k_store * p.k_store + a * a + EPS);
            let room = (1.0 - r / p.r_max.max(EPS)).max(0.0);
            sat * room
        }
        PairChannel::Bh => {
            let rr = r / (p.k_r + r + EPS);
            let low = p.k_low / (p.k_low + a + EPS);
            rr * low
        }
        PairChannel::Bb => {
            let rg = r / (p.k_growth + r + EPS);
            let d_local = local_damage_demand(mesh, pos, p.w_s, p.w_m);
            let dd = d_local / (p.k_d + d_local + EPS);
            rg * dd
        }
    }
}

fn channel_of(monomers: &[MonomerKind], i: usize, circular: bool) -> Option<PairChannel> {
    let n = monomers.len();
    if n < 2 {
        return None;
    }
    let j = if circular {
        (i + 1) % n
    } else if i + 1 < n {
        i + 1
    } else {
        return None;
    };
    Some(PairChannel::from_monomers(monomers[i], monomers[j]))
}

/// Free capacity at monomer j: f_j = 1 − o_{j−1} − o_j (circular).
fn free_capacity(site_k: &[f64], j: usize, k_site: f64) -> f64 {
    let n = site_k.len();
    if n == 0 {
        return 1.0;
    }
    let ks = k_site.max(EPS);
    let o_prev = site_k[(j + n - 1) % n].max(0.0) / ks;
    let o_here = site_k[j % n].max(0.0) / ks;
    (1.0 - o_prev - o_here).clamp(0.0, 1.0)
}

fn availability(site_k: &[f64], i: usize, k_site: f64) -> f64 {
    let n = site_k.len();
    if n == 0 {
        return 0.0;
    }
    let f_i = free_capacity(site_k, i, k_site);
    let f_ip1 = free_capacity(site_k, (i + 1) % n, k_site);
    f_i * f_ip1
}

/// Overlapping occupancy invariant on circular sites.
pub fn occupancy_invariant_ok(mesh: &MaterialMesh, k_site: f64) -> bool {
    let ks = k_site.max(EPS);
    for t in &mesh.templates {
        let n = t.site_k.len();
        if n == 0 {
            continue;
        }
        for j in 0..n {
            let o_prev = t.site_k[(j + n - 1) % n].max(0.0) / ks;
            let o_here = t.site_k[j].max(0.0) / ks;
            if o_prev + o_here > 1.0 + 1e-9 {
                return false;
            }
        }
    }
    true
}

/// Competitive local binding/unbinding. Conserves C_total = C_free + Σ K_i / area.
pub fn network_binding_step(
    mesh: &mut MaterialMesh,
    react: &ReactionParams,
    dt: f64,
) -> NetworkLedger {
    let mut led = NetworkLedger::default();
    let p = &react.network;
    if !p.enable || !network_schema_load_ok(mesh, p) {
        if p.enable {
            led.rejected_steps += 1;
        }
        return led;
    }
    ensure_all_site_k(mesh);
    if p.k_on <= 0.0 {
        for t in &mut mesh.templates {
            for k in &mut t.site_k {
                led.unbind_mass += *k;
                *k = 0.0;
            }
        }
        return led;
    }

    let area = mesh.area().max(EPS);
    let k_site = p.k_site.max(EPS);
    let n_chains = mesh.templates.len();

    for ci in 0..n_chains {
        let circular = mesh.templates[ci].is_complete_template();
        if !circular {
            for k in &mut mesh.templates[ci].site_k {
                led.unbind_mass += *k;
                *k = 0.0;
            }
            continue;
        }
        let n_sites = mesh.templates[ci].site_k.len();
        let pos = mesh.templates[ci].pos;
        for i in 0..n_sites {
            let Some(ch) = channel_of(&mesh.templates[ci].monomers, i, true) else {
                continue;
            };
            if !p.channel_enabled(ch) {
                let k = mesh.templates[ci].site_k[i];
                if k > 0.0 {
                    led.unbind_mass += k;
                    mesh.templates[ci].site_k[i] = 0.0;
                }
                continue;
            }
            let g = site_gate(mesh, ch, pos, react);
            let c_f = c_free(mesh);
            let v = availability(&mesh.templates[ci].site_k, i, k_site);
            let k_i = mesh.templates[ci].site_k[i].max(0.0);
            let j_bind = p.k_on * g * c_f * v * dt * area;
            let j_unbind = p.k_off * k_i * dt;
            let mut dk = j_bind - j_unbind;
            if dk > 0.0 {
                let free_mass = c_f * area;
                dk = dk.min(free_mass);
                let room = (k_site - k_i).max(0.0);
                dk = dk.min(room);
                let f_i = free_capacity(&mesh.templates[ci].site_k, i, k_site);
                let f_ip1 = free_capacity(&mesh.templates[ci].site_k, (i + 1) % n_sites, k_site);
                let o_i = k_i / k_site;
                let max_o = (f_i.min(f_ip1) + o_i).min(1.0);
                let max_k = max_o * k_site;
                dk = dk.min((max_k - k_i).max(0.0));
            } else {
                dk = dk.max(-k_i);
            }
            mesh.templates[ci].site_k[i] = (k_i + dk).max(0.0);
            if dk > 0.0 {
                led.bind_mass += dk;
            } else {
                led.unbind_mass += -dk;
            }
        }
    }

    if !occupancy_invariant_ok(mesh, k_site) {
        led.occupancy_violations += 1;
        repair_occupancy(mesh, k_site);
    }

    let (hh, hb, bh, bb) = sum_channel_masses(mesh);
    led.k_hh = hh;
    led.k_hb = hb;
    led.k_bh = bh;
    led.k_bb = bb;
    led
}

fn repair_occupancy(mesh: &mut MaterialMesh, k_site: f64) {
    let ks = k_site.max(EPS);
    for t in &mut mesh.templates {
        let n = t.site_k.len();
        if n == 0 {
            continue;
        }
        for j in 0..n {
            let o_prev = t.site_k[(j + n - 1) % n].max(0.0) / ks;
            let o_here = t.site_k[j].max(0.0) / ks;
            let s = o_prev + o_here;
            if s > 1.0 + 1e-12 {
                let scale = 1.0 / s;
                t.site_k[(j + n - 1) % n] *= scale;
                t.site_k[j] *= scale;
            }
        }
    }
}

pub fn sum_channel_masses(mesh: &MaterialMesh) -> (f64, f64, f64, f64) {
    let mut hh = 0.0;
    let mut hb = 0.0;
    let mut bh = 0.0;
    let mut bb = 0.0;
    for t in &mesh.templates {
        if !t.is_complete_template() {
            continue;
        }
        for (i, &k) in t.site_k.iter().enumerate() {
            let Some(ch) = channel_of(&t.monomers, i, true) else {
                continue;
            };
            match ch {
                PairChannel::Hh => hh += k.max(0.0),
                PairChannel::Hb => hb += k.max(0.0),
                PairChannel::Bh => bh += k.max(0.0),
                PairChannel::Bb => bb += k.max(0.0),
            }
        }
    }
    (hh, hb, bh, bb)
}

/// Response vector V_T = (K_HH, K_HB, K_BH, K_BB) as concentrations (mass/area).
pub fn response_vector(mesh: &MaterialMesh) -> [f64; 4] {
    let area = mesh.area().max(EPS);
    let (hh, hb, bh, bb) = sum_channel_masses(mesh);
    [hh / area, hb / area, bh / area, bb / area]
}

/// Cosine similarity of two response vectors.
pub fn response_similarity(a: &[f64; 4], b: &[f64; 4]) -> f64 {
    let mut dot = 0.0;
    let mut na = 0.0;
    let mut nb = 0.0;
    for i in 0..4 {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    if na <= EPS || nb <= EPS {
        return 0.0;
    }
    (dot / (na.sqrt() * nb.sqrt())).clamp(-1.0, 1.0)
}
