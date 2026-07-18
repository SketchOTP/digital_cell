## Project Identity

- Source of truth: `.agent/PROJECT_GOAL.md`
- Repo profile: `.agent/PROJECT_PROFILE.md` when present
- Mimir project identity MUST come from `.agent/PROJECT_PROFILE.md` first; if absent, use the repo name and record that choice in `.agent/CURRENT.md`
- Never guess project identity, slug, or workspace binding from the path
- Do not hardcode another repo's product, stack, or identity
- Keep paths platform-neutral when the agent may run remotely

## Operating Mode

Use Ponytail discipline: do the least work that is actually correct.

- Question whether the requested thing needs to exist
- Reuse repo code before writing new code
- Prefer stdlib/native features over custom code
- Prefer installed dependencies over new dependencies
- Prefer deletion over addition
- Prefer boring over clever
- Prefer one clear line over a helper
- Prefer one clear helper over a new abstraction layer
- Fewest files, smallest correct diff
- Stop when acceptance is met

Not negotiable:

- Understand the touched flow before changing it
- Fix the root cause, not only the symptom
- Do not skip validation, security, accessibility, data safety, or required error handling
- Non-trivial logic gets one runnable check: the smallest test/self-check that would fail if broken
- Mark deliberate shortcuts with `ponytail:` and name the ceiling or upgrade path
- Do not invent future requirements

## Instruction Priority

When instructions conflict, follow this order:

1. Direct user request
2. Existing repo behavior and tests
3. `AGENTS.md`
4. `.cursor/rules/`
5. General assumptions

Governance files never override explicit user constraints.

## Repository Memory Files

Maintain these files as lightweight working memory:

- `.agent/CURRENT.md` - current task state
- `.agent/DIRECTIVES.md` - append-only directive log
- `.agent/OUTCOMES.md` - append-only outcome log
- `.agent/LEARNINGS.md` - append-only repo lessons
- `.agent/REPO_MAP.md` - concise module/file navigation map
- `.agent/PROJECT_PROFILE.md` - optional repo identity and tool summary

Rules:

- Create missing files
- Never rewrite append-only logs
- Do not reread whole append-only logs unless explicitly needed
- Keep `.agent/CURRENT.md` small and current only
- Update `.agent/CURRENT.md` at task start and end
- Append to `.agent/DIRECTIVES.md` at task start
- Append to `.agent/OUTCOMES.md` before final response
- Append to `.agent/LEARNINGS.md` only when a repo-specific fact will save future time
- Update `.agent/REPO_MAP.md` only for touched or newly understood files/modules

## Low-Token Start

At task start, read the smallest useful context:

- `.agent/PROJECT_PROFILE.md` if present
- `.agent/CURRENT.md`
- recent lines from `.agent/DIRECTIVES.md`
- recent lines from `.agent/OUTCOMES.md`
- recent lines from `.agent/LEARNINGS.md` when touching unfamiliar code
- `.agent/REPO_MAP.md` for the touched area
- `git status --short`

Do not scan the whole repo unless required.

## Drift Prevention

Before editing:

- Restate the task in one sentence
- Define `accept:<observable done condition>`
- Identify expected touched files/modules
- Check `.agent/CURRENT.md` for blockers or unfinished work

During work:

- If the solution touches unrelated areas, stop and ask whether scope expanded
- If a refactor becomes tempting, verify it is required by acceptance criteria
- If new behavior is discovered, log it in `.agent/LEARNINGS.md` only if useful
- If current work invalidates `.agent/CURRENT.md`, update it before continuing

Before final response:

- Compare completed work against `accept:`
- Log `accept:met`, `accept:partial`, or `accept:not met`
- Do not claim done if acceptance is not met
- If blocked, record the smallest next action

## Repo Map Workflow

Use `.agent/REPO_MAP.md` before random file hunting.

Fast path:

1. Search for the feature or module name in `.agent/REPO_MAP.md`
2. Open only mapped files likely related to the task
3. Follow imports or callers from those files
4. Use targeted search only when the map is missing or stale
5. Update the map if a touched file is missing, renamed, or better understood

Rules:

- Keep entries short
- Add only files/modules that help future agents move faster
- Do not map generated, vendor, cache, or build output
- If a map entry is wrong, correct that line only

## Code Navigation

Use the repo's code navigation tools on every non-trivial code task, except the trivial-edit exception: single file, 10 or fewer lines, no behavior/API/schema change.

Guidance:

- Start with symbol or graph search for unfamiliar code
- Use precise symbol lookup for known names
- Prefer surgical edits over broad scans
- If blocked by the navigation stack, report it plainly and fall back to targeted reads

## Mimir V2: Mandatory Lifecycle

Mimir V2 is mandatory for every coding task in this repository, including investigation, implementation, bug fixes, refactors, tests, and documentation that changes or explains code behavior. Pure conversation with no repository work does not start a Mimir task.

Use only this lifecycle, in order, for every coding task:

1. `mimir_project_register(workspace_root, name?, client_id?)` when the repository is not registered; otherwise `mimir_project_resolve(project_id, workspace_root, client_id)` to bind the known project to the actual checkout. Never guess project identity from a path.
2. `mimir_task_begin(project_id, workspace_root, client_id, worktree_id, objective)` at the start of every coding task; retain the returned task ID and version.
3. `mimir_context_compile(project_id, objective)` after task begin and before overlapping work. This is the normal project-scoped retrieval step.
4. `mimir_task_observe(task_id, version, event_type, payload_json, evidence_json)` only for durable causal facts Git cannot prove, such as decisions, constraints, hypotheses, and root causes. Do not use it for routine narration or file-change reporting. Every successful observation returns a new task version. Retain the latest version after every observation and pass that latest version to every later `mimir_task_observe` call and to `mimir_task_close`.
5. `mimir_validation_run(task_id, command, timeout_seconds?)` for completion-critical checks only. The command must be on the server allowlist.
6. `mimir_task_evidence(task_id)` before closure; failed or timed-out validations must not be represented as passing evidence.
7. `mimir_task_close(task_id, version, status, changed_files_json, tests_json, lessons_json)` only after evidence inspection, using the latest version and verified result.

Rules:

- Use the same lifecycle through CLI and MCP adapters
- Do not infer tool names from pasted legacy guidance
- Treat the repository as the source of truth and Mimir as the store for reasons, evidence, failures, fixes, and predictions
- Keep routine narration, raw command output, full files, credentials, and unsupported claims out of Mimir
- Reject high-confidence memory that lacks evidence
- Never mix project memory or store secrets/source files
- Keep paths platform-neutral when the agent can run remotely

## Mimir Direct Memory

The optional direct memory tools are only:

- `mimir_memory_query`
- `mimir_memory_propose`
- `mimir_memory_explain`

They do not replace task begin, compiled context, validation, evidence inspection, or task close.

Use the separate reviewed `mimir_project_onboard` and backfill workflow only for mature-repository import. Do not run it on every task.

## Mimir Failure Handling

If Mimir V2 is unavailable:

- Continue only when the change is safe
- Record the blockage honestly in the repository's local outcome/current-state files
- Never claim that Mimir context, evidence, or close-out succeeded

If a required Mimir step cannot be completed, say so plainly and do not fake the result.

## Validation

Run the smallest useful validation.

Priority:

1. Existing targeted test for the touched area
2. New minimal test for changed behavior
3. Typecheck, lint, or build only if relevant
4. Manual self-check if no test harness exists

Log skipped validation honestly.

## Append-Only Discipline

Append-only files:

- `.agent/DIRECTIVES.md`
- `.agent/OUTCOMES.md`
- `.agent/LEARNINGS.md`

Rules:

- Add new lines only
- Do not reorder
- Do not rewrite history
- If correcting an old entry, append a correction line

## Current State File

Keep `.agent/CURRENT.md` under 80 lines and use this shape:

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

## Start Of Every Directive

Before edits:

1. Create a local directive ID in this form: `D-YYYYMMDD-HHMM-short-slug`
2. If there is an Animus project directive, preserve it separately as `DIRECTIVE ID: D-###`
3. Read only the needed context
4. Define observable acceptance
5. Append the directive start line to `.agent/DIRECTIVES.md`
6. Update `.agent/CURRENT.md`

## While Coding

Use this ladder:

1. Can this be skipped because it is speculative?
2. Does existing repo code already do it?
3. Can stdlib or native platform features do it?
4. Can an installed dependency do it?
5. Can one clear line do it?
6. Can a small local change do it?
7. Only then add a helper or module

Rules:

- No abstraction with one implementation
- No config for values that do not vary
- No factories, managers, or services for later
- No new dependency for a few lines of code
- No broad refactor while fixing a narrow bug
- Search callers before changing shared code
- Keep existing naming and style unless actively harmful
- If two options are equal size, choose the safer edge-case-correct one
- Do not silence errors; fix causes
- Add comments only for non-obvious constraints

## End-Of-Task Sequence

Before final response:

1. Compare work against acceptance criteria
2. Run the smallest useful validation
3. Append `.agent/OUTCOMES.md`
4. Update `.agent/CURRENT.md`
5. Append `.agent/LEARNINGS.md` if useful
6. Update `.agent/REPO_MAP.md` if files/modules were added, removed, renamed, or better understood
7. Run the required Mimir close-out flow if reachable
8. Check `git status --short`
9. Respond in the required final format

## Final Response

Keep it short. No essay. No fake certainty.

Default format:

```md
D-YYYYMMDD-HHMM-slug

Changed:
- <short bullet>

Tests:
- <command/result or not run + why>

Memory/MCP:
- session outcome recorded: yes / BLOCKED (<reason>)

Next:
- <only if needed>
```

If there is an Animus project directive:

```md
PROJECT DIRECTIVE
- D-###

AGENT MEMORY DIRECTIVE
- D-YYYYMMDD-HHMM-slug

Changed:
- <short bullet>

Files changed:
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
