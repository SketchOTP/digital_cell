#!/usr/bin/env python3
"""Offline metadata eligibility only; this evaluator never admits a source."""
from __future__ import annotations

from typing import Any

PARAMETERS = ("K", "lambda", "theta", "epsilon_A")
KINDS = frozenset({"EXTERNAL_COMPUTATIONAL", "EXTERNAL_EXPERIMENTAL",
                   "NATIVE_CONSTITUTIVE", "INTERNAL_GENERATED", "OTHER_DOMAIN", "PROTOTYPE"})
EXTERNAL_KINDS = frozenset({"EXTERNAL_COMPUTATIONAL", "EXTERNAL_EXPERIMENTAL"})
INDEPENDENCE = frozenset({"VERIFIED", "MISSING", "REJECTED"})
GATE_STATUSES = frozenset({"VERIFIED", "MISSING", "INCOMPATIBLE"})
GATES = ("variable_mapping", "unit_species_mapping", "observations", "design_coverage",
         "uncertainty_budget", "pre_response_seal", "independent_review")
CANDIDATE_FIELDS = frozenset({"id", "kind", "targets", "evidence_ids", "independence", "target_gates"})
GATE_FIELDS = frozenset({"status", "evidence_ids", "rationale"})


def _defect(message: str) -> None:
    raise ValueError(f"EVIDENCE_DEFECT: {message}")


def _text(value: Any, name: str) -> str:
    if type(value) is not str or not value.strip() or value != value.strip():
        _defect(f"{name} must be a nonempty trimmed string")
    return value


def _object(value: Any, expected: frozenset[str], name: str) -> dict:
    if type(value) is not dict or set(value) != expected:
        _defect(f"{name} requires exactly {sorted(expected)}")
    return value


def _strings(value: Any, name: str) -> list[str]:
    # An empty registry means no evidence, not fabricated evidence. VERIFIED
    # gates below require actual nonempty references into this registry.
    if type(value) is not list:
        _defect(f"{name} must be a list")
    result = [_text(item, name) for item in value]
    if len(set(result)) != len(result):
        _defect(f"{name} contains duplicates")
    return result


def _enum(value: Any, allowed: frozenset[str], name: str) -> str:
    if type(value) is not str or value not in allowed:
        _defect(f"{name} must be one of {sorted(allowed)}")
    return value


def evaluate(candidate: Any) -> dict[str, Any]:
    """Return per-parameter proposed eligibility, always admitted=False.

    Proof IDs and VERIFIED metadata are declarations, not proof authenticity or
    semantic validation. A separate hash-bound semantic review remains required.
    Missing information must be represented by explicit MISSING gate objects;
    omitted/extra fields, invalid types and dangling proof IDs are defects.
    The input object is never mutated and no files/network/resources are read.
    """
    data = _object(candidate, CANDIDATE_FIELDS, "candidate")
    candidate_id = _text(data["id"], "id")
    kind = _enum(data["kind"], KINDS, "kind")
    independence = _enum(data["independence"], INDEPENDENCE, "independence")
    targets = _strings(data["targets"], "targets")
    if any(target not in PARAMETERS for target in targets):
        _defect("targets contains an unknown parameter")
    evidence_ids = set(_strings(data["evidence_ids"], "evidence_ids"))
    target_gates = _object(data["target_gates"], frozenset(targets), "target_gates")
    findings: dict[str, dict[str, list[str]]] = {}
    # Validate the entire candidate before issuing any decisions, including
    # metadata that cannot overcome a nonindependent or wrong-domain source.
    for parameter in targets:
        gates = _object(target_gates[parameter], frozenset(GATES), f"{parameter} gates")
        missing, incompatible = [], []
        for name in GATES:
            gate = _object(gates[name], GATE_FIELDS, f"{parameter}.{name}")
            status = _enum(gate["status"], GATE_STATUSES, f"{parameter}.{name}.status")
            refs = _strings(gate["evidence_ids"], f"{parameter}.{name}.evidence_ids")
            if not set(refs) <= evidence_ids:
                _defect(f"{parameter}.{name} references undeclared evidence")
            rationale = gate["rationale"]
            if type(rationale) is not str:
                _defect(f"{parameter}.{name}.rationale must be a string")
            if status == "VERIFIED":
                # This only rejects empty/placeholder prose; it cannot certify
                # the truth of a sufficiently long declaration.
                if not refs or len(rationale.strip()) < 16 or len(rationale.split()) < 3:
                    _defect(f"{parameter}.{name} VERIFIED requires proof refs and meaningful rationale")
            elif status == "MISSING":
                missing.append(name)
            else:
                incompatible.append(name)
        findings[parameter] = {"missing_gates": missing, "incompatible_gates": incompatible}

    decisions = {}
    for parameter in PARAMETERS:
        if parameter not in targets:
            decision = "NOT_TARGETED"
        elif independence == "REJECTED" or kind in {"INTERNAL_GENERATED", "PROTOTYPE"}:
            decision = "REJECTED_NONINDEPENDENT"
        elif kind not in EXTERNAL_KINDS:
            # ks/c0 or another domain's chemical potentials are not independent
            # identifying observations for these four new coupling parameters.
            decision = "NOT_ADMITTED_INCOMPATIBLE"
        elif findings[parameter]["incompatible_gates"]:
            decision = "NOT_ADMITTED_INCOMPATIBLE"
        elif independence == "MISSING" or findings[parameter]["missing_gates"]:
            decision = "NOT_ADMITTED_MISSING_EVIDENCE"
        else:
            decision = "ELIGIBLE_FOR_REVIEW"
        decisions[parameter] = decision
    return {
        "candidate_id": candidate_id,
        "kind": kind,
        "per_parameter": decisions,
        "gate_findings": findings,
        "reference_disposition": "REFERENCE_ONLY" if kind in {"NATIVE_CONSTITUTIVE", "OTHER_DOMAIN"} else "NOT_ASSESSED",
        "admitted": False,
        "calibrated": False,
        "semantic_review_required": True,
        "eligibility_is_proof_authentication": False,
        "final_admission_requirement": "separate_hash_bound_semantic_review_of_source_dependencies_raw_evidence_mappings_and_uncertainty",
    }
