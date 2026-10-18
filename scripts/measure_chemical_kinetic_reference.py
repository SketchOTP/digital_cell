#!/usr/bin/env python3
"""Parent-owned sealed computational measurement and independent numerical oracle."""
from __future__ import annotations
import argparse, csv, hashlib, json, math
from pathlib import Path
import chemical_kinetic_reference as model


def independent_rhs(y,c):
    E,ES,ESS,Pr,S,Sm,L=y;P=.2;a0=.1
    strain=(L-c.L0)/c.L0-c.lam*(ES+ESS-a0)/P
    g=-c.K*c.lam*strain
    forward=(S,Sm*E*math.exp(-g/2),Sm*ES,ESS*math.exp(g/2))
    reverse=(Sm,ES*math.exp(g/2),ESS,E*Pr*math.exp(-g/2))
    j1,j2,j3,j4=[c.rate_scale*(x-y) for x,y in zip(forward,reverse)]
    stretch=c.k_s*(L-c.L0)/c.L0*(c.phi if L>c.L0 else 1)
    velocity=-(c.K*P/c.L0*strain+stretch)/c.gamma
    return (-j2+j4,j2-j3,j3-j4,j4,-j1,j1-j2-j3,velocity)


def independent_midpoint(c,horizon,step=.001):
    # Different derivative implementation and integrator; no model helper calls.
    y=[.1,.05,.05,.01,1.,.1,c.L0];n=round(horizon/step);h=horizon/n
    for _ in range(n):
        k1=independent_rhs(y,c);mid=[v+h*k/2 for v,k in zip(y,k1)]
        k2=independent_rhs(mid,c);y=[v+h*k for v,k in zip(y,k2)]
    return y


def source_oracles():
    # Exact rational initial vector/derivative and an independent scalar bracket.
    from fractions import Fraction as F
    z=list(map(F,('0.1','0.05','0.05','0.01','1','0.1')))
    E,ES,ESS,Pr,S,Sm=z;j=(S-Sm,Sm*E-ES,Sm*ES-ESS,ESS-E*Pr)
    d=(-j[1]+j[3],j[1]-j[2],j[2]-j[3],j[3],-j[0],j[0]-j[1]-j[2])
    assert j==tuple(map(F,('.9','-.04','-.045','.049')))
    assert d==tuple(map(F,('.089','.005','-.094','.049','-.9','.985')))
    lo,hi=F(0),F(1)
    def poly(x):return 200*x**4+400*x**3+313*x*x+93*x-127
    for _ in range(100):
        m=(lo+hi)/2
        if poly(m)<0:lo=m
        else:hi=m
    s=float((lo+hi)/2);E=.2/(1+s+s*s)
    return {'initial_currents':[float(v) for v in j],'initial_derivative':[float(v) for v in d],
            'equilibrium':[E,s*E,s*s*E,s*s,s,s],
            'positive_root_enclosure':[str(lo),str(hi)],
            'method':'exact Fraction initial oracle and monotone positive-root bracket independent of ODE'}


def run(repo,out):
    repo=Path(repo).resolve();out=Path(out).resolve()
    if out.exists() or out.is_relative_to(repo):raise ValueError('fresh external output required')
    protocol_path=repo/'docs/dc-m4-chemical-kinetic-protocol.json'
    raw=protocol_path.read_bytes();protocol=json.loads(raw);bounds=protocol['acceptance'];oracle=source_oracles()
    results={};initial=list(protocol['published_initial'])+[1.]
    # Verify published initial current and stoichiometry without using a trajectory.
    base=model.Config(K=0,lam=0,k_s=1,L0=1,gamma=1,phi=.7,rate_scale=1)
    # Public evaluation details are checked through model tests; independent
    # physical derivative remains explicit above and in the source-oracle data.
    assert max(abs(a-b) for a,b in zip(independent_rhs(initial,base)[:6],oracle['initial_derivative']))<1e-14
    actual=model.evaluate(base,initial)
    assert max(abs(a-b) for a,b in zip(actual['currents'],oracle['initial_currents']))<1e-14
    assert max(abs(a-b) for a,b in zip(actual['ode'][:6],oracle['initial_derivative']))<1e-14
    for case in protocol['cases']:
        c=model.Config(**{k:v for k,v in case.items() if k not in ('id','horizon','steps','initial')})
        horizon=case.get('horizon',protocol['horizon']);steps=case.get('steps',protocol['steps'])
        kwargs={}
        if case.get('initial')=='independent_equilibrium_oracle':kwargs['initial_species']=tuple(oracle['equilibrium'])
        runs=[model.simulate(c,dt,horizon,sample_every=protocol['sample_every'],**kwargs) for dt in steps]
        results[case['id']]={'runs':runs,'refinement_endpoint_difference':max(abs(a-b) for a,b in zip(runs[0]['final_state'],runs[1]['final_state']))}
        assert results[case['id']]['refinement_endpoint_difference']<=bounds['max_step_refinement_endpoint_difference'],case['id']+' refinement'
    # Keep independent numerical comparison at the originally published initial
    # state; equilibrium uses the independent rational polynomial instead.
    independent={}
    for name in ('published_unloaded','coupled_reference'):
        case=next(c for c in protocol['cases'] if c['id']==name)
        c=model.Config(**{k:v for k,v in case.items() if k!='id'})
        endpoint=independent_midpoint(c,protocol['horizon']);measured=results[name]['runs'][1]['final_state']
        difference=max(abs(x-y) for x,y in zip(endpoint,measured));assert difference<=bounds['max_independent_solver_endpoint_difference'],name+' independent solver'
        independent[name]={'method':'independent explicit midpoint; independently written derivative','step':.001,'endpoint':endpoint,'max_endpoint_difference':difference}
    # Model summary field names are an explicit public measurement contract.
    for name,item in results.items():
        for run in item['runs']:
            s=run['summary']
            assert s['maximum_conservation_error']<=bounds['max_moiety_drift'],name+' moiety'
            assert s['maximum_G_increase']<=bounds['max_total_G_increase_per_step'],name+' free energy'
            assert s['maximum_absolute_energy_dissipation_defect']<=bounds['max_abs_total_balance_defect'],name+' balance'
            assert s['maximum_gradient_identity_error']<=bounds['max_instantaneous_thermodynamic_identity_residual'],name+' identity'
    assert results['coupled_reference']['runs'][1]['summary']['peak_Us']>bounds['min_coupled_receiver_peak_energy']
    for name in ('zero_lambda','zero_chemistry'):
        assert results[name]['runs'][1]['summary']['peak_Us']<=bounds['max_zero_lambda_and_zero_chemistry_receiver_energy']
    eq=results['equilibrium_unloaded']['runs'][1]['final_state'];assert max(abs(x-y) for x,y in zip(eq,oracle['equilibrium']+[1.]))<=bounds['max_equilibrium_state_drift']
    time_difference=max(abs(x-y) for x,y in zip(results['coupled_reference']['runs'][1]['final_state'],results['time_rescale']['runs'][1]['final_state']))
    assert time_difference<=bounds['max_time_rescale_endpoint_difference']
    out.mkdir(parents=True)
    for name,item in results.items():
        for i,run in enumerate(item['runs']):
            (out/f'{name}-step{i}.json').write_text(json.dumps(run,indent=2,allow_nan=False)+'\n')
    summary={'classification':'BUILT_AND_MEASURED_COMPUTATIONAL_CHEMICAL_MECHANICAL_REFERENCE_ONLY',
             'protocol_sha256':hashlib.sha256(raw).hexdigest(),'source_oracles':oracle,'independent_solver':independent,
             'cases':{name:{'refinement_endpoint_difference':item['refinement_endpoint_difference'],'summary':item['runs'][1]['summary'],'final_state':item['runs'][1]['final_state']} for name,item in results.items()},
             'time_rescale_max_endpoint_difference':time_difference,'protocol_acceptance':'PASS',
             'physical_apparatus_built':False,'target_calibration_parameters_identified':False,'production_values':dict.fromkeys(('K','lambda','theta','epsilon_A')),
             'source_affinity_model':'independent published full-state ideal closed CRN','kinetic_map':'reference channel affinities and reciprocal-work bias only; not native cubic-law identification',
             'chemical_heat_measured':False,'automatic_successor':False}
    (out/'measurements.json').write_text(json.dumps(summary,indent=2,allow_nan=False)+'\n')
    return summary

if __name__=='__main__':
    ap=argparse.ArgumentParser();ap.add_argument('--repo',default='.');ap.add_argument('--output',required=True);a=ap.parse_args()
    s=run(a.repo,a.output);print(json.dumps({'classification':s['classification'],'cases':len(s['cases']),'acceptance':s['protocol_acceptance']}))
