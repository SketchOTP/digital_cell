# Event schema and ledger

`EventV1` records event ID, accepted simulated time, accepted step, replicate, event type, organism/parent/lineage IDs, environment ID, protocol ID, and metadata. The event set includes founder, birth, fission start/completion, death, extinction, resource pulse, scarcity, damage, environment switch, mutation, transfer, and experiment end.

`EventLedger` is append-only and rejects backward accepted time or step. Validation fails closed on duplicate IDs, protocol/replicate/environment drift, unknown organisms, death-before-birth, double births, double deaths, dead parents, children before parents, and malformed ancestry. Event hashes are exported with replicate provenance.

Environment pulses, scarcity transitions, damage, environment transitions, mutation, and experiment termination are appended only when their scheduled or adapter-reported event occurs.
