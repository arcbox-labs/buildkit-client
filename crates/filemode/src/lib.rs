//! File mode conversion utilities between Unix mode_t and Go os.FileMode formats.
//!
//! This crate provides utilities for converting between Unix file mode representations
//! (as used in POSIX systems) and Go's os.FileMode format (as used in BuildKit and fsutil).
//!
//! # Examples
//!
//! ## Using type-safe conversion with `From` trait
//!
//! ```
//! use filemode::{UnixMode, GoFileMode};
//!
//! // Convert Unix directory mode (0o040755) to Go FileMode
//! let unix_mode = UnixMode::from(0o040755);
//! let go_mode: GoFileMode = unix_mode.into();
//! assert_eq!(go_mode.as_u32(), 0x800001ed); // ModeDir | 0o755
//!
//! // Or use the convenience function
//! let go_mode = GoFileMode::from(UnixMode::from(0o100644));
//! assert_eq!(go_mode.as_u32(), 0o644); // Regular file with 0o644 permissions
//! ```
//!
//! ## Using the legacy function API
//!
//! ```
//! use filemode::unix_mode_to_go_filemode;
//!
//! let go_mode = unix_mode_to_go_filemode(0o040755);
//! assert_eq!(go_mode, 0x800001ed);
//! ```

// Unix file type constants
const S_IFMT: u32 = 0o170000; // File type mask
const S_IFDIR: u32 = 0o040000; // Directory
const S_IFREG: u32 = 0o100000; // Regular file
const S_IFLNK: u32 = 0o120000; // Symbolic link
const S_IFIFO: u32 = 0o010000; // Named pipe (FIFO)
const S_IFSOCK: u32 = 0o140000; // Socket
const S_IFCHR: u32 = 0o020000; // Character device
const S_IFBLK: u32 = 0o060000; // Block device

// Go FileMode type bits (from Go's os package)
const GO_MODE_DIR: u32 = 0x80000000; // 1 << 31 - Directory
const GO_MODE_SYMLINK: u32 = 0x08000000; // 1 << 27 - Symbolic link
const GO_MODE_DEVICE: u32 = 0x04000000; // 1 << 26 - Device file
const GO_MODE_NAMED_PIPE: u32 = 0x02000000; // 1 << 25 - Named pipe (FIFO)
const GO_MODE_SOCKET: u32 = 0x01000000; // 1 << 24 - Socket
const GO_MODE_SETUID: u32 = 0x00800000; // 1 << 23 - Setuid
const GO_MODE_SETGID: u32 = 0x00400000; // 1 << 22 - Setgid
const GO_MODE_CHAR_DEVICE: u32 = 0x00200000; // 1 << 21 - Character device
const GO_MODE_STICKY: u32 = 0x00100000; // 1 << 20 - Sticky bit

/// A Unix file mode (mode_t) value.
///
/// This type represents file permissions and type as used in POSIX systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnixMode(u32);

impl UnixMode {
    /// Create a new UnixMode from a raw u32 value.
    #[inline]
    pub const fn new(mode: u32) -> Self {
        Self(mode)
    }

    /// Get the raw u32 value.
    #[inline]
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

impl From<u32> for UnixMode {
    #[inline]
    fn from(mode: u32) -> Self {
        Self(mode)
    }
}

impl From<UnixMode> for u32 {
    #[inline]
    fn from(mode: UnixMode) -> Self {
        mode.0
    }
}

/// A Go os.FileMode value.
///
/// This type represents file permissions and type as used in Go's os package.
/// BuildKit and fsutil use this format for file synchronization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GoFileMode(u32);

impl GoFileMode {
    /// Create a new GoFileMode from a raw u32 value.
    #[inline]
    pub const fn new(mode: u32) -> Self {
        Self(mode)
    }

    /// Get the raw u32 value.
    #[inline]
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

impl From<u32> for GoFileMode {
    #[inline]
    fn from(mode: u32) -> Self {
        Self(mode)
    }
}

impl From<GoFileMode> for u32 {
    #[inline]
    fn from(mode: GoFileMode) -> Self {
        mode.0
    }
}

/// Convert Unix mode_t to Go os.FileMode.
///
/// This is the core conversion implementation.
impl From<UnixMode> for GoFileMode {
    fn from(unix_mode: UnixMode) -> Self {
        let mode = unix_mode.0;
        let file_type = mode & S_IFMT;
        let permissions = mode & 0o7777; // rwxrwxrwx + sticky/setuid/setgid

        let mut go_mode = permissions & 0o777; // Base permissions (rwxrwxrwx)

        // Convert special permission bits
        if permissions & 0o4000 != 0 {
            go_mode |= GO_MODE_SETUID;
        }
        if permissions & 0o2000 != 0 {
            go_mode |= GO_MODE_SETGID;
        }
        if permissions & 0o1000 != 0 {
            go_mode |= GO_MODE_STICKY;
        }

        // Convert file type bits
        match file_type {
            S_IFDIR => go_mode |= GO_MODE_DIR,
            S_IFLNK => go_mode |= GO_MODE_SYMLINK,
            S_IFIFO => go_mode |= GO_MODE_NAMED_PIPE,
            S_IFSOCK => go_mode |= GO_MODE_SOCKET,
            S_IFCHR => go_mode |= GO_MODE_CHAR_DEVICE | GO_MODE_DEVICE,
            S_IFBLK => go_mode |= GO_MODE_DEVICE,
            S_IFREG => {} // Regular files have no special bit in Go
            _ => {}
        }

        GoFileMode(go_mode)
    }
}

/// Convert Unix mode_t format to Go os.FileMode format.
///
/// This is a convenience function that wraps the type-safe conversion.
/// For new code, consider using the [`From`] trait implementation instead.
///
/// # Arguments
///
/// * `unix_mode` - A Unix mode_t value combining file type and permissions
///
/// # Returns
///
/// A u32 value in Go os.FileMode format
///
/// # Examples
///
/// ```
/// use filemode::unix_mode_to_go_filemode;
///
/// // Directory with 0o755 permissions
/// let dir_mode = unix_mode_to_go_filemode(0o040755);
/// assert_eq!(dir_mode, 0x800001ed);
///
/// // Regular file with 0o644 permissions
/// let file_mode = unix_mode_to_go_filemode(0o100644);
/// assert_eq!(file_mode, 0o644);
///
/// // Symbolic link with 0o777 permissions
/// let link_mode = unix_mode_to_go_filemode(0o120777);
/// assert_eq!(link_mode, 0x080001ff);
/// ```
#[inline]
pub fn unix_mode_to_go_filemode(unix_mode: u32) -> u32 {
    GoFileMode::from(UnixMode::from(unix_mode)).as_u32()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regular_file() {
        // Regular file with 0o644 permissions
        let unix_mode = UnixMode::from(0o100644);
        let go_mode = GoFileMode::from(unix_mode);
        assert_eq!(go_mode.as_u32(), 0o644);

        // Also test the convenience function
        assert_eq!(unix_mode_to_go_filemode(0o100644), 0o644);
    }

    #[test]
    fn test_directory() {
        // Directory with 0o755 permissions
        let unix_mode = UnixMode::from(0o040755);
        let go_mode = GoFileMode::from(unix_mode);
        assert_eq!(go_mode.as_u32(), GO_MODE_DIR | 0o755);
        assert_eq!(go_mode.as_u32(), 0x800001ed);
    }

    #[test]
    fn test_symlink() {
        // Symbolic link with 0o777 permissions
        let unix_mode = UnixMode::from(0o120777);
        let go_mode = GoFileMode::from(unix_mode);
        assert_eq!(go_mode.as_u32(), GO_MODE_SYMLINK | 0o777);
    }

    #[test]
    fn test_named_pipe() {
        // Named pipe with 0o644 permissions
        let unix_mode = UnixMode::from(0o010644);
        let go_mode = GoFileMode::from(unix_mode);
        assert_eq!(go_mode.as_u32(), GO_MODE_NAMED_PIPE | 0o644);
    }

    #[test]
    fn test_socket() {
        // Socket with 0o666 permissions
        let unix_mode = UnixMode::from(0o140666);
        let go_mode = GoFileMode::from(unix_mode);
        assert_eq!(go_mode.as_u32(), GO_MODE_SOCKET | 0o666);
    }

    #[test]
    fn test_char_device() {
        // Character device with 0o666 permissions
        let unix_mode = UnixMode::from(0o020666);
        let go_mode = GoFileMode::from(unix_mode);
        assert_eq!(
            go_mode.as_u32(),
            GO_MODE_CHAR_DEVICE | GO_MODE_DEVICE | 0o666
        );
    }

    #[test]
    fn test_block_device() {
        // Block device with 0o666 permissions
        let unix_mode = UnixMode::from(0o060666);
        let go_mode = GoFileMode::from(unix_mode);
        assert_eq!(go_mode.as_u32(), GO_MODE_DEVICE | 0o666);
    }

    #[test]
    fn test_setuid_bit() {
        // Regular file with setuid and 0o755 permissions
        let unix_mode = UnixMode::from(0o104755);
        let go_mode = GoFileMode::from(unix_mode);
        assert_eq!(go_mode.as_u32(), GO_MODE_SETUID | 0o755);
    }

    #[test]
    fn test_setgid_bit() {
        // Regular file with setgid and 0o755 permissions
        let unix_mode = UnixMode::from(0o102755);
        let go_mode = GoFileMode::from(unix_mode);
        assert_eq!(go_mode.as_u32(), GO_MODE_SETGID | 0o755);
    }

    #[test]
    fn test_sticky_bit() {
        // Directory with sticky bit and 0o755 permissions
        let unix_mode = UnixMode::from(0o041755);
        let go_mode = GoFileMode::from(unix_mode);
        assert_eq!(go_mode.as_u32(), GO_MODE_DIR | GO_MODE_STICKY | 0o755);
    }

    #[test]
    fn test_all_special_bits() {
        // File with setuid, setgid, sticky, and 0o777 permissions
        let unix_mode = UnixMode::from(0o107777);
        let go_mode = GoFileMode::from(unix_mode);
        assert_eq!(
            go_mode.as_u32(),
            GO_MODE_SETUID | GO_MODE_SETGID | GO_MODE_STICKY | 0o777
        );
    }

    #[test]
    fn test_into_conversion() {
        // Test using .into() for ergonomic conversion
        let unix_mode = UnixMode::from(0o040755);
        let go_mode: GoFileMode = unix_mode.into();
        assert_eq!(go_mode.as_u32(), 0x800001ed);
    }

    #[test]
    fn test_type_conversions() {
        // Test u32 -> UnixMode -> GoFileMode -> u32 round trip
        let original = 0o100644u32;
        let unix_mode = UnixMode::from(original);
        let go_mode = GoFileMode::from(unix_mode);
        let result = go_mode.as_u32();

        assert_eq!(result, 0o644); // Regular file loses type bits in Go
    }
}
