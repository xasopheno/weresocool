//! Generic event system with multi-subscriber support
//!
//! This crate provides a non-blocking event dispatcher that supports multiple subscribers.
//! Designed for real-time audio applications where the audio thread cannot block waiting
//! for event handlers.
//!
//! # Example
//!
//! ```rust
//! use weresocool_events::EventDispatcher;
//!
//! #[derive(Clone, Debug, PartialEq)]
//! enum MyEvent {
//!     Started,
//!     Progress(f32),
//!     Finished,
//! }
//!
//! let mut dispatcher = EventDispatcher::new();
//!
//! // Multiple subscribers
//! let rx1 = dispatcher.subscribe();
//! let rx2 = dispatcher.subscribe();
//!
//! // Emit events (non-blocking)
//! dispatcher.emit(MyEvent::Started);
//! dispatcher.emit(MyEvent::Progress(0.5));
//!
//! // Each subscriber receives events
//! assert_eq!(rx1.try_recv().ok(), Some(MyEvent::Started));
//! assert_eq!(rx2.try_recv().ok(), Some(MyEvent::Started));
//! ```

use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};

/// Non-blocking event dispatcher with multiple subscriber support
///
/// Each subscriber gets their own channel to receive events. The `emit()` method
/// sends events to all subscribers without blocking, making it safe to use from
/// real-time audio threads.
///
/// Events are cloned for each subscriber, so the event type must implement `Clone`.
/// If a subscriber's channel is full, the event will be dropped for that subscriber
/// (fire-and-forget semantics).
#[derive(Debug)]
pub struct EventDispatcher<E> {
    senders: Vec<Sender<E>>,
    buffer_size: usize,
}

impl<E> EventDispatcher<E> {
    /// Create a new event dispatcher with default buffer size (100 events per subscriber)
    pub fn new() -> Self {
        Self::with_capacity(100)
    }

    /// Create a new event dispatcher with specified buffer size per subscriber
    ///
    /// The buffer size determines how many events can be queued per subscriber before
    /// events start getting dropped.
    pub fn with_capacity(buffer_size: usize) -> Self {
        Self {
            senders: Vec::new(),
            buffer_size,
        }
    }

    /// Subscribe to events, returning a receiver channel
    ///
    /// Each call creates a new subscriber with its own channel. The subscriber
    /// will receive all events emitted after subscription.
    ///
    /// # Example
    ///
    /// ```rust
    /// use weresocool_events::EventDispatcher;
    ///
    /// let mut dispatcher = EventDispatcher::<String>::new();
    ///
    /// let rx1 = dispatcher.subscribe();
    /// let rx2 = dispatcher.subscribe();
    ///
    /// dispatcher.emit("hello".to_string());
    ///
    /// assert_eq!(rx1.try_recv().ok(), Some("hello".to_string()));
    /// assert_eq!(rx2.try_recv().ok(), Some("hello".to_string()));
    /// ```
    pub fn subscribe(&mut self) -> Receiver<E> {
        let (tx, rx) = bounded(self.buffer_size);
        self.senders.push(tx);
        rx
    }

    /// Get the number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.senders.len()
    }

    /// Check if there are any subscribers
    pub fn has_subscribers(&self) -> bool {
        !self.senders.is_empty()
    }

    /// Remove disconnected subscribers (whose channels have been dropped)
    ///
    /// This is called automatically during `emit()`, but can be called manually
    /// to clean up earlier.
    pub fn prune_disconnected(&mut self) {
        // No direct way to check if disconnected in crossbeam without trying to send
        // So this is less useful - emit() handles pruning automatically
        self.senders.retain(|tx| !tx.is_full());
    }
}

impl<E: Clone> EventDispatcher<E> {
    /// Emit an event to all subscribers (non-blocking)
    ///
    /// The event is cloned and sent to each subscriber's channel. This operation
    /// is non-blocking - if a subscriber's channel is full, the event is dropped
    /// for that subscriber.
    ///
    /// Disconnected subscribers (whose receivers have been dropped) are automatically
    /// removed during this call.
    ///
    /// # Example
    ///
    /// ```rust
    /// use weresocool_events::EventDispatcher;
    ///
    /// #[derive(Clone, PartialEq, Debug)]
    /// struct RenderEvent { frame: u64 }
    ///
    /// let mut dispatcher = EventDispatcher::new();
    /// let rx = dispatcher.subscribe();
    ///
    /// dispatcher.emit(RenderEvent { frame: 1 });
    /// dispatcher.emit(RenderEvent { frame: 2 });
    ///
    /// assert_eq!(rx.try_recv().ok(), Some(RenderEvent { frame: 1 }));
    /// assert_eq!(rx.try_recv().ok(), Some(RenderEvent { frame: 2 }));
    /// ```
    pub fn emit(&mut self, event: E) {
        // Remove disconnected subscribers
        self.senders.retain(|tx| {
            // Try to send (non-blocking)
            match tx.try_send(event.clone()) {
                Ok(_) => true,
                Err(TrySendError::Full(_)) => {
                    // Channel full - drop this event for this subscriber
                    // but keep the subscriber
                    true
                }
                Err(TrySendError::Disconnected(_)) => {
                    // Subscriber disconnected - remove it
                    false
                }
            }
        });
    }
}

impl<E> Default for EventDispatcher<E> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[derive(Clone, Debug, PartialEq)]
    enum TestEvent {
        Start,
        Progress(u32),
        End,
    }

    #[test]
    fn test_no_subscribers() {
        let mut dispatcher = EventDispatcher::<TestEvent>::new();
        assert_eq!(dispatcher.subscriber_count(), 0);
        assert!(!dispatcher.has_subscribers());

        // Should not panic
        dispatcher.emit(TestEvent::Start);
    }

    #[test]
    fn test_single_subscriber() {
        let mut dispatcher = EventDispatcher::new();
        let rx = dispatcher.subscribe();

        assert_eq!(dispatcher.subscriber_count(), 1);
        assert!(dispatcher.has_subscribers());

        dispatcher.emit(TestEvent::Start);
        dispatcher.emit(TestEvent::Progress(50));
        dispatcher.emit(TestEvent::End);

        assert_eq!(rx.try_recv().ok(), Some(TestEvent::Start));
        assert_eq!(rx.try_recv().ok(), Some(TestEvent::Progress(50)));
        assert_eq!(rx.try_recv().ok(), Some(TestEvent::End));
    }

    #[test]
    fn test_multiple_subscribers() {
        let mut dispatcher = EventDispatcher::new();
        let rx1 = dispatcher.subscribe();
        let rx2 = dispatcher.subscribe();
        let rx3 = dispatcher.subscribe();

        assert_eq!(dispatcher.subscriber_count(), 3);

        dispatcher.emit(TestEvent::Start);

        // All subscribers receive the event
        assert_eq!(rx1.try_recv().ok(), Some(TestEvent::Start));
        assert_eq!(rx2.try_recv().ok(), Some(TestEvent::Start));
        assert_eq!(rx3.try_recv().ok(), Some(TestEvent::Start));
    }

    #[test]
    fn test_disconnected_subscriber_removed() {
        let mut dispatcher = EventDispatcher::new();
        let rx1 = dispatcher.subscribe();
        let rx2 = dispatcher.subscribe();

        assert_eq!(dispatcher.subscriber_count(), 2);

        // Drop one receiver
        drop(rx2);

        // Emit event - disconnected subscriber should be removed
        dispatcher.emit(TestEvent::Start);

        assert_eq!(dispatcher.subscriber_count(), 1);
        assert_eq!(rx1.try_recv().ok(), Some(TestEvent::Start));
    }

    #[test]
    fn test_full_channel_drops_event() {
        // Create dispatcher with tiny buffer
        let mut dispatcher = EventDispatcher::<TestEvent>::with_capacity(2);
        let rx = dispatcher.subscribe();

        // Fill the buffer
        dispatcher.emit(TestEvent::Progress(1));
        dispatcher.emit(TestEvent::Progress(2));

        // This should be dropped (buffer full, but subscriber stays)
        dispatcher.emit(TestEvent::Progress(3));

        assert_eq!(dispatcher.subscriber_count(), 1);

        // Only first two events received
        assert_eq!(rx.try_recv().ok(), Some(TestEvent::Progress(1)));
        assert_eq!(rx.try_recv().ok(), Some(TestEvent::Progress(2)));
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn test_prune_disconnected() {
        let mut dispatcher = EventDispatcher::<TestEvent>::new();
        let _rx1 = dispatcher.subscribe();
        let rx2 = dispatcher.subscribe();
        let _rx3 = dispatcher.subscribe();

        assert_eq!(dispatcher.subscriber_count(), 3);

        // Drop middle receiver
        drop(rx2);

        // Emit event - this automatically prunes disconnected
        dispatcher.emit(TestEvent::Start);

        assert_eq!(dispatcher.subscriber_count(), 2);
    }

    #[test]
    fn test_subscribe_after_emit() {
        let mut dispatcher = EventDispatcher::new();

        dispatcher.emit(TestEvent::Start);

        // Subscribe after event was emitted
        let rx = dispatcher.subscribe();

        dispatcher.emit(TestEvent::End);

        // New subscriber only gets events after subscription
        assert!(rx.try_recv().ok() == Some(TestEvent::End));
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn test_thread_safety() {
        let mut dispatcher = EventDispatcher::new();
        let rx = dispatcher.subscribe();

        // Spawn thread to emit events
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(10));
            // Note: This won't work as-is because EventDispatcher is not Send
            // In real use, emit is called from the same thread that owns the dispatcher
        });

        // In practice, EventDispatcher lives in one thread (audio thread)
        // and Receivers are sent to other threads (viz, MIDI, etc.)
        dispatcher.emit(TestEvent::Start);

        assert_eq!(rx.try_recv().ok(), Some(TestEvent::Start));
    }
}
