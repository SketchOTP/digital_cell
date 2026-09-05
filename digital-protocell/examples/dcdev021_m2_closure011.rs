//! CLOSURE-011 opt-in reproduction ceiling assay.
//!
//! The accepted CLOSURE-010 assay remains the default path.  This wrapper
//! selects one assay-local material interpretation and writes its evidence to
//! the CLOSURE-011 directory; it does not alter production runtime behavior.

mod accepted_closure010 {
    include!("dcdev021_m2_closure010.rs");
}

fn main() {
    std::env::set_var("DC_CLOSURE011_A_FRACTION", "1");
    accepted_closure010::run();
}
