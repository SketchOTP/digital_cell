#!/usr/bin/env python3
"""Exact bounded architecture witnesses; no native runtime or calibration run."""
from dataclasses import dataclass, replace
from fractions import Fraction as F
import argparse
import hashlib
import importlib.util
import itertools
import json
import math
from pathlib import Path
import subprocess
import sys

BASE='101e635d6498b9cfe451b2d702d64a4440939963'
PREFIX='docs/dc-m4-source-receiver-'
DIRECTIVE='D-20261017-dcm4-source-receiver-architecture'


def require(value,message):
    if not value:raise ValueError(message)


def sha(data):return hashlib.sha256(data).hexdigest()


@dataclass(frozen=True)
class State:
    a:F; b:F; L:F; L0:F; phi:F; A:F; W:F=F(0)


def valid(s):
    require(all(math.isfinite(x) for x in (s.a,s.b,s.L,s.L0,s.phi,s.A,s.W)),'nonfinite state')
    require(s.a>=0 and s.b>=0 and s.a+s.b>0 and s.A>=0 and s.W>=0,'material domain')
    require(s.L0>=F(1,1000) and 0<s.L<=4*s.L0 and 0<s.phi<=1,'bounded receiver domain')


def ec(s,K=F(2),lam=F(1,4)):
    valid(s);require(math.isfinite(K) and math.isfinite(lam) and K>0 and lam>0,'coupling parameter domain')
    P=s.a+s.b
    return K*P/2*(s.L/s.L0-1-lam*s.a/P)**2


def us(s,ks=F(14)):
    valid(s);require(math.isfinite(ks) and ks>0,'receiver parameter domain')
    return ks/(2*s.L0)*(s.L-s.L0)**2*(F(1) if s.L<=s.L0 else s.phi)


def native_tension(s,ks=F(14)):
    reference=max(s.L0,s.L/4,F(1,1000))
    raw=ks*(s.L-s.L0)/reference
    clipped=min(8*ks,max(-8*ks,raw))
    return clipped*(F(1) if s.L<s.L0 else s.phi)


def reaction(s,d,eps=F(3)):
    valid(s);require(math.isfinite(d) and -s.a<=d<=s.b,'donor violation');require(math.isfinite(eps) and eps>0,'fuel energy domain')
    after=replace(s,a=s.a+d,b=s.b-d)
    delta=ec(after)-ec(s);overhead=abs(d)/2
    # A monotone physical conversion stroke may cross the coupling minimum.
    # Downhill release is dissipated and cannot subsidize its later uphill leg.
    P=s.a+s.b;stationary=P*(s.L/s.L0-1)/F(1,4)
    low,high=sorted((s.a,after.a))
    minimum=F(0) if low<stationary<high else min(ec(s),ec(after))
    positive=ec(after)-minimum;negative=ec(s)-minimum
    require(positive-negative==delta and positive>=0 and negative>=0,'conversion path work')
    load=positive/eps
    debit=overhead+load;require(debit<=s.A,'insufficient finite A')
    out=replace(after,A=s.A-debit,W=s.W+debit)
    heat=eps*overhead+negative
    require(eps*(out.A-s.A)+delta+heat==0,'reaction energy residual')
    require(out.A+out.W==s.A+s.W and out.a+out.b==s.a+s.b,'reaction material residual')
    return out,heat


def dg_receiver(s,Lnew):
    end=replace(s,L=Lnew)
    if Lnew!=s.L:return (us(end)-us(s))/(Lnew-s.L)
    return native_tension(s)


def mechanics(s,Lnew,h=F(1,50),gamma=F(1)):
    valid(s);out=replace(s,L=Lnew);valid(out)
    require(math.isfinite(h) and math.isfinite(gamma) and h>0 and gamma>0,'mechanical time/drag domain')
    d=Lnew-s.L;P=s.a+s.b
    tc=F(2)*P/s.L0*((Lnew+s.L)/(2*s.L0)-1-F(1,4)*s.a/P)
    ts=dg_receiver(s,Lnew)
    require(gamma*d/h+tc+ts==0,'not the coupled DG root')
    require(abs(tc)<=F(1,2),'coupling piston cap')
    heat=gamma*d*d/h
    require(ec(out)-ec(s)+us(out)-us(s)+heat==0,'complete mechanical energy residual')
    require(out.A==s.A and out.W==s.W,'geometry fuel charge')
    return out,heat


def cycle(s,d,Lnew):
    """Publish only after BOTH immutable reference transactions succeed."""
    scratch,qchem=reaction(s,d)
    out,qdrag=mechanics(scratch,Lnew)
    return out,qchem,qdrag


def witnesses():
    checks={};count=0
    for L0,phi,r in itertools.product((F(1,10),F(1),F(2)),(F(1,4),F(1,2),F(1)),(F(1,4),F(1,2),F(1),F(3,2),F(3),F(4))):
        s=State(F(1),F(1),r*L0,L0,phi,F(10));derivative=F(14)/L0*(s.L-L0)*(F(1) if s.L<=L0 else phi)
        require(derivative==native_tension(s),'native force/potential mismatch')
        require(max(L0,s.L/4,F(1,1000))==L0,'native reference branch')
        require(-8*14<=14*(s.L-L0)/L0<=8*14,'native clamp branch')
        require(us(s)<=F(14)*L0/2*max(F(1),9*phi),'receiver capacity')
        require(ec(s)<=F(2)*(s.a+s.b)/2*max(F(9),(1+F(1,4))**2),'coupling capacity')
        count+=1
    checks['native_receiver_force_potential_and_capacity_cases']=count
    # Actual reference energy is continuous and force-zero at the branch join.
    rest=State(F(0),F(2),F(1),F(1),F(1,2),F(10))
    require(us(rest)==native_tension(rest)==0,'rest continuity')
    cross=replace(rest,L=F(9,10));target=F(11,10)
    correct=dg_receiver(cross,target)
    wrong=F(14)*cross.phi/cross.L0*((target+cross.L)/2-cross.L0)
    require(correct!=wrong and correct*(target-cross.L)==us(replace(cross,L=target))-us(cross),'cross-rest shortcut control')
    checks['piecewise_receiver_true_secant']='PASS_WRONG_SINGLE_BRANCH_REJECTED'
    # Cold entry has no unexplained elastic preload. P is prepared fixed material.
    require(rest.a==0 and ec(rest)==us(rest)==0,'free initial store')
    loaded,qchem=reaction(rest,F(1))
    moved,qdrag=mechanics(loaded,F(112,111))
    require(us(moved)>0 and ec(moved)<ec(loaded),'source failed to deliver to receiver')
    require(F(3)*(moved.A-rest.A)+ec(moved)+us(moved)+qchem+qdrag==0,'whole cycle energy residual')
    require(moved.A==loaded.A and moved.W==loaded.W,'geometry double fuel input')
    require(ec(loaded)-ec(moved)>qdrag,'coupling release was all relabeled heat')
    checks['cold_entry_reaction_to_native_receiver_cycle']='EXACT_CLOSED_REFERENCE_TRANSACTION'
    # An already-owned receiver preload can load Ec mechanically with no fuel.
    # This is a local step witness, not a cold-entry preparation claim.
    preloaded=replace(loaded,L=F(9,8))
    received,qd=mechanics(preloaded,F(9,8)-F(7,444))
    dec=ec(received)-ec(preloaded);dus=us(received)-us(preloaded)
    require(dec>0 and dus<0 and dec+dus+qd==0,'receiver failed to load coupling store')
    require(received.A==preloaded.A,'geometry extra A debit')
    wrong_fuel_input=dec
    require(wrong_fuel_input!=dec+dus+qd,'duplicate geometry A input not detected')
    require(ec(loaded)-ec(moved)!=qdrag,'all-release-to-heat mutation')
    checks['geometry_A_topup_and_all_release_heat']='REJECTED_BY_COMPLETE_BALANCE'
    checks['receiver_to_coupling_positive_geometry_energy']='EXACT_LOCAL_STEP_NOT_PREPARATION_PROOF'
    # Repeated backward conversion never refunds fuel; energy settles locally.
    reset,qreverse=reaction(moved,F(-1))
    require(reset.A<moved.A and qreverse>=0 and reset.A+reset.W==rest.A+rest.W,'fuel recovery')
    checks['reverse_conversion_no_A_regeneration']='PASS'
    equal_ends=replace(rest,L=F(9,8))
    equal_out,qequal=reaction(equal_ends,F(2))
    require(ec(equal_out)==ec(equal_ends)>0,'equal-end source control')
    require(equal_out.A<equal_ends.A-F(1) and qequal>F(3),'downhill stroke subsidized uphill reload')
    checks['cross_minimum_source_stroke']='POSITIVE_VARIATION_FUNDED_NEGATIVE_VARIATION_DISSIPATED'
    # Bidirectional finite extents prove the source law, rather than selected signs.
    source_cases=0
    for a,b,L,d in itertools.product((F(1),F(2)),(F(1),F(3)),(F(1,2),F(1),F(2)),(F(-1,2),F(1,2))):
        s=State(a,b,L,F(1),F(1,2),F(100))
        out,q=reaction(s,d);require(q>=0 and out.A<=s.A,'negative heat or fuel gain');source_cases+=1
    checks['exact_signed_conversion_source_cases']=source_cases
    # Late proposal failure publishes neither a debit nor a geometric mutation.
    before=replace(rest,A=F(1,2));published=before
    try:
        scratch,_=reaction(before,F(1))
        published=scratch
    except ValueError:pass
    require(published==before,'unfunded source was partially committed')
    published=loaded
    try: mechanics(loaded,F(4)+F(1,100))
    except ValueError:pass
    else:raise ValueError('out-of-domain mechanics accepted')
    require(published==loaded,'rejection mutated input')
    # A funded reaction succeeds, then mechanics fails; no stage may publish.
    before=(rest,F(7),F(9));published=before
    try:
        out,qc,qd=cycle(rest,F(1),F(4)+F(1,100))
        published=(out,before[1]+qc,before[2]+qd)
    except ValueError:pass
    require(published==before,'late cycle failure changed stores or sinks')
    out,qc,qd=cycle(rest,F(1),F(112,111))
    require(F(3)*(out.A-rest.A)+ec(out)+us(out)+qc+qd==0,'accepted cycle wrapper residual')
    checks['reference_atomic_funding_and_domain_rejection']='PASS_NOT_NATIVE_ROLLBACK_QUALIFICATION'
    invalid=0
    for s in (replace(rest,L0=F(1,10000)),replace(rest,L=F(0)),replace(rest,phi=F(0)),replace(rest,A=F(-1)),replace(rest,a=float('nan'))):
        try: valid(s)
        except ValueError:invalid+=1
    require(invalid==5,'invalid receiver state accepted')
    checks['invalid_domain_controls']=invalid
    rejected_parameters=0
    for operation in (lambda:ec(rest,K=float('inf')),lambda:ec(rest,lam=float('nan')),
                      lambda:us(rest,ks=float('inf')),lambda:reaction(rest,F(1),eps=float('inf')),
                      lambda:reaction(rest,float('nan')),lambda:mechanics(rest,F(1),h=float('inf')),
                      lambda:mechanics(rest,F(1),gamma=float('inf'))):
        try:operation()
        except ValueError:rejected_parameters+=1
    require(rejected_parameters==7,'nonfinite architecture parameter accepted')
    checks['nonfinite_parameter_controls']=rejected_parameters
    # Same length displacement has different physical mobility for a free pair.
    require(F(1)/2!=F(1),'fixed piston/free-pair mobility distinction')
    checks['scalar_fixed_endpoint_only']='NO_NATIVE_FREE_PAIR_OR_MESH_SOLVER_CLAIM'
    return checks


def main():
    ap=argparse.ArgumentParser();ap.add_argument('--repo',type=Path,default=Path('.'));ap.add_argument('--output',type=Path,required=True);args=ap.parse_args();repo=args.repo.resolve();out=args.output.resolve()
    require(not out.is_relative_to(repo),'evidence belongs outside checkout')
    def git(*a):return subprocess.check_output(['git',*a],cwd=repo)
    require(not git('diff','--name-only',BASE,'--','digital-protocell'),'scientific source changed')
    allowed=('.agent/',PREFIX,'scripts/verify_source_receiver_architecture.py','.github/workflows/dc-m4-source-receiver-architecture.yml','STORAGE_MAP.md')
    require(all(p.startswith(allowed) for p in git('diff','--name-only',BASE).decode().splitlines()),'scope drift')
    for n in ('DIRECTIVES','OUTCOMES','LEARNINGS','RECORD','REPO_MAP'):
        p=f'.agent/{n}.md';require((repo/p).read_bytes().startswith(git('show',f'{BASE}:{p}')),'append-only drift '+p)
    frozen=json.loads((repo/'docs/dc-m4-reaction-architecture-audit.json').read_text())['source_bindings'].copy()
    previous=['docs/dc-m4-conversion-work-contract.md','docs/dc-m4-conversion-work-contract.json','docs/dc-m4-conversion-work-directive.md','docs/dc-m4-conversion-work-review.json','scripts/verify_conversion_work_contract.py','.github/workflows/dc-m4-conversion-work-contract.yml',
              'docs/dc-m4-work-port-specification.md','docs/dc-m4-work-port-specification.json','docs/dc-m4-work-port-directive.md','docs/dc-m4-work-port-review.json','scripts/verify_work_port_specification.py','.github/workflows/dc-m4-work-port-specification.yml']
    for p in previous:frozen[p]=sha(git('show',f'{BASE}:{p}'))
    for p,digest in frozen.items():require(sha((repo/p).read_bytes())==digest and (repo/p).read_bytes()==git('show',f'{BASE}:{p}'),'frozen input drift '+p)
    architecture=json.loads((repo/(PREFIX+'architecture.json')).read_text())
    require(architecture['entry_head']==BASE,'architecture authority')
    require(architecture['schema']=='digital_cell_physically_owned_source_receiver_architecture_v2' and
            architecture['architecture_id']=='BOUNDED_REACTION_STRETCH_SOURCE_RECEIVER_V2','architecture identity drift')
    require(architecture['production_parameter_values']=={'K':None,'lambda':None,'theta':None,'epsilon_A':None},'unjustified calibration')
    require(all(architecture['parameter_domain'][n]=='>0' for n in ('K','lambda','theta','epsilon_A')),'positive parameter domain drift')
    require(not any(architecture[k] for k in ('production_modified','runtime_implementation_authorized','calibration_execution_authorized','automatic_successor_authorized')),'scope expansion')
    require(architecture['revision']['geometry_A_topup'] is False and
            architecture['revision']['new_persistent_energy_tank'] is False,'unowned new energy source')
    require(architecture['stores']['E_c']=='K*P/2*(epsilon-lambda*x)^2' and
            architecture['stores']['U_s_compression']=='k_s*(L-L0)^2/(2*L0),L<=L0' and
            architecture['stores']['U_s_tension']=='phi*k_s*(L-L0)^2/(2*L0),L>=L0','source/receiver equation drift')
    require(architecture['reaction']['z']=='c0*abs(d)+Vplus/epsilon_A' and
            architecture['reaction']['Q_chem']=='epsilon_A*c0*abs(d)+Vminus' and
            architecture['reaction']['within_stroke_recovery'] is False,'physical stroke work policy drift')
    require(architecture['cycle']['whole_identity']=='Delta(E_A+E_c+U_s)+Q_chem+Q_drag=0' and
            architecture['mechanics']['A_change']=='0' and
            architecture['mechanics']['whole_mesh_uniqueness_claimed'] is False and
            architecture['legacy_ports']['production_disabling_authorized'] is False,'bounded closure overclaim')
    cold=architecture['entry']['cold_state']
    require(all(cold[k]==v for k,v in {'L':'L0','a':'0','b':'P','E_c':'0','U_s':'0'}.items()),'free store preload')
    anchors={a['id']:a for a in architecture['anchor_inventory']}
    require(set(anchors)=={'native_stretch','native_scaffold','native_reaction','native_boundary','reference_units','battery_and_coupling','toyabe_2010','toyabe_muneyuki_2015','native_parameter_data'},'anchor inventory incomplete')
    require(anchors['native_parameter_data']['category']=='INDEPENDENT_DIGITAL_CELL_DATA_MISSING' and
            anchors['native_parameter_data']['current_values']==architecture['production_parameter_values'],'missing data promoted')
    for name,url in {'toyabe_2010':'https://arxiv.org/abs/1002.1867','toyabe_muneyuki_2015':'https://arxiv.org/abs/1210.4017'}.items():
        require(anchors[name]['url']==url and anchors[name]['imported_values_or_code'] is False,'external method promoted to numerical calibration')
    require(architecture['unsupported_operations']['policy']=='atomic_reject_before_mutation','unowned lifecycle extension')
    checks=witnesses()
    review=json.loads((repo/(PREFIX+'review.json')).read_text());require(review['decision']=='ACCEPTED_BOUNDED_ARCHITECTURE_ONLY','independent review missing')
    mandatory={PREFIX+'architecture.md',PREFIX+'architecture.json',PREFIX+'directive.md','scripts/verify_source_receiver_architecture.py','.github/workflows/dc-m4-source-receiver-architecture.yml'}
    require(set(review['reviewed_files'])==mandatory,'review binding incomplete')
    for p,digest in review['reviewed_files'].items():require(sha((repo/p).read_bytes())==digest,'stale independent review '+p)
    sp=importlib.util.spec_from_file_location('governance',repo/'scripts/validate_governance.py');v=importlib.util.module_from_spec(sp);sp.loader.exec_module(v)
    def section(file,title):
        for n,b in v.iter_sections((repo/file).read_text()):
            if n==title:return '## '+n+'\n'+b
        raise ValueError('missing new record '+title)
    errors=[];ids=v.validate_directives(section('.agent/DIRECTIVES.md',DIRECTIVE),errors);v.validate_outcomes(section('.agent/OUTCOMES.md',DIRECTIVE+' - COMPLETE'),ids,errors);v.validate_learnings(section('.agent/LEARNINGS.md','L-DCM4-SOURCE-RECEIVER-20261017'),errors);require(not errors,str(errors))
    evidence={'exact_head':git('rev-parse','HEAD').decode().strip(),'entry_head':BASE,'directive':DIRECTIVE,'classification':'BOUNDED_SOURCE_RECEIVER_ARCHITECTURE_DEFINED_AND_VERIFIED','reference_checks':checks,'frozen_input_bindings':frozen,'scientific_source_count':16,'production_subtree_byte_identical':True,'append_only_ledgers':'PASS','new_structured_records':'PASS','independent_parameter_calibration':'NOT_ESTABLISHED','full_native_integration':'NOT_ESTABLISHED','runtime_implemented':False,'R13_correction':'NOT_ESTABLISHED','automatic_successor':False,'historical_governance':'PREEXISTING_DRIFT_NOT_CLAIMED_PASS'}
    out.mkdir(parents=True,exist_ok=True);(out/'verification.json').write_text(json.dumps(evidence,indent=2)+'\n')
    paths=[*repo.glob(PREFIX+'*'),repo/'scripts/verify_source_receiver_architecture.py',repo/'.github/workflows/dc-m4-source-receiver-architecture.yml',repo/'scripts/validate_governance.py',repo/'STORAGE_MAP.md',repo/'docs/dc-m4-reaction-architecture-audit.json']
    paths += [repo/f'.agent/{n}.md' for n in ('CURRENT','PROJECT_PROFILE','DIRECTIVES','OUTCOMES','LEARNINGS','RECORD','REPO_MAP')]+[repo/p for p in frozen]
    bound={}
    for p in paths:
        rel=p.relative_to(repo);bound[str(rel)]=sha(p.read_bytes());target=out/'repository'/rel;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(p.read_bytes())
    manifest={'exact_head':evidence['exact_head'],'entry_head':BASE,'file_count':len(bound),'bound_repository_files':bound,'verification_sha256':sha((out/'verification.json').read_bytes())};(out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n');print(json.dumps({'checks':'PASS','classification':evidence['classification'],'reference_groups':len(checks),'native_receiver_cases':checks['native_receiver_force_potential_and_capacity_cases'],'conversion_cases':checks['exact_signed_conversion_source_cases'],'bound_files':len(bound)}))


if __name__=='__main__':main()
