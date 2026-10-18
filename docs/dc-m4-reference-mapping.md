# Calibration reference and variable/unit mapping disposition

**PARTIAL / FULL CALIBRATION BLOCKED.** A reference hierarchy and exact conditional unit/work maps are established. No independent physical or computational system has yet been shown to realize the complete accepted coupling/source/kinetic architecture. All production K, lambda, theta and epsilon_A remain null; zero target observations are admitted.

## Selected references and why

1. **Toyabe et al. 2010: experimental work/heat/chemical-affinity comparator.** F1-ATPase supplies independently observed rotary mechanical work and a chemically driven cycle with separately accountable dissipation. This justifies a measurement methodology reference and a rotary work-port chart. It does not establish ATP=model A, catalytic state=a/P or a physical edge lever arm. Actual PRL DOI is [10.1103/PhysRevLett.104.198103](https://doi.org/10.1103/PhysRevLett.104.198103); the discrepant arXiv related-DOI field is retained in prior evidence. [Primary manuscript](https://arxiv.org/html/1002.1867v1). Toyabe/Muneyuki2015 remains a reverse-transduction methodological companion, not a finite-A calibration dataset: [primary manuscript](https://arxiv.org/html/1210.4017v4).
2. **Proposed two-port Hookean fixture: operational mechanical reference specification.** It is selected because its two independently controlled displacement ports can expose the target's reciprocal force and conversion work. This is a designed realization specification: NOT_BUILT, NOT_MEASURED, no apparatus or data asserted, no externally certified spring constant/gearing supplied. It is not independent evidence merely because its equation matches ours. [NIST force-transducer metrology](https://www.nist.gov/programs-projects/calibration-force-transducers) supports independent applied-force/deformation calibration as a future method; no NIST calibration or uncertainty is claimed for this proposed fixture.
3. **Albaugh/Gingrich2022: contrasting independent computational motor.** Retain the acquired original archive/model as evidence about a driven many-body motor, not as the calibration identity of Digital Cell. [Primary paper](https://www.nature.com/articles/s41467-022-29393-3), [archive](https://zenodo.org/records/4481182). The existing source-admission registry remains historically unchanged; these new restricted reference roles do not admit its processed summaries as target calibration data.

Theoretical F1 comparison: [Kawaguchi/Sasa/Sagawa2014](https://arxiv.org/html/1307.3998v4) models angular motion with chemical state switching and local detailed balance. Chemical state index is discrete; its effective dwell potential combines wells rather than establishing the single global target spring. It supports separating mechanical and chemical ports, not a continuous occupancy identity. No parameter is imported. Its reversible local-detailed-balance chemistry does not establish the candidate’s irreversible path-uphill-only finite-A funding law.

## Exact native unit frame: convention, not calibration

Target base dimensions are length L, time T, polarity amount P, fuel amount A and energy E. Polarity and fuel need separate amount scales. Target epsilon=(L-L0)/L0; x=a/P; b=P-a. Fixed P,L0 only. Coupling store Ec=K*P/2*(epsilon-lambda*x)^2 and finite ideal battery EA=epsilon_A*A.

| Target quantity | Dimensions |
|---|---|
| L,L0 | L |
| a,b,P | P |
| A,W | A |
| K,theta,g=dEc/da | E/P |
| epsilon_A | E/A |
| lambda,epsilon,x | 1 |
| ks | E/L (force, not E/L²) |
| gamma | E*T/L² |
| c0 | A/P |
| k0,kd | 1/T |
| kf,kq in kf*a²*b/L²,kq*a³/L² | L²/(T*P²) |

At a fixed reference edge with positive native ks, gamma and c0, choose positive Lref and Pref, then Fref=ks, Eref=ks*Lref, Aref=c0*Pref, tref=gamma*Lref/ks. These are native normalization conventions for the bounded fixed-endpoint scalar receiver; they do not assign SI magnitudes or identify the new sector. The independent groups remain lambda, K*Pref/(ks*Lref), K/theta and K/(epsilon_A*c0). Unit choices cannot determine them. No numerical reference edge, amount or SI scale is selected here.

## Complete similarity dictionary

Let reference/source quantities equal target quantities multiplied by positive scales ell (length), tau (time), p (polarity), f (fuel), e (energy). This is a conditional change of units **within the same constitutive model**, not proof that an external system obeys it.

| Quantity | Reference/source divided by target |
|---|---|
| L,L0 | ell |
| t,step duration h | tau |
| a,b,P | p |
| A,W | f |
| energies,heat,work | e |
| mechanical force,ks | e/ell |
| K,theta,g | e/p |
| epsilon_A | e/f |
| lambda,strain,x | 1 |
| gamma | e*tau/ell² |
| c0 | f/p |
| diffusion | ell²/tau |
| k0,kd | 1/tau |
| kf,kq | ell²/(tau*p²) |

If scaffold amount has independent scale m, linear scaffold density scales m/ell. Transport concentration conventions and stoichiometry must also be supplied before any full native map. These identities preserve A+W for the isolated conversion/work sector (not synthesis material transfer), a+b=P, F*dL, g*da, epsilon_A*dA, force balance, the signed entire-stroke variation funding and the dimensionless groups. Rates R± scale p/tau for the written native reaction monomials; their chemical tilt is invariant because g/theta is invariant. Every dimensional cutoff/reference must transform as well, including native length and scaffold-density floors and activity thresholds. Keeping their numerical values unchanged is not a change of units. The helper checks algebra, not all native guard/transport/remesh semantics. No new kinetic trajectory is validated.

## Rotary mechanical chart

For an unwrapped angular branch in radians and positive, independently specified radius r, define q=r*angle, force=torque/r and gamma_linear=gamma_rot/r². Then torque*dangle=force*dq, including signs and reverse motion. Degree-valued sources first multiply angle by pi/180 and transform conjugate torque reciprocally. A radius is a coordinate chart parameter until actual geometry independently fixes it; choosing r to force agreement is not identification. An additive angle-coordinate origin does not change displacement work. The torque zero must be independently calibrated: an additive torque offset changes work by offset times angular displacement. Repeated rotary winding is not a global invertible map to the bounded positive edge interval. No r, angular branch, ATP identity or catalytic occupancy is assigned to target edges.

## Proposed two-port mechanical realization contract

At one fixed P>0,L0>0 let a differential Hookean spring have displacements u=sL*(L-L0), v=sa*a, and storage Uref=kref*(u-v)^2/2. For positive kref,sL,sa (positive target lambda), its reference energy equals e*Ec for all L,a, where e>0 is the independently specified reference-energy/target-energy scale, exactly iff

    kref*sL² = e*K*P/L0²
    sa/sL = lambda*L0/P

The Jacobian gives target conservative efforts: dEc/dL=sL*kref*(u-v)/e, g=dEc/da=-sa*kref*(u-v)/e. Thus dUref=e*[(dEc/dL)*dL+g*da] and the mechanical force on L is minus dEc/dL. Both reference spring ports carry work: external effort at v is not battery work or ATP by declaration. A separately observed source must fund it and account for all losses. This specifies reciprocity at fixed material and does not solve finite fuel/kinetic mapping.

**No fixture setting is assigned.** If independently measured kref,sL,sa, independently established energy scale e and fixed measured P,L0 later exist, the conditional mechanical inverse is K=kref*sL²*L0²/(e*P) and lambda=sa*P/(sL*L0). It identifies only mechanical K/lambda after physical amount/geometry definitions, uncertainty and held-out target observations are established. Tuning a fixture from assumed K/lambda is merely a realization control.

A single constant fixture cannot implement the multi-P calibration design. The target stiffness is proportional to P and its conversion lever is proportional to 1/P. Either restrict calibration to one fixed P,L0 and explicitly leave cross-P tests unresolved, or independently characterize a fixture family with those relations. Recomputing fixture settings from unknown target coefficients supplies no independent evidence. No cross-P fit or 80-row force-template completion is claimed.

## Why superficially plausible maps fail

**Independent binary harmonic elements.** For N elements with spring stiffness k, common extension q and binary rest shift d, occupied fraction x gives

    Umean = N*k/2*(q-d*x)^2 + N*k*d²/2*x*(1-x).

This is exact mean energy of independent states, not the free energy of a log-summed equilibrium distribution. Match the target force with q=L-L0, N*k=K*P/L0², d=lambda*L0, x=a/P. The second term has zero q derivative yet changes conversion affinity by K*lambda²*(1-2*x)/2. Force matching alone hides this chemical cost. At x=1/2 the derivative vanishes only at a point; this does not establish equality over a nonzero interval. Symmetric endpoints can cancel the variance-energy difference while path-resolved costs still differ. Equilibrium mixing entropy, where applicable, adds another term; it cannot be silently omitted. A coherent collective spring could realize the target energy but is a new constitutive realization, not established by an independent two-state motor.

**Finite chemical source.** In an ideal, isolated fixed-volume mixture for ATP→ADP+Pi at fixed T with positive initial molecule counts nT,nD,nP, reaction extent xi gives affinity

    D(xi)=D0+kBT*log[(nT-xi)*nD*nP/(nT*(nD+xi)*(nP+xi))].
    D'(xi)=-kBT*[1/(nT-xi)+1/(nD+xi)+1/(nP+xi)] < 0.

Hence no affine amount/unit map identifies a constant epsilon_A battery over a positive depletion interval. This is conditional ideal-mixture mathematics, not a measured fit to the acquired motor; count versus molar conventions require kB versus R and activities/standard states. It does not prove all chemical batteries impossible. If 0<=xi<=X<nT, B=kBT*[1/(nT-X)+1/nD+1/nP] bounds |D'|. Affinity error <=B*X and integrated constant-affinity energy error <=B*xi²/2. A local approximation could be admitted only with independently established composition, T, activity corrections, positive-affinity domain, uncertainty and a presealed total error budget. These are not available. A chemostat may keep affinity constant but introduces externally owned material/energy ports excluded from the bounded finite-A model.

**Acquired Albaugh source.** Source forcefield.cpp uses nondimensional m0,e0,d0 (e0/d0 are interaction reference scales, not SI metrology); source system.h/system.cpp expose three open chemical reservoirs and insertion/deletion. The eight-state reduction retains ring position and two separate blocker states; total blocked count loses site/ring correlations and need not close a Markov rate law. Modified reservoir potentials omit standard-state offsets needed for full reaction affinity. FTC→ETC+C can have an isolated reaction-extent/material-equivalent dictionary only under separately established closed chemistry; it is not the acquired open NESS or a constant-energy battery proof. Current/cycle summaries do not recover the joint target U(L,a), g or full fuel-path cost. Prior acquired code remains unexecuted.

## What is established, and what remains

- Established: named limited experimental comparator; proposed fixed-material mechanical reference contract; complete conditional unit/work dictionary; inverse mechanical relations; exact counterexample showing force agreement is insufficient; conditional finite-source approximation bound.
- Not established: an independently realized complete Digital Cell reference, physical A/W/polarity identity, SI scales, independent mechanical fixture measurements, finite-source affinity or kinetic theta map, target observations, independent uncertainty/holdout/design admission, parameter values or native integration.
- The executable checks test algebra and failure controls; separate human scientific review assesses source meaning. Neither package integrity nor symbolic agreement admits calibration data.
- Next decision-changing work would be independently characterized mechanics/source data for the selected fixed-material fixture contract, or an explicit owner-approved constitutive strategy revision. No apparatus acquisition, model revision, fabricated data or automatic successor is performed here. The reaction-architecture calibration gap narrows but remains open.

## Verification and preservation

See the distinct hash-bound review and parent-owned standalone verification report. Previous scientific subtree, all prior contracts/prototypes/source-admission artifacts and five append-only ledger prefixes are frozen. All 11 previous checkouts, PR44–51, 1347 entry Atlas evidence files and dirty original source are preserved. This report defines reference roles without rewriting historical negative admission. R13 remains uncorrected, R4 production reference and R10 diagnostic-only. Exact-head hosted artifact, Notion and custody readbacks follow externally after commit creation.
