# Compatibility regression

`digital-protocell/crates/chemistry-core/tests/d098_tests.rs` verifies:

- D-091 metabolic reserve remains accepted;
- D-092 catalytic template remains accepted;
- D-093 template network remains accepted;
- D-094 autocatalytic set remains accepted;
- a valid D-096 stamped finite-allocation mesh is accepted;
- base, unknown, and unstamped historical identities remain fail-closed;
- the D-096 processing-heavy and repair-heavy candidate hashes remain the
  sealed values `faa5c27f...` and `e3897848...`;
- the reserve causal path is active and closes accounting.

The full existing D-096 integration test remains unchanged and passes its
original reciprocal Gate 5 assertions.
