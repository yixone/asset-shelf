use std::path::PathBuf;

pub struct StorageMountPoint {
    path: PathBuf,
}

impl StorageMountPoint {
    pub fn resolve_path(&self, path: impl AsRef<str>) -> PathBuf {
        self.path.join(path.as_ref())
    }
}
