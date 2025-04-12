// Cutting module for processing materials after discovery
// This module contains the CuttingActor and related functionality

use std::fmt::Debug;
use thiserror::Error;

pub mod actor;
pub mod cut;
pub mod cutter;
pub mod repository;

pub use actor::messages;
pub use actor::CuttingActor;
pub use cut::Cut;
pub use cutter::{CutterConfig, TextCutter};
pub use repository::InMemoryCutsRepository;

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
pub trait CutsRepository: Send + Sync + Debug + 'static {
    /// Save a cut to the repository
    fn save_cut(&self, cut: &Cut) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Save multiple cuts in a batch operation
    fn save_cuts(&self, cuts: &[Cut]) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Get a cut by its ID
    fn get_cut_by_id(
        &self,
        cut_id: &str,
    ) -> impl std::future::Future<Output = Result<Option<Cut>>> + Send;

    /// Get all cuts for a specific material
    fn get_cuts_by_material_id(
        &self,
        material_id: &str,
    ) -> impl std::future::Future<Output = Result<Vec<Cut>>> + Send;

    /// Delete a cut by its ID
    fn delete_cut(&self, cut_id: &str) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Delete all cuts for a material
    fn delete_cuts_by_material_id(
        &self,
        material_id: &str,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Count cuts for a material
    fn count_cuts_by_material_id(
        &self,
        material_id: &str,
    ) -> impl std::future::Future<Output = Result<usize>> + Send;
}
