# D-096 Gate 5 replay

The replay used the original contract, not SR-004 D-094 ecology:

- paired seeds `1..=8`;
- processing-heavy `[0.55, 0.25, 0.05, 0.15]` and repair-heavy
  `[0.10, 0.20, 0.55, 0.15]`;
- H pulse, B damage, and neutral environment from the existing D-096 code;
- 1,000 accepted steps at `dt=0.02`;
- mutation off and fission off;
- original continuous reciprocal effect definitions and thresholds.

Observed effects:

- H processing-heavy minus repair-heavy reserve change: mean
  `0.5988859008884848`; neutral mean `0.5600240590850483`;
- B repair-heavy minus processing-heavy final material: mean
  `3.811469763347633`; neutral mean `3.58612402240918`;
- all eight H, B, and neutral paired runs passed the original positive and
  treatment-amplified criteria.

Verdict: `D096_GATE5_PASS_REPAIRED`; no Gate 6 execution occurred.
