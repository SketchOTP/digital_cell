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
