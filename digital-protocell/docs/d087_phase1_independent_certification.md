# D-087 Independent Phase 1 Certification

## Conclusion

`D087_PHASE1_AUTOPOIETIC_PROTOCELL_CERTIFIED`

- Phase 1 status: `PHASE1_COMPLETE` / `PHASE1_AUTOPOIETIC_PROTOCELL_CERTIFIED` / `MESH_PHASE1_V1_FROZEN`
- Phase 2: `PHASE2_REPRODUCTION_AUTHORIZED`
- Production verdict: `PHASE1_RESEARCH_RUNTIME_QUALIFIED` (not final digital-life product)

## Frozen candidate

```text
schema = autopoietic_material_mesh_v1
state  = mesh_vertices_edges_v1
gamma=1  k_s=14  kappa_b=2  k_pi=0.22  dt=0.02  α≈0.022
commit/tag: 6f8a80a / D-086-mesh-protocell-phase1-pass
```

## Metric semantics (Gate 1)

D-086 reported `tracer_m≈0.35`, `tracer_b≈0.00`, `tracer_c≈0.23` are **pool label fractions**

\[
f_{\mathrm{pool},X}=\frac{M_{\mathrm{labeled},X}(T)}{M_{\mathrm{total},X}(T)}
\]

They are **not** replacement equivalents \(R_X\).

Independent recomputation (5000 steps, same horizon as D-086 Gate4):

| Component | \(R_X\) | \(f_{\mathrm{label}}=L(T)/L(0)\) | \(f_{\mathrm{pool}}\) (D-086 style) |
|-----------|---------|-----------------------------------|-------------------------------------|
| m | 1.018 | 0.191 | 0.349 |
| b | 5.818 | 0.002 | 0.003 |
| C | 1.446 | 0.321 | 0.230 |

Retention `final/initial` for C and A can exceed 1.0 when production is accounted in the ledger (`qualifies_above_one`).

D-087 dual requirement \(R_X\ge1\) and \(f_{\mathrm{label}}\le e^{-1}\) **passes**.

## Seed effectiveness

`seed_organism` / `seed_mesh` only varies vertex count `n = 24 + (seed % 3)`. At most three distinct trajectory classes. Gate 3 used the **deterministic perturbation path** (10 lawful perturbations × 3 sizes), not duplicate seeds as independent stochastic evidence.

## Certifier boundary

`crates/phase1-certifier` imports mesh kernels and schemas only. It does **not** import `d086_analysis` gate conclusions.

## Runtime

Package identity: `digital-protocell-phase1-v1`  
Binary: `digital-protocell-phase1`  
Script: `scripts/package_phase1_linux.sh`

Gate 7 used 80 000 autonomous steps with snapshot/resume (full wall-clock 90 min–6 h available via `D087_FULL_RUNTIME=1`).

## Artifacts

`experiments/generated/d087/` (archive-backed symlink).
