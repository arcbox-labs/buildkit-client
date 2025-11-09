# 快速入门指南

## 首次设置（3 步）

### 1. 初始化 Proto 文件

```bash
./scripts/init-proto.sh
```

或使用 Makefile：

```bash
make proto-init
```

### 2. 启动 BuildKit 和 Registry

```bash
docker-compose up -d
# 或
make up
```

### 3. 构建并测试

```bash
cargo build
cargo run -- health
# 或
make build
make health
```

## 常用命令

### Proto 文件管理

```bash
# 初始化（首次使用）
make proto-init

# 更新到最新版本
make proto-clean
make proto-init
```

### 开发

```bash
# 查看所有可用命令
make help

# 构建
make build

# 运行测试
make test

# 代码检查
make check
make fmt
make clippy
```

### Docker 管理

```bash
# 启动服务
make up

# 停止服务
make down

# 查看日志
make logs
```

### 测试构建

```bash
# 健康检查
make health

# 测试本地构建
make run-local

# 测试 GitHub 构建
make run-github
```

## 项目结构

```
buildkit-client/
├── scripts/
│   └── init-proto.sh      # Proto 文件初始化脚本
├── proto/                  # Proto 文件（自动生成，已在 .gitignore）
├── src/                    # 源代码
├── tests/                  # 测试
├── examples/               # 示例 Dockerfile
├── Makefile               # 常用命令
├── PROTO_SETUP.md         # Proto 管理详细说明
└── README.md              # 完整文档
```

## 故障排除

### Proto 文件问题

```bash
# 完全重置
rm -rf proto
./scripts/init-proto.sh
cargo clean
cargo build
```

### BuildKit 连接问题

```bash
# 检查服务状态
docker-compose ps

# 重启服务
make down
make up

# 查看日志
make logs
```

### 编译问题

```bash
# 清理并重新构建
cargo clean
cargo build

# 或使用 Makefile
make clean
make build
```

## 下一步

- 阅读 [README.md](./README.md) 了解完整功能
- 查看 [PROTO_SETUP.md](./PROTO_SETUP.md) 了解 Proto 管理细节
- 查看 `examples/` 目录的示例 Dockerfile
- 运行 `make help` 查看所有可用命令
