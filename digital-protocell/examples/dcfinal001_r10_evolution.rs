mod r10_closure {
    include!("dcfinal001_r5_v4_neck.rs");
}

mod r10_evolution {
    include!("dcfinal001_r4_evolution.rs");
}

fn main() {
    r10_evolution::run_r10_evolution();
}
