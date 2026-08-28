# DC-DEV-020-M1-REPLAN-002-R3

## D-087 N-starvation causal divergence audit

Status: executed observer-only diagnostic; Architect review pending.

Starting authority: `7d7303900e17e1fc3cb0ded911e60ddfe70bb621` on
`strategy/dc-dev-020r9-mesh-contract-requalification`.

The frozen D-087 Gate-2 protocol was replayed for ConservativeV2,
ConservativeV3, and the versioned MaturationCoupledV4 candidate: 200 coupled
warmup steps, N/F removal from external and internal compartments, and 6,000
coupled starvation steps. No physical equation, contract, certifier predicate,
threshold, or production behavior was changed.

## Results

- ConservativeV2: D-087 `8/8`; first `A < 0.05` at step 2349 and first
  observer-nonviability at step 3192. Final `A=0.0005899769713183956`, total
  M `13.101233878701333`.
- ConservativeV3: D-087 `8/8`; first `A < 0.05` at step 4031 and first
  observer-nonviability at step 5395. Final `A=0.011062155811317344`, total
  M `15.049253185803678`.
- MaturationCoupledV4: D-087 `6/8`, retaining the known Gate-1 and Gate-2
  failures. At step 6200, `A=0.09087892901751628`, total M
  `36.62413505096765`, young M `9.02102373815217`, mature M
  `27.603111312815482`, `observer_viable=true`, `alive=true`, no observer death
  reason, and no topology rupture.

V4 nevertheless declines structurally from total M `84.19393140889689` at N
removal to `36.62413505096765`. The frozen-horizon diagnosis is
`CONTINUING_DECLINE_WITHOUT_CERTIFIED_COLLAPSE`.

## Causal decomposition

V4 has no post-removal A production. Its cumulative A loss is
`309.13556158355976`, greater than V2 (`272.79017531969623`) and V3
(`231.35550322765766`), so the higher terminal A is not explained by lower
aggregate energetic loss.

V4's post-removal structural fluxes are production `33.79695218971461`,
maturation `26.922945344348868`, and mature-only turnover
`81.36674854764374`. V2 and V3 have no young pool and turnover
`82.46077080052895` and `87.30067148233664`, respectively. V4 ends with
24.63% of structural M young and retains substantially more area:

| contract | area at N removal | final area | net structural delta |
| --- | ---: | ---: | ---: |
| ConservativeV2 | 604.9275490785683 | 10.141142151389658 | -70.82553614113897 |
| ConservativeV3 | 604.9275490785683 | 13.516839685916693 | -68.87751683403663 |
| MaturationCoupledV4 | 599.6449640024833 | 71.64059009041944 | -47.569796357929235 |

The first full-state post-removal divergence from both comparators is step
201, involving A, C, W, total/young/mature M, free L, bound B, area, and
perimeter. The evidence supports a family-level combined lifecycle/embodied
coupling classification: mature-only turnover plus the resulting preserved
embodied geometry precede lower structural loss and delayed V4 energetic and
observer collapse. This is not an independent necessity claim for either
family; no intervention was authorized in R3.

## Preservation and boundaries

R1 V4 shadow parity, fed homeostasis, recovery, starvation structural decline,
material closure, damage, remesh, fission, and serialization remain preserved
by the referenced accepted evidence. V2 and V3 remain `8/8`; V4 remains the
expected legacy `6/8`. Historical contracts are comparison authorities even
where their legacy material residuals are nonzero; the R3 qualification closure
uses the V4 conservative closure and does not reinterpret historical semantics.

Dense per-step ledgers are stored on Atlas at
`/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/dcdev020m1replan002r3/`.
Compact evidence is under
`digital-protocell/experiments/generated/dcdev020m1replan002r3/`.

Classification: `M1_V4_N_STARVATION_COMBINED_COUPLING`.

This result does not authorize Gate-1 repair, V4 biology changes, physical-death
qualification, production selection, M1 closure, M2, reserve/recycling/salvage,
or any successor execution. `NEXT_EXECUTION_STARTED:false`.
