use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, error, info};

use crate::events::types::ProcessingStage;
use crate::events::{EventBus, EventBusError, QuiltEvent};
use crate::materials::types::{Material, MaterialStatus};
use crate::materials::{MaterialRepository, RepositoryError};

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
    repository: Arc<dyn MaterialRepository>,
    /// The event bus for publishing events
    event_bus: Arc<EventBus>,
}

impl MaterialRegistry {
    /// Create a new registry with the given repository and event bus
    pub fn new(repository: Arc<dyn MaterialRepository>, event_bus: Arc<EventBus>) -> Self {
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
        self.event_bus
            .publish(event)
            .map_err(RegistryError::EventBus)?;

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

        // Get the current material (to determine the processing stage if we're transitioning to Error)
        let current_material = self.repository.get_material(id).await;
        let current_status = current_material.as_ref().map(|m| m.status.clone());

        // Use status.clone() to avoid moving the value
        let status_clone = status.clone();

        // Update the status in the repository
        self.repository
            .update_material_status(id, status, error.clone())
            .await
            .map_err(RegistryError::Repository)?;

        debug!(
            "Material status updated successfully: {} -> {:?}",
            id, status_clone
        );

        // Publish appropriate events based on the new status
        match status_clone {
            MaterialStatus::Cut => {
                // Material has been cut, publish a MaterialCut event
                let event = QuiltEvent::material_cut(id);
                self.event_bus
                    .publish(event)
                    .map_err(RegistryError::EventBus)?;
                debug!("Published MaterialCut event for material: {}", id);
            }
            MaterialStatus::Swatched => {
                // Material has been swatched, publish a MaterialSwatched event
                let event = QuiltEvent::material_swatched(id);
                self.event_bus
                    .publish(event)
                    .map_err(RegistryError::EventBus)?;
                debug!("Published MaterialSwatched event for material: {}", id);
            }
            MaterialStatus::Error => {
                // Material has encountered an error, publish a ProcessingError event
                if let Some(error_message) = error {
                    // Determine the processing stage based on the previous status
                    let stage = match current_status {
                        Some(MaterialStatus::Discovered) => ProcessingStage::Cutting,
                        Some(MaterialStatus::Cut) => ProcessingStage::Swatching,
                        _ => ProcessingStage::Custom("Unknown".to_string()),
                    };

                    let event =
                        QuiltEvent::create_processing_error_event(id, stage, &error_message);
                    self.event_bus
                        .publish(event)
                        .map_err(RegistryError::EventBus)?;
                    debug!("Published ProcessingError event for material: {}", id);
                }
            }
            _ => {
                // No events to publish for other status changes
            }
        }

        // Log progress after status update and event publishing
        let status_counts = self.repository.count_by_status().await;
        let total_count = status_counts.values().sum::<usize>();
        let swatched_count = status_counts
            .get(&MaterialStatus::Swatched)
            .copied()
            .unwrap_or(0);
        let error_count = status_counts
            .get(&MaterialStatus::Error)
            .copied()
            .unwrap_or(0);
        let processed_count = swatched_count + error_count;

        if total_count > 0 {
            let percentage = (processed_count as f32 / total_count as f32) * 100.0;
            info!(
                "Progress: {} / {} materials processed ({:.1}%), Swatched: {}, Error: {}",
                processed_count, total_count, percentage, swatched_count, error_count
            );
        }

        Ok(())
    }

    /// Get the underlying repository
    pub fn repository(&self) -> &Arc<dyn MaterialRepository> {
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
    use crate::materials::InMemoryMaterialRepository;
    use std::sync::Arc;
    use tokio::sync::broadcast::Receiver;

    // Helper function to create a registry with a subscriber
    async fn setup_registry() -> (MaterialRegistry, Receiver<QuiltEvent>) {
        let repository = Arc::new(InMemoryMaterialRepository::new());
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

    #[tokio::test]
    async fn test_update_status_to_cut_publishes_event() {
        let (registry, mut receiver) = setup_registry().await;

        // Create and register a material
        let material = Material::new("test/file.md".to_string());
        let material_id = material.id.clone();
        registry.register_material(material).await.unwrap();

        // Consume the MaterialDiscovered event
        let _ = receiver.recv().await.unwrap();

        // Update the status to Cut
        registry
            .update_material_status(&material_id, MaterialStatus::Cut, None)
            .await
            .unwrap();

        // Check that a MaterialCut event was published
        let event = receiver.recv().await.unwrap();
        if let QuiltEvent::MaterialCut(evt) = event {
            assert_eq!(evt.material_id.as_str(), material_id);
        } else {
            panic!("Expected MaterialCut event, got {:?}", event);
        }
    }

    #[tokio::test]
    async fn test_update_status_to_error_publishes_event() {
        let (registry, mut receiver) = setup_registry().await;

        // Create and register a material
        let material = Material::new("test/file.md".to_string());
        let material_id = material.id.clone();
        registry.register_material(material).await.unwrap();

        // Consume the MaterialDiscovered event
        let _ = receiver.recv().await.unwrap();

        // Update the status to Error
        let error_message = "Test error message";
        registry
            .update_material_status(
                &material_id,
                MaterialStatus::Error,
                Some(error_message.to_string()),
            )
            .await
            .unwrap();

        // Check that a ProcessingError event was published
        let event = receiver.recv().await.unwrap();
        if let QuiltEvent::ProcessingError(evt) = event {
            assert_eq!(evt.material_id.as_str(), material_id);
            assert_eq!(evt.message, error_message);
            assert!(matches!(evt.stage, ProcessingStage::Cutting));
        } else {
            panic!("Expected ProcessingError event, got {:?}", event);
        }
    }
}
