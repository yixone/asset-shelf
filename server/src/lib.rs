use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

use config::ApplicationConfig;
use instance::library::{self, LibManifest};
use result::{Result, create_error, error::ResultExt};

pub mod dto;
pub mod middlewares;
pub mod routes;
pub mod utils;

pub mod di;
pub mod metrics;

pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

const SELECTED_LIB_PATH: &str = "current_lib";

/// Loads the application configuration from the file
pub fn load_config(path: impl AsRef<Path>) -> Result<Arc<ApplicationConfig>> {
    let path = path.as_ref();
    tracing::info!("Reading config from `{path:?}`");
    let cfg = ApplicationConfig::try_load(path, true).to_app_err()?;
    Ok(Arc::new(cfg))
}

pub fn save_selected_lib(lib: &Path) -> Result<()> {
    let mut file = File::create(SELECTED_LIB_PATH).to_app_err()?;
    file.write_all(format!("{}", lib.display()).as_bytes())
        .to_app_err()
}

pub fn load_selected_lib() -> Result<(PathBuf, LibManifest)> {
    let file_path = Path::new(SELECTED_LIB_PATH);

    if !file_path.is_file() {
        return Err(create_error!(NotFound));
    }

    let path_bytes = fs::read(file_path).to_app_err()?;
    let path = PathBuf::from(String::from_utf8(path_bytes).to_app_err()?);

    let manifest = LibManifest::load_dir(&path).to_app_err()?;
    Ok((path, manifest))
}

pub fn load_library() -> Result<(PathBuf, LibManifest)> {
    match load_selected_lib() {
        Ok(l) => {
            return Ok(l);
        }
        Err(e) if e.is_not_found() => (),
        Err(e) => {
            return Err(e);
        }
    };

    let libs = library::load_dir_libs("./").to_app_err()?;

    let Some(first) = libs.into_iter().next() else {
        // TODO: Create new lib
        return Err(create_error!(NotFound));
    };
    save_selected_lib(&first.0)?;

    Ok(first)
}
