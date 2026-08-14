# Exact parity preflight

For candidates processing-heavy, repair-heavy, and neutral; environments H, B,
and Neutral; and seeds `1..=8`, the authoritative `pre_fission_assay` path was
compared with `DigitalCellMeshAdapter` using mechanics, topology, and fission
disabled.

All `72/72` cells passed the frozen tolerance:

`abs(A-B) <= 1e-9 * (1 + abs(A))`

The maximum absolute residual was `0.0`, and boolean survival/fission endpoints
matched exactly. No reproductive event was possible in this preflight.
