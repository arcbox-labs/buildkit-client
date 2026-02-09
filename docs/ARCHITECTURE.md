# Architecture

This document provides a comprehensive overview of the BuildKit Rust Client architecture, including key components, protocols, and implementation details.

## Table of Contents

- [Overview](#overview)
- [High-Level Data Flow](#high-level-data-flow)
- [Core Components](#core-components)
- [Project Structure](#project-structure)
- [Implementation Details](#implementation-details)
- [Common Pitfalls](#common-pitfalls)
- [References](#references)

## Overview

BuildKit Rust Client is a full-featured Rust implementation of the BuildKit gRPC protocol. It implements:

- **Bidirectional gRPC streaming** for real-time build communication
- **HTTP/2-over-gRPC tunneling** for BuildKit callbacks
- **DiffCopy protocol** for efficient file synchronization
- **Session management** with proper metadata handling
- **Registry authentication** for pushing built images

The client can build container images from both local directories and GitHub repositories, with support for multi-platform builds, build arguments, and cache management.

## High-Level Data Flow

```
BuildKitClient.build(config)
  ↓
1. Create Session with UUID
   - Generate unique session ID
   - Initialize FileSync service (for local builds)
   - Setup auth service for registry credentials
  ↓
2. Session.start() → Opens bidirectional gRPC stream
   - Establish connection to BuildKit daemon
   - Start HTTP/2 tunnel inside session stream
   - Register callback handlers (DiffCopy, Auth, Health)
  ↓
3. Prepare SolveRequest
   - Frontend: "dockerfile.v0"
   - Context:
     * Local: "input:session-{uuid}:context"
     * GitHub: "https://token@github.com/user/repo.git#branch"
   - Attach session metadata headers
     * X-Docker-Expose-Session-Uuid
     * X-Docker-Expose-Session-Name
     * X-Docker-Expose-Session-Grpc-Method
  ↓
4. control.solve(request) → BuildKit begins build
   - BuildKit validates session headers
   - Starts build process using Dockerfile
   - Streams progress updates
  ↓
5. BuildKit callbacks through HTTP/2 tunnel
   - DiffCopy: Request and receive build context files
   - Credentials: Get registry authentication
   - Health: Verify session is alive
  ↓
6. Build completion
   - BuildKit pushes image to registry
   - Returns image digest
   - Session closes gracefully
```

## Core Components

### 1. BuildKit Client ([src/client.rs](../src/client.rs))

The main client interface for interacting with BuildKit:

```rust
pub struct BuildKitClient {
    control: ControlClient<Channel>,
    addr: String,
}
```

**Key responsibilities:**
- Establish gRPC connection to BuildKit daemon
- Execute build operations via `control.solve()`
- Stream build progress and logs
- Handle health checks

**Main methods:**
- `connect(addr)` - Connect to BuildKit daemon
- `build(config, progress_handler)` - Execute a build
- `health()` - Check daemon health status
- `info()` - Get BuildKit daemon information

### 2. Session Protocol ([src/session/mod.rs](../src/session/mod.rs))

Orchestrates the bidirectional communication with BuildKit:

```rust
pub struct Session {
    id: String,
    shared_key: String,
    services: Vec<Box<dyn SessionService>>,
}
```

**Key responsibilities:**
- Generate session ID and shared key
- Manage session services (FileSync, Auth)
- Generate required metadata headers for BuildKit
- Handle bidirectional gRPC stream lifecycle

**Critical implementation details:**
- Session ID must be UUID format
- Shared key identifies the session in BuildKit
- Metadata headers MUST be attached to solve requests
- Services registered before session starts

**Required headers:**
```
X-Docker-Expose-Session-Uuid: {session_id}
X-Docker-Expose-Session-Name: {shared_key}
X-Docker-Expose-Session-Grpc-Method: /moby.filesync.v1.FileSync/DiffCopy
X-Docker-Expose-Session-Grpc-Method: /moby.filesync.v1.Auth/Credentials
X-Docker-Expose-Session-Grpc-Method: /grpc.health.v1.Health/Check
```

### 3. HTTP/2-over-gRPC Tunnel ([src/session/grpc_tunnel.rs](../src/session/grpc_tunnel.rs))

**The most complex component** - Implements a complete gRPC server inside a gRPC stream.

#### Architecture

```
BuildKit Control.Session stream (outer gRPC)
  ↓
BytesMessage containing HTTP/2 frames
  ↓
h2::server::Builder (inner HTTP/2 server)
  ↓
HTTP/2 requests decoded to gRPC calls
  ↓
Route to appropriate handler:
  - /moby.filesync.v1.FileSync/DiffCopy → Bidirectional stream
  - /moby.filesync.v1.Auth/Credentials → Unary RPC
  - /grpc.health.v1.Health/Check → Unary RPC
```

#### gRPC Message Framing

Each gRPC message uses this format:
```
[compression flag: 1 byte] + [length: 4 bytes BE] + [protobuf payload]
```

**Implementation details:**
- Parse frame boundaries from buffered BytesMessage chunks
- Add 5-byte prefix when sending responses
- Handle partial frames across multiple BytesMessage packets
- Properly decode and encode protobuf messages

### 4. DiffCopy Protocol ([src/session/grpc_tunnel.rs](../src/session/grpc_tunnel.rs) lines 168-511)

Implements fsutil's bidirectional streaming protocol for file synchronization.

#### Protocol Flow

**Phase 1: Server sends file listing**
```
Server → Client: STAT packet (file1, id=0)
Server → Client: STAT packet (file2, id=1)
Server → Client: STAT packet (dir1, id=2)
Server → Client: STAT packet (file3, id=3)
Server → Client: STAT packet (empty) // End of listing
```

**Phase 2: Client requests file contents**
```
Client → Server: REQ packet (id=0)  // Request file1
Server → Client: DATA packet (chunk1)
Server → Client: DATA packet (chunk2)
Server → Client: DATA packet (empty) // EOF for file1

Client → Server: REQ packet (id=3)  // Request file3
Server → Client: DATA packet (data)
Server → Client: DATA packet (empty) // EOF for file3

Client → Server: FIN packet  // Done requesting
Server → Client: FIN packet  // Acknowledge
```

#### Critical Protocol Details

1. **ID Assignment**: IDs are assigned to ALL entries (files AND directories), starting from 0
2. **Root Directory**: Root directory itself gets NO STAT packet, only its children
3. **File Map**: Only files (not directories) are stored in the file map
4. **Mode Bits**: Must be correct
   - Files: `0o100644` (regular file, rw-r--r--)
   - Directories: `0o040755` (directory, rwxr-xr-x)
5. **Empty Packets**: Empty DATA packet signals EOF for a file, NOT end of entire transfer
6. **FIN Packets**: Signal completion of entire transfer

#### Example Implementation

```rust
// Walking directory and sending STAT packets
let mut id = 0;
for entry in WalkDir::new(context_path) {
    let stat = Stat {
        path: relative_path,
        mode: if is_dir { 0o040755 } else { 0o100644 },
        size: file_size,
        ..
    };

    let packet = Packet {
        r#type: PacketType::PacketStat as i32,
        stat: Some(stat),
        id: id,
    };

    send_packet(packet);
    id += 1;
}

// Send empty STAT to signal end of listing
send_packet(Packet {
    r#type: PacketType::PacketStat as i32,
    stat: None,
    ..
});
```

### 5. Auth Protocol ([src/session/auth.rs](../src/session/auth.rs))

Provides registry authentication for pushing images.

**Implemented methods:**
- `Credentials(host)` - Return credentials for registry
- `FetchToken(host)` - Return authentication token
- `GetTokenAuthority(host)` - Return token authority

**Implementation strategy:**
- `GetTokenAuthority`: Return error → BuildKit falls back to basic auth
- `Credentials`: Return empty if no auth → BuildKit proceeds without auth
- `FetchToken`: Return empty token

**Critical:** All unary responses MUST include `grpc-status: 0` in trailers.

### 6. Solve Operation ([src/solve.rs](../src/solve.rs))

Prepares and executes the BuildKit solve request.

**Key configuration:**
- **Frontend**: Always `"dockerfile.v0"`
- **Context source**:
  - Local: `"input:{session_shared_key}:context"`
  - GitHub: `"https://{token}@github.com/{user}/{repo}.git#{ref}"`
- **Frontend attributes**:
  - `build-arg:KEY=value` - Build arguments
  - `target` - Target stage
  - `platform` - Target platform(s)
  - `filename` - Dockerfile name
  - `no-cache` - Disable cache
- **Exporters**: Image push configuration
- **Cache**: Import/export settings

**Critical:** Session metadata headers MUST be attached to the solve request.

## Project Structure

```
src/
├── main.rs                 # CLI entry point
├── lib.rs                  # Library entry point
├── client.rs              # BuildKitClient implementation
├── builder.rs             # BuildConfig and configuration
├── solve.rs               # Solve request preparation and execution
├── progress.rs            # Progress handlers (Console, JSON, Silent)
├── session/
│   ├── mod.rs             # Session lifecycle and metadata
│   ├── grpc_tunnel.rs     # HTTP/2-over-gRPC tunnel (most complex)
│   ├── filesync.rs        # FileSyncServer implementation
│   └── auth.rs            # AuthServer for registry credentials
└── proto.rs               # Protobuf generated code

proto/                      # Auto-generated at build time
├── github.com/
│   ├── moby/buildkit/     # BuildKit proto definitions
│   ├── tonistiigi/fsutil/ # File sync protocol
│   ├── containerd/        # Container types
│   └── planetscale/       # Proto extensions
└── google/rpc/            # Google RPC types
```

## Implementation Details

### BuildKit gRPC API Usage

The client directly uses BuildKit's gRPC API:

```
┌─────────────────────────────────────────┐
│         BuildKit Control API            │
├─────────────────────────────────────────┤
│ Control.Solve      - Execute builds     │
│ Control.Status     - Stream progress    │
│ Control.Session    - Bidirectional      │
│ Control.Info       - Get daemon info    │
└─────────────────────────────────────────┘
```

### Nested Loop Exit Pattern

**Important:** Avoid this common bug in bidirectional streams:

```rust
// ❌ INCORRECT - breaks inner loop only
loop {                              // Outer: read from stream
    match stream.receive() {
        Some(data) => {
            while buffer.has_data() {  // Inner: parse messages
                if is_fin_packet {
                    break;  // Only breaks inner loop!
                }
            }
        }
    }
}

// ✅ CORRECT - use flag to break outer loop
let mut received_fin = false;
loop {
    match stream.receive() {
        Some(data) => {
            while buffer.has_data() {
                if is_fin_packet {
                    received_fin = true;
                    break;
                }
            }
            if received_fin { break; }  // Break outer loop
        }
    }
}
```

### Progress Handling

Three progress handlers are provided:

1. **ConsoleProgressHandler** - Colored terminal output with spinners
2. **JsonProgressHandler** - Structured JSON output for parsing
3. **SilentProgressHandler** - No output

Progress updates are streamed in real-time via `Control.Status` RPC.

## Common Pitfalls

### 1. Missing Session Headers

**Symptom:** "no active session" error

**Solution:** Ensure session metadata headers are attached to SolveRequest:
```rust
let mut request = Request::new(solve_request);
for (key, value) in session.metadata() {
    request.metadata_mut().insert(key, value);
}
```

### 2. DiffCopy Stream Not Closed

**Symptom:** Build timeout or "context canceled" error

**Solution:** Properly handle FIN packets and close streams:
- Send FIN packet when done requesting files
- Wait for FIN response from BuildKit
- Close HTTP/2 stream with trailers

### 3. Incorrect File ID Mapping

**Symptom:** "invalid file request N" error

**Solution:**
- Assign IDs to ALL entries (files + directories)
- Only store files in file_map (not directories)
- IDs must be sequential starting from 0

### 4. Incorrect Mode Bits

**Symptom:** Build fails with permission errors

**Solution:** Use correct Unix mode bits:
- Regular files: `0o100644`
- Directories: `0o040755`

## References

### BuildKit Resources

- [BuildKit GitHub](https://github.com/moby/buildkit) - Official BuildKit project
- [BuildKit Documentation](https://github.com/moby/buildkit/tree/master/docs) - Official docs
- [BuildKit API](https://github.com/moby/buildkit/tree/master/api) - Proto definitions

### Protocol References

- [fsutil](https://github.com/tonistiigi/fsutil) - File sync utility
  - `send.go` (lines 150-250) - DiffCopy server behavior
  - `receive.go` - DiffCopy client behavior
- [gRPC Protocol](https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-HTTP2.md) - gRPC over HTTP/2 spec
- [HTTP/2 Spec](https://httpwg.org/specs/rfc7540.html) - HTTP/2 protocol

### Rust Libraries

- [tonic](https://github.com/hyperium/tonic) - Rust gRPC framework
- [prost](https://github.com/tokio-rs/prost) - Protocol Buffers implementation
- [h2](https://github.com/hyperium/h2) - HTTP/2 implementation
- [tokio](https://tokio.rs/) - Async runtime

### Related Documentation

- [Usage Guide](./USAGE.md) - CLI and library usage
- [Development Guide](./DEVELOPMENT.md) - Development workflows
- [Testing Guide](./TESTING.md) - Testing strategies
- [CLAUDE.md](../CLAUDE.md) - Detailed implementation notes

---

For detailed implementation guidance and troubleshooting, see [CLAUDE.md](../CLAUDE.md).
