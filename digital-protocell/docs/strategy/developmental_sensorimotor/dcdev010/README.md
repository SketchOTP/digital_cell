# DC-DEV-010: Passive directional substrate coupling

Status: scientifically failed at the first qualification gate. No parameter
repair, second traction law, or DC-DEV-011 work is authorized.

Entry authority: `8d6fe59397cabfa47bc1d8103acd68f544acc190` on
`strategy/dc-dev-009-motility-feasibility-audit`.

This directive tested one reusable passive substrate law against the accepted
DC-DEV-009 fixed ring. The law uses a fixed x-axis and local piecewise
direction-dependent resistance based on each vertex's pre-step attempted
velocity. It is dissipative and bounded, and it does not read regulatory
meaning, targets, resources, navigation, or reward.

## Frozen parameter set

The first qualification execution used exactly one parameter set:

| parameter | value |
| --- | ---: |
| substrate axis | `[1.0, 0.0]` |
| forward resistance ratio | `0.25` |
| reverse resistance ratio | `0.75` |
| transverse / isotropic control ratio | `0.50` |
| maximum local reaction force | `0.45` |

The ratios are dimensionless relative to the existing `MechParams.gamma = 1`.
They remain below one so the accepted velocity stays aligned with the
attempted velocity and the substrate work is non-positive. The force bound is
below the existing post-Phase-1 external-force bound. No screening or
post-result parameter adjustment occurred.

## Scientific result

The passive reaction itself passed its work and bounded-force checks. The
matched motor-off directional-substrate arm translated by
`0.013504913541228361` along the substrate axis, while the preregistered
translation tolerance was `2.220446049250313e-13`. Therefore the first failed
gate is:

`GATE1_PASSIVITY_AND_MOTOR_OFF_NO_PROPULSION`

The law converts baseline mechanics relaxation into translation even without
funded contractility. The active directional arm translated by
`0.09066122609307165`, but that result cannot be attributed lawfully to the
existing reserve-funded contractility after the motor-off control fails.

Conclusion:

`DCDEV010_DIRECTIONAL_SUBSTRATE_TRANSLATION_NOT_ESTABLISHED`

The negative result is preserved as evidence. No tuning is permitted under
this directive. A future directive may decide whether to revisit the physical
contact model; this branch does not do so.

Generated evidence is under `digital-protocell/experiments/generated/dcdev010/`.

## R1 mechanical-rest causal isolation

The architect-authorized R1 falsification repair starts from PR #19 head
`b4178417e30907835183c7f9c16a639bdd8d31db`, preserves the original evidence,
and writes separate artifacts under `experiments/generated/dcdev010r1/`.
R1 stopped at the preregistered baseline-rest gate:

`DCDEV010R1_BASELINE_MECHANICAL_REST_NOT_ESTABLISHED`

See [r1_mechanical_rest.md](r1_mechanical_rest.md). DC-DEV-011 remains blocked.
