use crate::actors::{Ping, Shutdown};
use crate::discovery::scanner::{DirectoryScanner, ScanResults};
use crate::materials::{MaterialRegistry, RegistryError, RepositoryError};
use actix::prelude::*;
use log::{debug, error, info};
use std::path::Path;

/// Configuration for directory scanning
///
/// This configuration is passed to the DiscoveryActor to control how it scans directories
/// for materials. It includes the directory path, whether to ignore hidden files, and
/// patterns to exclude from scanning.
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Directory to scan
    pub directory: String,
    /// Whether to ignore hidden files and directories
    pub ignore_hidden: bool,
    /// Patterns to exclude from scanning
    pub exclude_patterns: Vec<String>,
}

/// Messages specific to the DiscoveryActor
///
/// This module contains all message types that can be sent to the DiscoveryActor
/// to request operations and their respective response types.
pub mod messages {
    use super::DiscoveryConfig;
    use actix::prelude::*;
    use thiserror::Error;

    /// Discovery operation errors
    ///
    /// These errors can occur during discovery operations and provide
    /// detailed information about what went wrong.
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

        /// Scanner error
        #[error("Scanner error: {0}")]
        ScannerError(String),

        /// Repository error
        #[error("Repository error: {0}")]
        RepositoryError(String),
    }

    /// Command to start discovery using the provided configuration
    ///
    /// Send this message to a DiscoveryActor to begin the discovery process
    /// in the specified directory with the given configuration options.
    #[derive(Message)]
    #[rtype(result = "Result<DiscoverySuccess, DiscoveryError>")]
    pub struct StartDiscovery {
        /// Configuration for the discovery operation
        pub config: DiscoveryConfig,
    }

    /// Success response for discovery operation
    ///
    /// This is returned when a discovery operation completes, whether
    /// materials were found or not.
    #[derive(Debug)]
    pub struct DiscoverySuccess {
        /// Was the discovery operation successful
        pub success: bool,
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

/// Actor responsible for discovering materials in directories
///
/// The DiscoveryActor is responsible for scanning directories for materials,
/// registering them with the MaterialRegistry, and reporting the results.
/// It handles validation of directory paths, scanning for files, and registering
/// the discovered materials with the registry.
///
/// # Message Handlers
///
/// * `Ping` - Responds with `true` to indicate the actor is alive
/// * `Shutdown` - Gracefully shuts down the actor
/// * `StartDiscovery` - Begins the discovery process with the given configuration
pub struct DiscoveryActor {
    /// Name of this actor instance for logging
    name: String,
    /// Registry to manage materials and publish events
    registry: MaterialRegistry,
}

impl DiscoveryActor {
    /// Create a new DiscoveryActor with the given name and registry
    ///
    /// # Arguments
    ///
    /// * `name` - Name for this actor instance, used in logging
    /// * `registry` - Registry to manage materials and publish events
    pub fn new(name: &str, registry: MaterialRegistry) -> Self {
        Self {
            name: name.to_string(),
            registry,
        }
    }

    /// Validate a directory path exists and is accessible
    ///
    /// # Arguments
    ///
    /// * `path` - The directory path to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the directory exists and is accessible
    /// * `Err(DiscoveryError)` with a specific error if validation fails
    fn validate_directory(&self, path: &str) -> Result<(), messages::DiscoveryError> {
        let path = Path::new(path);

        if !path.exists() {
            return Err(messages::DiscoveryError::DirectoryNotFound(
                path.display().to_string(),
            ));
        }

        if !path.is_dir() {
            return Err(messages::DiscoveryError::OperationFailed(format!(
                "Path exists but is not a directory: {}",
                path.display()
            )));
        }

        // Basic access check - more sophisticated checks could be added
        match path.read_dir() {
            Ok(_) => Ok(()),
            Err(e) => Err(messages::DiscoveryError::PermissionDenied(format!(
                "Cannot read directory {}: {}",
                path.display(),
                e
            ))),
        }
    }

    /// Register materials with the registry
    ///
    /// # Arguments
    ///
    /// * `scan_results` - Results from a directory scan
    ///
    /// # Returns
    ///
    /// * `Ok((found_count, failed_count, registered_count, total_materials))` if registration was successful
    /// * `Err(DiscoveryError)` if registration failed
    async fn register_materials(
        &self,
        scan_results: ScanResults,
    ) -> Result<(usize, usize, usize, usize), messages::DiscoveryError> {
        let found_count = scan_results.found.len();
        let failed_count = scan_results.failed.len();
        let mut registered_count = 0;

        // Register all found materials
        for material in scan_results.found {
            debug!(
                "Registering material '{}' from path '{}'",
                material.id, material.file_path
            );

            match self.registry.register_material(material).await {
                Ok(_) => {
                    registered_count += 1;
                }
                Err(RegistryError::Repository(ref err))
                    if matches!(err, RepositoryError::MaterialAlreadyExists(_)) =>
                {
                    // Extract the material ID if possible
                    let id = if let RepositoryError::MaterialAlreadyExists(id) = err {
                        id
                    } else {
                        "unknown"
                    };
                    debug!("Material '{}' already exists in registry, skipping", id);
                }
                Err(err) => {
                    error!("Failed to register material: {}", err);
                    return Err(messages::DiscoveryError::RepositoryError(format!(
                        "Failed to register material: {}",
                        err
                    )));
                }
            }
        }

        // Get total materials count from registry
        let total_materials = self.registry.list_materials().await.len();

        // Return counts including total registry count
        Ok((found_count, failed_count, registered_count, total_materials))
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
    type Result = ResponseFuture<Result<messages::DiscoverySuccess, messages::DiscoveryError>>;

    fn handle(&mut self, msg: messages::StartDiscovery, _ctx: &mut Self::Context) -> Self::Result {
        info!(
            "DiscoveryActor '{}' starting discovery in '{}'",
            self.name, msg.config.directory
        );

        // Capture actor fields for use in async block
        let registry = self.registry.clone();
        let validate_fn = self.validate_directory(&msg.config.directory);
        let scan_config = msg.config;
        let actor_name = self.name.clone();

        // Create async block for scanning and processing
        Box::pin(async move {
            // First validate the directory
            validate_fn?;

            // Create scanner
            let scanner = DirectoryScanner::new(&scan_config.directory)
                .map_err(|e| messages::DiscoveryError::ScannerError(format!("{}", e)))?
                .ignore_hidden(scan_config.ignore_hidden)
                .exclude(scan_config.exclude_patterns);

            // Perform scan
            info!("Starting scan in directory: {}", scan_config.directory);
            let scan_results = scanner
                .scan()
                .map_err(|e| messages::DiscoveryError::ScannerError(format!("{}", e)))?;

            // Log the basic results
            info!(
                "Scan complete. Found {} materials, {} failed",
                scan_results.found.len(),
                scan_results.failed.len()
            );

            // Create DiscoveryActor with the same registry to use its methods
            let discovery_actor = DiscoveryActor {
                name: actor_name,
                registry,
            };

            // Register the discovered materials using the dedicated method
            let (found_count, failed_count, registered_count, total_materials) =
                discovery_actor.register_materials(scan_results).await?;

            // Log the registration results
            info!(
                "Registration complete. Found: {}, Failed: {}, Registered: {}, Total in registry: {}",
                found_count, failed_count, registered_count, total_materials
            );

            Ok(messages::DiscoverySuccess { success: true })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EventBus;
    use crate::materials::{InMemoryMaterialRepository, MaterialStatus};
    use std::fs::File;
    use std::sync::Arc;
    use tempfile::tempdir;

    fn init_test_logger() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .try_init();
    }

    #[actix::test]
    async fn test_discovery_actor_ping() {
        init_test_logger();

        // Create a registry with an event bus
        let repository = Arc::new(InMemoryMaterialRepository::new());
        let event_bus = Arc::new(EventBus::new());
        let registry = MaterialRegistry::new(repository, event_bus);

        // Create a discovery actor
        let actor = DiscoveryActor::new("test-discovery", registry).start();

        // Send a ping message
        let response = actor.send(Ping).await.unwrap();
        assert!(response, "Ping should return true");
    }

    #[actix::test]
    async fn test_discovery_actor_shutdown() {
        init_test_logger();

        // Create a registry with an event bus
        let repository = Arc::new(InMemoryMaterialRepository::new());
        let event_bus = Arc::new(EventBus::new());
        let registry = MaterialRegistry::new(repository, event_bus);

        // Create a discovery actor
        let actor = DiscoveryActor::new("test-discovery", registry).start();

        // Send a shutdown message
        let _ = actor.send(Shutdown).await;
        // Actor should have been stopped by the message
    }

    #[actix::test]
    async fn test_discovery_valid_directory() {
        init_test_logger();

        // Create a temporary directory with a test file
        let dir = tempdir().unwrap();
        let test_file_path = dir.path().join("test.md");
        File::create(&test_file_path).expect("Failed to create test file");

        // Create a registry with an event bus
        let repository = Arc::new(InMemoryMaterialRepository::new());
        let event_bus = Arc::new(EventBus::new());

        // Create a subscriber to keep the event channel open
        let _subscriber = event_bus.subscribe();

        let registry = MaterialRegistry::new(repository, event_bus.clone());

        // Create a discovery actor
        let actor = DiscoveryActor::new("test-discovery", registry.clone()).start();

        // Create scan config
        let config = DiscoveryConfig {
            directory: dir.path().to_string_lossy().to_string(),
            ignore_hidden: true,
            exclude_patterns: vec![],
        };

        // Send start discovery message
        let result = actor
            .send(messages::StartDiscovery { config })
            .await
            .unwrap();
        let success = result.unwrap();
        assert!(success.success);

        // Check that the material was registered in the registry
        let materials = registry.list_materials().await;
        assert_eq!(materials.len(), 1, "Registry should have one material");
        assert_eq!(
            materials[0].status,
            MaterialStatus::Discovered,
            "Material status should be Discovered"
        );
    }

    #[actix::test]
    async fn test_discovery_invalid_directory() {
        init_test_logger();

        let invalid_path = "/path/to/nonexistent/directory";

        // Create a registry with an event bus
        let repository = Arc::new(InMemoryMaterialRepository::new());
        let event_bus = Arc::new(EventBus::new());
        let registry = MaterialRegistry::new(repository, event_bus);

        // Create a discovery actor
        let actor = DiscoveryActor::new("test-discovery", registry).start();

        // Create scan config
        let config = DiscoveryConfig {
            directory: invalid_path.to_string(),
            ignore_hidden: true,
            exclude_patterns: vec![],
        };

        // Send start discovery message and check for the expected error
        let result = actor
            .send(messages::StartDiscovery { config })
            .await
            .unwrap();

        match result {
            Ok(_) => panic!("Expected an error for invalid directory"),
            Err(err) => {
                assert!(
                    matches!(err, messages::DiscoveryError::DirectoryNotFound(_)),
                    "Expected DirectoryNotFound error, got: {:?}",
                    err
                );
            }
        }
    }

    #[actix::test]
    async fn test_discovery_with_exclude_patterns() {
        init_test_logger();

        // Create a temporary directory with several test files
        let dir = tempdir().unwrap();
        let test_file_path = dir.path().join("test.md");
        let subdir = dir.path().join("subdir");
        std::fs::create_dir(&subdir).expect("Failed to create subdirectory");
        File::create(&test_file_path).expect("Failed to create test file");
        File::create(subdir.join("excluded.md")).expect("Failed to create test file");

        // Create a registry with an event bus
        let repository = Arc::new(InMemoryMaterialRepository::new());
        let event_bus = Arc::new(EventBus::new());

        // Create a subscriber to keep the event channel open
        let _subscriber = event_bus.subscribe();

        let registry = MaterialRegistry::new(repository, event_bus);

        // Create a discovery actor
        let actor = DiscoveryActor::new("test-discovery", registry.clone()).start();

        // Create scan config with exclude patterns - use the exact subdir path
        let subdir_str = format!("{}", subdir.display());
        let config = DiscoveryConfig {
            directory: dir.path().to_string_lossy().to_string(),
            ignore_hidden: true,
            exclude_patterns: vec![subdir_str],
        };

        // Send start discovery message
        let result = actor
            .send(messages::StartDiscovery { config })
            .await
            .unwrap();
        let success = result.unwrap();
        assert!(success.success);

        // Check that the material was registered in the registry
        let materials = registry.list_materials().await;
        assert_eq!(materials.len(), 1, "Registry should have one material");

        // Verify the material is the test.md file (not the excluded one)
        assert!(
            materials[0].file_path.ends_with("test.md"),
            "Material path should be the test.md file, not the excluded one"
        );
    }
}
