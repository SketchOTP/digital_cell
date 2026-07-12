use chemistry_core::*;

fn main() {
    let mut sim = Simulation::new(baseline_params());
    let m0 = total_mass(&sim.grid, &sim.fields.structure);
    let c0 = total_mass(&sim.grid, &sim.fields.catalyst);
    println!("initial structure={m0:.2} catalyst={c0:.2}");
    for _ in 0..5000 {
        sim.step();
    }
    let m1 = total_mass(&sim.grid, &sim.fields.structure);
    let c1 = total_mass(&sim.grid, &sim.fields.catalyst);
    println!("after 5000 structure={m1:.2} catalyst={c1:.2}");
    let mut p = baseline_params();
    p.k_structure = 0.0;
    let mut sim2 = Simulation::new(p);
    let m2_0 = total_mass(&sim2.grid, &sim2.fields.structure);
    for _ in 0..5000 {
        sim2.step();
    }
    let m2_1 = total_mass(&sim2.grid, &sim2.fields.structure);
    println!("knockout initial={m2_0:.2} final={m2_1:.2}");
}
