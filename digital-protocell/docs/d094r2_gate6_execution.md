# D-094R2 Gate 6 execution

## Sealed attempt

- Infrastructure source commit: `bf58edddef40753107ba18854eb85cc41ec78859`
- Source tree: `d9dc1f0e9543be22019044d6033062d7502f22d3`
- Campaign binary SHA-256: `6c49dd04411cce128ddcb9008d5ecbd4b77afe5da2a7a0cdc3a88f8a4c25f8aa`
- Attempt: `experiments/generated/d094r/gate6/attempt_001/`
- Job: `job-ms2eyr1f-2fdd31b8`, exit `0` at `2026-07-27T02:51:34Z`

The run used eight paired, mutation-off replicates for each frozen H, B, and
neutral regime. Gates 7 and 8 were hard-blocked; no adaptation, reversal, or
Phase 3 work ran.

## Completion and provenance

All 24 replicates reached eight completed generations. The attempt contains 192
atomic generation checkpoints (8 replicates × 8 generations × 3 regimes), each
with source, binary/configuration, founder/treatment/seed, population-state, and
lineage-ledger identity. All campaign records declare atomic checkpoints and
lineage ledgers complete.

| regime | config SHA-256 | completed replicates | generation range |
| --- | --- | ---: | --- |
| H | `8cd865e452f3f59239e21f80006b7b3e54fa8239175ca90712df58b1c1fd6694` | 8/8 | 8–8 |
| B | `7c60fe55d4149de1628878f82f8b6290e67183866602f89092d128ba98332412` | 8/8 | 8–8 |
| neutral | `56c16311a06336f003fe37466c13895d2f0779d1d85769d98aa780996ec80658` | 8/8 | 8–8 |

The executed binary hash matches campaign provenance. The attempt manifest
SHA-256 is `e9b03ff268da69b91a8ced053b311cc7e1e0c439c75503dde6e7002207d2e01e`.

## Validation and environment

- Rust `1.95.0 (59807616e 2026-04-14)`; Cargo `1.95.0 (f2d3ce0bd 2026-03-21)`; host `x86_64-unknown-linux-gnu`.
- Recovered the existing `/home/sketch/.cargo/bin` toolchain; no dependency or lockfile change.
- `Cargo.lock` SHA-256: `5fb683c2eb6bea4d7ee2b91e721a6e23ce215d1d98bbcacfd38c15f56a7b3058`.
- Focused tests passed: D-094 selection 8/8; D-094R provenance/effects 3/3; runner lock 3/3.
- Affected regressions passed: D-087 certification 4/4; D-088 4/4; D-091 7/7; D-092 7/7; D-093 5/5.
- `cargo check --workspace --all-targets --locked` fails only in `chemistry-core` test target `d008_tests`: lines 234 and 255 have non-exhaustive `SnapshotFields` matches missing `NineFieldSurfaceDensity` and `NineFieldSurfaceMaturation`.

The D-008 failure was not linked into the executed release binary: the campaign
successfully compiled and linked `experiment-runner`, then executed that binary
from sealed source. `d008_tests` is a separate test target. This prevents a
workspace-wide green claim but does not invalidate the Gate 6 binary or provenance.

## Implementation identities

- Lock implementation SHA-256: `4b898d18673aa7eedfebf4c0eca33afa32ae7924f0757eeeec57442fdc15ac3e`
- Selection/analysis implementation SHA-256: `0cde217989878a09e89344eeddb343f92a8931b32dc3f8c0a1585ba586c3dc54`

The archive mount rejected writes with an I/O error, so the immutable attempt
was retained at the required canonical repository path on the primary volume.
