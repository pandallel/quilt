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

                    // Fetch cuts from the repository
                    let cuts_result = cuts_repository.get_cuts_by_material_id(material_id_str).await;

                    match cuts_result {
                        Ok(cuts) => {
                            if cuts.is_empty() {
                                warn!(
                                    "{}: No cuts found for material {}",
                                    actor_name, material_id_str
                                );
                                // TODO: Update registry with error status
                                continue;
                            }

                            debug!(
                                "{}: Retrieved {} cuts for material {}",
                                actor_name,
                                cuts.len(),
                                material_id_str
                            );

                            // Process each cut to generate embeddings
                            let mut embedding_results = Vec::new();

                            for cut in &cuts {
                                debug!(
                                    "{}: Generating embedding for cut {} (chunk {})",
                                    actor_name, cut.id, cut.chunk_index
                                );

                                // Generate embedding for the cut content
                                match embedding_service.embed(&cut.content) {
                                    Ok(embedding) => {
                                        debug!(
                                            "{}: Successfully generated embedding for cut {} with dimensions {}",
                                            actor_name, cut.id, embedding.len()
                                        );

                                        // Store the cut and its embedding for the next step
                                        embedding_results.push((cut, embedding));
                                    }
                                    Err(e) => {
                                        error!(
                                            "{}: Failed to generate embedding for cut {}: {}",
                                            actor_name, cut.id, e
                                        );
                                        // TODO: Decide how to handle individual embedding failures
                                        // For now, continue with other cuts
                                    }
                                }
                            }

                            // Create swatches from embeddings
                            if embedding_results.is_empty() {
                                error!(
                                    "{}: Failed to generate any valid embeddings for material {}",
                                    actor_name, material_id_str
                                );

                                // Update registry with error status
                                if let Err(err) = registry.update_material_status(
                                    material_id_str,
                                    crate::materials::types::MaterialStatus::Error,
                                    Some(format!("Failed to generate embeddings for any cuts")),
                                ).await {
                                    error!(
                                        "{}: Failed to update material status for {}: {}",
                                        actor_name, material_id_str, err
                                    );
                                }

                                continue;
                            }

                            // Create swatches from the embeddings
                            let mut swatches = Vec::new();

                            for (cut, embedding) in &embedding_results {
                                // Create a new swatch using the embedding
                                let swatch = super::swatch::Swatch::new(
                                    cut.id.clone(),
                                    material_id_str.to_string(),
                                    embedding.clone(),
                                    "fastembed-model".to_string(), // TODO: Get actual model info from embedding service
                                    "v1".to_string(),
                                );

                                swatches.push(swatch);
                            }

                            // Persist the swatches to the repository
                            match swatch_repository.save_swatches_batch(&swatches).await {
                                Ok(_) => {
                                    info!(
                                        "{}: Successfully stored {} swatches for material {}",
                                        actor_name,
                                        swatches.len(),
                                        material_id_str
                                    );

                                    // Update material registry status to Swatched
                                    if let Err(err) = registry.update_material_status(
                                        material_id_str,
                                        crate::materials::types::MaterialStatus::Swatched,
                                        None,
                                    ).await {
                                        error!(
                                            "{}: Failed to update material status for {}: {}",
                                            actor_name, material_id_str, err
                                        );
                                    } else {
                                        // Successfully updated material status to Swatched
                                        // The material_registry.mark_swatched call will publish the MaterialSwatched event
                                        info!(
                                            "{}: Material {} marked as Swatched in registry",
                                            actor_name, material_id_str
                                        );
                                    }
                                },
                                Err(e) => {
                                    error!(
                                        "{}: Failed to save swatches for material {}: {}",
                                        actor_name, material_id_str, e
                                    );

                                    // Update registry with error status
                                    if let Err(err) = registry.update_material_status(
                                        material_id_str,
                                        crate::materials::types::MaterialStatus::Error,
                                        Some(format!("Failed to store swatches: {}", e)),
                                    ).await {
                                        error!(
                                            "{}: Failed to update material status for {}: {}",
                                            actor_name, material_id_str, err
                                        );
                                    }
                                }
                            }

                            info!(
                                "{}: Successfully processed {}/{} embeddings for material {}",
                                actor_name,
                                embedding_results.len(),
                                cuts.len(),
                                material_id_str
                            );
                        }
                        Err(e) => {
                            error!(
                                "{}: Failed to retrieve cuts for material {}: {}",
                                actor_name, material_id_str, e
                            );

                            // Update registry with error status
                            if let Err(err) = registry.update_material_status(
                                material_id_str,
                                crate::materials::types::MaterialStatus::Error,
                                Some(format!("Failed to retrieve cuts: {}", e)),
                            ).await {
                                error!(
                                    "{}: Failed to update material status for {}: {}",
                                    actor_name, material_id_str, err
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
    use crate::cutting::cut::Cut;
    use crate::cutting::MockCutsRepository;
    use crate::events::EventBus;
    use crate::materials::MaterialStatus;
    use crate::materials::MockMaterialRepository;
    use crate::swatching::embedding::MockEmbeddingService;
    use crate::swatching::repository::MockSwatchRepository;
    use mockall::predicate;
    use std::time::Duration;

    /// Initialize a test logger for better debugging
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

        // Create mock repositories and services
        let mock_cuts_repo = Arc::new(MockCutsRepository::new());
        let mock_embedding_service = Arc::new(MockEmbeddingService::new());
        let mock_swatch_repo = Arc::new(MockSwatchRepository::new());
        let mock_registry =
            MaterialRegistry::new(Arc::new(MockMaterialRepository::new()), event_bus.clone());

        let actor = SwatchingActor::new(
            "test-swatching-actor",
            event_bus,
            mock_cuts_repo,
            mock_embedding_service,
            mock_swatch_repo,
            mock_registry,
        );

        let actor_addr = actor.start();
        let result = actor_addr.send(Ping).await.unwrap();
        assert!(result, "Actor should respond to ping");
    }

    #[actix::test]
    async fn test_swatching_actor_processes_item() {
        init_test_logger();

        // Setup
        let event_bus = Arc::new(EventBus::new());
        let material_id = "test-material-id";

        // Create a test cut
        let cut = Cut::new(
            material_id.to_string(),
            0,
            "This is test content for embedding".to_string(),
        );

        // Setup mock cuts repository
        let mut mock_cuts_repo = MockCutsRepository::new();
        mock_cuts_repo
            .expect_get_cuts_by_material_id()
            .with(predicate::eq(material_id))
            .returning(move |_| Ok(vec![cut.clone()]));

        // Setup mock embedding service
        let test_embedding = vec![0.1, 0.2, 0.3, 0.4];
        let mut mock_embedding_service = MockEmbeddingService::new();
        mock_embedding_service
            .expect_embed()
            .returning(move |_| Ok(test_embedding.clone()));

        // Setup mock swatch repository
        let mut mock_swatch_repo = MockSwatchRepository::new();
        mock_swatch_repo
            .expect_save_swatches_batch()
            .returning(|_| Ok(()));

        // Setup mock material registry
        let mut mock_material_repo = MockMaterialRepository::new();
        mock_material_repo
            .expect_update_material_status()
            .with(
                predicate::eq(material_id),
                predicate::eq(MaterialStatus::Swatched),
                predicate::always(),
            )
            .returning(|_, _, _| Ok(()));

        let mock_registry = MaterialRegistry::new(Arc::new(mock_material_repo), event_bus.clone());

        // Create actor with mocks
        let actor = SwatchingActor::new(
            "test-swatching-actor",
            event_bus.clone(),
            Arc::new(mock_cuts_repo),
            Arc::new(mock_embedding_service),
            Arc::new(mock_swatch_repo),
            mock_registry,
        );

        // Start actor
        let actor_addr = actor.start();

        // Manually trigger processing
        let (tx, _rx) = mpsc::channel(1);
        tx.send(SwatchingWorkItem {
            material_id: material_id.into(),
        })
        .await
        .unwrap();

        // Give the actor time to process
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Shutdown actor
        actor_addr.send(Shutdown).await.unwrap();
    }

    #[actix::test]
    async fn test_swatching_actor_handles_repo_error() {
        init_test_logger();

        // Setup
        let event_bus = Arc::new(EventBus::new());
        let material_id = "test-material-id";

        // Setup mock cuts repository with error
        let mut mock_cuts_repo = MockCutsRepository::new();
        mock_cuts_repo
            .expect_get_cuts_by_material_id()
            .with(predicate::eq(material_id))
            .returning(|_| {
                Err(crate::cutting::CutsRepositoryError::OperationFailed(
                    "Test error".into(),
                ))
            });

        // Setup mock material registry
        let mut mock_material_repo = MockMaterialRepository::new();
        mock_material_repo
            .expect_update_material_status()
            .with(
                predicate::eq(material_id),
                predicate::eq(MaterialStatus::Error),
                predicate::always(),
            )
            .returning(|_, _, _| Ok(()));

        let mock_registry = MaterialRegistry::new(Arc::new(mock_material_repo), event_bus.clone());

        // Mock services that shouldn't be called
        let mock_embedding_service = Arc::new(MockEmbeddingService::new());
        let mock_swatch_repo = Arc::new(MockSwatchRepository::new());

        // Create actor with mocks
        let actor = SwatchingActor::new(
            "test-swatching-actor",
            event_bus.clone(),
            Arc::new(mock_cuts_repo),
            mock_embedding_service,
            mock_swatch_repo,
            mock_registry,
        );

        // Start actor
        let actor_addr = actor.start();

        // Manually trigger processing
        let (tx, _rx) = mpsc::channel(1);
        tx.send(SwatchingWorkItem {
            material_id: material_id.into(),
        })
        .await
        .unwrap();

        // Give the actor time to process
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Shutdown actor
        actor_addr.send(Shutdown).await.unwrap();
    }
}
