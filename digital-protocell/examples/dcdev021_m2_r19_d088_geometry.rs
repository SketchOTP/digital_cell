//! R19 observer-only geometry-integrity audit for the D-088 fission basis.
//! No production geometry, mechanics, topology, or fission code is changed.

use chemistry_core::material_mesh::{MaterialMesh, MeshEdge};
use chemistry_core::mesh_fission::{topology_step, try_local_fission, FissionParams};
use chemistry_core::mesh_growth::{growth_step, GrowthParams};
use chemistry_core::mesh_mechanics::{mechanics_step, remesh, MechParams};
use chemistry_core::mesh_reactions::{reactions_step, ReactionParams};
use chemistry_core::mesh_topology::TopologyLedger;
use chemistry_core::mesh_transport::{transport_step, TransportParams};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::PathBuf;

const STEPS: usize = 4_000;
const EPS: f64 = f64::EPSILON;

#[derive(Debug, Clone)]
struct Geometry { simple: bool, intersections: Vec<[usize; 2]>, min_nonadjacent_distance: f64 }

fn scale(points: &[[f64; 2]]) -> f64 {
    points.iter().flat_map(|p| p).map(|v| v.abs()).fold(1.0, f64::max)
}
fn orient(a: [f64;2], b: [f64;2], c: [f64;2]) -> f64 { (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0]) }
fn on_segment(a: [f64;2], b: [f64;2], p: [f64;2], tol: f64) -> bool {
    orient(a,b,p).abs() <= tol && p[0] >= a[0].min(b[0])-tol && p[0] <= a[0].max(b[0])+tol && p[1] >= a[1].min(b[1])-tol && p[1] <= a[1].max(b[1])+tol
}
fn segment_intersects(a: [f64;2], b: [f64;2], c: [f64;2], d: [f64;2], tol: f64) -> bool {
    let o1=orient(a,b,c); let o2=orient(a,b,d); let o3=orient(c,d,a); let o4=orient(c,d,b);
    ((o1 > tol && o2 < -tol) || (o1 < -tol && o2 > tol)) && ((o3 > tol && o4 < -tol) || (o3 < -tol && o4 > tol))
        || on_segment(a,b,c,tol) || on_segment(a,b,d,tol) || on_segment(c,d,a,tol) || on_segment(c,d,b,tol)
}
fn point_segment_distance(p:[f64;2], a:[f64;2], b:[f64;2]) -> f64 {
    let ab=[b[0]-a[0],b[1]-a[1]]; let ap=[p[0]-a[0],p[1]-a[1]]; let den=ab[0]*ab[0]+ab[1]*ab[1];
    let t=(if den>0.0 {(ap[0]*ab[0]+ap[1]*ab[1])/den} else {0.0}).clamp(0.0,1.0); let q=[a[0]+t*ab[0],a[1]+t*ab[1]]; (p[0]-q[0]).hypot(p[1]-q[1])
}
fn segment_distance(a:[f64;2],b:[f64;2],c:[f64;2],d:[f64;2]) -> f64 {
    if segment_intersects(a,b,c,d,0.0) { 0.0 } else { point_segment_distance(a,c,d).min(point_segment_distance(b,c,d)).min(point_segment_distance(c,a,b)).min(point_segment_distance(d,a,b)) }
}
fn inspect(points: &[[f64;2]]) -> Geometry {
    let n=points.len(); let tol=128.0*EPS*(1.0+scale(points)).powi(2); let mut pairs=Vec::new(); let mut min=f64::INFINITY;
    for i in 0..n { for j in (i+1)..n {
        if j==i+1 || (i==0 && j+1==n) { continue; }
        min=min.min(segment_distance(points[i],points[(i+1)%n],points[j],points[(j+1)%n]));
        if segment_intersects(points[i],points[(i+1)%n],points[j],points[(j+1)%n],tol) { pairs.push([i,j]); }
    }}
    Geometry { simple:pairs.is_empty(), intersections:pairs, min_nonadjacent_distance:min }
}
fn metrics(points:&[[f64;2]]) -> Value {
    let n=points.len(); let mut signed=0.0; let mut perimeter=0.0;
    for i in 0..n { let q=points[(i+1)%n]; signed += points[i][0]*q[1]-q[0]*points[i][1]; perimeter += (q[0]-points[i][0]).hypot(q[1]-points[i][1]); }
    let signed=0.5*signed; let area=signed.abs(); let sf=perimeter*perimeter/(4.0*std::f64::consts::PI*area.max(1e-300));
    let g=inspect(points);
    json!({"perimeter":perimeter,"signed_area":signed,"absolute_area":area,"shape_factor":sf,"isoperimetric_quotient":1.0/sf,"polygon_simple":g.simple,"intersection_count":g.intersections.len(),"intersecting_edge_pairs":g.intersections,"minimum_nonadjacent_segment_distance":g.min_nonadjacent_distance})
}
fn mesh_metrics(mesh:&MaterialMesh) -> Value { metrics(&mesh.vertices) }
fn perturb(mesh:&mut MaterialMesh, kind:&str, mag:f64) {
    match kind { "rotate"=>{let c=mesh.centroid();let(s,co)=mag.sin_cos();for p in &mut mesh.vertices{let(x,y)=(p[0]-c[0],p[1]-c[1]);p[0]=c[0]+co*x-s*y;p[1]=c[1]+s*x+co*y;}},
        "vertex"=>for(i,p)in mesh.vertices.iter_mut().enumerate(){let f=(((i as f64+1.0)*12.9898).sin()*43758.5453).fract();p[0]+=mag*(f-0.5);p[1]+=mag*((f*7.13).fract()-0.5);},
        "c"=>mesh.interior.c=(mesh.interior.c*(1.0+mag)).max(0.0), "a"=>mesh.interior.a=(mesh.interior.a*(1.0+mag)).max(0.0), "l"=>mesh.free_l=(mesh.free_l*(1.0+mag)).max(0.0), "env"=>{mesh.exterior.n=(mesh.exterior.n*(1.0+mag)).max(0.0);mesh.exterior.f=(mesh.exterior.f*(1.0+mag)).max(0.0);}, _=>{} }
}
fn fixture(seed:u64, kind:&str, mag:f64)->MaterialMesh { let mut m=chemistry_core::mesh_population::MeshPopulation::seed_one(14.0,seed,2.2).individuals.remove(0).mesh; perturb(&mut m,kind,mag); perturb(&mut m,"vertex",0.35); let c=m.centroid(); for p in &mut m.vertices{p[0]=c[0]+(p[0]-c[0])*1.25;} m }
fn step(mesh:&mut MaterialMesh, step:usize, mech:&MechParams, react:&ReactionParams, tr:&TransportParams, growth:&GrowthParams, fission:&FissionParams)->TopologyLedger { let _=transport_step(mesh,tr,mech.dt);let _=reactions_step(mesh,react,mech.dt,true,true);let _=growth_step(mesh,react,growth,mech.dt);let _=mechanics_step(mesh,mech);remesh(mesh);if step%10==0{topology_step(mesh,fission)}else{TopologyLedger::default()} }
fn trace(mut mesh:MaterialMesh, campaign: &str)->Value {
    let mech=MechParams::default();let react=ReactionParams::default();let tr=TransportParams::default();let growth=GrowthParams{y_g:0.9,enable_growth:true};let fission=FissionParams::default();let birth=mesh.total_structural_mass();let mut rows=Vec::new();let mut event=None;
    for s in 0..STEPS { if !mesh.can_advance_physics(){break;} let ledger=step(&mut mesh,s,&mech,&react,&tr,&growth,&fission); let mass=mesh.total_structural_mass(); let mass_gate=mass>=1.35*birth; let attempt=mass_gate && s%25==0; let pinch=chemistry_core::mesh_topology::find_local_pinch(&mesh,&fission.topo).map(|(i,j)|json!({"i":i,"j":j})); let shadow=attempt && try_local_fission(&mesh,&fission).is_some(); rows.push(json!({"step":s+1,"mass_gate":mass_gate,"attempt_tick":attempt,"structural_mass":mass,"birth_mass":birth,"absolute_A":mesh.interior.a.max(0.0)*mesh.area().max(0.0),"topology_ledger":{"tension_ruptures":ledger.tension_ruptures,"local_rebonds":ledger.local_rebonds,"cross_bonds":ledger.cross_bonds},"geometry":mesh_metrics(&mesh),"pinch_candidate":pinch,"try_local_fission":if shadow{"SUCCESS"}else{"FAIL"}})); if attempt { if let Some((a,b,e))=try_local_fission(&mesh,&fission){event=Some(json!({"step":s+1,"pinch":e.pinch,"daughter_a_geometry":mesh_metrics(&a),"daughter_b_geometry":mesh_metrics(&b),"daughter_a_vertices":a.vertices,"daughter_b_vertices":b.vertices}));break;} }}
    json!({"campaign":campaign,"birth_mass":birth,"physical_fission":event.is_some(),"event":event,"readiness_trace":rows})
}
fn synthetic()->Value { let convex=vec![[0.,0.],[2.,0.],[2.,2.],[0.,2.]];let concave=vec![[0.,0.],[3.,0.],[3.,3.],[1.5,1.],[0.,3.]];let bow=vec![[0.,0.],[2.,2.],[0.,2.],[2.,0.]];let near=vec![[0.,0.],[4.,0.],[4.,4.],[0.01,4.],[0.01,0.01],[0.,0.01]];json!({"convex":{"geometry":metrics(&convex),"expected_simple":true},"concave":{"geometry":metrics(&concave),"expected_simple":true},"bow_tie":{"geometry":metrics(&bow),"expected_simple":false},"near_contact":{"geometry":metrics(&near),"expected_simple":true}})}
fn main(){let args:Vec<String>=env::args().collect();let mut out=PathBuf::from("/tmp/dcdev021_m2_r19_d088_geometry.json");for i in 1..args.len(){if args[i]=="--output"{out=PathBuf::from(&args[i+1]);}}
    let kinds=[("rotate",0.3), ("vertex",0.12),("c",0.08),("a",0.08),("env",0.1),("l",0.1),("rotate",-0.5),("vertex",-0.1),("c",-0.05),("env",-0.08)]; let mut campaign=Vec::new();for(i,(k,m))in kinds.iter().enumerate(){campaign.push(trace(fixture((i+1)as u64,k,*m),&format!("seed_{}_{}_{}",i+1,k,m)));}
    let primary=trace(fixture(1,"rotate",0.3),"primary_seed_1");
    let tests=synthetic(); let value=json!({"directive":"DC-DEV-021-M2-R19-D088-GEOMETRY-VALIDITY-AND-FISSION-QUALIFICATION-INTEGRITY-AUDIT-001","observer_only":true,"shape_metric_definition":{"perimeter":"sum physical edge lengths","signed_area":"shoelace","absolute_area":"abs(signed_area)","shape_factor":"P^2/(4*pi*A)","isoperimetric_quotient":"4*pi*A/P^2"},"synthetic_tests":tests,"primary":primary,"campaign":campaign,"source_contract":{"physical_runtime_valid_enforces_simple_polygon":false,"closed_intact_enforces_simple_polygon":false,"fission_requires_simple_polygon":false,"classification":"SIMPLE_POLYGON_REQUIRED_NOT_ENFORCED"}});
    fs::write(out,serde_json::to_vec_pretty(&value).unwrap()).unwrap();
}
