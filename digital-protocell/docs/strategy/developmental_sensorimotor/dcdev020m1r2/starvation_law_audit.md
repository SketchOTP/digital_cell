# DC-DEV-020-M1-R2 — starvation-law audit

## Authority

- Directive: `DC-DEV-020-M1-R2-STARVATION-LAW-AUDIT-001`
- Entry head: `7bb48874771144795a9559f7570f5ebc77e1004a`
- Branch / PR: `strategy/dc-dev-020r9-mesh-contract-requalification` / PR #44
- Execution: observer-only; no production repair is authorized.

## Question

M1-R1-R1 established that finite N/F conversion capacity is sufficient over
the frozen 480-step challenge when the starvation decay confound is
neutralized. This audit asks whether the production fourfold activated-A
decay multiplier is necessary for causal starvation deterioration and
irreversible physical failure.

The matched arms differ only in the declared `ReactionParams.k_a_decay`:

| Arm | Resource delivery | Declared K | Frozen starvation multiplier | Effective starvation K |
| --- | --- | ---: | ---: | ---: |
| A production | none | 0.008 | 4 | 0.032 |
| B ordinary-decay shadow | none | 0.002 | 4 | 0.008 |
| C fed control | accepted finite-resource reference | 0.008 | existing production branch | diagnostic reference |

No resource quantity, uptake, transport, chemistry-core source, ConservativeV2
contract, death semantics, D-087 code, reserve, recycling, or downstream
capability is changed.

## Provenance finding

The fourfold branch first appears with the D-086 mesh reaction implementation
at commit `20e9f7814020ca38ed1893fdd94fb3264307de2e`, under
`D-20260724-d086-autopoietic-material-mesh-protocell`. Its source comment says
only that A loss is accelerated when activation substrates are absent. The
repository does not contain an explicit quantitative scientific rationale for
the coefficient. D-087 explicitly qualifies starvation/death behavior but
does not independently require a fourfold A-decay term.

The term is therefore frozen production behavior for this audit, not a newly
qualified universal starvation law. External biology remains reference-only;
no external coefficient is imported.

## Measurement boundary

Each no-resource arm runs the exact accepted M1-R1-R1 entry state for 480
accepted reaction steps, then continues for up to 20,000 additional accepted
steps. The runner records A, C, structural M, bound/free membrane, waste,
organized and strict material, observer viability, rupture count, cumulative
A decay, structural production/turnover, catalyst production/turnover,
starvation observer timing, and non-starvation physical failure timing.

The run does not stop merely because the reversible `starvation_collapse`
observer predicate becomes false. Restoration is attempted only after an
existing irreversible physical condition, using the same finite N/F condition
without restoring organism state, seed, or topology.

## Current result

The exact local audit reaches the full 20,000-step continuation in both arms.
Both arms lose observer viability through the existing starvation predicate,
but neither reaches mesh rupture, catalytic/structural physical collapse, or
invalid runtime geometry within the authorized bound. The current
classification is therefore:

`M1_STARVATION_LAW_AUDIT_INCONCLUSIVE`

This is a bounded diagnostic result, not authorization to change production
starvation behavior. Fresh actual D-087 certification is a separate required
preservation stage and must remain 8/8.

## Artifacts

Compact authoritative artifacts are under
`experiments/generated/dcdev020m1r2/`. Dense per-step ledgers are not
committed. The scoped workflow regenerates the audit, verifies the committed
artifacts, runs actual D-087, and runs Phase-1, D-091, D-088, and
evolution-harness preservation checks.

## Stop boundary

No source-law implementation, production parameter change, recycling/salvage,
M2, behavior, evolution, or DC-DEV-021 work begins from this result.
