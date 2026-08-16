# DC-DEV-008 implementation contract

## Environment

The world contains one static disk of radius `1.5` centered at `[4.8, 0.0]`
for the resource-bearing arm. Its finite initial inventory is `3.0` mass
units of N and `3.0` mass units of F. The noncontact arm uses the same disk
and inventory at `[30.0, 30.0]`; the resource-free arm uses the local disk with
zero inventory.

The region stores finite inventory explicitly. While inventory remains, its
local boundary concentration is fixed by the declared initial material
volume; every accepted inward flux is clamped by remaining inventory and
removed from the world. Once inventory reaches zero, uptake is zero. This is
an environmental material-boundary convention, not a new organism law.

## Organism path

For each exposed mesh edge, the adapter reuses `mesh_transport::permeability`
with the edge occupancy and `TransportParams.k_flux`. Delivered N/F is added
to the existing interior concentrations. The next step is the unchanged
`reactions_step`, which performs the existing N+F→A+W chemistry; existing
D-091 reserve chemistry then stores/releases A as R.

The adapter does not call or alter the global `transport_step` implementation.
The remesh compatibility check uses the existing coupled mechanics path only
to verify body authority and does not make the resource world decide growth,
remesh, death, fission, or heredity.
