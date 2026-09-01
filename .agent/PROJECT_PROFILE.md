# Project Profile

## Lifecycle

- Status: `ADOPTED`
- Last verified: `2026-08-31T17:53:19-04:00`

## Identity

- Project name or identifier: Digital Cell
- Purpose: A self-maintaining digital protocell developing toward a persistent embodied digital lifeform.
- Repository root: `/home/sketch/Projects/digital_cell-m1-baseline`
- Verified remote: `git@github.com:SketchOTP/digital_cell.git`
- Maturity or current phase: M1 closed/frozen; M2 ENTRY-001 actuator, ENTRY-005 target-free intrinsic exploration, ENTRY-011 metabolically live in-contact composition, and the accepted ENTRY-006 through ENTRY-012 boundaries are established; local ENTRY-013 records `M2_SEARCH_REACH_POLARITY_DECAY_OR_HOMOGENIZATION_CONFIRMED` with exact-head Linux CI pending. Autonomous resource acquisition remains not established.

## Languages and runtimes

- Rust workspace under `digital-protocell`; Linux is the target runtime.
- Markdown, JSON, shell, and repository governance configuration.

## Tools

- Build: `cargo` with the project-sanctioned Rust 1.89.0 toolchain where available.
- Test: Cargo package and focused integration tests.
- Lint: Rustfmt through Cargo.
- Type-check: Cargo compilation and test builds.
- Packaging: Repository packaging scripts under `digital-protocell/scripts`.
- Preferred navigation/indexing: Configured repository navigation tools when available, with targeted inspection as the fallback.

## Verified commands

- `python3 scripts/validate_governance.py --mode ADOPTED`
- `cargo test -p phase1-certifier --release --test metrics_semantics`
- `cargo test -p evolution-harness`
- `cargo fmt --all -- --check`
- Project-specific commands: `python3 -m unittest discover scripts`

## Constraints

- Platform/compatibility: Atlas is accessed through configured SSH or SSHFS; Linux remains the target platform.
- Security: Preserve credentials, private configuration, external Authority systems, and unrelated user work.
- Data handling: Preserve certified biology, append-only history, evidence, provenance, and generated-artifact boundaries.
- Deployment: No deployment or merge is authorized by DC-DEV-001A; architect review remains required.
- M2 boundary: The opt-in A-funded actuator and ENTRY-005 raw intrinsic motor coupling are qualified. ENTRY-007 may only replay the frozen DC-DEV-013 finite N/F ecology and perform assay-local observer reconstruction of unchanged DC-DEV-008 uptake; it may not add sensing, temporal memory, navigation, tuning, or a resource-dependent motor rule. Production remains V4/reserve OFF and PR #44 remains historical provenance.
