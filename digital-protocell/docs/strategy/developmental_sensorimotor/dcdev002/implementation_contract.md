# Implementation contract

`regulatory-core` depends one-way on `chemistry-core` for immutable mesh
observation types.  `chemistry-core` does not depend on `regulatory-core` and
its source is unchanged.

The mesh adapter accepts `&MaterialMesh` and produces `LocalMaterialFrameV1`.
`RegulatoryNetworkV1` accepts only that frame.  The regulator cannot receive
`&mut MaterialMesh`, and there is no callback into chemistry, mechanics,
growth, transport, fission, death, reserve, or any effector/motor surface.
