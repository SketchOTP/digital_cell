# R13 operator graph and prospective analysis

This contract is adopted before R13 outcomes. It specifies the existing operator and how discarded-clone responses are attributed. The executable parameter seal and verifier are authoritative for exact machine settings.

| Stage | Existing source owner | State and units | Order and locality |
| --- | --- | --- | --- |
| Persistent pools | `regulatory-core/src/polarity_mass.rs::PolarityMassStateV1` | Active/inactive polarity amounts per physical edge; accepted-step counter | Read from the complete pre-polarity boundary; clone before each intervention. |
| Numerical staging | `PolarityMassStateV1::advance` and `advance_with_load_bearing_strain` | dt=0.01 time units, four substeps of h=0.0025 | Each native substep applies transport before reaction. R9 strain input is absent. |
| Conservative lateral transport | `polarity_mass.rs::diffuse_once` | Coefficients0.1/1.0 length squared/time; concentration=amount/current edge length; amount flux | Adjacent-edge concentration differences divided by their mean edge length; donor limitation preserves nonnegative pools and total amount. |
| Local conversion | `advance_with_load_bearing_strain` reaction loop | Active/inactive concentration, concentration/time rate, amount transfer | Post-transport local concentrations; activation `(0.01+u*u)*v`, deactivation `(0.1+0.01*u*u)*u`; transfer bounded by the two local pools. |
| Energy and positivity | Same reaction/transaction owner; finite budget supplied by caller | A cost0.5 per absolute converted polarity amount; equal W credit | Funding checked after each substep; positivity/finite-state checks precede final commit. |
| Atomic commit | Same transaction owner | Final active/inactive amounts, one accepted native interval | Scratch arrays replace persistent arrays only after the entire transaction succeeds. Rejected transactions do not mutate the source. |
| Geometry intervention | Existing `examples/dcfinal001_r4_evolution.rs::r8_normal_harmonic_basis` and `r8_perturbed_cohort` | Displacement=mean edge length times0.001 or0.0005, both signs | Same rotation-covariant R8 basis, rigid components removed; geometry changes only on discarded clones. |
| Local activity phase | `regulatory-core/src/polarity_actuation.rs::derive_local_activity` | Dimensionless clipped excess over the equation-derived homogeneous active reference at total concentration0.8 | Adjacent edge concentration average after native polarity advance; independently reconstructed for the exact historical R8 damping predicate. |

## Isolated paths and interaction

FULL calls the unchanged native advance. REACTION_ONLY uses a cloned parameter object with both diffusion coefficients zero. TRANSPORT_ONLY uses a cloned parameter object with the four reaction-rate coefficients zero. Actual-measure and baseline-measure cuts use the same perturbed cohort and finite A budget. Every path begins with the same persistent polarity amounts.

Both alternative substep orders are measured prospectively: four h-sized reaction/transport pairs on discarded states, using native cut advances with one substep. The artificial counters in these probes are discarded. Native transport-then-reaction pool/energy parity with FULL is required.

E1 observes concentrations at fixed amounts; its amount response is zero. E2 measures active AMOUNT response. Its interaction remainder is FULL minus REACTION_ONLY minus TRANSPORT_ONLY; the identity amount response is exactly zero. No additivity assumption is imposed.

## Attribution rule

The independent verifier first reproduces the exact historical R8 plus-sign, first-scale, dominant-activity-harmonic predicate, including its valid-mode and strict sign-symmetry selection. Richer E3 central-response vectors do not replace that predicate.

E3 concatenates all sealed harmonic/phase active-amount responses. Half-scale responses are multiplied by two for comparison at the same physical amplitude. Each numerical envelope is constructed from sign-even error, half-scale discrepancy and replay tolerance; interaction uncertainty is the sum of its constituent uncertainties. FULL direction uncertainty is propagated into projected contributions. If FULL's direction is unidentifiable, no component is nominated.

A component is nominated only when its damping phase agrees beyond uncertainty, its conservative lower projected contribution exceeds the summed upper absolute contributions of the other components, and its response residual is uniquely smallest beyond the combined error envelopes. Correlation, signed projection, gain and normalized residual are retained; no universal correlation cutoff is imported. A component need not perfectly equal FULL.

At least24 of the thirty exact states must be usable and uniquely nominate the same route for a dominant route-level classification. A mixed classification requires at least24 unambiguous individual nominations spanning routes. Otherwise the result is inconclusive. Explicit non-smooth branch/sign exclusions are recorded; missing state identity, corrupted ledger or nonfinite evidence is an implementation/evidence defect, not a favorable exclusion.

All settings are committed and sealed before execution. Neither endpoint morphogenesis labels nor reproductive outcomes select the operator, parameters, perturbations, envelope or classification. R13 ends with one bounded architecture consequence and no automatic successor.
