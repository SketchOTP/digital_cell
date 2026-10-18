# Independent calibration-evidence acquisition plan

**PLAN ONLY. No observations acquired, no parameters fitted.** Entry
`fb3366995de9a8dbc5bceb6d3ae24ed653972dca`; scope is the accepted v2 source/receiver
architecture and the standalone bounded reference prototype. Production `K`,
`lambda`, `theta` and `epsilon_A` remain null. Prototype constants and generated
trajectories are illustrative controls, never independent measurements.

## Acquisition readiness and independence

The current result is `MISSING_EVIDENCE`. The independent source registry and raw
dataset registry are empty. No apparatus, species identity, force/length/amount
mapping or independent computational constitutive source is presently admitted.
The 80-row CSV is an **empty acquisition template**: its rational coordinates and
train/holdout roles are planned inputs, all response/provenance cells are blank,
and every row is NOT_ACQUIRED. It contains zero observed force responses.

Before a separately authorized campaign, an evidence custodian must identify an
independent source and preregister a signed, hashed acquisition manifest. For a
physical source, record apparatus identity, raw force/position/time/material
channels, calibration certificates, environmental conditions, species identity,
reaction stoichiometry, chemical reference state and mapping to model A/W/P and
native F/L/time units. Biological experiments do not acquire that mapping merely
by using ATP. For an independent computational source, record its version,
license, equations, input dependencies and raw outputs; audit that the additional
coupling force, net rate and fuel free energy do not derive from this candidate,
its prototype, assumed target parameters or fitted R13 responses. Document the
intended model-to-model mapping and its uncertainty. Such data support only that
declared constitutive comparison, not a physical biological measurement claim.

The owner/custodian selects and attests source eligibility; an implementation
author cannot self-attest synthetic outputs as independent data. An independent
reviewer must verify raw-file hashes, source version/dependency audit, unit and
species mappings, train/holdout custody and uncertainty provenance. Missing
eligibility, source or mapping blocks acquisition/identification; it is not a
reason to assume ATP identity or copy a convenient literature constant.

## Staged, sealed procedure

0. **Seal before responses.** Bind the accepted architecture and prior protocol
   hashes, source versions, fixture coordinates, raw channel definitions,
   instrument sampling/replicate policy, estimator implementation, data file IDs,
   train/holdout custody, arithmetic method, independently justified absolute
   accuracy budgets and maximum uncertainty widths. Budgets derive from source
   capability and the resolution of the declared physical decision, not observed
   residuals or prototype tolerances. Numerical measurement budgets are currently
   null; this plan cannot certify data until they are sealed. Freeze excluded
   inputs: all thirty R13 signs, growing/fission outcomes, the desired correction
   target and candidate-generated synthetic outputs. Preserve every response,
   including failed/ineligible probes. No row dropping, post-response probe
   movement, favorable pivot selection, budget widening or holdout refitting.

1. **Force evidence for K and lambda.** Retain the prior 80-point lattice:
   x={0,1/4,1/2,3/4,1}; epsilon={-1/4,0,1/4,1/2}; relative
   (P/Pref,L0/Lref)={(1,1),(1,2),(2,1),(2,2)}. Train only the four rows with
   x in {0,1}, epsilon in {0,1/4} and both scales one. The other 76 are held out.
   Observe signed additional tension after matched, independently measured
   native-force subtraction, with raw tension and a,b,L,L0/P intervals. Use
   Y=T_c*L0/P=K*epsilon-(K*lambda)*x. Certify rank two and intersect **all four**
   training strips to obtain the positive bounded correlated region in (K,Klambda).
   Seal that region before opening holdouts. Carry correlations into lambda and
   every later prediction. Require predictions over the entire sealed region to
   meet all 76 held-out accuracy and width budgets. Partial region compatibility
   is INCONCLUSIVE; do not narrow the region using holdouts. Native passive ks
   anchors its own receiver, not an active-fraction coupling law. If no independent
   additional force exists, K/lambda remain null.

2. **Net kinetic evidence for theta.** Only after independently sealing the force
   region, set lambda_star to the midpoint of its certified lambda projection.
   At x={1/4,1/2,3/4}, use epsilon=lambda_star*x+sigma*d_epsilon, sigma={-1,+1}.
   Train on d_epsilon=1/4 (six design points); hold out d_epsilon={1/8,3/8}
   (12 points). Propagate the full force region into g; each entire enclosure must
   have its declared nonzero sign and lie inside the admitted source domain.
   Ambiguous sign or unsupported geometry returns INCONCLUSIVE/DOMAIN_UNCOVERED
   without changing the fixture after responses. Observe unclipped net J with
   independently measured a,b,L and sufficient A, using sealed h,h/2,h/4 windows
   and source-certified truncation/refinement uncertainty. Require Rplus,Rminus>0,
   independently verified lack of donor/fuel clipping and unchanged native rate
   inputs. Invert the specified net-rate law using a stable interval equivalent
   of y=(-J+sqrt(J^2+4 Rplus Rminus))/(2 Rminus), theta=g/(2 log(y)). Opposed signs
   must identify one bounded positive theta region and predict every holdout.
   A zero-g control is admitted only if g=0 is independently certified: the
   midpoint selector alone does not make true g zero. Net J is not gross turnover
   or local detailed balance. The present prototype does not execute this law.

3. **Independent chemical/work evidence for epsilon_A.** Establish A/W species,
   stoichiometric amount, source free energy/reference and unit mappings first.
   Prefer independent source free-energy loss per source A spent, with uncertainty
   and all other material obligations separated. An alternative named physical
   transducer must measure total work crossing its boundary, all relevant heat
   and independently measured stored-energy changes, plus A/W material. Infer
   epsilon_A=(W_out+Q_loss+DeltaU_independent)/A_source, with consistent signs and
   positive independently measured A_source. Do not infer epsilon_A from assumed
   candidate Ec, its funding debit or an efficiency assumption.

   This is a **v2 clarification of the prior work-only recipe**, not a rewrite of
   that historical document: load output divided by A is valid only with a
   separate certificate that omitted heat/storage terms vanish. V2 dissipates
   downhill path variation and conversion overhead, so endpoint work alone can
   undercount actual battery loss. Separate overhead, synthesis, other native
   obligations, coupling work and dissipation without double counting. When
   overhead is subtracted from A, the corresponding independently justified
   energetic term must also be removed from the numerator; otherwise use total
   source debit and total transducer energy. W amount is not calorimetry.

   Preserve the planned positive work scales in ratio 1:2 for training and 3/2
   held out, with absolute magnitude/domain fixed from the admitted source before
   responses. Include zero-motion, zero-work, negative-release/no-A-recovery and
   synthesis-not-W controls. Require A/W material closure and compatible positive
   estimates across all trials. No reverse-source/regeneration experiment or
   policy is authorized here. Missing independent source energy or unobserved loss
   leaves epsilon_A null; a loss term computed from this prototype is circular.

4. **Review and decision.** A separate reviewer checks bindings, source
   independence, all uncertainty/correlation propagation, tightness, exclusions
   and every held-out response. A residual interval merely containing zero proves
   compatibility, not acceptance: its full enclosure must lie inside the sealed
   absolute budget and meet the separate width limit; inequalities must hold over
   the whole joint parameter/data enclosure. Use outward-rounded interval
   arithmetic or a justified counted-operation forward bound for floating data.
   Exact reference fractions do not certify instrument uncertainty. Any budget
   missing/wide, ill-conditioned inversion or boundary overlap is INCONCLUSIVE.
   Missing evidence stays MISSING_EVIDENCE; inconsistent hashes/mappings are
   EVIDENCE_DEFECT; valid contradictory observations reject the constitutive
   candidate. An admissible identified region permits a later implementation
   proposal, never automatic native integration or an efficacy claim.

## Deliverables for a later authorized campaign

Required package: source-admission manifest and independence audit; immutable raw
data with checksums and acquisition timestamps; raw-to-model species/unit map;
instrument/arithmetic uncertainty certificates; pre-response roles/budgets;
all-response registry; sealed correlated training estimates; withheld-data custody
and release receipt; full held-out predictions/residual enclosures; reviewer
decision and rejected/inconclusive rows. Each stage reseals inherited inputs
before responses. Acquisition, fitting and any dataset import are **not performed
or authorized by this plan-only milestone**.

## Prior art and reuse

Reuse the project's accepted v2 equations, fixed prospective design and Python
standard-library exact rational arithmetic; no external implementation or new
dependency is imported. [Toyabe et al. 2010](https://arxiv.org/html/1002.1867v1)
is a primary observed precedent for separating load work, probe heat and chemical
free energy. [Toyabe and Muneyuki 2015](https://arxiv.org/abs/1210.4017) supplies a
reverse-transduction measurement precedent. Both are REFERENCE methods only;
their efficiency, ATP identity, measured values and apparatus are not Digital
Cell observations or selected parameters. No technical novelty or biological
correction is claimed. Full native pressure/bending/contact/material/lifecycle
integration remains outside this acquisition plan and bounded prototype.
