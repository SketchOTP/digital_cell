#!/usr/bin/env python3
"""Temporary-fixture tests for both governance validator modes."""

from __future__ import annotations

import re
import shutil
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
VALIDATOR = ROOT / "scripts/validate_governance.py"


def copy_repo(destination: Path) -> None:
    shutil.copytree(ROOT, destination, ignore=shutil.ignore_patterns("__pycache__", "*.pyc"))


def run(root: Path, mode: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["python3", str(VALIDATOR), "--mode", mode, "--root", str(root)],
        text=True,
        capture_output=True,
        check=False,
    )


def append(path: Path, text: str) -> None:
    path.write_text(path.read_text(encoding="utf-8") + text, encoding="utf-8")


def add_outcome(root: Path, outcome_id: str, closed: str, supersedes: str = "none") -> None:
    append(
        root / ".agent/OUTCOMES.md",
        f"""\n## D-FIXTURE-001 - COMPLETE\n\n- Outcome ID: {outcome_id}\n- Supersedes outcome: {supersedes}\n- Closed: {closed}\n- Acceptance: MET\n- Summary: Fixture passed.\n- Changed areas: none\n- Validation:\n  - validator - PASSED\n- Remaining risks: none\n- Blockers: none\n- Follow-up directive: none\n""",
    )


def add_learning(root: Path, learning_id: str, supersedes: str = "none") -> None:
    append(root / ".agent/LEARNINGS.md", f"\n## L-FIXTURE-001\n\n- Learning ID: {learning_id}\n- Date: 2026-08-04\n- Fact or lesson: Fixture learning.\n- Evidence location: fixture evidence.\n- Confidence: VERIFIED\n- Scope: fixture.\n- Supersedes learning: {supersedes}\n")


def make_fresh_adopted(root: Path) -> None:
    make_adopted(root)
    current = (ROOT / ".agent/CURRENT.md").read_text(encoding="utf-8")
    current = current.replace("Status: `TEMPLATE`", "Status: `ADOPTED`", 1).replace("Last updated: `UNSET UNTIL ADOPTION`", "Last updated: `2026-08-04T09:00:00-04:00`", 1).replace("Command or check: `NONE`", "Command or check: no validation run", 1).replace("Result: `NONE`", "Result: `NOT RUN`", 1)
    (root / ".agent/CURRENT.md").write_text(current, encoding="utf-8")
    for relative in ("DIRECTIVES.md", "OUTCOMES.md", "LEARNINGS.md", "RECORD.md"):
        (root / ".agent" / relative).write_text((ROOT / ".agent" / relative).read_text(encoding="utf-8"), encoding="utf-8")


def expect_failure(name: str, mode: str, mutate) -> None:
    with tempfile.TemporaryDirectory(prefix="governance-test-") as directory:
        candidate = Path(directory) / "fixture"
        copy_repo(candidate)
        mutate(candidate)
        result = run(candidate, mode)
        if result.returncode == 0:
            raise AssertionError(f"{name}: validator unexpectedly passed\n{result.stdout}\n{result.stderr}")


def make_adopted(root: Path) -> None:
    (root / ".agent/PROJECT_GOAL.md").write_text(
        """# Project Goal\n\n## Lifecycle\n- Status: `ADOPTED`\n- Last verified: `2026-08-04`\n\n## Goal\n- Provide the adopted project purpose.\n\n## Observable success measures\n- Produce the intended result.\n\n## Scope\n- Project scope.\n\n## Non-goals\n- Out-of-scope work.\n\n## Constraints\n- Verified constraint.\n\n## Governing external specifications\n- None.\n\n## Owner or approval authority\n- User.\n""",
        encoding="utf-8",
    )
    (root / ".agent/PROJECT_PROFILE.md").write_text(
        """# Project Profile\n\n## Lifecycle\n- Status: `ADOPTED`\n- Last verified: `2026-08-04`\n\n## Identity\n- Project name or identifier: `fixture-project`\n- Purpose: Test fixture.\n- Repository root: `/tmp/fixture-project`\n- Verified remote: `NONE VERIFIED`\n- Maturity or current phase: `active`\n\n## Languages and runtimes\n- Python.\n\n## Tools\n- Build: `NOT APPLICABLE`\n- Test: Python.\n- Lint: `NOT CONFIGURED`\n- Type-check: `NOT CONFIGURED`\n- Packaging: `NOT CONFIGURED`\n- Preferred navigation/indexing: `rg`.\n\n## Verified commands\n- `python3 scripts/validate_governance.py --mode ADOPTED`\n\n## Constraints\n- Platform/compatibility: portable.\n- Security: no secrets.\n- Data handling: local fixture only.\n- Deployment: none.\n""",
        encoding="utf-8",
    )
    (root / ".agent/PROJECT_GOAL.md").write_text((root / ".agent/PROJECT_GOAL.md").read_text(encoding="utf-8").replace("Last verified: `2026-08-04`", "Last verified: `2026-08-04T09:00:00-04:00`"), encoding="utf-8")
    (root / ".agent/PROJECT_PROFILE.md").write_text((root / ".agent/PROJECT_PROFILE.md").read_text(encoding="utf-8").replace("Last verified: `2026-08-04`", "Last verified: `2026-08-04T09:00:00-04:00`"), encoding="utf-8")
    (root / ".agent/PROJECT_PROFILE.md").write_text((root / ".agent/PROJECT_PROFILE.md").read_text(encoding="utf-8").replace("- `python3 scripts/validate_governance.py --mode ADOPTED`\n", "- `python3 scripts/validate_governance.py --mode ADOPTED`\n- Project-specific commands: `NOT APPLICABLE`\n"), encoding="utf-8")
    (root / ".agent/CURRENT.md").write_text((root / ".agent/CURRENT.md").read_text(encoding="utf-8").replace("## Active state", "## Active state after adoption").replace("## Last validation", "## Temporary task-relevant facts\n- None.\n\n## Last validation after adoption", 1), encoding="utf-8")
    (root / ".agent/CURRENT.md").write_text(
        """# Current State\n\n## Lifecycle\n- Status: `ADOPTED`\n- Last updated: `2026-08-04T10:00:00-04:00`\n\n## Active state\n- Local directive ID: `D-FIXTURE-001`\n- External directive ID: `NONE`\n- Objective: `Fixture task`\n- Current status: `IN_PROGRESS`\n- Acceptance: `Fixture passes`\n- Current phase: `testing`\n- Expected or actual touched areas: `fixture`\n- Immediate next action: `none`\n\n## Last validation\n- Command or check: `python3 scripts/validate_governance.py --mode ADOPTED`\n- Result: `PASSED`\n\n## Risks\n- None.\n\n## Blockers\n- None.\n\n## Pending decisions\n- None.\n\n## Status vocabulary\n- Adopted statuses.\n""",
        encoding="utf-8",
    )
    (root / ".agent/CURRENT.md").write_text((root / ".agent/CURRENT.md").read_text(encoding="utf-8").replace("## Active state", "## Active state after adoption").replace("## Last validation", "## Temporary task-relevant facts\n- None.\n\n## Last validation after adoption", 1), encoding="utf-8")
    append(
        root / ".agent/DIRECTIVES.md",
        """\n## D-FIXTURE-001\n\n- Issued: 2026-08-04T10:00:00-04:00\n- Issuer: User\n- External directive: none\n- Objective: Validate the fixture.\n- Scope: Temporary fixture.\n- Exclusions: Deployment.\n- Acceptance: Validator passes.\n- Risk class: LOW\n- Relationship: new\n- Related directive: none\n- Status at issuance: ISSUED\n""",
    )
    append(
        root / ".agent/OUTCOMES.md",
        """\n## D-FIXTURE-001 - COMPLETE\n\n- Outcome ID: OUT-FIXTURE-001\n- Supersedes outcome: none\n- Closed: 2026-08-04T10:01:00-04:00\n- Acceptance: MET\n- Summary: Fixture passed.\n- Changed areas: none\n- Validation:\n  - validator - PASSED\n- Remaining risks: none\n- Blockers: none\n- Follow-up directive: none\n""",
    )
    append(
        root / ".agent/RECORD.md",
        """\n## DEC-FIXTURE-001\n\n- Date: 2026-08-04\n- Record or decision ID: DEC-FIXTURE-001\n- Status: ACTIVE\n- Decision or event: Fixture adopted.\n- Rationale: Test adopted mode.\n- Affected areas: temporary fixture.\n""",
    )
    append(root / ".agent/RECORD.md", "- Supersedes record: none\n")
    (root / ".agent/REPO_MAP.md").write_text(
        """# Repository Map\n\n- `src/` — fixture source area.\n- `tests/` — fixture validation area.\n""",
        encoding="utf-8",
    )


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="governance-positive-") as directory:
        template = Path(directory) / "template"
        copy_repo(template)
        if run(template, "TEMPLATE").returncode != 0:
            raise AssertionError("clean TEMPLATE fixture failed")
        adopted = Path(directory) / "adopted"
        copy_repo(adopted)
        make_adopted(adopted)
        result = run(adopted, "ADOPTED")
        if result.returncode != 0:
            raise AssertionError(f"positive ADOPTED fixture failed\n{result.stdout}\n{result.stderr}")

    template_cases = [
        ("missing mandatory .agent contract file", lambda root: (root / ".agent/LEARNINGS.md").unlink()),
        ("missing Codex governance skill", lambda root: shutil.rmtree(root / ".agents")),
        ("active directive ID", lambda root: (root / ".agent/CURRENT.md").write_text((root / ".agent/CURRENT.md").read_text(encoding="utf-8").replace("Local directive ID: `NONE`", "Local directive ID: `D-LIVE-001`", 1), encoding="utf-8")),
        ("active template current status", lambda root: (root / ".agent/CURRENT.md").write_text((root / ".agent/CURRENT.md").read_text(encoding="utf-8").replace("Current status: `IDLE`", "Current status: `IN_PROGRESS`", 1), encoding="utf-8")),
        ("live template objective", lambda root: (root / ".agent/CURRENT.md").write_text((root / ".agent/CURRENT.md").read_text(encoding="utf-8").replace("Objective: `NONE`", "Objective: `Live task`", 1), encoding="utf-8")),
        ("live directive", lambda root: append(root / ".agent/DIRECTIVES.md", "\n## D-LIVE-001\n")),
        ("live outcome", lambda root: append(root / ".agent/OUTCOMES.md", "\n## D-LIVE-001 - COMPLETE\n")),
        ("project decision", lambda root: append(root / ".agent/RECORD.md", "\n## DEC-LIVE-001\n")),
        ("arbitrary decision heading", lambda root: append(root / ".agent/RECORD.md", "\n## Architecture decision\n")),
        ("arbitrary learning heading", lambda root: append(root / ".agent/LEARNINGS.md", "\n## Durable fact\n")),
        ("active blocker", lambda root: (root / ".agent/CURRENT.md").write_text((root / ".agent/CURRENT.md").read_text(encoding="utf-8").replace("## Blockers\n\n- None.", "## Blockers\n\n- Blocked by an active issue.", 1), encoding="utf-8")),
        ("active validation", lambda root: (root / ".agent/CURRENT.md").write_text((root / ".agent/CURRENT.md").read_text(encoding="utf-8").replace("Result: `NONE`", "Result: `PASSED`", 1), encoding="utf-8")),
        ("template identity", lambda root: (root / ".agent/PROJECT_PROFILE.md").write_text((root / ".agent/PROJECT_PROFILE.md").read_text(encoding="utf-8").replace("<adopted project name>", "authority", 1), encoding="utf-8")),
        ("Linux repository path", lambda root: (root / ".agent/PROJECT_PROFILE.md").write_text((root / ".agent/PROJECT_PROFILE.md").read_text(encoding="utf-8").replace("<repository root>", "/home/example/project", 1), encoding="utf-8")),
        ("Windows repository path", lambda root: (root / ".agent/PROJECT_PROFILE.md").write_text((root / ".agent/PROJECT_PROFILE.md").read_text(encoding="utf-8").replace("<repository root>", "C:\\Users\\example\\project", 1), encoding="utf-8")),
        ("template history", lambda root: append(root / ".agent/RECORD.md", "\n## GOV-20260804-001\n")),
        ("migration report", lambda root: (root / "MIGRATION_REPORT.md").write_text("development evidence", encoding="utf-8")),
        ("extra template goal content", lambda root: append(root / ".agent/PROJECT_GOAL.md", "\n## Goal\n\n- Actual project: Example.\n")),
        ("extra template profile content", lambda root: append(root / ".agent/PROJECT_PROFILE.md", "\n## Identity\n\n- Actual owner: User.\n")),
        ("extra template current content", lambda root: append(root / ".agent/CURRENT.md", "\n## Active state after adoption\n\n- Hidden active task: yes.\n")),
        ("template decision paragraph", lambda root: append(root / ".agent/RECORD.md", "\nDecision: use PostgreSQL.\n")),
        ("template learning bullet", lambda root: append(root / ".agent/LEARNINGS.md", "\n- Production uses PostgreSQL.\n")),
    ]
    for name, mutate in template_cases:
        expect_failure(name, "TEMPLATE", mutate)

    adopted_cases = [
        ("orphan outcome", lambda root: (make_adopted(root), append(root / ".agent/OUTCOMES.md", "\n## D-ORPHAN - COMPLETE\n\n- Closed: 2026-08-04T10:01:00-04:00\n- Acceptance: MET\n- Validation: check - PASSED\n"))),
        ("duplicate directive ID", lambda root: (make_adopted(root), append(root / ".agent/DIRECTIVES.md", "\n## D-FIXTURE-001\n"))),
        ("missing directive objective", lambda root: (make_adopted(root), (root / ".agent/DIRECTIVES.md").write_text((root / ".agent/DIRECTIVES.md").read_text(encoding="utf-8").replace("- Objective: Validate the fixture.", "- Objective:", 1), encoding="utf-8"))),
        ("invalid timestamp", lambda root: (make_adopted(root), (root / ".agent/DIRECTIVES.md").write_text((root / ".agent/DIRECTIVES.md").read_text(encoding="utf-8").replace("2026-08-04T10:00:00-04:00", "not-a-timestamp", 1), encoding="utf-8"))),
        ("missing relationship target", lambda root: (make_adopted(root), (root / ".agent/DIRECTIVES.md").write_text((root / ".agent/DIRECTIVES.md").read_text(encoding="utf-8").replace("- Related directive: none\n", "", 1), encoding="utf-8"))),
        ("invalid relationship target", lambda root: (make_adopted(root), (root / ".agent/DIRECTIVES.md").write_text((root / ".agent/DIRECTIVES.md").read_text(encoding="utf-8").replace("- Related directive: none", "- Related directive: D-NOPE", 1), encoding="utf-8"))),
        ("adopted profile placeholder", lambda root: (make_adopted(root), (root / ".agent/PROJECT_PROFILE.md").write_text((root / ".agent/PROJECT_PROFILE.md").read_text(encoding="utf-8").replace("fixture-project", "UNRESOLVED", 1), encoding="utf-8"))),
        ("invalid adopted current timestamp", lambda root: (make_adopted(root), (root / ".agent/CURRENT.md").write_text((root / ".agent/CURRENT.md").read_text(encoding="utf-8").replace("2026-08-04T10:00:00-04:00", "not-a-timestamp", 1), encoding="utf-8"))),
        ("adopted current nonexistent directive", lambda root: (make_adopted(root), (root / ".agent/CURRENT.md").write_text((root / ".agent/CURRENT.md").read_text(encoding="utf-8").replace("D-FIXTURE-001", "D-NOPE", 1), encoding="utf-8"))),
        ("missing outcome summary", lambda root: (make_adopted(root), (root / ".agent/OUTCOMES.md").write_text((root / ".agent/OUTCOMES.md").read_text(encoding="utf-8").replace("- Summary: Fixture passed.", "- Summary:", 1), encoding="utf-8"))),
        ("invalid second outcome timestamp", lambda root: (make_adopted(root), add_outcome(root, "OUT-FIXTURE-002", "not-a-timestamp", "OUT-FIXTURE-001"))),
        ("ANIMUS authority claim", lambda root: (make_adopted(root), append(root / "AGENTS.md", "\nANIMUS ONE issues directives.\n"))),
        ("ANIMUS control claim", lambda root: (make_adopted(root), append(root / "AGENTS.md", "\nANIMUS ONE controls project lifecycle.\n"))),
        ("missing governance file", lambda root: (make_adopted(root), (root / ".cursor/MCP.md").unlink())),
        ("conditional fallback", lambda root: (make_adopted(root), (root / ".cursor/rules/02-mimir.mdc").write_text(re.sub(r"\n## Fallback\n.*?(?=\n## Control status)", "", (root / ".cursor/rules/02-mimir.mdc").read_text(encoding="utf-8"), flags=re.S), encoding="utf-8"))),
        ("multiple always-applied rules", lambda root: (make_adopted(root), shutil.copy2(root / ".cursor/rules/00-core-governance.mdc", root / ".cursor/rules/99-duplicate-core.mdc"))),
        ("duplicate policy content", lambda root: (make_adopted(root), (root / ".cursor/rules/00-core-copy.mdc").write_text((root / ".cursor/rules/00-core-governance.mdc").read_text(encoding="utf-8").replace("alwaysApply: true", "alwaysApply: false", 1), encoding="utf-8"))),
        ("duplicate outcome ID", lambda root: (make_adopted(root), add_outcome(root, "OUT-FIXTURE-001", "2026-08-04T10:02:00-04:00", "none"))),
        ("invalid outcome supersession target", lambda root: (make_adopted(root), add_outcome(root, "OUT-FIXTURE-002", "2026-08-04T10:02:00-04:00", "OUT-NOPE"))),
        ("empty conditional fallback", lambda root: (make_adopted(root), (root / ".cursor/rules/02-mimir.mdc").write_text(re.sub(r"(?s)(## Fallback\n).*?(?=\n## Control status)", r"\1", (root / ".cursor/rules/02-mimir.mdc").read_text(encoding="utf-8")), encoding="utf-8"))),
        ("mixed ANIMUS negation", lambda root: (make_adopted(root), append(root / "AGENTS.md", "\nANIMUS ONE does not merely copy files; it controls project directives.\n"))),
        ("missing outcome validation field", lambda root: (make_adopted(root), (root / ".agent/OUTCOMES.md").write_text((root / ".agent/OUTCOMES.md").read_text(encoding="utf-8").replace("- Validation:\n  - validator - PASSED\n", "- Validation result: validator PASSED\n", 1), encoding="utf-8"))),
        ("validation state outside field", lambda root: (make_adopted(root), (root / ".agent/OUTCOMES.md").write_text((root / ".agent/OUTCOMES.md").read_text(encoding="utf-8").replace("- Summary: Fixture passed.", "- Summary: Fixture PASSED.", 1), encoding="utf-8"))),
        ("unknown follow-up directive", lambda root: (make_adopted(root), (root / ".agent/OUTCOMES.md").write_text((root / ".agent/OUTCOMES.md").read_text(encoding="utf-8").replace("- Follow-up directive: none", "- Follow-up directive: D-NOPE", 1), encoding="utf-8"))),
        ("adopted goal missing content", lambda root: (make_adopted(root), (root / ".agent/PROJECT_GOAL.md").write_text((root / ".agent/PROJECT_GOAL.md").read_text(encoding="utf-8").replace("- Provide the adopted project purpose.", "", 1), encoding="utf-8"))),
        ("adopted profile missing purpose", lambda root: (make_adopted(root), (root / ".agent/PROJECT_PROFILE.md").write_text((root / ".agent/PROJECT_PROFILE.md").read_text(encoding="utf-8").replace("- Purpose: Test fixture.", "- Purpose:", 1), encoding="utf-8"))),
        ("adopted record missing fields", lambda root: (make_adopted(root), (root / ".agent/RECORD.md").write_text("# Record\n\n## DEC-FIXTURE-001\n\n- Decision or event: incomplete.\n", encoding="utf-8"))),
        ("duplicate adopted record ID", lambda root: (make_adopted(root), append(root / ".agent/RECORD.md", "\n## DEC-FIXTURE-002\n\n- Date: 2026-08-05\n- Record or decision ID: DEC-FIXTURE-001\n- Status: ACTIVE\n- Decision or event: Duplicate.\n- Rationale: Test.\n- Affected areas: none\n- Supersedes record: none\n"))),
        ("invalid adopted record supersession", lambda root: (make_adopted(root), (root / ".agent/RECORD.md").write_text((root / ".agent/RECORD.md").read_text(encoding="utf-8").replace("- Supersedes record: none", "- Supersedes record: DEC-NOPE", 1), encoding="utf-8"))),
        ("invalid project goal timestamp", lambda root: (make_adopted(root), (root / ".agent/PROJECT_GOAL.md").write_text((root / ".agent/PROJECT_GOAL.md").read_text(encoding="utf-8").replace("2026-08-04T09:00:00-04:00", "not-a-timestamp", 1), encoding="utf-8"))),
        ("template current lifecycle status", lambda root: (make_adopted(root), (root / ".agent/CURRENT.md").write_text((root / ".agent/CURRENT.md").read_text(encoding="utf-8").replace("Status: `ADOPTED`", "Status: `TEMPLATE`", 1), encoding="utf-8"))),
        ("missing adopted profile status", lambda root: (make_adopted(root), (root / ".agent/PROJECT_PROFILE.md").write_text((root / ".agent/PROJECT_PROFILE.md").read_text(encoding="utf-8").replace("- Status: `ADOPTED`\n", "", 1), encoding="utf-8"))),
        ("duplicate adopted goal status", lambda root: (make_adopted(root), append(root / ".agent/PROJECT_GOAL.md", "\n- Status: `ADOPTED`\n"))),
        ("malformed learning entry", lambda root: (make_adopted(root), append(root / ".agent/LEARNINGS.md", "\n## L-FIXTURE-001\n\n- Fact or lesson: incomplete.\n"))),
        ("duplicate learning ID", lambda root: (make_adopted(root), add_learning(root, "L-FIXTURE-001"), add_learning(root, "L-FIXTURE-001"))),
        ("invalid learning supersession", lambda root: (make_adopted(root), add_learning(root, "L-FIXTURE-001", "L-NOPE"))),
        ("learning self-supersession", lambda root: (make_adopted(root), add_learning(root, "L-FIXTURE-001", "L-FIXTURE-001"))),
        ("multiline ANIMUS claim", lambda root: (make_adopted(root), append(root / "AGENTS.md", "\nANIMUS ONE\ncontrols project directives.\n"))),
        ("mixed case ANIMUS claim", lambda root: (make_adopted(root), append(root / "AGENTS.md", "\nAnimus One manages the project lifecycle.\n"))),
        ("decision self-supersession", lambda root: (make_adopted(root), (root / ".agent/RECORD.md").write_text((root / ".agent/RECORD.md").read_text(encoding="utf-8").replace("- Supersedes record: none", "- Supersedes record: DEC-FIXTURE-001", 1), encoding="utf-8"))),
        ("arbitrary directive heading", lambda root: (make_adopted(root), append(root / ".agent/DIRECTIVES.md", "\n## Unstructured directive\n"))),
        ("arbitrary outcome heading", lambda root: (make_adopted(root), append(root / ".agent/OUTCOMES.md", "\n## Unstructured outcome\n"))),
        ("arbitrary learning heading", lambda root: (make_adopted(root), append(root / ".agent/LEARNINGS.md", "\n## Miscellaneous note\n"))),
        ("duplicate directive field", lambda root: (make_adopted(root), append(root / ".agent/DIRECTIVES.md", "- Objective: Conflicting objective.\n"))),
        ("duplicate outcome field", lambda root: (make_adopted(root), append(root / ".agent/OUTCOMES.md", "- Summary: Conflicting summary.\n"))),
        ("duplicate current field", lambda root: (make_adopted(root), append(root / ".agent/CURRENT.md", "- Objective: `Conflicting objective`\n"))),
        ("unknown directive field", lambda root: (make_adopted(root), append(root / ".agent/DIRECTIVES.md", "- Unknown directive field: value\n"))),
        ("unknown outcome field", lambda root: (make_adopted(root), append(root / ".agent/OUTCOMES.md", "- Unknown outcome field: value\n"))),
        ("unknown learning field", lambda root: (make_adopted(root), add_learning(root, "L-FIXTURE-001"), append(root / ".agent/LEARNINGS.md", "- Unknown learning field: value\n"))),
        ("unknown record field", lambda root: (make_adopted(root), append(root / ".agent/RECORD.md", "- Unknown record field: value\n"))),
        ("duplicate current validation command", lambda root: (make_adopted(root), append(root / ".agent/CURRENT.md", "- Command or check: duplicate\n"))),
        ("duplicate current validation result", lambda root: (make_adopted(root), append(root / ".agent/CURRENT.md", "- Result: `FAILED`\n"))),
        ("idle stale acceptance", lambda root: (make_fresh_adopted(root), (root / ".agent/CURRENT.md").write_text((root / ".agent/CURRENT.md").read_text(encoding="utf-8").replace("Acceptance: `NONE`", "Acceptance: `stale`", 1), encoding="utf-8"))),
        ("idle stale phase", lambda root: (make_fresh_adopted(root), (root / ".agent/CURRENT.md").write_text((root / ".agent/CURRENT.md").read_text(encoding="utf-8").replace("Current phase: `NONE`", "Current phase: `stale`", 1), encoding="utf-8"))),
        ("idle stale next action", lambda root: (make_fresh_adopted(root), (root / ".agent/CURRENT.md").write_text((root / ".agent/CURRENT.md").read_text(encoding="utf-8").replace("Immediate next action: `NONE`", "Immediate next action: `stale`", 1), encoding="utf-8"))),
        ("learning heading ID mismatch", lambda root: (make_adopted(root), add_learning(root, "L-OTHER"))),
        ("record heading ID mismatch", lambda root: (make_adopted(root), (root / ".agent/RECORD.md").write_text((root / ".agent/RECORD.md").read_text(encoding="utf-8").replace("Record or decision ID: DEC-FIXTURE-001", "Record or decision ID: DEC-OTHER", 1), encoding="utf-8"))),
        ("forward directive relationship", lambda root: (make_adopted(root), append(root / ".agent/DIRECTIVES.md", "\n## D-FIXTURE-002\n\n- Issued: 2026-08-04T11:00:00-04:00\n- Issuer: User\n- External directive: none\n- Objective: Later.\n- Scope: fixture.\n- Exclusions: none.\n- Acceptance: pass.\n- Risk class: LOW\n- Relationship: amends\n- Related directive: D-FIXTURE-003\n- Status at issuance: ISSUED\n\n## D-FIXTURE-003\n\n- Issued: 2026-08-04T12:00:00-04:00\n- Issuer: User\n- External directive: none\n- Objective: New.\n- Scope: fixture.\n- Exclusions: none.\n- Acceptance: pass.\n- Risk class: LOW\n- Relationship: new\n- Related directive: none\n- Status at issuance: ISSUED\n"))),
        ("forward outcome supersession", lambda root: (make_adopted(root), add_outcome(root, "OUT-FIXTURE-002", "2026-08-04T10:02:00-04:00", "OUT-FIXTURE-003"), add_outcome(root, "OUT-FIXTURE-003", "2026-08-04T10:03:00-04:00"))),
        ("forward learning supersession", lambda root: (make_adopted(root), add_learning(root, "L-FIXTURE-001", "L-FIXTURE-002"), add_learning(root, "L-FIXTURE-002"))),
        ("forward record supersession", lambda root: (make_adopted(root), append(root / ".agent/RECORD.md", "\n## DEC-FIXTURE-002\n\n- Date: 2026-08-05\n- Record or decision ID: DEC-FIXTURE-002\n- Status: ACTIVE\n- Decision or event: First.\n- Rationale: Test.\n- Affected areas: none\n- Supersedes record: DEC-FIXTURE-003\n\n## DEC-FIXTURE-003\n\n- Date: 2026-08-06\n- Record or decision ID: DEC-FIXTURE-003\n- Status: ACTIVE\n- Decision or event: Later.\n- Rationale: Test.\n- Affected areas: none\n- Supersedes record: none\n"))),
        ("duplicate adopted profile purpose", lambda root: (make_adopted(root), append(root / ".agent/PROJECT_PROFILE.md", "- Purpose: Conflicting purpose.\n"))),
        ("duplicate adopted goal heading", lambda root: (make_adopted(root), append(root / ".agent/PROJECT_GOAL.md", "\n## Goal\n- Duplicate goal.\n"))),
        ("ANIMUS conjunction bypass", lambda root: (make_adopted(root), append(root / "AGENTS.md", "\nANIMUS ONE does not merely copy files and instead controls project directives.\n"))),
        ("unsupported directive correction heading", lambda root: (make_adopted(root), append(root / ".agent/DIRECTIVES.md", "\n## Correction: D-FIXTURE-001\n\n- Recorded: 2026-08-04T11:00:00-04:00\n"))),
        ("duplicate outcome closed", lambda root: (make_adopted(root), append(root / ".agent/OUTCOMES.md", "- Closed: 2026-08-04T11:02:00-04:00\n"))),
        ("duplicate outcome acceptance", lambda root: (make_adopted(root), append(root / ".agent/OUTCOMES.md", "- Acceptance: MET\n"))),
        ("duplicate outcome validation", lambda root: (make_adopted(root), append(root / ".agent/OUTCOMES.md", "- Validation:\n  - duplicate - PASSED\n"))),
        ("unknown adopted goal heading", lambda root: (make_adopted(root), append(root / ".agent/PROJECT_GOAL.md", "\n## Unknown\n- text.\n"))),
        ("duplicate adopted goal heading", lambda root: (make_adopted(root), append(root / ".agent/PROJECT_GOAL.md", "\n## Goal\n- duplicate.\n"))),
        ("unknown adopted profile heading", lambda root: (make_adopted(root), append(root / ".agent/PROJECT_PROFILE.md", "\n## Unknown\n- text.\n"))),
        ("duplicate adopted profile heading", lambda root: (make_adopted(root), append(root / ".agent/PROJECT_PROFILE.md", "\n## Identity\n- duplicate.\n"))),
        ("passive ANIMUS authority claim", lambda root: (make_adopted(root), append(root / "AGENTS.md", "\nProject directives are controlled by ANIMUS ONE.\n"))),
        ("preceding ANIMUS authority claim", lambda root: (make_adopted(root), append(root / "AGENTS.md", "\nDirective control belongs to Animus One.\n"))),
        ("duplicate current risks section", lambda root: (make_adopted(root), append(root / ".agent/CURRENT.md", "\n## Risks\n- duplicate.\n"))),
        ("duplicate current blockers section", lambda root: (make_adopted(root), append(root / ".agent/CURRENT.md", "\n## Blockers\n- duplicate.\n"))),
        ("invalid record status", lambda root: (make_adopted(root), (root / ".agent/RECORD.md").write_text((root / ".agent/RECORD.md").read_text(encoding="utf-8").replace("- Status: ACTIVE", "- Status: UNKNOWN", 1), encoding="utf-8"))),
        ("arbitrary repository map paragraph", lambda root: (make_adopted(root), append(root / ".agent/REPO_MAP.md", "\nThis prose is not a map entry.\n"))),
        ("malformed repository map entry", lambda root: (make_adopted(root), append(root / ".agent/REPO_MAP.md", "\n- src/ — missing backticks.\n"))),
    ]
    for name, mutate in adopted_cases:
        expect_failure(name, "ADOPTED", mutate)
    with tempfile.TemporaryDirectory(prefix="governance-fresh-adopted-") as directory:
        fresh = Path(directory) / "fresh"
        copy_repo(fresh)
        make_fresh_adopted(fresh)
        result = run(fresh, "ADOPTED")
        if result.returncode != 0:
            raise AssertionError(f"fresh adopted fixture failed\n{result.stdout}\n{result.stderr}")
    print("Governance mode tests PASSED")
    print("- TEMPLATE positive fixture plus 22 TEMPLATE rejection cases")
    print("- ADOPTED positive fixture plus 82 ADOPTED rejection cases")
    print("- Fresh ADOPTED fixture with empty ledgers passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
