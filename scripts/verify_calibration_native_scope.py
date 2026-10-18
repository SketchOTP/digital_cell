#!/usr/bin/env python3
"""Enforce additive bounded scope and separate independent review bindings."""
import argparse,hashlib,json,pathlib,subprocess
BASE='bc44dbac88b0754ae53f030d0852f462468ce3af'
RECORDS={'.agent/'+n+'.md' for n in ('CURRENT','PROJECT_PROFILE','DIRECTIVES','OUTCOMES','LEARNINGS','RECORD','REPO_MAP')}|{'STORAGE_MAP.md'}
MODIFIED={'digital-protocell/crates/chemistry-core/Cargo.toml','digital-protocell/crates/chemistry-core/src/lib.rs'}
CORE={
'docs/dc-m4-calibration-native-directive.md','docs/dc-m4-calibration-native-protocol.json','docs/dc-m4-calibration-native-qualification.md','docs/dc-m4-calibration-native-measurements.json',
'scripts/calibration_native_checks.py','scripts/test_calibration_native_checks.py','scripts/qualify_calibration_native.py','scripts/verify_calibration_native_scope.py',
'.github/workflows/dc-m4-calibration-native-qualification.yml',
'digital-protocell/crates/chemistry-core/src/chemical_reference_port.rs','digital-protocell/crates/chemistry-core/tests/chemical_reference_port.rs','digital-protocell/crates/chemistry-core/examples/chemical_reference_port.rs'}|MODIFIED
REVIEW='docs/dc-m4-calibration-native-review.json'
def require(x,m):
    if not x:raise ValueError(m)
def main():
    p=argparse.ArgumentParser();p.add_argument('--output',required=True);p.add_argument('--require-review',action='store_true');a=p.parse_args()
    r=pathlib.Path(__file__).resolve().parents[1];out=pathlib.Path(a.output);out.mkdir(parents=True,exist_ok=True)
    def git(*args):return subprocess.check_output(['git',*args],cwd=r)
    old=set(git('ls-tree','-r','--name-only',BASE).decode().splitlines())
    changed=set(git('diff','--name-only',BASE).decode().splitlines())|set(git('ls-files','--others','--exclude-standard').decode().splitlines())
    require(changed<=CORE|RECORDS|{REVIEW},'scope drift '+str(changed-(CORE|RECORDS|{REVIEW})))
    require(changed&old<=RECORDS|MODIFIED,'old scientific file changed')
    for n in ('DIRECTIVES','OUTCOMES','LEARNINGS','RECORD','REPO_MAP'):
        f='.agent/'+n+'.md';require((r/f).read_bytes().startswith(git('show',BASE+':'+f)),'ledger prefix '+f)
    cargo='digital-protocell/crates/chemistry-core/Cargo.toml';lib='digital-protocell/crates/chemistry-core/src/lib.rs'
    oldcargo=git('show',BASE+':'+cargo).decode();require((r/cargo).read_text()==oldcargo.replace('long-experiments = []','long-experiments = []\nchemical-reference-port = ["serde_json/float_roundtrip"]'),'Cargo/default feature drift')
    require((r/lib).read_bytes()==git('show',BASE+':'+lib)+b'\n/// Opt-in fixed-volume reference chamber coupled to actual native mesh mechanics.\n#[cfg(feature = "chemical-reference-port")]\npub mod chemical_reference_port;\n','native library body drift')
    proto=r/'docs/dc-m4-calibration-native-protocol.json';require(hashlib.sha256(proto.read_bytes()).hexdigest()=='4a12e5510736e0d5abe427a42beb67d78b4335f8319dd0da22459f6aa32707c8','premesurement seal drift')
    if a.require_review:
        review=json.loads((r/REVIEW).read_text());require(review['entry_head']==BASE and review['decision']=='ACCEPTED_BOUNDED_DESIGNED_RECOVERY_AND_NATIVE_MECHANICS_ONLY','review disposition')
        require(set(review['reviewed_files'])==CORE,'review coverage')
        for f,h in review['reviewed_files'].items():require(hashlib.sha256((r/f).read_bytes()).hexdigest()==h,'stale review '+f)
    bindings={}
    for f in sorted(CORE|MODIFIED|RECORDS|{REVIEW,'digital-protocell/Cargo.lock','digital-protocell/Cargo.toml','digital-protocell/crates/chemistry-core/src/material_mesh.rs','digital-protocell/crates/chemistry-core/src/mesh_mechanics.rs'}):
        if (r/f).is_file():
            dest=out/'repository'/f;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes((r/f).read_bytes());bindings[f]=hashlib.sha256(dest.read_bytes()).hexdigest()
    v={'head':git('rev-parse','HEAD').decode().strip(),'entry_head':BASE,'all_prior_tracked_science_frozen_except_exact_opt_in_feature_export':True,'append_only_prefixes':'PASS','independent_review_enforced':a.require_review,'bundled_source_bindings':bindings,'prior_tracked_files':len(old)}
    (out/'scope.json').write_text(json.dumps(v,indent=2)+'\n');print(json.dumps(v))
if __name__=='__main__':main()
