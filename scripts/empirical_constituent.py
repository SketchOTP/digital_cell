#!/usr/bin/env python3
"""Independent physical relaxation analysis; no Digital Cell parameter assignment."""
from __future__ import annotations
import array,hashlib,json,math,pathlib,sys

def require(v,m):
    if not v:raise ValueError(m)
def finite(v):return type(v) in (int,float) and math.isfinite(v)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def mean(x):return math.fsum(x)/len(x)

def normalized_wave(channels,selection,gain,dark=(0.,0.,0.)):
    x,y,s=channels;require(len(x)==len(y)==len(s)==selection['samples_per_period'],'waveform dimensions')
    den=[(v-dark[2])*gain for v in s];require(all(finite(v) and v>0 for v in den),'nonpositive QPD denominator')
    qx=[-(v-dark[0])/d for v,d in zip(x,den)];qy=[-(v-dark[1])/d for v,d in zip(y,den)]
    b0,b1=selection['baseline_slice'];p0,p1=selection['plateau_slice']
    baseline=[mean(qx[b0:b1]),mean(qy[b0:b1])];plateau=[mean(qx[p0:p1]),mean(qy[p0:p1])]
    delta=[p-b for p,b in zip(plateau,baseline)];amp2=math.fsum(v*v for v in delta)
    require(math.isfinite(amp2) and amp2>1e-30,'unidentified QPD response amplitude')
    u=[((a-baseline[0])*delta[0]+(b-baseline[1])*delta[1])/amp2 for a,b in zip(qx,qy)]
    require(all(finite(v) for v in u),'nonfinite response')
    return u,{'baseline_qpd':baseline,'plateau_qpd':plateau,'amplitude_qpd':math.sqrt(amp2),'amplitude_transfer_qualified':False}

def fit_tau(waves,selection):
    start,end=selection['fit_slice'];fs=selection['sample_rate_hz'];step=selection['step_sample']
    require(len(waves)>0 and all(len(w)>=end for w in waves),'insufficient fit observations')
    values=[(i-step)/fs for i in range(start,end)]
    ys=[float(v) for wave in waves for v in wave[start:end]];require(all(finite(v) for v in ys),'invalid fit observation')
    require(all(max(wave[start:end])-min(wave[start:end])>1e-8 for wave in waves),'flat/nonidentified observed response')
    def objective(logtau):
        tau=math.exp(logtau);pred=[1-math.exp(-t/tau) for t in values]
        return math.fsum((y-pred[j%len(pred)])**2 for j,y in enumerate(ys))/len(ys)
    low,high=selection['relaxation_tau_seconds_range'];a,b=math.log(low),math.log(high);lo_obj,hi_obj=objective(a),objective(b)
    ratio=(math.sqrt(5)-1)/2;c=b-ratio*(b-a);d=a+ratio*(b-a);fc,fd=objective(c),objective(d)
    for _ in range(64):
        if fc<fd:b,d,fd=d,c,fc;c=b-ratio*(b-a);fc=objective(c)
        else:a,c,fc=c,d,fd;d=a+ratio*(b-a);fd=objective(d)
    logtau=(a+b)/2;tau=math.exp(logtau);best=objective(logtau)
    require(low*1.001<tau<high/1.001,'fit on sealed boundary/unidentified response')
    require(min(lo_obj,hi_obj)-best>1e-8,'flat/nonidentified relaxation fit')
    return {'tau_seconds':tau,'train_normalized_rmse':math.sqrt(best),'objective':best,'low_boundary_objective':lo_obj,'high_boundary_objective':hi_obj}

def prediction_rmse(wave,tau,selection):
    start,end=selection['fit_slice'];fs=selection['sample_rate_hz'];edge=selection['step_sample']
    return math.sqrt(mean([(wave[i]-(1-math.exp(-(i-edge)/fs/tau)))**2 for i in range(start,end)]))

def analyze(source,protocol):
    root=pathlib.Path(source);meta=json.loads((root/'raw-subset-metadata.json').read_text());receipt=json.loads((root/'acquisition-receipt.json').read_text());sel=protocol['selection']
    require(meta['selection']==sel,'sealed selection mismatch');require(receipt['external_source_software_executed'] is False and receipt['original_full_zip_verified_in_hosted_acquisition'] is True,'source acquisition role')
    require(receipt['url']==protocol['source']['url'],'source URL')
    require((root/'raw-selected-f64.bin').stat().st_size==sum(d['bytes'] for d in meta['datasets']),'untagged raw bytes')
    require(receipt['md5_verified']==protocol['source']['md5'] and receipt['bytes']==protocol['source']['bytes'],'independent source identity')
    require(sha(root/'raw-selected-f64.bin')==meta['binary_sha256'],'raw selected binary identity')
    beta=meta['provided_beta_pNs_per_nm'];gain=meta['gain'];require(finite(beta) and beta>0 and finite(gain) and gain>0,'invalid conditional drag/gain')
    inventory=json.loads((root/'hdf5-inventory.json').read_text());dark=[]
    for name in ('x_signal_darkness','y_signal_darkness','sum_signal_darkness'):
        dark.append(inventory['measurement_datum/mean_background_signal_in_darkness/'+name]['scalar_or_small_value'])
    require(all(finite(x) for x in dark),'invalid source dark metadata')
    period=sel['samples_per_period'];n=sel['first_complete_periods_per_displacement'];raw={}
    with (root/'raw-selected-f64.bin').open('rb') as f:
        for d in meta['datasets']:
            f.seek(d['offset_bytes']);blob=f.read(d['bytes']);require(len(blob)==d['bytes'] and hashlib.sha256(blob).hexdigest()==d['sha256'],'raw dataset checksum')
            values=array.array('d');values.frombytes(blob)
            if sys.byteorder!='little':values.byteswap()
            require(len(values)==n*period and all(math.isfinite(v) for v in values),'raw shape/finite')
            raw.setdefault(d['planned_displacement_nm'],{})[d['channel']]=values
    require(set(raw)==set(sel['displacements_nm']) and len(meta['datasets'])==12,'sealed stimulus count')
    sums={};averages={};waves={};normalization={}
    for displacement,cols in raw.items():
        channels=[cols[k] for k in ('displacement_x','displacement_y','sum_signal_in_light')]
        s=[[math.fsum(ch[j+period*i] for i in range(n)) for j in range(period)] for ch in channels];sums[displacement]=s
        avg=[[v/n for v in ch] for ch in s];averages[displacement]=avg
        waves[displacement],normalization[displacement]=normalized_wave(avg,sel,gain)
    train=sel['training_nm'];held=sel['heldout_nm'];fit=fit_tau([waves[x] for x in train],sel);tau=fit['tau_seconds']
    heldout={str(x):{'prediction_normalized_rmse':prediction_rmse(waves[x],tau,sel),'diagnostic_only_fitted_tau_seconds':fit_tau([waves[x]],sel)['tau_seconds']} for x in held}
    for x,v in heldout.items():v['diagnostic_relative_tau_difference']=abs(v['diagnostic_only_fitted_tau_seconds']/tau-1)
    jack=[]
    for displacement in train:
        cols=raw[displacement];channels=[cols[k] for k in ('displacement_x','displacement_y','sum_signal_in_light')]
        for drop in range(n):
            avg=[[(sums[displacement][c][j]-channels[c][drop*period+j])/(n-1) for j in range(period)] for c in range(3)]
            u,_=normalized_wave(avg,sel,gain)
            jack.append(fit_tau([u if x==displacement else waves[x] for x in train],sel)['tau_seconds'])
    jack_mean=mean(jack);jack_se=math.sqrt((len(jack)-1)/len(jack)*math.fsum((x-jack_mean)**2 for x in jack))
    sensitivity={}
    try:
        alternate=[normalized_wave(averages[x],sel,gain,tuple(dark))[0] for x in train];altfit=fit_tau(alternate,sel)
        sensitivity={'convention':'alternative plain stored offset subtraction x-dark_x,y-dark_y,(sum-dark_sum)*gain; not asserted uniquely correct', 'second_dark_subtraction_tau_seconds':altfit['tau_seconds'],'relative_difference_from_primary':abs(altfit['tau_seconds']/tau-1),'used_for_primary_estimation_or_acceptance':False}
    except ValueError as e:sensitivity={'alternative_rejected':str(e),'used_for_primary_estimation_or_acceptance':False}
    qualified=all(v['prediction_normalized_rmse']<=protocol['budgets']['max_heldout_normalized_rmse'] and v['diagnostic_relative_tau_difference']<=protocol['budgets']['max_heldout_relative_tau_difference'] for v in heldout.values())
    return {'schema':'digital_cell_empirical_constituent_measurement_v1','source_role':'INDEPENDENT_EXPERIMENTAL_INSTRUMENT_CONSTITUENT_ONLY','source_md5':receipt['md5_verified'],'raw_binary_sha256':meta['binary_sha256'],'periods_per_stimulus':n,'stimuli_nm':sel['displacements_nm'],'train_stimuli_nm':train,'heldout_stimuli_nm':held,'training_fit':fit,'tau_jackknife_standard_error_seconds_conditional_within_record':jack_se,'conditional_kappa_pN_per_nm':beta/tau,'provided_beta_pNs_per_nm':beta,'provided_beta_is_independently_calibrated_here':False,'heldout_shape_test':heldout,'linear_relaxation_shape_transfer_within_sealed_budgets':qualified,'normalizations':{str(k):v for k,v in normalization.items()},'background':{'primary_no_second_subtraction':True,'stored_dark_scalars':dark,'sensitivity':sensitivity},'uncertainty_limits':['one bead/one date only; sequential periods not independently certified','jackknife does not bound between-bead drift or instrumental/systematic error','provided drag beta/temperature/viscosity/height/radius uncertainty not independently established','per-heldout amplitude/baseline normalization tests relaxation shape, not amplitude or displacement transfer'],'production_values':dict.fromkeys(('K','lambda','theta','epsilon_A')),'empirical_full_target_qualified':False,'chemical_kinetic_mapping_qualified':False,'automatic_successor':False}
