use std::marker::PhantomData;

use result::error::ResultExt;
use tokio::sync::broadcast::{Receiver, Sender};

use crate::events::{AbstractEvent, Event};

/// Event Sender
///
/// Sends an event to all subscribers for the specified Kind
pub struct EventSender {
    pub tx: Sender<Event>,
}

impl EventSender {
    /// Attempts to send an event to everyone subscribed to it
    pub fn send(&self, event: Event) -> bool {
        if let Err(e) = self.tx.send(event) {
            tracing::warn!(error = ?e, "Failed to send event");
            return false;
        }
        true
    }
}

/// Event Stream
///
/// Receives event as `<E>`
pub struct EventStream<E>
where
    E: AbstractEvent,
{
    pub marker: PhantomData<E>,
    pub rx: Receiver<Event>,
}

impl<E> EventStream<E>
where
    E: AbstractEvent + Clone,
{
    /// Receives the next [`Event`] for the [`EventStream`]
    pub async fn recv(&mut self) -> Result<E, result::Error> {
        let event = self.rx.recv().await.to_app_err()?;
        let event = E::try_from(event).expect("EventBus routing incorrect");
        Ok(event)
    }
}
