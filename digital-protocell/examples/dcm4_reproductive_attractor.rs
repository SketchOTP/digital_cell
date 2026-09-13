//! Observer-only DC-M4 reproductive-attractor architecture gate.

#[path = "dcfinal001_r4_evolution.rs"]
mod r5_evolution;

fn main() {
    std::env::set_var("DCFINAL001_R10R9R1_RESERVE", "1");
    std::env::set_var("DCFINAL001_R10R9R3_RESERVE", "1");
    std::env::set_var("DCFINAL001_R10R9R5_CANONICAL", "1");
    r5_evolution::run_dc_m4_architecture_gate();
}
