# Current structural-build decomposition

For each D-096 structural-build evaluation, the observer decomposes the existing unscaled flux into:

`J_unscaled = J_base + J_strain`

where `J_base` is the `g0` contribution and `J_strain` is the additional contribution from `g_strain(eps) - g0`.

The unchanged current law is:

`J_current = (J_base + J_strain) * g_repair`

The observer ledger records actual integrated build, both pre-gain components, and their repair-gain amplification. It has no causal feedback. The 216 audit rows are in `current_flux_decomposition.json`.
