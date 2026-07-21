use result::Result;

pub trait DatabaseProvider {
    type Transaction<'a>: TransactionUnit;
    type Connection: ConnectionUnit;
}

pub trait DatabaseConnector: DatabaseProvider {
    async fn begin(&self) -> Result<Self::Transaction<'_>>;
    async fn acquire(&self) -> Result<Self::Connection>;
}

pub trait TransactionUnit {
    async fn commit(self) -> Result<()>;
    async fn rollback(self) -> Result<()>;
}

pub trait ConnectionUnit {
    type Error;
}
