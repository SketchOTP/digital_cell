# DC-DEV-011: passive isotropic stick-slip traction

Status: qualification executed locally; scoped remote preservation CI and
architect review pending. The result is not yet architect-accepted.

Entry authority: `8d6fe59397cabfa47bc1d8103acd68f544acc190` on
`strategy/dc-dev-009-motility-feasibility-audit`.

The question is whether the existing reserve-funded local contractility can
obtain retained body displacement through one passive, local, direction-neutral
substrate. The substrate has no axis, front/rear concept, target, stimulus,
regulatory input, reward, or vertex-specific parameter.

## Frozen production mechanism

The sole implementation is
`regulatory-core/src/stick_slip_traction.rs`.

Each contact reads only the local attempted velocity and the local total force
that the existing mechanics path would present. If the required holding force
is within the static limit, the reaction is equal and opposite and the contact
sticks. Otherwise the contact slips and receives a bounded kinetic reaction
opposite the local attempted velocity. The reaction is passed to the existing
`chemistry-core` mechanics integrator, which remains the only position-update
authority.

Frozen parameters, selected once before qualification:

| Parameter | Value | Basis |
|---|---:|---|
| static traction limit | `0.45` | below the existing `0.5` external-force bound and on the scale of the approximately `0.52` late standalone component forces reported by DC-DEV-010-R2 |
| kinetic traction magnitude | `0.20` | strictly below static traction and observable against the accepted local contractility force scale |
| zero-motion tolerance | `1e-12` | solver-scale contact classification tolerance |

No parameter sweep, optimization, or post-result adjustment is authorized.

## Frozen qualification protocol

The common 24-vertex seed and the already-preserved DC-DEV-009 local stimulus
are used. The body first settles for at most 5,000 accepted legacy passive
mechanics steps. Failure to reproduce the accepted DC-DEV-010-R2 mechanical
rest reference stops the directive.

From the exact settled body, run 240 accepted active steps followed by 240
accepted zero-stimulus relaxation steps. Growth, remeshing, fission, chemistry
reactions, resource acquisition, and topology changes remain disabled. The
regulator decays normally during relaxation.

Matched arms:

1. active reserve-funded contractility plus stick-slip;
2. motor-off plus stick-slip;
3. active reserve-funded contractility with no substrate;
4. zero reserve plus stick-slip.

The primary metric is final material-centroid displacement from the settled
body after all 480 accepted steps. A transient displacement that disappears in
relaxation is not locomotion.

## Preregistered acceptance thresholds

The translation tolerance is `1e-10`, the maximum of the DC-DEV-009 free-space
metric floor and the DC-DEV-010-R2 settled centroid-noise reference. Active
stick-slip must exceed both the motor-off stick-slip and active no-substrate
final material-centroid displacements by at least this tolerance. At least one
stuck contact and one slipping contact are required. At least 25% of active
phase displacement must remain after relaxation. The 180-degree rotational
equivalence control uses an absolute displacement-vector tolerance of `1e-9`.
Material- and vertex-centroid displacement vectors must agree within `1e-8`.

The pass/fail conclusions are exactly:

- `DCDEV011_PASSIVE_ISOTROPIC_STICK_SLIP_TRANSLATION_QUALIFIED`
- `DCDEV011_STICK_SLIP_TRANSLATION_NOT_ESTABLISHED`

Neither conclusion establishes autonomous gait, continuous locomotion,
steering, navigation, resource seeking, learning, or evolution.

The local qualification produced
`DCDEV011_PASSIVE_ISOTROPIC_STICK_SLIP_TRANSLATION_QUALIFIED`; see
`qualification_results.md` and the compact generated JSON artifacts. The
remote preservation status remains pending until the scoped workflow passes.

`NEXT_EXECUTION_STARTED:false`.
