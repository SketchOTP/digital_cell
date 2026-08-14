# Neutral control contract

Treatment and neutral protocols must use the same founder preparation, population size, placement, seed policy, mutation protocol, accepted horizon, termination, generation tracker, event schema, and measurement. Only the declared environmental selective-pressure field may differ.

The current crate exposes one protocol validator and one executor shape; it does not provide treatment-specific generation or lineage logic. Future parity validation should compare normalized protocol fields and fail on accidental drift before execution.
