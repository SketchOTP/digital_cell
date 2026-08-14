# DC-SR-004C-R1 parity and horizon audit

This directory records the shadow-only audit requested after the architect
withheld scientific acceptance of Gate 7. The audit reconstructs the sealed
Gate 5 assay, the existing Gate 7 adapter execution configuration, and the
D-088 4,000-step horizon authority.

The audit does not rerun the 144-cell Gate 7 campaign, modify the adapter,
extend the horizon, tune biology, or begin Gate 8. The original
`experiments/generated/sr004c/` evidence is preserved unchanged.

The audit runner is
`crates/evolution-harness/examples/d096_gate7_parity_audit.rs`. It emits the
JSON evidence under `experiments/generated/sr004cr1/`.

Current bounded conclusion: `SR004CR1_GATE5_TO_GATE7_PHYSIOLOGY_PARITY_FAILED`.
The D-088 horizon transfer shadow is valid at the unchanged horizon: both the
documented perturbed and the unperturbed 10-seed preparations fissioned within
4,000 steps. Gate 7 remains architect-blocked because its execution path does
not preserve Gate 5 physiology.
