#!/usr/bin/env python3
"""Reference witnesses/provenance, not future calibration or solver qualification.

The fixed floating sqrt/log inverse is a synthetic smoke check only. Its local
1e-14 comparison is not the prospective protocol's certified interval/budget gate.
All force/work witnesses use exact rational arithmetic.
"""
import argparse
from fractions import Fraction as F
import hashlib
import importlib.util
import json
import math
from pathlib import Path
import subprocess

BASE='a007e7e08acc768f08b83d96139a7cea25f0e04e'
PREFIX='docs/dc-m4-work-port-'
DIRECTIVE='D-20261017-dcm4-work-port-feasibility-spec'


def require(ok,message):
    if not ok: raise ValueError(message)


def sha(data): return hashlib.sha256(data).hexdigest()


def reference_witnesses():
    checks={}
    # Synthetic anchors demonstrate inversion only; none is a project parameter.
    K,lam,theta,eps=F(3),F(1,4),F(3,2),F(6)
    e1,x1,e2,x2=F(1,5),F(1,4),F(3,10),F(1,2)
    y1=K*(e1-lam*x1);y2=K*(e2-lam*x2)
    det=e1*x2-e2*x1
    require(det!=0,'rank-deficient force anchors')
    inferred_K=(y1*x2-y2*x1)/det
    inferred_B=(e1*y2-e2*y1)/det
    require(inferred_K==K and -inferred_B/inferred_K==lam,'force-anchor inversion')
    g=K*lam*(lam*x1-e1);log_tilt=-g/theta
    require(g!=0 and -g/log_tilt==theta,'kinetic-scale inversion')
    work,Awork=F(3),F(1,2)
    require(work/Awork==eps,'independent work/fuel anchor')
    require(e1*(2*x1)-(2*e1)*x1==0,'rank-deficient control')
    checks['synthetic_four_parameter_inversion']='PASS_NOT_EMPIRICAL_CALIBRATION'
    checks['rank_deficient_force_and_zero_load_anchors']='REJECT_REQUIRED_NONZERO_RANK_AND_LOAD'
    def infer_theta(rplus,rminus,J,g,unclipped=True):
        require(all(math.isfinite(x) for x in (rplus,rminus,J,g)),'nonfinite kinetic input')
        require(rplus>0 and rminus>0 and g!=0 and unclipped,'invalid kinetic anchor')
        root=math.sqrt(J*J+4*rplus*rminus)
        y=2*rplus/(root+J) if J>=0 else (root-J)/(2*rminus)
        require(math.isfinite(y) and y>0 and y!=1,'uninformative kinetic anchor')
        estimated=g/(2*math.log(y))
        require(math.isfinite(estimated) and estimated>0,'inconsistent kinetic anchor')
        return estimated
    for y in (0.5,2.0):
        expected=1.5;g=2*expected*math.log(y);J=2/y-0.5*y
        require(abs(infer_theta(2,0.5,J,g)-expected)<1e-14,'net-rate inversion')
    rejected=0
    for args in ((2,0.5,0,0),(2,0.5,0,float('nan')),(2,0.5,0,1,False),(2,0.5,0,-1)):
        try:infer_theta(*args)
        except ValueError:rejected+=1
    require(rejected==4,'invalid net-rate anchor accepted')
    checks['net_rate_theta_inversion_both_load_signs']='SYNTHETIC_FLOAT_SMOKE_ONLY_NOT_A_QUALIFICATION_CERTIFICATE; FOUR_INVALID_ANCHORS_REJECTED'
    # Scaling K,theta,epsilonA leaves kinetic/fuel ratios but changes mechanics.
    for s in (F(1,8),F(2),F(7)):
        require(s*K/(s*theta)==K/theta and s*K/(s*eps)==K/eps,'ratio identity')
        require(s*y1!=y1,'mechanical scale confounding control')
    checks['common_energy_rescaling_with_frozen_mechanical_units']='NOT_A_REMOVABLE_PARAMETER_GAUGE'
    # Units are powers of (energy, length, time, A amount, polarity amount).
    force=(1,-1,0,0,0);gunit=(1,0,0,0,-1);epsunit=(1,0,0,-1,0)
    inv_effort_price=(1,0,1,-1,0)
    require(gunit!=epsunit and inv_effort_price!=epsunit,'unit conflation')
    require(tuple(inv_effort_price[j]-epsunit[j] for j in range(5))==(0,0,1,0,0),'missing timescale')
    checks['actuator_price_inverse_not_epsilonA']='EXTRA_TIME_DIMENSION_REJECTED'
    # Simultaneous overdamped work, including positive and negative crosswork.
    rows=[]
    for mu in (F(1,4),F(1),F(2)):
        for fc in (F(-1,4),F(1,4)):
            for legacy in (F(-1,2),F(-1,8),F(0),F(1,8),F(1,2)):
                dx=mu*(fc+legacy);wc=fc*dx;wl=legacy*dx;drag=dx*dx/mu
                require(wc+wl==drag,'combined force-work balance')
                rows.append((mu,fc,legacy,wc,wl,drag))
    require(len(rows)==30,'fixture matrix size')
    mu,fc,legacy=F(1),F(1,4),F(-1,8)
    dx=mu*(fc+legacy);wc=fc*dx;wl=legacy*dx;drag=dx*dx/mu
    require(wc==F(1,32) and wl==-F(1,64) and drag==F(1,64),'cap-respecting crosswork counterexample')
    require(wc>drag and wc!=mu*fc*fc,'all-release-to-bath or isolated bath assignment')
    checks['simultaneous_force_balance_exact_cases']=len(rows)
    checks['coupling_release_is_not_total_bath_heat']='NAIVE_ADAPTER_REJECTED_WITHIN_FORCE_CAP'
    dx=F(1,4)-F(1,4)
    require(dx==0 and F(1,4)*dx==0 and F(1,4)**2>0,'zero-net-force counterexample')
    checks['isolated_force_squared_bath_at_zero_velocity']='REJECTED'
    # Two actual quadratic elastic sectors under an exact scalar DG solve.
    def springs(kc,kl,qc,ql,mu=F(1)):
        d=mu*(kc*qc+kl*ql)/(1+mu*(kc+kl)/2)
        fc=kc*(qc-d/2);fl=kl*(ql-d/2)
        dc=kc*((d-qc)**2-qc**2)/2
        dl=kl*((d-ql)**2-ql**2)/2
        drag=d*d/mu
        require(fc*d==-dc and fl*d==-dl and dc+dl+drag==0,'two-spring DG balance')
        return {'dx':d,'fc':fc,'fl':fl,'delta_c':dc,'delta_legacy':dl,'drag':drag}
    release=springs(F(1,4),F(1,4),F(1),F(-1,2))
    require(release['delta_c']==-F(19,800) and release['delta_legacy']==F(11,800) and release['drag']==F(1,100),'release-to-legacy-storage example')
    require(-release['delta_c']!=release['drag'],'coupling release silently called heat')
    loading=springs(F(1,4),F(1,4),F(-1),F(2))
    require(loading['delta_c']==F(11,200) and loading['delta_legacy']==-F(19,200),'loading witness')
    legacy_input=-loading['delta_legacy'];claimed_Ainput=loading['delta_c']
    require(legacy_input==loading['delta_c']+loading['drag'],'legacy loading port balance')
    require(legacy_input+claimed_Ainput!=loading['delta_c']+loading['drag'],'unnamed A loading input not detected')
    checks['two_spring_storage_and_loading_work']='EXACT_REDUCED_FIXTURES_ONLY'
    checks['additional_A_input_without_physical_port']='REJECTED_DOUBLE_INPUT'
    # A finite vector discrete gradient, unlike a frozen starting tangent.
    r0=(F(3),F(4));r1=(F(0),F(13));L0,L1=F(5),F(13)
    v=tuple((r0[j]+r1[j])/(L0+L1) for j in range(2));dr=tuple(r1[j]-r0[j] for j in range(2))
    require(sum(v[j]*dr[j] for j in range(2))==L1-L0,'finite length gradient')
    require(sum(r0[j]/L0*dr[j] for j in range(2))!=L1-L0,'frozen tangent mutation')
    checks['noncollinear_discrete_work']='PASS_EXPLICIT_START_TANGENT_REJECTED'
    # Positive radial curvature does not imply coordinate-space convexity.
    stiffness=F(4);length=F(1,2);preferred=F(3,2)
    tension=stiffness*(length-preferred)
    require(stiffness>0 and tension/length<0,'negative tangential Hessian control')
    checks['radial_convexity_does_not_prove_vertex_uniqueness']='NEGATIVE_TANGENTIAL_HESSIAN'
    # Source interface bounds must not be repaired by clipping a signed force.
    cap=F(1,2);signed_tension=-F(1,4)
    require(signed_tension<0 and abs(signed_tension)<cap,'signed tension witness')
    require(F(3,8)+F(3,8)>cap,'shared external-port composition control')
    checks['signed_tension_and_aggregate_external_cap']='NAIVE_ADAPTER_REJECTED'
    # Decision tiers must never promote missing input or a reduced witness.
    def verdict(anchors,port,solver,full):
        if not anchors:return 'CALIBRATION_INPUTS_MISSING'
        if not port:return 'WORK_PORT_UNRESOLVED'
        if not solver:return 'SOLVER_FEASIBILITY_UNRESOLVED'
        if not full:return 'FULL_NATIVE_QUALIFICATION_NOT_RUN'
        return 'BOUNDED_MECHANICAL_FEASIBILITY_ONLY'
    require(verdict(False,True,True,True)=='CALIBRATION_INPUTS_MISSING','missing anchors promoted')
    require(verdict(True,False,True,True)=='WORK_PORT_UNRESOLVED','missing port promoted')
    require(verdict(True,True,False,True)=='SOLVER_FEASIBILITY_UNRESOLVED','missing solver promoted')
    require(verdict(True,True,True,False)=='FULL_NATIVE_QUALIFICATION_NOT_RUN','reduced fixture promoted')
    require(verdict(True,True,True,True)=='BOUNDED_MECHANICAL_FEASIBILITY_ONLY','mechanics promoted to biology')
    checks['missing_inputs_reduced_witnesses_and_tier_promotion']='REJECTED'
    return checks


def main():
    ap=argparse.ArgumentParser();ap.add_argument('--repo',type=Path,default=Path('.'));ap.add_argument('--output',type=Path,required=True)
    args=ap.parse_args();repo=args.repo.resolve();out=args.output.resolve()
    require(not out.is_relative_to(repo),'evidence belongs outside checkout')
    def git(*args):return subprocess.check_output(['git',*args],cwd=repo)
    require(not git('diff','--name-only',BASE,'--','digital-protocell'),'scientific subtree changed')
    allowed=('.agent/',PREFIX,'scripts/verify_work_port_specification.py','.github/workflows/dc-m4-work-port-specification.yml','STORAGE_MAP.md')
    require(all(p.startswith(allowed) for p in git('diff','--name-only',BASE).decode().splitlines()),'out-of-scope change')
    for name in ('DIRECTIVES','OUTCOMES','LEARNINGS','RECORD','REPO_MAP'):
        path=f'.agent/{name}.md';require((repo/path).read_bytes().startswith(git('show',f'{BASE}:{path}')),'history changed '+path)
    frozen=json.loads((repo/'docs/dc-m4-reaction-architecture-audit.json').read_text())['source_bindings'].copy()
    for path in ['docs/dc-m4-conversion-work-contract.md','docs/dc-m4-conversion-work-contract.json','docs/dc-m4-conversion-work-directive.md','docs/dc-m4-conversion-work-review.json','scripts/verify_conversion_work_contract.py','.github/workflows/dc-m4-conversion-work-contract.yml']:
        frozen[path]=sha(git('show',f'{BASE}:{path}'))
    for path,digest in frozen.items():require(sha((repo/path).read_bytes())==digest and (repo/path).read_bytes()==git('show',f'{BASE}:{path}'),'frozen input drift '+path)
    checks=reference_witnesses()
    specification=json.loads((repo/(PREFIX+'specification.json')).read_text())
    require(specification['entry_head']==BASE,'specification entry mismatch')
    require(specification['schema']=='digital_cell_prospective_parameter_work_port_protocol_v1' and
            specification['status']=='PROSPECTIVE_PROTOCOL_DEFINED','protocol schema/status drift')
    require(specification['production_values']=={'K':None,'lambda':None,'theta':None,'epsilon_A':None},'unsupported production values')
    require(specification['production_modified'] is False and specification['future_execution_authorized'] is False,'scope expansion')
    calibration=specification['calibration']
    require(calibration['independent_group_count']==4 and calibration['dimensionless_groups']==
            {'lambda':'lambda','kappa':'K*Pref/(Fref*Lref)','beta':'K/theta','eta':'K/(epsilon_A*c0)'},'dimensional group drift')
    require(calibration['anchors_currently_available'] is False and
            calibration['current_anchor_decision']=='CALIBRATION_ANCHOR_MISSING','missing anchors promoted')
    force=calibration['force'];xs=list(map(F,force['design_x']));strains=list(map(F,force['design_epsilon']));pairs=force['design_relative_P_L0']
    design=[(x,e,p,l) for x in xs for e in strains for p,l in pairs]
    train=[row for row in design if row[0] in (0,1) and row[1] in (0,F(1,4)) and row[2:]==(1,1)]
    require(len(design)==len(set(design))==force['rows']==80 and len(train)==4 and len(design)-len(train)==76,'calibration roles drift')
    require([d['stage'] for d in specification['decisions']]==['S0','S1','S2','S3','S4','S5'],'decision gate inventory')
    require(specification['port']['balance']=='Dgamma=Wlegacy+Wc+Wcontact' and
            specification['port']['global_legacy_potential_assumed'] is False,'power balance drift')
    for name,expected in {'cap_respecting_release':{'Fc':F(1,4),'Wc':F(1,32),'Wlegacy':-F(1,64),'Dgamma':F(1,64)},
                          'two_spring_release':{'DeltaEc':-F(19,800),'DeltaUlegacy':F(11,800),'Dgamma':F(1,100)},
                          'two_spring_loading':{'DeltaEc':F(11,200),'DeltaUlegacy':-F(19,200),'Dgamma':F(1,25)}}.items():
        require(all(F(specification['benchmarks'][name][key])==value for key,value in expected.items()),'fixture drift '+name)
    require(specification['current_adapter']['naive_bath_topup']=='NAIVE_PORT_REALIZATION_REJECTED' and
            specification['current_adapter']['whole_candidate_impossible'] is False,'adapter finding overclaim')
    require(specification['claims']['physical_calibration']=='NOT_PERFORMED' and
            specification['claims']['native_port_realization']=='NOT_ESTABLISHED' and
            specification['claims']['automatic_successor'] is False,'qualification drift')
    review=json.loads((repo/(PREFIX+'review.json')).read_text())
    require(review['decision']=='ACCEPTED_SPECIFICATION_ONLY','independent review missing')
    mandatory={PREFIX+'specification.md',PREFIX+'specification.json',PREFIX+'directive.md','scripts/verify_work_port_specification.py','.github/workflows/dc-m4-work-port-specification.yml'}
    require(set(review['reviewed_files'])==mandatory,'independent review incomplete')
    for path,digest in review['reviewed_files'].items():require(sha((repo/path).read_bytes())==digest,'stale review '+path)
    module=importlib.util.spec_from_file_location('governance',repo/'scripts/validate_governance.py');v=importlib.util.module_from_spec(module);module.loader.exec_module(v)
    def section(file,title):
        for n,b in v.iter_sections((repo/file).read_text()):
            if n==title:return '## '+n+'\n'+b
        raise ValueError('missing new record '+title)
    errors=[];ids=v.validate_directives(section('.agent/DIRECTIVES.md',DIRECTIVE),errors)
    v.validate_outcomes(section('.agent/OUTCOMES.md',DIRECTIVE+' - COMPLETE'),ids,errors)
    v.validate_learnings(section('.agent/LEARNINGS.md','L-DCM4-WORK-PORT-SPEC-20261017'),errors)
    require(not errors,str(errors))
    evidence={'exact_head':git('rev-parse','HEAD').decode().strip(),'entry_head':BASE,'directive':DIRECTIVE,
              'classification':'PROSPECTIVE_PARAMETER_AND_WORK_PORT_SPECIFICATION_VERIFIED',
              'reference_checks':checks,'frozen_input_bindings':frozen,'scientific_source_count':16,
              'force_calibration_design_rows':80,'force_training_rows':4,'force_holdout_rows':76,
              'current_naive_adapter':'REJECTED_NOT_UNIVERSAL_CANDIDATE_IMPOSSIBILITY',
              'production_subtree_byte_identical':True,'append_only_ledgers':'PASS','new_structured_records':'PASS',
              'parameters_identified':False,'physical_work_port_realized':False,'native_feasibility_qualified':False,
              'biological_correction_established':False,'runtime_implemented':False,'automatic_successor':False,
              'historical_governance':'PREEXISTING_DRIFT_NOT_CLAIMED_PASS'}
    out.mkdir(parents=True,exist_ok=True);(out/'verification.json').write_text(json.dumps(evidence,indent=2)+'\n')
    paths=[*repo.glob(PREFIX+'*'),repo/'scripts/verify_work_port_specification.py',repo/'.github/workflows/dc-m4-work-port-specification.yml',repo/'scripts/validate_governance.py',repo/'STORAGE_MAP.md',repo/'docs/dc-m4-reaction-architecture-audit.json']
    paths += [repo/f'.agent/{n}.md' for n in ('CURRENT','PROJECT_PROFILE','DIRECTIVES','OUTCOMES','LEARNINGS','RECORD','REPO_MAP')]+[repo/p for p in frozen]
    bound={}
    for path in paths:
        rel=path.relative_to(repo);bound[str(rel)]=sha(path.read_bytes());target=out/'repository'/rel;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(path.read_bytes())
    manifest={'exact_head':evidence['exact_head'],'entry_head':BASE,'file_count':len(bound),'bound_repository_files':bound,'verification_sha256':sha((out/'verification.json').read_bytes())}
    (out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n');print(json.dumps({'checks':'PASS','classification':evidence['classification'],'exact_force_cases':30,'control_groups':len(checks),'bound_files':len(bound)}))


if __name__=='__main__':main()
