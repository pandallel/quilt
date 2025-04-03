// This file can contain other general types that don't fit into specific modules
// Event-related types have been moved to src/events.rs

use std::sync::mpsc::{channel, Receiver, Sender};

/// A simple handle for receiving events
pub struct EventListener<T> {
    receiver: Receiver<T>,
}

impl<T> EventListener<T> {
    /// Receive the next event, blocking until one is available
    pub fn recv(&self) -> Option<T> {
        self.receiver.recv().ok()
    }

    /// Try to receive an event without blocking
    pub fn try_recv(&self) -> Option<T> {
        self.receiver.try_recv().ok()
    }
}

/// A simple event source that can have multiple listeners
pub struct EventSource<T> {
    listeners: Vec<Sender<T>>,
}

impl<T: Clone> EventSource<T> {
    /// Create a new event source
    pub fn new() -> Self {
        Self {
            listeners: Vec::new(),
        }
    }

    /// Add a new listener and return a handle to receive events
    pub fn listen(&mut self) -> EventListener<T> {
        let (sender, receiver) = channel();
        self.listeners.push(sender);
        EventListener { receiver }
    }

    /// Send an event to all listeners
    pub fn send(&self, event: T) {
        self.listeners.retain(|listener| listener.send(event.clone()).is_ok());
    }
}

impl<T> Default for EventSource<T> {
    fn default() -> Self {
        Self::new()
    }
} 