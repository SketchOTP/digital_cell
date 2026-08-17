# DC-DEV-014 qualification results

## Authority and boundary

- Entry commit: 5a4e0a2d7314af411ec2283b0ffcf4950eb217db
- Branch: strategy/dc-dev-014-homeostatic-exploration
- Source/base: strategy/dc-dev-013-resource-contact-feeding
- Scientific scope: one existing material variable, one direction-neutral local nucleation process, fixed A-E assay
- Settlement: 5,000 accepted legacy mechanics steps
- Assay: 480 accepted steps, three 160-step analysis windows
- DC-DEV-012 imported: no
- DC-DEV-015 started: no

## Selected material signal

The selected signal is existing activated material A in MaterialMesh.interior.a. The accepted replete reference is the existing seed value A=0.5; no target value or new hunger state was introduced.

The accepted no-resource maintenance audit decreased A from 0.5 to 0.303630027599798 in 480 steps. Finite N/F uptake followed by existing reactions_step restored A relative to the same-geometry no-delivery arm: C ended at 0.2502233661813926, while D ended at 0.20689179981214934. This closes the required material depletion/restoration audit, but it does not establish late homeostatic relief.

## Exploration process

regulatory-core/src/homeostatic_exploration.rs emits rare local stimulus pulses. It reads only normalized A need, local topology size, and its own deterministic SplitMix-style state/provenance. At maximum need, total nucleation rate is the existing regulator k_decay=0.5, equivalent to one event per decay timescale 1/k_decay=2.0. Patch selection is uniform. The module has no body geometry, resource, contact, coordinate, target, reward, fitness, or motor input and cannot write coordinates or force.

The existing regulator spreads and decays the pulse. Existing funded contractility and DC-DEV-011 stick-slip remain the only active body path.

## Arm outcome

- A deprived/no resource: 7 exploration events, reserve spent 0.09751472397436492, final A 0.20689179981214934.
- B replete/no resource: 3 exploration events, reserve spent 0.04394020132900351, final A 0.3036475417211845.
- C deprived/finite N/F: 7 exploration events, N delivered 2.0556764371637604, F delivered 2.0556764371637604, final A 0.2502233661813926.
- D same contact geometry/no delivery: 7 exploration events, contact-positive for 480 steps, zero N/F delivered, final A 0.20689179981214934.
- E zero reserve: zero funded tension and zero reserve spend; passive residual displacement is recorded but not claimed as funded locomotion.

Gates 1-5 and 7-11 passed. Gate 6 failed: C late mean need/activity did not decrease below the preregistered deprived early comparison. This is a valid bounded negative result for the homeostatic exploration/feeding-state switch.

## Preservation

No chemistry-core source, mesh mechanics, substrate law, reserve law, DC-DEV-013 sensor, Phase 1, D-088, evolution-harness, or governance contract was changed. The production addition is confined to regulatory-core homeostatic exploration; the assay composes existing physiology and motor paths.

## Conclusion

DCDEV014_HOMEOSTATIC_EXPLORATION_NOT_ESTABLISHED

This result does not authorize parameter tuning, a longer horizon, a new hunger variable, navigation, resource seeking, or DC-DEV-015.
