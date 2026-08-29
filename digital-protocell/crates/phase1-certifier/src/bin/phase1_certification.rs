//! Direct launcher for the existing D-087 Gates 0–7 certifier.

use phase1_certifier::campaign::run_certification;
use std::env;
use std::path::PathBuf;

fn main() {
    let mut output = PathBuf::from("experiments/generated/dcdev020r9r2/actual_d087");
    let args: Vec<String> = env::args().collect();
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--output" {
            i += 1;
            if let Some(path) = args.get(i) {
                output = PathBuf::from(path);
            }
        }
        i += 1;
    }
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let repo_root = if cwd
        .join("digital-protocell/crates/phase1-certifier")
        .exists()
    {
        cwd
    } else if cwd.join("crates/phase1-certifier").exists() {
        cwd.parent().unwrap_or(&cwd).to_path_buf()
    } else {
        cwd
    };
    match run_certification(&repo_root, &output) {
        Ok(report) => {
            println!(
                "D087_ACTUAL_CERTIFIER_COMPLETE contract={} reserve={} gates={}/8 conclusion={} output={}",
                report.mesh_contract,
                report.reserve_enabled,
                [
                    &report.gate0,
                    &report.gate1,
                    &report.gate2,
                    &report.gate3,
                    &report.gate4,
                    &report.gate5,
                    &report.gate6,
                    &report.gate7,
                ]
                .iter()
                .filter(|gate| gate.pass)
                .count(),
                report.primary_conclusion,
                output.display()
            );
        }
        Err(error) => {
            eprintln!("D087_ACTUAL_CERTIFIER_FAIL_CLOSED: {error}");
            std::process::exit(1);
        }
    }
}
