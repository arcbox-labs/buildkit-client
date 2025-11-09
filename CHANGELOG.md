# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added
- 自动化 proto 文件管理系统
  - `scripts/init-proto.sh` - 自动拉取 BuildKit 和 googleapis proto 文件
  - `build.rs` 自动检测并初始化 proto 文件
  - proto 目录添加到 .gitignore
- Makefile 提供常用开发命令
  - `make init` - 初始化项目
  - `make proto-init` - 拉取 proto 文件
  - `make proto-clean` - 清理 proto 文件
  - `make help` - 查看所有命令
- GitHub Actions CI 配置
- 详细文档
  - `PROTO_SETUP.md` - Proto 管理说明
  - `QUICKSTART.md` - 快速入门指南
  - `.envrc.example` - 环境变量示例
- 基础测试框架

### Changed
- proto 文件不再提交到 git，改为按需拉取
- README 更新，添加 proto 初始化步骤说明
- build.rs 更新以支持自动化 proto 管理

### Improved
- 仓库体积大幅减小（proto 目录约 30MB）
- 新用户入门更简单（一条命令初始化）
- proto 文件更新更方便
- CI/CD 流程更清晰

## [0.1.0] - 2025-11-09

### Added
- 完整的 BuildKit gRPC 客户端实现
- 支持本地 Dockerfile 和 GitHub 仓库构建
- 实时进度监控（Console/JSON/Silent）
- 多平台构建支持
- Build args、target stage、缓存管理
- Docker registry 认证支持
- Session 协议框架（部分实现）
- CLI 工具（health/local/github 命令）
- Docker Compose 测试环境
- 完整的中文文档

[Unreleased]: https://github.com/yourusername/buildkit-client/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/buildkit-client/releases/tag/v0.1.0
