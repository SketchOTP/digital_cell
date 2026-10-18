#!/usr/bin/env python3
"""Verify actual computational measurements, source custody and isolated scope."""
from __future__ import annotations
import argparse,hashlib,importlib.util,io,json,subprocess,sys,unittest
from pathlib import Path
BASE='524fb8c85b35923fd4fd8e0d1b91abbff629f460'
DIRECTIVE='D-20261018-dcm4-chemical-kinetic-reference'
CORE={'docs/dc-m4-chemical-kinetic-directive.md','docs/dc-m4-chemical-kinetic-reference.md',
      'docs/dc-m4-chemical-kinetic-sources.json','docs/dc-m4-chemical-kinetic-protocol.json',
      'docs/dc-m4-chemical-kinetic-measurements.json','scripts/chemical_kinetic_reference.py',
      'scripts/test_chemical_kinetic_reference.py','scripts/measure_chemical_kinetic_reference.py',
      'scripts/verify_chemical_kinetic_reference.py','.github/workflows/dc-m4-chemical-kinetic-reference.yml'}
REVIEW='docs/dc-m4-chemical-kinetic-review.json'
RECORDS={'.agent/'+n+'.md' for n in ('CURRENT','PROJECT_PROFILE','DIRECTIVES','OUTCOMES','LEARNINGS','RECORD','REPO_MAP')}|{'STORAGE_MAP.md'}
def require(v,m):
    if not v:raise ValueError(m)
def sha(b):return hashlib.sha256(b).hexdigest()
def module(name,path):
    s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);sys.modules[name]=m;s.loader.exec_module(m);return m

def compare_numbers(observed,stored,path=''):
    if isinstance(stored,dict):
        require(set(observed)==set(stored),'measurement fields '+path)
        for k in stored:compare_numbers(observed[k],stored[k],path+'/'+k)
    elif isinstance(stored,list):
        require(len(observed)==len(stored),'measurement length '+path)
        for i,(a,b) in enumerate(zip(observed,stored)):compare_numbers(a,b,path+'/'+str(i))
    elif type(stored) in (int,float):
        require(type(observed) in (int,float) and abs(observed-stored)<=1e-9*(1+abs(stored)),'measurement replay drift '+path)
    else:require(observed==stored,'measurement declaration drift '+path)

def main():
    ap=argparse.ArgumentParser();ap.add_argument('--repo',type=Path,default=Path('.'));ap.add_argument('--output',type=Path,required=True);a=ap.parse_args();r=a.repo.resolve();out=a.output.resolve()
    require(not out.exists() and not out.is_relative_to(r),'fresh external output required')
    sys.dont_write_bytecode=True;sys.path.insert(0,str(r/'scripts'))
    def git(*args):return subprocess.check_output(['git',*args],cwd=r)
    git('diff','--check',BASE)
    old=set(git('ls-tree','-r','--name-only',BASE).decode().splitlines());allowed=CORE|{REVIEW}|RECORDS
    changed=set(git('diff','--name-only',BASE).decode().splitlines())|set(git('ls-files','--others','--exclude-standard').decode().splitlines())
    require(changed<=allowed and (CORE|{REVIEW}).isdisjoint(old),'scope or prior source drift')
    require(not git('diff','--name-only',BASE,'--','digital-protocell'),'production source drift')
    for n in ('DIRECTIVES','OUTCOMES','LEARNINGS','RECORD','REPO_MAP'):
        p='.agent/'+n+'.md';require((r/p).read_bytes().startswith(git('show',BASE+':'+p)),'append-only drift '+p)
    frozen={p:sha(git('show',BASE+':'+p)) for p in old if p.startswith(('docs/dc-m4-','scripts/verify_','.github/workflows/dc-m4-'))}
    frozen.update(json.loads((r/'docs/dc-m4-reaction-architecture-audit.json').read_text())['source_bindings'])
    for rp in sorted((r/'docs').glob('dc-m4-*-review.json')):
        if str(rp.relative_to(r)) in old:
            for p,h in json.loads(rp.read_text()).get('reviewed_files',{}).items():
                require(p in old and sha(git('show',BASE+':'+p))==h,'prior review drift '+p);frozen[p]=h
    for p,h in frozen.items():require(sha((r/p).read_bytes())==h,'frozen input drift '+p)
    review=json.loads((r/REVIEW).read_text());require(review['entry_head']==BASE and review['decision']=='ACCEPTED_BUILT_MEASURED_COMPUTATIONAL_REFERENCE_ONLY','missing independent review')
    require(set(review['reviewed_files'])==CORE,'review coverage')
    for p,h in review['reviewed_files'].items():require(sha((r/p).read_bytes())==h,'stale review '+p)
    proto=json.loads((r/'docs/dc-m4-chemical-kinetic-protocol.json').read_text())
    require(proto['entry_head']==BASE and proto['production_values']==dict.fromkeys(('K','lambda','theta','epsilon_A')) and proto['physical_apparatus_built'] is False and proto['automatic_successor'] is False,'production/physical overclaim')
    require(sha((r/'docs/dc-m4-chemical-kinetic-protocol.json').read_bytes())=='d82cbabff2dd56e1bb4e39aff0099cfc3b31de3d63c65f516dd2a70ac5e2649c','premeasurement protocol seal drift')
    sources=json.loads((r/'docs/dc-m4-chemical-kinetic-sources.json').read_text())
    require(sources['admitted_role']=='PUBLISHED_COMPUTATIONAL_CHEMICAL_REFERENCE_SPECIFICATION_ONLY' and sources['target_calibration_data_admitted'] is False and sources['external_code_executed'] is False,'source role drift')
    require([(d['bytes'],d['sha256']) for d in sources['documents']]==[(1967105,'fadc9f37a18565c244e6b73853914b04a008fe8a0ce440d52adf8f5afc7b009d'),(546881,'6664420e9c4f53ec56945e2a23cd140a8231e4753b3d8bf13694a487e7916a9b')],'primary receipt drift')
    tests=module('chemical_reference_tests',r/'scripts/test_chemical_kinetic_reference.py');stream=io.StringIO();test_result=unittest.TextTestRunner(stream=stream,verbosity=2).run(unittest.defaultTestLoader.loadTestsFromModule(tests));require(test_result.wasSuccessful() and test_result.testsRun==11,stream.getvalue())
    # Parent measurement harness executes all sealed controls; raw trajectories
    # and measured diagnostics are archived separately from immutable code bindings.
    out.mkdir(parents=True)
    measure=module('chemical_reference_measure',r/'scripts/measure_chemical_kinetic_reference.py');observed=measure.run(r,out/'measurements')
    stored=json.loads((r/'docs/dc-m4-chemical-kinetic-measurements.json').read_text());compare_numbers(observed,stored)
    gov=module('governance',r/'scripts/validate_governance.py')
    def section(p,title):
        for n,b in gov.iter_sections((r/p).read_text()):
            if n==title:return '## '+n+'\n'+b
        raise ValueError('missing record '+title)
    errors=[];ids=gov.validate_directives(section('.agent/DIRECTIVES.md',DIRECTIVE),errors);gov.validate_outcomes(section('.agent/OUTCOMES.md',DIRECTIVE+' - COMPLETE'),ids,errors);gov.validate_learnings(section('.agent/LEARNINGS.md','L-DCM4-CHEMICAL-KINETIC-20261018'),errors);require(not errors,str(errors))
    report={'exact_head':git('rev-parse','HEAD').decode().strip(),'entry_head':BASE,'directive':DIRECTIVE,'classification':observed['classification'],
            'unit_tests_passed':test_result.testsRun,'measurement_cases':6,'runs_at_two_steps':12,'measurement_replay_within_declared_numeric_tolerance':True,
            'frozen_input_bindings':dict(sorted(frozen.items())),'all_prior_tracked_files_except_mutable_records_frozen':True,'production_subtree_unchanged':True,'append_only_prefixes':'PASS','new_structured_records':'PASS',
            'source_admission':'published computational chemical specification only','new_mechanical_extension':'designed and measured computationally; not externally observed biology',
            'target_calibration_parameters_identified':False,'chemical_heat_measured':False,'physical_apparatus_built':False,'automatic_successor':False,
            'primary_fulltext_custody':'external originals; CI binds receipts only','historical_global_governance':'PREEXISTING_DRIFT_NOT_CLAIMED_PASS'}
    (out/'verification.json').write_text(json.dumps(report,indent=2)+'\n');(out/'unit-tests.log').write_text(stream.getvalue());bindings={}
    for p in sorted(allowed|set(frozen)|{'scripts/validate_governance.py','docs/dc-m4-reaction-architecture-audit.json'}):
        data=(r/p).read_bytes();bindings[p]=sha(data);dest=out/'repository'/p;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(data)
    measured={str(p.relative_to(out)):sha(p.read_bytes()) for p in sorted((out/'measurements').rglob('*')) if p.is_file()}
    manifest={'exact_head':report['exact_head'],'entry_head':BASE,'file_count':len(bindings),'bound_repository_files':bindings,'verification_sha256':sha((out/'verification.json').read_bytes()),'measured_output_bindings':measured}
    (out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n');print(json.dumps({'package':'PASS','unit_tests':11,'measurement_cases':6,'measured_runs':12,'bound_files':len(bindings),'frozen_files':len(frozen),'full_target_calibration':False}))
if __name__=='__main__':main()
