# DC-DEV-020-R9-R6 — mobilize-first/store-last audit

This is the final observer-only D-091 phase-order audit. It starts from
`f1704acff5ca64e509a28c74af8cccbf76439ef2` and runs exactly 5,000 accepted
steps for matched `FULL` and `MOBILIZE_FIRST_STORE_LAST` arms.

The shadow calls the existing frozen release and loss kernels before the
existing productive chemistry, leaves catalyst, structural, and membrane
equations unchanged, and calls the existing store kernel after productive
demand. It introduces no new rate, target, health state, controller, direct
R→M/R→L pathway, or production default.

The V20 control reproduces 8/8. FULL reproduces `R_m=0.839869520280528`.
The shadow reaches `R_m=0.839973528362306`, with D-087 gates
`[true,false,false,false,false,true,true,true]`. The small improvement is
real but insufficient for certification. Replete `A→R` is
`147.5982725689982`; post-starvation `R→A` is `26.15666583047419`; reserve
rejects are zero and strict material closure passes.

## Classification

`DCDEV020R9R6_MOBILIZE_FIRST_STORE_LAST_CONTRIBUTORY_NOT_SUFFICIENT`

No production reserve physiology or chemistry was changed. Recycling,
salvage, and DC-DEV-021 remain unauthorized. The project returns to the
Architect for the production D-091 decision; no R9-R7 observer investigation
is authorized.
