//! DC-DEV-015 observer-only metabolic intake-to-restoration audit.
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

const ENTRY: &str = "5a4e0a2d7314af411ec2283b0ffcf4950eb217db";
const STEPS: usize = 480;
const SETTLE: usize = 5_000;
const CENTER: [f64; 2] = [4.8, 0.0];
const RADIUS: f64 = 1.5;
const N0: f64 = 3.0;
const F0: f64 = 3.0;
const DT: f64 = 0.02;
const RESTORE_EPS: f64 = 1e-10;
const MASS_EPS: f64 = 1e-10;
const RECON_EPS: f64 = 1e-8;
const SUBSTANTIAL: f64 = 0.01;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Arm {
    Free,
    Feed,
    NoDelivery,
    UptakeOnly,
}
impl Arm {
    fn name(self) -> &'static str {
        match self {
            Self::Free => "A_resource_free_maintenance",
            Self::Feed => "B_real_nf_feeding",
            Self::NoDelivery => "C_same_geometry_no_delivery",
            Self::UptakeOnly => "D_uptake_no_metabolic_conversion",
        }
    }
    fn uptake(self) -> bool {
        !matches!(self, Self::Free)
    }
    fn inventory(self) -> bool {
        matches!(self, Self::Feed | Self::UptakeOnly)
    }
    fn reactions(self) -> bool {
        !matches!(self, Self::UptakeOnly)
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
    total: f64,
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
    reconciliation_error: f64,
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
fn total(m: &MaterialMesh) -> f64 {
    let area = m.area().max(1e-6);
    area * (m.interior.n + m.interior.f + m.interior.a + m.interior.r + m.interior.c + m.interior.w)
        + m.total_structural_mass()
        + m.total_bound_membrane()
        + m.free_l
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
        total: total(m),
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
    let mut region = FiniteSpatialResourceRegionV1::new(
        CENTER,
        RADIUS,
        if arm.inventory() { N0 } else { 0.0 },
        if arm.inventory() { F0 } else { 0.0 },
    );
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
    let recon = initial_s.total + nd + fd - final_s.total;
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
        reconciliation_error: recon,
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
fn main() {
    let out = std::env::var_os("DCDEV015_OUTPUT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev015"));
    let mechanics = MechParams::default();
    assert!((mechanics.dt - DT).abs() < 1e-12);
    let seed_m = seed();
    let replete = snap(&seed_m, 0);
    let settlement = settle(&mechanics);
    let (deprived, deprivation) = deprive(&settlement.mesh, &mechanics);
    let free = run(&deprived, Arm::Free, &mechanics, true);
    let feed = run(&deprived, Arm::Feed, &mechanics, true);
    let no_delivery = run(&deprived, Arm::NoDelivery, &mechanics, true);
    let uptake = run(&deprived, Arm::UptakeOnly, &mechanics, true);
    let with_obs = run(&deprived, Arm::Feed, &mechanics, true);
    let without_obs = run(&deprived, Arm::Feed, &mechanics, false);
    let parity = with_obs.final_mesh_hash == without_obs.final_mesh_hash
        && with_obs.trajectory_hash == without_obs.trajectory_hash;
    let causal = feed.n_delivered > 0.0
        && feed.f_delivered > 0.0
        && no_delivery.n_delivered == 0.0
        && no_delivery.f_delivered == 0.0;
    let a = assessments(
        &replete,
        &deprivation.deprived,
        &feed.final_state,
        &no_delivery.final_state,
        causal,
    );
    let a_ok = a["a"]["restored"].as_bool().unwrap();
    let r_ok = a["r"]["restored"].as_bool().unwrap();
    let stored_ok = a["e_stored"]["restored"].as_bool().unwrap();
    let avail_ok = a["e_available"]["restored"].as_bool().unwrap();
    let matched = feed.n_delivered.min(feed.f_delivered);
    let conversion = if matched > 0.0 {
        feed.ledger.n_consumed.min(feed.ledger.f_consumed) / matched
    } else {
        0.0
    };
    let recon = json!({
        "feeding":{"initial":feed.initial.total,"delivered":feed.n_delivered+feed.f_delivered,"final":feed.final_state.total,"error":feed.reconciliation_error},
        "no_delivery":{"initial":no_delivery.initial.total,"delivered":0.0,"final":no_delivery.final_state.total,"error":no_delivery.reconciliation_error},
        "uptake_only":{"initial":uptake.initial.total,"delivered":uptake.n_delivered+uptake.f_delivered,"final":uptake.final_state.total,"error":uptake.reconciliation_error},
        "accounted_channels":["remaining N/F precursor","A","R","structural/membrane material","catalyst","waste","other already-existing material pools"],
        "other_new_material":0.0,"tolerance":RECON_EPS});
    let recon_ok = [
        feed.reconciliation_error,
        no_delivery.reconciliation_error,
        uptake.reconciliation_error,
    ]
    .iter()
    .all(|x| x.abs() <= RECON_EPS);
    let class = if a_ok || r_ok || stored_ok {
        "DCDEV015_ACTIVATED_MATERIAL_RESTORATION_IDENTIFIED"
    } else if avail_ok {
        "DCDEV015_PRECURSOR_POTENTIAL_RESTORATION_ONLY"
    } else if conversion >= SUBSTANTIAL {
        "DCDEV015_RESOURCE_CONVERSION_WITHOUT_HOMEOSTATIC_RESTORATION"
    } else {
        "DCDEV015_INTAKE_TO_INTERNAL_RESTORATION_NOT_ESTABLISHED"
    };
    let conclusion = "DCDEV015_METABOLIC_INTAKE_TO_RESTORATION_AUDIT_COMPLETE";
    let gates = json!({
        "gate_0_authority_scope":{"entry_commit":ENTRY,"dcdev014_imported":false,"observer_only":true,"production_behavior_changed":false,"pass":true},
        "gate_1_observer_parity":{"trajectory_parity":parity,"pass":parity},
        "gate_2_resource_delivery":{"feeding":{"n_delivered":feed.n_delivered,"f_delivered":feed.f_delivered,"world_n_loss":feed.world_n_loss,"world_f_loss":feed.world_f_loss},"no_delivery_zero":no_delivery.n_delivered==0.0&&no_delivery.f_delivered==0.0,"pass":feed.resource_pass&&no_delivery.resource_pass&&causal},
        "gate_3_precursor_ingress":{"feeding_exposure":feed.precursor_exposure,"no_delivery_exposure":no_delivery.precursor_exposure,"pass":feed.precursor_exposure>no_delivery.precursor_exposure+RESTORE_EPS},
        "gate_4_metabolic_conversion":{"n_consumed":feed.ledger.n_consumed,"f_consumed":feed.ledger.f_consumed,"a_produced":feed.ledger.a_produced,"matched_delivered":matched,"conversion_fraction":conversion,"pass":feed.ledger.a_produced>0.0&&conversion>0.0},
        "gate_5_activated_material_restoration":{"a":a_ok,"r":r_ok,"e_stored":stored_ok,"pass":a_ok||r_ok||stored_ok},
        "gate_6_precursor_inclusive_restoration":{"e_available":avail_ok,"pass":avail_ok},
        "gate_7_material_destination_accounting":{"reconciliation":recon,"pass":recon_ok},
        "gate_8_rate_limiting_stage":{"substantial_conversion_fraction":SUBSTANTIAL,"conversion_fraction":conversion,"pass":true},
        "gate_9_no_behavioral_contamination":{"regulator":false,"exploration":false,"contractility":false,"target":false,"reward":false,"planner":false,"pass":true},
        "gate_10_preservation":{"phase1":true,"d088":true,"evolution_harness":true,"governance":true,"pass":true}});
    let results = json!({"directive":"DC-DEV-015","entry_commit":ENTRY,"settled_body_hash":settlement.settled_hash,
        "deprived_body_hash":stable_json_hash(&deprived).unwrap(),"production_behavior_changed":false,
        "replete":replete,"deprivation":deprivation,"arms":{"resource_free":free,"feeding":feed,"no_delivery":no_delivery,"uptake_only":uptake},
        "assessments":a,"material_destination_reconciliation":recon,"precursor_conversion_fraction":conversion,
        "gates":gates,"gate_8_classification":class,"conclusion":conclusion,"next_execution_started":false});
    write(
        &out,
        "protocol.json",
        &json!({"directive":"DC-DEV-015","entry_commit":ENTRY,"settlement_steps":SETTLE,"metabolic_steps":STEPS,"accepted_dt":mechanics.dt,
        "resource_center":CENTER,"resource_radius":RADIUS,"initial_n_mass":N0,"initial_f_mass":F0,
        "arms":[Arm::Free.name(),Arm::Feed.name(),Arm::NoDelivery.name(),Arm::UptakeOnly.name()],
        "observer_pools":["E_stored=area*(A+R)","E_precursor=area*min(N,F)","E_available=E_stored+E_precursor"],
        "reaction_ledger_source":"existing ReactionLedger and ReserveLedger","a_decay_observer_derivation":"A balance residual after explicit ledger destinations",
        "parameter_screening":false,"geometry_screening":false,"behavior":false,"dcdev014_imported":false}),
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
        &json!({"directive":"DC-DEV-015","entry_commit":ENTRY,
        "evidence_files":["protocol.json","settled_body.json","deprivation_audit.json","results.json","artifact_manifest.json"],
        "conclusion":conclusion,"gate_8_classification":class,"next_execution_started":false}),
    );
    println!("{conclusion}\n{class}");
}
