#!/usr/bin/env python3
"""Offline exact algebra, preservation and distinct-review package verification."""
from __future__ import annotations
import argparse, hashlib, importlib.util, io, json, subprocess, sys, unittest
from pathlib import Path

BASE='004d7f6c775b5adf2a092cde33d42c349b8f4cee'
DIRECTIVE='D-20261017-dcm4-calibration-reference-mapping'
CORE={'docs/dc-m4-reference-mapping-directive.md','docs/dc-m4-reference-mapping.md',
      'docs/dc-m4-reference-mapping.json','scripts/reference_mapping.py',
      'scripts/test_reference_mapping.py','scripts/verify_reference_mapping.py',
      '.github/workflows/dc-m4-reference-mapping.yml'}
REVIEW='docs/dc-m4-reference-mapping-review.json'
RECORDS={'.agent/'+n+'.md' for n in ('CURRENT','PROJECT_PROFILE','DIRECTIVES','OUTCOMES','LEARNINGS','RECORD','REPO_MAP')}|{'STORAGE_MAP.md'}

def require(v,m):
    if not v: raise ValueError(m)

def sha(b):return hashlib.sha256(b).hexdigest()

def module(name,path):
    s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s)
    sys.modules[name]=m;s.loader.exec_module(m);return m

def main():
    ap=argparse.ArgumentParser();ap.add_argument('--repo',type=Path,default=Path('.'))
    ap.add_argument('--output',type=Path,required=True);a=ap.parse_args();r=a.repo.resolve();out=a.output.resolve()
    require(not out.exists() and not out.is_relative_to(r),'fresh external output required')
    sys.dont_write_bytecode=True;sys.path.insert(0,str(r/'scripts'))
    def git(*args):return subprocess.check_output(['git',*args],cwd=r)
    git('diff','--check',BASE)
    old=set(git('ls-tree','-r','--name-only',BASE).decode().splitlines())
    changed=set(git('diff','--name-only',BASE).decode().splitlines())
    unknown=set(git('ls-files','--others','--exclude-standard').decode().splitlines())
    allowed=CORE|{REVIEW}|RECORDS
    require((changed|unknown)<=allowed,'reference mapping scope drift')
    require((CORE|{REVIEW}).isdisjoint(old),'new reference files overwrite prior work')
    require(not git('diff','--name-only',BASE,'--','digital-protocell'),'production source drift')
    for name in ('DIRECTIVES','OUTCOMES','LEARNINGS','RECORD','REPO_MAP'):
        p='.agent/'+name+'.md';require((r/p).read_bytes().startswith(git('show',BASE+':'+p)),'append-only drift '+p)
    # Every old tracked file except the explicitly mutable records is frozen by scope.
    frozen={p:sha(git('show',BASE+':'+p)) for p in old if p.startswith(('docs/dc-m4-','scripts/verify_','scripts/calibration_source_','scripts/test_calibration_source_','.github/workflows/dc-m4-'))}
    frozen.update(json.loads((r/'docs/dc-m4-reaction-architecture-audit.json').read_text())['source_bindings'])
    for review_path in sorted((r/'docs').glob('dc-m4-*-review.json')):
        if str(review_path.relative_to(r)) in old:
            for p,h in json.loads(review_path.read_text()).get('reviewed_files',{}).items():
                require(p in old and sha(git('show',BASE+':'+p))==h,'prior review binding drift '+p)
                frozen[p]=h
    for p,h in frozen.items():require(sha((r/p).read_bytes())==h,'frozen input drift '+p)
    review=json.loads((r/REVIEW).read_text());require(review['entry_head']==BASE and review['decision']=='ACCEPTED_CONDITIONAL_REFERENCE_MAPPING_ONLY','independent decision missing')
    require(set(review['reviewed_files'])==CORE,'review coverage mismatch')
    for p,h in review['reviewed_files'].items():require(sha((r/p).read_bytes())==h,'stale review '+p)
    data=json.loads((r/'docs/dc-m4-reference-mapping.json').read_text())
    require(data['schema']=='digital_cell_reference_mapping_v1' and data['entry_head']==BASE and data['directive']==DIRECTIVE,'reference identity')
    require(data['status']=='PARTIAL_BLOCKED_FULL_CALIBRATION' and data['production_values']==dict.fromkeys(('K','lambda','theta','epsilon_A')),'calibration overclaim')
    for key in ('full_calibration_established','source_species_mapping_established','kinetic_mapping_established','source_affinity_mapping_established','fitting_performed','native_runtime_modified','automatic_successor'):
        require(data[key] is False,'unsupported positive declaration '+key)
    require(data['admitted_target_observations']==0 and type(data['admitted_target_observations']) is int,'invented observations')
    require(data['fixture']=={'built':False,'measured':False,'fixed_P_L0_only':True,'kref':None,'sL':None,'sa':None,'independent_calibration_evidence':False,'energy_scale':None,'energy_identity':'Uref=e*Ec'},'fixture overclaim')
    require(data['scales']==dict.fromkeys(('ell','tau','p','f','e')),'numerical scales assigned')
    algebra=module('reference_mapping',r/'scripts/reference_mapping.py')
    checks=algebra.evaluate_reference_checks()
    require(checks['full_calibration_established'] is False,'algebra relabelled calibration')
    tests=module('reference_mapping_tests',r/'scripts/test_reference_mapping.py')
    stream=io.StringIO();result=unittest.TextTestRunner(stream=stream,verbosity=2).run(unittest.defaultTestLoader.loadTestsFromModule(tests))
    require(result.wasSuccessful() and result.testsRun>=8,stream.getvalue())
    gov=module('governance',r/'scripts/validate_governance.py')
    def section(p,title):
        for n,b in gov.iter_sections((r/p).read_text()):
            if n==title:return '## '+n+'\n'+b
        raise ValueError('missing record '+title)
    errors=[];ids=gov.validate_directives(section('.agent/DIRECTIVES.md',DIRECTIVE),errors)
    gov.validate_outcomes(section('.agent/OUTCOMES.md',DIRECTIVE+' - PARTIAL'),ids,errors)
    gov.validate_learnings(section('.agent/LEARNINGS.md','L-DCM4-REFERENCE-MAPPING-20261017'),errors)
    require(not errors,str(errors))
    report={'exact_head':git('rev-parse','HEAD').decode().strip(),'entry_head':BASE,'directive':DIRECTIVE,
            'classification':'CONDITIONAL_REFERENCE_UNIT_WORK_MAPS_ESTABLISHED_FULL_CALIBRATION_BLOCKED',
            'reference_checks':checks,'unit_tests_passed':result.testsRun,'frozen_input_bindings':dict(sorted(frozen.items())),
            'all_prior_tracked_files_except_mutable_records_frozen':True,'production_subtree_unchanged':True,
            'append_only_prefixes':'PASS','new_structured_records':'PASS','full_calibration_established':False,
            'independent_observations_admitted':0,'fixture_built_or_measured':False,'automatic_successor':False,
            'historical_global_governance':'PREEXISTING_DRIFT_NOT_CLAIMED_PASS'}
    out.mkdir(parents=True);(out/'verification.json').write_text(json.dumps(report,indent=2)+'\n');(out/'unit-tests.log').write_text(stream.getvalue())
    bindings={}
    for p in sorted(allowed|set(frozen)|{'scripts/validate_governance.py','docs/dc-m4-reaction-architecture-audit.json'}):
        b=(r/p).read_bytes();bindings[p]=sha(b);dest=out/'repository'/p;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(b)
    manifest={'exact_head':report['exact_head'],'entry_head':BASE,'file_count':len(bindings),'bound_repository_files':bindings,'verification_sha256':sha((out/'verification.json').read_bytes())}
    (out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    print(json.dumps({'package_checks':'PASS','unit_tests':result.testsRun,'reference_checks':checks,'frozen_files':len(frozen),'bound_files':len(bindings)}))

if __name__=='__main__':main()
