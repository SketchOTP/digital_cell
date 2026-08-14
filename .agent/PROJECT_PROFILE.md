# PROJECT_PROFILE.md

## Identity
- Product: Digital Cell — standalone Linux-based digital lifeform (protocell → embodied creature)
- Repo: digital_cell
- Mimir slug: digital_cell
- Serena project: digital_cell (`.serena/project.yml`)
- Source of truth: `.agent/PROJECT_GOAL.md`
- Github repo: git@github.com:SketchOTP/digital_cell.git
- Github username: SketchOTP
- Github email: sketchotp@gmail.com

## What we are building
A self-maintaining digital protocell that develops into a persistent, embodied individual. Aliveness must emerge from internal organization — digital chemistry, cells, metabolism, body, nervous system, memory, and environment — not from scripted behavior, reaction tables, animation states, or an LLM prompt.

## Phase 1 status (D-087 certified; runtime closed)
- Certified branch seal: `phase1-autopoietic-material-mesh` @ `D-087-phase1-autopoietic-protocell-certified` / release `phase1-v1.0-research`
- Active Phase 2 branch: `phase2-growth-division-inheritance`
- Strategic rebase branch: `strategy/prior-art-integration-rebase`
- Strategic status: DC-SR-001 complete; external prior-art audit required before further evolutionary or generic subsystem work
- D-094R status: `D094R_PRESERVED_PENDING_PRIOR_ART_REBASE`
- Body substrate: conserved material mesh (`autopoietic_material_mesh_v1`) — **independently certified** (`MESH_PHASE1_V1_FROZEN`)
- Science records: `PHASE1_COMPLETE`, `PHASE1_AUTOPOIETIC_PROTOCELL_CERTIFIED`, `MESH_PHASE1_V1_FROZEN`, `PHASE2_REPRODUCTION_AUTHORIZED`, `PHASE1_SCIENCE_CERTIFIED`
- Runtime: `PHASE1_RESEARCH_RUNTIME_QUALIFIED` (≥90 min wall-clock packaged run)
- Phase 2: `D088_CAUSAL_GROWTH_FISSION_INHERITANCE_QUALIFIED` / `PHASE2_PHYSICAL_REPRODUCTION_QUALIFIED` / `D088_PHYSICAL_REPRODUCTION_FROZEN`
- D-089/D-090: heredity qualified; `D090_VALID_SELECTION_ECOLOGY_NOT_ESTABLISHED`
- D-091: `D091_METABOLIC_RESERVE_QUALIFIED_COMPOSITIONAL_SELECTION_REJECTED`; schema `autopoietic_material_mesh_metabolic_reserve_v1`; sealed `58817ac`
- D-092: `D092_TEMPLATE_HEREDITY_QUALIFIED_MOTIF_SELECTION_REJECTED`; schema `autopoietic_material_mesh_catalytic_template_v1`; μ=0.01; σ=0.15; Phase 3 not authorized
- D-093: `D093_TEMPLATE_NETWORK_HEREDITY_QUALIFIED_SELECTION_UNTESTABLE_ZERO_GENERATION`; schema `autopoietic_material_mesh_template_network_v1`; circular L=12 pair sites; Phase 3 not authorized
- D-094R2: `D094_AUTOCATALYTIC_SET_HEREDITY_QUALIFIED_SELECTION_REJECTED`; valid H/B/neutral eight-generation Gate 6 campaign. The autocatalytic-set route is closed as selectable substrate; Gates 7/8, D-095, and Phase 3 are not authorized.
- Next: evolutionary substrate architecture review (no execution directive active)
- Stack: Rust (`digital-protocell`), chemistry-core + experiment-runner + phase1-certifier
- Runtime package: `digital-protocell-phase1-v1`
- Current strategy: `AUTOPIETIC_CORE_WITH_EXTERNAL_ALIFE_INTEGRATION`

## Storage / disk
- Single policy: `.cursor/rules/06-storage-archive.mdc`
- Regenerable bulk → `/mnt/storage1tb/cache/project-artifacts/digital_cell/` (symlinks at repo paths)
- Space relief (when `/` ≥90% used or <15 GiB free): move closed/clean trees to `/mnt/storage1tb/archived-projects/`, absolute symlink at original `Projects/` path, verify Git through it, log `ARCHIVE_MANIFEST.jsonl`
- Never auto-archive active worktrees, canonical repo root, shared Git object stores, `.git/worktrees`, uncommitted work, or active provenance
- `independent_backup_status: NOT ESTABLISHED` (archive-to-disk ≠ independent backup)

## Architectural constraints (from PROJECT_GOAL)
- No central controller may directly command eat, repair, approach, bond, fear, reproduce, or scripted emotional responses
- Behavior emerges from interacting subsystems and developmental history
- Continued existence depends on real simulated resource and energy flows
- Damage, starvation, development, reproduction, and death are causal, not state-scripted
- Different life histories must produce observably different individuals
- Must remain autonomous when no observer is present
- Mature form: screen as boundary to our world; webcam/microphone perception; movement, posture, proximity, environment manipulation, nonverbal vocalization
- Success is measured by emergent individuality and self-maintenance — not by proving consciousness

## Stack
- Target platform: Linux
- Implementation: Rust workspace `digital-protocell` (chemistry-core, experiment-runner)
- Agent/tooling docs: Markdown, Cursor rules, Mimir, Serena, cocoindex-code

## Common commands
- Tests: `cargo test -p phase1-certifier --release --test metrics_semantics`
- Pipeline: `cargo run --release -- d087 pipeline`
- Package: `scripts/package_phase1_linux.sh`

<!-- MIMIR_PROJECT_BINDING_START -->
## Mimir binding
- Mimir project ID: 7bff443192353517
- Project name: digital_cell
- On every machine, call mimir_project_resolve with this ID and that machine's workspace path.
- Register only when this binding is absent; never create a host path or map a drive.
<!-- MIMIR_PROJECT_BINDING_END -->

## DC-SR-002 strategy decisions
- External ALife audit is complete at tag `DC-SR-002-external-alife-audit`.
- Certified material mesh, physical growth/fission, template heredity, phase-1 certifier, and Godot bridge remain KEEP/authoritative.
- D-094 remains frozen pending a repaired protocol/evolution harness. The next authorized shape is a thin Digital Cell-owned harness with neutral controls, explicit ecology schedules, generation/parent accounting, lineage, extinction reasons, and immutable artifacts.
- GPU is deferred; a future measured need may justify a thin wgpu backend, but ALIEN/Ribossome organism code and CUDA migration are rejected.
- ASAL is a possible future hypothesis sidecar only; its novelty/embedding metrics never certify Digital Cell biology.

## DC-SR-003 harness boundary
- The next implementation layer is `evolution-harness`, above organism biology and below experiment orchestration.
- `chemistry-core` remains authoritative and must not depend on the harness.
- Selection observation is measurement-only; mutation context contains no fitness/winner/survival input.
- D-090 through D-093 are representable protocol fixtures; D-094 is translated but never executed in SR-003.
