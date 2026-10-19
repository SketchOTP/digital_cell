# Composed native chemical and structural integration

This phase implements a new opt-in **prescribed hybrid reference**, extending the accepted baseline `552e3795c09961ba56ceef4f8da2503fbb63e0f9`. It composes a full chemical source, native geometry and newly owned structural operators in one evolving state. The first sealed campaign passes independent numerical reconstruction; final distinct file-bound review and exact-head packaging are pending below. Physical calibration and whole-organism qualification remain unestablished; production K/lambda/theta/epsilon_A remain null.

## One state and one energy owner

Each compartment carries six chemical amounts `(E,ES,ESS,Pr,S,Sm)`, an actual native `MaterialMesh`, structural origin tags and inherited P/a0/R_ref. One global birth area fixes every compartment volume `V=area/root_area`; one outside S/Sm/inert bath has `Vout=10−sum(V)`. Daughters inherit these owners without reseeding or creating a second bath. No generic native interior chemistry is populated or identified with this reference source.

The complete potential is inside/outside ideal-mixture free energy plus solid standard chemical potentials, exact regularized native elastic storage and the explicitly new whole-perimeter coupling

`Ec=K*P/2*[((sum L−sum r)/R_ref)−lambda*(ES+ESS−a0)/P]^2`.

The earlier accepted single-ancestral-arc port and inert-only outside energy are absent from this new module. Native `compute_forces` and `mechanics_step_with_external_forces` own spring forces and vertex integration. Their pressure/bending parameters are zero; the new layer supplies the complete perimeter and osmotic forces. All old biology, defaults, Cargo manifests and source modules remain frozen except an exact feature-gated library export.

The four chemical stoichiometries retain the earlier reference. Continuous amount currents are `V*j(n/V)`, with symmetric rate tilt from `g=−K*lambda*extension` and reaction bound-enzyme change `(0,1,0,−1)`. For geometry the added osmotic force is `theta*(sum(n)/V−sum(nout)/Vout)*grad(area)/root_area`. The continuous identity is `dG/dt=−Dchem−Ddrag`; joint old-state Euler defects are measured without repair.

The [Rao/Esposito2016 framework](https://journals.aps.org/prx/abstract/10.1103/PhysRevX.6.041064) and [Avanzini/Falasco/Esposito2020 source](https://arxiv.org/abs/2008.10495) are theory/reference inputs. The reciprocal mechanical tilt, solid chemistry, coupling constants and preparations are declared computational designs, not observed biological mechanisms or empirical calibration.

## Prescribed structural chronology and boundaries

Each positive-event case starts with a regular radius10 native hexagon, rho.1, mature mass rho*birth edge length and young mass.1 per edge. The prepared passive membrane is `.7*birth edge length`; it is conservatively split/inherited. New closures have zero bound membrane. Membrane synthesis, self-sealing, templates and reserves therefore remain unqualified. The additive pre-outcome preparation clarification binds all native metadata and exact birth origin names without changing the protocol's cases or budgets.

The externally specified schedule advances continuous chemistry and native geometry to each opportunity, then executes:

| Time | Newly owned operator |
|---|---|
| .05 | Move.2 Sm from finite outside to cell0. |
| .10 | Consume.2 Sm to produce.18 young structural material and.02 S on edge0. |
| .15 | Convert.04 young to mature material on edge0, including full changed elastic/coupling storage. |
| .20 | Call actual native midpoint remeshing6→12, restricted to inactive reference-floor geometry and checked energy/area/force pullback. |
| .30 | Prescribe the evolved diameter cut0→6; debit `2*rho*actual_cut_length/.9` Sm, create two wholly young structural closures, partition source/anchors by actual area and retain exact old material origins. |
| .40 | Mature.01 closure material in daughter1, then daughter2 at the same global time. |
| .40–.60 | Continue both daughters' source and actual native geometry without reset. |

Transport/growth/maturation/fission here are **new owned mutations of native mesh/source state**. They do not call or energetically rehabilitate the unchanged legacy `mesh_transport`, `mesh_growth`, `mesh_reactions` or `mesh_fission` biology. Actual reused kernels are native force calculation, vertex integration and midpoint splitting. The prescribed loop cut is not dynamically generated neck formation.

Finite event DeltaG includes every changed chemical, solid, elastic and coupling store. Bounded Glauber forward/reverse endpoint rates are diagnostics only: event times/branches are prescribed, inverse fission is not implemented, and negative growth does not restore an identical history-tag preimage. Neither sampled hazards, a complete reversible generator nor autonomous reproduction follows. Uphill scheduled events remain visible. Their energy changes are never erased or called chemical-only fuel funding, calorimetric heat or continuous dissipation.

The global ledger is

`Gfinal−Ginitial=−integrated_Dchem−integrated_Ddrag+continuous_signed_defect+sum_event_DeltaG`.

Daughters are additive ideal compartments sharing their initial cut; there is no intercell contact/exclusion owner. Native drag remains per vertex, so6→12 remeshing changes the discretized mobility and is not continuum convergence. The absolute reference-floor counterexample from the prior phase remains valid; wholly young newborn closure cannot be neutrally remeshed under this contract. Restart verifies trusted configuration, permitted state shape, material invariants and ledger consistency, not authenticity of arbitrary coherent histories.

## Prospective tests and independent reconstruction

Five sealed cases are coupled, no events, zero source rates, zero coupling and reversed maturation standard potential. Both dt.001 and.0005 run to.6. Zero source and zero coupling retain real osmotic/native/material responses and are not static controls. Reversed solid potential changes pair preference; prescribed branch choice remains external.

The independent Python oracle does not import the Rust model. It reconstructs every full before/after source/mesh/tag state, exact schedule, native primitive by separate quadrature, all forces/currents, Euler amount/geometry updates, finite event stoichiometry and material mappings, shared-bath/anchor inheritance and complete global ledger. Additional central differences probe total-potential gradients at birth, post-fission and final states including both daughters. Complete source/material/geometry/outside endpoints and unrepaired continuous defects are compared under time refinement. Rejection controls bind raw before/after states and owner serializations for evolved donor shortage, newborn remesh, corrupted restart and late geometry failure.

## Observed results

[Hosted39243436386](https://github.com/SketchOTP/digital_cell/actions/runs/39243436386) at870d9517da40a913366caf135d29c9fe717da8b0 passes36 Rust tests (24 prior regressions and12 new composed tests), five initial oracle controls, all9000 global native steps,56 prescribed finite/numerical events including8 actual native6→12 remesh operations and8 paid prescribed cuts. Each positive-event daughter continues300 coarse or600 fine global steps after fission. Eight raw rejection/restart controls pass, including evolved source/bath donor infeasibility and newborn floor rejection. Six additional coherent raw-campaign oracle controls pass locally on the retained actual raw campaign, for11 total oracle tests; the final exact-head workflow reruns all11. The strengthened checker independently reconstructs the standalone newborn-remesh and fast-gamma negative seed histories, binds their full configurations and serialized cell inventories, and validates event mappings.

Coupled continuous cumulative absolute/signed defects are .00557212122830 atdt.001 and.00278258980822 atdt.0005; both pass the original .02/.01 budgets and refinement ratio. Complete source/geometry/structural-material/outside endpoint difference is.000206939666244. Peak native elastic storage is.778902654156. Fine integrated chemical/drag dissipation is3.78741317009/.135904254040; finite event DeltaG sum is−7.22984145655. Independent full raw source/force/material/tag/jump/ledger reconstruction passes; maximum coupled global ledger error is4.45e−15 and material drift1.43e−14. Total-potential finite-difference probe error is at most1.18e−8 across cases. Endpoint difference is a coarse/fine observation, not a separate RK4 solution or rigorous global integration-error bound.

The no-event control's endpoint difference is.000827950606532 and continuous defect.00644153943238→.00321739355331. Zero-source chemistry has exactly zero continuous chemical dissipation while preserving prescribed structural events and real osmotic/native motion; its defect.00000948103834→.00000474013150. Zero coupling also remains mechanically active. All five cases satisfy unchanged prospective budgets.

**Decision-changing limitation:** atfine resolution the parent maturation event has DeltaG−.003786108609: it releases.01 chemical free energy and stores.006213891391 mechanical energy. In contrast, prescribed newborn-closure maturation in daughters1/2 has DeltaG+.384087888779/+.384050895287. Each releases only.0025 chemical free energy while adding approximately.38659/.38655 mechanical storage. These are explicit uphill scheduled events under the declared thermostat, not demonstrated chemically paid autonomous maturation. Reversing the solid standard potential reverses parent maturation preference without changing the externally prescribed branch. Endogenous event selection and a complete physical work/reservoir interpretation remain necessary before autonomous-life claims.

First artifact12182276935 is30670436bytes, SHA2569fc0603d2c60b4011728c2eb417b5896bb3eac7313510b0bd8cab1b4cd0612c3; complete ZIP member/CRC/evidence hashes pass. Parent runs only the downloaded bounded two-step --probe, obtains byte-identical output and independently reconstructs both raw native/source steps. Binary SHA25659b0df8b8d5a7a764472710cf6d686d27c432a97116d28ac56efe6b8a602e9ed; probe SHA2568805963e7125753d21a2d0f90bffd23a66bd67efeefd95b6ab8cb3ceedbcbcdf. The full9000-step numerical checker ran on the hosted runner; no local full campaign rerun is claimed. Final file-bound review, exact-head artifact and custody receipts follow the final commit.

## Preservation and custody

[Draft PR57](https://github.com/SketchOTP/digital_cell/pull/57); [Notion phase](https://app.notion.com/p/3fe833cb27ff81af99bbf13f188beefa). Local evidence is `/home/sketch/Projects/digital_cell-native-composed-evidence/`; additive mounted Atlas group is `evidence/native-composed-integration/` after volume-identity verification. Preserve16 earlier checkouts, PR44–56,2641 prior scoped Atlas files and original dirty source including32 untracked hashes. Distinct implementation/reviewer roles are retained; parent owns every execution. Bridge tools unavailable; complete local handoff is retained without claimed dispatch. No merge/deploy, physical apparatus, empirical parameter assignment or automatic successor.
