# Independent mechanochemical observations and chemical-work owners

Disposition: **PARTIAL**. A new hypothetical finite-event reference defines and executes explicit material/free-energy owners for transport, growth, maturation and a prescribed fission cut. Independent quadrature and raw-state reconstruction verify its prepared transition pairs. Actual native operator/force probes are separate audits. Independent licensed DNA force-position observations were acquired, but their prospective six-pair coverage and force-conjugate work admission do **not** pass. No empirical Digital Cell parameter or whole-organism qualification follows.

Entry `a800279a355dc49060d81f805800f0a6def7b7a8`; [draft PR56](https://github.com/SketchOTP/digital_cell/pull/56); [Notion phase record](https://app.notion.com/p/3fd833cb27ff81d1bb04d4db9318028b). Earlier source, defaults, biology, parameters and qualified evidence remain frozen. All production K/lambda/theta/epsilon_A remain null.

## Independent physical observations

[Rissone, Rico-Pasto, Smith and Ritort, Figshare28211567 v1](https://doi.org/10.6084/m9.figshare.28211567) supplies original instrument records from DNA unzipping/rezipping. Dataset license is CC BY4.0, separately from the article license. The complete820104841-byte publisher ZIP matched MD5 `0a23bffa7ae665def8e73a5360e54080`; all ZIP CRCs were checked in hosted acquisition39144641592. Fourteen original molecular/COM/Stokes members are preserved losslessly with per-file hashes, publisher metadata and inventory. The full ZIP remains at the publisher and was only temporary on the runner. No publisher software executed.

Selection used directory metadata before waveform processing: the first lexicographic molecular file in each HighT/LowT session containing Stokes records. This selects HighT/02102019/mol1aA and LowT/071020/mol10aA; lexicographic ordering is deliberate. First three bounded recorded unzipping/rezipping status pairs per file, at least200 samples per leg, were sealed prospectively. Files/cycles were not substituted after inspecting outcomes.

The first strict parser stopped at the instrument's `#Skipped` marker before completing a summary. A retained source-format amendment handles only that marker and terminal `#End Write`; unknown malformed rows or appended data after termination still reject. Primary integration breaks at each known gap. A separately labelled linearly bridged sensitivity is not primary work. Exact source bytes and original protocol/budgets remain unchanged.

The [Mini Tweezers Basic User Guide, page2](https://timothyspringer.org/file_url/906) documents force in pN, trap-displacement channels in nm and time in seconds; COM force-offset labels corroborate pN. The manual was read through browser PDF text; direct local downloads returned403, so no local original PDF/hash is claimed. Publisher instructions define coordinate `(A_dist-Y+B_dist-Y)/2` and force `-Y_force`. That coordinate is not independently established here as force-conjugate external displacement or corrected molecular extension. In fact the recorded unzipping coordinate often decreases. Preserve its signed integral; do not silently reverse its sign to obtain expected positive work.

Observed coverage is **2 HighT +3 LowT =5 pairs**, not six. All10 selected legs have known gaps, totaling6448 gap boundaries. Selected signed sums of observed-chunk integrals, in pN nm, are HighT3494.3075/−1116.4515 and LowT−293.40135/62389.2585/2663.5137. These are diagnostic force-coordinate integrals with missing intervals excluded. Aborted/repositioning records and endpoint mismatches remain visible, including a3794.57nm mismatch; they are not full closed thermodynamic cycles or calorimetric dissipation.

Independent integration by parts agrees within1.17e−10pN nm; a separately written raw TSV/decimal reader reconstructs selection, rows, gap counts and leg integrals within1.82e−10pN nm. Numerical agreement does not qualify conjugacy, missing work or chemical free energy. Retained instrument thermistors and Stokes/COM data do not establish local fluid temperature without bead/viscosity/calibration assumptions. No temperature/rate/NN thermodynamic fit was performed. Metadata3653bp versus article3593bp remains unresolved; no sequence-length inference is made. The closer AdK/AP5A study remains documentary/SI comparison with no newly verified raw traces. The2026 MarkovianF1 repository is software/model prior art, not admitted empirical Digital Cell calibration.

## New finite-event owner contract

This is an explicitly new computational design, not identification of native A/R or a measured biochemical mechanism. Six source amount variables `(E,ES,ESS,Pr,S,Sm)` retain the earlier ideal-mixture convention; source n=(.1,.05,.05,.01,1,10) is a newly declared monomer-rich preparation, not the published initial state. The original four-channel chemistry and these new event pairs are not composed into a continuous kinetic generator in this phase.

One finite material bath holds S10,Sm10 and inert10. This replaces the earlier inert-only preparation; the old Nout is not copied or added again. Root area is immutable, cell volumes are actual area/root_area, and outside volume is10 minus the sum over cells. A separate fixed-theta thermostat is the thermal boundary. Finite material closure is not first-law energy closure or a heat measurement.

| Operator | Fixed material proposal | Work and inheritance owner |
|---|---|---|
| Transport | S or Sm moves between inside and its depleted finite outside pool | Both ideal-mixture stores; downhill/outward and uphill/inward directions have reciprocal finite-pair rates. No deterministic uphill pump is inferred. |
| Growth | Sm ⇄ .9Y+.1S on a named edge | Source amount, solid standard free energy and every changed stretch/coupling store. Extent is fixed before evaluating energy. |
| Maturation | Y ⇄ M at fixed total edge material | Different solid standard potentials plus actual regularized elastic and coupling changes. No universal positive per-mass mechanical cost. |
| Fission | Prescribed hexagon cut0→3; two wholly young closures each of mass rho*cut_length; same .9-yield growth stoichiometry pays total closure material | All daughter sources, solid storage, coupling graph and geometry; one global bath/root volume scale; exact old edges/tags and new closure tags. |

Bound chemistry is a solid-phase standard-state model `Gsolid=muY0*Y+muM0*M`, with muY0=−.25 and muM0=−.5. No per-edge mixing entropy or artificial reference capacity is added. Substrate equivalents use weights `(0,1,2,2,1,1)` plus all bound Y+M and outside S+Sm; enzyme equivalents use `(1,1,1,0,0,0)`. These totals, a0 and Rref are conserved by each reference event. Parameters theta1,ks1,rho1,K.25,lambda.1,yield.9,kappa1 are declared design inputs, never empirical calibration outputs.

### Regularized native stretch storage

The old mature-only spring formula does not apply to wholly young newborn closure. For edge mature amount h and young amount y, use the **actual native** rest floor `r=max(h/max(rho,1e−15),1e−15)`, `R=max(r,.001)` and tension factor phi=h/(h+y), or zero if total material is zero. Compression is unscaled. Integrate the native force from r to L:

For `L<=4R`, `U=chi*ks*(L−r)^2/(2R)`, where chi=1 in compression and phi in tension. For `L>4R`,

`U=phi*ks*((4R−r)^2/(2R)+4*(L−4R)−4*r*ln(L/(4R)))`.

The native8ks force clamp is inactive in this nonnegative-material domain because tension is at most4ks and compression magnitude at mostks. Pressure/bending are disabled in the force audit. The primitive is continuous through h→0, unlike extending the old mature-only formula into the floor regime. Parent quadrature integrates the force independently;24 actual native force-difference probes match within4.45e−16.

Absolute regularization is not extensive under arbitrary remeshing. At L=.004,h=.0005,y=.0005, midpoint splitting changes native elastic storage .0030625→.00153125, a loss .00153125. This counterexample remains an obstruction to declaring all-domain neutral remeshing. A separately loaded neutral split passes only in the sealed inactive-floor domain; no continuum mesh-convergence or newborn remesh neutrality is claimed.

### Coupling and fission inheritance

The new reference deliberately uses whole-perimeter coupling

`Ec=K*P/2*[((sum L−sum r)/Rref)−lambda*(ES+ESS−a0)/P]^2`.

This is **not** the accepted single-ancestral-arc body port. A daughter inherits n6,P,a0,Rref by actual area fraction after fixed closure chemistry; no birth anchor resets, fresh V=1, duplicated bath or hidden zero-energy new terminal is introduced. Old material edges and tags are partitioned exactly. New closure tags correspond to the explicit monomer debit. The complete graph/storage change belongs to the fission reaction proposal. The model does not yet own membrane synthesis, catalysts, templates, reserve architecture or a full autonomous daughter preparation.

Each tested operator is an independent prepared pair from the same root state. They are **not one composed stock-funded lifecycle**. The mathematical hexagon split is externally prescribed, not dynamically generated neck formation. The native fission audit uses a distinct externally compressed dumbbell geometry and unchanged native predicates. Its actual accepted partition is evidence of that native proposal only.

### Rates and atomic transactions

For each fixed paired transition, `DeltaG=Gafter−Gbefore` includes all inside/outside ideal, solid, regularized elastic and new coupling storage. The new bounded Glauber convention uses the same kappa in both directions:

`kxy=kappa/(1+exp(DeltaG/theta))`, `kyx=kappa/(1+exp(−DeltaG/theta))`.

Actual positive computed rates and `log(kxy)−log(kyx)=−DeltaG/theta` are checked, including underflow rejection. A normalized equal-occupation px=py=.5 witness has nonnegative pair entropy production; this is not an arbitrary-population/global-generator or realized-path heat claim. Uphill transitions remain possible under the declared thermostat. Chemical-only payment is reported separately and never forced by retrospectively selecting a fuel debit or repairing a residual.

Typed immutable proposals bind the supplied owner/config/state, fixed action and recomputed target. Rejected donor, type, ancestry, stale or foreign-owner proposals leave state unchanged. Exact inverse receipts recover the physical preimage while advancing revision; they are not a native fusion mechanism. Public dataclasses permit explicitly prepared input states, so hashes alone do not authenticate arbitrary historical provenance. Independent qualification reconstructs every before-state from sealed preparation/controls and every after-state's exact actions/tags/inheritance, rather than accepting a supplied plausible registry.

## Measured bounded results

The new reference's separate transport/growth/maturation/fission pairs have DeltaG .443445182177/−.503304870468/−.009229658120/−9.956923869249. The selected inward transport is uphill and has negative chemical release; it is not labelled a fuel-paid pump. Maturation releases .01 chemical free energy while storing .000769230769 native elastic and .000001111111 coupling energy. The prescribed fission consumes4/.9 source Sm, creates4 young structural mass and stores .011111111111 new coupling energy; its prepared chemical release is9.96803498036. All quantities are normalized design units.

Positive/negative/constructed-zero chemical-drive controls preserve rate reciprocity without tuning physical data. Loaded neutral remesh and the explicit floor counterexample stay separate. All prepared source/material/anchor/inverse controls pass independent raw-state/quadrature reconstruction; five malformed/donor/stale fixtures preserve state. Tests include channel underflow, lost/duplicated ancestry, exact preparation and source comment termination.

Hosted39145917043 at82a738e passes14 new owner tests,8 observation tests and24 accepted-port Rust regression tests, then builds the actual native audit. It observes native growth .0468653696961 and matured material .000218240997343 in a full reaction call, plus an accepted native partition with .8 wholly young closure mass. The maturation call includes other native channels and is not isolated kinetics. Native exterior concentrations remain unchanged during transport. These probes confirm the integration gaps; they do not implement the new reference laws in native biology.

Parent separately reconstructs every raw event, material tag, chemical/mechanical store and kinetic ratio, and independently reads the raw DNA TSV with decimal arithmetic. Six additional oracle tests check analytic spring limits and malformed preparations. Final exact-head workflow/review/source-artifact/preservation/Notion/native Atlas readbacks follow as external receipts after the final commit.

## Remaining acceptance gates and custody

Empirical force-conjugate work, continuous complete traces, local fluid temperature/systematic calibration, molecular free energy and a physical mapping to native/source species remain unqualified. The native finite bath, growth/maturation reciprocal rates, full coupling graph/daughter preparation, membrane/template/chemical inheritance and continuous multi-event orchestration still need integration and evidence. Finite pair closure is not persistent individuality, fission-driven reproduction, selection or life.

Preserve15 previous live checkouts,PR44–55,2538 scoped Atlas files and original dirty source including32 untracked-file hashes. Local evidence `/home/sketch/Projects/digital_cell-mechanochemical-ownership-evidence/`; additive Atlas group `evidence/mechanochemical-work-ownership/` only after mounted UUID/FSROOT verification. Independent coder/reviewer roles remain distinct; architect bridge unavailable, complete local handoff retained without fabricated dispatch. No merge, deployment, physical apparatus/author outreach, acquired source execution or automatic successor.
