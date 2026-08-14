# D-094 evidence chronology

## Gate 0 result

The D-094 architecture first appears in the preserved D-094R change at
`82bf09d13b1cad3f7734386c8060ad28315cef41`, whose parent is the D-093 seal
`973222edafdba55cb911236ddb39f79a7f63171e`. There is no distinct D-094 tag;
the later tag `D-094-autocatalytic-selection-rejected` resolves to the sealed
D-094R result commit `935359eea2fcdb08cb1365f58128eaba3f10f3e8`.

| directive | commit | tag | source tree | binary hash | config hash | artifact path | manifest hash | conclusion | status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| D-094 | `82bf09d13b1cad3f7734386c8060ad28315cef41` | none; D-093 parent `973222e...` | `973222edafdba55cb911236ddb39f79a7f63171e` | NOT_RECORDED | NOT_RECORDED | `experiments/generated/d094/manifest.json` | `a497a4c4352745ff5ddc6f0e2987d74c1ca4695c00afef22c6bfe1836d91465e` | heredity and functional phenotype gates passed; Gate 6 pending/zero-generation blocker | historical evidence |
| D-094R | `bf58edddef40753107ba18854eb85cc41ec78859` | none | source implementation at `bf58edd...` | `6c49dd04411cce128ddcb9008d5ecbd4b77afe5da2a7a0cdc3a88f8a4c25f8aa` | H `8cd865e...`; B `7c60fe...`; neutral `56c163...` | `experiments/generated/d094r/gate6/attempt_001/` | `e9b03ff268da69b91a8ced053b311cc7e1e0c439c75503dde6e7002207d2e01e` | reproduction-qualified Gate 6 source and analysis hardened | superseded as execution description by D-094R2 |
| D-094R2 | sealed result `935359eea2fcdb08cb1365f58128eaba3f10f3e8`; source `bf58edd...` | `D-094-autocatalytic-selection-rejected` | source tree `d9dc1f0e9543be22019044d6033062d7502f22d3` | `6c49dd04411cce128ddcb9008d5ecbd4b77afe5da2a7a0cdc3a88f8a4c25f8aa` | H `8cd865e452f3f59239e21f80006b7b3e54fa8239175ca90712df58b1c1fd6694`; B `7c60fe55d4149de1628878f82f8b6290e67183866602f89092d128ba98332412`; neutral `56c16311a06336f003fe37466c13895d2f0779d1d85769d98aa780996ec80658` | `experiments/generated/d094r/gate6/attempt_001/` | `e9b03ff268da69b91a8ced053b311cc7e1e0c439c75503dde6e7002207d2e01e` | `D094_AUTOCATALYTIC_SET_HEREDITY_QUALIFIED_SELECTION_REJECTED` | immutable valid negative evidence |
| D-095 | `9a6dfd85b9f0e6f157a33c96256d6d5d48056a7a` | `D-095-evolutionary-architecture-selected` | D-095 manifest | NOT_RECORDED | NOT_RECORDED | `experiments/generated/d095/manifest.json` | `b85f25104aa36bcbeef2dba874ffda8cd2c89fa230e8e969a5314f9544d98209` | environment/phenotype interaction absent; Candidate B selected for D-096 contract only | depends on D-094 selection failure |
| D-096 | `b06ef28f3530fad8e3f6f3704e015b86981159a0` | `D-096-finite-allocation-physiology-fail` | D-096 manifest | NOT_RECORDED | NOT_RECORDED | `experiments/generated/d096/manifest.json` | `898bcf7cafdfad77017f60a5ea8a9f45cdfe7a3f9ed69bd6a0c90d2106dfc0f4` | processing advantage not established at Gate 5 | later dependent physiology route |
| D-097 | `4ca2a12fb2bfef49135392abfbe0628693f11b73` | `D-097-processing-implementation-defect` | D-097 manifest | NOT_RECORDED | NOT_RECORDED | `experiments/generated/d097/manifest.json` | `e426bcff610015eef40fbd390e872f6bd8b2cecf1c4b55a0eca74be28cdc37f6` | activated-resource production to reserve accumulation defect | later repair route; no D-094 reinterpretation |

The later directives are relevant as dependent negative/diagnostic evidence,
not as authority to rewrite D-094R2 or to authorize a new D-094 campaign.
