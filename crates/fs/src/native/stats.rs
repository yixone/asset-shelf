//! Tools for obtaining local file system statistics

use std::path::Path;

use crate::{backend::FsStats, result::StorageError};

#[cfg(unix)]
/// Calls `statvfs` and returns [`FsStats`] for the specified path
pub fn statvfs(path: &Path) -> Result<FsStats, StorageError> {
    use std::{ffi::CString, mem, os::unix::ffi::OsStrExt};

    use libc;

    // Converts the path to a `CString`
    let cstr = CString::new(path.as_os_str().as_bytes()).map_err(|_| StorageError::InvalidPath)?;

    unsafe {
        let mut stat: libc::statvfs = mem::zeroed();

        if libc::statvfs(cstr.as_ptr() as *const _, &mut stat) != 0 {
            Err(StorageError::Io(std::io::Error::last_os_error()))
        } else {
            let f_frsize = stat.f_frsize;

            Ok(FsStats {
                available: stat.f_bavail as u64 * f_frsize,
                free: stat.f_bfree as u64 * f_frsize,
                total: stat.f_blocks as u64 * f_frsize,
            })
        }
    }
}

#[cfg(windows)]
/// Calls `GetDiskFreeSpaceExW` and returns [`FsStats`] for the specified path
pub fn statvfs(path: &Path) -> Result<FsStats, StorageError> {
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
            Err(StorageError::Io(std::io::Error::last_os_error()))
        } else {
            Ok(FsStats {
                available: bavail as u64,
                free: bfree as u64,
                total: btotal as u64,
            })
        }
    }
}
