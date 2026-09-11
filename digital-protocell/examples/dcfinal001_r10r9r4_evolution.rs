mod r10r9r4_evolution {
    include!("dcfinal001_r4_evolution.rs");
}

fn main() {
    std::env::set_var("DCFINAL001_R10R9R1_RESERVE", "1");
    std::env::set_var("DCFINAL001_R10R9R3_RESERVE", "1");
    r10r9r4_evolution::run_r10r9r4_evolution();
}
