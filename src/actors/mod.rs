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

/// Error types for actor operations
pub mod error {
    use thiserror::Error;

    /// Actor operation errors
    #[derive(Debug, Error)]
    pub enum ActorError {
        /// Error when an actor is not available
        #[error("Actor is not available: {0}")]
        NotAvailable(String),
        
        /// Error when sending a message to an actor fails
        #[error("Failed to send message to actor: {0}")]
        MessageSendFailure(String),
        
        /// Error when receiving a response from an actor fails
        #[error("Failed to receive response from actor: {0}")]
        ResponseFailure(String),
        
        /// Generic actor operation error
        #[error("Actor operation failed: {0}")]
        OperationFailure(String),
    }
}

// Re-export common types
pub use self::messages::*;
pub use self::error::*;

#[cfg(test)]
mod tests {
    use super::*;
    use actix::prelude::*;
    
    // A simple test actor for unit testing
    struct TestActor;
    
    impl Actor for TestActor {
        type Context = Context<Self>;
    }
    
    impl Handler<Ping> for TestActor {
        type Result = bool;
        
        fn handle(&mut self, _msg: Ping, _ctx: &mut Self::Context) -> Self::Result {
            true
        }
    }
    
    impl Handler<Shutdown> for TestActor {
        type Result = ();
        
        fn handle(&mut self, _msg: Shutdown, ctx: &mut Self::Context) -> Self::Result {
            ctx.stop();
        }
    }
    
    #[actix::test]
    async fn test_ping_message() {
        // Start the test actor
        let actor = TestActor.start();
        
        // Send a ping message
        let result = actor.send(Ping).await;
        
        // Check that the message was received and handled correctly
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
    
    #[actix::test]
    async fn test_shutdown_message() {
        // Start the test actor
        let actor = TestActor.start();
        
        // Send a shutdown message
        let result = actor.send(Shutdown).await;
        
        // Check that the message was received and handled
        assert!(result.is_ok());
        
        // Try to send another message, which should fail since the actor is stopped
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let ping_result = actor.send(Ping).await;
        assert!(ping_result.is_err());
    }
}
