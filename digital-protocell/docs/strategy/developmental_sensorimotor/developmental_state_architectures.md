# Developmental state architectures

The preferred developmental state is distributed and persistent. Each local patch has bounded internal variables such as activation, adaptation, refractory/cooldown state, and developmental phase. These variables are not a second organism: they are local state coupled to the material patch and must have explicit provenance.

Three possible update families were compared:

* deterministic local regulator with short memory;
* local graph/spiking update with explicit neighbor edges;
* NCA-inspired local growth/regeneration update as an isolated reference.

The first implementation contract selects the first family. It can later host the second family behind the same local interface, while the NCA reference remains a non-authoritative spike. Development is therefore tested as persistence, differentiation of local state, response to perturbation, and return of function—not as target-shape matching.

