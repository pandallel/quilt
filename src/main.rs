// Quilt - Local-first, modular memory and context engine
//
// Main entry point for the Quilt application with actor-based implementation.

use clap::Parser;
use env_logger::Env;
use log::{error, info};
use std::time::Duration;

use quilt::orchestrator::{OrchestratorConfig, QuiltOrchestrator};

/// Local-first, modular memory and context engine
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory to scan for materials
    #[arg(short, long, default_value = ".")]
    dir: String,

    /// Include hidden files in scan
    #[arg(long)]
    include_hidden: bool,

    /// Patterns to exclude from scanning (can be provided multiple times)
    #[arg(short, long)]
    exclude: Vec<String>,
}

#[actix::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    // Parse command line arguments
    let args = Args::parse();

    // Create orchestrator configuration
    let config = OrchestratorConfig {
        discovery_dir: args.dir,
        ignore_hidden: !args.include_hidden,
        exclude_patterns: args.exclude,
        actor_timeout: Duration::from_secs(30),
    };

    // Log the configuration
    info!(
        "Starting Quilt with configuration: 
        Directory: {}
        Ignore Hidden: {}
        Exclude Patterns: {:?}",
        config.discovery_dir, config.ignore_hidden, config.exclude_patterns
    );

    // Run the orchestrator
    match QuiltOrchestrator::new().run(config).await {
        Ok(_) => {
            info!("Quilt application completed successfully");
        }
        Err(err) => {
            error!("Quilt application error: {}", err);
            return Err(Box::new(err) as Box<dyn std::error::Error>);
        }
    }

    info!("Quilt application shutdown complete");

    Ok(())
}
