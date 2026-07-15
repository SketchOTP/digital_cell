# D-012 Conservative Stoichiometric Closure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close incomplete D-011 evidence behind an exact v1 stoichiometric gate; if v1 is nonconservative, implement and validate `membrane_metabolism_v2_conservative` through bounded Stage E with material and activation-potential proof.

**Architecture:** Extend the existing Rust chemistry engine. Shared compile-time reaction descriptors feed exact rational matrix analysis, ledger expectations, runtime-delta verification, and docs. Expensive D-011 completion runs only if v1 is conservative. V2 retains seven fields, revalidates Stages B–D, then uses the constrained-radius assay for Stage E.

**Tech Stack:** Rust (`chemistry-core`, `experiment-runner`), Cargo tests, TOML configs, JSON/Markdown governed artifacts, git tags. No new numeric dependencies.

## Global Constraints

- Project: `digital_cell`; branch: `d008-membrane-metabolic-closure`
- Exact rational conservation proof is mandatory; float RREF is supplementary only
- Shared `ReactionStoichiometry` descriptors are the sole stoichiometric source of truth
- `D011_TRANSPORT_COUPLED_BALANCE_NO_SOLUTION_CONFIRMED` requires conservative v1 + complete horizons + exhausted eligible domain + closed accounting
- If v1 is nonconservative: record `D012_NONCONSERVATIVE_V1_CONFIRMED`, classify D-011 as `D011_LONG_HORIZON_INCOMPLETE_SUPERSEDED_BY_INVALID_STOICHIOMETRY`, skip exhaustive v1 balance search
- V2 is a new scientific model; no v1 snapshot resume; Stages B–E revalidated
- Yield branch only after unit-yield diagnosis with ledger-supported overproduction; one yield at a time from `{1, 17/20, 7/10}`
- Until conservative Stage E passes: Phase 1 `PHASE1_SELF_MAINTENANCE_PARTIAL`; production `REQUIRES_REMEDIATION`
- Preserve tags: `D-008-stage-e-balance-fail`, `D-011-transport-coupled-balance-fail`, `D-011-transport-coupled-balance-fail-corrected`
- Do not amend commits `a04e098` or `7aa9c63`
- Ponytail: fewest files; no generic reaction engine; mark deliberate ceilings with `ponytail:`

## File Map

| Path | Responsibility |
| --- | --- |
| `chemistry-core/src/stoichiometry.rs` | Exact rationals, reaction descriptors, matrices, nullspaces, conservation classes |
| `chemistry-core/src/activated_metabolism.rs` | V1/V2 activation/catalyst rates and isolated deltas |
| `chemistry-core/src/membrane.rs` | Membrane synthesis/decay/detachment deltas including A/W coupling for v2 |
| `chemistry-core/src/reactions.rs` / `simulation.rs` | Structure production/decay and equation-version dispatch |
| `chemistry-core/src/accounting.rs` (+ new helpers if needed) | Material-equivalent and activation-potential observer ledgers |
| `chemistry-core/src/config.rs` | `MembraneMetabolismV2Conservative`, yields, schema version fields |
| `chemistry-core/src/candidate_identity.rs` / `snapshot.rs` | Identity hashes, snapshot non-resume for v1→v2 |
| `chemistry-core/src/d011_analysis.rs` | Classification completion evidence and conditional branch |
| `experiment-runner/src/d011.rs` / new `d012.rs` | Preservation, audit, conditional D-011, v2 stage orchestration |
| `chemistry-core/tests/d012_tests.rs` | Stoichiometry, conservation, runtime-delta, accounting, stage gates |
| `docs/d012_*.md`, `experiments/generated/d012/` | Reports and hashed artifacts |

---

### Task 1: Preservation and status normalization

**Files:**
- Create: `digital-protocell/experiments/generated/d012/preservation/manifest.json`
- Create: `scripts/d012_preservation_manifest.py` (or small Rust one-shot in runner)
- Modify: `docs/d011_candidate_report.md`, `docs/d011_balance_controllability.md`
- Modify: `.agent/CURRENT.md`, append `.agent/OUTCOMES.md` as needed
- Test: preservation self-check embedded in manifest generator; no chemistry behavior change

**Entry gate:** HEAD contains design commits `a04e098` and `7aa9c63`; tags `D-008-stage-e-balance-fail`, `D-011-transport-coupled-balance-fail`, `D-011-transport-coupled-balance-fail-corrected` exist.

**Acceptance:**
- Manifest covers Stage E `attempt_003`, D-011 `attempt_015`, `attempt_017`, related configs/sensitivity/reports/hashes
- Manifest records content hash
- Operative D-011 status corrected to `D011_LONG_HORIZON_CONFIRMATION_INCOMPLETE`
- Tag `D-011-long-horizon-incomplete` created on clean baseline
- Historical failure tags unchanged

**Failure behavior:** Stop D-012 if any required artifact/tag/commit is missing.

- [ ] **Step 1: Verify governed history**

```bash
git rev-parse HEAD
git show-ref --verify refs/tags/D-008-stage-e-balance-fail
git show-ref --verify refs/tags/D-011-transport-coupled-balance-fail
git show-ref --verify refs/tags/D-011-transport-coupled-balance-fail-corrected
test -f digital-protocell/experiments/generated/d008/stage_e_balance/attempt_003/result.json
test -f digital-protocell/experiments/generated/d011/attempt_015/result.json
test -f digital-protocell/experiments/generated/d011/attempt_017/result.json
```

Expected: all commands succeed.

- [ ] **Step 2: Generate preservation manifest and update D-011 status docs**

Hash listed artifacts; write `experiments/generated/d012/preservation/manifest.json` with source commits, paths, sha256, and aggregate content hash. Update D-011 docs to operative incomplete status without deleting quick/50k evidence.

- [ ] **Step 3: Commit and tag**

```bash
git add docs/d011_*.md digital-protocell/experiments/generated/d012/preservation scripts/d012_preservation_manifest.py .agent/CURRENT.md
git commit -m "D-012: Preserve D-011 evidence and normalize incomplete status"
git tag D-011-long-horizon-incomplete
```

---

### Task 2: Shared fixed stoichiometric descriptors

**Files:**
- Create: `digital-protocell/crates/chemistry-core/src/stoichiometry.rs`
- Modify: `digital-protocell/crates/chemistry-core/src/lib.rs`
- Test: `digital-protocell/crates/chemistry-core/tests/d012_tests.rs`

**Interfaces:**
- Produces:
  - `pub const SEVEN_FIELD_COUNT: usize = 7;`
  - `pub enum SpeciesId { Phi, C, N, F, W, A, M }`
  - `pub enum ReactionId { Activation, CatalystProduction, StructureProduction, MembraneProduction, StructureDecay, CatalystDecay, ActivatedDecay, MembraneDecay, MembraneDetachment }`
  - `pub struct Rational { num: i64, den: i64 }` with reduce/add/mul/neg/is_zero
  - `pub struct ReactionStoichiometry { reaction: ReactionId, delta: [Rational; SEVEN_FIELD_COUNT] }`
  - `pub fn v1_internal_reactions() -> &'static [ReactionStoichiometry]`
  - `pub fn v2_internal_reactions(eta_c: Rational, eta_phi: Rational, eta_m: Rational) -> [ReactionStoichiometry; 9]`

**Entry gate:** Task 1 complete.

**Acceptance:** Compile-time descriptors exist for all nine internal reactions; transport/reservoir/clearance excluded; docs map column order to `ReactionId`.

**Failure behavior:** Do not invent undocumented reactions; encode current v1 runtime meaning first, then governance docs.

- [ ] **Step 1: RED tests for descriptor presence and ordering**

```rust
#[test]
fn test_v1_descriptor_order_matches_governed_reaction_list() {
    let ids: Vec<_> = v1_internal_reactions().iter().map(|r| r.reaction).collect();
    assert_eq!(ids, vec![
        ReactionId::Activation,
        ReactionId::CatalystProduction,
        ReactionId::StructureProduction,
        ReactionId::MembraneProduction,
        ReactionId::StructureDecay,
        ReactionId::CatalystDecay,
        ReactionId::ActivatedDecay,
        ReactionId::MembraneDecay,
        ReactionId::MembraneDetachment,
    ]);
}
```

- [ ] **Step 2: Implement rational + descriptor tables; GREEN; commit**

```bash
cargo test -p chemistry-core --release --test d012_tests test_v1_descriptor_order_matches_governed_reaction_list -- --exact
git add digital-protocell/crates/chemistry-core/src/stoichiometry.rs digital-protocell/crates/chemistry-core/src/lib.rs digital-protocell/crates/chemistry-core/tests/d012_tests.rs
git commit -m "D-012: Add shared fixed stoichiometric descriptors"
```

---

### Task 3: Exact matrix and conservation analysis

**Files:**
- Modify: `stoichiometry.rs`
- Test: `d012_tests.rs`

**Interfaces:**
- Produces:
  - `pub fn stoichiometric_matrix(reactions: &[ReactionStoichiometry]) -> Vec<Vec<Rational>>`
  - `pub fn exact_rank(matrix: &[Vec<Rational>]) -> usize`
  - `pub fn left_nullspace(matrix: &[Vec<Rational>]) -> Vec<Vec<Rational>>`
  - `pub fn right_nullspace(matrix: &[Vec<Rational>]) -> Vec<Vec<Rational>>`
  - `pub fn classify_conservation(matrix: &[Vec<Rational>]) -> ConservationClass`
  - `pub fn verify_m_transpose_s_zero(m: &[Rational], s: &[Vec<Rational>]) -> bool`
  - `pub enum ConservationClass { StrictlyConservative, PartiallyConservative, NoPositiveConservationVector, InconsistentStoichiometry }`

**Entry gate:** Task 2 complete.

**Acceptance:** Exact rank/nullspace/positivity work on small hand-checked matrices; proposed vectors verify exactly; float analysis optional and secondary.

- [ ] **Step 1: RED tests**

```rust
#[test]
fn test_positive_conservation_vector_detection() {
    // identity production A->W is conservative under all-ones
}
#[test]
fn test_nonconservative_reaction_is_identified() {
    // A -> C + W has no strictly positive left null vector over productive columns alone
}
```

- [ ] **Step 2: Implement exact Gaussian elimination over rationals; GREEN; commit**

```bash
cargo test -p chemistry-core --release --test d012_tests test_positive_conservation_vector_detection test_nonconservative_reaction_is_identified -- --exact
git commit -m "D-012: Add exact stoichiometric conservation analysis"
```

---

### Task 4: V1 formal audit

**Files:**
- Create: `docs/d012_v1_stoichiometric_audit.md`
- Create: `digital-protocell/experiments/generated/d012/v1_stoichiometric_audit/`
- Modify: runner or analysis helper to dump matrices
- Test: `d012_tests.rs`

**Required tests:**
- `test_v1_stoichiometric_matrix_matches_reactions`
- `test_v1_positive_conservation_vector_search`
- `test_v1_nonconservative_productive_reaction_detection`
- `test_field_ledgers_can_close_while_total_stoichiometry_fails`

**Entry gate:** Task 3 complete.

**Acceptance:** Audit artifact states one of the four conservation classes. If nonconservative, primary finding `D012_NONCONSERVATIVE_V1_CONFIRMED` and Tag `D-012-stoichiometric-audit`.

**Failure behavior:** If audit inconclusive (`D012_STOICHIOMETRIC_AUDIT_INCONCLUSIVE`), stop.

- [ ] **Step 1: RED tests that bind descriptors to expected v1 columns and class**
- [ ] **Step 2: Encode actual v1 runtime columns; emit audit; GREEN**
- [ ] **Step 3: Commit and tag**

```bash
git commit -m "D-012: Add formal stoichiometric conservation audit"
git tag D-012-stoichiometric-audit
```

**Scientific branch after this task:**
- If nonconservative → skip Task 5 expensive run; mark D-011 superseded; continue Task 6.
- If conservative → execute Task 5 fully before Task 6.

---

### Task 5: Conditional D-011 completion branch

**Files:**
- Modify: `d011_analysis.rs`, `experiment-runner/src/d011.rs`
- Create: `experiments/generated/d012/d011_long_horizon/`, `d011_solver_completion/`
- Create: `docs/d012_d011_definitive_closure.md`
- Update append-only: `docs/d011_candidate_report.md`, `docs/d011_balance_controllability.md`
- Test: `d011_tests.rs`, `d012_tests.rs`

**Required tests:**
- `test_d011_full_horizon_requirement`
- `test_no_solution_requires_solver_domain_exhaustion`
- `test_not_converged_cannot_prove_no_solution`
- Existing four-rate mutability/hash tests must pass

**Entry gate:** Task 4 conservation class is `STRICTLY_CONSERVATIVE`.

**Acceptance:** Radii 18/22/26 to 200k/windows; max 4 rounds/5 candidates; one definitive D-011 class from the authorized enum.

**Failure behavior:** Incomplete evidence yields `D011_LONG_HORIZON_INCOMPLETE`, never confirmed no-solution.

- [ ] **Step 1: Harden classification predicates with completion evidence**
- [ ] **Step 2: Run Stage E candidate + corrected candidate horizons**
- [ ] **Step 3: Complete bounded solver only with valid sensitivity**
- [ ] **Step 4: Docs, commit, tag `D-011-definitive-closure`**

If skipped because Task 4 nonconservative:

- [ ] **Step S: Record supersession classification and continue to Task 6 without 200k search**

---

### Task 6: V2 equation and identity versioning

**Files:**
- Modify: `config.rs`, `candidate_identity.rs`, `snapshot.rs`, `fields.rs` as needed
- Test: `d012_tests.rs`, affected snapshot tests

**Interfaces:**
- Add `EquationVersion::MembraneMetabolismV2Conservative` serde `"membrane_metabolism_v2_conservative"`
- Add stoichiometric schema version constant `2`
- Yield params `eta_c`, `eta_phi`, `eta_m` with validation `0 < η ≤ 1`

**Required tests:**
- `test_v2_equation_version`
- `test_v2_snapshot_rejects_v1_resume`
- `test_yield_cannot_exceed_one`

**Entry gate:** Task 4 complete; Task 5 complete or superseded.

- [ ] **Step 1: RED version/snapshot/yield tests**
- [ ] **Step 2: Implement enum + hash/schema wiring; GREEN; commit**

```bash
git commit -m "D-012: Add membrane_metabolism_v2_conservative identity"
```

---

### Task 7: V2 runtime reaction deltas

**Files:**
- Modify: `activated_metabolism.rs`, `membrane.rs`, `reactions.rs`/`simulation.rs`
- Export isolated delta helpers used by tests
- Test: `d012_tests.rs`

**Governed stoichiometry (unit yield):**
- Activation: `N+F → A+W`
- Catalyst: `A → C`
- Structure: `A → φ`
- Membrane: `A → M`
- Turnovers/detachment: `φ|C|A|M → W`

**Required tests:**
- `test_v2_activation_stoichiometry`
- `test_v2_catalyst_yield_stoichiometry`
- `test_v2_structure_yield_stoichiometry`
- `test_v2_membrane_yield_stoichiometry`
- `test_v2_turnover_converts_to_waste`
- `test_runtime_*_delta_matches_matrix` for all nine reactions

**Entry gate:** Task 6 complete.

- [ ] **Step 1: RED runtime-vs-descriptor tests**
- [ ] **Step 2: Implement specialized rate updates matching descriptors; GREEN**
- [ ] **Step 3: Commit**

```bash
git commit -m "D-012: Implement conservative membrane metabolism v2"
```

---

### Task 8: Material-equivalent accounting

**Files:**
- Modify: accounting modules / simulation observer hooks
- Artifacts: `experiments/generated/d012/accounting/`
- Test: `d012_tests.rs`

**Required identity:**

```text
observed total change = reservoir input - waste clearance + numerical correction
```

Tolerance controlled tests: relative residual ≤ `1e-6`.

**Required tests:**
- `test_v2_total_change_equals_boundary_exchange`
- `test_v2_waste_clearance_is_explicit_output`
- `test_closed_v2_network_does_not_create_material`
- `test_v2_membrane_detachment_converts_to_waste`

**Entry gate:** Task 7 complete.

- [ ] Implement LEDGER + RED/GREEN tests; commit `D-012: Add material-equivalent accounting`

---

### Task 9: Activation-potential accounting

**Files:**
- Extend stoichiometry/accounting docs and observer
- Doc: declare weights in `docs/d012_conservation_proof.md`

**Initial justified weights (exact):**
- `e_F = 1`, `e_A = 1`
- Optional component potentials start at 0 unless later gate requires otherwise
- Interpretation: `N` material substrate; `F` fuel substrate; activation transfers potential `F→A`; productive chemistry consumes `A`; waste inactive

**Required tests:**
- `test_closed_v2_network_does_not_create_activation_potential`
- `test_fuel_is_only_external_activation_potential_source`
- `test_waste_cannot_reactivate_spontaneously`

**Entry gate:** Task 8 complete.

- [ ] Implement potential ledger + controls; commit `D-012: Add activation-potential accounting`

---

### Task 10: V2 conservation gate (hard stop)

**Files:**
- Aggregate tests in `d012_tests.rs`
- Artifacts: `v2_stoichiometric_matrix/`
- Docs: `docs/d012_conservative_network.md`, `docs/d012_conservation_proof.md`

**Required tests:**
- `test_v2_each_internal_reaction_is_conservative`
- `test_v2_has_strictly_positive_conservation_vector` / `test_v2_positive_conservation_vector`
- `test_v2_total_accounting_closes`
- All Task 7–9 matching tests

**Entry gate:** Tasks 7–9 complete.

**Acceptance:** All conservation predicates pass before any Stage B–E spatial experiment begins.

**Failure behavior:** Stop with `D012_ACCOUNTING_FAILURE` or `D012_FAIL`; do not start Stages B–E.

- [ ] Run full conservation suite; commit `D-012: Close v2 conservation gate`

---

### Task 11: Stage A transport equivalence

**Files:**
- Focused regression in `d008_tests.rs` / `d012_tests.rs`
- No full sweep unless transport code changed

**Required test:** `test_v2_transport_matches_v1`

**Entry gate:** Task 10 pass.

- [ ] RED/GREEN equivalence; commit if code changes needed

---

### Task 12: Stage B localization

**Files:** runner stage B path under equation v2; artifacts `v2_stage_b_localization/`
**Required:** localization ≥ 0.90; active production/loss; bounded M
**Test:** `test_v2_membrane_localization`

**Entry gate:** Task 11 pass.

**Failure behavior:** stop before Stage C.

- [ ] Governed Stage B experiment; commit evidence `D-012: Validate conservative Stage B`

---

### Task 13: Stage C metabolism

**Files:** zero-D reactor under v2; artifacts `v2_stage_c_metabolism/`
**Required:** activation depends on C/N/F; catalyst on A; bounded fields; material accounting closes
**Test:** `test_v2_metabolic_reactor_bounded`

**Entry gate:** Task 12 pass.

- [ ] Governed Stage C; commit `D-012: Validate conservative Stage C`

---

### Task 14: Stage D fixed compartments

**Files:** R=16/24/32 v2; artifacts `v2_stage_d_fixed_compartment/`
**Required:** prior retention/flux/scaling gates; transport active
**Test:** `test_v2_fixed_compartment_retention`
**Doc:** `docs/d012_v2_stage_validation.md`

**Entry gate:** Task 13 pass.

- [ ] Governed Stage D; commit `D-012: Validate conservative Stage B-D foundations` if packaging B–D together otherwise Stage D commit alone

---

### Task 15: Transport-coupled Stage E reference

**Files:** reuse constrained-radius assay with v2; artifacts `v2_stage_e_reference/`
**Radii:** 14/18/22/26/30/34
**Required tests:**
- `test_v2_stage_e_requires_quasi_steady_state`
- `test_v2_stage_e_requires_all_four_balances`
- `test_v2_stage_e_requires_restoring_radius`
- `test_v2_stage_e_requires_resource_throughput`
- `test_v2_stage_e_requires_total_conservation`

**Entry gate:** Task 14 pass.

**Acceptance:** reference metrics and convergence classifications recorded even if no pass yet.

- [ ] Progressive analytical calibration of `k_activation → k_rep → k_membrane → k_structure` with 0.75/1.00/1.25 screens, not Cartesian product
- [ ] Commit `D-012: Run transport-coupled conservative Stage E`

---

### Task 16: Bounded four-rate solver

**Files:** sensitivity/solver under broader v2 bounds 0.25–4.00 global, 0.67–1.50 per round; max 4 rounds / 5 candidates
**Artifacts:** `v2_sensitivity/`, `v2_joint_candidates/`
**Frozen during first sequence:** transport, diffusion, turnover, reservoirs, ICs, yields

**Entry gate:** Task 15 reaches quasi-steady + accounting pass + valid sensitivity.

**Failure behavior:** if exhausted without overlap, later Task 19 may conclude `D012_CONSERVATIVE_NETWORK_NO_JOINT_FIXED_POINT`.

- [ ] Execute solver; commit candidate artifacts

---

### Task 17: Conditional yield branch

**Files:** yield candidates under `v2_yield_candidates/`
**Tests:**
- `test_yield_branch_changes_one_component`
- `test_underproduced_component_yield_is_not_reduced`

**Entry gate:** Task 16 produces converged ledger diagnosis of persistent overproduction satisfying all required evidence.

**Failure behavior:** skip if unit-yield path already balances or diagnosis unsupported.

- [ ] At most three candidates; one yield change each; commit if used

---

### Task 18: Robust overlap and restoring radius

**Files:** `v2_robust_overlap/`
**Required:** ±2% rate perturbation; ±5% initial C/A/M; restoring geometry around center; throughput active; conservation passes

**Entry gate:** candidate claiming joint balance from Tasks 15–17.

- [ ] Execute robustness; classify provisional Stage E pass/fail

---

### Task 19: Final reports, manifests, commits, and tags

**Files:**
- Create: all remaining `docs/d012_*.md`
- Update append-only D-008/D-011 reports
- Create: `experiments/generated/d012/manifest.json`
- Tags: `D-012-conservative-network-pass` or `D-012-conservative-network-fail`
- Optional Stage E pass revision tag `D-008-stage-e-conservative-balance-pass` without moving old fail tag

**Required report fields:** scientific conclusion; subsidiary conclusions; D-008 status; Phase 1; production verdict; highest-value remaining blocker; next bounded mechanism; commit hash; tags; Memory/MCP outcome.

**Entry gate:** Tasks through applicable branch complete.

- [ ] Write final docs/manifest
- [ ] Commit `D-012: Close conservative stoichiometric repair`
- [ ] Tag pass or fail
- [ ] Update `.agent/CURRENT.md` / `.agent/OUTCOMES.md`
- [ ] Record Mimir outcome when reachable

---

## Spec coverage checklist

| Spec requirement | Task |
| --- | --- |
| Preserve before mutation | 1 |
| Audit before expensive D-011 | 4 before 5 |
| Shared descriptors | 2,7 |
| Exact rational proof | 3,4,10 |
| Material + activation ledgers | 8,9 |
| V1 non-equivalence to v2 | 6 |
| Conservation gate before Stage B–E | 10 |
| Stage A→E sequence | 11–18 |
| Bounded rates + yield evidence | 15–17 |
| Production readiness fields | 19 |

## Placeholder scan

No TBD/TODO deferred work remains in the plan; uncertain runtime v1 columns are resolved in Task 4 by matching descriptors to actual isolated deltas rather than aspirational docs.
