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

## Architectural constraints (from PROJECT_GOAL)
- No central controller may directly command eat, repair, approach, bond, fear, reproduce, or scripted emotional responses
- Behavior emerges from interacting subsystems and developmental history
- Continued existence depends on real simulated resource and energy flows
- Damage, starvation, development, reproduction, and death are causal, not state-scripted
- Different life histories must produce observably different individuals
- Must remain autonomous when no observer is present
- Mature form: screen as boundary to our world; webcam/microphone perception; movement, posture, proximity, environment manipulation, nonverbal vocalization
- Success is measured by emergent individuality and self-maintenance — not by proving consciousness

## Stack (greenfield)
- Target platform: Linux
- Implementation languages, persistence, and runtime: TBD — no application code in repo yet
- Agent/tooling docs: Markdown, Cursor rules, Mimir, Serena, cocoindex-code

## Common commands
- Index (when code exists): `ccc init` once, then `ccc index` from repo root
- Tests / build / CLI: not configured yet

<!-- MIMIR_PROJECT_BINDING_START -->
## Mimir binding
- Mimir project ID: 7bff443192353517
- Project name: digital_cell
- On every machine, call mimir_project_resolve with this ID and that machine's workspace path.
- Register only when this binding is absent; never create a host path or map a drive.
<!-- MIMIR_PROJECT_BINDING_END -->
