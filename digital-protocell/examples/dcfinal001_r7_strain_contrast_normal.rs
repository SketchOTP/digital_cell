//! DC-FINAL-001-R7: exact R5R1 mean-relative strain contrast composed with
//! the exact R6 A-funded inward-normal constriction mapping.

mod r5_engine {
    include!("dcfinal001_r5_v4_neck.rs");
}

fn main() {
    r5_engine::run_r7();
}
