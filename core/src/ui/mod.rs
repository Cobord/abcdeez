#[cfg(feature = "cli")]
pub mod tui;
#[cfg(feature = "cli")]
pub mod legacy;
#[cfg(feature = "cli")]
pub mod research_dashboard;
pub mod hints;

pub use hints::{
    HintGenerator, HintLevel, InterventionAction, InterventionSystem, StruggleDetector,
    StruggleLevel,
};