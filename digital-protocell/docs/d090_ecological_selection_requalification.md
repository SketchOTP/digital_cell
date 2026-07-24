# D-090: Ecological Timescale Repair and Natural Selection Requalification

## Entry

- Branch: `phase2-growth-division-inheritance`
- D-089 seal commit: `6d363a7`
- Tag: `D-089-natural-selection-not-established`
- Records: `D089_HEREDITY_AND_PHENOTYPE_QUALIFIED`, `D089_SELECTION_RESULT_PROVISIONAL_PENDING_ECOLOGICAL_TIMESCALE_AUDIT`
- Hypothesis under test: `EARLY_FISSION_PRECEDED_SELECTION_PRESSURE`
- Organism frozen (σ=0.15, μ=0.01, D-086/D-088 biology unchanged)

## Ecology-only changes

- Spatial shared dish (`spatial_shared_dish.rs`) with diffusion and local uptake
- Founder preconditioning with σ=0 / μ=0 maintenance, birth_mass reset at transfer
- Compact competition dish (8×8, dx=2.5, volume≈400) so inventory→concentration is organism-usable
- Timescale observers and Gate-3 contracts (`ecological_timescales.rs`)

## Gate results

| Gate | Result |
|------|--------|
| 0 D-089 reproduction | PASS |
| 1 Shared-dish harness + pairwise interference | PASS (after uptake/diffusion repair) |
| 2 Founder preconditioning / reserve control | PASS (matched ≤5%; measured reserve growth <10% fission need) |
| 3 Ecological timescale contract / bounded H,B ID | **FAIL** |
| 4–10 | Not reached |

## Gate 3 evidence (demand-derived candidates)

Demand (compact dish): `M≈46.4`, `G≈1.07e4` (calibration floor/overcount under long rich run), `T_f` median fallback `150`.

| Candidate | Viability | t_limit | t_growth10 | t_fission | frac_post | Contract |
|-----------|-----------|---------|------------|-----------|-----------|----------|
| H 1.05M | 1.0 | 93 | — | — | 0.54 | FAIL (<0.80 post-transfer A; no fission after scarcity) |
| H 1.15M | 1.0 | — | — | — | 0.55 | FAIL (no scarcity) |
| H 1.25M | 1.0 | — | 57 | — | 0.56 | FAIL (growth before scarcity) |
| B 5–10% | 0.75 | — | 4.2 | 11.7 | 0.97 | FAIL (viability <0.80; fission before damage at 0.20 T_f) |

Interpretation: under the frozen organism, either scarcity never binds before growth, post-transfer activation never dominates inherited A, or fission occurs before the intended damage schedule. This independently supports `EARLY_FISSION_PRECEDED_SELECTION_PRESSURE` as an ecology/timescale incompatibility, not a failed selection campaign under a valid contract.

## Primary conclusion

`D090_VALID_SELECTION_ECOLOGY_NOT_ESTABLISHED`

Do **not** claim selection failure. Do **not** reject the `C_H`/`C_B` trait on selection grounds. The frozen organism cannot be placed in a shared-dish ecology that simultaneously keeps founders viable and applies the intended resource/repair pressure **before** reproduction under Gate 3.

## Phase status

- Phase 2 physical reproduction: still qualified
- Phase 2 heredity: still partial (`D089_HEREDITY_AND_PHENOTYPE_QUALIFIED`)
- Phase 3: **not** authorized
- `next_execution_started`: false

## Next architecture review

Organism–environment resource coupling must be redesigned so reproductive timescale and ecological scarcity/repair timescale can coexist. Options remain open among catalytic network topology, bonded complexes, or template polymers — but only after a valid ecology exists.

## Product

`RESEARCH_PROGRAM_ACTIVE_FINAL_PRODUCT_NOT_READY`
