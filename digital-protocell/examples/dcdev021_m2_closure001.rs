#![allow(dead_code)]

// Closure-level assay.  ENTRY-027 is included only as the accepted authority
// for the unforced 198 -> 78/122 physical fission and inherited polarity
// state.  The new closure implementation is kept in a separate file so the
// historical example remains immutable in meaning.
mod accepted_entry027 {
    include!("dcdev021_m2_entry027.rs");
    include!("dcdev021_m2_closure001_impl.rs");
}

fn main() {
    accepted_entry027::closure_main();
}
