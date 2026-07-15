#!/usr/bin/env python3
"""Generate the D-012 preservation manifest for Stage E / D-011 artifacts."""

from __future__ import annotations

import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "digital-protocell/experiments/generated/d012/preservation/manifest.json"

REQUIRED = [
    "digital-protocell/experiments/generated/d008/stage_e_balance/attempt_003/result.json",
    "digital-protocell/experiments/generated/d011/attempt_015/result.json",
    "digital-protocell/experiments/generated/d011/attempt_017/result.json",
    "docs/d011_candidate_report.md",
    "docs/d011_balance_controllability.md",
    "docs/d011_joint_rate_solver.md",
    "docs/d011_horizon_sensitivity.md",
    "docs/d011_constrained_radius_assay.md",
    "docs/d011_stage_e_model_audit.md",
]

OPTIONAL_GLOBS = [
    "digital-protocell/experiments/generated/d011/attempt_015/**/*.json",
    "digital-protocell/experiments/generated/d011/attempt_017/**/*.json",
    "digital-protocell/experiments/generated/d008/stage_e_balance/attempt_003/**/*.json",
]

TAGS = [
    "D-008-stage-e-balance-fail",
    "D-011-transport-coupled-balance-fail",
    "D-011-transport-coupled-balance-fail-corrected",
]

COMMITS = {
    "d008_stage_e_source": "dfadb10",
    "d008_stage_e_failure": "2db93f6",
    "d011_correction": "b8e7ef8",
    "d011_learning": "5d0051f",
}


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def git(*args: str) -> str:
    return subprocess.check_output(["git", *args], cwd=ROOT, text=True).strip()


def expand_optional() -> list[Path]:
    paths: list[Path] = []
    for pattern in OPTIONAL_GLOBS:
        paths.extend(ROOT.glob(pattern))
    return sorted({p.resolve() for p in paths if p.is_file()})


def main() -> None:
    entries = []
    for rel in REQUIRED:
        path = ROOT / rel
        if not path.is_file():
            raise SystemExit(f"missing required artifact: {rel}")
        entries.append(
            {
                "path": rel,
                "required": True,
                "sha256": sha256_file(path),
                "bytes": path.stat().st_size,
            }
        )

    seen = {e["path"] for e in entries}
    for path in expand_optional():
        rel = str(path.relative_to(ROOT))
        if rel in seen:
            continue
        entries.append(
            {
                "path": rel,
                "required": False,
                "sha256": sha256_file(path),
                "bytes": path.stat().st_size,
            }
        )
        seen.add(rel)

    entries.sort(key=lambda item: item["path"])
    aggregate = hashlib.sha256()
    for item in entries:
        aggregate.update(item["path"].encode())
        aggregate.update(b"\0")
        aggregate.update(item["sha256"].encode())
        aggregate.update(b"\0")

    tag_refs = {tag: git("rev-parse", tag) for tag in TAGS}
    commit_full = {name: git("rev-parse", short) for name, short in COMMITS.items()}

    manifest = {
        "directive": "D-012",
        "purpose": "preservation_gate",
        "source_commit": git("rev-parse", "HEAD"),
        "operative_d011_status": "D011_LONG_HORIZON_CONFIRMATION_INCOMPLETE",
        "preserved_tags": tag_refs,
        "governed_commits": commit_full,
        "artifact_count": len(entries),
        "content_hash": aggregate.hexdigest(),
        "artifacts": entries,
    }

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
    print(OUT.relative_to(ROOT))
    print(f"content_hash={manifest['content_hash']}")
    print(f"artifact_count={manifest['artifact_count']}")


if __name__ == "__main__":
    main()
