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

## Current authoritative evidence storage

- Shared-drive root: `\\RPI5\\RPI5SharedDrive\\100_ACTIVE\\Projects\\DIGITAL_CELL`
- Evidence root: `\\RPI5\\RPI5SharedDrive\\100_ACTIVE\\Projects\\DIGITAL_CELL\\evidence\\`
- R5 evidence root: `\\RPI5\\RPI5SharedDrive\\100_ACTIVE\\Projects\\DIGITAL_CELL\\evidence\\dcdev020m1r5\\`
- R6 evidence root: `\\RPI5\\RPI5SharedDrive\\100_ACTIVE\\Projects\\DIGITAL_CELL\\evidence\\dcdev020m1r6\\`
- R6 dense runtime root: `\\RPI5\\RPI5SharedDrive\\100_ACTIVE\\Projects\\DIGITAL_CELL\\evidence\\dcdev020m1r6\\dense\\`
- R6 evidence manifest: `\\RPI5\\RPI5SharedDrive\\100_ACTIVE\\Projects\\DIGITAL_CELL\\evidence\\dcdev020m1r6\\R6_EVIDENCE_MANIFEST.json`
- Policy: dense experiment evidence, runtime ledgers, and archived prior evidence are copied to the shared-drive root, checksum-verified, and referenced by a manifest before local cleanup. Compact protocol/results/qualification/manifests required for GitHub CI remain in Git and are duplicated on the shared drive.
- Restore: copy the manifest-listed files back to the repository-relative path, verify SHA-256, then rerun the scoped verifier. Do not restore build caches as scientific evidence.

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
after copy, checksum verification, and any reference changes. Evidence is now
authoritatively stored on the RPI5 shared drive above; local evidence copies
must not be deleted until the shared-drive manifest and checksums verify.
