# filemode

[![Crates.io](https://img.shields.io/crates/v/filemode.svg)](https://crates.io/crates/filemode)
[![Documentation](https://docs.rs/filemode/badge.svg)](https://docs.rs/filemode)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](../LICENSE)

A Rust library for converting between Unix `mode_t` and Go `os.FileMode` formats.

## Overview

This crate provides type-safe, zero-cost abstractions for converting file permission and type information between Unix (POSIX) systems and Go's file mode representation. It's particularly useful when working with systems that bridge Rust and Go ecosystems, such as BuildKit, Docker, or other container build systems.

## Features

- **Type-safe conversions** - Prevents accidental mixing of Unix and Go mode values
- **Zero-cost abstractions** - All conversions are inlined with no runtime overhead
- **Idiomatic Rust** - Uses `From`/`Into` traits for ergonomic conversions
- **Well-tested** - Comprehensive test coverage including edge cases
- **Well-documented** - Detailed documentation with examples
- **Backward compatible** - Maintains legacy function API

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
filemode = "0.1"
```

## Usage

### Type-Safe API (Recommended)

```rust
use filemode::{UnixMode, GoFileMode};

// Convert Unix directory mode to Go FileMode
let unix_mode = UnixMode::from(0o040755);
let go_mode: GoFileMode = unix_mode.into();
assert_eq!(go_mode.as_u32(), 0x800001ed); // ModeDir | 0o755

// Convert Unix regular file mode
let unix_mode = UnixMode::from(0o100644);
let go_mode = GoFileMode::from(unix_mode);
assert_eq!(go_mode.as_u32(), 0o644);
```

### Legacy Function API

```rust
use filemode::unix_mode_to_go_filemode;

// Directory with 0o755 permissions
let go_mode = unix_mode_to_go_filemode(0o040755);
assert_eq!(go_mode, 0x800001ed);

// Regular file with 0o644 permissions
let go_mode = unix_mode_to_go_filemode(0o100644);
assert_eq!(go_mode, 0o644);
```

### Real-World Example

```rust
use std::fs;
use std::os::unix::fs::PermissionsExt;
use filemode::{UnixMode, GoFileMode};

// Get file metadata from filesystem
let metadata = fs::metadata("some_file.txt")?;
let unix_mode = metadata.permissions().mode();

// Convert to Go FileMode for use with BuildKit or other Go-based tools
let go_mode = GoFileMode::from(UnixMode::from(unix_mode));

// Send go_mode.as_u32() to your gRPC/protocol buffer message
```

## Technical Details

### Unix mode_t Format

Unix uses bits 14-12 for file type:
- `S_IFDIR` (0o040000) - Directory
- `S_IFREG` (0o100000) - Regular file
- `S_IFLNK` (0o120000) - Symbolic link
- And others (FIFO, socket, device)

Permissions are stored in the lower 12 bits (0o7777).

### Go os.FileMode Format

Go uses high bits for file type flags:
- Bit 31 (0x80000000) - Directory
- Bit 27 (0x08000000) - Symbolic link
- Bit 26 (0x04000000) - Device
- And others

Regular files have no special type bit set.

### Conversion Mapping

| Unix Type | Octal | Go Mode | Hex |
|-----------|-------|---------|-----|
| Regular file | 0o100000 | (none) | 0x00000000 |
| Directory | 0o040000 | ModeDir | 0x80000000 |
| Symlink | 0o120000 | ModeSymlink | 0x08000000 |
| Named pipe | 0o010000 | ModeNamedPipe | 0x02000000 |
| Socket | 0o140000 | ModeSocket | 0x01000000 |

Special permission bits (setuid, setgid, sticky) are also converted appropriately.

## Supported File Types

- ✅ Regular files
- ✅ Directories
- ✅ Symbolic links
- ✅ Named pipes (FIFO)
- ✅ Sockets
- ✅ Character devices
- ✅ Block devices
- ✅ Special permission bits (setuid, setgid, sticky)

## Why This Crate?

When building systems that interact with both Rust and Go code (like container build systems), you often need to serialize file metadata in a way that Go can understand. Direct conversion of Unix mode_t values to Go's FileMode format requires careful bit manipulation. This crate handles all the edge cases correctly.

## Use Cases

- Building gRPC services that communicate with Go-based systems
- Container build tools (BuildKit, Docker)
- File synchronization protocols (fsutil)
- Any system bridging Rust and Go file handling

## Performance

All conversion functions are marked `#[inline]` and compile down to simple bit operations. There is zero runtime overhead compared to manual bit manipulation.

## License

This project is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Related Projects

- [buildkit-client](https://github.com/AprilNEA/buildkit-client) - Rust client for BuildKit
- [BuildKit](https://github.com/moby/buildkit) - Concurrent, cache-efficient build toolkit
