use crate::events::types::QuiltEvent;
use thiserror::Error;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tracing::{debug, error};

/// Default capacity for the event bus channel
const DEFAULT_CHANNEL_CAPACITY: usize = 128;

/// Errors that can occur during event bus operations
#[derive(Error, Debug)]
pub enum EventBusError {
    /// Error when trying to send an event
    #[error("Failed to send event: {0}")]
    SendError(String),

    /// Error when trying to receive an event
    #[error("Failed to receive event: {0}")]
    ReceiveError(String),
}

/// Event Bus for broadcasting events throughout the system
#[derive(Debug, Clone)]
pub struct EventBus {
    /// The sender for broadcasting events
    sender: Sender<QuiltEvent>,
}

impl EventBus {
    /// Create a new event bus with default capacity
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CHANNEL_CAPACITY)
    }

    /// Create a new event bus with the specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Publish an event to all subscribers
    pub fn publish(&self, event: QuiltEvent) -> Result<(), EventBusError> {
        debug!("Publishing event: {}", event);

        if self.sender.receiver_count() == 0 {
            debug!("No subscribers for event: {}", event);
        }

        self.sender.send(event.clone()).map_err(|e| {
            error!("Failed to send event: {}: {}", event, e);
            EventBusError::SendError(e.to_string())
        })?;

        debug!(
            "Successfully published event to {} receivers",
            self.sender.receiver_count()
        );
        Ok(())
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> Receiver<QuiltEvent> {
        let receiver = self.sender.subscribe();
        debug!(
            "New subscription created. Current receiver count: {}",
            self.sender.receiver_count()
        );
        receiver
    }

    /// Get the number of subscribers
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

/// Helper trait for subscribing to specific events
pub trait EventSubscriber {
    /// Start processing events from a receiver
    fn process_events(&self, receiver: Receiver<QuiltEvent>);
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::materials::types::Material;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_event_bus_publish_subscribe() {
        let bus = EventBus::new();
        let mut receiver = bus.subscribe();

        // Create and publish a test event
        let material = Material::new("test/file.md".to_string());
        let event = QuiltEvent::material_discovered(&material);

        // Publish the event
        bus.publish(event.clone()).unwrap();

        // Receive the event
        let received = receiver.recv().await.unwrap();

        // Verify the received event
        if let QuiltEvent::MaterialDiscovered(evt) = received {
            assert_eq!(evt.material_id, material.id);
        } else {
            panic!("Received wrong event type");
        }
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let bus = EventBus::new();
        let mut receiver1 = bus.subscribe();
        let mut receiver2 = bus.subscribe();

        assert_eq!(bus.subscriber_count(), 2);

        // Create and publish a test event
        let material = Material::new("test/file.md".to_string());
        let event = QuiltEvent::material_discovered(&material);

        // Publish the event
        bus.publish(event.clone()).unwrap();

        // Both receivers should get the event
        let received1 = receiver1.recv().await.unwrap();
        let received2 = receiver2.recv().await.unwrap();

        if let QuiltEvent::MaterialDiscovered(evt) = received1 {
            assert_eq!(evt.material_id, material.id);
        } else {
            panic!("Receiver 1 got wrong event type");
        }

        if let QuiltEvent::MaterialDiscovered(evt) = received2 {
            assert_eq!(evt.material_id, material.id);
        } else {
            panic!("Receiver 2 got wrong event type");
        }
    }

    #[tokio::test]
    async fn test_event_processing() {
        // Create an event bus
        let bus = EventBus::new();
        let receiver = bus.subscribe();

        // Channel to communicate test results
        let (tx, mut rx) = mpsc::channel(10);

        // Spawn a task to process events
        tokio::spawn(async move {
            let mut receiver = receiver;
            loop {
                match receiver.recv().await {
                    Ok(event) => {
                        if let QuiltEvent::MaterialDiscovered(evt) = event {
                            tx.send(evt.material_id.clone()).await.unwrap();
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        // Create and publish events
        let material = Material::new("test/file.md".to_string());
        let material_id = material.id.clone();
        let event = QuiltEvent::material_discovered(&material);

        bus.publish(event).unwrap();

        // Verify the event was processed
        let received_id = rx.recv().await.unwrap();
        assert_eq!(received_id, material_id);
    }
}
