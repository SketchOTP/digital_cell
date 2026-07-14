# D-008 Membrane Metabolic Closure Design

## Status

Approved by the project directive on 2026-07-14.

- Project directive: `D-008`
- Agent memory directive: `D-20260714-d008-membrane-metabolic-closure`
- Baseline commit: `bd7d5cfd5ea6a1689feae34f4285e950d61bc21d`
- Baseline tag: `D-007-five-field-model-rejected`
- Branch: `d008-membrane-metabolic-closure`
- Equation version: `membrane_metabolism_v1`
- Phase status remains `PHASE1_SELF_MAINTENANCE_PARTIAL`

## Goal

Replace the rejected five-field protocell with a seven-field system in which internal metabolism produces an activated resource, the activated resource produces catalyst, structure, and membrane, and the self-produced membrane selectively retains catalyst and activated resource while admitting nutrient and fuel and releasing waste.

The seven fixed fields are structure `φ`, catalyst `C`, nutrient `N`, fuel `F`, waste `W`, activated resource `A`, and membrane `M`. None is a health, target, observer, or controller field.

## Architecture

Extend the existing simulation engine rather than creating a second engine or a generic species framework. Retain shared grids, Cahn–Hilliard phase dynamics, reservoir handling, adaptive stepping, validation, double buffering, accounting primitives, diagnostics infrastructure, and deterministic execution.

Use fixed compile-time field storage. Every field has current and next buffers allocated with the simulation. All accepted steps swap all seven pairs together; rejected attempts swap none. No field allocation occurs during a substep.

Dispatch once at the simulation-step boundary by a typed equation version:

- legacy bulk path;
- `surface_turnover_v1` path;
- `membrane_metabolism_v1` path.

Legacy reactions and transport do not pass through D-008 abstractions. D-008 reaction, transport, accounting, diagnostics, and classification remain isolated in dedicated modules.

## Modules

- `activated_metabolism.rs` — activation, activated consumption/decay, catalyst reproduction from `A`.
- `membrane.rs` — membrane synthesis, diffusion parameters, decay, detachment, and localization.
- `membrane_transport.rs` — face membrane density, permeability, face diffusivity, and conservative species fluxes.
- `membrane_accounting.rs` — activated, membrane, and species-transport ledgers.
- `d008_diagnostics.rs` — observer-only retention, localization, flux, turnover, and four-dimensional macrostate metrics.
- `d008_analysis.rs` — stage gates, prescribed-radius balance, fixed-point, basin, controls, and final conclusion.
- `experiment-runner/src/d008.rs` — deterministic staged runner and artifact provenance.

Diagnostics may read simulation state after accepted steps. They may never influence chemistry, transport, timestep selection, or reaction rates.

## Snapshot and Identity Schema

Use a versioned snapshot envelope with explicit five-field and seven-field payloads. Every snapshot and artifact records:

- snapshot schema version;
- field schema version;
- equation version;
- candidate ID and hash;
- configuration hash;
- source commit;
- accepted substeps;
- simulated time.

A five-field snapshot remains readable for a compatible legacy equation. It is rejected for `membrane_metabolism_v1`; missing `A` and `M` are never silently synthesized. A seven-field snapshot is accepted only for a compatible D-008 configuration and round-trips all seven fields.

Canonical candidate hashing includes every D-008 parameter in a fixed order while preserving historical hashes for historical equation versions.

## D-008 Substep

One accepted D-008 substep uses only old-state values:

1. Apply reservoir relaxation to working buffers.
2. Calculate local interior and interface weights.
3. Calculate structural chemical potential and phase-field transport.
4. Calculate old-state face membrane densities and species permeabilities.
5. Calculate conservative face fluxes for `C`, `A`, `N`, `F`, and `W`.
6. Calculate membrane diffusion.
7. Calculate all eight reaction rates.
8. Integrate all seven next buffers.
9. Validate finite values and bounds.
10. Build complete field, reaction, transport, clamp, and reservoir ledgers.
11. Reject and reduce `dt` on instability.
12. Swap every field buffer only after acceptance.
13. Record observer-only diagnostics.

New membrane values never affect soluble transport in the same attempted substep.

## Reaction and Transport Model

Use the eight reactions and unit stoichiometry specified by D-008:

- internal catalyst-dependent activation from `N` and `F`;
- catalyst reproduction from `A`;
- interface structure production from `A`;
- interface membrane production from `A`;
- structure turnover;
- phase-dependent catalyst turnover;
- activated-resource turnover;
- membrane decay and off-interface detachment.

Only `A` directly supports productive catalyst, structure, or membrane chemistry.

For each soluble species, compute face diffusivity from the mean base diffusivity multiplied by:

```text
exp(-β_X × M_face × I_face)
```

Apply membrane permeability to `C`, `A`, `N`, `F`, and `W`, not to `φ`. `M` uses its own low diffusivity. Face fluxes are symmetric and conservative.

## Scientific Stage Gates

Each stage stops later work if its required mechanism fails.

### Stage 0 — Schema and engine scaffold

Add seven fields, typed equation dispatch, versioned snapshots, configuration and identity hashing, accounting structures, and empty D-008 modules. Prove all seven buffers swap, D-008 rejects five-field snapshots, seven-field snapshots round-trip, and legacy behavior remains reproducible.

Commit and tag: `D-008 Stage 0: Add seven-field versioned engine scaffold`, `D-008-stage-0-schema-pass`.

### Stage A — Static planar transport

Implement fixed-membrane conservative transport and transport accounting only. Validate zero-membrane equivalence, monotonic attenuation, selectivity targets, symmetry, and mass conservation at membrane densities 0 through 1.

Use separate source and result commits. Pass tag: `D-008-stage-a-transport-pass`. On failure preserve evidence, tag failure, and conclude boundary retention failure or oversealing from the measured result.

### Stage B — Membrane localization

With fixed `φ`, `C`, and `A`, enable membrane production, diffusion, decay, and detachment. Calibrate membrane production from the prescribed production/loss basis only. Require active turnover and at least 90% localization where `I(φ) ≥ 0.25`.

Pass tag: `D-008-stage-b-localization-pass`. Stop on localization failure.

### Stage C — Zero-dimensional metabolism

Implement activation, activated decay, and catalyst reproduction from `A`. Prove independent `C`, `N`, and `F` requirements, `A` dependence of catalyst reproduction, boundedness, decline without resource input, waste production, and stoichiometric closure.

Pass tag: `D-008-stage-c-metabolism-pass`. Stop on metabolic activation failure.

### Stage D — Fixed compartment

With fixed circular structure and membrane geometry, couple selective transport, activation, catalyst reproduction, turnover, and reservoir exchange at radii 16, 24, and 32. Require `C` and `A` retention at least 0.80, resource entry, waste exit, strictly decreasing resource influx per internal area with radius, and removal of the D-007 small-cell leakage defect.

Pass tag: `D-008-stage-d-fixed-compartment-pass`. Reaction rates may not compensate for failed transport.

### Stage E — Prescribed-radius balance

Enable all reactions with fixed geometry. Calibrate membrane production, activation, catalyst reproduction, then structure production, each through staged 0.8×/1.0×/1.2× screens. Require overlapping zero-flow regions for structure, catalyst, membrane, and activated resource.

Pass tag: `D-008-stage-e-balance-pass`. If no overlap exists, conclude `D008_NO_JOINT_FIXED_POINT`.

### Stage F — Fully coupled pilot

Enable complete seven-field dynamics for only the bounded pilot grid. Require bounded four-dimensional macrostates, a joint restoring tendency, fresh analytic seeds, and no stabilization through extinction, exhaustion, fragmentation, dish contact, or timestep collapse.

Pass tag: `D-008-stage-f-coupled-pilot-pass`. Stop if no candidate has a stable joint fixed point.

### Stage G — Basin and acceptance

Progress through center seed, cardinal neighbors, contiguous basin, noise sensitivity, causal controls, puncture response, and five-seed long acceptance. Do not launch later grids before cheaper gates pass.

Scientific closure receives exactly one pass or failure tag and one permitted D-008 conclusion. Even a D-008 pass leaves Phase 1 partial.

## Preservation and Provenance

Each scientific stage normally has:

1. a clean source commit containing executable code, tests, configs, runner, and schema;
2. governed experiments whose artifacts record that source commit and binary hash;
3. a result commit containing reports, manifest pointer/hash, selected configuration, and memory updates.

Large generated data stays under gitignored `digital-protocell/experiments/generated/d008/`. Small manifests, summaries, configuration files, artifact pointers, and reports are committed. Reruns use immutable `attempt_NNN` directories and record supersession reasons.

A failed stage is preserved and terminates later work. Only demonstrated implementation, accounting, provenance, or numerical defects may be repaired and rerun at that same gate.

## Validation

Use test-driven stage implementation and run focused D-008 tests before each governed experiment. When shared numerical code changes, add focused legacy-equivalence coverage. Before final closure, run all chemistry-core integration, validation, D-003 through D-008 test targets in release mode.

The D-007 baseline test passed on this branch before D-008 engine edits: 26 passed, 0 failed.

## Tooling Limitation

Serena is configured but unavailable for Rust symbol navigation and reported `Active languages: []`. Use semantic code search, `cargo metadata`, targeted `rg`, targeted source reads, compiler diagnostics, filtered Cargo tests, and rust-analyzer tools when available. Do not claim Serena symbol validation.

## Explicit Exclusions

No second simulation engine, dynamic species registry, runtime reaction graph, plugin chemistry, global controller, target radius/mass, observer feedback, additional chemical species, genetic material, mutation, evolution, division, movement, sensing, neural fields, memory, explicit lipid particles, visual polish, or LLM integration.
