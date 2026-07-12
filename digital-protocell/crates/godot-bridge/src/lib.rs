use godot::prelude::*;
use chemistry_core::{baseline_params, total_mass, Grid, Simulation, ViabilityClass};

struct ChemistryExtension;

#[gdextension]
unsafe impl ExtensionLibrary for ChemistryExtension {}

#[derive(GodotClass)]
#[class(base=Node)]
struct ChemistrySimulator {
    base: Base<Node>,
    sim: Option<Simulation>,
    paused: bool,
    speed: f64,
    steps_per_frame: u32,
}

#[godot_api]
impl INode for ChemistrySimulator {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            sim: Some(Simulation::new(baseline_params())),
            paused: false,
            speed: 1.0,
            steps_per_frame: 2,
        }
    }

    fn process(&mut self, delta: f64) {
        if self.paused {
            return;
        }
        let steps = ((delta * 60.0 * self.speed) as u32).max(1) * self.steps_per_frame;
        if let Some(sim) = &mut self.sim {
            for _ in 0..steps {
                sim.step();
            }
        }
    }
}

#[godot_api]
impl ChemistrySimulator {
    #[func]
    fn pause_sim(&mut self) {
        self.paused = true;
    }

    #[func]
    fn resume_sim(&mut self) {
        self.paused = false;
    }

    #[func]
    fn single_substep(&mut self) {
        if let Some(sim) = &mut self.sim {
            sim.step();
        }
    }

    #[func]
    fn set_speed(&mut self, speed: f64) {
        self.speed = speed.max(0.0);
    }

    #[func]
    fn reset_experiment(&mut self) {
        self.sim = Some(Simulation::new(baseline_params()));
    }

    #[func]
    fn get_grid_width(&self) -> i64 {
        Grid::new().width as i64
    }

    #[func]
    fn get_grid_height(&self) -> i64 {
        Grid::new().height as i64
    }

    #[func]
    fn get_structure_at(&self, x: i32, y: i32) -> f64 {
        let Some(sim) = self.sim.as_ref() else {
            return 0.0;
        };
        let w = sim.grid.width;
        if x < 0 || y < 0 || x as usize >= w || y as usize >= sim.grid.height {
            return 0.0;
        }
        let idx = Grid::index(w, x as usize, y as usize);
        if sim.grid.in_dish(idx) {
            sim.fields.structure[idx]
        } else {
            0.0
        }
    }

    #[func]
    fn get_catalyst_at(&self, x: i32, y: i32) -> f64 {
        let Some(sim) = self.sim.as_ref() else {
            return 0.0;
        };
        let w = sim.grid.width;
        if x < 0 || y < 0 || x as usize >= w || y as usize >= sim.grid.height {
            return 0.0;
        }
        let idx = Grid::index(w, x as usize, y as usize);
        if sim.grid.in_dish(idx) {
            sim.fields.catalyst[idx]
        } else {
            0.0
        }
    }

    #[func]
    fn get_waste_at(&self, x: i32, y: i32) -> f64 {
        let Some(sim) = self.sim.as_ref() else {
            return 0.0;
        };
        let w = sim.grid.width;
        if x < 0 || y < 0 || x as usize >= w || y as usize >= sim.grid.height {
            return 0.0;
        }
        let idx = Grid::index(w, x as usize, y as usize);
        if sim.grid.in_dish(idx) {
            sim.fields.waste[idx]
        } else {
            0.0
        }
    }

    #[func]
    fn get_sim_time(&self) -> f64 {
        self.sim.as_ref().map(|s| s.sim_time).unwrap_or(0.0)
    }

    #[func]
    fn get_dt(&self) -> f64 {
        self.sim.as_ref().map(|s| s.dt).unwrap_or(0.0)
    }

    #[func]
    fn get_structural_mass(&self) -> f64 {
        self.sim
            .as_ref()
            .map(|s| total_mass(&s.grid, &s.fields.structure))
            .unwrap_or(0.0)
    }

    #[func]
    fn get_catalyst_mass(&self) -> f64 {
        self.sim
            .as_ref()
            .map(|s| total_mass(&s.grid, &s.fields.catalyst))
            .unwrap_or(0.0)
    }

    #[func]
    fn get_classification(&self) -> GString {
        let class = self
            .sim
            .as_ref()
            .map(|s| s.detector.last_classification)
            .unwrap_or(ViabilityClass::Seeding);
        GString::from(format!("{class:?}"))
    }

    #[func]
    fn run_puncture(&mut self) {
        if let Some(sim) = &mut self.sim {
            chemistry_core::apply_intervention(
                &sim.grid,
                &mut sim.fields,
                &chemistry_core::InterventionAction::PunctureRepair,
                &mut sim.params,
            );
        }
    }

    #[func]
    fn remove_nutrient(&mut self) {
        if let Some(sim) = &mut self.sim {
            sim.params.n_reservoir = 0.0;
        }
    }

    #[func]
    fn remove_fuel(&mut self) {
        if let Some(sim) = &mut self.sim {
            sim.params.f_reservoir = 0.0;
        }
    }

    #[func]
    fn disable_catalyst_reproduction(&mut self) {
        if let Some(sim) = &mut self.sim {
            sim.params.k_rep = 0.0;
        }
    }

    #[func]
    fn restore_reservoir(&mut self) {
        if let Some(sim) = &mut self.sim {
            sim.params.n_reservoir = 1.0;
            sim.params.f_reservoir = 1.0;
            sim.params.w_reservoir = 0.0;
        }
    }

    #[func]
    fn save_snapshot(&self, path: GString) -> bool {
        if let Some(sim) = self.sim.as_ref() {
            let snap = sim.snapshot();
            chemistry_core::save_snapshot(std::path::Path::new(&path.to_string()), &snap).is_ok()
        } else {
            false
        }
    }
}
