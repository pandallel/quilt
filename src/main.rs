// Quilt - Local-first, modular memory and context engine
//
// Main entry point for the Quilt application with actor-based implementation.

use env_logger::Env;
use log::info;
use std::time::Duration;

mod orchestrator;
use orchestrator::{QuiltOrchestrator, OrchestratorConfig};

// Maximum wait time for actor operations
const ACTOR_TIMEOUT: Duration = Duration::from_secs(5);

#[actix::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger with the new API
    env_logger::init_from_env(Env::new().default_filter_or("debug"));

    info!("Quilt - Local-first, modular memory and context engine");

    // Get current directory for discovery
    let current_dir = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?
        .to_string_lossy()
        .to_string();

    // Create orchestrator configuration
    let config = OrchestratorConfig {
        discovery_dir: current_dir,
        actor_timeout: ACTOR_TIMEOUT,
    };

    // Create and run the orchestrator
    let orchestrator = QuiltOrchestrator::new();
    orchestrator.run(config).await?;

    info!("Application shutdown complete");
    Ok(())
}
