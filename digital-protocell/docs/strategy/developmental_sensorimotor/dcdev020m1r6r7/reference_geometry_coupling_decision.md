# DC-DEV-020-M1-R6-R7 — Reference-Geometry Coupling Decision

## Authority

- Directive: `DC-DEV-020-M1-R6-R7-REFERENCE-GEOMETRY-COUPLING-DECISION-001`
- Starting head: `821f6a85c1d4825715090c8ccb3482ceddccbde5`
- R6-R6 authority: `M1_SOURCE_FRONTLOAD_AND_GEOMETRY_STRUCTURAL_CYCLE_CONFIRMED`
- Scope: observer-only diagnostic; no production selection or biology repair.
- Dense ledgers: `\\atlas\ATLAS\100_ACTIVE\Projects\DIGITAL_CELL\evidence\dcdev020m1r6r7\`

## Question and method

The diagnostic tests whether the instantaneous production relation `M -> rest_length = M / rho_s` is itself sufficient to explain the moving-body homeostasis failure. The current production arm is replayed unchanged. A separate shadow keeps a persistent per-edge reference-length vector initialized from the production rest lengths; split mapping halves the parent reference and merge mapping sums the parents. The reference vector is diagnostic state only and never changes the production mesh or serialized organism state.

Arms use the sealed R6-R6 successful source schedule: frozen static geometry, current moving production, and moving reference-decoupled shadow. The same no-reset deprivation/recovery challenge is run for the current and reference arms. D-087 V2 and V3 preservation reports are generated independently.

## Local result

| Measure | Static frozen | Current moving | Reference-decoupled moving |
| --- | ---: | ---: | ---: |
| Organized-material delta | `+0.342140676890381` | `-9.95495920654304` | `-7.90146640791825` |
| N delivered | `162.464640538382` | `162.464640538382` | `162.464640538382` |
| Maximum strict residual | `3.41e-13` | `2.84e-13` | `2.84e-13` |
| Recovery deficit reduced | n/a | no | yes |

The current moving reference reproduces the accepted R6-R6 moving result and the static arm reproduces the sealed positive reference. Persistent reference lengths improve the recovery outcome, but the moving shadow remains materially negative over the diagnostic horizon. Therefore the reference-geometry coupling is not sufficient to explain the full homeostasis failure.

Classification:

```text
M1_REFERENCE_GEOMETRY_COUPLING_NOT_SUFFICIENT
```

## Preservation and boundaries

- R6-R6 reproduction: pass.
- Geometry/material closure: pass.
- V2 D-087: 8/8; V3 D-087: 8/8.
- Explicit split/merge reference-lineage fixture: pass; the actual sealed trajectory required no remesh operation.
- Production biology, coefficients, transport, resource schedule, production selection, controller, recycling, salvage, M2, R6-R8, and DC-DEV-021: unchanged/unauthorized.
- Exact-head CI and Architect review remain required before this result is authoritative.
- `NEXT_EXECUTION_STARTED:false`; if the remote result confirms this classification, the authorized microarchitecture line terminates and returns to Architect rather than advancing to R6-R8/R9.
