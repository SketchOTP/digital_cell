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

## Phase 1 status (D-086 amendment)
- Active branch: `phase1-autopoietic-material-mesh`
- Sealed lineage: `d008-membrane-metabolic-closure` (phase-field body closed/rejected)
- Body substrate: conserved material mesh (`autopoietic_material_mesh_v1`) — D-086 Phase 1 candidate PASS (`MESH_PHASE1_LINEAGE_QUALIFIED`)
- Records: `D008_PHASE_FIELD_LINEAGE_CLOSED_REJECTED`, `PHASE1_PHASE_FIELD_BODY_RETIRED`, `PHASE1_AUTOPOIETIC_MESH_RESET_AUTHORIZED`
- Stack: Rust (`digital-protocell`), chemistry-core + experiment-runner

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
- Tests: `cargo test -p chemistry-core --test d086_tests --release`
- Pipeline: `cargo run --release -- d086 pipeline`

<!-- MIMIR_PROJECT_BINDING_START -->
## Mimir binding
- Mimir project ID: 7bff443192353517
- Project name: digital_cell
- On every machine, call mimir_project_resolve with this ID and that machine's workspace path.
- Register only when this binding is absent; never create a host path or map a drive.
<!-- MIMIR_PROJECT_BINDING_END -->
