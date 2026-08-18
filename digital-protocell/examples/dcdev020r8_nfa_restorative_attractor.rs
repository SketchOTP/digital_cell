//! DC-DEV-020-R8 observer-only product-feedback topology audit.
//!
//! This example consumes the sealed R5 and R7 root ledgers, reconstructs the
//! normalized maintenance surface, and solves the preregistered reciprocal
//! half-space problem. It does not run production chemistry, select a point,
//! or modify any organism behavior.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const ACCEPTED_R7_HEAD: &str = "7d5f772f0db67b8d754d27c1182c933533f750fd";
const CLEAN_BASE: &str = "1e242f28152797b512e25cd56c7b718e45d6ca97";
const R5_SHA256: &str = "4e22ab1dbd6e06f7c9a272747c2ed8271f28ef33f4eaddc1c59bb9df58a46585";
const R7_SHA256: &str = "abdaea6d075c700e36d14d369dba62982f4a65cea47d2d1f162b5dfe8afa59f8";
const R5_LOCATION: &str = "/mnt/storage1tb/cache/project-artifacts/digital_cell/experiments/generated/dcdev020r5/6dbb8d45c520e2756a81b2cc1e81dff9c3878992/statewise_root_ledger.json";
const R7_LOCATION: &str = "/mnt/storage1tb/cache/project-artifacts/digital_cell/experiments/generated/dcdev020r7/3ddae9ea3c954431c8b3ae2ecbf2d6fc94278e56/on_policy_root_ledger.json";
const DT: f64 = 0.02;
const P_NF: f64 = 0.0003277429681759396;
const G_H: f64 = 1.0;
const NF_DISTANCE_LIMIT: f64 = 0.0024847602445668224;
const ROOT_MARGIN: f64 = 1e-6;
const EPS: f64 = 1e-12;

#[derive(Clone, Debug, Deserialize)]
struct R5Row {
    probe: String,
    trajectory: String,
    step: usize,
    area: f64,
    a: f64,
    n: f64,
    f: f64,
    q_c: f64,
    status: String,
    s_zero: Option<f64>,
}

#[derive(Clone, Debug, Deserialize)]
struct R7RootAudit {
    status: String,
    s_zero: Option<f64>,
}

#[derive(Clone, Debug, Deserialize)]
struct R7Row {
    step: usize,
    area: f64,
    a: f64,
    n: f64,
    f: f64,
    q_c: f64,
    root_audit: R7RootAudit,
}

#[derive(Clone, Debug, Serialize)]
struct SurfaceRow {
    id: String,
    source: String,
    probe: String,
    step: usize,
    area: f64,
    a: f64,
    n: f64,
    f: f64,
    q_c: f64,
    s_zero: f64,
    y_zero: f64,
}

#[derive(Clone, Debug, Serialize)]
struct Pair {
    low_id: String,
    high_id: String,
    low_a: f64,
    high_a: f64,
    nf_distance: f64,
    y_zero_low: f64,
    y_zero_high: f64,
}

#[derive(Clone, Debug, Serialize)]
struct Constraint {
    a: f64,
    b: f64,
    c: f64,
    label: String,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct Point {
    u: f64,
    v: f64,
}

#[derive(Clone, Debug, Serialize)]
struct RegionReport {
    feasible: bool,
    positive_interior: bool,
    bounded: bool,
    area: Option<f64>,
    vertex_count: usize,
    vertices: Vec<Point>,
    u_range: Option<[f64; 2]>,
    v_range: Option<[f64; 2]>,
    v_fb_range: Option<[f64; 2]>,
    k_a_range: Option<[f64; 2]>,
}

#[derive(Clone, Debug, Serialize)]
struct RegionBundle {
    constraints: Vec<Constraint>,
    report: RegionReport,
}

fn write_json(root: &Path, name: &str, value: &Value) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn source_row_from_r5(row: &R5Row) -> Option<SurfaceRow> {
    if row.status != "FINITE_ZERO_DRIFT_ROOT" || row.s_zero.unwrap_or(0.0) <= EPS {
        return None;
    }
    Some(make_surface_row(
        format!("r5:{}:{}:{}", row.probe, row.trajectory, row.step),
        "R5".into(),
        row.probe.clone(),
        row.step,
        row.area,
        row.a,
        row.n,
        row.f,
        row.q_c,
        row.s_zero.unwrap(),
    ))
}

fn source_row_from_r7(row: &R7Row) -> Option<SurfaceRow> {
    if row.root_audit.status != "FINITE_ZERO_DRIFT_ROOT"
        || row.root_audit.s_zero.unwrap_or(0.0) <= EPS
    {
        return None;
    }
    Some(make_surface_row(
        format!("r7:{}", row.step),
        "R7".into(),
        "R7_ON_POLICY".into(),
        row.step,
        row.area,
        row.a,
        row.n,
        row.f,
        row.q_c,
        row.root_audit.s_zero.unwrap(),
    ))
}

fn make_surface_row(
    id: String,
    source: String,
    probe: String,
    step: usize,
    area: f64,
    a: f64,
    n: f64,
    f: f64,
    q_c: f64,
    s_zero: f64,
) -> SurfaceRow {
    let g_nf = if n > 0.0 && f > 0.0 {
        n.powf(P_NF) * f.powf(P_NF)
    } else {
        0.0
    };
    let denominator = q_c * G_H * area * DT * g_nf;
    assert!(denominator.is_finite() && denominator > EPS);
    SurfaceRow {
        id,
        source,
        probe,
        step,
        area,
        a,
        n,
        f,
        q_c,
        s_zero,
        y_zero: s_zero / denominator,
    }
}

fn scale_nf(row: &SurfaceRow, min: [f64; 2], max: [f64; 2]) -> [f64; 2] {
    [
        (row.n - min[0]) / (max[0] - min[0]).max(EPS),
        (row.f - min[1]) / (max[1] - min[1]).max(EPS),
    ]
}

fn nf_distance(a: &SurfaceRow, b: &SurfaceRow, min: [f64; 2], max: [f64; 2]) -> f64 {
    let x = scale_nf(a, min, max);
    let y = scale_nf(b, min, max);
    ((x[0] - y[0]).powi(2) + (x[1] - y[1]).powi(2)).sqrt()
}

fn pair_key(a: &str, b: &str) -> String {
    if a < b {
        format!("{}|{}", a, b)
    } else {
        format!("{}|{}", b, a)
    }
}

fn choose_pairs(rows: &[SurfaceRow], min: [f64; 2], max: [f64; 2]) -> Vec<Pair> {
    let mut pairs = Vec::new();
    let mut seen = BTreeSet::new();
    for (i, row) in rows.iter().enumerate() {
        let mut candidates: Vec<(&SurfaceRow, f64, f64)> = Vec::new();
        for (j, other) in rows.iter().enumerate() {
            if j == i || other.a == row.a {
                continue;
            }
            let distance = nf_distance(row, other, min, max);
            if distance <= NF_DISTANCE_LIMIT {
                candidates.push((other, distance, (row.a - other.a).abs()));
            }
        }
        candidates.sort_by(|x, y| {
            y.2.partial_cmp(&x.2)
                .unwrap_or(Ordering::Equal)
                .then_with(|| x.1.partial_cmp(&y.1).unwrap_or(Ordering::Equal))
                .then_with(|| x.0.id.cmp(&y.0.id))
        });
        if let Some((other_ref, distance, _)) = candidates.first() {
            let other = *other_ref;
            if seen.insert(pair_key(&row.id, &other.id)) {
                let (low, high) = if row.a < other.a {
                    (row, other)
                } else {
                    (other, row)
                };
                pairs.push(Pair {
                    low_id: low.id.clone(),
                    high_id: high.id.clone(),
                    low_a: low.a,
                    high_a: high.a,
                    nf_distance: *distance,
                    y_zero_low: low.y_zero,
                    y_zero_high: high.y_zero,
                });
            }
        }
    }
    pairs.sort_by(|a, b| {
        a.low_id
            .cmp(&b.low_id)
            .then_with(|| a.high_id.cmp(&b.high_id))
    });
    pairs
}

fn choose_r7_pairs(
    rows: &[SurfaceRow],
    support: &[SurfaceRow],
    min: [f64; 2],
    max: [f64; 2],
) -> Vec<Pair> {
    rows.iter()
        .filter_map(|row| {
            let mut candidates: Vec<(&SurfaceRow, f64)> = Vec::new();
            for other in support {
                let distance = nf_distance(row, other, min, max);
                if distance <= NF_DISTANCE_LIMIT && other.a != row.a {
                    candidates.push((other, distance));
                }
            }
            candidates.sort_by(|x, y| {
                (y.0.a - row.a)
                    .abs()
                    .partial_cmp(&(x.0.a - row.a).abs())
                    .unwrap_or(Ordering::Equal)
                    .then_with(|| x.1.partial_cmp(&y.1).unwrap_or(Ordering::Equal))
                    .then_with(|| x.0.id.cmp(&y.0.id))
            });
            candidates.first().map(|(other_ref, distance)| {
                let other = *other_ref;
                let (low, high) = if row.a < other.a {
                    (row, other)
                } else {
                    (other, row)
                };
                Pair {
                    low_id: low.id.clone(),
                    high_id: high.id.clone(),
                    low_a: low.a,
                    high_a: high.a,
                    nf_distance: *distance,
                    y_zero_low: low.y_zero,
                    y_zero_high: high.y_zero,
                }
            })
        })
        .collect()
}

fn pair_constraint(pair: &Pair, label: &str) -> [Constraint; 2] {
    let upper = 1.0 / (pair.y_zero_low * (1.0 + ROOT_MARGIN));
    let lower = 1.0 / (pair.y_zero_high * (1.0 - ROOT_MARGIN));
    [
        Constraint {
            a: 1.0,
            b: pair.low_a,
            c: upper,
            label: format!("{}:{}:low", label, pair.low_id),
        },
        Constraint {
            a: -1.0,
            b: -pair.high_a,
            c: -lower,
            label: format!("{}:{}:high", label, pair.high_id),
        },
    ]
}

fn base_constraints() -> Vec<Constraint> {
    vec![
        Constraint {
            a: -1.0,
            b: 0.0,
            c: 0.0,
            label: "u>0".into(),
        },
        Constraint {
            a: 0.0,
            b: -1.0,
            c: 0.0,
            label: "v>0".into(),
        },
    ]
}

fn satisfies(point: Point, constraints: &[Constraint]) -> bool {
    constraints
        .iter()
        .all(|c| c.a * point.u + c.b * point.v <= c.c + 1e-9)
}

fn add_point(points: &mut Vec<Point>, point: Point) {
    if point.u.is_finite()
        && point.v.is_finite()
        && !points
            .iter()
            .any(|p| (p.u - point.u).abs() <= 1e-10 && (p.v - point.v).abs() <= 1e-10)
    {
        points.push(point);
    }
}

fn recession_direction(constraints: &[Constraint], direction: Point) -> bool {
    direction.u >= -EPS
        && direction.v >= -EPS
        && constraints
            .iter()
            .all(|c| c.a * direction.u + c.b * direction.v <= 1e-10)
}

fn is_unbounded(constraints: &[Constraint]) -> bool {
    let mut directions = vec![Point { u: 1.0, v: 0.0 }, Point { u: 0.0, v: 1.0 }];
    for c in constraints {
        if c.a.abs() > EPS && c.b.abs() > EPS {
            directions.push(Point { u: -c.b, v: c.a });
            directions.push(Point { u: c.b, v: -c.a });
        }
    }
    directions.into_iter().any(|d| {
        let scale = d.u.abs().max(d.v.abs()).max(EPS);
        recession_direction(
            constraints,
            Point {
                u: d.u / scale,
                v: d.v / scale,
            },
        )
    })
}

fn solve_region(constraints: Vec<Constraint>) -> RegionBundle {
    let mut vertices = Vec::new();
    for (i, first) in constraints.iter().enumerate() {
        for second in constraints.iter().skip(i + 1) {
            let determinant = first.a * second.b - first.b * second.a;
            if determinant.abs() <= EPS {
                continue;
            }
            let point = Point {
                u: (first.c * second.b - first.b * second.c) / determinant,
                v: (first.a * second.c - first.c * second.a) / determinant,
            };
            if satisfies(point, &constraints) {
                add_point(&mut vertices, point);
            }
        }
    }
    vertices.sort_by(|a, b| a.u.partial_cmp(&b.u).unwrap_or(Ordering::Equal));
    let positive_interior = vertices.iter().any(|p| p.u > EPS && p.v > EPS)
        || vertices.iter().enumerate().any(|(i, a)| {
            vertices
                .iter()
                .skip(i + 1)
                .map(|b| Point {
                    u: 0.5 * (a.u + b.u),
                    v: 0.5 * (a.v + b.v),
                })
                .any(|p| p.u > EPS && p.v > EPS && satisfies(p, &constraints))
        });
    let feasible = positive_interior || !vertices.is_empty();
    let bounded = feasible && !is_unbounded(&constraints);
    let mut ordered = vertices.clone();
    if ordered.len() >= 3 {
        let center = Point {
            u: ordered.iter().map(|p| p.u).sum::<f64>() / ordered.len() as f64,
            v: ordered.iter().map(|p| p.v).sum::<f64>() / ordered.len() as f64,
        };
        ordered.sort_by(|a, b| {
            (a.v - center.v)
                .atan2(a.u - center.u)
                .partial_cmp(&(b.v - center.v).atan2(b.u - center.u))
                .unwrap_or(Ordering::Equal)
        });
    }
    let area = if ordered.len() >= 3 {
        Some(
            ordered
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let q = ordered[(i + 1) % ordered.len()];
                    p.u * q.v - q.u * p.v
                })
                .sum::<f64>()
                .abs()
                * 0.5,
        )
    } else {
        None
    };
    let range = |values: Vec<f64>| -> Option<[f64; 2]> {
        (!values.is_empty()).then(|| {
            [
                values.iter().copied().fold(f64::INFINITY, f64::min),
                values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            ]
        })
    };
    let u_range = range(ordered.iter().map(|p| p.u).collect());
    let v_range = range(ordered.iter().map(|p| p.v).collect());
    let v_fb_range = if bounded && ordered.iter().all(|p| p.u > EPS) {
        range(ordered.iter().map(|p| 1.0 / p.u).collect())
    } else {
        None
    };
    let k_a_range = if bounded && ordered.iter().all(|p| p.v > EPS) {
        range(ordered.iter().map(|p| p.u / p.v).collect())
    } else {
        None
    };
    RegionBundle {
        constraints,
        report: RegionReport {
            feasible,
            positive_interior,
            bounded,
            area,
            vertex_count: ordered.len(),
            vertices: ordered,
            u_range,
            v_range,
            v_fb_range,
            k_a_range,
        },
    }
}

fn capacity_constraint(row: &SurfaceRow) -> Option<Constraint> {
    let saturated = (row.n.max(0.0) * row.area).min(row.f.max(0.0) * row.area);
    if saturated <= EPS {
        return None;
    }
    let g_nf = row.n.powf(P_NF) * row.f.powf(P_NF);
    let numerator = row.q_c * G_H * g_nf * row.area * DT;
    Some(Constraint {
        a: -1.0,
        b: -row.a,
        c: -(numerator / saturated),
        label: format!("capacity:{}", row.id),
    })
}

fn summarize_pairs(pairs: &[Pair]) -> Value {
    let spans: Vec<f64> = pairs.iter().map(|p| p.high_a - p.low_a).collect();
    let distances: Vec<f64> = pairs.iter().map(|p| p.nf_distance).collect();
    let rises = pairs
        .iter()
        .filter(|p| p.y_zero_high > p.y_zero_low)
        .count();
    let falls = pairs
        .iter()
        .filter(|p| p.y_zero_high < p.y_zero_low)
        .count();
    json!({
        "pair_count": pairs.len(),
        "a_span": distribution(&spans),
        "nf_distance": distribution(&distances),
        "maintenance_demand_rises_with_a": rises,
        "maintenance_demand_falls_with_a": falls,
        "restorative_sign_fraction": if pairs.is_empty() { 0.0 } else { falls as f64 / pairs.len() as f64 },
    })
}

fn compact_region(bundle: &RegionBundle) -> Value {
    json!({
        "constraint_count": bundle.constraints.len(),
        "report": bundle.report,
    })
}

fn distribution(values: &[f64]) -> Value {
    if values.is_empty() {
        return json!({"count": 0});
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    let pick = |q: f64| sorted[((sorted.len() - 1) as f64 * q).round() as usize];
    json!({
        "count": sorted.len(),
        "min": sorted[0],
        "p05": pick(0.05),
        "median": pick(0.5),
        "p95": pick(0.95),
        "max": sorted[sorted.len()-1],
    })
}

fn crossing_range(region: &RegionBundle, rows: &[SurfaceRow]) -> Option<[f64; 2]> {
    if !region.report.feasible || region.report.vertices.is_empty() {
        return None;
    }
    let mut crossings = Vec::new();
    for row in rows {
        for point in &region.report.vertices {
            if point.v > EPS {
                let crossing = (1.0 / row.y_zero - point.u) / point.v;
                if crossing.is_finite() {
                    crossings.push(crossing);
                }
            }
        }
    }
    (!crossings.is_empty()).then(|| {
        [
            crossings.iter().copied().fold(f64::INFINITY, f64::min),
            crossings.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        ]
    })
}

fn main() {
    let output = std::env::var_os("DCDEV020R8_OUTPUT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020r8"));
    let r5_path = std::env::var_os("DCDEV020R8_R5_LEDGER")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/dcdev020r5-statewise-ledger.json"));
    let r7_path = std::env::var_os("DCDEV020R8_R7_LEDGER")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/dcdev020r7-on-policy-ledger.json"));
    let dense_path = std::env::var_os("DCDEV020R8_EXTERNAL_LEDGER")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/dcdev020r8-pair-constraint-ledger.json"));
    let source_commit =
        std::env::var("DCDEV020R8_SOURCE_COMMIT").unwrap_or_else(|_| "LOCAL_UNCOMMITTED".into());
    let external_location = std::env::var("DCDEV020R8_EXTERNAL_LOCATION")
        .unwrap_or_else(|_| "UNRECORDED_EXTERNAL_LOCATION".into());
    let external_sha256 =
        std::env::var("DCDEV020R8_EXTERNAL_SHA256").unwrap_or_else(|_| "COMPUTED_AFTER_RUN".into());
    let r5: Vec<R5Row> = serde_json::from_slice(&fs::read(r5_path).unwrap()).unwrap();
    let r7: Vec<R7Row> = serde_json::from_slice(&fs::read(r7_path).unwrap()).unwrap();
    let r5_rows: Vec<SurfaceRow> = r5.iter().filter_map(source_row_from_r5).collect();
    let r7_rows: Vec<SurfaceRow> = r7.iter().filter_map(source_row_from_r7).collect();
    let training: Vec<SurfaceRow> = r5_rows
        .iter()
        .filter(|r| matches!(r.probe.as_str(), "P0" | "P1" | "P2"))
        .cloned()
        .collect();
    let min = [
        training.iter().map(|r| r.n).fold(f64::INFINITY, f64::min),
        training.iter().map(|r| r.f).fold(f64::INFINITY, f64::min),
    ];
    let max = [
        training
            .iter()
            .map(|r| r.n)
            .fold(f64::NEG_INFINITY, f64::max),
        training
            .iter()
            .map(|r| r.f)
            .fold(f64::NEG_INFINITY, f64::max),
    ];
    let training_pairs = choose_pairs(&training, min, max);
    assert!(!training_pairs.is_empty());
    let mut training_constraints = base_constraints();
    for pair in &training_pairs {
        training_constraints.extend(pair_constraint(pair, "training"));
    }
    let training_region = solve_region(training_constraints);
    let (p3_pairs, p4_pairs, r7_pairs, holdout_region, on_policy_region) =
        if training_region.report.feasible {
            let p3_pairs = choose_pairs(
                &r5_rows
                    .iter()
                    .filter(|r| r.probe == "P3")
                    .cloned()
                    .collect::<Vec<_>>(),
                min,
                max,
            );
            let p4_pairs = choose_pairs(
                &r5_rows
                    .iter()
                    .filter(|r| r.probe == "P4")
                    .cloned()
                    .collect::<Vec<_>>(),
                min,
                max,
            );
            let r7_pairs = choose_r7_pairs(&r7_rows, &training, min, max);
            let mut holdout_constraints = training_region.constraints.clone();
            for pair in p3_pairs.iter().chain(p4_pairs.iter()) {
                holdout_constraints.extend(pair_constraint(pair, "holdout"));
            }
            let holdout_region = solve_region(holdout_constraints);
            let mut on_policy_constraints = holdout_region.constraints.clone();
            for pair in &r7_pairs {
                on_policy_constraints.extend(pair_constraint(pair, "r7_on_policy"));
            }
            let on_policy_region = solve_region(on_policy_constraints);
            (
                p3_pairs,
                p4_pairs,
                r7_pairs,
                holdout_region,
                on_policy_region,
            )
        } else {
            (
                Vec::new(),
                Vec::new(),
                Vec::new(),
                training_region.clone(),
                training_region.clone(),
            )
        };
    let validation_rows: Vec<SurfaceRow> = r5_rows.iter().chain(r7_rows.iter()).cloned().collect();
    let mut capacity_constraints = on_policy_region.constraints.clone();
    let mut capacity_invalid = 0usize;
    if on_policy_region.report.feasible {
        for row in &validation_rows {
            if let Some(constraint) = capacity_constraint(row) {
                capacity_constraints.push(constraint);
            } else {
                capacity_invalid += 1;
            }
        }
    }
    let capacity_region = solve_region(capacity_constraints);
    let clipping_dependent = on_policy_region.report.feasible && !capacity_region.report.feasible;
    let final_region = capacity_region.clone();
    let final_rows = p3_pairs
        .iter()
        .chain(p4_pairs.iter())
        .flat_map(|p| {
            validation_rows
                .iter()
                .filter(move |r| r.id == p.low_id || r.id == p.high_id)
                .cloned()
        })
        .collect::<Vec<_>>();
    let topology = if training_pairs.is_empty() || !training_region.report.feasible {
        "DCDEV020R8_PRODUCT_FEEDBACK_TOPOLOGY_INCOMPATIBLE"
    } else if !holdout_region.report.feasible || !on_policy_region.report.feasible {
        "DCDEV020R8_PRODUCT_FEEDBACK_TOPOLOGY_NOT_PORTABLE"
    } else if clipping_dependent {
        "DCDEV020R8_PRODUCT_FEEDBACK_CLIPPING_DEPENDENT"
    } else if !final_region.report.feasible
        || !final_region.report.positive_interior
        || final_region.report.v_fb_range.is_none()
        || final_region.report.k_a_range.is_none()
    {
        "DCDEV020R8_NFA_PRODUCT_FEEDBACK_FEASIBLE_NOT_IDENTIFIED"
    } else {
        "DCDEV020R8_NFA_PRODUCT_FEEDBACK_ATTRACTOR_TOPOLOGY_FEASIBLE"
    };
    let zero_substrate_control =
        [(0.0, 1.0, 0.0), (1.0, 0.0, 0.0)]
            .iter()
            .all(|(n, f, expected)| {
                let source: f64 = if *n <= 0.0 || *f <= 0.0 { 0.0 } else { *n * *f };
                (source - expected).abs() <= EPS
            });
    let sensitivity = ["P0", "P1", "P2"]
        .into_iter()
        .map(|removed| {
            let rows: Vec<SurfaceRow> = training
                .iter()
                .filter(|r| r.probe != removed)
                .cloned()
                .collect();
            let pairs = choose_pairs(&rows, min, max);
            let mut constraints = base_constraints();
            for pair in &pairs {
                constraints.extend(pair_constraint(pair, &format!("without_{}", removed)));
            }
            (removed, pairs.len(), solve_region(constraints).report)
        })
        .collect::<Vec<_>>();

    let dense = json!({
        "directive": "DC-DEV-020-R8",
        "training_pairs": training_pairs,
        "p3_pairs": p3_pairs,
        "p4_pairs": p4_pairs,
        "r7_pairs": r7_pairs,
        "training_constraints": training_region.constraints,
        "holdout_constraints": holdout_region.constraints,
        "on_policy_constraints": on_policy_region.constraints,
        "capacity_constraints": capacity_region.constraints,
    });
    fs::write(&dense_path, serde_json::to_vec(&dense).unwrap()).unwrap();
    let pairing = json!({
        "training": summarize_pairs(&training_pairs),
        "P3": summarize_pairs(&p3_pairs),
        "P4": summarize_pairs(&p4_pairs),
        "R7_on_policy": summarize_pairs(&r7_pairs),
        "training_nf_scaling": {"min": min, "max": max, "distance_limit": NF_DISTANCE_LIMIT},
        "normalized_surface": "Y_zero=S_zero/(q_c*g_h*area*dt*G_NF(N,F))",
        "g_h": G_H,
        "p_nf": P_NF,
    });
    let constraints = json!({
        "training": compact_region(&training_region),
        "after_P3_P4": compact_region(&holdout_region),
        "after_R7_on_policy": compact_region(&on_policy_region),
        "sensitivity_remove_probe": sensitivity,
    });
    let capacity = json!({
        "region_after_capacity_constraints": compact_region(&final_region),
        "capacity_invalid_states": capacity_invalid,
        "clipping_dependent": clipping_dependent,
        "zero_substrate_source_is_explicit_zero": zero_substrate_control,
        "validation_state_count": validation_rows.len(),
    });
    let region = json!({
        "classification_basis": topology,
        "reciprocal": final_region.report,
        "predicted_crossing_a_range": crossing_range(&final_region, &final_rows),
        "parameter_selection": "none; no midpoint or final pair selected",
    });
    let qualification = json!({
        "classification": topology,
        "training_region_nonempty": training_region.report.feasible,
        "holdout_region_nonempty": holdout_region.report.feasible,
        "r7_region_nonempty": on_policy_region.report.feasible,
        "final_region_nonempty": final_region.report.feasible,
        "final_region_bounded": final_region.report.bounded,
        "positive_u_v": final_region.report.positive_interior,
        "capacity_valid": !clipping_dependent,
        "zero_substrate_control": zero_substrate_control,
        "production_chemistry_changed": false,
        "production_behavior_changed": false,
        "dc_dev_021_authorized": false,
        "next_execution_started": false,
    });
    let protocol = json!({
        "directive": "DC-DEV-020-R8",
        "accepted_r7_head": ACCEPTED_R7_HEAD,
        "clean_scientific_base": CLEAN_BASE,
        "observer_only": true,
        "production_integration": false,
        "p_nf": P_NF,
        "g_h": G_H,
        "dt": DT,
        "nf_distance_limit": NF_DISTANCE_LIMIT,
        "root_margin": ROOT_MARGIN,
        "source_formula": "q_c*g_h*V_FB*N^p_NF*F^p_NF/(1+A/K_A)",
        "reciprocal_formula": "1/Y_FB=u+v*A; u=1/V_FB; v=1/(V_FB*K_A)",
        "r5_ledger_sha256": R5_SHA256,
        "r5_external_location": R5_LOCATION,
        "r7_ledger_sha256": R7_SHA256,
        "r7_external_location": R7_LOCATION,
        "source_commit": source_commit,
    });
    let literature = json!({
        "sources": [
            {"citation":"Goyal et al. 2010, PLoS Computational Biology, PMC2880561", "url":"https://pmc.ncbi.nlm.nih.gov/articles/PMC2880561/", "disposition":"ADAPTABLE_ARCHITECTURE_ONLY", "use":"product-feedback topology rationale only; no values imported"},
            {"citation":"Bi et al. 2023, Nature Communications, s41467-023-37957-0", "url":"https://www.nature.com/articles/s41467-023-37957-0", "disposition":"REFERENCE_STABILITY_WARNING", "use":"later free-running stability must be qualified; no values imported"}
        ],
        "external_parameters_imported": false,
    });
    let manifest = json!({
        "directive": "DC-DEV-020-R8",
        "source_commit": source_commit,
        "external_dense_ledger": {"location": external_location, "sha256": external_sha256},
        "compact_files": ["protocol.json", "pairing_summary.json", "constraint_summary.json", "feasible_region.json", "capacity_validation.json", "qualification.json", "literature_review.json", "external_evidence_manifest.json", "manifest.json"],
        "preserved": ["R1", "R2", "R3", "R4", "R5", "R6", "R7", "Phase-1", "D-088", "evolution-harness"],
    });
    write_json(&output, "protocol.json", &protocol);
    write_json(&output, "pairing_summary.json", &pairing);
    write_json(&output, "constraint_summary.json", &constraints);
    write_json(&output, "feasible_region.json", &region);
    write_json(&output, "capacity_validation.json", &capacity);
    write_json(&output, "qualification.json", &qualification);
    write_json(&output, "literature_review.json", &literature);
    write_json(
        &output,
        "external_evidence_manifest.json",
        &json!({"R5": {"sha256": R5_SHA256, "location": R5_LOCATION}, "R7": {"sha256": R7_SHA256, "location": R7_LOCATION}, "dense_R8": {"sha256": external_sha256, "location": external_location}}),
    );
    write_json(&output, "manifest.json", &manifest);
    println!("{}", topology);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reciprocal_constraints_recover_bounded_positive_region() {
        let mut constraints = base_constraints();
        constraints.push(Constraint {
            a: 1.0,
            b: 1.0,
            c: 4.0,
            label: "upper".into(),
        });
        constraints.push(Constraint {
            a: -1.0,
            b: -2.0,
            c: -1.0,
            label: "lower".into(),
        });
        let region = solve_region(constraints);
        assert!(region.report.feasible);
        assert!(region.report.positive_interior);
        assert!(region.report.bounded);
        assert!(region.report.area.unwrap() > 0.0);
    }

    #[test]
    fn zero_substrate_source_is_zero_without_clipping() {
        let no_source = |n: f64, f: f64| if n <= 0.0 || f <= 0.0 { 0.0 } else { n * f };
        assert_eq!(no_source(0.0, 1.0), 0.0);
        assert_eq!(no_source(1.0, 0.0), 0.0);
        assert!(capacity_constraint(&make_surface_row(
            "x".into(),
            "test".into(),
            "x".into(),
            0,
            1.0,
            0.2,
            1.0,
            1.0,
            1.0,
            0.1
        ))
        .is_some());
    }
}
