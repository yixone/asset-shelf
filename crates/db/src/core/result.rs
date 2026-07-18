/// Result of insertion into the database
pub enum InsertResult {
    Inserted,
    NoChanges,
}

/// Result of updating records in the database
pub enum UpdateResult {
    Updated(usize),
    NoChanges,
}

/// Result of deleting records from the database
pub enum DeleteResult {
    Deleted(usize),
    NoChanges,
}

impl InsertResult {
    /// Returns `true` if rows were inserted into the database;
    /// otherwise, returns `false`
    pub fn no_changes(&self) -> bool {
        matches!(self, Self::NoChanges)
    }
}

impl UpdateResult {
    /// Returns `true` if database records were updated;
    /// otherwise, returns `false`
    pub fn no_changes(&self) -> bool {
        matches!(self, Self::NoChanges)
    }
}

impl DeleteResult {
    /// Returns `true` if records were deleted from the database;
    /// otherwise, returns `false`.
    pub fn no_changes(&self) -> bool {
        matches!(self, Self::NoChanges)
    }
}
