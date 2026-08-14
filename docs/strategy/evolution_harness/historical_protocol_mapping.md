# Historical protocol mapping

`historical_protocols()` represents four prior designs as sealed-evidence fixtures:

| Historical design | Protocol/environment representation | Execution |
|---|---|---|
| D-090 | `d090_historical_mapping` / sealed D-090 analysis and dish artifacts | non-executable; runtime sweep values unresolved |
| D-091 | `d091_historical_mapping` / sealed D-091 analysis and seasonal artifacts | non-executable; derived sweep values unresolved |
| D-092 | `d092_historical_mapping` / sealed D-092 analysis and template artifacts | non-executable; campaign values unresolved |
| D-093 | `d093_historical_mapping` / sealed D-093 analysis and network artifacts | non-executable; selection campaign values unresolved |

Each fixture records exact repository source paths in `ProtocolProvenanceV1`; unresolved runtime-dependent values are listed and `execution_authorized` is false. No historical manifest is rewritten and no expensive campaign is run. The zero-generation rule is a synthetic harness regression, not a rerun of D-093.
