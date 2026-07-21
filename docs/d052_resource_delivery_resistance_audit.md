# D-052 — Nutrient/Fuel Delivery Resistance Decomposition

## Primary conclusion

`D052_MIXED_RESOURCE_DELIVERY_LIMIT`

Selected route: **X — Mixed exterior-vicinity + membrane-crossing delivery resistance**.

Diagnostic only. No biological parameter or equation change.

## Preservation

| Item | Value |
|------|-------|
| Start commit | `e08075a` |
| Start tag | `D-051-coupled-activation-throughput-audit` |
| Frozen | `D049_COUPLED_ACTIVATION_CAPACITY_FAILURE`, `D050_COUPLED_ACTIVATION_CAPACITY_NOT_RECOVERED`, `D051_RESOURCE_THROUGHPUT_LIMIT`, `COUPLED_ACTIVATION_TOPOLOGY_CAPABLE` |
| Activation probe | schema-2 fitted center (`V_A≈0.1254`); schema-1 historical control |
| Record | `ACTIVATION_SUPPLY_LAW_NOT_CURRENT_REPAIR_TARGET` |
| Stage E | `BLOCKED_NOT_RECOVERED` |
| Phase 1 | `PHASE1_SELF_MAINTENANCE_PARTIAL` |
| Stage F | not authorized |
| Production | `REQUIRES_REMEDIATION` |

## Gate 0 — D-051 reproduction (horizon 10000 / controls 5000)

| Case | A retention | Notes |
|------|-------------|-------|
| Schema 1 | ≈0.0104 | collapse |
| Schema 2 center | ≈0.0357 | collapse |
| Schema 2 4× | ≈0.0312 | weak free-A V_A response |
| Healthy N only | ≈0.105 | insufficient alone |
| Healthy F only | ≈0.105 | insufficient alone |
| Healthy N+F | ≈1.093 | material rescue |
| Unlimited N/F | ≈2.002 | material rescue |
| Reservoir 5× (matched horizon) | ≈0.081 | no rescue vs matched baseline ≈0.081 |

PASS: ordinary collapse; weak V_A free-A response; N/F rescue; reservoir non-rescue; accounting closed.

## Gate 1 — N/F ledgers

Regional observer ledgers close within tolerance for N and F on schema-2 center.

## Gate 2 — Spatial profiles

Final radial N/F (schema-2 center @10k):

| Region | N≈F |
|--------|-----|
| Reservoir | 1.00 |
| Exterior | 0.99 |
| Immediately outside | 0.62 |
| Immediately inside | 0.30 |
| Peripheral / activation | 0.18 |
| Central | 0.13 |

Depletion locus: **before_membrane** (largest single drop exterior→outside), with a comparable **across-membrane** drop.

## Gate 3 — Resistance fractions (N and F identical within noise)

| Segment | Fraction |
|---------|----------|
| Reservoir relaxation | ≈0.0 |
| Reservoir→exterior | ≈0.01 |
| Exterior diffusion | ≈0.43 |
| Membrane crossing | ≈0.37 |
| Peripheral interior | ≈0.14 |
| Central interior | ≈0.05 |

No single segment ≥50%. Exterior + membrane ≈80%.

## Gate 4 — Resource identity

`JOINT_RESOURCE_LIMIT` — healthy N-only and F-only insufficient; joint healthy N+F restores.

## Gate 5 — Reservoir controls

1× / 5× / 20× reservoir rate and exterior concentration hold do **not** materially restore accepted activation or free A. Exterior already near reservoir → reservoir not the repair target.

## Gate 6 — Permeability controls

N/F attenuation bypass and membrane-free N/F transport raise A only ≈0.081→0.085 (not ≥50%). Not `MEMBRANE_RESOURCE_PERMEABILITY_LIMIT` alone.

Stage A healthy N/F permeability at θ=1: Π≈0.301 — **inside** approved 0.20–0.50 band.

## Gate 7 — Diffusion controls

Exterior or interior 5× diffusivity and conservative mixing do not meet the ≥50% activation-rise criterion (best: interior N/F 5× ≈0.081→0.114).

## Gate 8 — Membrane-state dependence

`no_s` / low S still leave A collapsed despite near-unit N/F permeability. High S reduces interface flux but does not establish a clean selectivity tradeoff that alone explains failure.

`SELECTIVITY_THROUGHPUT_INCOMPATIBILITY` = **false** (corrected criterion).

## Gate 9 — Radius scaling

R16/R22/R32 all remain collapsed (A≈0.087/0.081/0.076). χ decreases with radius; not a small-cell-only rescue → not `RESOURCE_SURFACE_VOLUME_SCALING_LIMIT`.

## Gate 10 — Yield diagnostic

χ_activation ≪ 1 under ordinary flux vs healthy-N/F demand; transport not adequate → yield path **not** authorized.

## Gate 11 — Long validation

Strongest reference control: healthy interior N+F. A remains ≈1.03 through 25k accepted attempts, but positivity rejections exceed 500 under the clamp (numerical stress). Short healthy-N/F rescue remains the causal reference; clamp is nonpromotable.

## Gate 12 — Route

**Route X** → `D052_MIXED_RESOURCE_DELIVERY_LIMIT`

Bounded combination: exterior near-membrane drop + membrane-crossing drop dominate resistance; neither alone, nor reservoir/diffusion/permeability/selectivity single controls, qualifies a repair.

## Secondary findings

| Item | Result |
|------|--------|
| Limiting resource identity | `JOINT_RESOURCE_LIMIT` |
| Resistance fractions | exterior ≈43%, membrane ≈37% |
| Interface concentration drops | outside 0.62 → inside 0.30 |
| Interior gradients | inside 0.30 → central 0.13 |
| Cap-site fractions | joint limitation under ordinary delivery |
| Radius scaling | mild worsening with R; not sole cause |
| Membrane-state dependence | occupancy alone does not restore delivery |
| Long-control persistence | A-value rescue yes; numerical stress under clamp |
| Stage A transport contract | N/F Π≈0.30 still in approved band |

## Tests / artifacts

- `cargo test -p chemistry-core --test d052_tests --release` — 13/13 PASS
- Artifacts: `digital-protocell/experiments/generated/d052/`
- Pipeline: `D052_MAX_ACCEPTED=10000`, `D052_CONTROL_HORIZON=5000`, `D052_LONG_HORIZONS=25000`

## Deviations

- Diagnostic clamps/bypasses only; nonpromotable.
- Exterior/interior diffusion multipliers applied as uniform `d_n`/`d_f` scales (region split approximated).
- Reservoir annulus-width control proxied by exterior concentration hold.
- Gate-8 selectivity criterion tightened after a false positive (collapse under `no_s` shows attenuation is not sufficient cause).
- Route decision recomputed from sealed gate artifacts after that correction.

## Status

- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Stage F: not authorized
- Production: `REQUIRES_REMEDIATION`

## Next directive

May target only a **bounded combined** exterior-near-interface + membrane-crossing delivery repair while freezing C/A/W selectivity and activation law. Do not alter activation until that combined resistance is addressed. Do not weaken all permeability.
