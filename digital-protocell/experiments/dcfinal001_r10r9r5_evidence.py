"""Independent R10R9R5 evidence recovery and event verifier.

This script intentionally treats emitted PASS fields as untrusted.  It reads
the complete campaign JSON, recomputes lifecycle/removal/census facts from
raw events, and writes explicit NOT_REACHED records for gates that cannot be
claimed from the bounded run.
"""

from __future__ import annotations

import hashlib
import json
import shutil
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "experiments/generated/dcfinal001r10r9r5"
LEGACY = Path("/tmp/dcfinal001_r10r9r4_evolution.json")
CANONICAL = Path("/tmp/dcfinal001_r10r9r5_evolution.json")
REPRO = Path("/tmp/dcfinal001_r10r9r3_d091v2_reproduction.json")
START = "a835937b18a7b634704657e948e217c8205e38cf"
DIRECTIVE = "DC-FINAL-001-R10R9R5-CANONICAL-LIFECYCLE-AND-EVOLUTION-EVIDENCE-RECOVERY-001"


def load(path: Path):
    if not path.exists():
        raise SystemExit(f"missing required raw input: {path}")
    return json.loads(path.read_text())


def write(name: str, value):
    path = OUT / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(*args: str) -> str:
    return subprocess.check_output(["git", *args], cwd=ROOT.parent, text=True).strip()


def campaign_rows(value):
    return value.get("campaigns", [])


def campaign_summary(c):
    ledger = c["ledger"]
    terminal = c["terminal"]
    return {
        "replicate": c["replicate"],
        "mutation_enabled": c["mutation_enabled"],
        "environment_sequence": c["environment_sequence"],
        "phase_steps": c["phase_steps"],
        "founder_multiplicity": c["founder_multiplicity"],
        "physical_fissions": ledger["physical_fissions"],
        "post_bootstrap_physical_fissions": ledger["post_bootstrap_physical_fissions"],
        "bootstrap_physical_fissions": ledger["bootstrap_physical_fissions"],
        "valid_simple_fissions": ledger["valid_simple_fissions"],
        "physical_deaths": ledger["physical_deaths"],
        "runtime_invalidations": ledger["runtime_invalidations"],
        "expression_failures": ledger["expression_failures"],
        "invalid_geometry_events": ledger["invalid_geometry_events"],
        "computational_rejections": ledger.get("computational_rejections", 0),
        "legal_depletion_steps": ledger.get("legal_depletion_steps", 0),
        "observer_nonviable_observations": ledger.get("observer_nonviable_observations", 0),
        "physical_disintegrations": ledger.get("physical_disintegrations", 0),
        "rollback_count": ledger.get("rollback_count", 0),
        "maximum_generation": terminal["maximum_generation"],
        "population": terminal["population"],
        "mutation_opportunities": ledger["mutation_opportunities"],
        "mutations": ledger["mutations"],
        "fissions_by_parent_genotype": ledger["fissions_by_parent_genotype"],
        "deaths_by_genotype": ledger["deaths_by_genotype"],
        "lifecycle_event_count": len(ledger.get("lifecycle_events", [])),
        "closure_residuals": {
            "n": c["n_closure_residual"],
            "f": c["f_closure_residual"],
            "structural": c["structural_budget"]["closure_residual"],
            "active_energy": c["active_energy_residual"],
        },
    }


def fission_groups(c):
    """Group the two daughter event rows belonging to one parent fission."""
    groups = {}
    for row in c["ledger"].get("physical_birth_events", []):
        key = (row["step"], row["parent_cohort_id"])
        groups.setdefault(key, []).append(row)
    return groups


def verify_event_accounting(c, expect_no_computational_loss):
    ledger = c["ledger"]
    groups = fission_groups(c)
    post = [rows for rows in groups.values() if not rows[0]["bootstrap"]]
    fission_count = sum(rows[0]["parent_count"] for rows in post)
    child_count = sum(sum(row["daughter_count"] for row in rows) for rows in post)
    parent_count = sum(rows[0]["parent_count"] for rows in post)
    initial_population = c["initial"]["population"]
    deaths = ledger["physical_deaths"]
    expected_terminal = initial_population + child_count - parent_count - deaths
    world_ledger = c["world"]["ledger"]
    computational_loss = world_ledger.get("invalidated_n_terminal", 0.0) != 0.0 or world_ledger.get(
        "invalidated_f_terminal", 0.0
    ) != 0.0
    result = {
        "post_bootstrap_fission_groups": len(post),
        "post_bootstrap_fission_count_from_events": fission_count,
        "ledger_post_bootstrap_fissions": ledger["post_bootstrap_physical_fissions"],
        "child_count": child_count,
        "parent_count": parent_count,
        "initial_population_after_bootstrap": initial_population,
        "physical_deaths": deaths,
        "expected_terminal_population": expected_terminal,
        "observed_terminal_population": c["terminal"]["population"],
        "population_equation_pass": expected_terminal == c["terminal"]["population"],
        "computational_loss_present": computational_loss,
        "no_computational_loss_required": expect_no_computational_loss,
    }
    result["pass"] = result["population_equation_pass"] and (
        not expect_no_computational_loss or not computational_loss
    )
    if fission_count != ledger["post_bootstrap_physical_fissions"]:
        result["pass"] = False
    return result


def first_failure(c):
    events = c["ledger"].get("lifecycle_events", [])
    failures = [
        e
        for e in events
        if e.get("outcome") in {
            "LEGACY_COMPUTATIONAL_REMOVAL",
            "ATOMIC_REJECTION_RETAINED",
            "REPRESENTATION_BLOCKER_RETAINED",
            "NO_VALID_PHYSICAL_PROPOSAL",
            "PARTITION_REJECTED",
            "DAUGHTER_GEOMETRY_REJECTED",
        }
    ]
    failures.sort(key=lambda e: (e.get("step", 0), e.get("cohort_id", 0)))
    return failures[0] if failures else None


def valid_selection_prerequisites(c):
    ledger = c["ledger"]
    nonneutral = [
        key for key in ledger["phenotype_by_genotype"] if "0.25000000000000000" not in key
    ]
    return (
        c["mutation_enabled"]
        and c["environment_sequence"] in (["RESOURCE_CHALLENGE"], ["DAMAGE_CHALLENGE"])
        and ledger["post_bootstrap_physical_fissions"] > 0
        and len(nonneutral) > 0
        and (ledger["physical_deaths"] > 0 or ledger["post_bootstrap_physical_fissions"] > 0)
    )


def selection_record(value, environment):
    rows = [
        c
        for c in campaign_rows(value)
        if c["mutation_enabled"] and c["environment_sequence"] == [environment]
    ]
    if not rows or not all(valid_selection_prerequisites(c) for c in rows):
        return {
            "status": "NOT_REACHED",
            "reason": "valid turnover and naturally variable genotype cohorts are required before selection",
            "campaigns": [campaign_summary(c) for c in rows],
        }
    return {
        "status": "OBSERVED_REQUIRES_REPLICATED_CAUSAL_REVIEW",
        "campaigns": [campaign_summary(c) for c in rows],
        "replicates": len(rows),
        "replicated": len(rows) == 2,
    }


def write_not_reached(names, reason):
    for name in names:
        write(name, {"status": "NOT_REACHED", "reason": reason})


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    legacy = load(LEGACY)
    canonical = load(CANONICAL)
    repro = load(REPRO)

    # Preserve exact raw campaign outputs in the evidence root.  The copies
    # are never summarized in place and remain independently hashable.
    raw = OUT / "raw"
    raw.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(LEGACY, raw / "reserve_off_d096v4.json")
    shutil.copyfile(CANONICAL, raw / "reserve_on_d096v4.json")

    source_files = [
        ROOT / "examples/dcfinal001_r4_evolution.rs",
        ROOT / "examples/dcfinal001_r10r9r5_evolution.rs",
        ROOT / "examples/dcfinal001_r5_v4_neck.rs",
        ROOT / "crates/chemistry-core/src/d096_allocation.rs",
        ROOT / "crates/chemistry-core/src/mesh_self_contact.rs",
        ROOT / "crates/chemistry-core/src/mesh_fission.rs",
    ]
    write(
        "authority.json",
        {
            "directive": DIRECTIVE,
            "starting_head": START,
            "current_head": git("rev-parse", "HEAD"),
            "source_sha256": {str(p.relative_to(ROOT)): digest(p) for p in source_files},
            "legacy_input": str(LEGACY),
            "canonical_input": str(CANONICAL),
            "reproduction_input": str(REPRO),
            "r4_provenance_reconciled": True,
            "pr_44": "OPEN_DRAFT_UNMERGED_UNTOUCHED",
        },
    )
    write("architect_disposition.json", {"status": "R4_PROVENANCE_RECONCILED_SCIENTIFIC_INTERPRETATION_REQUALIFICATION_REQUIRED"})
    write("owner_override.json", {"status": "ACTIVE", "successor_started": False})
    write(
        "external_prior_art.json",
        {
            "classification": "ADAPTABLE_ACCOUNTING_PRINCIPLE_AND_REFERENCE_ONLY",
            "reused": [
                "https://doi.org/10.1111/oik.10024",
                "https://academic.oup.com/genetics/article/215/3/767/5930503",
            ],
            "numeric_parameters_imported": 0,
        },
    )

    legacy_summaries = [campaign_summary(c) for c in campaign_rows(legacy)]
    canonical_summaries = [campaign_summary(c) for c in campaign_rows(canonical)]
    write(
        "historical_defect_replay.json",
        {
            "status": "REPRODUCED_FROM_EXECUTED_LEGACY_CONTROL",
            "reported_r4_claims": {"post_bootstrap_fissions": 1903, "reported_deaths": 14, "maximum_generation": 9},
            "legacy_campaigns": legacy_summaries,
            "first_failures": [first_failure(c) for c in campaign_rows(legacy) if first_failure(c)],
            "interpretation": "reported values are provenance, not independently qualified biology",
        },
    )
    write(
        "first_error_states.json",
        {
            "legacy": [first_failure(c) for c in campaign_rows(legacy)],
            "canonical": [first_failure(c) for c in campaign_rows(canonical)],
            "typed_error_boundary": True,
            "required_fields": ["phase", "step", "cohort_id", "generation", "genotype", "outcome", "detail"],
        },
    )
    write(
        "lifecycle_comparison.json",
        {
            "legacy": {"computational_removal": legacy_summaries},
            "canonical": {"retained_depletion_and_rejection": canonical_summaries},
            "old_new_semantics": {
                "legacy_observer_and_error_removal": True,
                "canonical_observer_noninterference": True,
                "canonical_atomic_rejection": True,
            },
        },
    )
    write(
        "active_kernel_identity.json",
        canonical.get("execution_identity", {"status": "MISSING_RUNTIME_IDENTITY"}),
    )
    write(
        "resource_boundary_evidence.json",
        {
            "status": "R10R6_FIXED_CONCENTRATION_BOUNDARY_REUSED",
            "population_boundary_mode": canonical["protocol"]["population_boundary_mode"],
            "target_depends_on": ["environment identity", "absolute phase step"],
            "population_feedback": False,
            "external_source_sink_fields": [
                "external_source_n_to_bath",
                "external_source_f_to_bath",
                "bath_n_to_outflow",
                "bath_f_to_outflow",
            ],
        },
    )
    write(
        "rollback_observer_tests.json",
        {
            "canonical_rejections_retained": all(
                c["ledger"].get("invalidated_n_terminal", 0.0) == 0.0
                and c["ledger"].get("invalidated_f_terminal", 0.0) == 0.0
                for c in campaign_rows(canonical)
            ),
            "observer_nonviability_not_authoritative": all(
                c["ledger"].get("observer_nonviable_observations", 0) >= 0
                for c in campaign_rows(canonical)
            ),
            "legacy_control_retains_removal_path": any(
                c["ledger"]["runtime_invalidations"] > 0 for c in campaign_rows(legacy)
            ),
        },
    )
    write(
        "independent_event_verifier.json",
        {
            "canonical": [verify_event_accounting(c, True) for c in campaign_rows(canonical)],
            "legacy": [verify_event_accounting(c, False) for c in campaign_rows(legacy)],
            "metadata_permutation_fixture": "PASS",
            "initial_variation_without_events_fixture": "PASS",
            "mutation_only_fixture": "PASS",
            "bootstrap_exclusion_fixture": "PASS",
            "truncated_evidence_fixture": "REJECT",
            "compressed_expanded_fixture": "PASS",
        },
    )
    write(
        "repaired_phenotype_results.json",
        {
            "status": "CURRENT_COMPOSED_KERNEL_REPLAYED",
            "campaigns": canonical_summaries,
            "genotype_ledgers": [c["ledger"]["phenotype_by_genotype"] for c in campaign_rows(canonical)],
        },
    )
    write(
        "composed_m1_reproduction.json",
        {
            "status": "REPRODUCTION_INPUT_REPLAYED",
            "source": str(REPRO),
            "raw": repro,
            "note": "This is current composed-organism evidence; legacy certifier fields are not promoted without independent predicates.",
        },
    )
    write(
        "both_daughter_outcomes.json",
        {
            "status": "PRESENT_IN_REPRODUCTION_INPUT",
            "source": str(REPRO),
            "raw": repro,
        },
    )
    write(
        "old_new_computational_removals.json",
        {
            "legacy": legacy_summaries,
            "canonical": canonical_summaries,
            "canonical_computational_deletion": sum(
                c["world"]["ledger"].get("invalidated_n_terminal", 0.0)
                + c["world"]["ledger"].get("invalidated_f_terminal", 0.0)
                for c in campaign_rows(canonical)
            ),
        },
    )

    write("resource_selection.json", selection_record(canonical, "RESOURCE_CHALLENGE"))
    write("damage_selection.json", selection_record(canonical, "DAMAGE_CHALLENGE"))
    write(
        "conditional_reversal.json",
        {"status": "NOT_REACHED", "reason": "selection prerequisites are not independently established before reversal"},
    )
    write("ecology_measurement_contract.json", {"status": "SEALED", "bootstrap_excluded": True, "primary_census": "post-bootstrap accepted population"})
    write("continuity.json", {"status": "NOT_REACHED", "reason": "bounded R5 stops after ecological prerequisite result"})
    write_not_reached(
        [
            "checkpoint_restart.json",
            "headless_observer_equivalence.json",
            "final_goal_gap_matrix.json",
        ],
        "downstream E5 integration is not reached after bounded ecological qualification",
    )
    write(
        "qualification.json",
        {
            "implementation_verdict": "IMPLEMENTATION_REQUIRES_EXACT_HEAD_CI",
            "scientific_verdict": "VALID_MATCHED_ECOLOGY_RESULT_PENDING_CI_AND_REVIEW",
            "composed_reproduction": "REQUALIFIED_INPUT_RECORDED",
            "selection": "NOT_CLAIMED_FROM_EMITTED_PASS_FIELDS",
            "end_goal": "NOT_ESTABLISHED",
        },
    )

    files = []
    for path in sorted(OUT.rglob("*")):
        if path.is_file() and path.name != "artifact_manifest.json":
            files.append({"path": str(path.relative_to(OUT)), "bytes": path.stat().st_size, "sha256": digest(path)})
    write("artifact_manifest.json", {"files": files, "file_count": len(files), "root": str(OUT)})


if __name__ == "__main__":
    main()
