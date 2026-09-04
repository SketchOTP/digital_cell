#![allow(dead_code)]

// CLOSURE-002 is an additive, assay-only requalification.  The R1 assay and
// its sealed evidence remain unchanged.  The accepted ENTRY-027 source is
// included first so this module can reuse its physical fission replay and the
// R1 native inherited-polarity helpers without changing production code.
mod accepted_closure_context {
    include!("dcdev021_m2_entry027.rs");
    include!("dcdev021_m2_closure001_impl.rs");
    include!("dcdev021_m2_closure002_impl.rs");
}

fn main() {
    accepted_closure_context::closure002_main();
}
