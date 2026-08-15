# Storage archive policy

The system root NVMe fills quickly with Rust build output, code indexes, and
directive experiment histories. The writable secondary storage device is
`/mnt/storage1tb`.

## Canonical project locator

Read [`STORAGE_MAP.md`](../STORAGE_MAP.md) first. It is the canonical map for
Digital Cell material stored outside the primary repository.

The current repository archive root is:

`/mnt/storage1tb/project-archives/digital_cell/`

The 2026-08-15 migration archive is:

`/mnt/storage1tb/project-archives/digital_cell/2026-08-15/`

Its `ARCHIVE_MANIFEST.yaml`, `CHECKSUMS.sha256`, and `RESTORE.md` record the
relocated material, integrity verification, and recovery procedure.

The pre-existing July 2026 archive remains at
`/mnt/storage1tb/cache/project-artifacts/digital_cell/`. Existing
experiment-runner defaults and historical records may refer to that root; it
is retained and is not changed by the current build-cache migration.

## Current migration

The regenerated Rust build cache formerly at
`digital-protocell/target` was copied and SHA-256 verified before its local
copy was removed. It is retained under the archive's
`files/digital-protocell/target` path and may be regenerated with the sanctioned
Rust toolchain. No symlink is required or created.

Actively referenced source, governance files, `.git`, workflows,
documentation, and scientific evidence remain local unless a future migration
updates all references and the canonical map.

## Rules for agents

1. Classify candidates before moving them; use `STORAGE_MAP.md` as the project
   locator.
2. Copy first, compare SHA-256 checksums, verify readability, update references
   and manifests, then remove only the verified local copy.
3. Never move tracked source, `.git`, `.agent` memory, secrets, or actively
   required runtime material without an explicit supported storage design.
4. Keep evidence discoverable and preserve its provenance and checksums.
5. Do not create undocumented or fragile symlink dependencies.

## Recovery

Follow the `RESTORE.md` instructions in the dated archive, or regenerate the
build cache when an exact previous cache is not needed.

## Related tooling

`.cursor/rules/06-storage-archive.mdc` contains the always-on agent reminder;
it must defer to `STORAGE_MAP.md` for current paths.
