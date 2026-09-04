use std::{any::TypeId, collections::HashMap, marker::PhantomData, sync::RwLock};

use tokio::sync::broadcast::channel;

use crate::{DynamicEvent, EventStream, channel::EventSender};

/// Event bus for working and managing events
pub struct EventBus {
    senders: RwLock<HashMap<TypeId, EventSender>>,
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

    /// Creates a new event listener for the specified event
    pub fn listener<E>(&self) -> EventStream<E>
    where
        E: DynamicEvent + 'static,
    {
        let id = TypeId::of::<E>();

        let mut senders = self.senders.write().unwrap_or_else(|e| e.into_inner());

        if let Some(s) = senders.get(&id) {
            return EventStream {
                marker: PhantomData,
                rx: s.tx.subscribe(),
            };
        }

        let (tx, rx) = channel(self.channel_size);
        let sender = EventSender { tx };
        senders.insert(id, sender);

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
        E: DynamicEvent + 'static,
    {
        let id = TypeId::of::<E>();

        let senders = self.senders.read().unwrap_or_else(|e| e.into_inner());
        let Some(sender) = senders.get(&id) else {
            return false;
        };

        sender.send(event)
    }
}
