# Local reaction architecture feasibility result

**Completed bounded negative:** `LOCAL_REACTION_COUPLING_PHYSICAL_OWNERSHIP_OR_ENERGETICS_GAP`.

Existing local mechanical observables and finite A/W funding do not establish a physically owned mechanical affinity for active↔inactive polarity conversion in the accepted R3/R4 MaterialMesh path. The missing relation is constitutive/energetic; this audit does not prove that a new persistent state is necessary or that future mechanosensing is impossible. No feedback law is implemented or qualified.

## Entry and method

This fulfills `DC-M4-REACTION-ARCHITECTURE-FEASIBILITY-GATE-001` from exact R13 final head `74dfbf9a12fc9cdafcc72c3080150a7b9c04be90`. R13's accepted thirty-state reaction-mediated damping is preserved, still uncorrected; R4 remains production reference and R10 diagnostic-only. See [the adopted directive](dc-m4-reaction-architecture-directive.md).

The architecture analyst and independent reviewer separately read the relevant source. The parent independently inspected the call graph, algebra, other-domain counterexamples and preservation evidence. Source ranges below refer to the frozen entry head and are cryptographically bound by `dc-m4-reaction-architecture-audit.json`. This is source-architecture evidence, not another biological campaign. The automated checker verifies source identity, exact symbolic algebra and evidence structure; semantic absence/ownership claims also require the independent source review.

## Existing causal graph and units

`PolarityMassStateV1` persists active/inactive edge **amounts** and an accepted-step counter (`polarity_mass.rs:103–109`). Its native `advance` passes `None` for R9 strain (`273–279`). The accepted R4 replay supplies current edge lengths, frozen parameters and finite cell A (`dcfinal001_r4_evolution.rs:5617–5641`). Polarity advances before activity is derived and paid mechanics is invoked (`5642–5667`). There is no tension, mechanical potential or conversion-work argument at this boundary.

Each of four native substeps transports amounts before evaluating local concentration-dependent reaction. Rate has concentration/time units; multiplication by edge length and substep time gives an amount transfer. Clipped net transfer conserves local active+inactive polarity; `0.5 × |net transfer|` is charged to A with equal W credit. Scratch-state computation, funding/positivity checks and atomic commit are preserved (`polarity_mass.rs:325–407`). The source documents `conversion_a_per_amount` as A amount per polarity amount (`20–26`), not a signed energy or chemical potential.

The R4 activity adapter converts amounts/current length to a clipped dimensionless signal; it neither moves coordinates nor owns spending (`polarity_actuation.rs:1–7,94–108`). Paid contractility owns active efforts and finite cell A→W expenditure. Its accepted tension charge scales as tension × current edge length × time (`contractility.rs:603–629,652–660`). That is an adopted effort-duration funding law, not signed force × displacement work per polarity conversion. The distinction does not invalidate the existing funding contract.

## Reaction geometry susceptibility

For fixed local active amount `a`, inactive amount `b`, edge length `L>0`, and the frozen rates, the unclipped amount-rate is

`J = L[(k0+kf(a/L)²)(b/L) - (kd+kq(a/L)²)(a/L)]`

`  = k0 b - kd a + (kf a² b - kq a³)/L²`.

Therefore `partial_L J |_(a,b) = -2 a²(kf b-kq a)/L³`. Basal amount-rate terms have no explicit length dependence at fixed amounts. The checker expands the original concentration expression as an exact sparse Laurent polynomial and differentiates it symbolically, independently of the written factorization.

The amount-transfer derivative is `h × partial_L J` only for an unclipped substep. At pool bounds the derivative is piecewise and can be non-smooth. Native transport and subsequent reaction substeps evolve `a,b`; this identity neither replaces R13's measured full-update attribution nor identifies a corrective feedback sign.

An exact illustrative algebraic state, not an additional R13 snapshot, uses `a=L=1`, `b=11/101`, and frozen `k0=1/100,kf=1,kd=1/10,kq=1/100`. Forward and backward rates both equal `11/100`; net rate and native net-conversion charge are zero despite positive kinetic terms. This demonstrates why the adopted net-transfer ledger does not identify gross microscopic turnover, fuel stoichiometry or a signed affinity. A formal log of the forward/backward kinetic terms remains a computable rate bias; it is not an established mechanical chemical potential merely because another model uses a logarithmic affinity.

## Candidate ownership audit

All paths are under `digital-protocell`. A real observable can be reused for observation without being licensed as conversion work.

| Existing candidate | Source owner and units | Finding for polarity conversion |
| --- | --- | --- |
| Edge length, mature mass, rest length and strain | `material_mesh.rs:232–254,314–356`; persistent vertices/structural masses; derived length/rest length and dimensionless strain | At fixed geometry and structural material, converting active↔inactive polarity changes neither rest length nor passive stretch-law inputs. No conversion-associated extension or material transfer is declared. |
| Load-bearing strain and passive stretch effort | `material_mesh.rs:326–329`; `mesh_mechanics.rs:137–165`; mature-fraction-weighted strain and piecewise/clamped force law | Local mechanics is measurable. No displacement per conversion or mechanical work derivative with respect to conversion extent is defined. R9's positive-strain forward multiplier cannot be recycled. A naive global Hooke energy would also ignore the actual clamps/maturation branches. |
| Curvature and bending | `mesh_mechanics.rs:167–208`; adjacent-segment geometry and hinge force proxy | No active/inactive-specific spontaneous curvature or conversion-associated bending work. Adding one would be a constitutive change. |
| Osmotic-like pressure | `mesh_mechanics.rs:87–95,160–165`; lumped inside/outside chemical contrast converted to force | A spending can alter later pressure, but no polarity-conversion volume or pressure-work stoichiometry exists. Shared fuel does not identify a signed mechanical affinity. |
| R4 active tension / R10 diagnostic normal request | `polarity_actuation.rs:94–108`, `contractility.rs:603–629`; `dcfinal001_r5_v4_neck.rs:556–595` | Real, funded force generation from polarity is present. It does not uniquely fix a reverse conversion potential. Even an integrable force determines energy only up to a polarity-dependent additive function; no such state potential is adopted. R10 remains diagnostic-only. |
| Substrate/contact work and observer force projections | `stick_slip_traction.rs:400–430`, `mesh_self_contact.rs:1–12`; `dcfinal001_r4_evolution.rs:6675–6700,7069–7076` | Dissipation/contact geometry and discarded force·displacement observations are not a persistent recoverable reservoir or conversion-work donor. Observer projections must not become causal reaction inputs. |
| Regulatory activity/adaptation and plasticity | `regulatory-core/src/lib.rs:427–455`, `plasticity.rs:1–55`; local dimensionless signals/history | Persistence alone does not confer energy or material ownership. Reinterpreting these as an affinity would change their contract. |
| Strain-sensitive structural build/turnover and membrane occupancy | `mesh_reactions.rs:305–308,341–342,674–747` | Genuine existing mechanochemical/material pathways concern structural M and membrane L/binding, not polarity active↔inactive conversion. Their coefficients and stoichiometry cannot be silently reassigned. |

## Existing thermodynamics in other domains

The repository does contain thermodynamic models; the gap is specific to the active MaterialMesh polarity path.

| Source | Existing model | Why it cannot supply this coupling unchanged |
| --- | --- | --- |
| `phase_field.rs:7–27` | Cartesian phi free energy, Cahn–Hilliard chemical potential | Different grid field and state variable, with no adopted mapping to the two edge-local polarity pools. |
| `surface_density.rs:709–751` | Precursor↔surface exchange activities, log-ratio affinity and dissipation expression | Different reaction/material ownership; neither a polarity-conversion mechanical term nor a common mechanical energy/fuel unit mapping. |
| `d077_analysis.rs:197–231` | Reduced cooperative adsorption theta/P free energy and chemical potentials in kT=1 units | Analysis-model occupancy states and parameters are not the active/inactive MaterialMesh pools. |
| `membrane.rs:139–163` | Optional grid-interface membrane-affinity transport | Different transported material/interface representation, with no conversion-conjugate polarity work term. |

These are methodological precedents, not latent authorization or a reusable drop-in affinity. None is called through the native R3 polarity interface above.

## Missing relation and bounded consequence

No audited candidate closes all three required links: **(1)** a named conversion-associated mechanical work/free-energy contribution, **(2)** common units and finite fuel/material ownership for that contribution, and **(3)** a signed rate relation connecting it to active↔inactive conversion. Physical observables and existing operational A/W debits close useful parts of the model but not this chain.

The single bounded consequence is to return a **conversion-work ownership contract requirement**, before selecting a corrective formula. A future separately authorized proposal must identify the local physical conversion extent and either a supported cross-energy relation with its mechanical counterpart or an explicitly driven reaction cycle with identifiable fuel and work terms. It must resolve double-counting with existing conversion/actuator A/W costs, locality, pool conservation, positivity, atomic rejection, persistence/checkpoint/remesh/fission ownership and exact route-OFF parity. Existing state should be reused where justified; a new state is not presumed necessary.

An equilibrium cross-energy generally requires a consistent mechanical counterpart; introducing that would exceed frozen mechanics unless an existing counterpart is demonstrated. An explicitly driven mechanism need not obey equilibrium reciprocity, but its physical/energetic cycle must still be specified. Onsager symmetry or local detailed balance from another regime is not imposed wholesale on the existing far-from-equilibrium model. No supported coupling law, parameter, new persistent state or implementation is selected by this result.

## External discovery and evidence status

Kruse et al., [Generic theory of active polar gels](https://arxiv.org/pdf/physics/0406058), sections II–III: **REFERENCE_ONLY / ADAPTABLE_METHOD** for identifying owned fields, conjugate fluxes/forces and fuel work. Their near-equilibrium continuum expansion is not imported into this discrete driven system. Seifert, [Stochastic thermodynamics of single enzymes and molecular motors](https://arxiv.org/abs/1012.3119): **REFERENCE_ONLY / ADAPTABLE_PRINCIPLE** for distinguishing state energetics, transition work and rate consistency. Its microscopic/state-separation assumptions and thermodynamic parameters are not adopted. No code, coefficients, biological species, flow or thresholds are imported. Existing R13 biological references remain reference-only.

- **VERIFIED:** frozen source interfaces/accounting and exact reaction algebra; production source preservation; distinct source-review agreement; existing other-domain thermodynamic models.
- **SUPPORTED:** this bounded active-path audit finds no existing-owner mechanical conversion-affinity contract.
- **INFERRED:** effort funding and a kinetic log-ratio do not by themselves identify conversion thermodynamics.
- **UNKNOWN:** a lawful corrective mechanism, whether additional persistent state is needed, feedback correction and downstream biological outcomes.
- **DISPROVEN IN PRIOR SCOPE:** R9's single tensile-strain forward-activation multiplier repairs damping; that prior negative remains preserved.

## Validation and boundaries

`scripts/verify_reaction_architecture_feasibility.py` verifies frozen source bindings, unchanged scientific subtree, exact symbolic identities, the net-vs-gross ledger counterexample, source-audit completeness and append-only preservation. The branch-specific audit workflow runs these checks and archives the exact-head receipt/manifest. It does not replay R13 or claim a new scientific executable/binary. Semantic audit acceptance additionally depends on the distinct read-only review recorded in the audit JSON. Final CI/artifact and Notion receipts are retained under `/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/reaction-architecture-feasibility/` after the documentation commit exists.

Historical governance-validator schema drift remains recorded; new structured entries are checked separately. No callable architect bridge status/review tool is present in the parent enabled tool inventory; no bridge dispatch or acceptance is claimed. Existing checkouts and PR #44/#45 are preserved. This feasibility gate is complete; the coupling is **NOT ESTABLISHED**. Production biology and all downstream campaigns remain unchanged/unrun, and no successor starts automatically.
