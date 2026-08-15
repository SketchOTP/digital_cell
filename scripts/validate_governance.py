#!/usr/bin/env python3
"""Dependency-free validator for the reusable governance package."""

from __future__ import annotations

import argparse
import json
import re
import textwrap
from datetime import datetime
from pathlib import Path


REQUIRED = [
    "AGENTS.md", "CLAUDE.md", "GEMINI.md", "COMMANDMENTS_OF_THE_CODE.md",
    ".agent/PROJECT_GOAL.md", ".agent/PROJECT_PROFILE.md", ".agent/CURRENT.md",
    ".agent/DIRECTIVES.md", ".agent/OUTCOMES.md", ".agent/LEARNINGS.md",
    ".agent/RECORD.md", ".agent/REPO_MAP.md", ".cursor/mcp.json", ".cursor/MCP.md",
    ".cursor/skills/mimir/SKILL.md", ".agents/skills/authority-governance/SKILL.md",
    ".agents/skills/external-discovery/SKILL.md",
    "scripts/validate_governance.py",
    "scripts/test_validate_governance.py",
]
AGENT_CONTRACT_FILES = (
    ".agent/PROJECT_GOAL.md", ".agent/PROJECT_PROFILE.md", ".agent/CURRENT.md",
    ".agent/DIRECTIVES.md", ".agent/OUTCOMES.md", ".agent/LEARNINGS.md",
    ".agent/RECORD.md", ".agent/REPO_MAP.md",
)
VALIDATION_STATES = {"PASSED", "FAILED", "NOT RUN", "NOT APPLICABLE", "BLOCKED"}
CURRENT_STATUSES = {"IDLE", "PLANNING", "IN_PROGRESS", "VALIDATING", "BLOCKED", "COMPLETE"}
OUTCOME_STATUSES = {"COMPLETE", "PARTIAL", "BLOCKED", "FAILED", "CANCELLED", "SUPERSEDED"}
RECORD_STATUSES = {"PROPOSED", "ACTIVE", "SUPERSEDED", "REVERSED", "CLOSED"}
RISK_CLASSES = {"LOW", "NORMAL", "HIGH", "DESTRUCTIVE"}
RELATIONSHIPS = {"new", "resumes", "amends", "supersedes"}
DIRECTIVE_HEADING = re.compile(r"(D-[A-Za-z0-9][A-Za-z0-9_-]*)\s*$")
OUTCOME_HEADING = re.compile(r"(D-[A-Za-z0-9][A-Za-z0-9_-]*) - (COMPLETE|PARTIAL|BLOCKED|FAILED|CANCELLED|SUPERSEDED)\s*$")
LEARNING_HEADING = re.compile(r"L-[A-Za-z0-9][A-Za-z0-9_-]*\s*$")


def fail(errors: list[str], message: str) -> None:
    errors.append(message)


def read(root: Path, relative: str) -> str:
    return (root / relative).read_text(encoding="utf-8")


def parse_timestamp(value: str) -> bool:
    value = value.strip().strip("`")
    try:
        parsed = datetime.fromisoformat(value)
    except ValueError:
        return False
    return parsed.tzinfo is not None and parsed.utcoffset() is not None


def parse_date(value: str) -> bool:
    try:
        datetime.strptime(value, "%Y-%m-%d")
    except ValueError:
        return False
    return True


def outside_fences(text: str) -> str:
    result: list[str] = []
    fenced = False
    for line in text.splitlines():
        if line.strip().startswith("```"):
            fenced = not fenced
            continue
        if not fenced:
            result.append(line)
    return "\n".join(result)


def iter_sections(text: str) -> list[tuple[str, str]]:
    clean = outside_fences(text)
    matches = list(re.finditer(r"^## (.+)$", clean, re.MULTILINE))
    sections: list[tuple[str, str]] = []
    for index, match in enumerate(matches):
        end = matches[index + 1].start() if index + 1 < len(matches) else len(clean)
        sections.append((match.group(1).strip(), clean[match.end():end]))
    return sections


def body_for(text: str, heading: str, occurrence: int = 0) -> str:
    matches = [body for title, body in iter_sections(text) if title == heading]
    return matches[occurrence] if occurrence < len(matches) else ""


def normalized_nonfenced(text: str) -> str:
    lines: list[str] = []
    for line in outside_fences(text).strip().splitlines():
        line = line.rstrip()
        if not line and lines and not lines[-1]:
            continue
        lines.append(line)
    return "\n".join(lines)


TEMPLATE_LEDGER_CONTENT = {
    ".agent/DIRECTIVES.md": textwrap.dedent("""\
        # Project Directive Ledger Template

        After adoption, this append-only ledger records project directives issued by the user to the AI coder. The adopted project records those directives locally.

        ANIMUS ONE may copy or aggregate the resulting governance files for centralized visibility. ANIMUS ONE does not issue, approve, modify, execute, reconcile, or close directives.

        ## Entry schema after adoption

        Use one live entry for each accepted project task. Keep examples outside this file; do not add a heading beginning with a live directive ID until adoption.

        Do not record execution results here. Do not rewrite historical entries after adoption. Append corrections, amendments, and supersessions referencing the original entry.
    """).strip(),
    ".agent/OUTCOMES.md": textwrap.dedent("""\
        # Project Outcome Ledger Template

        After adoption, this append-only ledger records results for project directives. Every live outcome must reference one local directive ID.

        ## Entry schema after adoption

        Use live outcome headings only after adoption. The following schema is instructional and is not a live entry:

        Allowed adopted-project outcome states: `COMPLETE`, `PARTIAL`, `BLOCKED`, `FAILED`, `CANCELLED`, `SUPERSEDED`. Do not rewrite earlier entries; append corrections referencing the original.
    """).strip(),
    ".agent/LEARNINGS.md": textwrap.dedent("""\
        # Project Learnings Template

        After adoption, use this append-only file for durable, verified project knowledge only.

        ## Entry guidance after adoption

        Each live entry should include:

        - Learning ID.
        - Date.
        - Fact or lesson.
        - Evidence location.
        - Confidence: `VERIFIED` or `PROVISIONAL`.
        - Scope.
        - Supersedes or superseded-by reference when applicable.

        Do not add live entries to this template. Exclude temporary narration, raw logs, full source files, secrets, unsupported guesses, and facts already obvious from stable project documentation.
    """).strip(),
    ".agent/RECORD.md": textwrap.dedent("""\
        # Project Decision and Milestone Record Template

        After adoption, use this append-only record for major durable project events and decisions, not routine task outcomes.

        ## Entry guidance after adoption

        Use it for architectural decisions, governance changes, releases, qualification or certification events, major reversals, important milestones, and decision supersessions.

        Each live entry should include:

        - Date.
        - Record or decision ID.
        - Status.
        - Decision or event.
        - Rationale.
        - Affected areas.
        - Supersession relationship when applicable.

        Allowed status values are `PROPOSED`, `ACTIVE`, `SUPERSEDED`, `REVERSED`, and `CLOSED`.

        Do not add live decisions or milestones to this template. Examples must remain outside the shipped template state.
    """).strip(),
    ".agent/REPO_MAP.md": textwrap.dedent("""\
        # Repository Map Template

        After adoption, maintain a concise navigation aid. This template intentionally contains no map of its own repository.

        ## Recommended sections after adoption

        - Entry points.
        - Core modules.
        - Interfaces and contracts.
        - Tests and validation.
        - Configuration.
        - Generated areas.
        - External integration points.
        - Areas that must not be edited manually.

        ## Inclusion rules

        - Explain why every mapped path matters.
        - Prefer important entry points and boundaries over exhaustive listings.
        - Exclude vendored dependencies, caches, temporary task notes, and generated files unless their role matters.
        - Update the map when a touched or newly understood area changes.

        ## Entry format after adoption

        Use entries like this only in an adopted repository:
    """).strip(),
}

TEMPLATE_DOCUMENT_CONTENT = {
    ".agent/PROJECT_GOAL.md": textwrap.dedent("""\
        # Project Goal Template

        ## Lifecycle

        - Status: `TEMPLATE`
        - Last verified: `UNSET UNTIL ADOPTED`

        ## Goal

        <Describe the adopted repository’s purpose. This placeholder is non-authoritative while Status is `TEMPLATE`.>

        ## Observable success measures

        - <Define measurable outcomes for the adopted repository.>

        ## Scope

        - <Define what the adopted project includes.>

        ## Non-goals

        - <Define what the adopted project does not include.>

        ## Constraints

        - <Record only verified constraints after adoption.>

        ## Governing external specifications

        - None recorded until adoption.

        ## Owner or approval authority

        - <Identify the responsible owner or authority after adoption.>

        ## Adoption guidance

        Change Status to `ADOPTED` only after replacing every placeholder with verified project facts. Do not record task history in this file.
    """).strip(),
    ".agent/PROJECT_PROFILE.md": textwrap.dedent("""\
        # Project Profile Template

        ## Lifecycle

        - Status: `TEMPLATE`
        - Last verified: `UNSET UNTIL ADOPTED`

        ## Identity

        - Project name or identifier: `<adopted project name>`
        - Purpose: `<short verified purpose>`
        - Repository root: `<repository root>`
        - Verified remote: `UNSET UNTIL VERIFIED`
        - Maturity or current phase: `<verified phase or maturity>`

        ## Languages and runtimes

        - `<verified language or runtime>`

        ## Tools

        - Build: `UNSET UNTIL VERIFIED`
        - Test: `UNSET UNTIL VERIFIED`
        - Lint: `UNSET UNTIL VERIFIED`
        - Type-check: `UNSET UNTIL VERIFIED`
        - Packaging: `UNSET UNTIL VERIFIED`
        - Preferred navigation/indexing: `UNSET UNTIL VERIFIED`

        ## Verified commands

        - Governance template validation: `python3 scripts/validate_governance.py --mode TEMPLATE`
        - Adopted-project validation: `python3 scripts/validate_governance.py --mode ADOPTED`
        - Project-specific commands: `UNSET UNTIL ADOPTED`

        ## Constraints

        - Platform/compatibility: `<verified constraint or none>`
        - Security: `<verified constraint or none>`
        - Data handling: `<verified constraint or none>`
        - Deployment: `<verified constraint or none>`

        ## Adoption guidance

        Change Status to `ADOPTED` only after replacing placeholders and `UNSET` values with verified facts. Never leave executable-looking project-specific commands unverified.
    """).strip(),
    ".agent/CURRENT.md": textwrap.dedent("""\
        # Current State Template

        ## Lifecycle

        - Status: `TEMPLATE`
        - Last updated: `UNSET UNTIL ADOPTION`

        ## Active state after adoption

        - Local directive ID: `NONE`
        - External directive ID: `NONE`
        - Objective: `NONE`
        - Current status: `IDLE`
        - Acceptance: `NONE`
        - Current phase: `NONE`
        - Expected or actual touched areas: `NONE`
        - Immediate next action: `NONE`

        ## Temporary task-relevant facts

        - None.

        ## Last validation after adoption

        - Command or check: `NONE`
        - Result: `NONE`

        ## Risks

        - None.

        ## Blockers

        - None.

        ## Pending decisions

        - None.

        ## Status vocabulary

        Allowed adopted-project statuses: `IDLE`, `PLANNING`, `IN_PROGRESS`, `VALIDATING`, `BLOCKED`, `COMPLETE`. `CURRENT.md` is mutable and never replaces historical ledgers. Reset it to `IDLE` when an adopted task closes.
    """).strip(),
}


def require_line(text: str, expected: str, label: str, errors: list[str]) -> None:
    if expected not in text.splitlines():
        fail(errors, f"{label}: missing exact template field: {expected}")


def field_values(text: str, label: str, errors: list[str], pattern: str | None = None) -> list[str]:
    expression = pattern or rf"^- {re.escape(label)}: (.*)$"
    values = [match.group(1).strip() for match in re.finditer(expression, text, re.MULTILINE)]
    if len(values) != 1:
        fail(errors, f"duplicate or missing {label} field (found {len(values)})")
    if any(not value for value in values):
        fail(errors, f"empty {label} field")
    return values


def reject_unknown_fields(text: str, allowed: set[str], label: str, errors: list[str]) -> None:
    for match in re.finditer(r"^- ([A-Za-z][A-Za-z -]+):", text, re.MULTILINE):
        name = match.group(1).strip()
        if name not in allowed:
            fail(errors, f"{label}: unknown field {name}")


def frontmatter(path: Path, errors: list[str]) -> dict[str, str]:
    lines = path.read_text(encoding="utf-8").splitlines()
    if len(lines) < 3 or lines[0].strip() != "---":
        fail(errors, f"{path}: missing frontmatter")
        return {}
    try:
        end = lines.index("---", 1)
    except ValueError:
        fail(errors, f"{path}: unterminated frontmatter")
        return {}
    values: dict[str, str] = {}
    for line in lines[1:end]:
        if ":" not in line:
            fail(errors, f"{path}: malformed frontmatter")
            continue
        key, value = line.split(":", 1)
        values[key.strip()] = value.strip()
    if "alwaysApply" not in values:
        fail(errors, f"{path}: frontmatter lacks alwaysApply")
    return values


def validate_rules(root: Path, errors: list[str]) -> None:
    rules = sorted((root / ".cursor/rules").glob("*.mdc"))
    always_apply: list[Path] = []
    fingerprints: dict[str, Path] = {}
    for rule in rules:
        metadata = frontmatter(rule, errors)
        body = rule.read_text(encoding="utf-8")
        if metadata.get("alwaysApply") == "true":
            always_apply.append(rule)
        if metadata.get("alwaysApply") not in {"true", "false"}:
            fail(errors, f"{rule}: alwaysApply must be true or false")
        if "## Purpose" not in body and "# Canonical Policy Adapter" not in body:
            fail(errors, f"{rule}: missing purpose section")
        if metadata.get("alwaysApply") == "false":
            for required in ("Purpose", "Activation", "Non-applicability", "Fallback", "Control status"):
                sections = [section_body for title, section_body in iter_sections(body) if title == required]
                if len(sections) != 1:
                    fail(errors, f"{rule}: conditional rule lacks ## {required}")
                elif not re.search(r"\w", sections[0]):
                    fail(errors, f"{rule}: conditional rule has empty ## {required}")
            if not re.search(r"(?i)advisory|enforced", body):
                fail(errors, f"{rule}: conditional rule lacks advisory/enforced declaration")
        for paragraph in re.split(r"\n\s*\n", body):
            normalized = re.sub(r"\s+", " ", paragraph).strip()
            if len(normalized) >= 100 and not normalized.startswith("---"):
                previous = fingerprints.get(normalized)
                if previous:
                    fail(errors, f"duplicate Cursor policy content: {previous} and {rule}")
                fingerprints[normalized] = rule
    if len(always_apply) != 1:
        fail(errors, "Cursor rules must contain exactly one alwaysApply: true rule")


def validate_animus(root: Path, errors: list[str]) -> None:
    terms = r"authority|authorities|control|controls|manage|manages|own|owns|govern|governs|administer|administers|issue|issues|approve|approves|modify|modifies|execute|executes|reconcile|reconciles|close|closes"
    positive = re.compile(rf"(?i)\b(?:{terms})\b")
    files = [root / "AGENTS.md", root / "CLAUDE.md", root / "GEMINI.md", root / "COMMANDMENTS_OF_THE_CODE.md"]
    files += list((root / ".agent").glob("*.md")) + list((root / ".cursor").rglob("*.md"))
    files += list((root / ".cursor").rglob("*.mdc"))
    for path in files:
        paragraphs = re.split(r"\n\s*\n", path.read_text(encoding="utf-8"))
        for paragraph in paragraphs:
            normalized = re.sub(r"\s+", " ", paragraph)
            for match in re.finditer(r"(?i)animus one", normalized):
                segment = normalized[match.end():]
                for clause in re.split(r"[.;]|\bbut\b|\binstead\b", segment, flags=re.IGNORECASE):
                    for action in positive.finditer(clause):
                        prefix = clause[:action.start()].lower()
                        if not re.search(r"\b(?:does not|do not|not|no|never)\b", prefix) or re.search(r"\b(?:but|however)\b", prefix):
                            fail(errors, f"{path}: positive ANIMUS ONE authority claim")
                sentence = re.split(r"[.!?;]", normalized[:match.start()])[-1]
                if re.search(r"(?i)\b(?:authority|control|controls|controlled|manage|manages|managed|own|owns|govern|governs|administer|administers|issue|issues|approve|approves|modify|modifies|execute|executes|reconcile|reconciles|close|closes)\b", sentence) and not re.search(r"(?i)\b(?:does not|do not|not|no|never|without)\b", sentence):
                    fail(errors, f"{path}: positive ANIMUS ONE authority claim")


def validate_common(root: Path, errors: list[str]) -> None:
    for relative in REQUIRED:
        if not (root / relative).is_file():
            fail(errors, f"missing required file: {relative}")
    if (root / "MIGRATION_REPORT.md").exists():
        fail(errors, "MIGRATION_REPORT.md must remain outside the distributable template tree")
    try:
        data = json.loads(read(root, ".cursor/mcp.json"))
        if not isinstance(data, dict) or not isinstance(data.get("mcpServers"), dict):
            fail(errors, ".cursor/mcp.json: mcpServers must be an object")
    except (json.JSONDecodeError, FileNotFoundError) as exc:
        fail(errors, f".cursor/mcp.json: invalid configuration: {exc}")
    validate_rules(root, errors)
    agents = read(root, "AGENTS.md") if (root / "AGENTS.md").is_file() else ""
    for heading in ("## Mandatory preflight", "## Detailed guidance", "## Operating requirements"):
        if heading not in agents:
            fail(errors, f"AGENTS.md: missing required content: {heading}")
    for required in (
        "Codex is the primary coding agent", "complete `.agent/` contract",
        "Reading or validating them must not alter their data", "`.agents/skills/`",
        "Cursor rules and Claude/Gemini files are compatibility adapters",
    ):
        if required not in agents:
            fail(errors, f"AGENTS.md: missing Codex-first governance content: {required}")
    for relative in AGENT_CONTRACT_FILES:
        if not (root / relative).is_file():
            fail(errors, f"missing mandatory .agent contract file: {relative}")
    if "@AGENTS.md" not in read(root, "CLAUDE.md"):
        fail(errors, "CLAUDE.md: must import AGENTS.md")
    if "@./AGENTS.md" not in read(root, "GEMINI.md"):
        fail(errors, "GEMINI.md: must import AGENTS.md")
    skill = read(root, ".cursor/skills/mimir/SKILL.md")
    for heading in ("## Inputs", "## Ordered steps", "## Outputs", "## Failure handling", "## Verification"):
        if heading not in skill:
            fail(errors, f"Mimir skill: missing distinct workflow section {heading}")
    codex_skill = read(root, ".agents/skills/authority-governance/SKILL.md")
    for heading in ("## Purpose", "## Mandatory inputs", "## Ordered workflow", "## `.agent/` contract", "## Failure handling"):
        if heading not in codex_skill:
            fail(errors, f"Codex governance skill: missing distinct workflow section {heading}")
    for required in ("AGENTS.md", "PROJECT_GOAL.md", "PROJECT_PROFILE.md", "CURRENT.md", "DIRECTIVES.md", "OUTCOMES.md", "LEARNINGS.md", "RECORD.md", "REPO_MAP.md"):
        if required not in codex_skill:
            fail(errors, f"Codex governance skill: missing mandatory contract reference {required}")
    discovery_skill = read(root, ".agents/skills/external-discovery/SKILL.md")
    for heading in ("## Activation", "## Governing principle", "## Existing work", "## Discovery", "## Evaluation", "## Mid-implementation discovery", "## Reporting"):
        if heading not in discovery_skill:
            fail(errors, f"External discovery skill: missing distinct workflow section {heading}")
    validate_animus(root, errors)


def validate_template_state(root: Path, errors: list[str]) -> None:
    goal = read(root, ".agent/PROJECT_GOAL.md")
    profile = read(root, ".agent/PROJECT_PROFILE.md")
    current = read(root, ".agent/CURRENT.md")
    goal_fields = (
        "- Status: `TEMPLATE`", "- Last verified: `UNSET UNTIL ADOPTED`",
        "<Describe the adopted repository’s purpose. This placeholder is non-authoritative while Status is `TEMPLATE`.>",
        "- <Define measurable outcomes for the adopted repository.>", "- <Define what the adopted project includes.>",
        "- <Define what the adopted project does not include.>", "- <Record only verified constraints after adoption.>",
        "- None recorded until adoption.", "- <Identify the responsible owner or authority after adoption.>",
    )
    profile_fields = (
        "- Status: `TEMPLATE`", "- Last verified: `UNSET UNTIL ADOPTED`",
        "- Project name or identifier: `<adopted project name>`", "- Purpose: `<short verified purpose>`",
        "- Repository root: `<repository root>`", "- Verified remote: `UNSET UNTIL VERIFIED`",
        "- Maturity or current phase: `<verified phase or maturity>`", "- `<verified language or runtime>`",
        "- Build: `UNSET UNTIL VERIFIED`", "- Test: `UNSET UNTIL VERIFIED`", "- Lint: `UNSET UNTIL VERIFIED`",
        "- Type-check: `UNSET UNTIL VERIFIED`", "- Packaging: `UNSET UNTIL VERIFIED`",
        "- Preferred navigation/indexing: `UNSET UNTIL VERIFIED`", "- Project-specific commands: `UNSET UNTIL ADOPTED`",
        "- Platform/compatibility: `<verified constraint or none>`", "- Security: `<verified constraint or none>`",
        "- Data handling: `<verified constraint or none>`", "- Deployment: `<verified constraint or none>`",
    )
    for field in goal_fields:
        require_line(goal, field, "PROJECT_GOAL.md", errors)
    for field in profile_fields:
        require_line(profile, field, "PROJECT_PROFILE.md", errors)
    allowed_goal_headings = {"Lifecycle", "Goal", "Observable success measures", "Scope", "Non-goals", "Constraints", "Governing external specifications", "Owner or approval authority", "Adoption guidance"}
    allowed_profile_headings = {"Lifecycle", "Identity", "Languages and runtimes", "Tools", "Verified commands", "Constraints", "Adoption guidance"}
    for heading, _ in iter_sections(goal):
        if heading not in allowed_goal_headings:
            fail(errors, f"PROJECT_GOAL.md: undocumented template heading: ## {heading}")
    for heading, _ in iter_sections(profile):
        if heading not in allowed_profile_headings:
            fail(errors, f"PROJECT_PROFILE.md: undocumented template heading: ## {heading}")
    current_fields = (
        "- Status: `TEMPLATE`", "- Last updated: `UNSET UNTIL ADOPTION`", "- Local directive ID: `NONE`",
        "- External directive ID: `NONE`", "- Objective: `NONE`", "- Current status: `IDLE`",
        "- Acceptance: `NONE`", "- Current phase: `NONE`", "- Expected or actual touched areas: `NONE`",
        "- Immediate next action: `NONE`", "- Command or check: `NONE`", "- Result: `NONE`",
    )
    for field in current_fields:
        require_line(current, field, "CURRENT.md", errors)
    for relative, expected in {**TEMPLATE_DOCUMENT_CONTENT, **TEMPLATE_LEDGER_CONTENT}.items():
        actual = normalized_nonfenced(read(root, relative))
        if actual != expected:
            fail(errors, f"{relative}: non-fenced content differs from the clean template schema")
    allowed_current_headings = {"Lifecycle", "Active state after adoption", "Temporary task-relevant facts", "Last validation after adoption", "Risks", "Blockers", "Pending decisions", "Status vocabulary"}
    for heading, _ in iter_sections(current):
        if heading not in allowed_current_headings:
            fail(errors, f"CURRENT.md: undocumented template heading: ## {heading}")
    for heading in ("Temporary task-relevant facts", "Risks", "Blockers", "Pending decisions"):
        body = body_for(current, heading).strip()
        if body != "- None.":
            fail(errors, f"CURRENT.md: {heading} is not clean template state")
    ledger_allowed = {
        ".agent/DIRECTIVES.md": {"Entry schema after adoption"},
        ".agent/OUTCOMES.md": {"Entry schema after adoption"},
        ".agent/LEARNINGS.md": {"Entry guidance after adoption"},
        ".agent/RECORD.md": {"Entry guidance after adoption"},
        ".agent/REPO_MAP.md": {"Recommended sections after adoption", "Inclusion rules", "Entry format after adoption"},
    }
    for relative, allowed in ledger_allowed.items():
        if read(root, relative).count("```") % 2:
            fail(errors, f"{relative}: unbalanced fenced schema")
        for heading, _ in iter_sections(read(root, relative)):
            if heading not in allowed:
                fail(errors, f"{relative}: undocumented live or arbitrary heading: ## {heading}")
        expected = TEMPLATE_LEDGER_CONTENT[relative]
        actual = normalized_nonfenced(read(root, relative))
        if actual != expected:
            fail(errors, f"{relative}: non-fenced content differs from the clean template schema")
    repo_map = read(root, ".agent/REPO_MAP.md")
    for heading, _ in iter_sections(repo_map):
        if heading not in {"Recommended sections after adoption", "Inclusion rules", "Entry format after adoption"}:
            fail(errors, f".agent/REPO_MAP.md: undocumented template heading: ## {heading}")
    for marker in ("## Recommended sections after adoption", "## Inclusion rules", "## Entry format after adoption", "<path/to/important-area>"):
        if marker not in repo_map:
            fail(errors, f".agent/REPO_MAP.md: missing exact template marker: {marker}")


def validate_directives(text: str, errors: list[str]) -> set[str]:
    sections = iter_sections(text)
    entries = []
    for title, body in sections:
        if title == "Entry schema after adoption":
            continue
        if not DIRECTIVE_HEADING.fullmatch(title):
            fail(errors, f"DIRECTIVES.md: unstructured adopted heading: ## {title}")
        else:
            entries.append((title, body))
    ids = [DIRECTIVE_HEADING.fullmatch(title).group(1) for title, _ in entries]
    if len(ids) != len(set(ids)):
        fail(errors, "DIRECTIVES.md: duplicate directive ID")
    directive_ids = set(ids)
    prior_ids: set[str] = set()
    for directive_id, (_, body) in zip(ids, entries):
        required = {
            "Issued": r"^- Issued: (.+)$", "Issuer": r"^- Issuer: (.+)$",
            "External directive": r"^- External directive: (.+)$", "Objective": r"^- Objective: (.+)$",
            "Scope": r"^- Scope: (.+)$", "Exclusions": r"^- Exclusions: (.+)$",
            "Acceptance": r"^- Acceptance: (.+)$", "Risk class": r"^- Risk class: (.+)$",
            "Relationship": r"^- Relationship: (.+)$", "Related directive": r"^- Related directive: (.+)$",
            "Status at issuance": r"^- Status at issuance: ISSUED$",
        }
        reject_unknown_fields(body, set(required), f"DIRECTIVES.md: {directive_id}", errors)
        values: dict[str, str] = {}
        for name, pattern in required.items():
            matches = list(re.finditer(pattern, body, re.MULTILINE))
            if len(matches) != 1:
                fail(errors, f"DIRECTIVES.md: {directive_id} {name} must occur exactly once")
            elif matches[0].groups():
                values[name] = matches[0].group(1).strip()
                if not values[name]:
                    fail(errors, f"DIRECTIVES.md: {directive_id} empty {name}")
            else:
                values[name] = "ISSUED"
        if "Issued" in values and not parse_timestamp(values["Issued"]):
            fail(errors, f"DIRECTIVES.md: {directive_id} invalid issuance timestamp")
        if values.get("Issuer") != "User":
            fail(errors, f"DIRECTIVES.md: {directive_id} issuer must be User")
        if values.get("Risk class") not in RISK_CLASSES:
            fail(errors, f"DIRECTIVES.md: {directive_id} invalid risk class")
        relationship = values.get("Relationship")
        related = values.get("Related directive")
        if relationship not in RELATIONSHIPS:
            fail(errors, f"DIRECTIVES.md: {directive_id} invalid relationship")
        elif relationship == "new" and related != "none":
            fail(errors, f"DIRECTIVES.md: {directive_id} new relationship must target none")
        elif relationship != "new" and related not in prior_ids:
            fail(errors, f"DIRECTIVES.md: {directive_id} related directive is invalid")
        prior_ids.add(directive_id)
    return directive_ids


def validate_outcomes(text: str, directive_ids: set[str], errors: list[str]) -> None:
    entries = [(title, body) for title, body in iter_sections(text) if OUTCOME_HEADING.fullmatch(title)]
    for title, _ in iter_sections(text):
        if title not in {"Entry schema after adoption"} and not OUTCOME_HEADING.fullmatch(title):
            fail(errors, f"OUTCOMES.md: unstructured adopted heading: ## {title}")
    outcome_ids: set[str] = set()
    parsed: list[tuple[str, str, str]] = []
    for title, body in entries:
        match = OUTCOME_HEADING.fullmatch(title)
        assert match
        directive_id, state = match.groups()
        reject_unknown_fields(body, {"Outcome ID", "Supersedes outcome", "Closed", "Acceptance", "Summary", "Changed areas", "Validation", "Remaining risks", "Blockers", "Follow-up directive"}, f"OUTCOMES.md: {title}", errors)
        outcome_matches = list(re.finditer(r"^- Outcome ID: (.+)$", body, re.MULTILINE))
        supersedes_matches = list(re.finditer(r"^- Supersedes outcome: (.+)$", body, re.MULTILINE))
        outcome = outcome_matches[0] if len(outcome_matches) == 1 else None
        supersedes = supersedes_matches[0] if len(supersedes_matches) == 1 else None
        if len(outcome_matches) != 1:
            fail(errors, f"OUTCOMES.md: {title} Outcome ID must occur exactly once")
        if len(supersedes_matches) != 1:
            fail(errors, f"OUTCOMES.md: {title} Supersedes outcome must occur exactly once")
        for field in ("Closed", "Acceptance"):
            if len(re.findall(rf"^- {field}: .+$", body, re.MULTILINE)) != 1:
                fail(errors, f"OUTCOMES.md: {title} {field} field must occur exactly once")
        if not outcome or not outcome.group(1).strip():
            fail(errors, f"OUTCOMES.md: {title} missing Outcome ID")
            outcome_id = ""
        else:
            outcome_id = outcome.group(1).strip()
            if outcome_id in outcome_ids:
                fail(errors, f"OUTCOMES.md: duplicate Outcome ID {outcome_id}")
            outcome_ids.add(outcome_id)
        if directive_id not in directive_ids:
            fail(errors, f"OUTCOMES.md: orphan outcome references {directive_id}")
        if not supersedes or not supersedes.group(1).strip():
            fail(errors, f"OUTCOMES.md: {title} missing Supersedes outcome")
        closed = re.search(r"^- Closed: (.+)$", body, re.MULTILINE)
        if not closed or not parse_timestamp(closed.group(1).strip()):
            fail(errors, f"OUTCOMES.md: {title} invalid closure timestamp")
        if not re.search(r"^- Acceptance: (?:`)?(MET|PARTIAL|NOT MET)(?:`)?$", body, re.MULTILINE):
            fail(errors, f"OUTCOMES.md: {title} invalid acceptance state")
        for field in ("Summary", "Changed areas", "Remaining risks", "Blockers", "Follow-up directive"):
            if len(re.findall(rf"^- {re.escape(field)}: (.+)$", body, re.MULTILINE)) != 1:
                fail(errors, f"OUTCOMES.md: {title} missing {field}")
        validation_lines = re.findall(r"^  - .+ - (PASSED|FAILED|NOT RUN|NOT APPLICABLE|BLOCKED)$", body, re.MULTILINE)
        if not re.search(r"^- Validation:\s*$", body, re.MULTILINE) or not validation_lines:
            fail(errors, f"OUTCOMES.md: {title} lacks structured validation field")
        if len(re.findall(r"^- Validation:\s*$", body, re.MULTILINE)) != 1:
            fail(errors, f"OUTCOMES.md: {title} Validation field must occur exactly once")
        outside_validation = re.sub(r"^- Validation:\s*$.*?(?=^- [A-Za-z]|\Z)", "", body, flags=re.MULTILINE | re.DOTALL)
        if re.search(r"\b(?:PASSED|FAILED|NOT RUN|NOT APPLICABLE|BLOCKED)\b", outside_validation):
            fail(errors, f"OUTCOMES.md: {title} has validation state outside Validation field")
        follow_up = re.search(r"^- Follow-up directive: (.+)$", body, re.MULTILINE)
        if follow_up and follow_up.group(1).strip() not in {"none", *directive_ids}:
            fail(errors, f"OUTCOMES.md: {title} references an unknown follow-up directive")
        parsed.append((title, outcome_id, supersedes.group(1).strip() if supersedes else ""))
    seen: set[str] = set()
    for title, outcome_id, superseded in parsed:
        if superseded == outcome_id:
            fail(errors, f"OUTCOMES.md: {title} cannot supersede itself")
        if superseded != "none" and superseded not in seen:
            fail(errors, f"OUTCOMES.md: {title} supersession target is invalid")
        if outcome_id:
            seen.add(outcome_id)


def validate_learnings(text: str, errors: list[str]) -> None:
    entries = []
    for title, body in iter_sections(text):
        if title == "Entry guidance after adoption":
            continue
        if not LEARNING_HEADING.fullmatch(title):
            fail(errors, f"LEARNINGS.md: unstructured adopted heading: ## {title}")
        else:
            entries.append((title, body))
    learning_ids: set[str] = set()
    for title, body in entries:
        fields = {
            "Date": r"^- Date: (.+)$", "Learning ID": r"^- Learning ID: (.+)$",
            "Fact or lesson": r"^- Fact or lesson: (.+)$", "Evidence location": r"^- Evidence location: (.+)$",
            "Confidence": r"^- Confidence: (VERIFIED|PROVISIONAL)$", "Scope": r"^- Scope: (.+)$",
            "Supersedes learning": r"^- Supersedes learning: (.+)$",
        }
        reject_unknown_fields(body, set(fields), f"LEARNINGS.md: {title}", errors)
        values: dict[str, str] = {}
        for name, pattern in fields.items():
            matches = list(re.finditer(pattern, body, re.MULTILINE))
            if len(matches) != 1 or not matches[0].group(1).strip():
                fail(errors, f"LEARNINGS.md: {title} {name} must occur exactly once")
            else:
                values[name] = matches[0].group(1).strip()
        if "Date" in values and not parse_date(values["Date"]):
            fail(errors, f"LEARNINGS.md: {title} invalid date")
        learning_id = values.get("Learning ID", "")
        if learning_id and title != learning_id:
            fail(errors, f"LEARNINGS.md: {title} heading does not match Learning ID")
        if learning_id in learning_ids:
            fail(errors, f"LEARNINGS.md: duplicate learning ID {learning_id}")
        learning_ids.add(learning_id)
        supersedes = values.get("Supersedes learning", "")
        if supersedes == learning_id:
            fail(errors, f"LEARNINGS.md: {title} cannot supersede itself")
        if supersedes != "none" and supersedes not in learning_ids:
            fail(errors, f"LEARNINGS.md: {title} supersession reference is invalid")


def validate_adopted_current(root: Path, directive_ids: set[str], errors: list[str]) -> None:
    text = read(root, ".agent/CURRENT.md")
    reject_unknown_fields(text, {"Status", "Last updated", "Local directive ID", "External directive ID", "Objective", "Current status", "Acceptance", "Current phase", "Expected or actual touched areas", "Immediate next action", "Command or check", "Result"}, "CURRENT.md", errors)
    allowed_sections = {"Lifecycle", "Active state after adoption", "Temporary task-relevant facts", "Last validation after adoption", "Risks", "Blockers", "Pending decisions", "Status vocabulary"}
    for title, _ in iter_sections(text):
        if title not in allowed_sections:
            fail(errors, f"CURRENT.md: unknown section ## {title}")
    if any(sum(1 for title, _ in iter_sections(text) if title == heading) != 1 for heading in allowed_sections):
        fail(errors, "CURRENT.md: adopted sections must occur exactly once")
    allowed_sections = {"Lifecycle", "Active state after adoption", "Temporary task-relevant facts", "Last validation after adoption", "Risks", "Blockers", "Pending decisions", "Status vocabulary"}
    for title, _ in iter_sections(text):
        if title not in allowed_sections:
            fail(errors, f"CURRENT.md: unknown section ## {title}")
    statuses = field_values(text, "Status", errors, r"^- Status: `([^`]+)`$")
    if statuses != ["ADOPTED"]:
        fail(errors, "CURRENT.md: adopted mode requires exactly one ADOPTED Status")
    updated_values = field_values(text, "Last updated", errors)
    if len(updated_values) == 1 and not parse_timestamp(updated_values[0]):
        fail(errors, "CURRENT.md: invalid Last updated timestamp")
    status_values = field_values(text, "Current status", errors, r"^- Current status: `([^`]+)`$")
    status = status_values[0] if len(status_values) == 1 else ""
    if status not in CURRENT_STATUSES:
        fail(errors, "CURRENT.md: invalid current status")
    fields = ("Local directive ID", "External directive ID", "Objective", "Acceptance", "Current phase", "Expected or actual touched areas", "Immediate next action")
    values: dict[str, str] = {}
    for field in fields:
        matches = list(re.finditer(rf"^- {re.escape(field)}: `?([^`\n]+)`?$", text, re.MULTILINE))
        if len(matches) != 1 or not matches[0].group(1).strip():
            fail(errors, f"CURRENT.md: missing {field}")
        else:
            values[field] = matches[0].group(1).strip().strip("`")
    command_values = field_values(text, "Command or check", errors)
    result_values = field_values(text, "Result", errors, r"^- Result: `?([^`\n]+)`?$")
    if len(command_values) != 1 or not command_values[0]:
        fail(errors, "CURRENT.md: missing validation command/check")
    if len(result_values) != 1 or result_values[0] not in VALIDATION_STATES:
        fail(errors, "CURRENT.md: invalid validation result")
    for heading in ("Risks", "Blockers", "Pending decisions"):
        if not body_for(text, heading).strip():
            fail(errors, f"CURRENT.md: missing {heading} section")
    local_id = values.get("Local directive ID", "")
    if status == "IDLE":
        idle_fields = ("Local directive ID", "External directive ID", "Objective", "Acceptance", "Current phase", "Expected or actual touched areas", "Immediate next action")
        if any(values.get(field) != "NONE" for field in idle_fields):
            fail(errors, "CURRENT.md: IDLE state cannot contain stale task data")
    if status != "IDLE" and local_id not in directive_ids:
        fail(errors, "CURRENT.md: non-idle state must reference an existing directive")
    if status == "COMPLETE" and "awaiting reset" not in text.lower():
        fail(errors, "CURRENT.md: COMPLETE state must document awaiting reset")


def validate_adopted(root: Path, errors: list[str]) -> None:
    goal = read(root, ".agent/PROJECT_GOAL.md")
    profile = read(root, ".agent/PROJECT_PROFILE.md")
    for relative, text in ((".agent/PROJECT_GOAL.md", goal), (".agent/PROJECT_PROFILE.md", profile)):
        statuses = field_values(text, "Status", errors, r"^- Status: `([^`]+)`$")
        if statuses != ["ADOPTED"]:
            fail(errors, f"{relative}: adopted mode requires exactly one ADOPTED Status")
        if re.search(r"<[^>]+>|UNRESOLVED|UNSET|TEMPLATE", text):
            fail(errors, f"{relative}: adopted file contains unresolved template state")
    reject_unknown_fields(goal, {"Status", "Last verified"}, "PROJECT_GOAL.md", errors)
    reject_unknown_fields(profile, {"Status", "Last verified", "Project name or identifier", "Purpose", "Repository root", "Verified remote", "Maturity or current phase", "Build", "Test", "Lint", "Type-check", "Packaging", "Preferred navigation/indexing", "Project-specific commands", "Platform/compatibility", "Security", "Data handling", "Deployment"}, "PROJECT_PROFILE.md", errors)
    goal_sections = ("Goal", "Observable success measures", "Scope", "Non-goals", "Constraints", "Governing external specifications", "Owner or approval authority")
    allowed_goal_sections = {"Lifecycle", *goal_sections}
    for title, _ in iter_sections(goal):
        if title not in allowed_goal_sections:
            fail(errors, f"PROJECT_GOAL.md: unknown adopted section ## {title}")
    if any(sum(1 for title, _ in iter_sections(goal) if title == heading) != 1 for heading in allowed_goal_sections):
        fail(errors, "PROJECT_GOAL.md: adopted sections must occur exactly once")
    for heading in goal_sections:
        if len([title for title, _ in iter_sections(goal) if title == heading]) != 1 or not re.search(r"\w", body_for(goal, heading)):
            fail(errors, f"PROJECT_GOAL.md: missing substantive {heading}")
    goal_verified = field_values(goal, "Last verified", errors)
    if len(goal_verified) == 1 and not parse_timestamp(goal_verified[0]):
        fail(errors, "PROJECT_GOAL.md: invalid Last verified timestamp")
    profile_fields = ("Project name or identifier", "Purpose", "Repository root", "Verified remote", "Maturity or current phase")
    for field in profile_fields:
        if not re.search(rf"^- {re.escape(field)}: (?!$).+", profile, re.MULTILINE):
            fail(errors, f"PROJECT_PROFILE.md: missing {field}")
    for heading in ("Languages and runtimes", "Tools", "Verified commands", "Constraints"):
        if len([title for title, _ in iter_sections(profile) if title == heading]) != 1 or not re.search(r"\w", body_for(profile, heading)):
            fail(errors, f"PROJECT_PROFILE.md: missing substantive {heading}")
    allowed_profile_sections = {"Lifecycle", "Identity", "Languages and runtimes", "Tools", "Verified commands", "Constraints"}
    for title, _ in iter_sections(profile):
        if title not in allowed_profile_sections:
            fail(errors, f"PROJECT_PROFILE.md: unknown adopted section ## {title}")
    if any(sum(1 for title, _ in iter_sections(profile) if title == heading) != 1 for heading in allowed_profile_sections):
        fail(errors, "PROJECT_PROFILE.md: adopted sections must occur exactly once")
    profile_verified = field_values(profile, "Last verified", errors)
    if len(profile_verified) == 1 and not parse_timestamp(profile_verified[0]):
        fail(errors, "PROJECT_PROFILE.md: invalid Last verified timestamp")
    for field in ("Build", "Test", "Lint", "Type-check", "Packaging", "Preferred navigation/indexing", "Project-specific commands", "Platform/compatibility", "Security", "Data handling", "Deployment"):
        values = field_values(profile, field, errors)
        if len(values) == 1 and not values[0]:
            fail(errors, f"PROJECT_PROFILE.md: empty {field}")
    for field in ("Project name or identifier", "Purpose", "Repository root", "Verified remote", "Maturity or current phase"):
        field_values(profile, field, errors)
    directive_ids = validate_directives(read(root, ".agent/DIRECTIVES.md"), errors)
    validate_adopted_current(root, directive_ids, errors)
    validate_outcomes(read(root, ".agent/OUTCOMES.md"), directive_ids, errors)
    record = read(root, ".agent/RECORD.md")
    record_entries = [(title, body) for title, body in iter_sections(record) if title != "Entry guidance after adoption"]
    for title, _ in iter_sections(record):
        if title != "Entry guidance after adoption" and not title.startswith("DEC-"):
            fail(errors, f"RECORD.md: unstructured adopted heading: ## {title}")
    record_ids: set[str] = set()
    for title, body in record_entries:
        fields = {
            "Date": r"^- Date: (.+)$", "Record or decision ID": r"^- Record or decision ID: (.+)$",
            "Status": r"^- Status: (.+)$", "Decision or event": r"^- Decision or event: (.+)$",
            "Rationale": r"^- Rationale: (.+)$", "Affected areas": r"^- Affected areas: (.+)$",
            "Supersedes record": r"^- Supersedes record: (.+)$",
        }
        reject_unknown_fields(body, set(fields), f"RECORD.md: {title}", errors)
        values: dict[str, str] = {}
        for name, pattern in fields.items():
            matches = list(re.finditer(pattern, body, re.MULTILINE))
            if len(matches) != 1 or not matches[0].group(1).strip():
                fail(errors, f"RECORD.md: {title} missing {name}")
            else:
                values[name] = matches[0].group(1).strip()
        if "Date" in values and not (re.fullmatch(r"\d{4}-\d{2}-\d{2}", values["Date"]) and parse_date(values["Date"])):
            fail(errors, f"RECORD.md: {title} invalid date")
        if values.get("Status") not in RECORD_STATUSES:
            fail(errors, f"RECORD.md: {title} invalid status")
        record_id = values.get("Record or decision ID", "")
        if record_id and title != record_id:
            fail(errors, f"RECORD.md: {title} heading does not match Record or decision ID")
        if record_id in record_ids:
            fail(errors, f"RECORD.md: duplicate record ID {record_id}")
        record_ids.add(record_id)
        supersedes = values.get("Supersedes record", "")
        if supersedes == record_id:
            fail(errors, f"RECORD.md: {title} cannot supersede itself")
        if supersedes != "none" and supersedes not in record_ids:
            fail(errors, f"RECORD.md: {title} supersession reference is invalid")
    validate_learnings(read(root, ".agent/LEARNINGS.md"), errors)
    repo_map = read(root, ".agent/REPO_MAP.md")
    map_sections = {"Entry points", "Core modules", "Interfaces and contracts", "Tests and validation", "Configuration", "Generated areas", "External integration points", "Areas that must not be edited manually"}
    for line in outside_fences(repo_map).splitlines():
        stripped = line.strip()
        if not stripped or stripped == "# Repository Map" or stripped.startswith("## ") and stripped[3:] in map_sections:
            continue
        if not re.fullmatch(r"- `[^`]+` — .+", stripped):
            fail(errors, "REPO_MAP.md: arbitrary adopted prose or malformed map entry")
    for title, body in iter_sections(repo_map):
        if title not in map_sections:
            fail(errors, f"REPO_MAP.md: unknown adopted section ## {title}")
        for line in body.splitlines():
            if line.strip() and not re.fullmatch(r"- `[^`]+` — .+", line.strip()):
                fail(errors, f"REPO_MAP.md: malformed map entry or prose in ## {title}")
    if not re.search(r"^- `[^`]+` — .+", repo_map, re.MULTILINE):
        fail(errors, "REPO_MAP.md: adopted mode requires actual map entries")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mode", choices=("TEMPLATE", "ADOPTED"), required=True)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args(argv)
    root = args.root.resolve()
    errors: list[str] = []
    validate_common(root, errors)
    if args.mode == "TEMPLATE":
        validate_template_state(root, errors)
    else:
        validate_adopted(root, errors)
    if errors:
        print("Governance validation FAILED")
        for error in errors:
            print(f"- {error}")
        return 1
    print(f"Governance validation PASSED ({args.mode})")
    print(f"- checked {len(REQUIRED)} required files")
    print(f"- checked {len(list((root / '.cursor/rules').glob('*.mdc')))} Cursor rules")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
