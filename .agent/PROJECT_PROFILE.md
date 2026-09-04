# Project Profile

## Lifecycle

- Status: `ADOPTED`
- Last verified: `2026-09-01T07:01:00-04:00`

## Identity

- Project name or identifier: Digital Cell
- Purpose: A self-maintaining digital protocell developing toward a persistent embodied digital lifeform.
- Repository root: `/home/sketch/Projects/digital_cell-m2-entry001`
- Verified remote: `git@github.com:SketchOTP/digital_cell.git`
- Maturity or current phase: M1 closed/frozen; M2 locomotion, polarity, finite-resource, and lifecycle boundaries through CLOSURE-002 are accepted. CLOSURE-003 is the active isolated reproductive-resource-budget and heritable-phenotype audit. Autonomous resource-causal reproduction, mutable heritable phenotype, and evolution re-entry remain not established.

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
- M2 boundary: The opt-in A-funded actuator, ENTRY-005 raw intrinsic motor coupling, ENTRY-011 uptake/metabolism composition, ENTRY-014 isolated reference transfer, and ENTRY-015 assay-local `u/(u+v)` effector interface are qualified. ENTRY-016 established that the regular founder lacks a physical asymmetry seed. ENTRY-017 established lawful nonuniform 78/122-site daughters but left topology mapping unresolved. ENTRY-018 may only transfer the accepted continuous equations through a conservative normalized-arclength numerical operator and audit native stability/replay; it may not initialize polarity, couple behavior, modify fission/remesh, add randomness, interact with resources, tune parameters, or change production. Production remains V4/reserve OFF and PR #44 remains historical provenance.
