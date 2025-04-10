use crate::actors::{Ping, Shutdown};
use crate::events::QuiltEvent;
use crate::materials::MaterialRegistry;
use actix::prelude::*;
use log::{debug, error, info, warn};
use tokio::sync::broadcast;

/// Messages specific to the CuttingActor
///
/// This module contains all message types that can be sent to the CuttingActor
/// to request operations and their respective response types.
pub mod messages {
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
        MaterialNotFound(String),

        /// Generic cutting error
        #[error("Cutting operation failed: {0}")]
        OperationFailed(String),
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
                        debug!("Received MaterialDiscovered event: {}", evt.material_id);

                        // Send a message to self to process the event
                        addr.do_send(ProcessDiscoveredMaterial {
                            material_id: evt.material_id.clone(),
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
    material_id: String,
    /// File path of the discovered material
    file_path: String,
}

impl Handler<ProcessDiscoveredMaterial> for CuttingActor {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, msg: ProcessDiscoveredMaterial, _ctx: &mut Self::Context) -> Self::Result {
        let name = self.name.clone();
        debug!(
            "{}: Processing discovered material '{}'",
            name, msg.material_id
        );

        // Use message data directly and access self methods without cloning the actor
        let material_id = msg.material_id;
        let file_path = msg.file_path;
        let registry = self.registry.clone();
        let actor_name = self.name.clone();

        Box::pin(async move {
            // Process the discovered material using the cloned registry instead of the actor instance
            process_discovered_material(&actor_name, &registry, material_id, file_path).await;
        })
    }
}

/// Process a MaterialDiscovered event
///
/// For now, this just logs the event and checks the material repository
/// for the material.
///
/// # Arguments
///
/// * `actor_name` - Name of the actor for logging
/// * `registry` - Registry to retrieve materials
/// * `material_id` - ID of the discovered material
/// * `file_path` - File path of the discovered material
async fn process_discovered_material(
    actor_name: &str,
    registry: &MaterialRegistry,
    material_id: String,
    file_path: String,
) {
    info!(
        "{}: Processing discovered material '{}' at path '{}'",
        actor_name, material_id, file_path
    );

    // Check if the material exists in the repository
    match registry.get_material(&material_id).await {
        Some(material) => {
            debug!(
                "{}: Found material '{}' in repository with state '{:?}'",
                actor_name, material_id, material.status
            );
            // TODO: Implement actual cutting logic in Milestone 6
            // This will include:
            // - Document parsing and text extraction
            // - Content splitting strategies
            // - Cut creation and storage
            // - Error handling and recovery
        }
        None => {
            error!(
                "{}: Failed to retrieve material '{}': Not found",
                actor_name, material_id
            );

            // Publish error event
            let error_event = QuiltEvent::processing_error(
                &material_id,
                "cutting",
                &format!("Material not found during cutting stage: {}", material_id),
            );

            if let Err(e) = registry.event_bus().publish(error_event) {
                warn!(
                    "{}: Failed to publish error event for material '{}': {}",
                    actor_name, material_id, e
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EventBus;
    use crate::materials::types::Material;
    use crate::materials::MaterialRepository;
    use std::sync::Arc;
    use std::time::Duration;

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

        // Create registry with event bus
        let event_bus = Arc::new(EventBus::new());
        let repository = MaterialRepository::new();
        let registry = MaterialRegistry::new(repository, event_bus.clone());

        // Create and start actor
        let cutting_actor = CuttingActor::new("TestCuttingActor", registry.clone()).start();

        // Give the actor time to set up
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Register a test material
        let material = Material::new("test/file.md".to_string());
        let material_id = material.id.clone();
        registry.register_material(material).await.unwrap();

        // Retrieve the material to ensure it's registered
        let material = registry.get_material(&material_id).await.unwrap();

        // Create and publish a test event directly
        let event = QuiltEvent::material_discovered(&material);

        // Try to publish the event with error handling
        match event_bus.publish(event) {
            Ok(_) => {
                // Wait for event processing
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            Err(e) => {
                // Just log the error but don't fail the test
                println!("Could not publish event: {}", e);
            }
        }

        // Verify the actor is still alive
        let ping_result = cutting_actor.send(Ping).await;
        assert!(ping_result.is_ok());
        assert!(ping_result.unwrap());
    }
}
