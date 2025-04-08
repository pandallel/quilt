// Quilt - Local-first, modular memory and context engine
//
// Main entry point for the Quilt application with actor-based implementation.

use actix::prelude::*;
use env_logger::Env;
use log::{info, debug};
use quilt::actors::{Ping, Shutdown};
use quilt::discovery::DiscoveryActor;
use quilt::discovery::actor::messages::StartDiscovery;
use std::time::Duration;

#[actix::main]
async fn main() {
    // Initialize logger with the new API
    env_logger::init_from_env(Env::new().default_filter_or("debug"));
    
    info!("Quilt - Local-first, modular memory and context engine");
    info!("Actor system starting...");
    
    // Create the discovery actor
    let discovery = DiscoveryActor::new("main-discovery").start();
    
    // Send a ping to check if the actor is ready
    let is_ready = discovery.send(Ping).await.unwrap_or(false);
    if is_ready {
        debug!("Discovery actor is ready");
        
        // Start discovery in current directory
        let current_dir = std::env::current_dir()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
            
        discovery.send(StartDiscovery { directory: current_dir }).await.unwrap();
        
        // Give some time for the actor to process the message
        actix::clock::sleep(Duration::from_millis(100)).await;
    }
    
    // Shutdown the actor
    discovery.send(Shutdown).await.unwrap();
    
    // Give actors time to shut down gracefully
    actix::clock::sleep(Duration::from_millis(100)).await;
    
    info!("Actor system shutdown complete");
}
