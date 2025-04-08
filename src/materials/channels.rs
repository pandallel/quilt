use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;
use thiserror::Error;

use super::messages::MaterialMessage;

/// Default channel capacity for all pipeline channels
/// 
/// This value was chosen to provide enough buffering for temporary spikes in
/// document discovery, while still allowing backpressure to propagate if
/// downstream stages (especially Labeling) get overwhelmed.
pub const DEFAULT_CHANNEL_CAPACITY: usize = 100;

/// Errors that can occur during channel operations
/// 
/// This enum provides structured error types for different channel-related
/// failures, making it easier to diagnose and recover from communication issues.
#[derive(Error, Debug)]
pub enum ChannelError {
    /// Error when sending a message to a full or closed channel
    #[error("Failed to send message: {0}")]
    SendError(String),
    
    /// Error when a receive operation times out
    #[error("Channel receive timeout after {0:?}")]
    ReceiveTimeout(Duration),
    
    /// Error when a channel has been closed
    #[error("Channel closed")]
    ChannelClosed,
}

/// A paired sender and receiver for a specific channel stage
/// 
/// This struct encapsulates both ends of a channel to simplify channel management
/// and ensure that senders and receivers are created and passed together.
pub struct ChannelPair {
    /// The sender end of the channel
    pub sender: mpsc::Sender<MaterialMessage>,
    
    /// The receiver end of the channel
    pub receiver: mpsc::Receiver<MaterialMessage>,
}

/// Type alias for the Discovery-to-Cutting channel
/// 
/// This is used by the Discovery worker to send messages to the Cutting stage.
pub type CuttingChannel = mpsc::Sender<MaterialMessage>;

/// Type alias for the Cutting-to-Labeling channel
/// 
/// This is used by the Cutting worker to send messages to the Labeling stage.
pub type LabelingChannel = mpsc::Sender<MaterialMessage>;

/// Creates a new channel pair with the default capacity
/// 
/// This is the preferred method for creating channels in the pipeline,
/// as it uses a consistent capacity for all stages.
pub fn create_channel() -> ChannelPair {
    create_channel_with_capacity(DEFAULT_CHANNEL_CAPACITY)
}

/// Creates a new channel pair with the specified capacity
/// 
/// This allows creating channels with custom capacity for special cases,
/// such as adapting to different processing characteristics of worker stages.
pub fn create_channel_with_capacity(capacity: usize) -> ChannelPair {
    let (sender, receiver) = mpsc::channel(capacity);
    ChannelPair { sender, receiver }
}

/// Extension trait to provide helper methods for working with material channels
/// 
/// This trait adds convenience methods to channel senders to improve error handling
/// and simplify common operations like sending shutdown signals.
#[allow(async_fn_in_trait)]
pub trait MaterialChannelExt {
    /// Send a message with error handling
    /// 
    /// Provides a more ergonomic way to send messages with proper error handling.
    async fn send_message(&self, message: MaterialMessage) -> Result<(), ChannelError>;
    
    /// Try to send a message with a timeout
    /// 
    /// This allows setting a maximum time to wait when sending to a channel that
    /// might be full, avoiding indefinite blocking.
    async fn try_send_message_timeout(
        &self, 
        message: MaterialMessage, 
        timeout_duration: Duration
    ) -> Result<(), ChannelError>;
    
    /// Send a shutdown signal to the channel
    /// 
    /// Convenience method for sending the Shutdown message to gracefully
    /// terminate a worker.
    async fn send_shutdown(&self) -> Result<(), ChannelError>;
}

/// Implement the extension trait for mpsc::Sender<MaterialMessage>
impl MaterialChannelExt for mpsc::Sender<MaterialMessage> {
    async fn send_message(&self, message: MaterialMessage) -> Result<(), ChannelError> {
        self.send(message)
            .await
            .map_err(|e| ChannelError::SendError(e.to_string()))
    }
    
    async fn try_send_message_timeout(
        &self, 
        message: MaterialMessage, 
        timeout_duration: Duration
    ) -> Result<(), ChannelError> {
        timeout(
            timeout_duration,
            self.send(message)
        )
        .await
        .map_err(|_| ChannelError::ReceiveTimeout(timeout_duration))?
        .map_err(|e| ChannelError::SendError(e.to_string()))
    }
    
    async fn send_shutdown(&self) -> Result<(), ChannelError> {
        self.send_message(MaterialMessage::Shutdown).await
    }
}

/// Extension trait for receiver to provide timeout capabilities
/// 
/// Adds additional functionality to channel receivers, particularly for
/// implementing timeouts on receive operations.
#[allow(async_fn_in_trait)]
pub trait MaterialReceiverExt {
    /// Receive a message with a timeout
    /// 
    /// This allows setting a maximum wait time when receiving from a channel,
    /// which is useful for implementing non-blocking behavior or timeouts.
    async fn receive_timeout(
        &mut self, 
        timeout_duration: Duration
    ) -> Result<MaterialMessage, ChannelError>;
}

/// Implement the extension trait for mpsc::Receiver<MaterialMessage>
impl MaterialReceiverExt for mpsc::Receiver<MaterialMessage> {
    async fn receive_timeout(
        &mut self, 
        timeout_duration: Duration
    ) -> Result<MaterialMessage, ChannelError> {
        timeout(
            timeout_duration,
            self.recv()
        )
        .await
        .map_err(|_| ChannelError::ReceiveTimeout(timeout_duration))?
        .ok_or(ChannelError::ChannelClosed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::materials::types::Material;
    
    #[tokio::test]
    async fn test_channel_creation() {
        let channel = create_channel();
        assert!(channel.sender.capacity() >= DEFAULT_CHANNEL_CAPACITY);
    }
    
    #[tokio::test]
    async fn test_channel_with_capacity() {
        let capacity = 50;
        let channel = create_channel_with_capacity(capacity);
        assert!(channel.sender.capacity() >= capacity);
    }
    
    #[tokio::test]
    async fn test_send_and_receive() {
        let ChannelPair { sender, mut receiver } = create_channel();
        
        let material = Material::new("test/doc.md".to_string());
        let message = MaterialMessage::Discovered(material.clone());
        
        // Send the message
        sender.send_message(message.clone()).await.unwrap();
        
        // Receive the message
        let received = receiver.recv().await.unwrap();
        
        assert_eq!(received, message);
    }
    
    #[tokio::test]
    async fn test_send_shutdown() {
        let ChannelPair { sender, mut receiver } = create_channel();
        
        // Send shutdown signal
        sender.send_shutdown().await.unwrap();
        
        // Receive the message
        let received = receiver.recv().await.unwrap();
        
        assert!(matches!(received, MaterialMessage::Shutdown));
    }
} 