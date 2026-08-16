# Gate results

The executable assay records the gate booleans and measured values in
`experiments/generated/dcdev007/`.

- Gate 0: scope and entry authority pass.
- Gate 1: exact no-contact DC-DEV-006/DC-DEV-005 parity pass.
- Gate 2: contact signal, local regulatory increase, funded tension, and
  mechanics-mediated trajectory difference pass.
- Gate 3: active integrated contact penetration is lower than motor-off.
- Gate 4: active reserve is spent; zero reserve has zero active tension and
  matches passive contact.
- Gate 5: initial activity increase is local to contact-associated regions.
- Gate 6: repeated prior contact changes the later active response.
- Gate 7: existing no-contact recovery moves the response toward naive.
- Gate 8: ordinary remeshing remains supported and fission remains
  fail-closed.
- Gate 9: preservation is executed by the scoped workflow.

The assay emits `DCDEV007_ACTIVE_EXTERNAL_CONTACT_REGULATION_QUALIFIED` only
when all local gates pass. Remote CI and architect review remain separate
acceptance authorities.
