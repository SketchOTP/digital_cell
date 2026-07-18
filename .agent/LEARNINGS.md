# LEARNINGS.md

Append-only repo-specific lessons. Format:

```md
- YYYY-MM-DD | area:<module> | lesson:<specific repo fact under 25 words> | evidence:<path>
```

- 2026-07-12 | area:protocell | lesson:Phase 1 code in digital-protocell/; tuned decay rates from directive defaults | evidence:digital-protocell/crates/chemistry-core/src/config.rs
- 2026-07-13 | area:d005-d006 | lesson:Crowding bulk production lacks restoring R*; interface assembly I(φ)=16φ²(1-φ)² yields prescribed dR/dt crossing | evidence:docs/d005_final_closure.md,docs/d006_radius_flow_report.md
- 2026-07-14 | area:d006-stage-d | lesson:Prescribed-field restoring crossings for surface_turnover_v1 did not appear in 180 coupled Stage D runs; all median v_R>0 | evidence:docs/d006_stage_d_completion.md

- 2026-07-14 | area:d007 | lesson:surface_turnover_v1 factors 0.50–0.80 show reverse radius response (small decline, large grow); structural nullcline absent | evidence:docs/d007_structural_bracket.md
- 2026-07-14 | area:tooling | lesson:Serena is configured but reports Active languages [] for Rust; use Cargo, rg, targeted reads, and compiler diagnostics | evidence:.serena/project.yml
- 2026-07-14 | area:d008-stage0 | lesson:Versioned restores validate equation, schema, and field lengths before copying; legacy five-field hashes remain frozen | evidence:digital-protocell/crates/chemistry-core/src/snapshot.rs
- 2026-07-14 | area:d008-transport | lesson:Approved beta 4.6/1.2/0.2 yields C/A 0.0101, N/F 0.3012, W 0.8187 planar permeability | evidence:docs/d008_membrane_transport.md
- 2026-07-14 | area:d008-membrane | lesson:Calibrated k_membrane 0.1974823 keeps post-transient localization above 0.900 across initial M levels 0.25–0.75 | evidence:docs/d008_membrane_localization.md
- 2026-07-14 | area:d008-hash | lesson:Stage A Transport omits membrane hash fields; Stage B appends them when d008_stage_b_enabled; Stage C appends rates only under ActivatedMetabolism | evidence:candidate_identity.rs
- 2026-07-14 | area:d008-stage-c | lesson:Boundedness pass requires |C/A clamp correction| ≤ CUMULATIVE_RESIDUAL_TOL, not merely post-clamp range membership | evidence:activated_metabolism.rs,experiment-runner/d008.rs
- 2026-07-14 | area:d008-stage-c | lesson:Stage C qualitative rates k_act=0.02 k_rep=0.04 k_adecay=0.005 k_cturn=0.002 pass nine zero-D controls | evidence:docs/d008_activated_metabolism.md
- 2026-07-15 | area:d011 | lesson:D-011 may vary only k_structure,k_rep,k_membrane,k_activation; turnover constants stay frozen | evidence:digital-protocell/crates/chemistry-core/src/d011_analysis.rs
- 2026-07-15 | area:stoichiometry | lesson:v1 runtime catalyst production is A→C+W; membrane synthesis is ∅→M; decay/detach delete M | evidence:digital-protocell/crates/chemistry-core/src/stoichiometry.rs
- 2026-07-15 | area:d013 | lesson:Stage E windows must advance only on accepted substeps; JSON f64 snapshots need lossless bits for resume | evidence:digital-protocell/crates/experiment-runner/src/d013.rs
- 2026-07-15 | area:d014-numerics | lesson:R22 TIMESTEP_FLOOR was waste CONC_SAFETY_LIMIT validation; machine-eps project + unbound map, not chemistry | evidence:docs/d014_timestep_floor_postmortem.md
- 2026-07-15 | area:d015-waste | lesson:Peripheral reservoir idle was real; W-only sink at r=30 clears exterior but interior still hits CONC_SAFETY_LIMIT | evidence:docs/d015_waste_accumulation_postmortem.md
- 2026-07-15 | area:d016-transport | lesson:After W-sink r=30 repair, dominant resistance is internal; D_W_required≈1.06≫max(D_N,D_F)=0.18 | evidence:docs/d016_timescale_analysis.md
- 2026-07-15 | area:d016-transport | lesson:Baseline D_W=0.25 already faster than N/F=0.18; fixed-source still ceilings at t≈438 | evidence:experiments/generated/d016/fixed_source_baseline/result.json
- 2026-07-16 | area:d017 | lesson:η=1 checkpoint: structure turnover ~89% of W; activation only ~5%; perfect-interface center W≈12.7 fails export | evidence:docs/d017_source_decomposition.md
- 2026-07-16 | area:d017 | lesson:Activation yield α>0 needs E_A=E_F/(1+α); frozen E_A=1 creates potential | evidence:docs/d017_activation_yield_analysis.md
- 2026-07-16 | area:d018 | lesson:Constrained φ rebuilds decaying structure into W; production~R^1 vs decay~R^2 blocks restoring nullcline | evidence:docs/d018_radius_scaling.md
- 2026-07-16 | area:d018 | lesson:Observer StructureProvenanceTracer E/K is opt-in on Simulation; default None preserves causality | evidence:digital-protocell/crates/chemistry-core/src/d018_provenance.rs
- 2026-07-16 | area:d019 | lesson:Interface-limited decay (ε=0.05+I(φ)) restores prescribed/live g-crossing; Stage E still needs joint-rate recalibration under v3 | evidence:docs/d019_mechanism_comparison.md
- 2026-07-16 | area:d020 | lesson:D-020 sensitivity is full-rank; bounded rate recovery still fails long-run retention/localization | evidence:docs/d020_joint_rate_recovery_report.md
- 2026-07-16 | area:d021 | lesson:v4 ε-protected membrane decay recovers A retention (~1.0) but R22 localization stays ~0.889 under frozen rates | evidence:experiments/generated/d021/gate3/gate3_prebalance.json
- 2026-07-17 | area:d022-interface-affinity | lesson:χ_M/D_M≤2 lifts R22 M loc only ~0.889→0.891; A retention stays ~1.0; seven-field loc tuning rejected | evidence:experiments/generated/d022/manifest.json
- 2026-07-17 | area:d023 | lesson:v6 A→P→M with χ_M=0 fails Stage B loc (≤0.8895); higher k_assembly worsens loc via bulk M diffusion | evidence:docs/d023_membrane_precursor_assembly_report.md
- 2026-07-17 | area:d025 | lesson:n=∇φ/|∇φ| points inward so expansion mean v_n<0; Stage D fixed_geometry must check φ only not S | evidence:crates/chemistry-core/tests/d025_tests.rs
- 2026-07-17 | area:d025-stage-c | lesson:v7 Stage C stoichiometric closure must use η_c like v2; ActivatedMetabolism path has no P/S—check those under ConstrainedRadius | evidence:crates/experiment-runner/src/d025.rs
- 2026-07-17 | area:d025-stage-e | lesson:Constrained Stage E under frozen D-024 k_ads reaches 200k with Γ≈1 and C_ret≥0.80 but A_ret≈0.51 and zero qualifying windows; LONG_TRANSIENT forbids solver entry | evidence:experiments/generated/d025/stage_e_reference/reference_terminal_classification.json
- 2026-07-17 | area:d026-stage-e | lesson:Γ localization≈1 with θΓ decline and ads≪turnover while delta_P≫ads means coverage not localization fails Stage E | evidence:docs/d026_stage_e_activated_resource_recovery.md
- 2026-07-17 | area:d026-stage-e | lesson:Freeze-surface control rescues A_ret≈+0.14; zeroing A transport or structure/rep demand does not | evidence:experiments/generated/d026/causal_controls/summary.json
- 2026-07-17 | area:d027-checkpoint | lesson:Restored Stage E/v7 runs must copy dt_cap; default MAX_DT=0.0025 clamps checkpoint dt=0.005 and breaks rate parity | evidence:experiment-runner/src/d027.rs Gate0
- 2026-07-17 | area:d027-adsorption | lesson:Median k_ads_required across fixed/dynamic/StageE ≈30.4× D024 k_ads; span≈1.81× portable; 1×/2× isolate Q straddles 1.0 | evidence:experiments/generated/d027/adsorption_basis/
- 2026-07-17 | area:d028 | lesson:Isolated k_ads root ~0.04867 balances fixed-interface Q/g but fails portability on dynamic/Stage-E states (Q≫1.1) | evidence:experiments/generated/d028/portability/portability.json
- 2026-07-18 | area:surface-exchange | lesson:Six-state L≈αA−βB NNLS for reversible exchange projects β→0 (B≫A; L not anti-correlated with B); stop as NOT_IDENTIFIABLE | evidence:experiments/generated/d029/parameter_identification/parameter_identification.json
- 2026-07-18 | area:surface-exchange | lesson:Orthogonal θ=0/P=0 assays recover α,β of planted v8 law; D-029 β→0 was natural-balance non-excitation, not law absence | evidence:experiments/generated/d030/parameter_recovery/parameter_recovery.json
- 2026-07-18 | area:surface-exchange | lesson:Isolated renewal Gate7 can CapacityExceeded after long burn even when short seed Q looks finite; report accepted_in_window+last_reject | evidence:experiments/generated/d030/isolated_turnover/isolated_turnover.json
- 2026-07-18 | area:surface-exchange | lesson:V1 CapacityExceeded with 0 accepted steps is discrete overshoot; V2 BE+Strang accepts steps under same αβKk; short Q may still exceed 1.02 | evidence:experiments/generated/d031/isolated_turnover/short_diagnostic.json
