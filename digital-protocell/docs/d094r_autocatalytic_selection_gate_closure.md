# D-094R: Autocatalytic Selection Gate Closure

## Purpose

Close Gate 6 (environment-dependent selection) for the distributed autocatalytic-set
substrate before any mutation-adaptation (Gate 7) or environmental-reversal (Gate 8) work.

## Governed stop

The overnight D-094 pipeline was terminated while executing Gate 7 after Gate 6 had not
passed.

- terminated PID: `1509382`
- earlier orphan runner: `1382957` (already killed)
- reason: `DOWNSTREAM_EXECUTION_STOPPED_AFTER_GATE6_NONPASS`

This was a governed stop, not a crash. All existing artifacts were preserved unchanged under
`experiments/generated/d094/attempt_partial_overnight/` and mirrored to
`experiments/generated/d094r/overnight_partial_preservation/` with `ARTIFACT_HASHES.sha256`
and `preservation_record.json` (source commit, binary hash, configuration hash, start/stop
times, completed generations/replicates, termination reason).

Partial Gate 7 and Gate 8 artifacts are marked `UNUSABLE_FOR_SCIENTIFIC_CONCLUSION`.

## Stale conclusion correction

The on-disk primary `D094_AUTOCATALYTIC_SET_IMPLEMENTATION_DEFECT` is stale and rejected.
The root `experiments/generated/d094/manifest.json` now records
`D094_GATE6_SELECTION_CLOSURE_PENDING` with `stale_conclusion_superseded` retained for audit.

Evidence state at correction time:

| Gate | Status |
|---|---|
| 0–5 | pass |
| 6 | incomplete / nonpassing (horizon unmet) |
| 7 | partial, scientifically unauthorized |
| 8 | not validly reached |
| 9 | pass (prior compatible evidence) |

## Frozen gate dependency

```text
Gate 6 selection → Gate 7 mutation adaptation → Gate 8 environmental reversal
```

Enforced in code: when Gate 6 does not pass, `run_selection_gates` writes blocked markers for
Gate 7/8 and returns immediately with `GATE7_BLOCKED_AFTER_GATE6_NONPASS` /
`GATE8_BLOCKED_AFTER_GATE6_NONPASS`. No downstream result can compensate for an upstream failure.

## Checkpoint status

The selection harness never persisted population checkpoints (no complete population state,
node/edge material, positions, A/R, environmental N/F/W, lineage observers, accepted time, or
random state). Resume was therefore impossible:

- `D094_GATE6_CHECKPOINT_INVALID`
- action: rerun **Gate 6 only** from the sealed source, per directive §7.

Recorded in `experiments/generated/d094r/checkpoint_validation/checkpoint_status.json`.

## Execution-defect repair (process locking)

Two simultaneous runner processes were an execution defect. `experiment-runner`
now takes a single-instance lock (`d094_pipeline.lock`) containing PID and source identity:

- refuses startup when a live lock exists;
- clears stale locks only after PID liveness verification;
- `gate6-complete` additionally refuses to start when a live D-094 pipeline lock exists.

This repair changes no biology, ecology, threshold, or generation accounting.

## Gate 6 completion run

`experiment-runner d094 gate6-complete` runs mutation-off H, B, and neutral campaigns only
(8 replicates each), with a preregistered horizon of **8 completed generations** or lawful
ecological termination after at least 4 completed generations. Generation counting uses the
peak generation reached during the run rather than only surviving individuals.

Acceptance (directive §9), unchanged after seeing results:

- per-replicate win requires Δf ≥ 0.15 **and** ≥ 1.20x viable descendants relative to the
  other label, at ≥ 4 completed generations;
- H and B each require ≥ 6 of 8 winning replicates and median valid replicate ≥ 6 generations;
- neutral requires median |Δf| < 0.10, neither label winning more than 5 of 8, and adequate depth.

Artifacts: `selection_h_completion/`, `selection_b_completion/`, `neutral_completion/`,
`gate6_decision/decision.json`, `manifest.json`.

## Routing

| Outcome | Primary conclusion | Downstream |
|---|---|---|
| Gate 6 passes | `D094_AUTOCATALYTIC_SET_SELECTION_QUALIFIED` | Gate 7 authorized; Gate 8 still blocked |
| Valid completion, no selection | `D094_AUTOCATALYTIC_SET_HEREDITY_QUALIFIED_SELECTION_REJECTED` | Gates 7/8 remain blocked; Phase 3 not authorized |
| Cannot complete | `D094_GATE6_CHECKPOINT_INVALID` / `_NUMERICAL_FAILURE` / `_ECOLOGICAL_TERMINATION_INVALID` / `_EXECUTION_DEFECT` | no biological selection result inferred |

No retuning of catalytic efficiency, mutation rate, edge copying, node production, ecological
pressure, generation threshold, or founder networks is permitted in response to results.
