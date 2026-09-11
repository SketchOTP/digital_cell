#[path = "dcfinal001_r5_v4_neck.rs"]
mod closure;

fn main() {
    std::env::set_var("DCFINAL001_R10R9R1_RESERVE", "1");
    std::env::set_var("DCFINAL001_R10R9R3_RESERVE", "1");
    closure::run_r10r5_d096v4_integrated_reproduction();
}
