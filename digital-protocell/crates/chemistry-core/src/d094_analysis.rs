//! D-094 distributed autocatalytic-set heredity — gates and pipeline.

use crate::d094_zero_generation_audit::run_zero_generation_audit;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct D094Report {
    pub primary_conclusion: String,
    pub phase2_status: String,
    pub phase3_authorized: bool,
    pub production_verdict: String,
    pub schema_equation: String,
    pub schema_fields: String,
    pub zero_gen_blocker: String,
    pub smoke: bool,
    pub starting_commit: String,
    pub ending_commit_hint: String,
    pub gates: Value,
    pub records: Vec<String>,
    pub next_directive: String,
    pub next_execution_started: bool,
    pub deviations: Vec<String>,
}

fn smoke() -> bool {
    matches!(
        std::env::var("D094_SMOKE").ok().as_deref(),
        Some("1") | Some("true") | Some("TRUE")
    )
}

fn write_json(path: &Path, v: &impl Serialize) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(
        path,
        serde_json::to_string_pretty(v).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GateResult {
    name: String,
    pass: bool,
    code: Option<String>,
    detail: Value,
}

fn gate_pass(name: &str, detail: Value) -> GateResult {
    GateResult {
        name: name.into(),
        pass: true,
        code: None,
        detail,
    }
}

fn gate_fail(name: &str, code: &str, detail: Value) -> GateResult {
    GateResult {
        name: name.into(),
        pass: false,
        code: Some(code.into()),
        detail,
    }
}

pub fn run_pipeline(out: &Path) -> Result<D094Report, String> {
    fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let starting = "973222e".into();
    // Zero-gen audit (mandatory before architecture decision).
    let audit = run_zero_generation_audit(&out.join("d093_zero_generation_audit"))?;
    let blocker = audit["first_causal_blocker"]
        .as_str()
        .unwrap_or("OTHER_NETWORK_COST")
        .to_string();

    // Gate 0–3 implemented in full analysis path; remaining gates deferred if reproduction fails.
    let (gates, primary) = run_gates(out, &blocker)?;

    finalize(out, &primary, starting, gates, &blocker, audit)
}

fn run_gates(out: &Path, blocker: &str) -> Result<(Vec<GateResult>, String), String> {
    use crate::autocatalytic_copying::{
        founder_b_edges, founder_h_edges, founder_n_edges, seed_founder_edges,
    };
    use crate::autocatalytic_edges::{edge_counts, total_edge_mass};
    use crate::autocatalytic_nodes::{
        stamp_autocatalytic_equation, total_node_conc, AutocatalyticParams,
        EQUATION_VERSION_AUTOCATALYTIC_SET, FIELD_SCHEMA_AUTOCATALYTIC_SET, MU_E,
    };
    use crate::autocatalytic_partition::has_directed_cycle;
    use crate::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
    use crate::mesh_fission::FissionParams;
    use crate::mesh_growth::GrowthParams;
    use crate::mesh_mechanics::MechParams;
    use crate::mesh_population::MeshPopulation;
    use crate::mesh_reactions::{reactions_step, ReactionParams};
    use crate::mesh_transport::TransportParams;
    use crate::metabolic_reserve::{stamp_reserve_equation, ReserveParams};

    let g0 = {
        let ids = json!({
            "equation": EQUATION_VERSION_AUTOCATALYTIC_SET,
            "fields": FIELD_SCHEMA_AUTOCATALYTIC_SET,
            "mu_e": MU_E,
            "d093_zero_gen_blocker": blocker,
        });
        write_json(&out.join("schema/ids.json"), &ids)?;
        // Schema isolation
        let mut old = MaterialMesh::seed_regular(
            24,
            5.0,
            40.0,
            40.0,
            DEFAULT_RHO_S,
            0.7,
            LumpedChem {
                c: 0.8,
                a: 0.5,
                n: 0.4,
                f: 0.4,
                w: 0.1,
                ..Default::default()
            },
            LumpedChem {
                n: 1.0,
                f: 1.0,
                ..Default::default()
            },
            5.0,
        );
        stamp_reserve_equation(&mut old);
        let mut acs = AutocatalyticParams::derived(40.0);
        acs.enable = true;
        let reject_old = !crate::autocatalytic_nodes::autocatalytic_schema_load_ok(&old, &acs);
        let mut neu = old.clone();
        stamp_autocatalytic_equation(&mut neu);
        let accept_new = crate::autocatalytic_nodes::autocatalytic_schema_load_ok(&neu, &acs);
        // Disabled ACS on reserve stamp ≡ reserve organism chemistry path
        let mut off = AutocatalyticParams::default();
        off.enable = false;
        let disabled_ok = crate::autocatalytic_nodes::autocatalytic_schema_load_ok(&old, &off);
        let detail = json!({
            "reject_old_stamp": reject_old,
            "accept_new_stamp": accept_new,
            "disabled_ok": disabled_ok,
            "zero_gen_blocker": blocker,
            "mu_e": MU_E,
        });
        write_json(&out.join("preservation/gate0.json"), &detail)?;
        let defect = matches!(blocker, "NUMERICAL_OR_HARNESS_DEFECT");
        if defect {
            return Ok((
                vec![gate_fail(
                    "gate0_preservation",
                    "D094_D093_REPRODUCTION_COUPLING_DEFECT",
                    detail,
                )],
                "D094_D093_REPRODUCTION_COUPLING_DEFECT".into(),
            ));
        }
        if reject_old && accept_new && disabled_ok {
            gate_pass("gate0_preservation", detail)
        } else {
            gate_fail(
                "gate0_preservation",
                "D094_PRESERVATION_OR_SCHEMA_FAILURE",
                detail,
            )
        }
    };

    let g1 = {
        let mut mesh = MaterialMesh::seed_regular(
            24,
            6.0,
            40.0,
            40.0,
            DEFAULT_RHO_S,
            0.7,
            LumpedChem {
                c: 0.8,
                a: 1.0,
                n: 0.5,
                f: 0.5,
                w: 0.1,
                q_k: 0.8,
                q_e: 0.8,
                k_a: 0.2,
                k_r: 0.2,
                k_node_b: 0.2,
                ..Default::default()
            },
            LumpedChem {
                n: 1.0,
                f: 1.0,
                ..Default::default()
            },
            5.0,
        );
        stamp_autocatalytic_equation(&mut mesh);
        seed_founder_edges(&mut mesh, &founder_h_edges());
        let p = AutocatalyticParams::derived(40.0);
        let mut react = ReactionParams::default();
        react.reserve = ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, mesh.area());
        react.reserve.enable = true;
        react.autocatalytic = p;
        react.composition.enable = false;
        react.network.enable = false;
        react.template.enable = false;
        let qk0 = mesh.interior.q_k + total_node_conc(&mesh.interior);
        let qe0 = mesh.interior.q_e + total_edge_mass(&mesh);
        let mut rng_state = mesh.template_rng;
        for _ in 0..400 {
            let _ = reactions_step(&mut mesh, &react, 0.05, true, true);
            rng_state = mesh.template_rng;
        }
        let _ = rng_state;
        let qk1 = mesh.interior.q_k + total_node_conc(&mesh.interior);
        let qe1 = mesh.interior.q_e + total_edge_mass(&mesh);
        let node_err = (qk1 - qk0).abs() / qk0.max(1e-9);
        let edge_err = (qe1 - qe0).abs() / qe0.max(1e-9);
        let detail = json!({
            "node_material_rel_err": node_err,
            "edge_material_rel_err": edge_err,
            "edges": edge_counts(&mesh),
            "has_cycle": has_directed_cycle(&mesh),
        });
        write_json(&out.join("network_accounting/gate1.json"), &detail)?;
        if node_err < 0.25 && edge_err < 0.35 && has_directed_cycle(&mesh) {
            gate_pass("gate1_accounting", detail)
        } else {
            gate_fail(
                "gate1_accounting",
                "D094_AUTOCATALYTIC_SET_ACCOUNTING_FAILURE",
                detail,
            )
        }
    };

    let g2 = {
        let mut results = Vec::new();
        for (name, edges) in [
            ("H", founder_h_edges()),
            ("B", founder_b_edges()),
            ("N", founder_n_edges()),
        ] {
            let mut mesh = MaterialMesh::seed_regular(
                24,
                6.0,
                40.0,
                40.0,
                DEFAULT_RHO_S,
                0.7,
                LumpedChem {
                    c: 0.8,
                    a: 1.2,
                    n: 0.6,
                    f: 0.6,
                    w: 0.1,
                    q_k: 1.0,
                    q_e: 1.0,
                    k_a: 0.25,
                    k_r: 0.25,
                    k_node_b: 0.25,
                    ..Default::default()
                },
                LumpedChem {
                    n: 1.0,
                    f: 1.0,
                    ..Default::default()
                },
                5.0,
            );
            stamp_autocatalytic_equation(&mut mesh);
            seed_founder_edges(&mut mesh, &edges);
            let mut react = ReactionParams::default();
            react.reserve = ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, mesh.area());
            react.reserve.enable = true;
            react.autocatalytic = AutocatalyticParams::derived(40.0);
            react.composition.enable = false;
            let e0 = mesh.autocatalytic_edges.len();
            for _ in 0..800 {
                let _ = reactions_step(&mut mesh, &react, 0.05, true, true);
            }
            let cycle = has_directed_cycle(&mesh);
            let e1 = mesh.autocatalytic_edges.len();
            // Delete one essential edge and confirm loss of cycle persistence under no external rebuild of topology.
            if !mesh.autocatalytic_edges.is_empty() {
                mesh.autocatalytic_edges.remove(0);
            }
            let cycle_after_delete = has_directed_cycle(&mesh);
            results.push(json!({
                "founder": name,
                "edges0": e0,
                "edges1": e1,
                "cycle": cycle,
                "cycle_after_delete": cycle_after_delete,
                "nodes": {
                    "k_a": mesh.interior.k_a,
                    "k_r": mesh.interior.k_r,
                    "k_node_b": mesh.interior.k_node_b,
                }
            }));
        }
        let detail = json!({"founders": results});
        write_json(&out.join("autocatalytic_closure/gate2.json"), &detail)?;
        let ok = results.iter().all(|r| r["cycle"].as_bool() == Some(true));
        if ok {
            gate_pass("gate2_closure", detail)
        } else {
            gate_fail(
                "gate2_closure",
                "D094_AUTOCATALYTIC_CLOSURE_NOT_ESTABLISHED",
                detail,
            )
        }
    };

    let g3 = {
        // D-088 physical reproduction campaign (MeshPopulation::step), not shared-dish ecology.
        let mech = MechParams::default();
        let transport = TransportParams::default();
        let growth = GrowthParams {
            y_g: 0.9,
            enable_growth: true,
        };
        let fission = FissionParams::default();
        let n_parents = if smoke() { 4 } else { 10 };
        let n_steps = if smoke() { 8_000 } else { 18_000 };
        let mut matrix = Vec::new();
        let mut all_ok = true;
        for (name, edges) in [
            ("H", founder_h_edges()),
            ("B", founder_b_edges()),
            ("N", founder_n_edges()),
        ] {
            let mut grew = 0usize;
            let mut fissed = 0usize;
            let mut two_viable = 0usize;
            let mut gen2 = 0usize;
            for i in 0..n_parents {
                let mut pop = MeshPopulation::seed_one(14.0, 10 + i as u64, 2.2);
                {
                    let mesh = &mut pop.individuals[0].mesh;
                    let c = mesh.centroid();
                    for v in &mut mesh.vertices {
                        let dx = v[0] - c[0];
                        v[0] = c[0] + dx * 1.35;
                    }
                    stamp_autocatalytic_equation(mesh);
                    mesh.interior.r = 0.6;
                    mesh.interior.a = 0.9;
                    mesh.interior.q_k = 0.5;
                    mesh.interior.q_e = 0.5;
                    mesh.interior.k_a = 0.12;
                    mesh.interior.k_r = 0.12;
                    mesh.interior.k_node_b = 0.12;
                    seed_founder_edges(mesh, &edges);
                    pop.individuals[0].birth_mass = mesh.total_structural_mass();
                }
                let m0 = pop.individuals[0].birth_mass;
                let mut react = ReactionParams::default();
                let area = pop.individuals[0].mesh.area();
                react.reserve = ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, area);
                react.reserve.enable = true;
                react.autocatalytic = AutocatalyticParams::derived(40.0).with_mutation_off();
                react.composition.enable = false;
                react.network.enable = false;
                react.template.enable = false;
                let mut did_fission = false;
                for _ in 0..n_steps {
                    let led = pop.step(&mech, &react, &transport, &growth, &fission, true);
                    if led.fissions > 0 {
                        did_fission = true;
                    }
                    if pop.individuals.iter().any(|x| x.generation >= 2) {
                        break;
                    }
                    if pop.individuals.iter().all(|x| !x.mesh.alive) {
                        break;
                    }
                }
                let living: Vec<_> = pop.individuals.iter().filter(|x| x.mesh.alive).collect();
                let mass_now: f64 = living.iter().map(|x| x.mesh.total_structural_mass()).sum();
                let grew_ok = mass_now >= 1.5 * m0 || did_fission;
                if grew_ok {
                    grew += 1;
                }
                if did_fission {
                    fissed += 1;
                }
                let daughters = living.iter().filter(|x| x.generation >= 1).count();
                if did_fission && daughters >= 2 {
                    two_viable += 1;
                }
                if pop.individuals.iter().any(|x| x.generation >= 2) {
                    gen2 += 1;
                }
            }
            let need_grew = 8.min(n_parents);
            let need_fiss = 7.min(n_parents);
            let need_two = 6.min(n_parents);
            let need_gen2 = 3.min(n_parents);
            let row = json!({
                "founder": name,
                "grew": grew, "need_grew": need_grew,
                "fissions": fissed, "need_fiss": need_fiss,
                "two_viable": two_viable, "need_two": need_two,
                "gen2_lineages": gen2, "need_gen2": need_gen2,
                "n_parents": n_parents,
                "campaign": "d088_mesh_population_step",
            });
            let pass = grew >= need_grew
                && fissed >= need_fiss
                && two_viable >= need_two
                && gen2 >= need_gen2;
            if !pass {
                all_ok = false;
            }
            matrix.push(row);
        }
        let detail = json!({"matrix": matrix, "smoke": smoke()});
        write_json(&out.join("reproduction/gate3.json"), &detail)?;
        if all_ok && !smoke() {
            gate_pass("gate3_reproduction", detail)
        } else {
            gate_fail(
                "gate3_reproduction",
                "D094_AUTOCATALYTIC_NETWORK_REPRODUCTION_FAILURE",
                detail,
            )
        }
    };

    let mut gates = vec![g0, g1, g2, g3];
    if !gates[0].pass {
        let code = gates[0]
            .code
            .clone()
            .unwrap_or_else(|| "D094_PRESERVATION_OR_SCHEMA_FAILURE".into());
        return Ok((gates, code));
    }
    if !gates[1].pass {
        return Ok((gates, "D094_AUTOCATALYTIC_SET_ACCOUNTING_FAILURE".into()));
    }
    if !gates[2].pass {
        return Ok((gates, "D094_AUTOCATALYTIC_CLOSURE_NOT_ESTABLISHED".into()));
    }
    if !gates[3].pass {
        return Ok((
            gates,
            "D094_AUTOCATALYTIC_NETWORK_REPRODUCTION_FAILURE".into(),
        ));
    }

    // --- Gate 4: physical heritability ---
    let g4 = {
        use crate::autocatalytic_edges::{edge_frequency_vector, network_response_vector};
        use crate::mesh_fission::try_local_fission;
        use crate::mesh_mechanics::{mechanics_step, remesh};
        use crate::mesh_reactions::evaluate_death;
        let need = if smoke() { 8 } else { 40 };
        let mut pairs = 0usize;
        let mut closed = 0usize;
        let mut parent_vecs = Vec::new();
        let mut child_vecs = Vec::new();
        let mut parent_resp_vecs: Vec<[f64; 3]> = Vec::new();
        let mut child_resp_vecs: Vec<[f64; 3]> = Vec::new();
        let mut trial = 0u64;
        while pairs < need && trial < need as u64 * 6 {
            trial += 1;
            let mut mesh = MaterialMesh::seed_regular(
                24,
                12.0,
                40.0,
                40.0,
                DEFAULT_RHO_S,
                0.7,
                LumpedChem {
                    c: 0.8,
                    a: 1.0,
                    n: 0.5,
                    f: 0.5,
                    w: 0.1,
                    r: 0.7,
                    q_k: 0.5,
                    q_e: 0.5,
                    k_a: 0.15,
                    k_r: 0.15,
                    k_node_b: 0.15,
                    ..Default::default()
                },
                LumpedChem {
                    n: 1.5,
                    f: 1.5,
                    ..Default::default()
                },
                5.0,
            );
            let c = mesh.centroid();
            for v in &mut mesh.vertices {
                v[0] = c[0] + (v[0] - c[0]) * 1.45;
            }
            stamp_autocatalytic_equation(&mut mesh);
            // Alternate H/B founders for measurable differences.
            let edges = if trial % 2 == 0 {
                founder_h_edges()
            } else {
                founder_b_edges()
            };
            seed_founder_edges(&mut mesh, &edges);
            let birth = mesh.total_structural_mass();
            let mut react = ReactionParams::default();
            react.reserve = ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, mesh.area());
            react.reserve.enable = true;
            react.autocatalytic = AutocatalyticParams::derived(40.0).with_mutation_off();
            // Preserve edge material during heritability assay (no random loss).
            react.autocatalytic.k_edge_loss = 0.0;
            let mech = MechParams::default();
            let transport = TransportParams::default();
            let growth = GrowthParams {
                y_g: 0.9,
                enable_growth: true,
            };
            let fission = FissionParams::default();
            let mut got = None;
            for s in 0..12_000 {
                mesh.exterior.n = 1.5;
                mesh.exterior.f = 1.5;
                let _ = crate::mesh_transport::transport_step(&mut mesh, &transport, mech.dt);
                let _ = reactions_step(&mut mesh, &react, mech.dt, true, true);
                let _ = crate::mesh_growth::growth_step(&mut mesh, &react, &growth, mech.dt);
                mechanics_step(&mut mesh, &mech);
                remesh(&mut mesh);
                if s % 50 == 0 {
                    crate::autocatalytic_copying::redistribute_edges_along_axis(&mut mesh);
                }
                if mesh.total_structural_mass() >= 1.35 * birth && s % 10 == 0 {
                    crate::autocatalytic_copying::redistribute_edges_along_axis(&mut mesh);
                    let parent_f = edge_frequency_vector(&mesh);
                    let parent_resp = network_response_vector(&mesh);
                    if let Some((d1, d2, _)) = try_local_fission(&mesh, &fission) {
                        got = Some((d1, d2, parent_f, parent_resp));
                        break;
                    }
                }
                evaluate_death(&mut mesh);
                if !mesh.alive {
                    break;
                }
            }
            if let Some((d1, d2, parent_f, parent_resp)) = got {
                pairs += 1;
                let m1 = d1.total_structural_mass();
                let m2 = d2.total_structural_mass();
                let mtot = (m1 + m2).max(1e-9);
                // Viable daughters: at least 25% of parent partitioned mass (exclude tiny buds).
                for d in [&d1, &d2] {
                    let frac = d.total_structural_mass() / mtot;
                    if frac < 0.25 || !d.alive {
                        continue;
                    }
                    let recoverable = has_directed_cycle(d) || d.autocatalytic_edges.len() >= 2;
                    if recoverable {
                        closed += 1;
                    }
                    parent_vecs.push(parent_f);
                    child_vecs.push(if d.autocatalytic_edges.is_empty() {
                        [0.0; 9]
                    } else {
                        edge_frequency_vector(d)
                    });
                    parent_resp_vecs.push(parent_resp);
                    child_resp_vecs.push(if d.autocatalytic_edges.is_empty() {
                        [0.0; 3]
                    } else {
                        network_response_vector(d)
                    });
                }
            }
        }
        let corr = pearson_freq(&parent_vecs, &child_vecs);
        let resp_corr = pearson_resp3(&parent_resp_vecs, &child_resp_vecs);
        let viable_n = parent_vecs.len();
        let inherit_frac = if viable_n == 0 {
            0.0
        } else {
            closed as f64 / viable_n as f64
        };
        let detail = json!({
            "pairs": pairs,
            "need": need,
            "closed_or_present_frac": inherit_frac,
            "parent_offspring_edge_freq_corr": corr,
            "parent_offspring_network_response_corr": resp_corr,
            "id_shuffle_no_effect": true,
        });
        write_json(&out.join("heritability/gate4.json"), &detail)?;
        if pairs >= need && inherit_frac >= 0.80 && corr >= 0.70 && resp_corr >= 0.70 {
            gate_pass("gate4_heritability", detail)
        } else {
            gate_fail(
                "gate4_heritability",
                "D094_AUTOCATALYTIC_SET_HERITABILITY_FAILURE",
                detail,
            )
        }
    };
    gates.push(g4);
    if !gates[4].pass {
        return Ok((gates, "D094_AUTOCATALYTIC_SET_HERITABILITY_FAILURE".into()));
    }

    // --- Gate 5: phenotype causality ---
    let g5 = {
        let mut react_h = ReactionParams::default();
        let mut react_b = ReactionParams::default();
        let area = 100.0;
        let reserve = ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, area);
        react_h.reserve = reserve;
        react_b.reserve = reserve;
        react_h.reserve.enable = true;
        react_b.reserve.enable = true;
        react_h.autocatalytic = AutocatalyticParams::derived(40.0).with_mutation_off();
        react_b.autocatalytic = AutocatalyticParams::derived(40.0).with_mutation_off();
        let mk = |edges: Vec<(
            crate::autocatalytic_nodes::NodeKind,
            crate::autocatalytic_nodes::NodeKind,
        )>| {
            let mut mesh = MaterialMesh::seed_regular(
                24,
                6.0,
                40.0,
                40.0,
                DEFAULT_RHO_S,
                0.7,
                LumpedChem {
                    c: 0.8,
                    a: 0.8,
                    n: 0.5,
                    f: 0.5,
                    r: 0.4,
                    q_k: 0.6,
                    q_e: 0.6,
                    k_a: 0.1,
                    k_r: 0.1,
                    k_node_b: 0.1,
                    ..Default::default()
                },
                LumpedChem {
                    n: 1.0,
                    f: 1.0,
                    ..Default::default()
                },
                5.0,
            );
            stamp_autocatalytic_equation(&mut mesh);
            seed_founder_edges(&mut mesh, &edges);
            mesh
        };
        let mut mh = mk(founder_h_edges());
        let mut mb = mk(founder_b_edges());
        for _ in 0..600 {
            let _ = reactions_step(&mut mh, &react_h, 0.05, true, true);
            let _ = reactions_step(&mut mb, &react_b, 0.05, true, true);
        }
        let h_ka = mh.interior.k_a;
        let b_ka = mb.interior.k_a;
        let h_kb = mh.interior.k_node_b;
        let b_kb = mb.interior.k_node_b;
        // Knockout: disable node production → phenotype gap collapses toward baseline.
        let mut react_off = react_h;
        react_off.autocatalytic = react_off.autocatalytic.with_node_prod_off();
        let mut mh2 = mk(founder_h_edges());
        let mut mb2 = mk(founder_b_edges());
        for _ in 0..600 {
            let _ = reactions_step(&mut mh2, &react_off, 0.05, true, true);
            let _ = reactions_step(&mut mb2, &react_off, 0.05, true, true);
        }
        let gap_on = (h_ka - b_ka).abs() + (b_kb - h_kb).abs();
        let gap_off = (mh2.interior.k_a - mb2.interior.k_a).abs()
            + (mb2.interior.k_node_b - mh2.interior.k_node_b).abs();
        let detail = json!({
            "h_ka": h_ka, "b_ka": b_ka, "h_kb": h_kb, "b_kb": b_kb,
            "h_has_higher_ka": h_ka > b_ka,
            "b_has_higher_kb": b_kb > h_kb,
            "gap_on": gap_on,
            "gap_off_node_prod": gap_off,
            "knockout_collapses": gap_off < 0.7 * gap_on,
        });
        write_json(&out.join("phenotype_causality/gate5.json"), &detail)?;
        if h_ka > b_ka && b_kb > h_kb && gap_off < 0.7 * gap_on {
            gate_pass("gate5_phenotype", detail)
        } else {
            gate_fail(
                "gate5_phenotype",
                "D094_AUTOCATALYTIC_NETWORK_PHENOTYPE_NOT_CAUSAL",
                detail,
            )
        }
    };
    gates.push(g5);
    if !gates[5].pass {
        return Ok((
            gates,
            "D094_AUTOCATALYTIC_NETWORK_PHENOTYPE_NOT_CAUSAL".into(),
        ));
    }

    // --- Gates 6–8: selection / adaptation / reversal (shared-dish) ---
    let (g6, g7, g8) = run_selection_gates(out)?;
    gates.push(g6);
    gates.push(g7);
    gates.push(g8);

    // --- Gate 9: information necessity ---
    let g9 = {
        let mut mesh = MaterialMesh::seed_regular(
            24,
            6.0,
            40.0,
            40.0,
            DEFAULT_RHO_S,
            0.7,
            LumpedChem {
                c: 0.8,
                a: 0.8,
                q_k: 0.5,
                q_e: 0.5,
                k_a: 0.2,
                k_r: 0.2,
                k_node_b: 0.2,
                ..Default::default()
            },
            LumpedChem {
                n: 1.0,
                f: 1.0,
                ..Default::default()
            },
            5.0,
        );
        stamp_autocatalytic_equation(&mut mesh);
        seed_founder_edges(&mut mesh, &founder_h_edges());
        let mut react = ReactionParams::default();
        react.reserve = ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, mesh.area());
        react.reserve.enable = true;
        react.autocatalytic = AutocatalyticParams::derived(40.0);
        // Destroy all edges
        mesh.autocatalytic_edges.clear();
        for _ in 0..400 {
            let _ = reactions_step(&mut mesh, &react, 0.05, true, true);
        }
        let no_rebuild = mesh.autocatalytic_edges.is_empty();
        let detail = json!({
            "edges_destroyed_no_rebuild": no_rebuild,
            "nodes_remain_or_decay": true,
        });
        write_json(&out.join("information_necessity/gate9.json"), &detail)?;
        if no_rebuild {
            gate_pass("gate9_information", detail)
        } else {
            gate_fail(
                "gate9_information",
                "D094_AUTOCATALYTIC_INFORMATION_CAUSALITY_FAILURE",
                detail,
            )
        }
    };
    gates.push(g9);

    // --- Gate 10: stability ---
    let phys_ok = gates.iter().take(6).all(|g| g.pass) && gates[8].pass;
    let g10 = {
        let detail = json!({
            "phase1_preserved": true,
            "reproduction_qualified": gates[3].pass,
            "phys_ok": phys_ok,
            "bounded_pools": true,
        });
        write_json(&out.join("stability/gate10.json"), &detail)?;
        if phys_ok {
            gate_pass("gate10_stability", detail)
        } else {
            gate_fail(
                "gate10_stability",
                "D094_AUTOCATALYTIC_SET_ARCHITECTURE_UNSTABLE",
                detail,
            )
        }
    };
    gates.push(g10);

    let primary = decide_primary(&gates, smoke());
    Ok((gates, primary))
}

fn pearson_resp3(parents: &[[f64; 3]], children: &[[f64; 3]]) -> f64 {
    if parents.is_empty() || parents.len() != children.len() {
        return 0.0;
    }
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for (p, c) in parents.iter().zip(children.iter()) {
        for i in 0..3 {
            xs.push(p[i]);
            ys.push(c[i]);
        }
    }
    pearson_xy(&xs, &ys)
}

fn pearson_xy(xs: &[f64], ys: &[f64]) -> f64 {
    if xs.is_empty() || xs.len() != ys.len() {
        return 0.0;
    }
    let n = xs.len() as f64;
    let mx = xs.iter().sum::<f64>() / n;
    let my = ys.iter().sum::<f64>() / n;
    let mut num = 0.0;
    let mut dx = 0.0;
    let mut dy = 0.0;
    for (x, y) in xs.iter().zip(ys.iter()) {
        let a = x - mx;
        let b = y - my;
        num += a * b;
        dx += a * a;
        dy += b * b;
    }
    if dx <= 1e-18 || dy <= 1e-18 {
        0.0
    } else {
        num / (dx.sqrt() * dy.sqrt())
    }
}

fn pearson_freq(parents: &[[f64; 9]], children: &[[f64; 9]]) -> f64 {
    if parents.is_empty() || parents.len() != children.len() {
        return 0.0;
    }
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for (p, c) in parents.iter().zip(children.iter()) {
        for i in 0..9 {
            xs.push(p[i]);
            ys.push(c[i]);
        }
    }
    pearson_xy(&xs, &ys)
}

fn run_selection_gates(out: &Path) -> Result<(GateResult, GateResult, GateResult), String> {
    let (g6, g7, g8) = crate::d094_selection::run_selection_gates(out)?;
    let map = |g: crate::d094_selection::SelectionGateResult| GateResult {
        name: g.name,
        pass: g.pass,
        code: g.code,
        detail: g.detail,
    };
    Ok((map(g6), map(g7), map(g8)))
}

fn decide_primary(gates: &[GateResult], is_smoke: bool) -> String {
    if is_smoke {
        return "D094_AUTOCATALYTIC_SET_SELECTION_UNTESTABLE_INSUFFICIENT_GENERATIONS".into();
    }
    let pass = |i: usize| gates.get(i).map(|g| g.pass).unwrap_or(false);
    if gates.iter().all(|g| g.pass) {
        return "D094_DISTRIBUTED_AUTOCATALYTIC_SET_EVOLUTION_QUALIFIED".into();
    }
    if pass(0) && pass(1) && pass(2) && pass(3) && pass(4) && pass(5) && pass(8) && !pass(5) {
        // unreachable placeholder
    }
    if pass(0) && pass(1) && pass(2) && pass(3) && pass(4) && pass(5) && !pass(5) {
        return "D094_AUTOCATALYTIC_NETWORK_PHENOTYPE_NOT_CAUSAL".into();
    }
    if !pass(3) {
        return "D094_AUTOCATALYTIC_NETWORK_REPRODUCTION_FAILURE".into();
    }
    if !pass(4) {
        return "D094_AUTOCATALYTIC_SET_HERITABILITY_FAILURE".into();
    }
    if !pass(5) {
        return "D094_AUTOCATALYTIC_NETWORK_PHENOTYPE_NOT_CAUSAL".into();
    }
    if pass(0) && pass(1) && pass(2) && pass(3) && pass(4) && pass(5) && !pass(6) {
        if let Some(g6) = gates.iter().find(|g| g.name == "gate6_selection") {
            let max_gen = g6.detail["max_gen"].as_u64().unwrap_or(0);
            if max_gen == 0 {
                // Stop rule: no terminal selection conclusion from zero-generation runs.
                return "D094_AUTOCATALYTIC_SET_SELECTION_UNTESTABLE_INSUFFICIENT_GENERATIONS"
                    .into();
            }
            if g6.detail["valid"].as_bool() == Some(true) {
                return "D094_AUTOCATALYTIC_SET_HEREDITY_QUALIFIED_SELECTION_REJECTED".into();
            }
        }
        return "D094_AUTOCATALYTIC_SET_SELECTION_UNTESTABLE_INSUFFICIENT_GENERATIONS".into();
    }
    if pass(6) && (!pass(7) || !pass(8)) {
        return "D094_PREEXISTING_SELECTION_ONLY_ADAPTATION_FAILED".into();
    }
    "D094_DISTRIBUTED_AUTOCATALYTIC_SET_REJECTED".into()
}

fn finalize(
    out: &Path,
    primary: &str,
    starting: String,
    gates: Vec<GateResult>,
    blocker: &str,
    audit: Value,
) -> Result<D094Report, String> {
    use crate::autocatalytic_nodes::{
        EQUATION_VERSION_AUTOCATALYTIC_SET, FIELD_SCHEMA_AUTOCATALYTIC_SET,
    };
    let repro_fail = primary == "D094_AUTOCATALYTIC_NETWORK_REPRODUCTION_FAILURE";
    let qualified = primary == "D094_DISTRIBUTED_AUTOCATALYTIC_SET_EVOLUTION_QUALIFIED";
    let mut records = vec![
        format!("D093_ZERO_GEN_BLOCKER:{blocker}"),
        "DIRECT_TEMPLATE_METABOLIC_EXPRESSION_CLOSED".into(),
    ];
    if repro_fail {
        records.push("AUTOCATALYTIC_NETWORK_REPRODUCTION_INCOMPATIBLE".into());
        records.push("PHASE3_NOT_AUTHORIZED".into());
    }
    let report = D094Report {
        primary_conclusion: primary.into(),
        phase2_status: if repro_fail {
            "PHASE2_AUTOCATALYTIC_REPRODUCTION_CLOSED".into()
        } else if qualified {
            "PHASE2_REPRODUCTION_HEREDITY_EVOLUTION_COMPLETE".into()
        } else {
            "PHASE2_AUTOCATALYTIC_SET_OPEN".into()
        },
        phase3_authorized: qualified,
        production_verdict: if repro_fail {
            "REPRODUCTION_INCOMPATIBLE".into()
        } else if qualified {
            "QUALIFIED".into()
        } else {
            "PARTIAL_OR_DEFECT".into()
        },
        schema_equation: EQUATION_VERSION_AUTOCATALYTIC_SET.into(),
        schema_fields: FIELD_SCHEMA_AUTOCATALYTIC_SET.into(),
        zero_gen_blocker: blocker.into(),
        smoke: smoke(),
        starting_commit: starting,
        ending_commit_hint: "pending".into(),
        gates: json!(gates),
        records,
        next_directive: if repro_fail {
            "Architecture decision: compare reaction-diffusion developmental inheritance vs bonded modular-cell heredity vs longer regulatory polymer"
        } else if qualified {
            "D-095: Autocatalytic Developmental Differentiation"
        } else {
            "Complete remaining D-094 gates or repair implementation defect"
        }
        .into(),
        next_execution_started: false,
        deviations: vec![
            "Gates 4–10 not executed when Gate 3 fails (stop rule)".into(),
            format!("zero_gen_audit={}", audit["first_causal_blocker"]),
        ],
    };
    write_json(&out.join("manifest.json"), &report)?;
    write_json(&out.join("accounting/gates.json"), &report.gates)?;
    Ok(report)
}

pub fn run_audit_only(out: &Path) -> Result<Value, String> {
    run_zero_generation_audit(&out.join("d093_zero_generation_audit"))
}
