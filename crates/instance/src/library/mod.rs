pub mod manifest;
use std::path::{Path, PathBuf};

pub use manifest::{LibDatabase, LibManifest, LibStorage};

/// Iterates through each directory in the specified path and searches for the library manifest in it
pub fn load_dir_libs(
    dir: impl AsRef<Path>,
) -> Result<Vec<(PathBuf, LibManifest)>, manifest::ManifestError> {
    let mut entries = Vec::new();

    let read = std::fs::read_dir(dir.as_ref()).map_err(manifest::ManifestError::Io)?;

    for r in read {
        let Ok(entry) = r else {
            continue;
        };

        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        match LibManifest::load_dir(&path) {
            Ok(e) => {
                entries.push((path, e));
            }
            Err(manifest::ManifestError::FileNotFound) => {
                continue;
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    Ok(entries)
}
