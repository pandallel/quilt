use crate::actors::{Ping, Shutdown};
use crate::events::types::MaterialId;
use crate::events::QuiltEvent;
use actix::prelude::*;
use actix::SpawnHandle;
use log::{debug, error, info, warn};
use tokio::sync::broadcast;
use tokio::sync::mpsc;

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
    event_bus: std::sync::Arc<crate::events::EventBus>,
    /// Sender for the internal work queue
    work_sender: Option<mpsc::Sender<SwatchingWorkItem>>,
    /// Handle for the listener task
    listener_handle: Option<SpawnHandle>,
    /// Handle for the processor task
    processor_handle: Option<SpawnHandle>,
}

impl SwatchingActor {
    /// Create a new SwatchingActor with the given name and event bus
    ///
    /// # Arguments
    ///
    /// * `name` - Name for this actor instance, used in logging
    /// * `event_bus` - Event bus to subscribe to events
    pub fn new(name: &str, event_bus: std::sync::Arc<crate::events::EventBus>) -> Self {
        Self {
            name: name.to_string(),
            event_bus,
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

                while let Some(work_item) = work_receiver.recv().await {
                    debug!(
                        "{}: Processor received work item for: {}",
                        actor_name,
                        work_item.material_id.as_str()
                    );

                    // Currently just logging receipt of the work item
                    // Swatching processing logic will be implemented in Milestone 9
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
    use std::sync::Arc;

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
        let actor = SwatchingActor::new("test-swatching", event_bus).start();

        let result = actor.send(Ping).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[actix::test]
    async fn test_swatching_actor_shutdown() {
        init_test_logger();
        let event_bus = Arc::new(EventBus::new());
        let actor = SwatchingActor::new("test-swatching", event_bus).start();

        let result = actor.send(Shutdown).await;
        assert!(result.is_ok());

        // Wait a short period for the actor to shut down
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Actor should no longer respond
        let ping_result = actor.send(Ping).await;
        assert!(ping_result.is_err());
    }
}
