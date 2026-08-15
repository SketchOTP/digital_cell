# Implementation contract

`ContinuityNetworkV1` retains the DC-DEV-002 single scalar activity per current material vertex and applies the same synchronous local equation. `ContinuityMaterialFrameV1` adds immutable vertex positions solely for topology correspondence.

For each current vertex, `derive_local_mapping` selects the nearest previous vertex independently and rejects a source outside three local edge scales. There is no global assignment, interpolation target, optimization, or redistribution. The mapped field is then advanced synchronously using the current ring neighbors and the existing tensile-strain transducer.

The material adapter accepts `&MaterialMesh` only. No regulator API accepts `&mut MaterialMesh`, an organism command, an effector output, or a fission decision.
