use tokio::sync::mpsc::{Receiver, Sender, channel};

/// Queue for service events
pub struct EventsQueue<T> {
    /// Tasks sender
    pub tx: TasksSender<T>,
    /// Receiver for tasks
    pub rx: Receiver<T>,
}

impl<T> EventsQueue<T> {
    /// Creates a new [`WorkerEventsQueue`]
    pub fn new(queue_size: usize) -> Self {
        let (tx, rx) = channel(queue_size);
        EventsQueue {
            tx: TasksSender(tx),
            rx,
        }
    }

    pub fn recv(&mut self) -> impl Future<Output = std::option::Option<T>> {
        self.rx.recv()
    }

    pub fn close(&mut self) {
        self.rx.close();
    }
}

/// A wrapper around [`Sender`] for sending tasks to a background service
#[derive(Clone)]
pub struct TasksSender<T>(pub(crate) Sender<T>);

impl<T> TasksSender<T> {
    /// Sends the task to a background service
    pub async fn send(&self, task: T) -> bool {
        if let Err(e) = self.0.send(task).await {
            tracing::warn!(error = ?e, "Failed to send bg service task");
            return false;
        }
        true
    }
}
