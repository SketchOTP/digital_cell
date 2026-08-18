# DC-DEV-020-R5 local zero-drift source audit

## Disposition

`DCDEV020R5_NF_LOCAL_COORDINATE_SUFFICIENT`

Independent of that coordinate result, the R3/R4 endpoint-derived source target is:

`ENDPOINT_SURROGATE_NOT_LOCAL_REQUIREMENT`

This is an observer-only diagnostic. It identifies no production rate law, changes no chemistry, and authorizes no integration or DC-DEV-021 work.

## Authority and historical guard

- Accepted R4 evidence head: `669a511aacb227240bd7a4698efecfb564f481d4`.
- Clean production base: `1e242f28152797b512e25cd56c7b718e45d6ca97`.
- R4 remains accepted as `DCDEV020R4_SATURATING_FAMILY_STRUCTURAL_MISMATCH` for its endpoint-derived constant-gain surrogate surface. R4 did not establish a unique instantaneous source-demand surface.
- D-043 rejected a portable scalar recalibration of the historical `k*C*N*F` source.
- D-045 rejected catalyst-linear demand representability.
- D-067 found no durable portable activation-capacity law among its bounded/static candidates.

No historical implementation, constants, species, or mechanism was imported.

## Method

The exact R4 P0-P4 baseline and constant-gain trajectories were replayed. Their ten trajectory hashes match the accepted R4 evidence. At each of 480 feeding steps for both arms and all five probes, the organism was cloned after resource uptake and before `reactions_step`.

For each of the resulting 4,800 states, the observer varied physical source extent over:

`S in [0, min(N*area, F*area)]`

The fixed shape diagnostic evaluated `S/S_sat = 0, 0.25, 0.5, 0.75, 1`. When the response was monotone through its first crossing, bracketed bisection found the minimum source extent with `delta E_stored >= 0` to relative interval at most `1e-6`. Every counterfactual ran the full frozen chemistry step, including catalyst cost, A decay, reserve exchange/loss, structural production, membrane production, and reaction sequencing.

## Local capacity and sequencing

| Measure | Result |
|---|---:|
| States audited | 4,800 |
| `S_zero = 0` | 0 |
| Finite physical roots | 4,800 |
| Source-capacity-insufficient | 0 |
| Non-monotonic before first root | 0 |
| Accelerated A decay at root | 0 |
| Saturation diagnostic crosses accelerated-decay boundary | 4,800 |
| Median `S_zero/S_sat` | 0.00300455093383789 |
| 5th/95th percentile `S_zero/S_sat` | 0.00150203704833984 / 0.0211890697479249 |
| Maximum root relative interval | 9.53674316472667e-7 |
| Maximum source acceptance error | 1.04083408558608e-17 |
| Maximum stored-material accounting residual | 2.2849994466001e-14 |

Physical source capacity is locally sufficient throughout the audited manifold. Saturation itself crosses the starvation-accelerated A-decay boundary by consuming one limiting substrate, but every minimum zero-drift root occurs before that boundary. The local response is therefore usable for root extraction without changing reaction order.

## R3/R4 surrogate audit

The endpoint-derived constant gain is strongly time-dependent relative to local need. It under-supplies early states and over-supplies later states.

| Probe | Median constant/zero | 5th | 95th | Relative RMSE | Below local balance | Materially above |
|---|---:|---:|---:|---:|---:|---:|
| P0 | 1.206584032032501 | 0.027409379592661348 | 1.4625200764800528 | 0.5187525178653605 | 0.40625 | 0.5520833333333334 |
| P1 | 1.1138314553311401 | 0.01881146527283732 | 1.6485640396517784 | 0.5909694131145206 | 0.45625 | 0.50625 |
| P2 | 1.1138314553311401 | 0.01881146527283732 | 1.6485640396517787 | 0.5909694131145206 | 0.45625 | 0.50625 |
| P3 | 1.0736863825696112 | 0.01647386944908129 | 1.7246242862062164 | 0.6185216953236883 | 0.47291666666666665 | 0.4895833333333333 |
| P4 | 1.0736863825696112 | 0.016473869449081288 | 1.7246242862062167 | 0.6185216953236883 | 0.47291666666666665 | 0.4895833333333333 |

The R4 family rejection remains valid for the tested surrogate. These data show why it must not be reinterpreted as rejection against the instantaneous physiological balance boundary.

## Existing-coordinate diagnostic

The preregistered predictor was an unweighted Euclidean 16-nearest-neighbor observer. Features were min-max normalized using P0-P2 training data only. P3-P4 remained held out. The target was `S_zero/(q_c*area*dt)`.

| Coordinate | Held-out relative RMSE | Held-out p95 absolute relative error | Local ambiguity | Preregistered result |
|---|---:|---:|---:|---|
| `(N,F)` | 0.100445161424658 | 0.213953395530704 | 0.233276111779549 | sufficient |
| `(N,F,A)` | 0.0408720963666842 | 0.0943209342814106 | 0.219999558518084 | sufficient |

Because the simpler C0 coordinate passed its frozen preregistered limits before adding A, the diagnostic classification is `DCDEV020R5_NF_LOCAL_COORDINATE_SUFFICIENT`. A materially improves prediction, but R5 does not use that improvement to select a law or add state.

This conclusion is bounded to the frozen P0-P4 state manifold and one-step zero stored-material drift. It is not a claim that a memoryless N/F production controller is qualified, durable, or sufficient for organism-level restoration.

## Evidence

Compact authoritative files are under `experiments/generated/dcdev020r5/`. The 4,800-record dense ledger is stored at the governed external location recorded in `external_evidence_manifest.json`, with SHA-256 verification. Git retains the protocol, schema, compact results, qualification, literature disposition, representative records, and external manifest.

## External discovery

- Galvez, Varon, and Canovas (1981): `ADAPTABLE` for transient two-substrate analysis structure.
- Zechel et al. (1998): `REFERENCE_ONLY` for direct pre-steady-state observation of transient intermediates.
- Flach and Schnell (2006): `ADAPTABLE` as a warning that reduced quasi-steady-state equations can omit coupled dynamics.

No external biological constants, enzyme identities, concentrations, or mechanisms were imported.
