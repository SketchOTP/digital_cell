# D-013 Preflight

Before any expensive Stage E reference, run:

```text
R = 22
25,000 accepted substeps
```

Required demonstrations:

- 10,000 and 25,000 checkpoints exist
- rejected attempts do not enter convergence windows
- activation-potential and material accounting are complete
- artifact validator passes
- checkpoint continuation works
- termination is `MAX_ACCEPTED_SUBSTEPS_REACHED` unless valid convergence or biological failure occurs

Artifact root:

```text
digital-protocell/experiments/generated/d013/preflight/
```

Preflight is not a Stage E scientific result and must not be used to derive solver corrections.
