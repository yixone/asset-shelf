use std::{collections::HashMap, marker::PhantomData, sync::RwLock};

use tokio::sync::broadcast::channel;

use crate::{AbstractEvent, Event, EventKind, EventSender, EventStream};

/// Event bus for working and managing events
pub struct EventBus {
    senders: RwLock<HashMap<EventKind, EventSender>>,
    channel_size: usize,
}

impl EventBus {
    /// Creates a new [`EventBus`] with the specified channel buffer size
    pub fn new(channel_size: usize) -> Self {
        EventBus {
            senders: RwLock::new(HashMap::new()),
            channel_size,
        }
    }

    /// Creates a new subscriber for the specified event
    pub fn subscribe<E>(&self) -> EventStream<E>
    where
        E: AbstractEvent,
    {
        let kind = E::KIND;

        let mut senders = self
            .senders
            .write()
            .expect("Failed to get a lock for event subscription");
        if let Some(s) = senders.get(&kind) {
            return EventStream {
                marker: PhantomData,
                rx: s.tx.subscribe(),
            };
        }
        let (tx, rx) = channel(self.channel_size);
        senders.insert(kind, EventSender { tx });

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

        let senders = self
            .senders
            .read()
            .expect("Failed to get a lock for event publishing");
        let Some(sender) = senders.get(&kind) else {
            return false;
        };

        let event = event.into();
        sender.send(event)
    }
}
