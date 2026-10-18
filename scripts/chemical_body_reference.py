#!/usr/bin/env python3
"""Independent amount/finite-volume/full-vector body equations and RK4 oracle."""
import math

def source(preparation='Published'):
    if preparation=='Published':return [.1,.05,.05,.01,1.,.1]
    lo,hi=0.,1.
    for _ in range(100):
        x=(lo+hi)/2
        if 200*x**4+400*x**3+313*x*x+93*x-127>0:hi=x
        else:lo=x
    x=(lo+hi)/2;e=.2/(1+x+x*x)
    return [e,x*e,x*x*e,x*x,x,x]
def birth():return [[math.cos(2*math.pi*i/24),math.sin(2*math.pi*i/24)] for i in range(24)]
def area(v):return math.fsum(v[i][0]*v[(i+1)%len(v)][1]-v[(i+1)%len(v)][0]*v[i][1] for i in range(len(v)))/2
BIRTH_AREA=area(birth())
def remap(n,v,rest):
    m=len(v);return list(n),[p for i in range(m) for p in (list(v[i]),[(v[i][a]+v[(i+1)%m][a])/2 for a in range(2)])],[r/2 for r in rest for _ in range(2)]

def model(n,v,rest,arc,preparation='Published',lam=.75,rate=1.,full=True):
    count=len(v);init=source(preparation);P=sum(init[:3]);a0=init[1]+init[2];no=9*sum(init)
    vol=area(v)/BIRTH_AREA;vo=10-vol;z=[x/vol for x in n]
    lengths=[math.hypot(v[(i+1)%count][0]-v[i][0],v[(i+1)%count][1]-v[i][1]) for i in range(count)]
    la=math.fsum(lengths[i] for i in arc);r0=math.fsum(rest[i] for i in arc)
    ex=(la-r0)/r0-lam*(n[1]+n[2]-a0)/P;g=-2*lam*ex;tc=2*P*ex/r0
    bp=[z[4],z[5]*z[0],z[5]*z[1],z[2]];bm=[z[5],z[1],z[2],z[0]*z[3]];nu=(0,1,0,-1)
    fp=[vol*rate*bp[i]*math.exp(-nu[i]*g/2) for i in range(4)];fm=[vol*rate*bm[i]*math.exp(nu[i]*g/2) for i in range(4)];j=[p-m for p,m in zip(fp,fm)]
    dn=[-j[1]+j[3],j[1]-j[2],j[2]-j[3],j[3],-j[0],j[0]-j[1]-j[2]]
    nf=[[0.,0.] for _ in v];cf=[[0.,0.] for _ in v];of=[[0.,0.] for _ in v]
    pressure=(sum(n)/vol-no/vo)/BIRTH_AREA
    for i in range(count):
        nxt=(i+1)%count;prev=(i-1)%count;t=(lengths[i]-rest[i])/rest[i]
        for a in range(2):
            unit=(v[nxt][a]-v[i][a])/lengths[i];f=t*unit;nf[i][a]+=f;nf[nxt][a]-=f
            if i in arc:cf[i][a]+=tc*unit;cf[nxt][a]-=tc*unit
        of[i]=[pressure*(v[nxt][1]-v[prev][1])/2,pressure*(v[prev][0]-v[nxt][0])/2]
    ext=[[cf[i][a]+of[i][a] for a in range(2)] for i in range(count)];total=[[nf[i][a]+ext[i][a] for a in range(2)] for i in range(count)]
    if not full:return dn+[x for p in total for x in p]
    af=[math.log(bp[i])-math.log(bm[i])-nu[i]*g for i in range(4)];dc=math.fsum(j[i]*af[i] for i in range(4));dm=math.fsum(x*x for p in total for x in p)
    gc=math.fsum(n[i]*(math.log(z[i])-1) for i in range(6));go=no*(math.log(no/vo)-1);ec=P*ex*ex;us=math.fsum((l-r)**2/(2*r) for l,r in zip(lengths,rest))
    cd=math.fsum(dn[i]*(math.log(z[i])+(g if i in (1,2) else 0.)) for i in range(6));gd=cd-dm
    omitted=cd+dc-math.fsum(of[i][a]*(nf[i][a]+cf[i][a]) for i in range(count) for a in range(2))
    return {'concentrations':z,'volume':vol,'outside_volume':vo,'outside_amount':no,'birth_area':BIRTH_AREA,'p':P,'a0':a0,'lengths':lengths,'arc_length':la,'arc_rest_length':r0,'gchem':gc,'gout':go,'ec':ec,'us':us,'total_g':gc+go+ec+us,'g':g,'coupling_tension':tc,'bare_forward':bp,'bare_reverse':bm,'forward':fp,'reverse':fm,'affinities':af,'currents':j,'amount_derivative':dn,'native_forces':nf,'coupling_forces':cf,'osmotic_forces':of,'external_forces':ext,'combined_forces':total,'velocities':total,'dchem':dc,'dmech':dm,'gradient_dot_ode':gd,'gradient_identity_error':gd+dc+dm,'omitted_osmotic_identity_error':omitted}

def oracle(preparation='Published',lam=.75,rate=1.,step=.0001):
    n=source(preparation);v=birth();rest=[math.hypot(v[(i+1)%24][0]-v[i][0],v[(i+1)%24][1]-v[i][1]) for i in range(24)];arc=[0]
    y=n+[x for p in v for x in p];count=round(2/step);split=round(.2/step)
    def f(y):return model(y[:6],[y[i:i+2] for i in range(6,len(y),2)],rest,arc,preparation,lam,rate,False)
    for k in range(count):
        a=f(y);b=f([x+step*d/2 for x,d in zip(y,a)]);c=f([x+step*d/2 for x,d in zip(y,b)]);d=f([x+step*z for x,z in zip(y,c)])
        y=[x+step*(a0+2*b0+2*c0+d0)/6 for x,a0,b0,c0,d0 in zip(y,a,b,c,d)]
        if k+1==split:
            n,v,rest=remap(y[:6],[y[i:i+2] for i in range(6,len(y),2)],rest);y=n+[x for p in v for x in p];arc=[0,1]
    return y
