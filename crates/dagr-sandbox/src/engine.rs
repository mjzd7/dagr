use dagr_core::{DagrError, Result};
use std::path::Path;

pub struct CloneEngine;

impl CloneEngine {
    /// Copies or clones a file using the fastest OS-native block primitive
    pub fn clone_file(src: &Path, dst: &Path) -> Result<()> {
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)?;
        }

        #[cfg(target_os = "macos")]
        {
            use std::ffi::CString;
            use std::os::unix::ffi::OsStrExt;

            let src_c = CString::new(src.as_os_str().as_bytes())
                .map_err(|e| DagrError::Sandbox(e.to_string()))?;
            let dst_c = CString::new(dst.as_os_str().as_bytes())
                .map_err(|e| DagrError::Sandbox(e.to_string()))?;

            // clonefile(src, dst, flags) - flags = 0 for default CoW block clone
            extern "C" {
                fn clonefile(src: *const libc::c_char, dst: *const libc::c_char, flags: u32) -> libc::c_int;
            }

            let ret = unsafe { clonefile(src_c.as_ptr(), dst_c.as_ptr(), 0) };
            if ret == 0 {
                return Ok(());
            }
            // If clonefile fails (e.g. cross-volume), fall back to standard copy
        }

        std::fs::copy(src, dst).map_err(|e| DagrError::Sandbox(format!("File copy failed: {}", e)))?;
        Ok(())
    }
}
