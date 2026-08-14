# Selection qualification contract

SR-003 does not certify selection. It defines the minimum future interpretation boundary: required completed generations, pressure before sufficient reproduction, population viability, measurable heredity and phenotype, viable neutral control, and complete event/provenance data.

`SelectionObserver` reads `ReplicateResultV1` and produces `SelectionAnalysisV1`; it has no causal callback into an adapter or population. Future statistics remain small and replicate-first: mean/median, difference, relative effect, interval, replicate count, and direction consistency.
