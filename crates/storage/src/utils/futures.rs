//! Helpers for working with asynchronous code and futures

use tokio::task::spawn_blocking;

/// Converts a synchronous blocking operation into an asynchronous one
pub async fn asyncify<F, T>(f: F) -> std::io::Result<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    match spawn_blocking(f).await {
        Ok(res) => Ok(res),
        Err(e) => Err(std::io::Error::other(e)),
    }
}
