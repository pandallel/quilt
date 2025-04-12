// Cutting module for processing materials after discovery
// This module contains the CuttingActor and related functionality

pub mod actor;
pub mod cutter;

pub use actor::messages;
pub use actor::CuttingActor;
pub use cutter::{CutterConfig, TextCutter};

#[cfg(test)]
mod tests;
