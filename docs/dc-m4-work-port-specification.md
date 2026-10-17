# Prospective parameter and mechanical-work-port feasibility protocol

**SPECIFICATION ONLY. No calibrated parameter, production implementation or biological run is authorized by this document.** Entry is `a007e7e08acc768f08b83d96139a7cea25f0e04e`. The accepted conditional conversion-work contract and its limitations remain unchanged. This defines a finite, ordered decision procedure for that candidate; failures do not permit selecting another formula, force compensator, operator split or parameter family.

The present disposition is `PROSPECTIVE_PROTOCOL_DEFINED`; independent calibration anchors are `MISSING`, the naive heat/top-up adapter is `NO_GO`, and current mechanics APIs do not realize the required signed discrete work port. These are different findings. They do not establish that every realization of the candidate is impossible.

## 0. Seal the investigation before collecting evidence

Create a manifest with exact source base, accepted contract MD/JSON/review hashes, this protocol MD/JSON hashes, every mathematical fixture and expected identity, future input file IDs/hashes, train/holdout roles, measurement apparatus or independent model provenance, parameter-estimator version, uncertainty certificates, arithmetic precision and solver/domain/cap policies. Record all four production values as null. Record that the thirty R13 outcomes, growing/fission labels and sign-correction objective are forbidden calibration inputs.

A subsequent stage may consume an earlier stage's sealed estimate only through the deterministic recipes below. Reseal its inputs and roles before collecting the next observations. No changing probe ranges, estimator, uncertainty rule, domain or holdout assignment after inspecting that stage's responses. Missing files produce `MISSING_EVIDENCE`; malformed or contradictory bindings produce `EVIDENCE_DEFECT`. Synthetic data generated from this candidate's own equations are reference controls, never independent calibration evidence.

Exact rational fixtures require exact zero residual. Floating implementations must provide outward-rounded interval certificates or an operation-count forward-error bound, `gamma_n=n*u/(1-n*u)`, with the actual counted n and target arithmetic unit roundoff u bound before outcomes. Measurement uncertainty is independently certified and propagated; no favorable universal correlation cutoff or post-result absolute tolerance is introduced. A residual interval containing zero establishes compatibility only, never an equality pass by itself. Before observations, the manifest must specify independently justified absolute accuracy budgets and maximum enclosure widths from the certified arithmetic/instrument capabilities and the resolution needed for the stated physical decision. Require the full residual enclosure to lie inside that accuracy budget, its width to meet the separately sealed tightness bound, and every required inequality to hold across the entire certified parameter/data enclosure. Missing budgets, excessively wide enclosures (including a trivial interval containing zero), unresolved parameter regions or an interval overlapping a decision boundary give `INCONCLUSIVE`, not a pass. Accuracy budgets cannot be widened after observing residuals.

## 1. Four independent dimensionless groups

Choose reference scales from a sealed nonbiological calibration fixture: `Fref` in the fixed legacy force unit, `Lref>0`, `Pref>0` in polarity amount, and `Aref=c0*Pref`, where c0 is the unchanged R3 net-conversion overhead price. One acceptable force-unit anchor is the frozen native `k_s`; choosing the unit does not identify a coupling parameter. These reference scales are fixture/unit definitions, not production values.

Use

- `lambda`;
- `kappa=K*Pref/(Fref*Lref)`;
- `beta=K/theta`;
- `eta=K/(epsilon_A*c0)`.

All four are dimensionless and independently relevant: mechanical strength, kinetic bias and fuel affordability are not interchangeable. Equivalently use `Khat`, `thetahat`, `epsilonAhat`, lambda in the common energy unit `Fref*Lref`. A common energy rescaling is a unit gauge only if the legacy mechanical energy unit rescales too; with fixed legacy force/length units it cannot remove kappa. A-only ledgers do not identify epsilon_A, and rate-only data identify a ratio rather than K's mechanical magnitude.

The algebraic inverse is `K=kappa*Fref*Lref/Pref`, `theta=K/beta`, `epsilon_A=K/(eta*c0)`. No numerical production value is supplied here.

## 2. Independent force calibration identifies K and lambda

The observable is the **additional signed coupling tension** after matched, independently measured legacy force subtraction, with physical a,b,L,L0 and P known. The accepted candidate predicts

`Y=T_c*L0/P=K*epsilon-(K*lambda)*x`, `x=a/P`.

Collect the predeclared force-design lattice `x={0,1/4,1/2,3/4,1}`, `epsilon={-1/4,0,1/4,1/2}`, and relative `(P/Pref,L0/Lref)={(1,1),(1,2),(2,1),(2,2)}`. These 80 rational reference coordinates define measurement roles, not a parameter search. Train only on the four rows with x in {0,1}, epsilon in {0,1/4}, P=Pref and L0=Lref; all other rows are held out. The two-column design `[epsilon,-x]` must have certified rank two. Infer coefficients `s=K` and `r=K*lambda` by interval linear inversion, then `lambda=r/s`; require a certified bounded positive joint parameter region for K and lambda (a unique pair for exact noiseless data), sufficiently tight under the independently sealed identification budget.

Use every one of the four training observations together. Define the joint feasible region S={(s,r): s>0, r>0 and Y_j_low <= s*epsilon_j-r*x_j <= Y_j_high for all four training rows}, with certified propagation of input uncertainty. Do not select a favorable pair of pivot rows after observing data. Preserve the s,r correlations when projecting lambda=r/s and when propagating into later stages. Seal S before using holdouts; held-out observations do not refit or shrink S. Robust prediction over the whole sealed region must meet the independent accuracy/tightness budget. If only part of S is compatible with a held-out observation, report INCONCLUSIVE rather than selecting that part as a successful estimate.

An observation must originate in an independently specified physical/constitutive calibration source, not `T_c` evaluated from assumed K/lambda. Actual native passive mechanics has no existing active-fraction-dependent elastic law; fitting it cannot silently create one. If no independent source exists, return `CALIBRATION_ANCHOR_MISSING` and keep K/lambda null. If valid observations contradict the affine force law outside their certified uncertainties on train or held-out rows, return `CONSTITUTIVE_CANDIDATE_REJECTED`. Rank deficiency or intervals that fail to bound an identified positive region within the sealed accuracy budget give `PARAMETERS_UNIDENTIFIED`. Do not widen the model/range or drop unfavorable rows.

## 3. Signed kinetic calibration identifies theta

Consume only the previously sealed joint force-calibration region. Let [lambda_low,lambda_high] be its certified finite projection and define the deterministic fixture selector lambda_star=(lambda_low+lambda_high)/2 before any kinetic response. At fixed independent P,L0 and x in {1/4,1/2,3/4}, prescribe `epsilon=lambda_star*x+sigma*d_epsilon`, sigma in {-1,+1}. The selector defines probe geometry only; it does not replace the unknown true lambda or discard K/lambda correlations. Propagate the full sealed joint region into g=K*lambda*(lambda*x-epsilon), including certified geometry roundoff. Require the entire g enclosure to have the prescribed nonzero sign on each probe. If uncertainty makes a sign ambiguous or a fixture falls outside the independent domain, issue no calibration observation there and return INCONCLUSIVE or DOMAIN_UNCOVERED; never choose a different selector or larger probe after responses. Use `d_epsilon=1/4` for estimation and {1/8,3/8} as held-out probe magnitudes. These balanced probe amplitudes are fixture geometry, not production gains; their deterministic dependence on the sealed lambda_star selector is declared in advance. If any point lies outside the supported independent calibration domain, mark `DOMAIN_UNCOVERED`; do not move the probe after seeing a response.

Measure an unclipped local net reaction amount-rate J with independently observed a,b,L and sufficient finite A; gross turnover is not inferred. Use the frozen coefficients to compute Rplus/Rminus. Require Rplus,Rminus>0 and independently demonstrated absence of donor/energy clipping. Seal calibration observation intervals h, h/2 and h/4, where h is the native polarity substep, before responses. A source-certified truncation/refinement bound must reconcile all three observations and measurement uncertainties; an unrefined finite-step delta/h is not automatically an instantaneous empirical rate. No production timestep is changed by this calibration protocol. Compute `g=K*lambda*(lambda*x-epsilon)`. The inversion is

`y=(-J+sqrt(J*J+4*Rplus*Rminus))/(2*Rminus)`;

`theta=g/(2*log(y))`.

Use an algebraically stable equivalent and interval propagation where needed. This solves the declared net-rate law, not detailed balance. Nonzero g of both signs must yield the same bounded positive theta interval and predict every held-out row within its uncertainty certificate. Zero-g rows are controls with no theta information. Missing independent J observations yield `CALIBRATION_ANCHOR_MISSING`; incompatible observations reject the candidate; clipping or ill-conditioning yields `PARAMETERS_UNIDENTIFIED`/`INCONCLUSIVE` rather than an altered rate or selected subset.

## 4. Independent work/fuel calibration identifies epsilon_A

Require a physically identified finite loading device or work-producing channel with a measured force-displacement input `Wload`, and independent A/W material observations. In the calibration fixture, the channel must be the actual mechanical donor; an accounting top-up with no corresponding work-producing channel is not an anchor. The existing effort-duration actuator price is not such a force-displacement conversion constant.

Use two prescribed positive work magnitudes in ratio 1:2 for estimation and the intermediate 3/2 magnitude as held out. Magnitudes are chosen and sealed inside the independently supported apparatus domain before responses. Independently separate synthesis A-to-polarity material, the c0*abs(net conversion) overhead, other native obligations and additional work A. Infer `epsilon_A=Wload/Awork`, requiring positive nonzero independently measured Awork and no duplicate work source. Work integration and material observations must not be generated by charging `DeltaEc/epsilon_A` with a presumed epsilon_A and then dividing it back out.

Require consistent positive epsilon_A intervals across estimation and holdout, A/W closure, exact Ec storage/release and the work-port gates below. Include zero-motion, zero-work, and negative-release controls with no A recovery and no material synthesis mislabeled as W. W is spent material; it does not by itself measure heat. Missing such an independently owned work/fuel channel leaves epsilon_A and eta unassigned.

## 5. Audit the complete mechanical power balance first

Let `dq=qnew-qold`, `mu=dt/gamma`, and require an actual proposed update

`dq=mu*(Flegacy+Fc_DG+Rcontact)`.

Then `Dgamma=dot(dq,dq)/mu`, `Wlegacy=dot(Flegacy,dq)`, `Wc=dot(Fc_DG,dq)=-DeltaEc`, and `Wcontact=dot(Rcontact,dq)` must satisfy

`Dgamma=Wlegacy+Wc+Wcontact`.

For passive friction, `Qcontact=-Wcontact>=0`, so `DeltaEc+Dgamma+Qcontact=Wlegacy`. If a contact projection is used, its residual force must be derived from the actual before/after proposal and accounted separately; it cannot be assigned passive dissipation merely because the final polygon is simple. The native geometric projection has no already-certified work ledger for this new coupled solve.

Decompose Wlegacy by actual source channel: native stretch/bending/pressure, paid actuators and boundary/contact forces. For a reference spring with known potential Ulegacy, verify `Wlegacy=-DeltaUlegacy` using its discrete gradient. For a real native channel whose stored energy or donor is not established, return `WORK_OWNER_UNRESOLVED`; do not invent a global passive potential or relabel a negative channel work as bath heat. Conserve sign information and cross-work; do not inspect each coupling edge as if all other forces were absent.

Only actual gamma drag, passivity-certified contact loss and separately identified irreversible channel losses may enter the terminal bath, each once. Coupling release can instead increase another mechanical store. Positive Ec loading already receives mechanical work `-Wc`; an additional A debit represents that same input only if a physically named work-producing channel and its source ledger are identified and not also counted as an independent mechanical donor. A compensating force, extra split-step actuator or changed integration policy would be a new architecture requiring separate approval, not a repair performed by this specification.

## 6. Mandatory exact counterexamples and coupled reference fixture

Fixtures below are dimensionless mathematical controls; their numbers are never proposed production parameter values.

**Release into a legacy store.** With mu=1, constant coupling force Fc=1 and legacy force Flegacy=-1/2, dq=1/2. Therefore Wc=1/2, Wlegacy=-1/4 and Dgamma=1/4. Coupling energy decreases by 1/2, but 1/4 is stored in/absorbed by the opposing legacy channel. Claiming all 1/2 as bath heat overcounts actual drag. A concrete candidate-energy realization is P=K=L0=1, x=1/2, lambda=5/2, Lold=1, Lnew=3/2; its discrete force is Fc=1 and DeltaEc=-1/2. The opposing channel can be the independently declared linear potential Ulegacy=L/2 on this fixture interval.

A cap-respecting companion has Fc=1/4, Flegacy=-1/8, mu=1, dq=1/8, Wc=1/32, Wlegacy=-1/64 and Dgamma=1/64. Use P=K=L0=1, x=1/2, lambda=5/8, Lold=1, Lnew=9/8. This shows that the accounting issue is not caused solely by the illustrative unit-force magnitude exceeding the native external cap.

**Positive loading with duplicate fuel input.** Take mu=1, Fc=-1, Flegacy=3/2, dq=1/2. Then Wlegacy=3/4, Wc=-1/2, DeltaEc=1/2 and Dgamma=1/4. An independently owned external drive already supplied the complete 3/4. Adding another 1/2 of A-derived physical work without an actual additional channel creates unmatched input. Candidate realization: P=K=L0=1, x=0, positive lambda arbitrary, Lold=7/4, Lnew=9/4. The existing force balance cannot absorb the extra work by relabeling its source. The corresponding A top-up can be considered only after its physical role and the external donor accounting are explicitly reconciled.

**Zero total force and finite rotation.** Equal opposite Fc/Flegacy gives dq=0 and zero work even when Ec is positive. Separately use noncollinear old/new edge vectors and exact `vbar=(rnew+rold)/(Lnew+Lold)` to discriminate the true discrete gradient from start-tangent, normalized-midpoint and clipped-force substitutes.

**Controlled coupled spring benchmark.** Before any native integration, define one scalar positive length with finite initial A, known legacy spring `U0=k0*(L-Lr)^2/2`, candidate Ec, optional prescribed constant external force whose boundary work donor is explicit, positive gamma and fixed dt. Use discrete gradients for both declared energies. At fixed x/P/L0 the scalar equation is affine in Lnew and has denominator `gamma/dt+(k0+K*P/L0^2)/2>0`; therefore it has one exact root. Check positive length/domain, signed work, `DeltaU0+DeltaEc+Dgamma=Wexternal`, per-fragment funding and terminal sinks. When the candidate's no-transfer-credit/top-up policy cannot be embedded into that balance, record the conflict rather than changing U0, parameters or splitting the operator to force agreement.

A second exact two-spring witness uses q=L-1, qold=0, coupling stiffness kc=1/4 and preferred q_c=1, legacy stiffness kl=1/4 and preferred q_l=-1/2, and mu=1. Its unique discrete-gradient root is qnew=1/10: DeltaEc=-19/800, DeltaUlegacy=11/800 and Dgamma=1/100, with Fc=19/80 and Flegacy=-11/80. Both forces respect 1/2. The loading companion uses preferred q_c=-1 and q_l=2, giving qnew=1/5, DeltaEc=11/200, DeltaUlegacy=-19/200, Dgamma=1/25, Fc=-11/40 and Flegacy=19/40. These are mathematical quadratic port witnesses, not a claim that every preferred center maps to the candidate's positive-lambda material domain. Additional A work of 11/200 without an identified port duplicates the loading input. The isolated physical-candidate realization above and these coupled quadratic accounting witnesses have distinct declared roles.

Run the same sealed reference cases with no contact, an explicitly modeled unilateral ideal contact, and a dissipative Coulomb/friction contact with known complementarity/passivity conditions. Freeze contact branches and their ownership before numerical solving; branch change is not silently differentiated through. The reference benchmark distinguishes a consistent port realization from an invalid adapter; it does not prove the legacy mesh is that reference spring.

## 7. Native adapter and solver feasibility decisions

The current edge-tension APIs explicitly reject negative tensions (`mesh_mechanics.rs:341–343`, `mesh_self_contact.rs:203–206`), while Tc can have either sign. They use the current edge tangent, not the endpoint discrete gradient. The signed external-force hook is bounded and the native update is explicit (`mesh_mechanics.rs:244–246`); self-contact projects an explicit displacement (`mesh_self_contact.rs:244–260`). Thus direct injection into existing tension arrays is `ADAPTER_NO_GO_SIGNED_FORCE`, and a one-shot external force followed by the existing update is `ADAPTER_NO_GO_DISCRETE_WORK` unless a separately proved exceptional domain applies. No new force clipping or larger cap is permitted.

For a future separately authorized adapter, record its exact force assembly, pre/post state and complete residual, bound all external forces jointly, and prove the signed work identity and source/bath allocation above. A converged Newton solve is not a uniqueness certificate. The radial energy has tangential Hessian eigenvalue Tc/L, negative below the preferred extension, so the scalar proof does not extend automatically to a mesh. Require a prospectively sealed compact admissible domain, interval existence and uniqueness certificate or equivalent proof on that domain, plus exclusion of other admissible roots. A certificate that covers only the local basin is insufficient for a global uniqueness claim. No certificate, unresolved branch, multiple roots, nonsimple state or cap violation rejects the proposal. Solver domains/tolerances cannot be enlarged after failures.

A native realization's manifest must also fix the reservation/debit/force-evaluation order. A enters native pressure (`mesh_mechanics.rs:89–92`) and available-A actuator funding; debiting extra work fuel changes later forces even with unchanged coefficients. Fixed Flegacy is a restricted reference-fixture assumption, never a full-mesh claim. Future evidence must identify whether force assembly uses reserved pre-debit state or post-debit state, how native actuator/chemical obligations are retained, and where A/W commits. Post-debit assembly requires the corresponding recomputation/coupled fixed point. A pressure or funding term evaluated from stale unmatched A cannot be accepted as same-state parity. No runtime staging policy is silently selected here: an absent independently justified staging choice gives WORK_PORT_STAGING_UNSPECIFIED and blocks the native gate. This is a mandatory future-input field, not a reason to modify pressure, actuator cost, timestep or native biology now.

All state, material, A/W, Ec, time/counters and topology remain scratch until complete validation. A late finite-A shortfall, guard activation cost or one unfunded daughter requires whole-transaction rollback. Reference snapshots must show bit-equivalent old state after deliberate corruption/failure, but these reference controls do not certify production rollback.

The contact variants, interval solver certificates, native staging witnesses and whole-state rollback snapshots above are **mandatory future evidence**, not capabilities certified by the present arithmetic checker. Any current reference arithmetic/control execution is a separately recorded limited witness; it does not execute this full coupled native protocol.

## 8. Ordered decision procedure and permissible conclusions

1. **S0 authority/seal:** fail mismatched provenance as EVIDENCE_DEFECT; otherwise freeze protocol and records.
2. **S1 dimensional/identifiability reference:** exact inverses, rank and covariance controls must pass; no physical estimate is accepted from synthetic equations.
3. **S2 independent force/kinetic/fuel anchors:** consume the fixed stages above. Missing anchors stop calibrated-value issuance as CALIBRATION_ANCHOR_MISSING; inconsistent data reject the constitutive candidate; uncertainty/domain gaps remain explicit. Every production value remains null until all four are jointly identifiable with held-out support.
4. **S3 current adapter audit:** the mandatory counterexamples give NAIVE_PORT_REALIZATION_REJECTED; source incompatibilities give CURRENT_ADAPTER_NOT_REALIZABLE. This is a current-adapter result, not universal candidate impossibility.
5. **S4 reference coupled port feasibility:** on separately authorized future evidence, require complete force/work/storage/fuel/bath closure, exact discrete work, finite funding, domain/cap/contact compatibility and solver certificates for every sealed fixture. Do not bypass S3 by adding an unapproved mechanism.
6. **S5 native realization eligibility:** only all stages passing with independently calibrated anchors and a separately approved realizable adapter permits proposing a bounded implementation directive. It does not itself authorize implementation or the thirty-state correction assay.

The present task completes the **definition and independent verification of this prospective procedure**. Actual calibration data collection, native coupled solver implementation, biological campaigns, R13 sign fitting and automatic successors remain outside scope. Failure or missing evidence is a usable decision, not permission to tune around a fixture.
