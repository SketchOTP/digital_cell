# Repair diff

Authorized runtime diff, exactly:

```rust
|| mesh.equation_id == crate::d096_allocation::EQUATION_VERSION_FINITE_CATALYTIC_ALLOCATION
```

added to the existing explicit reserve compatibility dispatch in
`digital-protocell/crates/chemistry-core/src/metabolic_reserve.rs`.

The implementation uses the authoritative D-096 constant; no duplicate string
was introduced. Additional source changes are focused compatibility and causal
regressions only.
