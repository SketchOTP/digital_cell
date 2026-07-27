# D-097 processing-path reconstruction

The frozen eight H and eight neutral processing-heavy/repair-heavy pairs were
reconstructed with mutation and fission disabled for the original 1,000-step
endpoint.

Mean H processing-heavy minus repair-heavy effects:

- processing allocation: `0.45`;
- expressed processing catalyst: `0.426501`;
- internal resource exposure: `-39.242047` (greater conversion lowers residence);
- nutrient/fuel conversion: `10.518073`;
- activated-resource production: `10.518073`;
- reserve inflow: `0`;
- reserve change: `0`;
- reserve-funded growth: `0`;
- readiness: `-0.027226`.

Stages through activated-resource production are `DIFFERENT`. Reserve
accumulation and growth are `BYPASSED`; readiness is consequently `MASKED`.
The first broken link is activated-resource production to reserve accumulation.

The reason is exact: `reserve_schema_load_ok` recognizes D-091 through D-094
equations but not
`autopoietic_material_mesh_finite_catalytic_allocation_v1`. Reserve chemistry
and reserve-funded growth therefore fail closed under the D-096 schema.
