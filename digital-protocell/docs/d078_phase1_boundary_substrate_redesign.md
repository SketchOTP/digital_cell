# D-078 — Phase 1 Boundary Substrate Redesign Downselect

## Conclusion

`D078_CONTINUUM_BOUNDARY_SUBSTRATE_REJECTED` (Route N)

## Records

- `CURRENT_P_S_BOUNDARY_ARCHITECTURE_CLOSED`
- `ENERGY_DRIVEN_SURFACE_STATE_CYCLE_REJECTED` (from D-076)
- `PASSIVE_EXCHANGE_KINETICALLY_VALID_METABOLICALLY_UNREACHABLE` (from D-075)

## Mission

Architecture review only. After failure of the P/S membrane lineage (D-077 Route P), compare exactly two minimal continuum boundary substrates:

1. **Candidate A** — structure-native phase boundary (`φ` interface as seal)
2. **Candidate B** — single conserved amphiphile field `M` with explicit free energy

No production chemistry change. No Stage E claim. No Stage F. No explicit lipid particles.

## Entry

| Item | Value |
|------|-------|
| Branch | `d008-membrane-metabolic-closure` |
| Start | `5026f9f` / tag `D-077-cooperative-surface-condensation-review` |
| D-077 | `D077_COOPERATIVE_COHESION_NOT_PORTABLE` |
| Stage E | `BLOCKED_NOT_RECOVERED` |

## Candidate A — Structure-Native Boundary

\[
I_\phi = I(\phi,\lvert\nabla\phi\rvert),\quad
D_{X,\mathrm{face}}=D_X\exp(-\beta_X I_{\phi,\mathrm{face}})
\]

- No P/S fields; no membrane precursor demand
- Current structural production/turnover and dynamic execution retained
- Metabolic A cost of maintaining `φ` included

## Candidate B — Single Conserved Amphiphile

\[
\partial_t M=\nabla\cdot(L_M\nabla\mu_M)+R_M
\]

with \(\mu_M\) from local free energy (bulk mixing + \(\phi\) affinity + M cohesion + \(\lvert\nabla M\rvert^2\)).

- Mathematically distinct from D-021/D-022 affinity-flux implementations
- No precursor/mature split; damage may convert local M→W
- A cost of producing M included

## Gate results

| Gate | Candidate A | Candidate B |
|------|-------------|-------------|
| 0 Lineage novelty | **PASS** | **PASS** |
| 1 Conservation / local causality | **PASS** | **PASS** |
| 2 Coupled feasibility (R16/22/32) | **FAIL** — optimistic A after removing precursor still ≤ D-067 ordinary ceiling ≈0.355 ≪0.80; C stays below gate | **FAIL** — measured A collapsed; M production reintroduces material A sink |
| 3 Structural restoring | **FAIL** — D-061/D-062 universal growth; boundary≡φ | **FAIL** — same current φ kinetics; M does not create size nullcline |
| 4 Boundary function | **FAIL** — A/C retention | **FAIL** — A retention (algebraic C seal can pass alone) |
| 5 Repair / replacement | **FAIL** — no distinct molecular boundary material | Repair algebra OK / starvation OK; overall science still fails via Gates 2–4 |
| 6 Complexity | Lower (preferred if both passed) | Higher (CH-like stiffness) |

## Route selection

Neither candidate passes the scientific gates. Candidate B would resolve A's missing distinct membrane material, but still fails energy retention and structural restoring under frozen evidence. Prefer-A complexity rule does not apply.

## Scientific conclusion

The simplest continuum organizations that metabolism can maintain — a structure-native `φ` seal, or one conserved free-energy amphiphile — do not recover Phase 1 boundary function under the frozen D-075…D-077 metabolic states and D-061/D-062 structural kinetics. Closing further P/S rate/species tuning is confirmed. Continuum membrane elaboration should stop pending an operator decision on Phase 1 scope before particle or edge-network membranes.

## Status

- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Production: `REQUIRES_REMEDIATION`
- Production biology: unchanged

## Next directive

Do **not** implement either candidate. Do **not** add continuum membrane rates or species. Formally close the current D-008 continuum boundary lineage. Prepare operator decision on revising Phase 1 scope before considering explicit particle or edge-network membranes.

`next_execution_started`: false

## Evidence

- `chemistry-core/src/d078_analysis.rs`
- `chemistry-core/tests/d078_tests.rs`
- `experiment-runner/src/d078.rs`
- `experiments/generated/d078/`
