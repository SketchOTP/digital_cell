# Goal workflow scope reconciliation R12

The R12 push correctly passed the scoped shared-medium workflow, but two
archival Goal workflows also ran and failed their historical bounded-diff
allowlists because they saw the new R12 implementation. Those failures were
workflow-scope failures, not scientific failures.

The R10 current-flux-ledger and R11 contract-selection workflows now trigger
on push only when their own workflow, documentation, script, or evidence
surfaces change. They remain manually dispatchable for their exact historical
heads. This prevents later Goal work from being misclassified by stale
allowlists while preserving the historical workflows and their evidence.
