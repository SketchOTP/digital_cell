#!/usr/bin/env python3
"""Verify the bounded reference program, empty acquisition plan and custody."""
from __future__ import annotations

import argparse
import copy
import csv
from fractions import Fraction as F
import hashlib
import importlib.util
import io
import itertools
import json
from pathlib import Path
import subprocess
import sys
import unittest

BASE = 'fb3366995de9a8dbc5bceb6d3ae24ed653972dca'
DIRECTIVE = 'D-20261017-dcm4-reference-prototype'
REVIEWED = {
    'scripts/dc_m4_reference_prototype.py',
    'scripts/test_dc_m4_reference_prototype.py',
    'scripts/verify_reference_prototype.py',
    '.github/workflows/dc-m4-reference-prototype.yml',
    'docs/dc-m4-reference-prototype.md',
    'docs/dc-m4-reference-prototype-directive.md',
    'docs/dc-m4-calibration-acquisition-plan.md',
    'docs/dc-m4-calibration-acquisition-plan.json',
    'docs/dc-m4-calibration-force-template.csv',
}
REVIEW = 'docs/dc-m4-reference-prototype-review.json'
AGENT = [f'.agent/{n}.md' for n in
         ('CURRENT', 'PROJECT_PROFILE', 'DIRECTIVES', 'OUTCOMES', 'LEARNINGS', 'RECORD', 'REPO_MAP')]


def require(value, message):
    if not value:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def load_module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def validate_empty_plan(plan, rows):
    require(plan['schema'] == 'digital_cell_independent_calibration_acquisition_plan_v1'
            and plan['entry_head'] == BASE, 'plan identity')
    require(plan['readiness'] == 'MISSING_EVIDENCE' and
            plan['status'] == 'PLAN_DEFINED_DATA_MISSING', 'missing evidence promoted')
    require(plan['production_values'] == dict.fromkeys(('K', 'lambda', 'theta', 'epsilon_A')),
            'production values assigned')
    require(plan['source_registry'] == [] and plan['raw_dataset_registry'] == []
            and type(plan['observed_response_count']) is int
            and plan['observed_response_count'] == 0, 'nonempty data in plan-only scope')
    for key in ('campaign_executed', 'fitting_performed', 'automatic_successor',
                'acquisition_or_fitting_authorized', 'prototype_is_independent_calibration'):
        require(plan[key] is False, 'unperformed acquisition/fit claim: '+key)
    expected_missing = {
        'source_admission', 'source_identity_and_hash', 'noncircular_dependency_audit',
        'A_W_P_species_stoichiometry_mapping', 'force_length_time_amount_units',
        'uncertainty_certificates', 'physical_decision_resolution',
        'replicate_and_sampling_policy', 'estimator_version', 'holdout_custodian',
        'pre_response_manifest'}
    require(all(plan['preregistration'][k] == 'MISSING' for k in expected_missing)
            and plan['preregistration']['absolute_accuracy_budgets'] is None
            and plan['preregistration']['maximum_enclosure_widths'] is None,
            'unobserved prerequisites invented')
    require(plan['source_admission']['biological_identity_assumed'] is False,
            'biological identity assumed')
    force = plan['force']
    require((force['planned_rows'], force['training_rows'], force['heldout_rows']) == (80, 4, 76)
            and force['preserve_correlations'] is True
            and force['response_and_provenance_fields_blank'] is True, 'force design drift')
    design = list(itertools.product((F(0), F(1,4), F(1,2), F(3,4), F(1)),
                                   (F(-1,4), F(0), F(1,4), F(1,2)),
                                   ((F(1), F(1)), (F(1), F(2)), (F(2), F(1)), (F(2), F(2)))))
    require(len(rows) == 80, 'force template row count')
    train = []
    for i, (row, (x, e, (p, l))) in enumerate(zip(rows, design), 1):
        require(row['row_id'] == f'F{i:03d}' and
                tuple(F(row[k]) for k in ('x', 'epsilon', 'P_over_Pref', 'L0_over_Lref'))
                == (x, e, p, l), 'template fixture/identity drift')
        is_train = x in (0, 1) and e in (0, F(1,4)) and p == l == 1
        require(row['role'] == ('TRAIN' if is_train else 'HOLDOUT'), 'holdout role drift')
        require(row['status'] == 'NOT_ACQUIRED' and
                all(row[k] == '' for k in ('observation_low', 'observation_high', 'source_id',
                                           'raw_sha256', 'uncertainty_certificate')),
                'template populated or observation invented')
        if is_train:
            train.append((e, -x))
    require(any(a*d-b*c != 0 for (a,b), (c,d) in itertools.combinations(train,2)),
            'force design rank deficient')
    kinetic = plan['kinetic']
    require((kinetic['train_design_points'], kinetic['holdout_design_points']) == (6,12)
            and kinetic['observation_windows'] == ['h','h/2','h/4']
            and kinetic['gross_turnover_inferred'] is False
            and kinetic['prototype_executes_native_J'] is False, 'kinetic scope drift')
    require(plan['fuel']['alternative'] == 'epsilon_A=(W_out+Q_loss+DeltaU_independent)/A_source'
            and plan['fuel']['candidate_Ec_or_debit_as_anchor_forbidden'] is True
            and plan['fuel']['W_is_heat'] is False
            and plan['fuel']['regeneration_authorized'] is False, 'fuel ownership drift')
    require(plan['uncertainty']['retain_joint_correlations'] is True
            and plan['uncertainty']['zero_inclusion_is_pass'] is False
            and plan['uncertainty']['budget_widening_or_probe_movement_after_responses'] is False,
            'uncertainty/holdout policy weakened')
    require(set(plan['forbidden_inputs']) >= {'R13_thirty_response_signs',
            'candidate_or_prototype_generated_observations'}, 'circular calibration input admitted')
    require(all(r['adoption'] == 'REFERENCE_METHOD_ONLY' and r['imported_values'] is False
                for r in plan['method_references']) and len(plan['method_references']) == 2,
            'external methods promoted to project data')
    return 'MISSING_EVIDENCE'


def plan_controls(plan, rows):
    require(validate_empty_plan(plan, rows) == 'MISSING_EVIDENCE', 'readiness overclaim')
    mutants = []
    for key, value in [('production_values', {**plan['production_values'], 'K': 2}),
                       ('source_registry', [{'source': 'prototype', 'independent': True}]),
                       ('observed_response_count', 80), ('fitting_performed', True)]:
        bad = copy.deepcopy(plan); bad[key] = value; mutants.append((bad, rows))
    bad = copy.deepcopy(plan); bad['fuel']['alternative'] = 'epsilon_A=W_out/A_source'
    mutants.append((bad, rows))
    bad_rows = copy.deepcopy(rows); bad_rows[0]['role'] = 'TRAIN'; mutants.append((plan, bad_rows))
    bad_rows = copy.deepcopy(rows); bad_rows[0]['observation_low'] = '1'
    mutants.append((plan, bad_rows))
    for bad, bad_rows in mutants:
        try:
            validate_empty_plan(bad, bad_rows)
        except ValueError:
            continue
        raise ValueError('plan negative control accepted')
    return {'readiness': 'MISSING_EVIDENCE', 'observed_response_count': 0,
            'template_rows': 80, 'train_rows': 4, 'heldout_rows': 76,
            'rejected_plan_mutants': len(mutants)}


def independent_root_controls(ref):
    count = 0
    # Independent closed-form tensile-branch oracle, varied unit fixtures; none
    # of these numbers is an empirical calibration or production selection.
    for phi, lam, polarity, L0, fraction in itertools.product(
            (F(1,4), F(1,2), F(1)), (F(1,4), F(1,2), F(1)),
            (F(1), F(2)), (F(1), F(2)), (F(1,4), F(1,2), F(3,4))):
        p = ref.Params(F(2), lam, F(1), F(3), F(14), F(1), F(1,2), F(1,50))
        m = ref.Material(polarity, L0, F(1), phi)
        acc = ref.Accuracy(F(1,10**9), F(1,10**10), F(1,10**10), F(1,10**8), 80)
        run = ref.Run.cold(p,m,acc,F(100))
        d = polarity*fraction
        expected = L0 + p.K*lam*d/L0 / (p.gamma/p.h +
                   (p.K*polarity/L0**2+phi*p.k_s/L0)/2)
        result = run.cycle(d,(L0,4*L0)); c = result.certificates[-1]
        require(c.lo <= expected <= c.hi and c.lo <= c.root <= c.hi,
                'analytic root outside certified enclosure')
        require(c.f_lo <= 0 <= c.f_hi and c.width <= acc.width
                and abs(c.force_residual) <= acc.force and abs(c.energy_error) <= acc.step_energy,
                'root certificate accuracy')
        require(result.state.A <= run.state.A and result.state.Q_drag >= 0
                and result.state.Q_chem >= 0, 'source or heat sign')
        ec = ref.coupling(p,m,result.state.a,result.state.L)
        us = ref.stretch(p,m,result.state.L)
        require(ec <= p.K*polarity/2*max(F(9),(1+lam)**2)
                and us <= p.k_s*L0/2*max(F(1),9*phi), 'capacity certificate')
        # The selected rational root is approximate. Preserve its defect; never
        # call a nonzero defect exact energy conservation.
        imbalance = p.epsilon_A*(result.state.A-run.state.A)+ec+us+result.state.Q_chem+result.state.Q_drag
        require(imbalance == c.energy_error == result.state.signed_energy_error
                and result.state.absolute_energy_error == abs(imbalance), 'unrepaired energy defect')
        count += 1
    return {'independent_analytic_root_cases': count, 'source_capacity_and_error_certificates': 'PASS'}


def main():
    ap = argparse.ArgumentParser(); ap.add_argument('--repo',type=Path,default=Path('.'))
    ap.add_argument('--output',type=Path,required=True); args = ap.parse_args()
    repo, out = args.repo.resolve(), args.output.resolve()
    require(not out.is_relative_to(repo), 'evidence belongs outside checkout')
    require(not out.exists(), 'fresh evidence directory required')
    sys.dont_write_bytecode = True
    def git(*a): return subprocess.check_output(['git',*a],cwd=repo)
    git('diff','--check',BASE)
    allowed = REVIEWED | {REVIEW, 'STORAGE_MAP.md', *AGENT}
    changed = set(git('diff','--name-only',BASE).decode().splitlines())
    untracked = set(git('ls-files','--others','--exclude-standard').decode().splitlines())
    require((changed | untracked) <= allowed, 'scope drift')
    require(not git('diff','--name-only',BASE,'--','digital-protocell'), 'production subtree drift')
    for name in ('DIRECTIVES','OUTCOMES','LEARNINGS','RECORD','REPO_MAP'):
        path = f'.agent/{name}.md'
        require((repo/path).read_bytes().startswith(git('show',f'{BASE}:{path}')),
                'append-only prefix drift '+path)
    frozen = json.loads((repo/'docs/dc-m4-reaction-architecture-audit.json').read_text())['source_bindings'].copy()
    prior_docs = ('dc-m4-conversion-work-contract.md','dc-m4-conversion-work-contract.json',
        'dc-m4-conversion-work-directive.md','dc-m4-conversion-work-review.json',
        'dc-m4-work-port-specification.md','dc-m4-work-port-specification.json',
        'dc-m4-work-port-directive.md','dc-m4-work-port-review.json',
        'dc-m4-source-receiver-architecture.md','dc-m4-source-receiver-architecture.json',
        'dc-m4-source-receiver-directive.md','dc-m4-source-receiver-review.json')
    previous = [f'docs/{p}' for p in prior_docs] + [
        'scripts/verify_conversion_work_contract.py','scripts/verify_work_port_specification.py',
        'scripts/verify_source_receiver_architecture.py',
        '.github/workflows/dc-m4-conversion-work-contract.yml',
        '.github/workflows/dc-m4-work-port-specification.yml',
        '.github/workflows/dc-m4-source-receiver-architecture.yml']
    for path in previous: frozen[path] = sha(git('show',f'{BASE}:{path}'))
    for path, digest in frozen.items():
        require(sha((repo/path).read_bytes()) == digest and
                (repo/path).read_bytes() == git('show',f'{BASE}:{path}'), 'frozen input drift '+path)
    review = json.loads((repo/REVIEW).read_text())
    require(review['decision'] == 'ACCEPTED_BOUNDED_REFERENCE_AND_ACQUISITION_PLAN_ONLY'
            and review['entry_head'] == BASE and set(review['reviewed_files']) == REVIEWED,
            'independent review absent/incomplete')
    for path,digest in review['reviewed_files'].items():
        require(sha((repo/path).read_bytes()) == digest, 'stale independent review '+path)
    plan = json.loads((repo/'docs/dc-m4-calibration-acquisition-plan.json').read_text())
    with (repo/'docs/dc-m4-calibration-force-template.csv').open(newline='') as f:
        rows = list(csv.DictReader(f))
    acquisition = plan_controls(plan,rows)
    sys.path.insert(0,str(repo/'scripts'))
    ref = load_module('dc_m4_reference_prototype',repo/'scripts/dc_m4_reference_prototype.py')
    test = load_module('reference_tests',repo/'scripts/test_dc_m4_reference_prototype.py')
    stream = io.StringIO(); suite = unittest.defaultTestLoader.loadTestsFromModule(test)
    result = unittest.TextTestRunner(stream=stream,verbosity=2).run(suite)
    require(result.wasSuccessful() and result.testsRun >= 13, stream.getvalue())
    roots = independent_root_controls(ref)
    demo = ref.jsonable(ref.demo())
    require(demo['calibrated'] is False and demo['native_J_trajectory'] is False
            and len(demo['history']) == 5, 'prototype scope or demo drift')
    governance = load_module('governance',repo/'scripts/validate_governance.py')
    def section(path,title):
        for name,body in governance.iter_sections((repo/path).read_text()):
            if name == title: return '## '+name+'\n'+body
        raise ValueError('missing structured record '+title)
    errors = []
    ids = governance.validate_directives(section('.agent/DIRECTIVES.md',DIRECTIVE),errors)
    governance.validate_outcomes(section('.agent/OUTCOMES.md',DIRECTIVE+' - COMPLETE'),ids,errors)
    governance.validate_learnings(section('.agent/LEARNINGS.md','L-DCM4-REFERENCE-PROTOTYPE-20261017'),errors)
    require(not errors,str(errors))
    evidence = {'exact_head':git('rev-parse','HEAD').decode().strip(),'entry_head':BASE,
        'directive':DIRECTIVE,'classification':'BOUNDED_REFERENCE_PROTOTYPE_VERIFIED_CALIBRATION_DATA_MISSING',
        'unit_tests_passed':result.testsRun,'independent_controls':roots,'acquisition':acquisition,
        'frozen_input_bindings':frozen,'scientific_source_count':16,
        'production_subtree_byte_identical':True,'append_only_ledgers':'PASS','new_structured_records':'PASS',
        'historical_global_governance':'PREEXISTING_DRIFT_NOT_CLAIMED_PASS',
        'reference_runtime_implemented':True,'native_J_trajectory':False,
        'independent_parameter_calibration':'NOT_PERFORMED','full_native_integration':'NOT_IMPLEMENTED',
        'R13_correction':'NOT_ESTABLISHED','automatic_successor':False}
    out.mkdir(parents=True)
    (out/'verification.json').write_text(json.dumps(evidence,indent=2)+'\n')
    (out/'demo.json').write_text(json.dumps(demo,sort_keys=True,indent=2)+'\n')
    (out/'unit-tests.log').write_text(stream.getvalue())
    paths = sorted(allowed | set(frozen) | {'scripts/validate_governance.py',
                                          'docs/dc-m4-reaction-architecture-audit.json'})
    bound = {}
    for path in paths:
        data = (repo/path).read_bytes(); bound[path] = sha(data)
        target = out/'repository'/path; target.parent.mkdir(parents=True,exist_ok=True)
        target.write_bytes(data)
    manifest = {'exact_head':evidence['exact_head'],'entry_head':BASE,'file_count':len(bound),
        'bound_repository_files':bound,'verification_sha256':sha((out/'verification.json').read_bytes()),
        'demo_sha256':sha((out/'demo.json').read_bytes())}
    (out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    print(json.dumps({'checks':'PASS','classification':evidence['classification'],
        'unit_tests':result.testsRun,**roots,**acquisition,'bound_files':len(bound)}))


if __name__ == '__main__':
    main()
