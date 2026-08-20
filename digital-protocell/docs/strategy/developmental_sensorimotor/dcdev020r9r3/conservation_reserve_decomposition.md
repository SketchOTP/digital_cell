# DC-DEV-020-R9-R3 — Conservation × Reserve Certification Decomposition

Status: observer-only diagnostic; remote CI passed; architect review pending.

## Authority and scope

- Starting head: `904efc95919f67243681d3512fb7b46e0ab85710`
- Branch: `strategy/dc-dev-020r9-mesh-contract-requalification`
- PR: `#44`
- Matrix authority: the actual D-087 Phase-1 Gates 0–7 certifier
- Frozen axes: `HistoricalV1|ConservativeV2` × reserve `OFF|ON`
- No chemistry, behavior, recycling, tuning, source/sink, transport, or DC-DEV-021 work was authorized.

## Protocol

The four arms were executed with independent selectors:

| Arm | Contract | Reserve |
| --- | --- | --- |
| H0 | HistoricalV1 | OFF |
| V20 | ConservativeV2 | OFF |
| H1 | HistoricalV1 | ON |
| V21 | ConservativeV2 | ON |

H0 was the hard scientific gate. Because H0 passed Gates 0–6, the remaining three arms executed. The local Windows runtime packaging check did not qualify Gate 7 (`bin_ok=false`) for any arm; this is recorded as a runtime qualification result and does not change the scientific Gates 0–6 decomposition. Remote Linux CI is authoritative for final acceptance.

Exact-head remote CI run `32421756950` at head `6a266514fcb616084ea43be42ff726c4c51dec0e` passed all scoped stages, including the full matrix and artifact validation. The compact artifact `dcdev020r9r3-compact-evidence` has SHA-256 `951fb0f5bc79ab70dc2d50d614c3ca43520069eb8a73360817f01951b2ecfbdf`.

## Results

The compact local qualification is in `experiments/generated/dcdev020r9r3/qualification.json`.

- H0: Gates 0–6 pass; `R_m=1.0180981834599838`, `R_b=5.818353471059928`, `R_C=1.446090001246529`.
- V20: Gates 0–6 pass; same scientific metrics as H0.
- H1: Gates 0, 5, and 6 pass; Gates 1–4 fail. Reserve execution recorded A→R `26.594234371143322`, R→A `3.097930599815921`, R→W `0.24408264815689965`, rejects `0`.
- V21: Gates 0, 5, and 6 pass; Gates 1–4 fail. Reserve execution matched H1, with rejects `0`.

The result is:

`DCDEV020R9R3_RESERVE_PHYSIOLOGY_CERTIFICATION_GAP_CONFIRMED`

The decomposition localizes the certification loss to the reserve-enabled physiology: the contract-only comparison V20 remains scientifically equivalent to H0, while both reserve-enabled arms fail the D-087 scientific qualification gates. This does not authorize a reserve repair or production integration.

## Preservation

R9-R2 was not rerun as a campaign. Its compact evidence remains preserved and is checked by the R9-R3 runner: organized retained delta approximately `-10.277547850163131`, C→W approximately `5.29017338017132`, and all four sustained organized-material slopes negative. The preservation predicate is part of the compact artifact validation.

## Governance conclusion

Certified Phase-1 equations and production behavior were not changed. The bounded observer adds orthogonal contract/reserve selectors, reserve execution accounting, the actual four-arm certifier runner, compact evidence, documentation, and scoped CI. Recycling, salvage, DC-DEV-021, parameter tuning, and further biological work remain unauthorized pending architect review.
