//! At most one Settings process per config file path.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

/// Try to acquire an exclusive lock for `config_path`.
///
/// Returns `true` if this process is the sole holder; `false` if another
/// instance already holds the lock for the same canonical path.
pub fn acquire(config_path: &Path) -> bool {
    let canonical =
        std::fs::canonicalize(config_path).unwrap_or_else(|_| config_path.to_path_buf());
    let hash = path_hash(&canonical);
    imp::acquire(&hash)
}

fn path_hash(path: &Path) -> String {
    let mut hasher = DefaultHasher::new();
    path.as_os_str().as_encoded_bytes().hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[cfg(windows)]
mod imp {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
    use windows_sys::Win32::System::Threading::CreateMutexW;

    pub fn acquire(hash: &str) -> bool {
        let name = format!("Local\\Settings.{hash}");
        let wide: Vec<u16> = OsStr::new(&name)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        unsafe {
            let handle = CreateMutexW(std::ptr::null(), 1, wide.as_ptr());
            if handle.is_null() {
                return true;
            }
            if GetLastError() == ERROR_ALREADY_EXISTS {
                return false;
            }
            // Hold the mutex until process exit (never CloseHandle).
            let _ = handle;
            true
        }
    }
}

#[cfg(unix)]
mod imp {
    use std::fs::OpenOptions;
    use std::io;
    use std::os::unix::io::AsRawFd;

    pub fn acquire(hash: &str) -> bool {
        let lock_path = std::env::temp_dir().join(format!("settings-{hash}.lock"));
        let file = match OpenOptions::new()
            .create(true)
            .write(true)
            .open(&lock_path)
        {
            Ok(f) => f,
            Err(_) => return true,
        };
        let fd = file.as_raw_fd();
        let ret = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };
        if ret != 0 {
            let err = io::Error::last_os_error();
            if err.kind() == io::ErrorKind::WouldBlock {
                return false;
            }
            return true;
        }
        std::mem::forget(file);
        true
    }
}

#[cfg(not(any(windows, unix)))]
mod imp {
    pub fn acquire(_hash: &str) -> bool {
        true
    }
}
