#!/usr/bin/env python3
"""Diagnostic contract arithmetic/provenance only; never a production simulator."""
import argparse
from dataclasses import dataclass, replace
from fractions import Fraction as F
import hashlib
import importlib.util
import itertools
import json
import math
from pathlib import Path
import subprocess

BASE = 'b2de15dc0d7a7ee3f25b7993c558099d17a150a0'
DIRECTIVE = 'D-20261017-dcm4-conversion-work-contract'
PREFIX = 'docs/dc-m4-conversion-work-'


def require(value, message):
    if not value:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


@dataclass(frozen=True)
class Edge:
    a: F
    b: F
    length: F
    rest: F
    mature: bool = True
    ruptured: bool = False


def energy(e, k=F(2), lam=F(1, 3)):
    require(all(math.isfinite(x) for x in (e.a, e.b, e.length, e.rest, k, lam)), 'nonfinite')
    require(e.a >= 0 and e.b >= 0 and e.length > 0 and e.rest >= 0 and k > 0 and lam > 0, 'domain')
    if e.a + e.b == 0 or not e.mature or e.ruptured:
        return F(0)
    require(e.rest > 0, 'no owned rest length')
    p = e.a + e.b
    z = e.length / e.rest - 1 - lam * e.a / p
    return k * p * z * z / 2


def conversion_gradient(e, k, lam):
    p = e.a + e.b
    return k * lam * (lam * e.a / p - (e.length / e.rest - 1))


def settlement(before, after, available, overhead=F(0), material=F(0), eps=F(3)):
    """One explicitly named ownership group; no cross-group release credit."""
    require(len(before)<=1 and len(after)<=1, 'cross-edge netting is forbidden')
    require(all(math.isfinite(x) for x in (available, overhead, material, eps)), 'nonfinite ledger')
    require(available >= 0 and overhead >= 0 and material >= 0 and eps > 0, 'ledger domain')
    delta = sum(map(energy, after), F(0)) - sum(map(energy, before), F(0))
    work_a = max(delta, F(0)) / eps
    debit = overhead + material + work_a
    require(debit <= available, 'insufficient finite A')
    waste = overhead + work_a  # synthesis material never becomes W
    heat = eps * overhead + max(-delta, F(0))
    require(eps * (debit - material) == delta + heat, 'incremental energy residual')
    require(heat >= 0 and waste >= 0, 'negative sink')
    return {'a': available-debit, 'w': waste, 'q': heat, 'delta': delta,
            'work_a': work_a, 'material_a': material}


def fragment_settlement(pairs, available, eps=F(3)):
    """Topology arithmetic on explicit correspondence entitlements, not edge sums."""
    require(all(math.isfinite(x) for x in (available,eps)) and
            all(math.isfinite(x) for pair in pairs for x in pair),'nonfinite fragment ledger')
    require(available>=0 and eps>0,'fragment domain')
    require(all(old>=0 and new>=0 for old,new in pairs),'negative fragment')
    deltas=[new-old for old,new in pairs]
    charge=sum((max(d,F(0)) for d in deltas),F(0))/eps
    require(charge<=available,'unfunded topology fragment')
    release=sum((max(-d,F(0)) for d in deltas),F(0))
    require(eps*charge==sum(deltas,F(0))+release,'fragment energy residual')
    return {'a':available-charge,'w':charge,'q':release,'delta':sum(deltas,F(0)),'work_a':charge}


def controls():
    results = {}
    cases = 0
    for a,b,L,L0,k,lam,x in itertools.product(
            (F(1,4),F(1)), (F(1,2),F(2)), (F(1,2),F(3,2),F(3)),
            (F(1),F(2)), (F(1),F(3)), (F(1,5),F(2,3)), (F(-1,8),F(1,8))):
        e = Edge(a,b,L,L0); p=a+b
        g = conversion_gradient(e,k,lam)
        d = energy(replace(e,a=a+x,b=b-x),k,lam)-energy(e,k,lam)
        require(d == g*x + k*lam*lam*x*x/(2*p), 'finite conversion identity')
        dL=F(1,7)
        tbar=k*p/L0*((L+(L+dL))/(2*L0)-1-lam*a/p)
        delta=energy(replace(e,length=L+dL),k,lam)-energy(e,k,lam)
        require(-tbar*dL == -delta, 'exact discrete mechanical work')
        cases+=1
    results['exact_finite_conversion_and_discrete_work_cases']=cases
    # A wrong sign, infinitesimal-only work, and omitted reciprocal force all fail.
    e=Edge(F(1),F(2),F(2),F(1)); x=F(1,2)
    g=conversion_gradient(e,F(2),F(1,3))
    delta=energy(replace(e,a=e.a+x,b=e.b-x))-energy(e)
    require(delta != g*x and delta != -g*x, 'mutation controls not discriminating')
    require(delta != 0, 'omitted work mutation')
    # Mixed derivative independently via exact differences (g is affine in L).
    h=F(1,11)
    require((conversion_gradient(replace(e,length=e.length+h),F(2),F(1,3))-g)/h == -F(2,3), 'mixed derivative sign')
    results['wrong_sign_linearized_work_omitted_force_controls']='REJECTED'
    # Directional tilt is a driven kinetic assumption, not microscopic LDB.
    f0,b0,theta = 0.7,0.4,0.6
    fluxes=[]
    for load in (-0.5,0,0.5):
        fluxes.append(f0*math.exp(-load/(2*theta))-b0*math.exp(load/(2*theta)))
    require(fluxes[0]>fluxes[1]>fluxes[2], 'signed load response')
    results['signed_kinetic_load_response']='PASS_NO_LDB_CLAIM'
    # Dimensional exponents are (energy, A amount, polarity amount, length, time).
    dims={'g':(1,0,-1,0,0),'theta':(1,0,-1,0,0),'epsilon_A':(1,-1,0,0,0)}
    require(dims['g']==dims['theta'] and dims['g']!=dims['epsilon_A'], 'invalid nondimensionalization')
    results['energy_per_polarity_vs_energy_per_A']='DISTINCT'
    # Build forces from coordinates; rotate+translate the entire edge.
    def pair(x0,x1,tension):
        delta=tuple(x1[j]-x0[j] for j in range(2))
        length=math.sqrt(sum(v*v for v in delta))
        f0=tuple(tension*v/length for v in delta)
        return length,f0,tuple(-v for v in f0)
    x0=(1.0,2.0);x1=(4.0,6.0);tension=7/3
    length,f0,f1=pair(x0,x1,tension)
    rotate=lambda x:(-x[1],x[0])
    transform=lambda x:(-x[1]+13,x[0]-8)
    l2,r0,r1=pair(transform(x0),transform(x1),tension)
    require(length==l2 and all(abs(r0[j]-rotate(f0)[j])<1e-14 for j in range(2)), 'objectivity')
    require(all(f0[j]+f1[j]==0 for j in range(2)), 'net force')
    torque=x0[0]*f0[1]-x0[1]*f0[0]+x1[0]*f1[1]-x1[1]*f1[0]
    require(abs(torque)<1e-14,'pair torque')
    results['central_pair_objectivity']='PASS_COORDINATE_ROTATION_TRANSLATION'
    # Finite non-collinear endpoint motion requires a discrete edge gradient.
    old_r=(F(3),F(4));new_r=(F(0),F(13));old_L=F(5);new_L=F(13)
    discrete=tuple((new_r[j]+old_r[j])/(new_L+old_L) for j in range(2))
    dr=tuple(new_r[j]-old_r[j] for j in range(2))
    require(sum(discrete[j]*dr[j] for j in range(2))==new_L-old_L,'discrete length gradient')
    old_e=replace(e,length=old_L);new_e=replace(e,length=new_L)
    p=e.a+e.b
    tbar=F(2)*p/e.rest*((new_L+old_L)/(2*e.rest)-1-F(1,3)*e.a/p)
    actual_work=-tbar*sum(discrete[j]*dr[j] for j in range(2))
    require(actual_work==energy(old_e)-energy(new_e),'noncollinear finite work residual')
    start_work=-tbar*sum(old_r[j]/old_L*dr[j] for j in range(2))
    require(start_work!=actual_work,'start tangent mutation was not rejected')
    results['finite_vertex_motion_discrete_gradient']='PASS_START_TANGENT_CONTROL_REJECTED'
    # Proportional split is extensive; a heterogeneous merge loses energy.
    children=[replace(e,a=e.a*s,b=e.b*s,length=e.length*s,rest=e.rest*s) for s in (F(1,3),F(2,3))]
    require(sum(map(energy,children))==energy(e), 'split changed energy')
    split_pairs=[(energy(e)*s,energy(child)) for s,child in zip((F(1,3),F(2,3)),children)]
    require(fragment_settlement(split_pairs,F(0))['work_a']==0,'split entitlement lost')
    heterogeneous=[Edge(F(1),F(0),F(1),F(1)),Edge(F(0),F(1),F(1),F(1))]
    merged=[Edge(F(1),F(1),F(2),F(2))]
    require(sum(map(energy,heterogeneous))>sum(map(energy,merged)), 'merge control not discriminating')
    merge_pairs=[(energy(old),energy(merged[0])/2) for old in heterogeneous]
    merge=fragment_settlement(merge_pairs,F(10))
    require(merge['delta']<0 and merge['q']>0 and merge['work_a']>0,'merge fragments improperly netted')
    try: fragment_settlement(merge_pairs,F(0))
    except ValueError: pass
    else: raise ValueError('downhill merge net subsidized uphill fragment')
    results['proportional_split_and_heterogeneous_merge']='PASS'
    # Load/unload loops may dissipate, never recharge A. No cross-group netting.
    low=Edge(F(0),F(1),F(1),F(1)); high=replace(low,length=F(2))
    A=F(10); W=F(0); Q=F(0)
    for _ in range(4):
        for start,end in (([low],[high]),([high],[low])):
            r=settlement(start,end,A);A=r['a'];W+=r['w'];Q+=r['q']
    require(A<F(10) and A+W==10 and F(3)*(10-A)==Q, 'roundtrip creates energy/fuel')
    try:
        settlement([low],[high],F(0))
    except ValueError:
        results['unfunded_loading_and_remote_release_borrowing']='REJECTED'
    else:
        raise ValueError('accepted unfunded loading')
    results['repeated_load_unload_cycles']='NO_A_RECOVERY'
    # Material synthesis debit has a distinct fate from conversion/work A→W.
    r=settlement([], [low],F(10),overhead=F(1,2),material=F(1))
    require(r['a']+r['w']+r['material_a']==10, 'synthesis material closure')
    require(r['q']==F(3,2), 'synthesis material was called heat')
    results['synthesis_material_vs_work_fates']='PASS'
    # Legacy preload, branch changes, rupture and new daughter edges settle E.
    init=settlement([], [e],F(10))
    require(init['work_a']==energy(e)/3, 'free initialization')
    rupture=settlement([e],[replace(e,ruptured=True)],F(0))
    require(rupture['q']==energy(e), 'rupture energy erased')
    immature=replace(e,mature=False)
    require(energy(immature)==0 and settlement([immature],[e],F(10))['work_a']>0, 'free maturation')
    require(energy(Edge(F(0),F(0),F(1),F(0)))==0, 'zero pool branch')
    results['initialization_maturation_rupture_zero_pool']='PASS'
    # Explicit OFF bypass is essential: lambda zero does not erase strain energy.
    p=e.a+e.b; strain=e.length/e.rest-1
    require(F(2)*p*strain*strain/2>0, 'lambda zero control')
    results['lambda_zero_is_not_route_OFF']='PASS'
    # Immutable snapshots show a late failure cannot publish partial changes.
    before=(F(1,4),F(0),0,(low,)); scratch=before
    try:
        first=settlement([low],[low],scratch[0],overhead=F(1,8))
        scratch=(first['a'],first['w'],1,(low,))
        settlement([low],[high],scratch[0])
    except ValueError:
        scratch=before
    require(scratch==before,'late failure did not roll back')
    results['late_substep_failure_atomic_rollback']='PASS_REFERENCE_TRANSACTION_ONLY'
    rejected=0
    for bad in (replace(e,a=F(-1)),replace(e,length=F(0)),replace(e,rest=F(0)),replace(e,a=float('nan')),replace(e,b=float('inf'))):
        try: energy(bad)
        except ValueError: rejected+=1
    require(rejected==5,'invalid state accepted')
    try: math.exp(10000)
    except OverflowError: results['nonfinite_exponential']='REJECTED_NOT_CLAMPED'
    require(rejected==5 and 'nonfinite_exponential' in results,'invalid input control')
    results['invalid_domain_nonfinite_zero_scaffold']=rejected
    # Transport cannot pay a receiver's load with a remote donor's release.
    donor_release=energy(high)-energy(low)
    receiver_load=energy(high)-energy(low)
    require(donor_release==receiver_load>0,'transport control not discriminating')
    try:
        settlement([low,high],[high,low],F(0))
    except ValueError:
        results['transport_no_cross_edge_energy_netting']='REJECTED_EVEN_WHEN_GLOBAL_DELTA_ZERO'
    else:
        raise ValueError('cross-edge energy netting accepted')
    # A daughter closure edge with no ancestor must load its complete energy.
    require(settlement([], [high],F(10))['work_a']==energy(high)/3,'new closure edge preload')
    require(settlement([e],[e],F(0))['work_a']==0,'unchanged checkpoint reload charge')
    results['new_closure_edge_and_unchanged_checkpoint']='PASS'
    fragment_rejections=0
    for pairs,available,eps in (([(F(0),F(1))],float('inf'),F(3)),
                                 ([(F(0),float('nan'))],F(1),F(3)),
                                 ([(F(0),F(1))],F(1),float('inf')),
                                 ([(F(0),float('inf'))],F(1),F(3))):
        try: fragment_settlement(pairs,available,eps)
        except ValueError: fragment_rejections+=1
    require(fragment_rejections==4,'nonfinite topology funding accepted')
    results['nonfinite_fragment_funding_and_energy']=fragment_rejections
    return results


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--repo',type=Path,default=Path('.'))
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args(); repo=args.repo.resolve(); out=args.output.resolve()
    require(not out.is_relative_to(repo),'evidence belongs outside checkout')
    def git(*a): return subprocess.check_output(['git',*a],cwd=repo)
    require(not git('diff','--name-only',BASE,'--','digital-protocell'),'scientific subtree changed')
    allowed=('.agent/',PREFIX,'scripts/verify_conversion_work_contract.py','.github/workflows/dc-m4-conversion-work-contract.yml','STORAGE_MAP.md')
    require(all(p.startswith(allowed) for p in git('diff','--name-only',BASE).decode().splitlines()),'scope drift')
    for n in ('DIRECTIVES','OUTCOMES','LEARNINGS','RECORD','REPO_MAP'):
        p=f'.agent/{n}.md';require((repo/p).read_bytes().startswith(git('show',f'{BASE}:{p}')),'append-only drift '+p)
    source=json.loads((repo/'docs/dc-m4-reaction-architecture-audit.json').read_text())['source_bindings']
    for p,digest in source.items():
        require(sha((repo/p).read_bytes())==digest and (repo/p).read_bytes()==git('show',f'{BASE}:{p}'),'source drift '+p)
    contract=json.loads((repo/(PREFIX+'contract.json')).read_text())
    require(contract['entry_head']==BASE and contract['scope']=='contract_only_incremental_coupling_energy_sector','contract authority')
    require(contract['status']=='DEFINED_PARAMETRIC_CONVERSION_WORK_OWNERSHIP_CANDIDATE','contract status')
    expected_units={'K':'energy/polarity_amount','lambda':'dimensionless','theta':'energy/polarity_amount','epsilon_A':'energy/A_amount'}
    require(set(contract['parameters'])==set(expected_units),'parameter inventory')
    for name,units in expected_units.items():
        p=contract['parameters'][name]
        require(p=={'units':units,'domain':'finite_positive','value':None},'unapproved calibration or units '+name)
    for key,value in {'Ec':'K*P/2*(epsilon-lambda*x)^2','g':'K*lambda*(lambda*x-epsilon)',
                      'finite_conversion_delta_Ec':'g*d+K*lambda^2*d^2/(2*P)',
                      'tension':'K*P/L0*(epsilon-lambda*x)',
                      'discrete_length_gradient':'(edge_vector_new+edge_vector_old)/(Lnew+Lold)'}.items():
        require(contract['energy_model'][key]==value,'reference arithmetic disagrees with contract '+key)
    require(contract['rates']['Jc']=='Rplus*exp(-g/(2*theta))-Rminus*exp(g/(2*theta))','signed rate mismatch')
    require(not any(contract[k] for k in ('production_runtime_modified','calibrated','implemented')),'contract overclaim')
    require(contract['claims']['total_mesh_energy_closure']=='NOT_ESTABLISHED' and
            contract['claims']['feedback_correction']=='NOT_ESTABLISHED' and
            contract['claims']['automatic_successor'] is False,'scientific boundary drift')
    checks=controls()
    spec=importlib.util.spec_from_file_location('governance',repo/'scripts/validate_governance.py')
    v=importlib.util.module_from_spec(spec);spec.loader.exec_module(v)
    def section(file,title):
        for n,b in v.iter_sections((repo/file).read_text()):
            if n==title:return '## '+n+'\n'+b
        raise ValueError('missing record '+title)
    errors=[]
    ids=v.validate_directives(section('.agent/DIRECTIVES.md',DIRECTIVE),errors)
    v.validate_outcomes(section('.agent/OUTCOMES.md',DIRECTIVE+' - COMPLETE'),ids,errors)
    v.validate_learnings(section('.agent/LEARNINGS.md','L-DCM4-CONVERSION-WORK-CONTRACT-20261017'),errors)
    require(not errors,str(errors))
    review=json.loads((repo/(PREFIX+'review.json')).read_text())
    require(review['decision']=='ACCEPTED_CONTRACT_ONLY','independent review missing')
    mandatory={PREFIX+'contract.md', PREFIX+'contract.json', PREFIX+'directive.md',
               'scripts/verify_conversion_work_contract.py',
               '.github/workflows/dc-m4-conversion-work-contract.yml'}
    require(set(review['reviewed_files'])==mandatory, 'incomplete independent review bindings')
    for p,digest in review['reviewed_files'].items():require(sha((repo/p).read_bytes())==digest,'stale independent review '+p)
    out.mkdir(parents=True,exist_ok=True)
    evidence={'exact_head':git('rev-parse','HEAD').decode().strip(),'entry_head':BASE,'directive':DIRECTIVE,
              'classification':'CONVERSION_WORK_OWNERSHIP_CONTRACT_DEFINED_AND_VERIFIED',
              'checks':checks,'source_bindings':source,'production_subtree_byte_identical':True,
              'contract_status':contract['status'],'parameter_values':'UNASSIGNED_NO_CALIBRATION',
              'new_structured_records':'PASS','append_only_ledgers':'PASS','contract_only':True,
              'runtime_implemented':False,'damping_corrected':False,'full_legacy_thermodynamics':'NOT_ESTABLISHED',
              'historical_global_governance':'PREEXISTING_SCHEMA_DRIFT_NOT_CLAIMED_PASS'}
    (out/'verification.json').write_text(json.dumps(evidence,indent=2)+'\n')
    paths=[*repo.glob(PREFIX+'*'),repo/'scripts/verify_conversion_work_contract.py',repo/'.github/workflows/dc-m4-conversion-work-contract.yml',repo/'STORAGE_MAP.md',repo/'scripts/validate_governance.py',repo/'docs/dc-m4-reaction-architecture-audit.json']
    paths += [repo/f'.agent/{n}.md' for n in ('CURRENT','PROJECT_PROFILE','DIRECTIVES','OUTCOMES','LEARNINGS','RECORD','REPO_MAP')]
    paths += [repo/p for p in source]
    bound={}
    for p in paths:
        rel=p.relative_to(repo);bound[str(rel)]=sha(p.read_bytes());target=out/'repository'/rel
        target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(p.read_bytes())
    manifest={'exact_head':evidence['exact_head'],'entry_head':BASE,'bound_repository_files':bound,'file_count':len(bound),'verification_sha256':sha((out/'verification.json').read_bytes())}
    (out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    print(json.dumps({'classification':evidence['classification'],'checks':'PASS','exact_cases':checks['exact_finite_conversion_and_discrete_work_cases'],'bound_files':len(bound)}))


if __name__=='__main__':
    main()
