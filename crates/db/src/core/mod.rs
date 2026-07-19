pub mod pagination;
pub mod patches;
pub mod result;

pub type Result<T> = sqlx::Result<T>;
