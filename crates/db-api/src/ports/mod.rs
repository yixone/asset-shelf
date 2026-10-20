pub mod assets;
pub mod files;

/// A one-shot operation that validates and modifies an entity
///
/// Returning an error aborts the update
type UpdateFn<T> = dyn FnOnce(&mut T) -> std::result::Result<(), models::EntityError>;

/// A one-shot operation that validates whether an entity can be deleted
///
/// Returning an error aborts the deletion
type DeleteFn<T> = dyn FnOnce(&T) -> std::result::Result<(), models::EntityError>;
