#!/usr/bin/env python3
"""R13 independent reconstruction: no Rust summary determines attribution.

The parameter-cut kernel below is an independent Python transcription of the
unchanged production equations. Every output pool and A/W ledger is compared;
R8's historical phase predicate and the prospectively sealed dominance rule
are recomputed from raw values. Only discarded-clone evidence is inspected.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import math
import subprocess
from pathlib import Path

BASE = "d944d4fabc46333203bfd6e441d600b9717c349d"
DIRECTIVE = "DC-M4-R13-GEOMETRY-TO-POLARITY-INTRA-OPERATOR-CAUSAL-DECOMPOSITION-GATE-001"
PATHS = ["FULL", "REACTION_ONLY", "TRANSPORT_ONLY", "REACTION_THEN_TRANSPORT", "TRANSPORT_THEN_REACTION"]
COMPONENTS = ["REACTION_ONLY", "TRANSPORT_ONLY", "INTERACTION"]
CLASSES = ["G_TO_P_DAMPING_LOCAL_REACTION_MEDIATED_BY_GEOMETRIC_CONCENTRATION", "G_TO_P_DAMPING_LATERAL_TRANSPORT_DOMINATED", "G_TO_P_DAMPING_REACTION_TRANSPORT_INTERACTION_DOMINATED"]
ABS, REL = 1e-12, 1e-10

def load(path): return json.loads(Path(path).read_text())
def sha(path): return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def git(repo, *args): return subprocess.check_output(["git", *args], cwd=repo, text=True).strip()
def dump(root, name, value):
    path = root / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n")
def norm(v): return math.sqrt(sum(x*x for x in v))
def sub(a,b):
    assert len(a)==len(b)
    return [x-y for x,y in zip(a,b)]
def add(a,b):
    assert len(a)==len(b)
    return [x+y for x,y in zip(a,b)]
def mul(a,k): return [x*k for x in a]
def dot(a,b):
    assert len(a)==len(b)
    return sum(x*y for x,y in zip(a,b))
def close(a,b):
    assert math.isfinite(a) and math.isfinite(b)
    assert abs(a-b) <= ABS + REL*max(abs(a),abs(b)), (a,b)
def vclose(a,b):
    assert len(a)==len(b)
    for x,y in zip(a,b): close(x,y)
def dft(values,k):
    n=len(values); factor=1/n if k==0 or 2*k==n else 2/n
    return (factor*sum(v*math.cos(2*math.pi*k*i/n) for i,v in enumerate(values)), factor*sum(v*math.sin(2*math.pi*k*i/n) for i,v in enumerate(values)))
def lengths(vertices):
    return [math.hypot(vertices[(i+1)%len(vertices)][0]-v[0],vertices[(i+1)%len(vertices)][1]-v[1]) for i,v in enumerate(vertices)]
def area(vertices):
    return abs(sum(v[0]*vertices[(i+1)%len(vertices)][1]-vertices[(i+1)%len(vertices)][0]*v[1] for i,v in enumerate(vertices)))*0.5

def normal_basis(vertices):
    n=len(vertices); center=[sum(p[k] for p in vertices)/n for k in range(2)]
    tx=[x for _ in vertices for x in [1.0,0.0]]; ty=[x for _ in vertices for x in [0.0,1.0]]
    rot=[x for p in vertices for x in [-(p[1]-center[1]),p[0]-center[0]]]
    constraints=[]
    for v in [tx,ty,rot]:
        for previous in constraints: v=sub(v,mul(previous,dot(v,previous)))
        if norm(v)>1e-10: constraints.append(mul(v,1/norm(v)))
    normals=[]
    for p in vertices:
        v=sub(p,center); normals.append(mul(v,1/norm(v)))
    result=[]
    for k in range(n//2+1):
        phases=[("cos",0.0)] if k==0 or 2*k==n else [("cos",0.0),("sin",math.pi/2)]
        for name,shift in phases:
            v=[x*math.cos(2*math.pi*k*i/n+shift) for i,p in enumerate(normals) for x in p]
            for previous in constraints: v=sub(v,mul(previous,dot(v,previous)))
            if norm(v)>1e-10: result.append((k,name,mul(v,1/norm(v))))
    return result

def simple_polygon(vertices):
    def orient(a,b,c): return (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])
    n=len(vertices)
    for i in range(n):
        a,b=vertices[i],vertices[(i+1)%n]
        for j in range(i+1,n):
            if j==i or j==(i+1)%n or (j+1)%n==i: continue
            c,d=vertices[j],vertices[(j+1)%n]
            if orient(a,b,c)*orient(a,b,d)<0 and orient(c,d,a)*orient(c,d,b)<0: return False
    return True

def diffusion(amount, measures, coefficient, h):
    n=len(amount); fluxes=[]; outgoing=[0.0]*n
    for i in range(n):
        j=(i+1)%n
        flux=coefficient*(amount[i]/measures[i]-amount[j]/measures[j])/(0.5*(measures[i]+measures[j]))*h
        fluxes.append(flux)
        if flux>0: outgoing[i]+=flux
        else: outgoing[j]+=-flux
    scale=[a/o if o>a else 1.0 for a,o in zip(amount,outgoing)]
    result=list(amount); limited=[o>a for a,o in zip(amount,outgoing)]
    for i,flux in enumerate(fluxes):
        j=(i+1)%n
        flux*=scale[i] if flux>=0 else scale[j]
        result[i]-=flux; result[j]+=flux
    assert min(result)>=0
    return result,limited

def kernel(active,inactive,measures,path,available):
    """Four original h substeps; retain original diffusion-before-reaction."""
    active=list(active); inactive=list(inactive); spent=0.0; branch_masks=[]
    for _ in range(4):
        branch={"donor_active":[],"donor_inactive":[],"reaction_clip":[]}
        order=["T","R"]
        if path=="REACTION_ONLY": order=["R"]
        if path=="TRANSPORT_ONLY": order=["T"]
        if path=="REACTION_THEN_TRANSPORT": order=["R","T"]
        for op in order:
            if op=="T":
                active,la=diffusion(active,measures,0.1,0.0025)
                inactive,li=diffusion(inactive,measures,1.0,0.0025)
                branch["donor_active"]=la; branch["donor_inactive"]=li
            else:
                for i,length in enumerate(measures):
                    u,v=active[i]/length,inactive[i]/length
                    proposed=((0.01+u*u)*v-(0.1+0.01*u*u)*u)*length*0.0025
                    transfer=min(max(proposed,-active[i]),inactive[i])
                    branch["reaction_clip"].append("LOWER" if proposed < -active[i] else "UPPER" if proposed > inactive[i] else "NONE")
                    active[i]+=transfer; inactive[i]-=transfer
                    spent+=abs(transfer)*0.5
                assert spent<=available
        branch_masks.append(branch)
    return active,inactive,spent,branch_masks

def operator_graph():
    return [
        {"owner":"crates/regulatory-core/src/polarity_mass.rs::advance_with_load_bearing_strain", "stage":"clone persistent active/inactive amounts", "units":"polarity amount", "locality":"one edge"},
        {"stage":"four h=0.0025 substeps, each diffusion then reaction", "units":"time", "locality":"same ring"},
        {"stage":"donor-limited nearest-neighbor diffusion from amount/current-length differences divided by adjacent mean length", "units":"diffusion length^2/time; flux polarity amount", "locality":"two adjacent edges"},
        {"stage":"u=active/length, v=inactive/length; activation=(0.01+u^2)*v; deactivation=(0.1+0.01*u^2)*u", "units":"concentration; rate concentration/time", "locality":"one edge"},
        {"stage":"transfer=clip(rate*length*h,-active,inactive); active+=transfer; inactive-=transfer", "units":"polarity amount", "locality":"one edge"},
        {"stage":"A cost=abs(transfer)*0.5, W credit identical; finite available A checked per substep", "units":"A/W amount", "locality":"caller supplies finite cell budget"},
        {"stage":"positivity/finite checks then atomic commit after all substeps; error leaves original state intact", "units":"amount", "locality":"transaction"},
    ]

def source_audit(repo):
    paths=["crates/regulatory-core/src/polarity_mass.rs","crates/regulatory-core/src/polarity_actuation.rs","examples/dcfinal001_r5_v4_neck.rs"]
    for head in [BASE,"18cd5daff29267469db838acbba9bf9691e94ba6","00a7a26f1c5e23b8b9895de7ba1c71c51b2dbbc4"]:
        subprocess.check_call(["git","merge-base","--is-ancestor",head,"HEAD"],cwd=repo)
    for path in paths:
        historical=subprocess.check_output(["git","show",f"{BASE}:digital-protocell/{path}"],cwd=repo)
        assert (repo/path).read_bytes()==historical, path
    shared=repo/"examples/dcfinal001_r4_evolution.rs"
    historical=subprocess.check_output(["git","show",f"{BASE}:digital-protocell/examples/dcfinal001_r4_evolution.rs"],cwd=repo)
    assert shared.read_bytes().startswith(historical), "shared source must be append-only"
    paths += ["examples/dcfinal001_r4_evolution.rs","examples/dcm4_r13_intra_operator_causal_decomposition.rs","experiments/dcm4_r13_intra_operator_causal_decomposition_verify.py","experiments/fixtures/dcm4r13/r12_input_identities.json"]
    return {"files":{p:sha(repo/p) for p in paths}, "frozen_sources_byte_identical":True,"shared_source_append_only":True,"operator_graph":operator_graph()}

def validate_contract(raw):
    assert raw["directive"]==DIRECTIVE and raw["entry_head"]==BASE
    assert raw["entry_ci"]=="35060451335"
    assert raw["entry_r8"]=={"head":"18cd5daff29267469db838acbba9bf9691e94ba6","ci":"34865782025","artifact":"sha256:30b75d9389adda97bde60654d00caf8b86b80c4ff0998efc52d43764b9303073"}
    assert raw["entry_r9"]=={"head":"00a7a26f1c5e23b8b9895de7ba1c71c51b2dbbc4","ci":"34914837619","artifact":"sha256:d379b78bb56cfce3ca5ba9986e8007869d6961bd6679da0cdf3b2d971fff9bfe"}
    assert raw["entry_artifact"]=="sha256:5949df1110598142f6c32f3492a06f9d39516e18fa016dead6d4223dd2c3e432"
    assert raw["entry_binary"]=="cec057e369ce03bff0720b35245372d53e86e59d0af9fadf7b3c57f1f45fda64"
    assert raw["checkpoints"]==[3694,7389,11083] and raw["horizon"]==14778
    assert raw["normalized_amplitudes"]==[0.001,0.0005] and raw["signs"]==[1.0,-1.0]
    assert raw["paths"]==PATHS and raw["dt"]==0.01 and raw["integration_substeps"]==4
    assert raw["attribution"]["usable_minimum"]==24
    assert raw["attribution"]["outcome_selected_settings"] is False
    assert raw["attribution"]["candidate_order"]==COMPONENTS
    assert raw["controls"]["replay_absolute_tolerance"]==ABS
    assert raw["controls"]["replay_relative_tolerance"]==REL
    assert raw["controls"]["r8_symmetry_relative_limit"]==0.05
    assert raw["production_biology_modified"] is False and raw["observer_only"] is True
    for key in ["morphogenesis","reproduction","population_selection","reversal","final_integration"]: assert raw[key]=="NOT_REACHED"

def phase(delta,harmonic,activity):
    ac,ass=dft(activity,harmonic); rc,rs=dft(delta,harmonic)
    return (rc*ac+rs*ass)/max(math.hypot(ac,ass),1e-300)

def control(vectors,input_norm):
    """All vectors already concatenate all fixed modes for this state."""
    central=mul(sub(vectors[0][0],vectors[0][1]),0.5)
    half=mul(sub(vectors[1][0],vectors[1][1]),0.5)
    even=max(norm(mul(add(*vectors[i]),0.5))*(1 if i==0 else 2) for i in range(2))
    scale_error=norm(sub(central,mul(half,2)))
    replay=math.sqrt(len(central))*ABS+REL*max(input_norm,norm(central))
    return central,max(even,scale_error,replay),{"sign_even":even,"half_scale":scale_error,"replay":replay}

def attribution(vectors,phase_values,phase_errors,errors):
    full=vectors["FULL"]; fullnorm=norm(full)
    metrics={}; nominations=[]
    full_direction_identifiable=fullnorm>errors["FULL"]
    def projection_error(key):
        if not full_direction_identifiable: return None
        return errors[key]+norm(vectors[key])*2*errors["FULL"]/(fullnorm-errors["FULL"])
    for candidate in COMPONENTS:
        v=vectors[candidate]; vnorm=norm(v)
        projection=dot(v,full)/max(fullnorm,1e-300)
        residual=norm(sub(full,v)); others=[k for k in COMPONENTS if k!=candidate]
        direction_error=projection_error(candidate)
        lower=projection-direction_error if direction_error is not None else 0.0
        other_upper=sum(abs(dot(vectors[k],full)/max(fullnorm,1e-300))+(projection_error(k) or 0.0) for k in others)
        unique=all(residual+errors[candidate]+errors["FULL"] < norm(sub(full,vectors[k]))-errors[k]-errors["FULL"] for k in others)
        damp=phase_values[candidate] < -phase_errors[candidate]
        supported=full_direction_identifiable and projection>0 and lower>other_upper and unique and damp and phase_values["FULL"] < -phase_errors["FULL"]
        metrics[candidate]={"full_vector_correlation":dot(v,full)/max(vnorm*fullnorm,1e-300),"signed_projection":projection,"gain":projection/max(fullnorm,1e-300),"normalized_residual":residual/max(fullnorm,1e-300),"projection_error_including_full_direction":direction_error,"full_direction_identifiable":full_direction_identifiable,"projection_lower":lower,"other_absolute_projection_upper":other_upper,"uniquely_smallest_residual":unique,"r8_phase_indicator":phase_values[candidate],"phase_error_envelope":phase_errors[candidate],"damping_beyond_envelope":damp,"dominance_supported":supported}
        if supported: nominations.append(candidate)
    assert len(nominations)<=1
    return nominations[0] if nominations else None,metrics


def bind_boundary(snapshot,r12_digest):
    assert snapshot["input_boundary_digest"]==r12_digest
    serialized=snapshot["input_boundary_serialized"]
    assert json.loads(serialized)==snapshot["input_boundary"]
    fnv=0xcbf29ce484222325
    for byte in serialized.encode(): fnv=((fnv^byte)*0x100000001b3)&((1<<64)-1)
    assert f"fnv1a64:{fnv:016x}"==r12_digest, "raw boundary hash does not match R12"

def bind_arms(arms,authority_arms):
    assert len(arms)==10
    assert {a["arm"] for a in arms}=={a["arm"] for a in authority_arms}==set(range(1,11))
    assert len({a["arm"] for a in arms})==10
    for arm in arms: assert [s["checkpoint_step"] for s in arm["snapshots"]]==[3694,7389,11083]

def r8_mode_valid(reference):
    """Recompute the original strict R8 selector, independently of R13 envelopes."""
    assert len(reference["scales"])==2
    valid_scales=[]
    for scale in reference["scales"]:
        signs=scale["signs"]
        assert len(signs)==2
        valid=all(sign["status"]=="VALID" for sign in signs)
        if valid:
            plus=signs[0]["active_delta"]+signs[0]["inactive_delta"]
            minus=signs[1]["active_delta"]+signs[1]["inactive_delta"]
            error=norm(add(plus,minus))/max(norm(sub(plus,minus)),1e-300)
            close(error,scale["sign_symmetry"]["opposite_error"])
            expected="PASS" if error<=0.05 else "FAIL"
            assert scale["sign_symmetry"]["status"]==expected
            valid=expected=="PASS"
        else:
            assert scale["sign_symmetry"]["status"]=="NONSMOOTH_OR_INVALID"
        valid_scales.append(valid)
    valid=all(valid_scales)
    assert reference["status"]==("VALID" if valid else "G_NONLINEAR_OR_NONSMOOTH")
    return valid

def state_metrics(snapshot,r12_digest):
    bind_boundary(snapshot,r12_digest)
    assert snapshot["r7_replay_identity"]["same_phase_boundary_exact"] is True
    rec=snapshot["decomposition"]
    assert rec["source_state_before"]==rec["source_state_after"]
    assert rec["no_mechanics_topology_contact_remesh_in_cut"] is True
    state=rec["input_polarity"]; active=state["active_amount"]; inactive=state["inactive_amount"]
    boundary=snapshot["input_boundary"]; cohort=boundary["cohort"]; mesh=cohort["mesh"]
    assert state==boundary["polarity_states"][str(cohort["id"])]
    assert rec["baseline_vertices"]==mesh["vertices"]
    reconstructed_basis=normal_basis(rec["baseline_vertices"])
    assert len(reconstructed_basis)==len(rec["modes"])
    n=len(active); assert n==len(inactive) and n>2 and min(active+inactive)>=0
    measures=lengths(rec["baseline_vertices"]); vclose(measures,rec["baseline_measures"])
    assert min(measures)>0
    activity=rec["r8_activity"]
    lo,hi=0.0,0.8
    for _ in range(100):
        mid=0.5*(lo+hi)
        rate=(0.01+mid*mid)*(0.8-mid)-(0.1+0.01*mid*mid)*mid
        if rate>0: lo=mid
        else: hi=mid
    reference_active=0.5*(lo+hi)
    advanced,_,_,_=kernel(active,inactive,measures,"FULL",max(mesh["interior"]["a"],0.0)*area(mesh["vertices"]))
    concentration=[a/l for a,l in zip(advanced,measures)]
    expected_activity=[min(1.0,max(0.0,(0.5*(concentration[(i+n-1)%n]+concentration[i])-reference_active)/reference_active)) for i in range(n)]
    vclose(activity,expected_activity)
    harmonics=rec["r8_activity_harmonics"]
    assert [h["harmonic"] for h in harmonics]==list(range(n//2+1))
    dominant=max(harmonics[1:] or harmonics,key=lambda h:h["amplitude"])["harmonic"]
    # Confirm activity-harmonic selection itself against raw activity DFT.
    for item in harmonics:
        c,s=dft(activity,item["harmonic"]); close(item["amplitude"],math.hypot(c,s))
    vectors={p:[[[],[]],[[],[]]] for p in PATHS}
    phase_vectors={p:[[[],[]],[[],[]]] for p in PATHS}
    e1_vectors=[[[],[]],[[],[]]]; r8_projections=[]; e1_rows=[]; branches=[]; nonsmooth_reasons=[]
    assert len(rec["modes"])==len(rec["r8_reference"]["modes"])
    for mi,(mode,reference) in enumerate(zip(rec["modes"],rec["r8_reference"]["modes"])):
        old_mode_valid=r8_mode_valid(reference)
        k,name,basis_expected=reconstructed_basis[mi]
        assert (mode["harmonic"],mode["phase"])==(k,name)
        vclose(mode["basis"],basis_expected)
        assert mode["harmonic"]==reference["harmonic"] and mode["phase"]==reference["phase"]
        basis=mode["basis"]; close(norm(basis),1.0)
        for si,scale in enumerate(mode["scales"]):
            close(scale["amplitude"],sum(measures)/n*[0.001,0.0005][si])
            for qi,sign in enumerate(scale["signs"]):
                q=[1.0,-1.0][qi]; assert sign["sign"]==q
                expected=[[point[0]+q*scale["amplitude"]*basis[2*i],point[1]+q*scale["amplitude"]*basis[2*i+1]] for i,point in enumerate(rec["baseline_vertices"])]
                vclose([x for p in expected for x in p],[x for p in sign["vertices"] for x in p])
                actual=lengths(expected); vclose(actual,sign["actual_measures"])
                assert simple_polygon(expected), "geometry probe crossed polygon branch"
                assert sign["simple"] is True and sign["runtime_valid"] is True and sign["amounts_unchanged"] is True
                du=[a/l-a/b for a,l,b in zip(active,actual,measures)]
                dv=[a/l-a/b for a,l,b in zip(inactive,actual,measures)]
                vclose(du,sign["active_concentration_delta"]); vclose(dv,sign["inactive_concentration_delta"])
                e1_vectors[si][qi].extend(du+dv)
                e1_rows.append({"harmonic":mode["harmonic"],"phase":mode["phase"],"scale":scale["normalized_amplitude"],"sign":q,"edge_measure_delta":sub(actual,measures),"active_concentration_delta":du,"inactive_concentration_delta":dv,"local_amount_invariance_max_error":max(abs(a/l*l-a) for a,l in zip(active+inactive,actual+actual)),"active_phase_projection":phase(du,dominant,activity),"inactive_phase_projection":phase(dv,dominant,activity)})
                input_digests=set(); outputs={}
                for path in PATHS:
                    pair=sign["paths"][path]; outputs[path]=pair
                    branch_pair=[]
                    for label,length in [("full",actual),("cut",measures)]:
                        output=pair[label]; input_digests.add(output["input_state_digest"])
                        close(output["a_before"],max(mesh["interior"]["a"],0.0)*area(expected))
                        close(output["w_before"],max(mesh["interior"]["w"],0.0)*area(expected))
                        assert output["discarded_clone"] is True and output["path"]==path
                        ua,va,cost,branch=kernel(active,inactive,length,path,output["a_before"])
                        branch_pair.append(branch)
                        vclose(ua,output["active_after"]); vclose(va,output["inactive_after"])
                        assert min(output["active_after"]+output["inactive_after"])>=0
                        close(sum(ua)+sum(va),sum(active)+sum(inactive))
                        close(output["total_before"],sum(active)+sum(inactive)); close(output["total_after"],sum(ua)+sum(va))
                        close(cost,output["a_consumed"]); close(cost,output["w_produced"])
                        close(output["a_before"]-output["a_after"],cost); close(output["w_after"]-output["w_before"],cost)
                        close(sum(l["a_consumed"] for l in output["ledgers"]),cost)
                        for ledger in output["ledgers"]:
                            assert ledger["accepted"] is True
                            close(ledger["total_before"],ledger["total_after"])
                            close(ledger["a_consumed"],ledger["w_produced"])
                        if path=="TRANSPORT_ONLY": assert cost==0
                    if branch_pair[0]!=branch_pair[1]: nonsmooth_reasons.append({"reason":"actual/cut per-edge donor or clipping branch mismatch","path":path,"harmonic":mode["harmonic"],"phase":mode["phase"],"scale":si,"sign":qi})
                    branches.extend(b for sequence in branch_pair for b in sequence)
                    delta=sub(pair["full"]["active_after"],pair["cut"]["active_after"])
                    iv=sub(pair["full"]["inactive_after"],pair["cut"]["inactive_after"])
                    vclose(delta,pair["active_delta"]); vclose(iv,pair["inactive_delta"])
                    vectors[path][si][qi].extend(delta)
                    if mode["harmonic"]==dominant: phase_vectors[path][si][qi].append(phase(delta,dominant,activity))
                    if path=="FULL":
                        old=reference["scales"][si]["signs"][qi]
                        assert old["status"]=="VALID"
                        vclose(delta,old["active_delta"]); vclose(iv,old["inactive_delta"])
                        combined=delta+iv
                        # Exact old R8 two-channel sign-symmetry checked below.
                assert len(input_digests)==1
                for label in ["full","cut"]:
                    vclose(outputs["FULL"][label]["active_after"],outputs["TRANSPORT_THEN_REACTION"][label]["active_after"])
                    vclose(outputs["FULL"][label]["inactive_after"],outputs["TRANSPORT_THEN_REACTION"][label]["inactive_after"])
                if mode["harmonic"]==dominant and si==0 and qi==0 and old_mode_valid and reference["scales"][0]["sign_symmetry"]["status"]=="PASS":
                    r8_projections.append(phase(outputs["FULL"]["active_delta"],dominant,activity))
            # Old R8 uses concatenated active and inactive amount response.
            for path in PATHS:
                a=scale["signs"][0]["paths"][path]; b=scale["signs"][1]["paths"][path]
                plus=a["active_delta"]+a["inactive_delta"]; minus=b["active_delta"]+b["inactive_delta"]
                denominator=norm(sub(plus,minus)); error=norm(add(plus,minus))
                if error>0.05*denominator + math.sqrt(len(plus))*ABS + REL*norm(active+inactive): nonsmooth_reasons.append({"reason":"sign symmetry exceeds sealed R8 limit and replay bound","path":path,"harmonic":mode["harmonic"],"phase":mode["phase"],"scale":si})
    assert r8_projections and sum(r8_projections)/len(r8_projections)<-1e-12
    r8_sign=sum(r8_projections)/len(r8_projections)
    # Phase functional applies only to the exact dominant harmonic's two phases.
    pvalues={}; perrors={}; response={}; errors={}; controls={}
    for path in PATHS:
        response[path],errors[path],controls[path]=control(vectors[path],norm(active+inactive))
        p=phase_vectors[path]
        means=[[sum(v)/len(v) for v in row] for row in p]
        pvalues[path]=means[0][0]  # exact R8 plus-sign epsilon indicator
        perrors[path]=max(abs((means[0][0]+means[0][1])/2),abs(means[0][0]-2*means[1][0]),ABS+REL*max(abs(x) for row in means for x in row))
    response["INTERACTION"]=sub(sub(response["FULL"],response["REACTION_ONLY"]),response["TRANSPORT_ONLY"])
    errors["INTERACTION"]=sum(errors[p] for p in ["FULL","REACTION_ONLY","TRANSPORT_ONLY"])
    pvalues["INTERACTION"]=pvalues["FULL"]-pvalues["REACTION_ONLY"]-pvalues["TRANSPORT_ONLY"]
    perrors["INTERACTION"]=sum(perrors[p] for p in ["FULL","REACTION_ONLY","TRANSPORT_ONLY"])
    nomination,metrics=attribution(response,pvalues,perrors,errors)
    if nonsmooth_reasons: nomination=None
    e1,ee,ec=control(e1_vectors,norm([a/l for a,l in zip(active+inactive,measures+measures)]))
    order_delta=sub(response["REACTION_THEN_TRANSPORT"],response["TRANSPORT_THEN_REACTION"])
    return {"input_boundary_digest":r12_digest,"checkpoint_step":snapshot["checkpoint_step"],"usable":not nonsmooth_reasons,"exclusion_reasons":nonsmooth_reasons,"r8_damping_indicator":r8_sign,"dominant_activity_harmonic":dominant,"nomination":nomination,"metrics":metrics,"controls":controls,"response_envelopes":errors,"phase_indicators":pvalues,"phase_envelopes":perrors,"interaction_norm":norm(response["INTERACTION"]),"interaction_material_beyond_envelope":norm(response["INTERACTION"])>errors["INTERACTION"],"order_probe_difference_norm":norm(order_delta),"order_probe_material_beyond_envelope":norm(order_delta)>errors["REACTION_THEN_TRANSPORT"]+errors["TRANSPORT_THEN_REACTION"],"e1_control":ec,"e1_response_norm":norm(e1),"e1_envelope":ee,"e1_records":e1_rows,"branch_counts":{"donor_limited":sum(sum(x["donor_active"])+sum(x["donor_inactive"]) for x in branches),"reaction_clipped":sum(sum(mask!="NONE" for mask in x["reaction_clip"]) for x in branches)},"complete_per_substep_edge_branch_masks_match":not any("branch mismatch" in x["reason"] for x in nonsmooth_reasons),"non_smooth_exclusions":int(bool(nonsmooth_reasons))}


def consequence(classification):
    if classification==CLASSES[0]: return "Specify a future local mechanochemical reaction architecture gate preserving amount/current-length concentration physics, finite conversion A/W and R3 coefficients; distinguish a physical membrane/cortical coupling from R9's already-falsified tensile activation multiplier. No repair is implemented or qualified."
    if classification==CLASSES[1]: return "Audit whether the existing step-local material/mechanics state supplies a persistent causal tangential transport field with material and energy semantics. Such a qualified flow state is not established; return that ownership gap before any advection architecture."
    if classification==CLASSES[2]: return "Require coupled-process evidence linking the observed substep order sensitivity to the geometry-return response on the same fixed states before proposing any reaction/transport mechanism or parameter change."
    if classification=="G_TO_P_DAMPING_MIXED_STATE_DEPENDENT": return "Require causal interventions on a prospectively chosen physical state variable separating the uniquely nominated regimes; current statewise association alone does not establish a separator."
    return "Require evidence that distinguishes the isolated routes beyond the current sign, half-scale and replay envelopes; no dominant route or corrective architecture is established."

def classify(records):
    counts={k:sum(r["nomination"]==k for r in records) for k in COMPONENTS}
    for k,c in zip(COMPONENTS,CLASSES):
        if counts[k]>=24: return c,counts
    if sum(counts.values())>=24: return "G_TO_P_DAMPING_MIXED_STATE_DEPENDENT",counts
    return "G_TO_P_INTERNAL_ATTRIBUTION_INCONCLUSIVE",counts

def manifest(repo,root):
    files={str(p.relative_to(root)):sha(p) for p in sorted(root.rglob("*")) if p.is_file() and p.name!="artifact_manifest.json"}
    dump(root,"artifact_manifest.json",{"exact_head":git(repo,"rev-parse","HEAD"),"files":files,"file_count":len(files),"source_audit":source_audit(repo)})

def main():
    parser=argparse.ArgumentParser(); parser.add_argument("--repo",type=Path,default=Path(".")); parser.add_argument("--raw",type=Path,required=True); parser.add_argument("--output",type=Path,required=True); parser.add_argument("--stage",choices=["seal","qualification"],required=True); parser.add_argument("--r12-raw",type=Path)
    args=parser.parse_args(); repo=args.repo.resolve(); root=args.output.resolve(); raw=load(args.raw)
    validate_contract(raw); audit=source_audit(repo)
    if args.stage=="seal":
        assert "arms" not in raw
        dump(root,"prospective_contract.json",raw); dump(root,"operator_graph.json",audit)
        prior=repo.parent/"docs/dc-m4-r13-prior-art.md"
        dump(root,"external_prior_art.json",{"source":"docs/dc-m4-r13-prior-art.md","sha256":sha(prior),"content":prior.read_text(),"imports":None})
        dump(root,"pre_outcome_seal.json",{"contract_sha256":sha(root/"prospective_contract.json"),"exact_head":git(repo,"rev-parse","HEAD"),"source_files":audit["files"],"outcome_observed":False})
    else:
        sealed=load(root/"prospective_contract.json"); assert {k:v for k,v in raw.items() if k!="arms"}==sealed
        seal=load(root/"pre_outcome_seal.json"); assert seal["contract_sha256"]==sha(root/"prospective_contract.json")
        assert seal["source_files"]==audit["files"]
        assert args.r12_raw is not None, "sealed R12 raw required"
        authority=load(args.r12_raw); assert authority["snapshot_count"]==30
        assert authority["entry_r11"]["head"]=="c5d5342b19e912c004dd87eaf33a1ce1f16d27d7"
        digest_map={(arm["arm"],s["checkpoint_step"]):s["input_boundary_digest"] for arm in authority["arms"] for s in arm["snapshots"]}
        assert len(digest_map)==30
        frozen=load(repo/"experiments/fixtures/dcm4r13/r12_input_identities.json")
        assert {(x["arm"],x["checkpoint_step"]):x["input_boundary_digest"] for x in frozen}==digest_map
        records=[]; bind_arms(raw["arms"],authority["arms"])
        for arm in raw["arms"]:
            assert arm["accepted"] is True and arm["accepted_steps"]==14778 and arm["physical_fissions"]==0
            assert [s["checkpoint_step"] for s in arm["snapshots"]]==[3694,7389,11083]
            for snapshot in arm["snapshots"]:
                row=state_metrics(snapshot,digest_map[(arm["arm"],snapshot["checkpoint_step"])])
                row["arm"]=arm["arm"]; records.append(row)
        classification,counts=classify(records)
        usable=sum(r["usable"] for r in records)
        exclusions=sum(r["non_smooth_exclusions"] for r in records)
        assert len(records)==30
        if usable<24: classification="G_TO_P_INTERNAL_ATTRIBUTION_INCONCLUSIVE"
        dump(root,"internal_causal_attribution.json",{"classification":classification,"counts":counts,"records":[{k:v for k,v in r.items() if k!="e1_records"} for r in records]})
        dump(root,"instantaneous_concentration.json",{"records":[{"arm":r["arm"],"checkpoint_step":r["checkpoint_step"],"records":r["e1_records"]} for r in records]})
        dump(root,"r8_reproduction.json",{"same_r12_boundary_digests":30,"damping":sum(r["r8_damping_indicator"]<-1e-12 for r in records),"reinforcing":sum(r["r8_damping_indicator"]>1e-12 for r in records),"r12_raw_sha256":sha(args.r12_raw),"r8_operator_recomputed":True})
        dump(root,"architecture_consequence.json",{"classification":classification,"consequence":consequence(classification),"implemented":False})
        dump(root,"evidence_table.json",{"VERIFIED":["R8 damping exactly reproduced on30/30 same R12 input-boundary digests","Production polarity, actuator and mechanics sources byte-identical","All actual/cut kernel pools and A/W reconstructed independently","Isolated and nonadditive interaction responses and both substep orders measured"],"SUPPORTED_HYPOTHESIS":[classification] if classification in CLASSES else [],"INFERRED":["Route-level dominance uses prospectively sealed state-specific control envelopes"],"UNKNOWN":["Corrected feedback, morphogenesis, continuing reproduction and integrated lifeform"],"DISPROVEN":["R9's already-tested tensile-strain multiplier repairs the accepted damping"]})
        dump(root,"preservation.json",{"production_biology_modified":False,"original_shared_source_prefix_identical":True,"all_cuts_discarded":True,"no_topology_contact_remesh_in_cuts":True,"material_and_energy_reconstruction":"PASS","non_smooth_exclusions":exclusions})
        dump(root,"qualification.json",{"directive":DIRECTIVE,"implementation":"PASS","scientific_acceptance":"ATTRIBUTION_ONLY","terminal_classification":classification,"usable_states":usable,"non_smooth_exclusions":exclusions,"coverage_gate":"PASS" if usable>=24 else "STOP_FEWER_THAN24_USABLE","r8_damping_states":sum(r["r8_damping_indicator"]<-1e-12 for r in records),"dominant_route_counts":counts,"morphogenesis":"NOT_REACHED","reproduction":"NOT_REACHED","population_selection":"NOT_REACHED","reversal":"NOT_REACHED","final_integration":"NOT_REACHED","next_execution_started":False})
        print(json.dumps({"classification":classification,"usable_states":usable,"r8_damping":sum(r["r8_damping_indicator"]<-1e-12 for r in records),"counts":counts}))
    manifest(repo,root)
if __name__=="__main__": main()
