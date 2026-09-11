#[path = "dcfinal001_r4_evolution.rs"]
mod evolution;

fn main() {
    std::env::set_var("DCFINAL001_R10R9R1_RESERVE", "1");
    std::env::set_var("DCFINAL001_R10R9R3_RESERVE", "1");
    std::env::set_var("DCFINAL001_R10R9R5_CANONICAL", "1");
    evolution::run_r10r9r5_shared_kernel_reproduction();
}
