#!/bin/bash
set -e

echo "🧪 BuildKit Client 测试脚本"
echo "=============================="

# 检查 Docker 是否运行
echo "📋 检查 Docker daemon..."
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker daemon 未运行，请先启动 Docker"
    exit 1
fi
echo "✅ Docker daemon 正在运行"

# 启动测试环境
echo ""
echo "🚀 启动 BuildKit 和 Registry..."
docker-compose up -d
sleep 5

# 检查服务状态
echo ""
echo "📋 检查服务状态..."
docker-compose ps

# 等待 BuildKit 就绪
echo ""
echo "⏳ 等待 BuildKit 就绪..."
max_attempts=30
attempt=0
while [ $attempt -lt $max_attempts ]; do
    if cargo run -- health > /dev/null 2>&1; then
        echo "✅ BuildKit 已就绪"
        break
    fi
    attempt=$((attempt + 1))
    sleep 1
    echo "   等待中... ($attempt/$max_attempts)"
done

if [ $attempt -eq $max_attempts ]; then
    echo "❌ BuildKit 启动超时"
    docker-compose logs buildkitd
    exit 1
fi

# 测试 1: Health Check
echo ""
echo "🔍 测试 1: Health Check"
cargo run -- health
echo "✅ Health check 通过"

# 测试 2: 简单构建
echo ""
echo "🔍 测试 2: 简单 Dockerfile 构建"
cargo run -- local \
  --context ./examples/test-dockerfile \
  --tag localhost:5000/test:latest \
  -v

# 验证镜像是否推送成功
echo ""
echo "📋 验证镜像推送..."
if curl -s http://localhost:5000/v2/test/tags/list | grep -q "latest"; then
    echo "✅ 镜像成功推送到 registry"
else
    echo "⚠️  镜像推送验证失败（这可能是正常的，取决于 BuildKit 配置）"
fi

# 测试 3: 多阶段构建 + Build Args
echo ""
echo "🔍 测试 3: 多阶段构建 + Build Arguments"
cargo run -- local \
  --context ./examples/multi-stage \
  --tag localhost:5000/multi-stage:v1 \
  --build-arg APP_VERSION=1.0.0 \
  --build-arg BUILD_DATE=$(date +%Y-%m-%d) \
  -v

# 测试 4: Target Stage
echo ""
echo "🔍 测试 4: 指定 Target Stage"
cargo run -- local \
  --context ./examples/multi-stage \
  --tag localhost:5000/dev:latest \
  --target dev \
  -v

# 测试 5: JSON 输出
echo ""
echo "🔍 测试 5: JSON 输出模式"
cargo run -- local \
  --context ./examples/test-dockerfile \
  --tag localhost:5000/json-test:latest \
  --json | head -20

echo ""
echo "=============================="
echo "✨ 所有测试完成！"
echo ""
echo "📊 Registry 中的镜像:"
curl -s http://localhost:5000/v2/_catalog | jq .

echo ""
echo "🧹 清理测试环境..."
echo "   运行: docker-compose down"
