// Cutting module for processing materials after discovery
// This module contains the CuttingActor and related functionality

pub mod actor;

pub use actor::messages;
pub use actor::CuttingActor;

#[cfg(test)]
mod tests;
