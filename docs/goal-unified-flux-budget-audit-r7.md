# Goal-mode unified flux-budget and preservation audit R7

## Purpose

This increment compares the sealed whole-membrane reproductive reference with
the best current integrated spatial post-fission path on common checkpoints.
It is an observer/replay audit only. No new organism-world material-flow
mechanism, coefficient, threshold, gate, controller, or transport layer is
implemented.

The ledger follows the same causal order:

```text
environmental N/F available
→ environmental N/F transferred
→ retained assimilation material
→ environmental N/F processed
→ A produced
→ W produced
→ A maintenance cost
→ A active-work cost
→ A reaching growth
→ young structural material
→ mature structural material
→ total structural mass
→ 1.35 × birth mass
→ physical fission
```

## Authority and comparison boundary

The audit starts from `c51e5d997099b8cac703b3d0345ebf39cab729b5`, the exact
goal-agent R6 post-fission ecological replay. The comparison reference is the
accepted whole-membrane reproductive evidence under
`dcdev021m2closure003r1/whole_membrane_reference.json`.

The spatial arm is replayed after the existing unforced first fission. The
transfer-disabled arm is an otherwise identical control. Checkpoints are
steps 1, 250, 350, and 12000. The existing `1.35 * birth_mass` gate is not
changed.

## Result

The first common checkpoint already diverges at environmental transfer: the
spatial arm receives finite N/F, but its delivered amount is lower than the
whole-membrane reference. This is descriptive ledger evidence, not a new
acceptance threshold. Processing and growth remain positive in the active
spatial arm, while the disabled control receives no environmental material.

The legacy whole-membrane evidence does not retain environmental provenance
after transfer into bulk N/F. Therefore environmental processing, A/W
production, maintenance cost, and growth attribution for that reference are
reported as unavailable rather than inferred from total reaction counters.
That provenance gap is itself preserved in the evidence and prevents a false
claim that the two paths have an equivalent intracellular ledger.

The audit classification is:

`GOAL_AGENT_PROVISIONAL_FLUX_LEDGER_FIRST_DIVERGENCE_IDENTIFIED`

Assimilation remains `INVESTIGATE_NOT_ACCEPTED`. Resource-causal fission,
three generations, heritable ecological phenotype, evolution, learning,
individuality, unattended persistence, and standalone-lifeform completion
remain `NOT_ESTABLISHED`.

## Preservation boundary

The new persistent assimilation fields are now included in the existing
`physical_runtime_valid()` finite-state guard. Legacy serialized chemistry
without those fields still defaults them to zero. Focused D-088, D-091,
geometry-conservation, and runtime checkpoint tests are required by the
scoped workflow. The repository-wide governance validator remains a known
pre-existing baseline failure and is not represented as a pass.

## Stop condition

Do not implement another spatial field, reserve, assimilation, buffer,
conversion, active-work, or material-allocation variant from this audit. The
next architecture must be selected only after the preserved first divergence
and the legacy environmental-provenance limitation are reviewed.
