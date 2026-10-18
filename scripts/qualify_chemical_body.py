#!/usr/bin/env python3
"""Rebuild every native raw state and remesh event; independent RK4 endpoints."""
import argparse,hashlib,json,math,pathlib,subprocess
from calibration_native_checks import require
from qualify_calibration_native import finite_tree,compare_tree,maximum
from chemical_body_reference import model,source,birth,area,BIRTH_AREA,remap,oracle

def flat(d):return d['amounts']+[x for p in d['vertices'] for x in p]
def rest_birth():
    v=birth();return [math.hypot(v[(i+1)%24][0]-v[i][0],v[(i+1)%24][1]-v[i][1]) for i in range(24)]
def check(d,rest,split,prep,lam,rate):
    count=48 if split else 24;require(len(d['vertices'])==count and len(d['amounts'])==6,'joint state shape')
    require(all(len(p)==2 for p in d['vertices']) and all(x>0 for x in d['amounts']),'positive source/geometry shape')
    for k in ('native_forces','coupling_forces','osmotic_forces','external_forces','combined_forces','velocities'):
        require(len(d[k])==count and all(len(p)==2 for p in d[k]),'force shape '+k)
    for k in ('bare_forward','bare_reverse','forward','reverse','currents','affinities'):require(len(d[k])==4,'reaction shape '+k)
    require(maximum(d['rest_lengths'],rest)<1e-14 and maximum(d['structural_masses'],rest)<1e-14,'native birth-material fraction ownership')
    ancestry=[{'birth_edge':i//2,'start':(i%2)/2,'end':(i%2+1)/2} for i in range(48)] if split else [{'birth_edge':i,'start':0.,'end':1.} for i in range(24)]
    require(d['ancestry']==ancestry,'ancestral material arc mismatch');arc=[0,1] if split else [0];require(d['arc_indices']==arc,'coupling arc owner')
    require(area(d['vertices'])>0 and 0<d['volume']<10 and 0<d['outside_volume']<10,'finite bath/positive body')
    independent=model(d['amounts'],d['vertices'],rest,arc,prep,lam,rate)
    require(compare_tree(d,independent)<1e-11,'independent source/force/storage reconstruction')
    return independent

def qualify(data,protocol):
    finite_tree(data);lim=protocol['thresholds'];require(data['whole_organism_qualified'] is False and data['empirical_full_target_qualified'] is False,'scope overclaim')
    require(set(data['runs'])=={'coupled','zero_lambda','zero_chemistry','closed_equilibrium'},'sealed case inventory')
    endpoints={};results={};source_peak=0.;forcefd=0.;positive_omitted=0.
    for name in ('coupled','zero_lambda','zero_chemistry','closed_equilibrium'):
        prep='ClosedEquilibrium' if name=='closed_equilibrium' else 'Published';lam=0. if name=='zero_lambda' else .75;rate=0. if name=='zero_chemistry' else 1.
        if name in ('coupled','zero_lambda'):endpoints[name]=oracle(prep,lam,rate)
        results[name]={}
        require(set(data['runs'][name])=={'coarse','fine'},'sealed step inventory')
        for resolution,dt in (('coarse',.002),('fine',.001)):
            run=data['runs'][name][resolution];rows=run['trajectory'];require(run['dt']==dt and run['horizon']==2. and len(rows)==round(2/dt),'sealed horizon/step')
            rest=rest_birth();previous=run['initial'];require(maximum(previous['amounts'],source(prep))<1e-14 and maximum([x for p in previous['vertices'] for x in p],[x for p in birth() for x in p])<1e-14,'sealed birth source/body')
            old=check(previous,rest,False,prep,lam,rate);initial_g=old['total_g'];split=False;remesh_delta=0.;signed=0.;absolute=0.;dc=0.;dm=0.;peak=0.;identity=0.;omitted=0.;drift=0.;max_ledger=0.;cw=0.;ow=0.;nw=0.
            for i,row in enumerate(rows,1):
                d=row['diagnostics'];require(row['dt']==dt and abs(row['time']-i*dt)<1e-10,'time ownership')
                reconstructed=check(d,rest,split,prep,lam,rate)
                require(maximum(d['amounts'],[previous['amounts'][j]+dt*old['amount_derivative'][j] for j in range(6)])<1e-12,'independent chemical amount Euler update')
                require(maximum([x for p in d['vertices'] for x in p],[previous['vertices'][j][a]+dt*old['combined_forces'][j][a] for j in range(len(rest)) for a in range(2)])<=lim['max_native_vertex_euler_error'],'independent actual native vertex update')
                require(max(math.hypot(*p) for p in d['external_forces'])<=lim['native_external_force_cap'],'native combined force cap')
                require(d['total_g']-previous['total_g']<=1e-12,'joint G increased')
                work=lambda f:math.fsum(f[j][a]*(d['vertices'][j][a]-previous['vertices'][j][a]) for j in range(len(rest)) for a in range(2))
                wc,wo,wn=(work(old[k]) for k in ('coupling_forces','osmotic_forces','native_forces'));cw+=wc;ow+=wo;nw+=wn
                require(max(abs(row[k]-v) for k,v in [('coupling_work',wc),('osmotic_work',wo),('native_receiver_work',wn)])<1e-12,'conjugate force/displacement work')
                step=d['total_g']-previous['total_g']+dt*(old['dchem']+old['dmech']);signed+=step;absolute+=abs(step);dc+=dt*old['dchem'];dm+=dt*old['dmech']
                discrepancies=[abs(row[k]-v) for k,v in [('step_defect',step),('signed_step_defect',signed),('absolute_step_defect',absolute),('integrated_dchem',dc),('integrated_dmech',dm),('remesh_defect',remesh_delta)]];max_ledger=max(max_ledger,*discrepancies)
                n=d['amounts'];drift=max(drift,abs(sum(n[:3])-.2),abs(n[4]+n[5]+n[1]+2*n[2]+2*n[3]-1.27));identity=max(identity,abs(d['gradient_identity_error']));omitted=max(omitted,abs(d['omitted_osmotic_identity_error']));peak=max(peak,d['us'])
                previous=d;old=reconstructed
                if i==round(.2/dt):
                    event=run['remesh'];require(abs(event['time']-.2)<1e-10 and event['native_splits']==24,'actual native remesh event')
                    require(compare_tree(event['before'],d)<1e-12,'event entry differs from actual trajectory')
                    n2,v2,r2=remap(d['amounts'],d['vertices'],rest);after=event['after'];require(maximum(after['amounts'],n2)<1e-14 and maximum([x for p in after['vertices'] for x in p],[x for p in v2 for x in p])<1e-14,'native midpoint/source lineage')
                    m2=check(after,r2,True,prep,lam,rate);pullback=max(abs(m2['combined_forces'][2*j][a]+.5*m2['combined_forces'][(2*j-1)%48][a]+.5*m2['combined_forces'][2*j+1][a]-reconstructed['combined_forces'][j][a]) for j in range(24) for a in range(2))
                    delta=m2['total_g']-reconstructed['total_g'];adelta=area(v2)-area(d['vertices'])
                    require(abs(delta)<=lim['max_remesh_storage_change'] and abs(adelta)<=lim['max_remesh_area_change'] and pullback<=lim['max_force_pullback_error'],'native remesh energy/area/gradient pullback')
                    require(max(abs(event[k]-v) for k,v in [('storage_delta',delta),('area_delta',adelta),('force_pullback_error',pullback)])<1e-12 and event['ancestry']==after['ancestry'],'remesh receipt integrity')
                    remesh_delta+=delta;previous=after;old=m2;rest=r2;split=True
            require(maximum(flat(previous),flat(run['final']))<1e-14,'final snapshot mismatch');require(split and len(previous['vertices'])==48,'missing remesh continuation')
            require(max_ledger<1e-10 and drift<=lim['max_material_moiety_drift'] and identity<=lim['max_instantaneous_identity_error'],'ledger/material/instantaneous identity')
            require(abs(signed)<=lim['max_signed_step_balance_defect'] and abs(old['total_g']-initial_g+dc+dm-signed-remesh_delta)<1e-10,'unrepaired signed/free energy accounting')
            s=run['summary'];require(s['restart_equivalent'] is True,'full state replay')
            for k,v in [('peak_us',peak),('signed_step_defect',signed),('absolute_step_defect',absolute),('remesh_defect',remesh_delta),('integrated_dchem',dc),('integrated_dmech',dm),('coupling_work',cw),('osmotic_work',ow),('native_receiver_work',nw)]:require(abs(s[k]-v)<1e-10,'summary '+k)
            if name in endpoints:
                error=maximum(flat(previous),endpoints[name]);require(error<=lim['max_endpoint_reference_error_'+resolution],'separate RK4 endpoint budget')
            else:
                error=None;require(peak<=lim['max_static_control_receiver_energy'],'static finite-bath/equilibrium control')
            if name=='coupled':require(peak>=lim['min_coupled_receiver_loading'],'no receiver loading');positive_omitted=max(positive_omitted,omitted)
            results[name][resolution]={'accepted_steps':len(rows),'birth_vertices':24,'final_vertices':48,'native_midpoint_splits':24,'source_moiety_drift':drift,'max_gradient_identity_error':identity,'max_omitted_osmotic_identity_error':omitted,'max_independent_ledger_error':max_ledger,'signed_step_defect':signed,'absolute_step_defect':absolute,'remesh_defect':remesh_delta,'peak_us':peak,'net_coupling_work':cw,'net_osmotic_work':ow,'net_native_receiver_work':nw,'independent_rk4_endpoint_error':error,'final_volume':previous['volume'],'restart_equivalent':True}
        if name in endpoints:require(results[name]['fine']['independent_rk4_endpoint_error']<results[name]['coarse']['independent_rk4_endpoint_error'] and abs(results[name]['fine']['signed_step_defect'])<abs(results[name]['coarse']['signed_step_defect']),'time refinement')
    require(positive_omitted>1e-10,'omitting osmotic work control did not expose gap')
    c=data['counterexamples']['curved_merge'];require(c['native_merges']>=1,'missing actual curved merge diagnostic')
    def us(v,r):return math.fsum((math.hypot(v[(i+1)%len(v)][0]-v[i][0],v[(i+1)%len(v)][1]-v[i][1])-r[i])**2/(2*r[i]) for i in range(len(v)))
    before=us(c['vertices_before'],c['rest_lengths_before']);after=us(c['vertices_after'],c['rest_lengths_after']);require(abs(before-c['us_before'])<1e-12 and abs(after-c['us_after'])<1e-12 and after-before>1e-4,'unowned curved merge witness')
    maturation=(1-1.1)**2/(2*1.1);require(abs(data['counterexamples']['maturation']['us_after']-maturation)<1e-14,'local maturation energy witness')
    return {'schema':'digital_cell_full_state_body_measurement_v1','status':'BOUNDED_FULL_STATE_MOVING_VOLUME_NATIVE_BODY_AND_REMESH_QUALIFIED','runs':results,'actual_native_curved_merge_unpaid_storage':after-before,'local_maturation_unpaid_storage_witness':maturation,'chemical_source_volume_and_osmotic_work_owned':True,'remesh_drag_discretization_is_not_continuum_mesh_convergence':True,'whole_organism_qualified':False,'empirical_full_target_qualified':False,'production_values':dict.fromkeys(('K','lambda','theta','epsilon_A')),'automatic_successor':False}

def main():
    p=argparse.ArgumentParser();p.add_argument('--input',required=True);p.add_argument('--binary',required=True);p.add_argument('--output',required=True);a=p.parse_args();r=pathlib.Path(__file__).resolve().parents[1];data=json.loads(pathlib.Path(a.input).read_text());protocol=json.loads((r/'docs/dc-m4-empirical-organism-protocol.json').read_text());result=qualify(data,protocol)
    result.update(head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=r,text=True).strip(),input_sha256=hashlib.sha256(pathlib.Path(a.input).read_bytes()).hexdigest(),binary_sha256=hashlib.sha256(pathlib.Path(a.binary).read_bytes()).hexdigest());out=pathlib.Path(a.output);out.mkdir(parents=True,exist_ok=True);(out/'body-qualification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
if __name__=='__main__':main()
