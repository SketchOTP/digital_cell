//! DC-FINAL-001-R8R1: sign-aware V4 maturation mechanics qualification.

mod r5_engine {
    include!("dcfinal001_r5_v4_neck.rs");
}

fn main() {
    r5_engine::run_r8r1();
}
