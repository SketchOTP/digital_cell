//! DC-FINAL-001 work package 2: finite-field metabolism-coupled chemotaxis.

use chemistry_core::material_mesh::{MaterialMesh, MeshContractVersion};
use chemistry_core::mesh_mechanics::MechParams;
use chemistry_core::mesh_population::MeshPopulation;
use chemistry_core::mesh_reactions::{
    reactions_step_with_reserve_mode, ReactionParams, ReserveDiagnosticMode,
};
use chemistry_core::mesh_self_contact::polygon_simple;
use chemistry_core::mesh_transport::TransportParams;
use regulatory_core::{
    apply_local_activated_energy_contractility_with_external_forces,
    apply_local_activated_energy_contractility_with_stick_slip,
    apply_stick_slip_to_legacy_mechanics, ContractilityParamsV1, SpatialMaterialFieldV1,
    StickSlipTractionParamsV1,
};
use serde_json::{json, Value};
use std::{env, fs, path::PathBuf};

const STEPS: usize = 3_000;
const L: f64 = std::f64::consts::TAU;
const B: f64 = 0.067;
const GAMMA: f64 = 3.55;
const S: f64 = 0.41;
const EPSILON: f64 = 0.6;
const P0: f64 = 0.8;
const P1: f64 = 3.8;
const DU: f64 = 0.1;
const DV: f64 = 1.0;
const DF: f64 = 0.001;

#[derive(Default)]
struct Correlation {
    n: usize,
    x: f64,
    y: f64,
    xx: f64,
    yy: f64,
    xy: f64,
}

impl Correlation {
    fn add(&mut self, x: f64, y: f64) {
        self.n += 1;
        self.x += x;
        self.y += y;
        self.xx += x * x;
        self.yy += y * y;
        self.xy += x * y;
    }

    fn value(&self) -> Option<f64> {
        let n = self.n as f64;
        let covariance = n * self.xy - self.x * self.y;
        let variance = ((n * self.xx - self.x * self.x) * (n * self.yy - self.y * self.y)).sqrt();
        (variance > 0.0).then_some(covariance / variance)
    }
}

fn outward_normal(mesh: &MaterialMesh, edge: usize) -> [f64; 2] {
    let next = (edge + 1) % mesh.n();
    let delta = [
        mesh.vertices[next][0] - mesh.vertices[edge][0],
        mesh.vertices[next][1] - mesh.vertices[edge][1],
    ];
    let length = delta[0].hypot(delta[1]).max(1e-15);
    let orientation = if mesh.signed_area() >= 0.0 { 1.0 } else { -1.0 };
    [orientation * delta[1] / length, -orientation * delta[0] / length]
}

#[derive(Clone)]
struct Polarity {
    u: Vec<f64>,
    v: Vec<f64>,
    f: Vec<f64>,
}

#[derive(Clone)]
struct Derivative {
    u: Vec<f64>,
    v: Vec<f64>,
    f: Vec<f64>,
}

fn equilibrium() -> (f64, f64, f64) {
    let reaction = |u: f64| {
        let v = 2.0 - u;
        let f = P0 + P1 * u;
        (B + GAMMA * u * u) * v - (1.0 + S * f + u * u) * u
    };
    let mut x = 0.0;
    let mut fx = reaction(x);
    for index in 1..=100_000 {
        let y = 2.0 * index as f64 / 100_000.0;
        let fy = reaction(y);
        if fx * fy < 0.0 {
            let (mut lo, mut hi, mut flo) = (x, y, fx);
            for _ in 0..80 {
                let mid = 0.5 * (lo + hi);
                let fm = reaction(mid);
                if flo * fm <= 0.0 {
                    hi = mid;
                } else {
                    lo = mid;
                    flo = fm;
                }
            }
            let u = 0.5 * (lo + hi);
            return (u, 2.0 - u, P0 + P1 * u);
        }
        x = y;
        fx = fy;
    }
    panic!("published Polar homogeneous equilibrium missing")
}

fn widths(mesh: &MaterialMesh) -> Vec<f64> {
    let lengths: Vec<f64> = (0..mesh.n()).map(|i| mesh.edge_length(i)).collect();
    let perimeter: f64 = lengths.iter().sum();
    lengths.into_iter().map(|x| L * x / perimeter).collect()
}

fn diffusion(values: &[f64], ds: &[f64], coefficient: f64, i: usize) -> f64 {
    let n = values.len();
    let previous = (i + n - 1) % n;
    let next = (i + 1) % n;
    let left = 0.5 * (ds[previous] + ds[i]);
    let right = 0.5 * (ds[i] + ds[next]);
    (coefficient * (values[next] - values[i]) / right
        - coefficient * (values[i] - values[previous]) / left)
        / ds[i]
}

fn rhs(state: &Polarity, ds: &[f64], signal: &[f64]) -> Derivative {
    let mut out = Derivative {
        u: vec![0.0; state.u.len()],
        v: vec![0.0; state.u.len()],
        f: vec![0.0; state.u.len()],
    };
    for i in 0..state.u.len() {
        let intrinsic = (B + GAMMA * state.u[i] * state.u[i]) * state.v[i]
            - (1.0 + S * state.f[i] + state.u[i] * state.u[i]) * state.u[i];
        // The only added coupling is the already-published basal activation
        // coefficient B multiplied by a dimensionless, site-local nutrient
        // concentration contrast. It transfers v to u and therefore exactly
        // conserves the active+inactive polarity pool.
        let nutrient_activation = B * signal[i] * state.v[i];
        out.u[i] = intrinsic + nutrient_activation + diffusion(&state.u, ds, DU, i);
        out.v[i] = -intrinsic - nutrient_activation + diffusion(&state.v, ds, DV, i);
        out.f[i] = EPSILON * (P0 + P1 * state.u[i] - state.f[i]) + diffusion(&state.f, ds, DF, i);
    }
    out
}

fn add(state: &Polarity, d: &Derivative, h: f64) -> Polarity {
    Polarity {
        u: state.u.iter().zip(&d.u).map(|(a, b)| a + h * b).collect(),
        v: state.v.iter().zip(&d.v).map(|(a, b)| a + h * b).collect(),
        f: state.f.iter().zip(&d.f).map(|(a, b)| a + h * b).collect(),
    }
}

impl Polarity {
    fn homogeneous(n: usize) -> Self {
        let (u, v, f) = equilibrium();
        Self {
            u: vec![u; n],
            v: vec![v; n],
            f: vec![f; n],
        }
    }

    fn advance(&mut self, mesh: &MaterialMesh, signal: &[f64], total: f64) {
        let ds = widths(mesh);
        let minimum = ds.iter().copied().fold(f64::INFINITY, f64::min);
        let h0 = (0.08 * minimum * minimum).min(total);
        let count = (total / h0).ceil().max(1.0) as usize;
        let h = total / count as f64;
        for _ in 0..count {
            let a = rhs(self, &ds, signal);
            let b = rhs(&add(self, &a, 0.5 * h), &ds, signal);
            let c = rhs(&add(self, &b, 0.5 * h), &ds, signal);
            let d = rhs(&add(self, &c, h), &ds, signal);
            for i in 0..self.u.len() {
                self.u[i] += h * (a.u[i] + 2.0 * b.u[i] + 2.0 * c.u[i] + d.u[i]) / 6.0;
                self.v[i] += h * (a.v[i] + 2.0 * b.v[i] + 2.0 * c.v[i] + d.v[i]) / 6.0;
                self.f[i] += h * (a.f[i] + 2.0 * b.f[i] + 2.0 * c.f[i] + d.f[i]) / 6.0;
            }
        }
    }

    fn motor(&self) -> Vec<f64> {
        self.u
            .iter()
            .zip(&self.v)
            .map(|(u, v)| u / (u + v).max(1e-300))
            .collect()
    }

    fn pool(&self, mesh: &MaterialMesh) -> f64 {
        self.u
            .iter()
            .zip(&self.v)
            .zip(widths(mesh))
            .map(|((u, v), ds)| (u + v) * ds)
            .sum()
    }
}

#[derive(Clone, Copy)]
enum Arm {
    Active,
    SensorOff,
    MotorOff,
    FieldOff,
}

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Self::Active => "sensor_on_motor_on",
            Self::SensorOff => "sensor_off_motor_on",
            Self::MotorOff => "sensor_on_motor_off",
            Self::FieldOff => "nutrient_field_off",
        }
    }
}

fn founder(seed: u64) -> MaterialMesh {
    let mut mesh = MeshPopulation::seed_one(5.0, seed, 2.2)
        .individuals
        .remove(0)
        .mesh;
    mesh.contract_version = MeshContractVersion::MaturationCoupledV4;
    let center = mesh.centroid();
    for point in &mut mesh.vertices {
        point[0] -= center[0];
        point[1] -= center[1];
    }
    mesh
}

fn field(mesh: &MaterialMesh, bearing: f64, enabled: bool) -> SpatialMaterialFieldV1 {
    let nx = 64usize;
    let ny = 64usize;
    let dx = 1.0;
    let origin = [-32.0, -32.0];
    let mut n = vec![0.0; nx * ny];
    let mut f = vec![0.0; nx * ny];
    if enabled {
        let radius = mesh
            .vertices
            .iter()
            .map(|p| p[0].hypot(p[1]))
            .fold(0.0, f64::max);
        let center = [
            (radius + 4.0) * bearing.cos(),
            (radius + 4.0) * bearing.sin(),
        ];
        let ci = ((center[0] - origin[0]) / dx).floor() as isize;
        let cj = ((center[1] - origin[1]) / dx).floor() as isize;
        for y in (cj - 1)..=(cj + 1) {
            for x in (ci - 1)..=(ci + 1) {
                let k = y as usize * nx + x as usize;
                n[k] = 3.0 / 9.0;
                f[k] = 3.0 / 9.0;
            }
        }
    }
    SpatialMaterialFieldV1::new(nx, ny, dx, origin, n, f, 6.0).unwrap()
}

fn local_signal(field: &SpatialMaterialFieldV1, mesh: &MaterialMesh) -> Vec<f64> {
    (0..mesh.n())
        .map(|edge| {
            let a = mesh.vertices[edge];
            let b = mesh.vertices[(edge + 1) % mesh.n()];
            let midpoint = [0.5 * (a[0] + b[0]), 0.5 * (a[1] + b[1])];
            let Some(cell) = field.world_to_cell(midpoint) else {
                return 0.0;
            };
            let (n, f, _) = field.concentration(cell);
            let n_signal = if n + mesh.interior.n > 0.0 {
                n / (n + mesh.interior.n)
            } else {
                0.0
            };
            let f_signal = if f + mesh.interior.f > 0.0 {
                f / (f + mesh.interior.f)
            } else {
                0.0
            };
            0.5 * (n_signal + f_signal)
        })
        .collect()
}

fn run(seed: u64, bearing: f64, arm: Arm) -> Value {
    let mut mesh = founder(seed);
    let initial = mesh.clone();
    let mut world = field(&mesh, bearing, !matches!(arm, Arm::FieldOff));
    let initial_world_n = world.total_n_mass();
    let initial_world_f = world.total_f_mass();
    let mut polarity = Polarity::homogeneous(mesh.n());
    let initial_pool = polarity.pool(&mesh);
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    let transport = TransportParams::default();
    let reaction = ReactionParams::conservative_v3();
    let start = mesh.centroid();
    let mut path = 0.0;
    let mut previous = start;
    let mut delivered_n = 0.0;
    let mut delivered_f = 0.0;
    let mut a_produced = 0.0;
    let mut first_transfer = None;
    let mut first_contact = None;
    let mut max_pool_error: f64 = 0.0;
    let mut invalid = false;
    let mut signal_tension = Correlation::default();
    let mut signal_free_outward = Correlation::default();
    let mut signal_traction = Correlation::default();
    let mut signal_accepted_outward = Correlation::default();
    let mut signal_bearing_velocity = Correlation::default();
    let mut contraction_dominant_sectors = 0usize;
    let mut protrusion_dominant_sectors = 0usize;
    for step in 1..=STEPS {
        world.diffuse(mechanics.dt);
        let mut signal = local_signal(&world, &mesh);
        if matches!(arm, Arm::SensorOff) {
            signal.fill(0.0);
        }
        polarity.advance(&mesh, &signal, mechanics.dt);
        let motor = polarity.motor();
        let before_motion = mesh.clone();
        let mut free_motion = mesh.clone();
        let zero_forces = vec![[0.0, 0.0]; mesh.n()];
        if matches!(arm, Arm::Active) {
            let _ = apply_local_activated_energy_contractility_with_external_forces(
                &mut free_motion,
                &motor,
                &mechanics,
                &contractility,
                Some(&zero_forces),
            )
            .ok();
        }
        let movement = if matches!(arm, Arm::MotorOff) {
            apply_stick_slip_to_legacy_mechanics(&mut mesh, &mechanics, &traction).ok()
                .map(|ledger| (ledger.contacts, None))
        } else {
            apply_local_activated_energy_contractility_with_stick_slip(
                &mut mesh,
                &motor,
                &mechanics,
                &contractility,
                &traction,
            )
            .ok()
            .map(|ledger| (ledger.contacts, ledger.contractility))
        };
        let Some((contacts, accepted_ledger)) = movement else {
            invalid = true;
            break;
        };
        if matches!(arm, Arm::Active) {
            let funding_scale = accepted_ledger
                .as_ref()
                .map(|ledger| ledger.resource_spent / ledger.requested_resource.max(f64::MIN_POSITIVE))
                .unwrap_or(0.0);
            let centroid_before = before_motion.centroid();
            let centroid_after = mesh.centroid();
            let bearing_velocity = ((centroid_after[0] - centroid_before[0]) * bearing.cos()
                + (centroid_after[1] - centroid_before[1]) * bearing.sin())
                / mechanics.dt;
            for edge in 0..mesh.n() {
                let next = (edge + 1) % mesh.n();
                let edge_signal = signal[edge];
                let edge_activity = 0.5 * (motor[edge] + motor[next]);
                let tension = contractility.max_active_tension * edge_activity * funding_scale;
                let normal = outward_normal(&before_motion, edge);
                let free_displacement = [
                    0.5 * (free_motion.vertices[edge][0] + free_motion.vertices[next][0]
                        - before_motion.vertices[edge][0] - before_motion.vertices[next][0]),
                    0.5 * (free_motion.vertices[edge][1] + free_motion.vertices[next][1]
                        - before_motion.vertices[edge][1] - before_motion.vertices[next][1]),
                ];
                let accepted_displacement = [
                    0.5 * (mesh.vertices[edge][0] + mesh.vertices[next][0]
                        - before_motion.vertices[edge][0] - before_motion.vertices[next][0]),
                    0.5 * (mesh.vertices[edge][1] + mesh.vertices[next][1]
                        - before_motion.vertices[edge][1] - before_motion.vertices[next][1]),
                ];
                let free_outward = free_displacement[0] * normal[0] + free_displacement[1] * normal[1];
                let accepted_outward = accepted_displacement[0] * normal[0]
                    + accepted_displacement[1] * normal[1];
                let traction_magnitude = contacts[edge].reaction[0].hypot(contacts[edge].reaction[1]);
                signal_tension.add(edge_signal, tension);
                signal_free_outward.add(edge_signal, free_outward);
                signal_traction.add(edge_signal, traction_magnitude);
                signal_accepted_outward.add(edge_signal, accepted_outward);
                signal_bearing_velocity.add(edge_signal, bearing_velocity);
                if free_outward < 0.0 {
                    contraction_dominant_sectors += 1;
                } else if free_outward > 0.0 {
                    protrusion_dominant_sectors += 1;
                }
            }
        }
        if !polygon_simple(&mesh.vertices) {
            invalid = true;
            break;
        }
        let delivery = world.exchange(std::slice::from_mut(&mut mesh), &transport, mechanics.dt);
        let delivery = &delivery[0];
        if delivery.exposed_edges > 0 && first_contact.is_none() {
            first_contact = Some(step);
        }
        if delivery.n_delivered > 0.0 && first_transfer.is_none() {
            first_transfer = Some(step);
        }
        delivered_n += delivery.n_delivered;
        delivered_f += delivery.f_delivered;
        let ledger = reactions_step_with_reserve_mode(
            &mut mesh,
            &reaction,
            mechanics.dt,
            true,
            true,
            ReserveDiagnosticMode::Full,
        );
        a_produced += ledger.a_produced;
        let centroid = mesh.centroid();
        path += (centroid[0] - previous[0]).hypot(centroid[1] - previous[1]);
        previous = centroid;
        max_pool_error = max_pool_error.max((polarity.pool(&mesh) - initial_pool).abs());
    }
    let end = mesh.centroid();
    json!({
        "seed": seed,
        "bearing_radians": bearing,
        "arm": arm.name(),
        "initial_state_equal_hash_basis": {
            "vertices": &initial.vertices,
            "interior": &initial.interior,
            "structural_mass": initial.total_structural_mass()
        },
        "zero_initial_contact": local_signal(&field(&initial, bearing, !matches!(arm, Arm::FieldOff)), &initial).iter().all(|x| *x == 0.0),
        "first_contact": first_contact,
        "first_transfer": first_transfer,
        "delivered_n": delivered_n,
        "delivered_f": delivered_f,
        "a_produced": a_produced,
        "path": path,
        "net": (end[0]-start[0]).hypot(end[1]-start[1]),
        "displacement_vector": [end[0]-start[0], end[1]-start[1]],
        "displacement_toward_resource":
            (end[0]-start[0])*bearing.cos() + (end[1]-start[1])*bearing.sin(),
        "world_n_remaining": world.total_n_mass(),
        "world_f_remaining": world.total_f_mass(),
        "world_n_closure": (initial_world_n-world.total_n_mass()-delivered_n).abs(),
        "world_f_closure": (initial_world_f-world.total_f_mass()-delivered_f).abs(),
        "polarity_pool_error": max_pool_error,
        "mechanistic_attribution": {
            "signal_vs_local_tension_correlation": signal_tension.value(),
            "signal_vs_free_outward_displacement_correlation": signal_free_outward.value(),
            "signal_vs_traction_magnitude_correlation": signal_traction.value(),
            "signal_vs_accepted_outward_displacement_correlation": signal_accepted_outward.value(),
            "signal_vs_bearing_velocity_correlation": signal_bearing_velocity.value(),
            "contractile_sector_samples": contraction_dominant_sectors,
            "protrusive_sector_samples": protrusion_dominant_sectors,
            "explicit_protrusive_force": false
        },
        "simple": polygon_simple(&mesh.vertices),
        "invalid": invalid
    })
}

fn main() {
    let mut output = PathBuf::from("/tmp/dcfinal001_acquisition.json");
    let args: Vec<String> = env::args().collect();
    for i in 1..args.len() {
        if args[i] == "--output" && i + 1 < args.len() {
            output = PathBuf::from(&args[i + 1]);
        }
    }
    let bearings = [
        0.0,
        std::f64::consts::FRAC_PI_2,
        std::f64::consts::PI,
        3.0 * std::f64::consts::FRAC_PI_2,
    ];
    let mut rows = Vec::new();
    for seed in 1..=3 {
        for bearing in bearings {
            for arm in [Arm::Active, Arm::SensorOff, Arm::MotorOff, Arm::FieldOff] {
                rows.push(run(seed, bearing, arm));
            }
        }
    }
    let mut wins = 0usize;
    for seed in 1..=3 {
        for bearing in bearings {
            let matching: Vec<&Value> = rows
                .iter()
                .filter(|row| {
                    row["seed"].as_u64() == Some(seed)
                        && row["bearing_radians"].as_f64() == Some(bearing)
                })
                .collect();
            let active = matching
                .iter()
                .find(|r| r["arm"] == "sensor_on_motor_on")
                .unwrap();
            let sensor = matching
                .iter()
                .find(|r| r["arm"] == "sensor_off_motor_on")
                .unwrap();
            let motor = matching
                .iter()
                .find(|r| r["arm"] == "sensor_on_motor_off")
                .unwrap();
            if active["delivered_n"].as_f64().unwrap()
                > sensor["delivered_n"].as_f64().unwrap() + 1e-12
                && active["delivered_n"].as_f64().unwrap()
                    > motor["delivered_n"].as_f64().unwrap() + 1e-12
            {
                wins += 1;
            }
        }
    }
    let value = json!({
        "directive":"DC-FINAL-001-END-TO-END-AUTONOMOUS-LIFEFORM-CLOSURE-OR-SHUTDOWN-001",
        "work_package":"WP2_AUTONOMOUS_RESOURCE_ACQUISITION",
        "architecture":"finite conservative SpatialMaterialFieldV1 plus site-local N/F contrast driving B*v transfer in accepted polarity chemistry",
        "new_coupling":"published basal activation B=0.067; no search",
        "wins_vs_sensor_and_motor_controls":wins,
        "comparisons":12,
        "rows":rows
    });
    fs::write(output, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
}
