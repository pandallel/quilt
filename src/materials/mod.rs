// Materials module for Quilt
// Defines material types and storage

use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt::Debug;
use thiserror::Error;

pub mod registry;
pub mod repository;
pub mod sqlite_repository;
pub mod types;

// Re-export material types
pub use registry::*;

#[cfg(test)]
mod tests;

#[cfg(test)]
use mockall::automock;

pub use repository::InMemoryMaterialRepository;
pub use sqlite_repository::SqliteMaterialRepository;
pub use types::{Material, MaterialFileType, MaterialStatus};

/// Errors that can occur during material repository operations
#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Material with id {0} not found")]
    MaterialNotFound(String),

    #[error("Material with id {0} already exists")]
    MaterialAlreadyExists(String),

    #[error("Invalid state transition from {from} to {to}")]
    InvalidStateTransition {
        from: MaterialStatus,
        to: MaterialStatus,
    },
}

/// Result type for material repository operations
pub type Result<T> = std::result::Result<T, RepositoryError>;

/// Repository trait for managing materials
#[cfg_attr(test, automock)]
#[async_trait]
pub trait MaterialRepository: Send + Sync + Debug + 'static {
    /// Register a new material in the repository
    ///
    /// Returns an error if a material with the same ID already exists
    async fn register_material(&self, material: Material) -> Result<()>;

    /// Get a material by its ID
    async fn get_material(&self, id: &str) -> Option<Material>;

    /// Update the status of a material
    ///
    /// Returns an error if the material is not found or the status transition is invalid
    async fn update_material_status(
        &self,
        id: &str,
        new_status: MaterialStatus,
        error_message: Option<String>,
    ) -> Result<()>;

    /// List all materials
    async fn list_materials(&self) -> Vec<Material>;

    /// List materials by status
    async fn list_materials_by_status(&self, status: MaterialStatus) -> Vec<Material>;

    /// Count materials by status
    async fn count_by_status(&self) -> HashMap<MaterialStatus, usize>;
}
