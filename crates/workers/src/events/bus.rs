use std::{collections::HashMap, marker::PhantomData};

use tokio::sync::broadcast::channel;

use crate::events::{
    AbstractEvent,
    event::{Event, EventKind, EventSender, EventStream},
};

/// Event bus for working and managing events
pub struct EventBus {
    senders: HashMap<EventKind, EventSender>,
    channel_size: usize,
}

impl EventBus {
    /// Creates a new [`EventBus`] with the specified channel buffer size
    pub fn new(channel_size: usize) -> Self {
        EventBus {
            senders: HashMap::new(),
            channel_size,
        }
    }

    /// Creates a new subscriber for the specified event
    pub fn subscribe<E>(&mut self) -> EventStream<E>
    where
        E: AbstractEvent,
    {
        let kind = E::KIND;

        if let Some(s) = self.senders.get(&kind) {
            return EventStream {
                marker: PhantomData,
                rx: s.tx.subscribe(),
            };
        }

        let (tx, rx) = channel(self.channel_size);
        self.senders.insert(kind, EventSender { tx });

        EventStream {
            marker: PhantomData,
            rx,
        }
    }

    /// Receives an event and sends it to everyone who subscribes to it
    ///
    /// Returns false if no one received the event
    pub fn publish<E>(&self, event: E) -> bool
    where
        E: AbstractEvent + Into<Event>,
    {
        let kind = E::KIND;

        let Some(sender) = self.senders.get(&kind) else {
            return false;
        };

        let event = event.into();
        sender.send(event)
    }
}
