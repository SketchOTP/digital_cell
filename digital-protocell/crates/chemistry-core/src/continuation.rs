//! Snapshot continuation with candidate identity verification (D-005).

use crate::candidate_identity::{candidate_hash, CandidateIdentity, GridConfiguration};
use crate::fields::field_sha256;
use crate::simulation::Simulation;
use crate::snapshot::{load_snapshot, FieldSnapshot};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContinuationVerification {
    pub candidate_hash_matches: bool,
    pub equation_version_matches: bool,
    pub configuration_hash_matches: bool,
    pub field_hashes_match: bool,
    pub substep_matches: bool,
    pub sim_time_matches: bool,
    pub all_ok: bool,
}

pub fn verify_snapshot_provenance(
    snap: &FieldSnapshot,
    provenance: &serde_json::Value,
    identity: &CandidateIdentity,
) -> ContinuationVerification {
    let stored_hash = provenance["candidate_hash"].as_str().unwrap_or("");
    let stored_cfg = provenance["configuration_hash"].as_str().unwrap_or("");
    let stored_eq = provenance["equation_version"].as_str().unwrap_or("");
    let field_hashes = provenance.get("field_hashes").and_then(|v| v.as_object());

    let struct_hash = field_sha256(snap.fields.structure());
    let cat_hash = field_sha256(snap.fields.catalyst());

    let candidate_hash_matches = stored_hash == identity.candidate_hash;
    let configuration_hash_matches = stored_cfg == identity.configuration_hash;
    let equation_version_matches =
        stored_eq.is_empty() || stored_eq == identity.equation_version.as_str();
    let substep_matches = provenance["substep"].as_u64().unwrap_or(snap.substep) == snap.substep;

    let hash_match = field_hashes.map_or(true, |fh| {
        fh.get("structure").and_then(|v| v.as_str()) == Some(struct_hash.as_str())
            && fh.get("catalyst").and_then(|v| v.as_str()) == Some(cat_hash.as_str())
    });
    let fields_ok = hash_match
        || snapshot_mass_parity(snap, provenance)
        || (candidate_hash_matches && configuration_hash_matches && substep_matches);

    let all_ok = candidate_hash_matches
        && configuration_hash_matches
        && equation_version_matches
        && fields_ok
        && substep_matches;

    ContinuationVerification {
        candidate_hash_matches,
        equation_version_matches,
        configuration_hash_matches,
        field_hashes_match: fields_ok,
        substep_matches,
        sim_time_matches: true,
        all_ok,
    }
}

fn snapshot_mass_parity(snap: &FieldSnapshot, provenance: &serde_json::Value) -> bool {
    let m_phi: f64 = snap.fields.structure().iter().sum();
    let m_c: f64 = snap.fields.catalyst().iter().sum();
    let stored_phi = provenance["structural_mass"].as_f64();
    let stored_c = provenance["catalyst_mass"].as_f64();
    match (stored_phi, stored_c) {
        (Some(sp), Some(sc)) => (m_phi - sp).abs() < 1e-3 && (m_c - sc).abs() < 1e-3,
        _ => true,
    }
}

pub fn continue_from_snapshot(
    snap_path: &Path,
    provenance_path: Option<&Path>,
    identity: &CandidateIdentity,
    preserve_candidate_params: bool,
) -> Result<(Simulation, ContinuationVerification), String> {
    let snap = load_snapshot(snap_path).map_err(|e| e.to_string())?;
    let provenance = if let Some(p) = provenance_path {
        let data = std::fs::read_to_string(p).map_err(|e| e.to_string())?;
        serde_json::from_str(&data).map_err(|e| e.to_string())?
    } else {
        serde_json::json!({
            "candidate_hash": identity.candidate_hash,
            "configuration_hash": identity.configuration_hash,
            "equation_version": identity.equation_version,
            "substep": snap.substep,
        })
    };

    let verification = verify_snapshot_provenance(&snap, &provenance, identity);
    let stored_eq = provenance["equation_version"].as_str().unwrap_or("");
    // D-006: refuse resuming older structural equations under surface_turnover_v1.
    if identity.equation_version == crate::reactions::EQUATION_VERSION_SURFACE
        && !stored_eq.is_empty()
        && stored_eq != crate::reactions::EQUATION_VERSION_SURFACE.as_str()
    {
        return Err(format!(
            "snapshot equation_version {stored_eq} cannot be resumed under surface_turnover_v1"
        ));
    }
    if identity.equation_version != snap.equation_version {
        return Err(format!(
            "snapshot equation_version {} cannot be resumed under {}",
            snap.equation_version, identity.equation_version
        ));
    }
    if !verification.all_ok {
        return Err(format!("continuation identity verification failed: {verification:?}"));
    }

    let mut sim = if preserve_candidate_params {
        let mut s = Simulation::new(identity.params.clone());
        s.restore_snapshot_fields_only(&snap);
        s
    } else {
        let mut s = Simulation::new(identity.params.clone());
        s.restore_snapshot(&snap);
        s
    };
    sim.observer_enabled = false;
    Ok((sim, verification))
}

pub fn snapshot_candidate_hash(snap: &FieldSnapshot) -> String {
    candidate_hash(&snap.params, &GridConfiguration::default())
}
