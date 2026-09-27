use std::path::Path;

/// Creates all parent directories for the specified path.
pub async fn create_parents(path: impl AsRef<Path>) -> std::io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    Ok(())
}

/// Deletes all empty parent directories for the specified path.
/// Stops deletion if a directory contains files
pub async fn delete_parents_safely(root: &Path, path: &Path) {
    let mut p = path;
    while let Some(parent) = p.parent() {
        if parent == root {
            break;
        }
        p = parent;
        match tokio::fs::remove_dir(parent).await {
            Ok(_) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => break,
            Err(e) if e.kind() == std::io::ErrorKind::DirectoryNotEmpty => break,
            Err(e) => {
                tracing::error!(
                    err = ?e,
                    "Failed to delete directory parent"
                );
                return;
            }
        }
    }
    tracing::info!(
        from = ?path, to = ?p,
        "Parent directories have been removed"
    );
}
