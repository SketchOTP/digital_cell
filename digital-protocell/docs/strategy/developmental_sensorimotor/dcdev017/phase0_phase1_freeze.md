# DC-DEV-017 Phase 0 and Phase 1 freeze

Entry authority: `1e242f28152797b512e25cd56c7b718e45d6ca97`.

This bounded slice records the live metabolic control surface and executes
the existing metabolism over its accepted reserve-storage horizon. It does
not modify chemistry-core production behavior. The assay-only clamp uses
the lower matched N/F concentration reached at the formal DC-DEV-016
challenge endpoint, `0.1476710565778127` each, with top-up before each
reaction step.

The live accepted reserve configuration derives a 40.0 simulation-time-unit
maintenance horizon from `1 / k_release`, and an 80.0-unit storage horizon
from `store_horizon_mult = 2`. With `dt = 0.02`, Phase 1 is exactly 4,000
accepted reaction steps, divided into preregistered quarters Q1 through Q4.

Phase 1 has two matched arms: P1-A uses the deprived body with no precursor,
and P1-B uses the same body with only N/F top-up. No A, R, catalyst, reserve,
or other field is injected or clamped. The formal finding is emitted by the
assay and determines whether the single conditional Phase 2 repair may begin.
