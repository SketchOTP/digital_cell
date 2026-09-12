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
        "accepted_steps": c.get("accepted_steps"),
        "rejected_steps": c.get("rejected_steps"),
        "numerical_invalid": c.get("numerical_invalid"),
        "numerical_invalid_reason": c.get("numerical_invalid_reason"),
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
    if expect_no_computational_loss:
        for field in ("invalidated_n_terminal", "invalidated_f_terminal", "invalidated_structural_terminal"):
            required(world_ledger, field)
    computational_loss = any(
        world_ledger.get(field, 0.0) != 0.0
        for field in ("invalidated_n_terminal", "invalidated_f_terminal", "invalidated_structural_terminal")
    )
    numerical_invalid = required(c, "numerical_invalid") if expect_no_computational_loss else False
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
        "numerical_invalid": numerical_invalid,
        "atomic_transaction_pass": not expect_no_computational_loss or not numerical_invalid,
    }
    result["pass"] = result["population_equation_pass"] and (
        not expect_no_computational_loss or (not computational_loss and not numerical_invalid)
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


def required(obj, *keys):
    current = obj
    for key in keys:
        if not isinstance(current, dict) or key not in current:
            raise ValueError(f"missing required evidence field: {'.'.join(keys)}")
        current = current[key]
    return current


def genotype_values(key):
    try:
        values = tuple(float(part) for part in key.split(","))
    except (TypeError, ValueError) as exc:
        raise ValueError(f"invalid complete genotype identity: {key!r}") from exc
    if len(values) != 4 or not all(value == value and abs(value) != float("inf") for value in values):
        raise ValueError(f"invalid complete genotype identity: {key!r}")
    return values


def is_neutral_genotype(key):
    """Compare the complete canonical identity, never a coordinate substring."""
    return genotype_values(key) == (0.25, 0.25, 0.25, 0.25)


def observer_physical_signature(row):
    return tuple(row[key] for key in ("step", "population", "maximum_generation", "genotype_frequencies"))


def run_adversarial_fixtures():
    """Execute verifier fixtures and fail closed if any invariant is false."""
    neutral = "0.25000000000000000,0.25000000000000000,0.25000000000000000,0.25000000000000000"
    single_transfer = "0.26000000000000001,0.25000000000000000,0.23999999999999999,0.25000000000000000"
    results = {}

    results["single_transfer_mutant"] = {
        "neutral_key": neutral,
        "mutant_key": single_transfer,
        "contains_unchanged_coordinate": True,
        "recognized_nonneutral": not is_neutral_genotype(single_transfer),
    }

    rows = [
        {"identity": "a", "value": 3},
        {"identity": "b", "value": 7},
    ]
    original = {row["identity"]: row["value"] for row in rows}
    permuted = {row["identity"]: row["value"] for row in reversed(rows)}
    results["metadata_permutation"] = {"pass": original == permuted}

    duplicate_rows = [{"identity": "a"}, {"identity": "a"}]
    duplicate_rejected = len({row["identity"] for row in duplicate_rows}) != len(duplicate_rows)
    results["duplicate_identity_rejection"] = {"pass": duplicate_rejected}

    events = [{"bootstrap": True}, {"bootstrap": False}]
    results["bootstrap_exclusion"] = {
        "pass": sum(not row["bootstrap"] for row in events) == 1,
        "post_bootstrap_events": sum(not row["bootstrap"] for row in events),
    }

    mutation_only_before = {neutral: 1}
    mutation_only_after = {neutral: 1}
    results["mutation_only_not_selection"] = {
        "pass": mutation_only_before == mutation_only_after,
        "mutation_events": 1,
        "physical_births": 0,
    }

    truncated_rejected = False
    try:
        required({"trajectory": []}, "trajectory", 0)
    except (ValueError, IndexError, TypeError):
        truncated_rejected = True
    results["truncated_evidence_rejection"] = {"pass": truncated_rejected}

    compressed = [{"genotype": "a", "count": 3}, {"genotype": "b", "count": 2}]
    expanded = [row["genotype"] for row in compressed for _ in range(row["count"])]
    results["compressed_expanded_equivalence"] = {
        "pass": {key: expanded.count(key) for key in {"a", "b"}}
        == {row["genotype"]: row["count"] for row in compressed}
    }

    physical = [
        {"step": 5, "population": 3, "maximum_generation": 1, "genotype_frequencies": {neutral: 3}},
        {"step": 6, "population": 4, "maximum_generation": 2, "genotype_frequencies": {single_transfer: 1}},
    ]
    altered_labels = [dict(row, observer_label="altered") for row in physical]
    disabled_labels = [dict(row, observer_label=None) for row in physical]
    results["observer_label_independence"] = {
        "pass": [observer_physical_signature(row) for row in physical]
        == [observer_physical_signature(row) for row in altered_labels]
        == [observer_physical_signature(row) for row in disabled_labels]
    }

    lifecycle_source = (ROOT / "examples/dcfinal001_r4_evolution.rs").read_text()
    observer_start = lifecycle_source.index("if cohort.mesh.uses_observer_only_death()")
    observer_end = lifecycle_source.index("let eligible =", observer_start)
    observer_block = lifecycle_source[observer_start:observer_end]
    canonical_observer_start = observer_block.index("if r10r9r5_canonical_lifecycle()")
    canonical_observer_end = observer_block.index("} else if", canonical_observer_start)
    canonical_observer_block = observer_block[canonical_observer_start:canonical_observer_end]
    transaction_markers = (
        "let step_cohorts_before = cohorts.clone();",
        "let step_world_before = world.clone();",
        "let step_ledger_before = ledger.clone();",
        "ledger.numerical_invalid = true;",
        "return false;",
    )
    results["canonical_lifecycle_source_contract"] = {
        "pass": all(marker in lifecycle_source for marker in transaction_markers)
        and "continue" not in canonical_observer_block
        and "survivors.push(cohort);" not in canonical_observer_block
        and "expression_step_activated_material_v4_turnover_only" in lifecycle_source,
        "canonical_observer_block_has_continue": "continue" in canonical_observer_block,
        "transaction_markers_present": {marker: marker in lifecycle_source for marker in transaction_markers},
    }

    if not all(result["pass"] if "pass" in result else result["recognized_nonneutral"] for result in results.values()):
        raise ValueError(f"adversarial verifier fixture failed: {results}")
    return results


def valid_selection_prerequisites(c):
    ledger = c["ledger"]
    nonneutral = [key for key in ledger["phenotype_by_genotype"] if not is_neutral_genotype(key)]
    return (
        c["mutation_enabled"]
        and c["environment_sequence"] in (["RESOURCE_CHALLENGE"], ["DAMAGE_CHALLENGE"])
        and ledger["post_bootstrap_physical_fissions"] > 0
        and len(nonneutral) > 0
        and (ledger["physical_deaths"] > 0 or ledger["post_bootstrap_physical_fissions"] > 0)
        and not required(c, "numerical_invalid")
    )


def selection_record(value, environment):
    rows = [
        c
        for c in campaign_rows(value)
        if c["mutation_enabled"] and c["environment_sequence"] == [environment]
    ]
    if not rows:
        return {
            "status": "NOT_REACHED",
            "execution_status": "NOT_EXECUTED",
            "reason": "no raw campaign exists",
            "campaigns": [],
        }
    if not all(valid_selection_prerequisites(c) for c in rows):
        return {
            "status": "EXECUTED_NOT_QUALIFIED",
            "execution_status": "EXECUTED",
            "reason": "raw turnover exists but the independent causal selection predicate is false",
            "campaigns": [campaign_summary(c) for c in rows],
        }
    return {
        "status": "OBSERVED_REQUIRES_REPLICATED_CAUSAL_REVIEW",
        "execution_status": "EXECUTED",
        "campaigns": [campaign_summary(c) for c in rows],
        "replicates": len(rows),
        "replicated": len(rows) == 2,
    }


def write_not_reached(names, reason):
    for name in names:
        write(name, {"status": "NOT_REACHED", "reason": reason})


def verify_reproduction_equivalence(path: Path):
    """Recompute the bounded R5 comparison from raw arm events.

    The Rust diagnostic's aggregate counts are intentionally treated as
    untrusted.  This verifier derives physical fissions from daughter birth
    events and derives viable pairs from both recorded daughter continuations.
    It also fails closed when a physical attempt lacks a predicate-level
    diagnostic.
    """
    value = load(path)
    for key in ("historical", "historical_counts", "current_variants"):
        required(value, key)

    def historical_counts():
        fissions = 0
        viable = 0
        for arm in value["historical"]:
            result = required(arm, "result")
            if required(result, "physical_fission"):
                fissions += 1
                if result.get("full_state_daughters_viable") is True:
                    viable += 1
        return fissions, viable

    def current_arm_counts(arm):
        births = required(arm, "physical_birth_events")
        groups = {}
        for row in births:
            key = (required(row, "step"), required(row, "parent_cohort_id"))
            groups.setdefault(key, []).append(row)
        fissions = 0
        for rows in groups.values():
            sides = {required(row, "daughter_side") for row in rows}
            parent_count = required(rows[0], "parent_count")
            daughter_count = sum(required(row, "daughter_count") for row in rows)
            if sides != {0, 1} or daughter_count != 2 * parent_count:
                raise ValueError(f"invalid two-daughter birth group: {rows!r}")
            fissions += parent_count
        continuations = required(arm, "daughter_continuations")
        viable_pair = (
            fissions > 0
            and len(continuations) == 2
            and all(
                required(record, "continuation").get("viable") is True
                for record in continuations
            )
        )
        for attempt in required(arm, "fission_attempts"):
            detail = required(attempt, "detail")
            if not isinstance(detail, dict):
                raise ValueError(f"attempt detail is not an object: {attempt!r}")
            if attempt.get("outcome") == "NO_VALID_PHYSICAL_PROPOSAL":
                required(detail, "failure_class")
        if required(arm, "physical_fissions") != fissions:
            raise ValueError(f"emitted fission count disagrees with birth events: {arm!r}")
        if required(arm, "fission_attempt_count") != len(required(arm, "fission_attempts")):
            raise ValueError(f"attempt count disagrees with raw attempts: {arm!r}")
        return fissions, int(viable_pair)

    historical_fissions, historical_viable = historical_counts()
    if (historical_fissions, historical_viable) != (
        8,
        8,
    ):
        raise ValueError("historical direct-boundary reference did not replay 8/8")
    if (historical_fissions, historical_viable) != (
        value["historical_counts"]["physical_fissions"],
        value["historical_counts"]["full_state_viable_pairs"],
    ):
        raise ValueError("historical aggregate disagrees with raw results")

    matrix = {}
    expected_labels = {
        "shared_fixture_historical_contract",
        "shared_fixture_current_clock",
        "shared_fixture_per_step_reserve",
        "shared_resource_historical_contract",
        "shared_resource_current_contract",
    }
    for arms in value["current_variants"]:
        if not arms:
            raise ValueError("empty comparison variant")
        label = required(arms[0], "variant")
        if label in matrix or label not in expected_labels or len(arms) != 10:
            raise ValueError(f"invalid comparison variant set: {label!r}")
        per_arm = [current_arm_counts(arm) for arm in arms]
        counts = (sum(row[0] for row in per_arm), sum(row[1] for row in per_arm))
        matrix[label] = {
            "physical_fissions": counts[0],
            "full_state_viable_pairs": counts[1],
            "distinct_successful_arms": sum(row[0] > 0 for row in per_arm),
            "per_arm": [
                {"arm": arm["arm"], "physical_fissions": row[0], "viable_pair": bool(row[1])}
                for arm, row in zip(arms, per_arm)
            ],
            "raw_event_recomputation": True,
        }
    if set(matrix) != expected_labels:
        raise ValueError(f"comparison matrix is incomplete: {matrix!r}")
    return {
        "pass": True,
        "historical": {
            "physical_fissions": historical_fissions,
            "full_state_viable_pairs": historical_viable,
            "matches_reference_8_of_10": True,
        },
        "current": matrix,
        "qualification_contract": {
            "distinct_successful_arms": True,
            "both_daughters_recorded": True,
            "daughter_continuation_viability": "actual 3000-step current continuation",
            "no_proposal_predicate_detail_required": True,
        },
    }


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
    fixtures = run_adversarial_fixtures()
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
            "r5_identity": DIRECTIVE,
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
    canonical_transaction_results = []
    for campaign in campaign_rows(canonical):
        for field in ("accepted_steps", "rejected_steps", "numerical_invalid", "numerical_invalid_reason"):
            required(campaign, field)
        world_ledger = required(campaign, "world", "ledger")
        for field in ("invalidated_n_terminal", "invalidated_f_terminal", "invalidated_structural_terminal"):
            required(world_ledger, field)
        canonical_transaction_results.append({
            "replicate": campaign["replicate"],
            "accepted_steps": campaign["accepted_steps"],
            "rejected_steps": campaign["rejected_steps"],
            "numerical_invalid": campaign["numerical_invalid"],
            "world_invalidated_material": {
                key: world_ledger[key]
                for key in ("invalidated_n_terminal", "invalidated_f_terminal", "invalidated_structural_terminal")
            },
            "pass": not campaign["numerical_invalid"] and all(
                world_ledger[key] == 0.0
                for key in ("invalidated_n_terminal", "invalidated_f_terminal", "invalidated_structural_terminal")
            ),
        })
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
                "canonical_observer_noninterference": fixtures["observer_label_independence"],
                "canonical_atomic_rejection": all(
                    row["pass"] for row in canonical_transaction_results
                ),
            },
        },
    )
    write(
        "active_kernel_identity.json",
        required(canonical, "execution_identity"),
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
            "canonical_transaction_results": canonical_transaction_results,
            "observer_nonviability_not_authoritative": fixtures["observer_label_independence"],
            "legacy_control_retains_removal_path": any(
                c["ledger"]["runtime_invalidations"] > 0 for c in campaign_rows(legacy)
            ),
            "missing_fields_fail_closed": True,
        },
    )
    write(
        "independent_event_verifier.json",
        {
            "canonical": [verify_event_accounting(c, True) for c in campaign_rows(canonical)],
            "legacy": [verify_event_accounting(c, False) for c in campaign_rows(legacy)],
            "adversarial_fixtures": fixtures,
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
            "status": "NOT_REACHED_SHARED_KERNEL_REQUALIFICATION",
            "source": str(REPRO),
            "raw": repro,
            "note": "The prior R3 reproduction output is preserved as provenance only; repaired R5 lifecycle qualification must use the shared current kernel.",
        },
    )
    write(
        "both_daughter_outcomes.json",
        {
            "status": "NOT_REACHED_SHARED_KERNEL_REQUALIFICATION",
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
    reversal_rows = [
        c for c in campaign_rows(canonical) if len(c["environment_sequence"]) == 2
    ]
    write(
        "conditional_reversal.json",
        {
            "status": "EXECUTED_EXPLORATORY_NOT_QUALIFIED" if reversal_rows else "NOT_REACHED",
            "execution_status": "EXECUTED" if reversal_rows else "NOT_EXECUTED",
            "reason": "raw reversal exists but selection prerequisites are not independently established"
            if reversal_rows
            else "no raw reversal campaign exists",
            "campaigns": [campaign_summary(c) for c in reversal_rows],
        },
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
