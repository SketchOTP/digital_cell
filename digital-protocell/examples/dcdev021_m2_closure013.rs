//! CLOSURE-013 execution-semantics reconciliation.
//!
//! The accepted CLOSURE-012 evidence documents an A-fraction motor law, while
//! its helper executed the complement.  This assay selects the documented law
//! explicitly and emits the same fixed ecology and controls for comparison.
//! It does not modify production scientific runtime behavior.

mod accepted_closure010 {
    include!("dcdev021_m2_closure010.rs");
}

fn main() {
    std::env::set_var("DC_CLOSURE011_A_FRACTION", "1");
    std::env::set_var("DC_CLOSURE013_A_FRACTION_LAW", "1");
    accepted_closure010::run();
}
