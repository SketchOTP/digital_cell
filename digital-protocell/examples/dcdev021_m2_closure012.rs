//! CLOSURE-012 observer-only diagnosis of the existing active-work/fission gate.
//!
//! This replays the accepted CLOSURE-011 A-fraction assay and records the
//! already-existing pinch, cross-bond, and A-availability conditions at every
//! fission-eligible checkpoint.  It does not alter production biology.

mod accepted_closure010 {
    include!("dcdev021_m2_closure010.rs");
}

fn main() {
    std::env::set_var("DC_CLOSURE011_A_FRACTION", "1");
    std::env::set_var("DC_CLOSURE012_FISSION_AUDIT", "1");
    accepted_closure010::run();
}
