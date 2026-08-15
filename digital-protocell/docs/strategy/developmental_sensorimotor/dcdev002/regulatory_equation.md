# Regulatory equation

For patch `i`,

```text
neighbor_mean_i = 0.5 * (activity_prev + activity_next)

d(activity_i)/dt =
    2.0 * (neighbor_mean_i - activity_i)
  + 4.0 * stimulus_i * (1 - activity_i)
  - 0.5 * activity_i

activity_i_next = clamp(activity_i + 0.02 * d(activity_i)/dt, 0, 1)
```

All patches read the complete prior state and commit together.  The constants
are substrate-test constants, not biological fitness parameters.  No parameter
screening or post-result adjustment is permitted.
