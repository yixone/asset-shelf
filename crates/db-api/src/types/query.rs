use crate::{DatabaseError, Result};

/// Offset-based pagination for retrieving a page of entities
pub struct Pagination {
    page: u64,
    limit: u64,
}

impl Pagination {
    /// Creates a new [`Pagination`]
    pub fn new(page: u64, limit: u64) -> Self {
        Self { page, limit }
    }

    /// Returns the limit of this [`Pagination`]
    pub fn limit(&self) -> u64 {
        self.limit
    }

    /// Returns the page of this [`Pagination`]
    pub fn page(&self) -> u64 {
        self.page
    }

    /// Returns the offset of this [`Pagination`]
    ///
    /// Returns [`DatabaseError::PaginationOverflow`] if the calculation overflows
    #[inline]
    pub fn try_offset(&self) -> Result<u64> {
        self.page()
            .checked_mul(self.limit())
            .ok_or(DatabaseError::PaginationOverflow)
    }
}

/// Keyset pagination using an ordered cursor
///
/// The cursor defines the boundary for the next page. The comparison
/// direction depends on the query's sort order
pub struct CursorPagination<I: Ord> {
    cursor: I,
    limit: u64,
}

impl<I: Ord> CursorPagination<I> {
    /// Creates a new [`CursorPagination`]
    pub fn new(cursor: I, limit: u64) -> Self {
        Self { cursor, limit }
    }

    /// Returns the limit of this [`CursorPagination`]
    pub fn limit(&self) -> u64 {
        self.limit
    }

    /// Returns a reference to the cursor of this [`CursorPagination`]
    pub fn cursor(&self) -> &I {
        &self.cursor
    }
}
