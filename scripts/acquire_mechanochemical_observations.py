#!/usr/bin/env python3
"""Acquire licensed original observations; never execute publisher software."""
import argparse,hashlib,json,pathlib,tempfile,urllib.request,zipfile,time

def acquire(out):
    root=pathlib.Path(__file__).resolve().parents[1]
    protocol=json.loads((root/'docs/dc-m4-mechanochemical-observation-protocol.json').read_text())
    source=protocol['source'];out=pathlib.Path(out);out.mkdir(parents=True,exist_ok=True)
    metadata=json.load(urllib.request.urlopen('https://api.figshare.com/v2/articles/28211567',timeout=60))
    file=next(v for v in metadata['files'] if v['id']==source['file_id'])
    assert metadata['version']==1 and metadata['license']['name']=='CC BY 4.0'
    assert file['size']==source['bytes'] and file['computed_md5']==source['md5']
    (out/'publisher-metadata.json').write_text(json.dumps(metadata,indent=2)+'\n')
    with tempfile.TemporaryDirectory() as tmp:
        archive=pathlib.Path(tmp)/'publisher.zip';md5=hashlib.md5();sha=hashlib.sha256();size=0
        with urllib.request.urlopen(source['url']+'?acquisition='+str(time.time_ns()),timeout=120) as response,archive.open('wb') as f:
            assert response.status==200
            for blob in iter(lambda:response.read(4*1024*1024),b''):
                size+=len(blob);md5.update(blob);sha.update(blob);f.write(blob)
        assert size==source['bytes'] and md5.hexdigest()==source['md5']
        selected={v['name']:v for v in protocol['selection']['selected_files']};bindings={};headers={}
        with zipfile.ZipFile(archive) as z:
            assert z.testzip() is None
            inventory=[{'name':i.filename,'bytes':i.file_size,'crc32':i.CRC,'compressed_bytes':i.compress_size} for i in z.infolist()]
            with zipfile.ZipFile(out/'raw-selected.zip','w',compression=zipfile.ZIP_DEFLATED,compresslevel=6) as curated:
                for name,expected in selected.items():
                    info=z.getinfo(name);assert info.file_size==expected['bytes'] and info.CRC==expected['crc32']
                    blob=z.read(info);bindings[name]={'bytes':len(blob),'sha256':hashlib.sha256(blob).hexdigest(),'original_zip_crc32':info.CRC}
                    curated.writestr(name,blob)
                    # Raw headers only, not outcome selection or numerical analysis.
                    headers[name]=blob.splitlines()[0].decode('utf-8',errors='replace')
        (out/'publisher-zip-inventory.json').write_text(json.dumps(inventory,indent=2)+'\n')
        receipt={'source':source,'source_full_bytes':size,'source_full_md5_verified':md5.hexdigest(),'source_full_sha256':sha.hexdigest(),'whole_publisher_zip_crc_verified':True,'selected_original_members':bindings,'headers_only':headers,'external_source_software_executed':False,'full_publisher_zip_retained':'runner temporary only; publisher remains original source','scope':'DNA force-position observations only; no Digital Cell chemical/parameter identity'}
        (out/'acquisition-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
        print(json.dumps({'source_bytes':size,'selected_members':len(bindings),'curated_zip_bytes':(out/'raw-selected.zip').stat().st_size,'headers_only':headers},indent=2))

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--output',required=True);a=p.parse_args();acquire(a.output)
