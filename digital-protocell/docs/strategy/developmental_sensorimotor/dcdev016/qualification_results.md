# DC-DEV-016 Metabolic Break-Even Resource Sufficiency Challenge

## Authority

- Entry: `aa33c5d2fa5dfe545a82925c28d95e57c480293f`
- Source: `strategy/dc-dev-015-metabolic-restoration-audit`
- Scope: observer-only; one derived resource inventory; no chemistry, mechanics, regulation, behavior, or resource-seeking changes.
- Formal sequence: 5,000 accepted settlement steps, 480-step deprivation, 480-step challenge.

## Derived challenge

DC-DEV-015 measured a matched no-delivery activated-store decline of `11.387290380605897` and a delivery fraction of `0.7805418875976666`. The single preregistered inventory was therefore:

`11.387290380605897 / 0.7805418875976666 = 14.588954880632265`

N and F were both initialized to that value at the unchanged center `[4.8, 0.0]` and radius `1.5`. No second inventory was tried.

## Result

The derived arm delivered `11.401893960861464` matched N/F units, exceeding the target. Resource-world N/F conservation passed, and `E_available` increased from `60.82781514212436` to `64.13760842349555`.

The exact settled and deprived body hashes remain required. Arm-level final mesh hashes are retained as diagnostics, but arm parity is evaluated against the committed A/R/N/F and delivery values within `1e-10`, because cross-platform floating-point mesh serialization can change those diagnostic hashes without changing the accepted numeric state.

Stored activated material did not restore: final challenge A was `0.25977003489308437`, R was `0.4981052670434316`, and `E_stored` was `53.67843279629684`, all below the deprived start. N/F-to-A conversion was observed at fraction `0.082680854329883`.

The legacy DC-DEV-015 scalar destination sum is not used as a conservation gate. Its status is recorded as `ACCOUNTING_CONTRACT_NOT_CLOSED`; only the explicit resource-world N/F conservation relationship is classified as conserved here.

## Classification

`DCDEV016_CONVERSION_STORAGE_BOTTLENECK_CONFIRMED`

This means the single derived resource condition was physically sufficient for available-potential break-even, but existing conversion/storage did not restore A, R, or E_stored on the 480-step window.

No metabolic repair, parameter tuning, behavior, hunger variable, or DC-DEV-017 work is authorized by this result.
