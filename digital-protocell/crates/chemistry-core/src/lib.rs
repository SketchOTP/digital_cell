pub mod accounting;
pub mod config;
pub mod diagnostics;
pub mod fields;
pub mod grid;
pub mod interventions;
pub mod operators;
pub mod phase_field;
pub mod reactions;
pub mod reservoir;
pub mod simulation;
pub mod snapshot;

pub use accounting::*;
pub use config::*;
pub use diagnostics::*;
pub use fields::*;
pub use grid::*;
pub use interventions::*;
pub use operators::*;
pub use phase_field::*;
pub use reactions::*;
pub use reservoir::*;
pub use simulation::*;
pub use snapshot::*;

pub const SIM_VERSION: &str = env!("CARGO_PKG_VERSION");
