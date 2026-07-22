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
- `digital-protocell/crates/chemistry-core/src/activated_metabolism.rs` — Stage C activation, reproduction, decay rates and clamp gate
- `digital-protocell/crates/chemistry-core/src/stoichiometry.rs` — D-012 exact rationals, v1/v2 descriptors, conservation audit
- `digital-protocell/crates/chemistry-core/src/d012_accounting.rs` — D-012 material and activation-potential observer ledgers
- `digital-protocell/crates/chemistry-core/tests/d012_tests.rs` — D-012 v1 audit and v2 conservation gate tests (36)
- `docs/d012_conservative_network.md` — v2 network summary and non-equivalence notes
- `docs/d012_conservation_proof.md` — exact conservation proof and gate checklist
- `docs/d008_activated_metabolism.md` — Stage C metabolism pass report
- `digital-protocell/configs/d008/stage_c_selected.toml` — selected Stage C qualitative reference rates

## D-013 Stage E harness
- `digital-protocell/crates/chemistry-core/src/d013_harness.rs` — accepted-step windows, checkpoints, activation, validator
- `digital-protocell/crates/experiment-runner/src/d013.rs` — governed reference runner/preflight
- `digital-protocell/crates/chemistry-core/tests/d013_tests.rs` — D-013 harness integrity tests
- `digital-protocell/experiments/generated/d013/` — preservation, preflight, R22 reference, manifest

## D-014 numerical stability
- `digital-protocell/crates/chemistry-core/src/d014_numerics.rs` — dt limiters, recovery, cause classification
- `digital-protocell/crates/experiment-runner/src/d014.rs` — failure replay, diagnostic, preflight, fresh R22
- `digital-protocell/crates/chemistry-core/tests/d014_tests.rs` — controller/activation/projection tests
- `digital-protocell/experiments/generated/d014/` — preservation, telemetry, fresh R22, manifest

## D-015 waste throughput
- `digital-protocell/crates/chemistry-core/src/reservoir.rs` — N/F reservoir mask; W clearance on waste_sink_cell
- `digital-protocell/crates/chemistry-core/src/config.rs` — SimParams.waste_sink_inner_radius (default 83.0)
- `digital-protocell/crates/chemistry-core/src/d015_waste.rs` — waste budget, W-sink repair helpers, env hash v2
- `digital-protocell/crates/chemistry-core/tests/d015_tests.rs` — D-015 waste budget and repair tests (32)
- `digital-protocell/crates/experiment-runner/src/d015.rs` — preserve, controls, preflight, fresh-r22 runners
- `digital-protocell/experiments/generated/d015/` — preservation, controls, preflight, fresh R22 artifacts
- `docs/d015_*.md` — postmortem, clearance audit, sink capacity, env repair, candidate report

## D-016 waste transport timescale
- `digital-protocell/crates/chemistry-core/src/d016_transport.rs` — W transport audit, source field, timescales, fixed-source assay
- `digital-protocell/crates/chemistry-core/tests/d016_tests.rs` — D-016 transport/source/calibration tests (24)
- `digital-protocell/crates/experiment-runner/src/d016.rs` — preserve, audit, source/timescales, fixed-source campaign
- `digital-protocell/experiments/generated/d016/` — preservation, assays, manifest (local; gitignored)
- `docs/d016_*.md` — audit, source, timescales, conductance, resistance, calibration, candidate report

## D-017 waste architecture comparison
- `digital-protocell/crates/chemistry-core/src/d017_comparison.rs` — activation-yield vs active-export bounds
- `digital-protocell/crates/chemistry-core/tests/d017_tests.rs` — D-017 comparison falsification tests (17)
- `digital-protocell/crates/experiment-runner/src/d017.rs` — observer-only artifact pipeline
- `digital-protocell/experiments/generated/d017/` — comparison artifacts and manifest
- `docs/d017_*.md` — source, yield, feedback, interface, export, matrix, report

## D-018 structural provenance
- `digital-protocell/crates/chemistry-core/src/d018_provenance.rs` — observer E/K structure provenance tracer
- `digital-protocell/crates/chemistry-core/src/d018_analysis.rs` — basis, scaling, nullcline, conclusions
- `digital-protocell/crates/chemistry-core/tests/d018_tests.rs` — D-018 provenance/scaling tests
- `digital-protocell/crates/experiment-runner/src/d018.rs` — diagnostic pipeline and artifacts
- `digital-protocell/experiments/generated/d018/` — D-018 governed artifacts and manifest
- `docs/d018_*.md` — constraint semantics, provenance, scaling, nullcline, report

## experiment-runner / D-019
- `digital-protocell/crates/chemistry-core/src/structural_kinetics.rs` — v3 mechanism comparison and kinetics
- `digital-protocell/crates/chemistry-core/tests/d019_tests.rs` — D-019 mechanism/conservation tests
- `digital-protocell/crates/experiment-runner/src/d019.rs` — pipeline + Stage E reference runner
- `docs/d019_*.md` — mechanism comparison, selected mechanism, candidate report drafts

## D-020 joint-rate recovery
- `digital-protocell/crates/chemistry-core/src/d020_analysis.rs` — D-020 bounds, scoring, promotion gates
- `digital-protocell/crates/chemistry-core/tests/d020_tests.rs` — D-020 focused unit tests
- `digital-protocell/crates/experiment-runner/src/d020.rs` — D-020 flow audit and recovery runner
- `digital-protocell/experiments/generated/d020/` — D-020 recovery artifacts and manifest
- `docs/d020_joint_rate_recovery_report.md` — D-020 conclusion and evidence report

## D-021 interface-protected membrane
- `digital-protocell/crates/chemistry-core/src/d021_analysis.rs` — ε screen gates and bounded solver limits
- `digital-protocell/crates/chemistry-core/tests/d021_tests.rs` — D-021 focused unit tests
- `digital-protocell/crates/experiment-runner/src/d021.rs` — Gates 1–5 retention/localization pipeline
- `docs/d021_retention_localization_report.md` — D-021 conclusion and gate evidence
- `digital-protocell/experiments/generated/d021/` — Gate artifacts and manifest

## D-022 interface-affinity localization
- `digital-protocell/crates/chemistry-core/src/d022_analysis.rs` — χ screen gates and v5 solver bounds
- `digital-protocell/crates/chemistry-core/tests/d022_tests.rs` — antisymmetric flux/conservation/χ=0≡v4 tests
- `digital-protocell/crates/experiment-runner/src/d022.rs` — Gates 1–4 interface-affinity pipeline
- `docs/d022_interface_affinity_localization_report.md` — D-022 conclusion and gate evidence
- `digital-protocell/experiments/generated/d022/` — Gate artifacts and manifest


## D-023 membrane-precursor assembly
- `digital-protocell/crates/chemistry-core/tests/d023_tests.rs` — Gate0/1 eight-field schema and conservation tests
- `digital-protocell/crates/experiment-runner/src/d023.rs` — Gates 0–2 precursor pipeline (3–5 blocked on Gate2 fail)
- `docs/d023_membrane_precursor_assembly_report.md` — D-023 conclusion and gate evidence
- `digital-protocell/experiments/generated/d023/` — Gate artifacts and manifest
- `digital-protocell/crates/chemistry-core/tests/d024_tests.rs` — Gate0–5 v7 surface-density unit tests
- `digital-protocell/crates/experiment-runner/src/d024.rs` — Gates 0–6 interfacial surface-density pipeline
- `digital-protocell/experiments/generated/d024/` — Gate artifacts and manifest

## D-024 surface density
- `digital-protocell/crates/chemistry-core/src/surface_density.rs` — v7 S=δΓ geometry, reconstruction, surface diffusion/advection, adsorption/turnover helpers
- `digital-protocell/crates/chemistry-core/tests/d024_tests.rs` — D-024 Gate 0–5 unit coverage
- `digital-protocell/crates/experiment-runner/src/d024.rs` — D-024 Gates 0–6 artifact runner
- `digital-protocell/experiments/generated/d024/` — D-024 pass artifacts and manifest
- `docs/d024_interfacial_surface_density_report.md` — D-024 conclusion/report

## D-025 autonomous surface + Stage E re-entry
- `digital-protocell/crates/chemistry-core/src/surface_density.rs` — v_n estimator, autonomous S advection (`evolve_surface_density_with_vn`)
- `digital-protocell/crates/chemistry-core/tests/d025_tests.rs` — Gate 1–3 unit coverage
- `digital-protocell/crates/experiment-runner/src/d025.rs` — Gates 3–7 runner (Stage B–D, dynamic R22)
- `digital-protocell/experiments/generated/d025/` — Gate artifacts + `d024_provenance_seal/`
- `docs/d024_provenance_seal_addendum.md` — D-024 provenance seal

## D-025 Stage E
- `digital-protocell/crates/experiment-runner/src/d025_stage_e.rs` — Gate 8 constrained-radius Stage E reference/solve
- `digital-protocell/crates/chemistry-core/src/d025_analysis.rs` — D025 conclusions, joint balance, Stage E gates
- `docs/d025_stage_e_long_transient.md` — Gate 8 failure record
- `digital-protocell/experiments/generated/d025/` — gates 0–8 artifacts + manifest

## D-026 (Stage E A-budget diagnosis)
- `digital-protocell/crates/chemistry-core/src/d026_analysis.rs` — Gate0 parity, Gate1 observability, Gate2/5/6 classification
- `digital-protocell/crates/chemistry-core/src/simulation.rs` — `last_surface_totals` + D-026 diagnostic control flags
- `digital-protocell/crates/chemistry-core/tests/d026_tests.rs` — Gate0/1/5 unit tests (21)
- `digital-protocell/crates/experiment-runner/src/d026.rs` — `d026 gate0|gate1|gate2|gate5|classify`
- `docs/d026_stage_e_activated_resource_recovery.md` — conclusion `D026_SURFACE_COVERAGE_MAINTENANCE_DEFICIT`
- `digital-protocell/experiments/generated/d026/` — gated artifacts (gitignored generated/)

## D-027 coupled surface renewal
- `digital-protocell/crates/chemistry-core/src/surface_density.rs` — `SurfaceAccountingState` cumulative + window baseline
- `digital-protocell/crates/chemistry-core/src/d027_analysis.rs` — adsorption basis, portability, candidates
- `digital-protocell/crates/chemistry-core/tests/d027_tests.rs` — Gate0–2 unit tests
- `digital-protocell/crates/experiment-runner/src/d027.rs` — pipeline Gates 0/1/2/4
- `digital-protocell/crates/experiment-runner/src/d013.rs` — checkpoint `surface_accounting` + lossless precursor
- `docs/d027_coupled_surface_renewal.md` — governed report
- `digital-protocell/experiments/generated/d027/` — artifacts + manifest
- digital-protocell/crates/chemistry-core/src/d028_analysis.rs — bracketed regula-falsi/bisection, Q/g metrics, D-027 machine constants
- digital-protocell/crates/chemistry-core/tests/d028_tests.rs — Gate0–1 unit tests (bracket, monotonicity, solver)
- digital-protocell/crates/experiment-runner/src/d028.rs — Gates 0–3 pipeline + six-state portability
- docs/d028_bracketed_surface_renewal.md — governed report
- digital-protocell/experiments/generated/d028/ — artifacts + manifest

## D-029 reversible surface exchange (v8)
- `digital-protocell/crates/chemistry-core/src/surface_density.rs` — reversible P↔S exchange law + ledgers
- `digital-protocell/crates/chemistry-core/src/d029_analysis.rs` — A/B/L basis, weighted NNLS, candidates
- `digital-protocell/crates/chemistry-core/tests/d029_tests.rs` — Gate 1 unit suite
- `digital-protocell/crates/experiment-runner/src/d029.rs` — Gate 2/5/6 runner
- `digital-protocell/experiments/generated/d029/` — governed artifacts
- `docs/d029_reversible_surface_exchange.md` — report
## D-030 orthogonal reversible exchange ID
- `digital-protocell/crates/chemistry-core/src/d030_analysis.rs` — orthogonal assays, α/β estimators, fixed-inventory equilibrium
- `digital-protocell/crates/chemistry-core/tests/d030_tests.rs` — Gate 1–6 unit suite
- `digital-protocell/crates/experiment-runner/src/d030.rs` — Gates 0–8 pipeline + seed screen
- `digital-protocell/experiments/generated/d030/` — governed artifacts
- `docs/d030_orthogonal_reversible_exchange.md` — report


## D-031 invariant exchange
- `digital-protocell/crates/chemistry-core/src/surface_density.rs` — V2 BE+Strang integrator
- `digital-protocell/crates/chemistry-core/src/d031_analysis.rs` — Gate0 capacity/overshoot classification
- `digital-protocell/crates/chemistry-core/tests/d031_tests.rs` — unit invariant tests
- `digital-protocell/crates/experiment-runner/src/d031.rs` — Gate0/3/4 runner
- `docs/d031_invariant_domain_surface_exchange.md` — report
- `digital-protocell/experiments/generated/d031/` — artifacts (gitignored)

## D-032 activated surface assembly
- chemistry-core/src/d032_analysis.rs — k_active reconstruction, candidates, domain corners
- chemistry-core/src/surface_density.rs — apply_active_assembly_bounded; SurfaceAccountingTotals.active_assembly
- chemistry-core/src/config.rs — MembraneMetabolismV9ActivatedSurfaceAssembly; k_active; a_reference
- experiment-runner/src/d032.rs — Gates 0/2/3/5 pipeline
- chemistry-core/src/d033_analysis.rs — v10 orthogonal rate ID, bounded intermediate helpers
- chemistry-core/src/config.rs — MembraneMetabolismV11SurfaceMaturation; k_mature; d_u; is_surface_maturation()
- chemistry-core/src/fields.rs — immature_membrane field; FIELD_NAMES_V11
- chemistry-core/src/snapshot.rs — NineFieldSurfaceMaturationV1 schema; v10↔v11 resume rejection
- chemistry-core/src/surface_density.rs — evolve_surface_maturation_v11; dual P↔U exchange; maturation U+A→S+W (v11)
- experiment-runner/src/d033.rs — Gates 0–5 pipeline (buffering, numerical, isolated renewal)
- experiments/generated/d033/ — preservation, kinetics, buffering, numerical, manifest
- experiments/generated/d032/ — preservation, active_basis, manifest
- docs/d032_activated_surface_assembly.md — terminal report
- chemistry-core/src/d034_analysis.rs — v11_params(k_mature), passive U exchange regression, maturation ID/reconstruction, candidate screen
- chemistry-core/src/surface_density.rs — evolve_surface_maturation_v11; dual exchange helpers; maturation_delta accounting
- chemistry-core/tests/d034_tests.rs — schema, causality, conservation, capacity, snapshot v10↛v11
- experiment-runner/src/d034.rs — Gates 0–8 pipeline (preservation, passive exchange, maturation ID, rate reconstruction, isolated renewal)
- experiments/generated/d034/ — preservation, passive_exchange_regression, maturation_identification, rate_reconstruction, manifest

## D-034 surface maturation (v11)
- chemistry-core/src/d034_analysis.rs — P↔U assays, maturation ID, rate reconstruction
- chemistry-core/src/surface_density.rs — evolve_surface_maturation_v11, dual exchange, maturation
- chemistry-core/tests/d034_tests.rs — Gate0/1 unit suite (9)
- experiment-runner/src/d034.rs — Gates 0–8 pipeline (stops at Gate6)
- experiments/generated/d034/ — preservation, exchange regression, ID, rate_reconstruction, manifest
- docs/d034_surface_bound_membrane_maturation.md — terminal report

## D-035 membrane catalytic assembly
- `digital-protocell/crates/chemistry-core/src/d035_analysis.rs` — Gate 0–4 architecture/saturation/conservation/signature/rate screen + v12 helpers
- `digital-protocell/crates/chemistry-core/src/config.rs` — `MembraneMetabolismV12MembraneCatalyticAssembly` + k_mature_basal/cat, K_A, K_U
- `digital-protocell/crates/chemistry-core/src/surface_density.rs` — v12 saturating catalytic `maturation_rate_j`
- `digital-protocell/crates/experiment-runner/src/d035.rs` — D-035 pipeline Gates 0–5
- `digital-protocell/experiments/generated/d035/` — governed artifacts; terminal `D035_ISOLATED_CATALYTIC_RENEWAL_FAILURE`
- `digital-protocell/docs/d035_membrane_catalytic_assembly.md` — completion report

## D-036 membrane-bound catalytic complex
- `digital-protocell/crates/chemistry-core/src/d036_analysis.rs` — Gate 0 observer/runtime/ledger parity; Gate 1 η architecture screen
- `digital-protocell/crates/chemistry-core/tests/d036_tests.rs` — Gate 0–1 unit suite
- `digital-protocell/crates/experiment-runner/src/d036.rs` — pipeline Gates 0–1 (stop before v13)
- `digital-protocell/experiments/generated/d036/` — preservation, d035_parity, architecture_review, manifest
- `digital-protocell/docs/d036_membrane_bound_catalytic_complex.md` — terminal report (`D036_CATALYTIC_COMPLEX_ARCHITECTURE_REJECTED`)

## D-037 membrane assumption audit
- `digital-protocell/crates/chemistry-core/src/d037_analysis.rs` — Gates 0–7 turnover lineage, bulk/surface loss, provenance, state/gate semantics, reduced dynamics, route
- `digital-protocell/crates/chemistry-core/tests/d037_tests.rs` — focused audit tests (11)
- `digital-protocell/crates/experiment-runner/src/d037.rs` — artifact pipeline
- `digital-protocell/experiments/generated/d037/` — preservation + gate artifacts + manifest
- `docs/d037_membrane_assumption_audit.md` — `D037_TURNOVER_AND_GATE_DEFECTS` / Route A

## D-038 corrected turnover transfer + renewal replay
- `digital-protocell/crates/chemistry-core/src/config.rs` — `SurfaceTurnoverSchema` (schema 1 historical default; schema 2 D-021-equivalent)
- `digital-protocell/crates/chemistry-core/src/surface_density.rs` — `surface_turnover_lambda` / `apply_surface_turnover_exact` with `ε_M+(1−I(φ))`
- `digital-protocell/crates/chemistry-core/src/d038_analysis.rs` — Gates 0–2 helpers, multistart/route selection
- `digital-protocell/crates/chemistry-core/tests/d038_tests.rs` — focused suite (14)
- `digital-protocell/crates/experiment-runner/src/d038.rs` — Gates 0–10 pipeline
- `digital-protocell/experiments/generated/d038/` — governed artifacts
- `docs/d038_corrected_turnover_renewal.md` — report

## D-039 exchange+damage membrane maintenance
- `digital-protocell/crates/chemistry-core/src/d039_analysis.rs` — Gates 0–1/10 helpers, schema3 params, conclusion selection
- `digital-protocell/crates/chemistry-core/src/membrane_label_tracer.rs` — observer-only pulse-chase tracer
- `digital-protocell/crates/chemistry-core/tests/d039_tests.rs` — focused suite (10)
- `digital-protocell/crates/experiment-runner/src/d039.rs` — Gates 0–10 pipeline (schema3 exchange+damage-only)

## D-040 exchange–precursor coupling decomposition
- `digital-protocell/crates/chemistry-core/src/d040_analysis.rs` — equilibrium, chronology, endogenous/route classifiers, reduced APS model
- `digital-protocell/crates/chemistry-core/tests/d040_tests.rs` — focused suite (15)
- `digital-protocell/crates/experiment-runner/src/d040.rs` — Gates 0–9 diagnostic pipeline
- `digital-protocell/experiments/generated/d040/` — preservation through route_decision artifacts (gitignored)
- `docs/d040_exchange_precursor_decomposition.md` — `D040_MEMBRANE_METABOLISM_BISTABILITY` / Route F

## D-041 structural A retention
- `digital-protocell/crates/chemistry-core/src/membrane_transport.rs` — schema-3 Π_A=ρ_A exp(−β_A θ) on φ-crossing A faces
- `digital-protocell/crates/chemistry-core/src/d041_analysis.rs` — ρ_A screen helpers, nonredundancy, conclusions
- `digital-protocell/crates/chemistry-core/tests/d041_tests.rs` — transport isolation suite (9)
- `digital-protocell/crates/experiment-runner/src/d041.rs` — Gates 0–10 + diagnose-rho
- `digital-protocell/experiments/generated/d041/` — preservation, route_confirmation, retention_candidates, manifest
- `docs/d041_structural_a_retention.md` — `D041_STRUCTURAL_A_RETENTION_NOT_SUFFICIENT`

## D-042 activation capacity / buffer feasibility
- `digital-protocell/crates/chemistry-core/src/d042_analysis.rs` — A ledger, cumulative deficit, capacity class, ideal buffer, route rules
- `digital-protocell/crates/chemistry-core/tests/d042_tests.rs` — focused suite (13)
- `digital-protocell/crates/experiment-runner/src/d042.rs` — Gates 0–5 architecture audit (no chemistry change)
- `digital-protocell/experiments/generated/d042/` — preservation through route_decision artifacts (gitignored)
- `docs/d042_activation_capacity_buffer_feasibility.md` — `D042_ACTIVATION_CAPACITY_DEFICIT` / Route A

## D-043 activation capacity repair
- `digital-protocell/crates/chemistry-core/src/d043_analysis.rs` — rate law B=C·N·F, parity, capacity class, portable k reconstruction, candidate screen
- `digital-protocell/crates/chemistry-core/tests/d043_tests.rs` — focused suite (18)
- `digital-protocell/crates/experiment-runner/src/d043.rs` — Gates 0–9 stop-on-fail pipeline
- `digital-protocell/experiments/generated/d043/` — preservation through rate_reconstruction (gitignored)
- `docs/d043_activation_capacity_repair.md` — `D043_ACTIVATION_RATE_NOT_PORTABLE`

## D-044 activation-law architecture review
- `digital-protocell/crates/chemistry-core/src/d044_analysis.rs` — eligibility, scaling, candidate laws A/B/C, fit/holdout, conclusions
- `digital-protocell/crates/chemistry-core/tests/d044_tests.rs` — focused suite (16)
- `digital-protocell/crates/experiment-runner/src/d044.rs` — Gates 0–13 pipeline
- `digital-protocell/experiments/generated/d044/` — governed artifacts (gitignored)
- `docs/d044_activation_law_architecture_review.md` — D-044 conclusion report

## D-045 fuel-charged catalyst (Phase A stop)
- `digital-protocell/crates/chemistry-core/src/d045_analysis.rs` — demand scaling + QSS helpers
- `digital-protocell/crates/chemistry-core/tests/d045_tests.rs` — seal/scaling/QSS tests
- `digital-protocell/crates/experiment-runner/src/d045.rs` — Gate −1/0 stop-on-fail pipeline
- `docs/d045_fuel_charged_catalyst_activation.md` — Gate0 rejection report
- `digital-protocell/experiments/generated/d045/` — d044_seal, demand_scaling, manifest

## D-046 activated-resource demand topology
- `digital-protocell/crates/chemistry-core/src/d046_analysis.rs` — provenance, lineage, elasticities, models, route
- `digital-protocell/crates/chemistry-core/tests/d046_tests.rs` — focused Gate0–9 unit coverage
- `digital-protocell/crates/experiment-runner/src/d046.rs` — diagnostic pipeline Gates0–9
- `docs/d046_activated_resource_demand_topology.md` — completion report
- `digital-protocell/experiments/generated/d046/` — artifacts + manifest

## D-047 shared activated-resource pool sufficiency
- `digital-protocell/crates/chemistry-core/src/d047_analysis.rs` — fixed/altered classification, A role, tracer, competition, reduced dyn, route
- `digital-protocell/crates/chemistry-core/tests/d047_tests.rs` — focused Gate0–10 unit coverage
- `digital-protocell/crates/experiment-runner/src/d047.rs` — diagnostic pipeline Gates0–10
- `docs/d047_shared_activated_resource_pool.md` — completion report
- `digital-protocell/experiments/generated/d047/` — artifacts + manifest

## D-048 frozen-biology membrane basin and repair
- `digital-protocell/crates/chemistry-core/src/d048_analysis.rs` — candidate identity, seed contract, basin/healthy helpers, conclusions
- `digital-protocell/crates/chemistry-core/tests/d048_tests.rs` — focused Gate0–10 unit coverage
- `digital-protocell/crates/experiment-runner/src/d048.rs` — membrane basin/repair pipeline Gates0–10
- `digital-protocell/crates/chemistry-core/src/d049_analysis.rs` — D-049 completeness, ledgers, chronology, route, empirical reduced helpers
- `digital-protocell/crates/chemistry-core/tests/d049_tests.rs` — focused D-049 unit coverage (22 tests)
- `digital-protocell/crates/experiment-runner/src/d049.rs` — coupled A/P/S collapse decomposition pipeline Gates0–11
- `digital-protocell/crates/chemistry-core/src/d050_analysis.rs` — schema1/schema2 activation helpers, identification, conclusions
- `digital-protocell/crates/chemistry-core/tests/d050_tests.rs` — focused D-050 unit coverage (21 tests)
- `digital-protocell/crates/experiment-runner/src/d050.rs` — catalyst-saturating activation repair pipeline Gates0–13
- `digital-protocell/experiments/generated/d050/` — D-050 artifacts + manifest
- `docs/d048_frozen_biology_membrane_basin.md` — completion report (Gate2 fail)
- `digital-protocell/experiments/generated/d048/` — artifacts + manifest

## D-049 coupled A/P/S collapse feedback
- `digital-protocell/crates/chemistry-core/src/d049_analysis.rs` — completeness, ledgers, chronology, routes, empirical APS
- `digital-protocell/crates/chemistry-core/tests/d049_tests.rs` — focused suite (22)
- `digital-protocell/crates/experiment-runner/src/d049.rs` — Gates 0–11 diagnostic pipeline
- `docs/d049_coupled_aps_collapse_feedback.md` — completion report (`D049_COUPLED_ACTIVATION_CAPACITY_FAILURE`)
- `digital-protocell/experiments/generated/d049/` — governed artifacts + manifest

## D-050 catalyst-saturating activation
- `digital-protocell/crates/chemistry-core/src/d050_analysis.rs` — schema2 rate, Model C ID, conclusions
- `digital-protocell/crates/chemistry-core/tests/d050_tests.rs` — focused suite (21)
- `digital-protocell/crates/experiment-runner/src/d050.rs` — Gates 0–13 stop-on-fail pipeline
- `docs/d050_catalyst_saturating_activation.md` — completion (`D050_COUPLED_ACTIVATION_CAPACITY_NOT_RECOVERED`)
- `digital-protocell/experiments/generated/d050/` — governed artifacts + manifest

## D-051 coupled activation throughput audit
- `digital-protocell/crates/chemistry-core/src/d051_analysis.rs` — extent/resource/cohort/route classifiers
- `digital-protocell/crates/chemistry-core/tests/d051_tests.rs` — focused suite (13)
- `digital-protocell/crates/experiment-runner/src/d051.rs` — Gates −1–10 diagnostic pipeline
- `docs/d051_coupled_activation_throughput_audit.md` — completion (`D051_RESOURCE_THROUGHPUT_LIMIT`)
- `digital-protocell/experiments/generated/d051/` — governed artifacts + manifest

## D-052 resource delivery resistance
- `digital-protocell/crates/chemistry-core/src/d052_analysis.rs` — ledgers/resistance/route classifiers
- `digital-protocell/crates/chemistry-core/tests/d052_tests.rs` — focused suite (13)
- `digital-protocell/crates/experiment-runner/src/d052.rs` — Gates 0–12 diagnostic pipeline
- `docs/d052_resource_delivery_resistance_audit.md` — completion (`D052_MIXED_RESOURCE_DELIVERY_LIMIT`)
- `digital-protocell/experiments/generated/d052/` — governed artifacts + manifest

## D-053 combined resource-delivery repair
- `digital-protocell/crates/chemistry-core/src/d053_analysis.rs` — frozen schema-2 constants, isolated delivery repair helpers, candidates, conclusions
- `digital-protocell/crates/experiment-runner/src/d053.rs` — CLI pipeline Gates 0–9 plus explicit Gate9+ partial stop
- `digital-protocell/experiments/generated/d053/` — runtime artifacts and manifest

## D-053 combined resource delivery repair
- `digital-protocell/crates/chemistry-core/src/membrane_transport.rs` — m_ext exterior N/F faces; m_beta N/F beta co-scale
- `digital-protocell/crates/chemistry-core/src/d053_analysis.rs` — candidates, isolation, sensitivity, conclusions
- `digital-protocell/crates/chemistry-core/tests/d053_tests.rs` — focused suite (12)
- `digital-protocell/crates/experiment-runner/src/d053.rs` — Gates 0–14 pipeline
- `docs/d053_combined_resource_delivery_repair.md` — completion (`D053_NO_HEALTHY_RESOURCE_REPAIRED_ATTRACTOR`)
- `digital-protocell/experiments/generated/d053/` — governed artifacts + manifest

## D-054 resource geometry architecture audit
- `digital-protocell/crates/chemistry-core/src/d054_analysis.rs` — provenance/route helpers; geometry ratios; no biology change
- `digital-protocell/crates/chemistry-core/tests/d054_tests.rs` — focused suite (10)
- `docs/d054_resource_geometry_architecture_audit.md` — completion (`D054_D053_PROVENANCE_RERUN_DIVERGED`)
- `digital-protocell/experiments/generated/d054/` — seal + route_decision artifacts

## D-055 strict resource-gate replay + passive architecture
- `digital-protocell/crates/chemistry-core/src/d053_analysis.rs` — shared `evaluate_gate5` / `evaluate_gate8` (strict χ≥1.05; no short-horizon relax)
- `digital-protocell/crates/chemistry-core/src/d055_analysis.rs` — admission audit, route selection, architecture classifiers
- `digital-protocell/crates/chemistry-core/tests/d055_tests.rs` — focused suite (10)
- `digital-protocell/crates/experiment-runner/src/d055.rs` — Gates 0–12 pipeline
- `digital-protocell/crates/experiment-runner/src/d053.rs` — Gate5/8 wired to shared evaluator
- `docs/d055_strict_resource_architecture_review.md` — `D055_PASSIVE_RESOURCE_TRANSPORT_ARCHITECTURE_INSUFFICIENT` Route_P
- `digital-protocell/experiments/generated/d055/` — preservation through route_decision artifacts
## D-056 waste-coupled resource carrier
- `digital-protocell/crates/chemistry-core/src/d056_analysis.rs` — carrier law, conservation, capacity, ID helpers
- `digital-protocell/crates/chemistry-core/tests/d056_tests.rs` — focused suite (9)
- `digital-protocell/crates/experiment-runner/src/d056.rs` — Phase A Gates 0–5 stop-on-fail pipeline
- `docs/d056_waste_coupled_resource_carrier.md` — `D056_CARRIER_KINETICS_NOT_IDENTIFIABLE`
- `digital-protocell/experiments/generated/d056/` — preservation through parameter_identification artifacts
## D-057 carrier geometry / driving-force audit
- `digital-protocell/crates/chemistry-core/src/d057_analysis.rs` — dimensional ledger, measures, drives, route rules
- `digital-protocell/crates/chemistry-core/tests/d057_tests.rs` — focused suite (10)
- `digital-protocell/crates/experiment-runner/src/d057.rs` — Gates −1…10 observer pipeline
- `docs/d057_carrier_geometry_driving_force_audit.md` — `D057_CARRIER_GRID_OR_SURFACE_NORMALIZATION_DEFECT` Route G
- `digital-protocell/experiments/generated/d057/` — seal through route_decision artifacts

## D-058 corrected carrier normalization
- `digital-protocell/crates/chemistry-core/src/d058_analysis.rs` — canonical face op, corrected k_T★, synthetic invariance, Route V/Q/D/R/I
- `digital-protocell/crates/chemistry-core/tests/d058_tests.rs` — δ/face/dt/volume/invariance/fixture/route tests
- `digital-protocell/crates/experiment-runner/src/d058.rs` — Gates −1…9 observer/shadow pipeline
- `docs/d058_corrected_carrier_normalization_audit.md` — `D058_CARRIER_SURFACE_VOLUME_CAPACITY_LIMIT` Route V
- `digital-protocell/experiments/generated/d058/` — artifacts + manifest

## D-059 viable size basin / membrane-area review
- `digital-protocell/crates/chemistry-core/src/d059_analysis.rs` — matched scaling, global k_T ladder, size/area/topology route rules
- `digital-protocell/crates/chemistry-core/tests/d059_tests.rs` — Route V repro, frontier, restoring, area, topology tests
- `digital-protocell/crates/experiment-runner/src/d059.rs` — Gates −1…11 shadow/observer pipeline
- `docs/d059_viable_size_basin_membrane_area_review.md` — size vs membrane-area architecture review
- `digital-protocell/experiments/generated/d059/` → `/mnt/storage1tb/.../d059`

## experiment-runner / D-060
- `digital-protocell/crates/chemistry-core/src/d060_analysis.rs` — structural ledger, drive surface, neutrality cause, candidate laws, route rules
- `digital-protocell/crates/chemistry-core/tests/d060_tests.rs` — Route L repro, ledger, drive, causality, route selection tests
- `digital-protocell/crates/experiment-runner/src/d060.rs` — Gates −1…12 shadow/observer pipeline
- `docs/d060_structural_growth_resource_size_feedback.md` — equations, gates, production unauthorized
- `digital-protocell/experiments/generated/d060/` → `/mnt/storage1tb/.../d060`

## Storage archive (1TB)
- `docs/storage_archive_policy.md` — NVMe vs `/mnt/storage1tb` policy
- `.cursor/rules/06-storage-archive.mdc` — always-on agent reminder
- `/mnt/storage1tb/cache/project-artifacts/digital_cell/` — target, cocoindex, experiments/generated archive + manifest

## D-060 structural growth / size feedback
- `digital-protocell/crates/chemistry-core/src/d060_analysis.rs` — ledger, drive surface, neutrality, candidates, route rules
- `digital-protocell/crates/chemistry-core/tests/d060_tests.rs` — Gate classifiers and route selection
- `digital-protocell/crates/experiment-runner/src/d060.rs` — Gates −1…12 shadow pipeline (stops at geometry defect)
- `docs/d060_structural_growth_resource_size_feedback.md` — Route G: structure-constraint freezes φ
- `digital-protocell/experiments/generated/d060/` → `/mnt/storage1tb/.../d060`

## D-061 structure execution repair
- `digital-protocell/crates/chemistry-core/src/d061_analysis.rs` — typed-mode, parity, drive, runaway, and route classifiers
- `digital-protocell/crates/chemistry-core/tests/d061_tests.rs` — mode identity/resume/parity/route regression suite
- `digital-protocell/crates/experiment-runner/src/d061.rs` — Gates 0–11 fixed/dynamic shadow campaign and artifacts
- `docs/d061_structural_constraint_execution_repair.md` — Route G runaway growth after DynamicStructure repair
- `digital-protocell/experiments/generated/d061/` → `/mnt/storage1tb/.../d061`

## D-062 structural maintenance/decay review
- `digital-protocell/crates/chemistry-core/src/d062_analysis.rs` — long-horizon baseline/scaling/candidate/route classifiers
- `digital-protocell/crates/chemistry-core/tests/d062_tests.rs` — decay parity, scalar trend, candidate, route suite
- `digital-protocell/crates/experiment-runner/src/d062.rs` — Gates −1–11 shadow maintenance campaign
- `docs/d062_structural_maintenance_decay_review.md` — Route N: no local maintenance law
- `digital-protocell/experiments/generated/d062/` → `/mnt/storage1tb/.../d062`

## D-063 connected membrane architecture
- `digital-protocell/crates/chemistry-core/src/d063_analysis.rs` — topology flood-fill, geometry families A–E, area/material/bootstrap/route
- `digital-protocell/crates/chemistry-core/tests/d063_tests.rs` — connectivity/area/material/carrier/route suite
- `digital-protocell/crates/experiment-runner/src/d063.rs` — Gates −1–12 shadow pipeline
- `docs/d063_connected_membrane_architecture_review.md` — `D063_CONNECTED_MEMBRANE_SHADOW_REPAIR_FAILURE`
- `digital-protocell/experiments/generated/d063/` → `/mnt/storage1tb/.../d063`

## D-064 connected-geometry coupled failure audit
- `digital-protocell/crates/chemistry-core/src/d064_analysis.rs` — canonical χ, budgets, joint allocator, seeds, route
- `digital-protocell/crates/chemistry-core/tests/d064_tests.rs` — accounting/rejection/budget/seed/route suite
- `digital-protocell/crates/experiment-runner/src/d064.rs` — Gates −1–12 diagnostic shadow pipeline
- `docs/d064_connected_geometry_coupled_failure_audit.md` — `D064_STATIC_COUPLED_RESOURCE_METRIC_DEFECT`
- `digital-protocell/experiments/generated/d064/` → `/mnt/storage1tb/.../d064`

## D-065 canonical resource topology requalification
- `digital-protocell/crates/chemistry-core/src/d065_analysis.rs` — signed net flux evaluator, topology necessity, fate/W/A ledgers, route
- `digital-protocell/crates/chemistry-core/tests/d065_tests.rs` — parity, fate, topology, route tests
- `digital-protocell/crates/experiment-runner/src/d065.rs` — Gates −1–10 shadow pipeline
- `docs/d065_canonical_resource_topology_requalification.md` — completion (`D065_RESOURCE_DELIVERY_SUFFICIENT_ACTIVATION_LIMITED`)

## D-066 activation utilization capacity audit
- `digital-protocell/crates/chemistry-core/src/d066_analysis.rs` — activation lineage, limiter/overlap/redistribution/capacity/catalyst/A-demand classifiers, route select
- `digital-protocell/crates/chemistry-core/tests/d066_tests.rs` — parity, limiter, redistribute, route tests
- `digital-protocell/crates/experiment-runner/src/d066.rs` — Gates −1–12 shadow pipeline
- `docs/d066_activation_utilization_capacity_audit.md` — completion (`D066_FROZEN_ACTIVATION_CAPACITY_LIMIT`)
- `digital-protocell/experiments/generated/d066` → archive symlink

## D-067 activation-capacity law identification
- `digital-protocell/crates/chemistry-core/src/d067_analysis.rs` — observer-only substrate lineage, candidate laws, identification gates, and route selection
- `digital-protocell/crates/chemistry-core/src/d050_analysis.rs` — schema 3 bounded-N/F diagnostic dispatcher (schema 2 remains production path for V13)
- `digital-protocell/crates/experiment-runner/src/d067.rs` — shadow-only Gates −1–10 activation-capacity pipeline and artifacts
- `digital-protocell/crates/chemistry-core/tests/d067_tests.rs` — focused D-067 analysis and route tests
- `digital-protocell/docs/d067_activation_capacity_law_identification.md` — completion report (`D067_NO_PORTABLE_ACTIVATION_CAPACITY_LAW`)
- `digital-protocell/experiments/generated/d067` → archive symlink

## D-068 precursor demand / membrane assembly
- `digital-protocell/crates/chemistry-core/src/d068_analysis.rs` — A/P/S/W ledgers, utility/replacement/assembly classifiers, route selection
- `digital-protocell/crates/experiment-runner/src/d068.rs` — shadow-only Gates −1–15 precursor→membrane audit (uses accepted exchange_net)
- `digital-protocell/crates/chemistry-core/tests/d068_tests.rs` — focused parity/ledger/route tests
- `digital-protocell/docs/d068_precursor_demand_membrane_assembly_audit.md` — `D068_MEMBRANE_DESORPTION_DOMINANT`
- `digital-protocell/experiments/generated/d068` → archive symlink

## D-069 mature membrane exchange
- `digital-protocell/crates/chemistry-core/src/d069_analysis.rs` — frozen P↔S equilibrium, identification, and route classifiers
- `digital-protocell/crates/experiment-runner/src/d069.rs` — shadow-only Gates −1–16 equilibrium/desorption audit
- `digital-protocell/crates/experiment-runner/src/main.rs` — `d069 pipeline` CLI wiring

## D-070 mature-membrane seed capacity contract
- `digital-protocell/crates/chemistry-core/src/d070_analysis.rs` — SEED_CAPACITY_CONTRACT_V1 validator + Policies A–D + route select
- `digital-protocell/crates/chemistry-core/tests/d070_tests.rs` — capacity/migration/route tests
- `digital-protocell/crates/experiment-runner/src/d070.rs` — Gates −1–12 pipeline
- `digital-protocell/docs/d070_mature_membrane_seed_capacity_repair.md` — terminal report
- `digital-protocell/experiments/generated/d070/` — gate artifacts (archived symlink)

## D-071 — Precursor demand regulation
- `digital-protocell/crates/chemistry-core/src/d071_analysis.rs` — PrecursorRegulationParams, derive_m_p/k_i_candidates, maintenance_windows_pass, radius_portable_row, RouteEvidence071, select_route
- `digital-protocell/crates/chemistry-core/tests/d071_tests.rs` — unit tests for d071 analysis helpers
- `digital-protocell/crates/experiment-runner/src/d071.rs` — Gates 0–8 pipeline (control_reproduction→demand_ledger→candidates→accounting→maintenance→repair→causal→portability→stage_e)
- `digital-protocell/experiments/generated/d071/` — gate artifacts (archived symlink → /mnt/storage1tb/...)

## D-071 capacity-bounded precursor demand regulation
- `digital-protocell/crates/chemistry-core/src/d071_analysis.rs` — regulation schema, candidate helpers, route select
- `digital-protocell/crates/chemistry-core/tests/d071_tests.rs` — focused Gate/equation tests
- `digital-protocell/crates/experiment-runner/src/d071.rs` — Gates 0–8 pipeline
- `digital-protocell/docs/d071_capacity_bounded_precursor_demand_regulation.md` — terminal report
- `digital-protocell/experiments/generated/d071/` — gate artifacts (archived symlink)
- Opt-in params: `SimParams.precursor_m_p`, `precursor_product_inhibition_ki` (defaults 1 / 0 preserve constitutive)

## D-072 mature-membrane damage refill audit
- `digital-protocell/crates/experiment-runner/src/d072.rs` — Gates 0–6 artifact pipeline and frozen D-071 repair diagnostics
- `digital-protocell/crates/experiment-runner/src/main.rs` — `d072 pipeline` CLI wiring
- `digital-protocell/crates/chemistry-core/src/d072_analysis.rs` — refill route/timescale/intervention helper contracts

## D-073 mature-membrane equilibrium sufficiency
- `digital-protocell/crates/chemistry-core/src/d073_analysis.rs` — p_required inversion, fixed-P control class, long-horizon class, RouteEvidence073/select_route
- `digital-protocell/crates/chemistry-core/tests/d073_tests.rs` — Gate0–7 focused unit tests
- `digital-protocell/crates/experiment-runner/src/d073.rs` — Gates 0–7 pipeline (diagnostic fixed-P holds)
- `digital-protocell/docs/d073_mature_membrane_equilibrium_sufficiency.md` — terminal report
- `digital-protocell/experiments/generated/d073/` — gate artifacts (archived symlink)

## D-074 cellwise exchange integration parity
- `digital-protocell/crates/chemistry-core/src/d074_analysis.rs` — discrete bath BE, runtime BE, exposure/ceiling, Route T/Q/E/I/M/X
- `digital-protocell/crates/chemistry-core/tests/d074_tests.rs` — focused parity/exposure/route tests
- `digital-protocell/crates/experiment-runner/src/d074.rs` — Gates 0–7 pipeline
- `digital-protocell/docs/d074_cellwise_exchange_integration_parity.md` — report
- `digital-protocell/experiments/generated/d074/` — artifacts (symlink → storage1tb)

## D-075 cellwise exposure-gated membrane requalification
- `digital-protocell/crates/chemistry-core/src/d075_analysis.rs` — E_i observer (`CellExposureState`, FE/BE contraction, `qualify_exposure_capacity`, `classify_long_horizon`, `select_route`)
- `digital-protocell/crates/chemistry-core/tests/d075_tests.rs` — contraction/exposure/route/preservation tests (12)
- `digital-protocell/crates/experiment-runner/src/d075.rs` — Gates 0–8 pipeline; shared `ExposureObserver`; recovery-aware fixed-P stop
- `digital-protocell/docs/d075_cellwise_exposure_membrane_requalification.md` — report
- `digital-protocell/experiments/generated/d075/` — artifacts (symlink → storage1tb)
