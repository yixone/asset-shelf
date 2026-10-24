pub mod assets;
pub mod files;

pub use assets::{AssetsDatabase, AssetsDatabaseExt};
pub use files::{FilesDatabase, FilesDatabaseExt};

/// A one-shot operation that validates and modifies an entity
///
/// Returning an error aborts the update
pub type UpdateFn<T> = dyn (FnOnce(&mut T) -> std::result::Result<(), models::EntityError>) + Send;

/// A one-shot operation that validates whether an entity can be deleted
///
/// Returning an error aborts the deletion
pub type DeleteFn<T> = dyn (FnOnce(&T) -> std::result::Result<(), models::EntityError>) + Send;
