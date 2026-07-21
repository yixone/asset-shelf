use tokio_util::sync::CancellationToken;

use crate::traits::AbstractWorker;

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

    pub fn run(self, cancel: CancellationToken) {
        for w in self.workers {
            let cancel = cancel.clone();
            tokio::spawn(async move {
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
        }
    }
}

impl Default for WorkersSupervisor {
    fn default() -> Self {
        Self::new()
    }
}
