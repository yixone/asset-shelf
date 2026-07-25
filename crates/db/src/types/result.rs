use sqlx::sqlite::SqliteQueryResult;

/// Result of insertion into the database
pub enum InsertResult {
    Inserted,
    NoChanges,
}

/// Result of updating records in the database
pub enum UpdateResult<T> {
    Updated(T),
    NotFound,
}

/// Result of deleting records from the database
pub enum DeleteResult {
    Deleted(u64),
    NoChanges,
}

impl InsertResult {
    /// Returns `true` if rows were inserted into the database;
    /// otherwise, returns `false`
    pub fn no_changes(&self) -> bool {
        matches!(self, Self::NoChanges)
    }
}

impl<T> UpdateResult<T> {
    /// Returns `true` if database records were updated;
    /// otherwise, returns `false`
    pub fn no_changes(&self) -> bool {
        matches!(self, Self::NotFound)
    }
}

impl DeleteResult {
    /// Returns `true` if records were deleted from the database;
    /// otherwise, returns `false`.
    pub fn no_changes(&self) -> bool {
        matches!(self, Self::NoChanges)
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
