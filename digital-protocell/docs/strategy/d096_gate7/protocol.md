# Frozen protocol

Candidates use the accepted D-096 `AllocationParams` and retain candidate
identity based on those parameters:

- processing-heavy: `[0.55, 0.25, 0.05, 0.15]`;
- repair-heavy: `[0.10, 0.20, 0.55, 0.15]`;
- neutral: `[0.25, 0.25, 0.25, 0.25]`.

The environments use the existing D-096 forcing without redesign:

- H: fuel `1.0`; nutrient `2.75` when `step % 400 < 100`, otherwise `0.264`;
- B: nutrient `1.98`, fuel `1.0`, structural damage `0.08` and membrane damage
  `0.048` every 350 steps;
- Neutral: nutrient `1.54`, fuel `1.0`, no damage.

All cells use `mutation_none`, one gen0 founder, one of the same 16 seeds, and
the real Digital Cell mesh transport/reaction/growth/mechanics/fission path.

Exact serialized protocol and environment hashes are in `protocol.json`.

