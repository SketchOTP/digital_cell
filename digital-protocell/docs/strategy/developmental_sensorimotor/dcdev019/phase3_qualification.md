# DC-DEV-019 Phase 3 — metabolic homeostasis qualification

## Frozen homeostat

The implementation is `digital_cell_metabolic_acquisition_homeostat_v1`.
It derives `e0=(E_target-E_deprived)/E_target`, uses `tau=80.0`,
`k_h=2/(e0*tau)`, and interpolates the existing N/F→A activation extent to
`G_source_max=6.97512279078733`. `G_transport_max=1.0`, so no transport
actuator was added. The feature is disabled by default and emits neutral gains
when disabled.

The M3 sustained assay uses the authorized R1-style clamp: before every
accepted reaction step, interior N and F are set to the frozen
`0.1476710565778127` clamp. This is an explicit assay clamp, not a production
world source or an implicit material claim.

## Results

- M0 feature-off exact parity: passed.
- M1 starvation: homeostat demand increased from `0.0005` to `0.32456607717984454`;
  N/F remained zero and stored material did not increase.
- M2 finite feeding: failed its required restoration check. Final
  `E_stored=55.84948101858201`, below the window start; distance to target and
  both A/R replete-reference comparisons also failed, although resource
  conservation, finite nonnegative state, and viability passed.
- M3 sustained clamp: failed. Final `E_stored=76.82632823803954` is below the
  lower target bound `0.95*E_target`; Q4 slope was
  `0.004561319184379901`, against the allowed
  `0.00007871648451166625`, and Q4 mean `h` was `1.0`, above `0.95`.
- M4 and M5: correctly not executed because the frozen M3 qualification did
  not pass.

Gate 3 therefore returns:

`DCDEV019_COORDINATED_METABOLIC_HOMEOSTASIS_NOT_ESTABLISHED`

The directive's stop condition applies: no Phase 4 exploration, Phase 5 food
encounter, or Phase 6 repeatability behavior work is authorized from this
result. No controller tuning or second controller was attempted.
