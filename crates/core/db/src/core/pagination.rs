use sqlx::QueryBuilder;

#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    limit: u32,
    offset: u32,
}

impl Pagination {
    /// Creates a new [`Pagination`]
    pub fn new(limit: u32, offset: u32) -> Self {
        Self { limit, offset }
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
