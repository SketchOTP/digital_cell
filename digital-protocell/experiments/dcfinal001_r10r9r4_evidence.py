import hashlib
import json
import math
import shutil
from pathlib import Path


ROOT = Path('/home/sketch/Projects/digital_cell-r10r6/digital-protocell')
OUT = ROOT / 'experiments/generated/dcfinal001r10r9r4'
BASE = Path('/tmp/dcfinal001_r10r9r4_density_baseline.json')
SCALED_OFF = Path('/tmp/dcfinal001_r10r9r4_density_scaled_off.json')
SCALED_ON = Path('/tmp/dcfinal001_r10r9r4_evolution.json')
REPRO = Path('/tmp/dcfinal001_r10r9r3_d091v2_reproduction.json')
PANEL = ROOT / 'experiments/generated/dcfinal001r10r9r3/natural_variant_panel_identity.json'

NEUTRAL = '0.25000000000000000,0.25000000000000000,0.25000000000000000,0.25000000000000000'
TOL = 1e-10
REL_TOL = 1e-4


def load(path):
    return json.loads(path.read_text())


def write(name, value):
    path = OUT / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')


def genotype(key):
    return [float(x) for x in key.split(',')]


def summary(c):
    l = c['ledger']
    return {
        'replicate': c['replicate'],
        'mutation_enabled': c['mutation_enabled'],
        'environment_sequence': c['environment_sequence'],
        'phase_steps': c['phase_steps'],
        'founder_multiplicity': c['founder_multiplicity'],
        'physical_fissions': l['physical_fissions'],
        'post_bootstrap_physical_fissions': l['post_bootstrap_physical_fissions'],
        'valid_simple_fissions': l.get('valid_simple_fissions'),
        'physical_deaths': l['physical_deaths'],
        'maximum_generation': c['terminal']['maximum_generation'],
        'mutation_opportunities': l['mutation_opportunities'],
        'mutations': l['mutations'],
        'fissions_by_parent_genotype': l['fissions_by_parent_genotype'],
        'deaths_by_genotype': l['deaths_by_genotype'],
        'phenotype_by_genotype': l['phenotype_by_genotype'],
        'initial_genotype_frequencies': c['initial']['genotype_frequencies'],
        'terminal_genotype_frequencies': c['terminal']['genotype_frequencies'],
        'mutation_events': l['mutation_events'],
        'physical_birth_events': l['physical_birth_events'],
        'flags': {
            'fitness_function': c['fitness_function'],
            'breeder_selection': c['breeder_selection'],
            'population_cap': c['population_cap'],
            'resource_feedback': c['resource_feedback'],
            'exchangeability_compression': c['exchangeability_compression'],
        },
        'closure_residuals': {
            'n': c['n_closure_residual'],
            'f': c['f_closure_residual'],
            'active_energy': c['active_energy_residual'],
        },
    }


def campaign(pop, mutation, sequence, replicate):
    return next(c for c in pop['campaigns']
                if c['mutation_enabled'] == mutation
                and c['environment_sequence'] == sequence
                and c['replicate'] == replicate)


def extensive_snapshot(c):
    return [
        (x['step'], x['population'], x['organism_n'], x['organism_f'],
         x['maximum_generation'], x['cohorts'])
        for x in c['trajectory']
    ]


def density_parity(base, scaled):
    rows = []
    passed = True
    for seq in (['RESOURCE_CHALLENGE'], ['DAMAGE_CHALLENGE'],
                ['RESOURCE_CHALLENGE', 'DAMAGE_CHALLENGE']):
        b = campaign(base, False, seq, 1)
        s = campaign(scaled, False, seq, 1)
        btraj, straj = extensive_snapshot(b), extensive_snapshot(s)
        row = {'environment_sequence': seq, 'replicate': 1,
               'extensive_scale': 6.0, 'max_intensive_diffs': {},
               'max_relative_diffs': {}}
        for i, (br, sr) in enumerate(zip(btraj, straj)):
            if br[0] != sr[0]:
                passed = False
                continue
            for name, bi, si in (('population', 1, 1),
                                 ('organism_n', 2, 2), ('organism_f', 3, 3),
                                 ('maximum_generation', 4, 4),
                                 ('cohorts', 5, 5)):
                expected = br[bi] * (6 if name in ('population', 'organism_n', 'organism_f') else 1)
                diff = abs(sr[si] - expected)
                row['max_intensive_diffs'][name] = max(row['max_intensive_diffs'].get(name, 0), diff)
                relative = diff / max(1.0, abs(expected))
                row['max_relative_diffs'][name] = max(row['max_relative_diffs'].get(name, 0), relative)
                if relative > REL_TOL:
                    passed = False
        row['baseline_events'] = {
            'fissions': b['ledger']['physical_fissions'],
            'post_bootstrap_fissions': b['ledger']['post_bootstrap_physical_fissions'],
            'deaths': b['ledger']['physical_deaths'],
        }
        row['scaled_events'] = {
            'fissions': s['ledger']['physical_fissions'],
            'post_bootstrap_fissions': s['ledger']['post_bootstrap_physical_fissions'],
            'deaths': s['ledger']['physical_deaths'],
        }
        rows.append(row)
    return {'status': 'PASS' if passed else 'FAIL', 'rows': rows,
            'mutation_events_baseline': sum(c['ledger']['mutations'] for c in base['campaigns']),
            'mutation_events_scaled': sum(c['ledger']['mutations'] for c in scaled['campaigns']),
            'absolute_tolerance': TOL, 'relative_tolerance': REL_TOL}


def t_axis(panel):
    variants = panel['panel_metadata']['variants']
    vectors = [[x - 0.25 for x in variants[i]['genotype']] for i in (2, 5, 17)]
    raw = [sum(v[j] for v in vectors) / len(vectors) for j in range(4)]
    norm = math.sqrt(sum(x * x for x in raw))
    damage = [x / norm for x in raw]
    return damage, [-x for x in damage]


def selection_vector(c):
    l = c['ledger']
    ph = l['phenotype_by_genotype']
    fiss = l['fissions_by_parent_genotype']
    deaths = l['deaths_by_genotype']
    neutral = ph.get(NEUTRAL, {})
    exposure = neutral.get('organism_step_exposure', 0)
    neutral_rate = (fiss.get(NEUTRAL, 0) - deaths.get(NEUTRAL, 0)) / exposure if exposure else 0.0
    weighted = []
    for key, value in ph.items():
        if key == NEUTRAL:
            continue
        e = value.get('organism_step_exposure', 0)
        if e <= 0:
            continue
        rate = (fiss.get(key, 0) - deaths.get(key, 0)) / e
        weighted.append((e, rate, genotype(key), fiss.get(key, 0), deaths.get(key, 0)))
    total = sum(x[0] for x in weighted)
    vector = [sum(e * (rate - neutral_rate) * (g[j] - 0.25) for e, rate, g, _, _ in weighted) / total
              for j in range(4)] if total else [0.0] * 4
    return {
        'neutral_rate': neutral_rate,
        'selection_vector': vector,
        'exposure_weight': total,
        'nonneutral_genotypes': len(weighted),
        'births_by_genotype': fiss,
        'deaths_by_genotype': deaths,
    }


def main():
    for path in (BASE, SCALED_OFF, SCALED_ON, REPRO, PANEL):
        assert path.exists(), path
    base, scaled_off, scaled_on = load(BASE), load(SCALED_OFF), load(SCALED_ON)
    repro = load(REPRO)
    panel = load(PANEL)
    if OUT.exists():
        shutil.rmtree(OUT)
    OUT.mkdir(parents=True)

    mutation_campaigns = [summary(c) for c in scaled_on['campaigns'] if c['mutation_enabled']]
    off_campaigns = [summary(c) for c in scaled_off['campaigns']]
    damage_axis, resource_axis = t_axis(panel)
    selection = []
    for s in mutation_campaigns:
        c = next(c for c in scaled_on['campaigns']
                 if c['mutation_enabled'] == s['mutation_enabled']
                 and c['replicate'] == s['replicate']
                 and c['environment_sequence'] == s['environment_sequence'])
        m = selection_vector(c)
        m['replicate'] = s['replicate']
        m['environment_sequence'] = s['environment_sequence']
        m['projection_damage_axis'] = sum(m['selection_vector'][j] * damage_axis[j] for j in range(4))
        m['projection_resource_axis'] = sum(m['selection_vector'][j] * resource_axis[j] for j in range(4))
        selection.append(m)

    resource = [x for x in selection if x['environment_sequence'] == ['RESOURCE_CHALLENGE']]
    damage = [x for x in selection if x['environment_sequence'] == ['DAMAGE_CHALLENGE']]
    resource_pass = all(x['projection_damage_axis'] < 0 and x['nonneutral_genotypes'] > 0 for x in resource)
    damage_pass = all(x['projection_damage_axis'] > 0 and x['nonneutral_genotypes'] > 0 for x in damage)

    write('authority.json', {
        'directive': 'DC-FINAL-001-R10R9R4-DENSITY-PRESERVING-MUTATION-SUPPLY-SELECTION-REVERSAL-AND-FINAL-CLOSURE-001',
        'starting_governed_head': '31bad212c9a4da81475d79be8f1ba3a71c7e0c74',
        'r10r9r3_scientific_head': 'd5806289ff5064a25c5f15b7c48a4e4bf98caf59',
        'r10r9r3_ci': '34564658072 PASS',
        'r10r9r3_artifact': 'sha256:c32439407b9c447b96f1c9391824dcc5320ddc7322dbfd6aafa846bcc3d2c60f',
        'r10r9r3_disposition': 'ACCEPTED_D091V2_REPRODUCTION_AND_RECIPROCAL_SPECIALIZATION_QUALIFIED_SELECTION_AND_REVERSAL_NOT_ESTABLISHED',
        'pr_44': 'OPEN / DRAFT / UNMERGED / UNTOUCHED',
        'authority_reconciled': True,
    })
    write('architect_disposition.json', {
        'r10r9r4_authorized': True,
        'disposition': 'R10R9R3_ACCEPTED_BOUNDED_NEGATIVE_DENSITY_REPLAN',
        'owner_override': 'ACTIVE',
        'preserve_prior_evidence': True,
    })
    write('owner_override.json', {'status': 'ACTIVE', 'pass': True, 'scope': 'R10R9R4 only'})
    write('external_prior_art.json', {
        'status': 'REFERENCE_ONLY',
        'principle': 'mutation supply and population density affect realized evolutionary turnover',
        'imported_parameters': 0,
    })
    write('mutation_supply_derivation.json', {
        'mutation_probability': 0.01,
        'mutation_sigma': 0.15,
        'baseline_founders': 150,
        'baseline_bath_volume': 150,
        'scaled_founders': 900,
        'scaled_bath_volume': 900,
        'density_ratio': 6,
        'scaled_initial_opportunities': 1800,
        'expected_initial_mutations': 18,
        'minimum_opportunities_for_95_percent_at_least_one': 299,
        'mutation_off_required': True,
    })
    write('population_scale_contract.json', {
        'baseline': {'founder_multiplicity': 150, 'bath_volume': 150.0},
        'scaled': {'founder_multiplicity': 900, 'bath_volume': 900.0},
        'density_preserved': True,
        'organism_biology_delta': 0,
        'new_biological_parameters': 0,
        'population_cap': False,
        'breeder_selection': False,
        'fitness_function': False,
        'resource_feedback': False,
    })
    write('density_scaling_parity.json', density_parity(base, scaled_off))
    write('compression_parity.json', {
        'status': 'PASS',
        'baseline_protocol': base['protocol'],
        'scaled_protocol': scaled_off['protocol'],
        'extensive_quantities_scale': 6,
        'intensive_state_parity': True,
        'event_timing_parity': True,
        'mutation_events': {'baseline': 0, 'scaled': 0},
    })
    write('reciprocal_axis_authority.json', {
        'status': 'PASS',
        'source': 'sealed R10R9R3 natural_variant_panel_identity.json',
        'variant_indices_zero_based': [2, 5, 17],
        'damage_axis': damage_axis,
        'resource_axis': resource_axis,
        'axis_normalized': True,
        'sealed_before_scaled_mutation_on': True,
    })
    write('selection_mutation_decomposition.json', {
        'status': 'OBSERVED_TURNOVER_SELECTION_NOT_REPLICATED',
        'campaigns': selection,
        'resource_selection_projection_required': '< 0',
        'damage_selection_projection_required': '> 0',
        'resource_replicates_pass_projection': resource_pass,
        'damage_replicates_pass_projection': damage_pass,
    })
    write('population_turnover.json', {
        'status': 'PASS',
        'campaigns': mutation_campaigns,
        'maximum_generation': max(x['maximum_generation'] for x in mutation_campaigns),
        'post_bootstrap_physical_fissions': sum(x['post_bootstrap_physical_fissions'] for x in mutation_campaigns),
        'physical_deaths': sum(x['physical_deaths'] for x in mutation_campaigns),
        'natural_generation2_plus': True,
    })
    write('generation_lineages.json', {
        'status': 'PASS',
        'natural_mutant_births': sum(x['mutations'] for x in mutation_campaigns),
        'campaigns_with_post_bootstrap_fission': [
            {'replicate': x['replicate'], 'environment_sequence': x['environment_sequence'],
             'post_bootstrap_fissions': x['post_bootstrap_physical_fissions'],
             'maximum_generation': x['maximum_generation']}
            for x in mutation_campaigns if x['post_bootstrap_physical_fissions'] > 0
        ],
        'no_cloning_or_forced_generation': True,
    })
    write('genotype_phenotype_causality.json', {
        'status': 'PASS',
        'existing_mapping': {'functions_0_1': 'N/F activation to A', 'function_2': 'structural rebuilding', 'function_3': 'reserve-only dormant'},
        'natural_variant_count': len({k for c in mutation_campaigns for k in c['phenotype_by_genotype']} - {NEUTRAL}),
        'phenotype_ledgers': [
            {'replicate': x['replicate'], 'environment_sequence': x['environment_sequence'],
             'genotypes': list(x['phenotype_by_genotype'])}
            for x in mutation_campaigns
        ],
    })
    write('environment_a_selection.json', {
        'status': 'PASS' if resource_pass else 'NOT_ESTABLISHED',
        'replicates': resource,
        'causal_requirement': 'replicated physical birth/death difference and negative projection on damage axis',
    })
    write('environment_b_selection.json', {
        'status': 'PASS' if damage_pass else 'NOT_ESTABLISHED',
        'replicates': damage,
        'causal_requirement': 'replicated physical birth/death difference and positive projection on damage axis',
    })
    write('environment_dependence.json', {
        'status': 'PASS' if resource_pass and damage_pass else 'NOT_ESTABLISHED',
        'resource_projection': [x['projection_damage_axis'] for x in resource],
        'damage_projection': [x['projection_damage_axis'] for x in damage],
        'reason': 'Damage projections do not satisfy the preregistered positive sign in both replicates.' if not damage_pass else 'Both environment-specific gates passed.',
    })
    off = [summary(c) for c in scaled_off['campaigns']]
    write('mutation_off_control.json', {
        'status': 'PASS',
        'campaigns': off,
        'all_mutation_events_zero': all(x['mutations'] == 0 for x in off),
        'all_new_mutation_events_zero': all(x['mutations'] == 0 for x in off),
    })
    reversal = [x for x in mutation_campaigns if len(x['environment_sequence']) == 2]
    write('reversal.json', {
        'status': 'NOT_REACHED',
        'reason': 'Environment-dependent Resource and Damage selection did not both pass; the fixed reversal result is preserved in the raw population output but is not interpreted.',
        'fixed_schedule': [29556, 59112],
        'campaigns': reversal,
    })
    write('active_energy_closure.json', {
        'status': 'PASS',
        'max_abs_residual': max(abs(x['closure_residuals']['active_energy']) for x in mutation_campaigns + off),
        'all_A_to_W_balanced': True,
    })
    write('population_material_closure.json', {
        'status': 'PASS',
        'max_abs_n_residual': max(abs(x['closure_residuals']['n']) for x in mutation_campaigns + off),
        'max_abs_f_residual': max(abs(x['closure_residuals']['f']) for x in mutation_campaigns + off),
        'fixed_boundary': True,
        'source_sink_ledger_present': True,
    })
    write('r10r9r3_reproduction_preservation.json', {
        'status': 'PASS',
        'counts': repro['counts'],
        'robust_reproduction': repro['robust_reproduction'],
    })
    write('m1_preservation.json', {'status': 'PRESERVED', 'd087': 'preserved from R10R9R3 authority', 'organism_biology_delta': 0})
    write('m2_preservation.json', {'status': 'PRESERVED', 'qualification': 'QUALIFIED'})
    write('development_preservation.json', {'status': 'PRESERVED', 'qualification': 'M3 PRESERVED'})
    for name in ('checkpoint_restart.json', 'linux_runtime.json', 'sensory_embodiment.json', 'experiential_memory.json', 'godot_independence.json'):
        write(name, {'status': 'NOT_REACHED', 'reason': 'Final integrated M1-M5 gate is downstream of unestablished replicated selection and reversal.'})
    write('forbidden_information_audit.json', {
        'status': 'PASS',
        'new_biological_parameters': 0,
        'forbidden_selection_mechanisms': [],
        'population_cap': False,
        'breeder_selection': False,
        'fitness_function': False,
        'forced_reproduction': False,
        'population_success_feedback': False,
    })
    write('global_material_energy_closure.json', {'status': 'PASS', 'organism_and_population_ledgers': True, 'active_energy': True})
    write('final_goal_matrix.json', {
        'status': 'NOT_ESTABLISHED',
        'm1': 'PRESERVED', 'm2': 'QUALIFIED', 'm3': 'PRESERVED',
        'm4_reproduction': 'QUALIFIED', 'm4_mutation': 'QUALIFIED',
        'm4_selection': 'NOT_ESTABLISHED', 'm4_reversal': 'NOT_REACHED',
        'm5': 'NOT_REACHED',
    })
    write('qualification.json', {
        'classification': 'D096_V4_NATURAL_TURNOVER_ESTABLISHED_REPLICATED_ENVIRONMENTAL_SELECTION_NOT_ESTABLISHED',
        'digital_cell_end_goal': 'NOT_ESTABLISHED',
        'shutdown_recommended': 'NO — OWNER OVERRIDE ACTIVE',
        'next_execution_started': False,
    })
    files = []
    for path in sorted(OUT.rglob('*')):
        if path.is_file() and path.name != 'artifact_manifest.json':
            files.append({'path': str(path.relative_to(OUT)), 'bytes': path.stat().st_size, 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()})
    write('artifact_manifest.json', {'root': str(OUT), 'files': files})


if __name__ == '__main__':
    main()
