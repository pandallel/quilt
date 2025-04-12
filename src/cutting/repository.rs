use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, info};

use super::cut::Cut;

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
pub trait CutsRepository: Send + Sync + 'static {
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

/// In-memory implementation of the CutsRepository
#[derive(Debug, Clone)]
pub struct InMemoryCutsRepository {
    /// The inner storage using a thread-safe hashmap of cut_id -> Cut
    cuts_by_id: Arc<RwLock<HashMap<String, Cut>>>,
    /// Index for fast lookup by material_id
    material_cut_index: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl InMemoryCutsRepository {
    /// Create a new empty cuts repository
    pub fn new() -> Self {
        Self {
            cuts_by_id: Arc::new(RwLock::new(HashMap::new())),
            material_cut_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryCutsRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl CutsRepository for InMemoryCutsRepository {
    fn save_cut(&self, cut: &Cut) -> impl std::future::Future<Output = Result<()>> + Send {
        let cut = cut.clone();
        async move {
            let cut_id = cut.id.clone();
            let material_id = cut.material_id.clone();

            // Check if the cut already exists
            let mut cuts = self.cuts_by_id.write().await;
            if cuts.contains_key(&cut_id) {
                return Err(CutsRepositoryError::CutAlreadyExists(cut_id));
            }

            // Insert the cut
            cuts.insert(cut_id.clone(), cut.clone());
            debug!("Saved cut: {} for material: {}", cut_id, material_id);

            // Update the material index
            let mut material_index = self.material_cut_index.write().await;
            material_index
                .entry(material_id.clone())
                .or_insert_with(Vec::new)
                .push(cut_id.clone());

            Ok(())
        }
    }

    fn save_cuts(&self, cuts: &[Cut]) -> impl std::future::Future<Output = Result<()>> + Send {
        let cuts = cuts.to_vec();
        async move {
            if cuts.is_empty() {
                return Ok(());
            }

            let mut cuts_by_id = self.cuts_by_id.write().await;
            let mut material_index = self.material_cut_index.write().await;

            // Check for duplicates first
            for cut in &cuts {
                if cuts_by_id.contains_key(&cut.id) {
                    return Err(CutsRepositoryError::CutAlreadyExists(cut.id.clone()));
                }
            }

            // Insert all cuts
            for cut in &cuts {
                let cut_id = cut.id.clone();
                let material_id = cut.material_id.clone();

                cuts_by_id.insert(cut_id.clone(), cut.clone());
                material_index
                    .entry(material_id.clone())
                    .or_insert_with(Vec::new)
                    .push(cut_id.clone());
            }

            info!("Saved {} cuts in batch", cuts.len());
            Ok(())
        }
    }

    fn get_cut_by_id(
        &self,
        cut_id: &str,
    ) -> impl std::future::Future<Output = Result<Option<Cut>>> + Send {
        let cut_id = cut_id.to_string();
        async move {
            let cuts = self.cuts_by_id.read().await;
            Ok(cuts.get(&cut_id).cloned())
        }
    }

    fn get_cuts_by_material_id(
        &self,
        material_id: &str,
    ) -> impl std::future::Future<Output = Result<Vec<Cut>>> + Send {
        let material_id = material_id.to_string();
        async move {
            let material_index = self.material_cut_index.read().await;
            let cuts_by_id = self.cuts_by_id.read().await;

            if let Some(cut_ids) = material_index.get(&material_id) {
                let mut result = Vec::new();
                for cut_id in cut_ids {
                    if let Some(cut) = cuts_by_id.get(cut_id) {
                        result.push(cut.clone());
                    }
                }
                // Sort by chunk_index for consistent ordering
                result.sort_by_key(|cut| cut.chunk_index);
                Ok(result)
            } else {
                Ok(Vec::new())
            }
        }
    }

    fn delete_cut(&self, cut_id: &str) -> impl std::future::Future<Output = Result<()>> + Send {
        let cut_id = cut_id.to_string();
        async move {
            let mut cuts_by_id = self.cuts_by_id.write().await;

            // Get the material_id for this cut before deletion
            let material_id = if let Some(cut) = cuts_by_id.get(&cut_id) {
                cut.material_id.clone()
            } else {
                return Err(CutsRepositoryError::CutNotFound(cut_id.to_string()));
            };

            // Remove from the primary map
            cuts_by_id.remove(&cut_id);

            // Update the material index
            let mut material_index = self.material_cut_index.write().await;
            if let Some(cut_ids) = material_index.get_mut(&material_id) {
                cut_ids.retain(|id| id != &cut_id);
                // If this was the last cut for this material, remove the material entry
                if cut_ids.is_empty() {
                    material_index.remove(&material_id);
                }
            }

            debug!("Deleted cut: {}", cut_id);
            Ok(())
        }
    }

    fn delete_cuts_by_material_id(
        &self,
        material_id: &str,
    ) -> impl std::future::Future<Output = Result<()>> + Send {
        let material_id = material_id.to_string();
        async move {
            let mut material_index = self.material_cut_index.write().await;

            // Get the cut IDs for this material
            let cut_ids = if let Some(ids) = material_index.remove(&material_id) {
                ids
            } else {
                // No cuts found for this material
                return Ok(());
            };

            // Remove all cuts from the primary map
            let mut cuts_by_id = self.cuts_by_id.write().await;
            for cut_id in &cut_ids {
                cuts_by_id.remove(cut_id);
            }

            info!(
                "Deleted {} cuts for material: {}",
                cut_ids.len(),
                material_id
            );
            Ok(())
        }
    }

    fn count_cuts_by_material_id(
        &self,
        material_id: &str,
    ) -> impl std::future::Future<Output = Result<usize>> + Send {
        let material_id = material_id.to_string();
        async move {
            let material_index = self.material_cut_index.read().await;
            if let Some(cut_ids) = material_index.get(&material_id) {
                Ok(cut_ids.len())
            } else {
                Ok(0)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_cut(material_id: &str, chunk_index: usize) -> Cut {
        Cut::new(
            material_id.to_string(),
            chunk_index,
            format!("Content for chunk {}", chunk_index),
        )
    }

    #[tokio::test]
    async fn test_save_and_get_cut() {
        let repo = InMemoryCutsRepository::new();
        let cut = create_test_cut("material1", 0);
        let cut_id = cut.id.clone();

        // Save the cut
        repo.save_cut(&cut).await.unwrap();

        // Retrieve the cut
        let retrieved = repo.get_cut_by_id(&cut_id).await.unwrap().unwrap();
        assert_eq!(retrieved.id, cut_id);
        assert_eq!(retrieved.material_id, "material1");
    }

    #[tokio::test]
    async fn test_save_duplicate_cut() {
        let repo = InMemoryCutsRepository::new();
        let cut = create_test_cut("material1", 0);

        // Save the cut
        repo.save_cut(&cut).await.unwrap();

        // Try to save the same cut again
        let result = repo.save_cut(&cut).await;
        assert!(result.is_err());

        if let Err(CutsRepositoryError::CutAlreadyExists(id)) = result {
            assert_eq!(id, cut.id);
        } else {
            panic!("Expected CutAlreadyExists error");
        }
    }

    #[tokio::test]
    async fn test_save_and_get_cuts_by_material() {
        let repo = InMemoryCutsRepository::new();
        let material_id = "material1";

        // Create and save multiple cuts for the same material
        let cut1 = create_test_cut(material_id, 0);
        let cut2 = create_test_cut(material_id, 1);
        let cut3 = create_test_cut(material_id, 2);

        repo.save_cuts(&[cut1, cut2, cut3]).await.unwrap();

        // Retrieve cuts for the material
        let cuts = repo.get_cuts_by_material_id(material_id).await.unwrap();
        assert_eq!(cuts.len(), 3);

        // Verify they're sorted by chunk_index
        assert_eq!(cuts[0].chunk_index, 0);
        assert_eq!(cuts[1].chunk_index, 1);
        assert_eq!(cuts[2].chunk_index, 2);
    }

    #[tokio::test]
    async fn test_delete_cut() {
        let repo = InMemoryCutsRepository::new();
        let cut = create_test_cut("material1", 0);
        let cut_id = cut.id.clone();

        // Save and then delete the cut
        repo.save_cut(&cut).await.unwrap();
        repo.delete_cut(&cut_id).await.unwrap();

        // Verify it's gone
        let result = repo.get_cut_by_id(&cut_id).await.unwrap();
        assert!(result.is_none());

        // Check that the material index is updated
        let cuts = repo.get_cuts_by_material_id("material1").await.unwrap();
        assert_eq!(cuts.len(), 0);
    }

    #[tokio::test]
    async fn test_delete_cuts_by_material() {
        let repo = InMemoryCutsRepository::new();
        let material_id = "material1";

        // Create and save multiple cuts for the same material
        let cuts = vec![
            create_test_cut(material_id, 0),
            create_test_cut(material_id, 1),
            create_test_cut(material_id, 2),
        ];

        repo.save_cuts(&cuts).await.unwrap();

        // Delete all cuts for the material
        repo.delete_cuts_by_material_id(material_id).await.unwrap();

        // Verify they're all gone
        let remaining = repo.get_cuts_by_material_id(material_id).await.unwrap();
        assert_eq!(remaining.len(), 0);

        // Check count is zero
        let count = repo.count_cuts_by_material_id(material_id).await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_count_cuts() {
        let repo = InMemoryCutsRepository::new();
        let material_id = "material1";

        // Initially zero
        let count = repo.count_cuts_by_material_id(material_id).await.unwrap();
        assert_eq!(count, 0);

        // Add three cuts
        let cuts = vec![
            create_test_cut(material_id, 0),
            create_test_cut(material_id, 1),
            create_test_cut(material_id, 2),
        ];

        repo.save_cuts(&cuts).await.unwrap();

        // Count should be 3
        let count = repo.count_cuts_by_material_id(material_id).await.unwrap();
        assert_eq!(count, 3);
    }
}
