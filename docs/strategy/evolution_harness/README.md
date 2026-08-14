# Modular Evolution and Ecology Harness

DC-SR-003R2 establishes a bounded native experiment substrate before any D-094 requalification. The harness is a measurement and protocol layer, not a fitness controller and not a new organism biology layer.

Verification: `cargo +1.89.0-x86_64-pc-windows-msvc test -p evolution-harness` passes 27/27 after installing the existing sanctioned local toolchain's missing MSVC target and rustfmt component. GitHub workflow `.github/workflows/evolution-harness.yml` repeats formatting and tests on the exact PR head. Atlas remains a source checkout without Rust/Cargo on PATH; no arbitrary system toolchain was installed there.
