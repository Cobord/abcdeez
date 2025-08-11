pub mod compliance;
pub mod core;
pub mod data;
pub mod demo;
pub mod experiments;
pub mod learning;
pub mod protocol;
pub mod statistics;
pub mod tasks;

#[cfg(test)]
mod tests;

pub use self::core::{Error, Result};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod prelude {
    pub use crate::core::{Error, Result};
}