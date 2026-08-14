# Stringmol

Source: `franticspider/stringmol`, commit `52b5625c1d654971a57c86b24360334fb4f34bba`, tree `2fb675d6e14e7bc24b0dbf3e7e1d1acca89698ac`; GPL-2.0 in `LICENSE`.

## Implementation findings

`src/stringPM.h/.cpp` and `src/agents_base.h` model molecules as executable strings with energy, radius, age/step counters, mutation/decay, lineage and species state. `src/instructions.cpp` implements search/label matching, pointer movement, HCopy, arithmetic, toggles, conditionals, cleavage and lineage update. `src/smspatial.cpp` adds a spatial grid and local interactions.

The hereditary object is also executable machinery: active/passive strings bind, align, copy with substitution/indel error, and cleave to create daughters. No fixed genotype-to-phenotype decoder is required. Reproduction is an explicit molecular event, and fitness can be measured from reproduction, lifetime, biomass, or population statistics.

## Digital Cell comparison

This is a strong benchmark for catalytic/executable heredity, local matching, mutation operators, reaction/event logs, and lineage/species ancestry. It has no equivalent to a conserved material mesh, membrane boundary, mesh mechanics, or physical fission partition. Digital Cell's template-directed copying and fission are materially embodied and therefore not superseded.

Classification: `BENCHMARK` and `ADAPT` methodology; `REJECT_INTEGRATION` for source. GPL-2.0, legacy C++, and different causal substrate make direct reuse inappropriate. D-094 is conceptually overlapping in catalytic heredity but not duplicative of the mesh mechanism; selection/ecology remains unestablished in Digital Cell.
