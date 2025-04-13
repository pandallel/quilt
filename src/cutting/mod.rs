// Cutting module for processing materials after discovery
// This module contains the CuttingActor and related functionality

use std::fmt::Debug;
use thiserror::Error;
use time::OffsetDateTime;

use async_trait::async_trait;

pub mod actor;
pub mod cut;
pub mod cutter;
pub mod repository;
pub mod sqlite_repository;

pub use actor::messages;
pub use actor::CuttingActor;
pub use cut::Cut;
pub use cutter::{CutterConfig, TextCutter};
pub use repository::InMemoryCutsRepository;
pub use sqlite_repository::SqliteCutsRepository;

#[cfg(test)]
mod tests;

/// Errors that can occur during cuts repository operations
#[derive(Error, Debug)]
pub enum CutsRepositoryError {
    #[error("Cut with id {0} not found")]
    CutNotFound(String),

    #[error("Cut with id {0} already exists")]
    CutAlreadyExists(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

/// Result type for cuts repository operations
pub type Result<T> = std::result::Result<T, CutsRepositoryError>;

/// Repository trait for managing cuts
#[async_trait]
pub trait CutsRepository: Send + Sync + Debug + 'static {
    /// Save a cut to the repository
    async fn save_cut(&self, cut: &Cut) -> Result<()>;

    /// Save multiple cuts in a batch operation
    async fn save_cuts(&self, cuts: &[Cut]) -> Result<()>;

    /// Get a cut by its ID
    async fn get_cut_by_id(&self, cut_id: &str) -> Result<Option<Cut>>;

    /// Get all cuts for a specific material
    async fn get_cuts_by_material_id(&self, material_id: &str) -> Result<Vec<Cut>>;

    /// Delete a cut by its ID
    async fn delete_cut(&self, cut_id: &str) -> Result<()>;

    /// Delete all cuts for a material
    async fn delete_cuts_by_material_id(&self, material_id: &str) -> Result<()>;

    /// Count cuts for a material
    async fn count_cuts_by_material_id(&self, material_id: &str) -> Result<usize>;
}
