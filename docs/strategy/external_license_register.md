# External ALife License Register

Status: pre-audit register. No external code has been copied, adapted, vendored, or added as a dependency by DC-SR-001.

| Project | Repository / source | License status | Files examined | Planned use | Decision |
|---|---|---|---|---|---|
| Stringmol | Repository to be located in SR-002 | UNVERIFIED | None | Heredity benchmark | REVIEW |
| Evo2Sim | Repository to be located in SR-002 | UNVERIFIED | None | Ecology benchmark | REVIEW |
| MABE2 | https://github.com/mercere99/MABE2 | UNVERIFIED from repository | None | Architecture pattern | REVIEW |
| Avida | https://github.com/devosoft/avida | UNVERIFIED from repository | None | Methodology benchmark | REVIEW |
| Aevol | Repository to be located in SR-002 | UNVERIFIED | None | Genome/evolution methods | REVIEW |
| Ribossome | https://github.com/Manalokosdev/Ribossome | UNVERIFIED from repository | None | Rust/wgpu infrastructure audit | REVIEW |
| ALIEN | https://github.com/chrxh/alien | UNVERIFIED from repository | None | GPU/world benchmark | REVIEW |
| ASAL | https://github.com/SakanaAI/asal | UNVERIFIED from repository | None | Discovery sidecar audit | REVIEW |
| CAX | Repository to be located in SR-002 | UNVERIFIED | None | JAX research sidecar | REVIEW |
| Lenia / Flow Lenia | Canonical repositories to be located in SR-002 | UNVERIFIED | None | Surrogate/reference only | REVIEW |
| DISHTINY | Repository to be located in SR-002 | UNVERIFIED | None | Future multicellularity benchmark | REVIEW |
| Polyworld | Repository to be located in SR-002 | UNVERIFIED | None | Future neural benchmark | REVIEW |
| Tierra | Repository to be located in SR-002 | UNVERIFIED | None | Historical control | REVIEW |
| Evochora | Repository to be located in SR-002 | UNVERIFIED | None | Runtime/data benchmark | REVIEW |

Before adoption, add exact commit, license file and SPDX classification, examined paths, copyright/attribution requirements, modifications, notices, and distribution constraints. Unknown license means no code reuse.

## DC-SR-002 pinned audit register

The audited shallow-clone evidence is pinned in `docs/strategy/external_alife_audit.json`. This is an engineering register, not legal advice.

| Project | Pinned license file | Engineering class | Source reuse decision |
|---|---|---|---|
| Stringmol | `LICENSE` | GPL-2.0 / high copyleft | BENCHMARK; no integration |
| Evo2Sim | `LICENSE` | GPL-3.0 / high copyleft | BENCHMARK; no integration |
| MABE2 | `LICENSE` | MIT / dependency review | ADAPT architecture; no source integration |
| Avida | `avida-core/COPYING` | LGPL-3.0-style / review required | BENCHMARK methodology; no integration |
| Aevol | `COPYING` | GPL-3.0 / high copyleft | BENCHMARK; no integration |
| Ribossome | `docs/LICENSE` | MIT / coupled implementation | PATTERN_ONLY; no integration |
| ALIEN | `LICENSE` | BSD-3-Clause / CUDA risk | PATTERN_ONLY; no integration |
| ASAL | `LICENSE` | Apache-2.0 / sidecar boundary | Future adapter only |
| Lenia | `LICENSE.md` | MIT / surrogate | Benchmark only |
| Flow Lenia | none found | UNKNOWN | NO_CODE_REUSE_UNTIL_RESOLVED |
| Evochora | `LICENSE` | MIT / separate VM | Benchmark; no integration |
| DISHTINY | `LICENSE` | MIT / multicell benchmark | Benchmark; no integration |
| Polyworld | `LICENSE.txt` | APSL-2.0 / high risk | Benchmark; no integration |
| Tierra | none clear | UNKNOWN | NO_CODE_REUSE_UNTIL_RESOLVED |
| CAX | source identity unresolved | UNKNOWN | NO_CODE_REUSE_UNTIL_RESOLVED |
