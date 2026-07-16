# D-015 waste accumulation postmortem

## Entry failure (D-014 fresh R22)

| Field | Value |
| --- | --- |
| Candidate hash | `9a452d3470be34ccf3bdd7d1397341b64617834e77131cf2899efb327728d626` |
| Configuration hash | `87ff7e6e4bd479972c3a02b0de4e6bc94a949041860b32b230e5b28863bb5ad6` |
| Accepted substeps | 161,157 |
| Simulated time | ≈402.89 |
| Terminal class | `UNBOUNDED_ACCUMULATION` |
| Cause | W reached `CONC_SAFETY_LIMIT` at grid index **18335** (cell center) |

## Causal chain

```text
W produced inside cell (metabolism + turnover + detachment)
        ↓
W exits membrane (near-exterior W > 0 at 150k)
        ↓
Bulk exterior accumulates weakly (mean ≪ interior)
        ↓
Peripheral reservoir (r ≳ 83) remains empty (mean ≈ 0)
        ↓
Clearance idle (cumulative removed ≈ 0.008 vs biological ΔW ≈ 1.6e4)
        ↓
Interior fills to ceiling at center
```

## Ruling out alternatives

| Hypothesis | Verdict |
| --- | --- |
| Clearance implementation defect | **No** — law matches config; classified `CORRECT` |
| Membrane export failure | **No** — Control B exports; near-exterior W present |
| Environmental clearance capacity below production | **No if delivered** — predicted eq W ≈ 0.032; margin at ceiling ≈ 312× |
| Finite-domain whole-dish saturation | **Secondary** — interior local fill timescale matches failure better |
| Safety-ceiling scale defect | **No** — no finite open eq above ceiling; raising ceiling only delays |
| Incorrect waste accounting | **Subsidiary** — legacy `waste_from_*` undercounts v2 yields; metabolism ledger authoritative; D-015 budget closes |

## Primary diagnosis

`D015_BULK_DIFFUSION_BOTTLENECK` (delivery to peripheral sink; `TRANSPORT_TO_SINK_LIMITED`)

τ_diff (cell → reservoir) ≈ L²/D_W ≈ 61²/0.25 ≈ **16129** ≫ t_fail ≈ **403**.

## Authorized repair

Branch B item 1, **W-only** sink expansion: `waste_sink_inner_radius = 30` for R=22.
N/F continue to use the original `reservoir_mask` annulus (supply unchanged).
