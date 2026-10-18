#!/usr/bin/env python3
"""Freeze prior work, bind independent review and reconstruct scoped evidence."""
import argparse,hashlib,json,pathlib,subprocess,importlib.util
from calibration_native_checks import require
BASE='59edb9ca2b1730d870f7a7f238f05718b97347ac'
LIB='digital-protocell/crates/chemistry-core/src/lib.rs'
RECORDS={'.agent/'+n+'.md' for n in ('CURRENT','PROJECT_PROFILE','DIRECTIVES','OUTCOMES','LEARNINGS','RECORD','REPO_MAP')}|{'STORAGE_MAP.md'}
CORE={LIB,
'digital-protocell/crates/chemistry-core/src/chemical_body_port.rs','digital-protocell/crates/chemistry-core/tests/chemical_body_port.rs','digital-protocell/crates/chemistry-core/examples/chemical_body_port.rs',
'docs/dc-m4-empirical-organism-directive.md','docs/dc-m4-empirical-organism-protocol.json','docs/dc-m4-empirical-organism-domain-amendment.json','docs/dc-m4-empirical-constituent-protocol.json','docs/dc-m4-empirical-background-amendment.json','docs/dc-m4-empirical-sources.json','docs/dc-m4-empirical-organism-integration.md','docs/dc-m4-empirical-constituent-measurements.json','docs/dc-m4-chemical-body-measurements.json',
'docs/dc-m4-empirical-constituent-evidence/README.md','docs/dc-m4-empirical-constituent-evidence/publisher-metadata.json','docs/dc-m4-empirical-constituent-evidence/raw-selected.zip',
'scripts/acquire_empirical_constituent.py','scripts/empirical_constituent.py','scripts/measure_empirical_constituent.py','scripts/test_empirical_constituent.py','scripts/chemical_body_reference.py','scripts/qualify_chemical_body.py','scripts/test_chemical_body_reference.py','scripts/verify_empirical_organism.py',
'.github/workflows/dc-m4-empirical-acquisition.yml','.github/workflows/dc-m4-empirical-organism-integration.yml'}
REVIEW='docs/dc-m4-empirical-organism-review.json'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    p=argparse.ArgumentParser();p.add_argument('--output',required=True);p.add_argument('--require-review',action='store_true');a=p.parse_args();r=pathlib.Path(__file__).resolve().parents[1];out=pathlib.Path(a.output);out.mkdir(parents=True,exist_ok=True)
    def git(*v):return subprocess.check_output(['git',*v],cwd=r)
    old=set(git('ls-tree','-r','--name-only',BASE).decode().splitlines());changed=set(git('diff','--name-only',BASE).decode().splitlines())|set(git('ls-files','--others','--exclude-standard').decode().splitlines())
    require(changed<=CORE|RECORDS|{REVIEW},'scope drift '+str(changed-(CORE|RECORDS|{REVIEW})));require(changed&old<=RECORDS|{LIB},'previous scientific input drift')
    require((r/LIB).read_bytes()==git('show',BASE+':'+LIB)+b'\n/// Finite-volume full-state body and native midpoint-remesh reference extension.\n#[cfg(feature = "chemical-reference-port")]\npub mod chemical_body_port;\n','old native library/default changed')
    for n in ('DIRECTIVES','OUTCOMES','LEARNINGS','RECORD','REPO_MAP'):
        f='.agent/'+n+'.md';require((r/f).read_bytes().startswith(git('show',BASE+':'+f)),'append-only ledger '+f)
    for f,h in [('docs/dc-m4-empirical-organism-protocol.json','5d3df634aff00b443f7f3b8a4c7d253b526f2b49c5fc2435272f8fb4f9bae09a'),('docs/dc-m4-empirical-constituent-protocol.json','ad866b55d5c9bcfaa5e0139d9aefbc861ff0517a7e84b074e222709bc3cab0a5'),('docs/dc-m4-empirical-organism-domain-amendment.json','b4e261d014bc23e0eaf382e6826716ebdb0c206dbeef4cbfb0cd78f513281050'),('docs/dc-m4-empirical-background-amendment.json','bcf6ee72b3398341e76f25490383c5ad8b61b062b90f45c1774a3b1d7c7fbad8')]:require(sha(r/f)==h,'pre-outcome seal drift '+f)
    sources=json.loads((r/'docs/dc-m4-empirical-sources.json').read_text());require(sources['independent_role']=='EXPERIMENTAL_OPTICAL_TRAP_RELAXATION_ONLY' and sources['empirical_full_target_qualified'] is False,'empirical source overclaim')
    for f,b in sources['curated_files'].items():require(sha(r/f)==b['sha256'] and (r/f).stat().st_size==b['bytes'],'curated empirical input drift '+f)
    if a.require_review:
        review=json.loads((r/REVIEW).read_text());require(review['entry_head']==BASE and review['decision']=='ACCEPTED_EMPIRICAL_CONSTITUENT_AND_BOUNDED_BODY_REMESH_ONLY','missing independent review')
        require(set(review['reviewed_files'])==CORE,'review coverage')
        for f,h in review['reviewed_files'].items():require(sha(r/f)==h,'stale independent review '+f)
        spec=importlib.util.spec_from_file_location('gov',r/'scripts/validate_governance.py');gov=importlib.util.module_from_spec(spec);spec.loader.exec_module(gov)
        def section(file,title):
            for n,b in gov.iter_sections((r/file).read_text()):
                if n==title:return '## '+n+'\n'+b
            raise ValueError('missing record '+title)
        errors=[];directive='D-20261018-dcm4-empirical-organism';ids=gov.validate_directives(section('.agent/DIRECTIVES.md',directive),errors);gov.validate_outcomes(section('.agent/OUTCOMES.md',directive+' - PARTIAL'),ids,errors);gov.validate_learnings(section('.agent/LEARNINGS.md','L-DCM4-EMPIRICAL-ORGANISM-20261018'),errors);require(not errors,str(errors))
        from verify_chemical_kinetic_reference import compare_numbers
        actual=json.loads((out/'body-qualification.json').read_text());stored=json.loads((r/'docs/dc-m4-chemical-body-measurements.json').read_text())
        for k in ('head','binary_sha256','input_sha256'):actual.pop(k);stored.pop(k)
        compare_numbers(actual,stored)
        empirical=json.loads((out/'empirical-measurements.json').read_text())
        require(empirical['linear_relaxation_shape_transfer_within_sealed_budgets'] is True,'empirical heldout shape budget failed')
        require(empirical['empirical_full_target_qualified'] is False and empirical['chemical_kinetic_mapping_qualified'] is False and empirical['production_values']==dict.fromkeys(('K','lambda','theta','epsilon_A')),'empirical overclaim')
        require(actual['whole_organism_qualified'] is False and actual['empirical_full_target_qualified'] is False and actual['production_values']==dict.fromkeys(('K','lambda','theta','epsilon_A')),'native whole-target overclaim')
        compare_numbers(empirical,json.loads((r/'docs/dc-m4-empirical-constituent-measurements.json').read_text()))
    bindings={}
    for f in sorted(CORE|RECORDS|{REVIEW,'digital-protocell/Cargo.lock','digital-protocell/Cargo.toml','digital-protocell/crates/chemistry-core/Cargo.toml','digital-protocell/crates/chemistry-core/src/material_mesh.rs','digital-protocell/crates/chemistry-core/src/mesh_mechanics.rs','digital-protocell/crates/chemistry-core/src/chemical_reference_port.rs'}):
        if (r/f).is_file():
            dst=out/'repository'/f;dst.parent.mkdir(parents=True,exist_ok=True);dst.write_bytes((r/f).read_bytes());bindings[f]=sha(dst)
    v={'head':git('rev-parse','HEAD').decode().strip(),'entry_head':BASE,'prior_tracked_files':len(old),'all_previous_science_frozen_except_exact_feature_gated_lib_export':True,'append_only_prefixes':'PASS','independent_review_enforced':a.require_review,'bundled_source_bindings':bindings,'production_values':dict.fromkeys(('K','lambda','theta','epsilon_A')),'empirical_full_target_qualified':False,'whole_organism_qualified':False,'automatic_successor':False}
    (out/'scope.json').write_text(json.dumps(v,indent=2)+'\n');print(json.dumps(v))
if __name__=='__main__':main()
