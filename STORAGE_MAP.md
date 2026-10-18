# Digital Cell Storage Map

This file is the canonical locator for project material intentionally stored
outside the primary repository filesystem.

## Primary repository

- Repository: `/home/sketch/Projects/digital_cell`
- Basis commit for the current storage migration: `456818fffbbbff2640984a8b81a41051bf03be75`
- Current branch at migration: `strategy/d096-repair-gain-scope-fix`

## Secondary project storage

- Mounted storage: `/mnt/storage1tb`
- Current repository archive root: `/mnt/storage1tb/project-archives/digital_cell/`
- Current migration archive: `/mnt/storage1tb/project-archives/digital_cell/2026-08-15/`
- Current archive manifest: `/mnt/storage1tb/project-archives/digital_cell/2026-08-15/ARCHIVE_MANIFEST.yaml`
- Current checksums: `/mnt/storage1tb/project-archives/digital_cell/2026-08-15/CHECKSUMS.sha256`
- Current restore instructions: `/mnt/storage1tb/project-archives/digital_cell/2026-08-15/RESTORE.md`

## Existing legacy archive

- Legacy archive root: `/mnt/storage1tb/cache/project-artifacts/digital_cell/`
- Status: pre-existing July 2026 archive; verified present and retained
- Use: existing experiment-runner defaults and historical evidence records
- Policy: do not relocate or rewrite this historical archive as part of the
  2026-08-15 build-cache migration; future changes must preserve its evidence
  and provenance chain.

## Relocated material in the current migration

The regenerated Rust build cache formerly at:

`/home/sketch/Projects/digital_cell/digital-protocell/target`

is stored under:

`/mnt/storage1tb/project-archives/digital_cell/2026-08-15/files/digital-protocell/target`

It was copied and SHA-256 verified before the local copy was removed. It is
safe to regenerate with the sanctioned Rust toolchain; restore only when an
exact prior build cache is needed.

## Material intentionally kept local

Active source, governance files, `.git`, workflows, documentation, and
scientific evidence remain in the primary repository. In particular,
`digital-protocell/experiments/generated/d094r` remains local because active
experiment runners, historical reconstruction, manifests, and provenance
documentation reference it directly. No evidence paths were relocated in the
current migration, so no scientific evidence references required rewriting.

Future archives should use a repository-isolated subdirectory beneath the
secondary storage root and must update this map and their archive manifest
after copy, checksum verification, and any reference changes.

## Reaction-feasibility local evidence custody — 2026-10-17

During final gate synchronization, the remote `/srv/ATLAS` and
`/mnt/storage1tb` data mounts became unavailable. Read-only mount inspection
found the underlying remote root filesystem at those paths. No mount,
unmount, disk repair, deletion or migration was attempted.

The verified reaction-feasibility artifacts and available cached R13 archives
have duplicate durable local custody at:

`/home/sketch/Projects/digital_cell-reaction-feasibility-evidence/`

`storage-custody.json` records the observation and
`FINAL_HANDOFF.md` identifies the accepted audit head and artifact receipts.
The original gate archive locator remains
`/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/reaction-architecture-feasibility/`;
its availability requires separate storage recovery. Existing historical
archive locators above are preserved, not relocated or silently recreated
on the unmounted root filesystem.

## Atlas evidence reconciliation verified — 2026-10-17

The owner reconnected and mounted UUID `840ca6b3-5d4f-41fc-9dac-21355e045a6d`
(`RPI5_2TB`, ext4) at `/mnt/atlas2tb`, with the configured `RPI5Pool`
bind at `/srv/ATLAS`. This supersedes the Atlas evidence-availability
limitation above; the outage observation remains historical evidence.

Digital Cell R12/R13 and reaction-feasibility accepted ZIPs, manifests and
available scientific binaries verify. All 83 pre-existing scoped files remain
hash-identical. The preserved local snapshot (77 files,
309554499 bytes) is verified at:

`/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/reaction-architecture-feasibility/preserved-local-20261017-174351/`

The accepted `final-08494f2` audit package is also present at the canonical
reaction-feasibility archive path with 29 manifest bindings and 16 source
bindings verified. Local originals remain. No existing archive file was
overwritten or deleted. Current reconciliation receipts are at:

`/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/reaction-architecture-feasibility/reconciliation-20261017-174351/`

and locally at `/home/sketch/Projects/digital_cell-reaction-feasibility-evidence/atlas-reconciliation-20261017-174351/`.
This verifies the scoped Atlas evidence archive, not whole-disk health or
the unrelated legacy `/mnt/storage1tb` archive. No disk repair or unrelated
mount action was performed by this reconciliation task.


## Conversion-work contract evidence — 2026-10-17

The separately authorized contract-only successor begins at b2de15dc0d7a7ee3f25b7993c558099d17a150a0.
Durable local evidence: `/home/sketch/Projects/digital_cell-conversion-work-contract-evidence/`.
Additive Atlas duplicate: `/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/conversion-work-contract/`.
Verify expected UUID840ca6b3-5d4f-41fc-9dac-21355e045a6d and RPI5Pool bind before writes.
Before/after scoped hashes preserve all 206 entry reaction-feasibility archive files and original source work;
new contract, exact-head artifact and Notion/PR readback receipts are copied without overwriting history.
The prior local/Atlas evidence remains retained. This qualifies custody, not full-disk health or runtime biology.


## Prospective work-port specification evidence — 2026-10-17

Entry a007e7e08acc768f08b83d96139a7cea25f0e04e; source checkout
`/home/sketch/Projects/digital_cell-work-port-feasibility-spec`.
Local evidence `/home/sketch/Projects/digital_cell-work-port-specification-evidence/`;
additive Atlas duplicate `/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/work-port-specification/`.
Verify expected UUID840ca6b3-5d4f-41fc-9dac-21355e045a6d and RPI5Pool bind before writes.
Preserve all 206 reaction-feasibility and 167 conversion-contract entry archive files,
existing checkouts, source work and PR44–47; retain local originals.
Reference mathematical fixtures are not new native or biological qualification evidence.


## Source/receiver architecture evidence — 2026-10-17

Entry101e635d6498b9cfe451b2d702d64a4440939963; isolated source
`/home/sketch/Projects/digital_cell-source-receiver-architecture`.
Local evidence `/home/sketch/Projects/digital_cell-source-receiver-evidence/`;
additive Atlas duplicate `/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/source-receiver-architecture/`.
Verify expected UUID840ca6b3-5d4f-41fc-9dac-21355e045a6d and RPI5Pool bind before writes.
Retain all entry archive bytes:206 reaction-feasibility,167 conversion-contract and151 work-port files.
Original local/Atlas source work and PR44–48 are preserved; local originals remain.
These mathematical architecture and anchor receipts do not qualify production implementation or biological efficacy.


## Bounded reference prototype and acquisition plan — 2026-10-17

- Checkout: `/home/sketch/Projects/digital_cell-bounded-reference-prototype`, isolated from preserved prior worktrees and original Atlas source.
- Local evidence: `/home/sketch/Projects/digital_cell-reference-prototype-evidence/` retains original snapshots, reference reports and final exact-head CI/artifact/Notion/preservation receipts.
- Atlas destination: `/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/reference-prototype/`; fresh publication/completion directories only, local originals retained.
- Required physical mount: UUID `840ca6b3-5d4f-41fc-9dac-21355e045a6d` and `/srv/ATLAS` FSROOT `/RPI5Pool`, independently checked via atlas-laptop before writes and native hash verification.
- Preserve all entry evidence: 206 reaction-feasibility +167 conversion-contract +151 work-port +172 source/receiver files =696 scoped files. No archive overwrite/delete, PR44–49 mutation or previous-checkout cleanup.
- Artifacts: standalone reference program/tests, empty acquisition templates and explicit missing-data plan; numerical reference constants and trajectories are not calibration observations. Final hosted hashes and custody counts are in the external exact-head receipt package.


## Independent calibration-source evidence — 2026-10-17

- Checkout: `/home/sketch/Projects/digital_cell-calibration-source-admission`; source scientific/prototype/plan history remains frozen.
- Local originals: `/home/sketch/Projects/digital_cell-calibration-source-evidence/`; acquired/ retains original primary HTML, extracted external source/readmes/tables and sparse range/selection/integrity evidence. External code is never executed.
- Git/hosted evidence: docs/dc-m4-calibration-source-evidence/ contains licensed original sparse archive bytes, metadata/receipts/table inventory and attribution, not unlicensed article text or a claimed complete archive. Forty-four processed rows are not target calibration data.
- Atlas: fresh publication/completion folders under `/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/calibration-source-admission/`; verify UUID840ca6b3-5d4f-41fc-9dac-21355e045a6d and FSROOT/RPI5Pool directly before writes. Retain local originals; no overwrite or cleanup.
- Prior archive preservation: 206 reaction-feasibility +167 conversion-contract +151 work-port +172 source/receiver +332 reference-prototype =1028 scoped files. Preserve ten previous checkouts, PR44–50 and original dirty Atlas source.
- Exact-head CI/artifact/member reconstruction, partial calibration outcome, independent review, final Notion and custody receipts live in the external package; do not confuse artifact checks with scientific admission.


## Calibration reference/mapping evidence — 2026-10-17

Local originals: `/home/sketch/Projects/digital_cell-reference-mapping-evidence/`. Additive Atlas group: `/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/calibration-reference-mapping/`, after expected UUID/FSROOT verification. Preserve all 1347 entry scoped archive files, prior original acquisitions and dirty original source. Exact-head hosted artifact, hashes, preservation and Notion readbacks are external receipts after commit creation; no archive success is inferred merely from this path declaration.


## Chemical/kinetic computational reference evidence — 2026-10-18

Local originals: `/home/sketch/Projects/digital_cell-chemical-kinetic-evidence/`. Additive Atlas group `/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/chemical-kinetic-reference/` requires expected UUID840ca6b3-5d4f-41fc-9dac-21355e045a6d and/RPI5Pool identity. Original primary full text retained externally without Git redistribution; own simulated trajectories, sealed protocol, exact-head hosted artifact and final Notion/preservation receipts accompany the reference. Preserve1733 entry scoped archive files, prior local originals and dirty original source. Final native write/readback receipts follow externally after commit creation.


## Calibration/native qualification evidence — 2026-10-18

Local originals: `/home/sketch/Projects/digital_cell-calibration-native-evidence/`. Additive Atlas group `/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/calibration-native-qualification/` requires direct expected UUID840ca6b3-5d4f-41fc-9dac-21355e045a6d and FSROOT/RPI5Pool verification before writes. Preserve the prior2208 scoped archive files,13 earlier checkouts,PR44–53 and original dirty source. Retain failed hosted attempt, compiled native executable/toolchain/build logs, all raw source/mesh trajectories and probe observations, exact-head bindings, independent reconstruction/review and final Notion/preservation/custody receipts. Final write/readback receipts follow externally after commit creation; no success is inferred from this path declaration.


## Empirical constituent/full-state body evidence — 2026-10-18

Local originals: `/home/sketch/Projects/digital_cell-empirical-organism-evidence/`. Additive Atlas group `/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/empirical-organism-integration/` requires direct UUID840ca6b3-5d4f-41fc-9dac-21355e045a6d and FSROOT/RPI5Pool verification before writes. Preserve2439priorarchivefiles,14priorlivecheckouts,PR44–54 and original dirty source. Preexisting absent worktree registrations are recorded without pruning. Retain source/checksum/selection seals, licensed lossless raw subset/annotations, quarantined unexecuted source, empirical analysis/corrections, native executable/raw trajectories, failed attempt, exact-head hosted source/observation/review bindings and final Notion/preservation/native custody receipts. Full336MB source ZIP/HDF5 were checksum-verified on hosted acquisition and remain at the publisher; only declared lossless selected observations are bundled. Publish compressed originals rather than recursively duplicating extracted raw campaigns. Final native receipts follow externally after commit; no success is inferred from this declaration.
