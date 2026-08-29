# DC-DEV-020-M1-R6-R1-R1 Cross-Platform Closure Validation

This observer-only validation starts from authority head `1d681694e0dd5334e8267a881a2e1c4bec71324d` and preserves the failed R6-R1 evidence namespace `dcdev020m1r6r1`. The audited scientific entry remains explicit as `adea13fafa1f2a85e521a44b5d77249820d107bd`.

No material-conservation repair is implemented. `MaterialMesh`, mechanics, remesh, rebond, V1 transport, V3 chemistry, resource geometry/inventory/concentration, D-087, D-091, and production selection remain unchanged.

## Portable protocol boundary

`protocol.json` contains scientific inputs only: authority heads, runtime stage order, `dt`, horizon, tolerance, resource geometry/inventory/concentration, and scope flags. Platform, OS, dense-output path, and storage mode are recorded separately in `execution_metadata.json`; execution paths cannot make two scientific protocols unequal.

## Validation gates

- Plain and instrumented replay must have exact trajectory and final-mesh parity within each platform.
- Mechanics-only and remesh-only strict-material changes must be nonzero and match their fixed-concentration area predictions.
- Full geometry residual must match the fixed-concentration prediction while uptake, reaction, and unexplained residuals remain at or below `1e-8`.
- Semantic checkpoints are compared at `0, 1, 10, 100, 480, 1000, 2000, 3466, 4000, 6000, 6931, 8000`.
- Any numeric or discrete remesh divergence is reported diagnostically; Linux and Windows trajectory hashes are not required to match.
- Contact loss must be observed as exposure changing from positive to zero while external resource remains available and after geometry evolution.

## Evidence

Canonical dense evidence is stored at:

`\\atlas\ATLAS\100_ACTIVE\Projects\DIGITAL_CELL\evidence\dcdev020m1r6r1r1\dense\stage_ledger.jsonl`

The original failed R6-R1 namespace remains preserved. Compact evidence is under `digital-protocell/experiments/generated/dcdev020m1r6r1r1/`. Exact-head remote CI and Architect acceptance remain pending until the new verifier runs on Linux.

## Status

```text
M1_RUNTIME_GEOMETRY_MASS_COUPLING_CROSS_PLATFORM_CONFIRMED: PENDING REMOTE VALIDATION
M1: NOT ESTABLISHED
PRODUCTION DEFAULT: UNCHANGED
M2: NOT AUTHORIZED
DC-DEV-021: NOT AUTHORIZED
NEXT_EXECUTION_STARTED: false
```
