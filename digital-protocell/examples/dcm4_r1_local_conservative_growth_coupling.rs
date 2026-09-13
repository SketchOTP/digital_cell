//! DC-M4 R1: bounded local conservative growth-placement candidate.

#[path = "dcfinal001_r4_evolution.rs"]
mod r5_evolution;

fn main() {
    // Match the accepted buffered-reserve/R5 execution context.  R1 changes
    // only the opt-in placement destination for existing D-088 growth.
    std::env::set_var("DCFINAL001_R10R9R1_RESERVE", "1");
    std::env::set_var("DCFINAL001_R10R9R3_RESERVE", "1");
    std::env::set_var("DCFINAL001_R10R9R5_CANONICAL", "1");
    r5_evolution::run_dc_m4_r1_local_growth_coupling();
}
