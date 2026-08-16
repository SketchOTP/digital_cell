# DC-DEV-008 — Finite Spatial Resource Acquisition

DC-DEV-008 qualifies one finite static spatial region containing existing N
and F material. Only boundary segments whose midpoints lie inside the region
can acquire material. The adapter reuses the existing D-086 permeability law
and hands imported N/F to the unchanged `reactions_step` N+F→A+W pathway and
D-091 A↔R reserve chemistry.

The primary matched arms are resource-bearing, resource-free, and
noncontact-resource. The fixed primary horizon is 120 accepted mechanics
steps. A separate fixed 2,000-step continuation verifies finite depletion and
zero uptake after exhaustion. The assay does not require navigation or
resource seeking.

No food points, reward, fitness, planner, new metabolic species, sensor,
actuator, plasticity trace, or evolution mechanism is introduced. The global
transport path remains unchanged for all prior assays.
