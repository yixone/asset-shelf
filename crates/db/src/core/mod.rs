pub mod pagination;
pub mod patches;
pub mod provider;
pub mod result;

pub type Result<T> = sqlx::Result<T>;
