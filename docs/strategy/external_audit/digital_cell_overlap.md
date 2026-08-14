# Digital Cell overlap map

At entry commit `24d57b4`, Digital Cell retains the certified material mesh and phase-1 certifier. `material_mesh.rs` owns monomers, template chains, edges, lumped chemistry, membrane/area/perimeter and reserve. `mesh_fission.rs` performs local pinch/rebonding and conservative spatial partition. `mesh_population.rs` records lineage/generation but does not make lineage causal. `population_selection.rs` and `spatial_shared_dish.rs` provide shared finite resources, diffusion, uptake, damage, topology and observation without fitness culling. `template_polymer.rs`, `template_copying.rs`, and `template_network.rs` provide local template/catalyst mechanisms; copying is not whole-chain cloning and fission does not read sequence.

External overlap is therefore strongest at the level of concepts: executable molecules (Stringmol), ecology/protocols (Evo²Sim/Avida), modular experiment plumbing (MABE2/Evochora), mutation/lineage analysis (Aevol), GPU patterns (Ribossome/ALIEN), and future sidecars/surrogates (ASAL/Lenia). No audited source supersedes the material-causal substrate, and none is a safe direct dependency.

The primary known gap is harness/ecology measurement, not proof that heredity cannot exist. D-094 remains frozen because the current evidence is not enough to execute a credible selection campaign under repaired controls.
