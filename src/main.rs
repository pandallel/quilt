// Quilt - Local-first, modular memory and context engine
//
// Main entry point for the Quilt application with actor-based implementation.

use actix::prelude::*;
use env_logger::Env;
use log::{debug, error, info};
use quilt::actors::{ActorError, Ping, Shutdown};
use quilt::discovery::actor::messages::StartDiscovery;
use quilt::discovery::DiscoveryActor;
use std::time::Duration;
use tokio::sync::oneshot;

// Maximum wait time for actor operations
const ACTOR_TIMEOUT: Duration = Duration::from_secs(5);

#[actix::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger with the new API
    env_logger::init_from_env(Env::new().default_filter_or("debug"));

    info!("Quilt - Local-first, modular memory and context engine");
    info!("Actor system starting...");

    // Create the discovery actor
    let discovery = DiscoveryActor::new("main-discovery").start();

    // Use a oneshot channel for shutdown coordination
    let (tx, rx) = oneshot::channel::<()>();
    // No need to clone oneshot senders as they can only be used once

    // Send a ping to check if the actor is ready
    match discovery.send(Ping).await {
        Ok(true) => {
            debug!("Discovery actor is ready");

            // Start discovery in current directory
            let current_dir = std::env::current_dir()
                .map_err(|e| {
                    Box::<dyn std::error::Error>::from(ActorError::OperationFailure(format!(
                        "Failed to get current directory: {}",
                        e
                    )))
                })?
                .to_string_lossy()
                .to_string();

            // Explicitly handle the result
            let discovery_result = discovery
                .send(StartDiscovery {
                    directory: current_dir,
                })
                .await
                .map_err(|e| {
                    Box::<dyn std::error::Error>::from(ActorError::MessageSendFailure(format!(
                        "Failed to send StartDiscovery: {}",
                        e
                    )))
                })?;

            // Handle any errors from the discovery operation
            if let Err(e) = discovery_result {
                error!("Discovery operation failed: {}", e);
                return Err(Box::<dyn std::error::Error>::from(
                    ActorError::OperationFailure(format!("Discovery operation failed: {}", e)),
                ));
            }

            // Proceed with application logic
            // ...

            // Example of scheduled shutdown after some work
            tokio::spawn(async move {
                // Simulate some work
                tokio::time::sleep(Duration::from_secs(1)).await;

                // Signal we're done
                let _ = tx.send(());
            });
        }
        Ok(false) => {
            return Err(Box::<dyn std::error::Error>::from(
                ActorError::NotAvailable("Discovery actor is not ready".into()),
            ));
        }
        Err(e) => {
            return Err(Box::<dyn std::error::Error>::from(
                ActorError::MessageSendFailure(format!("Failed to ping discovery actor: {}", e)),
            ));
        }
    }

    // Wait for work completion with timeout
    tokio::select! {
        _ = rx => {
            info!("Work completed, initiating shutdown");
        }
        _ = tokio::time::sleep(ACTOR_TIMEOUT) => {
            error!("Operation timed out after {:?}, forcing shutdown", ACTOR_TIMEOUT);
        }
    }

    // Shutdown the actor
    match discovery.send(Shutdown).await {
        Ok(_) => {
            info!("Shutdown message sent to discovery actor");
        }
        Err(e) => {
            error!("Failed to send shutdown message: {}", e);
        }
    }

    // Wait for actor system to shut down
    System::current().stop();

    info!("Actor system shutdown complete");
    Ok(())
}
