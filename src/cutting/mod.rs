// Cutting module for processing materials after discovery
// This module contains the CuttingActor and related functionality

pub mod actor;
pub mod cut;
pub mod cutter;
pub mod repository;

pub use actor::messages;
pub use actor::CuttingActor;
pub use cut::Cut;
pub use cutter::{CutterConfig, TextCutter};
pub use repository::{CutsRepository, CutsRepositoryError, InMemoryCutsRepository};

#[cfg(test)]
mod tests;
