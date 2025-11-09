#!/bin/bash
set -e

echo "Initializing proto files..."

# Create proto directory if it doesn't exist
mkdir -p proto

# Clone buildkit proto files
if [ ! -d "proto/.buildkit" ]; then
    echo "Cloning buildkit proto files..."
    git clone --depth 1 https://github.com/moby/buildkit.git proto/.buildkit
fi

# Copy necessary proto files with correct structure
echo "Copying proto files..."
mkdir -p proto/github.com/moby/buildkit
mkdir -p proto/github.com/moby/buildkit/session
mkdir -p proto/github.com/tonistiigi/fsutil/types
mkdir -p proto/github.com/planetscale/vtprotobuf/vtproto

cp -r proto/.buildkit/api proto/github.com/moby/buildkit/ 2>/dev/null || true
cp -r proto/.buildkit/solver proto/github.com/moby/buildkit/ 2>/dev/null || true
cp -r proto/.buildkit/sourcepolicy proto/github.com/moby/buildkit/ 2>/dev/null || true
cp -r proto/.buildkit/frontend proto/github.com/moby/buildkit/ 2>/dev/null || true

# Copy session proto files
cp -r proto/.buildkit/session/filesync proto/github.com/moby/buildkit/session/ 2>/dev/null || true
cp -r proto/.buildkit/session/auth proto/github.com/moby/buildkit/session/ 2>/dev/null || true
cp -r proto/.buildkit/session/secrets proto/github.com/moby/buildkit/session/ 2>/dev/null || true
cp -r proto/.buildkit/session/sshforward proto/github.com/moby/buildkit/session/ 2>/dev/null || true

# Copy fsutil proto files
cp proto/.buildkit/vendor/github.com/tonistiigi/fsutil/types/*.proto proto/github.com/tonistiigi/fsutil/types/ 2>/dev/null || true

# Copy vtprotobuf proto files (needed by fsutil)
if [ -f "proto/.buildkit/vendor/github.com/planetscale/vtprotobuf/vtproto/ext.proto" ]; then
    cp proto/.buildkit/vendor/github.com/planetscale/vtprotobuf/vtproto/ext.proto proto/github.com/planetscale/vtprotobuf/vtproto/ 2>/dev/null || true
else
    # Create a stub ext.proto if it doesn't exist
    cat > proto/github.com/planetscale/vtprotobuf/vtproto/ext.proto << 'EOF'
syntax = "proto3";
package vtproto;
option go_package = "github.com/planetscale/vtprotobuf/vtproto";
import "google/protobuf/descriptor.proto";
extend google.protobuf.MessageOptions {
  bool mempool = 65001;
}
EOF
fi

# Clone googleapis for google.rpc
if [ ! -d "proto/.googleapis" ]; then
    echo "Cloning googleapis..."
    git clone --depth 1 https://github.com/googleapis/googleapis.git proto/.googleapis
fi

mkdir -p proto/google/rpc
cp proto/.googleapis/google/rpc/*.proto proto/google/rpc/ 2>/dev/null || true

echo "Proto files initialized successfully!"
echo "You can now run: cargo build"
