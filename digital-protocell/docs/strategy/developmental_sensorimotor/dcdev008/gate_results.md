# DC-DEV-008 gate results

All nine assay gates pass locally at entry
`2968882769991f48c987ceb40c719fd351b2e046`. The reusable production boundary
is covered by six direct regulatory-core tests for finite inventory, locality,
transfer, conservation, exhaustion, and deterministic replay.

| Gate | Result | Evidence |
|---|---|---|
| 0 authority/scope | PASS | `governance_boundary.json` |
| 1 finite world inventory | PASS | `mass_conservation.json` |
| 2 local mass-conservative uptake | PASS | `mass_conservation.json` |
| 3 existing metabolic coupling | PASS | `metabolic_coupling.json` |
| 4 finite depletion | PASS | `finite_depletion.json` |
| 5 persistence-relevant internal state | PASS | `persistence_and_boundary.json` |
| 6 sensorimotor preservation boundary | PASS | `persistence_and_boundary.json` |
| 7 body/reproduction authority | PASS | `body_and_preservation.json` |
| 8 preservation matrix | PENDING REMOTE CI | workflow evidence |

The primary resource-bearing arm obtains N/F locally and has final retained
`A+R = 1.0341205318977045`, compared with `1.008496516372006` for the
resource-free arm. The finite-depletion continuation reaches zero N/F at step
543 and records no uptake afterward. Maximum per-step conservation error is
zero in the committed assay output.
