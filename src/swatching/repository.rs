use std::fmt::Debug;
use thiserror::Error;

use async_trait::async_trait;

use super::swatch::Swatch;

/// Errors that can occur during swatch repository operations
#[derive(Error, Debug)]
pub enum SwatchRepositoryError {
    #[error("Swatch with id {0} not found")]
    SwatchNotFound(Box<str>),

    #[error("Swatch with id {0} already exists")]
    SwatchAlreadyExists(Box<str>),

    #[error("Operation failed: {0}")]
    OperationFailed(Box<str>),

    #[error("Search operation failed: {0}")]
    SearchFailed(Box<str>),
}

/// Result type for swatch repository operations
pub type Result<T> = std::result::Result<T, SwatchRepositoryError>;

/// Repository trait for managing swatches
#[async_trait]
pub trait SwatchRepository: Send + Sync + Debug + 'static {
    /// Save a swatch to the repository
    async fn save_swatch(&self, swatch: &Swatch) -> Result<()>;

    /// Save multiple swatches in a batch operation
    async fn save_swatches_batch(&self, swatches: &[Swatch]) -> Result<()>;

    /// Get a swatch by its ID
    async fn get_swatch_by_id(&self, swatch_id: &str) -> Result<Option<Swatch>>;

    /// Get all swatches for a specific cut
    async fn get_swatches_by_cut_id(&self, cut_id: &str) -> Result<Vec<Swatch>>;

    /// Get all swatches for a specific material
    async fn get_swatches_by_material_id(&self, material_id: &str) -> Result<Vec<Swatch>>;

    /// Delete a swatch by its ID
    async fn delete_swatch(&self, swatch_id: &str) -> Result<()>;

    /// Delete all swatches for a cut
    async fn delete_swatches_by_cut_id(&self, cut_id: &str) -> Result<()>;

    /// Delete all swatches for a material
    async fn delete_swatches_by_material_id(&self, material_id: &str) -> Result<()>;

    /// Perform a similarity search to find the most similar swatches to an embedding
    ///
    /// * `embedding` - The query embedding to compare against
    /// * `limit` - Maximum number of results to return
    /// * `min_score` - Optional minimum similarity score (0.0 to 1.0)
    ///
    /// Returns a vector of (Swatch, score) pairs, sorted by decreasing similarity score
    async fn search_similar(
        &self,
        embedding: &[f32],
        limit: usize,
        min_score: Option<f32>,
    ) -> Result<Vec<(Swatch, f32)>>;
}
