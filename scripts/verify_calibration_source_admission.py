#!/usr/bin/env python3
"""Offline provenance reconstruction and negative source-admission verification."""
from __future__ import annotations

import argparse
from decimal import Decimal
import hashlib
import importlib.util
import io
import json
from pathlib import Path, PurePosixPath
import shutil
import struct
import subprocess
import sys
import tempfile
import unittest
import zlib

BASE = '2b2638047a79fcbf01b279e7d3a796873f0017c5'
DIRECTIVE = 'D-20261017-dcm4-calibration-source-admission'
FOLDER = 'docs/dc-m4-calibration-source-evidence'
CORE = {
    'docs/dc-m4-calibration-source-directive.md',
    'docs/dc-m4-calibration-source-admission.md',
    'docs/dc-m4-calibration-source-admission.json',
    'scripts/calibration_source_admission.py',
    'scripts/test_calibration_source_admission.py',
    'scripts/verify_calibration_source_admission.py',
    '.github/workflows/dc-m4-calibration-source-admission.yml',
}
REVIEW = 'docs/dc-m4-calibration-source-review.json'
AGENT = {f'.agent/{n}.md' for n in
         ('CURRENT','PROJECT_PROFILE','DIRECTIVES','OUTCOMES','LEARNINGS','RECORD','REPO_MAP')}


def require(value, message):
    if not value: raise ValueError(message)


def sha(data): return hashlib.sha256(data).hexdigest()


def module(name,path):
    spec=importlib.util.spec_from_file_location(name,path)
    result=importlib.util.module_from_spec(spec);sys.modules[name]=result
    spec.loader.exec_module(result);return result


def reconstruct(folder):
    meta=json.loads((folder/'zenodo-4481182-record.json').read_text())
    require(meta['id']==4481182 and meta['metadata']['license']['id']=='cc-by-4.0',
            'source identity/license drift')
    listed=meta['files'][0]
    require(listed['size']==551498479 and listed['checksum']=='md5:75c151155bd3187cb1e56e07539887f4',
            'publisher archive metadata drift')
    tail=(folder/'zenodo-4481182-zip-tail.bin').read_bytes()
    pos=tail.rfind(b'PK\x05\x06');require(pos>=0,'EOCD missing')
    end=struct.unpack_from('<4s4H2LH',tail,pos)
    require(end[1]==end[2]==0 and end[3]==end[4]==3122 and end[5]==424930
            and end[6]==551073527 and end[7]==0 and pos+22==len(tail), 'ZIP directory identity')
    central=(folder/'zenodo-4481182-central-directory.bin').read_bytes()
    require(len(central)==end[5] and central[-(len(tail)-22):]==tail[:-22],
            'tail and central directory disagree')
    rows={};offset=0
    while offset<len(central):
        v=struct.unpack_from('<4s6H3L5H2L',central,offset);require(v[0]==b'PK\x01\x02','bad central entry')
        n,e,c=v[10:13];name=central[offset+46:offset+46+n].decode('utf-8' if v[3]&2048 else 'cp437')
        require(name not in rows,'duplicate archive member')
        rows[name]={'name':name,'method':v[4],'crc32':v[7],'compressed_size':v[8],
                    'size':v[9],'flags':v[3],'offset':v[16]}
        offset+=46+n+e+c
    require(offset==len(central) and len(rows)==3122,'central directory count')
    payload=(folder/'zenodo-4481182-range-0-199999.bin').read_bytes()
    require(len(payload)==200000,'sparse payload extent')
    receipt=json.loads((folder/'acquisition-receipt.json').read_text())
    context=json.loads((folder/'plot-context-receipt.json').read_text())
    prereg=(folder/'acquisition-preregistration.json').read_bytes()
    require(sha(prereg)==receipt['preregistration_sha256'] and
            sha((folder/'plot-context-preregistration.json').read_bytes())==context['preregistration_sha256'],
            'selection preregistration binding')
    initial=json.loads(prereg);context_pre=json.loads((folder/'plot-context-preregistration.json').read_text())
    require(initial['selected_members']==[{k:r[k] for k in rows[r['name']]} for r in receipt['members']],
            'initial selection changed')
    require(context_pre['members']==[rows[r['name']] for r in context['members']] and
            context_pre['parent_receipt_sha256']==sha((folder/'acquisition-receipt.json').read_bytes()),
            'context selection lineage')
    require(receipt['member_count']==24 and context['member_count']==2
            and receipt['whole_archive_downloaded'] is False
            and receipt['whole_archive_MD5_verified'] is False
            and receipt['external_code_executed'] is False
            and receipt['digital_cell_calibration_rows_admitted']==0,
            'partial archive or acquisition overclaim')
    require(receipt['payload_range_sha256']==sha(payload)
            and receipt['central_directory_sha256']==sha(central),'source range hash')
    members={}
    for item in receipt['members']+context['members']:
        name=item['name'];row=rows[name]
        require(all(item[k]==v for k,v in row.items()),'member directory binding')
        rel=PurePosixPath(name)
        require(not rel.is_absolute() and '..' not in rel.parts and not row['flags']&1,
                'unsafe/encrypted source member')
        at=row['offset'];v=struct.unpack_from('<4s5H3L2H',payload,at)
        require(v[0]==b'PK\x03\x04' and v[2]==row['flags'] and v[3]==row['method'], 'local header mismatch')
        n,e=v[9:11];require(payload[at+30:at+30+n].decode()==name,'local filename mismatch')
        start=at+30+n+e;stop=start+row['compressed_size']
        require(stop<=len(payload) and row['method']==8 and row['size']<100000,'member bounds/method')
        data=zlib.decompress(payload[start:stop],-15)
        require(len(data)==row['size'] and zlib.crc32(data)==row['crc32']
                and sha(data)==item['sha256'],'member CRC/size/SHA mismatch')
        require(name not in members,'duplicate selected member');members[name]=data
    require(len(members)==26 and sum(n.startswith('zenodo/source_code/') for n in members)==18,
            'selected source/member count')
    inventory=json.loads((folder/'table-inventory.json').read_text());row_count=0
    require(len(inventory['tables'])==4 and inventory['digital_cell_calibration_rows_admitted']==0
            and inventory['raw_replicate_trajectories_acquired'] is False
            and inventory['source_code_executed'] is False,'table scope drift')
    for table in inventory['tables']:
        data=members[table['member']];lines=data.decode().splitlines()
        require(table['sha256']==sha(data) and table['header'] is False
                and table['rows']==len(lines) and table['columns']==23,'table inventory mismatch')
        for line in lines:
            values=line.split('\t')
            require(len(values)==23 and all(Decimal(v).is_finite() for v in values),'malformed numeric table')
        row_count+=len(lines)
    require(row_count==inventory['external_summary_rows']==44
            and inventory['external_numeric_cells']==1012,'observed external row/cell count')
    return {'original_members_verified':26,'source_code_files_not_executed':18,
            'external_processed_tables':4,'external_processed_rows':44,'external_numeric_cells':1012,
            'raw_replicate_trajectories_acquired':False,'whole_archive_MD5_verified':False,
            'member_CRC_and_SHA256':'PASS','admitted_calibration_rows':0}


def integrity_controls(folder):
    def refresh_context_lineage(p):
        f=p/'plot-context-preregistration.json';x=json.loads(f.read_text())
        x['parent_receipt_sha256']=sha((p/'acquisition-receipt.json').read_bytes())
        f.write_text(json.dumps(x))
        f2=p/'plot-context-receipt.json';x=json.loads(f2.read_text())
        x['preregistration_sha256']=sha(f.read_bytes());f2.write_text(json.dumps(x))
    def bad_range(p):
        f=p/'zenodo-4481182-range-0-199999.bin';data=bytearray(f.read_bytes())
        data[500]^=1;f.write_bytes(data)
    def bad_whole_hash_claim(p):
        f=p/'acquisition-receipt.json';x=json.loads(f.read_text())
        x['whole_archive_MD5_verified']=True;f.write_text(json.dumps(x))
        refresh_context_lineage(p)
    def bad_table_header(p):
        f=p/'table-inventory.json';x=json.loads(f.read_text())
        x['tables'][0]['header']=True;f.write_text(json.dumps(x))
    def bad_member_content(p):
        f=p/'zenodo-4481182-range-0-199999.bin';data=bytearray(f.read_bytes())
        receipt_path=p/'acquisition-receipt.json';receipt=json.loads(receipt_path.read_text())
        item=next(x for x in receipt['members'] if x['name'].endswith('fig3/motorI_analyzed.txt'))
        data[item['compressed_range_start']+10]^=1;f.write_bytes(data)
        # Even updating the local transport hash cannot conceal invalid member
        # content. Preserve context lineage so the CRC/decompression gate runs.
        receipt['payload_range_sha256']=sha(data);receipt_path.write_text(json.dumps(receipt))
        refresh_context_lineage(p)
    for mutate in (bad_range,bad_whole_hash_claim,bad_table_header,bad_member_content):
        with tempfile.TemporaryDirectory(prefix='dc-source-integrity-') as tmp:
            target=Path(tmp)/'evidence';shutil.copytree(folder,target);mutate(target)
            try:reconstruct(target)
            except (ValueError,zlib.error):pass
            else:raise ValueError('source integrity negative control accepted')
    return 4


def main():
    ap=argparse.ArgumentParser();ap.add_argument('--repo',type=Path,default=Path('.'))
    ap.add_argument('--output',type=Path,required=True);a=ap.parse_args();repo=a.repo.resolve();out=a.output.resolve()
    require(not out.is_relative_to(repo) and not out.exists(),'fresh external evidence path required')
    sys.dont_write_bytecode=True
    def git(*args):return subprocess.check_output(['git',*args],cwd=repo)
    git('diff','--check',BASE)
    evidence_paths={str(p.relative_to(repo)) for p in (repo/FOLDER).iterdir() if p.is_file()}
    reviewed=CORE|evidence_paths;allowed=reviewed|{REVIEW,'STORAGE_MAP.md'}|AGENT
    changed=set(git('diff','--name-only',BASE).decode().splitlines())
    unknown=set(git('ls-files','--others','--exclude-standard').decode().splitlines())
    require((changed|unknown)<=allowed,'source-admission scope drift')
    require(not git('diff','--name-only',BASE,'--','digital-protocell'),'scientific runtime drift')
    for name in ('DIRECTIVES','OUTCOMES','LEARNINGS','RECORD','REPO_MAP'):
        p=f'.agent/{name}.md';require((repo/p).read_bytes().startswith(git('show',f'{BASE}:{p}')),
                                    'append-only history drift '+p)
    prior=json.loads((repo/'docs/dc-m4-reaction-architecture-audit.json').read_text())['source_bindings'].copy()
    # Freeze every prior protocol/prototype artifact, not mutable current records.
    docs=('dc-m4-conversion-work-contract.md','dc-m4-conversion-work-contract.json',
          'dc-m4-conversion-work-directive.md','dc-m4-conversion-work-review.json',
          'dc-m4-work-port-specification.md','dc-m4-work-port-specification.json',
          'dc-m4-work-port-directive.md','dc-m4-work-port-review.json',
          'dc-m4-source-receiver-architecture.md','dc-m4-source-receiver-architecture.json',
          'dc-m4-source-receiver-directive.md','dc-m4-source-receiver-review.json',
          'dc-m4-reference-prototype.md','dc-m4-reference-prototype-directive.md',
          'dc-m4-reference-prototype-review.json','dc-m4-calibration-acquisition-plan.md',
          'dc-m4-calibration-acquisition-plan.json','dc-m4-calibration-force-template.csv',
          'd003_calibration_report.md','d006_planar_calibration.md','d016_transport_calibration.md')
    scripts=('verify_conversion_work_contract.py','verify_work_port_specification.py',
             'verify_source_receiver_architecture.py','dc_m4_reference_prototype.py',
             'test_dc_m4_reference_prototype.py','verify_reference_prototype.py')
    workflows=('dc-m4-conversion-work-contract.yml','dc-m4-work-port-specification.yml',
               'dc-m4-source-receiver-architecture.yml','dc-m4-reference-prototype.yml')
    for p in [*('docs/'+d for d in docs),*('scripts/'+s for s in scripts),
              *('.github/workflows/'+w for w in workflows)]:prior[p]=sha(git('show',f'{BASE}:{p}'))
    for path,h in prior.items():
        require(sha((repo/path).read_bytes())==h and (repo/path).read_bytes()==git('show',f'{BASE}:{path}'),
                'frozen source/protocol drift '+path)
    review=json.loads((repo/REVIEW).read_text())
    require(review['decision']=='ACCEPTED_NEGATIVE_ADMISSION_AND_QUARANTINED_ACQUISITION_ONLY'
            and review['entry_head']==BASE and set(review['reviewed_files'])==reviewed,
            'missing/incomplete independent review')
    for path,h in review['reviewed_files'].items():require(sha((repo/path).read_bytes())==h,'stale review '+path)
    registry=json.loads((repo/'docs/dc-m4-calibration-source-admission.json').read_text())
    require(registry['schema']=='digital_cell_calibration_source_admission_v1'
            and registry['entry_head']==BASE and registry['status'].startswith('PARTIAL_BLOCKED_'),
            'partial acquisition status lost')
    require(registry['admitted_source_ids']==[] and registry['admitted_calibration_datasets']==[]
            and registry['production_values']==dict.fromkeys(('K','lambda','theta','epsilon_A'))
            and registry['fitting_performed'] is False and registry['native_runtime_modified'] is False
            and registry['automatic_successor'] is False,'calibration/runtime overclaim')
    require(registry['owner_context']['specific_reference_established'] is False
            and registry['owner_context']['verbatim_reply']=='No specific reference is established',
            'owner source clarification lost')
    acquisition=reconstruct(repo/FOLDER)
    rejected_integrity_mutants=integrity_controls(repo/FOLDER)
    for key,value in {'external_original_members':26,'external_processed_tables':4,
                      'external_processed_rows':44,'external_numeric_cells':1012,
                      'admitted_calibration_rows':0,'raw_replicate_trajectories_acquired':False,
                      'full_archive_MD5_verified':False,'external_code_executed':False}.items():
        require(type(registry['acquisition'][key]) is type(value) and registry['acquisition'][key]==value,
                'registry acquisition scope mismatch '+key)
    for identity,item in registry['evidence_catalog'].items():
        require(set(item['paths'])==set(item['sha256']),'evidence path binding '+identity)
        for path,h in item['sha256'].items():require(sha((repo/path).read_bytes())==h,'evidence stale '+path)
    sys.path.insert(0,str(repo/'scripts'))
    admission=module('calibration_source_admission',repo/'scripts/calibration_source_admission.py')
    decisions=[]
    for c in registry['candidates']:
        require(set(c['evidence_ids'])<=set(registry['evidence_catalog']),'undeclared evidence catalog ID')
        d=admission.evaluate(c);require(not d['admitted'] and not d['calibrated'],'automated admission')
        require('ELIGIBLE_FOR_REVIEW' not in d['per_parameter'].values(),'actual source missing gate silently passed')
        decisions.append(d)
    require(len(decisions)==8 and len({d['candidate_id'] for d in decisions})==8,'source family identity')
    tests=module('admission_tests',repo/'scripts/test_calibration_source_admission.py')
    stream=io.StringIO();result=unittest.TextTestRunner(stream=stream,verbosity=2).run(
        unittest.defaultTestLoader.loadTestsFromModule(tests))
    require(result.wasSuccessful() and result.testsRun==10,stream.getvalue())
    gov=module('governance',repo/'scripts/validate_governance.py')
    def section(path,title):
        for n,b in gov.iter_sections((repo/path).read_text()):
            if n==title:return '## '+n+'\n'+b
        raise ValueError('missing new record '+title)
    errors=[];ids=gov.validate_directives(section('.agent/DIRECTIVES.md',DIRECTIVE),errors)
    gov.validate_outcomes(section('.agent/OUTCOMES.md',DIRECTIVE+' - PARTIAL'),ids,errors)
    gov.validate_learnings(section('.agent/LEARNINGS.md','L-DCM4-CALIBRATION-SOURCE-20261017'),errors)
    require(not errors,str(errors))
    report={'exact_head':git('rev-parse','HEAD').decode().strip(),'entry_head':BASE,'directive':DIRECTIVE,
        'classification':'SOURCE_ADMISSION_NEGATIVE_EXTERNAL_EVIDENCE_ACQUIRED_CALIBRATION_BLOCKED',
        'acquisition':acquisition,'unit_tests_passed':result.testsRun,'candidate_decisions':decisions,
        'rejected_integrity_mutants':rejected_integrity_mutants,
        'frozen_input_bindings':prior,'scientific_source_count':16,'production_subtree_byte_identical':True,
        'append_only_ledgers':'PASS','new_structured_records':'PASS',
        'independent_calibration_readiness':'BLOCKED_NO_ESTABLISHED_REFERENT_OR_ADMISSIBLE_SOURCE',
        'requested_calibration_acquisition_complete':False,'global_absence_or_impossibility_proven':False,
        'primary_document_bytes':'retained in external archive; Git binds receipts only',
        'historical_global_governance':'PREEXISTING_DRIFT_NOT_CLAIMED_PASS','automatic_successor':False}
    out.mkdir(parents=True)
    (out/'verification.json').write_text(json.dumps(report,indent=2)+'\n')
    (out/'unit-tests.log').write_text(stream.getvalue());bound={}
    paths=sorted(allowed|set(prior)|{'scripts/validate_governance.py','docs/dc-m4-reaction-architecture-audit.json'})
    for path in paths:
        data=(repo/path).read_bytes();bound[path]=sha(data)
        target=out/'repository'/path;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(data)
    manifest={'exact_head':report['exact_head'],'entry_head':BASE,'file_count':len(bound),
              'bound_repository_files':bound,'verification_sha256':sha((out/'verification.json').read_bytes())}
    (out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    print(json.dumps({'package_checks':'PASS','classification':report['classification'],
          'unit_tests':result.testsRun,'sources_reviewed':8,**acquisition,'bound_files':len(bound)}))


if __name__=='__main__':main()
