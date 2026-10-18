#!/usr/bin/env python3
"""Parent-authored oracle: raw worlds/native probes, independent quadrature.

Does not import the implementation's energy, invariants or proposal functions.
"""
import argparse,hashlib,json,math,pathlib,subprocess

def require(v,m):
    if not v:raise ValueError(m)

def near(a,b,tol=1e-10):
    require(math.isfinite(a) and math.isfinite(b) and abs(a-b)<=tol,f'number mismatch {a} {b}')

def same(a,b):
    if type(b) is dict:
        require(type(a) is dict and set(a)==set(b),'object shape')
        for k in b:same(a[k],b[k])
    elif type(b) is list:
        require(type(a) is list and len(a)==len(b),'array shape')
        for x,y in zip(a,b):same(x,y)
    elif type(b) in (float,int) and type(b) is not bool:near(a,b)
    else:require(a==b,'value mismatch')

def prepared(p):
    vertices=[[math.cos(i*math.pi/3),math.sin(i*math.pi/3)] for i in range(6)];ls=[math.dist(a,b) for a,b in zip(vertices,vertices[1:]+vertices[:1])]
    return {'owner':'reference-root','params':p,'root_area':area(vertices),'outside_s':10.,'outside_sm':10.,'outside_inert':10.,'revision':0,'cells':[{'id':'cell:0','vertices':vertices,'edges':[{'id':f'edge:{i}','tags':[{'root':f'birth:{i}','fraction':1.,'young':.1,'mature':L*p['rho']}]} for i,L in enumerate(ls)],'n':[.1,.05,.05,.01,1.,10.],'P':.2,'a0':.1,'R_ref':sum(ls)}]}

def area(v):
    o=v[0];return math.fsum((a[0]-o[0])*(b[1]-o[1])-(a[1]-o[1])*(b[0]-o[0]) for a,b in zip(v[1:],v[2:]))/2

def amounts(e):return [math.fsum(t[k] for t in e['tags']) for k in ('mature','young')]

def force(L,h,y,rho=1.,ks=1.):
    r=max(h/max(rho,1e-15),1e-15);factor=1 if L<r else h/(h+y) if h+y else 0.
    return ks*(L-r)/max(r,L/4,.001)*factor

def integrate(f,a,b,tol=1e-12):
    if a==b:return 0.
    if a>b:return -integrate(f,b,a,tol)
    def sim(a,b,fa,fm,fb):return (b-a)*(fa+4*fm+fb)/6
    def rec(a,b,fa,fm,fb,old,tol,depth):
        m=(a+b)/2;fl=f((a+m)/2);fr=f((m+b)/2);left=sim(a,m,fa,fl,fm);right=sim(m,b,fm,fr,fb);err=left+right-old
        if abs(err)<=15*tol:return left+right+err/15
        require(depth>0,'independent quadrature did not converge')
        return rec(a,m,fa,fl,fm,left,tol/2,depth-1)+rec(m,b,fm,fr,fb,right,tol/2,depth-1)
    fa,fm,fb=f(a),f((a+b)/2),f(b);return rec(a,b,fa,fm,fb,sim(a,b,fa,fm,fb),tol,25)

def elastic(L,h,y,rho=1.,ks=1.):
    r=max(h/max(rho,1e-15),1e-15);a,b=sorted((r,L));cuts=[a]+[v for v in (4*max(r,.001),) if a<v<b]+[b]
    result=math.fsum(integrate(lambda x:force(x,h,y,rho,ks),lo,hi) for lo,hi in zip(cuts,cuts[1:]));return result if L>=r else -result

def world(w):
    p=w['params'];volumes=[area(c['vertices'])/w['root_area'] for c in w['cells']];out=p['total_volume']-sum(volumes);require(out>0,'outsidevolume')
    def ideal(n,V):return p['theta']*math.fsum(z*(math.log(z)-math.log(V)-1) for z in n if z>0)
    bulk=ideal([w[k] for k in ('outside_s','outside_sm','outside_inert')],out);solid=us=ec=0.;q=w['outside_s']+w['outside_sm'];enzyme=0.;origins={}
    for c,V in zip(w['cells'],volumes):
        require(V>0 and all(z>=0 for z in c['n']),'source/volume domain');bulk+=ideal(c['n'],V);enzyme+=sum(c['n'][:3]);near(c['P'],sum(c['n'][:3]));ls=[math.dist(a,b) for a,b in zip(c['vertices'],c['vertices'][1:]+c['vertices'][:1])];rs=[]
        q+=sum(a*b for a,b in zip(c['n'],(0,1,2,2,1,1)))
        for L,e in zip(ls,c['edges']):
            h,y=amounts(e);require(min(h,y)>=0,'negative material');q+=h+y;solid+=p['mu_y']*y+p['mu_m']*h;rs.append(max(h/max(p['rho'],1e-15),1e-15));us+=elastic(L,h,y,p['rho'],p['k_s'])
            for t in e['tags']:origins[t['root']]=origins.get(t['root'],0.)+t['fraction']
        ext=(sum(ls)-sum(rs))/c['R_ref']-p['lam']*(sum(c['n'][1:3])-c['a0'])/c['P'];ec+=p['K']*c['P']*ext**2/2
    for f in origins.values():near(f,1.,1e-12)
    stores={'bulk_ideal':bulk,'solid_standard':solid,'native_elastic':us,'new_coupling':ec}
    return {'stores':stores,'total':math.fsum(stores.values()),'invariants':{'enzyme':enzyme,'substrate':q,'a0':sum(c['a0'] for c in w['cells']),'R_ref':sum(c['R_ref'] for c in w['cells'])},'volumes':volumes,'outside_volume':out,'origins':origins}

def physical(w):return {k:v for k,v in w.items() if k!='revision'}
def hash_world(w):return hashlib.sha256(json.dumps(w,sort_keys=True,separators=(',',':'),allow_nan=False).encode()).hexdigest()

def pair(v,dg,p):
    require(v['forward']>0 and v['reverse']>0,'zero kinetic channel')
    near(math.log(v['forward'])-math.log(v['reverse']),-dg/p['theta']);near(v['delta_g'],dg);near(v['log_ratio'],-dg/p['theta']);near(v['forward']+v['reverse'],p['kappa'])
    kf=p['kappa']*(1-math.tanh(dg/(2*p['theta'])))/2;near(v['forward'],kf)
    require(v['reference_occupancies']==[.5,.5],'pair occupancy convention');near(v['ensemble_forward_flux'],v['forward']/2);near(v['ensemble_reverse_flux'],v['reverse']/2)
    sigma=(v['forward']/2-v['reverse']/2)*(math.log(v['forward'])-math.log(v['reverse']));near(v['ensemble_pair_entropy'],sigma);require(sigma>=-1e-12,'negative equal-occupation witness')

def event(r,recover=True):
    b,a=r['before'],r['after'];p=b['params'];require(a['params']==p and a['owner']==b['owner'] and a['root_area']==b['root_area'],'owner/units reset');require(a['revision']==b['revision']+1,'revision');wb,wa=world(b),world(a);action=r['action'];kind=action['kind'];bc=b['cells'][0]
    for k in wb['invariants']:near(wb['invariants'][k],wa['invariants'][k]);near(r['invariants_before'][k],wb['invariants'][k]);near(r['invariants_after'][k],wa['invariants'][k])
    near(r['energy_before']['total'],wb['total']);near(r['energy_after']['total'],wa['total']);dg=wa['total']-wb['total'];errors=[]
    for k in wb['stores']:
        d=wa['stores'][k]-wb['stores'][k];near(r['energy_changes'][k],d);errors.append(abs(r['energy_changes'][k]-d))
    near(r['chemical_free_energy_release'],-r['energy_changes']['bulk_ideal']-r['energy_changes']['solid_standard']);near(r['mechanical_storage_change'],r['energy_changes']['native_elastic']+r['energy_changes']['new_coupling']);pair(r['pair'],dg,p)
    if recover:
        require(physical(r['recovered'])==physical(b) and r['recovered']['revision']>a['revision'],'inverse physical preimage');require(r['inverse_physical_recovery'] is True,'inverse report');pair(r['inverse_pair'],-dg,p)
        require(r['receipt']['before_hash']==hash_world(b) and r['receipt']['after_hash']==hash_world(a),'receipt source hash')
    if kind=='fission':
        require(len(a['cells'])==2 and a['outside_s']==b['outside_s'] and a['outside_sm']==b['outside_sm'] and a['outside_inert']==b['outside_inert'],'bath duplication')
        i,j=action['i'],action['j'];indices=(list(range(i,j+1)),list(range(j,len(bc['edges'])))+list(range(i+1)));mass=p['rho']*math.dist(bc['vertices'][i],bc['vertices'][j]);q=2*mass/p['yield_y'];n=bc['n'].copy();n[5]-=q;n[4]+=(1-p['yield_y'])*q
        near(sum(wa['volumes']),sum(wb['volumes']))
        for c,ix in zip(a['cells'],indices):
            require(c['vertices']==[bc['vertices'][k] for k in ix],'daughter geometry');require(c['edges'][:-1]==[bc['edges'][k] for k in ix[:-1]],'exact inherited edges');frac=area(c['vertices'])/area(bc['vertices'])
            for k in ('P','a0','R_ref'):near(c[k],bc[k]*frac)
            for av,nv in zip(c['n'],n):near(av,nv*frac)
            h,y=amounts(c['edges'][-1]);near(h,0);near(y,mass);require(all(t['root'] not in wb['origins'] for t in c['edges'][-1]['tags']),'closure material created without new provenance')
            number=list(a['cells']).index(c);label=f"{b['owner']}:{b['revision']+1}:fission:closure:{number}";same(c['edges'][-1],{'id':label,'tags':[{'root':label,'fraction':1.,'young':mass,'mature':0.}]})
    else:
        require(len(a['cells'])==1,'unexpected cell count');ac=a['cells'][0]
        for k in ('P','a0','R_ref'):near(ac[k],bc[k])
        n=bc['n'].copy();s,sm=b['outside_s'],b['outside_sm']
        if kind=='transport':
            q=action['amount']*(1 if action['direction']=='in' else -1);idx=4 if action['species']=='S' else 5;n[idx]+=q
            if idx==4:s-=q
            else:sm-=q
            require(ac['edges']==bc['edges'],'transport altered material')
        elif kind in ('growth','maturation'):
            i=action['edge'];q=action['extent'];hb,yb=amounts(bc['edges'][i]);ha,ya=amounts(ac['edges'][i])
            if kind=='growth':n[5]-=q;n[4]+=(1-p['yield_y'])*q;near(ha,hb);near(ya,yb+p['yield_y']*q)
            else:near(ha,hb+q);near(ya,yb-q)
            tags=[dict(t) for t in bc['edges'][i]['tags']]
            if kind=='growth':
                require(q>0,'sealed campaign growth direction');tags.append({'root':f"{b['owner']}:{b['revision']+1}:growth",'fraction':1.,'young':p['yield_y']*q,'mature':0.})
            else:
                for t,old in zip(tags,bc['edges'][i]['tags']):t['young']-=q*old['young']/yb;t['mature']+=q*old['young']/yb
            same(ac['edges'][i],{'id':bc['edges'][i]['id'],'tags':tags})
            require(ac['vertices']==bc['vertices'],'reaction moved geometry');require(ac['edges'][:i]+ac['edges'][i+1:]==bc['edges'][:i]+bc['edges'][i+1:],'nonlocal material mutation')
        elif kind=='remesh':
            i=action['edge'];va,vb=bc['vertices'][i],bc['vertices'][(i+1)%len(bc['vertices'])];require(ac['vertices']==bc['vertices'][:i+1]+[[(va[0]+vb[0])/2,(va[1]+vb[1])/2]]+bc['vertices'][i+1:],'midpoint geometry')
            require(len(ac['edges'])==len(bc['edges'])+1,'split dimension')
            for e in ac['edges'][i:i+2]:
                for t,old in zip(e['tags'],bc['edges'][i]['tags']):
                    require(t['root']==old['root'],'split ancestry')
                    for k in ('fraction','young','mature'):near(t[k],old[k]/2)
        else:raise ValueError('unknown raw event')
        for x,y in zip(ac['n'],n):near(x,y)
        near(a['outside_s'],s);near(a['outside_sm'],sm);near(a['outside_inert'],b['outside_inert'])
    return {'operator':kind,'delta_g':dg,'chemical_free_energy_release':r['chemical_free_energy_release'],'mechanical_storage_change':r['mechanical_storage_change'],'maximum_store_error':max(errors),'invariant_max_error':max(abs(wb['invariants'][k]-wa['invariants'][k]) for k in wb['invariants']),'forward_rate':r['pair']['forward'],'reverse_rate':r['pair']['reverse']}

def native_check(n):
    require(n['new_owner_integrated_biology'] is False and n['physical_fusion'] is False,'native overclaim');errs=[]
    require(len(n['force_samples'])==24,'native force grid')
    for v in n['force_samples']:
        expected=force(v['L'],v['mature'],v['young'],v['rho'],v['k_s']);near(v['measured_tension'],expected,1e-7);errs.append(abs(v['measured_tension']-expected))
    op=n['operators'];t=op['native_transport'];require(t['before']['exterior']==t['after']['exterior'],'native boundary behavior changed');require(t['before']['interior']!=t['after']['interior'],'transport probe inactive')
    g=op['native_growth'];A=area(g['before']['vertices']);near((g['before']['interior']['a']-g['after']['interior']['a'])*A,g['ledger']['a_consumed_growth']);near(sum(e['m'] for e in g['after']['edges'])-sum(e['m'] for e in g['before']['edges']),g['ledger']['m_grown']);near(sum(e['m_young'] for e in g['after']['edges'])-sum(e['m_young'] for e in g['before']['edges']),g['ledger']['m_grown']);require(g['ledger']['m_grown']>0,'growth probe inactive')
    mat=op['native_maturation'];require(mat['ledger']['m_matured']>0,'maturation probe inactive')
    f=op['native_fission'];result={'force_samples':len(errs),'maximum_force_error':max(errs),'fixed_native_exterior_confirmed':True,'native_growth_mass':g['ledger']['m_grown'],'native_matured_mass':mat['ledger']['m_matured'],'native_fission_observed':f['success'],'native_integration_qualified':False}
    if f['success']:
        result['native_partition_ok']=f['event']['partition']['ok'];require(result['native_partition_ok'] is True,'native Some with partition failure is not accepted')
        closures=[]
        for d in f['daughters']:
            closures.extend(e for e in d['edges'] if e['m_young']>0)
        require(len(closures)==2 and all(e['m']==e['m_young'] for e in closures),'newborn closure ownership');result['pure_young_closure_mass']=sum(e['m'] for e in closures)
    return result

def qualify(v,native=None):
    root=pathlib.Path(__file__).resolve().parents[1];protocol=json.loads((root/'docs/dc-m4-chemical-work-owner-protocol.json').read_text());p=v['params'];mapping={'theta':'theta','k_s':'k_s','rho':'rho','K':'K','lambda':'lam','mu_young':'mu_y','mu_mature':'mu_m','growth_yield':'yield_y','kappa':'kappa','total_volume':'total_volume'}
    for k,m in mapping.items():near(protocol['config'][k],p[m])
    require(v['identified_biological_mechanism'] is False and v['native_full_integration'] is False and v['physical_fusion'] is False,'owner overclaim');require([r['action']['kind'] for r in v['events']]==['transport','growth','maturation','fission','remesh'],'event campaign')
    for r in v['events']:same(r['before'],prepared(p))
    same(v['loaded_neutral_remesh']['before'],v['events'][2]['after'])
    for r in v['drive_controls']:
        cp=dict(p);cp['mu_m']=r['before']['params']['mu_m'];same(r['before'],prepared(cp))
    rows=[event(r) for r in v['events']];loaded=event(v['loaded_neutral_remesh'],False);near(loaded['delta_g'],0);require(v['loaded_neutral_remesh']['energy_before']['cells'][0]['elastic']>0,'zero-load-only remesh witness')
    controls=[event(r,False) for r in v['drive_controls']];require(controls[0]['delta_g']<0<controls[1]['delta_g'],'two chemical drive signs');near(controls[2]['delta_g'],0)
    require(len(v['force_grid'])==24,'reference force grid')
    for x in v['force_grid']:near(x['energy'],elastic(x['L'],x['mature'],x['young']));near(x['force'],force(x['L'],x['mature'],x['young']),1e-7)
    x=v['floor_remesh_counterexample'];near(x['before'],elastic(x['L'],x['mature'],x['young']));near(x['after'],2*elastic(x['L']/2,x['mature']/2,x['young']/2));near(x['delta'],x['after']-x['before']);require(abs(x['delta'])>1e-4,'floor counterexample lost')
    require(len(v['rejections'])==5 and all(x['unchanged'] is True for x in v['rejections']),'atomic negative fixtures')
    return {'scope':'independently reconstructed new finite-event work owners only','event_results':rows,'loaded_remesh':loaded,'chemical_drive_controls':controls,'force_grid_samples':24,'floor_remesh_storage_change':x['delta'],'atomic_rejection_fixtures':5,'native_audit':native_check(native) if native else None,'native_full_integration_qualified':False,'identified_biological_mechanism':False,'whole_organism_qualified':False,'production_values':dict.fromkeys(('K','lambda','theta','epsilon_A'))}

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--input',required=True);p.add_argument('--native');p.add_argument('--output',required=True);a=p.parse_args();v=qualify(json.loads(pathlib.Path(a.input).read_text()),json.loads(pathlib.Path(a.native).read_text()) if a.native else None);v['head']=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip();v['input_sha256']=hashlib.sha256(pathlib.Path(a.input).read_bytes()).hexdigest();v['native_sha256']=hashlib.sha256(pathlib.Path(a.native).read_bytes()).hexdigest() if a.native else None;pathlib.Path(a.output).write_text(json.dumps(v,indent=2)+'\n');print(json.dumps(v))
