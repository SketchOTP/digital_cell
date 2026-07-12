# AGENTS.md

## Project

**digital_cell** — a standalone Linux-based digital lifeform: protocell → persistent embodied creature.

- **Source of truth:** `.agent/PROJECT_GOAL.md`
- **Identity and constraints:** `.agent/PROJECT_PROFILE.md`
- **Greenfield:** no application code yet; do not assume myCompanion/creature companion patterns

Key product constraints: no central behavior controller; no scripted emotions or LLM-as-personality; behavior must emerge from simulated chemistry, cells, metabolism, body, memory, and environment; autonomous when unobserved.

## Mode

Use Ponytail discipline: lazy senior dev, not careless junior.

Do the least work that is actually correct.

Default posture:

* Question whether the requested thing needs to exist.
* Reuse what is already in the repo before writing anything new.
* Prefer stdlib/native platform features over custom code.
* Prefer installed dependencies over new dependencies.
* Prefer deletion over addition.
* Prefer boring over clever.
* Prefer one clear line over a new helper.
* Prefer one clear helper over a new abstraction layer.
* Fewest files, smallest correct diff.
* Stop when the acceptance criteria are met.

Not negotiable:

* Understand the touched flow before changing it.
* Fix root cause, not only the reported symptom.
* Do not skip validation, security, accessibility, data safety, or needed error handling.
* Non-trivial logic gets one runnable check: the smallest test/self-check that would fail if broken.
* Mark deliberate shortcuts with `ponytail:` and name the ceiling/upgrade path.
* Do not invent future requirements.

## Instruction priority

When instructions **conflict**, follow this order:

1. Direct user request.
2. Existing repo behavior and tests.
3. This `AGENTS.md`.
4. Cursor rules.
5. General assumptions.

Never use this file to override explicit user constraints.

## Animus Project Directive Rules

This repo may receive work from Animus Directive, an external project-architect system.

Animus project directives use this format:

```text
DIRECTIVE ID: D-###
```

This is different from the local agent-memory directive ID used in `.agent/DIRECTIVES.md`, which uses:

```text
D-YYYYMMDD-HHMM-short-slug
```

Both IDs must be preserved.

When executing an Animus project directive:

1. Treat `DIRECTIVE ID: D-###` as the user-facing project directive ID.
2. Create a separate local agent-memory directive ID for `.agent/DIRECTIVES.md`.
3. Record the Animus project directive ID inside the `.agent/DIRECTIVES.md` entry.
4. Include the Animus project directive ID in `.agent/OUTCOMES.md`.
5. Include the Animus project directive ID in the final response.
6. Do not rename, replace, or collapse one ID into the other.

Example `.agent/DIRECTIVES.md` entry:

```md
- D-20260707-1130-roadmap-fix | project:D-014 | status:started | scope:directive-pipeline | ask:Implement roadmap-aware auto-next behavior | accept:Auto-next prioritizes failed/partial remediation and roadmap unmet criteria | plan:trace auto-next, patch priority, run targeted tests
```

Example `.agent/OUTCOMES.md` entry:

```md
- D-20260707-1130-roadmap-fix | project:D-014 | status:done | files:packages/core/orchestrator/auto-next.ts,tests/auto-next.test.ts | tests:npm test -- auto-next.test.ts PASS | accept:met | summary:Auto-next now prioritizes repair and roadmap acceptance criteria | next:
```

## Animus Directive Completion Evidence

For every Animus project directive, the final response must include enough evidence for Animus Result Review to judge the pasted response.

The final response must include:

* Animus project directive ID
* Local agent-memory directive ID
* Summary
* Files changed
* Tests run
* Test results
* Manual verification
* Deviations from directive
* Known issues
* Backup/savepoint created, if requested
* Next suggested step

Do not claim tests passed unless the exact command was run.

If proof is missing, state that plainly. Missing proof is acceptable; fake proof is not.

## Partial or Failed Animus Directive Work

If the directive is only partially completed or failed:

1. Do not continue into unrelated work.
2. Report exactly what completed.
3. Report exactly what remains broken.
4. Report the blocker.
5. Recommend a repair-focused next step.
6. Do not mark the work as satisfied unless the acceptance criteria were actually met.

## All rules, every time

Follow **every** applicable rule on **every** task. No cherry-picking. No silent skips.

* `AGENTS.md`, `.cursor/rules/`, user rules, and workspace rules all apply together.
* Instruction priority resolves **conflicts** only — it does **not** exempt lower-priority rules when there is no conflict.
* Task/sprint scope lines such as “no UI” or “do not edit PROJECT_GOAL.md” narrow **what to build or edit** — they do **not** suspend governance, validation, Ponytail discipline, or agent memory.
* Never treat one carve-out line as permission to skip unrelated requirements.
* If two instructions appear irreconcilable, stop and ask — do not silently drop a rule.

## Non-negotiable agent memory

Directive memory is **mandatory on every coding task**, including sprints that say “DO NOT touch `.agent/`”.

Always:

1. Append to `.agent/DIRECTIVES.md` at task start.
2. Update `.agent/CURRENT.md` at start and end.
3. Append to `.agent/OUTCOMES.md` before final response.

Task/sprint “do not touch `.agent/`” protects `.agent/PROJECT_GOAL.md` and append-only log history — it does **not** exempt the three steps above.

## Agent memory files

Maintain these files:

* `.agent/CURRENT.md` — small mutable current-state file.
* `.agent/DIRECTIVES.md` — append-only directive tracker.
* `.agent/OUTCOMES.md` — append-only outcome log.
* `.agent/LEARNINGS.md` — append-only repo lessons.
* `.agent/REPO_MAP.md` — concise module/file navigation map.

Create missing files. Never rewrite append-only logs. Do not reread entire append-only logs unless explicitly asked.

## How to use the memory files

Use the memory files as a lightweight operating system for future agents.

| File                   | Use it when                                      | Purpose                                                         |
| ---------------------- | ------------------------------------------------ | --------------------------------------------------------------- |
| `.agent/CURRENT.md`    | Start of task, mid-task drift check, end of task | Shows where the repo is right now                               |
| `.agent/DIRECTIVES.md` | Start of task and audit trail                    | Records what was asked and the acceptance target                |
| `.agent/OUTCOMES.md`   | Before final response and future debugging       | Records what actually happened                                  |
| `.agent/LEARNINGS.md`  | Before touching unfamiliar areas                 | Saves repo-specific lessons future agents should not rediscover |
| `.agent/REPO_MAP.md`   | Before searching or opening files                | Helps agents find the right files fast                          |

Use them in this order:

1. **Check where we are:** read `.agent/CURRENT.md`.
2. **Check recent history:** tail recent directives and outcomes.
3. **Find the right area:** grep `.agent/REPO_MAP.md`.
4. **Avoid repeated mistakes:** grep `.agent/LEARNINGS.md`.
5. **Do the smallest correct work.**
6. **Update state:** write outcome, current state, learnings, and repo map changes.

## Drift prevention

Use the directive and acceptance criteria as the compass.

Before editing:

* Restate the task in one sentence.
* Define `accept:<observable done condition>`.
* Identify expected touched files/modules.
* Check `.agent/CURRENT.md` for active blockers or unfinished work.

During work:

* If the solution starts touching unrelated areas, stop and ask whether the scope expanded.
* If a refactor becomes tempting, verify it is required by acceptance criteria.
* If new behavior is discovered, log it in `.agent/LEARNINGS.md` only if useful.
* If current work invalidates `.agent/CURRENT.md`, update it before continuing.

Before final response:

* Compare the completed work against `accept:`.
* Log `accept:met`, `accept:partial`, or `accept:not met`.
* Do not claim done if acceptance is not met.
* If blocked, record the smallest next action.

Drift warning signs:

* More than three unrelated files changed.
* New dependency added.
* New abstraction added for one caller.
* Build/test files changed without need.
* UI changed when task was backend-only.
* Repo map cannot explain why the touched file matters.
* Outcome summary no longer matches the original directive.

## Repo map navigation workflow

Before opening random files, use `.agent/REPO_MAP.md`.

Fast path:

```bash
grep -n "<feature-or-module>" .agent/REPO_MAP.md 2>/dev/null || true
grep -n "<known-file-or-symbol>" .agent/REPO_MAP.md 2>/dev/null || true
```

Then:

1. Open only the mapped files likely related to the task.
2. Follow imports/callers from those files.
3. Use targeted search only when the map is missing or stale — **required:** cocoindex-code `search` then Serena symbol tools (see Code navigation MCPs).
4. Update the map if a touched file is missing, renamed, or better understood.

Rules:

* The map is for navigation, not documentation bloat.
* Keep each entry short.
* Add only files/modules that help future agents move faster.
* Do not map generated, vendor, cache, or build output.
* If the map is wrong, correct the relevant line. Do not rewrite the whole file.

## Code navigation MCPs (required)

**Canonical rules:** `.cursor/rules/03-serena.mdc`, `.cursor/rules/04-cocoindex-code.mdc`
**Deep reference:** `docs/serena-tools.md`, `docs/cocoindex-code.md`

Configured in repo `.cursor/mcp.json` (`cocoindex-code`); Serena in global `~/.cursor/mcp.json`.

### When required

Use this two-layer stack on **every non-trivial code task** (not the trivial-edit exception: single file, ≤10 lines, no behavior/API/schema change):

| Situation                            | Start with                          | Then                                            |
| ------------------------------------ | ----------------------------------- | ----------------------------------------------- |
| Unfamiliar module or feature         | cocoindex-code `search`             | Serena symbol tools on hits                     |
| Known symbol name                    | Serena `find_symbol`                | `find_referencing_symbols` / `find_declaration` |
| Refactor, rename, or surgical edit   | Serena symbol edit tools            | `get_diagnostics_for_file`                      |
| Broad “how does X work?”             | cocoindex-code `search`             | Serena on result paths                          |
| Exact text/regex when keywords known | `rg` or Serena `search_for_pattern` | narrow `read_file`                              |

**Required:** Do not broad-scan the repo or guess from chat memory when these MCPs are reachable. Read each tool schema in the MCP panel before first use in a session.

**If blocked:** report `BLOCKED: <server> unavailable: <reason>`, fall back to `rg` + targeted reads, and log honestly in the outcome.

### Workflow

```text
1. .agent/REPO_MAP.md grep (if mapped)
2. cocoindex-code search — narrow scope by meaning
3. Serena initial_instructions (once/session) → activate_project (if needed)
4. get_symbols_overview / find_symbol → find_referencing_symbols / find_declaration
5. narrow read_file on target regions
6. edit (Serena symbol tools when appropriate)
7. get_diagnostics_for_file → smallest useful verify
```

Typical flow: **cocoindex-code** narrows → **Serena** grounds symbols → **edit** → **verify**.

### cocoindex-code MCP

Semantic repo search and indexing — find related code without exact keywords.

| Tool     | When to use                                                  |
| -------- | ------------------------------------------------------------ |
| `search` | Primary discovery — natural-language or code-snippet queries |

**`search` parameters:**

| Parameter       | Purpose                                                                   |
| --------------- | ------------------------------------------------------------------------- |
| `query`         | Natural language or code snippet (required)                               |
| `limit`         | Max results (default 5; max 100)                                          |
| `offset`        | Pagination offset                                                         |
| `refresh_index` | Update index before search (default true; false for back-to-back queries) |
| `languages`     | Filter, e.g. `["python"]`                                                 |
| `paths`         | Glob filter, e.g. `["src/utils/*"]`                                       |

Returns: file path, language, content chunk, start/end line, relevance score.

**Reindex after large changes (required):** After substantial edits (new modules, merges, refactors, bulk moves, or ~10+ files / 500+ lines in one task), run `ccc index` from repo root before the next semantic search. If blocked, use `refresh_index: true` on the first `search` after large changes.

### Serena MCP

LSP-backed symbol navigation and surgical edits. **Task start:** `initial_instructions` once per session before other Serena tools.

#### Setup / project

| Tool                   | When to use                                 |
| ---------------------- | ------------------------------------------- |
| `initial_instructions` | **First** — read Serena Instructions Manual |
| `activate_project`     | Bind Serena to the workspace project        |
| `onboarding`           | First-time Serena project setup             |
| `get_current_config`   | Inspect Serena configuration                |

#### Symbol navigation

| Tool                       | When to use                                        |
| -------------------------- | -------------------------------------------------- |
| `find_symbol`              | Locate symbols by name path pattern                |
| `find_declaration`         | Jump to symbol declaration                         |
| `find_referencing_symbols` | Find all references to a symbol                    |
| `find_implementations`     | Find implementations of interface/abstract symbols |
| `get_symbols_overview`     | File-level symbol tree overview                    |
| `get_diagnostics_for_file` | LSP diagnostics especially post-edit               |

#### Search / files

| Tool                 | When to use                         |
| -------------------- | ----------------------------------- |
| `search_for_pattern` | Regex/pattern search within project |
| `find_file`          | Locate files by name or pattern     |
| `read_file`          | Read file contents via Serena       |
| `list_dir`           | List directory contents             |

#### Symbol editing

| Tool                   | When to use                         |
| ---------------------- | ----------------------------------- |
| `replace_symbol_body`  | Replace entire symbol body          |
| `insert_before_symbol` | Insert code before a symbol         |
| `insert_after_symbol`  | Insert code after a symbol          |
| `rename_symbol`        | Rename symbol across codebase       |
| `safe_delete_symbol`   | Delete symbol with reference safety |
| `replace_content`      | Replace file content regions        |
| `create_text_file`     | Create new file                     |

#### Serena memory local notes only — not Mimir

| Tool            | When to use                     |
| --------------- | ------------------------------- |
| `write_memory`  | Store Serena-local project note |
| `read_memory`   | Read Serena-local note          |
| `list_memories` | List Serena-local notes         |
| `edit_memory`   | Edit Serena-local note          |
| `delete_memory` | Delete Serena-local note        |
| `rename_memory` | Rename Serena-local note        |

#### Shell

| Tool                    | When to use                                 |
| ----------------------- | ------------------------------------------- |
| `execute_shell_command` | Run shell command via Serena; use sparingly |

**Symbol tips:** name paths `ClassName/methodName`; append `[0]` for overloads; prefix `/` for absolute path within file; `depth > 0` on `find_symbol` for children; pass `relative_path` to scope search; prefer `rename_symbol` over manual search-replace.

## Mimir MCP (required)

**Canonical rule:** `.cursor/rules/02-mimir.mdc`
**Deep reference:** `docs/mimir-tools.md`, `.cursor/skills/mimir/SKILL.md`

Configured globally in `~/.cursor/mcp.json` with server key `mimir`.

Mimir is **durable cross-session memory only** — recall, search, remember, and record outcomes. Use cocoindex-code for codebase search and Serena for symbol navigation. Do not duplicate the same fact in Mimir and `.agent/LEARNINGS.md` unless audit requires both.

### Project slug

Use the correct Mimir project slug for the active repo.

If a project slug is already documented in repo-local rules or continuity docs, use that slug.

If no slug is documented, use the repo name as the project slug and record that choice in `.agent/CURRENT.md` and `.agent/OUTCOMES.md`.

Do not reuse a stale slug from another repo.

### When required

| Phase                                                          | Required action                                                                                      |
| -------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| **Every meaningful task start** not the trivial-edit exception | `memory_recall` → `project_status_summary` if resuming → `memory_search` before new/overlapping work |
| **During work**                                                | `memory_remember` for durable discoveries; `reflection_log` after repeated failures                  |
| **Before risky changes**                                       | `approval_request` for migrations, deps, schema, security, destructive edits                         |
| **End of every meaningful run**                                | **`memory_record_outcome`** — mandatory before final response when Mimir is reachable                |

**Required:** Never claim `session outcome recorded: yes` unless `memory_record_outcome` succeeded in this session. Never report `no`. Use `BLOCKED (<reason>)` only after a failed attempt or when MCP is unreachable.

**If blocked:** report `BLOCKED: Mimir MCP unavailable: <reason>`; fall back to `.agent/OUTCOMES.md` plus `docs/project-continuity.md`; include session summary in final response for manual entry.

**Never store:** secrets, credentials, `.env`, API keys, raw dumps, full files, private user data.

### Workflow

```text
Start:  memory_recall → project_status_summary? → memory_search?
        skill_list? before reusable automation
During: memory_remember / reflection_log / approval_request as needed
        plus cocoindex-code + Serena per Code navigation MCPs
End:    verify → .agent/ OUTCOMES + CURRENT → memory_record_outcome → final response
```

### `memory_record_outcome` mandatory at completion

Call **before** the final response on every meaningful task, same scope as `.agent/OUTCOMES.md`, excluding the trivial-edit exception.

| Parameter                                | Purpose                                                |
| ---------------------------------------- | ------------------------------------------------------ |
| `content`                                | What was done, files touched, validation run; required |
| `result`                                 | `COMPLETE`, `PARTIAL`, or `BLOCKED`; required          |
| `lesson`                                 | One durable takeaway for future sessions; optional     |
| `project`                                | Active repo Mimir project slug                         |
| `task_outcome`                           | `success`, `failure`, or `partial`; optional alias     |
| `has_correction` / `has_harmful_outcome` | Set when applicable                                    |

Example payload:

```text
content="Fixed X in acma/foo.py; pytest PASS", result="COMPLETE", lesson="…", project="<active-repo-slug>"
```

### Memory tools

| Tool                    | When to use                                                     |
| ----------------------- | --------------------------------------------------------------- |
| `memory_recall`         | Task start, context shifts, before modifying existing systems   |
| `memory_search`         | Before creating functionality that may already exist            |
| `memory_remember`       | Durable facts: root causes, constraints, architecture decisions |
| `memory_get`            | Inspect or validate a specific memory record                    |
| `memory_list`           | Audit memory layers or filtered sets                            |
| `memory_edit`           | Correct inaccurate or stale memory                              |
| `memory_delete`         | Remove invalid or unsafe memory                                 |
| `memory_supersede`      | Replace older knowledge with newer canonical record             |
| `memory_merge`          | Consolidate duplicate or fragmented memories                    |
| `memory_record_outcome` | **End of every meaningful run** — session result + validation   |

### Skill tools

| Tool             | When to use                                                  |
| ---------------- | ------------------------------------------------------------ |
| `skill_list`     | Before building reusable automation — check existing skills  |
| `skill_propose`  | Workflow is repeated, measurable, worth lifecycle management |
| `skill_test`     | Before activating or relying on a skill                      |
| `skill_activate` | After skill tested and validated                             |
| `skill_run`      | Execute a skill by `skill_id`; `input_data` optional         |

### Approval tools

| Tool               | When to use                                                            |
| ------------------ | ---------------------------------------------------------------------- |
| `approval_request` | Migrations, destructive changes, schema/deps/API/security/architecture |
| `approval_status`  | Check pending approval queue                                           |
| `approval_decide`  | Approve or reject with rationale                                       |

### Reflection tools

| Tool             | When to use                                                                                |
| ---------------- | ------------------------------------------------------------------------------------------ |
| `reflection_log` | Repeated failures, major debugging, operational lessons; requires observations and lessons |

### Improvement tools

| Tool                  | When to use                                      |
| --------------------- | ------------------------------------------------ |
| `improvement_list`    | Before proposing improvements — avoid duplicates |
| `improvement_get`     | Review existing proposal by ID                   |
| `improvement_propose` | Evidence-backed measurable improvement only      |

### Quarantine review tools

| Tool                           | When to use                                |
| ------------------------------ | ------------------------------------------ |
| `quarantine_review_list`       | Trust/safety audit of quarantined memories |
| `quarantine_review_reactivate` | After manual validation confirms safe      |
| `quarantine_review_keep`       | Safety concerns remain                     |

### Telemetry / project tools

| Tool                     | When to use                                                    |
| ------------------------ | -------------------------------------------------------------- |
| `telemetry_snapshot`     | Before performance or runtime-health claims                    |
| `retrieval_stats`        | Diagnose recall/search quality                                 |
| `project_status_summary` | Resuming dormant work or restoring project awareness           |
| `project_bootstrap`      | First connection or major context reset                        |
| `project_delete`         | Remove project materialization; destructive and approval-gated |

## Learnings workflow

Use `.agent/LEARNINGS.md` to avoid rediscovering repo-specific facts.

Before touching an unfamiliar area:

```bash
grep -n "<module-or-feature>" .agent/LEARNINGS.md 2>/dev/null || true
```

Append a learning only when it would save future time.

Good learning:

```md
- 2026-07-04 | area:auth | lesson:Session expiry is enforced in middleware before page loaders run | evidence:src/middleware.ts
```

Bad learning:

```md
- 2026-07-04 | area:auth | lesson:Always write clean code | evidence:none
```

Rules:

* Repo-specific only.
* No generic programming advice.
* No duplicate lessons.
* Keep under 25 words.
* Include evidence path.
* If a lesson changes, append a refinement instead of editing old history.

## Low-token context rule

At task start, read the smallest useful context:

```bash
cat .agent/CURRENT.md 2>/dev/null
tail -n 40 .agent/DIRECTIVES.md 2>/dev/null
tail -n 40 .agent/OUTCOMES.md 2>/dev/null
tail -n 60 .agent/LEARNINGS.md 2>/dev/null
grep -n "<touched-area>" .agent/REPO_MAP.md 2>/dev/null || true
git status --short
```

Rules:

* Use `tail`, `grep`, and targeted file reads.
* Do not scan the whole repo unless required.
* Do not reread full append-only logs.
* Prefer exact symbol/path search over broad search.
* If context is too large, summarize the needed part into `.agent/CURRENT.md`.

## Current state file

Maintain `.agent/CURRENT.md` as small mutable working memory.

Keep it under 80 lines.

Format:

```md
# CURRENT.md

## Active directive
- ID:
- Project directive:
- Goal:
- Status:
- Acceptance:
- Touched files:
- Next action:

## Repo facts needed now
- ...

## Last validation
- Command:
- Result:

## Open blockers
- ...
```

Rules:

* Read this first at task start.
* Update it at the start and end of every directive.
* Use it for current state only, not history.
* History belongs in append-only logs.
* Remove stale items when no longer relevant.
* Keep language short and factual.
* If another agent left work unfinished, use this file to continue without guessing.
* If the task comes from an Animus project directive, include the Animus `D-###` value under `Project directive:`.

## Start of every directive

Before edits:

1. Create local agent-memory directive ID:

```text
D-YYYYMMDD-HHMM-short-slug
```

2. If the task includes an Animus project directive ID, preserve it separately:

```text
DIRECTIVE ID: D-###
```

3. Read only needed context using the low-token context rule.
4. Define an observable acceptance condition.
5. Append the directive to `.agent/DIRECTIVES.md`.
6. Update `.agent/CURRENT.md`.

Directive format without Animus project directive:

```md
- D-YYYYMMDD-HHMM-slug | status:started | scope:<files/modules> | ask:<one sentence> | accept:<observable done condition> | plan:<max 3 tiny steps>
```

Directive format with Animus project directive:

```md
- D-YYYYMMDD-HHMM-slug | project:D-### | status:started | scope:<files/modules> | ask:<one sentence> | accept:<observable done condition> | plan:<max 3 tiny steps>
```

Example:

```md
- D-20260704-2215-login-fix | status:started | scope:auth/login | ask:Fix login redirect after expired session | accept:Expired session redirects to /login and test passes | plan:trace redirect, patch guard, run auth test
```

Animus example:

```md
- D-20260707-1130-roadmap-fix | project:D-014 | status:started | scope:directive-pipeline | ask:Implement roadmap-aware auto-next behavior | accept:Auto-next prioritizes failed/partial remediation and roadmap unmet criteria | plan:trace auto-next, patch priority, run targeted tests
```

## While coding

Use this ladder after reading the touched code:

1. Can this be skipped because it is speculative?
2. Does existing repo code already do it?
3. Can stdlib/native platform features do it?
4. Can an already-installed dependency do it?
5. Can one clear line do it?
6. Can a small local change do it?
7. Only then add a helper/module.

Rules:

* No abstraction with one implementation.
* No config for values that do not vary.
* No factories/services/managers “for later.”
* No new dependency for a few lines of code.
* No broad refactor while fixing a narrow bug.
* Grep callers before changing shared code.
* Keep existing naming/style unless it is actively harmful.
* If two options are equal size, choose the safer edge-case-correct one.
* Ask “does existing Y cover requested X?” when scope looks bloated.
* Do not silence errors. Fix causes.
* Do not add comments that restate code. Add comments only for non-obvious constraints.

## Mid-task checkpoint

Use a checkpoint when:

* More than one file changed.
* More than 15 minutes of work passed.
* The plan changed.
* New repo facts were discovered.
* A test failed.
* The task scope feels larger than expected.

Checkpoint questions:

1. Does the current work still match the directive?
2. Is acceptance still the same?
3. Are touched files still inside scope?
4. Is there a smaller fix?
5. Should `.agent/CURRENT.md` be updated before continuing?

If the answer exposes drift, stop and narrow the work.

## Validation

Run the smallest useful validation.

Priority:

1. Existing targeted test for touched area.
2. New minimal test for changed behavior.
3. Typecheck/lint/build only if relevant.
4. Manual command/self-check if no test harness exists.

Log skipped validation honestly.

Valid test log examples:

```md
tests:npm test -- auth/login.test.ts PASS
tests:python -m pytest tests/test_auth.py PASS
tests:not run; repo has no test script and change is markdown-only
```

Invalid:

```md
tests:should work
tests:probably fine
```

## Outcome log

Before final response, append to `.agent/OUTCOMES.md`.

Format without Animus project directive:

```md
- D-YYYYMMDD-HHMM-slug | status:done|partial|blocked | files:<changed files> | tests:<cmd/result or not run + why> | accept:<met|partial|not met> | summary:<one sentence> | next:<optional>
```

Format with Animus project directive:

```md
- D-YYYYMMDD-HHMM-slug | project:D-### | status:done|partial|blocked | files:<changed files> | tests:<cmd/result or not run + why> | accept:<met|partial|not met> | summary:<one sentence> | next:<optional>
```

Examples:

```md
- D-20260704-2215-login-fix | status:done | files:src/auth/guard.ts,tests/auth.guard.test.ts | tests:npm test -- auth.guard.test.ts PASS | accept:met | summary:Expired sessions now redirect to login before rendering protected routes | next:
```

```md
- D-20260704-2232-api-timeout | status:blocked | files:none | tests:not run; no code changed | accept:not met | summary:Could not reproduce because API credentials are missing | next:Provide local API env vars
```

```md
- D-20260707-1130-roadmap-fix | project:D-014 | status:done | files:packages/core/orchestrator/auto-next.ts,tests/auto-next.test.ts | tests:npm test -- auto-next.test.ts PASS | accept:met | summary:Auto-next now prioritizes repair and roadmap acceptance criteria | next:
```

## Learnings log

Append to `.agent/LEARNINGS.md` only when learning something useful to future agents.

Format:

```md
- YYYY-MM-DD | area:<module> | lesson:<specific repo fact under 25 words> | evidence:<path>
```

Rules:

* Repo-specific only.
* No generic programming advice.
* No repeated lessons; `grep` first.
* If a lesson changes, append `refines:<old keyword>`; do not edit history.
* Keep each lesson under 25 words.
* Prefer facts that save future investigation time.

## Repo map

Update `.agent/REPO_MAP.md` when files/modules are added, removed, renamed, or better understood.

Format:

```md
## <folder/module>
- `<path>` — <purpose in 12 words or less>
```

Rules:

* Update only touched or newly understood areas.
* Do not remap the whole repo during a small task.
* Do not map vendor/build/cache/generated output.
* Keep entries navigational, not architectural fan fiction.
* Prefer plain file purpose over implementation details.

Example:

```md
## Auth
- `src/middleware.ts` — protects routes and handles expired sessions
- `src/auth/session.ts` — reads and validates session tokens
```

## Append-only rules

Append-only files:

* `.agent/DIRECTIVES.md`
* `.agent/OUTCOMES.md`
* `.agent/LEARNINGS.md`

Rules:

* Add new lines only.
* Do not reorder.
* Do not rewrite old entries.
* Do not compact unless explicitly asked.
* If correcting an old entry, append a new correction line.

Correction format:

```md
- D-YYYYMMDD-HHMM-slug | correction:<what was wrong> | correct:<fixed fact>
```

Animus correction format:

```md
- D-YYYYMMDD-HHMM-slug | project:D-### | correction:<what was wrong> | correct:<fixed fact>
```

## End-of-task sequence

Before final response:

1. Compare work against acceptance criteria.
2. Run smallest useful validation.
3. Append `.agent/OUTCOMES.md`.
4. Update `.agent/CURRENT.md`.
5. Append `.agent/LEARNINGS.md` if a useful repo fact was discovered.
6. Update `.agent/REPO_MAP.md` if touched files are missing, moved, or better understood.
7. **Required:** call Mimir `memory_record_outcome` when MCP is reachable; see Mimir MCP section.
8. Check `git status --short`.
9. Respond with the required final format.

## Final response

Return only:

```md
D-YYYYMMDD-HHMM-slug

Changed:
- <short bullet>
- <short bullet>

Tests:
- <command/result or not run + why>

Memory/MCP:
- session outcome recorded: yes / BLOCKED (<reason>)

Next:
- <only if needed>
```

Rules:

* No essay.
* No fake certainty.
* No “future-proofing” claims.
* Mention blockers plainly.
* **`Memory/MCP`:** use `yes` only if `memory_record_outcome` succeeded; never `no`.
* Keep it short.
* If the task came from an Animus project directive, use the Animus override below instead of this default format.

## Final Response Override for Animus Project Directives

When the active task came from an Animus project directive, use this final response format instead of the shorter default format:

```md
PROJECT DIRECTIVE
- D-###

AGENT MEMORY DIRECTIVE
- D-YYYYMMDD-HHMM-slug

Changed:
- <short bullet>
- <short bullet>

Files changed:
- <path> — <reason>
- <path> — <reason>

Tests:
- <command/result or not run + why>

Manual verification:
- <check performed or None>

Deviations:
- <anything outside directive scope or None>

Known issues:
- <remaining issue or None>

Backup/savepoint:
- <tag/branch created or Not requested>

Memory/MCP:
- session outcome recorded: yes / BLOCKED (<reason>)

Next:
- <smallest logical next step, only if needed>
```

Rules:

* Keep it short.
* Do not omit the Animus project directive ID.
* Do not omit the local agent-memory directive ID.
* Do not claim completion if acceptance was partial or not met.
* Do not hide failed tests.
* Do not suggest broad future work when a repair step is needed.
* If the directive requested a backup/savepoint, report the exact tag or branch.
