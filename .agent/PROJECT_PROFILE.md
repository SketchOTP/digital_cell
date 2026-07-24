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
- Body substrate: conserved material mesh (`autopoietic_material_mesh_v1`) — **independently certified** (`MESH_PHASE1_V1_FROZEN`)
- Science records: `PHASE1_COMPLETE`, `PHASE1_AUTOPOIETIC_PROTOCELL_CERTIFIED`, `MESH_PHASE1_V1_FROZEN`, `PHASE2_REPRODUCTION_AUTHORIZED`, `PHASE1_SCIENCE_CERTIFIED`
- Runtime: `PHASE1_RESEARCH_RUNTIME_QUALIFIED` (≥90 min wall-clock packaged run)
- Phase 2: `D088_CAUSAL_GROWTH_FISSION_INHERITANCE_QUALIFIED` / `PHASE2_PHYSICAL_REPRODUCTION_QUALIFIED`
- Next: D-089 heritable catalytic variation
- Stack: Rust (`digital-protocell`), chemistry-core + experiment-runner + phase1-certifier
- Runtime package: `digital-protocell-phase1-v1`

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
