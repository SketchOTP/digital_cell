//! DC-FINAL-001-R6: curvature-gated inward-normal constriction emergency
//! closure. Gate 1 is observer-only and must pass before the authorized
//! normal-force composition is implemented.

mod r5_engine {
    include!("dcfinal001_r5_v4_neck.rs");
}

fn main() {
    r5_engine::run_r6();
}
