# Preregistration and parameter boundary

DC-DEV-006 does not screen contact parameters. The following values are fixed
before qualification:

| Quantity | Value |
| --- | ---: |
| contact stiffness per penetration length | `0.5` |
| mechanics-hook maximum force per vertex | `0.5` |
| stimulus normalization | `0.5` force units |
| obstacle center | `[5.0, 0.0]` |
| obstacle radius | `0.9` |

The contact signal is `clamp(|F_i| / 0.5, 0, 1)`. No value is selected from
assay outcomes. Existing mechanics `MechParams.dt`, DC-DEV-004 contractility,
and DC-DEV-005 plasticity parameters are inherited unchanged.

