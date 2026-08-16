# DC-DEV-008 — Finite Spatial Resource Acquisition

This directive adds exactly one static circular world region containing finite
existing `N` and `F` material. The region reuses the certified permeability law
on individual exposed boundary segments and transfers material conservatively
into the existing `MaterialMesh::interior` pools before the existing reaction
step.

The assay compares an exposed finite-resource region, an identical zero-
inventory region, and an identical noncontact region over 120 accepted steps.
It does not add a chemical species, controller, sensor, actuator, reward,
fitness measure, or resource-seeking behavior.

## Acceptance

`DCDEV008_SPATIAL_RESOURCE_ACQUISITION_QUALIFIED` requires Gates 0–8, exact
head remote CI, and architect review. The first failed gate stops the
directive. DC-DEV-009 is not started by this work.

## Provenance

- Entry: `2968882769991f48c987ceb40c719fd351b2e046`
- Source: `strategy/dc-dev-007-active-contact-regulation`
- Implementation: `strategy/dc-dev-008-spatial-resource-acquisition`
- Runtime: `crates/chemistry-core/src/spatial_resource.rs`
- Assay: `examples/dcdev008_gate_assay.rs`
