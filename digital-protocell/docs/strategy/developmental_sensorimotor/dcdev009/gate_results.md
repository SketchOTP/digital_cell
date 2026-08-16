# DC-DEV-009 gate results

## Gates 0-4

| Gate | Result | Evidence |
|---|---|---|
| 0 authority and scope | PASS | exact entry, observer-only boundary, no DC-DEV-010 |
| 1 force accounting | PASS | contractile sum max norm `6.804363002006077e-16`; motor-off `0.0` |
| 2 fixed-topology translation | NO_VALID_TRANSLATION | contractility-only centroid displacement `2.473548217003853e-18` |
| 3 artifact audit | PASS | active-minus-control drift equals baseline force-field integral within `1e-15` |
| 4 environment coupling inventory | PASS | no friction, adhesion, substrate, or fluid coupling in the free-space arm |

The active shape-change L2 is `0.49448972028754373`; motor-off is
`0.06468419178539876`. The active-minus-control shape difference is
`0.44683399200843127`. These are deformation results only.

## Gate 5 prior art

The external review searched the underlying physical problem rather than
Digital Cell labels.

| Candidate | Disposition | Reason |
|---|---|---|
| nonreciprocal deformation | REFERENCE / COMPOSE | useful symmetry principle, but deformation alone needs a coupling that reacts differently to the cycle |
| frictional crawling and anisotropic drag | ADOPT candidate | smallest physical symmetry-breaking substrate coupling; local and compatible with a mesh body |
| adhesion or anchoring | COMPOSE candidate | can break translation symmetry locally, but requires a bounded contact/traction law and energy accounting |
| peristaltic locomotion | DEFER / COMPOSE | depends on substrate friction or anchoring, so it is not the next isolated dependency |
| fluid-coupled swimming | DEFER | requires a new environmental momentum-transfer model and substantially larger validation scope |
| cilia or flagella | REJECT for this directive | new actuator and explicitly excluded |

Sources:

- Purcell-style low-Reynolds symmetry and nonreciprocal motion:
  [Swimming by reciprocal motion at low Reynolds number](https://pmc.ncbi.nlm.nih.gov/articles/PMC4241991/)
- Friction control and anchoring in peristaltic locomotion:
  [Mechanics of peristaltic locomotion and role of anchoring](https://pmc.ncbi.nlm.nih.gov/articles/PMC3243396/)
- Anisotropic friction in soft crawling:
  [Frictional Anisotropic Locomotion and Adaptive Neural Control for a Soft Crawling Robot](https://pubmed.ncbi.nlm.nih.gov/36459126/)
- A frictional one-degree-of-freedom counterexample:
  [Crawling scallop: Friction-based locomotion with one degree of freedom](https://arxiv.org/abs/1303.2669)

These sources support architecture selection only. They are not Digital Cell
evidence and no external implementation was copied.

## Gates 6-7 recommendation

Three viable options remain:

1. A local substrate law with anisotropic drag or anchoring, reusing the mesh,
   local contractility, and existing overdamped integration. This is the
   recommended smallest next experiment.
2. A bounded local adhesion/anchor contact law, reusing the existing spatial
   contact boundary and adding only a substrate traction coupling after a
   separate energy contract is approved.
3. A fluid-coupled nonreciprocal swimmer environment, which is physically
   legitimate but materially larger and should be deferred.

The smallest next experiment is a read-only, fixed-topology two-arm test of one
bounded local substrate traction coefficient, with a motor-off control and the
same force/centroid ledger. It is recommended only. It is not implemented by
DC-DEV-009.
