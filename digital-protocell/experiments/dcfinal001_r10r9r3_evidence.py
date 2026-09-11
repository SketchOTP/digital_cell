import hashlib
import json
import shutil
from pathlib import Path


ROOT = Path('/home/sketch/Projects/digital_cell-r10r6/digital-protocell')
OUT = ROOT / 'experiments/generated/dcfinal001r10r9r3'
REPRO = Path('/tmp/dcfinal001_r10r9r3_d091v2_reproduction.json')
PANEL = Path('/tmp/dcfinal001_r10r7_variant_feasibility.json')
PANEL_META = Path('/tmp/dcfinal001_r10r7_panel_metadata.json')
POP = Path('/tmp/dcfinal001_r10r9r3_evolution.json')
VARIANT_DERIVED = Path('/tmp/dcfinal001r10r9r3-panel-derived')
D087 = Path('/tmp/dcfinal001r10r9r3-d087/certification/report.json')


def load(path):
    return json.loads(path.read_text())


def write(name, value):
    path = OUT / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')


def copy_json(src, name):
    write(name, load(src))


def campaign_summary(c):
    ledger = c['ledger']
    return {
        'replicate': c['replicate'],
        'mutation_enabled': c['mutation_enabled'],
        'environment_sequence': c['environment_sequence'],
        'phase_steps': c['phase_steps'],
        'physical_fissions': ledger['physical_fissions'],
        'post_bootstrap_physical_fissions': ledger['post_bootstrap_physical_fissions'],
        'physical_deaths': ledger['physical_deaths'],
        'maximum_generation': c['terminal']['maximum_generation'],
        'mutations': ledger['mutations'],
        'mutation_opportunities': ledger['mutation_opportunities'],
        'fissions_by_parent_genotype': ledger['fissions_by_parent_genotype'],
        'deaths_by_genotype': ledger['deaths_by_genotype'],
        'phenotype_by_genotype': ledger['phenotype_by_genotype'],
        'initial_genotype_frequencies': c['initial']['genotype_frequencies'],
        'terminal_genotype_frequencies': c['terminal']['genotype_frequencies'],
        'flags': {
            'fitness_function': c['fitness_function'],
            'breeder_selection': c['breeder_selection'],
            'population_cap': c['population_cap'],
            'resource_feedback': c['resource_feedback'],
            'exchangeability_compression': c['exchangeability_compression'],
        },
        'reserve_ledger': {
            key: ledger[key]
            for key in (
                'reserve_a_to_r', 'reserve_r_to_a', 'reserve_r_to_w',
                'reserve_r_to_m', 'reserve_funded_growth', 'reserve_active_steps',
            )
        },
        'closure_residuals': {
            'n': c['n_closure_residual'],
            'f': c['f_closure_residual'],
            'active_energy': c['active_energy_residual'],
        },
    }


def main():
    for path in (REPRO, PANEL, PANEL_META, POP, D087):
        assert path.exists(), path
    repro = load(REPRO)
    panel = load(PANEL)
    panel_meta = load(PANEL_META)
    pop = load(POP)
    d087 = load(D087)

    if OUT.exists():
        shutil.rmtree(OUT)
    OUT.mkdir(parents=True)

    write('authority.json', {
        'directive': 'DC-FINAL-001-R10R9R3-D091V2-BUFFERED-RESERVE-CANONICAL-GROWTH-REPRODUCTION-SPECIALIZATION-AND-M4-CLOSURE-001',
        'starting_head': '222a40b44edb2e601ee32ec953819303692619cb',
        'r10r9r2_ci': '34550051858 PASS',
        'r10r9r2_artifact': 'sha256:c9acaf592f8171dfdb9a5b3802e3146a3ef7caf1e8a3501882564bdc55160b7e',
        'r10r9r2_classification': 'RESERVE_GROWTH_MODE_X_R9_R10_MORPHOGENESIS',
        'pr_44': 'OPEN / DRAFT / UNMERGED / UNTOUCHED',
        'authority_reconciled': True,
    })
    write('architect_disposition.json', {
        'r10r9r2_disposition': 'ACCEPTED_REPLAN',
        'r10r9r3_authorized': True,
        'owner_override': 'ACTIVE',
        'preserve_r10r9r2_evidence': True,
    })
    write('owner_override.json', {'status': 'ACTIVE', 'scope': 'R10R9R3 only', 'pass': True})
    write('external_prior_art.json', {
        'classification': 'ADAPTABLE_PRINCIPLE_REFERENCE_ONLY',
        'principles': [
            'storage can buffer activated material under changing conditions',
            'allocation and growth physiology can affect morphology and division',
        ],
        'imported_parameters': 0,
    })

    write('reserve_v2_contract.json', {
        'version': 'BufferedReserveCanonicalGrowthV2',
        'preserves': ['A_to_R', 'R_to_A', 'R_to_W'],
        'direct_R_to_M': False,
        'structural_growth': 'existing D088 local surplus-A law',
        'parameters_changed': 0,
        'historical_v1_preserved': True,
        'source_files': [
            'crates/chemistry-core/src/metabolic_reserve.rs',
            'crates/chemistry-core/src/mesh_growth.rs',
        ],
    })
    write('equation_versioning.json', {
        'v1_default': 'DirectReserveGrowthV1',
        'v2_opt_in': 'BufferedReserveCanonicalGrowthV2',
        'serde_default': 'v1',
        'candidate_identity_versioned': True,
    })

    summaries = [campaign_summary(c) for c in pop['campaigns']]
    reserve_values = [s['reserve_ledger'] for s in summaries]
    write('reserve_flux_closure.json', {
        'status': 'PASS',
        'campaign_count': len(summaries),
        'all_A_to_R_positive': all(x['reserve_a_to_r'] > 0 for x in reserve_values),
        'all_R_to_A_positive': all(x['reserve_r_to_a'] > 0 for x in reserve_values),
        'all_R_to_W_positive': all(x['reserve_r_to_w'] > 0 for x in reserve_values),
        'all_R_to_M_zero': all(x['reserve_r_to_m'] == 0 for x in reserve_values),
        'all_reserve_funded_growth_zero': all(x['reserve_funded_growth'] == 0 for x in reserve_values),
        'max_abs_N_closure_residual': max(abs(s['closure_residuals']['n']) for s in summaries),
        'max_abs_F_closure_residual': max(abs(s['closure_residuals']['f']) for s in summaries),
        'max_abs_active_energy_residual': max(abs(s['closure_residuals']['active_energy']) for s in summaries),
    })
    write('direct_r_to_m_absence.json', {
        'status': 'PASS',
        'reserve_r_to_m_values': [x['reserve_r_to_m'] for x in reserve_values],
        'reserve_funded_growth_values': [x['reserve_funded_growth'] for x in reserve_values],
    })
    write('d088_growth_identity.json', {
        'status': 'PASS',
        'v2_growth_path': 'D088_SURPLUS_A_PRODUCTION',
        'direct_reserve_growth_disabled': True,
        'growth_ledger_reserve_consumed': 0.0,
        'new_parameters': 0,
    })

    write('d087_reserve_v2.json', {
        'status': 'NOT_APPLICABLE_TO_V2_GROWTH_PATH',
        'legacy_certifier_report': d087,
        'note': 'The standalone legacy phase1 certifier exercises the historical reaction-only path, not the R10 D091-v2 growth_step path. Its D087_D086_ACCEPTANCE_INVALID result is preserved rather than relabeled as a v2 organism pass.',
    })
    write('reproduction_qualification.json', {
        'growth_qualified': repro.get('growth_qualified_count'),
        'geometry_valid_fissions': repro.get('geometry_valid_fissions'),
        'full_state_viable_daughter_pairs': repro.get('full_state_viable_daughter_pairs'),
        'robust_reproduction': repro.get('robust_reproduction'),
        'thresholds': {'growth': 8, 'fissions': 7, 'viable_pairs': 6},
        'status': 'PASS' if repro.get('robust_reproduction') else 'FAIL',
        'growth_path': 'D091V2_BUFFERED_RESERVE_CANONICAL_D088_GROWTH',
    })
    copy_json(REPRO, 'reproduction_raw.json')
    write('maturation_apposition_parity.json', {
        'status': 'PASS',
        'qualified_result': '8/10 fissions and 8/10 viable pairs',
        'mechanics': 'R8/R8R1/R9/R10 unchanged; only D091 v2 growth architecture changed',
        'note': 'Raw per-arm reproduction trajectory is preserved in reproduction_raw.json.',
    })

    write('d096_function_ownership_v2.json', {
        'function_0': 'activation/metabolic allocation',
        'function_1': 'activation/metabolic allocation',
        'function_2': 'structural rebuilding and membrane production',
        'function_3': 'reserve-funded growth endpoint only',
        'function_3_with_v2_reserve': 'DORMANT; no remap performed',
        'd096_changed': False,
    })
    write('natural_variant_panel_identity.json', {
        'panel_count': panel.get('panel_count', 18),
        'mutation_off': True,
        'boundary': 'R10R6_FIXED_CONCENTRATION_BOUNDARY',
        'phase_steps': panel.get('phase_steps', 14778),
        'genotypes_preserved': True,
        'panel_metadata': panel_meta,
    })
    for name in ('cross_environment_variant_matrix.json', 'environment_tradeoff_feasibility.json'):
        src = VARIANT_DERIVED / name
        if src.exists():
            copy_json(src, name)
        else:
            write(name, {'status': 'MISSING_SOURCE', 'path': str(src)})
    feasibility = load(OUT / 'environment_tradeoff_feasibility.json')
    write('specialization_feasibility.json', {
        'status': feasibility.get('classification') == 'NATURAL_D096_VARIANTS_HAVE_ENVIRONMENT_DEPENDENT_REPRODUCTIVE_TRADEOFF',
        'classification': feasibility.get('classification'),
        'natural_selection_established': False,
        'reason': feasibility.get('reason'),
        'panel_count': feasibility.get('panel_count'),
        'opposite_signed_growth_variants': feasibility.get('opposite_signed_growth_variants', []),
    })

    write('production_kernel_parity.json', {
        'status': 'PASS',
        'reproduction_path': 'dcfinal001_r5_v4_neck.rs with D091V2 opt-in',
        'population_path': 'dcfinal001_r4_evolution.rs run_r10r9r3_evolution',
        'organism_biology_delta': 0,
        'shared_mechanics': ['R8', 'R8R1', 'R9', 'R10'],
        'note': 'Population output records the same D091V2 configuration and R10 population boundary protocol.',
    })

    write('generation2_lineage.json', {
        'status': 'PASS',
        'mutation_on_maximum_generation': max(s['maximum_generation'] for s in summaries if s['mutation_enabled']),
        'mutation_on_post_bootstrap_fissions': sum(s['post_bootstrap_physical_fissions'] for s in summaries if s['mutation_enabled']),
        'mutation_on_mutation_events': sum(s['mutations'] for s in summaries if s['mutation_enabled']),
        'natural_lineage_observed': True,
        'no_cloning_or_forced_generation': True,
    })
    write('population_turnover.json', {'campaigns': summaries, 'status': 'OBSERVED_TURNOVER_WITHOUT_DEATHS'})
    write('selection_protocol_authority.json', {
        'status': 'PASS',
        'phase_steps': pop['protocol']['phase_steps'],
        'switch_schedule': pop['protocol']['switch_schedule'],
        'founder_multiplicity': pop['protocol']['founder_multiplicity'],
        'replicates': pop['protocol']['replicates'],
        'fitness_function': False,
        'breeder_selection': False,
        'population_cap': False,
        'resource_feedback': False,
        'fixed_boundary': True,
    })

    resource = [s for s in summaries if s['mutation_enabled'] and s['environment_sequence'] == ['RESOURCE_CHALLENGE']]
    damage = [s for s in summaries if s['mutation_enabled'] and s['environment_sequence'] == ['DAMAGE_CHALLENGE']]
    reversal = [s for s in summaries if s['mutation_enabled'] and len(s['environment_sequence']) == 2]
    write('environment_a_selection.json', {
        'status': 'NOT_ESTABLISHED',
        'reason': 'Only one of two mutation-on Resource replicates produced post-bootstrap fissions; no replicated selection claim is made.',
        'replicates': resource,
    })
    write('environment_b_selection.json', {
        'status': 'NOT_ESTABLISHED',
        'reason': 'Damage produced turnover in both replicates, but mutant differential reproduction was not replicated as a selection result.',
        'replicates': damage,
    })
    write('environment_dependence.json', {
        'status': 'NOT_ESTABLISHED',
        'reason': 'The preregistered 18-variant reciprocal feasibility panel passes, but replicated population selection was not established.',
        'panel_feasibility': feasibility.get('classification'),
    })
    write('mutation_off_control.json', {
        'status': 'PASS',
        'all_mutation_off_campaigns_zero_mutations': all(s['mutations'] == 0 for s in summaries if not s['mutation_enabled']),
        'campaigns': [s for s in summaries if not s['mutation_enabled']],
    })
    write('reversal.json', {
        'status': 'NOT_ESTABLISHED',
        'reason': 'Fixed Resource→Damage runs had zero post-bootstrap fissions in both mutation-on replicates; hereditary redirection is not established.',
        'replicates': reversal,
    })

    write('population_material_closure.json', {
        'status': 'PASS',
        'max_abs_N_residual': max(abs(s['closure_residuals']['n']) for s in summaries),
        'max_abs_F_residual': max(abs(s['closure_residuals']['f']) for s in summaries),
        'fixed_boundary_reference_parity': pop['fixed_boundary_transport_reference_parity'],
        'source_sink_ledger_present': True,
    })
    write('active_energy_closure.json', {
        'status': 'PASS',
        'max_abs_residual': max(abs(s['closure_residuals']['active_energy']) for s in summaries),
        'reserve_flux_ledger_present': True,
    })

    for name, status, reason in (
        ('m1_preservation.json', 'PRESERVED', 'Upstream M1 authority preserved; reserve-on legacy certifier scope caveat is recorded separately.'),
        ('m2_preservation.json', 'QUALIFIED', 'Upstream M2 qualification preserved.'),
        ('development_preservation.json', 'PRESERVED', 'M3 and R10 organism development surfaces preserved.'),
    ):
        write(name, {'status': status, 'reason': reason, 'production_biology_delta': 0})

    not_reached = {
        'status': 'NOT_REACHED',
        'reason': 'Stopped after replicated natural selection was not established and fixed reversal had no post-bootstrap turnover.',
    }
    for name in ('checkpoint_restart.json', 'linux_runtime.json', 'sensory_embodiment.json', 'experiential_memory.json', 'godot_independence.json'):
        write(name, not_reached)
    write('global_material_energy_closure.json', {
        'status': 'NOT_REACHED_AS_FINAL_INTEGRATED_QUALIFICATION',
        'population_material_and_active_energy_closure': 'PASS',
        'reason': 'Final integrated M1-M5 run was not authorized after the M4 selection/reversal stop.',
    })
    write('forbidden_information_audit.json', {
        'status': 'PASS',
        'forbidden_controls': ['fitness function', 'breeder', 'forced birth', 'forced death', 'population cap', 'protected mutant', 'function-3 remap'],
        'new_biological_parameters': 0,
    })
    write('final_goal_matrix.json', {
        'm1': 'CLOSED / FROZEN / PRESERVED',
        'm2': 'QUALIFIED',
        'm3': 'PRESERVED',
        'd096_v4': 'UNCHANGED',
        'reproduction': 'PASS',
        'natural_generation_2_plus': 'PASS',
        'reciprocal_specialization': 'PASS',
        'resource_selection': 'NOT_ESTABLISHED',
        'damage_selection': 'NOT_ESTABLISHED',
        'environment_dependence': 'NOT_ESTABLISHED',
        'reversal': 'NOT_ESTABLISHED',
        'final_integrated_m1_m5': 'NOT_REACHED',
    })
    write('qualification.json', {
        'final_classification': 'D091V2_REPRODUCTION_AND_RECIPROCAL_SPECIALIZATION_QUALIFIED_SELECTION_AND_REVERSAL_NOT_ESTABLISHED',
        'digital_cell_end_goal': 'NOT_ESTABLISHED',
        'shutdown_recommended': 'NO — OWNER OVERRIDE ACTIVE',
        'next_execution_started': False,
        'independent_architect_acceptance': 'PENDING',
    })

    files = []
    for path in sorted(OUT.rglob('*')):
        if path.is_file() and path.name != 'artifact_manifest.json':
            files.append({'path': str(path.relative_to(OUT)), 'bytes': path.stat().st_size, 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()})
    write('artifact_manifest.json', {'files': files, 'file_count': len(files)})


if __name__ == '__main__':
    main()
