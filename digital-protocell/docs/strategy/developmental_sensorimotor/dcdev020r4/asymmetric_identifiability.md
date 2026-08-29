# DC-DEV-020-R4 asymmetric two-substrate identifiability

## Authority and boundary

R4 starts from architect-accepted R3 head `2f32cd40e62c8874d14dfe5aa98d1837c890547f`. Production biology remains rooted in clean DC-DEV-016 head `1e242f28152797b512e25cd56c7b718e45d6ca97`. The assay is observer-only: it does not change or invoke a production candidate law.

R3 established material bilinear low-substrate suppression but could constrain only `V_max/K_S^2` on a symmetric `N=F` trajectory. R4 tests whether fixed asymmetric resource compositions break that degeneracy.

## Frozen probes

With `M = 19.878372106390554`, the exact probes are P0 `(M,M)`, P1 `(2M,M)`, P2 `(M,2M)`, P3 `(4M,M)`, and P4 `(M,4M)`. Body, location, radius, transport, timestep, 5,000-step settlement, 480-step deprivation, and 480-step feeding horizon are unchanged. The limiting paired inventory remains `M` in every probe.

Each probe runs the existing bilinear source, a deterministic constant-gain break-even root, and the existing source-saturated upper-bound observer. All five remained finite and nonnegative, conserved resource transfer exactly within the recorded tolerance, delivered paired substrate, found a finite root, and kept the constant source within the source-saturated envelope.

| Probe | Break-even gain | Baseline final E_stored | Break-even final E_stored | Source-saturated final E_stored |
|---|---:|---:|---:|---:|
| P0 | 13.9482421875 | 54.3584702923158 | 60.82782595938608 | 61.68434818478833 |
| P1 | 4.765045166015625 | 55.70460111106059 | 60.82783645605774 | 61.68434818478833 |
| P2 | 4.765045166015625 | 55.70460111106059 | 60.82783645605774 | 61.68434818478833 |
| P3 | 2.0837860107421875 | 57.91265618036632 | 60.82782435986802 | 61.68434818478833 |
| P4 | 2.0837860107421875 | 57.91265618036632 | 60.82782435986802 | 61.68434818478833 |

The committed JSON is authoritative for full precision.

## Identification result

For positive, non-capacity-limited points the assay forms:

`Z = q_c g_h N F / J_required = alpha + beta(N+F) + gamma N F`.

Because the ledger records accepted extent, the implementation includes the exact `dt * area` conversion needed to express `J_required` as a rate. P0-P2 are training data; P3 and P4 remain holdout arms.

The scaled design has rank `3` and condition number `25.682977705016`. Independent-axis excitation therefore removes the R3 degeneracy. The deterministic reciprocal least-squares coefficients are:

- `alpha = 0.558251618117446`
- `beta = -0.629755501300906`
- `gamma = 20.9305816462364`
- relative consistency error for `alpha*gamma = beta^2`: `0.966058373333954`

The permitted family requires all three coefficients to be positive and the consistency relation to hold. Negative beta and the large consistency error are decisive structural failures. No finite `V_max` or `K_S` is derived.

## Fail-closed disposition

Classification:

`DCDEV020R4_SATURATING_FAMILY_STRUCTURAL_MISMATCH`

Finite-model P3/P4 predictions, boundary witnesses, qualification, production integration, behavior, and DC-DEV-021 were not run. R4 does not reject all two-substrate kinetics; it rejects only the authorized symmetric two-parameter family as an explanation of this required-source surface.

## Literature disposition

Cleland 1963, Pettersson 1969, and Wang and Mittermaier 2021 are `ADAPTABLE` for experimental structure: explicit multi-reactant equations, reciprocal constraints, and independent variation of both substrate axes. No constants, identities, species, or experimental concentrations were imported.
