# D-075 Cellwise Exposure-Gated Membrane Requalification

## Mission

Replace invalid mean-timescale and accepted-step membrane horizons with
**cellwise, capacity-weighted exact effective exposure** \(E_i\), then requalify
mature-membrane maintenance, precursor regulation, damage repair, radius
portability, and the bounded Stage E membrane screen.

Frozen contracts (unchanged): \(K_{eq}\), \(k_{exchange}\), \(\Gamma_{max}\),
`SEED_CAPACITY_CONTRACT_V1`, Seed B / Policy D, precursor-production equations,
D-071 candidate parameters, activation / catalyst chemistry, structural
chemistry, permeability / surface transport, damage extent, repair threshold,
numerical integration rules.

## Canonical exposure metric

For each lawful interface cell and accepted step:

\[
\lambda_{i,n}=k_{exchange}\,q(C_{i,n})\,(K_{eq}p_{i,n}+1)
\]

Diagnostic continuous exposure:

\[
\Lambda_i=\sum_n\lambda_{i,n}\Delta t_n
\]

Exact discrete contraction from the production mild-FE / BE dispatch:

- explicit: \(c_{i,n}=|1-\lambda_{i,n}\Delta t_n|\)
- backward-Euler: \(c_{i,n}=1/(1+\lambda_{i,n}\Delta t_n)\)

Authoritative effective exposure:

\[
E_i=-\sum_n\ln(c_{i,n})
\]

Rejected attempts contribute no exposure, no simulated time, and no exchange
extent.

## Qualification

A membrane horizon qualifies only when:

- ≥95% of relevant lawful capacity has \(E_i\ge 5\)
- zero-exposure lawful capacity <1%
- unsupported capacity reported separately
- accepted accounting closes
- run has not terminated through numerical or biological invalidity

Damage experiments qualify the damaged-region capacity. Undamaged maintenance
qualifies the full mature-membrane support. Raw accepted-step counts and
mean-\(\tau\) estimates cannot qualify a result.

## Entry state

- Branch: `d008-membrane-metabolic-closure`
- Starting commit / tag: `b06254b` / `D-074-cellwise-exchange-parity-audit`
- D-074 conclusion: `D074_EXCHANGE_TIMESCALE_CLASSIFICATION_DEFECT`
- Stage E: `BLOCKED_NOT_RECOVERED`

## Gates

| Gate | Content |
|------|---------|
| 0 | D-074 preservation + reproduction (`fraction_E_ge5=0` under prior mean-τ) |
| 1 | Shared exposure observer (production FE/BE dispatch, capacity weighting) |
| 2 | Synthetic contraction calibration at \(E\in\{1,3,5\}\) |
| 3 | Fixed-P controls at \(p\in\{0.38,0.418,2.48\}\) until damaged exposure qualifies |
| 4 | Exposure-qualified undamaged maintenance (constitutive / D-071 / \(k_P=0\)) |
| 5 | Exposure-qualified damage repair from Gate-4-qualified undamaged state |
| 6 | Precursor regulation decision |
| 7 | Radius portability R16/R22/R32 |
| 8 | Bounded Stage E re-entry (only if Gates 0–7 pass) |

## Routes

Exactly one primary: Q / R / T / M / C / H / F (plus Stage E recovery when Q and
Stage E both pass).

## Artifacts

`digital-protocell/experiments/generated/d075/` →
`/mnt/storage1tb/cache/project-artifacts/digital_cell/experiments/generated/d075/`.

## Status

**Primary conclusion:** `D075_FROZEN_EXCHANGE_METABOLICALLY_UNREACHABLE` (Route M).

### Key evidence

- Gate 0: D-074 Route T preserved; prior mean-τ horizon defect confirmed (`fraction_E_ge5=0`).
- Gate 2: synthetic FE/BE contraction parity at \(E\in\{1,3,5\}\) PASS.
- Gate 3: exchange-isolated fixed-\(p\) controls reach damaged-region exposure
  qualification; \(p=0.418\) recovery \(\approx 0.957\), \(p=2.48\) recovery
  \(\approx 0.995\) (non-promotable diagnostics).
- Gate 4: constitutive / D-071 reduced / \(k_P=0\) all reach full-membrane
  \(E\)-qualification, but classify `EQUILIBRIUM_BELOW_CONTRACT`
  (constitutive interface \(p\approx 0.19\Rightarrow\theta_{eq}\approx 0.90\);
  occupancy falls to \(\sim 0.67\); A retention \(\sim 0.06\)).
- Gate 5: constitutive/regulated repair skipped — no Gate-4 maintenance
  baseline qualifies; no-precursor control fails as required.
- Gates 7–8: radius/Stage E not promoted (Gates 0–7 incomplete for biology).

### Scientific reading

Frozen exchange kinetics are locally correct and can refill damage under held
interface precursor. Endogenous chemistry cannot sustain the local precursor
activity required for contract-level mature occupancy without A collapse.
This authorizes a subsequent **bounded exchange-architecture review**; do not
alter exchange inside D-075.

### Next

Bounded exchange-architecture review directive. Do not promote D-071 regulation.
Stage E remains `BLOCKED_NOT_RECOVERED`. `next_execution_started=false`.
