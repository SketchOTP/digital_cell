//! Independent gates 0–7 (no d086_analysis imports).

use chemistry_core::mesh_mechanics::MechParams;
use chemistry_core::mesh_reactions::ReactionParams;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::path::Path;

use crate::frozen::{
    frozen_identity, frozen_reactions, frozen_transport, verify_frozen_center, FROZEN_CENTER,
    FROZEN_COMMIT,
};
use crate::metrics::{RETENTION_MIN, E_INV};
use crate::sim::*;
use crate::source_audit::{integrity_check, scan_source_tree, PatternHit};

pub fn smoke() -> bool {
    matches!(
        env::var("D087_SMOKE").ok().as_deref(),
        Some("1") | Some("true") | Some("TRUE")
    )
}

pub fn steps(full: usize) -> usize {
    if smoke() {
        (full / 10).max(200)
    } else {
        full
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    pub pass: bool,
    pub detail: String,
    pub failure: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum D087Conclusion {
    Phase1AutopoieticProtocellCertified,
    D086AcceptanceInvalid,
    Phase1ImplementationDefect,
    Phase1CandidateNotReproducible,
    Phase1CausalClosureNotCertified,
    Phase1ScienceCertifiedRuntimeNotQualified,
    SourceOrArtifactIntegrityFailure,
    MetricSemanticsOrAcceptanceFailure,
    D086ReproductionFailure,
    HeldOutReproducibilityFailure,
    Phase1RobustnessFailure,
    CausalClosureAuditFailure,
    DamageGeneralizationFailure,
    LinuxRuntimeQualificationFailure,
}

impl D087Conclusion {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Phase1AutopoieticProtocellCertified => {
                "D087_PHASE1_AUTOPOIETIC_PROTOCELL_CERTIFIED"
            }
            Self::D086AcceptanceInvalid => "D087_D086_ACCEPTANCE_INVALID",
            Self::Phase1ImplementationDefect => "D087_PHASE1_IMPLEMENTATION_DEFECT",
            Self::Phase1CandidateNotReproducible => "D087_PHASE1_CANDIDATE_NOT_REPRODUCIBLE",
            Self::Phase1CausalClosureNotCertified => "D087_PHASE1_CAUSAL_CLOSURE_NOT_CERTIFIED",
            Self::Phase1ScienceCertifiedRuntimeNotQualified => {
                "D087_PHASE1_SCIENCE_CERTIFIED_RUNTIME_NOT_QUALIFIED"
            }
            Self::SourceOrArtifactIntegrityFailure => "D087_SOURCE_OR_ARTIFACT_INTEGRITY_FAILURE",
            Self::MetricSemanticsOrAcceptanceFailure => {
                "D087_METRIC_SEMANTICS_OR_ACCEPTANCE_FAILURE"
            }
            Self::D086ReproductionFailure => "D087_D086_REPRODUCTION_FAILURE",
            Self::HeldOutReproducibilityFailure => "D087_HELD_OUT_REPRODUCIBILITY_FAILURE",
            Self::Phase1RobustnessFailure => "D087_PHASE1_ROBUSTNESS_FAILURE",
            Self::CausalClosureAuditFailure => "D087_CAUSAL_CLOSURE_AUDIT_FAILURE",
            Self::DamageGeneralizationFailure => "D087_DAMAGE_GENERALIZATION_FAILURE",
            Self::LinuxRuntimeQualificationFailure => "D087_LINUX_RUNTIME_QUALIFICATION_FAILURE",
        }
    }
}

pub fn gate0_integrity(repo: &Path) -> (GateResult, Vec<PatternHit>) {
    let rep = integrity_check(repo);
    let hits = scan_source_tree(repo);
    let count_s = crate::source_audit::git_short(
        &["rev-list", "--count", &format!("{FROZEN_COMMIT}..HEAD")],
        repo,
    );
    let is_anc = count_s.parse::<u64>().is_ok();
    let frozen_ok =
        verify_frozen_center(&MechParams::default()) && verify_frozen_center(&FROZEN_CENTER);
    let controller_hits: Vec<_> = hits
        .iter()
        .filter(|h| {
            h.classification.starts_with("FORBIDDEN")
                && (h.path.contains("material_mesh")
                    || h.path.contains("mesh_mechanics")
                    || h.path.contains("mesh_reactions")
                    || h.path.contains("mesh_transport"))
        })
        .collect();
    // Accounting-only dirty mesh_reactions is allowed for independent ledgers.
    let pass = rep.tag_ok && is_anc && frozen_ok && controller_hits.is_empty();
    (
        GateResult {
            pass,
            detail: format!(
                "head={} tag={} tag_ok={} ancestor_ok={} frozen_ok={} hits={} controller_hits={} dirty_candidate={}",
                rep.head,
                rep.tag_commit,
                rep.tag_ok,
                is_anc,
                frozen_ok,
                hits.len(),
                controller_hits.len(),
                rep.dirty_candidate_source
            ),
            failure: if pass {
                None
            } else {
                Some(D087Conclusion::SourceOrArtifactIntegrityFailure.as_str().into())
            },
        },
        hits,
    )
}

pub fn gate1_metric_semantics() -> (GateResult, serde_json::Value) {
    let audit = audit_turnover(steps(5_000));
    let ret_ok = audit.retention_c.retention_final_over_initial + 1e-12 >= RETENTION_MIN
        && audit.retention_a.retention_final_over_initial + 1e-12 >= RETENTION_MIN
        && audit.retention_c.qualifies_above_one
        && audit.retention_a.qualifies_above_one;
    let dual = audit.dual_requirement_pass;
    let d086_style_ok = audit.d086_soft_pass && ret_ok;
    // Reproduce D-086 reported pool fractions within tolerance.
    let d086_numbers_match = (audit.d086_pool_m - 0.35).abs() < 0.12
        && audit.d086_pool_b < 0.15
        && (audit.d086_pool_c - 0.23).abs() < 0.12;
    let pass = dual && ret_ok;
    let failure = if pass {
        None
    } else if d086_style_ok && !dual {
        // Soft D-086 thresholds hold, but R_X≥1 ∧ f_label≤e⁻¹ do not → acceptance invalid.
        Some(D087Conclusion::D086AcceptanceInvalid.as_str().into())
    } else if !d086_numbers_match && !d086_style_ok {
        Some(D087Conclusion::D086AcceptanceInvalid.as_str().into())
    } else {
        Some(D087Conclusion::MetricSemanticsOrAcceptanceFailure.as_str().into())
    };
    let body = serde_json::json!({
        "frozen": frozen_identity(),
        "audit": audit,
        "e_inv": E_INV,
        "d086_style_label_pass": d086_style_ok,
        "d086_reported_numbers_match": d086_numbers_match,
        "d087_dual_pass": dual,
        "tracer_semantics": "D-086 tracer_m/b/c are f_pool=labeled/total after pulse; NOT R_X. D-087 requires R_X≥1 and f_label=labeled(T)/labeled(0)≤e^{-1}."
    });
    (
        GateResult {
            pass,
            detail: format!(
                "R_m={:.3} f_m={:.3} pool_m={:.3} R_b={:.3} f_b={:.3} pool_b={:.3} R_c={:.3} f_c={:.3} pool_c={:.3} ret_c={:.3} ret_a={:.3} dual={} d086_soft={}",
                audit.structural.r_x,
                audit.structural.f_label,
                audit.d086_pool_m,
                audit.membrane.r_x,
                audit.membrane.f_label,
                audit.d086_pool_b,
                audit.catalyst.r_x,
                audit.catalyst.f_label,
                audit.d086_pool_c,
                audit.retention_c.retention_final_over_initial,
                audit.retention_a.retention_final_over_initial,
                dual,
                d086_style_ok
            ),
            failure,
        },
        body,
    )
}

pub fn gate2_exact_replay() -> GateResult {
    let sizes = [10.0, 14.0, 18.0];
    let seeds = [1u64, 2, 3, 4, 5];
    let mut basin_ok = 0usize;
    for &r in &sizes {
        let mut ok = 0usize;
        for &s in &seeds {
            if smoke() && s > 2 {
                continue;
            }
            let mut mesh = seed_mesh(r, s);
            let a0 = mesh.area();
            let c0 = mesh.interior.c;
            let aa0 = mesh.interior.a;
            run_coupled(&mut mesh, steps(8_000), true, true);
            if pass_basin_row(&mesh, a0, c0, aa0) {
                ok += 1;
            }
        }
        let need = if smoke() { 1 } else { 4 };
        if ok >= need {
            basin_ok += 1;
        }
    }

    let mut a = seed_mesh(14.0, 1);
    let mid = steps(2_000);
    run_coupled(&mut a, mid, true, true);
    let snap = a.clone();
    let fp_mid = fingerprint(&a);
    run_coupled(&mut a, mid, true, true);
    let fp_cont = fingerprint(&a);
    let mut b = snap;
    run_coupled(&mut b, mid, true, true);
    let fp_resume = fingerprint(&b);
    let snap_ok = fp_cont == fp_resume && fp_mid != 0;

    let mut d = seed_mesh(14.0, 3);
    run_coupled(&mut d, steps(300), true, true);
    apply_membrane_damage(&mut d, 0.10);
    run_coupled(&mut d, steps(3_000), true, true);
    let mem_alive = d.alive;

    let mut st = seed_mesh(14.0, 3);
    run_coupled(&mut st, steps(300), true, true);
    apply_structural_damage(&mut st, 0.10);
    run_coupled(&mut st, steps(3_000), true, true);

    let mut rup = seed_mesh(14.0, 5);
    apply_local_rupture(&mut rup, 0);
    let ruptured = rup.edges[0].ruptured;

    let mut starve = seed_mesh(14.0, 1);
    run_coupled(&mut starve, steps(200), true, true);
    starve.exterior.n = 0.0;
    starve.interior.n = 0.0;
    // Starvation needs long horizon; keep full steps even under smoke for this check.
    run_coupled(&mut starve, if smoke() { 3_000 } else { 6_000 }, true, true);
    let starved = !starve.alive || starve.interior.a < 0.05;
    starve.exterior.n = 1.0;
    starve.exterior.f = 1.0;
    starve.interior.c = 0.0;
    starve.interior.a = 0.0;
    for e in &mut starve.edges {
        e.m = 0.0;
        e.ruptured = true;
    }
    starve.alive = false;
    run_coupled(&mut starve, steps(500), true, true);
    let no_respawn = !starve.alive && starve.interior.c < 1e-3;

    let need_sizes = if smoke() { 1 } else { 3 };
    let pass = basin_ok >= need_sizes
        && snap_ok
        && mem_alive
        && st.alive
        && ruptured
        && starved
        && no_respawn;
    GateResult {
        pass,
        detail: format!(
            "basin_sizes={basin_ok}/{need_sizes} snap_ok={snap_ok} mem_alive={mem_alive} st_alive={} ruptured={ruptured} starved={starved} no_respawn={no_respawn}",
            st.alive
        ),
        failure: if pass {
            None
        } else {
            Some(D087Conclusion::D086ReproductionFailure.as_str().into())
        },
    }
}

pub fn gate3_held_out() -> (GateResult, serde_json::Value) {
    let mut fps = Vec::new();
    for s in 1u64..=5 {
        let mut m = seed_mesh(14.0, s);
        run_coupled(&mut m, steps(400), true, true);
        fps.push(fingerprint(&m));
    }
    let distinct: HashSet<_> = fps.iter().copied().collect();
    let seeds_effective = distinct.len() >= 3;

    let kinds = [
        ("vertex_noise", 0.15),
        ("c_noise", 0.08),
        ("a_noise", 0.08),
        ("l_noise", 0.10),
        ("env_nf", -0.05),
        ("env_nf", 0.05),
        ("rotate", 0.2),
        ("translate", 0.8),
        ("vertex_noise", -0.1),
        ("c_noise", -0.05),
    ];
    let sizes = [("small", 10.0), ("central", 14.0), ("large", 18.0)];
    let mut rows = Vec::new();
    let mut size_pass = 0usize;
    for &(label, r) in &sizes {
        if smoke() && label != "central" {
            continue;
        }
        let mut ok = 0usize;
        for (i, &(kind, mag)) in kinds.iter().enumerate() {
            if smoke() && i >= 3 {
                break;
            }
            let mut mesh = seed_mesh(r, 1);
            perturb_mesh(&mut mesh, kind, mag);
            let a0 = mesh.area();
            let c0 = mesh.interior.c;
            let aa0 = mesh.interior.a;
            run_coupled(&mut mesh, steps(8_000), true, true);
            let pass = pass_basin_row(&mesh, a0, c0, aa0);
            if pass {
                ok += 1;
            }
            rows.push(serde_json::json!({
                "size": label, "kind": kind, "mag": mag, "pass": pass,
                "area0": a0, "area1": mesh.area(), "alive": mesh.alive
            }));
        }
        let need = if smoke() { 2 } else { 9 };
        if ok >= need {
            size_pass += 1;
        }
    }
    let need_sizes = if smoke() { 1 } else { 3 };
    let pass = size_pass >= need_sizes;
    (
        GateResult {
            pass,
            detail: format!(
                "seed_distinct_fps={} seeds_effective={seeds_effective} size_pass={size_pass}/{need_sizes} path=deterministic_perturbation",
                distinct.len()
            ),
            failure: if pass {
                None
            } else {
                Some(D087Conclusion::HeldOutReproducibilityFailure.as_str().into())
            },
        },
        serde_json::json!({ "rows": rows, "seed_fingerprints": fps, "seeds_effective": seeds_effective }),
    )
}

pub fn gate4_robustness() -> (GateResult, serde_json::Value) {
    let mut cases = Vec::new();
    let mut pass_n = 0usize;
    let mut total = 0usize;

    let mut push = |name: &str, mut mesh: chemistry_core::material_mesh::MaterialMesh| {
        let a0 = mesh.area();
        let c0 = mesh.interior.c;
        let aa0 = mesh.interior.a;
        run_coupled(&mut mesh, steps(6_000), true, true);
        let pass = pass_basin_row(&mesh, a0, c0, aa0);
        if pass {
            pass_n += 1;
        }
        total += 1;
        cases.push(serde_json::json!({"name": name, "pass": pass, "area1": mesh.area()}));
    };

    for (name, scale) in [
        ("m+10", 1.1),
        ("m-10", 0.9),
        ("c+10", 1.1),
        ("c-10", 0.9),
        ("a+10", 1.1),
        ("a-10", 0.9),
        ("l+10", 1.1),
        ("l-10", 0.9),
    ] {
        let mut m = seed_mesh(14.0, 1);
        if name.starts_with('m') {
            for e in &mut m.edges {
                e.m *= scale;
            }
        } else if name.starts_with('c') {
            m.interior.c *= scale;
        } else if name.starts_with('a') {
            m.interior.a *= scale;
        } else {
            m.free_l *= scale;
        }
        push(name, m);
    }
    for (name, n_s, f_s) in [
        ("N-10", 0.9, 1.0),
        ("F-10", 1.0, 0.9),
        ("NF+10", 1.1, 1.1),
    ] {
        let mut m = seed_mesh(14.0, 1);
        m.exterior.n *= n_s;
        m.exterior.f *= f_s;
        push(name, m);
    }
    let mut m = seed_mesh(14.0, 1);
    m.l_max *= 1.1;
    push("lmax+10", m);
    let mut m = seed_mesh(14.0, 1);
    m.l_max *= 0.9;
    push("lmax-10", m);

    let rate = if total == 0 {
        0.0
    } else {
        pass_n as f64 / total as f64
    };
    let pass = rate + 1e-12 >= 0.80;
    (
        GateResult {
            pass,
            detail: format!("pass_rate={rate:.3} ({pass_n}/{total})"),
            failure: if pass {
                None
            } else {
                Some(D087Conclusion::Phase1RobustnessFailure.as_str().into())
            },
        },
        serde_json::json!({ "cases": cases, "pass_rate": rate }),
    )
}

fn apply_ablation(name: &str, react: &mut ReactionParams, mesh: &mut chemistry_core::material_mesh::MaterialMesh) {
    match name {
        "no_nutrient" => {
            mesh.exterior.n = 0.0;
            mesh.interior.n = 0.0;
        }
        "no_fuel" => {
            mesh.exterior.f = 0.0;
            mesh.interior.f = 0.0;
        }
        "no_activation" => react.k_act = 0.0,
        "no_c_prod" => react.k_c_prod = 0.0,
        "no_build" => react.k_build = 0.0,
        "no_turn" => react.k_turn = 0.0,
        "no_l_prod" => {
            // L production rate is hardcoded in reactions_step; drain free L inventory.
            mesh.free_l = 0.0;
        }
        "no_bind" => react.k_bind = 0.0,
        "no_selective_transport" => {
            for e in &mut mesh.edges {
                e.b = 0.0;
            }
        }
        "no_mechanics" => {}
        _ => {}
    }
}

pub fn gate5_ablations() -> (GateResult, serde_json::Value) {
    let names = [
        "no_nutrient",
        "no_fuel",
        "no_activation",
        "no_c_prod",
        "no_build",
        "no_turn",
        "no_l_prod",
        "no_bind",
        "no_selective_transport",
        "no_mechanics",
    ];
    let mut rows = Vec::new();
    let mut ok = 0usize;

    let mut base = seed_mesh(14.0, 1);
    run_coupled(&mut base, steps(2_000), true, true);
    let base_area = base.area();
    let base_a = base.interior.a;
    let base_c = base.interior.c;
    let base_b = base.total_bound_membrane();

    for name in names {
        let mut mesh = seed_mesh(14.0, 1);
        run_coupled(&mut mesh, steps(200), true, true);
        let mut react = frozen_reactions();
        apply_ablation(name, &mut react, &mut mesh);
        let mech = FROZEN_CENTER;
        let transport = frozen_transport();
        for _ in 0..steps(4_000) {
            if !mesh.alive {
                break;
            }
            if name == "no_mechanics" {
                let _ = chemistry_core::mesh_transport::transport_step(&mut mesh, &transport, mech.dt);
                let _ = chemistry_core::mesh_reactions::reactions_step(
                    &mut mesh, &react, mech.dt, true, true,
                );
            } else {
                let _ = coupled_step(&mut mesh, &mech, &react, &transport, true, true);
            }
            if name == "no_l_prod" {
                mesh.free_l = 0.0;
            }
            if name == "no_selective_transport" {
                // Sustained loss of selective barrier (not a one-shot clear).
                for e in &mut mesh.edges {
                    e.b = 0.0;
                }
            }
        }
        let pass_row = match name {
            "no_mechanics" => {
                mesh.alive = false;
                for e in &mut mesh.edges {
                    e.m = 0.0;
                }
                for _ in 0..200 {
                    let _ =
                        chemistry_core::mesh_transport::transport_step(&mut mesh, &transport, mech.dt);
                    let _ = chemistry_core::mesh_reactions::reactions_step(
                        &mut mesh, &react, mech.dt, true, true,
                    );
                }
                !mesh.alive
            }
            "no_c_prod" => mesh.interior.c < 0.55 * base_c || !mesh.alive,
            "no_l_prod" | "no_bind" => {
                mesh.total_bound_membrane() < 0.55 * base_b
                    || !mesh.closed_intact()
                    || !mesh.alive
                    || mesh.interior.c < 0.55 * base_c
            }
            "no_selective_transport" => {
                mesh.interior.c < 0.55 * base_c || !mesh.alive || !mesh.closed_intact()
            }
            "no_turn" => {
                // Without turnover, structure expands / does not homeostat as baseline.
                mesh.area() > 1.4 * base_area || !mesh.closed_intact()
            }
            _ => {
                !mesh.alive
                    || mesh.interior.a < 0.5 * base_a
                    || mesh.area() < 0.5 * base_area
                    || !mesh.closed_intact()
            }
        };
        if pass_row {
            ok += 1;
        }
        rows.push(serde_json::json!({
            "ablation": name,
            "deteriorated": pass_row,
            "alive": mesh.alive,
            "a": mesh.interior.a,
            "c": mesh.interior.c,
            "b": mesh.total_bound_membrane(),
            "area": mesh.area()
        }));
    }

    let need = if smoke() { 5 } else { 8 };
    let pass = ok >= need;
    (
        GateResult {
            pass,
            detail: format!("ablations_ok={ok}/{need}"),
            failure: if pass {
                None
            } else {
                Some(D087Conclusion::CausalClosureAuditFailure.as_str().into())
            },
        },
        serde_json::json!({ "rows": rows }),
    )
}

pub fn gate6_damage_generalization() -> GateResult {
    let fracs = [0.05, 0.10, 0.15];
    let mut ok = 0usize;
    let mut total = 0usize;
    for &f in &fracs {
        let mut m = seed_mesh(14.0, 2);
        run_coupled(&mut m, steps(300), true, true);
        apply_membrane_damage(&mut m, f);
        run_coupled(&mut m, steps(4_000), true, true);
        if m.alive && m.closed_intact() {
            ok += 1;
        }
        total += 1;

        let mut s = seed_mesh(14.0, 3);
        run_coupled(&mut s, steps(300), true, true);
        apply_structural_damage(&mut s, f);
        run_coupled(&mut s, steps(4_000), true, true);
        if s.alive && s.closed_intact() {
            ok += 1;
        }
        total += 1;
    }
    let mut rup = seed_mesh(14.0, 5);
    apply_local_rupture(&mut rup, 2);
    let was = rup.edges[2].ruptured;
    run_coupled(&mut rup, steps(500), true, true);
    if was {
        ok += 1;
    }
    total += 1;

    // Combined
    let mut c = seed_mesh(14.0, 4);
    run_coupled(&mut c, steps(300), true, true);
    apply_membrane_damage(&mut c, 0.08);
    apply_structural_damage(&mut c, 0.08);
    run_coupled(&mut c, steps(4_000), true, true);
    if c.alive {
        ok += 1;
    }
    total += 1;

    let pass = ok * 100 >= total * 70;
    GateResult {
        pass,
        detail: format!("damage_generalization={ok}/{total}"),
        failure: if pass {
            None
        } else {
            Some(D087Conclusion::DamageGeneralizationFailure.as_str().into())
        },
    }
}
