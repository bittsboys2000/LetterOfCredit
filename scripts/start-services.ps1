# LC Chain - Service Startup Script (Windows PowerShell)
# This script starts all backend services for development

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  LC Chain - Starting Services" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# Check if Docker is running
$dockerRunning = docker info 2>$null
if (-not $dockerRunning) {
    Write-Host "ERROR: Docker is not running. Please start Docker Desktop first." -ForegroundColor Red
    exit 1
}

# Start infrastructure
Write-Host "`n[1/6] Starting infrastructure (PostgreSQL, IPFS, Besu)..." -ForegroundColor Yellow
docker-compose up -d

Write-Host "Waiting for databases to be ready..."
Start-Sleep -Seconds 5

# Set environment variables
$env:DATABASE_URL = "postgres://admin:password@localhost:5432/lc_main"
$env:AUDIT_DATABASE_URL = "postgres://admin:password@localhost:5433/lc_audit"
$env:IPFS_API_URL = "http://localhost:5001"
$env:BESU_RPC_URL = "http://localhost:8545"
$env:CONTRACT_ADDRESS = "0x0000000000000000000000000000000000000000"
$env:JWT_SECRET = "supersecretkey"
$env:CHAIN_ADAPTER_URL = "http://localhost:3004"
$env:AUDIT_SERVICE_URL = "http://localhost:3003"
$env:RUST_LOG = "info"

# Run migrations
Write-Host "`n[2/6] Running database migrations..." -ForegroundColor Yellow
cargo run --package migration-runner

# Start services in background
Write-Host "`n[3/6] Starting Document Service (port 3001)..." -ForegroundColor Yellow
Start-Process -NoNewWindow -FilePath "cargo" -ArgumentList "run --package document"

Write-Host "[4/6] Starting LC Workflow Service (port 3002)..." -ForegroundColor Yellow
Start-Process -NoNewWindow -FilePath "cargo" -ArgumentList "run --package lc-workflow"

Write-Host "[5/6] Starting Audit Service (port 3003)..." -ForegroundColor Yellow
Start-Process -NoNewWindow -FilePath "cargo" -ArgumentList "run --package audit"

Write-Host "[6/6] Starting Chain Adapter Service (port 3004)..." -ForegroundColor Yellow
Start-Process -NoNewWindow -FilePath "cargo" -ArgumentList "run --package chain-adapter"

Write-Host "`n========================================" -ForegroundColor Green
Write-Host "  All services starting!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "Services will be available at:"
Write-Host "  - Document Service:    http://localhost:3001"
Write-Host "  - LC Workflow Service: http://localhost:3002"
Write-Host "  - Audit Service:       http://localhost:3003"
Write-Host "  - Chain Adapter:       http://localhost:3004"
Write-Host "  - User Service:        http://localhost:3005"
Write-Host ""
Write-Host "Open ui/index.html in your browser to access the UI."
Write-Host ""
Write-Host "To stop services: docker-compose down"

