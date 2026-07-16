# D-015 transport controls

## Control A — External waste pulse

Frozen R22 geometry; reactions off; W initialized in bulk exterior; clearance + transport on.

Repaired-env result (`controls/result.json`):

- `cleared: true` (mass declined 21516 → 11588 in short diagnostic window)
- Reservoir/ledger clearance active

## Control B — Internal waste pulse

W inside prescribed cell; baseline `β_W`; diffusion + clearance on.

- `center_before: 5` → `center_after ≈ 2.55`
- `exported: true` → **not** `WASTE_EXPORT_FAILURE`

## Control C — Membrane bypass (`β_W = 0`)

Diagnostic only. Recorded baseline vs diagnostic betas; must not become candidate.

## Control E — No-clearance

Measured/injected W with clearance disabled → `accumulated: true` (clearance, not numerical loss, removes W).

## Control D — Measured-source injection

Profile injection multipliers 0.5×/1.0×/1.5×: exercised via unit tests
(`test_measured_source_injection_matches_budget`, `test_sink_capacity_prediction_matches_control`)
and analytical capacity; full long-horizon profile replay is diagnostic-only relative to Stage E.
