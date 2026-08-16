# Gate results

The local DC-DEV-006 assay passes Gates 0–6:

0. Scope contains one static primitive and one contact signal only.
1. Far-obstacle zero-world trajectory matches the accepted DC-DEV-005 path.
2. Contact force is local; coordinate movement is performed by mechanics.
3. Contact stimulus is deterministic, bounded, and local.
4. Contact raises local regulatory activity while distant patches remain at the
   matched control value in the first step.
5. Repeated identical contact loads the existing adaptation trace and lowers a
   later matched contact response; zero-activity recovery reduces the trace.
6. Growth/remesh continuity passes and fission transfer fails closed.

Preservation regressions and exact-head remote CI remain required before the
directive can be architect-qualified.

