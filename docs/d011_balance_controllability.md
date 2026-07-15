# D-011 Balance Controllability

## Question

Can the Stage E calibrated rate vector admit a joint fixed point once transport coupling
and membrane evolution are restored while φ remains prescribed?

## Stage E vs D-011

| Aspect | Stage E | D-011 |
| --- | --- | --- |
| φ | Fixed | Fixed |
| Transport | None | Old-state selective |
| Reservoir | None | Yes |
| M | Prescribed | Evolves |
| Structure | Instantaneous on fixed fields | Virtual (A/W only) |
| Time averaging | None | Quasi-steady windows |

## Controllability signals

1. **Sensitivity rank** — full rank suggests local adjustability; rank-deficient → structural degeneracy.
2. **Condition number** — ill-conditioned Jacobian → fragile tuning.
3. **Bounded solver** — if corrections stay inside bounds but |g| remains large, coupled network may lack a fixed point.
4. **Horizon sensitivity** — distinguishes slow convergence from true non-existence.

## Outcomes (§26)

| Conclusion | Meaning |
| --- | --- |
| `D011_TRANSPORT_COUPLED_JOINT_BALANCE_PASS` | Joint overlap under coupled assay; Stage E failure was model-undercoupling |
| `D011_TRANSPORT_COUPLED_JOINT_BALANCE_PASS` | Pass without revising Stage E classification label |
| `D011_TRANSPORT_COUPLED_BALANCE_NO_SOLUTION` | No overlap after replay, horizons, and bounded solver |
| `D008_NO_JOINT_FIXED_POINT` | Prior Stage E conclusion; revised only if `D011_TRANSPORT_COUPLED_JOINT_BALANCE_PASS` |


## Corrected four-rate sensitivity

`attempt_017` reports a 4×4 sensitivity matrix at R=22 with rank 4 and condition number ≈9.04. The authorized productive rates are locally controllable in the quick assay, but the replay state remains far from joint overlap and non-converged. Longer replay from `attempt_015` also remained non-converged with persistent structure and membrane deficits.
