# D-016 membrane conductance

## Required outward flux

```text
J_required = interior_W_production / interface_length
           ≈ 33.55 / (2π × 22) ≈ 0.243
```

## Modeled conductance

```text
G_W = D_W × P_W(M=1) / Δx
    ≈ 0.25 × 0.819 / 1 ≈ 0.205
```

```text
ΔW_required = J_required / G_W ≈ 1.19
```

Allowable interior–exterior difference before the safety ceiling is O(10).
Membrane drop ≈ 1.2 is therefore **not** the primary limiter relative to the
analytical interior rise ΔW_center ≈ 12.7.

## Classification

`NOT_PRIMARY_LIMIT` / adequate-to-marginal relative to fill headroom;
internal diffusion dominates the resistance budget after sink repair.

Artifact: `digital-protocell/experiments/generated/d016/conductance_analysis/conductance.json`
