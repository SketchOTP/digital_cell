# D-012 Conservative Stoichiometric Closure Design

## Scope

D-012 closes the incomplete D-011 transport-coupled balance search, audits the
seven-field v1 chemistry for conserved chemical equivalents, and, if v1 is
nonconservative, introduces and validates
`membrane_metabolism_v2_conservative`.

The work remains limited to `φ, C, N, F, W, A, M`. It does not add later-life
features or a central behavior controller.

## Execution boundaries

D-012 is executed as three sequential governed phases. Each phase has an
immutable artifact boundary and a focused commit. Gate results determine
whether dependent work proceeds.

### Phase A — Preservation and v1 stoichiometric gate

1. Verify the governed D-008/D-011 commits, tags, reports, configurations,
   sensitivity matrices, and binary artifacts.
2. Generate a content-addressed preservation manifest covering the named
   Stage E and D-011 attempts. Commit outstanding D-011 status/report updates,
   establish a clean baseline, and create `D-011-long-horizon-incomplete`
   without moving historical tags.
3. Construct the exact 7×9 internal-reaction matrix for v1. Reservoir input,
   clearance, diffusion, and transport are excluded.
4. Report rank, left and right nullspaces, nonnegative and strictly positive
   conservation vectors, and per-reaction chemical-equivalent residuals.
5. Branch on this audit before any additional expensive D-011 run.

### Phase B — Conditional D-011 closure or conservative v2

If v1 is conservative:

1. Audit the runner's four authorized adjustable rates, 4×4
   central-difference sensitivity matrix, candidate identity,
   constrained-radius state evolution, observer-only constraint flux, and
   old-state membrane transport.
2. Run the failed Stage E and corrected D-011 candidates at R=18/22/26 for up
   to 200,000 accepted substeps with 10,000-step windows and three consecutive
   qualifying windows.
3. If sensitivity is valid, execute at most four bounded correction rounds and
   five total candidates. Candidate order is center R=22, neighbors R=18/26,
   then the full radius grid only for a promising center.
4. Produce exactly one governed D-011 classification. `NOT_CONVERGED` never
   proves overlap, zero flow, or domain exhaustion.

If v1 is nonconservative:

1. Record `D012_NONCONSERVATIVE_V1_CONFIRMED`.
2. Classify D-011 as
   `D011_LONG_HORIZON_INCOMPLETE_SUPERSEDED_BY_INVALID_STOICHIOMETRY`.
3. Preserve quick and 50,000-step results as historical evidence, but skip
   exhaustive v1 rate-domain execution.
4. Add `MembraneMetabolismV2Conservative` using the existing seven field
   buffers. Productive reactions consume one `A` equivalent and split it
   between product and `W` according to yields bounded by `(0, 1]`.
5. Convert all v2 turnover and membrane detachment into `W`. Keep reservoir
   exchange and waste clearance as the only material boundary terms.
6. Prove every v2 internal reaction conserves unit chemical-equivalent weight
   and cannot create activation potential before any governed v2 experiment
   runs.

### Phase C — V2 validation and joint balance

1. Prove Stage A transport equivalence without repeating the full transport
   sweep unless shared transport code changed.
2. Rerun Stage B localization, Stage C zero-dimensional metabolism, and Stage D
   fixed compartments at R=16/24/32.
3. Estimate the four productive rates from constrained-radius ledgers in the
   required order. Evaluate progressive 0.75×/1.00×/1.25× candidates rather
   than a Cartesian product.
4. Run the transport-coupled Stage E assay at R=14/18/22/26/30/34 with the same
   three-window convergence rule and total-conservation gate.
5. Use the bounded four-rate solver only after a valid converged sensitivity
   matrix exists. It permits four rounds and five candidates, with global
   bounds 0.25×–4.00× and per-round bounds 0.67×–1.50×. Yields remain one
   during the first solver sequence.
6. Enter the yield branch only when a product remains persistently
   overproduced after rate calibration, changing its rate alone would destroy
   another required balance or turnover, and the ledger predicts the required
   yield reduction. Change one yield per candidate among 1, 17/20, and 7/10.
7. A pass requires four-component balance, a restoring radius, active boundary
   throughput, ±2% rate robustness, ±5% initial C/A/M robustness, and closed
   accounting.

## Code organization

The existing simulation engine and constrained-radius assay remain canonical.
D-012 extends them rather than introducing a second chemistry engine.

- `chemistry-core/src/stoichiometry.rs` owns fixed compile-time
  `ReactionStoichiometry` descriptors, exact coefficients, species/reaction
  order, matrix construction, rank/nullspace analysis, positivity detection,
  and reaction residuals.
- Existing metabolism, membrane, simulation, accounting, snapshot, and
  candidate-identity modules dispatch on equation version where behavior
  differs.
- Existing D-011 analysis is hardened for completion evidence and bounded
  multi-round solving.
- A D-012 runner orchestrates resumable phases and writes each expensive assay
  result before starting the next.

Specialized rate functions and optimized field updates remain. Tests compare
each isolated runtime delta with the same governed descriptor used by formal
analysis, ledger expectations, and documentation. A duplicated audit-only
matrix is forbidden.

No generalized reaction-network framework or new dependency is introduced.

## Stoichiometric analysis

Matrices use rows `(φ, C, N, F, W, A, M)` and the governed nine-reaction column
order. Coefficients use a small reduced rational type backed by signed integers
and greatest-common-divisor normalization. Governed yields are exact:
`1`, `17/20`, and `7/10`.

Exact Gaussian elimination reports matrix rank and nullspace bases. Strict
positivity is tested on the exact left-nullspace cone after homogeneous
normalization. Every proposed conservation vector is verified with exact
rational arithmetic, and every per-reaction residual is exactly zero or an
explicit nonzero rational. Floating-point projections may be reported only as
supplementary numerical diagnostics.

V2 unit yields have the all-ones conservation vector. Lower permitted yields
remain conservative because the unconverted fraction goes to `W`.

## Versioning and identity

`MembraneMetabolismV2Conservative` has an explicit stoichiometric schema
version `2`. The field schema remains seven-field because field layout is
unchanged.

Every v2 candidate and artifact includes equation, field-schema,
stoichiometric-schema, candidate, candidate-hash, and configuration-hash
identity. V1 snapshots may be inspected but restoration under v2 parameters is
rejected.

Every v2 report records scientific non-equivalence:

- v1 balance evidence is historical only;
- v1 and v2 candidate hashes are not comparable;
- v1 snapshots cannot initialize governed v2 runs;
- v1 acceptance does not transfer to v2;
- affected D-008 Stages B–E require v2 revalidation.

## Accounting

Existing field ledgers remain intact. Two observer-only scientific ledgers are
added.

### Material-equivalent ledger

A strictly positive vector `m` must satisfy `mᵀS = 0`. The total
material-equivalent ledger records:

```text
observed total change
= reservoir input
- waste clearance
+ numerical correction
```

Internal reaction extents contribute zero under v2. Membrane detachment is an
internal `M → W` conversion and therefore cannot appear as deletion.

Controlled tests require relative residual at most `1e-6`. Long runs use the
established governed spatial tolerance and report both absolute and relative
residuals.

### Activation-potential ledger

`F` is a simulated fuel substrate carrying usable chemical potential, not an
unaccounted abstract-energy field. `N` supplies material substrate, activation
transfers usable potential from `F` into `A`, and `W` contains spent material
and energetic products.

The governed observer defines:

```text
E_chemical = e_F F + e_A A + declared component potentials
```

The initial weights are chosen and documented so activation transfers rather
than creates potential, productive reactions consume `A` potential, and
turnover/waste formation cannot increase potential. In a closed system,
chemistry cannot increase total activation potential without consuming `F`.
Fuel import is the only external source. No v2 reaction converts `W` into `F`
or `A`.

## V2 conservation gate

Before any governed v2 spatial experiment:

- every isolated internal reaction conserves exact material equivalents;
- every isolated runtime delta equals its governed reaction descriptor;
- membrane detachment converts `M` to `W`;
- no internal reaction creates or deletes material;
- closed-reactor material remains constant;
- boundary-coupled material changes only through reservoir input, waste
  clearance, and numerical correction;
- closed chemistry cannot create activation potential;
- a closed reactor with `F=0` cannot increase activation potential;
- productive chemistry stops after `F` and `A` are exhausted;
- waste cannot spontaneously reactivate.

## Runner and artifact safety

Each expensive run writes a result atomically before the next begins. A job
ledger records pending, running, completed, invalid, and rejected states.
Restarting reuses only results whose complete identity and artifact hash match.
Historical attempts are never overwritten.

Candidate expansion is sequential because later work depends on the center and
neighbor gates. Independent radii may be executed as separate deterministic
jobs, but candidate selection remains ordered.

Scientific classification checks explicit completion evidence:

- convergence windows evaluated;
- required radii complete or properly rejected;
- solver bounds and stopping reason recorded;
- candidate count and rounds recorded;
- accounting and termination valid.

Missing evidence produces incomplete, unresolved, accounting-failure, or
numerical-failure classifications, never a definitive no-solution result.

## Verification strategy

Implementation uses test-first slices:

1. D-011 horizon, mutability, identity, and domain-exhaustion invariants.
2. Shared exact reaction descriptors, v1 matrix construction,
   nullspace/positivity detection, and proof that field ledgers can close while
   total stoichiometry fails.
3. V2 reaction deltas, yields, turnover, detachment, per-reaction conservation,
   and isolated runtime-delta equivalence.
4. Equation/snapshot/candidate identity, material accounting, and
   activation-potential controls.
5. Stage A equivalence and Stage B–D acceptance gates.
6. Stage E convergence, four-balance, restoring-radius, throughput,
   conservation, bounded-solver, yield, and robustness gates.

After targeted tests pass, affected D-008, D-011, accounting, transport,
snapshot, and legacy-equivalence suites run in release mode. Governed
experiments begin only after the v2 conservation gate passes.

## Failure behavior

- Invalid preservation evidence stops all implementation.
- `D011_LONG_HORIZON_INCOMPLETE` means the protocol remains scientifically
  relevant but unfinished.
- `D011_LONG_HORIZON_INCOMPLETE_SUPERSEDED_BY_INVALID_STOICHIOMETRY` means the
  exact v1 audit invalidated the network before exhaustive balance completion.
- `D011_TRANSPORT_COUPLED_BALANCE_NO_SOLUTION_CONFIRMED` requires conservative
  v1, complete required horizons and eligible rounds, exhausted bounds, closed
  accounting, and valid terminal classifications.
- Nonconservative v1 is permanently blocked from Stage F and skips expensive
  D-011 completion.
- Failed v2 Stage B, C, or D stops dependent Stage E work.
- A conservative, fully exhausted v2 search without restoring overlap rejects
  the seven-field model but does not introduce an eighth field inside D-012.

## Completion evidence

The final report lists preservation hashes, exact run counts and horizons,
solver rounds, matrices and conservation vectors, stage results, accounting,
tests, artifact manifest, performance, deviations, commits, tags, one primary
D-012 conclusion, and all subsidiary findings. Intermediate phase reports are
explicitly partial until their governed evidence is complete.

Every terminal report also states the D-008 status, Phase 1 status, production
verdict, highest-value remaining blocker, and next bounded mechanism. Until
conservative Stage E passes, Phase 1 remains
`PHASE1_SELF_MAINTENANCE_PARTIAL` and production remains
`REQUIRES_REMEDIATION`; later-life feature work is unauthorized.
