use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, error, info};

use crate::events::{EventBus, EventBusError, QuiltEvent};
use crate::materials::repository::{MaterialRepository, RepositoryError};
use crate::materials::types::{Material, MaterialStatus};

/// Errors that can occur during registry operations
#[derive(Error, Debug)]
pub enum RegistryError {
    /// Error from the repository
    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),

    /// Error from the event bus
    #[error("Event bus error: {0}")]
    EventBus(#[from] EventBusError),

    /// Operation failed
    #[error("Registry operation failed: {0}")]
    OperationFailed(String),
}

/// Material Registry coordinates state management and event publishing
#[derive(Debug, Clone)]
pub struct MaterialRegistry {
    /// The underlying repository for persistence
    repository: MaterialRepository,
    /// The event bus for publishing events
    event_bus: Arc<EventBus>,
}

impl MaterialRegistry {
    /// Create a new registry with the given repository and event bus
    pub fn new(repository: MaterialRepository, event_bus: Arc<EventBus>) -> Self {
        Self {
            repository,
            event_bus,
        }
    }

    /// Register a new material and publish a MaterialDiscovered event
    pub async fn register_material(&self, material: Material) -> Result<(), RegistryError> {
        debug!("Registering material: {}", material.id);

        // First register in the repository
        self.repository.register_material(material.clone()).await?;

        // Then publish the event
        let event = QuiltEvent::material_discovered(&material);
        self.event_bus.publish(event)?;

        info!("Material registered successfully: {}", material.id);
        Ok(())
    }

    /// Get a material by ID (passthrough to repository)
    pub async fn get_material(&self, id: &str) -> Option<Material> {
        self.repository.get_material(id).await
    }

    /// List all materials (passthrough to repository)
    pub async fn list_materials(&self) -> Vec<Material> {
        self.repository.list_materials().await
    }

    /// List materials by status (passthrough to repository)
    pub async fn list_materials_by_status(&self, status: MaterialStatus) -> Vec<Material> {
        self.repository.list_materials_by_status(status).await
    }

    /// Update the status of a material in the repository
    pub async fn update_material_status(
        &self,
        id: &str,
        status: MaterialStatus,
        error: Option<String>,
    ) -> Result<(), RegistryError> {
        debug!("Updating material status: {} -> {:?}", id, &status);
        
        // Use status.clone() to avoid moving the value
        let status_clone = status.clone();
        
        // Update the status in the repository
        self.repository
            .update_material_status(id, status, error)
            .await
            .map_err(RegistryError::Repository)?;
        
        debug!("Material status updated successfully: {} -> {:?}", id, status_clone);
        Ok(())
    }

    /// Get the underlying repository
    pub fn repository(&self) -> &MaterialRepository {
        &self.repository
    }

    /// Get the event bus
    pub fn event_bus(&self) -> &EventBus {
        &self.event_bus
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::broadcast::Receiver;

    // Helper function to create a registry with a subscriber
    async fn setup_registry() -> (MaterialRegistry, Receiver<QuiltEvent>) {
        let repository = MaterialRepository::new();
        let event_bus = Arc::new(EventBus::new());
        let receiver = event_bus.subscribe();
        let registry = MaterialRegistry::new(repository, event_bus);
        (registry, receiver)
    }

    #[tokio::test]
    async fn test_register_material() {
        let (registry, mut receiver) = setup_registry().await;

        // Create and register a material
        let material = Material::new("test/file.md".to_string());
        let material_id = material.id.clone();

        registry.register_material(material).await.unwrap();

        // Check that the material was registered in the repository
        let stored = registry.get_material(&material_id).await.unwrap();
        assert_eq!(stored.id, material_id);

        // Check that an event was published
        let event = receiver.recv().await.unwrap();
        if let QuiltEvent::MaterialDiscovered(evt) = event {
            assert_eq!(evt.material_id.as_str(), material_id);
        } else {
            panic!("Expected MaterialDiscovered event");
        }
    }
}
