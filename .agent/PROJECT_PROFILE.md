# Project Profile

## Lifecycle

- Status: `ADOPTED`
- Last verified: `2026-08-29T12:00:00-04:00`

## Identity

- Project name or identifier: Digital Cell
- Purpose: A self-maintaining digital protocell developing toward a persistent embodied digital lifeform.
- Repository root: `/home/sketch/Projects/digital_cell-m1-baseline`
- Verified remote: `git@github.com:SketchOTP/digital_cell.git`
- Maturity or current phase: M1 closed/frozen; M2 ENTRY-001 actuator qualified, ENTRY-002 found no existing exploration substrate, ENTRY-003 is Architect-accepted as mechanically insufficient, and ENTRY-004 is diagnosing intrinsic-to-traction transfer with observer-only free-step reconstruction. Autonomous resource acquisition is not established.

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
- M2 boundary: The opt-in A-funded actuator is qualified. ENTRY-003's separate opt-in intrinsic local state did not establish retained exploratory motion. ENTRY-004 may diagnose its frozen traction transfer only; no tuning or extension is authorized. Production remains V4/reserve OFF and PR #44 remains historical provenance.
