# CURRENT.md

## Active directive
- ID: D-20260725-d094-distributed-autocatalytic-set-heredity
- Project directive: D-094
- Goal: Seal corrected D-093; audit zero-gen; implement distributed autocatalytic-set heredity Gates 0–10
- Status: started
- Acceptance: One exact D094_* primary; D-093 tagged UNTESTABLE_ZERO_GENERATION; Phase3 only if adaptation+reversal
- Touched files: d093_* (seal), autocatalytic_*, d094_*, material_mesh, mesh_*, docs/d094_*, .agent/*
- Next action: Finish full D-093 pipeline reproduce → commit+tag → zero-gen audit

## Repo facts needed now
- D-093 uncommitted; conclusion corrected to UNTESTABLE_ZERO_GENERATION
- Ancestor: 381ac64; branch phase2-growth-division-inheritance
- μ_E = 0.0089 frozen from D-093 measured mismatch
- 1TB mount emergency_ro — local NVMe artifacts only

## Last validation
- Command: cargo test -p chemistry-core --release --test d093_tests; d093 repair-info
- Result: 5/5 PASS; primary UNTESTABLE_ZERO_GENERATION

## Open blockers
- Full D-093 pipeline rerun in progress (local d093_rerun)
- /mnt/storage1tb I/O error (emergency_ro)

## Mimir V2
- project: 7bff443192353517
- task: 5fa2c781627d4246a213acdfc95a0f7a version 2
- retrieval.session_id: cf99fb2bf0dc4960a01d73e71f14222b
