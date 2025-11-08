# BuildKit Rust Client

一个功能完整的 Rust 客户端，用于通过 gRPC 与 moby/buildkit 交互，构建容器镜像。

## 特性

- ✅ **完整的 gRPC 实现** - 直接使用 BuildKit 的 gRPC API
- 🏗️ **多种构建源** - 支持本地 Dockerfile 和 GitHub 仓库
- 🔐 **认证支持** - 支持 GitHub 私有仓库和 Docker Registry 认证
- 🚀 **高级构建选项** - Build args、target stage、multi-platform builds
- 📊 **实时进度** - 实时显示构建进度和日志
- 💾 **缓存管理** - 支持 cache import/export
- 🎯 **推送到 Registry** - 自动推送构建好的镜像

## 前置要求

- Rust 1.70+
- Docker 或 BuildKit daemon
- (可选) buf - 用于管理 protobuf 文件

## 快速开始

### 1. 启动 BuildKit 和 Registry

```bash
docker-compose up -d
```

这将启动：
- BuildKit daemon (端口 1234)
- 本地 Docker Registry (端口 5000)

### 2. 编译项目

```bash
cargo build --release
```

### 3. 运行示例

#### Health Check

```bash
cargo run -- health
```

#### 构建本地 Dockerfile

```bash
cargo run -- local \
  --context ./examples/test-dockerfile \
  --tag localhost:5000/test:latest
```

#### 使用 Build Arguments

```bash
cargo run -- local \
  --context ./examples/multi-stage \
  --tag localhost:5000/multi-stage:latest \
  --build-arg APP_VERSION=2.0.0 \
  --build-arg BUILD_DATE=$(date +%Y-%m-%d)
```

#### 指定 Target Stage

```bash
cargo run -- local \
  --context ./examples/multi-stage \
  --tag localhost:5000/dev:latest \
  --target dev
```

#### 多平台构建

```bash
cargo run -- local \
  --context ./examples/test-dockerfile \
  --tag localhost:5000/multi-arch:latest \
  --platform linux/amd64 \
  --platform linux/arm64
```

#### 从 GitHub 仓库构建

```bash
# 公开仓库
cargo run -- github https://github.com/user/repo.git \
  --tag localhost:5000/from-github:latest \
  --git-ref main

# 私有仓库（使用环境变量）
export GITHUB_TOKEN=ghp_your_token_here
cargo run -- github https://github.com/user/private-repo.git \
  --tag localhost:5000/private:latest \
  --git-ref main
```

#### 带 Registry 认证的构建

```bash
cargo run -- local \
  --context ./examples/test-dockerfile \
  --tag registry.example.com/myapp:latest \
  --registry-host registry.example.com \
  --registry-user myuser \
  --registry-password mypassword
```

#### JSON 输出模式

```bash
cargo run -- local \
  --context ./examples/test-dockerfile \
  --tag localhost:5000/test:latest \
  --json
```

## 作为库使用

在 `Cargo.toml` 中添加：

```toml
[dependencies]
buildkit-client = { path = "." }
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
```

### 基本示例

```rust
use buildkit_client::{BuildKitClient, BuildConfig};
use buildkit_client::progress::ConsoleProgressHandler;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 连接到 BuildKit
    let mut client = BuildKitClient::connect("http://localhost:1234").await?;

    // 配置构建
    let config = BuildConfig::local("./my-app")
        .tag("localhost:5000/my-app:latest")
        .build_arg("VERSION", "1.0.0");

    // 执行构建
    let progress = Box::new(ConsoleProgressHandler::new(true));
    let result = client.build(config, Some(progress)).await?;

    println!("✅ Build completed!");
    if let Some(digest) = result.digest {
        println!("📦 Image digest: {}", digest);
    }

    Ok(())
}
```

### GitHub 仓库构建

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

### 多平台构建

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

## 项目结构

```
buildkit-client/
├── src/
│   ├── main.rs          # CLI 工具入口
│   ├── lib.rs           # 库入口
│   ├── client.rs        # BuildKit gRPC 客户端
│   ├── builder.rs       # 构建配置
│   ├── solve.rs         # 构建执行逻辑
│   ├── progress.rs      # 进度处理
│   └── proto.rs         # Protobuf 生成代码
├── proto/               # BuildKit protobuf 定义
├── examples/            # 示例 Dockerfile
├── docker-compose.yml   # 测试环境配置
└── README.md
```

## BuildKit gRPC API

本项目直接使用 BuildKit 的 gRPC API：

- `Control.Solve` - 执行构建操作
- `Control.Status` - 获取构建状态流
- `Control.Info` - 获取 BuildKit 信息

所有的 protobuf 定义都从 [moby/buildkit](https://github.com/moby/buildkit) 仓库获取。

## 配置选项

### BuildConfig

- `source` - 构建源（本地或 GitHub）
- `dockerfile_path` - Dockerfile 路径
- `build_args` - 构建参数
- `target` - 目标 stage
- `platforms` - 目标平台列表
- `tags` - 镜像标签列表
- `registry_auth` - Registry 认证信息
- `cache_from` - 缓存导入源
- `cache_to` - 缓存导出目标
- `secrets` - 构建时使用的 secrets
- `no_cache` - 禁用缓存
- `pull` - 总是拉取基础镜像

### ProgressHandler

提供了三种进度处理器：

1. **ConsoleProgressHandler** - 输出到控制台
2. **JsonProgressHandler** - JSON 格式输出
3. **SilentProgressHandler** - 静默模式

## 环境变量

- `GITHUB_TOKEN` - GitHub 认证令牌
- `RUST_LOG` - 日志级别 (trace, debug, info, warn, error)

## 故障排除

### BuildKit 连接失败

```bash
# 检查 BuildKit 是否运行
docker-compose ps

# 查看 BuildKit 日志
docker-compose logs buildkitd

# 重启服务
docker-compose restart
```

### Registry 推送失败

确保 registry 允许 insecure 连接（对于 localhost）：

```yaml
# docker-compose.yml
services:
  buildkitd:
    environment:
      - BUILDKIT_REGISTRY_INSECURE=true
```

### Proto 编译错误

如果遇到 protobuf 编译错误：

```bash
# 清理并重新编译
cargo clean
cargo build
```

## 开发

### 更新 Protobuf 定义

```bash
# 从 buildkit 仓库更新 proto 文件
cd /tmp
git clone https://github.com/moby/buildkit.git
cp -r buildkit/api proto/
cp -r buildkit/solver proto/
cp -r buildkit/sourcepolicy proto/
cp -r buildkit/frontend proto/

# 从 googleapis 更新
git clone https://github.com/googleapis/googleapis.git
cp googleapis/google/rpc/*.proto proto/google/rpc/
```

### 运行测试

```bash
cargo test
```

### 代码格式化

```bash
cargo fmt
cargo clippy
```

## 许可证

本项目采用 MIT 或 Apache-2.0 双许可证。

## 致谢

- [moby/buildkit](https://github.com/moby/buildkit) - BuildKit 项目
- [tonic](https://github.com/hyperium/tonic) - Rust gRPC 库
- [prost](https://github.com/tokio-rs/prost) - Protocol Buffers 实现

## 贡献

欢迎提交 Issue 和 Pull Request！

## 相关链接

- [BuildKit 文档](https://github.com/moby/buildkit/tree/master/docs)
- [BuildKit API 参考](https://github.com/moby/buildkit/tree/master/api)
- [Docker Buildx](https://github.com/docker/buildx)
