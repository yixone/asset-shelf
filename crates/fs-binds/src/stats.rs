//! Cross-platform access to local file system statistics

use std::path::Path;

/// Statistics describing the capacity and free space of a file system
#[derive(Debug, Clone)]
pub struct FsStats {
    /// Number of bytes available for the calling process
    b_available: u64,
    /// Number of unused bytes in the file system
    b_free: u64,
    /// Total capacity of the file system in bytes
    b_total: u64,
}

impl FsStats {
    /// Returns the number of bytes available for the calling process
    pub fn bytes_available(&self) -> u64 {
        self.b_available
    }

    /// Returns the number of unused bytes in the file system
    pub fn bytes_free(&self) -> u64 {
        self.b_free
    }

    /// Returns the number of bytes currently occupied in the file system
    pub fn bytes_used(&self) -> u64 {
        self.b_total - self.b_free
    }

    /// Returns the total capacity of the file system in bytes
    pub fn bytes_total(&self) -> u64 {
        self.b_total
    }
}

/// Returns file system statistics for the file system containing specified [`Path`]
///
/// The returning statistics describe the total capacity, unused space,
/// and space available to the calling process
///
/// ### Usage
/// ``` no_run
/// use std::path::Path;
///
/// use fs_binds::statvfs;
///
/// let stats = statvfs(Path::new("./")).unwrap();
///
/// println!("total: {} bytes", stats.bytes_total());
/// println!("free: {} bytes", stats.bytes_free());
/// println!("available: {} bytes", stats.bytes_available());
/// ```
pub fn statvfs(path: impl AsRef<Path>) -> std::io::Result<FsStats> {
    _statvfs(path.as_ref())
}

#[cfg(unix)]
/// Calls `statvfs` and returns [`FsStats`] for the specified path
fn _statvfs(path: &Path) -> std::io::Result<FsStats> {
    use std::{ffi::CString, mem, os::unix::ffi::OsStrExt};

    use libc;

    // Converts the path to a `CString`
    let cstr = CString::new(path.as_os_str().as_bytes()).map_err(std::io::Error::other)?;

    unsafe {
        let mut stat: libc::statvfs = mem::zeroed();

        if libc::statvfs(cstr.as_ptr() as *const _, &mut stat) != 0 {
            Err(std::io::Error::last_os_error())
        } else {
            let f_frsize = stat.f_frsize;

            Ok(FsStats {
                b_available: stat.f_bavail as u64 * f_frsize,
                b_free: stat.f_bfree as u64 * f_frsize,
                b_total: stat.f_blocks as u64 * f_frsize,
            })
        }
    }
}

#[cfg(windows)]
/// Calls `GetDiskFreeSpaceExW` and returns [`FsStats`] for the specified path
fn _statvfs(path: &Path) -> Result<StorageStats, StorageError> {
    use std::{iter::Once, os::windows::ffi::OsStrExt};

    use winapi;

    // Converts the path to a null-terminated UTF-16 str
    let lpcwstr: Vec<u16> = path.as_os_str().encode_wide().chain(Once(0)).collect();

    unsafe {
        let mut bavail = 0;
        let mut bfree = 0;
        let mut btotal = 0;

        let ret = winapi::um::fileapi::GetDiskFreeSpaceExW(
            lpcwstr.as_ptr(),
            &mut bavail,
            &mut btotal,
            &mut bfree,
        );

        if ret == 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(FsStats {
                b_available: bavail as u64,
                b_free: bfree as u64,
                b_total: btotal as u64,
            })
        }
    }
}
