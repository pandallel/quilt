use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

use super::types::{Material, MaterialStatus};

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

/// Thread-safe in-memory store for material objects
#[derive(Debug, Clone)]
pub struct MaterialRepository {
    /// The inner storage using a thread-safe hashmap
    materials: Arc<RwLock<HashMap<String, Material>>>,
}

impl MaterialRepository {
    /// Create a new empty material repository
    pub fn new() -> Self {
        Self {
            materials: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new material in the repository
    ///
    /// Returns an error if a material with the same ID already exists
    pub async fn register_material(&self, material: Material) -> Result<(), RepositoryError> {
        let mut materials = self.materials.write().await;

        if materials.contains_key(&material.id) {
            return Err(RepositoryError::MaterialAlreadyExists(material.id));
        }

        materials.insert(material.id.clone(), material);
        Ok(())
    }

    /// Get a material by its ID
    pub async fn get_material(&self, id: &str) -> Option<Material> {
        let materials = self.materials.read().await;
        materials.get(id).cloned()
    }

    /// Update the status of a material
    ///
    /// Returns an error if the material is not found or the status transition is invalid
    pub async fn update_material_status(
        &self,
        id: &str,
        status: MaterialStatus,
        error: Option<String>,
    ) -> Result<(), RepositoryError> {
        let mut materials = self.materials.write().await;

        let material = materials
            .get_mut(id)
            .ok_or_else(|| RepositoryError::MaterialNotFound(id.to_string()))?;

        // Any state can transition to Error
        if status == MaterialStatus::Error {
            material.status = status;
            if let Some(err_msg) = error {
                material.error = Some(err_msg);
            }
            return Ok(());
        }

        // For now, only allow linear progression
        match (material.status.clone(), status.clone()) {
            (MaterialStatus::Discovered, MaterialStatus::Cut) => {}
            (MaterialStatus::Cut, MaterialStatus::Swatched) => {}
            _ => {
                return Err(RepositoryError::InvalidStateTransition {
                    from: material.status.clone(),
                    to: status,
                });
            }
        }

        material.status = status;
        if error.is_some() {
            material.error = error;
        }

        Ok(())
    }

    /// List all materials in the repository
    pub async fn list_materials(&self) -> Vec<Material> {
        let materials = self.materials.read().await;
        materials.values().cloned().collect()
    }

    /// List all materials with a specific status
    pub async fn list_materials_by_status(&self, status: MaterialStatus) -> Vec<Material> {
        let materials = self.materials.read().await;
        materials
            .values()
            .filter(|m| m.status == status)
            .cloned()
            .collect()
    }

    /// Count materials by status
    pub async fn count_by_status(&self) -> HashMap<MaterialStatus, usize> {
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

impl Default for MaterialRepository {
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
        let repo = MaterialRepository::new();
        let material = create_test_material(MaterialStatus::Discovered);
        let id = material.id.clone();

        // Register the material
        repo.register_material(material).await.unwrap();

        // Retrieve the material
        let retrieved = repo.get_material(&id).await.unwrap();
        assert_eq!(retrieved.id, id);
        assert_eq!(retrieved.status, MaterialStatus::Discovered);
    }

    #[tokio::test]
    async fn test_register_duplicate_material() {
        let repo = MaterialRepository::new();
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
        let repo = MaterialRepository::new();
        let material = create_test_material(MaterialStatus::Discovered);
        let id = material.id.clone();

        // Register the material
        repo.register_material(material).await.unwrap();

        // Update status to Cut
        repo.update_material_status(&id, MaterialStatus::Cut, None)
            .await
            .unwrap();

        // Verify status change
        let updated = repo.get_material(&id).await.unwrap();
        assert_eq!(updated.status, MaterialStatus::Cut);

        // Update status to Swatched
        repo.update_material_status(&id, MaterialStatus::Swatched, None)
            .await
            .unwrap();

        // Verify status change
        let updated = repo.get_material(&id).await.unwrap();
        assert_eq!(updated.status, MaterialStatus::Swatched);
    }

    #[tokio::test]
    async fn test_invalid_status_transition() {
        let repo = MaterialRepository::new();
        let material = create_test_material(MaterialStatus::Discovered);
        let id = material.id.clone();

        // Register the material
        repo.register_material(material).await.unwrap();

        // Try to update from Discovered directly to Swatched (invalid)
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
    async fn test_transition_to_error() {
        let repo = MaterialRepository::new();
        let material = create_test_material(MaterialStatus::Discovered);
        let id = material.id.clone();

        // Register the material
        repo.register_material(material).await.unwrap();

        // Update status to Error
        let error_msg = "Test error message".to_string();
        repo.update_material_status(&id, MaterialStatus::Error, Some(error_msg.clone()))
            .await
            .unwrap();

        // Verify status change and error message
        let updated = repo.get_material(&id).await.unwrap();
        assert_eq!(updated.status, MaterialStatus::Error);
        assert_eq!(updated.error, Some(error_msg));
    }

    #[tokio::test]
    async fn test_list_materials() {
        let repo = MaterialRepository::new();

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
        let repo = MaterialRepository::new();

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
