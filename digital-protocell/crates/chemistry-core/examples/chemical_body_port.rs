//! Deterministic sealed body/remesh campaign, pure JSON stdout.
#[cfg(feature = "chemical-reference-port")]
use chemistry_core::chemical_body_port::{BodyConfig, ChemicalBodyState, SourcePreparation};
#[cfg(feature = "chemical-reference-port")]
use serde_json::{json, Value};

#[cfg(feature = "chemical-reference-port")]
fn run(config: BodyConfig, dt: f64) -> Result<Value, String> {
    let mut s = ChemicalBodyState::cold(config)?;
    let initial = s.diagnostics()?;
    let mut trajectory = Vec::new();
    let mut remesh = None;
    let mut max_energy_increase: f64 = 0.;
    let mut max_step_defect: f64 = 0.;
    let mut peak_us: f64 = 0.;
    let mut max_identity: f64 = initial.gradient_identity_error.abs();
    let mut max_omitted: f64 = 0.;
    let mut source_drift: f64 = 0.;
    let mut restart_equivalent = true;
    let mut coupling_work = 0.;
    let mut osmotic_work = 0.;
    let mut native_receiver_work = 0.;
    let count = (2. / dt).round() as usize;
    let split_step = (0.2 / dt).round() as usize;
    for i in 0..count {
        let old = s.diagnostics()?;
        let (next, r) = s.try_step(dt)?;
        max_energy_increase =
            max_energy_increase.max((r.diagnostics.total_g - old.total_g).max(0.));
        max_step_defect = max_step_defect.max(r.step_defect.abs());
        peak_us = peak_us.max(r.diagnostics.us);
        max_identity = max_identity.max(r.diagnostics.gradient_identity_error.abs());
        max_omitted = max_omitted.max(r.diagnostics.omitted_osmotic_identity_error.abs());
        let n = r.diagnostics.amounts;
        source_drift = source_drift
            .max((n[0] + n[1] + n[2] - initial.p).abs())
            .max((n[4] + n[5] + n[1] + 2. * n[2] + 2. * n[3] - 1.27).abs());
        coupling_work += r.coupling_work;
        osmotic_work += r.osmotic_work;
        native_receiver_work += r.native_receiver_work;
        s = next;
        trajectory.push(r);
        if i + 1 == split_step {
            let (next, event) = s.try_remesh_split()?;
            s = next;
            remesh = Some(event);
        }
        if i == count / 2 {
            let restored = ChemicalBodyState::from_restart_json(&s.restart_json()?, s.config())?;
            restart_equivalent &=
                s.try_step(dt)?.0.restart_json()? == restored.try_step(dt)?.0.restart_json()?;
        }
    }
    let final_state = s.diagnostics()?;
    let last = trajectory.last().ok_or("empty trajectory")?;
    Ok(
        json!({"dt":dt,"horizon":2.,"initial":initial,"final":final_state,
        "trajectory":trajectory,"remesh":remesh,"summary":{
            "accepted_steps":count,"max_energy_increase":max_energy_increase,"max_abs_step_defect":max_step_defect,
            "signed_step_defect":last.signed_step_defect,"absolute_step_defect":last.absolute_step_defect,
            "remesh_defect":last.remesh_defect,"integrated_dchem":last.integrated_dchem,"integrated_dmech":last.integrated_dmech,
            "peak_us":peak_us,"max_gradient_identity_error":max_identity,"max_omitted_osmotic_identity_error":max_omitted,
            "source_conservation_error":source_drift,"restart_equivalent":restart_equivalent,
            "coupling_work":coupling_work,"osmotic_work":osmotic_work,"native_receiver_work":native_receiver_work}}),
    )
}

#[cfg(feature = "chemical-reference-port")]
fn counterexamples() -> Value {
    use chemistry_core::material_mesh::{LumpedChem, MaterialMesh};
    use chemistry_core::mesh_mechanics::remesh_merge;
    let mut m = MaterialMesh::seed_regular(
        7,
        1.,
        0.,
        0.,
        1.,
        0.7,
        LumpedChem::default(),
        LumpedChem::default(),
        0.,
    );
    m.stamp_maturation_coupled_schema();
    m.vertices = vec![
        [0., 0.],
        [1., 0.],
        [1., 1.],
        [0.5, 2.],
        [-0.5, 2.],
        [-1., 1.],
        [-1., 0.],
    ];
    for i in 0..7 {
        m.edges[i].m = m.edge_length(i);
        m.edges[i].m_young = 0.;
    }
    m.l_min = 1.1;
    let storage = |m: &MaterialMesh| -> f64 {
        (0..m.n())
            .map(|i| (m.edge_length(i) - m.rest_length(i)).powi(2) / (2. * m.rest_length(i)))
            .sum()
    };
    let before = storage(&m);
    let vertices_before = m.vertices.clone();
    let rest_before: Vec<f64> = (0..m.n()).map(|i| m.rest_length(i)).collect();
    let merges = remesh_merge(&mut m);
    let after = storage(&m);
    json!({"curved_merge":{"kind":"externally_prepared_actual_native_remesh_merge_diagnostic_not_trajectory",
        "native_merges":merges,"vertices_before":vertices_before,"rest_lengths_before":rest_before,
        "vertices_after":m.vertices,"rest_lengths_after":(0..m.n()).map(|i|m.rest_length(i)).collect::<Vec<_>>(),
        "us_before":before,"us_after":after,"unpaid_storage_increase":after-before},
        "maturation":{"kind":"local_storage_witness_for_native_maturation_assignment_not_full_reaction_step",
            "L":1.,"total_mass":2.,"mature_before":1.,"mature_after":1.1,"rho_s":1.,"k_s":1.,
            "us_before":0.,"us_after":(1.0_f64-1.1).powi(2)/(2.*1.1),
            "boundary":"native material bookkeeping does not identify chemical free energy for this work"}})
}

#[cfg(feature = "chemical-reference-port")]
fn main() -> Result<(), String> {
    if std::env::args().len() != 1 {
        return Err("sealed campaign accepts no arguments".into());
    }
    let mut runs = serde_json::Map::new();
    for name in [
        "coupled",
        "zero_lambda",
        "zero_chemistry",
        "closed_equilibrium",
    ] {
        let mut c = BodyConfig::default();
        match name {
            "zero_lambda" => c.lambda = 0.,
            "zero_chemistry" => c.rate_scale = 0.,
            "closed_equilibrium" => c.preparation = SourcePreparation::ClosedEquilibrium,
            _ => {}
        }
        runs.insert(
            name.into(),
            json!({"coarse":run(c.clone(),0.002)?,"fine":run(c,0.001)?}),
        );
    }
    let output = json!({"scope":"new_computational_full_state_finite_volume_body_and_native_remesh_only",
        "whole_organism_qualified":false,"empirical_full_target_qualified":false,
        "design_truth":BodyConfig::default(),"runs":runs,"counterexamples":counterexamples()});
    println!(
        "{}",
        serde_json::to_string(&output).map_err(|e| e.to_string())?
    );
    Ok(())
}
#[cfg(not(feature = "chemical-reference-port"))]
fn main() {
    eprintln!("enable chemical-reference-port");
    std::process::exit(2);
}
