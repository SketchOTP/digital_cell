# Digital Cell Repository Instructions

Codex is the primary coding agent. This file is the always-on router; detailed Authority governance lives in `COMMANDMENTS_OF_THE_CODE.md`, the complete `.agent/` contract, and `.agents/skills/authority-governance/SKILL.md`.

## Mandatory preflight

Before planning, editing, or validating work, read the complete `.agent/` contract:

- `.agent/PROJECT_GOAL.md`
- `.agent/PROJECT_PROFILE.md`
- `.agent/CURRENT.md`
- `.agent/DIRECTIVES.md`
- `.agent/OUTCOMES.md`
- `.agent/LEARNINGS.md`
- `.agent/RECORD.md`
- `.agent/REPO_MAP.md`

These files are authoritative and consumed by multiple processes. Reading or validating them must not alter their data. Do not rewrite, reorder, reformat, truncate, replace, or selectively ignore them. Change them only for an authorized governance or project-state update, while preserving append-only rules.

## Detailed guidance

- Precedence, scope, safety, lifecycle, validation, and reporting: `COMMANDMENTS_OF_THE_CODE.md` and the `.agent/` contract.
- Codex governance workflow: `.agents/skills/authority-governance/SKILL.md`; canonical Codex skills live under `.agents/skills/`.
- External prior-art discovery: `.agents/skills/external-discovery/SKILL.md` when its activation conditions are met.
- Mimir workflow: `.cursor/skills/mimir/SKILL.md` only when Mimir is configured and applicable.
- Cursor rules and Claude/Gemini files are compatibility adapters; they must defer to this file.
- Digital Cell project facts, architecture, storage, validated tools, and scientific boundaries are recorded in `.agent/PROJECT_GOAL.md` and `.agent/PROJECT_PROFILE.md`.

## Operating requirements

Preserve existing qualified work and project-specific instructions. Inspect existing implementations and tests before substantial changes. Search for external prior art before building significant new capabilities. Validate proportionally, record outcomes in `.agent/OUTCOMES.md`, update `.agent/CURRENT.md`, and report failed, unavailable, skipped, and unrun checks honestly.

Nested `AGENTS.md` files may add scoped guidance. They must preserve this mandatory `.agent/` preflight and may not silently replace the repository contract.

The canonical Authority reference is `atlas:/home/sketch/Projects/authority/`. It is reference-only for this repository; do not modify it unless the user explicitly authorizes Authority changes.
