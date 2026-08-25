use sqlx::sqlite::SqliteQueryResult;

/// Result of insertion into the database
#[derive(Debug)]
pub enum InsertResult {
    Inserted,
    NoChanges,
}

/// Result of updating records in the database
#[derive(Debug)]
pub enum UpdateResult<T> {
    Updated(T),
    NotFound,
}

/// Result of deleting records from the database
#[derive(Debug)]
pub enum DeleteResult {
    Deleted(u64),
    NoChanges,
}

impl InsertResult {
    pub fn no_changes(&self) -> bool {
        matches!(self, Self::NoChanges)
    }

    pub fn has_changes(&self) -> bool {
        matches!(self, Self::Inserted)
    }
}

impl<T> UpdateResult<T> {
    pub fn no_changes(&self) -> bool {
        matches!(self, Self::NotFound)
    }

    pub fn has_changes(&self) -> bool {
        matches!(self, Self::Updated(_))
    }
}

impl DeleteResult {
    pub fn no_changes(&self) -> bool {
        matches!(self, Self::NoChanges)
    }

    pub fn has_changes(&self) -> bool {
        matches!(self, Self::Deleted(_))
    }
}

impl<T> From<Option<T>> for UpdateResult<T> {
    fn from(t: Option<T>) -> Self {
        match t {
            Some(t) => UpdateResult::Updated(t),
            None => UpdateResult::NotFound,
        }
    }
}

impl From<SqliteQueryResult> for InsertResult {
    fn from(res: SqliteQueryResult) -> Self {
        if res.rows_affected() == 0 {
            InsertResult::NoChanges
        } else {
            InsertResult::Inserted
        }
    }
}

impl From<SqliteQueryResult> for DeleteResult {
    fn from(res: SqliteQueryResult) -> Self {
        if res.rows_affected() == 0 {
            DeleteResult::NoChanges
        } else {
            DeleteResult::Deleted(res.rows_affected())
        }
    }
}
