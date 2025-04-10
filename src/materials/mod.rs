// Materials module for Quilt
// Defines material types and storage

pub mod types;
pub mod repository;
pub mod registry;

// Re-export material types
pub use registry::*;

#[cfg(test)]
mod tests;

pub use types::{Material, MaterialFileType, MaterialStatus};
// Material registry will be reimplemented for actor model
pub use repository::{MaterialRepository, RepositoryError};
