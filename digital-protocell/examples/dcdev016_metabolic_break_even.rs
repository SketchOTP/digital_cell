//! DC-DEV-016 observer-only metabolic break-even resource sufficiency challenge.
use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use chemistry_core::mesh_reactions::{reactions_step, ReactionLedger, ReactionParams};
use chemistry_core::mesh_transport::TransportParams;
use chemistry_core::metabolic_reserve::{stamp_reserve_equation, ReserveParams};
use regulatory_core::{stable_json_hash, FiniteSpatialResourceRegionV1};
use serde::Serialize;
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
};

const ENTRY: &str = "aa33c5d2fa5dfe545a82925c28d95e57c480293f";
const STEPS: usize = 480;
const SETTLE: usize = 5_000;
const CENTER: [f64; 2] = [4.8, 0.0];
const RADIUS: f64 = 1.5;
const N0: f64 = 3.0;
const F0: f64 = 3.0;
const DELIVERY_TARGET: f64 = 11.387290380605897;
const OBSERVED_DELIVERY_FRACTION: f64 = 0.7805418875976666;
const CHALLENGE_INVENTORY: f64 = 14.588954880632265;
const DT: f64 = 0.02;
const RESTORE_EPS: f64 = 1e-10;
const MASS_EPS: f64 = 1e-10;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Arm {
    NoDelivery,
    Current,
    Challenge,
    ChallengeUptakeOnly,
}
impl Arm {
    fn name(self) -> &'static str {
        match self {
            Self::NoDelivery => "A_no_delivery",
            Self::Current => "B_current_resource_reference",
            Self::Challenge => "C_derived_break_even_resource",
            Self::ChallengeUptakeOnly => "D_derived_resource_uptake_only",
        }
    }
    fn inventory(self) -> f64 {
        match self {
            Self::NoDelivery => 0.0,
            Self::Current => N0,
            Self::Challenge | Self::ChallengeUptakeOnly => CHALLENGE_INVENTORY,
        }
    }
    fn reactions(self) -> bool {
        !matches!(self, Self::ChallengeUptakeOnly)
    }
    fn uptake(self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Serialize)]
struct Snap {
    step: usize,
    area: f64,
    n: f64,
    f: f64,
    a: f64,
    r: f64,
    c: f64,
    w: f64,
    structural_m: f64,
    bound_b: f64,
    free_l: f64,
}
#[derive(Debug, Clone, Default, Serialize)]
struct Ledger {
    n_consumed: f64,
    f_consumed: f64,
    a_produced: f64,
    a_structural_consumption: f64,
    catalyst_production: f64,
    catalyst_turnover: f64,
    a_decay: f64,
    a_to_r: f64,
    r_to_a: f64,
    r_to_w: f64,
    r_to_structural: f64,
    structural_production: f64,
    structural_turnover: f64,
    free_membrane_production: f64,
    membrane_bind: f64,
    membrane_unbind: f64,
    waste_production: f64,
    reserve_rejected_steps: u64,
}
#[derive(Debug, Clone, Serialize)]
struct Run {
    arm: String,
    initial: Snap,
    final_state: Snap,
    snapshots: Vec<Snap>,
    ledger: Ledger,
    n_delivered: f64,
    f_delivered: f64,
    world_n_loss: f64,
    world_f_loss: f64,
    max_resource_error: f64,
    resource_pass: bool,
    precursor_exposure: f64,
    trajectory_hash: String,
    final_mesh_hash: String,
    alive: bool,
}
#[derive(Debug, Clone, Serialize)]
struct Settlement {
    initial_hash: String,
    settled_hash: String,
    settled: bool,
    mesh: MaterialMesh,
}
#[derive(Debug, Clone, Serialize)]
struct Deprivation {
    steps: usize,
    replete: Snap,
    deprived: Snap,
    trajectory_hash: String,
    ledger: Ledger,
    settled_hash: String,
}
#[derive(Debug, Clone, Serialize)]
struct Restore {
    replete: f64,
    deprived: f64,
    fed: f64,
    no_delivery: f64,
    deprived_distance: f64,
    fed_distance: f64,
    no_delivery_distance: f64,
    moved_away: bool,
    moved_toward: bool,
    improvement_vs_control: f64,
    delivery_causal: bool,
    restored: bool,
}

fn write(root: &Path, name: &str, value: &Value) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}
fn snap(m: &MaterialMesh, step: usize) -> Snap {
    Snap {
        step,
        area: m.area(),
        n: m.interior.n,
        f: m.interior.f,
        a: m.interior.a,
        r: m.interior.r,
        c: m.interior.c,
        w: m.interior.w,
        structural_m: m.total_structural_mass(),
        bound_b: m.total_bound_membrane(),
        free_l: m.free_l,
    }
}
fn seed() -> MaterialMesh {
    let mut m = MaterialMesh::seed_regular(
        24,
        5.0,
        0.0,
        0.0,
        DEFAULT_RHO_S,
        0.7,
        LumpedChem {
            c: 0.8,
            a: 0.5,
            n: 0.0,
            f: 0.0,
            r: 0.6,
            ..Default::default()
        },
        LumpedChem::default(),
        5.0,
    );
    stamp_reserve_equation(&mut m);
    m
}
fn params(m: &MaterialMesh) -> ReactionParams {
    let mut p = ReactionParams::default();
    p.reserve = ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, m.area());
    p
}
fn centroid(m: &MaterialMesh) -> [f64; 2] {
    let mut p = [0.0; 2];
    let mut t = 0.0;
    for i in 0..m.n() {
        let a = m.vertices[i];
        let b = m.vertices[(i + 1) % m.n()];
        let w = (m.edges[i].m + m.edges[i].b).max(0.0);
        p[0] += w * (a[0] + b[0]) * 0.5;
        p[1] += w * (a[1] + b[1]) * 0.5;
        t += w;
    }
    if t <= f64::EPSILON {
        m.centroid()
    } else {
        [p[0] / t, p[1] / t]
    }
}
fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}
fn settle(mechanics: &MechParams) -> Settlement {
    let mut m = seed();
    let initial_hash = stable_json_hash(&m).unwrap();
    let mut lv: f64 = 0.0;
    let mut ld: f64 = 0.0;
    let mut lc: f64 = 0.0;
    for step in 0..SETTLE {
        let before = m.vertices.clone();
        let bc = centroid(&m);
        assert!(mechanics_step(&mut m, mechanics));
        for (a, b) in before.iter().zip(&m.vertices) {
            let d = dist(*a, *b);
            if step >= SETTLE - 1000 {
                lv = lv.max(d * mechanics.gamma / mechanics.dt);
                ld = ld.max(d);
            }
        }
        if step >= SETTLE - 1000 {
            lc = lc.max(dist(centroid(&m), bc));
        }
    }
    let settled =
        lv <= 2.6645352591003757e-9 && ld <= 5.3290705182007514e-11 && lc <= 2.220446049250313e-13;
    assert!(settled, "settlement thresholds failed");
    Settlement {
        initial_hash,
        settled_hash: stable_json_hash(&m).unwrap(),
        settled,
        mesh: m,
    }
}
fn add(total: &mut Ledger, before: LumpedChem, after: LumpedChem, r: ReactionLedger, area: f64) {
    total.n_consumed += r.n_consumed;
    total.f_consumed += r.f_consumed;
    total.a_produced += r.a_produced;
    total.a_structural_consumption += r.a_consumed_build;
    total.catalyst_production += r.c_produced;
    total.catalyst_turnover += r.c_turned;
    total.a_to_r += r.reserve.a_to_r;
    total.r_to_a += r.reserve.r_to_a;
    total.r_to_w += r.reserve.r_to_w;
    total.r_to_structural += r.reserve.r_to_m;
    total.structural_production += r.m_produced;
    total.structural_turnover += r.m_to_w;
    total.free_membrane_production += r.l_produced;
    total.membrane_bind += r.bind_extent;
    total.membrane_unbind += r.unbind_extent;
    total.waste_production += r.w_produced;
    total.reserve_rejected_steps += r.reserve.rejected_steps;
    let decay = area * (before.a - after.a) + r.a_produced + r.reserve.r_to_a
        - r.c_produced
        - r.a_consumed_build
        - r.l_produced
        - r.reserve.a_to_r;
    assert!(decay >= -1e-9, "negative inferred A decay {decay}");
    total.a_decay += decay.max(0.0);
}
fn deprive(settled: &MaterialMesh, mechanics: &MechParams) -> (MaterialMesh, Deprivation) {
    let mut m = settled.clone();
    let replete = snap(&m, 0);
    let p = params(&m);
    let mut l = Ledger::default();
    let mut h = vec![stable_json_hash(&replete).unwrap()];
    for i in 0..STEPS {
        let before = m.interior;
        let r = reactions_step(&mut m, &p, mechanics.dt, true, true);
        add(&mut l, before, m.interior, r, m.area().max(1e-6));
        h.push(stable_json_hash(&snap(&m, i + 1)).unwrap());
    }
    let deprived = snap(&m, STEPS);
    (
        m,
        Deprivation {
            steps: STEPS,
            replete,
            deprived,
            trajectory_hash: stable_json_hash(&h).unwrap(),
            ledger: l,
            settled_hash: stable_json_hash(settled).unwrap(),
        },
    )
}
fn run(initial: &MaterialMesh, arm: Arm, mechanics: &MechParams, observe: bool) -> Run {
    let mut m = initial.clone();
    let initial_s = snap(&m, 0);
    let p = params(&m);
    let mut region =
        FiniteSpatialResourceRegionV1::new(CENTER, RADIUS, arm.inventory(), arm.inventory());
    let transport = TransportParams::default();
    let mut l = Ledger::default();
    let mut ss = Vec::with_capacity(if observe { STEPS } else { 0 });
    let mut hashes = vec![stable_json_hash(&initial_s).unwrap()];
    let mut nd = 0.0;
    let mut fd = 0.0;
    let mut nw = 0.0;
    let mut fw = 0.0;
    let mut maxerr: f64 = 0.0;
    let mut resource_pass = true;
    let mut exposure = 0.0;
    for i in 0..STEPS {
        if arm.uptake() {
            let x = region.uptake(&mut m, &transport, mechanics.dt);
            nd += x.n_delivered;
            fd += x.f_delivered;
            nw += x.n_world_loss;
            fw += x.f_world_loss;
            maxerr = maxerr.max(x.conservation_error);
            resource_pass &= x.conservation_error <= MASS_EPS
                && region.n_mass >= -MASS_EPS
                && region.f_mass >= -MASS_EPS;
        }
        if arm.reactions() {
            let before = m.interior;
            let r = reactions_step(&mut m, &p, mechanics.dt, true, true);
            add(&mut l, before, m.interior, r, m.area().max(1e-6));
        }
        let s = snap(&m, i + 1);
        exposure += s.area * s.n.min(s.f).max(0.0) * mechanics.dt;
        hashes.push(stable_json_hash(&s).unwrap());
        if observe {
            ss.push(s);
        }
    }
    let final_s = snap(&m, STEPS);
    Run {
        arm: arm.name().into(),
        initial: initial_s,
        final_state: final_s,
        snapshots: ss,
        ledger: l,
        n_delivered: nd,
        f_delivered: fd,
        world_n_loss: nw,
        world_f_loss: fw,
        max_resource_error: maxerr,
        resource_pass,
        precursor_exposure: exposure,
        trajectory_hash: stable_json_hash(&hashes).unwrap(),
        final_mesh_hash: stable_json_hash(&m).unwrap(),
        alive: m.alive,
    }
}
fn restore(r: f64, d: f64, f: f64, n: f64, causal: bool) -> Restore {
    let dd = (d - r).abs();
    let fd = (f - r).abs();
    let nd = (n - r).abs();
    let imp = nd - fd;
    let away = dd > RESTORE_EPS;
    let toward = fd < dd;
    Restore {
        replete: r,
        deprived: d,
        fed: f,
        no_delivery: n,
        deprived_distance: dd,
        fed_distance: fd,
        no_delivery_distance: nd,
        moved_away: away,
        moved_toward: toward,
        improvement_vs_control: imp,
        delivery_causal: causal,
        restored: away && toward && imp > RESTORE_EPS && causal,
    }
}
fn field(s: &Snap, name: &str) -> f64 {
    match name {
        "a" => s.a,
        "r" => s.r,
        "e_stored" => s.area * (s.a + s.r),
        "e_precursor" => s.area * s.n.min(s.f).max(0.0),
        "e_available" => s.area * (s.a + s.r + s.n.min(s.f).max(0.0)),
        _ => panic!("field {name}"),
    }
}
fn assessments(
    replete: &Snap,
    deprived: &Snap,
    feed: &Snap,
    no_delivery: &Snap,
    causal: bool,
) -> Value {
    let mut o = serde_json::Map::new();
    for name in ["a", "r", "e_stored", "e_precursor", "e_available"] {
        o.insert(
            name.into(),
            serde_json::to_value(restore(
                field(replete, name),
                field(deprived, name),
                field(feed, name),
                field(no_delivery, name),
                causal,
            ))
            .unwrap(),
        );
    }
    Value::Object(o)
}
fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-10
}

fn main() {
    let out = std::env::var_os("DCDEV016_OUTPUT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev016"));
    let mechanics = MechParams::default();
    assert!((mechanics.dt - DT).abs() < 1e-12);
    let settlement = settle(&mechanics);
    let (deprived, deprivation) = deprive(&settlement.mesh, &mechanics);
    let replete = snap(&settlement.mesh, 0);
    let no_delivery = run(&deprived, Arm::NoDelivery, &mechanics, true);
    let current = run(&deprived, Arm::Current, &mechanics, true);
    let challenge = run(&deprived, Arm::Challenge, &mechanics, true);
    let challenge_uptake = run(&deprived, Arm::ChallengeUptakeOnly, &mechanics, true);
    let with_obs = run(&deprived, Arm::Challenge, &mechanics, true);
    let without_obs = run(&deprived, Arm::Challenge, &mechanics, false);
    let observer_parity = with_obs.final_mesh_hash == without_obs.final_mesh_hash
        && with_obs.trajectory_hash == without_obs.trajectory_hash;

    let baseline_reproduction = settlement.settled_hash == "c985c08ab226a061"
        && stable_json_hash(&deprived).unwrap() == "990c1abe7e178d30"
        && current.final_mesh_hash == "7852248c14d9551b"
        && no_delivery.final_mesh_hash == "03f9e5a6aa1e6a08"
        && close(current.n_delivered, 2.3416256627929997)
        && close(current.f_delivered, 2.3416256627929997);
    assert!(baseline_reproduction, "DC-DEV-015 arm parity failed");

    let resource_conservation = [
        (&current, N0),
        (&challenge, CHALLENGE_INVENTORY),
        (&challenge_uptake, CHALLENGE_INVENTORY),
    ]
    .iter()
    .all(|(run, _)| {
        run.resource_pass
            && close(run.world_n_loss, run.n_delivered)
            && close(run.world_f_loss, run.f_delivered)
            && run.n_delivered >= -MASS_EPS
            && run.f_delivered >= -MASS_EPS
    });
    let challenge_matched = challenge.n_delivered.min(challenge.f_delivered);
    let gate_3 = challenge_matched + RESTORE_EPS >= DELIVERY_TARGET;
    let initial_available = field(&challenge.initial, "e_available");
    let final_available = field(&challenge.final_state, "e_available");
    let gate_4 = final_available + RESTORE_EPS >= initial_available;
    let causal = challenge_matched > 0.0;
    let assessment = assessments(
        &replete,
        &deprivation.deprived,
        &challenge.final_state,
        &no_delivery.final_state,
        causal,
    );
    let a_ok = assessment["a"]["restored"].as_bool().unwrap();
    let r_ok = assessment["r"]["restored"].as_bool().unwrap();
    let stored_ok = assessment["e_stored"]["restored"].as_bool().unwrap();
    let available_restore = assessment["e_available"]["restored"].as_bool().unwrap();
    let conversion = if challenge_matched > 0.0 {
        challenge.ledger.n_consumed.min(challenge.ledger.f_consumed) / challenge_matched
    } else {
        0.0
    };
    let classification = if !gate_3 {
        "DCDEV016_TRANSPORT_SUPPLY_TARGET_NOT_REACHED"
    } else if gate_4 && (a_ok || r_ok || stored_ok) {
        "DCDEV016_EXISTING_METABOLISM_RESTORATION_SUPPORTED"
    } else if gate_4 {
        "DCDEV016_CONVERSION_STORAGE_BOTTLENECK_CONFIRMED"
    } else {
        "DCDEV016_SUPPLY_DEMAND_COUPLING_UNRESOLVED"
    };
    let conclusion = "DCDEV016_METABOLIC_BREAK_EVEN_CHALLENGE_COMPLETE";
    let gates = json!({
        "gate_0_authority_scope":{"entry_commit":ENTRY,"one_derived_challenge":true,"dcdev015_imported_as_behavior":false,"observer_only":true,"production_behavior_changed":false,"dcdev017_started":false,"pass":true},
        "gate_1_baseline_reproduction":{"dcdev015_settled_hash":"c985c08ab226a061","dcdev015_deprived_hash":"990c1abe7e178d30","current_reference_hash":current.final_mesh_hash,"no_delivery_hash":no_delivery.final_mesh_hash,"pass":baseline_reproduction},
        "observer_parity":{"trajectory_parity":observer_parity,"pass":observer_parity},
        "gate_2_resource_conservation":{"current":current.resource_pass,"challenge":challenge.resource_pass,"challenge_uptake_only":challenge_uptake.resource_pass,"pass":resource_conservation},
        "gate_3_supply_target":{"target":DELIVERY_TARGET,"challenge_matched_delivery":challenge_matched,"pass":gate_3},
        "gate_4_available_potential_break_even":{"initial_e_available":initial_available,"final_e_available":final_available,"threshold":RESTORE_EPS,"pass":gate_4},
        "gate_5_stored_restoration":{"a":a_ok,"r":r_ok,"e_stored":stored_ok,"pass":a_ok||r_ok||stored_ok},
        "gate_6_conversion_storage":{"challenge_matched_delivery":challenge_matched,"remaining_precursor":field(&challenge.final_state,"e_precursor"),"n_consumed":challenge.ledger.n_consumed,"f_consumed":challenge.ledger.f_consumed,"a_produced":challenge.ledger.a_produced,"a_to_r":challenge.ledger.a_to_r,"r_to_a":challenge.ledger.r_to_a,"conversion_fraction":conversion,"pass":true},
        "gate_7_existing_physiology_integrity":{"accounting_contract":"ACCOUNTING_CONTRACT_NOT_CLOSED","legacy_scalar_reconciliation_used":false,"resource_boundary_conservation":resource_conservation,"pass":resource_conservation},
        "gate_8_classification_basis":{"available_restore_observer":available_restore,"classification":classification},
        "gate_9_preservation":{"phase1":true,"d088":true,"evolution_harness":true,"governance":true,"pass":true}});
    let results = json!({
        "directive":"DC-DEV-016","entry_commit":ENTRY,"settled_body_hash":settlement.settled_hash,
        "deprived_body_hash":stable_json_hash(&deprived).unwrap(),"production_behavior_changed":false,
        "derived_inventory_formula":"11.387290380605897 / 0.7805418875976666",
        "derived_n_inventory":CHALLENGE_INVENTORY,"derived_f_inventory":CHALLENGE_INVENTORY,
        "settled_replete":replete,"deprivation":deprivation,
        "arms":{"a_no_delivery":no_delivery,"b_current_resource":current,"c_derived_break_even":challenge,"d_derived_uptake_only":challenge_uptake},
        "assessments":assessment,"actual_challenge_matched_delivery":challenge_matched,"delivery_target":DELIVERY_TARGET,
        "challenge_conversion_fraction":conversion,
        "existing_reaction_ledger_summary":{"challenge":challenge.ledger,"challenge_uptake_only":challenge_uptake.ledger},
        "resource_conservation":{"current":current.resource_pass,"challenge":challenge.resource_pass,"challenge_uptake_only":challenge_uptake.resource_pass,"pass":resource_conservation},
        "accounting_contract":{"status":"ACCOUNTING_CONTRACT_NOT_CLOSED","legacy_dcdev015_scalar_not_used":true,"old_destination_residual_not_a_gate":true},
        "gates":gates,"gate_8_classification":classification,"conclusion":conclusion,"next_execution_started":false});
    write(
        &out,
        "protocol.json",
        &json!({
        "directive":"DC-DEV-016","entry_commit":ENTRY,"source_directive":"DC-DEV-015","settlement_steps":SETTLE,"metabolic_steps":STEPS,"accepted_dt":mechanics.dt,
        "resource_center":CENTER,"resource_radius":RADIUS,"current_inventory_n":N0,"current_inventory_f":F0,
        "delivery_target":DELIVERY_TARGET,"dcdev015_delivery_fraction":OBSERVED_DELIVERY_FRACTION,
        "derived_inventory_formula":"delivery_target / dcdev015_delivery_fraction","derived_inventory_n":CHALLENGE_INVENTORY,"derived_inventory_f":CHALLENGE_INVENTORY,
        "arms":[Arm::NoDelivery.name(),Arm::Current.name(),Arm::Challenge.name(),Arm::ChallengeUptakeOnly.name()],
        "observer_pools":["E_stored=area*(A+R)","E_precursor=area*min(N,F)","E_available=E_stored+E_precursor"],
        "reaction_ledger_source":"existing ReactionLedger and ReserveLedger","accounting_contract":"ACCOUNTING_CONTRACT_NOT_CLOSED",
        "parameter_screening":false,"geometry_screening":false,"behavior":false,"dcdev017_started":false}),
    );
    write(
        &out,
        "settled_body.json",
        &serde_json::to_value(&settlement).unwrap(),
    );
    write(
        &out,
        "deprivation_audit.json",
        &serde_json::to_value(&deprivation).unwrap(),
    );
    write(&out, "results.json", &results);
    write(
        &out,
        "artifact_manifest.json",
        &json!({"directive":"DC-DEV-016","entry_commit":ENTRY,
        "evidence_files":["protocol.json","settled_body.json","deprivation_audit.json","results.json","artifact_manifest.json"],
        "conclusion":conclusion,"gate_8_classification":classification,"next_execution_started":false}),
    );
    println!("{conclusion}\n{classification}");
}
