# Lineage semantics

Experiment identity is independent of Rust object identity. Every population record has `organism_id`, `parent_id`, `lineage_id`, `birth_event_id`, `birth_time`, `birth_generation`, and optional death time/state.

`LineageTracker` preserves ancestry after death and supports parent, children, lineage members, descendant count, ancestor depth, and descendant depth. `lineage_depth` is the compatibility name for maximum descendant depth below the organism; a leaf has depth zero. In the deterministic A→B/C→D/E/F tree, A has five descendants and descendant depth two.
