# Governance carryforward

Gate 0 classified prior repository material before architecture work began.

| Classification | Carried material | Treatment |
|---|---|---|
| `GOVERNANCE_ONLY` | `AGENTS.md`, commandments, adapters, `.agents/`, `.cursor/`, validators, project goal, historical ledgers | carried and reconciled to current governance schema |
| `OPERATIONS_ONLY` | `STORAGE_MAP.md`, `docs/storage_archive_policy.md` | carried as operations documentation; no scientific meaning |
| `SCIENTIFIC` | later R4/D-096 source and evidence | not carried; clean base is authoritative |
| `MIXED` | mutable `.agent/CURRENT.md`, profile, map, active ledgers | reconstructed for DC-DEV-001A; old state preserved in the dated legacy snapshot |
| `DO_NOT_CARRY` | later crates, generated artifacts, and workflows from the dirty R4 checkout | excluded from this branch |

Append-only historical records are preserved in the legacy snapshot. Mutable current state now describes only clean-base establishment and architecture selection. This separation prevents old completion claims from becoming active authority merely because they remain auditable.

