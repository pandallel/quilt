use crate::actors::{Ping, Shutdown};
use actix::prelude::*;
use log::{debug, error, info};
use std::path::Path;

/// Messages specific to the DiscoveryActor
pub mod messages {
    use actix::prelude::*;
    use thiserror::Error;

    /// Discovery operation errors
    #[derive(Debug, Error)]
    pub enum DiscoveryError {
        /// Directory not found error
        #[error("Directory not found: {0}")]
        DirectoryNotFound(String),
        
        /// Permission error during discovery
        #[error("Permission denied while accessing directory: {0}")]
        PermissionDenied(String),
        
        /// Generic discovery error
        #[error("Discovery operation failed: {0}")]
        OperationFailed(String),
    }

    /// Command to start discovery in a directory
    #[derive(Message)]
    #[rtype(result = "Result<(), DiscoveryError>")]
    pub struct StartDiscovery {
        pub directory: String,
    }
    
    /// Response for operation completion status
    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct OperationComplete {
        pub success: bool,
        pub message: String,
    }
}

/// Actor responsible for discovering materials in directories
pub struct DiscoveryActor {
    name: String,
}

impl DiscoveryActor {
    /// Create a new DiscoveryActor with the given name
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
    
    /// Validate a directory path exists and is accessible
    fn validate_directory(&self, path: &str) -> Result<(), messages::DiscoveryError> {
        let path = Path::new(path);
        
        if !path.exists() {
            return Err(messages::DiscoveryError::DirectoryNotFound(path.display().to_string()));
        }
        
        if !path.is_dir() {
            return Err(messages::DiscoveryError::OperationFailed(
                format!("Path exists but is not a directory: {}", path.display())
            ));
        }
        
        // Basic access check - more sophisticated checks could be added
        match path.read_dir() {
            Ok(_) => Ok(()),
            Err(e) => Err(messages::DiscoveryError::PermissionDenied(
                format!("Cannot read directory {}: {}", path.display(), e)
            )),
        }
    }
}

impl Actor for DiscoveryActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("DiscoveryActor '{}' started", self.name);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("DiscoveryActor '{}' stopped", self.name);
    }
}

/// Handler for Ping messages
impl Handler<Ping> for DiscoveryActor {
    type Result = bool;

    fn handle(&mut self, _msg: Ping, _ctx: &mut Self::Context) -> Self::Result {
        debug!("DiscoveryActor '{}' received ping", self.name);
        true
    }
}

/// Handler for Shutdown messages
impl Handler<Shutdown> for DiscoveryActor {
    type Result = ();

    fn handle(&mut self, _msg: Shutdown, ctx: &mut Self::Context) -> Self::Result {
        info!("DiscoveryActor '{}' shutting down", self.name);
        ctx.stop();
    }
}

/// Handler for StartDiscovery messages
impl Handler<messages::StartDiscovery> for DiscoveryActor {
    type Result = Result<(), messages::DiscoveryError>;

    fn handle(&mut self, msg: messages::StartDiscovery, _ctx: &mut Self::Context) -> Self::Result {
        info!(
            "DiscoveryActor '{}' starting discovery in '{}'",
            self.name, msg.directory
        );
        
        // Validate the directory before proceeding
        match self.validate_directory(&msg.directory) {
            Ok(_) => {
                // In a real implementation, this would trigger actual discovery
                println!("Discovery started in directory: {}", msg.directory);
                Ok(())
            },
            Err(e) => {
                error!("Discovery validation failed: {}", e);
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    // Helper function to set up test environment
    fn init_test_logger() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .try_init();
    }
    
    #[actix::test]
    async fn test_discovery_actor_ping() {
        init_test_logger();
        
        // Create a discovery actor
        let actor = DiscoveryActor::new("test-discovery").start();
        
        // Send a ping message
        let result = actor.send(Ping).await;
        
        // Check that the ping response is successful
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }
    
    #[actix::test]
    async fn test_discovery_actor_shutdown() {
        init_test_logger();
        
        // Create a discovery actor
        let actor = DiscoveryActor::new("test-discovery").start();
        
        // Send a shutdown message
        let result = actor.send(Shutdown).await;
        
        // Check that the shutdown message was processed
        assert!(result.is_ok());
        
        // Wait a moment for the actor to shut down
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        
        // Try to ping the actor, which should fail
        let ping_result = actor.send(Ping).await;
        assert!(ping_result.is_err());
    }
    
    #[actix::test]
    async fn test_discovery_valid_directory() {
        init_test_logger();
        
        // Create a temporary directory for testing
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let temp_path = temp_dir.path().to_string_lossy().to_string();
        
        // Create a discovery actor
        let actor = DiscoveryActor::new("test-discovery").start();
        
        // Send a StartDiscovery message with a valid directory
        let result = actor.send(messages::StartDiscovery {
            directory: temp_path,
        }).await;
        
        // Check that the discovery operation was successful
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }
    
    #[actix::test]
    async fn test_discovery_invalid_directory() {
        init_test_logger();
        
        // Use a non-existent directory
        let invalid_path = "/path/to/nonexistent/directory";
        
        // Create a discovery actor
        let actor = DiscoveryActor::new("test-discovery").start();
        
        // Send a StartDiscovery message with an invalid directory
        let result = actor.send(messages::StartDiscovery {
            directory: invalid_path.to_string(),
        }).await;
        
        // Check that the operation returns an error
        assert!(result.is_ok());
        let inner_result = result.unwrap();
        assert!(inner_result.is_err());
        
        // Verify the error is of the correct type
        match inner_result {
            Err(messages::DiscoveryError::DirectoryNotFound(_)) => {
                // This is the expected error type
                assert!(true);
            },
            _ => {
                // Unexpected error type
                assert!(false, "Expected DirectoryNotFound error");
            }
        }
    }
}
