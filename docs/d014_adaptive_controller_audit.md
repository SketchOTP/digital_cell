# D-014 Adaptive Controller Audit

## Required properties

| Property | Status |
| --- | --- |
| Rejection reduces dt | yes (halve on reject) |
| Accepted steps permit bounded recovery | yes (`×1.25`, capped by `dt_cap`) |
| Recovery uses latest accepted state | yes (`self.dt = accepted attempt`) |
| dt never exceeds current stability / cap | yes (`min(dt_cap)`) |
| No permanent historical min latch | yes (recovery from current accepted dt) |
| Floor only when required dt below floor | yes (`D014_DT_FLOOR`) |

## D-013 floor event

Reproduction showed the floor was **not** caused by one-way ratcheting of a healthy limit.
The dominant limiter was `FIELD_BOUND_VALIDATION` on `waste_next` at the safety ceiling.
Controller recovery was already present during reproduction and did not prevent the floor;
hard concentration rejects now abort without cascading to the floor.

## Tests

`test_accepted_steps_can_recover_dt`, `test_rejected_dt_is_not_latched`,
`test_dt_uses_latest_accepted_state`, `test_safety_factor_applied_once`,
`test_dt_floor_uses_current_limit`, `test_controller_reports_dominant_limiter`.
