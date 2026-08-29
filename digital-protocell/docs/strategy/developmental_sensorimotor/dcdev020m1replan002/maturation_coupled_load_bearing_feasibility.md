# DC-DEV-020-M1-REPLAN-002

## Maturation-coupled load-bearing feasibility

This is an observer-only diagnostic starting from `92075021ae1f4c9917f7ace9b160e5694e001de2`. It does not change the production selector, serialized organism, chemistry coefficients, mechanics equations, resource schedule, reserve, recycling, salvage, M2, or DC-DEV-021 behavior.

The diagnostic preserves the accepted REPLAN-001 age ledger: existing structural material starts mature, new structural build enters a young pool, maturation is `min(M_young, k_turn * M_young * dt)`, and ordinary turnover acts only on mature material. The physical edge material is always `M_young + M_mature`.

The D arm adds one diagnostic load-bearing reference. For each intact edge, `reference_length = M_mature / rho_s`, using only the existing `1e-15` numerical division guard when mature material is zero. This reference is used for stretch-force rest length, structural-build strain, and structural-turnover strain. It is not used as a physical material amount, rupture threshold, remesh identity, or target geometry.

## Arms and checks

- A: frozen-geometry current production control.
- B: moving current-production control.
- C: moving age-structured turnover shadow.
- D: moving maturation-coupled load-bearing shadow.
- B, C, and D each receive a 480-step no-resource deprivation followed by the same sealed source schedule without resetting state.
- D also receives the fixed 150,000-step zero-resource starvation continuation and explicit damage fixture.

The source schedule is generated once by A and reused unchanged by B, C, and D. All compact evidence is versioned in Git for CI; dense step ledgers belong under the canonical Atlas evidence root `\\atlas\ATLAS\100_ACTIVE\Projects\DIGITAL_CELL\evidence\dcdev020m1replan002\`.

The current local result is provisional until exact-head Linux CI and Architect review. It records the controls, candidate state, material/age identity, closure, recovery, starvation, topology outcome, D-087 preservation, and the distinction between candidate recovery and the current-control recovery result.

## Boundary

No production scientific code is authorized by this diagnostic. A positive feasibility result does not establish M1, select the candidate for production, authorize a production repair, or authorize M2. The next execution remains false pending Architect disposition.
