// Quilt - A local-first, modular memory and context engine
//
// This codebase is being refactored to implement an actor-based architecture.
// Current status: Basic structure with Material types, Directory Scanner, and Material Repository.
// Next steps: Implement the actor framework and message channel system.

pub mod actors;
pub mod cutting;
pub mod db;
pub mod discovery;
pub mod events;
pub mod materials;
pub mod orchestrator;
pub mod swatching;

// Re-export the core types for users of the library
// Material discovery and processing types
pub use discovery::{DirectoryScanner, ScanError, ScanResult, ScanResults};

// Event system types
pub use events::{EventBus, QuiltEvent};

// Material types and repository
pub use materials::{InMemoryMaterialRepository, RepositoryError, SqliteMaterialRepository};
pub use materials::{Material, MaterialFileType, MaterialStatus};
pub use materials::{MaterialRegistry, RegistryError};

// Cutting and processing types
pub use cutting::{Cut, CutsRepository, CutsRepositoryError, InMemoryCutsRepository};

// Swatching types
pub use swatching::{SwatchingActor, SwatchingError};

// Database functionality
pub use db::init_memory_db;
