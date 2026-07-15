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

### Phase A — Preservation and definitive D-011 closure

1. Verify the governed D-008/D-011 commits, tags, reports, configurations,
   sensitivity matrices, and binary artifacts.
2. Generate a content-addressed preservation manifest covering the named
   Stage E and D-011 attempts. Commit outstanding D-011 status/report updates,
   establish a clean baseline, and create `D-011-long-horizon-incomplete`
   without moving historical tags.
3. Audit the runner's four authorized adjustable rates, 4×4 central-difference
   sensitivity matrix, candidate identity, constrained-radius state evolution,
   observer-only constraint flux, and old-state membrane transport.
4. Run the failed Stage E and corrected D-011 candidates at R=18/22/26 for up
   to 200,000 accepted substeps with 10,000-step windows and three consecutive
   qualifying windows.
5. If sensitivity is valid, execute at most four bounded correction rounds and
   five total candidates. Candidate order is center R=22, neighbors R=18/26,
   then the full radius grid only for a promising center.
6. Produce exactly one governed D-011 classification. `NOT_CONVERGED` never
   proves overlap, zero flow, or domain exhaustion.

### Phase B — V1 audit and conservative v2

1. Construct the exact 7×9 internal-reaction matrix for v1. Reservoir input,
   clearance, diffusion, and transport are excluded.
2. Report rank, left and right nullspaces, nonnegative and strictly positive
   conservation vectors, and per-reaction chemical-equivalent residuals.
3. If v1 lacks a strictly positive conservation vector, record
   `D012_NONCONSERVATIVE_V1_CONFIRMED` and prevent v1 Stage F advancement.
4. Add `MembraneMetabolismV2Conservative` using the existing seven field
   buffers. Productive reactions consume one `A` equivalent and split it
   between product and `W` according to yields bounded by `(0, 1]`.
5. Convert all v2 turnover and membrane detachment into `W`. Keep reservoir
   exchange and waste clearance as the only material boundary terms.
6. Prove every v2 internal reaction conserves unit chemical-equivalent weight
   before any governed v2 experiment runs.

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
   matrix exists. Yields remain one during the first solver sequence.
6. Enter the yield branch only for ledger-supported persistent overproduction,
   changing one yield per candidate.
7. A pass requires four-component balance, a restoring radius, active boundary
   throughput, ±2% rate robustness, ±5% initial C/A/M robustness, and closed
   accounting.

## Code organization

The existing simulation engine and constrained-radius assay remain canonical.
D-012 extends them rather than introducing a second chemistry engine.

- `chemistry-core/src/stoichiometry.rs` owns fixed species/reaction order,
  matrix construction, rank/nullspace analysis, positivity detection, and
  reaction residuals.
- Existing metabolism, membrane, simulation, accounting, snapshot, and
  candidate-identity modules dispatch on equation version where behavior
  differs.
- Existing D-011 analysis is hardened for completion evidence and bounded
  multi-round solving.
- A D-012 runner orchestrates resumable phases and writes each expensive assay
  result before starting the next.

No generalized reaction-network framework or new dependency is introduced.

## Stoichiometric analysis

Matrices use rows `(φ, C, N, F, W, A, M)` and the governed nine-reaction column
order. Rank and nullspace use deterministic reduced row-echelon elimination.

Strict positivity is tested on the left-nullspace cone after homogeneous
normalization to `m_i ≥ 1`. Because the problem is fixed at seven species, the
feasibility search enumerates bounded active sets rather than adding a linear
programming dependency. Every reported vector is verified by recomputing
`mᵀS` within the audit tolerance.

V2 unit yields have the all-ones conservation vector. Lower permitted yields
remain conservative because the unconverted fraction goes to `W`.

## Versioning and identity

`MembraneMetabolismV2Conservative` has an explicit stoichiometric schema
version. The field schema remains seven-field because field layout is
unchanged.

Every v2 candidate and artifact includes equation, field-schema,
stoichiometric-schema, candidate, candidate-hash, and configuration-hash
identity. V1 snapshots may be inspected but restoration under v2 parameters is
rejected.

## Accounting

Existing field ledgers remain intact. A parallel total chemical-equivalent
ledger records:

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
2. V1 matrix construction, nullspace/positivity detection, and proof that
   field ledgers can close while total stoichiometry fails.
3. V2 reaction deltas, yields, turnover, detachment, and per-reaction
   conservation.
4. Equation/snapshot/candidate identity and total boundary accounting.
5. Stage A equivalence and Stage B–D acceptance gates.
6. Stage E convergence, four-balance, restoring-radius, throughput,
   conservation, bounded-solver, yield, and robustness gates.

After targeted tests pass, affected D-008, D-011, accounting, transport,
snapshot, and legacy-equivalence suites run in release mode. Governed
experiments begin only after the v2 conservation gate passes.

## Failure behavior

- Invalid preservation evidence stops all implementation.
- Invalid D-011 numerical/accounting evidence receives the corresponding
  non-definitive classification; the v1 stoichiometric audit still proceeds.
- Nonconservative v1 is permanently blocked from Stage F.
- Failed v2 Stage B, C, or D stops dependent Stage E work.
- A conservative, fully exhausted v2 search without restoring overlap rejects
  the seven-field model but does not introduce an eighth field inside D-012.

## Completion evidence

The final report lists preservation hashes, exact run counts and horizons,
solver rounds, matrices and conservation vectors, stage results, accounting,
tests, artifact manifest, performance, deviations, commits, tags, one primary
D-012 conclusion, and all subsidiary findings. Intermediate phase reports are
explicitly partial until their governed evidence is complete.
