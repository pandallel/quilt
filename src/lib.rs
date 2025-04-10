// Quilt - A local-first, modular memory and context engine
//
// This codebase is being refactored to implement an actor-based architecture.
// Current status: Basic structure with Material types, Directory Scanner, and Material Repository.
// Next steps: Implement the actor framework and message channel system.

pub mod actors;
pub mod cutting;
pub mod discovery;
pub mod events;
pub mod materials;
pub mod orchestrator;

// Re-export the scanner and repository types
pub use discovery::{DirectoryScanner, ScanError, ScanResult, ScanResults};
pub use events::{EventBus, QuiltEvent};
pub use materials::{Material, MaterialFileType, MaterialStatus};
pub use materials::{MaterialRegistry, RegistryError};
pub use materials::{MaterialRepository, RepositoryError};
pub use cutting::CuttingActor;
