#!/usr/bin/env python3
"""Acquire one sealed CC0 experimental record; never execute source software."""
import argparse,pathlib,json,hashlib,urllib.request,zipfile,sys,subprocess

def clean(v):
    if hasattr(v,'tolist'):v=v.tolist()
    if isinstance(v,bytes):return v.decode('utf8',errors='replace')
    if isinstance(v,(tuple,list)):return [clean(x) for x in v]
    if isinstance(v,dict):return {str(k):clean(x) for k,x in v.items()}
    return v

def main():
    import h5py,numpy as np
    p=argparse.ArgumentParser();p.add_argument('--output',required=True);a=p.parse_args();out=pathlib.Path(a.output);out.mkdir(parents=True,exist_ok=True)
    repo=pathlib.Path(__file__).resolve().parents[1];protocol=repo/'docs/dc-m4-empirical-constituent-protocol.json';proto=json.loads(protocol.read_text());source=proto['source']
    raw=out.parent/'raw-source';raw.mkdir(exist_ok=True);archive=raw/source['file'];digest=hashlib.md5();size=0
    with urllib.request.urlopen(source['url'],timeout=40) as response,archive.open('wb') as f:
        headers=dict(response.headers)
        while True:
            chunk=response.read(1024*1024)
            if not chunk:break
            size+=len(chunk)
            if size>source['bytes']:raise ValueError('source exceeds sealed size')
            digest.update(chunk);f.write(chunk)
    if size!=source['bytes'] or digest.hexdigest()!=source['md5']:raise ValueError('source original identity failure')
    receipt={'url':source['url'],'bytes':size,'md5_verified':digest.hexdigest(),'sha256':hashlib.sha256(archive.read_bytes()).hexdigest(),'headers':headers,'original_full_zip_verified_in_hosted_acquisition':True,'full_original_zip_bundled_in_artifact':False,'role':proto['role'],'protocol_sha256':hashlib.sha256(protocol.read_bytes()).hexdigest(),'head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip(),'external_source_software_executed':False}
    with zipfile.ZipFile(archive) as z:
        if z.testzip() is not None:raise ValueError('ZIP CRC failure')
        inventory=[{'name':i.filename,'bytes':i.file_size,'crc32':i.CRC} for i in z.infolist()]
        names=[i.filename for i in z.infolist() if i.filename.lower().endswith(('.h5','.hdf5'))]
        if len(names)!=1:raise ValueError('expected one HDF5 source member: '+str(names))
        name=names[0];hdf=raw/'source.h5'
        with z.open(name) as src,hdf.open('wb') as dst:
            while True:
                b=src.read(1024*1024)
                if not b:break
                dst.write(b)
    receipt['zip_members']=inventory;receipt['source_hdf5_sha256']=hashlib.sha256(hdf.read_bytes()).hexdigest()
    (out/'acquisition-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
    selection=proto['selection'];count=selection['samples_per_period']*selection['first_complete_periods_per_displacement'];datasets=[];nodes={}
    with h5py.File(hdf,'r') as h:
        def visit(name,obj):
            nodes[name]={'attrs':clean(dict(obj.attrs)),'kind':'dataset' if isinstance(obj,h5py.Dataset) else 'group'}
            if isinstance(obj,h5py.Dataset):
                nodes[name].update(shape=list(obj.shape),dtype=str(obj.dtype))
                if obj.size<=16:nodes[name]['scalar_or_small_value']=clean(obj[()])
        h.visititems(visit);nodes['/']={'attrs':clean(dict(h.attrs))};(out/'hdf5-inventory.json').write_text(json.dumps(nodes,indent=2,allow_nan=False)+'\n')
        beta=h['calibration/Stokes_Faxen_coeffiecent'][()];gain=h['calibration/gain'][()]
        with (out/'raw-selected-f64.bin').open('wb') as f:
            offset=0
            for displacement in selection['displacements_nm']:
                for channel in ('displacement_x','displacement_y','sum_signal_in_light'):
                    key=f'measurement_datum/planned_displacement_of_trapped_bead/{displacement}/{channel}';obj=h[key]
                    if obj.ndim!=1 or len(obj)<count:raise ValueError('insufficient sealed raw observations '+key)
                    arr=np.asarray(obj[:count],dtype='<f8')
                    if not np.isfinite(arr).all():raise ValueError('nonfinite raw data')
                    b=arr.tobytes();f.write(b);datasets.append({'path':key,'planned_displacement_nm':displacement,'channel':channel,'samples':count,'offset_bytes':offset,'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest(),'attrs':clean(dict(obj.attrs)),'original_shape':list(obj.shape)})
                    offset+=len(b)
        meta={'schema':'digital_cell_empirical_constituent_raw_subset_v1','source_receipt':'acquisition-receipt.json','exact_lossless_selected_IEEE754_values':True,'selection':selection,'gain':clean(gain),'provided_beta_pNs_per_nm':clean(beta),'datasets':datasets,'binary_sha256':hashlib.sha256((out/'raw-selected-f64.bin').read_bytes()).hexdigest(),'raw_total_samples':sum(d['samples'] for d in datasets),'warning':'selected16periods only; full source originals verified on hosted runner and remain at publisher, not bundled; no target map or physical systematic uncertainty inferred'}
        (out/'raw-subset-metadata.json').write_text(json.dumps(meta,indent=2,allow_nan=False)+'\n')
    print(json.dumps({'acquired':'PASS','md5':digest.hexdigest(),'datasets':len(datasets),'scalar_samples':sum(d['samples'] for d in datasets)}))
if __name__=='__main__':main()
