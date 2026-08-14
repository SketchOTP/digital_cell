# Neutral control contract

Treatment and neutral protocols must use the same founder preparation, population size, placement, seed policy, mutation protocol, accepted horizon, termination, generation tracker, event schema, and measurement. Only the declared environmental selective-pressure field may differ.

`ReplicateRunner::run_campaign` is shared by treatment and neutral campaigns. `DefaultSelectionObserver` compares actual paired result vectors, reports both means, absolute/relative effects, replicate count, direction consistency, and a bounded normal-approximation half-width. It never synthesizes a neutral value from the treatment result.
