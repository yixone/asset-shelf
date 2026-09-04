use std::{marker::PhantomData, sync::Arc};

use result::error::ResultExt;
use tokio::sync::broadcast::{Receiver, Sender};

use crate::DynamicEvent;

/// Event Sender
///
/// Sends an event to all subscribers for the specified Kind
pub(crate) struct EventSender {
    pub tx: Sender<Arc<dyn DynamicEvent>>,
}

impl EventSender {
    /// Attempts to send an event to everyone subscribed to it
    pub fn send<E>(&self, event: E) -> bool
    where
        E: DynamicEvent + 'static,
    {
        if let Err(e) = self.tx.send(Arc::new(event)) {
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
    E: DynamicEvent,
{
    pub(crate) marker: PhantomData<E>,
    pub(crate) rx: Receiver<Arc<dyn DynamicEvent>>,
}

impl<E> EventStream<E>
where
    E: DynamicEvent + Clone + 'static,
{
    /// Receives the next `event` for the [`EventStream`]
    pub async fn recv(&mut self) -> Result<Arc<E>, result::Error> {
        let event = self.rx.recv().await.to_app_err()?;

        let Ok(e) = Arc::downcast(event) else {
            unreachable!()
        };

        Ok(e)
    }
}
