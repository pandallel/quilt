// Quilt - Orchestrator for actor system coordination
//
// The QuiltOrchestrator manages the lifecycle of all actors in the system,
// coordinating their startup, inter-communication, and shutdown.

use actix::prelude::*;
use log::{debug, error, info};
use std::time::Duration;
use tokio::sync::oneshot;

use quilt::actors::{ActorError, Ping, Shutdown};
use quilt::discovery::actor::messages::StartDiscovery;
use quilt::discovery::DiscoveryActor;

/// Configuration for the Quilt orchestrator
pub struct OrchestratorConfig {
    /// Directory to start discovery in
    pub discovery_dir: String,
    /// Timeout for actor operations
    pub actor_timeout: Duration,
}

/// Main orchestrator for the Quilt system
///
/// Manages actor lifecycle and coordinates the material processing pipeline
pub struct QuiltOrchestrator {
    discovery: Option<Addr<DiscoveryActor>>,
    // Future actors:
    // cutting: Option<Addr<CuttingActor>>,
    // swatching: Option<Addr<SwatchingActor>>,
}

impl QuiltOrchestrator {
    /// Create a new orchestrator
    pub fn new() -> Self {
        Self { discovery: None }
    }

    /// Run the orchestrator with the given configuration
    pub async fn run(
        mut self,
        config: OrchestratorConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Actor system starting...");

        // Initialize actors
        self.initialize_actors()?;

        // Start discovery process
        self.start_discovery(&config.discovery_dir).await?;

        // Run application logic
        let (tx, rx) = oneshot::channel::<()>();
        self.run_application_logic(tx).await;

        // Wait for completion with timeout
        tokio::select! {
            _ = rx => {
                info!("Work completed, initiating shutdown");
            }
            _ = tokio::time::sleep(config.actor_timeout) => {
                error!("Operation timed out after {:?}, forcing shutdown", config.actor_timeout);
            }
        }

        // Shutdown actors
        self.shutdown_actors().await;

        Ok(())
    }

    /// Initialize all actors in the system
    fn initialize_actors(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Create the discovery actor
        self.discovery = Some(DiscoveryActor::new("main-discovery").start());

        // Future: Initialize other actors
        // self.cutting = Some(CuttingActor::new().start());
        // self.swatching = Some(SwatchingActor::new().start());

        Ok(())
    }

    /// Start the discovery process
    async fn start_discovery(&self, directory: &str) -> Result<(), Box<dyn std::error::Error>> {
        let discovery = self.discovery.as_ref().ok_or_else(|| {
            Box::<dyn std::error::Error>::from(ActorError::NotAvailable(
                "Discovery actor not initialized".into(),
            ))
        })?;

        // Check if actor is ready
        match discovery.send(Ping).await {
            Ok(true) => {
                debug!("Discovery actor is ready");

                // Start discovery
                discovery
                    .send(StartDiscovery {
                        directory: directory.to_string(),
                    })
                    .await
                    .map_err(|e| {
                        Box::<dyn std::error::Error>::from(ActorError::MessageSendFailure(format!(
                            "Failed to send StartDiscovery: {}",
                            e
                        )))
                    })?
                    .map_err(|e| {
                        Box::<dyn std::error::Error>::from(ActorError::OperationFailure(format!(
                            "Discovery operation failed: {}",
                            e
                        )))
                    })?;

                Ok(())
            }
            Ok(false) => Err(Box::<dyn std::error::Error>::from(
                ActorError::NotAvailable("Discovery actor is not ready".into()),
            )),
            Err(e) => Err(Box::<dyn std::error::Error>::from(
                ActorError::MessageSendFailure(format!("Failed to ping discovery actor: {}", e)),
            )),
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

    /// Shutdown all actors in the system
    async fn shutdown_actors(&self) {
        if let Some(discovery) = &self.discovery {
            // Shutdown the discovery actor
            match discovery.send(Shutdown).await {
                Ok(_) => {
                    info!("Shutdown message sent to discovery actor");
                }
                Err(e) => {
                    error!("Failed to send shutdown message: {}", e);
                }
            }
        }

        // Future: Shutdown other actors
        // if let Some(cutting) = &self.cutting {
        //     cutting.send(Shutdown).await.ok();
        // }

        // Wait for actor system to shut down
        System::current().stop();

        info!("Actor system shutdown complete");
    }
}
