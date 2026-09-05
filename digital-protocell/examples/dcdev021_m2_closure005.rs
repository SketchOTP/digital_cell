#![allow(dead_code)]

mod accepted_closure_context {
    include!("dcdev021_m2_entry027.rs");
    include!("dcdev021_m2_closure001_impl.rs");
    include!("dcdev021_m2_closure002_impl.rs");
    include!("dcdev021_m2_closure003_impl.rs");
    include!("dcdev021_m2_closure003r1_impl.rs");
    include!("dcdev021_m2_closure004_impl.rs");
    include!("dcdev021_m2_closure005_impl.rs");
}

fn main() {
    accepted_closure_context::c5_main();
}
