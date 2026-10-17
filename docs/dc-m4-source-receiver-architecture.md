# Physically owned source and receiver: bounded architecture v2

This document defines one proposed mathematical architecture, `BOUNDED_REACTION_STRETCH_SOURCE_RECEIVER_V2`. It closes the energy balance of a fixed-material, single-edge prototype. It supplies an actual finite chemical source and a force-backed mechanical receiver. It is not a runtime implementation, parameter calibration, full MaterialMesh energy model, or demonstration of biological correction. Production values of `K`, `lambda`, `theta`, and `epsilon_A` remain unassigned.

Entry is `101e635d6498b9cfe451b2d702d64a4440939963`. The earlier conversion-work contract and work-port specification remain historical and unchanged. V2 proposes replacing v1's geometry top-up/disposal policy **only in a future bounded module**: geometry transfers energy between coupling and stretch stores through reciprocal forces; it does not debit A for positive coupling-energy changes. V2 also resolves reaction work along the physical monotone conversion path rather than allowing net endpoint work to hide a dissipative downhill/uphill crossing. These are explicit architecture revisions, not a claim that the v1 adapter became realizable. No production Rust, examples, coefficients, diffusion, ecology, actuator, or mechanics behavior changes here.

## Owners, units and domain

One intact edge owns active/inactive polarity amounts `a,b` with fixed `P=a+b>0`, fixed mature scaffold `M>0`, fixed material density `rho_s>=1e-15`, rest length `L0=M/rho_s>=1e-3`, and fixed mature structural fraction `0<phi<=1`. The prototype also owns finite, nonnegative A and W material pools. For V4, `M=m-m_young` and `phi=M/m`; for the historical fully mature contract, `M=m` and `phi=1`. No numeric rest-length floor supplies physical scaffold. All inputs must be finite, and `0<L<=4L0` throughout every accepted transaction. Nonzero pressure, bending, active/normal forces, contact work, external work, topology changes and material changes are outside this boundary.

Native `k_s>0` and `gamma>0` are fixed source-backed constitutive coefficients. Length has unit L, force F, energy E=F L, polarity amount P, fuel amount A, and time t. The explicit new assumptions are `K>0` and `theta>0` in E/P, `lambda>0` dimensionless, and `epsilon_A>0` in E/A. The native cost `c0` has A/P units. Native c0 is a net conversion overhead, not measured chemical free energy or gross turnover.

The finite chemical owner is the **ideal coarse-grained A-to-W chemical battery**, `E_A=epsilon_A*A`, with W assigned reference energy zero. This is a proposed constitutive ownership assumption: it does not identify A with ATP, establish local detailed balance, or account for synthesis of the scaffold/polarity material. W records spent fuel material, not heat. There is no new persistent energy tank.

## Cold preparation and finite capacity

The sole cold entry is `L=L0`, `a=0`, `b=P`, `E_c=U_s=0`, with explicitly given finite inactive P and scaffold material. Their formation is an initial-condition boundary, not a simulated A-to-P synthesis or energy-free material-creation claim. Native A-to-polarity synthesis lies outside the fixed-P prototype. Arbitrary prestrained or active native snapshots cannot be imported as funded stores.

Any desired `a>0` preparation must be generated at fixed `L=L0` by recorded reaction-coordinate transactions with the exact funding and overhead below. Geometry can subsequently store stretch energy only by the coupled mechanics balance. There is no debit-only preload operation and no arbitrary nonzero `U_s` initialization. The initial total available modeled energy is exactly `epsilon_A*A_initial`; no later regeneration or outside work is admitted.

Let `x=a/P`, `epsilon=(L-L0)/L0`. The coupling owner is

`E_c = K*P/2 * (epsilon-lambda*x)^2`.

The existing stretch force supplies the receiver potential on the bounded domain:

`U_s = k_s*(L-L0)^2/(2*L0)` for `L<=L0`, and

`U_s = phi*k_s*(L-L0)^2/(2*L0)` for `L>=L0`.

Both stores are nonnegative and finite. On this domain, `U_s <= k_s*L0/2*max(1,9*phi)` and `E_c <= K*P/2*max(9,(1+lambda)^2)`. Their actual sum is additionally bounded by initial battery energy minus accumulated heat. Material-fixed cold entry prevents an unrecorded preload from evading that bound. These bounds are capacity certificates, not target lengths or new energy sources.

## Reaction coordinate: physical chemical source

At fixed geometry, define local conversion coordinate `xi=a`, with transfer `d`: `a'=a+d`, `b'=b-d`, `-a<=d<=b`. P, L, L0 and phi remain fixed. The conjugate coupling work is

`g = dE_c/da = K*lambda*(lambda*x-epsilon)`;

`Delta E_c = g*d + K*lambda^2*d^2/(2*P)`.

The finite stroke follows a monotone physical path in xi from a to a+d at fixed L. The quadratic has at most one minimum, at `xi_star=P*epsilon/lambda`. Let `E_start=E_c(a)`, `E_end=E_c(a+d)` and `E_min` be zero when xi_star lies within the closed stroke interval, otherwise `min(E_start,E_end)`. The source supplies all uphill variation and the chemical sink irreversibly receives all downhill variation:

`Vplus=E_end-E_min>=0`, `Vminus=E_start-E_min>=0`;

`Vplus-Vminus=Delta E_c`.

Thus the battery supplies work through the conversion coordinate itself; no vertex moves and no mechanical source is invented during this stage. The path-resolved source debit and chemical sink are

`z = c0*abs(d) + Vplus/epsilon_A`;

`A'=A-z`, `W'=W+z`,

`Q_chem = epsilon_A*c0*abs(d) + Vminus`.

This is not endpoint-only `[Delta E_c]+` funding: a stroke that passes downhill through the minimum and then uphill cannot recycle the dissipated energy. In particular an equal-energy cross-minimum stroke still consumes `Vplus/epsilon_A` extra A and releases `Vminus` heat. Reserve the complete z from pre-stroke finite A before any stroke begins; downhill release cannot fund a later uphill segment. No hidden battery or within-stroke recovery is admitted.

The exact balance is `Delta E_A + Delta E_c + Q_chem=0`. The material ledger is `Delta A+Delta W=0` and `Delta a+Delta b=0`. Never also count `epsilon_A*Delta W` as heat: positive work is stored in `E_c`. There is no negative-work refund, A regeneration, or gross-turnover cost inference.

A prospective proposal may retain the earlier phenomenological signed work bias

`J = Rplus*exp(-g/(2*theta)) - Rminus*exp(g/(2*theta))`,

with native `Rplus=k0*b+kf*a^2*b/L^2` and `Rminus=kd*a+kq*a^3/L^2`. This is a declared design hypothesis, not a thermodynamic derivation. Donor limits and finite-energy feasibility act on the finite proposed transfer. The final finite transfer is checked using exact path variations Vplus/Vminus as well as `Delta E_c`, not `g*d` or endpoint positive change alone. A future sealed funding policy must select reject or a prospectively specified reduced transfer; the bounded reference contract chooses atomic reject on insufficient A, invalid/nonfinite state, or failed energy check. It never adapts K/lambda/theta/epsilon_A after an outcome. Native c0 is retained as separate net-transfer overhead.

## Mechanical receiver and reciprocal work

Hold `a,b,A,W,M,rho_s,phi,k_s,K,lambda` fixed during mechanics. The total potential is `U=E_c+U_s`. Positive geometry `Delta E_c` comes from a decrease of the force-backed stretch receiver within the same two-store force balance. It receives **no extra A debit**. A decrease of `E_c` is not automatically heat: `U_s` may absorb it.

Use a one-dimensional collinear edge with one endpoint fixed and the other free, no external load and no moving support. The fixed support does zero work. The free endpoint drag is gamma. For step h>0, solve inside the domain

`gamma*(L'-L)/h = -(Tbar_c + Tbar_s)`.

Here

`Tbar_c = K*P/L0 * ((L'+L)/(2*L0)-1-lambda*x)`;

`Tbar_s = (U_s(L')-U_s(L))/(L'-L)` for unequal lengths, and `T_s(L)` for equal lengths.

The stretch derivative is `T_s=k_s*(L-L0)/L0` below L0 and `phi*k_s*(L-L0)/L0` above L0. At L0 it is zero. Use the **true potential secant across L0**, not one selected branch or a coefficient average. U_s is C1 and convex. Future numerical evaluation must use cancellation-safe exact piecewise algebra with a certified roundoff enclosure. No energy residual is removed by clamping or ledger repair.

Multiplication by displacement gives

`Delta E_c + Delta U_s + Q_drag = 0`,

`Q_drag = gamma*(L'-L)^2/h >=0`.

Both force contributions act in the same force balance. The coupling work `W_c=-Delta E_c` and stretch work `W_s=-Delta U_s` are internal signed transfers; `W_c+W_s=Q_drag`. Do not separately add `W_c` to heat. The legacy counterexample `W_c>Q_drag` is allowed when stretch absorbs the difference.

For two freely moving identical endpoints, center-of-mass force is zero and length mobility is `2/gamma`; the effective scalar drag is `gamma/2`, not gamma. This is a distinct boundary condition and must be declared in evidence. A future vector finite-rotation extension would require discrete length direction `(r'+r)/(L'+L)`, whose dot product with `r'-r` equals `L'-L`, and equal/opposite endpoint forces. It is not established by the present collinear proof.

For fixed old L, convexity makes the stretch secant nondecreasing in L'; the coupling secant has derivative `K*P/(2*L0^2)>0`. The scalar residual is strictly increasing because gamma/h>0. Therefore at most one root exists; a sign-certified bracket inside `(0,4L0]` establishes existence. A root outside the physical domain, missing bracket, solver uncertainty, force/API cap mismatch, or residual outside the independently sealed tight accuracy budget causes atomic rejection. This proves neither 2D mesh uniqueness nor native explicit-Euler compatibility.

## Atomic cycle, lifecycle and legacy ports

The bounded order is cold entry, optional funded reaction preparation, then repeated **reaction transaction followed by fixed-pool mechanics transaction**. All proposed values and ledger increments are scratch values until that stage's domain, donor, finite-A, energy and accuracy checks pass. Rejected stages leave amounts, geometry, source/stores and cumulative sinks unchanged. If an enclosing operation promises cycle atomicity, a later mechanical failure rolls back the entire cycle, including the preceding reaction; it must not silently retain a paid partial cycle. The sealed reference boundary chooses cycle atomicity.

The complete accepted-cycle invariant is

`Delta(E_A+E_c+U_s) + Q_chem + Q_drag = 0`.

`Q_chem` and `Q_drag` are separate nonnegative cumulative sink ledger entries. No A refund, energy recovery or independent work tank is admitted. Round trips cannot restore A or erase heat. Repeated preparation/enable cannot reset store history. OFF is a separate explicit bypass of the future module; toggling a live energy-owning state to OFF is unsupported and atomically rejected rather than deleting its store.

Diffusion, synthesis, maturation, rupture, remesh, edge creation/deletion, fusion/fission, coefficient changes and external reservoirs are frozen and unsupported in this first bounded model. Reject them before mutation. There is no claim of a material/energy partition for topology. A future extension requires explicit ancestry-based material and energy ownership, charges for newly created stores, and exactly-once release/transfer; changing IDs cannot erase energy. A/W remain finite in a sealed finite run; W and sink overflow rejects without partial commit.

The native stretch receiver is admitted only on its exact derivative domain. Native pressure, bending, active tension, normal actuator, contact/projected corrections and external hooks are not silently disabled in production. Any nonzero such port is outside the bounded prototype and prevents a claim of full native realization. In particular A-dependent pressure and actuator funding demand a new, justified causal evaluation/reservation/commit order in a future full model. Fixed A during bounded mechanics prevents that staging issue only inside this excluded-pressure boundary. Signed candidate coupling tension also cannot be passed through the current nonnegative-tension API unchanged.

## Independent anchor inventory and calibration disposition

| Category | Available anchor or evidence | Permitted inference and limit |
|---|---|---|
| Native constitutive anchor | `mesh_mechanics.rs:137–156`: bounded native stretch force and endpoint signs; `material_mesh.rs:305–356`: mature mass/fraction and physical rest length | Integrating this fixed native force gives U_s on the stated domain. It anchors receiver ownership in native F L units; it is not an experimentally measured elastic modulus. |
| Native constitutive/kinetic input | `polarity_mass.rs:50–59,325–391`: frozen rates, amount/concentration semantics, c0 net-transfer funding | Supplies uncoupled proposal inputs and finite A/material semantics. It does not identify K, lambda, theta or epsilon_A, or gross turnover. |
| Native boundary audit | `mesh_mechanics.rs:89–92,167–191,226–255,341–369`: A-dependent pressure, bending, explicit integration and force-hook limits | Establishes why the bounded potential is not a complete native mechanics energy model. These ports need ownership before full integration. |
| Unit normalization | E=Fref Lref; Pref, Lref and fixed native force convention; four groups lambda, K Pref/(Fref Lref), K/theta, K/(epsilon_A c0) | Units make numerical comparison meaningful; they supply no observation and cannot determine a physical energy scale independently. |
| Explicit new design hypotheses | Ideal EA=epsilon_A A battery with W reference zero; Ec; signed rate bias; irreversible chemical sink; fixed-endpoint drag boundary; cold-entry preparation | These close the proposed model by assumption. They are not recovered native biology, calibrated constants, or local detailed balance. |
| External independently observed measurement method | [Toyabe et al. (2010)](https://arxiv.org/abs/1002.1867), single-molecule F1-ATPase energetics: load work and probe heat compared with chemical free energy | REFERENCE / ADAPTABLE_MEASUREMENT_METHOD only. It supports independently measuring work, dissipation and fuel free energy, not assigning ATP identities, values or efficiency to Digital Cell. |
| External independently observed reverse-transduction method | [Toyabe and Muneyuki (2015)](https://arxiv.org/abs/1210.4017), ATP-synthesis thermodynamics under imposed rotations | REFERENCE / ADAPTABLE_MEASUREMENT_METHOD only. It does not authorize regeneration here. No numerical parameter or code is imported. |
| Independent Digital Cell calibration data | **MISSING**: signed additional force versus geometry/active fraction; unclipped biased net J; independently measured fuel free-energy/work/material mapping | No native K/lambda/theta/epsilon_A value can presently be selected. Native ks and c0 do not substitute for these measurements. |

The earlier prospective four-parameter protocol remains the measurement specification, with joint uncertainty, fixed holdouts, independently sealed accuracy budgets and no R13 thirty-state outcome fitting. A force dataset independent of the candidate must identify K and lambda through `T_c L0/P=K epsilon-K lambda x`. A net-J dataset with known Rplus/Rminus and nonzero g identifies theta through the specified positive-root inversion; model rate terms are not measured gross turnover. epsilon_A requires an independent chemical free-energy/material or physical load-work plus dissipative-loss anchor, with an explicit A/species and F/L unit mapping. Computing energy from assumed epsilon_A and solving back is circular. Merely observing net A loss and declaring ideal battery conversion is a model assumption, not an independent calibration anchor.

Current disposition is **BOUNDED_MATHEMATICAL_SOURCE_RECEIVER_DEFINED; INDEPENDENT_PARAMETER_DATA_MISSING; FULL_NATIVE_REALIZATION_UNRESOLVED**. Arithmetic/source checks and independent review can verify the defined bounded architecture. They cannot promote it to calibrated efficacy or authorize implementation. A future implementation or calibration campaign requires its own bounded directive, sealed inputs/uncertainty/decision rules and independent verification; no automatic successor is authorized.
