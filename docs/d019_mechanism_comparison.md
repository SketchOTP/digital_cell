# D-019 Mechanism Comparison

Evaluated at D-018 radii `R ∈ {14, 18, 22, 26, 30, 34}` with prescribed interior
`(C,A)=(0.4,0.2)` and live constrained windows (settle 100 / measure 200).

| Mechanism | Equations changed | New params | Prescribed restoring | Full turnover at φ=1 | Selection |
| --- | --- | --- | --- | --- | --- |
| A Phase-volume synthesis `r∝A·act(C)·H(φ)` | 1 (production) | 0 | **No** (anti-restoring under uniform A) | Yes | Rejected |
| B Interface-limited turnover `decay∝φ·(ε+I(φ))` | 1 (decay) | 1 frozen floor ε=0.05 | **Yes** | Yes | **Selected** |
| C Local curvature maintenance `decay∝φ·(ε+\|∇²φ\|)` | 1 (decay) + stencil | 1 floor | Yes (prescribed) | Yes | Not preferred (more machinery) |

Selection priority applied: fewest changed equations → strongest local membrane-causal
interpretation → widest restoring span → lowest new parameter count.

Selected: **interface_limited_turnover** (`D019_SELECT_INTERFACE_LIMITED_TURNOVER`).

Live pre-balance (v3): restoring crossing at `k_center≈0.2576` with
`g(R18)>0`, `g(R22)≈0`, `g(R26)<0`; max constraint contamination ≈0.0016 ≤0.05.
