#!/usr/bin/env python3
"""Bound phase scope, sealed protocol, distinct review and raw measurement identity."""
import argparse,hashlib,importlib.util,json,pathlib,subprocess
from calibration_native_checks import require
from verify_chemical_kinetic_reference import compare_numbers
BASE='552e3795c09961ba56ceef4f8da2503fbb63e0f9'
LIB='digital-protocell/crates/chemistry-core/src/lib.rs'
EXPORT=b'\n/// Opt-in composed native chemical and structural hybrid reference.\n#[cfg(feature = "chemical-reference-port")]\npub mod chemical_composed_port;\n'
RECORDS={'.agent/'+n+'.md' for n in ('CURRENT','PROJECT_PROFILE','DIRECTIVES','OUTCOMES','LEARNINGS','RECORD','REPO_MAP')}|{'STORAGE_MAP.md'}
CORE={LIB,'digital-protocell/crates/chemistry-core/src/chemical_composed_port.rs','digital-protocell/crates/chemistry-core/tests/chemical_composed_port.rs','digital-protocell/crates/chemistry-core/examples/chemical_composed_port.rs','docs/dc-m4-native-composed-directive.md','docs/dc-m4-native-composed-protocol.json','docs/dc-m4-native-composed-sources.json','docs/dc-m4-native-composed-preparation-clarification.json','docs/dc-m4-native-composed-integration.md','docs/dc-m4-native-composed-measurements.json','scripts/qualify_native_composed.py','scripts/test_native_composed_oracle.py','scripts/verify_native_composed.py','.github/workflows/dc-m4-native-composed.yml'}
REVIEW='docs/dc-m4-native-composed-review.json'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 ap=argparse.ArgumentParser();ap.add_argument('--output',required=True);ap.add_argument('--require-review',action='store_true');a=ap.parse_args();r=pathlib.Path(__file__).resolve().parents[1];out=pathlib.Path(a.output);out.mkdir(parents=True,exist_ok=True)
 def git(*args):return subprocess.check_output(['git',*args],cwd=r)
 changed=set(git('diff','--name-only',BASE).decode().splitlines())|set(git('ls-files','--others','--exclude-standard').decode().splitlines());old=set(git('ls-tree','-r','--name-only',BASE).decode().splitlines())
 require(changed<=CORE|RECORDS|{REVIEW},'scope drift '+str(changed-(CORE|RECORDS|{REVIEW})));require(changed&old<=RECORDS|{LIB},'previous science changed')
 require((r/LIB).read_bytes()==git('show',BASE+':'+LIB)+EXPORT,'old library/defaults changed beyond exact opt-in export')
 for n in ('DIRECTIVES','OUTCOMES','LEARNINGS','RECORD','REPO_MAP'):
  f='.agent/'+n+'.md';require((r/f).read_bytes().startswith(git('show',BASE+':'+f)),'append-only prefix '+f)
 require(sha(r/'docs/dc-m4-native-composed-protocol.json')=='86aff8841f05b3711f20cb3d298689842fc8657d57b9902088d092fd2b43f42c','sealed protocol changed')
 require(sha(r/'docs/dc-m4-native-composed-preparation-clarification.json')=='e6ca2f425cc4205e026424f0d508420a2221cadefac27426578e26f3c109b3ff','pre-outcome clarification changed')
 if a.require_review:
  rev=json.loads((r/REVIEW).read_text());require(rev['entry_head']==BASE and rev['decision']=='ACCEPTED_BOUNDED_COMPOSED_NATIVE_HYBRID_ONLY','independent acceptance missing');require(set(rev['reviewed_files'])==CORE,'review coverage')
  for f,h in rev['reviewed_files'].items():require(sha(r/f)==h,'stale review '+f)
  spec=importlib.util.spec_from_file_location('gov',r/'scripts/validate_governance.py');gov=importlib.util.module_from_spec(spec);spec.loader.exec_module(gov)
  def section(f,title):
   for name,body in gov.iter_sections((r/f).read_text()):
    if name==title:return '## '+name+'\n'+body
   raise ValueError('missing governed section '+title)
  errors=[];d='D-20261019-dcm4-native-composed';ids=gov.validate_directives(section('.agent/DIRECTIVES.md',d),errors);gov.validate_outcomes(section('.agent/OUTCOMES.md',d+' - PARTIAL'),ids,errors);gov.validate_learnings(section('.agent/LEARNINGS.md','L-DCM4-NATIVE-COMPOSED-20261019'),errors);require(not errors,str(errors))
  q=json.loads((out/'qualification.json').read_text());stored=json.loads((r/'docs/dc-m4-native-composed-measurements.json').read_text());require(q['input_sha256']==sha(out/'native-observations.json') and q['binary_sha256']==sha(out/'native-composed'),'raw/binary result binding')
  for k in ('head','input_sha256','binary_sha256'):q.pop(k);stored.pop(k)
  compare_numbers(q,stored);require(q['accepted_native_steps']==9000 and q['prepared_case_count']==5,'fixed campaign inventory')
  require(q['native_full_autonomous_generator_qualified'] is False and q['empirical_full_target_qualified'] is False and q['whole_organism_qualified'] is False,'bounded target overclaim');require(q['production_values']==dict.fromkeys(('K','lambda','theta','epsilon_A')),'production parameters assigned')
 native={'digital-protocell/Cargo.toml','digital-protocell/Cargo.lock','digital-protocell/crates/chemistry-core/Cargo.toml','digital-protocell/crates/chemistry-core/src/material_mesh.rs','digital-protocell/crates/chemistry-core/src/mesh_mechanics.rs','digital-protocell/crates/chemistry-core/src/chemical_reference_port.rs','scripts/qualify_chemical_work_owners.py','scripts/validate_governance.py'}
 bindings={}
 for f in sorted(CORE|RECORDS|native|{REVIEW}):
  if (r/f).is_file():
   dst=out/'repository'/f;dst.parent.mkdir(parents=True,exist_ok=True);dst.write_bytes((r/f).read_bytes());bindings[f]=sha(dst)
 v={'head':git('rev-parse','HEAD').decode().strip(),'entry_head':BASE,'prior_tracked_files':len(old),'all_previous_science_frozen_except_exact_feature_gated_lib_export':True,'append_only_prefixes':'PASS','sealed_protocol_enforced':True,'independent_review_enforced':a.require_review,'bundled_source_bindings':bindings,'production_values':dict.fromkeys(('K','lambda','theta','epsilon_A')),'empirical_full_target_qualified':False,'whole_organism_qualified':False,'automatic_successor':False}
 (out/'scope.json').write_text(json.dumps(v,indent=2)+'\n');print(json.dumps(v))
if __name__=='__main__':main()
