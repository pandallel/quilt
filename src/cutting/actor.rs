use crate::actors::{Ping, Shutdown};
use crate::events::types::MaterialId;
use crate::events::QuiltEvent;
use crate::materials::types::MaterialStatus;
use crate::materials::MaterialRegistry;
use actix::prelude::*;
use actix::SpawnHandle;
use log::{debug, error, info, warn};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

use super::cutter::TextCutter;
use super::{Cut, CutsRepository};

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
        OperationFailed(Box<str>),

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
    /// Repository for storing cuts
    cuts_repository: Arc<dyn CutsRepository>,
    /// Sender for the internal work queue
    work_sender: Option<mpsc::Sender<CuttingWorkItem>>,
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
    /// * `cuts_repository` - Repository for storing cut chunks
    pub fn new(
        name: &str,
        registry: MaterialRegistry,
        cuts_repository: Arc<dyn CutsRepository>,
    ) -> Self {
        Self {
            name: name.to_string(),
            registry,
            cutter: TextCutter::default(),
            cuts_repository,
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

        const INTERNAL_QUEUE_CAPACITY: usize = 128;
        let (work_sender, work_receiver) =
            mpsc::channel::<CuttingWorkItem>(INTERNAL_QUEUE_CAPACITY);
        self.work_sender = Some(work_sender.clone());

        let bus_receiver = self.registry.event_bus().subscribe();
        let actor_name = self.name.clone();
        let registry = self.registry.clone();
        let cutter = self.cutter.clone();
        let cuts_repository = self.cuts_repository.clone();

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
                                let work_item = CuttingWorkItem {
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
                let cuts_repository = cuts_repository;
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
                        work_item.material_id.clone(),
                        work_item.file_path,
                        &cutter,
                        cuts_repository.clone(),
                    )
                    .await
                    {
                        error!("{}: Error processing material: {}", actor_name, e);
                        // Handle material not found error by updating status to Error
                        if let messages::CuttingError::MaterialNotFound(material_id) = &e {
                            if let Err(update_err) = registry
                                .update_material_status(
                                    material_id.as_str(),
                                    MaterialStatus::Error,
                                    Some(format!(
                                        "Material not found for cutting: {}",
                                        material_id
                                    )),
                                )
                                .await
                            {
                                error!(
                                    "{}: Failed to update material status to Error for '{}': {}",
                                    actor_name,
                                    material_id.as_str(),
                                    update_err
                                );
                            }
                        } else {
                            // For all other errors, update the status of the material
                            let material_id = work_item.material_id.clone();
                            if let Err(update_err) = registry
                                .update_material_status(
                                    material_id.as_str(),
                                    MaterialStatus::Error,
                                    Some(format!("Error during cutting: {}", e)),
                                )
                                .await
                            {
                                error!(
                                    "{}: Failed to update material status to Error for '{}': {}",
                                    actor_name,
                                    material_id.as_str(),
                                    update_err
                                );
                            }
                        }
                    }
                }
                info!("{}: Processor task finished", actor_name);
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
/// This is a struct passed via the mpsc channel within the CuttingActor
/// to move work from the listener task to the processor task.
struct CuttingWorkItem {
    /// ID of the discovered material
    material_id: MaterialId,
    /// File path of the discovered material
    file_path: String,
}

/// Worker function to process a discovered material
/// This is a separate function to make the processing code easier to test
///
/// # Arguments
///
/// * `actor_name` - Name of the actor, for logging
/// * `registry` - Registry to retrieve and update materials
/// * `material_id` - ID of the discovered material to process
/// * `file_path` - Path to the material file
/// * `cutter` - Text cutter to use for processing
/// * `cuts_repository` - Repository for storing cut chunks
///
/// # Returns
///
/// * `Result<(), CuttingError>` - Success or error
pub async fn process_discovered_material(
    actor_name: &str,
    registry: &MaterialRegistry,
    material_id: MaterialId,
    file_path: String,
    cutter: &TextCutter,
    cuts_repository: Arc<dyn CutsRepository>,
) -> Result<(), messages::CuttingError> {
    info!(
        "{}: Processing discovered material: {}",
        actor_name,
        material_id.as_str()
    );

    // Look up the material from the registry
    let material = registry
        .get_material(material_id.as_str())
        .await
        .ok_or_else(|| messages::CuttingError::MaterialNotFound(material_id.clone()))?;

    // Skip if the material is not in Discovered status
    if material.status != MaterialStatus::Discovered {
        info!(
            "{}: Skipping material {} with status {:?} (not Discovered)",
            actor_name,
            material_id.as_str(),
            material.status
        );
        return Ok(());
    }

    info!(
        "{}: Retrieved material '{}' with path: {}",
        actor_name,
        material_id.as_str(),
        material.file_path
    );

    // Read the content from the file
    debug!("{}: Reading file content: {}", actor_name, file_path);
    let content = tokio::fs::read_to_string(&file_path).await.map_err(|e| {
        error!("{}: Failed to read file '{}': {}", actor_name, file_path, e);
        messages::CuttingError::FileError(e)
    })?;

    // Cut the content into chunks
    debug!(
        "{}: Cutting content into chunks for: {}",
        actor_name,
        material_id.as_str()
    );
    let chunks = cutter
        .cut(&content, Some(material_id.clone()))
        .map_err(|e| {
            error!("{}: Failed to cut content: {}", actor_name, e);
            messages::CuttingError::CuttingError(e)
        })?;

    debug!("{}: Cut material into {} chunks", actor_name, chunks.len());

    // Convert chunks to Cut objects
    let cuts: Vec<Cut> = chunks
        .iter()
        .map(|chunk| {
            Cut::with_details(
                material_id.to_string(),
                chunk.sequence,
                chunk.content.clone(),
                Some(cutter.config().get_token_count(&chunk.content)),
                None, // Byte offsets aren't available from TextCutter currently
                None,
            )
        })
        .collect();

    // Store the cuts in the repository
    debug!("{}: Saving {} cuts to repository", actor_name, cuts.len());
    cuts_repository.save_cuts(&cuts).await.map_err(|e| {
        error!(
            "{}: Failed to save cuts to repository: {}",
            actor_name,
            e
        );
        messages::CuttingError::OperationFailed(e.to_string().into_boxed_str())
    })?;

    // Update the material status to Cut
    debug!(
        "{}: Updating material status to Cut: {}",
        actor_name,
        material_id.as_str()
    );
    registry
        .update_material_status(material_id.as_str(), MaterialStatus::Cut, None)
        .await
        .map_err(|e| {
            error!(
                "{}: Failed to update material status: {}",
                actor_name,
                e
            );
            messages::CuttingError::OperationFailed(e.to_string().into_boxed_str())
        })?;

    info!(
        "{}: Successfully processed material: {}",
        actor_name,
        material_id.as_str()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cutting::InMemoryCutsRepository;
    use crate::events::EventBus;
    use crate::materials::InMemoryMaterialRepository;
    use crate::materials::Material;
    use crate::materials::MaterialStatus;
    use std::fs;
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::tempdir;

    fn init_test_logger() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .try_init();
    }

    #[actix::test]
    async fn test_cutting_actor_ping() {
        init_test_logger();

        let event_bus = Arc::new(EventBus::new());
        let repository = Arc::new(InMemoryMaterialRepository::new());
        let registry = MaterialRegistry::new(repository, event_bus);
        let cuts_repository = Arc::new(InMemoryCutsRepository::new());

        let actor = CuttingActor::new("test-cutter", registry, cuts_repository).start();

        let response = actor.send(Ping).await.unwrap();
        assert!(response, "Ping should return true");
    }

    #[actix::test]
    async fn test_cutting_actor_shutdown() {
        init_test_logger();

        let event_bus = Arc::new(EventBus::new());
        let repository = Arc::new(InMemoryMaterialRepository::new());
        let registry = MaterialRegistry::new(repository, event_bus);
        let cuts_repository = Arc::new(InMemoryCutsRepository::new());

        let actor = CuttingActor::new("TestCuttingActor", registry, cuts_repository).start();

        let result = actor.send(Shutdown).await;
        assert!(result.is_ok());

        tokio::time::sleep(Duration::from_millis(50)).await;

        let ping_result = actor.send(Ping).await;
        assert!(ping_result.is_err());
    }

    #[actix::test]
    async fn test_cutting_actor_processes_material() {
        init_test_logger();

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test_file.md");
        let file_path_str = file_path.to_string_lossy().to_string();

        fs::write(
            &file_path,
            "# Test Document\n\nThis is a test document for cutting.",
        )
        .expect("Failed to write test file");

        let event_bus = Arc::new(EventBus::new());
        let repository = Arc::new(InMemoryMaterialRepository::new());
        let registry = MaterialRegistry::new(repository, event_bus.clone());
        let cuts_repository = Arc::new(InMemoryCutsRepository::new());

        let cutting_actor =
            CuttingActor::new("TestCuttingActor", registry.clone(), cuts_repository).start();

        tokio::time::sleep(Duration::from_millis(50)).await;

        let material = Material::new(file_path_str.clone());
        let material_id = material.id.clone();
        registry.register_material(material).await.unwrap();

        let material = registry.get_material(&material_id).await.unwrap();
        let event = QuiltEvent::material_discovered(&material);

        event_bus.publish(event).expect("Failed to publish event");

        tokio::time::sleep(Duration::from_millis(200)).await;

        let ping_result = cutting_actor.send(Ping).await;
        assert!(ping_result.is_ok());
        assert!(ping_result.unwrap());

        let updated_material = registry.get_material(&material_id).await.unwrap();
        assert_eq!(updated_material.status, MaterialStatus::Cut);

        temp_dir.close().expect("Failed to clean up temp dir");
    }

    #[actix::test]
    async fn test_cutting_actor_stores_multiple_cuts() {
        init_test_logger();

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test_cuts_file.md");
        let file_path_str = file_path.to_string_lossy().to_string();

        fs::write(
            &file_path,
            "# Test Document For Cuts\n\n".repeat(30) + "This is a test document for cutting with enough text to create multiple cuts based on token size.",
        )
        .expect("Failed to write test file");

        let event_bus = Arc::new(EventBus::new());
        let repository = Arc::new(InMemoryMaterialRepository::new());
        let registry = MaterialRegistry::new(repository, event_bus.clone());
        let cuts_repository = Arc::new(InMemoryCutsRepository::new());

        let cutting_actor = CuttingActor::new(
            "TestCuttingActor",
            registry.clone(),
            cuts_repository.clone(),
        )
        .start();

        tokio::time::sleep(Duration::from_millis(50)).await;

        let material = Material::new(file_path_str.clone());
        let material_id = material.id.clone();
        registry.register_material(material).await.unwrap();

        let material = registry.get_material(&material_id).await.unwrap();
        let event = QuiltEvent::material_discovered(&material);

        event_bus.publish(event).expect("Failed to publish event");

        tokio::time::sleep(Duration::from_millis(200)).await;

        let ping_result = cutting_actor.send(Ping).await;
        assert!(ping_result.is_ok());
        assert!(ping_result.unwrap());

        let updated_material = registry.get_material(&material_id).await.unwrap();
        assert_eq!(updated_material.status, MaterialStatus::Cut);

        let cuts = cuts_repository
            .get_cuts_by_material_id(&material_id)
            .await
            .expect("Failed to get cuts from repository");

        assert!(!cuts.is_empty(), "Expected at least one cut to be saved");

        for (i, cut) in cuts.iter().enumerate() {
            assert_eq!(cut.material_id, material_id);
            assert_eq!(cut.chunk_index, i);
            assert!(!cut.content.is_empty());
        }

        temp_dir.close().expect("Failed to clean up temp dir");
    }
}
