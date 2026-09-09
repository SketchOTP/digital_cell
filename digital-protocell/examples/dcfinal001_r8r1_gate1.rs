//! DC-FINAL-001-R8R1 Gate 1: frozen-state mechanics counterfactual.

mod r5_engine {
    include!("dcfinal001_r5_v4_neck.rs");
}

fn main() {
    r5_engine::run_r8r1_gate1();
}
