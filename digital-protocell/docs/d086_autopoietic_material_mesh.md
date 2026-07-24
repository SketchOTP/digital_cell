# D-086: Autopoietic Material-Mesh Protocell

## Strategic decision

D-008 phase-field structural lineage closed after D-085.

Records:

- `D008_PHASE_FIELD_LINEAGE_CLOSED_REJECTED`
- `PHASE1_PHASE_FIELD_BODY_RETIRED`
- `PHASE1_AUTOPOIETIC_MESH_RESET_AUTHORIZED`

## Equation identity

- Equation version: `autopoietic_material_mesh_v1`
- Field schema: `mesh_vertices_edges_v1`
- Branch: `phase1-autopoietic-material-mesh` (from `d57fed0` / `D-085-phase-field-structure-rejected`)
- Legacy `d008-membrane-metabolic-closure` preserved; φ equation versions untouched

## Mesh equations

Closed polygonal mesh. Edge rest length \(\ell_i^0 = m_i / \rho_s\).

Overdamped mechanics:

\[
\gamma \dot{\mathbf{x}}_i = \mathbf{F}_{\mathrm{stretch},i} + \mathbf{F}_{\mathrm{bend},i} + \mathbf{F}_{\mathrm{pressure},i}
\]

\[
E_{\mathrm{stretch}} = \sum_i \frac{k_s}{2\ell_i^0}(\ell_i-\ell_i^0)^2,\quad
E_{\mathrm{bend}} = \sum_i \kappa_b(1-\cos\theta_i)
\]

Local pressure from inside/outside chemistry only (no area/radius target).

Structural production: \(A \rightarrow m + W\) with

\[
J_{\mathrm{build},i} = k_{\mathrm{build}}\, q(C)\, a\, g(\varepsilon_i)\, \ell_i,\quad
g(\varepsilon)=g_0+\frac{0.45\max(0,\varepsilon)}{K_\varepsilon+\max(0,\varepsilon)}
\]

Membrane: \(L \rightleftharpoons b_i\); permeability from occupancy \(b_i/b_{\max,i}\).

## Selected mechanical candidate (`center`)

Derived from Laplace basin criterion \(\alpha = k_\pi \Pi_{\mathrm{chem}} / k_s < 1/R_{\max}\):

| param | value |
|------|------:|
| \(\gamma\) | 1.0 |
| \(k_s\) | 14.0 |
| \(\kappa_b\) | 2.0 |
| \(k_\pi\) | 0.22 |
| \(\mathrm{d}t\) | 0.02 |
| \(\alpha\) target | 0.022 |

## Campaign result

**Primary:** `D086_MESH_PROTOCELL_PHASE1_CANDIDATE_PASS`

| Gate | Result |
|------|--------|
| 0 Preservation | PASS |
| 1 Mechanics | PASS |
| 2 Passive basin | PASS (center) |
| 3 Metabolism/transport | PASS |
| 4 Turnover | PASS |
| 5 Dynamic basin 15-run | PASS 15/15 |
| 6 Damage/repair | PASS |
| 7 Starvation/death | PASS |
| 8 Phase 1 decision | PASS |

Phase 1 status: `PHASE1_AUTOPOIETIC_CANDIDATE_PASS`  
Production: `MESH_PHASE1_LINEAGE_QUALIFIED`

## Next

Independent causal audit; reproducibility campaign; Linux runtime hardening; then Phase 2 reproduction.
`next_execution_started`: false
