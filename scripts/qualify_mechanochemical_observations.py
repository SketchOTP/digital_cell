#!/usr/bin/env python3
"""Independent raw TSV/decimal reconstruction; does not import the analyzer."""
import argparse,decimal,hashlib,json,pathlib,zipfile
D=decimal.Decimal
decimal.getcontext().prec=40

def require(v,m):
    if not v:raise ValueError(m)

def check(source,observations):
    root=pathlib.Path(__file__).resolve().parents[1];protocol=json.loads((root/'docs/dc-m4-mechanochemical-observation-protocol.json').read_text());obs=protocol['observations'];budget=protocol['numerical_budgets'];errors=[];pair_count=legs=gaps=0;counts=[]
    require(len(observations['records'])==2,'physical source count')
    with zipfile.ZipFile(pathlib.Path(source)/'raw-selected.zip') as z:
        for name,result in zip(protocol['selection']['measurement_files'],observations['records']):
            require(name==result['original_member'],'selected source substitution');rows=z.read(name).decode('utf-8-sig').splitlines();header=rows[0].split('\t');ix={k:header.index(k) for k in ('Status','A_dist-Y','B_dist-Y','Y_force','time(sec)')};runs=[];gap=False;ended=False;total=0;markers={'skipped':0,'end_write':0,'equal_timestamps':0,'reversed_timestamps':0};last_time=None
            for row_index,row in enumerate(rows[1:],2):
                if not row.strip():continue
                require(not ended,'data after original end marker')
                if row=='#Skipped':gap=True;markers['skipped']+=1;continue
                if row=='#End Write':ended=True;markers['end_write']+=1;continue
                fields=row.split('\t');require(len(fields)==len(header),'source width');s=D(fields[ix['Status']]);require(s==int(s),'status quantization');status=int(s);x=(D(fields[ix['A_dist-Y']])+D(fields[ix['B_dist-Y']]))/2;f=-D(fields[ix['Y_force']]);t=D(fields[ix['time(sec)']]);require(all(v.is_finite() for v in (x,f,t)),'raw finite')
                if last_time is not None:
                    markers['equal_timestamps']+=int(t==last_time);markers['reversed_timestamps']+=int(t<last_time)
                last_time=t;total+=1
                if not runs or runs[-1]['status']!=status:runs.append({'status':status,'start':row_index,'count':0,'work':D(0),'bridged':D(0),'gaps':0,'last':None})
                r=runs[-1];r['count']+=1
                if r['last'] is not None:
                    px,pf=r['last'];term=(f+pf)*(x-px)/2;r['bridged']+=term
                    if gap:r['gaps']+=1
                    else:r['work']+=term
                r['last']=(x,f);gap=False
            eligible=[]
            for i in range(len(runs)-1):
                a,b=runs[i:i+2]
                if i==0 or i+1==len(runs)-1:continue
                if a['status']>=1000 and a['status']%10==0 and b['status']==a['status']+1 and min(a['count'],b['count'])>=obs['minimum_samples_per_leg']:eligible.append((a,b))
            require(total==result['rows'] and len(runs)==result['status_groups'],'row/group count');require(markers==result['instrument_markers'],'instrument marker count');require(len(eligible)==result['eligible_complete_recorded_status_pairs'],'candidate count');selected=eligible[:obs['first_adjacent_complete_unzip_rezip_pairs_per_file']];require(len(selected)==len(result['selected_pairs']),'prospective selection')
            for raw,pair in zip(selected,result['selected_pairs']):
                pair_count+=1
                for r,reported in zip(raw,pair['legs']):
                    legs+=1;gaps+=r['gaps'];require(r['status']==reported['status'] and r['start']==reported['start_row'] and r['count']==reported['samples'] and r['gaps']==reported['gap_boundaries'],'raw leg identity')
                    require(reported['continuous_path_qualified']==(r['gaps']==0),'gap qualification')
                    for field,k in (('signed_instrument_coordinate_work_pN_nm','work'),('linearly_bridged_sensitivity_work_pN_nm','bridged')):
                        expected=float(r[k]);error=abs(reported[field]-expected);errors.append(error);require(error<=budget['independent_integral_absolute']+budget['independent_integral_relative']*abs(expected),'decimal work disagreement')
            counts.append(len(selected))
    require(observations['six_prospective_pairs_observed']==(counts==[3,3]),'coverage relabeling')
    for key in ('force_conjugate_external_work_qualified','absolute_fluid_temperature_qualified','molecular_or_equilibrium_chemical_free_energy_qualified','calorimetric_heat_qualified','chemical_kinetic_mapping_qualified','empirical_full_target_qualified'):require(observations[key] is False,'unsupported physical qualification '+key)
    require(observations['production_values']==dict.fromkeys(('K','lambda','theta','epsilon_A')),'production assignment')
    return {'scope':'independent decimal raw-observation reconstruction only','selected_pairs_by_file':counts,'selected_pairs':pair_count,'selected_legs':legs,'known_gap_boundaries_in_selected_legs':gaps,'maximum_work_error_vs_decimal':max(errors),'six_pair_prospective_coverage':counts==[3,3],'all_selected_paths_continuous':gaps==0,'force_conjugate_external_work_qualified':False,'chemical_kinetic_mapping_qualified':False,'empirical_full_target_qualified':False,'production_values':dict.fromkeys(('K','lambda','theta','epsilon_A'))}

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--source',required=True);p.add_argument('--input',required=True);p.add_argument('--output',required=True);a=p.parse_args();v=check(a.source,json.loads(pathlib.Path(a.input).read_text()));v['observations_sha256']=hashlib.sha256(pathlib.Path(a.input).read_bytes()).hexdigest();pathlib.Path(a.output).write_text(json.dumps(v,indent=2)+'\n');print(json.dumps(v))
