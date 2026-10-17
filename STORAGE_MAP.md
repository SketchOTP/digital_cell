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
