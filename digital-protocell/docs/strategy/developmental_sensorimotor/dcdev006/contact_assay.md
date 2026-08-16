# Contact assay

The executable assay is
`examples/dcdev006_gate_assay.rs`. It uses a 24-vertex radius-5 mesh and the
frozen obstacle center `[5.0, 0.0]`, radius `0.9`.

The matched controls are:

- zero-world control: a far-away inert obstacle, producing an all-zero force
  and signal vector;
- contact condition: the fixed obstacle, contacting only the rightmost local
  boundary patch;
- repeated-contact condition: identical mesh, obstacle, current frame, and
  current activity on every exposure while only the existing adaptation state
  persists.

The assay records local force support, deterministic signal hashes, contact vs
no-contact regulator activity, repeated-contact response, recovery under zero
activity, ordinary remesh continuity, and fission rejection.

The conclusion is withheld from architect acceptance until the exact pushed
head is run by remote CI.

