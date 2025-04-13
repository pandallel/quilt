use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::RwLock;
use time::OffsetDateTime;

use super::{Material, MaterialRepository, MaterialStatus, RepositoryError, Result};

/// Thread-safe in-memory store for material objects
#[derive(Debug, Clone)]
pub struct InMemoryMaterialRepository {
    /// The inner storage using a thread-safe hashmap
    materials: Arc<RwLock<HashMap<String, Material>>>,
}

impl InMemoryMaterialRepository {
    /// Create a new empty material repository
    pub fn new() -> Self {
        Self {
            materials: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl MaterialRepository for InMemoryMaterialRepository {
    /// Register a new material in the repository
    ///
    /// Returns an error if a material with the same ID already exists
    async fn register_material(&self, material: Material) -> Result<()> {
        let mut materials = self.materials.write().await;

        if materials.contains_key(&material.id) {
            return Err(RepositoryError::MaterialAlreadyExists(material.id));
        }

        materials.insert(material.id.clone(), material);
        Ok(())
    }

    /// Get a material by its ID
    async fn get_material(&self, id: &str) -> Option<Material> {
        let materials = self.materials.read().await;
        materials.get(id).cloned()
    }

    /// Update the status of a material
    ///
    /// Returns an error if the material is not found or the status transition is invalid
    async fn update_material_status(
        &self,
        id: &str,
        new_status: MaterialStatus,
        error_message: Option<String>,
    ) -> Result<()> {
        let mut materials = self.materials.write().await;

        let material = materials
            .get_mut(id)
            .ok_or_else(|| RepositoryError::MaterialNotFound(id.to_string()))?;

        // Validate status transition
        match (material.status.clone(), new_status.clone()) {
            // Valid transitions
            (MaterialStatus::Discovered, MaterialStatus::Cut)
            | (MaterialStatus::Discovered, MaterialStatus::Error)
            | (MaterialStatus::Cut, MaterialStatus::Swatched)
            | (MaterialStatus::Cut, MaterialStatus::Error)
            | (MaterialStatus::Swatched, MaterialStatus::Error)
            | (MaterialStatus::Error, MaterialStatus::Discovered) => {
                let now = OffsetDateTime::now_utc();
                // Update status
                material.status = new_status;
                // Update timestamps
                material.updated_at = now;
                material.status_updated_at = now;
                // Update error message if provided
                if let Some(msg) = error_message {
                    material.error = Some(msg);
                } else {
                    material.error = None;
                }
                Ok(())
            }
            // Invalid transitions
            (from, to) => Err(RepositoryError::InvalidStateTransition { from, to }),
        }
    }

    /// List all materials
    async fn list_materials(&self) -> Vec<Material> {
        let materials = self.materials.read().await;
        materials.values().cloned().collect()
    }

    /// List materials by status
    async fn list_materials_by_status(&self, status: MaterialStatus) -> Vec<Material> {
        let materials = self.materials.read().await;
        materials
            .values()
            .filter(|m| m.status == status)
            .cloned()
            .collect()
    }

    /// Count materials by status
    async fn count_by_status(&self) -> HashMap<MaterialStatus, usize> {
        let materials = self.materials.read().await;
        let mut counts = HashMap::new();

        // Initialize counts for all statuses
        counts.insert(MaterialStatus::Discovered, 0);
        counts.insert(MaterialStatus::Cut, 0);
        counts.insert(MaterialStatus::Swatched, 0);
        counts.insert(MaterialStatus::Error, 0);

        // Count materials by status
        for material in materials.values() {
            let count = counts.entry(material.status.clone()).or_insert(0);
            *count += 1;
        }

        counts
    }
}

impl Default for InMemoryMaterialRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_material(status: MaterialStatus) -> Material {
        let mut material = Material::new("test/path.md".to_string());
        material.status = status;
        material
    }

    #[tokio::test]
    async fn test_register_and_get_material() {
        let repo = InMemoryMaterialRepository::new();
        let material = create_test_material(MaterialStatus::Discovered);
        let id = material.id.clone();

        // Register the material
        repo.register_material(material).await.unwrap();

        // Retrieve the material
        let retrieved = repo.get_material(&id).await.unwrap();
        assert_eq!(retrieved.id, id);
        assert_eq!(retrieved.status, MaterialStatus::Discovered);
        assert_eq!(retrieved.created_at, retrieved.updated_at);
        assert_eq!(retrieved.created_at, retrieved.status_updated_at);
    }

    #[tokio::test]
    async fn test_register_duplicate_material() {
        let repo = InMemoryMaterialRepository::new();
        let material = create_test_material(MaterialStatus::Discovered);
        let id = material.id.clone();

        // Register the material
        repo.register_material(material).await.unwrap();

        // Try to register the same material again
        let duplicate = create_test_material(MaterialStatus::Discovered);
        let duplicate_with_same_id = Material {
            id: id.clone(),
            ..duplicate
        };

        let result = repo.register_material(duplicate_with_same_id).await;
        assert!(result.is_err());
        if let Err(RepositoryError::MaterialAlreadyExists(existing_id)) = result {
            assert_eq!(existing_id, id);
        } else {
            panic!("Expected MaterialAlreadyExists error");
        }
    }

    #[tokio::test]
    async fn test_update_material_status() {
        let repo = InMemoryMaterialRepository::new();
        let material = create_test_material(MaterialStatus::Discovered);
        let id = material.id.clone();
        let created_at = material.created_at;

        // Register the material
        repo.register_material(material).await.unwrap();

        // Small delay to ensure timestamps will be different
        sleep(Duration::from_millis(1)).await;

        // Update status to Cut
        repo.update_material_status(&id, MaterialStatus::Cut, None)
            .await
            .unwrap();

        // Verify status change and timestamps
        let updated_after_cut = repo.get_material(&id).await.unwrap();
        assert_eq!(updated_after_cut.status, MaterialStatus::Cut);
        assert_eq!(updated_after_cut.created_at, created_at);
        assert!(updated_after_cut.updated_at > created_at);
        assert!(updated_after_cut.status_updated_at > created_at);
        assert_eq!(updated_after_cut.updated_at, updated_after_cut.status_updated_at);
        let first_update_time = updated_after_cut.updated_at; // Capture time after first update

        // Small delay to ensure timestamps will be different
        sleep(Duration::from_millis(1)).await;

        // Update status to Swatched
        repo.update_material_status(&id, MaterialStatus::Swatched, None)
            .await
            .unwrap();

        // Verify status change and timestamps
        let updated_after_swatched = repo.get_material(&id).await.unwrap();
        assert_eq!(updated_after_swatched.status, MaterialStatus::Swatched);
        assert_eq!(updated_after_swatched.created_at, created_at);
        assert!(updated_after_swatched.updated_at >= first_update_time);
        assert!(updated_after_swatched.status_updated_at >= first_update_time);
        assert_eq!(updated_after_swatched.updated_at, updated_after_swatched.status_updated_at); // Should be equal after this update
    }

    #[tokio::test]
    async fn test_update_material_status_with_error() {
        let repo = InMemoryMaterialRepository::new();
        let material = create_test_material(MaterialStatus::Discovered);
        let id = material.id.clone();
        let created_at = material.created_at;

        // Register the material
        repo.register_material(material).await.unwrap();

        // Small delay to ensure timestamps will be different
        sleep(Duration::from_millis(1)).await;

        // Update status to Error with a message
        let error_message = "Test error message".to_string();
        repo.update_material_status(&id, MaterialStatus::Error, Some(error_message.clone()))
            .await
            .unwrap();

        // Verify status change, error message, and timestamps
        let updated_after_error = repo.get_material(&id).await.unwrap();
        assert_eq!(updated_after_error.status, MaterialStatus::Error);
        assert_eq!(updated_after_error.error, Some(error_message));
        assert_eq!(updated_after_error.created_at, created_at);
        assert!(updated_after_error.updated_at > created_at);
        assert!(updated_after_error.status_updated_at > created_at);
        assert_eq!(updated_after_error.updated_at, updated_after_error.status_updated_at);
        let first_update_time = updated_after_error.updated_at; // Capture time after first update

        // Small delay to ensure timestamps will be different
        sleep(Duration::from_millis(1)).await;

        // Reset to Discovered (simulating retry)
        repo.update_material_status(&id, MaterialStatus::Discovered, None)
            .await
            .unwrap();

        // Verify status change, error message cleared, and timestamps
        let updated_after_reset = repo.get_material(&id).await.unwrap();
        assert_eq!(updated_after_reset.status, MaterialStatus::Discovered);
        assert_eq!(updated_after_reset.error, None);
        assert_eq!(updated_after_reset.created_at, created_at);
        assert!(updated_after_reset.updated_at >= first_update_time);
        assert!(updated_after_reset.status_updated_at >= first_update_time);
        assert_eq!(updated_after_reset.updated_at, updated_after_reset.status_updated_at); // Should be equal after this update
    }

    #[tokio::test]
    async fn test_invalid_state_transition() {
        let repo = InMemoryMaterialRepository::new();
        let material = create_test_material(MaterialStatus::Discovered);
        let id = material.id.clone();

        // Register the material
        repo.register_material(material).await.unwrap();

        // Try invalid transition (Discovered -> Swatched)
        let result = repo
            .update_material_status(&id, MaterialStatus::Swatched, None)
            .await;

        assert!(result.is_err());
        if let Err(RepositoryError::InvalidStateTransition { from, to }) = result {
            assert_eq!(from, MaterialStatus::Discovered);
            assert_eq!(to, MaterialStatus::Swatched);
        } else {
            panic!("Expected InvalidStateTransition error");
        }
    }

    #[tokio::test]
    async fn test_list_materials() {
        let repo = InMemoryMaterialRepository::new();

        // Create and register 3 materials with different statuses
        let material1 = create_test_material(MaterialStatus::Discovered);
        let material2 = create_test_material(MaterialStatus::Cut);
        let material3 = create_test_material(MaterialStatus::Swatched);

        repo.register_material(material1).await.unwrap();
        repo.register_material(material2).await.unwrap();
        repo.register_material(material3).await.unwrap();

        // List all materials
        let all = repo.list_materials().await;
        assert_eq!(all.len(), 3);

        // List by status
        let discovered = repo
            .list_materials_by_status(MaterialStatus::Discovered)
            .await;
        assert_eq!(discovered.len(), 1);

        let cut = repo.list_materials_by_status(MaterialStatus::Cut).await;
        assert_eq!(cut.len(), 1);

        let swatched = repo
            .list_materials_by_status(MaterialStatus::Swatched)
            .await;
        assert_eq!(swatched.len(), 1);

        let error = repo.list_materials_by_status(MaterialStatus::Error).await;
        assert_eq!(error.len(), 0);
    }

    #[tokio::test]
    async fn test_count_by_status() {
        let repo = InMemoryMaterialRepository::new();

        // Empty repository should have 0 counts
        let counts = repo.count_by_status().await;
        assert_eq!(counts.get(&MaterialStatus::Discovered), Some(&0));

        // Add 2 discovered materials
        let material1 = create_test_material(MaterialStatus::Discovered);
        let material2 = create_test_material(MaterialStatus::Discovered);
        repo.register_material(material1).await.unwrap();
        repo.register_material(material2).await.unwrap();

        // Add 1 cut material
        let material3 = create_test_material(MaterialStatus::Cut);
        repo.register_material(material3).await.unwrap();

        // Check counts
        let counts = repo.count_by_status().await;
        assert_eq!(counts.get(&MaterialStatus::Discovered), Some(&2));
        assert_eq!(counts.get(&MaterialStatus::Cut), Some(&1));
        assert_eq!(counts.get(&MaterialStatus::Swatched), Some(&0));
        assert_eq!(counts.get(&MaterialStatus::Error), Some(&0));
    }
}
