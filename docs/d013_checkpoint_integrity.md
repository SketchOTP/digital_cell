# D-013 Checkpoint Integrity

Atomic checkpoints are written when accepted-substep progress crosses:

```text
10,000  25,000  50,000  100,000  150,000  200,000
```

Crossing rule:

```text
previous_accepted < threshold AND current_accepted ≥ threshold
```

Write path:

```text
temporary file → fsync → atomic rename
```

Partial checkpoints (`clean_atomic_write = false`) are rejected on load.

Each checkpoint captures seven fields, next-buffer safe-reset rule, step counters, dt diagnostics, candidate/config/source/binary identity, field hashes, accepted ledgers, rolling-window state, convergence counter, material-equivalent ledger, and activation-potential ledger.

Deterministic continuation is validated at 25,000 → 50,000 against an uninterrupted 50,000 run (preflight exercises 10,000 → 25,000).
