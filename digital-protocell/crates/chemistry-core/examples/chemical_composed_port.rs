//! Explicitly prescribed hybrid campaign. Event opportunities are not hazards.
#[cfg(feature = "chemical-reference-port")]
use chemistry_core::chemical_composed_port::{
    ComposedConfig, ComposedWorld, Event, WorldDiagnostics,
};
#[cfg(feature = "chemical-reference-port")]
use serde_json::{json, Value};
#[cfg(feature = "chemical-reference-port")]
use std::io::Write;

#[cfg(feature = "chemical-reference-port")]
fn schedule() -> Vec<(f64, Event)> {
    vec![
        (
            0.05,
            Event::Transport {
                cell: 0,
                species: "Sm".into(),
                direction: "in".into(),
                amount: 0.2,
            },
        ),
        (
            0.10,
            Event::Growth {
                cell: 0,
                edge: 0,
                extent: 0.2,
            },
        ),
        (
            0.15,
            Event::Maturation {
                cell: 0,
                edge: 0,
                extent: 0.04,
            },
        ),
        (0.20, Event::Remesh { cell: 0 }),
        (
            0.30,
            Event::Fission {
                cell: 0,
                i: 0,
                j: 6,
            },
        ),
        (
            0.40,
            Event::Maturation {
                cell: 1,
                edge: 6,
                extent: 0.01,
            },
        ),
        (
            0.40,
            Event::Maturation {
                cell: 2,
                edge: 6,
                extent: 0.01,
            },
        ),
    ]
}
#[cfg(feature = "chemical-reference-port")]
#[derive(Default)]
struct Maxima {
    gradient: f64,
    external: f64,
    peak_native: f64,
    global_drift: f64,
}
#[cfg(feature = "chemical-reference-port")]
fn observe(d: &WorldDiagnostics, initial: &WorldDiagnostics, m: &mut Maxima) {
    for c in &d.cells {
        m.gradient = m.gradient.max(c.gradient_identity_error.abs());
        m.external = m.external.max(
            c.force_components
                .external
                .iter()
                .map(|f| f[0].hypot(f[1]))
                .fold(0., f64::max),
        );
    }
    m.peak_native = m.peak_native.max(d.cells.iter().map(|c| c.U_native).sum());
    let a = &d.global_invariants;
    let b = &initial.global_invariants;
    for x in [
        a.enzyme - b.enzyme,
        a.substrate - b.substrate,
        a.a0 - b.a0,
        a.R_ref - b.R_ref,
        a.membrane - b.membrane,
    ] {
        m.global_drift = m.global_drift.max(x.abs());
    }
}
#[cfg(feature = "chemical-reference-port")]
fn run(config: ComposedConfig, events_enabled: bool, dt: f64) -> Result<Value, String> {
    let mut w = ComposedWorld::prepared(config)?;
    let initial = w.diagnostics()?;
    let mut maxima = Maxima::default();
    observe(&initial, &initial, &mut maxima);
    let mut steps = Vec::new();
    let mut events = Vec::new();
    let prescribed = schedule();
    let count = (0.6 / dt).round() as usize;
    let mut max_increase: f64 = 0.;
    let mut max_defect: f64 = 0.;
    let mut replay = true;
    for i in 0..count {
        let (next, record) = w.try_step(dt)?;
        max_increase = max_increase.max(record.delta_G.max(0.));
        max_defect = max_defect.max(record.continuous_defect.abs());
        observe(&record.after, &initial, &mut maxima);
        w = next;
        steps.push(record);
        if events_enabled {
            for (time, event) in &prescribed {
                if i + 1 == (*time / dt).round() as usize {
                    let (next, record) = w.try_event(event.clone())?;
                    observe(&record.before, &initial, &mut maxima);
                    observe(&record.after, &initial, &mut maxima);
                    w = next;
                    events.push(record);
                }
            }
        }
        if i == count / 2 {
            let restored = ComposedWorld::from_restart_json(&w.restart_json()?, w.config())?;
            replay &=
                w.try_step(dt)?.0.restart_json()? == restored.try_step(dt)?.0.restart_json()?;
        }
    }
    let final_state = w.diagnostics()?;
    let l = &final_state.ledger;
    let residual = final_state.G_total - initial.G_total + l.integrated_Dchem + l.integrated_Dmech
        - l.continuous_signed_defect
        - l.event_delta_G_sum;
    Ok(
        json!({"initial":initial,"steps":steps,"events":events,"final":final_state,"dt":dt,"horizon":0.6,
        "summary":{"continuous_steps":count,"finite_events":events.len(),"final_cells":w.diagnostics()?.cells.len(),
            "integrated_Dchem":l.integrated_Dchem,"integrated_Dmech":l.integrated_Dmech,
            "continuous_signed_defect":l.continuous_signed_defect,"continuous_abs_defect":l.continuous_abs_defect,
            "event_delta_G_sum":l.event_delta_G_sum,"global_ledger_residual":residual,
            "max_continuous_G_increase":max_increase,"max_step_continuous_defect":max_defect,
            "max_gradient_identity_error":maxima.gradient,"max_external_force":maxima.external,
            "peak_native_storage":maxima.peak_native,"max_global_invariant_drift":maxima.global_drift,
            "restart_exact_replay":replay}}),
    )
}
#[cfg(feature = "chemical-reference-port")]
fn rejection_control(name: &str, w: &ComposedWorld, event: Event) -> Result<Value, String> {
    let before = w.diagnostics()?;
    let serialized = w.restart_json()?;
    let attempt = w.try_event(event.clone());
    let error = match attempt {
        Ok(_) => return Err(format!("negative control accepted: {name}")),
        Err(e) => e,
    };
    let after_serialized = w.restart_json()?;
    Ok(
        json!({"name":name,"config":w.config(),"event":event,"before":before,"after":w.diagnostics()?,
        "rejected":true,"unchanged":serialized==after_serialized,"error":error,
        "before_restart":serialized,"after_restart":after_serialized}),
    )
}
#[cfg(feature = "chemical-reference-port")]
fn controls() -> Result<Vec<Value>, String> {
    let prepared = ComposedWorld::prepared(ComposedConfig::default())?;
    let mut evolved = prepared.clone();
    for _ in 0..10 {
        evolved = evolved.try_step(0.001)?.0;
    }
    let mut out = Vec::new();
    for (name, w) in [("prepared", &prepared), ("evolved", &evolved)] {
        out.push(rejection_control(
            &format!("{name}_growth_donor_infeasibility"),
            w,
            Event::Growth {
                cell: 0,
                edge: 0,
                extent: 1000.,
            },
        )?);
        out.push(rejection_control(
            &format!("{name}_transport_donor_infeasibility"),
            w,
            Event::Transport {
                cell: 0,
                species: "Sm".into(),
                direction: "in".into(),
                amount: 1000.,
            },
        )?);
    }
    let split = evolved
        .try_event(Event::Remesh { cell: 0 })?
        .0
        .try_event(Event::Fission {
            cell: 0,
            i: 0,
            j: 6,
        })?
        .0;
    out.push(rejection_control(
        "newborn_absolute_floor_remesh",
        &split,
        Event::Remesh { cell: 1 },
    )?);
    for field in ["config", "ledger"] {
        let raw = evolved.restart_json()?;
        let mut altered: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        if field == "config" {
            altered["data"]["config"]["k"] = json!(9.);
        } else {
            altered["data"]["ledger"]["event_delta_G_sum"] = json!(9.);
        }
        let candidate = altered.to_string();
        let error = match ComposedWorld::from_restart_json(&candidate, evolved.config()) {
            Ok(_) => return Err("tampered restart accepted".into()),
            Err(e) => e,
        };
        let after = evolved.restart_json()?;
        out.push(json!({"name":format!("restart_{field}_tamper"),"config":evolved.config(),"before":evolved.diagnostics()?,"after":evolved.diagnostics()?,
            "attempted_restart":candidate,"before_restart":raw,"after_restart":after,"rejected":true,"unchanged":raw==after,"error":error}));
    }
    let mut fast = ComposedConfig::default();
    fast.gamma = 1e-10;
    let (seed, _) = ComposedWorld::prepared(fast)?.try_step(1e-13)?;
    let raw = seed.restart_json()?;
    let error = match seed.try_step(0.001) {
        Ok(_) => return Err("late geometry negative control accepted".into()),
        Err(e) => e,
    };
    let after = seed.restart_json()?;
    out.push(json!({"name":"late_geometry_trial","config":seed.config(),"dt":0.001,"before":seed.diagnostics()?,"after":seed.diagnostics()?,
        "before_restart":raw,"after_restart":after,"rejected":true,"unchanged":raw==after,"error":error}));
    Ok(out)
}
#[cfg(feature = "chemical-reference-port")]
fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() == 1 && matches!(args[0].as_str(), "--probe" | "--smoke") {
        let config = ComposedConfig::default();
        let mut w = ComposedWorld::prepared(config.clone())?;
        let initial = w.diagnostics()?;
        let mut steps = Vec::new();
        for _ in 0..2 {
            let (next, r) = w.try_step(0.001)?;
            w = next;
            steps.push(r);
        }
        println!("{}",serde_json::to_string(&json!({"mode":"probe_not_campaign","config":config,"initial":initial,"steps":steps,"final":w.diagnostics()?})).map_err(|e|e.to_string())?);
        return Ok(());
    }
    if !args.is_empty() {
        return Err("only --probe/--smoke or the sealed full campaign are supported".into());
    }
    // Stream the single JSON campaign rather than keeping all ten large raw
    // runs in memory. Each completed run remains fully observed and ordered.
    let negative_controls = controls()?;
    let metadata=serde_json::to_string(&json!({"scope":"new_prescribed_native_hybrid_reference_not_autonomous_biology",
        "config":ComposedConfig::default(),"autonomous_generator_qualified":false,"empirical_full_target_qualified":false,
        "whole_organism_qualified":false,"schedule_convention":"continuous joint old-state Euler to each event time; then prescribed events in listed order, never sampled hazards",
        "thermostat_boundary":"finite jumps use full free-energy changes and pair diagnostics; no calorimetric heat or chemical-only funding claim"})).map_err(|e|e.to_string())?;
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    out.write_all(&metadata.as_bytes()[..metadata.len() - 1])
        .map_err(|e| e.to_string())?;
    out.write_all(b",\"cases\":[").map_err(|e| e.to_string())?;
    for (index, name) in [
        "coupled",
        "no_events",
        "zero_source_rates",
        "zero_coupling",
        "reversed_maturation_drive",
    ]
    .into_iter()
    .enumerate()
    {
        let mut cfg = ComposedConfig::default();
        match name {
            "zero_source_rates" => cfg.source_rate_scale = 0.,
            "zero_coupling" => cfg.k = 0.,
            "reversed_maturation_drive" => cfg.mu_m = 0.5,
            _ => {}
        }
        if index != 0 {
            out.write_all(b",").map_err(|e| e.to_string())?;
        }
        let head =
            serde_json::to_string(&json!({"name":name,"config":cfg})).map_err(|e| e.to_string())?;
        out.write_all(&head.as_bytes()[..head.len() - 1])
            .map_err(|e| e.to_string())?;
        out.write_all(b",\"coarse\":").map_err(|e| e.to_string())?;
        let coarse = run(cfg.clone(), name != "no_events", 0.001)?;
        serde_json::to_writer(&mut out, &coarse).map_err(|e| e.to_string())?;
        drop(coarse);
        out.write_all(b",\"fine\":").map_err(|e| e.to_string())?;
        let fine = run(cfg, name != "no_events", 0.0005)?;
        serde_json::to_writer(&mut out, &fine).map_err(|e| e.to_string())?;
        drop(fine);
        out.write_all(b"}").map_err(|e| e.to_string())?;
    }
    out.write_all(b"],\"controls\":")
        .map_err(|e| e.to_string())?;
    serde_json::to_writer(&mut out, &negative_controls).map_err(|e| e.to_string())?;
    out.write_all(b"}\n").map_err(|e| e.to_string())?;
    out.flush().map_err(|e| e.to_string())?;
    Ok(())
}
#[cfg(not(feature = "chemical-reference-port"))]
fn main() {
    eprintln!("enable chemical-reference-port");
    std::process::exit(2);
}
