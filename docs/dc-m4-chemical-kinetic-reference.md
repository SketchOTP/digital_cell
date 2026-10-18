# Independent chemical/kinetic reference: build and measurement

The owner explicitly selected a computational reference first. This package implements a named independent closed chemical model and a separately declared reciprocal mechanical extension. It does not identify physical ATP, change production equations or assign any production parameter. The sealed computational measurement protocol has executed successfully; the results below remain subject to distinct review and exact-head publication verification.

## Independent evidence and reuse decision

[Avanzini, Falasco and Esposito2020](https://arxiv.org/pdf/2008.10495), DOI10.1088/1367-2630/abafea, provides the full elementary closed network in equations5–7 and numerical fixture in Figure1. We independently implement those equations; no external executable is imported. The six species are E,ES,ESS,Pr,S,Sm, where Pr names the publication's product and avoids collision with target polarity total P. Reactions are S↔Sm, Sm+E↔ES, Sm+ES↔ESS, ESS↔E+Pr. All eight directional constants are1 in the published units; initial concentrations are(.1,.05,.05,.01,1,.1). Time unit is1/k+1, concentration unit k+1/k+2. The full elementary system is reproduced; its quasi-steady approximation is not substituted.

[Rao and Esposito2016](https://arxiv.org/html/1602.07257v2), DOI10.1103/PhysRevX.6.041064, supports the ideal-mixture affinity/free-energy framework and rate-ratio constraint. Its enthalpy/entropy distinction prevents treating free-energy loss as measured heat. Both original documents are hash-receipted in external custody; full text is not redistributed in Git. The arXiv PDF does not supply assumed CC-BY redistribution permission.

Other researched candidates are narrower: ATP thermodynamic formation data depend on specified pH, magnesium, ionic strength and temperature; those conditions do not identify target A/W. Toyabe/F1 switching remains a different chemical coordinate. The MIT-licensed [TACOS Schlögl code](https://github.com/Shiling42/TACOS) offers an open-CRN kinetic comparator but its reservoirs do not supply the finite closed source needed here. Disposition: REFERENCE for those candidates; BUILD the small published closed elementary equations, reusing the prior native stretch and reciprocal-spring mathematics without editing accepted artifacts. No acquired source program runs.

## Positive source and variable/unit mappings

In normalized fixed volume V=1 with concentration reference c*=1 and thermal free-energy scale theta=1, choose a standard-potential gauge compatible with all unit forward/reverse ratios. Then

    Gchem = theta * sum_i z_i*(log(z_i)-1).
    mu_i = theta*log(z_i).
    affinity_r = -sum_i S_ir*mu_i = theta*log(jplus_r/jminus_r).

The source is the finite full six-species prepared state, not a constant-energy A tank. Conserved enzyme LE=E+ES+ESS=.2 and substrate LS=S+Sm+ES+2ESS+2Pr=1.27 follow from stoichiometry. Define operational remaining substrate equivalents A_src=S+Sm+ES+2ESS and product equivalents W_src=2Pr. Then A_src+W_src=LS exactly, and their net conversion is dW_src/dt=2*j4. They are material equivalents, not equal-mass arbitrary molecules or fixed energy packets. The initial chemical preparation is finite and nonzero.

Free-energy availability is Gchem−Gchem_eq for the isolated chemical reference at fixed conserved moieties, not absolute Gchem and not epsilon_A*A_src. The equilibrium oracle solves its strictly increasing positive-root polynomial and reconstructs all six concentrations. Two states with the same A_src,W_src can have different speciation, affinities and currents: a coarse two-variable battery is not a closed source model. The reference therefore retains all six species.

Units remain those of the published computational model. Physical concentration c*, volume V, temperature T and the source's standard-potential conventions are independently required for SI conversion; none is measured here. With dimensionless z=c/c*, the total-energy scale is RT*V*c*; converting bound-enzyme amount to concentration or molecule packets changes the corresponding conjugate scale. For energy scale Eunit=RT*V*c* and a target amount chart a_target=Aunit*a_reference, theta_target=Eunit/Aunit. This gives RT for a molar-amount chart, RT*V for a concentration chart, and kBT/delta for a molecular packet chart a_target=delta*n, under the declared matching scales. A numerical theta=1 is a reference normalization, not a measured target theta or a universal biological constant.

## Kinetic mapping and its limits

Each elementary directional current and affinity is independently specified by the published model and thermodynamic rate-ratio constraint. The operational reference polarity chart is a=ES+ESS, b=E, P=LE. It preserves a+b=P, but its net kinetics is da/dt=j2−j4. It is not the native cubic autocatalytic law; no physical enzyme identity for native a/b is inferred.

Native k0*b+kf*a²*b/L² and kd*a+kq*a³/L² has a Schlögl-like channel structure. Finite-copy stochastic propensities require falling factorials and individual channel owners; an aggregate net J alone cannot establish source affinity. For an explicitly defined polarity packet delta and quadratic coupling, paired finite transitions a→a+delta compare forward at a with reverse at a+delta. Symmetric endpoint gradient bias reproduces the coupling energy difference exactly when theta_target=kBT/delta in compatible amount/energy units. This is a conditional thermal map, not an empirical target parameter: the bare rate ratio must independently represent chemistry/mixing and fuel affinity. Other load-sharing splits require separate evidence. No microscopic packet size or SI energy scale is assigned to production.

## New reciprocal chemical/mechanical extension

The extension uses the operational bound-enzyme chart above, and fixes a0 to the prepared initial bound occupancy for the entire run. It does not reset the anchor as chemistry proceeds. Define

    Ec = K*P/2 * [(L-L0)/L0 - lambda*(a-a0)/P]^2.
    Us = ks*(L-L0)^2/(2*L0) * (phi if L>L0 else 1).
    g = dEc/da; nu_a = (0,+1,0,-1).
    jplus_r = bare_jplus_r * exp(-nu_a_r*g/(2*theta)).
    jminus_r = bare_jminus_r * exp(+nu_a_r*g/(2*theta)).
    Ldot = -(dEc/dL+dUs/dL)/gamma.

K,lambda,ks,L0,gamma,phi in coupled controls are declared design values, not values fitted from the publication or measurements. The published baseline has K=0 and the exact source fixture. At initial L=L0, both elastic stores are zero, while chemical preparation remains owned. Initial a-dot is negative, so positive lambda is expected to induce contraction and compressive receiver loading rather than extension. The chemical compartment volume remains fixed as the mechanical length changes; this deliberately separates V from L and does not reproduce native edge concentration dilution. This is a new anchored reference variant, preserving the old cold-entry architecture and all production equations bytewise.

With positive concentrations, one common theta, fixed volume/material/P/a0 and consistent rates, the full differential identity is

    d(Gchem+Ec+Us)/dt = -Dchem-Ddrag.
    Dchem = theta*sum_r (jplus_r-jminus_r)*log(jplus_r/jminus_r) >= 0.
    Ddrag = gamma*Ldot^2 >= 0.

This is a free-energy/dissipation statement. Chemical heat or a first-law calorimetric balance requires enthalpic/internal-energy and entropy owners not supplied by this model. Neither -DeltaG nor a residual is relabelled as chemical heat. Reciprocal gradient flow loads the receiver and dissipates through its specified drag; no external length command or chemical top-up supplies work.

## Sealed measurement design

The protocol fixes six cases before execution: exact published unloaded chemistry, coupled cold reference, zero-lambda, no-chemistry, common time-rescaling and isolated equilibrium control. Each has two predeclared step sizes. Record all six concentrations, length, separate elastic stores, chemical/total free energy, channel currents and affinities, material invariants, instantaneous thermodynamic identity, dissipation quadratures and signed integration defects. Step refinement, an independent analytic initial derivative/equilibrium oracle and a separately implemented numerical solver distinguish dynamics from self-consistency checks.

RK4 stage positivity and geometry guards reject invalid trials with bounded retries. No clipping, material projection, energy/heat repair or posthoc parameter fitting is allowed. Raw simulated reference trajectories remain new computational observations, not independent target calibration data. The source specification is independent of the program; the newly designed coupling is not independently observed biology. Frozen budgets in the protocol determine pass/failure; no horizon convergence is inferred without measuring distance to equilibrium. Local seal/hash lineage is not external preregistration attestation.

## Scope and completion boundary

A computational source/receiver prototype and its simulated response can be built and measured here. The physical Hookean fixture remains unbuilt and no empirical calibration campaign occurs. The old ideal constant-epsilon_A battery and native cubic kinetics do not silently become this six-species reversible reference. Full target physical/calibration/native integration remains unqualified where that mapping differs. All production K,lambda,theta,epsilon_A stay null. R13 remains uncorrected; R4 production reference and R10 diagnostic-only.

The actual measured result, independent review, exact-head artifact, preservation and Notion/custody receipts are recorded alongside this report. The new source registry admits the published computational specification only; it does not override the historical negative admission or populate the earlier 80-row target force template. No automatic successor or production change follows.

## Observed computational measurements

The presealed six cases and both step sizes pass their unchanged acceptance budgets. At normalized time60 with the finer step .01, the coupled reference contracts from length1 to 0.976380491163 (minimum 0.97536537148), with receiver peak energy 0.000303432461171 and final receiver energy 0.000278940598857. The isolated chemical reference starts with available free energy 0.617303553424 above its independently derived equilibrium; source preparation, not an elastic preload, owns this availability. All energies are normalized reference units. These values are simulated observations, not measured biological constants.

The coupled run records chemical dissipation 0.615746990412 and mechanical drag dissipation 0.000402829206675. Its largest unrepaired total free-energy/dissipation integration defect is 1.75354922137e-08 and maximum material drift 3.33066907388e-15. Independent midpoint endpoint discrepancies are 7.54224460664e-12 (unloaded) and 8.92552698417e-12 (coupled); refinement changes the coupled endpoint by 3.39894778989e-13. These are observed numerical checks against sealed budgets, not rigorous interval bounds on all continuum error.

Zero-lambda, zero-chemistry and equilibrium controls produce zero receiver loading. The common rate/drag time-rescaling control matches the coupled endpoint exactly in this execution. The unloaded finite-time state remains 0.000611953283449 from its independently derived equilibrium, so no completed relaxation at time60 is asserted; the coupled state is not compared to that oracle as its own equilibrium. The first harness attempt failed on a budget-field name mismatch before publishing results; the field lookup was corrected, preserving the protocol hash and every numerical budget. No posthoc budget or design tuning occurred.
