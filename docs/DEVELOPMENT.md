# Development Guide

This guide covers development workflows, testing, and proto file management.

## Table of Contents

- [Setup](#setup)
- [Using Makefile](#using-makefile)
- [Protobuf Management](#protobuf-management)
- [Testing](#testing)
- [Code Quality](#code-quality)
- [Benchmarks](#benchmarks)

## Setup

### Initial Setup

```bash
# Clone the repository
git clone https://github.com/corespeed-io/buildkit-client.git
cd buildkit-client

# Build the project (proto files will be fetched automatically)
cargo build

# Start development environment
docker-compose up -d
```

## Using Makefile

The project provides a Makefile to simplify common operations:

```bash
make help          # Show all available commands
make init          # Initialize project and build
make build         # Build project
make test          # Run tests
make up            # Start docker-compose services
make down          # Stop docker-compose services
make health        # Check BuildKit health status
```

## Protobuf Management

Proto files are automatically managed by `build.rs` during the build process. The build script fetches proto files from the BuildKit and GoogleAPIs repositories at build time.

### How It Works

The build script (`build.rs`) automatically:
1. Downloads proto files from GitHub repositories (BuildKit and GoogleAPIs)
2. Organizes them in the correct directory structure
3. Compiles them with `tonic-build`

No manual proto file management is required for normal development.

### Build Modes

Two fetch modes are supported:

#### 1. Content Mode (Default)
Downloads individual proto files directly from GitHub using HTTPS:
```bash
cargo build
```

#### 2. Clone Mode
Clones the entire repositories:
```bash
PROTO_FETCH_MODE=clone cargo build
```

### Environment Variables

Control proto fetching behavior with these environment variables:

```bash
# Fetch mode (content or clone, default: content)
PROTO_FETCH_MODE=clone cargo build

# Force rebuild/redownload proto files
PROTO_REBUILD=true cargo build

# Customize BuildKit repository and version
BUILDKIT_REPO=https://github.com/moby/buildkit.git \
BUILDKIT_REF=v0.12.0 \
cargo build

# Customize GoogleAPIs repository and version
GOOGLEAPIS_REPO=https://github.com/googleapis/googleapis.git \
GOOGLEAPIS_REF=master \
cargo build
```

### Troubleshooting Proto Issues

If you encounter proto-related build errors:

```bash
# Force clean rebuild (will redownload all proto files)
cargo clean
PROTO_REBUILD=true cargo build

# Or use clone mode if download fails
cargo clean
PROTO_FETCH_MODE=clone cargo build
```

The proto files are stored in the build output directory (`target/debug/build/buildkit-client-*/out/proto/`), not in the source tree.

### Proto File Structure

The build script fetches and organizes proto files in this structure:

```
proto/
├── github.com/
│   ├── moby/buildkit/          # BuildKit API definitions
│   │   ├── api/services/control/
│   │   ├── api/types/
│   │   ├── solver/pb/
│   │   ├── session/auth/
│   │   ├── session/filesync/
│   │   └── ...
│   ├── tonistiigi/fsutil/      # File sync protocol
│   ├── planetscale/vtprotobuf/ # Protocol extensions
│   └── containerd/containerd/  # Container types
└── google/rpc/                 # Google RPC types
```

## Testing

### Unit Tests

Unit tests don't require BuildKit to be running:

```bash
# All unit tests
cargo test --lib

# Specific unit test files
cargo test --test builder_test
cargo test --test session_test
cargo test --test progress_test

# Run a specific test with output
cargo test test_platform_parse -- --nocapture
```

### Integration Tests

Integration tests require a running BuildKit instance:

```bash
# Start BuildKit first
docker run -d --rm --privileged -p 1234:1234 moby/buildkit:latest --addr tcp://0.0.0.0:1234

# Or use docker-compose
docker-compose up -d

# Run all integration tests
cargo test --test integration_test -- --test-threads=1

# Run specific integration test
cargo test --test integration_test test_build_from_github_public -- --test-threads=1
```

### GitHub Repository Tests

Tests that build from GitHub repositories:

```bash
# Using default test token
cargo test --test integration_test github -- --test-threads=1

# Using custom GitHub token
GITHUB_TOKEN=ghp_your_token_here cargo test --test integration_test github -- --test-threads=1
```

### Using Test Scripts

The project includes test scripts for convenience:

```bash
# Run all tests
./scripts/test.sh all

# Run only unit tests
./scripts/test.sh unit

# Run only integration tests
./scripts/test.sh integration

# Run GitHub tests
./scripts/test.sh github
```

### Using Test Makefile

For more advanced testing workflows:

```bash
# Unit tests
make -f Makefile.test test

# Integration tests
make -f Makefile.test test-integration

# GitHub tests
make -f Makefile.test test-github

# Code coverage
make -f Makefile.test coverage

# Benchmarks
make -f Makefile.test bench
```

## Code Quality

### Formatting

```bash
# Format code
cargo fmt

# Check formatting without applying
cargo fmt -- --check
```

### Linting

```bash
# Run clippy
cargo clippy

# Run clippy with all features
cargo clippy --all-features

# Deny warnings
cargo clippy -- -D warnings
```

### Static Analysis

```bash
# Check for common mistakes
cargo clippy --all-targets --all-features
```

## Benchmarks

The project includes benchmarks for performance-critical components:

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench session_bench
```

Benchmark results are saved in `target/criterion/`.

## Development Workflow

### Typical Development Cycle

1. **Make changes** to the code
2. **Format** with `cargo fmt`
3. **Check** with `cargo clippy`
4. **Build** with `cargo build`
5. **Test** with `cargo test`
6. **Run integration tests** if needed
7. **Benchmark** if performance-critical changes

### Quick Health Check

```bash
# Test the build and CLI
cargo build --release
cargo run -- health
```

### Debug Logging

Use `RUST_LOG` environment variable for detailed logging:

```bash
# Info level
RUST_LOG=info cargo run -- local -c examples/test-dockerfile -t localhost:5000/test:latest

# Debug level for session
RUST_LOG=info,buildkit_client::session=debug cargo run -- local -c . -t test:latest

# Trace level for gRPC tunnel
RUST_LOG=info,buildkit_client::session::grpc_tunnel=trace cargo run -- local -c . -t test:latest
```

## Docker Environment

### Start Services

```bash
# Start BuildKit and registry
docker-compose up -d

# Check status
docker-compose ps

# View logs
docker-compose logs -f buildkitd
```

### Stop Services

```bash
# Stop services
docker-compose down

# Stop and remove volumes
docker-compose down -v
```

## Contributing

When contributing to the project:

1. Run `cargo fmt` and `cargo clippy`
2. Ensure all tests pass: `cargo test`
3. Add tests for new features
4. Update documentation as needed
5. Keep commits clean and descriptive

## Architecture Reference

For detailed architecture documentation, including the session protocol, HTTP/2 tunnel, and DiffCopy implementation, see [CLAUDE.md](../CLAUDE.md).
