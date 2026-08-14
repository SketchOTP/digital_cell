# ALIEN

Source: `chrxh/alien`, branch `develop`, commit `91b58172391014c1512d6919a45fdcc9f6e8b3ba`, tree `75bcd7e07abdfc1e99d12b61297e00f270e358af`; BSD-3-Clause in `LICENSE`; CMake/CUDA with HIP compatibility files.

The repository separates engine interfaces/implementation, kernels, persistence and GUI. CUDA data structures model energy, objects, neural nets, sensors, muscles, creatures, lineages, solid/fluid/free cells, connections and genome graphs. Genome mutation includes neuron/connection/cell-type/geometry and gene operations; processors and spatial/energy maps are GPU oriented.

Useful patterns are data-oriented entity arrays, processor/kernel separation, CPU/GPU façade, memory management, maps, persistence and profiling. Incompatible assumptions include genome-built creatures, neural/muscle/sensor semantics, runtime-created agents, CUDA-specific kernels/atomics, and UI/persister coupling. Direct extraction is high/prohibitive and platform risk is high. Classification: `PATTERN_ONLY`/`ADAPT` architecture, `REJECT_INTEGRATION` source. It is not a Digital Cell world implementation.
