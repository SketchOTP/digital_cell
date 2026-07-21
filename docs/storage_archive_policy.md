# Storage archive policy (1TB)

## Why

The system root NVMe (`/`, ~233 G) fills quickly with cargo `target/`, code indexes, and directive experiment histories. The attached drive **`/mnt/storage1tb`** (~938 G, label `storage1tb`, also `~/storage1tb`) is the durable bulk store.

## digital_cell archive root

```text
/mnt/storage1tb/cache/project-artifacts/digital_cell/
  README.md
  ARCHIVE_MANIFEST.txt
  target/                 ← symlinked from digital-protocell/target
  cocoindex/              ← symlinked from .cocoindex_code
  experiments/generated/  ← per-directive dirs (symlink or full copy)
```

Pass performed **2026-07-21**: moved ~6 G regenerable bulk; repo working tree dropped from ~7.0 G → ~0.8 G. Original paths kept via symlinks where safe.

## Rules for agents

1. **Migrate archivable folders to the 1TB drive**; keep them reachable (symlink at the old path, or document the archive path).
2. Prefer new large outputs under `/mnt/storage1tb/cache/project-artifacts/<project>/` when a run will produce hundreds of MiB+.
3. **Safe to move:** untracked `experiments/generated/*`, `target/`, `.cocoindex_code/`, other rebuildable caches.
4. **Unsafe to move:** tracked git files, `.git`, source, `.agent` memory, secrets.
5. **Mixed dirs** (tracked + untracked): full copy to archive; delete only untracked locals; leave tracked files on NVMe.
6. Whole inactive *projects*: use `~/.local/bin/archive-stale-projects.sh` → `/mnt/storage1tb/archived-projects/` (not this per-artifact layout).

## Recovery

- Follow symlink: `readlink -f digital-protocell/target`
- Or open `/mnt/storage1tb/cache/project-artifacts/digital_cell/ARCHIVE_MANIFEST.txt`
- If symlinks break, remount `/mnt/storage1tb` first

## Related tooling

| Tool | Role |
|------|------|
| `~/.local/bin/archive-stale-projects.sh` | Move stale project directories |
| `~/.local/bin/fix-partial-archives.sh` | Repair partial whole-project archives |
| `.cursor/rules/06-storage-archive.mdc` | Always-on agent reminder |
