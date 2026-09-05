//! CLOSURE-014 assay-only contact-boundary A-fraction reproduction ceiling.
//!
//! Contacted material patches receive exact zero motor activity; noncontact
//! patches retain the corrected literal A-fraction output. Production biology
//! and the accepted CLOSURE-013 result remain unchanged.

mod accepted_closure010 {
    include!("dcdev021_m2_closure010.rs");
}

fn main() {
    std::env::set_var("DC_CLOSURE011_A_FRACTION", "1");
    std::env::set_var("DC_CLOSURE013_A_FRACTION_LAW", "1");
    std::env::set_var("DC_CLOSURE014_CONTACT_BOUNDARY", "1");
    accepted_closure010::run();
}
