//! DC-DEV-020-R9-R2 exact material-fate audit.

mod r8r3 {
    include!("dcdev020r8r3_shared_affinity_helper.rs");

    use chemistry_core::mesh_contracts::{snapshot, MaterialLedgerSnapshot};

    const D016_INVENTORY: f64 = 14.588954880632265;
    const FINITE_STEPS: usize = 480;
    const R9R2_SUSTAINED_STEPS: usize = 8_000;
    const TOL: f64 = 1e-8;

    #[derive(Clone, Debug, Default, Serialize)]
    struct Fate {
        n_delivered: f64,
        f_delivered: f64,
        n_source_injected: f64,
        f_source_injected: f64,
        n_consumed: f64,
        f_consumed: f64,
        a_produced: f64,
        a_to_c: f64,
        c_to_w: f64,
        a_to_m: f64,
        m_to_w: f64,
        a_to_l: f64,
        l_to_b: f64,
        b_to_l: f64,
        a_to_r: f64,
        r_to_a: f64,
        r_to_w: f64,
        a_decay: f64,
        other_material_loss: f64,
    }

    impl Fate {
        fn absorb(&mut self, led: &ReactionLedger) {
            self.n_consumed += led.n_consumed;
            self.f_consumed += led.f_consumed;
            self.a_produced += led.a_produced;
            self.a_to_c += led.c_produced;
            self.c_to_w += led.c_turned;
            self.a_to_m += led.a_consumed_build;
            self.m_to_w += led.m_to_w;
            self.a_to_l += led.l_produced;
            self.l_to_b += led.bind_extent;
            self.b_to_l += led.unbind_extent;
            self.a_to_r += led.reserve.a_to_r;
            self.r_to_a += led.reserve.r_to_a;
            self.r_to_w += led.reserve.r_to_w;
            self.a_decay += led.a_decayed;
            let known_w = led.a_produced + led.c_turned + led.m_to_w + led.a_decayed;
            self.other_material_loss += (led.w_produced - known_w).max(0.0);
        }

        fn loss(&self) -> f64 {
            self.c_to_w + self.m_to_w + self.r_to_w + self.a_decay + self.other_material_loss
        }

        fn boundary(&self) -> f64 {
            self.n_delivered + self.f_delivered + self.n_source_injected + self.f_source_injected
        }
    }

    #[derive(Clone, Debug, Serialize)]
    struct Run {
        context: String,
        mode: String,
        steps: usize,
        initial: MaterialLedgerSnapshot,
        final_state: MaterialLedgerSnapshot,
        fate: Fate,
        strict_material_delta: f64,
        organized_retained_delta: f64,
        activation_delta: f64,
        boundary_material_delta: f64,
        closure_residual: f64,
        organized_reconciliation_residual: f64,
        final_quarter_organized_slope: f64,
        final_quarter_activation_slope: f64,
        final_quarter_waste_slope: f64,
        alive_throughout: bool,
        finite_nonnegative: bool,
        max_checkpoint_closure_residual: f64,
        max_checkpoint_organized_residual: f64,
    }

    fn nonnegative(mesh: &MaterialMesh) -> bool {
        [
            mesh.interior.n,
            mesh.interior.f,
            mesh.interior.a,
            mesh.interior.r,
            mesh.interior.c,
            mesh.interior.w,
            mesh.free_l,
        ]
        .iter()
        .all(|v| v.is_finite() && *v >= -TOL)
    }

    fn make_run(
        context: &str,
        mode: &str,
        initial: MaterialLedgerSnapshot,
        final_state: MaterialLedgerSnapshot,
        fate: Fate,
        frames: &[MaterialLedgerSnapshot],
        alive: bool,
        finite: bool,
        max_closure: f64,
        max_organized: f64,
    ) -> Run {
        let strict =
            final_state.strict_material_equivalent() - initial.strict_material_equivalent();
        let organized = final_state.organized_material() - initial.organized_material();
        let activation = final_state.activation_store() - initial.activation_store();
        let q4 = frames.len() * 3 / 4;
        let slope = |field: fn(&MaterialLedgerSnapshot) -> f64| {
            (field(frames.last().unwrap()) - field(&frames[q4]))
                / (frames.len() - q4 - 1).max(1) as f64
        };
        let reconciliation = (organized - (fate.a_produced - fate.loss())).abs();
        Run {
            context: context.into(),
            mode: mode.into(),
            steps: frames.len() - 1,
            initial,
            final_state,
            fate: fate.clone(),
            strict_material_delta: strict,
            organized_retained_delta: organized,
            activation_delta: activation,
            boundary_material_delta: fate.boundary(),
            closure_residual: (strict - fate.boundary()).abs(),
            organized_reconciliation_residual: reconciliation,
            final_quarter_organized_slope: slope(MaterialLedgerSnapshot::organized_material),
            final_quarter_activation_slope: slope(MaterialLedgerSnapshot::activation_store),
            final_quarter_waste_slope: slope(|s| s.waste),
            alive_throughout: alive,
            finite_nonnegative: finite,
            max_checkpoint_closure_residual: max_closure,
            max_checkpoint_organized_residual: max_organized,
        }
    }

    fn run_finite(start: &MaterialMesh) -> Run {
        let mut mesh = start.clone();
        let initial = snapshot(&mesh);
        let params = reaction_params(&mesh);
        let mut region = FiniteSpatialResourceRegionV1::new(
            RESOURCE_CENTER,
            RESOURCE_RADIUS,
            D016_INVENTORY,
            D016_INVENTORY,
        );
        let mut fate = Fate::default();
        let mut frames = vec![initial];
        let mut alive = true;
        let mut finite = true;
        let mut max_closure: f64 = 0.0;
        let mut max_organized: f64 = 0.0;
        for _ in 0..FINITE_STEPS {
            let before = snapshot(&mesh);
            let uptake = region.uptake(&mut mesh, &TransportParams::default(), DT);
            fate.n_delivered += uptake.n_delivered;
            fate.f_delivered += uptake.f_delivered;
            let led = reactions_step(&mut mesh, &params, DT, true, true);
            fate.absorb(&led);
            let current = snapshot(&mesh);
            let boundary = uptake.n_delivered + uptake.f_delivered;
            max_closure = max_closure.max(
                (current.strict_material_equivalent()
                    - before.strict_material_equivalent()
                    - boundary)
                    .abs(),
            );
            max_organized = max_organized.max(
                (current.organized_material()
                    - initial.organized_material()
                    - (fate.a_produced - fate.loss()))
                .abs(),
            );
            frames.push(current);
            alive &= mesh.alive;
            finite &= nonnegative(&mesh) && uptake.conservation_error.abs() <= TOL;
        }
        make_run(
            "D016 derived break-even finite replay",
            "finite_feed",
            initial,
            snapshot(&mesh),
            fate,
            &frames,
            alive,
            finite,
            max_closure,
            max_organized,
        )
    }

    fn run_sustained_r9r2(start: &MaterialMesh, law: SourceLaw, cprod: bool, context: &str) -> Run {
        let mut mesh = start.clone();
        let initial = snapshot(&mesh);
        let params = reaction_params(&mesh);
        let mut fate = Fate::default();
        let mut frames = vec![initial];
        let mut alive = true;
        let mut finite = true;
        let mut max_closure: f64 = 0.0;
        let mut max_organized: f64 = 0.0;
        for _ in 0..R9R2_SUSTAINED_STEPS {
            let before = snapshot(&mesh);
            mesh.interior.n = SUSTAINED_NF;
            mesh.interior.f = SUSTAINED_NF;
            let sourced = snapshot(&mesh);
            fate.n_source_injected += sourced.n - before.n;
            fate.f_source_injected += sourced.f - before.f;
            let (led, source) = apply_source_mode_r8r2(&mut mesh, &params, law, cprod);
            fate.absorb(&led);
            let current = snapshot(&mesh);
            let boundary = (sourced.n - before.n) + (sourced.f - before.f);
            max_closure = max_closure.max(
                (current.strict_material_equivalent()
                    - before.strict_material_equivalent()
                    - boundary)
                    .abs(),
            );
            max_closure = max_closure.max(source.accounting_residual.abs());
            max_organized = max_organized.max(
                (current.organized_material()
                    - initial.organized_material()
                    - (fate.a_produced - fate.loss()))
                .abs(),
            );
            frames.push(current);
            alive &= mesh.alive;
            finite &= nonnegative(&mesh);
        }
        make_run(
            context,
            if cprod { "normal" } else { "cprod_deferred" },
            initial,
            snapshot(&mesh),
            fate,
            &frames,
            alive,
            finite,
            max_closure,
            max_organized,
        )
    }

    fn certifier_pass(v: &Value) -> bool {
        (0..=7).all(|i| v[&format!("gate{i}")]["pass"].as_bool() == Some(true))
    }

    fn exact_replay_pass(v: &Value) -> bool {
        v["exact_correspondence"]["contract"].as_str()
            == Some("MeshContractVersion::ConservativeV2")
            && v["exact_correspondence"]["equation_lineage"].as_str()
                == Some("autopoietic_material_mesh_metabolic_reserve_v1")
            && v["rows"].as_array().is_some_and(|rows| {
                rows.len() == 7
                    && rows.iter().all(|row| {
                        row["contract_version"].as_str() == Some("ConservativeV2")
                            && row["result"]["closure_residual"]
                                .as_f64()
                                .is_some_and(|v| v <= TOL)
                            && row["result"]["reserve_rejected_steps"].as_u64() == Some(0)
                    })
            })
    }

    fn route(run: &Run) -> (&'static str, f64) {
        [
            ("CATALYST_DOMINANT", run.fate.c_to_w),
            ("STRUCTURE_DOMINANT", run.fate.m_to_w),
            ("RESERVE_DOMINANT", run.fate.r_to_w),
            ("A_DECAY_DOMINANT", run.fate.a_decay),
            ("DISTRIBUTED_LOSS", run.fate.other_material_loss),
        ]
        .into_iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .unwrap()
    }

    pub fn run() -> Result<(), String> {
        if !conservative_v2_enabled() {
            return Err("DCDEV020R9R2_V2=1 is required".into());
        }
        let out = std::env::var_os("DCDEV020R9R2_OUTPUT")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020r9r2"));
        fs::create_dir_all(&out).map_err(|e| e.to_string())?;
        let cert_path = std::env::var_os("DCDEV020R9R2_CERTIFIER_REPORT")
            .map(PathBuf::from)
            .unwrap_or_else(|| out.join("actual_d087/manifest.json"));
        let cert: Value = serde_json::from_str(
            &fs::read_to_string(&cert_path)
                .map_err(|e| format!("actual certifier missing: {e}"))?,
        )
        .map_err(|e| format!("actual certifier JSON invalid: {e}"))?;
        let replay_path = std::env::var_os("DCDEV020R9R2_REPLAY_REPORT")
            .map(PathBuf::from)
            .unwrap_or_else(|| out.join("r9r1_exact/manifest.json"));
        let replay: Value = serde_json::from_str(
            &fs::read_to_string(&replay_path)
                .map_err(|e| format!("exact D-015/D-016 replay missing: {e}"))?,
        )
        .map_err(|e| format!("exact replay JSON invalid: {e}"))?;
        let replay_ok = exact_replay_pass(&replay);
        let settled = settle();
        let deprived = deprive(&settled);
        let finite = run_finite(&deprived);
        let d016_normal =
            run_sustained_r9r2(&deprived, SourceLaw::Baseline, true, "D016 bilinear source");
        let d016_deferred = run_sustained_r9r2(
            &deprived,
            SourceLaw::Baseline,
            false,
            "D016 bilinear source",
        );
        let r6 = SourceLaw::PowerLaw(PowerLaw {
            k_pl: R6_K_PL_R8R2,
            p: R6_POWER_P_R8R2,
            g_h: 1.0,
        });
        let r6_normal = run_sustained_r9r2(&deprived, r6, true, "sealed R6 NF power-law source");
        let r6_deferred = run_sustained_r9r2(&deprived, r6, false, "sealed R6 NF power-law source");
        let (next_route, next_loss) = route(&finite);
        let total_loss = finite.fate.loss();
        let sustained_decline = [&d016_normal, &d016_deferred, &r6_normal, &r6_deferred]
            .iter()
            .all(|r| r.final_quarter_organized_slope < 0.0);
        let exact_material = certifier_pass(&cert)
            && replay_ok
            && finite.activation_delta >= 0.0
            && finite.organized_retained_delta < 0.0
            && total_loss > finite.fate.a_produced
            && finite.organized_reconciliation_residual <= TOL
            && finite.closure_residual <= TOL
            && sustained_decline;
        let classification = if !certifier_pass(&cert) || !replay_ok {
            "DCDEV020R9R2_CONSERVATIVE_CERTIFICATION_REGRESSION"
        } else if exact_material {
            "DCDEV020R9R2_TRUE_MATERIAL_CYCLE_DEFICIT_CONFIRMED"
        } else if finite.activation_delta < 0.0 && finite.organized_retained_delta < 0.0 {
            "DCDEV020R9R2_MATERIAL_AND_ACTIVATION_DEFICIT_MIXED"
        } else if finite.organized_retained_delta >= 0.0 {
            "DCDEV020R9R2_METRIC_CONFOUNDING_DOMINANT_CONFIRMED"
        } else {
            "DCDEV020R9R2_TRUE_ACTIVATION_DEFICIT_CONFIRMED"
        };
        let protocol = json!({
            "directive": "DC-DEV-020-R9-R2",
            "entry_head": "364599aea8d4a0def3964b1b299fe45edaaaa1b3",
            "classification_authority": "EXACT_PROTOCOL_REPLAYS",
            "exact_replay_report": replay_path,
            "contract": "MeshContractVersion::ConservativeV2",
            "equation_lineage": "autopoietic_material_mesh_metabolic_reserve_v1",
            "finite_d016_steps": FINITE_STEPS,
            "sustained_steps": R9R2_SUSTAINED_STEPS,
            "source_contexts": ["D016 bilinear source", "sealed R6 NF power-law source"],
            "normal": "frozen catalyst production and turnover",
            "cprod_deferred": "catalyst production zero, frozen turnover retained",
            "production_chemistry_changed": false,
            "production_behavior_changed": false,
            "dc_dev_021_authorized": false
        });
        let evidence = json!({
            "finite_d016": finite,
            "sustained": {"d016_normal": d016_normal, "d016_cprod_deferred": d016_deferred, "r6_normal": r6_normal, "r6_cprod_deferred": r6_deferred},
            "dominant_irreversible_loss": next_route,
            "dominant_loss_value": next_loss,
            "dominant_loss_fraction": next_loss / total_loss.max(1e-15),
            "material_loss_map": {"catalyst_turnover": finite.fate.c_to_w, "structure_turnover": finite.fate.m_to_w, "reserve_loss": finite.fate.r_to_w, "a_decay": finite.fate.a_decay, "other": finite.fate.other_material_loss}
        });
        let qualification = json!({
            "classification_authority": "EXACT_PROTOCOL_REPLAYS",
            "actual_d087_all_pass": certifier_pass(&cert),
            "exact_d015_d016_replay_pass": replay_ok,
            "actual_d087_gate_pass": (0..=7).map(|i| cert[&format!("gate{i}")]["pass"].clone()).collect::<Vec<_>>(),
            "exact_d016_activation_nonnegative": finite.activation_delta >= 0.0,
            "exact_d016_organized_negative": finite.organized_retained_delta < 0.0,
            "exact_d016_fate_reconciles": finite.organized_reconciliation_residual <= TOL,
            "exact_d016_strict_closure": finite.closure_residual <= TOL,
            "sustained_organized_decline_all_arms": sustained_decline,
            "primary_classification": classification,
            "next_loss_route": next_route,
            "production_chemistry_changed": false,
            "production_behavior_changed": false,
            "dc_dev_021_authorized": false,
            "architect_acceptance": "PENDING",
            "next_execution_started": false
        });
        for (name, value) in [
            ("protocol.json", protocol),
            ("material_fate.json", evidence),
            ("qualification.json", qualification),
            ("actual_d087_conservative.json", cert),
        ] {
            fs::write(out.join(name), serde_json::to_vec_pretty(&value).unwrap())
                .map_err(|e| e.to_string())?;
        }
        let manifest = json!({
            "directive": "DC-DEV-020-R9-R2",
            "classification": classification,
            "classification_authority": "EXACT_PROTOCOL_REPLAYS",
            "source_commit": "364599aea8d4a0def3964b1b299fe45edaaaa1b3",
            "actual_certifier_report": cert_path.display().to_string(),
            "next_loss_route": next_route,
            "production_chemistry_changed": false,
            "production_behavior_changed": false,
            "dc_dev_021_authorized": false,
            "next_execution_started": false
        });
        fs::write(
            out.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .map_err(|e| e.to_string())?;
        println!("DCDEV020R9R2_MATERIAL_FATE_AUDIT_COMPLETE");
        println!("classification={classification}");
        println!("next_loss_route={next_route}");
        println!("next_execution_started=false");
        Ok(())
    }
}

fn main() {
    if let Err(error) = r8r3::run() {
        eprintln!("DCDEV020R9R2_FAIL_CLOSED: {error}");
        std::process::exit(1);
    }
}
