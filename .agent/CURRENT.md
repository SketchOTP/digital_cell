# CURRENT.md

## Active directive
- ID: D-20260813-digital-cell-prior-art-integration-rebase
- Project directive: DC-SR-001
- Goal: Reconcile Digital Cell strategy with prior art while preserving certified science and freezing D-094R
- Status: complete — repository synchronized, backup verified, strategy branch/tag created, documents recorded
- Acceptance: certified core preserved; no external code or scientific behavior changes; GitHub synchronization and all strategy records verified
- Touched files: docs/strategy/* and .agent/*; Git refs; no organism source
- Next action: await DC-SR-002 External ALife Implementation Audit

## Repo facts needed now
- Start a6d574f / tag D-096-finite-allocation-physiology-fail / branch phase2-growth-division-inheritance
- D-096 manifest SHA-256 898bcf7cafdfad77017f60a5ea8a9f45cdfe7a3f9ed69bd6a0c90d2106dfc0f4
- D-096 result file SHA-256 4c3f14bae72c97b874c84cbf4a1295589735ee618fdbe9f6633eeb77de1c594c
- Reproduced root cause candidate: reserve_schema_load_ok omits D-096 equation, so reserve/growth fail closed
- D-096 artifacts are immutable; Gates 6–10 remain unexecuted

## Last validation
- Command: d097 tests/runner; d096 focused; D-087 metrics; D-088–D-095 regressions; artifact guards
- Result: 3/3; 0/0 with 19 filtered; 8/8; 4/4; 50/50 PASS; classification/manifest PASS

## Open blockers
- Mimir V2 lifecycle tools unavailable in current tool surface; legacy memory tools do not satisfy required lifecycle
- D-008 test-only non-exhaustive SnapshotFields matches remain a separate repair candidate

## Session constraints
- No H/B changes, budget/cost/mutation changes, heredity/selection/adaptation/reversal, production transport change, or Phase 3
- Mimir V2: BLOCKED; do not claim context/evidence/close-out success

## DC-SR-002 current state
- Directive: External Artificial-Life Implementation Audit
- Branch/entry: `strategy/prior-art-integration-rebase` at `24d57b474843da252efe9907e7cba510d11affb2`
- Scope: source-pinned audit only; no Digital Cell scientific/runtime changes, dependencies, D-094 resume, D-095 work, or GPU migration
- Tier 1: `TIER1_EVOLUTION_AUDIT_COMPLETE`; Tier 2: `TIER2_WORLD_GPU_AUDIT_COMPLETE`
- Recommendation: keep D-094 frozen; build a thin Digital Cell-owned evolution harness next; defer GPU and discovery sidecars
- Evidence root: `docs/strategy/external_alife_audit.json` and `docs/strategy/external_audit/`
- External cache: `/mnt/storage1tb/cache/prior-art/digital_cell/` (outside repository)

## DC-SR-003 current state
- Directive: Modular Evolution and Ecology Harness
- Branch: `strategy/modular-evolution-harness`, based on `cf7d311257ff92b7502b7f1a4ecc81c2e73c05ae`
- Scope: isolated harness contracts and synthetic controls; no D-094 execution, no biology changes, no external dependencies
- Toolchain: Atlas has no Rust/Cargo/rustup/cmake on PATH; local rustup has no configured toolchain; runtime verification is blocked unless a sanctioned toolchain appears
- D-094: translated only, execution unauthorized until SR-004
- New boundary: `digital-protocell/crates/evolution-harness/` depends on `chemistry-core`; reverse dependency forbidden

## DC-SR-001 state
- Starting HEAD: 2a54b4170b0cc316b63f6aee1339ed58d449da26
- Backup: /mnt/storage1tb/backups/digital_cell/digital_cell_pre_strategy_rebase_20260813.bundle; verified; SHA-256 281c53ab44144f9d8457150321444df0f89c3e2c4f545e590a77835c12fd782d
- Strategy branch: strategy/prior-art-integration-rebase
- Preservation tag: pre-prior-art-integration-rebase-20260813
- D-094R: D094R_PRESERVED_PENDING_PRIOR_ART_REBASE
- Scientific code changed: no
