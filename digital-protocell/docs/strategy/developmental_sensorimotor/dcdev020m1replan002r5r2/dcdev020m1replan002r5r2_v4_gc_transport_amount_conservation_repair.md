# DC-DEV-020-M1-REPLAN-002-R5-R2

## V4/GC transport amount-conservation repair

This package implements only the Architect-authorized transport/material contract correction from R5-R1 authority `4b1a82877246c58ba21464963eb5bc4cb2a535cf`.

For `GeometryConservativeV3` and `MaturationCoupledV4`, transport requests are represented as signed absolute amounts and applied against the actual finite positive mesh area. Outbound transfer is capped by the available interior amount, concentrations are reconstructed from the post-transfer amount, and catalyst tracer/composition follows the actual catalyst amount removed. `HistoricalV1` and `ConservativeV2` retain their historical floor-based transport behavior.

The focused tests pass for positive sub-floor areas near `1e-7`, `1e-9`, and `1e-12`, and the above-floor parity test passes. The bounded integrated V4 starvation replay reports no transport closure residual above `4.0946412926956555e-14`; the old step-7684/8177 transport failure class is eliminated. The authoritative replay stops at the first `mechanics_step == false` at step `8566`, so no post-failure death evidence is produced.

This qualification does not requalify R4's 150k trajectory, rerun R5 refeeding, establish irreversible death, close M1, switch production, or authorize M2. Dense stage evidence is stored on Atlas at `/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/dcdev020m1replan002r5r2/`; compact JSON is kept under `experiments/generated/dcdev020m1replan002r5r2/`.

Classification remains pending exact-head Linux CI and Architect review.
