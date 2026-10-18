#!/usr/bin/env python3
"""Blind designed-reference recovery and independently written native-fixture oracle."""
from __future__ import annotations
import math


def require(v,message):
    if not v:raise ValueError(message)


def finite(v):return type(v) in (int,float) and math.isfinite(v)


def recover(training):
    """Estimator sees observations only: no design truth or supplied g column."""
    require(len(training)>=3,'insufficient calibration observations')
    data=[]
    for r in training:
        require(all(finite(r[k]) for k in ('strain','da','P','L0','coupling_tension')),'invalid observation')
        require(r['P']>0 and r['L0']>0,'invalid amount/length units')
        data.append((r['P']*r['strain']/r['L0'],-r['da']/r['L0'],r['coupling_tension']))
    aa=math.fsum(x*x for x,y,z in data);bb=math.fsum(y*y for x,y,z in data);ab=math.fsum(x*y for x,y,z in data)
    at=math.fsum(x*z for x,y,z in data);bt=math.fsum(y*z for x,y,z in data);det=aa*bb-ab*ab
    require(det>1e-12*max(aa*bb,1e-30),'rank-deficient mechanical design')
    K=(at*bb-bt*ab)/det;Kl=(bt*aa-at*ab)/det
    require(K>0 and Kl>=0,'nonphysical fitted reference coefficients');lam=Kl/K
    ex=[]
    for r in training:
        g=-K*lam*(r['strain']-lam*r['da']/r['P'])
        for i,nu in ((1,1),(3,-1)):
            p,m,bp,bm=(r[k][i] for k in ('forward','reverse','bare_forward','bare_reverse'))
            require(all(finite(x) and x>0 for x in (p,m,bp,bm)),'invalid directional rates')
            x=-nu*g;y=math.log(p)-math.log(m)-math.log(bp)+math.log(bm);ex.append((x,y))
    xx=math.fsum(x*x for x,y in ex);xy=math.fsum(x*y for x,y in ex)
    require(xx>1e-14 and xy>0,'theta not identifiable from unexcited/inconsistent rates')
    return {'K':K,'lambda':lam,'theta':xx/xy,'epsilon_A':None,
            'claim':'DESIGNED_REFERENCE_NUMERICAL_RECOVERY_ONLY','mechanical_gram_determinant':det}


def holdout_errors(rows,fit):
    fe=le=0.
    for r in rows:
        T=fit['K']*(r['P']*r['strain']-fit['lambda']*r['da'])/r['L0']
        g=-fit['K']*fit['lambda']*(r['strain']-fit['lambda']*r['da']/r['P'])
        fe=max(fe,abs(T-r['coupling_tension']))
        for i,nu in enumerate((0,1,0,-1)):
            p,m,bp,bm=(r[k][i] for k in ('forward','reverse','bare_forward','bare_reverse'))
            observed=math.log(p)-math.log(m)-math.log(bp)+math.log(bm)
            le=max(le,abs(observed+nu*g/fit['theta']))
    return {'max_force_error':fe,'max_logratio_error':le}


def coarse_state_obstruction():
    z1=(.1,.05,.05,.01,1.,.1);z2=(.1,.05,.05,.01,.55,.55)
    def observations(z):
        E,ES,ESS,Pr,S,Sm=z
        return {'a':ES+ESS,'b':E,'P':E+ES+ESS,'A_src':S+Sm+ES+2*ESS,'W_src':2*Pr,
                'G':math.fsum(x*(math.log(x)-1) for x in z),'adot':Sm*E-ES-(ESS-E*Pr)}
    a,b=observations(z1),observations(z2)
    require(all(abs(a[k]-b[k])<1e-14 for k in ('a','b','P','A_src','W_src')),'counterexample macros differ')
    energy_gap=abs(a['G']-b['G']);rate_gap=abs(a['adot']-b['adot'])
    require(energy_gap>.4 and rate_gap>.04,'counterexample failed')
    return {'z1':z1,'z2':z2,'mapped_state1':a,'mapped_state2':b,'energy_gap_same_mapped_state':energy_gap,
            'active_rate_gap_same_mapped_state':rate_gap,'minimax_energy_error_lower_bound':energy_gap/2,
            'exact_four_parameter_native_coarse_mapping_admissible':False,
            'scope':'domain including both states, g=0,L=L0; does not exclude separately budgeted local approximations'}


def rhs(y,rest,K,lam,gamma,ks):
    z=y[:6];v=[y[6+2*i:8+2*i] for i in range(6)];E,ES,ESS,Pr,S,Sm=z
    dx=v[1][0]-v[0][0];dy=v[1][1]-v[0][1];L=math.hypot(dx,dy);P=.2;a0=.1
    extension=(L-rest[0])/rest[0]-lam*(ES+ESS-a0)/P;g=-K*lam*extension
    j1=S-Sm;j2=Sm*E*math.exp(-g/2)-ES*math.exp(g/2)
    j3=Sm*ES-ESS;j4=ESS*math.exp(g/2)-E*Pr*math.exp(-g/2)
    dz=(-j2+j4,j2-j3,j3-j4,j4,-j1,j1-j2-j3)
    f=[[0.,0.] for _ in range(6)]
    for i in range(6):
        j=(i+1)%6;d=[v[j][a]-v[i][a] for a in range(2)];length=math.hypot(*d)
        tension=ks*(length-rest[i])/rest[i]
        if i==0:tension+=K*P*extension/rest[0]
        for a in range(2):
            force=tension*d[a]/length;f[i][a]+=force;f[j][a]-=force
    return list(dz)+[x/gamma for point in f for x in point]


def native_fixture_oracle(initial_source,initial_vertices,rest,horizon,step=.0001):
    # Independent vector RK4; native implementation uses native-mesh explicit Euler.
    y=list(initial_source)+[x for p in initial_vertices for x in p];n=round(horizon/step);h=horizon/n
    for _ in range(n):
        k1=rhs(y,rest,2.,.75,1.,1.);k2=rhs([a+h*b/2 for a,b in zip(y,k1)],rest,2.,.75,1.,1.)
        k3=rhs([a+h*b/2 for a,b in zip(y,k2)],rest,2.,.75,1.,1.);k4=rhs([a+h*b for a,b in zip(y,k3)],rest,2.,.75,1.,1.)
        y=[a+h*(b+2*c+2*d+e)/6 for a,b,c,d,e in zip(y,k1,k2,k3,k4)]
    return y
