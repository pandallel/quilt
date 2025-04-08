// Actor system module for Quilt
// Defines common actor types, traits, and utilities

/// Common message types that can be shared across actors
pub mod messages {
    use actix::prelude::*;

    /// Message to check if an actor is ready
    #[derive(Message)]
    #[rtype(result = "bool")]
    pub struct Ping;

    /// Message to request an actor to shut down
    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct Shutdown;
}

// Re-export common types
pub use self::messages::*;
