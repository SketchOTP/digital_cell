//! DC-DEV-020-R8-R5-R1 corrected net A/C allocation drift observer.

#[path = "dcdev020r8r5_ac_allocation_upper_bound.rs"]
mod r8r5;

fn main() {
    r8r5::r8r3::run_r1();
}
