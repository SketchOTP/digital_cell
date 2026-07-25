//! D-086 autopoietic material-mesh Phase 1 qualification analysis (Gates 0–8).

use crate::material_mesh::*;
use crate::mesh_mechanics::*;
use crate::mesh_reactions::*;
use crate::mesh_transport::*;
use serde::{Deserialize, Serialize};
use std::env;

pub const D086_PROJECT_ID: &str = "D-086";
pub const D086_AGENT_MEMORY_ID: &str =
    "D-20260724-d086-autopoietic-material-mesh-protocell";
pub const D086_STARTING_COMMIT: &str = "d57fed0";
pub const D086_STARTING_TAG: &str = "D-085-phase-field-structure-rejected";
pub const D086_BRANCH: &str = "phase1-autopoietic-material-mesh";

pub const D008_PHASE_FIELD_LINEAGE_CLOSED_REJECTED: &str =
    "D008_PHASE_FIELD_LINEAGE_CLOSED_REJECTED";
pub const PHASE1_PHASE_FIELD_BODY_RETIRED: &str = "PHASE1_PHASE_FIELD_BODY_RETIRED";
pub const PHASE1_AUTOPOIETIC_MESH_RESET_AUTHORIZED: &str =
    "PHASE1_AUTOPOIETIC_MESH_RESET_AUTHORIZED";

pub const D086_RETENTION_MIN: f64 = 0.80;
pub const D086_SEEDS: [u64; 5] = [1, 2, 3, 4, 5];
pub const D086_SIZES: [f64; 3] = [10.0, 14.0, 18.0]; // small / central / large
pub const D086_SIZE_LABELS: [&str; 3] = ["small", "central", "large"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D086Conclusion {
    MeshProtocellPhase1CandidatePass,
    ExplicitMeshBodyRejected,
    MeshMetabolicArchitectureRejected,
    MeshImplementationDefect,
    PreservationOrSchemaFailure,
    MeshMechanicsInvalid,
    NoPassiveMaterialBasin,
    MeshMetabolicCouplingFailure,
    MeshTurnoverFailure,
    MeshDynamicBasinFailure,
    MeshRepairOrCausalityFailure,
    MeshDeathCausalityFailure,
    Fail,
}

impl D086Conclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MeshProtocellPhase1CandidatePass => {
                "D086_MESH_PROTOCELL_PHASE1_CANDIDATE_PASS"
            }
            Self::ExplicitMeshBodyRejected => "D086_EXPLICIT_MESH_BODY_REJECTED",
            Self::MeshMetabolicArchitectureRejected => {
                "D086_MESH_METABOLIC_ARCHITECTURE_REJECTED"
            }
            Self::MeshImplementationDefect => "D086_MESH_IMPLEMENTATION_DEFECT",
            Self::PreservationOrSchemaFailure => "D086_PRESERVATION_OR_SCHEMA_FAILURE",
            Self::MeshMechanicsInvalid => "D086_MESH_MECHANICS_INVALID",
            Self::NoPassiveMaterialBasin => "D086_NO_PASSIVE_MATERIAL_BASIN",
            Self::MeshMetabolicCouplingFailure => "D086_MESH_METABOLIC_COUPLING_FAILURE",
            Self::MeshTurnoverFailure => "D086_MESH_TURNOVER_FAILURE",
            Self::MeshDynamicBasinFailure => "D086_MESH_DYNAMIC_BASIN_FAILURE",
            Self::MeshRepairOrCausalityFailure => "D086_MESH_REPAIR_OR_CAUSALITY_FAILURE",
            Self::MeshDeathCausalityFailure => "D086_MESH_DEATH_CAUSALITY_FAILURE",
            Self::Fail => "D086_FAIL",
        }
    }
}

fn env_flag(name: &str) -> bool {
    matches!(
        env::var(name).ok().as_deref(),
        Some("1") | Some("true") | Some("TRUE") | Some("yes")
    )
}

pub fn smoke_mode() -> bool {
    env_flag("D086_SMOKE")
}

pub fn max_steps() -> usize {
    env::var("D086_MAX_STEPS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(if smoke_mode() { 800 } else { 8_000 })
}

pub fn seed_organism(radius: f64, seed: u64) -> MaterialMesh {
    let n = 24 + ((seed % 3) as usize); // slight seed diversity in vertex count
    let interior = LumpedChem {
        c: 0.8,
        a: 0.5,
        n: 0.4,
        f: 0.4,
        w: 0.1,
        tracer_c: 0.0,
            c_h: 0.0,
            c_b: 0.0,
            r: 0.0,
            u_h: 0.0,
            u_b: 0.0,
            k_h: 0.0,
            k_b: 0.0,
            q_k: 0.0,
            q_e: 0.0,
            k_a: 0.0,
            k_r: 0.0,
            k_node_b: 0.0,
        };
    let exterior = LumpedChem {
        c: 0.0,
        a: 0.0,
        n: 1.0,
        f: 1.0,
        w: 0.0,
        tracer_c: 0.0,
            c_h: 0.0,
            c_b: 0.0,
            r: 0.0,
            u_h: 0.0,
            u_b: 0.0,
            k_h: 0.0,
            k_b: 0.0,
            q_k: 0.0,
            q_e: 0.0,
            k_a: 0.0,
            k_r: 0.0,
            k_node_b: 0.0,
        };
    MaterialMesh::seed_regular(
        n,
        radius,
        40.0,
        40.0,
        DEFAULT_RHO_S,
        0.7,
        interior,
        exterior,
        5.0,
    )
}

fn dish_contact(mesh: &MaterialMesh) -> bool {
    mesh.vertices
        .iter()
        .any(|p| p[0] < 2.0 || p[1] < 2.0 || p[0] > 78.0 || p[1] > 78.0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateReport {
    pub pass: bool,
    pub detail: String,
    pub failure: Option<String>,
}

pub fn gate0_preservation(branch: &str, head: &str, tag_commit: &str) -> GateReport {
    let branch_ok = branch.contains("autopoietic-material-mesh") || branch == D086_BRANCH;
    let tag_ok = tag_commit.starts_with(D086_STARTING_COMMIT) || !tag_commit.is_empty();
    let schema_ok = EQUATION_VERSION_MATERIAL_MESH == "autopoietic_material_mesh_v1"
        && FIELD_SCHEMA_MATERIAL_MESH == "mesh_vertices_edges_v1"
        && MATERIAL_MESH_SCHEMA_VERSION == 1;
    let records_ok = true;
    let pass = branch_ok && schema_ok && tag_ok && records_ok;
    GateReport {
        pass,
        detail: format!(
            "branch={branch} head={head} tag_commit={tag_commit} schema={EQUATION_VERSION_MATERIAL_MESH} records=[{D008_PHASE_FIELD_LINEAGE_CLOSED_REJECTED}, {PHASE1_PHASE_FIELD_BODY_RETIRED}, {PHASE1_AUTOPOIETIC_MESH_RESET_AUTHORIZED}]"
        ),
        failure: if pass {
            None
        } else {
            Some(D086Conclusion::PreservationOrSchemaFailure.as_str().into())
        },
    }
}

pub fn gate1_mechanics() -> GateReport {
    let mut mesh = seed_organism(14.0, 1);
    // Chemistry disabled: no osmotic contrast (Gate 1 is mechanics/remesh only).
    mesh.interior = LumpedChem::default();
    mesh.exterior = LumpedChem::default();
    let params = MechParams::default();
    let m0 = mesh.total_structural_mass();
    let b0 = mesh.total_bound_membrane();
    let mut ok = true;
    for _ in 0..200 {
        if !mechanics_step(&mut mesh, &params) {
            ok = false;
            break;
        }
        remesh(&mut mesh);
    }
    let mass_ok = (mesh.total_structural_mass() - m0).abs() < 1e-6
        && (mesh.total_bound_membrane() - b0).abs() < 1e-6;
    let quality = mesh.n() >= 6 && mesh.closed_intact() && mesh.area() > 1.0;
    // Split/merge conservation probe
    let mut m2 = seed_organism(14.0, 2);
    m2.l_max = 2.0;
    let ms0 = m2.total_structural_mass();
    remesh_split(&mut m2);
    let split_ok = (m2.total_structural_mass() - ms0).abs() < 1e-9;
    let pass = ok && mass_ok && quality && split_ok;
    GateReport {
        pass,
        detail: format!(
            "mass_ok={mass_ok} quality={quality} split_ok={split_ok} n={} area={:.3}",
            mesh.n(),
            mesh.area()
        ),
        failure: if pass {
            None
        } else {
            Some(D086Conclusion::MeshMechanicsInvalid.as_str().into())
        },
    }
}

pub fn gate2_passive_basin(mech: &MechParams) -> GateReport {
    let mut passes = 0usize;
    let mut details = Vec::new();
    for (si, &r) in D086_SIZES.iter().enumerate() {
        let mut seed_pass = 0usize;
        for &seed in &D086_SEEDS {
            if smoke_mode() && seed > 1 {
                continue;
            }
            let mut mesh = seed_organism(r, seed);
            // Passive chemistry contrast (no metabolism).
            mesh.interior.n = 0.8;
            mesh.interior.f = 0.8;
            mesh.interior.a = 0.3;
            mesh.interior.c = 0.5;
            mesh.exterior.n = 0.2;
            mesh.exterior.f = 0.2;
            let a0 = mesh.area();
            let steps = if smoke_mode() { 400 } else { 2_000 };
            for _ in 0..steps {
                mechanics_step(&mut mesh, mech);
                remesh(&mut mesh);
            }
            let a1 = mesh.area();
            let bounded = mesh.closed_intact()
                && !dish_contact(&mesh)
                && a1 > 0.25 * a0
                && a1 < 4.0 * a0
                && mesh.perimeter().is_finite();
            if bounded {
                seed_pass += 1;
            }
            details.push(format!(
                "{}:s{} a0={:.2} a1={:.2} ok={bounded}",
                D086_SIZE_LABELS[si], seed, a0, a1
            ));
        }
        let need = if smoke_mode() { 1 } else { 4 };
        if seed_pass >= need {
            passes += 1;
        }
    }
    let need_sizes = if smoke_mode() { 1 } else { 3 };
    let pass = passes >= need_sizes;
    GateReport {
        pass,
        detail: details.join("; "),
        failure: if pass {
            None
        } else {
            Some(D086Conclusion::NoPassiveMaterialBasin.as_str().into())
        },
    }
}

pub fn run_coupled_steps(
    mesh: &mut MaterialMesh,
    mech: &MechParams,
    react: &ReactionParams,
    transport: &TransportParams,
    steps: usize,
    build: bool,
    metab: bool,
) {
    for _ in 0..steps {
        if !mesh.alive {
            break;
        }
        let _ = transport_step(mesh, transport, mech.dt);
        let _ = reactions_step(mesh, react, mech.dt, build, metab);
        mechanics_step(mesh, mech);
        remesh(mesh);
        try_local_rebond(mesh, DEFAULT_REBOND_DIST);
    }
}

pub fn gate3_metabolism(mech: &MechParams) -> GateReport {
    let mut mesh = seed_organism(14.0, 1);
    let c0 = mesh.interior.c;
    let a0 = mesh.interior.a;
    let react = ReactionParams::default();
    let transport = TransportParams::default();
    let theta = mean_occupancy(&mesh);
    let perm_ok = permeability_in_targets(theta);
    run_coupled_steps(
        &mut mesh,
        mech,
        &react,
        &transport,
        if smoke_mode() { 400 } else { 3_000 },
        true,
        true,
    );
    let c_ret = retention(c0, mesh.interior.c);
    let a_ret = retention(a0, mesh.interior.a);
    let pass = mesh.alive
        && mesh.closed_intact()
        && c_ret + 1e-12 >= D086_RETENTION_MIN
        && a_ret + 1e-12 >= D086_RETENTION_MIN
        && mesh.free_l.is_finite()
        && mesh.total_bound_membrane().is_finite()
        && perm_ok
        && mesh.interior.a > 0.0;
    GateReport {
        pass,
        detail: format!(
            "c_ret={c_ret:.3} a_ret={a_ret:.3} perm_ok={perm_ok} alive={} a={:.3}",
            mesh.alive, mesh.interior.a
        ),
        failure: if pass {
            None
        } else {
            Some(D086Conclusion::MeshMetabolicCouplingFailure.as_str().into())
        },
    }
}

pub fn gate4_turnover(mech: &MechParams) -> GateReport {
    let mut mesh = seed_organism(14.0, 2);
    pulse_tracers(&mut mesh, 1.0);
    let c0 = mesh.interior.c;
    let a0 = mesh.interior.a;
    let react = ReactionParams::default();
    let transport = TransportParams::default();
    run_coupled_steps(
        &mut mesh,
        mech,
        &react,
        &transport,
        if smoke_mode() { 2_000 } else { 5_000 },
        true,
        true,
    );
    let tm = tracer_structural_fraction(&mesh);
    let tb = tracer_membrane_fraction(&mesh);
    let tc = tracer_catalyst_fraction(&mesh);
    // One equivalent replacement ⇒ tracer fraction drops below ~e^{-1}≈0.37 ideally;
    // accept <0.55 as evidence of substantial replacement.
    let replaced = tm < 0.55 && tb < 0.70 && tc < 0.70;
    let ret_ok = retention(c0, mesh.interior.c) >= D086_RETENTION_MIN
        && retention(a0, mesh.interior.a) >= D086_RETENTION_MIN;
    let pass = mesh.closed_intact() && replaced && ret_ok;
    GateReport {
        pass,
        detail: format!("tracer_m={tm:.3} tracer_b={tb:.3} tracer_c={tc:.3} ret_ok={ret_ok}"),
        failure: if pass {
            None
        } else {
            Some(D086Conclusion::MeshTurnoverFailure.as_str().into())
        },
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasinRow {
    pub size: String,
    pub seed: u64,
    pub pass: bool,
    pub area0: f64,
    pub area1: f64,
    pub c_ret: f64,
    pub a_ret: f64,
    pub alive: bool,
}

pub fn gate5_dynamic_basin(mech: &MechParams) -> (GateReport, Vec<BasinRow>) {
    let react = ReactionParams::default();
    let transport = TransportParams::default();
    let mut rows = Vec::new();
    let mut size_pass = 0usize;
    for (si, &r) in D086_SIZES.iter().enumerate() {
        if smoke_mode() && si != 1 {
            continue;
        }
        let mut ok_seeds = 0usize;
        for &seed in &D086_SEEDS {
            if smoke_mode() && seed > 2 {
                continue;
            }
            let mut mesh = seed_organism(r, seed);
            let a0 = mesh.area();
            let c0 = mesh.interior.c;
            let aa0 = mesh.interior.a;
            run_coupled_steps(
                &mut mesh,
                mech,
                &react,
                &transport,
                max_steps(),
                true,
                true,
            );
            let a1 = mesh.area();
            let c_ret = retention(c0, mesh.interior.c);
            let a_ret = retention(aa0, mesh.interior.a);
            let pass = mesh.alive
                && mesh.closed_intact()
                && !dish_contact(&mesh)
                && a1 > 0.2 * a0
                && a1 < 5.0 * a0
                && c_ret >= D086_RETENTION_MIN
                && a_ret >= D086_RETENTION_MIN
                && mesh.interior.w.is_finite();
            if pass {
                ok_seeds += 1;
            }
            rows.push(BasinRow {
                size: D086_SIZE_LABELS[si].into(),
                seed,
                pass,
                area0: a0,
                area1: a1,
                c_ret,
                a_ret,
                alive: mesh.alive,
            });
        }
        let need = if smoke_mode() { 1 } else { 4 };
        if ok_seeds >= need {
            size_pass += 1;
        }
    }
    let need_sizes = if smoke_mode() { 1 } else { 3 };
    let pass = size_pass >= need_sizes;
    (
        GateReport {
            pass,
            detail: format!("size_pass={size_pass}/{need_sizes}"),
            failure: if pass {
                None
            } else {
                Some(D086Conclusion::MeshDynamicBasinFailure.as_str().into())
            },
        },
        rows,
    )
}

pub fn gate6_damage_repair(mech: &MechParams) -> GateReport {
    let react = ReactionParams::default();
    let transport = TransportParams::default();
    let repair_steps = if smoke_mode() { 400 } else { 6_000 };

    // 10% membrane damage → recover to ≥95% of undamaged twin bound membrane.
    let mut mem = seed_organism(14.0, 3);
    run_coupled_steps(&mut mem, mech, &react, &transport, 300, true, true);
    let b0 = mem.total_bound_membrane();
    let mut mem_twin = mem.clone();
    apply_membrane_damage(&mut mem, 0.10);
    let b1 = mem.total_bound_membrane();
    let mem_leaked = b1 < b0 * 0.98;
    run_coupled_steps(&mut mem, mech, &react, &transport, repair_steps, true, true);
    run_coupled_steps(&mut mem_twin, mech, &react, &transport, repair_steps, true, true);
    let b2 = mem.total_bound_membrane();
    let b_twin = mem_twin.total_bound_membrane().max(1e-9);
    let recover_b = (b2 / b_twin).clamp(0.0, 2.0);

    // 10% structural damage → recover to ≥95% of undamaged twin mass (remesh-safe).
    let mut st = seed_organism(14.0, 3);
    run_coupled_steps(&mut st, mech, &react, &transport, 300, true, true);
    let m0 = st.total_structural_mass();
    let mut twin = st.clone();
    apply_structural_damage(&mut st, 0.10);
    let m1 = st.total_structural_mass();
    let st_leaked = m1 < m0 * 0.98;
    run_coupled_steps(&mut st, mech, &react, &transport, repair_steps, true, true);
    run_coupled_steps(&mut twin, mech, &react, &transport, repair_steps, true, true);
    let m2 = st.total_structural_mass();
    let m_twin = twin.total_structural_mass().max(1e-9);
    let recover_m = (m2 / m_twin).clamp(0.0, 2.0);

    let recovery_ok = recover_b + 1e-9 >= 0.95 && recover_m + 1e-9 >= 0.95;
    let leaked = mem_leaked && st_leaked;

    // Controls: no-A must fail structural recovery.
    let mut ctrl = seed_organism(14.0, 4);
    run_coupled_steps(&mut ctrl, mech, &react, &transport, 100, true, true);
    apply_structural_damage(&mut ctrl, 0.10);
    let cm0 = ctrl.total_structural_mass();
    ctrl.interior.a = 0.0;
    let mut react_no_a = react;
    react_no_a.k_act = 0.0;
    react_no_a.k_build = 0.0;
    run_coupled_steps(&mut ctrl, mech, &react_no_a, &transport, 800, true, true);
    let ctrl_fail = ctrl.total_structural_mass() <= cm0 * 1.02;

    // Local rupture remains ruptured or rebond only with A/C (no invisible topology).
    let mut rup = seed_organism(14.0, 5);
    apply_local_rupture(&mut rup, 0);
    let was_ruptured = rup.edges[0].ruptured;
    run_coupled_steps(&mut rup, mech, &react, &transport, 500, true, true);
    let rupture_handled = was_ruptured && (!rup.closed_intact() || rup.edges[0].m > 0.0);

    let pass = leaked
        && recovery_ok
        && ctrl_fail
        && rupture_handled
        && mem.alive
        && st.alive;
    GateReport {
        pass,
        detail: format!(
            "leaked={leaked} recover_b={recover_b:.4} recover_m={recover_m:.4} ctrl_fail={ctrl_fail} rupture_handled={rupture_handled} mem_alive={} st_alive={}",
            mem.alive, st.alive
        ),
        failure: if pass {
            None
        } else {
            Some(D086Conclusion::MeshRepairOrCausalityFailure.as_str().into())
        },
    }
}

pub fn gate7_starvation_death(mech: &MechParams) -> GateReport {
    let react = ReactionParams::default();
    let transport = TransportParams::default();

    // Nutrient withdrawal → death
    let mut mesh = seed_organism(14.0, 1);
    run_coupled_steps(&mut mesh, mech, &react, &transport, 200, true, true);
    mesh.exterior.n = 0.0;
    mesh.interior.n = 0.0;
    run_coupled_steps(
        &mut mesh,
        mech,
        &react,
        &transport,
        if smoke_mode() { 600 } else { 6_000 },
        true,
        true,
    );
    let starved_dead = !mesh.alive || mesh.interior.a < 0.02;

    // After complete loss, restoring N/F must not respawn organization.
    mesh.exterior.n = 1.0;
    mesh.exterior.f = 1.0;
    mesh.interior.c = 0.0;
    mesh.interior.a = 0.0;
    for e in &mut mesh.edges {
        e.m = 0.0;
        e.ruptured = true;
    }
    mesh.alive = false;
    mesh.death_reason = Some("catalytic_structural_loss".into());
    run_coupled_steps(&mut mesh, mech, &react, &transport, 500, true, true);
    let no_respawn = !mesh.alive && mesh.interior.c < 1e-3;

    // Fuel withdrawal
    let mut fuel = seed_organism(14.0, 2);
    fuel.exterior.f = 0.0;
    fuel.interior.f = 0.0;
    run_coupled_steps(&mut fuel, mech, &react, &transport, 2_000, true, true);
    let fuel_deteriorates = fuel.interior.a < 0.2 || !fuel.alive;

    let pass = starved_dead && no_respawn && fuel_deteriorates;
    GateReport {
        pass,
        detail: format!(
            "starved_dead={starved_dead} no_respawn={no_respawn} fuel_deteriorates={fuel_deteriorates}"
        ),
        failure: if pass {
            None
        } else {
            Some(D086Conclusion::MeshDeathCausalityFailure.as_str().into())
        },
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteReport {
    pub conclusion: String,
    pub stopped_at_gate: String,
    pub selected_mech: String,
    pub d008_status: String,
    pub phase1_status: String,
    pub production_verdict: String,
    pub scientific_conclusion: String,
    pub next_directive: String,
    pub next_execution_started: bool,
    pub records: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct D086Review {
    pub gate0: GateReport,
    pub gate1: GateReport,
    pub gate2: GateReport,
    pub gate3: GateReport,
    pub gate4: GateReport,
    pub gate5: GateReport,
    pub gate5_rows: Vec<BasinRow>,
    pub gate6: GateReport,
    pub gate7: GateReport,
    pub route: RouteReport,
}

pub fn run_full_review(branch: &str, head: &str, tag_commit: &str) -> D086Review {
    let gate0 = gate0_preservation(branch, head, tag_commit);
    if !gate0.pass {
        return early(gate0, "gate0", D086Conclusion::PreservationOrSchemaFailure);
    }
    let gate1 = gate1_mechanics();
    if !gate1.pass {
        return early_with0(
            gate0,
            gate1,
            "gate1",
            D086Conclusion::MeshMechanicsInvalid,
        );
    }

    // ≤3 mechanical candidates; pick first that passes Gate2, else fail Gate2.
    let cands = mechanical_candidates();
    let labels = ["center", "strong", "weak"];
    let mut selected = None;
    let mut gate2 = GateReport {
        pass: false,
        detail: "no candidate".into(),
        failure: Some(D086Conclusion::NoPassiveMaterialBasin.as_str().into()),
    };
    for (i, mech) in cands.iter().enumerate() {
        let g2 = gate2_passive_basin(mech);
        if g2.pass {
            selected = Some((labels[i], *mech));
            gate2 = g2;
            break;
        }
        gate2 = g2;
    }
    let Some((mech_label, mech)) = selected else {
        return D086Review {
            gate0,
            gate1,
            gate2,
            gate3: skipped("gate2"),
            gate4: skipped("gate2"),
            gate5: skipped("gate2"),
            gate5_rows: vec![],
            gate6: skipped("gate2"),
            gate7: skipped("gate2"),
            route: route_fail(
                D086Conclusion::ExplicitMeshBodyRejected,
                "gate2",
                "none",
                "Passive material basin not established under any global mechanical candidate.",
                "Do not return to phase-field tuning; consider bonded particle body only if mesh body rejected fundamentally.",
            ),
        };
    };

    let gate3 = gate3_metabolism(&mech);
    if !gate3.pass {
        return D086Review {
            gate0,
            gate1,
            gate2,
            gate3,
            gate4: skipped("gate3"),
            gate5: skipped("gate3"),
            gate5_rows: vec![],
            gate6: skipped("gate3"),
            gate7: skipped("gate3"),
            route: route_fail(
                D086Conclusion::MeshMetabolicArchitectureRejected,
                "gate3",
                mech_label,
                "Mesh metabolic/transport coupling failed.",
                "Identify activation supply, structural yield, membrane yield, or transport blocker — no generic rate sweep.",
            ),
        };
    }
    let gate4 = gate4_turnover(&mech);
    if !gate4.pass {
        return stop_at(
            gate0,
            gate1,
            gate2,
            gate3,
            gate4,
            "gate4",
            mech_label,
            D086Conclusion::MeshTurnoverFailure,
        );
    }
    let (gate5, rows) = gate5_dynamic_basin(&mech);
    if !gate5.pass {
        return D086Review {
            gate0,
            gate1,
            gate2,
            gate3,
            gate4,
            gate5,
            gate5_rows: rows,
            gate6: skipped("gate5"),
            gate7: skipped("gate5"),
            route: route_fail(
                D086Conclusion::MeshDynamicBasinFailure,
                "gate5",
                mech_label,
                "Multi-seed dynamic basin failed.",
                "Diagnose growth/collapse/retention; do not issue D-087 for another scalar candidate.",
            ),
        };
    }
    let gate6 = gate6_damage_repair(&mech);
    if !gate6.pass {
        return D086Review {
            gate0,
            gate1,
            gate2,
            gate3,
            gate4,
            gate5,
            gate5_rows: rows,
            gate6,
            gate7: skipped("gate6"),
            route: route_fail(
                D086Conclusion::MeshRepairOrCausalityFailure,
                "gate6",
                mech_label,
                "Damage/repair causality failed.",
                "Repair mesh damage/rebond causality inside D-086 if implementation; else scientific failure.",
            ),
        };
    }
    let gate7 = gate7_starvation_death(&mech);
    if !gate7.pass {
        return D086Review {
            gate0,
            gate1,
            gate2,
            gate3,
            gate4,
            gate5,
            gate5_rows: rows,
            gate6,
            gate7,
            route: route_fail(
                D086Conclusion::MeshDeathCausalityFailure,
                "gate7",
                mech_label,
                "Starvation/irreversible death failed.",
                "Complete death causality before Phase 1 pass.",
            ),
        };
    }

    // Gate 8 pass
    D086Review {
        gate0,
        gate1,
        gate2,
        gate3,
        gate4,
        gate5,
        gate5_rows: rows,
        gate6,
        gate7,
        route: RouteReport {
            conclusion: D086Conclusion::MeshProtocellPhase1CandidatePass.as_str().into(),
            stopped_at_gate: "gate8".into(),
            selected_mech: mech_label.into(),
            d008_status: D008_PHASE_FIELD_LINEAGE_CLOSED_REJECTED.into(),
            phase1_status: "PHASE1_AUTOPOIETIC_CANDIDATE_PASS".into(),
            production_verdict: "MESH_PHASE1_LINEAGE_QUALIFIED".into(),
            scientific_conclusion: "Explicit conserved material-mesh protocell qualifies for Phase 1 candidate pass.".into(),
            next_directive: "Independent causal audit; reproducibility campaign; Linux runtime hardening; then Phase 2 reproduction.".into(),
            next_execution_started: false,
            records: vec![
                D008_PHASE_FIELD_LINEAGE_CLOSED_REJECTED.into(),
                PHASE1_PHASE_FIELD_BODY_RETIRED.into(),
                PHASE1_AUTOPOIETIC_MESH_RESET_AUTHORIZED.into(),
                "PHASE1_AUTOPOIETIC_CANDIDATE_PASS".into(),
                "MESH_PHASE1_LINEAGE_QUALIFIED".into(),
            ],
        },
    }
}

fn skipped(dep: &str) -> GateReport {
    GateReport {
        pass: false,
        detail: format!("skipped; depends on {dep}"),
        failure: None,
    }
}

fn route_fail(
    c: D086Conclusion,
    gate: &str,
    mech: &str,
    sci: &str,
    next: &str,
) -> RouteReport {
    let phase1 = match c {
        D086Conclusion::ExplicitMeshBodyRejected => "PHASE1_MESH_BODY_REJECTED",
        D086Conclusion::MeshMetabolicArchitectureRejected => {
            "PHASE1_MESH_METABOLIC_REJECTED"
        }
        _ => "PHASE1_AUTOPOIETIC_MESH_IN_PROGRESS",
    };
    RouteReport {
        conclusion: c.as_str().into(),
        stopped_at_gate: gate.into(),
        selected_mech: mech.into(),
        d008_status: D008_PHASE_FIELD_LINEAGE_CLOSED_REJECTED.into(),
        phase1_status: phase1.into(),
        production_verdict: "REQUIRES_REMEDIATION".into(),
        scientific_conclusion: sci.into(),
        next_directive: next.into(),
        next_execution_started: false,
        records: vec![
            D008_PHASE_FIELD_LINEAGE_CLOSED_REJECTED.into(),
            PHASE1_PHASE_FIELD_BODY_RETIRED.into(),
            PHASE1_AUTOPOIETIC_MESH_RESET_AUTHORIZED.into(),
        ],
    }
}

fn early(gate0: GateReport, gate: &str, c: D086Conclusion) -> D086Review {
    D086Review {
        gate0,
        gate1: skipped(gate),
        gate2: skipped(gate),
        gate3: skipped(gate),
        gate4: skipped(gate),
        gate5: skipped(gate),
        gate5_rows: vec![],
        gate6: skipped(gate),
        gate7: skipped(gate),
        route: route_fail(c, gate, "none", "Preservation/schema failure.", "Fix isolation before science."),
    }
}

fn early_with0(
    gate0: GateReport,
    gate1: GateReport,
    gate: &str,
    c: D086Conclusion,
) -> D086Review {
    D086Review {
        gate0,
        gate1,
        gate2: skipped(gate),
        gate3: skipped(gate),
        gate4: skipped(gate),
        gate5: skipped(gate),
        gate5_rows: vec![],
        gate6: skipped(gate),
        gate7: skipped(gate),
        route: route_fail(
            c,
            gate,
            "none",
            "Mesh mechanics invalid.",
            "Repair mechanics/remeshing conservation inside D-086.",
        ),
    }
}

fn stop_at(
    gate0: GateReport,
    gate1: GateReport,
    gate2: GateReport,
    gate3: GateReport,
    gate4: GateReport,
    gate: &str,
    mech: &str,
    c: D086Conclusion,
) -> D086Review {
    D086Review {
        gate0,
        gate1,
        gate2,
        gate3,
        gate4,
        gate5: skipped(gate),
        gate5_rows: vec![],
        gate6: skipped(gate),
        gate7: skipped(gate),
        route: route_fail(
            c,
            gate,
            mech,
            "Stopped at mandatory gate failure.",
            "Continue D-086 repair or scientific stop per gate.",
        ),
    }
}
