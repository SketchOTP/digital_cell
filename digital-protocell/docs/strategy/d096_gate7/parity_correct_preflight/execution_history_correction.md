# R1 execution-history correction

The R1 manifest field `gate7_rerun:false` is scoped to the R1 shadow audit
runner: that runner did not invoke Gate 7. Repository-wide, the pre-existing
original Gate 7 workflow did run automatically during R1 synchronization.

Recorded runs are `31845151456` and `31845154403` on intermediate head
`b258126...`, and `31845606065` on the final R1 head `aa98e40...`. These runs
are not R1 shadow-audit evidence. The correction is append-only and does not
rewrite `sr004c` or the accepted R1 scientific evidence.
