# D-008 Stage B — Self-Produced Membrane Localization

## Conclusion

`D008_STAGE_B_LOCALIZATION_PASS`

The approved membrane reaction produced bounded membrane localized to the fixed
structure interface while membrane diffusion, decay, and off-interface
detachment remained active. Structure, catalyst, and activated resource were
fixed; complete metabolism and moving structure remained disabled.

## Provenance

- Source commit: `31fd993123e16ca64474f3d1176f3a8d74933eb2`
- Experiment-runner SHA-256: `0b5248da9b70266e4b36e753f9e552167ec4a429ea4b26174eabe96419391639`
- Equation version: `membrane_metabolism_v1`
- Snapshot schema: `2`
- Field schema: `seven_field_v1`
- Selected candidate: `cand-82667856c123-kphi1-ks0.030000-kr0.012000`
- Candidate hash: `82667856c1230a1a3ace6c11cfb23c8ded1a8ad65101d72cdc6f5346a021cce2`
- Configuration hash: `4df48980ff97173e9c0d2068677ffd56a630782c5c5f26e371e5cf26b0ee7d1a`
- Selected `k_membrane`: `0.19748231883326484`

## Localization and turnover result

The prescribed balance estimate was `k_required = 0.19748231883326484`.
Only the required factors `0.75`, `1.00`, and `1.25` were evaluated. The
`1.00×` candidate was selected.

- Minimum localization after the 15,000-step transient: `0.9030577224564282`
- Final localization fraction: `0.9040090424894835`
- Robustness initial level `0.25`, minimum localization: `0.9003522074202676`
- Robustness initial level `0.75`, minimum localization: `0.9044660274294114`
- Accepted substeps: `16,000` per run; `80,000` aggregate across five runs
- Clean terminations: `5/5`
- Cumulative synthesis: `96.66570647213598`
- Cumulative decay: `24.203849046649736`
- Cumulative detachment: `72.32643092972452`
- Cumulative accounting residual: `-2.3305801732931286e-12`
- Clamp correction: `0`

All membrane values remained finite and within `[0, M_max]`. Fixed-field hashes
for structure, catalyst, nutrient, fuel, waste, and activated resource were
unchanged. The `1.25×` candidate failed the post-transient minimum-localization
gate and was not selected.

## Validation

- `cargo test -p chemistry-core --release --test d008_tests` — 36 passed.
- `cargo test -p experiment-runner --release d008::tests` — 5 passed.
- Governed Stage B run — `D008_STAGE_B_LOCALIZATION_PASS`.

Existing compiler warnings remain; no unrelated warning cleanup was performed.

## Artifact

- Runtime result: `digital-protocell/experiments/generated/d008/stage_b_localization/attempt_001/result.json`
- Result SHA-256: `d9dc7df27f2eb77c25a987266861cfb245998690b8dd80fd74d05157ddae55cb`
- Manifest: `digital-protocell/experiments/generated/d008/manifest.json`
- Manifest SHA-256: `3ec24bc002e96b37843c4006c7f44f017f8a45bf73305ae9013442c945c22eda`

Stage C may proceed. Stages D–G remain unstarted.
