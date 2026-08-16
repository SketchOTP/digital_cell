# DC-DEV-011 qualification result

This is a coder-generated result package from the frozen protocol. It is
pending independent architect review and is not an acceptance record.

## Authority and scope

- Entry commit: `8d6fe59397cabfa47bc1d8103acd68f544acc190`
- Frozen protocol commit: `f263536bc2da028630fe09108d07e7ada2e8ca38`
- Branch: `strategy/dc-dev-011-local-stick-slip-traction`
- Horizon: 5,000 accepted settlement steps, then 240 active and 240 relaxation steps
- Mechanism: `digital_cell_passive_isotropic_stick_slip_v1`
- Frozen parameters: static `0.45`, kinetic `0.20`, zero-motion tolerance `1e-12`
- Scope result: no DC-DEV-010 directional substrate, DC-DEV-012, parameter sweep,
  chemistry reaction, growth, remeshing, fission, obstacle, or contact import

## Local qualification

| Arm | Final material-centroid displacement | Active displacement | Stuck contacts | Slip contacts | Reserve spent | Positive substrate work |
|---|---:|---:|---:|---:|---:|---:|
| active + isotropic stick-slip | `0.004404569847979622` | `0.004404569847979622` | `5711` | `49` | `6.102457322546062` | `1.5634994880216084e-16` |
| active, no substrate | `0.0018021246021144236` | `0.0011062114197869174` | `0` | `0` | `5.646078559948914` | `0` |
| motor-off + isotropic stick-slip | `0` | `0` | `5760` | `0` | `0` | `0` |
| zero-reserve + isotropic stick-slip | `0` | `0` | `5760` | `0` | `0` | `0` |

The active stick-slip arm exceeded both matched controls by more than the
`1e-10` tolerance, retained `1.0` of active displacement after relaxation,
engaged both contact regimes, and passed the 180-degree rotational control.
The material/vertex centroid agreement error was
`2.740863092043355e-16`; the regulatory trajectory was identical between the
forecast and applied substrate boundary.

## Verification state

Local Rust 1.89.0 checks passed:

- `regulatory-core` library: 35 passed, 0 failed
- Phase-1 metrics semantics: 4 passed, 0 failed
- D-088 focused regression: 4 passed, 0 failed
- evolution-harness: 40 passed, 0 failed
- DC-DEV-002 through DC-DEV-009 preservation assay invocations: passed
- scoped formatting: passed

The generated manifest intentionally remains
`PENDING_SCOPED_REMOTE_CI` until the exact-head GitHub workflow passes. No
architect acceptance is claimed here.

`NEXT_EXECUTION_STARTED:false`
