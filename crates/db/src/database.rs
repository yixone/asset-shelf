use result::Result;

/// Common trait for the database
pub trait Database {
    type Transaction: DatabaseTransaction;
    type Session: DatabaseSession;
}

/// Database provider trait
///
/// Used for managing sessions and transactions
pub trait DatabaseProvider: Database {
    /// Opens a new database transaction
    async fn begin(&self) -> Result<Self::Transaction>;

    /// Creates a new database session
    async fn acquire(&self) -> Result<Self::Session>;

    /// Opens a new session and passes it to the closure
    async fn with_session<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: Fn(&mut Self::Session) -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let mut session = self.acquire().await?;
        f(&mut session).await
    }
}

/// Active database session
pub trait DatabaseSession {}

/// Active database transaction
pub trait DatabaseTransaction: DatabaseSession {
    /// Commit changes to the database and closes the transaction
    async fn commit(self) -> Result<()>;
    /// Rollsback changes and closes the transaction.
    async fn rollback(self) -> Result<()>;
}
