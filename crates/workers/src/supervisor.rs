use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::worker::AbstractWorker;

pub struct WorkersSupervisor {
    workers: Vec<Box<dyn AbstractWorker + Send>>,
}

impl WorkersSupervisor {
    /// Creates a new empty [`WorkersSupervisor`]
    pub fn new() -> Self {
        WorkersSupervisor {
            workers: Vec::new(),
        }
    }

    pub fn with_worker<W>(mut self, worker: W) -> Self
    where
        W: AbstractWorker + Send + 'static,
    {
        self.workers.push(Box::new(worker));
        self
    }

    pub fn run(self, cancel: CancellationToken) -> SupervisorHandle {
        let mut handles = Vec::with_capacity(self.workers.len());
        for w in self.workers {
            let cancel = cancel.clone();
            let handle = tokio::spawn(async move {
                let mut worker = w;

                let cfg = worker.cfg();
                let name = cfg.name;

                tracing::info!("{name}: Runtime created!");

                while let Err(e) = worker.runtime(cancel.clone()).await {
                    tracing::error!(err = ?e, "{name}: Runtime error occurred");
                    if cancel.is_cancelled() {
                        break;
                    }

                    if cfg.allow_restart {
                        tokio::time::sleep(cfg.restart_delay).await;
                        continue;
                    } else {
                        break;
                    }
                }

                tracing::info!("{name}: Runtime terminated!");
            });
            handles.push(handle);
        }
        SupervisorHandle { handles }
    }
}

impl Default for WorkersSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SupervisorHandle {
    handles: Vec<JoinHandle<()>>,
}

impl SupervisorHandle {
    pub async fn stop(self) {
        for h in self.handles {
            if let Err(e) = h.await {
                // TODO: ADD MESSAGE!
                tracing::error!(err = ?e);
            }
        }
    }
}
