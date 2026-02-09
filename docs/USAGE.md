# Usage Guide

This guide covers both CLI and library usage of the BuildKit Rust Client.

## Table of Contents

- [CLI Usage](#cli-usage)
- [Library Usage](#library-usage)
- [Configuration Options](#configuration-options)
- [Environment Variables](#environment-variables)

## CLI Usage

### Build Local Dockerfile

```bash
cargo run -- local \
  --context ./examples/test-dockerfile \
  --tag registry:5000/test:latest
```

### Using Build Arguments

```bash
cargo run -- local \
  --context ./examples/multi-stage \
  --tag registry:5000/multi-stage:latest \
  --build-arg APP_VERSION=2.0.0 \
  --build-arg BUILD_DATE=$(date +%Y-%m-%d)
```

### Specify Target Stage

```bash
cargo run -- local \
  --context ./examples/multi-stage \
  --tag registry:5000/dev:latest \
  --target dev
```

### Multi-platform Build

```bash
cargo run -- local \
  --context ./examples/test-dockerfile \
  --tag localhost:5000/multi-arch:latest \
  --platform linux/amd64 \
  --platform linux/arm64
```

### Build from GitHub Repository

```bash
# Public repository
cargo run -- github https://github.com/user/repo.git \
  --tag localhost:5000/from-github:latest \
  --git-ref main

# Private repository (with environment variable)
export GITHUB_TOKEN=ghp_your_token_here
cargo run -- github https://github.com/user/private-repo.git \
  --tag localhost:5000/private:latest \
  --git-ref main
```

### Build with Registry Authentication

```bash
cargo run -- local \
  --context ./examples/test-dockerfile \
  --tag registry.example.com/myapp:latest \
  --registry-host registry.example.com \
  --registry-user myuser \
  --registry-password mypassword
```

### JSON Output Mode

```bash
cargo run -- local \
  --context ./examples/test-dockerfile \
  --tag localhost:5000/test:latest \
  --json
```

## Library Usage

### Basic Example

```rust
use buildkit_client::{BuildKitClient, BuildConfig};
use buildkit_client::progress::ConsoleProgressHandler;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Connect to BuildKit
    let mut client = BuildKitClient::connect("http://localhost:1234").await?;

    // Configure build
    let config = BuildConfig::local("./my-app")
        .tag("localhost:5000/my-app:latest")
        .build_arg("VERSION", "1.0.0");

    // Execute build
    let progress = Box::new(ConsoleProgressHandler::new(true));
    let result = client.build(config, Some(progress)).await?;

    println!("✅ Build completed!");
    if let Some(digest) = result.digest {
        println!("📦 Image digest: {}", digest);
    }

    Ok(())
}
```

### GitHub Repository Build

```rust
use buildkit_client::{BuildKitClient, BuildConfig, RegistryAuth};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = BuildKitClient::connect("http://localhost:1234").await?;

    let config = BuildConfig::github("https://github.com/user/repo.git")
        .git_ref("main")
        .github_token("ghp_your_token")
        .dockerfile("path/to/Dockerfile")
        .tag("localhost:5000/from-github:latest")
        .build_arg("ENV", "production");

    let result = client.build(config, None).await?;
    Ok(())
}
```

### Multi-platform Build

```rust
use buildkit_client::{BuildKitClient, BuildConfig, Platform};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = BuildKitClient::connect("http://localhost:1234").await?;

    let config = BuildConfig::local("./my-app")
        .tag("localhost:5000/multi-arch:latest")
        .platform(Platform::linux_amd64())
        .platform(Platform::linux_arm64())
        .platform(Platform::parse("linux/arm/v7")?);

    let result = client.build(config, None).await?;
    Ok(())
}
```

## Configuration Options

### BuildConfig

- `source` - Build source (local or GitHub)
- `dockerfile_path` - Path to Dockerfile
- `build_args` - Build arguments
- `target` - Target stage
- `platforms` - List of target platforms
- `tags` - List of image tags
- `registry_auth` - Registry authentication info
- `cache_from` - Cache import sources
- `cache_to` - Cache export destinations
- `secrets` - Build-time secrets
- `no_cache` - Disable caching
- `pull` - Always pull base images

### ProgressHandler

Three progress handlers are provided:

1. **ConsoleProgressHandler** - Output to console with colors
2. **JsonProgressHandler** - JSON format output
3. **SilentProgressHandler** - Silent mode

## Environment Variables

- `BUILDKIT_ADDR` - BuildKit address (default: `http://localhost:1234`)
- `GITHUB_TOKEN` - GitHub authentication token
- `RUST_LOG` - Log level (trace, debug, info, warn, error)
  - `RUST_LOG=info,buildkit_client::session::grpc_tunnel=trace` for protocol debugging
