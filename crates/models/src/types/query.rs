pub enum SortOrder {
    Asc,
    Desc,
}

pub const DEFAULT_LIMIT: u32 = 100;
pub const DEFAULT_OFFSET: u32 = 0;

pub struct Pagination {
    limit: u32,
    offset: u32,
}

impl Pagination {
    /// Creates a new [`Pagination`]
    pub fn new(limit: u32, offset: u32) -> Self {
        Self { limit, offset }
    }

    /// Creates a new [`Pagination`] from [`Option`] values
    pub fn new_option(limit: Option<u32>, offset: Option<u32>) -> Self {
        Self {
            limit: limit.unwrap_or(DEFAULT_LIMIT),
            offset: offset.unwrap_or(DEFAULT_OFFSET),
        }
    }

    /// Returns the limit of this [`Pagination`]
    pub fn limit(&self) -> u32 {
        self.limit
    }

    /// Returns the offset of this [`Pagination`]
    pub fn offset(&self) -> u32 {
        self.offset
    }
}
