# Independent source admission and acquired evidence

**Result: PARTIAL / BLOCKED.** Independent primary documentary and computational
evidence was acquired and inspected. None of the eight inspected candidate
families passes admission for the four Digital Cell calibration parameters.
There are **zero admitted calibration observations**, and K, lambda, theta and
epsilon_A remain null. This is not a claim that no admissible source exists
elsewhere or that the constitutive candidate is globally impossible.

Entry is `2b2638047a79fcbf01b279e7d3a796873f0017c5`. The owner's new request
authorizes source admission and acquisition beyond the prior plan-only stage.
Earlier source, prototype, plan, empty template and scientific records are
byte-preserved. No production/native simulation or external source program ran.

## What was actually acquired

[Albaugh and Gingrich's independent code/data archive](https://zenodo.org/records/4481182)
is version v1, dated 2022-02-02, with a CC-BY-4.0 declaration in its repository
metadata. We acquired a sparse prefix, directory and tail from its original ZIP,
then reconstructed 26 selected members: all 18 source files, two directory readmes,
four processed figure tables and two plot scripts. The four headerless tables have
44 rows and 23 finite numeric columns each (1,012 cells). These are published
processed **computational summaries**, not raw replicate trajectories or physical
laboratory measurements. Original selected bytes are retained without modification.

The complete initial member list was sealed before inspecting contents. The
README then directed inspection of the two complete plot scripts for column
meaning, with a separately sealed contextual selection. No favorable row or
response subset was selected, transformed into target data or fitted. Acquiring a
historical publication cannot retroactively preregister its original experiment
or satisfy Digital Cell's prospective instrument/design requirements.

The 551,498,479-byte ZIP advertises MD5
`75c151155bd3187cb1e56e07539887f4`; **the whole ZIP was not downloaded and this MD5
was not verified**. The package preserves exact HTTP ranges, central-directory
entries, member CRC and local SHA-256 bindings. These establish the selected
transport/content checks, not publisher cryptographic attestation or complete
archive reconstruction. The offline verifier reconstructs members from the stored
range and directory without importing or executing any acquired C++/Python code.

Three primary article HTML documents were also retained in the external evidence
root with URL/version/byte/hash receipts. Full article text is not redistributed in
Git. The licensed sparse archive evidence, receipts and table inventory are under
`docs/dc-m4-calibration-source-evidence/`; original extracted files and documentary
HTML remain in local and additive Atlas evidence custody.

## Admission by inspected source family

| Source family | Actual support | Admission for the new parameters |
|---|---|---|
| Native material/stretch/polarity/actuator source | Existing constitutive law, amount semantics, native rate/c0 and effort price | Reference only; ks is the receiver coefficient, not new K; c0 is not source free energy |
| Historical D003/D006/D016 calibrations | Internal simulated targets and normalization/rate choices | Not independent identifying observations |
| R13 outcomes and internal response artifacts | Existing model behavior and damping attribution | Explicitly forbidden outcome-fitting input |
| New bounded reference prototype | Illustrative prescribed-stroke source/store/solver controls | Candidate-generated; fitting it back is circular |
| Older surface/phase-field thermodynamics | Legitimate different-domain potentials and species | Reference only; symbol reuse and kT=1 do not supply the missing map |
| Toyabe 2010 primary experiment | Rotary ATP work/probe-heat/chemical-energy methodology | No established Digital Cell species/coordinate/unit map or admitted raw trace package |
| Toyabe/Muneyuki 2015 primary experiment | Rotary reverse-transduction methodology | Same mapping and target-data gap; no regeneration or parameter import |
| Albaugh/Gingrich independent particle motor | Acquired source and processed data independent of our equations | Missing target mappings, measurements, uncertainty/design seals; not admitted |

The machine registry records target-specific missing gates and evidence IDs.
`NOT_ADMITTED_INCOMPATIBLE` for native/other-domain sources means incompatible
with the proposed **identifying-evidence role**, not a rejection of their original
scientific laws. External sources are not rejected as bad science; their current
use as target calibration evidence is unsupported.

## Concrete native and external mapping findings

Native `material_mesh.rs:168–189` defines generic lumped c,a,n,f,w. Native
`mesh_reactions.rs:490–525` produces A and W through N+F→A+W material accounting;
it supplies no measured chemical potentials. R3 `polarity_mass.rs:20–26,50–59`
defines amount/concentration/rate and A-per-polarity units, while synthesis
`172–224` transfers A into polarity material. None identifies A as ATP or specifies
its chemical free energy. `contractility.rs:24–28,297–355` prices an effort-duration
quantity, not independently measured signed force-displacement work.

Native stretch/maturity/rest-length law supplies the already accepted receiver.
Using ks as new K would conflate two separately owned sectors. D006's analytic
surface-profile target and reference scale, D003 simulated balance and D016
transport target are internal calibration domains. Older `d077_analysis.rs`
surface/precursor chemical potentials and `phase_field.rs` double-well coefficient
have different meanings from the target fuel A and polarity P=a+b. No inference
from a shared symbol or a chosen kT=1 is admissible.

The [independent motor model](https://arxiv.org/html/2102.06298v2) uses a shuttling
ring, FTC⇌ETC+C fuel and open chemical-potential reservoirs. Its particle and
coarse-state coordinates have no demonstrated map to a fixed edge's a,b,L,L0.
The acquired tables sweep chemical potential or friction and report motor current
and reaction-related summaries. They do not supply additional signed coupling
tension on the 80-point design, unclipped local target J, or a finite A/W source
energy ledger. Model independence alone cannot identify target coefficients.

[Toyabe 2010](https://arxiv.org/html/1002.1867v1) and
[Toyabe/Muneyuki 2015](https://arxiv.org/html/1210.4017v4) are independently measured
rotary ATP systems. No source establishes the required model-A/species/strain/force
mapping. A separately downloadable raw response dataset was not confirmed in the
inspected surfaces; some publisher endpoints were inaccessible. That is an
access/inspection limitation, not proof of raw-data absence. Published means,
figures or efficiencies are not an independent target calibration package.

Identity discrepancy retained: arXiv 1002.1867 lists journal reference PRL 104,
198103 (2010), but its Related DOI points to `10.1016/j.bpj.2009.12.905`.
The [actual PRL publisher article](https://harvest.aps.org/v2/journals/articles/10.1103/PhysRevLett.104.198103/fulltext)
uses `10.1103/PhysRevLett.104.198103`. Do not silently substitute the related DOI.
The 2015 journal article is `10.1088/1367-2630/17/1/015008`.

## Verification and the unresolved objective

The strict offline metadata evaluator validates declared IDs, target gates, proof
references and schemas. It **never admits a source or calibrates a value**. Even
all-complete synthetic declarations return only ELIGIBLE_FOR_REVIEW, requiring
separate hash-bound semantic review of actual source/raw/mapping/uncertainty proof.
Metadata assertions do not authenticate themselves. The package verifier checks
the actual evidence hashes, sparse ZIP reconstruction and all 44 table rows,
frozen prior sources/records and the independent review receipt. Synthetic gate
controls are labelled as schema tests, never observations.

The owner explicitly confirmed during this admission attempt that **no specific
independent physical or computational reference is established**. This is a
missing calibration referent, not merely a missing download. It does not authorize
assuming one or choosing numerical parameters.

The next scientific input is a named independent physical or computational
referent with a justified map into the abstract digital variables, followed by
target-specific observations, independently justified uncertainty/budgets and the
sealed design/holdouts. A new digital-physics design choice would be an explicit
model assumption, not empirical calibration. No such choice is silently made.
The bounded source search and quarantine acquisition are reviewable outputs;
**admissible calibration-source acquisition remains blocked** and must stay active
in project records. No fitting, production integration, R13 correction or automatic
successor follows this partial result.
