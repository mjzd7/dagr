//! Copy-on-Write capability probe.
//!
//! Doctor used to report "sandbox OK" from mere directory writability — which
//! says nothing about whether the <10ms clone-based rollback path actually
//! exists on the volume under test. This probe performs a REAL clone:
//!
//! - macOS: `fclonefileat(2)` (APFS)
//! - Linux: `FICLONE` ioctl (btrfs/XFS/ZFS; ext4 support varies)
//!
//! It reports honestly when neither exists: the sandbox falls back to deep
//! copy — rollback stays atomic, snapshots cost O(tree size).
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
    let cleanup = || {
        let _ = std::fs::remove_file(&src);
        let _ = std::fs::remove_file(&dst);
    };

    if std::fs::write(&src, b"dagr-cow-probe").is_err() {
        cleanup();
        return CowSupport::CopyFallback;
    }

    #[cfg(target_os = "macos")]
    let cloned: bool = {
        use std::os::unix::io::AsRawFd;
        // fclonefileat(src_fd, AT_FDCWD(-2), dst, flags=0): APFS instant clone.
        std::fs::File::open(&src).is_ok_and(|f| unsafe {
            match std::ffi::CString::new(dst.as_os_str().as_encoded_bytes()) {
                Ok(c) => libc::fclonefileat(f.as_raw_fd(), -2, c.as_ptr(), 0) == 0,
                Err(_) => false,
            }
        })
    };

    #[cfg(target_os = "linux")]
    let cloned: bool = {
        use std::os::unix::io::AsRawFd;
        const FICLONE: libc::c_ulong = 0x4004_9409;
        // FICLONE reflinks src into dst; fails on filesystems without
        // reflink support (tmpfs, ext4 pre-6.x).
        (|| -> std::io::Result<bool> {
            let src_f = std::fs::File::open(&src)?;
            let dst_f = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&dst)?;
            Ok(unsafe { libc::ioctl(dst_f.as_raw_fd(), FICLONE, src_f.as_raw_fd()) } == 0)
        })()
        .unwrap_or(false)
    };

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    let cloned: bool = false;

    let verified = cloned && std::fs::read(&dst).is_ok_and(|b| b == b"dagr-cow-probe");
    cleanup();
    if verified {
        CowSupport::Native
    } else {
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
