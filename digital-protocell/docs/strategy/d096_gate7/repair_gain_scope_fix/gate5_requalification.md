# Impacted Gate 5 requalification

The original Gate 5 protocol and semantic criteria were replayed without changing candidates, environments, seeds, horizon, endpoints, fission, or mutation.

H passed: per-seed reserve effects were positive, with mean `0.43641615393804756`, exceeding the neutral mean `0.4116922309199964`.

B failed: repair-heavy minus processing-heavy final-material effects were negative in every seed:

```text
[-0.430056230685679, -0.4315960726275705, -0.42918418459608176,
 -0.430056230685679, -0.4315960726275705, -0.42918418459608176,
 -0.430056230685679, -0.4315960726275705]
```

The B mean was `-0.430415659891489`, versus neutral mean `-0.4288781940320341`. All processing-heavy and repair-heavy founders survived. Because the original B criterion requires every paired effect and the mean effect to be positive and stronger than neutral, Gate 5 is invalidated under the corrected production implementation.
