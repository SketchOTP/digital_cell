# External ALife Audit Plan

No external code is copied or added by DC-SR-001.

## Tier 1 - evolutionary blocker

Audit in this order: Stringmol, Evo2Sim, MABE2, Avida, Aevol.

- Stringmol: executable/hereditary molecule roles and copier/interpreter/constructor closure.
- Evo2Sim: resource ecology, metabolism-selection coupling, batch/chemostat/seasonal timing, carrying capacity, cross-feeding, and generation-relative pressure.
- MABE2: organism/world separation and modular mutation/selection/placement/population/analysis interfaces.
- Avida: replicate structure, neutral controls, mutation experiments, lineage statistics, and selection effect sizes.
- Aevol: genome architecture, mutation/rearrangement, robustness/evolvability, phylogeny, and long-horizon lineage analysis.

Each audit must compare the external method to D-089-D-094 failure modes without weakening Digital Cell gates.

## Tier 2 - scalable world infrastructure

Audit Ribossome first, then ALIEN, for separable Rust/wgpu or GPU compute, fields, populations, sensors, movement, snapshots, rendering, profiling, and simulation/render decoupling. Reject any import that makes an external genome or controller the Digital Cell body.

## Tier 3 - discovery sidecars

Audit ASAL, CAX, Lenia, and Flow Lenia for bounded hypothesis generation, surrogate experiments, and candidate ranking. Sidecars may propose configurations only; governed Rust runs remain authoritative.

## Tier 4 - future references

Audit DISHTINY, Polyworld, and Tierra when the corresponding phase is authorized.

## Required audit record

For every project capture repository URL, exact source commit, license evidence, files examined, reusable/adaptable boundaries, original copyright, modifications, distribution implications, scientific risks, and a KEEP/ADAPT/REPLACE/BENCHMARK/ARCHIVE decision. Unknown license means no code reuse.

Integration gate: external source -> provenance/license review -> isolated adapter or benchmark -> causal boundary tests -> governed Digital Cell experiment -> long-horizon evidence.

