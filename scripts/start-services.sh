#!/bin/bash
# LC Chain - Service Startup Script (Unix/Linux/macOS)
# This script starts all backend services for development

echo "========================================"
echo "  LC Chain - Starting Services"
echo "========================================"

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "ERROR: Docker is not running. Please start Docker first."
    exit 1
fi

# Start infrastructure
echo ""
echo "[1/6] Starting infrastructure (PostgreSQL, IPFS, Besu)..."
docker-compose up -d

echo "Waiting for databases to be ready..."
sleep 5

# Set environment variables
export DATABASE_URL="postgres://admin:password@localhost:5432/lc_main"
export AUDIT_DATABASE_URL="postgres://admin:password@localhost:5433/lc_audit"
export IPFS_API_URL="http://localhost:5001"
export BESU_RPC_URL="http://localhost:8545"
export CONTRACT_ADDRESS="0x0000000000000000000000000000000000000000"
export JWT_SECRET="supersecretkey"
export CHAIN_ADAPTER_URL="http://localhost:3004"
export AUDIT_SERVICE_URL="http://localhost:3003"
export RUST_LOG="info"

# Run migrations
echo ""
echo "[2/6] Running database migrations..."
cargo run --package migration-runner

# Start services in background
echo ""
echo "[3/6] Starting Document Service (port 3001)..."
cargo run --package document &

echo "[4/6] Starting LC Workflow Service (port 3002)..."
cargo run --package lc-workflow &

echo "[5/6] Starting Audit Service (port 3003)..."
cargo run --package audit &

echo "[6/6] Starting Chain Adapter Service (port 3004)..."
cargo run --package chain-adapter &

# Optional: Start User Service
echo "[7/7] Starting User Service (port 3005)..."
cargo run --package user &

echo ""
echo "========================================"
echo "  All services starting!"
echo "========================================"
echo ""
echo "Services will be available at:"
echo "  - Document Service:    http://localhost:3001"
echo "  - LC Workflow Service: http://localhost:3002"
echo "  - Audit Service:       http://localhost:3003"
echo "  - Chain Adapter:       http://localhost:3004"
echo "  - User Service:        http://localhost:3005"
echo ""
echo "Open ui/index.html in your browser to access the UI."
echo ""
echo "To stop services: docker-compose down && pkill -f 'cargo run'"
echo ""

# Wait for all background jobs
wait

