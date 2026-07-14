//! Immutable candidate identity and canonical hashing (D-004).

use crate::config::{
    EquationVersion, SimParams, DISH_RADIUS, DX, GRID_HEIGHT, GRID_WIDTH, RESERVOIR_WIDTH,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InitialStateClass {
    Fresh,
    AgedD002,
    CalibrationEndpoint,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CandidateMatchClass {
    MatchFinalCalibratedCandidate,
    MatchAnalyticalInitialEstimate,
    MatchIntermediateIteration,
    MatchUnknownConfiguration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridConfiguration {
    pub width: usize,
    pub height: usize,
    pub dx: f64,
    pub dish_radius: f64,
    pub reservoir_width: f64,
}

impl Default for GridConfiguration {
    fn default() -> Self {
        Self {
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
            dx: DX,
            dish_radius: DISH_RADIUS,
            reservoir_width: RESERVOIR_WIDTH,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitialConditionConfiguration {
    pub seed_r0: f64,
    pub seed_interface_width: f64,
    pub seed_catalyst_scale: f64,
    pub noise_amplitude: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateIdentity {
    pub candidate_id: String,
    pub equation_version: EquationVersion,
    pub k_phi: f64,
    pub k_structure: f64,
    pub k_rep: f64,
    pub params: SimParams,
    pub grid: GridConfiguration,
    pub initial_condition: InitialConditionConfiguration,
    pub random_seed: u64,
    pub source_snapshot_id: Option<String>,
    pub source_snapshot_hash: Option<String>,
    pub configuration_hash: String,
    pub candidate_hash: String,
    pub source_commit: String,
    pub calibration_branch: Option<String>,
    pub calibration_iteration: Option<u32>,
    pub selection_reason: String,
}

pub fn canonical_params_bytes(params: &SimParams) -> Vec<u8> {
    // ponytail: fixed field order for stable hashing; surface fields appended only when active
    // so historical crowding candidate hashes remain unchanged.
    let mut s = format!(
        "a={};kappa={};mobility_m={};d_c_inside={};d_c_outside={};d_n={};d_f={};d_w={};\
k_rep={};k_structure={};k_structure_decay={};k_catalyst_decay_inside={};k_catalyst_decay_outside={};\
k_waste_decay={};c_max={};n_reservoir={};f_reservoir={};w_reservoir={};reservoir_rate={};\
alpha_n_rep={};alpha_n_structure={};alpha_f_rep={};alpha_f_structure={};alpha_w_rep={};alpha_w_structure={};\
structure_extinction_threshold={};catalyst_extinction_threshold={};extinction_hold_time={};minimum_viable_duration={};\
seed_r0={};seed_interface_width={};seed_catalyst_scale={};noise_amplitude={};random_seed={};\
reactions_enabled={};phase_separation_enabled={};diffusion_enabled={};k_phi={};use_legacy_structure_kinetics={}",
        params.a,
        params.kappa,
        params.mobility_m,
        params.d_c_inside,
        params.d_c_outside,
        params.d_n,
        params.d_f,
        params.d_w,
        params.k_rep,
        params.k_structure,
        params.k_structure_decay,
        params.k_catalyst_decay_inside,
        params.k_catalyst_decay_outside,
        params.k_waste_decay,
        params.c_max,
        params.n_reservoir,
        params.f_reservoir,
        params.w_reservoir,
        params.reservoir_rate,
        params.alpha_n_rep,
        params.alpha_n_structure,
        params.alpha_f_rep,
        params.alpha_f_structure,
        params.alpha_w_rep,
        params.alpha_w_structure,
        params.structure_extinction_threshold,
        params.catalyst_extinction_threshold,
        params.extinction_hold_time,
        params.minimum_viable_duration,
        params.seed_r0,
        params.seed_interface_width,
        params.seed_catalyst_scale,
        params.noise_amplitude,
        params.random_seed,
        params.reactions_enabled,
        params.phase_separation_enabled,
        params.diffusion_enabled,
        params.k_phi,
        params.use_legacy_structure_kinetics,
    );
    if params.equation_version != EquationVersion::D003CrowdingV1
        || params.k_structure_interface != 0.0
        || (params.k_c_structure - 0.10).abs() > 1e-15
    {
        s.push_str(&format!(
            ";equation_version={};k_structure_interface={};k_c_structure={}",
            params.equation_version, params.k_structure_interface, params.k_c_structure
        ));
    }
    match params.equation_version {
        EquationVersion::MembraneMetabolismV1 => {
            s.push_str(&format!(
                ";d_a={};beta_c={};beta_a={};beta_n={};beta_f={};beta_w={};\
field_schema_version=seven_field_v1;snapshot_schema_version=2",
                params.d_a,
                params.beta_c,
                params.beta_a,
                params.beta_n,
                params.beta_f,
                params.beta_w
            ));
        }
        EquationVersion::D001BulkV1
        | EquationVersion::D003CrowdingV1
        | EquationVersion::SurfaceTurnoverV1 => {}
    }
    s.into_bytes()
}

pub fn configuration_hash(params: &SimParams, grid: &GridConfiguration) -> String {
    let mut data = canonical_params_bytes(params);
    data.extend_from_slice(
        format!(
            "grid={}x{};dx={};dish={};reservoir={}",
            grid.width, grid.height, grid.dx, grid.dish_radius, grid.reservoir_width
        )
        .as_bytes(),
    );
    sha256_hex(&data)
}

pub fn candidate_hash(params: &SimParams, grid: &GridConfiguration) -> String {
    let mut data = params.equation_version.as_str().as_bytes().to_vec();
    data.push(0);
    data.extend_from_slice(&canonical_params_bytes(params));
    data.push(0);
    data.extend_from_slice(
        format!(
            "grid={}x{};dx={};dish={};reservoir={}",
            grid.width, grid.height, grid.dx, grid.dish_radius, grid.reservoir_width
        )
        .as_bytes(),
    );
    sha256_hex(&data)
}

pub fn build_candidate_identity(
    params: SimParams,
    source_commit: &str,
    calibration_branch: Option<&str>,
    calibration_iteration: Option<u32>,
    selection_reason: &str,
    source_snapshot_id: Option<String>,
    source_snapshot_hash: Option<String>,
) -> CandidateIdentity {
    let grid = GridConfiguration::default();
    let config_hash = configuration_hash(&params, &grid);
    let cand_hash = candidate_hash(&params, &grid);
    let candidate_id = format!(
        "cand-{}-kphi{}-ks{:.6}-kr{:.6}",
        &cand_hash[..12],
        params.k_phi,
        params.k_structure,
        params.k_rep
    );
    CandidateIdentity {
        candidate_id,
        equation_version: params.equation_version,
        k_phi: params.k_phi,
        k_structure: match params.equation_version {
            EquationVersion::SurfaceTurnoverV1 => params.k_structure_interface,
            EquationVersion::D001BulkV1
            | EquationVersion::D003CrowdingV1
            | EquationVersion::MembraneMetabolismV1 => params.k_structure,
        },
        k_rep: params.k_rep,
        initial_condition: InitialConditionConfiguration {
            seed_r0: params.seed_r0,
            seed_interface_width: params.seed_interface_width,
            seed_catalyst_scale: params.seed_catalyst_scale,
            noise_amplitude: params.noise_amplitude,
        },
        random_seed: params.random_seed,
        configuration_hash: config_hash,
        candidate_hash: cand_hash,
        params,
        grid,
        source_snapshot_id,
        source_snapshot_hash,
        source_commit: source_commit.to_string(),
        calibration_branch: calibration_branch.map(str::to_string),
        calibration_iteration,
        selection_reason: selection_reason.to_string(),
    }
}

pub fn classify_candidate_match(
    observed_hash: &str,
    final_hashes: &[(&str, &str)],
    analytical_hash: &str,
    intermediate_hashes: &[&str],
) -> CandidateMatchClass {
    if final_hashes.iter().any(|(_, h)| *h == observed_hash) {
        return CandidateMatchClass::MatchFinalCalibratedCandidate;
    }
    if observed_hash == analytical_hash {
        return CandidateMatchClass::MatchAnalyticalInitialEstimate;
    }
    if intermediate_hashes.iter().any(|h| *h == observed_hash) {
        return CandidateMatchClass::MatchIntermediateIteration;
    }
    CandidateMatchClass::MatchUnknownConfiguration
}

pub fn sha256_hex(data: &[u8]) -> String {
    let digest = sha256(data);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

fn sha256(data: &[u8]) -> [u8; 32] {
    // ponytail: minimal SHA-256 (no extra crate)
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];
    let k: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];
    let bit_len = (data.len() as u64) * 8;
    let mut msg = data.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());
    for chunk in msg.chunks(64) {
        let mut w = [0u32; 64];
        for (i, word) in chunk.chunks(4).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word.get(1).copied().unwrap_or(0), word.get(2).copied().unwrap_or(0), word.get(3).copied().unwrap_or(0)]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(k[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }
    let mut out = [0u8; 32];
    for (i, v) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&v.to_be_bytes());
    }
    out
}
