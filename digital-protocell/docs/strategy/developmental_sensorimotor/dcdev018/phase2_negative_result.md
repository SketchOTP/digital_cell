# DC-DEV-018 Phase 2 metabolic qualification result

Formal qualification was run from committed homeostat freeze `4291747` with
the exact DC-DEV-016 settlement/deprivation and the frozen parameters in the
Phase 1 document.

Finding:

```text
DCDEV018_INTEGRAL_METABOLIC_HOMEOSTASIS_NOT_ESTABLISHED
```

Failure diagnostic:

```text
DCDEV018_FAIL_SOURCE_OUTPUT_INSUFFICIENT
```

The result is not controller saturation. The derived-resource arm delivered
`8.33048196639209` N units and ended at `E_stored = 53.4029934261998`, below
the deprived starting value `60.82781514212436`. Its Q4 stored-material
slope was `-0.66704798098099`. The sustained matched-precursor arm ended at
`E_stored = 31.2540892006136` with Q4 slope `-0.19158697130204`; its maximum
assimilation capacity was `1.07089297644381`, below the frozen cap
`2.3684629878513`.

Passed checks included exact feature-off trajectory parity and resource
world-loss conservation. The starvation arm produced no additional A without
N/F. The finite-resource restoration and sustained-homeostasis gates failed.

The committed machine-readable evidence is:

- `experiments/generated/dcdev018/metabolic_results.json`
- `experiments/generated/dcdev018/failure_diagnostic.json`
- `experiments/generated/dcdev018/protocol.json`
- `experiments/generated/dcdev018/artifact_manifest.json`

Per the directive, controller tuning, another controller, sink changes,
behavior, exploration, resource encounter, and repeatability work were not
started. This is a valid negative result for the integral homeostat attempt,
not evidence for the persistence loop.
