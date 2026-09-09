//! R9 Gate 1 observer-only static-attractor replay.

mod r5_engine {
    include!("dcfinal001_r5_v4_neck.rs");
}

fn main() {
    r5_engine::run_r9_gate1();
}
