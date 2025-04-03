/// A simple event listener that receives events through a callback
#[derive(Debug)]
pub struct EventListener {
    id: usize,
}

/// A simple event source that can have multiple listeners
#[derive(Default)]
pub struct EventSource<T: Clone> {
    next_id: usize,
    listeners: Vec<(usize, Box<dyn FnMut(&T)>)>,
}

impl<T: Clone> EventSource<T> {
    /// Create a new event source
    pub fn new() -> Self {
        Self {
            next_id: 0,
            listeners: Vec::new(),
        }
    }

    /// Add a new listener that will receive events through the provided callback
    pub fn on<F>(&mut self, callback: F) -> EventListener
    where
        F: FnMut(&T) + 'static,
    {
        let id = self.next_id;
        self.next_id += 1;
        self.listeners.push((id, Box::new(callback)));
        EventListener { id }
    }

    /// Send an event to all listeners
    pub fn emit(&mut self, event: T) {
        for (_, listener) in &mut self.listeners {
            listener(&event);
        }
    }

    /// Remove a listener
    pub fn remove(&mut self, listener: EventListener) {
        self.listeners.retain(|(id, _)| *id != listener.id);
    }
}

/// Trait for types that can emit events
pub trait EventEmitter<Event> {
    /// Add a listener for events
    fn on<F>(&mut self, callback: F) -> EventListener
    where
        F: FnMut(&Event) + 'static;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_single_listener() {
        let mut source = EventSource::new();
        let received = Rc::new(RefCell::new(None));
        let received_clone = received.clone();

        let _listener = source.on(move |event: &String| {
            *received_clone.borrow_mut() = Some(event.clone());
        });

        source.emit("test event".to_string());
        assert_eq!(*received.borrow(), Some("test event".to_string()));
    }

    #[test]
    fn test_multiple_listeners() {
        let mut source = EventSource::new();
        let received1 = Rc::new(RefCell::new(None));
        let received2 = Rc::new(RefCell::new(None));
        let r1 = received1.clone();
        let r2 = received2.clone();

        let _listener1 = source.on(move |event: &String| {
            *r1.borrow_mut() = Some(event.clone());
        });
        let _listener2 = source.on(move |event: &String| {
            *r2.borrow_mut() = Some(event.clone());
        });

        source.emit("test event".to_string());
        assert_eq!(*received1.borrow(), Some("test event".to_string()));
        assert_eq!(*received2.borrow(), Some("test event".to_string()));
    }

    #[test]
    fn test_listener_receives_multiple_events() {
        let mut source = EventSource::new();
        let received = Rc::new(RefCell::new(Vec::new()));
        let received_clone = received.clone();

        let _listener = source.on(move |event: &String| {
            received_clone.borrow_mut().push(event.clone());
        });

        source.emit("event 1".to_string());
        source.emit("event 2".to_string());
        source.emit("event 3".to_string());

        assert_eq!(*received.borrow(), vec![
            "event 1".to_string(),
            "event 2".to_string(),
            "event 3".to_string(),
        ]);
    }

    #[test]
    fn test_remove_listener() {
        let mut source = EventSource::new();
        let received = Rc::new(RefCell::new(Vec::new()));
        let received_clone = received.clone();

        let listener = source.on(move |event: &String| {
            received_clone.borrow_mut().push(event.clone());
        });

        source.emit("event 1".to_string());
        source.remove(listener);
        source.emit("event 2".to_string());

        assert_eq!(*received.borrow(), vec!["event 1".to_string()]);
    }

    #[test]
    fn test_default_creation() {
        let source: EventSource<String> = EventSource::default();
        assert_eq!(source.listeners.len(), 0);
    }

    #[test]
    fn test_complex_type_events() {
        #[derive(Debug, Clone, PartialEq)]
        struct TestEvent {
            id: u32,
            message: String,
        }

        let mut source = EventSource::new();
        let received = Rc::new(RefCell::new(None));
        let received_clone = received.clone();

        let _listener = source.on(move |event: &TestEvent| {
            *received_clone.borrow_mut() = Some(event.clone());
        });

        let event = TestEvent {
            id: 1,
            message: "test".to_string(),
        };

        source.emit(event.clone());
        assert_eq!(*received.borrow(), Some(event));
    }
} 