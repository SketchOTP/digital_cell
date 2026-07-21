# CURRENT.md

## Active directive
- ID: D-20260721-1049-archive-bulk-to-storage1tb
- Project directive: (ops — disk archive)
- Goal: Archive regenerable bulk to 1TB; document for future agents
- Status: done
- Acceptance: met — ~6G freed; symlinks intact; policy rule + docs written
- Touched files: target/.cocoindex_code/generated symlinks; .cursor/rules/06-storage-archive.mdc; docs/storage_archive_policy.md; AGENTS.md; .gitignore; .agent/*
- Next action: optional archive other Projects' cargo targets/worktrees; next_execution_started=false

## Repo facts needed now
- Archive: /mnt/storage1tb/cache/project-artifacts/digital_cell/
- Repo size now ~799M (was ~7G)
- Root disk ~75% (was 78%); +~6G free

## Last validation
- Command: readlink + test -f d058/manifest.json and target/CACHEDIR.TAG; df -h
- Result: PASS; ~6G NVMe freed

## Open blockers
- Broader /home/sketch/Projects (~77G) still on NVMe — stale worktrees are the next win
