# Local conversion-work ownership candidate

**CONTRACT ONLY — PARAMETRIC CANDIDATE, NOT CALIBRATED OR IMPLEMENTED.**

This proposes one concrete incremental mechanical energy and funding contract after `LOCAL_REACTION_COUPLING_PHYSICAL_OWNERSHIP_OR_ENERGETICS_GAP`. Entry is `b2de15dc0d7a7ee3f25b7993c558099d17a150a0`. New constitutive assumptions below are design choices, not recovered R3 physiology or evidence that damping will be corrected. Production Rust, all existing coefficients, R4/R10 actuators, mechanics, ecology and historical evidence remain unchanged. Separate implementation authorization and scientific qualification are still required.

## Ownership, dimensions and domain

Each material edge owns an additional *derived* coupling-energy sector. Its active/inactive polarity amounts are `a,b`; `P=a+b`. Its physical mature structural mass is `Mmat`, structural density is `rho_s`, mature rest length is `L0=Mmat/rho_s`, and actual edge length is `L`. Do not interpret the native numerical `1e-15` rest-length floor as physical mature material. The owner is the same edge correspondence that owns polarity and structural material, not an observer label or mesh centroid.

Four positive constitutive parameters are explicit and unassigned: `K` and `theta` have energy/polarity-amount units; `epsilon_A` has energy/A-amount units; `lambda` is dimensionless. They are not imported biological values, fitted gains, or inferred from the existing actuator funding price. No parameter value is selected by this contract. `theta` is a phenomenological load-response scale, not a claim that it equals thermal kT.

Validate finite nonnegative `a,b,Mmat`, finite positive `rho_s,L`, finite nonnegative physical `L0`, positive finite parameters, and finite intermediate arithmetic. On a nonruptured edge with `P>0` and `Mmat>0`, set `x=a/P` and `epsilon=(L-L0)/L0`. Otherwise define coupling energy, affinity and coupling force to be exactly zero. Negative/NaN state is an error, not the zero-energy branch. Guard crossings are real energy-changing events and must be settled; they cannot erase energy silently. Large energies near the first positive mature mass may make the candidate unfundable; this is a declared feasibility risk, not permission for an energy floor or free initialization.

## One energy and its reciprocal mechanical term

The candidate constitutive assumption is that active polarity favors positive local extension of an additional edge-local elastic sector:

`Ec = (K*P/2) * (epsilon - lambda*x)^2`.

This nonnegative energy is extensive in P at fixed strain and active fraction. It is additional to the existing passive mechanics; it does not pretend that the actual clamped V4 passive force already has this potential. No additional independently initialized energy scalar is required: Ec is reconstructed from the owned material state and immutable parameter/schema identity. The enabled identity and accounting provenance must persist.

For local conversion extent `xi`, `da=dxi`, `db=-dxi` and P remains fixed. The exact conversion-associated energy derivative and curvature are:

`g = partial Ec/partial xi = K*lambda*(lambda*x - epsilon)`;

`partial^2 Ec/partial xi^2 = K*lambda^2/P`.

Thus for a finite conversion `d`,

`DeltaEc = g*d + K*lambda^2*d^2/(2*P)`.

This is the **coupling contribution** to conversion affinity, not the full chemical affinity of the existing driven reaction.

For geometry with `a,b,L0` fixed,

`T_c = partial Ec/partial L = (K*P/L0)*(epsilon-lambda*x)`.

Coupling endpoint forces are equal and opposite: `F_i=T_c*t`, `F_j=-T_c*t`, where t points from endpoint i to j. This is a new reciprocal mechanical contribution, not a reused R4 actuator, tension multiplier or free active-force request.

A future discrete mechanics implementation must preserve exact coupling work. For old/new edge vectors `d0,d1` and lengths `Lold,Lnew>0`, use

`Tbar=(K*P/L0)*((Lnew+Lold)/(2*L0)-1-lambda*x)`;

`vbar=(d1+d0)/(Lnew+Lold)`;

`Fbar_i=Tbar*vbar`, `Fbar_j=-Fbar_i`.

Then `sum(Fbar_endpoint dot displacement) = -DeltaEc` exactly. Simply evaluating a continuous gradient once before a finite Euler step does not establish this identity. Any future solver must find a unique physically valid step consistent with the existing integration/contact constraints and this discrete force, or reject it. No force clipping may silently break reciprocity. Existing hook/cap bounds must remain satisfied; otherwise reject the candidate proposal rather than change a bound. Solver existence, uniqueness, contact compatibility and sufficient admissible force range are unqualified.

## Signed reaction proposal

Preserve the native concentration law and frozen rate coefficients. Before donor limiting, express the native amount-rate terms as

`Rplus=k0*b+kf*a*a*b/(L*L)`;

`Rminus=kd*a+kq*a*a*a/(L*L)`.

On supported edges the candidate signed rate is

`Jc = Rplus*exp(-g/(2*theta)) - Rminus*exp(g/(2*theta))`.

On unsupported edges use `g=0` and the unmodified kinetic proposal. A native substep proposes `d=h*Jc`, then applies the existing donor interval `[-a,b]`. Nonfinite exponent/rate/proposal rejects the whole candidate transaction; no outcome-selected cap, epsilon or fallback is allowed. The exponential bias is a declared phenomenological constitutive assumption. It preserves the baseline kinetic expression when g=0, but does not claim microscopic detailed balance, thermodynamic equilibrium, forward/reverse turnover accounting, or a proven correction of R13.

For `d>0`, active material grows exactly at inactive material's expense; for `d<0`, the reverse holds. The exact finite DeltaEc, rather than `g*d` alone, determines energy settlement. Retain the existing R3 net-conversion overhead `c0*abs(d)` A amount with equal W credit. Here c0 is the unchanged `conversion_a_per_amount`; it is not gross reaction turnover or the mechanical-work energy scale.

## Atomic finite settlement and energy ledger

The sole additional fuel donor is existing finite A owned by this organism. Positive coupling-energy increments require **additional** A beyond unchanged chemistry/conversion/actuator obligations. No negative energy event creates A, W credit, future spending credit, or a hidden battery.

For each declared local owned energy fragment `f`, compute `de_f=Eafter_f-Ebefore_f`. Set

`Awork_f=max(de_f,0)/epsilon_A`;

`released_f=max(-de_f,0)`.

Pay `sum(Awork_f)`; never replace it by `max(sum(de_f),0)/epsilon_A`. Unrelated downhill edges cannot subsidize uphill edges. Cell A is the existing common material pool, not a new remote allocator. Baseline prospective obligations and this extra work charge must jointly fit the same finite pool. Otherwise roll back the *entire enclosing candidate accepted step*, including polarity, body, chemistry, A/W, topology, time, counters and all sidecars. No partial charge, partial mechanics, survivor culling or state reseeding is allowed.

For a conversion subtransaction, `Aoverhead=c0*abs(d)`. Debit `Aoverhead+Awork` and credit exactly that amount of W. Record `Qoverhead=epsilon_A*Aoverhead` and `Qstate=released` for non-geometrical changes. The exact incremental identity is

`epsilon_A*(Aoverhead+Awork) = DeltaEc + Qoverhead + Qstate`.

For geometry, define signed coupling work **to mechanics** `Wc=-DeltaEc`. Positive DeltaEc is loaded through a newly declared A-funded coupling-work input port: `Win=epsilon_A*Awork=DeltaEc`. This is a conservative external-fuel policy for this candidate sector, not work recovered or derived from the native passive/active funding laws. Negative DeltaEc leaves the coupling sector only as `Wout=-DeltaEc` through the reciprocal mechanical port, terminating in the explicitly owned overdamped mechanical bath. Count that terminal `Qmech=Wout` once. Do not also add the same release to Qstate. Signed work is an internal transfer, not a second external energy source or an A refund.

Therefore the geometry sector identity is `epsilon_A*Awork = DeltaEc + Qmech`; its mechanical-port audit additionally requires the exact discrete identity `Wc=-DeltaEc`. Work-input/output and the terminal bath entry are alternate views of that transfer and must not be added twice. W is **spent-fuel material**, not heat: `epsilon_A*DeltaW` is not all Q when part of the input resides in Ec.

A→loading port→Ec and Ec→mechanical terminal are the **same signed port** audited by Wc, not additional work beside the reciprocal force work. Its assignment to a terminal overdamped bath is a required candidate realization, not an observation about the existing composite mechanics: simultaneous legacy forces can exchange work with other sectors. The current arithmetic does not verify that allocation. Any implementation must demonstrate this terminal ownership without counting energy stored elsewhere as heat.

This closes the **incremental coupling-sector ledger** only. It does not establish a global passive potential, a thermodynamic calibration of legacy forces, total mesh work/heat closure, or microscopic thermodynamics. The original effort-duration charge remains separate adopted overhead; it supplies no assumed mechanical-work credit. A future implementation that cannot realize the signed work port and exact force identity fails this contract rather than relabeling an accounting top-up as native mechanical work.

Use scratch state, collect all obligations, verify every local material and energy identity, and commit once. A late shortfall or nonfinite result must leave the byte-equivalent pre-step state. Numerical tolerances and nonlinear-solver contracts must be preregistered before implementation tests, not selected from morphology.

## All state-changing operators and lifecycle

Every operation that changes an argument of Ec is covered. There is no special free transport, growth, guard activation or topology reset.

- **Existing-polarity enable/initialization:** A-funded creation of the new elastic sector pays its complete initial Ec/epsilon_A in additional A-to-W work fuel. Existing R3 polarity-material synthesis remains a distinct A-to-polarity material transfer, with no fabricated W or heat. Historical already-present polarity does not make Ec free. Insufficient A rejects enablement without altering the original individual.
- **New polarity synthesis/removal:** Account the original physical material transfer separately; settle the derived Ec change against the same finite remaining A. Removing polarity can release Ec to the nonmechanical disposal sink once; it never restores source A.
- **Reaction:** Four existing substeps and donor limits remain explicit. Native c0 net overhead and additional positive Ec charge are distinct. Entire-step atomicity includes failure in any later substep.
- **Conservative transport:** Compute the native donor-limited batch with unchanged diffusion coefficients/staging on scratch pools. Each affected edge owns its exact before/after Ec; charge the sum of positive per-edge increments and dispose negative increments independently. Do not reorder fluxes or rescale diffusion to find funding. Unfundable batches reject the whole enclosing step. Active+inactive material conservation remains exact.
- **Geometry:** Freeze a,b,L0 during the mechanics subtransaction; use the reciprocal discrete force/work pair above. Charge positive Ec loading, route negative Ec through the mechanical bath, and preserve all geometry/contact validity checks.
- **Structural growth, maturation, turnover, damage, rupture and rebond:** After the unchanged material law proposes its state, settle every affected edge's Ec change before committing the coupled step. This includes appearance/disappearance of mature support and rupture guard crossings. These events cannot free-reset or initialize energy. Their native material/A/W ledgers remain separate.
- **Remesh:** Use the exact existing conservative material/polarity correspondence. Partition each source edge's prior Ec among its inherited material fragments in proportion to inherited P; source P=0 implies Ec=0. Allocate each target edge's new Ec among its incoming fragments by their P shares. Compare corresponding old/new energy fragments and charge positive differences separately; no unrelated source drop may pay a target increase. Homogeneous proportional splits preserve epsilon,x and sum Ec exactly. Heterogeneous merges or chord-geometry changes require explicit settlement.
- **Fission:** Apply the same correspondence-fragment rule. Closing edges with no inherited polarity have P=0 and Ec=0; no copied energy or new polarity is inserted. After the unchanged fission proposal and its native material costs/partition, each daughter pays its own additional positive fragment charges from its allocated A. No sibling subsidy or parent/daughter energy duplication. Any daughter's shortfall rejects the full fission-containing transaction; never keep only the funded daughter. Parent-to-daughter total polarity and accounted energy shares must reconcile.
- **Death/disposal:** Transfer remaining Ec to the declared disposal sink once with explicit identity; do not convert it into survival support or A. Native death authority remains unchanged.
- **Checkpoint/restart:** Store the enabled schema and parameter identity, original physical state and its already-settled A/W. Recompute Ec and validate the identity; never recharge a legitimate restart or permit an uncharged ON snapshot. Corrupt or missing enabled provenance fails closed without mutating the file or original state.
- **OFF and toggling:** A never-enabled OFF path delegates directly to historical execution, skipping all new validation/state. `lambda=0` is not OFF: Ec can remain nonzero. ON→OFF is a separate declared discharge transaction, disposing the remaining Ec exactly once before detaching the sector. Repeated toggles and split/merge or reaction round trips cannot generate A or escape a positive charge.

Energy fragments/provenance are transactional accounting ownership, not an extra behavior input or newly invented material species. Whether an implementation needs additional persisted provenance beyond the enabled schema/parameter identity is an engineering question; material amounts cannot be duplicated to solve it.

## Hard issues and minimum verification before implementation

This contract closes a symbolic candidate, not parameter feasibility. K, lambda, theta and epsilon_A require an independently justified prospective specification; their values cannot be chosen to reverse the known thirty-state sign. The extra elastic sector and conversion/load policy are new physical assumptions. Small mature rest lengths, support activation, large force demands or depleted A can make the candidate non-executable. The discrete-gradient/contact solver may lack a unique acceptable root. Energy settlement during native diffusion/topology may reject useful biological transitions. None authorizes changing M1, concentration physics, mechanics coefficients, R3 diffusion or inherited results.

Verification must cover finite uphill/downhill conversion and exact quadratic work, both signs, zero P/support and invalid state, exact geometry work with rigid-transform covariance, separate fuel/material/heat accounting, initially unaffordable enablement, complete rollback on late shortfall, conservative transport and homogeneous split/heterogeneous merge/fission accounting, repeated toggles/round trips, restart provenance, and exact OFF parity. Any later raw diagnostic settings must be sealed before outcomes. R9 remains a negative control; this contract introduces a bidirectional conversion-energy term and reciprocal geometry work, rather than recycling its forward-only tensile multiplier.

The present deliverable is **DEFINED_PARAMETRIC_CONVERSION_WORK_OWNERSHIP_CANDIDATE**. Calibration, implementation, total mesh thermodynamics, corrected feedback, morphogenesis, reproduction and lifeform completion remain unestablished. No successor executes automatically.
