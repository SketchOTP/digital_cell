# D-013 Invalid Reference Postmortem

## Preserved artifact

Path: `digital-protocell/experiments/generated/d012/v2_stage_e_reference/`

| Field | Value |
| --- | --- |
| source commit | `15f9f21b68057864b42c140b633aefa7b0952440` |
| binary hash | `b044ac4083838e9ea6e21c32e093f2025f25b10876b26f9b67c917995e28e77d` |
| candidate hash | `9a452d3470be34ccf3bdd7d1397341b64617834e77131cf2899efb327728d626` |
| configuration hash | `87ff7e6e4bd479972c3a02b0de4e6bc94a949041860b32b230e5b28863bb5ad6` |
| result commit | `23f3d99` |
| manifest hash | `d4a7bf88b244761e054af4c12cb17afc39e6d9cad2e83f3451f412df1a744a7a` |

Tag: `D-012-stage-e-reference-invalid`

```text
scientific_usable = false
classification = INVALID_ARTIFACT
```

## Defects

1. Required atomic checkpoints absent.
2. Activation-potential ledger absent.
3. Rejected adaptive attempts entered convergence windows via chunk accounting.
4. False zero-motion windows after accepted progress stalled.
5. Premature termination before 200,000 accepted substeps while `steps_done` still advanced.
6. Clean governed classification unavailable (`clean_termination=false`).

## Root cause

`run_constrained_assay` advanced the outer window counter by the full requested chunk even when `Simulation::step()` returned false after a timestep-floor failure. Subsequent iterations appended duplicate end-state snapshots with zero simulated-time progress.

## Directional diagnostic (not scientific)

Classified as `INVALID_REFERENCE_DIRECTIONAL_DIAGNOSTIC`:

| Metric | Value |
| --- | --- |
| Q_structure | 0.0342457 |
| Q_catalyst | 0.4561964 |
| Q_membrane | 0.2940055 |
| Q_activated | 1.1260094 |
| g_structure | −36.9610 |
| g_catalyst | −0.554489 |
| g_membrane | −0.769932 |
| g_activated | −0.548245 |
| C retention | 0.939302 |
| A retention | 0.744125 |
| membrane localization | 0.899265 |

These may not be combined with a later valid reference.
