#!/usr/bin/env python3
"""Bind an independently reviewed source audit and check its exact algebra.

These checks do not turn source-search/absence claims into automated proof.
The separate semantic review remains part of acceptance.
"""
import argparse
from fractions import Fraction as Q
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True
BASE = "74dfbf9a12fc9cdafcc72c3080150a7b9c04be90"
DIRECTIVE = "D-20261017-dcm4-reaction-architecture-feasibility"
PREFIX = "docs/dc-m4-reaction-architecture-"
GAP = "LOCAL_REACTION_COUPLING_PHYSICAL_OWNERSHIP_OR_ENERGETICS_GAP"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def algebra():
    # Sparse exact Laurent polynomials in k0,kd,kf,kq,a,b,L.
    def monomial(index):
        power = [0] * 7
        power[index] = 1
        return {tuple(power): Q(1)}

    def add(*polys):
        out = {}
        for p in polys:
            for key, value in p.items():
                out[key] = out.get(key, Q(0)) + value
        return {k: v for k, v in out.items() if v}

    def mul(*polys):
        out = {(0,) * 7: Q(1)}
        for p in polys:
            new = {}
            for ka, va in out.items():
                for kb, vb in p.items():
                    key = tuple(a + b for a, b in zip(ka, kb))
                    new[key] = new.get(key, Q(0)) + va * vb
            out = {k: v for k, v in new.items() if v}
        return out

    def scale(p, amount):
        return {k: v * amount for k, v in p.items() if v * amount}

    k0, kd, kf, kq, a, b, length = [monomial(i) for i in range(7)]
    inv = {(0, 0, 0, 0, 0, 0, -1): Q(1)}
    u, v = mul(a, inv), mul(b, inv)
    original = mul(length, add(mul(add(k0, mul(kf, u, u)), v),
                               scale(mul(add(kd, mul(kq, u, u)), u), -1)))
    expanded = add(mul(k0, b), scale(mul(kd, a), -1),
                   mul(kf, a, a, b, inv, inv),
                   scale(mul(kq, a, a, a, inv, inv), -1))
    require(original == expanded, "amount-rate symbolic identity failed")
    derivative = {}
    for key, value in original.items():
        if key[6]:
            new = list(key)
            new[6] -= 1
            derivative[tuple(new)] = value * key[6]
    expected = scale(mul(a, a, add(mul(kf, b), scale(mul(kq, a), -1)),
                         inv, inv, inv), -2)
    require(derivative == expected, "fixed-amount length derivative failed")
    require(derivative != scale(expected, -1), "sign-error control failed")
    # An illustrative admissible algebraic state, not an R13 snapshot/assay.
    forward = (Q(1, 100) + Q(1)) * Q(11, 101)
    backward = Q(1, 10) + Q(1, 100)
    require(forward == backward == Q(11, 100), "net/gross example failed")
    require(abs(forward - backward) * Q(1, 2) == 0 and forward + backward > 0,
            "net charge was incorrectly interpreted as gross turnover")
    return {"amount_rate_identity": "PASS", "fixed_amount_derivative": "PASS",
            "wrong_derivative_sign_control": "REJECTED",
            "net_charge_zero_with_positive_kinetic_terms": "PASS",
            "illustrative_forward_rate": str(forward),
            "illustrative_backward_rate": str(backward),
            "scope": "exact unclipped substep algebra, not whole-update efficacy"}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", type=Path, default=Path("."))
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    repo = args.repo.resolve()

    def git(*values):
        return subprocess.check_output(["git", *values], cwd=repo)

    require(not git("diff", "--name-only", BASE, "--", "digital-protocell"),
            "scientific subtree differs from accepted R13")
    allowed = (".agent/", PREFIX, "scripts/verify_reaction_architecture_feasibility.py",
               ".github/workflows/dc-m4-reaction-architecture-feasibility.yml", "STORAGE_MAP.md")
    changed = git("diff", "--name-only", BASE).decode().splitlines()
    require(all(p.startswith(allowed) for p in changed), "out-of-scope tracked change")
    for name in ("DIRECTIVES", "OUTCOMES", "LEARNINGS", "RECORD", "REPO_MAP"):
        path = f".agent/{name}.md"
        require((repo / path).read_bytes().startswith(git("show", f"{BASE}:{path}")),
                f"append-only history changed: {path}")
    audit = json.loads((repo / f"{PREFIX}audit.json").read_text())
    require(audit["entry_head"] == BASE and audit["directive"] == DIRECTIVE,
            "audit authority mismatch")
    for path, digest in audit["source_bindings"].items():
        data = (repo / path).read_bytes()
        require(data == git("show", f"{BASE}:{path}") and sha(data) == digest,
                f"source binding mismatch: {path}")
    require(len(audit["candidates"]) == 8 and len(audit["other_domains"]) == 4,
            "incomplete candidate/counterexample inventory")
    require(len({c["id"] for c in audit["candidates"]}) == 8,
            "duplicate candidate")
    keys = ("conversion_work_law", "unit_fuel_contract", "signed_rate_law")
    supported = [c["id"] for c in audit["candidates"] if all(c[k] for k in keys)]
    require(not supported and audit["classification"] == GAP,
            "classification disagrees with reviewed candidate ledger")
    require(audit["physical_state_required"] == "NOT_DETERMINED" and
            audit["feedback_implemented"] is False and audit["next_execution_started"] is False,
            "audit overstates architecture or execution")
    require(audit["independent_review"]["status"] == "ACCEPTED_BOUNDED_SOURCE_AUDIT",
            "independent semantic review missing")
    # Check only new structured records; preserve historical schema drift.
    spec = importlib.util.spec_from_file_location("governance", repo / "scripts/validate_governance.py")
    validator = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(validator)

    def section(file, title):
        for name, body in validator.iter_sections((repo / file).read_text()):
            if name == title:
                return f"## {name}\n{body}"
        raise ValueError(f"missing record: {title}")

    errors = []
    ids = validator.validate_directives(section(".agent/DIRECTIVES.md", DIRECTIVE), errors)
    validator.validate_outcomes(section(".agent/OUTCOMES.md", f"{DIRECTIVE} - COMPLETE"), ids, errors)
    validator.validate_learnings(section(".agent/LEARNINGS.md", "L-DCM4-REACTION-ARCHITECTURE-GAP-20261017"), errors)
    require(not errors, str(errors))
    checks = algebra()
    output = args.output.resolve()
    require(not output.is_relative_to(repo), "write evidence outside preserved checkout")
    output.mkdir(parents=True, exist_ok=True)
    evidence = {"exact_head": git("rev-parse", "HEAD").decode().strip(), "entry_head": BASE,
                "directive": DIRECTIVE, "classification": GAP, "algebra": checks,
                "source_bindings": audit["source_bindings"],
                "candidate_count": 8, "other_domain_count": 4,
                "production_subtree_byte_identical": True, "append_only_ledgers": "PASS",
                "new_structured_records": "PASS", "semantic_review_required": True,
                "feedback_implemented": False, "next_execution_started": False,
                "new_scientific_binary": "NOT_APPLICABLE_SOURCE_AUDIT",
                "historical_global_governance_schema": "PREEXISTING_DRIFT_NOT_CLAIMED_PASS"}
    (output / "verification.json").write_text(json.dumps(evidence, indent=2) + "\n")
    paths = [*repo.glob(f"{PREFIX}*"), repo / "scripts/verify_reaction_architecture_feasibility.py",
             repo / ".github/workflows/dc-m4-reaction-architecture-feasibility.yml"]
    paths += [repo / f".agent/{n}.md" for n in ("CURRENT", "PROJECT_PROFILE", "DIRECTIVES", "OUTCOMES", "LEARNINGS", "RECORD", "REPO_MAP")]
    paths += [repo / "STORAGE_MAP.md"]
    paths += [repo / path for path in audit["source_bindings"]]
    bound = {str(p.relative_to(repo)): sha(p.read_bytes()) for p in paths}
    for path in paths:
        target = output / "repository" / path.relative_to(repo)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(path.read_bytes())
    manifest = {"exact_head": evidence["exact_head"], "entry_head": BASE,
                "verification_sha256": sha((output / "verification.json").read_bytes()),
                "bound_repository_files": bound, "file_count": len(bound)}
    (output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(json.dumps({"head": evidence["exact_head"], "classification": GAP,
                      "source_bindings": len(audit["source_bindings"]),
                      "bound_repository_files": len(bound), "checks": "PASS"}))


if __name__ == "__main__":
    main()
