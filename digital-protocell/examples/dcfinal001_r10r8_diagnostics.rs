use chemistry_core::d096_allocation::{
    apply_assay_environment, expression_step, pre_fission_assay, AllocationGenotype,
    AllocationParams, AssayEnvironment,
};
use chemistry_core::material_mesh::{LumpedChem, MaterialMesh};
use chemistry_core::mesh_growth::{growth_step, GrowthLedger, GrowthParams};
use chemistry_core::mesh_reactions::{reactions_step, ReactionParams};
use chemistry_core::mesh_transport::{transport_step, TransportParams};
use chemistry_core::metabolic_reserve::{
    local_r_growth_rate, reserve_schema_load_ok, ReserveParams,
};
use serde::Serialize;
use serde_json::json;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
struct Outcome {
    genotype: [f64; 4],
    environment: String,
    seed: u64,
    reserve_enabled: bool,
    function3_baseline: bool,
    reserve_change: f64,
    structural_change: f64,
    activated_produced: f64,
    damage_applied: f64,
    final_material: f64,
    survived: bool,
}

fn seed_mesh(genotype: AllocationGenotype, seed: u64) -> (MaterialMesh, ReactionParams) {
    let allocation = AllocationParams::default();
    let mut mesh = MaterialMesh::seed_regular(
        12 + (seed % 3) as usize,
        8.0,
        0.0,
        0.0,
        1.0,
        0.8,
        LumpedChem {
            c: 1.0,
            a: 0.5,
            n: 0.8,
            f: 0.8,
            r: 0.5,
            ..LumpedChem::default()
        },
        LumpedChem::default(),
        1.0,
    );
    mesh.enable_finite_allocation(genotype, &allocation);
    let mut reaction = ReactionParams::default();
    reaction.reserve = ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, mesh.area());
    reaction.reserve.enable = true;
    (mesh, reaction)
}

fn growth_step_function3_baseline(
    mesh: &mut MaterialMesh,
    react: &ReactionParams,
    growth: &GrowthParams,
    dt: f64,
) -> GrowthLedger {
    if !react.reserve.enable {
        return growth_step(mesh, react, growth, dt);
    }
    let mut led = GrowthLedger::default();
    if !growth.enable_growth
        || !mesh.can_advance_physics()
        || !reserve_schema_load_ok(mesh, &react.reserve)
    {
        return led;
    }
    let area = mesh.area().max(1e-6);
    for i in 0..mesh.n() {
        if mesh.edges[i].ruptured {
            continue;
        }
        let j_mass = local_r_growth_rate(mesh, i, react, growth.y_g) * dt;
        if j_mass <= 0.0 {
            continue;
        }
        let have_r = mesh.interior.r.max(0.0) * area;
        let take = j_mass.min(have_r);
        if take <= 0.0 {
            continue;
        }
        mesh.interior.r = (mesh.interior.r - take / area).max(0.0);
        let dm = take * growth.y_g.max(0.0);
        let w_product = if matches!(
            react.mesh_schema,
            chemistry_core::mesh_reactions::MeshChemistrySchema::ConservativeV2
                | chemistry_core::mesh_reactions::MeshChemistrySchema::ConservativeV3
        ) || mesh.uses_observer_only_death()
        {
            (take - dm).max(0.0)
        } else {
            take
        };
        mesh.interior.w += w_product / area;
        mesh.edges[i].m += dm;
        if mesh.is_maturation_coupled() {
            mesh.edges[i].m_young += dm;
        }
        led.r_consumed_growth += take;
        led.m_grown += dm;
        led.w_from_growth += w_product;
    }
    led
}

fn run_one(
    genotype: AllocationGenotype,
    environment: AssayEnvironment,
    seed: u64,
    reserve: bool,
    f3_baseline: bool,
) -> Outcome {
    let allocation = AllocationParams::default();
    let (mut mesh, mut reaction) = seed_mesh(genotype, seed);
    reaction.reserve.enable = reserve;
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let initial_reserve = mesh.interior.r * mesh.area();
    let initial_material = mesh.total_structural_mass();
    let mut activated_produced = 0.0;
    let mut damage_applied = 0.0;
    for step in 0..1_000 {
        let env = apply_assay_environment(&mut mesh, environment, step);
        damage_applied += env.structural_damage + env.membrane_damage;
        if expression_step(&mut mesh, &allocation, 0.02).is_err() {
            break;
        }
        let _ = transport_step(&mut mesh, &transport, 0.02);
        let chemistry = reactions_step(&mut mesh, &reaction, 0.02, true, true);
        activated_produced += chemistry.a_produced;
        if f3_baseline {
            let _ = growth_step_function3_baseline(&mut mesh, &reaction, &growth, 0.02);
        } else {
            let _ = growth_step(&mut mesh, &reaction, &growth, 0.02);
        }
        if !mesh.can_advance_physics() {
            break;
        }
    }
    Outcome {
        genotype: genotype.0,
        environment: format!("{environment:?}"),
        seed,
        reserve_enabled: reserve,
        function3_baseline: f3_baseline,
        reserve_change: mesh.interior.r * mesh.area() - initial_reserve,
        structural_change: mesh.total_structural_mass() - initial_material,
        activated_produced,
        damage_applied,
        final_material: mesh.total_structural_mass() + mesh.total_bound_membrane(),
        survived: mesh.observer_viable(),
    }
}

fn main() {
    let output = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp/dcfinal001_r10r8_historical.json"));
    let processing = AllocationGenotype([0.55, 0.25, 0.05, 0.15]);
    let repair = AllocationGenotype([0.10, 0.20, 0.55, 0.15]);
    let neutral = AllocationGenotype::neutral();
    let genotypes = [
        ("processing", processing),
        ("repair", repair),
        ("neutral", neutral),
    ];
    let environments = [
        ("H", AssayEnvironment::H),
        ("B", AssayEnvironment::B),
        ("Neutral", AssayEnvironment::Neutral),
    ];
    let mut rows = Vec::new();
    for (_, genotype) in genotypes {
        for (_, environment) in environments {
            for seed in 1..=8 {
                rows.push(run_one(genotype, environment, seed, true, false));
                rows.push(run_one(genotype, environment, seed, false, false));
                rows.push(run_one(genotype, environment, seed, true, true));
            }
        }
    }
    let historical_test: Vec<_> = (1..=8).map(|seed| {
        let hp = pre_fission_assay(processing, AssayEnvironment::H, seed, 1_000);
        let hr = pre_fission_assay(repair, AssayEnvironment::H, seed, 1_000);
        let bp = pre_fission_assay(processing, AssayEnvironment::B, seed, 1_000);
        let br = pre_fission_assay(repair, AssayEnvironment::B, seed, 1_000);
        let np = pre_fission_assay(processing, AssayEnvironment::Neutral, seed, 1_000);
        let nr = pre_fission_assay(repair, AssayEnvironment::Neutral, seed, 1_000);
        json!({"seed":seed,"h_processing":hp,"h_repair":hr,"b_processing":bp,"b_repair":br,"neutral_processing":np,"neutral_repair":nr,
            "h_reserve_effect": hp.reserve_change-hr.reserve_change,
            "b_material_effect": br.final_material-bp.final_material,
            "neutral_reserve_difference": np.reserve_change-nr.reserve_change,
            "neutral_material_difference": nr.final_material-np.final_material})
    }).collect();
    let source = json!({
        "function_0": "activation/processing gain in N+F->A reaction",
        "function_1": "activation gain paired with function 0 in N+F->A reaction",
        "function_2": "structural build and membrane production gain",
        "function_3": "reserve-funded growth gain only; source grep shows mesh_growth.rs reserve branch as the production read",
        "reserve_off": "function 3 has no production read in the reserve-disabled surplus-A growth branch",
        "historical_assay": "pre_fission_assay enables ReserveParams::derived and reserve.enable=true",
        "historical_duration_steps": 1000,
        "historical_seeds": [1,2,3,4,5,6,7,8]
    });
    let value = json!({
        "diagnostic_only": true,
        "historical_reference": "chemistry_core::d096_allocation::pre_fission_assay",
        "rows": rows,
        "historical_test_replay": historical_test,
        "source_ownership": source,
    });
    fs::write(
        &output,
        serde_json::to_string_pretty(&value).unwrap() + "\n",
    )
    .unwrap();
    println!(
        "R10R8_HISTORICAL_DIAGNOSTICS_COMPLETE output={}",
        output.display()
    );
}
