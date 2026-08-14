# Ribossome

Source: `Manalokosdev/Ribossome`, commit `cb3bb85f12b8aad44969437de56696583f8847b8`, tree `712b4133e41de5245a3746fb2896d94c93690704`; MIT in `docs/LICENSE`; Rust/Cargo with wgpu.

`src/main.rs` couples wgpu device/pipeline/buffer setup, compute/render scheduling, readback, UI, serialization and profiling. WGSL defines agents, bodies, packed two-bit genomes, chemical/fluid grids, spatial data, trails and spawn requests. Reproduction translates codons into bodies, copies or reverse-complements genomes, halves inherited energy, mutates, and increments generation. Simulation shaders couple movement, energy, feeding, damage/death and a stable-fluids-style environment.

Reusable patterns are GPU buffer layout, compute/render separation, profiling, grids, spatial indexing, readback and snapshot compression. Organism assumptions are incompatible: genome directly builds the body, runtime grants agent existence, and reproduction is genome copy rather than material fission. The monolithic binary makes extraction high/prohibitive. Classification: `ADAPT`/`PATTERN_ONLY` infrastructure, `REJECT_INTEGRATION` organism code; no source copy or dependency.
