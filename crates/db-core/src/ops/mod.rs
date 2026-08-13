use result::Result;

pub mod create_asset;

/// A repository derivative used to perform operations
/// requiring the atomic execution of multiple requests within the single pipeline
#[async_trait::async_trait]
pub trait Operation: Send + Sync {
    /// Applies changes and closes the [`Operation`]
    async fn commit(self: Box<Self>) -> Result<()>;

    /// Rolls back changes and closes the [`Operation`]
    async fn rollback(self: Box<Self>) -> Result<()>;
}
