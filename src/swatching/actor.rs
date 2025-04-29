use crate::actors::{Ping, Shutdown};
use crate::cutting::CutsRepository;
use crate::events::types::MaterialId;
use crate::events::QuiltEvent;
use crate::materials::MaterialRegistry;
use actix::prelude::*;
use actix::SpawnHandle;
use log::{debug, error, info, warn};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

use super::embedding::EmbeddingService;
use super::repository::SwatchRepository;

/// Messages specific to the SwatchingActor
///
/// This module contains all message types that can be sent to the SwatchingActor
/// to request operations and their respective response types.
pub mod messages {
    use crate::events::types::MaterialId;
    use actix::prelude::*;
    use thiserror::Error;

    /// Swatching operation errors
    ///
    /// These errors can occur during swatching operations and provide
    /// detailed information about what went wrong.
    #[derive(Debug, Error)]
    pub enum SwatchingError {
        /// Material not found error
        #[error("Material not found: {0}")]
        MaterialNotFound(MaterialId),

        /// Cuts not found error
        #[error("Cuts not found for material: {0}")]
        CutsNotFound(MaterialId),

        /// Generic swatching error
        #[error("Swatching operation failed: {0}")]
        OperationFailed(Box<str>),
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

/// Actor responsible for processing cut materials into swatches
///
/// The SwatchingActor subscribes to MaterialCut events and processes
/// the cut materials to create semantic embeddings.
///
/// # Message Handlers
///
/// * `Ping` - Responds with `true` to indicate the actor is alive
/// * `Shutdown` - Gracefully shuts down the actor
pub struct SwatchingActor {
    /// Name of this actor instance for logging
    name: String,
    /// Event bus to subscribe to events
    event_bus: Arc<crate::events::EventBus>,
    /// Swatch repository for persistence
    swatch_repository: Arc<dyn SwatchRepository>,
    /// Cuts repository for retrieving cut content
    cuts_repository: Arc<dyn CutsRepository>,
    /// Embedding service for generating embeddings
    embedding_service: Arc<dyn EmbeddingService>,
    /// Material registry for updating status
    registry: MaterialRegistry,
    /// Sender for the internal work queue
    work_sender: Option<mpsc::Sender<SwatchingWorkItem>>,
    /// Handle for the listener task
    listener_handle: Option<SpawnHandle>,
    /// Handle for the processor task
    processor_handle: Option<SpawnHandle>,
}

impl SwatchingActor {
    /// Create a new SwatchingActor with the given name, event bus, and repositories
    ///
    /// # Arguments
    ///
    /// * `name` - Name for this actor instance, used in logging
    /// * `event_bus` - Event bus to subscribe to events
    /// * `cuts_repository` - Repository for retrieving cuts
    /// * `embedding_service` - Service for generating embeddings
    /// * `swatch_repository` - Repository for swatch persistence
    /// * `registry` - Material registry for updating status
    pub fn new(
        name: &str,
        event_bus: Arc<crate::events::EventBus>,
        cuts_repository: Arc<dyn CutsRepository>,
        embedding_service: Arc<dyn EmbeddingService>,
        swatch_repository: Arc<dyn SwatchRepository>,
        registry: MaterialRegistry,
    ) -> Self {
        Self {
            name: name.to_string(),
            event_bus,
            cuts_repository,
            embedding_service,
            swatch_repository,
            registry,
            work_sender: None,
            listener_handle: None,
            processor_handle: None,
        }
    }
}

impl Actor for SwatchingActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("{}: Started", self.name);

        const INTERNAL_QUEUE_CAPACITY: usize = 128;
        let (work_sender, work_receiver) =
            mpsc::channel::<SwatchingWorkItem>(INTERNAL_QUEUE_CAPACITY);
        self.work_sender = Some(work_sender.clone());

        let bus_receiver = self.event_bus.subscribe();
        let actor_name = self.name.clone();

        // Clone repositories and services for the processor task
        let swatch_repo_clone = self.swatch_repository.clone();
        let cuts_repo_clone = self.cuts_repository.clone();
        let embedding_service_clone = self.embedding_service.clone();
        let registry_clone = self.registry.clone();

        let listener_actor_name = actor_name.clone();
        let listener_handle = ctx.spawn(
            async move {
                info!("{}: Listener task started", listener_actor_name);
                let mut bus_receiver = bus_receiver;
                let work_sender = work_sender;

                loop {
                    match bus_receiver.recv().await {
                        Ok(event) => {
                            if let QuiltEvent::MaterialCut(evt) = event {
                                debug!(
                                    "{}: Listener received MaterialCut: {}",
                                    listener_actor_name,
                                    evt.material_id.as_str()
                                );
                                let work_item = SwatchingWorkItem {
                                    material_id: evt.material_id,
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
                let actor_name = processor_actor_name;
                
                // Use the cloned dependencies
                let swatch_repository = swatch_repo_clone;
                let cuts_repository = cuts_repo_clone;
                let embedding_service = embedding_service_clone;
                let registry = registry_clone;

                while let Some(work_item) = work_receiver.recv().await {
                    let material_id_str = work_item.material_id.as_str();
                    debug!(
                        "{}: Processor received work item for: {}",
                        actor_name, material_id_str
                    );

                    // --- Temporary Read/Log to resolve dead_code --- //
                    debug!(
                        "{}: Checking for existing swatches for material {}",
                        actor_name, material_id_str
                    );
                    let check_result = swatch_repository
                        .get_swatches_by_material_id(material_id_str)
                        .await;
                    match check_result {
                        Ok(swatches) => {
                            debug!(
                                "{}: Found {} existing swatches for material {}",
                                actor_name,
                                swatches.len(),
                                material_id_str
                            );
                        }
                        Err(e) => {
                            warn!(
                                "{}: Failed to check swatches for material {}: {}",
                                actor_name, material_id_str, e
                            );
                        }
                    }
                    // --- End Temporary Read/Log --- //

                    // Original logging (can be kept or removed)
                    info!(
                        "{}: Received material to process for swatching: {}",
                        actor_name,
                        work_item.material_id.as_str()
                    );
                }
                info!("{}: Processor task finished", actor_name);
            }
            .into_actor(self),
        );
        self.processor_handle = Some(processor_handle);
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        info!("{}: Stopping", self.name);

        // Closing mpsc channel will cause processor task to complete
        self.work_sender = None;

        Running::Stop
    }
}

impl Handler<Ping> for SwatchingActor {
    type Result = bool;

    fn handle(&mut self, _msg: Ping, _ctx: &mut Self::Context) -> Self::Result {
        debug!("{}: Received ping", self.name);
        true
    }
}

impl Handler<Shutdown> for SwatchingActor {
    type Result = ();

    fn handle(&mut self, _msg: Shutdown, ctx: &mut Self::Context) -> Self::Result {
        info!("{}: Received shutdown request", self.name);
        ctx.stop();
    }
}

/// Internal work item for the SwatchingActor's processor
struct SwatchingWorkItem {
    /// ID of the material to process
    material_id: MaterialId,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EventBus;
    use crate::swatching::repository::MockSwatchRepository;
    use crate::swatching::SwatchRepositoryError;
    use std::sync::Arc;
    use std::time::Duration;
    use time::OffsetDateTime;

    fn init_test_logger() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .try_init();
    }

    #[actix::test]
    async fn test_swatching_actor_ping() {
        init_test_logger();
        let event_bus = Arc::new(EventBus::new());
        let mock_repo = MockSwatchRepository::new();
        let repo: Arc<dyn SwatchRepository> = Arc::new(mock_repo);
        let actor = SwatchingActor::new("test-swatching", event_bus, repo).start();

        let result = actor.send(Ping).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[actix::test]
    async fn test_swatching_actor_shutdown() {
        init_test_logger();
        let event_bus = Arc::new(EventBus::new());
        let mock_repo = MockSwatchRepository::new();
        let repo: Arc<dyn SwatchRepository> = Arc::new(mock_repo);
        let actor = SwatchingActor::new("test-swatching", event_bus, repo).start();

        let result = actor.send(Shutdown).await;
        assert!(result.is_ok());

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let ping_result = actor.send(Ping).await;
        assert!(ping_result.is_err());
    }

    #[actix::test]
    async fn test_swatching_actor_processes_item() {
        init_test_logger();
        let event_bus = Arc::new(EventBus::new());
        let test_material_id = MaterialId::from("test_mat_id");
        let id_for_closure = test_material_id.clone();

        let mut mock_repo = MockSwatchRepository::new();
        mock_repo
            .expect_get_swatches_by_material_id()
            .withf(move |id| id == id_for_closure.as_str())
            .times(1)
            .returning(|_| Ok(Vec::new()));

        let repo: Arc<dyn SwatchRepository> = Arc::new(mock_repo);
        let _actor = SwatchingActor::new("test-processor", event_bus.clone(), repo).start();

        let cut_event = QuiltEvent::MaterialCut(crate::events::types::MaterialCutEvent {
            material_id: test_material_id.clone(),
            timestamp: OffsetDateTime::now_utc(),
        });
        let _ = event_bus.publish(cut_event);

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[actix::test]
    async fn test_swatching_actor_handles_repo_error() {
        init_test_logger();
        let event_bus = Arc::new(EventBus::new());
        let test_material_id = MaterialId::from("test_mat_id_err");
        let id_for_closure = test_material_id.clone();

        let mut mock_repo = MockSwatchRepository::new();
        mock_repo
            .expect_get_swatches_by_material_id()
            .withf(move |id| id == id_for_closure.as_str())
            .times(1)
            .returning(|id| {
                Err(SwatchRepositoryError::OperationFailed(
                    format!("Test error checking swatches for {}", id).into(),
                ))
            });

        let repo: Arc<dyn SwatchRepository> = Arc::new(mock_repo);
        let _actor = SwatchingActor::new("test-error", event_bus.clone(), repo).start();

        let cut_event = QuiltEvent::MaterialCut(crate::events::types::MaterialCutEvent {
            material_id: test_material_id.clone(),
            timestamp: OffsetDateTime::now_utc(),
        });
        let _ = event_bus.publish(cut_event);

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
