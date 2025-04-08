// Discovery module for Quilt
// Responsible for discovering materials in directories

pub mod actor;

// Re-export the actor for easy access
pub use self::actor::DiscoveryActor; 