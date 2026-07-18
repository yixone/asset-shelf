pub trait DatabaseProvider {
    type Transaction<'a>: TransactionUnit;
    type Connection: ConnectionUnit;
}

pub trait DatabaseProviderExt: DatabaseProvider {
    type Error;

    async fn begin(&self) -> Result<Self::Transaction<'_>, Self::Error>;
    async fn acquire(&self) -> Result<Self::Connection, Self::Error>;
}

pub trait TransactionUnit {
    type Error;

    async fn commit(self) -> Result<(), Self::Error>;
    async fn rollback(self) -> Result<(), Self::Error>;
}

pub trait ConnectionUnit {
    type Error;
}
