// Materials module for Quilt
// Defines material types and storage

pub mod cut;
pub mod cuts_repository;
pub mod registry;
pub mod repository;
pub mod types;

// Re-export material types
pub use registry::*;

#[cfg(test)]
mod tests;

pub use types::{Material, MaterialFileType, MaterialStatus};
// Material registry will be reimplemented for actor model
pub use cut::Cut;
pub use cuts_repository::{CutsRepository, CutsRepositoryError, InMemoryCutsRepository};
pub use repository::{MaterialRepository, RepositoryError};
