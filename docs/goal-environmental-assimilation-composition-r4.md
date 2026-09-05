# Goal-mode environmental assimilation composition R4

Status: `GOAL_AGENT_PROVISIONALLY_ACCEPTED_REPLAN`

This is a goal-agent result. It is not an independent Architect acceptance.

## Largest unresolved end-goal gap

The project still lacks a causal chain in which environmental material is
transferred before reproductive divergence, increases organismal structural
mass, and produces viable fissioned descendants. The unchanged fission gate
remains `1.35 * birth_mass`.

The sealed runtime causality audit remains negative: active-transfer and
transfer-disabled controls both fissioned at step 25, while the first finite
environmental transfer occurred at step 256. That pre-resource event cannot
count toward resource-causal reproduction.

## Capability increment tested

This increment tested the first explicit finite assimilation composition after
the existing environmental-carrier audit closed Route A. The opt-in path is:

```text
finite environmental N/F
→ organism-owned assimilation_n/assimilation_f
→ existing N+F activation law
→ existing A pool
→ existing structural growth/fission gates
→ ordinary mechanics/remesh/fission path
```

The historical routes remain disabled by default. The new compartment is
area-based, partitioned by physical fission, conserved across V4 remeshing,
and does not alter the fission criterion or the frozen production selector.

## Observed result

The faithful existing-growth replay reached step 1101 with:

- finite N delivered: `278.7133755462829`;
- finite F delivered: `278.7133755462829`;
- N/F reaction-equivalent processed: `275.6712480280603` each;
- existing A produced: `275.6712480280603`;
- structural mass recorded across the existing growth step: `24.05079502860805`;
- world conservation error: `0.0` for N and F;
- fission events: `0`;
- structural mass / birth mass: approximately `0.21099`; below the unchanged
  `1.35` gate.

The transfer-disabled control delivered and processed zero environmental N/F,
had zero fission events, and also did not reach the gate. Its recorded growth
(`24.083975170152016`) is founder-state growth, not environmental growth. The
new path therefore proves real transfer/retention/processing, but the unchanged
growth law does not convert that processed A into enough structural mass for
resource-causal reproduction.

## What this supersedes

This result supersedes the assumption that adding an organism-owned finite
assimilation compartment plus the unchanged growth law is sufficient to close
the current hard-contact resource-to-fission architecture. It does not reopen CLOSURE-006 through
CLOSURE-014 and does not justify a new motor, contact, allocation, gain,
threshold, timer, or fission-gate variant.

## Replan boundary

The current organism/world material-flow architecture remains insufficient for
the required temporal and quantitative transfer-to-growth chain. The next
architecture must be justified at the organism/world material-flow boundary
from source-level prior art and Digital Cell conservation/fission invariants.
It must explain how environmental material remains available at a biologically
useful rate before reproductive divergence without borrowing an observer state,
changing the `1.35` gate, or hiding pre-resource fission as success.

No successor material-flow implementation is authorized by this negative; the
opt-in composition tested here is the bounded implementation under review.

## Stop condition

Do not run another local active-work, contact, material-fraction, or
parameter-free motor/allocation variant. Do not change the fission gate. Resume
runtime work only after the next organism/world material-flow architecture has
an explicit conservation, timing, transfer-disabled, and three-generation
validation contract.

## Preservation boundary

The new state is opt-in. Scientific runtime changes are isolated to the new
assimilation fields/module, conservation partitioning, and the opt-in runtime
flag; M1, the production selector, historical runtime routes, uptake law,
reaction parameters, mechanics, traction, and PR #44 are not intentionally
changed. This branch asserts only provisional goal-agent status pending
independent Architect review.
