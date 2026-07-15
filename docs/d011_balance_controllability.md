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
| `PASS_AFTER_D011` | Joint overlap under coupled assay; Stage E failure was model-undercoupling |
| `D011_JOINT_BALANCE_PASS` | Pass without revising Stage E classification label |
| `D011_TRANSPORT_COUPLED_NO_SOLUTION` | No overlap after replay, horizons, and bounded solver |
| `D008_NO_JOINT_FIXED_POINT` | Prior Stage E conclusion; revised only if `PASS_AFTER_D011` |
