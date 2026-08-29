# DC-DEV-020-M1-R3 — Ordinary-Decay Candidate Qualification

## Authority

- Directive: `DC-DEV-020-M1-R3-ORDINARY-DECAY-CANDIDATE-001`
- Exact starting head: `1622b664a4a37b8a0ac4ea51fbc97ca71f9d853c`
- Selected M0: `ConservativeV2`, reserve OFF
- Candidate: `ConservativeV3`, physical mesh contract still `ConservativeV2`

## Candidate boundary

`ConservativeV3` changes one semantic choice only: when internal `N×F < 1e-8`, activated-material decay uses the ordinary `k_a_decay` coefficient rather than the `ConservativeV2` fourfold starvation multiplier. `k_a_decay` remains `0.008`; no controller, health state, source-law change, or recovery behavior is introduced.

`ConservativeV2` remains unchanged and remains the selected production configuration. `ConservativeV3` is experimental and is not selected by default.

## Required qualification

The assay compares V3 with the accepted ordinary-decay reference (`ConservativeV2` with diagnostic `k_a_decay=0.002` and the existing fourfold multiplier), checks non-starved V2/V3 parity, replays the source-capacity upper bound without `/4` compensation, preserves topology death and non-recovery after refeeding, and runs actual D-087 for both V3 and V2 with reserve disabled.

The source-capacity shadow is the existing paired internal `N/F→A` upper bound. It is not production source implementation.

## Local result

- V2/V3 non-starved parity: passed; `N×F` remained positive and mesh/ledger replay was exact.
- Ordinary starvation equivalence: passed; observer collapse step `3279` and topology rupture step `124717` matched.
- V3 rupture: `24/24` edges, `closed_intact=false`.
- Ordinary and source-capacity refeeding: 5,000 accepted steps each; topology did not recover.
- Source-capacity V3 organized-material change: `+1.2571804904075918`.
- Source-capacity V3 final values: `A=23.4384728164839`, `C=54.3531757631337`, structural `M=25.4904282924088`, membrane `=29.7814998449361`.
- Internal closure residual: `3.37507799486048e-13`.
- Actual D-087: V3 `8/8`; preserved V2 `8/8`.

The candidate classification is `M1_ORDINARY_DECAY_CANDIDATE_QUALIFIED`, pending exact-head remote CI and independent architect acceptance. This does not establish M1 homeostasis or authorize source implementation.

## Prohibited follow-on work

No production switch, boundary-coupled source, transport repair, reserve redesign, recycling, salvage, M2, or DC-DEV-021 work is authorized by this directive.
