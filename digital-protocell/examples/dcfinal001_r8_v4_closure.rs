//! DC-FINAL-001-R8: V4 closure material and load-bearing consistency repair.

mod r5_engine {
    include!("dcfinal001_r5_v4_neck.rs");
}

fn main() {
    r5_engine::run_r8();
}
