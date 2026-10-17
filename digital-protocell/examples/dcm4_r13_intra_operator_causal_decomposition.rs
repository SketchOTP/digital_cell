//! R13 discarded-clone geometry-to-polarity internal causal decomposition.
#![recursion_limit = "512"]
#[path = "dcfinal001_r4_evolution.rs"]
mod current_population;
use serde_json::{json, Value};
use std::{env, fs, path::Path};
const DIRECTIVE: &str =
    "DC-M4-R13-GEOMETRY-TO-POLARITY-INTRA-OPERATOR-CAUSAL-DECOMPOSITION-GATE-001";
fn contract() -> Value {
    json!({
        "directive": DIRECTIVE,
        "entry_head": "d944d4fabc46333203bfd6e441d600b9717c349d",
        "entry_ci": "35060451335",
        "entry_r8": {"head":"18cd5daff29267469db838acbba9bf9691e94ba6","ci":"34865782025","artifact":"sha256:30b75d9389adda97bde60654d00caf8b86b80c4ff0998efc52d43764b9303073"},
        "entry_r9": {"head":"00a7a26f1c5e23b8b9895de7ba1c71c51b2dbbc4","ci":"34914837619","artifact":"sha256:d379b78bb56cfce3ca5ba9986e8007869d6961bd6679da0cdf3b2d971fff9bfe"},
        "entry_artifact": "sha256:5949df1110598142f6c32f3492a06f9d39516e18fa016dead6d4223dd2c3e432",
        "entry_binary": "cec057e369ce03bff0720b35245372d53e86e59d0af9fadf7b3c57f1f45fda64",
        "checkpoints": [3694, 7389, 11083], "histories": 10, "states": 30,
        "horizon": 14778,
        "normalized_amplitudes": [0.001, 0.0005], "signs": [1.0, -1.0],
        "basis": "exact r8_normal_harmonic_basis; rigid translation and rotation removed",
        "paths": ["FULL", "REACTION_ONLY", "TRANSPORT_ONLY", "REACTION_THEN_TRANSPORT", "TRANSPORT_THEN_REACTION"],
        "production_order": "four substeps; conservative lateral diffusion then local reaction per substep",
        "dt": 0.01, "integration_substeps": 4,
        "cuts": "unchanged native advance on clone parameters: diffusion=0 or all four reaction rates=0; same perturbed cohort and A budget for actual/baseline measures",
        "order_probes": "four h=0.0025 pairs of one-substep cut advances; artificial accepted-step counters discarded",
        "e1": "fixed persistent amounts divided by actual versus baseline positive edge measure; separate from E2 amount response",
        "response": "active amount(actual measures) minus active amount(baseline measures)",
        "interaction": "FULL - REACTION_ONLY - TRANSPORT_ONLY + IDENTITY; IDENTITY amount response is exactly zero",
        "r8_sign": "unchanged dominant R4 activity harmonic; plus-sign epsilon active-amount DFT projected onto activity DFT; average harmonic cos/sin modes",
        "r8_sign_epsilon": 1e-12,
        "controls": {
            "replay_absolute_tolerance": 1e-12,
            "replay_relative_tolerance": 1e-10,
            "r8_symmetry_relative_limit": 0.05,
            "roundoff_bound": "sqrt(response_dimension)*replay_absolute_tolerance + replay_relative_tolerance*max(input_amount_norm, response_norm)",
            "vector_envelope": "max(norm((plus+minus)/2), norm(central_epsilon-2*central_half), replay_roundoff); concatenate every sealed mode",
            "interaction_envelope": "sum FULL, REACTION_ONLY and TRANSPORT_ONLY envelopes",
            "phase_envelope": "same sign-even, half-scale and replay errors projected by exact R8 phase functional; per-path",
            "projection_direction_error": "candidate_error + norm(candidate)*2*FULL_error/(norm(FULL)-FULL_error); FULL norm<=FULL error is inconclusive",
            "branch_masks": "complete per-substep per-edge donor-limit and reaction-clip masks must match actual versus cut; counts alone insufficient",
        },
        "attribution": {
            "usable_minimum": 24,
            "candidate_order": ["REACTION_ONLY", "TRANSPORT_ONLY", "INTERACTION"],
            "dominant": "positive projection onto FULL; lower-bound signed projection > sum upper absolute projections of other components; uniquely lowest residual to FULL beyond combined error bounds; candidate R8 phase indicator damping beyond own envelope",
            "metrics": ["full_vector_correlation", "signed_projection", "gain", "normalized_residual"],
            "complete_explanation_required": false,
            "mixed": "at least24 unique per-state nominations across different routes, no route reaches24",
            "otherwise": "G_TO_P_INTERNAL_ATTRIBUTION_INCONCLUSIVE",
            "outcome_selected_settings": false,
        },
        "production_biology_modified": false, "observer_only": true,
        "morphogenesis": "NOT_REACHED", "reproduction": "NOT_REACHED",
        "population_selection": "NOT_REACHED", "reversal": "NOT_REACHED", "final_integration": "NOT_REACHED",
    })
}
fn main() {
    let mut args = env::args().skip(1);
    let mut stage = "seal".to_string();
    let mut output = "/tmp/dcm4_r13_seal.json".to_string();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--stage" => stage = args.next().expect("stage"),
            "--output" => output = args.next().expect("output"),
            other => panic!("unknown {other}"),
        }
    }
    let mut value = contract();
    match stage.as_str() {
        "seal" => {}
        "qualification" => {
            let arms = (0..10)
                .map(current_population::run_r13_intra_operator_arm)
                .collect::<Vec<_>>();
            value["arms"] = json!(arms);
        }
        other => panic!("unknown stage {other}"),
    }
    if let Some(parent) = Path::new(&output).parent() {
        fs::create_dir_all(parent).expect("directory");
    }
    fs::write(output, serde_json::to_vec_pretty(&value).expect("JSON")).expect("write");
}
