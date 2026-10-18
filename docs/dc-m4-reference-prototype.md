# Bounded prescribed-stroke reference prototype

This dependency-free Python module executes the accepted v2 architecture as a bounded scalar reference. It is a new reference implementation, separate from production Rust and native biological trajectories. The entry base is `fb3366995de9a8dbc5bceb6d3ae24ed653972dca`. Prior contracts and architecture remain unchanged. Constants in the executable demo are explicitly illustrative reference-unit fixtures, not calibrated production values.

The repository governance skill and complete retained historical contract govern this work. The new ledger additions, current state and profile were refreshed against the accepted entry. Parent-owned execution and independent review supply validation; this document does not claim that unrun checks passed.

## Interface and owned boundary

`Params(K, lam, theta, epsilon_A, k_s, gamma, c0, h)`, `Material(P, M, rho_s, phi)` and `Accuracy(force, width, step_energy, cumulative_energy, max_iterations)` are frozen dataclasses. Every physical value must have exact type `fractions.Fraction`. Integer/bool/float/string inputs, including NaN and infinity, reject instead of being silently converted. The finite iteration count is a separately validated ordinary integer. No production values or optional physical defaults are selected. Theta is stored to declare the complete illustrative parameter set; no exponential bias law or native J trajectory executes or is inferred.

The only supported run entry is `Run.cold(params, material, accuracy, A, W=Fraction(0))`. It fixes all parameters, material and error budgets for the run and starts with `a=0,b=P,L=L0=M/rho_s,Ec=Us=0`, finite A/W and zero sink/error ledgers. Inactive polarity and mature scaffold are explicit supplied initial materials, not simulated energy-free synthesis. The supported guards are P,M>0, rho_s>=1e-15, L0>=1e-3, 0<phi<=1, positive fixed coefficients, and 0<L<=4L0. Existing W has zero reference energy. No imported active state or stretch preload can enter through this interface.

`run.cycle(d, bracket=(lo,hi))` returns a new frozen Run only after the complete prescribed monotone reaction and mechanical solve pass. It accepts neither a replacement Params nor a supplied final length. Frozen State and tuple-valued certificate history are retained in every old Run. Constructor calls/imported state and `dataclasses.replace(run,...)` reject; ordinary attribute mutation raises `FrozenInstanceError`. This is an interface/immutability contract, not security against Python reflection, private internals, or `object.__setattr__`.

`run.unsupported(operation)` always rejects. There is no accepted diffusion, synthesis, maturation, rupture, material change, remesh, fission, parameter change, OFF toggle or nonzero pressure/bending/active/contact/external port. Such operations are not silently disabled in production: no production integration exists. The fixed-endpoint collinear boundary admits signed internal coupling/stretch forces without imposing an invented coupling cap. It is not the native nonnegative-tension API or an explicit Euler adapter. Native stretch caps are inactive on the declared domain. A two-free-endpoint model would use gamma/2 for length drag and is not executed here.

## Reaction work and atomic funding

At the current fixed geometry, the module computes the exact quadratic coupling energy `Ec=KP/2*(epsilon-lambda*a/P)^2` along the monotone prescribed stroke `a -> a+d`, with `b -> b-d` and donor bounds `-a<=d<=b`.

For `xi_star=P*epsilon/lambda`, Emin is zero if xi_star is within the closed stroke interval and otherwise min(Estart,Eend). The exact positive/negative work variations are `Vplus=Eend-Emin`, `Vminus=Estart-Emin`. The complete cost is reserved from pre-stroke finite A:

`z=c0*abs(d)+Vplus/epsilon_A`, `A'=A-z`, `W'=W+z`;

`Qchem=epsilon_A*c0*abs(d)+Vminus`.

The battery owner is the explicitly assumed ideal `EA=epsilon_A*A`, W reference zero. `Delta EA+Delta Ec+Qchem=0` exactly. No W heat double count, A refund, downhill fuel credit, within-stroke energy recovery or additional energy store exists. An equal-endpoint-energy stroke through the minimum still consumes positive uphill work and releases downhill heat. Local tests of such a preloaded geometry are algebra witnesses; they are not cold-run entry or claimed trajectory reachability.

## Certified mechanical solve

At fixed pools and material, `U=Ec+Us`. The native stretch receiver is `Us=ks*(L-L0)^2/(2L0)` below rest and `Us=phi*ks*(L-L0)^2/(2L0)` above rest. Its exact rational secant is `(Us(new)-Us(old))/(new-old)`, with the true derivative limit for equal lengths. This evaluates the complete piecewise potential across L0. It never substitutes a single branch for a crossing.

The actual root solver evaluates

`R(new)=gamma*(new-old)/h + Tbar_c + Tbar_s`,

`Tbar_c=KP/L0*((new+old)/(2L0)-1-lambda*a/P)`.

The caller supplies a positive ordered bracket wholly within `(0,4L0]`. Exact endpoint evaluations must certify `R(lo)<=0<=R(hi)`. Convexity and gamma/h>0 make R strictly increasing, so this bracket contains the unique scalar root. An exact endpoint or midpoint root is retained as a point certificate. Otherwise exact rational bisection narrows the bracket; no domain clamp, fallback root, adaptive coefficient or hidden extra iteration is permitted.

Acceptance simultaneously requires bracket width<=sealed width budget, absolute force residual<=sealed force budget and absolute per-step energy defect<=sealed step-energy budget, before the finite iteration cap. Merely bracketing a root or including zero in a wide interval does not pass. All certificates contain lo,hi, endpoint signs, chosen rational root approximation, force residual, width, energy defect and iteration count. Because arithmetic is exact rational there is no floating roundoff/noise enclosure to estimate; the remaining numerical defect comes solely from the approximate root.

For the actual accepted rational displacement, the code separately recomputes and checks

`delta = Delta Ec + Delta Us + Qdrag = R(new)*(new-old)`,

`Qdrag=gamma*(new-old)^2/h>=0`.

It reports delta and leaves heat and material ledgers untouched. The global signed energy imbalance equals `sum(delta)`, while the sealed cumulative absolute budget requires `sum(abs(delta))<=cumulative_energy`. Thus cancellation cannot conceal budget consumption. Root error is also enclosed by the certified bracket width; this is an exact certificate rather than an observed convergence heuristic. The approximation does not claim exact energy conservation when delta is nonzero.

A complete accepted cycle conserves a+b=P and A+W=initial(A+W) exactly. Its total modeled balance is

`EA+Ec+Us+Qchem_cumulative+Qdrag_cumulative-epsilon_A*A_initial = signed_energy_error`.

The signed total is bounded by cumulative absolute error. Positive geometry Delta Ec receives no A top-up; stretch can supply it through the same force balance. Coupling release can load stretch and is not separately labeled heat. The only mechanical heat sink is Qdrag.

## Failure semantics and test boundaries

Every calculation uses scratch local values until reaction donor/funding, bracket/domain, actual solve, force/width/energy accuracy, cumulative budget and whole-run identities pass. A late iteration, bracket or cumulative-budget failure publishes no new Run and retains the preceding geometry, amounts, nonzero cumulative sinks, defect totals and certificate history. There is no partial paid reaction commit or repaired ledger.

The focused tests specify multiple cold-reachable prescribed cycles, an independent exact analytic tensile-branch root oracle, certified approximation bounds, exact-root behavior, true rest-crossing local secants, equal-energy cross-minimum path funding, split-path variation additivity and separate cold-reachable staged runs. Two staged mechanical cycles need not equal one unsplit cycle. Further controls cover donor/A exhaustion, zero fuel/no-op, bad domain/type/scaffold inputs, immutable coefficients/state, imported-state rejection, unsupported operations, late mechanics failure after nonzero sinks and cumulative budget exhaustion. Local preloaded algebra witnesses are explicitly distinguished from reachable histories.

The parent runs these bounded checks; the implementation author has not launched tests or workloads. Suggested invocations are `python3 -m unittest discover -s scripts -p test_dc_m4_reference_prototype.py` and `python3 scripts/dc_m4_reference_prototype.py --output NEW_PATH.json`. The demo emits deterministic sorted JSON with rational values represented as strings. The optional file uses exclusive creation and never overwrites an existing path. Existing files and all production source remain outside this module's write path.

## Scientific disposition

This reference demonstrates only what its actual verification establishes: a prescribed-stroke, fixed-material, scalar source/receiver implementation with explicit rational solver error and rollback. It does not identify K/lambda/theta/epsilon_A, measure ATP conversion, run native kinetics, validate full MaterialMesh integration, correct R13 damping, or qualify morphogenesis/reproduction. The independent calibration-evidence acquisition plan is a separate parent-owned deliverable. Missing data remain missing; synthetic constants, trajectories and arithmetic are not independent calibration evidence.
