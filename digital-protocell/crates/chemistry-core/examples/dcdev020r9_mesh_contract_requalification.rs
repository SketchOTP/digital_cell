use chemistry_core::d020r9_analysis::run_pipeline;
use std::env;
use std::path::PathBuf;

fn main() {
    let out = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020r9"));
    match run_pipeline(&out) {
        Ok(report) => {
            println!("{}", report.primary_classification);
            println!("{}", out.display());
        }
        Err(error) => {
            eprintln!("DC-DEV-020-R9 failed: {error}");
            std::process::exit(1);
        }
    }
}
