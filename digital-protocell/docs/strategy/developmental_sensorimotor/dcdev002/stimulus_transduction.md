# Physical transduction

For edge `i`, `L_i` is the current immutable mesh edge length and `L0_i` is
the existing material rest length.  The positive tensile strain is

```text
epsilon_i_plus = max((L_i - L0_i) / max(L0_i, 1e-12), 0)
```

Patch `i` receives

```text
stimulus_i = clamp(0.5 * (epsilon_(i-1)_plus + epsilon_i_plus), 0, 1)
```

Compression produces zero positive-tensile stimulus.  The scalar is not
assigned a semantic interpretation such as pain, threat, touch, food, or
reward.
