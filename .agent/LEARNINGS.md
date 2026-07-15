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
