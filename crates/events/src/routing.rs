use std::fmt::Debug;

use crate::{Event, EventKind};

/// Abstract application event
pub trait AbstractEvent: Debug + TryFrom<Event, Error = EventRoutingError> {
    /// [`EventKind`] for routing
    const KIND: EventKind;
}

/// Event routing error
///
/// Occurs when an event is incorrectly routed to a subscriber
#[derive(Debug)]
pub struct EventRoutingError;

impl std::fmt::Display for EventRoutingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Event Routing Error")
    }
}

impl std::error::Error for EventRoutingError {}
