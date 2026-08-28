# DC-DEV-020-M1-REPLAN-002-R4

## V4 contract-aware preservation qualification

This package executes the only authorized R4 scope from the accepted R3 state at
`ad1642ec3b2e565e0651efe3daf36e0390351dfb`. It corrects the D-087 structural
observer tracer for `MaturationCoupledV4` and qualifies the existing causal
starvation predicate. It does not repair V4 biology or change any D-087
threshold, duration, or physical equation.

## Authority and scope

- Branch: `strategy/dc-dev-020r9-mesh-contract-requalification`
- PR: `#44`, open/draft/unmerged
- Contract under test: `MaturationCoupledV4`
- Historical controls: `ConservativeV2` and `ConservativeV3`
- Production default: `ConservativeV2 / reserve OFF` (unchanged)
- Dense evidence: `/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/dcdev020m1replan002r4/`
- Compact evidence: `experiments/generated/dcdev020m1replan002r4/`

The only scientific source change is observer-side tracer handling in
`coupled_step_with_reserve_mode()`. V4's existing physical lifecycle remains
unchanged: total edge material is authoritative, new material is young,
maturation transfers young to mature without creating label, and ordinary
turnover removes material from the mature pool. Historical contracts do not call
the V4 tracer helper.

## Tracer correction

The legacy structural tracer treated V4 as undifferentiated material. The R4
observer correction runs in parallel with the unchanged physical step:

1. The initial pulse label is mature and labeled.
2. Newly built material is young and unlabeled.
3. Maturation creates no label.
4. Mature-only turnover removes label in proportion to labeled material in the
   turnover-eligible mature pool.

The helper changes no physical state, chemistry, mechanics, topology, geometry,
or turnover amount. On the immutable physical trajectory, the resulting V4
structural labeled fraction is `0.24710833271945795` and satisfies the existing
frozen Gate-1 thresholds. This is a contract-aware observer qualification, not a
D-087 threshold change.

## Frozen starvation qualification

The exact existing D-087 starvation continuation is reused: 200 warmup steps,
external and internal N removal, then the existing `gc_preservation::causal_starvation_passes`
predicate through the fixed 150,000-step bound. The local R4 run records:

- V2 D-087: `8/8`
- V3 D-087: `8/8`
- V4 D-087: `[true, true, false, true, true, true, true, true]`
- V4 only remaining frozen D-087 failure: Gate 2 starvation
- causal starvation predicate: `true`
- first `A < 0.05`: step `5277`
- first observer viability loss: step `6130`
- first topology rupture: not reached
- final organized material: `29.51187358387537`
- post-switch N delivery: `0`

The V4 physical trajectory remains unchanged and the frozen starvation gate is
not modified. V4 is therefore qualified for this bounded contract-aware
preservation package, while M1 remains open because the unchanged biological
starvation certification failure is still present.

## Acceptance boundary

R4 is valid only when the compact evidence, preservation flags, V2/V3 controls,
V4 seven-of-eight gate vector, and exact-head Linux CI all agree. A failed
scientific predicate is evidence, not permission to weaken certification. No
physical-death follow-up, production selection, M1 closure, M2, reserve,
recycling, salvage, tuning, or successor work is authorized by this package.

## Result status

The local result is provisional pending exact-head remote CI and Architect
review. The final handoff must report the exact result head, both remote CI
results and artifact digest, with `PRODUCTION DEFAULT CHANGED: NO`, `M1: NOT
ESTABLISHED`, `M2 AUTHORIZED: NO`, and `NEXT_EXECUTION_STARTED: false`.
