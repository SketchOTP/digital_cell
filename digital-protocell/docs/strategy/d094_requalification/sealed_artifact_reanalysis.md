# Sealed D-094R2 artifact reanalysis

The original files under `experiments/generated/d094r/gate6/attempt_001/`
were not rewritten or regenerated. A byte-preserving import of the available
manifests and Gate 6 rows is stored under
`experiments/generated/sr004/sealed_d094r2_import/`.

## Provenance

- Source commit: `bf58edddef40753107ba18854eb85cc41ec78859`.
- Source tree: `d9dc1f0e9543be22019044d6033062d7502f22d3`.
- Campaign binary SHA-256: `6c49dd04411cce128ddcb9008d5ecbd4b77afe5da2a7a0cdc3a88f8a4c25f8aa`.
- Attempt manifest SHA-256: `e9b03ff268da69b91a8ced053b311cc7e1e0c439c75503dde6e7002207d2e01e`.
- H config SHA-256: `8cd865e452f3f59239e21f80006b7b3e54fa8239175ca90712df58b1c1fd6694`.
- B config SHA-256: `7c60fe55d4149de1628878f82f8b6290e67183866602f89092d128ba98332412`.
- Neutral config SHA-256: `56c16311a06336f003fe37466c13895d2f0779d1d85769d98aa780996ec80658`.

## Reproduced conclusion

- H: 8/8 complete, generation 8, viable; frequency effect mean `-0.0278`.
- B: 8/8 complete, generation 8, viable; frequency effect mean `+0.0083`, far below `0.15`.
- Neutral: 8/8 complete, generation 8, viable.
- H descendant effects: `[-0.0304, +0.0214]`, crossing zero.
- B descendant effects: `[-0.0463, +0.0424]`, crossing zero.
- All 24 rows were viable; no extinction or numerical/checkpoint failure was
  used to explain the negative result.

The original preregistered result and the new harness-compatible
interpretation agree: this is valid negative selection evidence, not a
zero-generation or invalid-run result. Metrics not emitted by the historical
runner, including generation duration and phenotype-specific survival or
reproduction, remain `NOT_RECORDED`.
