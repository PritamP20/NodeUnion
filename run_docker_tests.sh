#!/bin/bash
# Quick-start script for Docker integration tests
# This script sets up the environment, pulls required images, and runs tests

set -e

cd /Users/pritamp20/Documents/NodeUnion

echo "🐳 NodeUnion Agent - Docker Integration Test Runner"
echo "=================================================="
echo ""

# Check if Docker is running
echo "📋 Checking Docker daemon..."
if ! docker ps > /dev/null 2>&1; then
    echo "❌ Docker daemon is not running!"
    echo ""
    echo "Start Docker:"
    echo "  macOS:  open -a Docker"
    echo "  Linux:  sudo systemctl start docker"
    exit 1
fi
echo "✅ Docker daemon is running"
echo ""

# Pull required test images
echo "📥 Pulling test images (this may take a minute...)..."
IMAGES=("nginx:alpine" "python:3.11-alpine" "alpine:latest")

for image in "${IMAGES[@]}"; do
    echo "  Pulling $image..."
    docker pull "$image" > /dev/null 2>&1 || {
        echo "❌ Failed to pull $image"
        exit 1
    }
    echo "  ✅ $image ready"
done
echo ""

# Compile tests if not already built
echo "🔨 Building tests..."
cargo test -p agent --test docker_integration_test --no-run 2>&1 | grep -E "(Compiling|Finished)" || true
echo "✅ Tests compiled"
echo ""

# Run the tests
echo "🚀 Running Docker integration tests..."
echo "=================================================="
echo ""

cargo test -p agent --test docker_integration_test -- --ignored --nocapture

echo ""
echo "=================================================="
echo "Docker integration tests completed!"
echo ""
echo "📊 Test Summary:"
echo "  • Container starting and stopping"
echo "  • Port binding and app accessibility"
echo "  • Resource limits (CPU/RAM)"
echo "  • Multiple concurrent containers"
echo ""
echo "🔗 Access logs:"
echo "  To check container logs: docker logs <container_id>"
echo "  To list containers: docker ps -a"
echo ""
