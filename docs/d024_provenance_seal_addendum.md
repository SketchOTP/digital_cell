# D-024 Provenance Seal Addendum

**Directive:** D-025 Gate 0  
**Conclusion:** `D024_PROVENANCE_SEALED`  
**Prior operative status:** `D024_INTERFACIAL_SURFACE_DENSITY_PASS_PROVISIONAL`  
**Architecture:** `INTERFACIAL_SURFACE_DENSITY_SELECTED`

## Exact source

- Source commit: `06477f631d5e2ae19dbbfd09b288e866405fb628`
- Detached clean worktree build; tracked dirty count = 0
- Equation version: `membrane_metabolism_v7_surface_density`
- Prior pass tag preserved: `D-024-surface-density-pass`
- Seal tag: `D-024-surface-density-pass-provenance-sealed`

## Gate 6 R22 reproduction

Selected candidate Damköhler `0.5`, `k_ads = 0.0011111111111111111`.

| Metric | Sealed rerun |
|---|---|
| Γ localization | ≈ 1.0 |
| C retention | ≈ 0.991 |
| A retention | ≈ 0.924 |
| Material residual | ≈ 2.5×10⁻⁷ |
| Diagnostic steps | 5000 |
| Governed steps | 25000 |
| Termination | MaxAcceptedSubstepsReached (clean) |

Artifacts: `digital-protocell/experiments/generated/d025/d024_provenance_seal/`

This addendum seals committed-source provenance. It does not replace the existing D-024 pass tag.
