// Cutting module for processing materials after discovery
// This module contains the CuttingActor and related functionality

pub mod actor;

pub use actor::CuttingActor;
pub use actor::messages;

#[cfg(test)]
mod tests; 