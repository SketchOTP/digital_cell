# Selection qualification contract

SR-003 does not certify selection. It defines the minimum future interpretation boundary: required completed generations, pressure before sufficient reproduction, population viability, measurable heredity and phenotype, viable neutral control, and complete event/provenance data.

`SelectionObserver` reads separate treatment and neutral `CampaignResultV1` values and produces `SelectionAnalysisV1`; it has no causal callback into an adapter or population. A campaign cannot receive `VALID_NO_SELECTION_EFFECT` or `VALID_SELECTION_EFFECT` unless every paired replicate is qualified and both campaigns have complete event/provenance records. The current statistics are replicate means, absolute/relative effect, normal-approximation half-width, replicate count, and direction consistency.
