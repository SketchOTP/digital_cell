#!/usr/bin/env python3
"""Qualify measured opt-in native fixture; never promote design inputs to biology."""
import argparse,hashlib,json,math,pathlib,subprocess,sys
from calibration_native_checks import require,recover,holdout_errors,coarse_state_obstruction,native_fixture_oracle,rhs

def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def flat(d):return d['species']+[x for v in d['vertices'] for x in v]
def maximum(a,b):return max(abs(x-y) for x,y in zip(a,b))
def qualify(data,protocol):
    limits=protocol['thresholds']
    training=data['training_probes'];held=data['heldout_probes']
    require(len(training)==8 and len(held)==6,'sealed probe counts')
    for rows,strains,offsets in ((training,[-.15,-.05,.1,.2],[-.04,.04]),(held,[-.11,.04,.17],[-.025,.025])):
        require([(r['strain'],r['da']) for r in rows]==[(s,d) for s in strains for d in offsets],'sealed probe grid')
    fit=recover(training);err=holdout_errors(held,fit)
    require(max(abs(fit[k]-v) for k,v in [('K',2.),('lambda',.75),('theta',1.)])<=limits['max_parameter_recovery_error'],'designed recovery failed')
    require(err['max_force_error']<=limits['max_force_holdout_error'] and err['max_logratio_error']<=limits['max_logratio_holdout_error'],'held-out prediction failed')
    require(all(data['controls'].values()),'rejection/rollback controls')
    ref=native_fixture_oracle(data['initial_source'],data['initial_native_mesh']['vertices'],data['initial_native_mesh']['rest_lengths'],2.)
    results={};errors=[]
    for name,dt in [('coarse',.002),('fine',.001),('zero_lambda',.002),('zero_chemistry',.002)]:
        run=data['runs'][name];summary=run['summary'];rows=run['trajectory'];initial=run['initial'];previous=initial
        require(run['dt']==dt and run['horizon']==2. and len(rows)==round(2/dt),'sealed native campaign')
        require(summary['restart_equivalent'],'native restart replay')
        material=data['initial_native_mesh']['material_total'];max_identity=0.;max_ode=0.;drift=0.;balance=0.;abs_defect=0.;signed=0.;dc=0.;dm=0.;coupling=0.;peak=0.;max_native_euler=0.
        for n,row in enumerate(rows,1):
            d=row['diagnostics'];require(abs(row['time']-n*dt)<1e-10 and row['dt']==dt,'native timing')
            require(d['total_g']-previous['total_g']<=1e-12,'joint energy increased')
            require(all(x>0 and math.isfinite(x) for x in d['species']),'source positivity')
            for z in (d['species'],):
                drift=max(drift,abs(z[0]+z[1]+z[2]-.2),abs(z[4]+z[5]+z[1]+2*z[2]+2*z[3]-1.27))
            require(abs(row['native_material_total']-material)<=limits['max_material_moiety_drift'],'native material drift')
            max_identity=max(max_identity,abs(d['gradient_identity_error']))
            require(max(math.hypot(*f) for f in d['coupling_forces'])<=limits['native_force_cap'],'external force cap')
            # Recompute motion from actual emitted old native force, no adapter velocities trusted.
            predicted=[previous['vertices'][i][a]+dt*previous['combined_forces'][i][a] for i in range(6) for a in range(2)]
            max_native_euler=max(max_native_euler,maximum(predicted,[x for v in d['vertices'] for x in v]))
            if name in ('coarse','fine'):
                ode=rhs(flat(previous),previous['rest_lengths'],2.,.75,1.,1.)
                emitted=previous['species_derivative']+[x for v in previous['combined_forces'] for x in v]
                max_ode=max(max_ode,maximum(ode,emitted))
            step=d['total_g']-previous['total_g']+dt*(previous['dchem']+previous['dmech'])
            signed+=step;abs_defect+=abs(step);dc+=dt*previous['dchem'];dm+=dt*previous['dmech'];coupling+=row['coupling_work'];peak=max(peak,d['us'])
            balance=max(balance,abs(step-row['step_defect']),abs(signed-row['signed_defect']),abs(abs_defect-row['absolute_defect']),abs(dc-row['integrated_dchem']),abs(dm-row['integrated_dmech']))
            previous=d
        require(drift<=limits['max_material_moiety_drift'] and max_identity<=limits['max_instantaneous_identity_error'],'source invariant or instantaneous energy identity')
        require(balance<=1e-10 and max_native_euler<=1e-12 and max_ode<=1e-12,'independent native motion/ledger/ODE check')
        require(abs(signed)<=limits['max_native_signed_balance_defect'],'signed balance defect budget')
        require(maximum(flat(previous),flat(run['final']))<1e-14,'final trajectory endpoint')
        require(abs(summary['signed_defect']-signed)<1e-10 and abs(summary['peak_us']-peak)<1e-12,'summary consistency')
        if name in ('coarse','fine'):
            error=maximum(flat(previous),ref);errors.append(error)
            budget=limits['max_native_endpoint_reference_error_'+name]
            require(error<=budget and peak>=limits['min_receiver_loading'],'native endpoint or receiver loading')
        else:
            error=None;require(peak<=limits['max_zero_control_receiver_loading'],'negative control loads receiver')
        results[name]={'steps':len(rows),'max_source_drift':drift,'max_identity_error':max_identity,'max_independent_ode_error':max_ode,'max_native_euler_error':max_native_euler,'max_ledger_error':balance,'signed_defect':signed,'absolute_defect':abs_defect,'peak_us':peak,'coupling_work':coupling,'endpoint_rk4_error':error,'restart_equivalent':True}
    require(errors[1]<errors[0] and abs(results['fine']['signed_defect'])<abs(results['coarse']['signed_defect']),'native refinement convergence')
    return {'schema':'digital_cell_calibration_native_qualification_v1','status':'BOUNDED_DESIGNED_REFERENCE_RECOVERY_AND_NATIVE_MECHANICS_QUALIFIED','original_four_parameter_calibration_status':'PARTIAL_BLOCKED_EXACT_COARSE_MAPPING','recovery':fit,'heldout':err,'native_runs':results,'coarse_state_obstruction':coarse_state_obstruction(),'production_values':{'K':None,'lambda':None,'theta':None,'epsilon_A':None},'whole_organism_native_integration_qualified':False,'automatic_successor':False,'units':'published dimensionless chemical reference; V=1; native edge rest length and drag define computational mechanics units; no physical unit calibration'}

def main():
    p=argparse.ArgumentParser();p.add_argument('--input',required=True);p.add_argument('--output',required=True);p.add_argument('--binary',required=True);a=p.parse_args()
    root=pathlib.Path(__file__).resolve().parents[1];out=pathlib.Path(a.output);out.mkdir(parents=True,exist_ok=True)
    protocol=root/'docs/dc-m4-calibration-native-protocol.json';report=qualify(json.loads(pathlib.Path(a.input).read_text()),json.loads(protocol.read_text()))
    report['input_sha256']=sha(pathlib.Path(a.input));report['binary_sha256']=sha(pathlib.Path(a.binary));report['protocol_sha256']=sha(protocol)
    report['head']=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()
    (out/'qualification.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
if __name__=='__main__':main()
