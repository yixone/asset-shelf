use std::path::Path;

#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::{
    ffi::{CString, c_uint},
    os::unix::ffi::OsStrExt,
};

/// Renames a file or directory without replacing an existing destination
///
/// The opeartion fails if `to` already exists and does not replace it
///
/// The operation is performed using the platform's exclusive rename primitive
///
/// ### Usage
/// ``` no_run
/// use std::path::Path;
///
/// use storage::fs::rename_exclusive;
///
/// rename_exclusive(
///     Path::new("./staged"),
///     Path::new("./file"),
/// ).unwrap();
/// ```
pub fn rename_exclusive(from: impl AsRef<Path>, to: impl AsRef<Path>) -> std::io::Result<()> {
    _rename_exclusive(from.as_ref(), to.as_ref())
}

#[cfg(target_os = "linux")]
/// Calls `renameat(2)` with `RENAME_NOREPLACE`
fn _rename_exclusive(from: &Path, to: &Path) -> std::io::Result<()> {
    use std::ffi::c_int;

    const AT_FDCWD: c_int = -100;
    const RENAME_NOREPLACE: c_uint = 1;

    let from_str = CString::new(from.as_os_str().as_bytes())?;
    let to_str = CString::new(to.as_os_str().as_bytes())?;

    let ret = unsafe {
        let old = from_str.as_ptr();
        let new = to_str.as_ptr();
        libc::renameat2(AT_FDCWD, old, AT_FDCWD, new, RENAME_NOREPLACE)
    };

    if ret == -1 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(target_os = "macos")]
/// Calls `renamex_np` with `RENAME_EXCL = 4`
fn _rename_exclusive(from: &Path, to: &Path) -> std::io::Result<()> {
    const RENAME_EXCL: c_uint = 4;

    let from_str = CString::new(from.as_os_str().as_bytes())?;
    let to_str = CString::new(to.as_os_str().as_bytes())?;

    let ret = unsafe { libc::renamex_np(from_str.as_ptr(), to_str.as_ptr(), RENAME_EXCL) };

    if ret == -1 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(windows)]
/// Calls `MoveFileExW` with `dwFlags = 0`
fn _rename_exclusive(from: &Path, to: &Path) -> std::io::Result<()> {
    use std::{iter::Once, os::windows::ffi::OsStrExt};

    // Converts the paths to a null-terminated UTF-16 str
    let from_str: Vec<u16> = from.as_os_str().encode_wide().chain(Once(0)).collect();
    let to_str: Vec<u16> = to.as_os_str().encode_wide().chain(Once(0)).collect();

    let ret = unsafe { winapi::um::winbase::MoveFileExW(from_str.as_ptr(), to_str.as_ptr(), 0) };

    if ret == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}
