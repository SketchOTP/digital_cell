# D-059 — Viable Size Basin and Membrane-Area Architecture Review

## Primary conclusion

`D059_EXTERNAL_CARRIER_SIZE_LIMIT_NO_RESTORING_BASIN`

**Route L** — one global corrected carrier rate supports a contiguous smaller-radius throughput band, but unmodified structural dynamics do **not** restore organisms toward that band (`NEUTRAL_SIZE_MANIFOLD`).

No V15. No production carrier. No internal-membrane implementation. No target-radius controller.

## Preservation

`EXTERNAL_MEMBRANE_CARRIER_SURFACE_CAPACITY_LIMIT_CONFIRMED`

| Item | Value |
|------|-------|
| Start commit | `482882d` |
| Start tag | `D-058-corrected-carrier-normalization-audit` |
| D-056 | `ed6de2c` / `D-056-waste-coupled-resource-carrier-fail` |
| D-057 | `1c9d6ae` / `D-057-carrier-geometry-driving-force-audit` |
| D-058 | Route V `D058_CARRIER_SURFACE_VOLUME_CAPACITY_LIMIT` |

## Scaling (Gate 1)

Matched-state disk campaign (held C/A/P/N/F/W/occupancy; R ∈ {6…32}):

| Exponent | Value |
|----------|-------|
| `p_M^matched` | ≈ **2.00** |
| `p_T^matched` | ≈ **1.00** |
| Classification | `D058_RADIUS_EXPONENT_CONFOUNDED` |

D-058 coupled exponents (`p_M≈7.81`, `p_T≈1.07`) are **not** universal radius laws — they amplify chemical collapse on top of geometric surface/volume scaling. Matched campaign supersedes them for geometry.

## Global-rate frontier (Gate 2)

Predeclared ladder from sealed D-058 corrected `k_T★` bounds (chosen **before** trajectories):

`[0.00738, 0.0276, 0.103, 0.384, 1.435]`

Radius-specific `k_T` rejected.

## Size (Gates 3–4)

| Item | Result |
|------|--------|
| Horizons | 2500 → 5000 → 10000 |
| Best global `k_T` | ≈ **1.435** (highest ladder rate) |
| Viable contiguous range @10k | **R6–R14** (5 radii) |
| χ_N, χ_F @10k | ≥ 1.05 across the band |
| Restoring-size class | `NEUTRAL_SIZE_MANIFOLD` |
| Gates 5–6 | skipped (no restoring basin) |

Organisms initialized below/at/above the band remain near their seed equivalent radius (`dR/dt ≈ 0`). No common `R★` attractor under existing structural dynamics.

## Membrane area (Gates 7–10)

Evaluated because the size band lacks restoration:

| Item | Result |
|------|--------|
| Amplification @ selected global `k_T` | bounded (`α_max < 1.25` for sealed large-R proxies with this high rate) |
| Material bootstrap | feasible at proxy budget |
| Topology A/B | environmentally connected, admissible as **observer classes only** |
| Topology C | rejected (closed vesicles) |
| Gate 10 area architecture | **not** selected — Route L has priority; no production topology authorized |

Internal membrane area is **not** the selected next architecture while a viable external-size band exists without restoration.

## Shadow comparison (Gate 11)

| Case | χ | A retention | viable |
|------|---|-------------|--------|
| External-size best (R6, carrier) | ≈1.10 | ≈0.51 | yes |
| Internal-area proxy | ≈0.98 | ≈0.17 | no |
| Passive baseline | ≈0.81 | ≈0.11 | no |
| Carrier knockout | ≈0.81 | ≈0.11 | no |

## Selected route

**Route L** — next directive reviews the **existing structural growth law** and resource-coupled size feedback so endogenous dynamics can maintain the viable band.

Do **not** change carrier kinetics. Do **not** add a target-radius controller. Do **not** implement V15 from this result.

## Status (frozen until later implementation directive)

- Selected architecture: **none**
- V15: unauthorized
- Internal membrane architecture: unauthorized
- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Stage F: not authorized
- Production: `REQUIRES_REMEDIATION`

## Artifacts

`digital-protocell/experiments/generated/d059/` → `/mnt/storage1tb/cache/project-artifacts/digital_cell/experiments/generated/d059`

## Tests

`cargo test -p chemistry-core --test d059_tests` — 14/14 PASS.

## Pipeline

```bash
D059_MAX_ACCEPTED=10000 cargo run -p experiment-runner --release -- d059 pipeline
```
