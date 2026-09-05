# Goal-mode unified flux budget R10

Status: `GOAL_AGENT_PROVISIONAL_FLUX_LEDGER_FIRST_DIVERGENCE_CONFIRMED`

This is a current quantitative audit of the qualified whole-membrane
reproductive reference versus the best current integrated spatial path. It
does not add a material pool, transport law, allocation rule, active-work
formula, fission criterion, or organism behavior.

## Largest unresolved end-goal gap

The remaining causal chain is:

```text
finite spatial environmental material
→ adequate local transfer
→ endogenous processing
→ sustained structural growth
→ resource-causal physical fission
```

The current tested spatial contract remains below the reproductive-throughput
trajectory. Assimilation preservation passed, but assimilation is still
`INVESTIGATE_NOT_ACCEPTED` as project architecture.

## Common ledger result

The active spatial and transfer-disabled controls were replayed on the current
runtime at steps `1`, `250`, `350`, and `12000`. The sealed whole-membrane
reference was compared at the common checkpoints where available.

| checkpoint | reference N transfer | spatial N transfer | spatial N processed | environmental A produced | spatial structural mass | fission |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 5.2197561097164 | 0.5735320665105335 | 0.0000009196774391 | 0.0000009196774391 | 1454.868037089404 | 0 |
| 250 | 1379.4128503621268 | 83.42844009092562 | 1.549509643625439 | 1.549509643625439 | 1487.1220002485436 | 0 |
| 350 | 717.4064381138005 | 112.36181502780286 | 3.7049081842438993 | 3.7049081842438993 | 1486.7319304311868 | 0 |
| 12000 | unavailable after reference divergence | 1292.5610559030094 | 1255.2096180495673 | 1255.2096180495673 | 137.78241996842192 | 0 |

The transfer-disabled control received zero environmental N/F and produced no
environmental processing or fission. World N/F conservation error was zero in
the active replay.

The first measured divergence is therefore environmental N/F transfer at step
1. Processing and growth are downstream of that divergence, not the first
observed loss. The spatial replay also begins with `2043.3859906526618` N/F
available versus `8192` in the two-daughter whole-membrane reference, and the
two paths do not expose equivalent boundary or inventory contracts.

## Provenance boundary

Environmental assimilation-produced A is recorded. Environmental-provenance W
is not separately recorded by the current runtime and is intentionally emitted
as unknown. Total organism A/W are reported separately; they are not credited
to environmental material without a provenance ledger.

This prevents the audit from turning an A production counter into an inferred W
production or from treating total internal chemistry as environmental output.

## Architecture decision

The current evidence supports a Route-B material-flow replan, but does not yet
select or implement a successor organism-world mechanism. The next contract
must be source-justified and specify, before runtime execution:

1. finite ownership and exact environmental debit;
2. the local spatial boundary reaching the organism;
3. arrival, processing, and structural-incorporation order;
4. preservation of unchanged reaction, growth, and fission laws;
5. transfer-disabled and inherited-growth controls;
6. checkpoint, remesh, fission, lineage, rotation, and validity preservation;
7. a stop condition that excludes another field-placement, buffer,
   allocation, assimilation, or active-work search.

No new runtime execution is authorized by this audit. If a source-justified
contract cannot be written without an unledgered source, another unmotivated
intermediate, or frozen-law changes, the current organism-world material-flow
architecture is insufficient and local implementation stops.

## Authority boundary

- Result head: `465ff8000e47d34f5dd0133e10d7ec31e09c810b`
- Goal-agent status: provisional only; independent Architect acceptance is not claimed.
- Resource-causal reproduction: `NOT_ESTABLISHED`.
- Assimilation architecture: `INVESTIGATE_NOT_ACCEPTED`.
- CLOSURE-006 through CLOSURE-014: closed; not reopened.
- Next execution: not started.
