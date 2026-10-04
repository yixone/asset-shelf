use std::path::Path;

use storage_types::DiskUsageStats;

/// Returns file system statistics for the file system containing specified [`Path`]
///
/// The returning statistics describe the total capacity and unused space
///
/// ### Usage
/// ``` no_run
/// use std::path::Path;
///
/// use storage::fs::statvfs;
///
/// let stats = statvfs(Path::new("./")).unwrap();
///
/// println!("total: {} bytes", stats.bytes_total());
/// println!("free: {} bytes", stats.bytes_free());
/// ```
pub fn statvfs(path: impl AsRef<Path>) -> std::io::Result<DiskUsageStats> {
    _statvfs(path.as_ref())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
/// Calls `statvfs` and returns [`DiskUsageStats`] for the specified path
fn _statvfs(path: &Path) -> std::io::Result<DiskUsageStats> {
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

            Ok(DiskUsageStats::new(
                stat.f_bfree as u64 * f_frsize,
                stat.f_blocks as u64 * f_frsize,
            ))
        }
    }
}

#[cfg(windows)]
/// Calls `GetDiskFreeSpaceExW` and returns [`DiskUsageStats`] for the specified path
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
            Ok(DiskUsageStats::new(bfree as u64, btotal as u64))
        }
    }
}
