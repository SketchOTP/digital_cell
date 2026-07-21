# D-060: Structural Growth Law and Resource-Coupled Size Feedback

Observer/shadow diagnostic only. **Production biology is unauthorized** — no changes to
carrier defaults, activation, or structural production/decay kinetics.

## Primary conclusion

`D060_STRUCTURAL_GEOMETRY_EXECUTION_DEFECT` (Route G)

## Existing structural equations (frozen)

Interface-limited turnover (V3 selected mechanism, still active on V13):

- Production density: \(G = k_{\mathrm{d008,structure}} \, A \, I(\phi)\)
- Decay density: \(L = k_{\mathrm{structure,decay}} \, \phi \, (\varepsilon + I(\phi))\) with \(\varepsilon = 0.05\)
- Stoichiometry: \(A \to \eta_\phi \phi + (1-\eta_\phi) W\); decay \(\phi \to W\)

Elasticities at matched geometry: \(\varepsilon_A^{G} \approx 1\), \(\varepsilon_A^{L} = 0\),
\(\varepsilon_C^{G} = 0\) (InterfaceLimitedTurnover has no catalyst gate).

## Structural ledger

\[
\Delta M_\phi = G_\phi - L_\phi + J_\phi + C_\phi
\]

Observer disk integrals close with \(J_\phi = C_\phi = 0\) within tolerance.

## Radius mapping

Governed equivalent radius \(R_{\mathrm{eq}} = \sqrt{A_{\mathrm{interior}}/\pi}\).
Synthetic add/remove mass moves occupied area symmetrically. Mapping itself is valid.

## Why D-059 looked neutral

Gate 3 measured interior \(A,C\) from short shadows and integrated the existing rates.
Analytic drive classified **POSITIVE_ALL_RADII** (net \(G-L > 0\) at every tested radius).
Coupled \(dR/dt\) was **exactly 0** at every radius.

Root cause: `Simulation::enforce_structure_constraint == true` (default) sets
`apply_phi = false`, so structural synthesis/decay are virtual-ledgered but **do not
update \(\phi\)**. Productive structural biology cannot move the interface →
`STRUCTURAL_GEOMETRY_COUPLING_DEFECT` / Route G.

Candidate kinetic laws were **not** fitted (stop rule: do not propose a new law until
the primary cause is selected; geometry/execution defect forbids new kinetics).

## Frozen carrier

`D060_FROZEN_KT = 1.4346157818803311` (exact sealed D-059 best global \(k_T\)).

## Status (unchanged authorization)

- selected architecture: none
- V15: unauthorized
- structural-law change: unauthorized
- internal membrane: unauthorized
- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Stage F: not authorized
- production: `REQUIRES_REMEDIATION`

## Next directive

Repair only the structure-constraint / φ-application execution path so virtual
structural rates become interface motion — without changing biological kinetics —
then re-measure restoring-size dynamics under the fixed D-059 carrier rate.

## Run

```bash
cargo test -p chemistry-core --test d060_tests
D060_MAX_ACCEPTED=400 cargo run -p experiment-runner --release -- d060 pipeline
```

Artifacts: `digital-protocell/experiments/generated/d060/` → `/mnt/storage1tb/cache/project-artifacts/digital_cell/experiments/generated/d060/`.
