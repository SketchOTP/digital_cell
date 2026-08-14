# Paired analysis

The preregistered paired contrasts were:

- H: `D_H = T_rep*(processing-heavy) - T_rep*(repair-heavy)`, expected `< 0`;
- B: `D_B = T_rep*(repair-heavy) - T_rep*(processing-heavy)`, expected `< 0`;
- Neutral: processing-heavy versus repair-heavy separation expected to be
  smaller than both selected-environment effects.

Each contrast used the same seed in both cells and a deterministic paired
bootstrap with 10,000 resamples. Because every replicate stopped at the
horizon, all paired differences were `0.0`; all bootstrap intervals were
`[0.0, 0.0]`, and neither selected-environment direction met the preregistered
direction or confidence requirements.

This is not converted into a pooled score or a selection result. Full values
and interval metadata are in `paired_analysis.json`.

