# DC-DEV-013 qualification result

| Measure | Active | Sensor-off | Motor-off | Zero reserve |
| --- | ---: | ---: | ---: | ---: |
| cumulative N+F acquisition | 0.354201468008014 | 0.364097551510532 | 0.364097551510532 | 0.364097551510532 |
| N delivered | 0.177100734004007 | 0.182048775755266 | 0.182048775755266 | 0.182048775755266 |
| F delivered | 0.177100734004007 | 0.182048775755266 | 0.182048775755266 | 0.182048775755266 |
| integrated exposed-patch time | 19.1999999999998 | 0 | 19.1999999999998 | 19.1999999999998 |
| reserve spent | 3.56414368489786 | 0 | 0 | 0 |
| maximum funded tension | 1.59994057802459 | 0 | 0 | 0 |

The active arm sensed contact on all 480 accepted steps and engaged the
funded motor and stick-slip path, but its movement reduced acquisition rather
than increasing it.  The first failed scientific gate is Gate 5, active
acquisition benefit; Gate 6 is also false because the active arm does not
finish with greater contact than both controls.  Gate 7 conservation error is
zero within the existing tolerance.  The empty sham has zero signal and zero
acquisition.

The 180-degree active repeat acquired `0.354201468008014`, with zero difference
from the unrotated active arm.  Maximum positive substrate work was
`2.02583592044033e-16`; material/vertex displacement agreement was within
`1e-8`.  The exact evidence is in
`experiments/generated/dcdev013/` and is generated from the frozen protocol.

Conclusion: `DCDEV013_RESOURCE_CONTACT_FEEDING_NOT_ESTABLISHED`.
