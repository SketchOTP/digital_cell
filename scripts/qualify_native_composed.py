#!/usr/bin/env python3
"""Independent raw-state/source/force/event reconstruction. No Rust model import."""
import argparse,copy,hashlib,json,math,pathlib,subprocess
from qualify_chemical_work_owners import area,elastic,force,require

def near(a,b,tol=1e-9):
    require(math.isfinite(a) and math.isfinite(b) and abs(a-b)<=tol,f'mismatch {a} {b}')
def same(a,b,tol=1e-9):
    if type(b) is bool:require(type(a) is bool and a==b,'boolean evidence')
    elif isinstance(b,dict):
        require(isinstance(a,dict) and set(a)==set(b),'object shape')
        for k in b:same(a[k],b[k],tol)
    elif isinstance(b,list):
        require(isinstance(a,list) and len(a)==len(b),'array shape')
        for x,y in zip(a,b):same(x,y,tol)
    elif type(b) in (float,int):near(a,b,tol)
    else:require(a==b,'value mismatch')
def finite_tree(v):
    if isinstance(v,dict):
        for x in v.values():finite_tree(x)
    elif isinstance(v,list):
        for x in v:finite_tree(x)
    elif type(v) in (float,int):require(math.isfinite(v),'nonfinite evidence')

def geometry_gradient_probe(w,p):
    """Independent central differentiation of the complete shared-bath potential."""
    worst=0.
    for ci,c in enumerate(w['cells']):
        for axis in (0,1):
            pm=[]
            for sign in (-1,1):
                trial=copy.deepcopy(w);trial['cells'][ci]['mesh']['vertices'][0][axis]+=sign*1e-5
                pm.append(reconstruct(trial,p,False)['G'])
            err=abs((pm[1]-pm[0])/2e-5+c['force_components']['total'][0][axis]);worst=max(worst,err)
    require(worst<=1e-6,'independent total-potential geometry gradient probe')
    return worst

def endpoint(w):
    # Include both daughters' structural amounts and inherited anchors, plus the
    # shared bath. Event origin labels may include resolution-specific revisions.
    out=[w['root_area']]+[w['outside'][k] for k in ('S','Sm','inert','V')]
    for c in w['cells']:
        out += [c['id'],c['P'],c['a0'],c['R_ref']]+c['n']+[x for v in c['mesh']['vertices'] for x in v]
        out += [e[k] for e in c['mesh']['edges'] for k in ('m','m_young','b','tracer_m','tracer_b')]
        out += [t[k] for tags in c['material_tags'] for t in tags for k in ('fraction','young','mature')]
    return out

def rejection_controls(data):
    controls=data['controls'];names=['prepared_growth_donor_infeasibility','prepared_transport_donor_infeasibility','evolved_growth_donor_infeasibility','evolved_transport_donor_infeasibility','newborn_absolute_floor_remesh','restart_config_tamper','restart_ledger_tamper','late_geometry_trial']
    require([c['name'] for c in controls]==names,'exact negative control inventory')
    run=data['cases'][0]['coarse'];evolved=run['steps'][9]['after']
    for i,c in enumerate(controls):
        require(c['rejected'] is True and c['unchanged'] is True and type(c['error']) is str and c['error'],'negative result evidence');same(c['before'],c['after'],0.)
        require(c['before_restart']==c['after_restart'],'serialized owner and ledger rollback');raw=json.loads(c['before_restart']);d=raw['data'];same(d['config'],c['config']);same(d['ledger'],c['before']['ledger']);near(d['root_area'],c['before']['root_area']);near(d['time'],c['before']['time']);near(d['revision'],c['before']['revision']);near(d['steps'],c['before']['steps'])
        for x,y in zip(d['cells'],c['before']['cells']):
            for a,b in [('id','id'),('mesh','mesh'),('n','n'),('p','P'),('a0','a0'),('r_ref','R_ref'),('tags','material_tags')]:same(x[a],y[b])
        reconstruct(c['before'],c['config'])
        if i<2:same(c['before'],run['initial'])
        elif i in (2,3,5,6):same(c['before'],evolved)
        if i in (0,2):same(c['event'],{'kind':'growth','cell':0,'edge':0,'extent':1000.});require(c['before']['cells'][0]['n'][5]<1000,'growth donor infeasibility')
        if i in (1,3):same(c['event'],{'kind':'transport','cell':0,'species':'Sm','direction':'in','amount':1000.});require(c['before']['outside']['Sm']<1000,'bath donor infeasibility')
        if i==4:
            same(c['event'],{'kind':'remesh','cell':1});require([x['id'] for x in c['before']['cells']]==[1,2] and c['before']['cells'][0]['rest_lengths'][-1]/2<.001,'newborn floor rejection witness')
        if i in (5,6):
            expected=copy.deepcopy(raw)
            if i==5:expected['data']['config']['k']=9.
            else:expected['data']['ledger']['event_delta_G_sum']=9.
            same(json.loads(c['attempted_restart']),expected)
        if i==7:near(c['config']['gamma'],1e-10);near(c['dt'],.001);near(c['before']['time'],1e-13);require(c['before']['steps']==1,'late geometry after evolved state')
    return len(controls)

NU=[[0,-1,0,1],[0,1,-1,0],[0,0,1,-1],[0,0,0,1],[-1,0,0,0],[1,-1,-1,0]]
def source(n,V,g,p):
    c=[x/V for x in n];a=[c[4],c[5]*c[0],c[5]*c[1],c[2]];b=[c[5],c[1],c[2],c[0]*c[3]]
    f=[V*p['source_rate_scale']*x*math.exp(-z*g/(2*p['theta'])) for x,z in zip(a,[0,1,0,-1])]
    r=[V*p['source_rate_scale']*x*math.exp(z*g/(2*p['theta'])) for x,z in zip(b,[0,1,0,-1])]
    aff=[math.log(x)-math.log(y)-z*g/p['theta'] for x,y,z in zip(a,b,[0,1,0,-1])]
    j=[x-y for x,y in zip(f,r)];nd=[sum(v*q for v,q in zip(row,j)) for row in NU]
    return f,r,aff,j,nd,p['theta']*sum(x*y for x,y in zip(j,aff))

def reconstruct(w,p,check=True):
    V=[area(c['mesh']['vertices'])/w['root_area'] for c in w['cells']];vo=p['total_volume']-sum(V);require(vo>0,'bath volume')
    ideal=lambda ns,v:p['theta']*sum(n*(math.log(n/v)-1) for n in ns if n>0)
    outside=w['outside']
    if check:near(outside['V'],vo)
    require(all(outside[k]>=0 for k in ('S','Sm','inert')),'nonnegative outside material');bulk=ideal([outside[k] for k in ('S','Sm','inert')],vo)
    U=Ec=solid=0.;enzyme=0.;substrate=outside['S']+outside['Sm'];origins={};cells=[]
    for c,vol in zip(w['cells'],V):
        m=c['mesh'];v=m['vertices'];n=c['n'];N=len(v);require(N==len(m['edges'])==len(c['material_tags']),'native shape');require(vol>0 and all(x>0 for x in n),'positive chemical state');near(m['rho_s'],p['rho'])
        L=[math.dist(a,b) for a,b in zip(v,v[1:]+v[:1])];h=[];y=[]
        for e,tags in zip(m['edges'],c['material_tags']):
            mature=sum(t['mature'] for t in tags);young=sum(t['young'] for t in tags);h.append(mature);y.append(young)
            near(e['m'],mature+young);near(e['m_young'],young);require(min(mature,young)>=-1e-12 and not e['ruptured'],'native material')
            for t in tags:origins[t['root']]=origins.get(t['root'],0)+t['fraction']
        rest=[max(x/p['rho'],1e-15) for x in h];ext=(sum(L)-sum(rest))/c['R_ref']-p['lambda']*(n[1]+n[2]-c['a0'])/c['P']
        ec=p['k']*c['P']*ext*ext/2;g=-p['k']*p['lambda']*ext;tension=p['k']*c['P']*ext/c['R_ref']
        un=sum(elastic(l,a,b,p['rho'],p['k_s']) for l,a,b in zip(L,h,y));gs=sum(p['mu_y']*b+p['mu_m']*a for a,b in zip(h,y));gb=ideal(n,vol)
        native=[[0.,0.] for _ in v];coupling=copy.deepcopy(native);osmotic=copy.deepcopy(native)
        pressure=p['theta']*(sum(n)/vol-sum(outside[k] for k in ('S','Sm','inert'))/vo)
        for i in range(N):
            j=(i+1)%N;prev=(i-1)%N
            for axis in (0,1):
                unit=(v[j][axis]-v[i][axis])/L[i];f=force(L[i],h[i],y[i],p['rho'],p['k_s'])*unit
                native[i][axis]+=f;native[j][axis]-=f;coupling[i][axis]+=tension*unit;coupling[j][axis]-=tension*unit
            grad=[(v[j][1]-v[prev][1])/2,(v[prev][0]-v[j][0])/2]
            osmotic[i]=[pressure*x/w['root_area'] for x in grad]
        external=[[a+b for a,b in zip(x,y)] for x,y in zip(coupling,osmotic)];total=[[a+b for a,b in zip(x,y)] for x,y in zip(native,external)]
        require(max(math.hypot(*f) for f in external)<=.5+1e-12,'native external force cap')
        vel=[[f/p['gamma'] for f in row] for row in total];f,r,aff,j,nd,dc=source(n,vol,g,p);dm=sum(z*z for row in total for z in row)/p['gamma']
        gradient=sum(q*(p['theta']*math.log(n[i]/vol)+(g if i in (1,2) else 0)) for i,q in enumerate(nd))-dm
        near(gradient+dc+dm,0);require(dc>=-1e-12,'chemical dissipation')
        components={'native':native,'coupling':coupling,'osmotic':osmotic,'external':external,'total':total}
        if check:
            for key,value in [('V',vol),('lengths',L),('rest_lengths',rest),('force_components',components),('velocity',vel),('forward',f),('reverse',r),('affinities',aff),('currents',j),('ndot',nd),('G_bulk',gb),('G_solid',gs),('U_native',un),('Ec',ec),('Dchem',dc),('Dmech',dm),('gradient_dot_ode',gradient),('gradient_identity_error',gradient+dc+dm),('g',g),('tension',tension)]:same(c[key],value,1e-8 if key=='U_native' else 1e-9)
        enzyme+=sum(n[:3]);near(c['P'],sum(n[:3]));substrate+=sum(x*z for x,z in zip(n,[0,1,2,2,1,1]))+sum(h)+sum(y);bulk+=gb;U+=un;solid+=gs;Ec+=ec
        cells.append({'id':c['id'],'ndot':nd,'vel':vel,'forces':components,'Dchem':dc,'Dmech':dm})
    for x in origins.values():near(x,1.,1e-9)
    stores={'bulk_ideal':bulk,'solid_standard':solid,'native_elastic':U,'new_coupling':Ec};G=sum(stores.values())
    invariants={'enzyme':enzyme,'substrate':substrate,'a0':sum(c['a0'] for c in w['cells']),'R_ref':sum(c['R_ref'] for c in w['cells']),'membrane':sum(e['b'] for c in w['cells'] for e in c['mesh']['edges'])}
    if check:near(w['G_total'],G,1e-8);same(w['global_invariants'],invariants)
    return {'G':G,'stores':stores,'cells':cells,'invariants':invariants,'origins':origins,'Dchem':sum(c['Dchem'] for c in cells),'Dmech':sum(c['Dmech'] for c in cells)}

def birth(w,p):
    require(w['time']==0 and w['steps']==0 and w['revision']==0 and len(w['cells'])==1,'prepared chronology')
    c=w['cells'][0];require(c['id']==0,'birth id');v=[[p['radius']*math.cos(i*math.pi/3),p['radius']*math.sin(i*math.pi/3)] for i in range(6)]
    same(c['mesh']['vertices'],v);near(w['root_area'],area(v));same(c['n'],[.1,.05,.05,.01,1.,10.]);near(c['P'],.2);near(c['a0'],.1);near(c['R_ref'],sum(math.dist(a,b) for a,b in zip(v,v[1:]+v[:1])))
    for i,t in enumerate(c['material_tags']):require(len(t)==1,'birth tags');near(t[0]['young'],.1);near(t[0]['mature'],p['rho']*math.dist(v[i],v[(i+1)%6]));near(t[0]['fraction'],1.)
    clarity=json.loads((pathlib.Path(__file__).resolve().parents[1]/'docs/dc-m4-native-composed-preparation-clarification.json').read_text())
    same({k:z for k,z in c['mesh'].items() if k not in ('vertices','edges')},clarity['birth_native_metadata_excluding_vertices_edges'])
    for i,(edge,tags) in enumerate(zip(c['mesh']['edges'],c['material_tags'])):
        L=math.dist(v[i],v[(i+1)%6]);same(edge,{'m':p['rho']*L+.1,'m_young':.1,'b':.7*L,'tracer_m':0.,'tracer_b':0.,'ruptured':False});require(tags[0]['root']==f'birth:{i}','exact birth origin')
    same(w['ledger'],dict.fromkeys(('integrated_Dchem','integrated_Dmech','continuous_signed_defect','continuous_abs_defect','event_delta_G_sum'),0.))
    for k,pk in [('S','outside_s'),('Sm','outside_sm'),('inert','outside_inert')]:near(w['outside'][k],p[pk])

def event_check(row,p,expected):
    b,a=row['before'],row['after'];e=row['event'];kind=expected['kind'];action={k:z for k,z in expected.items() if k not in ('time','native','vertices')}
    if kind=='fission':action.update(i=0,j=6)
    same(e,action)
    near(b['time'],expected['time']);near(a['time'],b['time']);near(a['revision'],b['revision']+1);near(a['steps'],b['steps']);near(a['root_area'],b['root_area'])
    wb,wa=reconstruct(b,p),reconstruct(a,p);same(wa['invariants'],wb['invariants']);dg=wa['G']-wb['G'];near(row['delta_G'],dg,1e-8)
    same(row['energy_changes'],{k:wa['stores'][k]-wb['stores'][k] for k in wb['stores']},1e-8)
    idx=next(i for i,c in enumerate(b['cells']) if c['id']==expected['cell']);bc=b['cells'][idx]
    if kind=='fission':
        require(len(a['cells'])==2 and [x['id'] for x in a['cells']]==[1,2],'daughter inventory');v=bc['mesh']['vertices'];cut=math.dist(v[0],v[6]);mass=p['rho']*cut;q=2*mass/p['growth_yield'];n=bc['n'].copy();n[5]-=q;n[4]+=(1-p['growth_yield'])*q;require(n[5]>=0,'closure donor')
        for ac,ix in zip(a['cells'],[list(range(7)),list(range(6,12))+[0]]):
            same(ac['mesh']['vertices'],[v[i] for i in ix]);same(ac['material_tags'][:-1],[bc['material_tags'][i] for i in ix[:-1]])
            same({k:z for k,z in ac['mesh'].items() if k not in ('vertices','edges')},{k:z for k,z in bc['mesh'].items() if k not in ('vertices','edges')})
            for actual,old in zip(ac['mesh']['edges'][:-1],[bc['mesh']['edges'][i] for i in ix[:-1]]):same(actual,old)
            tags=ac['material_tags'][-1];require(len(tags)==1 and tags[0]['root'] not in wb['origins'],'paid fresh closure tag');same({k:tags[0][k] for k in ('fraction','young','mature')},{'fraction':1.,'young':mass,'mature':0.})
            require(tags[0]['root']==f"closure:{a['revision']}:{ac['id']}",'exact closure owner');same(ac['mesh']['edges'][-1],{'m':mass,'m_young':mass,'b':0.,'tracer_m':0.,'tracer_b':0.,'ruptured':False})
            frac=area(ac['mesh']['vertices'])/area(v);same(ac['n'],[x*frac for x in n])
            for key in ('P','a0','R_ref'):near(ac[key],bc[key]*frac)
        same(a['outside'],b['outside'])
        mapping=[]
        for ac,ix in zip(a['cells'],[list(range(7)),list(range(6,12))+[0]]):
            for ne,oe in enumerate(ix[:-1]):mapping.append({'new_cell':ac['id'],'new_edge':ne,'parent_cell':0,'parent_edge':oe,'start':0.,'end':1.,'creation_tag':None})
            mapping.append({'new_cell':ac['id'],'new_edge':6,'parent_cell':None,'parent_edge':None,'start':0.,'end':1.,'creation_tag':ac['material_tags'][-1][0]['root']})
        same(row['mapping'],mapping)
    else:
        require([x['id'] for x in a['cells']]==[x['id'] for x in b['cells']],'cell inventory');ac=a['cells'][idx]
        for i,(old,new) in enumerate(zip(b['cells'],a['cells'])):
            if i!=idx:
                for key in ('mesh','n','P','a0','R_ref','material_tags'):same(new[key],old[key])
        for key in ('P','a0','R_ref'):near(ac[key],bc[key])
        same({k:z for k,z in ac['mesh'].items() if k not in ('vertices','edges')},{k:z for k,z in bc['mesh'].items() if k not in ('vertices','edges')})
        n=bc['n'].copy();tags=copy.deepcopy(bc['material_tags']);outside=copy.deepcopy(b['outside']);v=bc['mesh']['vertices']
        if kind=='transport':n[5]+=.2;outside['Sm']-=.2;same(ac['mesh'],bc['mesh'])
        elif kind=='growth':
            n[5]-=.2;n[4]+=(1-p['growth_yield'])*.2;new=ac['material_tags'][0][-1];require(new['root']==f"growth:{a['revision']}:0:0",'new growth origin');same({k:new[k] for k in ('fraction','young','mature')},{'fraction':1.,'young':p['growth_yield']*.2,'mature':0.});tags[0].append(new)
        elif kind=='maturation':
            i=expected['edge'];q=expected['extent'];Y=sum(t['young'] for t in tags[i]);require(Y>=q,'maturation donor')
            for t in tags[i]:z=q*t['young']/Y;t['young']-=z;t['mature']+=z
        elif kind=='remesh':
            require(len(v)==6 and len(ac['mesh']['vertices'])==12,'native6to12');vs=[];ts=[]
            for i in range(6):
                require(bc['rest_lengths'][i]/2>=.001,'remesh floor restriction');vs.extend([v[i],[(x+y)/2 for x,y in zip(v[i],v[(i+1)%6])]])
                for _ in (0,1):ts.append([{k:(z if k=='root' else z/2) for k,z in t.items()} for t in tags[i]])
            same(ac['mesh']['vertices'],vs);tags=ts;near(dg,0,1e-8)
            same(row['mapping'],[{'new_cell':0,'new_edge':2*i+j,'parent_cell':0,'parent_edge':i,'start':j/2,'end':(j+1)/2,'creation_tag':None} for i in range(6) for j in (0,1)])
            es=[]
            for edge in bc['mesh']['edges']:
                child={k:(z if type(z) is bool else z/2) for k,z in edge.items()};es.extend([child,child])
            same(ac['mesh']['edges'],es)
            pull=max(abs(ac['force_components']['total'][2*i][ax]+.5*ac['force_components']['total'][(2*i-1)%12][ax]+.5*ac['force_components']['total'][2*i+1][ax]-bc['force_components']['total'][i][ax]) for i in range(6) for ax in (0,1));near(row['remesh_pullback_error'],pull);require(pull<=1e-8,'remesh force pullback')
        else:raise ValueError('unknown event')
        if kind!='remesh':same(ac['mesh']['vertices'],v)
        same(ac['n'],n);same(ac['material_tags'],tags);same(a['outside'],outside)
        if kind in ('growth','maturation'):
            for i,(eb,ea) in enumerate(zip(bc['mesh']['edges'],ac['mesh']['edges'])):
                expected_edge=dict(eb)
                if i==expected['edge']:
                    expected_edge['m']=sum(t['young']+t['mature'] for t in tags[i]);expected_edge['m_young']=sum(t['young'] for t in tags[i])
                same(ea,expected_edge)
    if kind not in ('remesh','fission'):require(row['mapping']==[],'unexpected event remap')
    if kind!='remesh':
        pair=row['pair'];f,r=pair['forward'],pair['reverse'];require(min(f,r)>0,'positive finite pair rates');near(f+r,p['kappa']);near(math.log(f)-math.log(r),-dg/p['theta'],1e-9)
        near(pair['log_ratio'],-dg/p['theta']);same(pair['reference_occupancies'],[.5,.5]);near(pair['ensemble_forward_flux'],f/2);near(pair['ensemble_reverse_flux'],r/2);sigma=(f-r)/2*math.log(f/r);near(pair['ensemble_pair_entropy'],sigma);require(sigma>=-1e-12,'pair entropy witness')
    else:require(row['pair'] is None,'remesh is numerical, not reaction pair')
    same(a['ledger']['event_delta_G_sum'],b['ledger']['event_delta_G_sum']+dg,1e-8)
    for k in ('integrated_Dchem','integrated_Dmech','continuous_signed_defect','continuous_abs_defect'):near(a['ledger'][k],b['ledger'][k])
    return {'kind':kind,'cell':expected['cell'],'time':expected['time'],'delta_G':dg,'chemical_release':-sum(wa['stores'][k]-wb['stores'][k] for k in ('bulk_ideal','solid_standard')),'mechanical_storage_change':sum(wa['stores'][k]-wb['stores'][k] for k in ('native_elastic','new_coupling'))}

def qualify(data,protocol):
    finite_tree(data);require(data['autonomous_generator_qualified'] is False and data['empirical_full_target_qualified'] is False and data['whole_organism_qualified'] is False,'source target overclaim');require([c['name'] for c in data['cases']]==[c['name'] for c in protocol['cases']],'sealed cases');results={};totalsteps=0
    for case,seal in zip(data['cases'],protocol['cases']):
        p=case['config'];mapping={'K':'k','outside_S':'outside_s','outside_Sm':'outside_sm'}
        for key,value in protocol['config'].items():near(p[mapping.get(key,key)],seal['overrides'].get(key,value))
        runs={};endpoints={}
        for resolution,dt in zip(('coarse','fine'),protocol['dt']):
            run=case[resolution];prev=run['initial'];birth(prev,p);initial=reconstruct(prev,p);inv=initial['invariants'];schedule=protocol['schedule'] if seal['events'] else [];require(len(run['events'])==len(schedule),'event inventory')
            require(len(run['steps'])==round(protocol['horizon']/dt),'step inventory');ei=0;eventresults=[];peak=initial['stores']['native_elastic'];maxdrift=0.;maxledger=0.;dc=dm=signed=absolute=jumps=0.;work={k:0. for k in ('native','coupling','osmotic','drag')};fd=geometry_gradient_probe(prev,p)
            for i,row in enumerate(run['steps'],1):
                same(row['before'],prev);b=row['before'];a=row['after'];old=reconstruct(b,p);new=reconstruct(a,p);near(row['dt'],dt);near(a['time'],i*dt);near(a['steps'],i);near(a['revision'],b['revision']+1);near(a['root_area'],b['root_area']);same(a['outside'],b['outside']|{'V':a['outside']['V']})
                require([c['id'] for c in a['cells']]==[c['id'] for c in b['cells']],'continuous cell inventory')
                stepwork={k:0. for k in work}
                for bc,ac,rhs in zip(b['cells'],a['cells'],old['cells']):
                    same(ac['n'],[x+dt*y for x,y in zip(bc['n'],rhs['ndot'])]);same(ac['mesh']['vertices'],[[x+dt*u for x,u in zip(v,z)] for v,z in zip(bc['mesh']['vertices'],rhs['vel'])]);same(ac['mesh']['edges'],bc['mesh']['edges']);same(ac['material_tags'],bc['material_tags'])
                    same({k:v for k,v in ac['mesh'].items() if k!='vertices'},{k:v for k,v in bc['mesh'].items() if k!='vertices'})
                    for key in ('id','P','a0','R_ref'):near(ac[key],bc[key])
                    for key in ('native','coupling','osmotic'):
                        value=sum(f[ax]*(av[ax]-bv[ax]) for f,av,bv in zip(rhs['forces'][key],ac['mesh']['vertices'],bc['mesh']['vertices']) for ax in (0,1));stepwork[key]+=value
                    stepwork['drag']+=dt*rhs['Dmech']
                same(row['work_components'],stepwork)
                for key in work:work[key]+=stepwork[key]
                delta=new['G']-old['G'];defect=delta+dt*(old['Dchem']+old['Dmech']);near(row['delta_G'],delta,1e-8);near(row['continuous_defect'],defect,1e-8);dc+=dt*old['Dchem'];dm+=dt*old['Dmech'];signed+=defect;absolute+=abs(defect)
                expectedledger={'integrated_Dchem':dc,'integrated_Dmech':dm,'continuous_signed_defect':signed,'continuous_abs_defect':absolute,'event_delta_G_sum':jumps};same(a['ledger'],expectedledger,1e-8)
                same(new['invariants'],inv);maxdrift=max(maxdrift,max(abs(new['invariants'][k]-inv[k]) for k in inv));peak=max(peak,new['stores']['native_elastic']);prev=a
                while ei<len(schedule) and abs(schedule[ei]['time']-i*dt)<1e-10:
                    event=run['events'][ei];same(event['before'],prev);er=event_check(event,p,schedule[ei]);eventresults.append(er);jumps+=er['delta_G'];prev=event['after'];ei+=1
                    ev=reconstruct(prev,p);peak=max(peak,ev['stores']['native_elastic']);maxdrift=max(maxdrift,max(abs(ev['invariants'][k]-inv[k]) for k in inv))
                    if er['kind']=='fission':fd=max(fd,geometry_gradient_probe(prev,p))
                rec=reconstruct(prev,p);maxledger=max(maxledger,abs(rec['G']-initial['G']+dc+dm-signed-jumps));same(rec['invariants'],inv)
            require(ei==len(schedule),'missing scheduled event');same(run['final'],prev);require(maxledger<=1e-8,'global continuous plus finite jump ledger');require(maxdrift<=1e-9,'material drift');near(run['dt'],dt);near(run['horizon'],.6)
            summary=run['summary'];require(summary['restart_exact_replay'] is True,'native restart replay evidence')
            for key,value in [('continuous_steps',len(run['steps'])),('finite_events',len(eventresults)),('final_cells',len(prev['cells'])),('integrated_Dchem',dc),('integrated_Dmech',dm),('continuous_signed_defect',signed),('continuous_abs_defect',absolute),('event_delta_G_sum',jumps),('peak_native_storage',peak)]:near(summary[key],value,1e-8)
            near(summary['global_ledger_residual'],reconstruct(prev,p)['G']-initial['G']+dc+dm-signed-jumps,1e-8)
            bound=protocol['budgets'][resolution+'_cumulative_absolute_continuous_defect'];require(absolute<=bound,'sealed continuous defect bound')
            if seal['events']:require([c['id'] for c in prev['cells']]==[1,2] and all(c['U_native']>0 for c in prev['cells']),'continuing loaded daughters')
            endpoints[resolution]=endpoint(prev);fd=max(fd,geometry_gradient_probe(prev,p))
            runs[resolution]={'accepted_steps':len(run['steps']),'finite_events':len(eventresults),'event_results':eventresults,'peak_native_elastic':peak,'integrated_Dchem':dc,'integrated_Dmech':dm,'continuous_signed_defect':signed,'continuous_abs_defect':absolute,'event_delta_G_sum':jumps,'max_global_ledger_error':maxledger,'max_material_drift':maxdrift,'max_independent_geometry_gradient_probe_error':fd,'work':work,'final_G':reconstruct(prev,p)['G'],'final_cell_ids':[c['id'] for c in prev['cells']],'daughter_continuation_steps':round((.6-.3)/dt) if seal['events'] else 0}
            totalsteps+=len(run['steps'])
        coarse=runs['coarse']['continuous_abs_defect'];fine=runs['fine']['continuous_abs_defect'];require(fine<=.8*coarse+1e-12,'continuous defect refinement');require(len(endpoints['coarse'])==len(endpoints['fine']),'endpoint shape');error=max(abs(a-b) for a,b in zip(endpoints['coarse'],endpoints['fine']));require(error<=.02,'common complete daughter/source endpoint difference');runs['coarse_fine_endpoint_max_difference']=error;results[case['name']]=runs
    a=results['coupled']['fine']['event_results'];b=results['reversed_maturation_drive']['fine']['event_results'];require(a[2]['delta_G']<0<b[2]['delta_G'],'reversed maturation preference')
    controls=rejection_controls(data)
    return {'status':'BOUNDED_COMPOSED_NATIVE_HYBRID_QUALIFIED','cases':results,'accepted_native_steps':totalsteps,'prepared_case_count':5,'atomic_rejection_controls':controls,'native_full_autonomous_generator_qualified':False,'empirical_full_target_qualified':False,'whole_organism_qualified':False,'production_values':dict.fromkeys(('K','lambda','theta','epsilon_A')),'automatic_successor':False}

def main():
    ap=argparse.ArgumentParser();ap.add_argument('--input',required=True);ap.add_argument('--binary',required=True);ap.add_argument('--output',required=True);a=ap.parse_args();root=pathlib.Path(__file__).resolve().parents[1];v=qualify(json.loads(pathlib.Path(a.input).read_text()),json.loads((root/'docs/dc-m4-native-composed-protocol.json').read_text()));v.update(head=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),input_sha256=hashlib.sha256(pathlib.Path(a.input).read_bytes()).hexdigest(),binary_sha256=hashlib.sha256(pathlib.Path(a.binary).read_bytes()).hexdigest());out=pathlib.Path(a.output);out.mkdir(parents=True,exist_ok=True);(out/'qualification.json').write_text(json.dumps(v,indent=2)+'\n');print(json.dumps(v))
if __name__=='__main__':main()
