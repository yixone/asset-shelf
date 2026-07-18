pub trait Provider {
    type Transaction<'a>: TransactionUnit;
    type Connection: ConnectionUnit;
}

pub trait DbProvider: Provider {
    type Error;

    async fn begin(&self) -> Result<Self::Transaction<'_>, Self::Error>;
    async fn acquire(&self) -> Result<Self::Connection, Self::Error>;
}

pub trait TransactionUnit: ExecutorUnit {
    type Error;

    async fn commit(self) -> Result<(), Self::Error>;
    async fn rollback(self) -> Result<(), Self::Error>;
}

pub trait ConnectionUnit: ExecutorUnit {
    type Error;
}

pub trait ExecutorUnit {
    type Executor;
    fn exec(&mut self) -> &mut Self::Executor;
}
