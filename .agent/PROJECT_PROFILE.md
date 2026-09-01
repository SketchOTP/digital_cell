# Project Profile

## Lifecycle

- Status: `ADOPTED`
- Last verified: `2026-08-31T22:03:33-04:00`

## Identity

- Project name or identifier: Digital Cell
- Purpose: A self-maintaining digital protocell developing toward a persistent embodied digital lifeform.
- Repository root: `/home/sketch/Projects/digital_cell-m2-entry001`
- Verified remote: `git@github.com:SketchOTP/digital_cell.git`
- Maturity or current phase: M1 closed/frozen; M2 ENTRY-001 actuator, ENTRY-005 target-free intrinsic exploration, ENTRY-011 metabolically live in-contact composition, and accepted ENTRY-006 through ENTRY-014 boundaries are established. ENTRY-015 is the active isolated polarity-to-actuator interface audit. Autonomous resource acquisition remains not established.

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
- M2 boundary: The opt-in A-funded actuator, ENTRY-005 raw intrinsic motor coupling, ENTRY-011 uptake/metabolism composition, and ENTRY-014 isolated reference transfer are qualified. ENTRY-015 may only run an isolated resource-free assay-local `u/(u+v)` interface and matched controls; it may not add production polarity, resource coupling, sensing, memory, navigation, tuning, or a new effector. Production remains V4/reserve OFF and PR #44 remains historical provenance.
