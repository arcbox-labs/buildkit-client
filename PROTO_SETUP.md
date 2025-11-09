# Proto 文件管理

本项目使用自动化脚本管理 protobuf 文件，无需手动复制。

## 快速开始

### 首次使用

运行初始化脚本自动拉取 proto 文件：

```bash
./scripts/init-proto.sh
```

该脚本会：
1. 从 [moby/buildkit](https://github.com/moby/buildkit) 仓库拉取最新的 proto 文件
2. 从 [googleapis](https://github.com/googleapis/googleapis) 拉取依赖的 google/rpc proto 文件
3. 将文件复制到 `proto/` 目录

### 构建项目

初始化 proto 文件后，直接构建：

```bash
cargo build
```

`build.rs` 会自动检测 proto 文件是否存在，如果不存在会尝试运行初始化脚本。

## 更新 Proto 文件

如果需要更新到最新的 BuildKit proto 定义：

```bash
# 删除缓存的 git 克隆
rm -rf proto/.buildkit proto/.googleapis

# 重新运行初始化脚本
./scripts/init-proto.sh
```

## Git 忽略

`proto/` 目录已添加到 `.gitignore`，因为：
- Proto 文件通过脚本自动管理
- 减小仓库大小
- 始终使用上游最新定义

临时的 git 克隆目录也被忽略：
- `proto/.buildkit/`
- `proto/.googleapis/`

## 手动管理（可选）

如果你希望手动管理 proto 文件而不使用脚本：

1. 从 https://github.com/moby/buildkit 克隆并复制以下目录到 `proto/`：
   - `api/`
   - `solver/`
   - `sourcepolicy/`
   - `frontend/`

2. 从 https://github.com/googleapis/googleapis 复制：
   - `google/rpc/*.proto` 到 `proto/google/rpc/`

3. 运行 `cargo build`

## 故障排除

### Proto 文件未找到

如果构建时提示找不到 proto 文件：

```bash
./scripts/init-proto.sh
cargo clean
cargo build
```

### 权限问题

如果脚本无法执行：

```bash
chmod +x scripts/init-proto.sh
./scripts/init-proto.sh
```

### 网络问题

如果 git clone 失败（网络问题），可以：
1. 手动下载 [buildkit](https://github.com/moby/buildkit/archive/refs/heads/master.zip)
2. 解压并复制相应的 proto 文件到 `proto/` 目录
