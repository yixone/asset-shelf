use result::{Result, create_error};
use sqlx::QueryBuilder;

#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    limit: u32,
    offset: u32,
}

impl Pagination {
    /// Creates a new [`Pagination`]
    pub const fn new(limit: u32, offset: u32) -> Self {
        Self { limit, offset }
    }

    pub fn try_new(limit: u32, offset: u32) -> Result<Self> {
        if limit > 1000 {
            return Err(create_error!(PaginationTooLarge));
        };

        Ok(Pagination::new(limit, offset))
    }

    /// Returns the [`Pagination`] limit
    pub fn limit(&self) -> u32 {
        self.limit
    }

    /// Returns the [`Pagination`] offset
    pub fn offset(&self) -> u32 {
        self.offset
    }

    /// Applies [`Pagination`] to the provided [`QueryBuilder`]
    pub fn apply_sql<'a, DB>(&self, qb: &mut QueryBuilder<'a, DB>)
    where
        DB: sqlx::Database,
        u32: sqlx::Encode<'a, DB> + sqlx::Type<DB>,
    {
        qb.push(" LIMIT ");
        qb.push_bind(self.limit);
        qb.push(" OFFSET ");
        qb.push_bind(self.offset);
    }
}
