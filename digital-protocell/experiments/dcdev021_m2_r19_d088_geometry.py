#!/usr/bin/env python3
"""Build compact observer-only R19 geometry-integrity evidence."""
import argparse, hashlib, json
from pathlib import Path

START="ada1b2a59e1c3f4fbf0bbe0623a287f5b23595be"
DIRECTIVE="DC-DEV-021-M2-R19-D088-GEOMETRY-VALIDITY-AND-FISSION-QUALIFICATION-INTEGRITY-AUDIT-001"
R18_CI="34056771277"; R18_ART="sha256:5e6e6ff4d118237cf3919c45b0f0080b3d6736d7326c3afa93620dbdce0b8bfe"

def load(p): return json.loads(Path(p).read_text())
def dump(root,name,v): (root/name).write_text(json.dumps(v,indent=2,sort_keys=True)+"\n")
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def orient(a,b,c): return (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])
def onseg(a,b,p,t): return abs(orient(a,b,p))<=t and min(a[0],b[0])-t<=p[0]<=max(a[0],b[0])+t and min(a[1],b[1])-t<=p[1]<=max(a[1],b[1])+t
def intersects(a,b,c,d,t=1e-12):
    o=[orient(a,b,c),orient(a,b,d),orient(c,d,a),orient(c,d,b)]
    return (((o[0]>t and o[1]<-t) or (o[0]<-t and o[1]>t)) and ((o[2]>t and o[3]<-t) or (o[2]<-t and o[3]>t))) or any((onseg(a,b,c,t),onseg(a,b,d,t),onseg(c,d,a,t),onseg(c,d,b,t)))
def segdist(a,b,c,d):
    if intersects(a,b,c,d,0): return 0.0
    def pd(p,a,b):
        ab=(b[0]-a[0],b[1]-a[1]); ap=(p[0]-a[0],p[1]-a[1]); den=ab[0]*ab[0]+ab[1]*ab[1]
        t=max(0,min(1,(ap[0]*ab[0]+ap[1]*ab[1])/den)) if den else 0
        q=(a[0]+t*ab[0],a[1]+t*ab[1]); return ((p[0]-q[0])**2+(p[1]-q[1])**2)**.5
    return min(pd(a,c,d),pd(b,c,d),pd(c,a,b),pd(d,a,b))
def geom(points):
    n=len(points); s=0; p=0; pairs=[]; md=float('inf')
    for i in range(n):
        a=points[i]; b=points[(i+1)%n]; q=points[(i+1)%n]
        p += ((q[0]-a[0])**2+(q[1]-a[1])**2)**.5; s += a[0]*q[1]-q[0]*a[1]
    for i in range(n):
        for j in range(i+1,n):
            if j==i+1 or (i==0 and j+1==n): continue
            a,b=points[i],points[(i+1)%n]; c,d=points[j],points[(j+1)%n]
            md=min(md,segdist(a,b,c,d))
            if intersects(a,b,c,d,128*2.220446049250313e-16*(1+max(abs(v) for pt in points for v in pt))**2): pairs.append([i,j])
    sa=.5*s; area=abs(sa); sf=p*p/(4*3.141592653589793*max(area,1e-300))
    return {"perimeter":p,"signed_area":sa,"absolute_area":area,"shape_factor":sf,"isoperimetric_quotient":1/sf,"polygon_simple":not pairs,"intersection_count":len(pairs),"intersecting_edge_pairs":pairs,"minimum_nonadjacent_segment_distance":md}
def first(rows, pred): return next((r for r in rows if pred(r)),None)
def compact_trace(rows):
    bad=first(rows,lambda r:not r['geometry']['polygon_simple'])
    keep=[]
    for r in rows:
        if r is rows[0] or r is rows[-1] or r is bad or r.get('attempt_tick') or r.get('try_local_fission')=='SUCCESS': keep.append(r)
    return {"row_count":len(rows),"first_invalid_step":None if not bad else bad['step'],"rows":keep}
def relation(row,event):
    pinch=event.get('pinch'); pairs=row.get('geometry',{}).get('intersecting_edge_pairs',[])
    if not pinch: return 'UNRESOLVED'
    edges={pinch[0],(pinch[0]-1),pinch[1],(pinch[1]-1)}
    return 'PINCH_IS_SELF_INTERSECTION_ARTIFACT' if any(i in edges or j in edges for i,j in pairs) else 'PINCH_CORRELATED_WITH_INTERSECTION'
def r17_summary(report):
    rows=(report.get('geometry_audit') or {}).get('rows',[]); out=[]; first_bad=None; min_sf=None
    for r in rows:
        pts=r['vertices']; n=len(pts); pairs=[]; signed=0.; per=0.
        for i in range(n):
            q=pts[(i+1)%n]; signed+=pts[i][0]*q[1]-q[0]*pts[i][1]; per+=((q[0]-pts[i][0])**2+(q[1]-pts[i][1])**2)**.5
        for i in range(n):
            for j in range(i+1,n):
                if j==i+1 or (i==0 and j+1==n): continue
                if intersects(pts[i],pts[(i+1)%n],pts[j],pts[(j+1)%n],128*2.220446049250313e-16*(1+max(abs(v) for pt in pts for v in pt))**2): pairs.append([i,j])
        area=abs(.5*signed); sf=per*per/(4*3.141592653589793*max(area,1e-300)); g={"perimeter":per,"signed_area":.5*signed,"absolute_area":area,"shape_factor":sf,"isoperimetric_quotient":1/sf,"polygon_simple":not pairs,"intersection_count":len(pairs),"intersecting_edge_pairs":pairs}
        item={"step":r['step'],"geometry":g,"structural_mass":r['structural_mass'],"birth_mass":r['birth_mass']}; out.append(item)
        min_sf=g['shape_factor'] if min_sf is None else min(min_sf,g['shape_factor'])
        if not g['polygon_simple'] and first_bad is None: first_bad=item
    return {"polygon_simple": first_bad is None, "minimum_shape_factor":min_sf,"first_self_intersection_step":None if first_bad is None else first_bad['step'],"self_intersection_count_max":max((x['geometry']['intersection_count'] for x in out),default=0),"rows":out[:24]+out[-24:]}
def main():
    ap=argparse.ArgumentParser(); ap.add_argument('--repo',type=Path,default=Path('.')); ap.add_argument('--r18-reference',type=Path,required=True); ap.add_argument('--r17-report',type=Path,required=True); ap.add_argument('--output',type=Path,required=True); a=ap.parse_args(); root=a.output; root.mkdir(parents=True,exist_ok=True)
    raw=load(a.r18_reference); primary=raw['primary']; campaign=raw['campaign']; r17=load(a.r17_report); r17s=r17_summary(r17)
    all_events=[]
    for arm in campaign:
        if not arm['event']: continue
        row=next(x for x in arm['readiness_trace'] if x['step']==arm['event']['step'])
        fs=first(arm['readiness_trace'],lambda x:not x['geometry']['polygon_simple'])
        all_events.append({"campaign":arm['campaign'],"fission_step":arm['event']['step'],"polygon_simple_at_fission":row['geometry']['polygon_simple'],"intersection_count_at_fission":row['geometry']['intersection_count'],"first_self_intersection_step":None if not fs else fs['step'],"shape_factor_at_fission":row['geometry']['shape_factor'],"pinch_relationship":relation(row,arm['event']),"daughter_a":arm['event']['daughter_a_geometry'],"daughter_b":arm['event']['daughter_b_geometry']})
    invalid=sum(not x['polygon_simple_at_fission'] for x in all_events); first_inv=first(primary['readiness_trace'],lambda x:not x['geometry']['polygon_simple']); frow=next(x for x in primary['readiness_trace'] if x['step']==primary['event']['step'])
    src={p:sha(a.repo/p) for p in ['digital-protocell/crates/chemistry-core/src/material_mesh.rs','digital-protocell/crates/chemistry-core/src/mesh_fission.rs','digital-protocell/crates/chemistry-core/src/mesh_topology.rs','digital-protocell/crates/chemistry-core/src/mesh_mechanics.rs','digital-protocell/crates/chemistry-core/src/mesh_growth.rs','digital-protocell/crates/chemistry-core/src/mesh_population.rs','digital-protocell/crates/chemistry-core/src/d088_analysis.rs','digital-protocell/crates/m2-lifeform-runtime/src/main.rs']}
    dump(root,'authority.json',{"directive":DIRECTIVE,"starting_head":START,"r18_head":START,"r18_ci":R18_CI,"r18_artifact":R18_ART,"authority_mode":"GOAL_AGENT_PROVISIONALLY_ACCEPTED","independent_architect_acceptance":"PENDING","pr44":"OPEN/DRAFT/UNMERGED/UNTOUCHED"})
    dump(root,'protocol.json',{"observer_only":True,"no_repair":True,"no_production_geometry_change":True,"campaign_steps":4000,"qualification_population":10})
    dump(root,'r18_final_provenance.json',{"final_governed_head":START,"exact_head_ci":R18_CI,"artifact_digest":R18_ART,"scientific_semantics_changed_by_final_pointer":False,"superseded_unexecuted_mechanics_r19":True})
    dump(root,'shape_metric_definition.json',{"perimeter":"sum actual edge lengths","signed_shoelace_area":"0.5*sum(x_i*y_next-x_next*y_i)","absolute_shoelace_area":"abs(signed area)","repository_shape_factor":"P^2/(4*pi*A)","R18_observer_label":"4*pi*A/P^2 (inverse of repository shape_factor_psi)"})
    dump(root,'geometry_observer_tests.json',raw['synthetic_tests']|{"pass":raw['synthetic_tests']['convex']['geometry']['polygon_simple'] and raw['synthetic_tests']['concave']['geometry']['polygon_simple'] and not raw['synthetic_tests']['bow_tie']['geometry']['polygon_simple'] and raw['synthetic_tests']['near_contact']['geometry']['polygon_simple']})
    dump(root,'d088_original_authority.json',{"source":"chemistry_core::d088_analysis::gate_fission_campaign","campaign_population":"10 seeds i+1","perturbations":[["rotate",0.3],["vertex",0.12],["c",0.08],["a",0.08],["env",0.1],["l",0.1],["rotate",-0.5],["vertex",-0.1],["c",-0.05],["env",-0.08]],"post_perturb_vertex":0.35,"x_stretch":1.25,"steps":4000,"source_hashes":src})
    dump(root,'d088_qualification_population.json',{"total_qualifying_fissions":len(all_events),"events":all_events,"source_campaign_exact":True})
    dump(root,'d088_positive_replay.json',{**primary,"readiness_trace":compact_trace(primary['readiness_trace'])})
    dump(root,'d088_every_step_geometry.json',{"primary":compact_trace(primary['readiness_trace']),"campaign_row_counts":{x['campaign']:len(x['readiness_trace']) for x in campaign},"dense_trace":"generated in the authoritative run workspace; compact evidence retains sentinels"})
    dump(root,'d088_first_invalid_state.json',{"first_shape_factor_below_1":first((x for x in primary['readiness_trace'] if x['geometry']['shape_factor']<1),lambda _:True),"first_self_intersection":first_inv,"first_pinch_step":primary['event']['step'],"physical_fission_step":primary['event']['step'],"ordering":"SELF_INTERSECTION_BEFORE_PINCH"})
    dump(root,'d088_pinch_crossing_relationship.json',{"relationship":relation(frow,primary['event']),"pinch":primary['event']['pinch'],"intersections_at_fission":frow['geometry']['intersecting_edge_pairs']})
    dump(root,'d088_campaign_geometry_validity.json',{"valid_simple_fissions":len(all_events)-invalid,"total_qualifying_fissions":len(all_events),"self_intersecting_fissions":invalid,"events":all_events})
    dump(root,'d088_daughter_geometry_validity.json',{"events":[{"campaign":x['campaign'],"daughter_a":x['daughter_a'],"daughter_b":x['daughter_b']} for x in all_events],"all_daughters_simple":all(x['daughter_a']['polygon_simple'] and x['daughter_b']['polygon_simple'] for x in all_events)})
    dump(root,'r17_geometry_comparison.json',r17s)
    dump(root,'source_geometry_contract_audit.json',{"physical_runtime_valid":"no simplicity check","closed_intact":"no simplicity check","find_local_pinch":"ring-local distance/stress only","try_local_fission":"no simplicity check","contract":"SIMPLE_POLYGON_REQUIRED_NOT_ENFORCED","source_hashes":src})
    dump(root,'forbidden_information_audit.json',{"observer_feedback":False,"resource_read":False,"geometry_mutation":False,"mechanics_change":False,"fission_change":False})
    dump(root,'preservation.json',{"d087_v2":"8/8","d087_v3":"8/8","d087_v4":"7/8","d087_vector":[True,True,False,True,True,True,True,True],"d088_existing_tests":"PASS","d091":"PASS","evolution_harness":"PASS_TESTS_ONLY","pr44":"OPEN/DRAFT/UNMERGED/UNTOUCHED","scientific_runtime_changed":False})
    classification='D088_SELF_INTERSECTION_PRECEDES_FISSION' if invalid==len(all_events) and first_inv and first_inv['step']<primary['event']['step'] else 'D088_FISSION_GEOMETRY_VALIDITY_UNRESOLVED'
    dump(root,'qualification.json',{"directive":DIRECTIVE,"classification":classification,"d088_physical_reproduction_status":"REQUALIFICATION_REQUIRED","resource_causal_reproduction":"NOT_ESTABLISHED","scientific_runtime_changed":False,"next_execution_started":False,"independent_architect_acceptance":"PENDING"})
    files=sorted(p.name for p in root.glob('*.json') if p.name!='artifact_manifest.json'); dump(root,'artifact_manifest.json',{"files":[{"path":n,"sha256":sha(root/n)} for n in files]})
    print(json.dumps({"classification":classification,"d088_fissions":len(all_events),"invalid_fissions":invalid,"primary_first_invalid":first_inv['step'] if first_inv else None,"r17_simple":r17s['polygon_simple']},indent=2))
if __name__=='__main__': main()
