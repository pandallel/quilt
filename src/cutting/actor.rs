use crate::actors::{Ping, Shutdown};
use crate::events::types::MaterialId;
use crate::events::QuiltEvent;
use crate::materials::types::MaterialStatus;
use crate::materials::MaterialRegistry;
use actix::prelude::*;
use actix::SpawnHandle;
use log::{debug, error, info, warn};
use std::path::Path;
use tokio::fs;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

use super::cutter::TextCutter;

/// Messages specific to the CuttingActor
///
/// This module contains all message types that can be sent to the CuttingActor
/// to request operations and their respective response types.
pub mod messages {
    use crate::cutting::cutter::text::CutterError;
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

        /// Cutting error
        #[error("Text cutting error: {0}")]
        CuttingError(#[from] CutterError),
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
    /// Text cutter with default configuration
    cutter: TextCutter,
    /// Sender for the internal work queue
    work_sender: Option<mpsc::Sender<ProcessDiscoveredMaterial>>,
    /// Handle for the listener task
    listener_handle: Option<SpawnHandle>,
    /// Handle for the processor task
    processor_handle: Option<SpawnHandle>,
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
            cutter: TextCutter::default(),
            work_sender: None,
            listener_handle: None,
            processor_handle: None,
        }
    }
}

impl Actor for CuttingActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("{}: Started", self.name);

        const INTERNAL_QUEUE_CAPACITY: usize = 32;
        let (work_sender, work_receiver) =
            mpsc::channel::<ProcessDiscoveredMaterial>(INTERNAL_QUEUE_CAPACITY);
        self.work_sender = Some(work_sender.clone());

        let bus_receiver = self.registry.event_bus().subscribe();
        let actor_name = self.name.clone();
        let registry = self.registry.clone();
        let cutter = self.cutter.clone();

        let listener_actor_name = actor_name.clone();
        let listener_handle = ctx.spawn(
            async move {
                info!("{}: Listener task started", listener_actor_name);
                let mut bus_receiver = bus_receiver;
                let work_sender = work_sender;

                loop {
                    match bus_receiver.recv().await {
                        Ok(event) => {
                            if let QuiltEvent::MaterialDiscovered(evt) = event {
                                debug!(
                                    "{}: Listener received MaterialDiscovered: {}",
                                    listener_actor_name,
                                    evt.material_id.as_str()
                                );
                                let work_item = ProcessDiscoveredMaterial {
                                    material_id: evt.material_id,
                                    file_path: evt.file_path.clone(),
                                };
                                if let Err(e) = work_sender.send(work_item).await {
                                    error!(
                                        "{}: Listener failed to send work item to processor: {}",
                                        listener_actor_name, e
                                    );
                                    break;
                                }
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            warn!(
                                "{}: Listener lagged behind {} events.",
                                listener_actor_name, n
                            );
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            info!(
                                "{}: Listener stopping as event bus channel closed.",
                                listener_actor_name
                            );
                            break;
                        }
                    }
                }
                info!("{}: Listener task finished", listener_actor_name);
            }
            .into_actor(self),
        );
        self.listener_handle = Some(listener_handle);

        let processor_actor_name = actor_name.clone();
        let processor_handle = ctx.spawn(
            async move {
                info!("{}: Processor task started", processor_actor_name);
                let mut work_receiver = work_receiver;
                let registry = registry;
                let cutter = cutter;
                let actor_name = processor_actor_name;

                while let Some(work_item) = work_receiver.recv().await {
                    debug!(
                        "{}: Processor received work item for: {}",
                        actor_name,
                        work_item.material_id.as_str()
                    );
                    if let Err(e) = process_discovered_material(
                        &actor_name,
                        &registry,
                        work_item.material_id,
                        work_item.file_path,
                        &cutter,
                    )
                    .await
                    {
                        error!("{}: Error processing material: {}", actor_name, e);
                    }
                }
                info!(
                    "{}: Processor task finished as work queue closed",
                    actor_name
                );
            }
            .into_actor(self),
        );
        self.processor_handle = Some(processor_handle);
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        info!("{}: Stopping", self.name);
        self.work_sender.take();

        if let Some(_handle) = self.listener_handle.take() {
            debug!("{}: Listener task will be stopped", self.name);
        }
        if let Some(_handle) = self.processor_handle.take() {
            debug!("{}: Processor task will be stopped", self.name);
        }
        Running::Stop
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
///
/// This is no longer an Actix message
/// It's just a struct passed via the mpsc channel.
struct ProcessDiscoveredMaterial {
    /// ID of the discovered material
    material_id: MaterialId,
    /// File path of the discovered material
    file_path: String,
}

/// Process a MaterialDiscovered event (called by the processor task)
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

    let material = match registry.get_material(material_id.as_str()).await {
        Some(m) => m,
        None => {
            error!(
                "{}: Failed to retrieve material '{}': Not found",
                actor_name,
                material_id.as_str()
            );
            return Err(messages::CuttingError::MaterialNotFound(material_id));
        }
    };

    if material.status != MaterialStatus::Discovered {
        warn!(
            "{}: Skipping processing for material '{}', status is {:?} (expected Discovered)",
            actor_name,
            material_id.as_str(),
            material.status
        );
        return Ok(());
    }

    let content =
        match fs::read_to_string(Path::new(&file_path)).await {
            Ok(content) => {
                debug!(
                    "{}: Read {} bytes from file '{}'",
                    actor_name,
                    content.len(),
                    file_path
                );
                content
            }
            Err(e) => {
                error!("{}: Failed to read file '{}': {}", actor_name, file_path, e);

                if let Err(update_err) = registry
                    .update_material_status(
                        material_id.as_str(),
                        MaterialStatus::Error,
                        Some(format!("Failed to read file: {}", e)),
                    )
                    .await
                {
                    error!(
                    "{}: Failed to update material status to Error for '{}' after read failure: {}",
                    actor_name,
                    material_id.as_str(),
                    update_err
                );
                }

                return Err(messages::CuttingError::FileError(e));
            }
        };

    let chunks = cutter.cut(&content, Some(material_id.clone()))?;
    debug!(
        "{}: Cut material '{}' into {} chunks",
        actor_name,
        material_id.as_str(),
        chunks.len()
    );

    if let Some(first_chunk) = chunks.first() {
        debug!(
            "{}: First chunk sample ({} chars): '{}...'",
            actor_name,
            first_chunk.content.chars().count(),
            first_chunk.content.chars().take(50).collect::<String>()
        );
    }

    debug!(
        "{}: Updating status to Cut for material '{}'",
        actor_name,
        material_id.as_str()
    );
    if let Err(e) = registry
        .update_material_status(material_id.as_str(), MaterialStatus::Cut, None)
        .await
    {
        error!(
            "{}: Failed to update material status to Cut for '{}': {}",
            actor_name,
            material_id.as_str(),
            e
        );
    } else {
        info!(
            "{}: Successfully processed and marked material '{}' as Cut",
            actor_name,
            material_id.as_str()
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
    use tempfile::tempdir;
    use tokio::fs;

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
        fs::write(
            &file_path,
            "# Test Document\n\nThis is a test document for cutting.",
        )
        .await
        .expect("Failed to write test file");

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
