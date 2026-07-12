---
name: mimir
description: Mimir memory MCP workflow — recall, search, remember, record session outcomes
---

# Mimir Workflow

**Canonical rule:** `.cursor/rules/02-mimir.mdc`  
**Full tool catalog:** `AGENTS.md` § Mimir MCP · `docs/mimir-tools.md`

Use when Mimir MCP is available (server key `mimir` in `~/.cursor/mcp.json`).

## Start

1. `memory_recall` with project context (`project`: slug from `.agent/PROJECT_PROFILE.md`, or repo name)
2. `project_status_summary` if resuming dormant work
3. `memory_search` before creating or modifying potentially existing functionality
4. `skill_list` before reusable automation work

**Code search:** use cocoindex-code `search` (not Mimir) — see `docs/cocoindex-code.md`.

## During

- `memory_remember` for durable discoveries (root causes, architecture, constraints)
- `reflection_log` after repeated failures or major lessons
- `approval_request` before approval-gated changes

## End (required)

1. Run relevant verification
2. Inspect final diff
3. Append `.agent/OUTCOMES.md` and update `.agent/CURRENT.md`
4. **`memory_record_outcome`** — mandatory before final response when Mimir is reachable
   - Required: `content`, `result` (`COMPLETE`|`PARTIAL`|`BLOCKED`)
   - Recommended: `lesson`, `project`: slug from `.agent/PROJECT_PROFILE.md`, or repo name
5. Final response must include `Memory/MCP: session outcome recorded: yes` or `BLOCKED (<reason>)` — never `no`

## If BLOCKED

- Report `BLOCKED: Mimir MCP unavailable: <reason>`
- Fall back to `.agent/OUTCOMES.md` + `docs/project-continuity.md`
- Include session report in final response for manual entry

## Never store

Secrets, credentials, `.env`, API keys, raw dumps, full files, private user data, noisy temporary details.
