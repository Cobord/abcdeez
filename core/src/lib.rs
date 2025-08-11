pub mod core;
pub mod learning;
pub mod tasks;
pub mod statistics;
pub mod experiments;
pub mod compliance;
pub mod protocol;
pub mod data;
pub mod demo;

#[cfg(test)]
mod tests;

pub use self::{
    compliance::*,
    core::*,
    data::*,
    experiments::*,
    learning::*,
    protocol::*,
    statistics::*,
    tasks::*,
};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod prelude {
    pub use crate::{
        core::{Topology, TopologyType},
        learning::{AdaptiveScheduler, LearnerMetrics, LearnerModel},
        tasks::{MusicStructure, MusicTheory, Task, TaskGenerator, TaskType},
    };
}