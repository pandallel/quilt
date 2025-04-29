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
                                    "{}: No cuts found for material {}. Marking as Error.",
                                    actor_name, material_id_str
                                );
                                // If no cuts are found, it's an error state for swatching
                                if let Err(err) = registry.update_material_status(
                                    material_id_str,
                                    crate::materials::types::MaterialStatus::Error,
                                    Some(format!("No cuts found for material {}", material_id_str)),
                                ).await {
                                    error!(
                                        "{}: Failed to update material status for {}: {}",
                                        actor_name, material_id_str, err
                                    );
                                }
                                continue; // Skip to the next work item
                            }

                            debug!(
                                "{}: Retrieved {} cuts for material {}",
                                actor_name,
                                cuts.len(),
                                material_id_str
                            );

                            // Get model info once
                            let model_name = embedding_service.model_name();
                            let model_version = embedding_service.model_version();

                            // Process each cut to generate embeddings
                            let mut embedding_results = Vec::new();
                            let mut failed_embedding_count = 0;

                            for cut in &cuts {
                                debug!(
                                    "{}: Generating embedding for cut {} (chunk {}) using model {} {}",
                                    actor_name, cut.id, cut.chunk_index, model_name, model_version
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
                                        failed_embedding_count += 1;
                                        // Log error and continue with other cuts
                                    }
                                }
                            }

                            info!(
                                "{}: Processed material {}: {} embeddings succeeded, {} failed.",
                                actor_name,
                                material_id_str,
                                embedding_results.len(),
                                failed_embedding_count
                            );

                            // Create swatches only if there were successful embeddings
                            if embedding_results.is_empty() {
                                error!(
                                    "{}: Failed to generate any valid embeddings for material {}. Marking as Error.",
                                    actor_name, material_id_str
                                );

                                // Update registry with error status
                                if let Err(err) = registry.update_material_status(
                                    material_id_str,
                                    crate::materials::types::MaterialStatus::Error,
                                    Some("Failed to generate embeddings for any cuts".to_string()),
                                ).await {
                                    error!(
                                        "{}: Failed to update material status for {}: {}",
                                        actor_name, material_id_str, err
                                    );
                                }

                                continue; // Skip swatch saving if no embeddings succeeded
                            }

                            // Create swatches from the successful embeddings
                            let mut swatches = Vec::new();

                            for (cut, embedding) in &embedding_results {
                                // Create a new swatch using the embedding and fetched model info
                                let swatch = super::swatch::Swatch::new(
                                    cut.id.clone(),
                                    material_id_str.to_string(),
                                    embedding.clone(),
                                    model_name.to_string(), // Use fetched model name
                                    model_version.to_string(), // Use fetched model version
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
    use crate::cutting::CutsRepositoryError;
    use crate::cutting::MockCutsRepository;
    use crate::events::EventBus;
    use crate::materials::types::{Material, MaterialFileType};
    use crate::materials::MaterialStatus;
    use crate::materials::MockMaterialRepository;
    use crate::swatching::embedding::{EmbeddingError, MockEmbeddingService};
    use crate::swatching::repository::MockSwatchRepository;
    use crate::swatching::swatch::Swatch;
    use mockall::{predicate, Sequence};
    use std::sync::Arc;
    use std::time::Duration;
    use time::OffsetDateTime;

    fn init_test_logger() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .try_init();
    }

    fn setup_common_mocks() -> (
        Arc<EventBus>,
        MockCutsRepository,
        MockEmbeddingService,
        MockSwatchRepository,
        MockMaterialRepository,
    ) {
        let event_bus = Arc::new(EventBus::new());
        let mock_cuts_repo = MockCutsRepository::new();
        let mock_embedding_service = MockEmbeddingService::new();
        let mock_swatch_repo = MockSwatchRepository::new();
        let mock_material_repo = MockMaterialRepository::new();
        (
            event_bus,
            mock_cuts_repo,
            mock_embedding_service,
            mock_swatch_repo,
            mock_material_repo,
        )
    }

    fn create_dummy_material(id: &str) -> Material {
        let file_path_str = format!("/{}.txt", id);
        let now = OffsetDateTime::now_utc();
        Material {
            id: id.to_string(),
            file_path: file_path_str.clone(),
            file_type: MaterialFileType::from_path(&file_path_str),
            created_at: now,
            updated_at: now,
            status_updated_at: now,
            status: MaterialStatus::Cut,
            error: None,
        }
    }

    #[actix::test]
    async fn test_swatching_actor_ping() {
        init_test_logger();
        let (
            event_bus,
            mock_cuts_repo,
            mock_embedding_service,
            mock_swatch_repo,
            mock_material_repo,
        ) = setup_common_mocks();

        let registry = MaterialRegistry::new(Arc::new(mock_material_repo), event_bus.clone());
        let actor = SwatchingActor::new(
            "test-ping",
            event_bus,
            Arc::new(mock_cuts_repo),
            Arc::new(mock_embedding_service),
            Arc::new(mock_swatch_repo),
            registry,
        );
        let actor_addr = actor.start();
        let result = actor_addr.send(Ping).await.unwrap();
        assert!(result, "Actor should respond to ping");
        actor_addr.send(Shutdown).await.unwrap();
    }

    #[actix::test]
    async fn test_swatching_actor_processes_item_successfully() {
        init_test_logger();

        let material_id = "mat-success";
        let model_name = "test-model";
        let model_version = "v-test";

        let (
            event_bus,
            mut mock_cuts_repo,
            mut mock_embedding_service,
            mut mock_swatch_repo,
            mut mock_material_repo,
        ) = setup_common_mocks();

        let cut = Cut::new(material_id.to_string(), 0, "Successful content".to_string());
        let cut_clone = cut.clone();
        let cut_id = cut_clone.id.clone();
        let cut_content_clone = cut_clone.content.clone();

        mock_cuts_repo
            .expect_get_cuts_by_material_id()
            .times(1)
            .returning(move |_| Ok(vec![cut_clone.clone()]));
        let test_embedding = vec![0.1, 0.2];
        let test_embedding_clone = test_embedding.clone();
        mock_embedding_service
            .expect_model_name()
            .times(1)
            .return_const(model_name.to_string());
        mock_embedding_service
            .expect_model_version()
            .times(1)
            .return_const(model_version.to_string());
        mock_embedding_service
            .expect_embed()
            .withf(move |text: &str| text == cut_content_clone)
            .times(1)
            .returning(move |_| Ok(test_embedding_clone.clone()));
        mock_swatch_repo
            .expect_save_swatches_batch()
            .withf(move |swatches: &[Swatch]| {
                swatches.len() == 1
                    && swatches[0].cut_id == cut_id
                    && swatches[0].embedding == test_embedding
                    && swatches[0].model_name == model_name
                    && swatches[0].model_version == model_version
            })
            .times(1)
            .returning(|_| Ok(()));

        let mat_id_clone = material_id.to_string();
        mock_material_repo
            .expect_get_material()
            .with(predicate::eq(material_id))
            .times(1)
            .returning(move |_| Some(create_dummy_material(&mat_id_clone)));
        mock_material_repo
            .expect_update_material_status()
            .with(
                predicate::eq(material_id),
                predicate::eq(MaterialStatus::Swatched),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let registry = MaterialRegistry::new(Arc::new(mock_material_repo), event_bus.clone());

        let actor = SwatchingActor::new(
            "test-success",
            event_bus,
            Arc::new(mock_cuts_repo),
            Arc::new(mock_embedding_service),
            Arc::new(mock_swatch_repo),
            registry,
        );
        let actor_addr = actor.start();

        let work_sender = actor_addr.send(GetWorkSender).await.unwrap().unwrap();
        work_sender
            .send(SwatchingWorkItem {
                material_id: material_id.into(),
            })
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(200)).await;
        actor_addr.send(Shutdown).await.unwrap();
    }

    #[actix::test]
    async fn test_swatching_actor_handles_cuts_repo_error() {
        init_test_logger();
        let material_id = "mat-cuts-error";
        let (
            event_bus,
            mut mock_cuts_repo,
            mock_embedding_service,
            mock_swatch_repo,
            mut mock_material_repo,
        ) = setup_common_mocks();

        mock_cuts_repo
            .expect_get_cuts_by_material_id()
            .times(1)
            .returning(|_| Err(CutsRepositoryError::OperationFailed("DB error".into())));

        let mat_id_clone = material_id.to_string();
        mock_material_repo
            .expect_get_material()
            .with(predicate::eq(material_id))
            .times(1)
            .returning(move |_| Some(create_dummy_material(&mat_id_clone)));
        mock_material_repo
            .expect_update_material_status()
            .with(
                predicate::eq(material_id),
                predicate::eq(MaterialStatus::Error),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let registry = MaterialRegistry::new(Arc::new(mock_material_repo), event_bus.clone());

        let actor = SwatchingActor::new(
            "test-cuts-error",
            event_bus,
            Arc::new(mock_cuts_repo),
            Arc::new(mock_embedding_service),
            Arc::new(mock_swatch_repo),
            registry,
        );
        let actor_addr = actor.start();

        let work_sender = actor_addr.send(GetWorkSender).await.unwrap().unwrap();
        work_sender
            .send(SwatchingWorkItem {
                material_id: material_id.into(),
            })
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;
        actor_addr.send(Shutdown).await.unwrap();
    }

    #[actix::test]
    async fn test_swatching_actor_handles_no_cuts() {
        init_test_logger();
        let material_id = "mat-no-cuts";
        let (
            event_bus,
            mut mock_cuts_repo,
            mock_embedding_service,
            mock_swatch_repo,
            mut mock_material_repo,
        ) = setup_common_mocks();

        mock_cuts_repo
            .expect_get_cuts_by_material_id()
            .times(1)
            .returning(|_| Ok(vec![]));

        let mat_id_clone = material_id.to_string();
        mock_material_repo
            .expect_get_material()
            .with(predicate::eq(material_id))
            .times(1)
            .returning(move |_| Some(create_dummy_material(&mat_id_clone)));
        let mat_id_clone_for_update = material_id.to_string();
        mock_material_repo
            .expect_update_material_status()
            .withf(move |mid, status, msg| {
                mid == mat_id_clone_for_update
                    && *status == MaterialStatus::Error
                    && msg.as_deref().unwrap_or("").contains("No cuts found")
            })
            .times(1)
            .returning(|_, _, _| Ok(()));

        let registry = MaterialRegistry::new(Arc::new(mock_material_repo), event_bus.clone());

        let actor = SwatchingActor::new(
            "test-no-cuts",
            event_bus,
            Arc::new(mock_cuts_repo),
            Arc::new(mock_embedding_service),
            Arc::new(mock_swatch_repo),
            registry,
        );
        let actor_addr = actor.start();

        let work_sender = actor_addr.send(GetWorkSender).await.unwrap().unwrap();
        work_sender
            .send(SwatchingWorkItem {
                material_id: material_id.into(),
            })
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;
        actor_addr.send(Shutdown).await.unwrap();
    }

    #[actix::test]
    async fn test_swatching_actor_handles_all_embedding_failures() {
        init_test_logger();
        let material_id = "mat-embed-all-fail";
        let model_name = "fail-model";
        let model_version = "v-fail";
        let (
            event_bus,
            mut mock_cuts_repo,
            mut mock_embedding_service,
            mock_swatch_repo,
            mut mock_material_repo,
        ) = setup_common_mocks();

        let cut1 = Cut::new(material_id.to_string(), 0, "Content 1".to_string());
        let cut2 = Cut::new(material_id.to_string(), 1, "Content 2".to_string());
        let cuts = vec![cut1.clone(), cut2.clone()];

        mock_cuts_repo
            .expect_get_cuts_by_material_id()
            .times(1)
            .returning(move |_| Ok(cuts.clone()));
        mock_embedding_service
            .expect_model_name()
            .times(1)
            .return_const(model_name.to_string());
        mock_embedding_service
            .expect_model_version()
            .times(1)
            .return_const(model_version.to_string());
        mock_embedding_service
            .expect_embed()
            .times(2)
            .returning(|_| Err(EmbeddingError::GenerationFailed("Mock embed error".into())));

        let mat_id_clone = material_id.to_string();
        mock_material_repo
            .expect_get_material()
            .with(predicate::eq(material_id))
            .times(1)
            .returning(move |_| Some(create_dummy_material(&mat_id_clone)));
        let mat_id_clone_for_update = material_id.to_string();
        mock_material_repo
            .expect_update_material_status()
            .withf(move |mid, status, msg| {
                mid == mat_id_clone_for_update
                    && *status == MaterialStatus::Error
                    && msg
                        .as_deref()
                        .unwrap_or("")
                        .contains("Failed to generate embeddings")
            })
            .times(1)
            .returning(|_, _, _| Ok(()));

        let registry = MaterialRegistry::new(Arc::new(mock_material_repo), event_bus.clone());

        let actor = SwatchingActor::new(
            "test-embed-all-fail",
            event_bus,
            Arc::new(mock_cuts_repo),
            Arc::new(mock_embedding_service),
            Arc::new(mock_swatch_repo),
            registry,
        );
        let actor_addr = actor.start();

        let work_sender = actor_addr.send(GetWorkSender).await.unwrap().unwrap();
        work_sender
            .send(SwatchingWorkItem {
                material_id: material_id.into(),
            })
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;
        actor_addr.send(Shutdown).await.unwrap();
    }

    #[actix::test]
    async fn test_swatching_actor_handles_partial_embedding_failures() {
        init_test_logger();
        let material_id = "mat-embed-partial-fail";
        let model_name = "partial-model";
        let model_version = "v-partial";
        let (
            event_bus,
            mut mock_cuts_repo,
            mut mock_embedding_service,
            mut mock_swatch_repo,
            mut mock_material_repo,
        ) = setup_common_mocks();

        let cut1 = Cut::new(material_id.to_string(), 0, "Content 1 ok".to_string());
        let cut2 = Cut::new(material_id.to_string(), 1, "Content 2 fail".to_string());
        let cut3 = Cut::new(material_id.to_string(), 2, "Content 3 ok".to_string());
        let cuts = vec![cut1.clone(), cut2.clone(), cut3.clone()];
        let cut1_id = cut1.id.clone();
        let cut3_id = cut3.id.clone();
        let cut1_content_clone = cut1.content.clone();
        let cut2_content_clone = cut2.content.clone();
        let cut3_content_clone = cut3.content.clone();

        mock_cuts_repo
            .expect_get_cuts_by_material_id()
            .times(1)
            .returning(move |_| Ok(cuts.clone()));
        mock_embedding_service
            .expect_model_name()
            .times(1)
            .return_const(model_name.to_string());
        mock_embedding_service
            .expect_model_version()
            .times(1)
            .return_const(model_version.to_string());
        let mut seq = Sequence::new();
        let embed_ok1 = vec![1.0];
        let embed_ok3 = vec![3.0];
        let embed_ok1_clone = embed_ok1.clone();
        let embed_ok3_clone = embed_ok3.clone();
        mock_embedding_service
            .expect_embed()
            .withf(move |text: &str| text == cut1_content_clone)
            .times(1)
            .in_sequence(&mut seq)
            .returning(move |_| Ok(embed_ok1_clone.clone()));
        mock_embedding_service
            .expect_embed()
            .withf(move |text: &str| text == cut2_content_clone)
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_| Err(EmbeddingError::GenerationFailed("Fail 2".into())));
        mock_embedding_service
            .expect_embed()
            .withf(move |text: &str| text == cut3_content_clone)
            .times(1)
            .in_sequence(&mut seq)
            .returning(move |_| Ok(embed_ok3_clone.clone()));
        mock_swatch_repo
            .expect_save_swatches_batch()
            .withf(move |swatches: &[Swatch]| {
                swatches.len() == 2
                    && swatches
                        .iter()
                        .any(|s| s.cut_id == cut1_id && s.embedding == embed_ok1)
                    && swatches
                        .iter()
                        .any(|s| s.cut_id == cut3_id && s.embedding == embed_ok3)
                    && swatches
                        .iter()
                        .all(|s| s.model_name == model_name && s.model_version == model_version)
            })
            .times(1)
            .returning(|_| Ok(()));

        let mat_id_clone = material_id.to_string();
        mock_material_repo
            .expect_get_material()
            .with(predicate::eq(material_id))
            .times(1)
            .returning(move |_| Some(create_dummy_material(&mat_id_clone)));
        mock_material_repo
            .expect_update_material_status()
            .with(
                predicate::eq(material_id),
                predicate::eq(MaterialStatus::Swatched),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let registry = MaterialRegistry::new(Arc::new(mock_material_repo), event_bus.clone());

        let actor = SwatchingActor::new(
            "test-embed-partial-fail",
            event_bus,
            Arc::new(mock_cuts_repo),
            Arc::new(mock_embedding_service),
            Arc::new(mock_swatch_repo),
            registry,
        );
        let actor_addr = actor.start();

        let work_sender = actor_addr.send(GetWorkSender).await.unwrap().unwrap();
        work_sender
            .send(SwatchingWorkItem {
                material_id: material_id.into(),
            })
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(200)).await;
        actor_addr.send(Shutdown).await.unwrap();
    }

    #[derive(Message)]
    #[rtype(result = "Option<mpsc::Sender<SwatchingWorkItem>>")]
    struct GetWorkSender;

    impl Handler<GetWorkSender> for SwatchingActor {
        type Result = MessageResult<GetWorkSender>;

        fn handle(&mut self, _msg: GetWorkSender, _ctx: &mut Context<Self>) -> Self::Result {
            MessageResult(self.work_sender.clone())
        }
    }
}
