#!/usr/bin/env python3
"""Source/history/role/review binding; scientific negatives remain negatives."""
import argparse,hashlib,importlib.util,json,pathlib,subprocess
from calibration_native_checks import require
from verify_chemical_kinetic_reference import compare_numbers
BASE='a800279a355dc49060d81f805800f0a6def7b7a8'
CORE=set([
 ".github/workflows/dc-m4-mechanochemical-acquisition.yml",
 ".github/workflows/dc-m4-mechanochemical-owners.yml",
 "digital-protocell/crates/chemistry-core/examples/chemical_work_owners.rs",
 "docs/dc-m4-chemical-work-owner-measurements.json",
 "docs/dc-m4-chemical-work-owner-protocol.json",
 "docs/dc-m4-mechanochemical-evidence/README.md",
 "docs/dc-m4-mechanochemical-evidence/acquisition-receipt.json",
 "docs/dc-m4-mechanochemical-evidence/publisher-metadata.json",
 "docs/dc-m4-mechanochemical-evidence/publisher-zip-inventory.json",
 "docs/dc-m4-mechanochemical-evidence/raw-selected.zip",
 "docs/dc-m4-mechanochemical-observation-check.json",
 "docs/dc-m4-mechanochemical-observation-protocol.json",
 "docs/dc-m4-mechanochemical-observations.json",
 "docs/dc-m4-mechanochemical-ownership-directive.md",
 "docs/dc-m4-mechanochemical-source-format-amendment.json",
 "docs/dc-m4-mechanochemical-sources.json",
 "docs/dc-m4-mechanochemical-unit-evidence.json",
 "docs/dc-m4-mechanochemical-work-ownership.md",
 "scripts/acquire_mechanochemical_observations.py",
 "scripts/chemical_work_ownership.py",
 "scripts/mechanochemical_observations.py",
 "scripts/qualify_chemical_work_owners.py",
 "scripts/qualify_mechanochemical_observations.py",
 "scripts/test_chemical_work_oracle.py",
 "scripts/test_chemical_work_ownership.py",
 "scripts/test_mechanochemical_observations.py",
 "scripts/verify_mechanochemical_owners.py"
])
RECORDS={'.agent/'+n+'.md' for n in ('CURRENT','PROJECT_PROFILE','DIRECTIVES','OUTCOMES','LEARNINGS','RECORD','REPO_MAP')}|{'STORAGE_MAP.md'}
REVIEW='docs/dc-m4-mechanochemical-owners-review.json'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 p=argparse.ArgumentParser();p.add_argument('--output',required=True);p.add_argument('--require-review',action='store_true');a=p.parse_args();r=pathlib.Path(__file__).resolve().parents[1];out=pathlib.Path(a.output);out.mkdir(parents=True,exist_ok=True)
 def git(*args):return subprocess.check_output(['git',*args],cwd=r)
 old=set(git('ls-tree','-r','--name-only',BASE).decode().splitlines());changed=set(git('diff','--name-only',BASE).decode().splitlines())|set(git('ls-files','--others','--exclude-standard').decode().splitlines())
 require(changed<=CORE|RECORDS|{REVIEW},'scope drift '+str(changed-(CORE|RECORDS|{REVIEW})));require(changed&old<=RECORDS,'previous native/scientific source changed')
 for name in ('DIRECTIVES','OUTCOMES','LEARNINGS','RECORD','REPO_MAP'):
  f='.agent/'+name+'.md';require((r/f).read_bytes().startswith(git('show',BASE+':'+f)),'append-only prefix '+f)
 for f,h in [('docs/dc-m4-mechanochemical-observation-protocol.json','d39788c379e9b669c88543f48e42b3ad7c890ff9613ab18aa7846454cde08ab0'),('docs/dc-m4-chemical-work-owner-protocol.json','5bb179d4c6ede6ad70ba95c7432d9cdce450a07b2b6fdc1cfa07c8aa0fbe37f2'),('docs/dc-m4-mechanochemical-source-format-amendment.json','8d7fb62c132a33bade406075783020472f57e10899f54eff99b0bd5576da8928')]:require(sha(r/f)==h,'sealed scientific input changed '+f)
 sources=json.loads((r/'docs/dc-m4-mechanochemical-sources.json').read_text());require(sources['empirical_full_target_qualified'] is False and sources['force_conjugate_work_qualified'] is False,'source overclaim')
 for f,v in sources['curated_files'].items():require((r/f).stat().st_size==v['bytes'] and sha(r/f)==v['sha256'],'original input identity '+f)
 if a.require_review:
  review=json.loads((r/REVIEW).read_text());require(review['entry_head']==BASE and review['decision']=='ACCEPTED_BOUNDED_FINITE_EVENT_OWNERS_AND_LIMITED_RAW_OBSERVATIONS','independent review decision')
  require(set(review['reviewed_files'])==CORE,'independent review coverage')
  for f,h in review['reviewed_files'].items():require(sha(r/f)==h,'stale independent review '+f)
  spec=importlib.util.spec_from_file_location('gov',r/'scripts/validate_governance.py');gov=importlib.util.module_from_spec(spec);spec.loader.exec_module(gov)
  def section(file,title):
   for name,body in gov.iter_sections((r/file).read_text()):
    if name==title:return '## '+name+'\n'+body
   raise ValueError('missing governed record '+title)
  errors=[];d='D-20261018-dcm4-mechanochemical-owners';ids=gov.validate_directives(section('.agent/DIRECTIVES.md',d),errors);gov.validate_outcomes(section('.agent/OUTCOMES.md',d+' - PARTIAL'),ids,errors);gov.validate_learnings(section('.agent/LEARNINGS.md','L-DCM4-MECHANOCHEMICAL-OWNERS-20261018'),errors);require(not errors,str(errors))
  q=json.loads((out/'owner-qualification.json').read_text());stored=json.loads((r/'docs/dc-m4-chemical-work-owner-measurements.json').read_text());require(q['input_sha256']==sha(out/'owner-observations.json') and q['native_sha256']==sha(out/'native-observations.json'),'raw measurement binding')
  for k in ('head','input_sha256','native_sha256'):q.pop(k);stored.pop(k)
  compare_numbers(q,stored);require(q['native_full_integration_qualified'] is False and q['identified_biological_mechanism'] is False and q['whole_organism_qualified'] is False,'owner/native/organism overclaim')
  empirical=json.loads((out/'physical-observations.json').read_text());check=json.loads((out/'physical-qualification.json').read_text());require(check['observations_sha256']==sha(out/'physical-observations.json'),'physical raw binding')
  compare_numbers(empirical,json.loads((r/'docs/dc-m4-mechanochemical-observations.json').read_text()));sc=json.loads((r/'docs/dc-m4-mechanochemical-observation-check.json').read_text());check.pop('observations_sha256');sc.pop('observations_sha256');compare_numbers(check,sc)
  require(check['selected_pairs_by_file']==[2,3] and check['known_gap_boundaries_in_selected_legs']==6448,'reported negative source result changed');require(check['six_pair_prospective_coverage'] is False and check['all_selected_paths_continuous'] is False and check['force_conjugate_external_work_qualified'] is False,'negative coverage/work relabeled')
  for x in (q,empirical,check):require(x['production_values']==dict.fromkeys(('K','lambda','theta','epsilon_A')),'production parameter assignment')
 native={'digital-protocell/Cargo.toml','digital-protocell/Cargo.lock','digital-protocell/crates/chemistry-core/Cargo.toml'}|{'digital-protocell/crates/chemistry-core/src/'+n+'.rs' for n in ('material_mesh','mesh_mechanics','mesh_transport','mesh_growth','mesh_reactions','mesh_fission','mesh_topology','mesh_population')}
 bindings={}
 for f in sorted(CORE|RECORDS|native|{REVIEW}):
  if (r/f).is_file():
   dst=out/'repository'/f;dst.parent.mkdir(parents=True,exist_ok=True);dst.write_bytes((r/f).read_bytes());bindings[f]=sha(dst)
 v={'head':git('rev-parse','HEAD').decode().strip(),'entry_head':BASE,'prior_tracked_files':len(old),'all_previous_native_scientific_source_frozen':True,'append_only_prefixes':'PASS','independent_review_enforced':a.require_review,'bundled_source_bindings':bindings,'scientific_negative_coverage_preserved':True,'whole_organism_qualified':False,'production_values':dict.fromkeys(('K','lambda','theta','epsilon_A')),'automatic_successor':False}
 (out/'scope.json').write_text(json.dumps(v,indent=2)+'\n');print(json.dumps(v))
if __name__=='__main__':main()
