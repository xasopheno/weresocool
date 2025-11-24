//! Event types for WereSoCool rendering
//!
//! This module defines the events that can be emitted during audio rendering.
//! These events can be subscribed to by visualization, MIDI output, logging, and other systems.

use crate::generation::Op4D;
use opmap::OpMap;
use weresocool_events::EventDispatcher;
use weresocool_instrument::renderable::RenderOp;

/// Events emitted during rendering for visualization
#[derive(Clone, Debug)]
pub enum RenderEvent {
    /// Operations ready for visualization (per-frame data)
    ///
    /// Contains a map of operation data organized by name/color for rendering.
    /// This is sent for each audio buffer rendered.
    Ops(OpMap<Op4D>),

    /// Rendering has been reset (new composition loaded)
    ///
    /// Signals that the renderer state has been cleared and visualization
    /// should reset.
    Reset,

    /// Audio buffer is ready
    ///
    /// Signals that a new audio buffer has been rendered and is ready for playback.
    AudioReady,

    /// Visualization is ready
    ///
    /// Response event from visualization system indicating it's ready to receive
    /// more data. Used for synchronization.
    VisReady,
}

/// Events for MIDI output
#[derive(Clone, Debug)]
pub struct MidiEvent {
    /// The render operations containing note/timing information
    pub ops: Vec<RenderOp>,

    /// Absolute timestamp in seconds from start of playback
    pub timestamp: f64,
}

/// Playback state change events
#[derive(Clone, Debug, PartialEq)]
pub enum StateEvent {
    /// Playback paused or resumed
    Paused(bool),

    /// Volume changed (0.0 to 1.0+)
    Volume(f32),

    /// Playback position changed (in seconds)
    Position(f64),

    /// Rendering started
    Started,

    /// Rendering stopped/finished
    Stopped,
}

/// Event dispatchers for all WereSoCool events
///
/// This struct holds the event dispatchers for different event types.
/// Multiple subscribers can listen to each event type independently.
///
/// # Example
///
/// ```rust,ignore
/// use weresocool_core::events::{Events, RenderEvent};
///
/// let mut events = Events::new();
///
/// // Subscribe to render events
/// let render_rx = events.render.subscribe();
///
/// // Subscribe to state events
/// let state_rx = events.state.subscribe();
///
/// // Emit events
/// events.render.emit(RenderEvent::Reset);
///
/// // Receive events
/// if let Ok(event) = render_rx.try_recv() {
///     println!("Got render event: {:?}", event);
/// }
/// ```
#[derive(Debug)]
pub struct Events {
    /// Rendering events (for visualization, analysis)
    pub render: EventDispatcher<RenderEvent>,

    /// MIDI output events
    pub midi: EventDispatcher<MidiEvent>,

    /// Playback state events
    pub state: EventDispatcher<StateEvent>,
}

impl Events {
    /// Create a new Events struct with default settings
    pub fn new() -> Self {
        Self {
            render: EventDispatcher::new(),
            midi: EventDispatcher::new(),
            state: EventDispatcher::new(),
        }
    }

    /// Create a new Events struct with custom buffer sizes
    pub fn with_capacity(buffer_size: usize) -> Self {
        Self {
            render: EventDispatcher::with_capacity(buffer_size),
            midi: EventDispatcher::with_capacity(buffer_size),
            state: EventDispatcher::with_capacity(buffer_size),
        }
    }
}

impl Default for Events {
    fn default() -> Self {
        Self::new()
    }
}
