# DC-DEV-020-R8-R5-R1 — Corrected net A/C allocation drift

This is an observer-only requalification of the R8-R5 local allocation result.
It starts from R8-R5 evidence head
`95247dfc6b2b0e9903338e2b76ee55c08f502f84` and reuses the exact 200 deferred
and 200 shared checkpoint states produced by the sealed R8-R5 machinery.

For each candidate partition, the observer records the incoming organism energy,
the immediate A↔C repartition contribution, the exact one-step R8-R5 reaction
contribution, and their sum:

```text
E_actual = area * (A_actual + R_actual)
ΔE_NET = (E_partitioned - E_actual) + (E_after - E_partitioned)
```

Two independent envelopes are evaluated with the R8-R5 65-point mesh and
deterministic refinement: reversible `0 <= C' <= A+C`, and forward-only
`C_actual <= C' <= A+C`. The unchanged `C'=C_actual` control is retained in
every state. No target or new production law participates in allocation.

The local result is:

```text
DCDEV020R8R5R1_RECYCLING_ONLY_LOCAL_CAPACITY
```

All 400 states retain a nonnegative reversible NET envelope, while all 400
forward-only envelopes are negative. The successful reversible optima require
C→A recovery, so this result does not establish forward A→C allocation
capacity and does not authorize catalyst recycling, a dynamic allocator, or
DC-DEV-021.

The prior R8-R5 dense ledger is sealed at
`afa9c26f8845f9321450ec12e7e4fe55dc54a088eb6857ff8e1e272dddc8c390`. It does
not contain a deferred checkpoint-hash field; R1 records deterministic hashes
reconstructed from the exact sealed replay and reports that limitation
explicitly.
