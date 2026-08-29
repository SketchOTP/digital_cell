# DC-DEV-020-R8-R1: Causal A-Demand Elasticity

## Disposition

This is an observer-only requalification from accepted R8 head
`5b314792fe896504f6f8b99218ba48f0328de9f0`. It does not fit or implement a
source law, change production chemistry, or authorize DC-DEV-021.

The exact R8 interpretation is preserved:

`OBSERVATIONAL_PRODUCT_FEEDBACK_TOPOLOGY_CLOSED`

R8 rejected the exact observational reciprocal product-feedback topology, but
matched N/F states allowed other demand-generating state to vary. R8-R1
therefore perturbs only A within reconstructed frozen states and solves the
existing physical zero-drift source requirement.

## Frozen inputs and reconstruction

- R5 training states: 2,880, P0/P1/P2, both accepted R5 trajectories.
- R7 on-policy states: 480.
- P3 and P4 portability states: 960 each.
- Every state is rebuilt through the accepted settlement, deprivation, uptake,
  and reaction procedures. Reconstructed pre-reaction hashes are compared with
  the sealed R5 and R7 ledgers before perturbation.
- A probes are `A*exp(-0.01)`, unchanged A, and `A*exp(+0.01)`.
- N, F, R, C, area, structure, membrane, geometry, strain, catalyst state,
  templates, networks, and all other state are held fixed.

Dense R8-R1 records are stored outside Git:

`/mnt/storage1tb/cache/project-artifacts/digital_cell/experiments/generated/dcdev020r8r1/d50037e53d041d8b06895553933c3b0a78c7a024/demand_elasticity_ledger.json`

SHA-256: `f44e8f9fa441451ee40bcbfccac5f556131e4d26868868607e9507c29e7bcf90`

## Results

All 10,560 finite A-minus/A-plus roots passed the tightened local root solve;
capacity failures and non-monotonic perturbations were zero. Demand
decomposition closure had maximum absolute residual
`2.6549075438087044e-14`, below the `1e-10` requirement.

The primary classification is:

`DCDEV020R8R1_A_DEMAND_ELASTICITY_POSITIVE`

The epsilon-A distributions were positive on every audited set:

| Set | States | Median | p05 | p95 |
| --- | ---: | ---: | ---: | ---: |
| P0-P2 training | 2,880 | 0.9057807017 | 0.9039517000 | 0.9242253515 |
| P3 | 960 | 0.9076652271 | 0.9046950941 | 0.9239163891 |
| P4 | 960 | 0.9076652271 | 0.9046950941 | 0.9239163891 |
| R7 on-policy | 480 | 0.9169330052 | 0.9070701708 | 0.9239570357 |

The same sign holds for normalized demand `Y_zero`. Catalyst production is
the dominant block by median demand magnitude. A decay, structural
production, membrane production, and reserve loss were also recorded. A↔R
exchange is reported separately and is not counted as net stored-material
destruction.

The pair result is independently:

`R8_PAIR_CONFOUNDING_CONFIRMED`

All 2,425 sealed R8 pairs were audited with both A-only swaps. The median
A-only contribution to the observed root difference was `0.9570488659`, the
median background-state contribution was `0.0429511341`, and 155 pairs
(`0.0639175258`) reversed sign because of background state. Causal-swap
asymmetry was `0.6437570971`. This confirms that the R8 contradictions were
not a clean causal test of A alone, even though the within-state A elasticity
is positive.

## Scientific boundary

The result makes A-product inhibition causally plausible as supply feedback,
but does not authorize an A-feedback implementation, production integration,
reserve or sink changes, a new source law, or DC-DEV-021. The next decision
remains with architect review.

External methodology references were used only as methodology: Hofmeyr and
Cornish-Bowden (2000), PubMed `10878248`, and Koebmann et al. (2002), PubMed
`12081962`. No external constants, enzyme identities, concentration ranges,
or control parameters were imported.
