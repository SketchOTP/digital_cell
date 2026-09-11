mod r10r7_evolution {
    include!("dcfinal001_r4_evolution.rs");
}

fn main() {
    std::env::set_var("DCFINAL001_R10R9R1_RESERVE", "1");
    std::env::set_var("DCFINAL001_R10R9R3_RESERVE", "1");
    let args = std::env::args().collect::<Vec<_>>();
    let panel = args
        .windows(2)
        .find(|pair| pair[0] == "--panel")
        .map(|pair| pair[1].clone())
        .unwrap_or_else(|| "/tmp/dcfinal001_r10r7_panel.json".to_string());
    r10r7_evolution::run_r10r7_variant_feasibility(
        "/tmp/dcfinal001_r10r7_variant_feasibility.json",
        &panel,
    );
}
