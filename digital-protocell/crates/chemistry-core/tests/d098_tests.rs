use chemistry_core::d096_allocation::{
    AllocationGenotype, AllocationParams, EQUATION_VERSION_FINITE_CATALYTIC_ALLOCATION,
};
use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, EQUATION_VERSION_MATERIAL_MESH};
use chemistry_core::mesh_growth::{growth_step, GrowthParams};
use chemistry_core::mesh_reactions::ReactionParams;
use chemistry_core::metabolic_reserve::{
    reserve_metab_step, reserve_schema_load_ok, ReserveParams, EQUATION_VERSION_METABOLIC_RESERVE,
};

fn mesh() -> MaterialMesh {
    MaterialMesh::seed_regular(
        12,
        5.0,
        0.0,
        0.0,
        1.0,
        0.7,
        LumpedChem {
            c: 1.0,
            a: 2.0,
            n: 1.0,
            f: 1.0,
            r: 0.0,
            ..LumpedChem::default()
        },
        LumpedChem::default(),
        2.0,
    )
}

fn reserve_params() -> ReserveParams {
    ReserveParams {
        enable: true,
        k_store: 0.2,
        k_release: 0.1,
        k_r_loss: 0.01,
        k_store_half: 0.4,
        k_low: 0.2,
        k_r: 0.2,
        k_growth: 0.3,
        r_max: 2.0,
        store_horizon_mult: 4.0,
    }
}

#[test]
fn reserve_compatibility_preserves_historical_ids_and_accepts_d096() {
    let params = reserve_params();
    let mut base = mesh();
    assert_eq!(base.equation_id, EQUATION_VERSION_MATERIAL_MESH);
    assert!(!reserve_schema_load_ok(&base, &params));

    let mut d091 = base.clone();
    d091.equation_id = EQUATION_VERSION_METABOLIC_RESERVE.into();
    assert!(reserve_schema_load_ok(&d091, &params));

    let mut d092 = base.clone();
    d092.equation_id = chemistry_core::template_polymer::EQUATION_VERSION_CATALYTIC_TEMPLATE.into();
    assert!(reserve_schema_load_ok(&d092, &params));

    let mut d093 = base.clone();
    d093.equation_id = chemistry_core::template_network::EQUATION_VERSION_TEMPLATE_NETWORK.into();
    assert!(reserve_schema_load_ok(&d093, &params));

    let mut d094 = base.clone();
    d094.equation_id = chemistry_core::autocatalytic_nodes::EQUATION_VERSION_AUTOCATALYTIC_SET.into();
    assert!(reserve_schema_load_ok(&d094, &params));

    base.equation_id = "historical_unstamped_or_unknown_mesh".into();
    assert!(!reserve_schema_load_ok(&base, &params));

    let mut d096 = mesh();
    d096.enable_finite_allocation(AllocationGenotype::pulse(), &AllocationParams::default());
    assert_eq!(d096.equation_id, EQUATION_VERSION_FINITE_CATALYTIC_ALLOCATION);
    assert!(reserve_schema_load_ok(&d096, &params));
}

#[test]
fn d096_reserve_path_accumulates_releases_grows_and_closes_accounting() {
    let params = reserve_params();
    let mut mesh = mesh();
    mesh.enable_finite_allocation(AllocationGenotype::pulse(), &AllocationParams::default());
    let mut reaction = ReactionParams::default();
    reaction.reserve = params;
    let area = mesh.area();
    let activation_before = mesh.interior.a * area;
    let reserve_before = mesh.interior.r * area;

    let store = reserve_metab_step(&mut mesh, &reaction, 0.05);
    assert!(store.a_to_r > 0.0);
    assert_eq!(store.rejected_steps, 0);
    let activation_after_store = mesh.interior.a * area;
    let reserve_after_store = mesh.interior.r * area;
    assert!((activation_before + reserve_before
        - activation_after_store
        - reserve_after_store
        - store.r_to_w)
        .abs()
        < 1e-9);

    mesh.interior.a = 0.0;
    let release = reserve_metab_step(&mut mesh, &reaction, 0.05);
    assert!(release.r_to_a > 0.0);
    assert_eq!(release.rejected_steps, 0);

    let reserve_before_growth = mesh.interior.r * area;
    let mass_before_growth = mesh.total_structural_mass();
    let growth = growth_step(
        &mut mesh,
        &reaction,
        &GrowthParams {
            y_g: 0.9,
            enable_growth: true,
        },
        0.2,
    );
    assert!(growth.r_consumed_growth > 0.0);
    assert!(growth.m_grown > 0.0);
    assert!(mesh.interior.r * area < reserve_before_growth);
    assert!(mesh.total_structural_mass() > mass_before_growth);
    let reserve_after_growth = mesh.interior.r * area;
    assert!((reserve_before_growth - reserve_after_growth - growth.r_consumed_growth).abs() < 1e-9);
    eprintln!(
        "d096 causal path: A_before={} A_after_store={} R_after_store={} A_to_R={} R_to_A={} R_to_W={} R_before_growth={} R_after_growth={} R_to_M={} R_to_W_growth={} accounting_residual={}",
        activation_before,
        activation_after_store,
        reserve_after_store,
        store.a_to_r,
        release.r_to_a,
        store.r_to_w + release.r_to_w,
        reserve_before_growth,
        reserve_after_growth,
        growth.r_consumed_growth,
        growth.w_from_growth,
        reserve_before_growth - reserve_after_growth - growth.r_consumed_growth,
    );
}

#[test]
fn d096_candidate_identity_and_budget_remain_frozen() {
    let params = AllocationParams::default();
    let processing = AllocationGenotype([0.55, 0.25, 0.05, 0.15]);
    let repair = AllocationGenotype([0.10, 0.20, 0.55, 0.15]);
    assert!(processing.valid(&params));
    assert!(repair.valid(&params));
    assert_eq!(processing.candidate_hash(&params), "faa5c27f8ee9516f1a71817d66f4ee82414dde8e5ce9ef2f1382162985831e6d");
    assert_eq!(repair.candidate_hash(&params), "e38978484b8872a9bd6344dc5e047c6f4ef53679607b84fd704aa4927af8b802");
}
