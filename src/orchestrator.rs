// Quilt - Orchestrator for actor system coordination
//
// The QuiltOrchestrator manages the lifecycle of all actors in the system,
// coordinating their startup, inter-communication, and shutdown.

use actix::dev::ToEnvelope;
use actix::prelude::*;
use log::{debug, error, info, warn};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::oneshot;
use tokio::time::timeout;

use crate::actors::{ActorError, Ping, Shutdown};
use crate::cutting::CuttingActor;
use crate::discovery::actor::messages::{DiscoverySuccess, StartDiscovery};
use crate::discovery::actor::DiscoveryConfig;
use crate::discovery::DiscoveryActor;
use crate::events::EventBus;
use crate::materials::{MaterialRegistry, MaterialRepository};

/// Configuration for the Quilt orchestrator
pub struct OrchestratorConfig {
    /// Directory to start discovery in
    pub discovery_dir: String,
    /// Whether to ignore hidden files and directories
    pub ignore_hidden: bool,
    /// Patterns to exclude from scanning
    pub exclude_patterns: Vec<String>,
    /// Timeout for actor operations
    pub actor_timeout: Duration,
}

/// Errors specific to orchestration
#[derive(Error, Debug)]
pub enum OrchestratorError {
    #[error("Actor operation timed out after {0:?}")]
    Timeout(Duration),

    #[error("Actor error: {0}")]
    ActorError(#[from] ActorError),

    #[error("{0}")]
    Other(Box<dyn std::error::Error>),
}

impl From<Box<dyn std::error::Error>> for OrchestratorError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        Self::Other(err)
    }
}

/// Main orchestrator for the Quilt system
///
/// Manages actor lifecycle and coordinates the material processing pipeline
pub struct QuiltOrchestrator {
    discovery: Option<Addr<DiscoveryActor>>,
    cutting: Option<Addr<CuttingActor>>,
    registry: MaterialRegistry,
    event_bus: Arc<EventBus>,
    // Future actors:
    // swatching: Option<Addr<SwatchingActor>>,
}

impl QuiltOrchestrator {
    /// Create a new QuiltOrchestrator with default configuration
    pub fn new() -> Self {
        let event_bus = Arc::new(EventBus::new());
        let repository = MaterialRepository::new();
        let registry = MaterialRegistry::new(repository, event_bus.clone());

        Self {
            discovery: None,
            cutting: None,
            registry,
            event_bus,
        }
    }

    /// Run the orchestrator with the given configuration
    pub async fn run(mut self, config: OrchestratorConfig) -> Result<(), OrchestratorError> {
        info!("Actor system starting...");

        // Set up event monitoring
        self.setup_event_monitoring();

        // Initialize actors
        self.initialize_actors()?;

        // Start discovery process with timeout
        let success = self
            .start_discovery_with_timeout(
                &config.discovery_dir,
                config.ignore_hidden,
                config.exclude_patterns.clone(),
                config.actor_timeout,
            )
            .await?;

        // Check success
        if success.success {
            info!("Discovery process completed successfully");
        } else {
            error!("Discovery process failed");
        }

        // Run application logic
        let (tx, rx) = oneshot::channel::<()>();
        self.run_application_logic(tx).await;

        // Wait for completion with timeout
        tokio::select! {
            _ = rx => {
                info!("Work completed, initiating shutdown");
            }
            _ = tokio::time::sleep(config.actor_timeout) => {
                warn!("Operation timed out after {:?}, forcing shutdown", config.actor_timeout);
            }
        }

        // Shutdown actors
        self.shutdown_actors_with_timeout(config.actor_timeout)
            .await;

        Ok(())
    }

    /// Set up monitoring for the event bus
    fn setup_event_monitoring(&self) {
        // Create a subscriber to the event bus
        let mut subscriber = self.event_bus.subscribe();

        // Spawn a task to monitor events
        tokio::spawn(async move {
            while let Ok(event) = subscriber.recv().await {
                debug!("Event received: {}", event);
            }
        });
    }

    /// Initialize all actors in the system
    fn initialize_actors(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Create the discovery actor with registry
        let discovery_actor = DiscoveryActor::new("main-discovery", self.registry.clone());
        self.discovery = Some(discovery_actor.start());

        // Verify discovery actor is running
        if let Some(_discovery) = &self.discovery {
            debug!("Initialized discovery actor, verifying it's running...");
            // Don't wait for verification here - we'll check before using it
        } else {
            return Err("Failed to start discovery actor".into());
        }

        // Initialize cutting actor
        let cutting_actor = CuttingActor::new("main-cutting", self.registry.clone());
        let cutting_addr = cutting_actor.start();
        debug!("Initialized cutting actor");
        self.cutting = Some(cutting_addr);

        // Future: Initialize other actors
        // self.swatching = Some(SwatchingActor::new(self.registry.clone()).start());

        Ok(())
    }

    /// Start the discovery process with a timeout
    async fn start_discovery_with_timeout(
        &self,
        directory: &str,
        ignore_hidden: bool,
        exclude_patterns: Vec<String>,
        timeout_duration: Duration,
    ) -> Result<DiscoverySuccess, OrchestratorError> {
        let discovery = self
            .discovery
            .as_ref()
            .ok_or_else(|| ActorError::NotAvailable("Discovery actor not initialized".into()))?;

        // Check if actor is ready with timeout
        match timeout(timeout_duration, discovery.send(Ping)).await {
            Ok(ping_result) => {
                match ping_result {
                    Ok(true) => {
                        debug!("Discovery actor is ready");

                        // Create scan configuration
                        let scan_config = DiscoveryConfig {
                            directory: directory.to_string(),
                            ignore_hidden,
                            exclude_patterns,
                        };

                        // Start discovery with timeout
                        match timeout(
                            timeout_duration,
                            discovery.send(StartDiscovery {
                                config: scan_config,
                            }),
                        )
                        .await
                        {
                            Ok(send_result) => {
                                let success = send_result
                                    .map_err(|e| {
                                        ActorError::MessageSendFailure(format!(
                                            "Failed to send StartDiscovery: {}",
                                            e
                                        ))
                                    })?
                                    .map_err(|e| {
                                        ActorError::OperationFailure(format!(
                                            "Discovery operation failed: {}",
                                            e
                                        ))
                                    })?;

                                Ok(success)
                            }
                            Err(_) => Err(OrchestratorError::Timeout(timeout_duration)),
                        }
                    }
                    Ok(false) => Err(OrchestratorError::ActorError(ActorError::NotAvailable(
                        "Discovery actor is not ready".into(),
                    ))),
                    Err(e) => Err(OrchestratorError::ActorError(
                        ActorError::MessageSendFailure(format!(
                            "Failed to ping discovery actor: {}",
                            e
                        )),
                    )),
                }
            }
            Err(_) => Err(OrchestratorError::Timeout(timeout_duration)),
        }
    }

    /// Run application logic
    async fn run_application_logic(&self, tx: oneshot::Sender<()>) {
        // Example of scheduled shutdown after some work
        tokio::spawn(async move {
            // Simulate some work
            tokio::time::sleep(Duration::from_secs(1)).await;

            // Signal we're done
            let _ = tx.send(());
        });
    }

    /// Shutdown all actors in the system with a timeout
    async fn shutdown_actors_with_timeout(&self, timeout_duration: Duration) {
        info!("Shutting down actors...");

        // Helper function to shutdown an actor with consistent error handling
        async fn shutdown_actor_with_timeout<A>(
            actor_name: &str,
            actor_addr: &Option<Addr<A>>,
            timeout_duration: Duration,
        ) where
            A: Actor,
            A: Handler<Shutdown>,
            <A as Actor>::Context: ToEnvelope<A, Shutdown>,
        {
            if let Some(addr) = actor_addr {
                match timeout(timeout_duration, addr.send(Shutdown)).await {
                    Ok(Ok(_)) => info!("{} actor shut down successfully", actor_name),
                    Ok(Err(e)) => error!("Failed to send shutdown to {} actor: {}", actor_name, e),
                    Err(_) => error!("Timeout shutting down {} actor", actor_name),
                }
            }
        }

        // Shutdown discovery actor
        shutdown_actor_with_timeout("Discovery", &self.discovery, timeout_duration).await;

        // Shutdown cutting actor
        shutdown_actor_with_timeout("Cutting", &self.cutting, timeout_duration).await;

        // Future: Add other actors here using the same pattern
        // shutdown_actor_with_timeout("Swatching", &self.swatching, timeout_duration).await;

        info!("All actors shut down");

        // Optionally stop the entire actor system
        // System::current().stop();
    }
}

impl Default for QuiltOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}
