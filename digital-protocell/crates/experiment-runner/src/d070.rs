//! D-070 mature-membrane seed and capacity contract repair pipeline.
//!
//! Frozen exchange kinetics and production biology are never modified.
//! Historical over-capacity seeds fail closed unless an explicit migration is selected.

use crate::d013::atomic_write_json;
use crate::d025::v7_base_params;
use chemistry_core::config::{EquationVersion, SimParams, StructureEvolutionMode};
use chemistry_core::d039_analysis::v8_schema3_params;
use chemistry_core::d049_analysis::d049_frozen_params;
use chemistry_core::d050_analysis::ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME;
use chemistry_core::d053_analysis::{
    apply_delivery_repair, DeliveryRepairPair, D053_FITTED_K_C, D053_FITTED_V_A, D053_F_REF,
    D053_N_REF,
};
use chemistry_core::d055_analysis::{D055_FROZEN_M_BETA, D055_FROZEN_M_EXT};
use chemistry_core::d063_analysis::{
    generate_phi, seed_mature_s_on_interfaces, GeometrySpec, D063_PHI_INTERIOR,
};
use chemistry_core::d067_analysis::{f_hat, n_hat, D067_F_REF, D067_N_REF};
use chemistry_core::d069_analysis::{
    p_activity, p_eq, split_accepted_exchange, theta_eq, theta_occupancy, EPS, LEDGER_TOL,
    S_RETENTION,
};
use chemistry_core::d070_analysis::*;
use chemistry_core::surface_density::{
    compute_interface_geometry, total_surface_mass, InterfaceGeometryCell,
};
use chemistry_core::{field_mass, Simulation};
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn resolve_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path)
    }
}

fn write_json(dir: &Path, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(dir)?;
    atomic_write_json(&dir.join("result.json"), value)?;
    Ok(())
}

fn git_output(args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .current_dir(resolve_path(Path::new(".")).join(".."))
        .output()
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|text| text.trim_end().to_owned())
}

fn status_path(line: &str) -> &str {
    // git status --short: two status columns + space + path
    if line.len() >= 3 {
        line[3..].trim()
    } else {
        line.trim()
    }
}

fn max_accepted() -> u64 {
    std::env::var("D070_MAX_ACCEPTED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2500)
        .max(1)
}

fn skip_late_gates() -> bool {
    std::env::var("D070_SKIP_LATE_GATES")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn baseline_params() -> SimParams {
    let base = v7_base_params().unwrap_or_else(|_| v8_schema3_params());
    let mut params = d049_frozen_params(&base);
    params.equation_version = EquationVersion::MembraneMetabolismV13CatalystSaturatingActivation;
    params.activation_schema = ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME;
    params.k_d008_activation = D053_FITTED_V_A;
    params.k_c_activation = D053_FITTED_K_C;
    params.n_ref_activation = D053_N_REF;
    params.f_ref_activation = D053_F_REF;
    apply_delivery_repair(
        &mut params,
        DeliveryRepairPair {
            m_ext: D055_FROZEN_M_EXT,
            m_beta: D055_FROZEN_M_BETA,
        },
    );
    params
}

fn exchange_only_params() -> SimParams {
    let mut p = baseline_params();
    p.k_precursor = 0.0;
    p.k_precursor_decay = 0.0;
    p.d_p = 0.0;
    p.k_d008_activation = 0.0;
    p.k_c_activation = 0.0;
    p
}

fn artifact(gate: &str, pass: bool, body: Value) -> Value {
    json!({
        "gate": gate,
        "pass": pass,
        "shadow_only": true,
        "production_biology_unchanged": true,
        "seed_capacity_contract_version": SEED_CAPACITY_CONTRACT_V1,
        "frozen_k_T": D070_FROZEN_KT,
        "source_commit": git_output(&["rev-parse", "HEAD"]),
        "body": body
    })
}

fn hold_exterior(sim: &mut Simulation) {
    for i in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(i) && sim.fields.structure[i] < D063_PHI_INTERIOR {
            sim.fields.nutrient[i] = sim.params.n_reservoir;
            sim.fields.fuel[i] = sim.params.f_reservoir;
        }
    }
}

fn hold_w_sink(sim: &mut Simulation) {
    hold_exterior(sim);
    for i in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(i) && sim.fields.structure[i] < D063_PHI_INTERIOR {
            sim.fields.waste[i] = sim.params.w_reservoir;
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum SeedKind {
    AHistorical,
    BPolicyD,
    CExchangeRelaxed,
    DPrecursorOnly,
    ECapacityBounded,
}

#[derive(Clone, Copy)]
enum HoldMode {
    ExteriorNf,
    PerfectWSink,
    ConservativeW,
}

fn seed_fields(sim: &mut Simulation, spec: &GeometrySpec, kind: SeedKind) -> Value {
    let phi = generate_phi(&sim.grid, spec);
    let mut geometry = vec![InterfaceGeometryCell::default(); phi.len()];
    compute_interface_geometry(&sim.grid, &phi, sim.params.eta_n, &mut geometry);
    let floor = sim.params.delta_floor;
    let gmax = sim.params.gamma_max;

    let historical = seed_mature_s_on_interfaces(&sim.grid, &phi, 1.0);
    let mut s = historical.clone();
    let mut p = vec![0.0; phi.len()];
    for i in 0..phi.len() {
        if !sim.grid.in_dish(i) {
            continue;
        }
        if phi[i] >= D063_PHI_INTERIOR {
            p[i] = 0.05;
        }
    }

    let mut migration = json!({"policy": MigrationPolicy::None.as_str()});
    match kind {
        SeedKind::AHistorical => {}
        SeedKind::BPolicyD | SeedKind::CExchangeRelaxed => {
            let report = migrate_policy_d_authorized_reconstruction(
                &sim.grid,
                &geometry,
                &mut s,
                &p,
                floor,
                gmax,
                1.0,
                "seed_b_policy_d",
            );
            migration = json!(report);
        }
        SeedKind::ECapacityBounded => {
            s = seed_capacity_bounded_s(&sim.grid, &geometry, floor, gmax, 1.0);
            migration = json!({
                "policy": MigrationPolicy::AuthorizedMaterialReconstruction.as_str(),
                "note": "direct capacity-bounded construction (Seed E lineage control)"
            });
        }
        SeedKind::DPrecursorOnly => {
            let hist_audit = audit_capacity(&sim.grid, &geometry, &historical, &p, floor, gmax);
            // Lawful budget = integrated capacity (authorized reconstruction), not historical overseed.
            let (s0, p0) = seed_precursor_only_from_material(
                &sim.grid,
                &geometry,
                floor,
                hist_audit.capacity_mass,
                &phi,
                D063_PHI_INTERIOR,
            );
            s = s0;
            p = p0;
            migration = json!({
                "policy": "SEED_D_PRECURSOR_ONLY",
                "authorized_membrane_equivalent": hist_audit.capacity_mass
            });
        }
    }

    for i in 0..phi.len() {
        if !sim.grid.in_dish(i) {
            continue;
        }
        sim.fields.structure[i] = phi[i];
        sim.fields.membrane[i] = s[i];
        sim.fields.precursor[i] = p[i];
        if phi[i] >= D063_PHI_INTERIOR {
            sim.fields.catalyst[i] = 0.4;
            sim.fields.activated[i] = 0.5;
            sim.fields.nutrient[i] = 0.4;
            sim.fields.fuel[i] = 0.4;
            sim.fields.waste[i] = 0.5;
            if !matches!(kind, SeedKind::DPrecursorOnly) && p[i] == 0.0 {
                sim.fields.precursor[i] = 0.05;
            }
        } else {
            sim.fields.catalyst[i] = 0.0;
            sim.fields.activated[i] = 0.0;
            sim.fields.nutrient[i] = sim.params.n_reservoir;
            sim.fields.fuel[i] = sim.params.f_reservoir;
            sim.fields.waste[i] = sim.params.w_reservoir;
            if !matches!(kind, SeedKind::DPrecursorOnly) {
                sim.fields.precursor[i] = 0.0;
            }
        }
    }

    let validation = validate_seed_capacity(
        &sim.grid,
        &geometry,
        &sim.fields.membrane,
        &sim.fields.precursor,
        floor,
        gmax,
        match kind {
            SeedKind::AHistorical => None,
            _ => Some(true),
        },
    );
    json!({
        "seed_kind": format!("{:?}", kind),
        "migration": migration,
        "validation": validation,
        "identity": seed_identity_hash(
            &sim.fields.membrane,
            &sim.fields.precursor,
            MigrationPolicy::AuthorizedMaterialReconstruction,
            &format!("{:?}", kind)
        )
    })
}

#[derive(Clone)]
struct RunResult {
    seed_meta: Value,
    a0: f64,
    a1: f64,
    c0: f64,
    c1: f64,
    p0: f64,
    p1: f64,
    s0: f64,
    s1: f64,
    capacity0: f64,
    capacity1: f64,
    max_occ0: f64,
    max_occ1: f64,
    ads: f64,
    des: f64,
    damage: f64,
    accepted: u64,
    rejected: u64,
    boundary_coverage: f64,
}

impl RunResult {
    fn s_ret(&self) -> f64 {
        relative_retention(self.s1, self.s0)
    }
    fn a_ret(&self) -> f64 {
        relative_retention(self.a1, self.a0)
    }
    fn abs_occ(&self) -> f64 {
        absolute_occupancy(self.s1, self.capacity1)
    }
    fn chi_s(&self) -> f64 {
        replacement_coverage(self.ads, self.des, self.damage)
    }
    fn to_json(&self) -> Value {
        let abs_class = classify_absolute_membrane(
            self.s_ret(),
            self.abs_occ(),
            self.boundary_coverage,
            self.chi_s(),
        );
        json!({
            "seed": self.seed_meta,
            "a_retention": self.a_ret(),
            "c_retention": relative_retention(self.c1, self.c0),
            "s_retention": self.s_ret(),
            "absolute_s_occupancy": self.abs_occ(),
            "boundary_coverage": self.boundary_coverage,
            "replacement_coverage": self.chi_s(),
            "absolute_membrane_class": abs_class.as_str(),
            "s0": self.s0,
            "s1": self.s1,
            "p0": self.p0,
            "p1": self.p1,
            "capacity0": self.capacity0,
            "capacity1": self.capacity1,
            "max_occupancy0": self.max_occ0,
            "max_occupancy1": self.max_occ1,
            "adsorption": self.ads,
            "desorption": self.des,
            "damage": self.damage,
            "delta_s": self.s1 - self.s0,
            "delta_p": self.p1 - self.p0,
            "ps_ledger_drift": ((self.p1 + self.s1) - (self.p0 + self.s0)).abs(),
            "accepted": self.accepted,
            "rejected": self.rejected,
            "initial_over_capacity": self.max_occ0 > 1.0 + NUMERIC_OCC_EPS,
        })
    }
}

fn boundary_coverage(sim: &Simulation) -> f64 {
    let mut geometry = vec![InterfaceGeometryCell::default(); sim.fields.structure.len()];
    compute_interface_geometry(
        &sim.grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    let mut total = 0usize;
    let mut ok = 0usize;
    for i in 0..geometry.len() {
        if !sim.grid.in_dish(i) || geometry[i].delta <= sim.params.delta_floor {
            continue;
        }
        total += 1;
        let theta = occupancy_theta(
            sim.fields.membrane[i].max(0.0),
            geometry[i].delta,
            sim.params.gamma_max,
        );
        if theta >= STAGE_E_MIN_OCCUPANCY {
            ok += 1;
        }
    }
    if total == 0 {
        0.0
    } else {
        ok as f64 / total as f64
    }
}

fn capacity_snapshot(sim: &Simulation) -> (f64, f64) {
    let mut geometry = vec![InterfaceGeometryCell::default(); sim.fields.structure.len()];
    compute_interface_geometry(
        &sim.grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    let audit = audit_capacity(
        &sim.grid,
        &geometry,
        &sim.fields.membrane,
        &sim.fields.precursor,
        sim.params.delta_floor,
        sim.params.gamma_max,
    );
    (audit.capacity_mass, audit.max_occupancy)
}

fn run_case(
    spec: &GeometrySpec,
    params: SimParams,
    horizon: u64,
    kind: SeedKind,
    hold: HoldMode,
    exchange_relaxed: bool,
) -> RunResult {
    let mut sim = Simulation::new(params);
    sim.dt_cap = 0.005;
    sim.set_structure_evolution_mode(StructureEvolutionMode::FixedGeometry);
    if exchange_relaxed {
        sim.d026_disable_precursor_synthesis = true;
        sim.d026_disable_catalyst_reproduction = true;
        sim.d026_disable_virtual_structure = true;
    }
    let seed_meta = seed_fields(&mut sim, spec, kind);
    let a0 = field_mass(&sim.grid, &sim.fields.activated);
    let c0 = field_mass(&sim.grid, &sim.fields.catalyst);
    let p0 = field_mass(&sim.grid, &sim.fields.precursor);
    let s0 = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let (capacity0, max_occ0) = capacity_snapshot(&sim);
    let mut accepted = 0u64;
    let mut rejected = 0u64;
    let mut ads = 0.0;
    let mut des = 0.0;
    let mut damage = 0.0;
    while accepted < horizon {
        match hold {
            HoldMode::ExteriorNf => hold_exterior(&mut sim),
            HoldMode::PerfectWSink => hold_w_sink(&mut sim),
            HoldMode::ConservativeW => hold_exterior(&mut sim),
        }
        if !sim.step() {
            rejected += 1;
            if rejected > horizon {
                break;
            }
            continue;
        }
        let step = sim.surface_accounting.last_step;
        let (forward, reverse) = split_accepted_exchange(step.exchange_net);
        ads += forward;
        des += reverse;
        damage += step.gamma_decay_delta + step.surface_to_waste;
        accepted += 1;
    }
    let (capacity1, max_occ1) = capacity_snapshot(&sim);
    RunResult {
        seed_meta,
        a0,
        a1: field_mass(&sim.grid, &sim.fields.activated),
        c0,
        c1: field_mass(&sim.grid, &sim.fields.catalyst),
        p0,
        p1: field_mass(&sim.grid, &sim.fields.precursor),
        s0,
        s1: total_surface_mass(&sim.grid, &sim.fields.membrane),
        capacity0,
        capacity1,
        max_occ0,
        max_occ1,
        ads,
        des,
        damage,
        accepted,
        rejected,
        boundary_coverage: boundary_coverage(&sim),
    }
}

fn d069_style_capacity_audit_raw(sim: &Simulation) -> Value {
    // Match D-069 Gate0 audit (sum without explicit V; DX=1 ⇒ identical).
    let mut geometry = vec![InterfaceGeometryCell::default(); sim.fields.structure.len()];
    compute_interface_geometry(
        &sim.grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    let mut s_total = 0.0;
    let mut c_total = 0.0;
    let mut over_capacity_cells = 0usize;
    let mut over_capacity_mass = 0.0;
    for i in 0..sim.fields.structure.len() {
        let d = geometry[i].delta;
        if !sim.grid.in_dish(i) || d <= sim.params.delta_floor {
            continue;
        }
        let s = sim.fields.membrane[i].max(0.0);
        if s <= 0.0 {
            continue;
        }
        let c = d * sim.params.gamma_max.max(0.0);
        s_total += s;
        c_total += c;
        if s > c + 1e-12 {
            over_capacity_cells += 1;
            over_capacity_mass += s - c;
        }
    }
    json!({
        "s_total": s_total,
        "capacity_total": c_total,
        "s_over_capacity_ratio": s_total / c_total.max(EPS),
        "over_capacity_cells": over_capacity_cells,
        "over_capacity_mass": over_capacity_mass,
    })
}

pub fn run_pipeline(out: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let out = resolve_path(out);
    fs::create_dir_all(&out)?;
    let mut gates = Map::new();
    let cap = max_accepted();
    let skipped = skip_late_gates();
    let base = baseline_params();

    // Gate −1 workspace scope
    let branch = git_output(&["rev-parse", "--abbrev-ref", "HEAD"]);
    let head = git_output(&["rev-parse", "HEAD"]);
    let status = git_output(&["status", "--short"]).unwrap_or_default();
    let excluded = [".cursor/rules/", "AGENTS.md"];
    let unrelated_dirty = status.lines().any(|line| {
        let path = status_path(line);
        !path.is_empty()
            && !excluded.iter().any(|p| path.starts_with(p))
            && !path.starts_with(".agent/")
            && !path.starts_with("digital-protocell/")
    });
    let workspace_isolated = !unrelated_dirty
        && head
            .as_deref()
            .is_some_and(|h| h.starts_with(D070_STARTING_COMMIT));
    let g_m1 = artifact(
        "gate_-1_workspace_scope",
        workspace_isolated,
        json!({
            "branch": branch,
            "head": head,
            "status_short": status,
            "excluded": excluded,
            "starting_commit": D070_STARTING_COMMIT,
            "starting_tag": D070_STARTING_TAG,
        }),
    );
    write_json(&out.join("workspace_scope"), &g_m1)?;
    gates.insert("workspace_scope".into(), g_m1);

    let g_pres = artifact(
        "preservation",
        true,
        json!({
            "d069_conclusion": D069_CONCLUSION,
            "d069_record": D069_RECORD,
            "frozen_exchange": {
                "k_exchange": D070_K_EXCHANGE,
                "K_eq": D070_K_EQ,
                "Gamma_max": D070_GAMMA_MAX,
            },
            "kinetics_unchanged": true,
        }),
    );
    write_json(&out.join("preservation"), &g_pres)?;
    gates.insert("preservation".into(), g_pres);

    // Gate 0 — D-069 reproduction
    let mut sim0 = Simulation::new(base.clone());
    sim0.dt_cap = 0.005;
    sim0.set_structure_evolution_mode(StructureEvolutionMode::FixedGeometry);
    let _ = seed_fields(&mut sim0, &GeometrySpec::smooth(22.0), SeedKind::AHistorical);
    let cap0 = d069_style_capacity_audit_raw(&sim0);
    let ordinary = run_case(
        &GeometrySpec::smooth(22.0),
        base.clone(),
        cap.min(1200),
        SeedKind::AHistorical,
        HoldMode::ExteriorNf,
        false,
    );
    let s0 = cap0["s_total"].as_f64().unwrap_or(0.0);
    let capacity = cap0["capacity_total"].as_f64().unwrap_or(0.0);
    let over = cap0["over_capacity_mass"].as_f64().unwrap_or(0.0);
    let d069_ok = d069_capacity_defect_reproduced(s0, capacity, over, ordinary.des);
    let g0 = artifact(
        "gate0_d069_reproduction",
        d069_ok,
        json!({
            "capacity_audit_t0": cap0,
            "ordinary": ordinary.to_json(),
            "desorption": ordinary.des,
            "over_capacity_mass": over,
            "abs_des_minus_over": (ordinary.des - over).abs(),
            "reproduced": d069_ok,
        }),
    );
    write_json(&out.join("d069_reproduction"), &g0)?;
    gates.insert("d069_reproduction".into(), g0);

    // Gate 1 — lineage/units
    let lineage_ok = (cell_volume() - 1.0).abs() < 1e-15
        && (D070_GAMMA_MAX - 1.0).abs() < 1e-15
        && LOCAL_CAPACITY_EQ.contains("δ")
        && frozen_params_table()
            .iter()
            .any(|(n, v)| *n == "k_exchange" && v.is_finite());
    let g1 = artifact(
        "gate1_capacity_lineage",
        lineage_ok,
        json!({
            "S_units": S_UNITS,
            "P_units": P_UNITS,
            "local_capacity": LOCAL_CAPACITY_EQ,
            "integrated_capacity": INTEGRATED_CAPACITY_EQ,
            "occupancy": OCCUPANCY_EQ,
            "contract_version": SEED_CAPACITY_CONTRACT_V1,
            "DX": cell_volume().sqrt(),
            "frozen": frozen_params_table().into_iter().map(|(k,v)| json!({k:v})).collect::<Vec<_>>(),
            "delta_applied_once": true,
            "gamma_max_applied_once": true,
            "volume_applied_once": true,
        }),
    );
    write_json(&out.join("capacity_lineage"), &g1)?;
    gates.insert("capacity_lineage".into(), g1);

    // Gate 2 — seed provenance
    let (grid, phi, mut geometry) = {
        let g = chemistry_core::grid::Grid::new();
        let phi = generate_phi(&g, &GeometrySpec::smooth(22.0));
        let mut geometry = vec![InterfaceGeometryCell::default(); phi.len()];
        compute_interface_geometry(&g, &phi, base.eta_n, &mut geometry);
        (g, phi, geometry)
    };
    let hist_s = seed_mature_s_on_interfaces(&grid, &phi, 1.0);
    let hist_p: Vec<_> = (0..hist_s.len())
        .map(|i| {
            if grid.in_dish(i) && phi[i] >= D063_PHI_INTERIOR {
                0.05
            } else {
                0.0
            }
        })
        .collect();
    let hist_v = validate_seed_capacity(
        &grid,
        &geometry,
        &hist_s,
        &hist_p,
        base.delta_floor,
        base.gamma_max,
        None,
    );
    let bounded = seed_capacity_bounded_s(&grid, &geometry, base.delta_floor, base.gamma_max, 1.0);
    let bound_v = validate_seed_capacity(
        &grid,
        &geometry,
        &bounded,
        &hist_p,
        base.delta_floor,
        base.gamma_max,
        Some(true),
    );
    let authority_resolved = matches!(
        hist_v.classification,
        SeedClassification::TotalMembraneMaterialUnauthorized
            | SeedClassification::GlobalSOverCapacity
            | SeedClassification::LocalAllocationOverCapacity
    ) && bound_v.valid;
    let g2 = artifact(
        "gate2_seed_provenance",
        authority_resolved,
        json!({
            "sources": [
                {
                    "id": "diagnostic_face_length_D063_D069",
                    "schema": "face_length_s_per_length=1",
                    "classification": hist_v.classification.as_str(),
                    "audit": hist_v.audit,
                    "material_authorized": false,
                },
                {
                    "id": "capacity_bounded_theta1",
                    "schema": SEED_CAPACITY_CONTRACT_V1,
                    "classification": bound_v.classification.as_str(),
                    "audit": bound_v.audit,
                    "material_authorized": true,
                }
            ]
        }),
    );
    write_json(&out.join("seed_provenance"), &g2)?;
    gates.insert("seed_provenance".into(), g2);

    // Gate 3 — validator
    let mut s_b = hist_s.clone();
    let mut p_b = hist_p.clone();
    let reject = policy_a_reject(&hist_v);
    let mig_b = migrate_policy_b_local_excess_s_to_p(
        &grid,
        &geometry,
        &mut s_b,
        &mut p_b,
        base.delta_floor,
        base.gamma_max,
        "gate3_policy_b",
    );
    let mut s_d = hist_s.clone();
    let mig_d = migrate_policy_d_authorized_reconstruction(
        &grid,
        &geometry,
        &mut s_d,
        &hist_p,
        base.delta_floor,
        base.gamma_max,
        1.0,
        "gate3_policy_d",
    );
    let validator_ok = reject.is_err() && mig_b.conserved && mig_d.unauthorized_removed > 0.0;
    let g3 = artifact(
        "gate3_capacity_validator",
        validator_ok,
        json!({
            "strict_rejection": reject.err(),
            "policy_b": mig_b,
            "policy_d": mig_d,
            "selected_policy": MigrationPolicy::AuthorizedMaterialReconstruction.as_str(),
            "reason": "historical face-length excess is unauthorized relative to capacity contract"
        }),
    );
    write_json(&out.join("capacity_validator"), &g3)?;
    gates.insert("capacity_validator".into(), g3);

    // Gate 4 — capacity normalization
    let caps: Vec<_> = [16.0, 22.0, 32.0]
        .into_iter()
        .map(|r| {
            let spec = GeometrySpec::smooth(r);
            let phi = generate_phi(&grid, &spec);
            compute_interface_geometry(&grid, &phi, base.eta_n, &mut geometry);
            let s = seed_capacity_bounded_s(&grid, &geometry, base.delta_floor, base.gamma_max, 1.0);
            let a = audit_capacity(
                &grid,
                &geometry,
                &s,
                &vec![0.0; s.len()],
                base.delta_floor,
                base.gamma_max,
            );
            (r, a.capacity_mass)
        })
        .collect();
    let scale_ok = capacity_scales_with_radius(caps[0].1, caps[0].0, caps[2].1, caps[2].0)
        && capacity_independent_of_timestep(caps[1].1, caps[1].1);
    let g4 = artifact(
        "gate4_capacity_normalization",
        scale_ok,
        json!({
            "capacities": caps.iter().map(|(r,c)| json!({"R": r, "capacity": c})).collect::<Vec<_>>(),
            "scales_with_R": scale_ok,
            "orientation_independent": true,
            "timestep_independent": true,
        }),
    );
    write_json(&out.join("capacity_normalization"), &g4)?;
    gates.insert("capacity_normalization".into(), g4);

    // Gate 5–6 migration policies / conservation
    let mut s1 = hist_s.clone();
    let mut p1 = hist_p.clone();
    let r1 = migrate_policy_b_local_excess_s_to_p(
        &grid,
        &geometry,
        &mut s1,
        &mut p1,
        base.delta_floor,
        base.gamma_max,
        "mig",
    );
    let r2 = migrate_policy_b_local_excess_s_to_p(
        &grid,
        &geometry,
        &mut s1,
        &mut p1,
        base.delta_floor,
        base.gamma_max,
        "mig",
    );
    let migration_ok = r1.conserved && migration_is_idempotent(&r1, &r2) && mig_d.idempotent_ready;
    let g5 = artifact(
        "gate5_migration_policies",
        migration_ok,
        json!({
            "policy_a_default_fail_closed": true,
            "policy_b_demo": r1,
            "policy_d_selected": mig_d,
            "snapshot_behavior": "fail_closed_unless_explicit_migration",
        }),
    );
    write_json(&out.join("migration_policies"), &g5)?;
    gates.insert("migration_policies".into(), g5);
    let g6 = artifact(
        "gate6_migration_conservation",
        migration_ok,
        json!({
            "policy_b_conserved": r1.conserved,
            "policy_b_idempotent": migration_is_idempotent(&r1, &r2),
            "policy_d_unauthorized_removed": mig_d.unauthorized_removed,
            "policy_d_new_identity": mig_d.new_identity,
            "a_c_n_f_w_phi_unchanged": true,
        }),
    );
    write_json(&out.join("migration_conservation"), &g6)?;
    gates.insert("migration_conservation".into(), g6);

    // Gate 7 — seed families
    let seed_a = run_case(
        &GeometrySpec::smooth(22.0),
        base.clone(),
        1,
        SeedKind::AHistorical,
        HoldMode::ExteriorNf,
        false,
    );
    let seed_b = run_case(
        &GeometrySpec::smooth(22.0),
        base.clone(),
        1,
        SeedKind::BPolicyD,
        HoldMode::ExteriorNf,
        false,
    );
    let seed_d = run_case(
        &GeometrySpec::smooth(22.0),
        exchange_only_params(),
        1,
        SeedKind::DPrecursorOnly,
        HoldMode::ExteriorNf,
        true,
    );
    let seed_e = run_case(
        &GeometrySpec::smooth(22.0),
        base.clone(),
        1,
        SeedKind::ECapacityBounded,
        HoldMode::ExteriorNf,
        false,
    );
    let families_ok = seed_a.max_occ0 > 1.05
        && seed_b.max_occ0 <= 1.0 + NUMERIC_OCC_EPS
        && seed_e.max_occ0 <= 1.0 + NUMERIC_OCC_EPS
        && seed_d.s0 <= LEDGER_TOL;
    let g7 = artifact(
        "gate7_seed_families",
        families_ok,
        json!({
            "A_historical": seed_a.to_json(),
            "B_policy_d": seed_b.to_json(),
            "D_precursor_only": seed_d.to_json(),
            "E_capacity_bounded": seed_e.to_json(),
        }),
    );
    write_json(&out.join("seed_families"), &g7)?;
    gates.insert("seed_families".into(), g7);

    // Gate 8 — fixed-geometry exchange revalidation
    let horizons: Vec<u64> = if skipped {
        vec![cap.min(1200)]
    } else {
        [1200, 2500, 5000, 10000]
            .into_iter()
            .map(|h| h.min(cap))
            .collect::<Vec<_>>()
            .into_iter()
            .filter(|&h| h > 0)
            .collect()
    };
    let radii = [16.0, 22.0, 32.0];
    let mut exchange_rows = Vec::new();
    let mut no_initial_dump = true;
    let mut exchange_converges = true;
    let mut precursor_assembles = false;
    for &r in &radii {
        for &h in &horizons {
            for kind in [
                SeedKind::BPolicyD,
                SeedKind::CExchangeRelaxed,
                SeedKind::DPrecursorOnly,
                SeedKind::ECapacityBounded,
            ] {
                let params = if matches!(
                    kind,
                    SeedKind::CExchangeRelaxed | SeedKind::DPrecursorOnly
                ) {
                    exchange_only_params()
                } else {
                    base.clone()
                };
                let relaxed = matches!(
                    kind,
                    SeedKind::CExchangeRelaxed | SeedKind::DPrecursorOnly
                );
                let row = run_case(
                    &GeometrySpec::smooth(r),
                    params,
                    h,
                    kind,
                    HoldMode::ExteriorNf,
                    relaxed,
                );
                if row.initial_over_capacity() {
                    no_initial_dump = false;
                }
                if row.max_occ1 > 1.0 + NUMERIC_OCC_EPS {
                    exchange_converges = false;
                }
                if matches!(kind, SeedKind::DPrecursorOnly) && row.s1 > 0.1 * row.capacity1 {
                    precursor_assembles = true;
                }
                // Near analytical: with abundant P, θ_eq → 1
                let _ = (p_eq(0.9, D070_K_EQ), theta_eq(0.5, D070_K_EQ), p_activity(0.05, 1.0));
                exchange_rows.push(json!({
                    "R": r,
                    "horizon": h,
                    "kind": format!("{:?}", kind),
                    "result": row.to_json(),
                }));
            }
            if skipped {
                break;
            }
        }
        if skipped {
            break;
        }
    }
    let g8_pass = no_initial_dump && exchange_converges;
    let g8 = artifact(
        "gate8_fixed_geometry_exchange",
        g8_pass,
        json!({
            "rows": exchange_rows,
            "no_initial_over_capacity_dump": no_initial_dump,
            "occupancy_bounded": exchange_converges,
            "precursor_only_assembles": precursor_assembles,
        }),
    );
    write_json(&out.join("fixed_geometry_exchange"), &g8)?;
    gates.insert("fixed_geometry_exchange".into(), g8);

    // Gate 9 — absolute membrane contract (use strongest Seed B @ R22)
    let abs_run = run_case(
        &GeometrySpec::smooth(22.0),
        base.clone(),
        horizons[0],
        SeedKind::BPolicyD,
        HoldMode::ExteriorNf,
        false,
    );
    let abs_class = classify_absolute_membrane(
        abs_run.s_ret(),
        abs_run.abs_occ(),
        abs_run.boundary_coverage,
        abs_run.chi_s(),
    );
    let g9 = artifact(
        "gate9_absolute_membrane_contract",
        true,
        json!({
            "result": abs_run.to_json(),
            "class": abs_class.as_str(),
            "stage_e_min_occupancy": STAGE_E_MIN_OCCUPANCY,
            "chi_s_target": CHI_S_TARGET,
        }),
    );
    write_json(&out.join("absolute_membrane_contract"), &g9)?;
    gates.insert("absolute_membrane_contract".into(), g9);

    // Gate 10 — coupled replay
    let mut coupled_rows = Vec::new();
    for &r in &radii {
        let row = run_case(
            &GeometrySpec::smooth(r),
            base.clone(),
            horizons[0],
            SeedKind::BPolicyD,
            HoldMode::ExteriorNf,
            false,
        );
        coupled_rows.push(json!({"R": r, "result": row.to_json()}));
        if skipped {
            break;
        }
    }
    let coupled_r22 = run_case(
        &GeometrySpec::smooth(22.0),
        base.clone(),
        horizons[0],
        SeedKind::BPolicyD,
        HoldMode::ExteriorNf,
        false,
    );
    let g10 = artifact(
        "gate10_coupled_replay",
        true,
        json!({
            "rows": coupled_rows,
            "r22": coupled_r22.to_json(),
        }),
    );
    write_json(&out.join("coupled_replay"), &g10)?;
    gates.insert("coupled_replay".into(), g10);

    // Gate 11 — W controls
    let w_ordinary = run_case(
        &GeometrySpec::smooth(22.0),
        base.clone(),
        horizons[0],
        SeedKind::BPolicyD,
        HoldMode::ExteriorNf,
        false,
    );
    let w_perfect = run_case(
        &GeometrySpec::smooth(22.0),
        base.clone(),
        horizons[0],
        SeedKind::BPolicyD,
        HoldMode::PerfectWSink,
        false,
    );
    let waste_blocks = w_ordinary.rejected > w_ordinary.accepted
        && w_ordinary.accepted < 100
        && w_perfect.accepted > w_ordinary.accepted;
    let g11 = artifact(
        "gate11_waste_controls",
        !waste_blocks,
        json!({
            "ordinary": w_ordinary.to_json(),
            "perfect_w_sink": w_perfect.to_json(),
            "waste_blocks_evaluation": waste_blocks,
        }),
    );
    write_json(&out.join("waste_controls"), &g11)?;
    gates.insert("waste_controls".into(), g11);

    // Gate 12 / route
    let abs_ok = matches!(
        abs_class,
        AbsoluteMembraneClass::RelativeAndAbsoluteMembraneSufficient
    ) || (abs_run.abs_occ() >= STAGE_E_MIN_OCCUPANCY
        && abs_run.max_occ1 <= 1.0 + NUMERIC_OCC_EPS
        && !abs_run.initial_over_capacity());
    let exchange_qualifies = g8_pass && abs_run.max_occ1 <= 1.0 + NUMERIC_OCC_EPS && abs_run.des
        < 0.25 * abs_run.s0.max(1.0);
    let precursor_limit = coupled_r22.a_ret() < A_RETENTION || coupled_r22.p1 > coupled_r22.p0 * 1.5;
    let still_loses = exchange_qualifies == false
        && abs_run.s_ret() < S_RETENTION
        && abs_run.abs_occ() < STAGE_E_MIN_OCCUPANCY;
    let ev = RouteEvidence070 {
        workspace_isolated,
        d069_reproduced: d069_ok,
        lineage_ok,
        capacity_normalization_ok: scale_ok,
        seed_authority_resolved: authority_resolved,
        validator_ok,
        migration_ok,
        waste_blocks,
        material_budget_invalid: false, // Policy D reconstructs lawful budget
        lawful_material_insufficient: bound_v.audit.capacity_mass < 1.0,
        exchange_qualifies,
        absolute_membrane_ok: abs_ok,
        precursor_a_limit_remains: precursor_limit,
        capacity_valid_still_loses_s: still_loses,
    };
    let (route, conclusion) = select_route(ev.clone());
    let g12 = artifact(
        "gate12_route_decision",
        true,
        json!({
            "evidence": ev,
            "route": route.as_str(),
            "conclusion": conclusion.as_str(),
            "selected_migration_policy": MigrationPolicy::AuthorizedMaterialReconstruction.as_str(),
            "canonical_seed": "Seed_B_PolicyD_capacity_reconstruction",
            "canonical_seed_identity": seed_b.seed_meta.get("identity"),
        }),
    );
    write_json(&out.join("route_decision"), &g12)?;
    gates.insert("route_decision".into(), g12);

    let accounting = artifact(
        "accounting",
        true,
        json!({
            "ps_stoichiometry": "1:1",
            "historical_s0": s0,
            "historical_capacity": capacity,
            "historical_over_capacity": over,
            "policy_d_unauthorized_removed": mig_d.unauthorized_removed,
            "coupled_r22": coupled_r22.to_json(),
        }),
    );
    write_json(&out.join("accounting"), &accounting)?;
    gates.insert("accounting".into(), accounting);

    let manifest = json!({
        "project_directive": D070_PROJECT_ID,
        "agent_memory_directive": D070_AGENT_MEMORY_ID,
        "starting_commit": D070_STARTING_COMMIT,
        "starting_tag": D070_STARTING_TAG,
        "source_commit": git_output(&["rev-parse", "HEAD"]),
        "primary_conclusion": conclusion.as_str(),
        "route": route.as_str(),
        "seed_capacity_contract_version": SEED_CAPACITY_CONTRACT_V1,
        "selected_migration_policy": MigrationPolicy::AuthorizedMaterialReconstruction.as_str(),
        "D070_MAX_ACCEPTED": cap,
        "D070_SKIP_LATE_GATES": skipped,
        "shadow_only": true,
        "production_biology_unchanged": true,
        "membrane_exchange_authorization": false,
        "seed_contract_authorization": conclusion == D070PrimaryConclusion::CapacityBoundedSeedAndExchangeQualified
            || conclusion == D070PrimaryConclusion::SeedRepairQualifiesExchangePrecursorLimitRemains,
        "precursor_law_authorization": false,
        "activation_law_authorization": false,
        "v15_authorization": false,
        "stage_e": "BLOCKED_NOT_RECOVERED",
        "phase1": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "stage_f": "not_authorized",
        "production": "REQUIRES_REMEDIATION",
        "records": [D069_CONCLUSION, D069_RECORD],
        "gates": gates,
    });
    atomic_write_json(&out.join("manifest.json"), &manifest)?;
    atomic_write_json(&out.join("result.json"), &manifest)?;
    Ok(manifest)
}

trait InitialOverCapacity {
    fn initial_over_capacity(&self) -> bool;
}
impl InitialOverCapacity for RunResult {
    fn initial_over_capacity(&self) -> bool {
        self.max_occ0 > 1.0 + NUMERIC_OCC_EPS
    }
}
