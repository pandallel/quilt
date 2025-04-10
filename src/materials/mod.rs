// Materials module for Quilt
// Defines material types and storage

pub mod registry;
pub mod repository;
pub mod types;

// Re-export material types
pub use registry::*;

#[cfg(test)]
mod tests;

pub use types::{Material, MaterialFileType, MaterialStatus};
// Material registry will be reimplemented for actor model
pub use repository::{MaterialRepository, RepositoryError};
