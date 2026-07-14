# REPO_MAP.md

Concise navigation map for agents. Add entries as application code lands.

## Governance
- `.agent/PROJECT_GOAL.md` — product and architecture source of truth (digital lifeform end goal)
- `.agent/PROJECT_PROFILE.md` — repo identity, Mimir slug, constraints, commands
- `AGENTS.md` — agent governance and MCP workflows
- `COMMANDMENTS_OF_THE_CODE.md` — ethical and execution principles for coding agents
- `.cursor/rules/` — Cursor rule adapters (governance, Mimir, Serena, cocoindex, Animus)

## Agent memory
- `.agent/CURRENT.md` — mutable working state
- `.agent/DIRECTIVES.md` — append-only task log
- `.agent/OUTCOMES.md` — append-only results
- `.agent/LEARNINGS.md` — repo-specific lessons
- `.agent/REPO_MAP.md` — this file
- `.agent/RECORD.md` — operator-only architect instruction log (agents must not edit)

## Tooling
- `.cursor/mcp.json` — repo-local MCP (cocoindex-code)
- `.cocoindex_code/` — cocoindex index config (`ccc init` + `ccc index`)
- `.serena/project.yml` — Serena project name `digital_cell`
- `.cursor/skills/mimir/SKILL.md` — Mimir session workflow skill

## Application code
- `digital-protocell/` — Phase 1 Rust workspace (chemistry-core, experiment-runner, godot-bridge)
- `digital-protocell/crates/chemistry-core/` — simulation engine, observer, tests
- `digital-protocell/crates/experiment-runner/` — headless experiment CLI
- `digital-protocell/crates/godot-bridge/` — Godot GDExtension microscope shell
- `digital-protocell/godot/` — Godot 4 project (display only, no chemistry in GDScript)
- `digital-protocell/configs/` — baseline, starvation, sweep TOML
- `digital-protocell/crates/chemistry-core/src/bottleneck.rs` — D-003 balance and bottleneck diagnostics
- `digital-protocell/crates/chemistry-core/src/time_audit.rs` — simulated-time and dt telemetry
- `digital-protocell/crates/chemistry-core/src/seed_audit.rs` — initial seed SHA-256 audit
- `digital-protocell/crates/experiment-runner/src/d003.rs` — D-003 diagnose/calibrate/screen CLI
- `digital-protocell/crates/chemistry-core/src/candidate_identity.rs` — immutable CandidateIdentity + SHA-256 hashes
- `digital-protocell/crates/chemistry-core/src/attractor.rs` — attractor/transient classification
- `digital-protocell/crates/experiment-runner/src/d004.rs` — D-004 provenance/attractor audit CLI
- `digital-protocell/configs/d004/` — final calibrated candidate TOML configs
- `digital-protocell/experiments/generated/d004/` — D-004 audit artifacts
## experiment-runner / D-005
- `digital-protocell/crates/chemistry-core/src/seed_generator.rs` — analytical fresh-seed recipes
- `digital-protocell/crates/chemistry-core/src/continuation.rs` — snapshot continuation verification
- `digital-protocell/crates/chemistry-core/src/basin.rs` — basin outcome and macrostate velocity
- `digital-protocell/crates/chemistry-core/src/nullcline.rs` — nullcline intersection and Jacobian class
- `digital-protocell/crates/experiment-runner/src/d005.rs` — D-005 pipeline CLI
- `digital-protocell/crates/chemistry-core/tests/d005_tests.rs` — D-005 unit tests (22)
- `digital-protocell/experiments/generated/d005/` — D-005 artifacts
- `docs/d005_*.md` — D-005 reports

## D-006 surface turnover
- `digital-protocell/crates/chemistry-core/src/reactions.rs` — surface_turnover_v1 assembly/decay
- `digital-protocell/crates/chemistry-core/src/surface_calibration.rs` — planar B and prescribed radius law
- `digital-protocell/crates/experiment-runner/src/d006.rs` — planar candidates + coupled screen
- `docs/d005_final_closure.md` — D-005 accessibility closure

## D-006 surface turnover
- `digital-protocell/crates/chemistry-core/src/stage_d_gate.rs` — Stage D/E gate helpers (pure analysis)
- `digital-protocell/crates/experiment-runner/src/d006.rs` — D-006 bootstrap/screen/run-one CLI
- `digital-protocell/scripts/d006_stage_d_ledger.py` — job ledger + radius/catalyst aggregate
- `docs/d006_stage_d_completion.md` — Stage D completion + scientific conclusion

## D-007 joint kinetic search
- `digital-protocell/crates/chemistry-core/src/d007_analysis.rs` — schema, brackets, joint gates
- `digital-protocell/crates/experiment-runner/src/d007.rs` — strict-schema runner + reference config
- `digital-protocell/crates/chemistry-core/tests/d007_tests.rs` — D-007 unit tests (26)
- `digital-protocell/scripts/d007_structural_bracket.py` — 63-job structural orchestrator
- `digital-protocell/scripts/d007_diagnose_krep.py` — Stage D required-k_rep estimator
- `docs/d007_candidate_report.md` — D-007 scientific closure

## D-008 membrane metabolism
- `digital-protocell/crates/chemistry-core/src/fields.rs` — fixed seven-field current and next buffers
- `digital-protocell/crates/chemistry-core/src/snapshot.rs` — versioned five-field and seven-field snapshots
- `digital-protocell/crates/chemistry-core/src/membrane_transport.rs` — conservative selective soluble face transport
- `digital-protocell/crates/chemistry-core/src/membrane.rs` — membrane production, loss, localization, and calibration
- `digital-protocell/crates/chemistry-core/src/membrane_accounting.rs` — species transport and membrane ledgers
- `digital-protocell/crates/chemistry-core/tests/d008_tests.rs` — D-008 gated regression tests
- `digital-protocell/crates/experiment-runner/src/d008.rs` — staged D-008 runner
- `docs/d008_stage_0_schema.md` — Stage 0 schema and compatibility result
- `docs/d008_membrane_transport.md` — Stage A planar selectivity result
- `docs/d008_membrane_localization.md` — Stage B localization and turnover result
- `digital-protocell/configs/d008/stage_b_selected.toml` — selected Stage B membrane candidate
- `docs/d008_manifest_pointer.json` — governed D-008 runtime manifest pointer
