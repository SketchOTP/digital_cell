# CURRENT.md

## Active directive
- ID: D-20260721-d059-viable-size-basin-membrane-area-review
- Project directive: D-059
- Goal: One global k_T viable-size basin vs metabolically generated membrane area
- Status: done
- Acceptance: met — Route L; contiguous R6–R14 @10k; NEUTRAL_SIZE_MANIFOLD; no V15
- Touched files: d059_analysis.rs, d059_tests.rs, d059.rs, main.rs, lib.rs, docs/d059_*, experiments/generated/d059, .agent/*
- Next action: next directive reviews structural growth law / size feedback; next_execution_started=false

## Repo facts needed now
- Primary: `D059_EXTERNAL_CARRIER_SIZE_LIMIT_NO_RESTORING_BASIN` (Route L)
- Matched exponents: p_M≈2.00, p_T≈1.00 (`D058_RADIUS_EXPONENT_CONFOUNDED`)
- Best global k_T≈1.435; viable band R6–R14 @10000 steps
- Restoring: `NEUTRAL_SIZE_MANIFOLD` (Gates 5–6 skipped)
- Artifacts: `digital-protocell/experiments/generated/d059` → `/mnt/storage1tb/.../d059`
- Unrelated untracked/modified Cursor rules + AGENTS.md — leave uncommitted

## Last validation
- Command: `cargo test -p chemistry-core --test d059_tests`; `D059_MAX_ACCEPTED=10000 cargo run -p experiment-runner --release -- d059 pipeline`
- Result: 14/14 PASS; primary Route L

## Open blockers
- None for D-059 diagnostic; Stage E remains BLOCKED_NOT_RECOVERED
