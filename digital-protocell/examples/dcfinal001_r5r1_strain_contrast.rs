//! DC-FINAL-001-R5R1: exhaust the sole preauthorized zero-parameter
//! mean-relative tensile-strain localization fallback.

mod r5_engine {
    include!("dcfinal001_r5_v4_neck.rs");
}

fn main() {
    r5_engine::run_r5r1();
}
