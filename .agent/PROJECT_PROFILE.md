# Project Profile

## Lifecycle

- Status: `ADOPTED`
- Last verified: `2026-08-31T22:21:36-04:00`

## Identity

- Project name or identifier: Digital Cell
- Purpose: A self-maintaining digital protocell developing toward a persistent embodied digital lifeform.
- Repository root: `/home/sketch/Projects/digital_cell-m2-entry001`
- Verified remote: `git@github.com:SketchOTP/digital_cell.git`
- Maturity or current phase: M1 closed/frozen; M2 ENTRY-001 actuator, ENTRY-005 target-free intrinsic exploration, ENTRY-011 metabolically live in-contact composition, ENTRY-014 reference transfer, and ENTRY-015 polarity-to-actuator interface are Architect accepted. ENTRY-016 is the active isolated autonomous-polarity-initiation substrate audit. Autonomous polarity initiation and resource acquisition remain not established.

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
- M2 boundary: The opt-in A-funded actuator, ENTRY-005 raw intrinsic motor coupling, ENTRY-011 uptake/metabolism composition, ENTRY-014 isolated reference transfer, and ENTRY-015 assay-local `u/(u+v)` effector interface are qualified. ENTRY-016 may only audit homogeneous stability and existing settled-body asymmetry; it may not add production polarity, coupling, noise, resource interaction, sensing, memory, navigation, tuning, or a new effector. Production remains V4/reserve OFF and PR #44 remains historical provenance.
