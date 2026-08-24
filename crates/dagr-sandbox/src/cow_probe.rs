//! Copy-on-Write capability probe.
//!
//! Doctor used to report "sandbox OK" from mere directory writability — which
//! says nothing about whether the <10ms clone-based rollback path actually
//! exists on the volume under test. This probe performs a REAL clone:
//! - macOS: `fclonefileat(2)` (APFS)
//! - Linux: `FICLONE` ioctl (btrfs/XFS/ZFS, ext4 ≥ 6.x on some setups)
//! and reports honestly when neither exists (copy-based fallback: rollback
//! remains atomic, but snapshot cost is proportional to tree size).
//!
//! ponytail: raw libc syscalls instead of a filesystem-crate dependency;
//! upgrade only if a third platform needs first-class support.

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CowSupport {
    /// OS-level instant clone available on this volume.
    Native,
    /// No clone syscall — sandbox falls back to deep copy; rollback stays
    /// atomic but snapshots cost O(tree size).
    CopyFallback,
}

pub fn probe(dir: &Path) -> CowSupport {
    let src = dir.join(format!(".dagr-cow-probe-src-{}", std::process::id()));
    let dst = dir.join(format!(".dagr-cow-probe-dst-{}", std::process::id()));
    let _ = std::fs::remove_file(&src);
    let _ = std::fs::remove_file(&dst);

    let cleanup = || {
        let _ = std::fs::remove_file(&src);
        let _ = std::fs::remove_file(&dst);
    };

    if std::fs::write(&src, b"dagr-cow-probe").is_err() {
        cleanup();
        return CowSupport::CopyFallback;
    }

    #[cfg(target_os = "macos")]
    {
        use std::os::unix::io::AsRawFd;
        // fclonefileat(src_fd, AT_FDCWD(-2), dst, flags=0): APFS instant clone.
        let ok = std::fs::File::open(&src).is_ok_and(|f| unsafe {
            let dst_c = std::ffi::CString::new(dst.as_os_str().as_encoded_bytes());
            match dst_c {
                Ok(c) => libc::fclonefileat(f.as_raw_fd(), -2, c.as_ptr(), 0) == 0,
                Err(_) => false,
            }
        });
        if ok && std::fs::read(&dst).is_ok_and(|b| b == b"dagr-cow-probe") {
            cleanup();
            return CowSupport::Native;
        }
        cleanup();
        return CowSupport::CopyFallback;
    }

    #[cfg(target_os = "linux")]
    {
        use std::os::unix::io::AsRawFd;
        const FICLONE: libc::c_ulong = 0x4004_9409;
        // FICLONE ioctl: dst becomes a reflink of src. Returns false on
        // filesystems without reflink support (ext4 pre-6.x, tmpfs).
        let ok = (|| -> std::io::Result<bool> {
            let src_f = std::fs::File::open(&src)?;
            let dst_f = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&dst)?;
            let rc = unsafe { libc::ioctl(dst_f.as_raw_fd(), FICLONE, src_f.as_raw_fd()) };
            Ok(rc == 0)
        })()
        .unwrap_or(false);
        if ok && std::fs::read(&dst).is_ok_and(|b| b == b"dagr-cow-probe") {
            cleanup();
            return CowSupport::Native;
        }
        cleanup();
        return CowSupport::CopyFallback;
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        cleanup();
        CowSupport::CopyFallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_returns_consistent_result_with_working_clone_semantics() {
        let dir = std::env::temp_dir();
        let support = probe(&dir);
        match support {
            CowSupport::Native => {
                // If we claim native cloning, a second probe must agree —
                // volume capability is not flaky between calls.
                assert_eq!(probe(&dir), CowSupport::Native);
            }
            CowSupport::CopyFallback => {
                // /tmp on this runner may legitimately lack reflink; both
                // results are honest. Nothing to assert beyond stability.
                assert_eq!(probe(&dir), CowSupport::CopyFallback);
            }
        }
    }
}
