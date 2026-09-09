//! R9 refractory curvature-normal reproduction and closure qualification.

mod r5_engine {
    include!("dcfinal001_r5_v4_neck.rs");
}

fn main() {
    r5_engine::run_r9();
}
