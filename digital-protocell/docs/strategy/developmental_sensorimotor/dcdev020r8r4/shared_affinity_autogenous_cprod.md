# DC-DEV-020-R8-R4

## Status

Observer-only counterfactual audit. Entry authority is the accepted R8-R3
head `c9b200ee24b88c542eeb0c14038867f4c7fbb466`. Production chemistry,
production behavior, and DC-DEV-021 remain unauthorized.

## Candidate

The single tested topology is the existing R6 source paired with the
target-free, memoryless shared-affinity catalyst-production law:

```text
q_c(C) = C / (K_C + C)
J_C,shared = k_c_prod * A * (1 - q_c(C))
```

`K_C` is the existing frozen `ReactionParams.q_c = 0.3`. No parameter,
target, timer, history, deficit state, source feedback, or behavior input is
introduced. The observer implements the law by overriding only the
counterfactual `k_c_prod` coefficient before calling the existing reaction
step. The certified chemistry source is not changed.

## Protocol

- R6 source: `K_PL=0.017556661171593057`, `p=0.0003277429681759396`.
- Finite feed: `N=F=19.878372106390554`, 480 accepted steps.
- Dose scales: `0.75`, `1.00`, `1.25`.
- Sustained assay: `N=F=0.1476710565778127`, 8,000 accepted steps.
- Sustained target reference: `E_TARGET=77.91027880846893`.
- Controls: frozen current production and zero-production R8-R2 reference.
- D016 is retained as a preservation/reference context and is not required to
  establish homeostasis.

Execution is fail-closed. The three-cycle reversibility assay is not run when
the sustained gate fails.

## External evidence

The literature is used only for architectural context:

- Hofmeyr and Cornish-Bowden, PMID 10878248: coupled supply-demand control
  method; no constants or equations imported.
- Negative autogenous regulation, PMC210154: topology context only; no
  molecular mechanism or parameter imported.
- Wu et al., `10.1038/s41564-022-01310-w`: dynamic reserve reference only; no
  timing constant or biological value imported.

PubMed/PMC anti-bot pages limited direct text retrieval during the local
browser pass; the cited identifiers and disposition remain recorded without
importing unsupported values. Nature's article page was readable and supports
the reserve/transition context.

## Evidence

Compact authoritative JSON is stored under
`experiments/generated/dcdev020r8r4/`. The dense per-step ledger is stored in
the governed Atlas evidence area listed in `external_evidence_manifest.json`.

The authoritative classification is written by the executable assay and must
remain `PENDING` for architect acceptance until exact-head CI and review pass.

