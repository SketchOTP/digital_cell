# Production diff

The default `structural_build_flux` path now detects an enabled finite allocation with the historical D-096 submodes disabled and returns the repaired equation directly. `StructuralBuildMode::D096RepairSpecificShadow` remains available as an audit oracle; the default runtime no longer needs that flag.

The observer ledger reports zero baseline amplification and strain-only amplification for both production and shadow modes. `d096_allocation::function_gain`, allocation parameters, mutation probability, mutation sigma, candidate hashes, and equation/schema identifiers are unchanged.
