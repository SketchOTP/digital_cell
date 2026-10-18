"""Prospectively selected instrument-coordinate work, not chemical free energy."""
import argparse,hashlib,io,json,math,pathlib,statistics,zipfile

def require(v,m):
    if not v:raise ValueError(m)

def chunks(points):
    result=[]
    for p in points:
        if not result or len(p)>3 and p[3]:result.append([])
        result[-1].append(p)
    return result

def path_work(points,stride=1):
    terms=[]
    for block in chunks(points):
        q=block[::stride]
        if q[-1]!=block[-1]:q=q+[block[-1]]
        terms.extend((b[1]+a[1])*(b[0]-a[0])/2 for a,b in zip(q,q[1:]))
    return math.fsum(terms)

def independent_work(points):
    # Integration by parts uses the other conjugate coordinate and boundary term.
    return math.fsum(block[-1][0]*block[-1][1]-block[0][0]*block[0][1]-math.fsum((a[0]+b[0])*(b[1]-a[1])/2 for a,b in zip(block,block[1:])) for block in chunks(points))

def window_work(points,lo,hi):
    require(math.isfinite(lo) and math.isfinite(hi) and hi>lo,'empty coordinate overlap')
    terms=[]
    for a,b in zip(points,points[1:]):
        if len(b)>3 and b[3]:continue
        dx=b[0]-a[0]
        if dx==0:continue
        t0,t1=sorted(((lo-a[0])/dx,(hi-a[0])/dx));t0=max(0.,t0);t1=min(1.,t1)
        if t0<t1:
            f0=a[1]+t0*(b[1]-a[1]);f1=a[1]+t1*(b[1]-a[1]);terms.append((f0+f1)*dx*(t1-t0)/2)
    return math.fsum(terms)

def group_rows(stream):
    text=io.TextIOWrapper(stream,encoding='utf-8-sig');header=text.readline().strip().split()
    required=('A_dist-Y','B_dist-Y','Y_force','time(sec)','Status')
    require(all(v in header for v in required),'missing source columns')
    cols={k:header.index(k) for k in required};groups=[];count=0;bad_time=0;last_time=None;pending_gap=False;ended=False;markers={'skipped':0,'end_write':0,'equal_timestamps':0,'reversed_timestamps':0}
    for lineno,line in enumerate(text,2):
        if not line.strip():continue
        require(not ended,'non-whitespace data after terminal #End Write')
        if line.strip()=='#Skipped':markers['skipped']+=1;pending_gap=True;continue
        if line.strip()=='#End Write':markers['end_write']+=1;pending_gap=True;ended=True;continue
        values=line.split();require(len(values)==len(header),f'row width {lineno}')
        x,y,f,t,s=(float(values[cols[k]]) for k in required)
        require(all(math.isfinite(v) for v in (x,y,f,t,s)) and int(s)==s,'nonfinite/inexact status')
        if last_time is not None and t<=last_time:bad_time+=1
        if last_time is not None and t==last_time:markers['equal_timestamps']+=1
        if last_time is not None and t<last_time:markers['reversed_timestamps']+=1
        last_time=t;status=int(s);count+=1;point=((x+y)/2,-f,t,pending_gap);pending_gap=False
        if not groups or groups[-1]['status']!=status:groups.append({'status':status,'start_row':lineno,'points':[]})
        groups[-1]['points'].append(point)
    return header,groups,count,bad_time,markers

def selected_pairs(groups,minimum,limit):
    candidates=[]
    # Both status boundaries must be observed; exclude a file-truncated first/last block.
    for i in range(1,len(groups)-2):
        a,b=groups[i:i+2];s=a['status']
        if s>=1000 and s%10==0 and b['status']==s+1 and min(len(a['points']),len(b['points']))>=minimum:
            candidates.append((i,a,b))
    return candidates[:limit],len(candidates)

def analyze(source,protocol):
    root=pathlib.Path(source);receipt=json.loads((root/'acquisition-receipt.json').read_text());proof=json.loads((pathlib.Path(__file__).resolve().parents[1]/'docs/dc-m4-mechanochemical-unit-evidence.json').read_text())
    require(receipt['source_full_md5_verified']==protocol['source']['md5'] and receipt['source_full_bytes']==protocol['source']['bytes'] and receipt['whole_publisher_zip_crc_verified'] is True,'publisher source identity')
    require(receipt['external_source_software_executed'] is False and proof['physical_output_units_supported'] is True,'source/unit role')
    records=[];all_complete=True;obs=protocol['observations'];budgets=protocol['numerical_budgets'];maximum_error=0.
    with zipfile.ZipFile(root/'raw-selected.zip') as z:
        expected=receipt['selected_original_members'];require(set(z.namelist())==set(expected),'selected original member coverage')
        for name,b in expected.items():
            data=z.read(name);require(len(data)==b['bytes'] and hashlib.sha256(data).hexdigest()==b['sha256'],'original raw hash '+name)
        for name in protocol['selection']['measurement_files']:
            with z.open(name) as f:header,groups,count,bad_time,markers=group_rows(f)
            pairs,eligible=selected_pairs(groups,obs['minimum_samples_per_leg'],obs['first_adjacent_complete_unzip_rezip_pairs_per_file']);all_complete&=len(pairs)==obs['first_adjacent_complete_unzip_rezip_pairs_per_file'];result=[]
            for idx,a,b in pairs:
                legs=[]
                for g in (a,b):
                    points=g['points'];primary=path_work(points);independent=independent_work(points);error=abs(primary-independent);maximum_error=max(maximum_error,error)
                    require(error<=budgets['independent_integral_absolute']+budgets['independent_integral_relative']*abs(primary),'independent path integral disagreement')
                    gaps=sum(bool(p[3]) for p in points[1:]);bridged=[p[:3] for p in points]
                    legs.append({'status':g['status'],'start_row':g['start_row'],'samples':len(points),'duration_seconds':points[-1][2]-points[0][2],'start':points[0],'end':points[-1],'coordinate_range_nm':[min(p[0] for p in points),max(p[0] for p in points)],'force_range_pN':[min(p[1] for p in points),max(p[1] for p in points)],'signed_instrument_coordinate_work_pN_nm':primary,'work_role':'sum over observed contiguous chunks; missing intervals excluded','continuous_path_qualified':gaps==0,'gap_boundaries':gaps,'linearly_bridged_sensitivity_work_pN_nm':path_work(bridged),'independent_work_pN_nm':independent,'independent_absolute_error':error,'coarsened_work_pN_nm':{str(k):path_work(points,k) for k in (2,4)},'sample_time_median_seconds':statistics.median(b[2]-a[2] for a,b in zip(points,points[1:]))})
                lo=max(legs[0]['coordinate_range_nm'][0],legs[1]['coordinate_range_nm'][0]);hi=min(legs[0]['coordinate_range_nm'][1],legs[1]['coordinate_range_nm'][1]);overlap=None
                if hi>lo:
                    wa=window_work(a['points'],lo,hi);wb=window_work(b['points'],lo,hi);overlap={'range_nm':[lo,hi],'unzip_signed_work_pN_nm':wa,'rezip_signed_work_pN_nm':wb,'signed_branch_sum_pN_nm':wa+wb,'meaning':'clipped instrument-coordinate path work; not calorimetric dissipation or equilibrium chemical free energy'}
                result.append({'group_index':idx,'laser_power_code':a['status']//10%100,'legs':legs,'signed_leg_work_sum_pN_nm':sum(v['signed_instrument_coordinate_work_pN_nm'] for v in legs),'endpoint_coordinate_mismatch_nm':b['points'][-1][0]-a['points'][0][0],'endpoint_force_mismatch_pN':b['points'][-1][1]-a['points'][0][1],'common_coordinate_window':overlap})
            records.append({'original_member':name,'rows':count,'status_groups':len(groups),'eligible_complete_recorded_status_pairs':eligible,'selected_pairs':result,'nonincreasing_timestamps':bad_time,'instrument_markers':markers,'header':header})
    return {'scope':'independent physical DNA instrument-coordinate work observations only','publisher_doi':protocol['source']['doi'],'source_full_md5':receipt['source_full_md5_verified'],'records':records,'six_prospective_pairs_observed':bool(all_complete),'all_selected_leg_paths_continuous':all(l['continuous_path_qualified'] for r in records for p in r['selected_pairs'] for l in p['legs']),'independent_integral_max_absolute_error':maximum_error,'physical_units':{'force':'pN','position':'nm','time':'s','work':'pN nm = 1e-21 J'},'unit_proof':proof,'force_conjugate_external_work_qualified':False,'absolute_fluid_temperature_qualified':False,'molecular_or_equilibrium_chemical_free_energy_qualified':False,'calorimetric_heat_qualified':False,'chemical_kinetic_mapping_qualified':False,'empirical_full_target_qualified':False,'production_values':dict.fromkeys(('K','lambda','theta','epsilon_A')),'limits':['Two preselected molecular files only; same-session cycles are correlated','Known #Skipped markers split integration into observed chunks; missing work is unmeasured, bridged sensitivity is not primary','Recorded force/position scales are not independently recalibrated here','Instrument coordinate includes apparatus/handles; no molecular-only work or free-energy subtraction','Temperature sensor channels are not an independently established local fluid temperature','Signed branch work is preserved without forcing positive hysteresis','Publisher metadata says3653bp while article says3593bp; no sequence-length thermodynamic inference']}

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--source',required=True);p.add_argument('--output',required=True);a=p.parse_args();root=pathlib.Path(__file__).resolve().parents[1];v=analyze(a.source,json.loads((root/'docs/dc-m4-mechanochemical-observation-protocol.json').read_text()));pathlib.Path(a.output).write_text(json.dumps(v,indent=2)+'\n');print(json.dumps({'six_pairs':v['six_prospective_pairs_observed'],'integral_max_error':v['independent_integral_max_absolute_error'],'pairs':[[p['signed_leg_work_sum_pN_nm'] for p in r['selected_pairs']] for r in v['records']]}))
