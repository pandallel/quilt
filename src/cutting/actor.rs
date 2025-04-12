use crate::actors::{Ping, Shutdown};
use crate::events::types::{MaterialId, ProcessingStage};
use crate::events::QuiltEvent;
use crate::materials::MaterialRegistry;
use crate::materials::types::MaterialStatus;
use actix::prelude::*;
use log::{debug, error, info, warn};
use tokio::sync::broadcast;
use tokio::fs;
use std::path::Path;

use super::cutter::TextCutter;

/// Messages specific to the CuttingActor
///
/// This module contains all message types that can be sent to the CuttingActor
/// to request operations and their respective response types.
pub mod messages {
    use crate::events::types::MaterialId;
    use actix::prelude::*;
    use thiserror::Error;

    /// Cutting operation errors
    ///
    /// These errors can occur during cutting operations and provide
    /// detailed information about what went wrong.
    #[derive(Debug, Error)]
    pub enum CuttingError {
        /// Material not found error
        #[error("Material not found: {0}")]
        MaterialNotFound(MaterialId),

        /// Generic cutting error
        #[error("Cutting operation failed: {0}")]
        OperationFailed(String),

        /// File error
        #[error("File operation failed: {0}")]
        FileError(#[from] std::io::Error),
    }

    /// Response for operation completion status
    ///
    /// This message can be sent to interested parties to notify them
    /// of the completion of an operation.
    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct OperationComplete {
        /// Whether the operation was successful
        pub success: bool,
        /// Additional information about the operation
        pub message: String,
    }
}

/// Actor responsible for processing materials that have been discovered
///
/// The CuttingActor subscribes to MaterialDiscovered events and processes
/// the discovered materials.
///
/// # Message Handlers
///
/// * `Ping` - Responds with `true` to indicate the actor is alive
/// * `Shutdown` - Gracefully shuts down the actor
pub struct CuttingActor {
    /// Name of this actor instance for logging
    name: String,
    /// Registry to retrieve materials and publish events
    registry: MaterialRegistry,
    /// Event receiver for MaterialDiscovered events
    event_receiver: Option<broadcast::Receiver<QuiltEvent>>,
    /// Text cutter with default configuration
    cutter: TextCutter,
}

impl CuttingActor {
    /// Create a new CuttingActor with the given name and registry
    ///
    /// # Arguments
    ///
    /// * `name` - Name for this actor instance, used in logging
    /// * `registry` - Registry to retrieve materials and publish events
    pub fn new(name: &str, registry: MaterialRegistry) -> Self {
        Self {
            name: name.to_string(),
            registry,
            event_receiver: None,
            cutter: TextCutter::default(),
        }
    }
}

impl Actor for CuttingActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("{}: Started", self.name);

        // Subscribe to events via the registry
        let event_receiver = self.registry.event_bus().subscribe();
        // Store the receiver in the actor state
        self.event_receiver = Some(event_receiver.resubscribe());
        let addr = ctx.address();

        // Process events in a separate task
        ctx.spawn(
            async move {
                info!("Started listening for material events");
                let mut receiver = event_receiver;

                // Process events until error or shutdown
                while let Ok(event) = receiver.recv().await {
                    if let QuiltEvent::MaterialDiscovered(evt) = event {
                        debug!(
                            "Received MaterialDiscovered event: {}",
                            evt.material_id.as_str()
                        );

                        // Send a message to self to process the event
                        addr.do_send(ProcessDiscoveredMaterial {
                            material_id: evt.material_id,
                            file_path: evt.file_path.clone(),
                        });
                    }
                }

                info!("Stopped listening for material events");
            }
            .into_actor(self),
        );
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("{}: Stopped", self.name);
        // Clear the event receiver on shutdown
        self.event_receiver = None;
    }
}

impl Handler<Ping> for CuttingActor {
    type Result = bool;

    fn handle(&mut self, _msg: Ping, _ctx: &mut Self::Context) -> Self::Result {
        debug!("{}: Received ping", self.name);
        true
    }
}

impl Handler<Shutdown> for CuttingActor {
    type Result = ();

    fn handle(&mut self, _msg: Shutdown, ctx: &mut Self::Context) -> Self::Result {
        info!("{}: Shutting down", self.name);
        ctx.stop();
    }
}

/// Internal message to process a discovered material
#[derive(Message)]
#[rtype(result = "()")]
struct ProcessDiscoveredMaterial {
    /// ID of the discovered material
    material_id: MaterialId,
    /// File path of the discovered material
    file_path: String,
}

impl Handler<ProcessDiscoveredMaterial> for CuttingActor {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, msg: ProcessDiscoveredMaterial, _ctx: &mut Self::Context) -> Self::Result {
        let name = self.name.clone();
        debug!(
            "{}: Processing discovered material '{}'",
            name,
            msg.material_id.as_str()
        );

        // Use message data directly and access self methods without cloning the actor
        let material_id = msg.material_id;
        let file_path = msg.file_path;
        let registry = self.registry.clone();
        let actor_name = self.name.clone();
        let cutter = self.cutter.clone();

        Box::pin(async move {
            // Process the discovered material using the cloned registry instead of the actor instance
            if let Err(e) = process_discovered_material(&actor_name, &registry, material_id, file_path, &cutter).await {
                error!("{}: Error processing material: {}", actor_name, e);
            }
        })
    }
}

/// Process a MaterialDiscovered event
///
/// Reads the file content, cuts it into chunks, and logs sample information.
///
/// # Arguments
///
/// * `actor_name` - Name of the actor for logging
/// * `registry` - Registry to retrieve materials
/// * `material_id` - ID of the discovered material
/// * `file_path` - File path of the discovered material
/// * `cutter` - TextCutter instance to cut the material content
async fn process_discovered_material(
    actor_name: &str,
    registry: &MaterialRegistry,
    material_id: MaterialId,
    file_path: String,
    cutter: &TextCutter,
) -> Result<(), messages::CuttingError> {
    info!(
        "{}: Processing discovered material '{}' at path '{}'",
        actor_name,
        material_id.as_str(),
        file_path
    );

    // Check if the material exists in the repository
    let _material = match registry.get_material(material_id.as_str()).await {
        Some(material) => {
            debug!(
                "{}: Found material '{}' in repository with state '{:?}'",
                actor_name,
                material_id.as_str(),
                material.status
            );
            material
        }
        None => {
            error!(
                "{}: Failed to retrieve material '{}': Not found",
                actor_name,
                material_id.as_str()
            );

            // Publish error event
            let error_event = QuiltEvent::create_processing_error_event(
                material_id.as_str(),
                ProcessingStage::Cutting,
                &format!(
                    "Material not found during cutting stage: {}",
                    material_id.as_str()
                ),
            );

            if let Err(e) = registry.event_bus().publish(error_event) {
                warn!(
                    "{}: Failed to publish error event for material '{}': {}",
                    actor_name,
                    material_id.as_str(),
                    e
                );
            }

            return Err(messages::CuttingError::MaterialNotFound(material_id));
        }
    };

    // Read the file content
    let file_path = Path::new(&file_path);
    let content = fs::read_to_string(file_path).await.map_err(|e| {
        // Publish error event for file read failure
        let error_msg = format!("Failed to read file: {}", e);
        let error_event = QuiltEvent::create_processing_error_event(
            material_id.as_str(),
            ProcessingStage::Cutting,
            &error_msg,
        );
        let _ = registry.event_bus().publish(error_event);
        
        messages::CuttingError::FileError(e)
    })?;

    // Cut the material into chunks
    let cuts = cutter.cut(&content, Some(material_id.clone())).map_err(|e| {
        // Publish error event for cutting failure
        let error_msg = format!("Failed to cut material: {}", e);
        let error_event = QuiltEvent::create_processing_error_event(
            material_id.as_str(),
            ProcessingStage::Cutting,
            &error_msg,
        );
        let _ = registry.event_bus().publish(error_event);
        
        messages::CuttingError::OperationFailed(e.to_string())
    })?;

    // Log information about cuts
    let cut_count = cuts.len();
    info!(
        "{}: Successfully cut material '{}' into {} chunks",
        actor_name,
        material_id.as_str(),
        cut_count
    );

    // Log a sample of the cuts (first 2 and last 2 if there are more than 4 cuts)
    if cut_count > 0 {
        if cut_count <= 4 {
            // Log all cuts for small numbers
            for (i, cut) in cuts.iter().enumerate() {
                let preview = if cut.content.len() > 50 {
                    format!("{}...", &cut.content[..50])
                } else {
                    cut.content.clone()
                };
                
                info!(
                    "{}: Cut {}/{} - ID: {}, Sequence: {}, Content: '{}'",
                    actor_name,
                    i + 1,
                    cut_count,
                    cut.id,
                    cut.sequence,
                    preview
                );
            }
        } else {
            // Log first 2
            for i in 0..2 {
                let cut = &cuts[i];
                let preview = if cut.content.len() > 50 {
                    format!("{}...", &cut.content[..50])
                } else {
                    cut.content.clone()
                };
                
                info!(
                    "{}: Cut {}/{} - ID: {}, Sequence: {}, Content: '{}'",
                    actor_name,
                    i + 1,
                    cut_count,
                    cut.id,
                    cut.sequence,
                    preview
                );
            }
            
            info!("{}: ... {} more cuts ...", actor_name, cut_count - 4);
            
            // Log last 2
            for i in (cut_count - 2)..cut_count {
                let cut = &cuts[i];
                let preview = if cut.content.len() > 50 {
                    format!("{}...", &cut.content[..50])
                } else {
                    cut.content.clone()
                };
                
                info!(
                    "{}: Cut {}/{} - ID: {}, Sequence: {}, Content: '{}'",
                    actor_name,
                    i + 1,
                    cut_count,
                    cut.id,
                    cut.sequence,
                    preview
                );
            }
        }
    }

    // Update material status to Cut
    if let Err(e) = registry.update_material_status(material_id.as_str(), MaterialStatus::Cut, None).await {
        error!(
            "{}: Failed to update material status: {}",
            actor_name,
            e
        );
        
        // Publish error event
        let error_msg = format!("Failed to update material status: {}", e);
        let error_event = QuiltEvent::create_processing_error_event(
            material_id.as_str(),
            ProcessingStage::Cutting,
            &error_msg,
        );
        let _ = registry.event_bus().publish(error_event);
        
        return Err(messages::CuttingError::OperationFailed(e.to_string()));
    }

    // TODO: In the next milestone, store the cuts in the CutsRepository
    
    // Publish MaterialCut event
    let cut_event = QuiltEvent::material_cut(material_id.as_str(), cut_count);
    if let Err(e) = registry.event_bus().publish(cut_event) {
        warn!(
            "{}: Failed to publish MaterialCut event for material '{}': {}",
            actor_name,
            material_id.as_str(),
            e
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EventBus;
    use crate::materials::types::Material;
    use crate::materials::MaterialRepository;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::fs;
    use tempfile::tempdir;

    // Helper function to initialize test logger
    fn init_test_logger() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .try_init();
    }

    #[actix::test]
    async fn test_cutting_actor_ping() {
        init_test_logger();

        // Create registry with event bus
        let event_bus = Arc::new(EventBus::new());
        let repository = MaterialRepository::new();
        let registry = MaterialRegistry::new(repository, event_bus);

        // Create and start actor
        let actor = CuttingActor::new("TestCuttingActor", registry).start();

        // Send ping and check response
        let result = actor.send(Ping).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[actix::test]
    async fn test_cutting_actor_shutdown() {
        init_test_logger();

        // Create registry with event bus
        let event_bus = Arc::new(EventBus::new());
        let repository = MaterialRepository::new();
        let registry = MaterialRegistry::new(repository, event_bus);

        // Create and start actor
        let actor = CuttingActor::new("TestCuttingActor", registry).start();

        // Send shutdown
        let result = actor.send(Shutdown).await;
        assert!(result.is_ok());

        // Give the actor some time to shut down
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Try to ping, should fail because actor is stopped
        let ping_result = actor.send(Ping).await;
        assert!(ping_result.is_err());
    }

    #[actix::test]
    async fn test_cutting_actor_processes_discovered_material() {
        init_test_logger();

        // Create a temporary directory for test files
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test_file.md");
        let file_path_str = file_path.to_string_lossy().to_string();
        
        // Create a test file with content
        fs::write(&file_path, "# Test Document\n\nThis is a test document for cutting.").await.expect("Failed to write test file");

        // Create registry with event bus
        let event_bus = Arc::new(EventBus::new());
        let repository = MaterialRepository::new();
        let registry = MaterialRegistry::new(repository, event_bus.clone());

        // Create and start actor
        let cutting_actor = CuttingActor::new("TestCuttingActor", registry.clone()).start();

        // Give the actor time to set up
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Register a test material with the file path we created
        let material = Material::new(file_path_str.clone());
        let material_id = material.id.clone();
        registry.register_material(material).await.unwrap();

        // Create and publish a test event directly
        let material = registry.get_material(&material_id).await.unwrap();
        let event = QuiltEvent::material_discovered(&material);

        // Publish the event
        event_bus.publish(event).expect("Failed to publish event");

        // Wait for event processing
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Verify the actor is still alive
        let ping_result = cutting_actor.send(Ping).await;
        assert!(ping_result.is_ok());
        assert!(ping_result.unwrap());

        // Verify the material status was updated to Cut
        let updated_material = registry.get_material(&material_id).await.unwrap();
        assert_eq!(updated_material.status, MaterialStatus::Cut);
        
        // Clean up
        temp_dir.close().expect("Failed to clean up temp dir");
    }
}
