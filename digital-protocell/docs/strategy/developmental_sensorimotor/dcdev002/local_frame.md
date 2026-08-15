# Local material frame

The versioned frame is `digital_cell_local_material_frame_v1`.  It contains
the topology size and identity plus one patch record for each current mesh
vertex.  A patch record contains its index, the previous and next ring-neighbor
indices, the bounded raw local stimulus, and the accepted `dt`.

The frame excludes whole-organism totals, population, generation, fitness,
treatment/environment labels, textual semantic labels, and target state.
Same-count reindexing is outside this assay; topology changes fail closed.
