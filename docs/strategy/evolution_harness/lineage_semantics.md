# Lineage semantics

Experiment identity is independent of Rust object identity. Every population record has `organism_id`, `parent_id`, `lineage_id`, `birth_event_id`, `birth_time`, `birth_generation`, and optional death time/state.

`LineageTracker` preserves ancestry after death and supports parent, children, lineage members, descendant count, lineage depth, death time, phenotype history, and hereditary-state history. Descendant count is graph-derived: in the deterministic A→B/C→D/E/F tree, A has five descendants and depth two.
