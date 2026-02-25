# Letter of Credit (L/C) Digitization Platform

A blockchain-based microservices platform for digitizing Letter of Credit approval workflows, built with Rust, PostgreSQL, IPFS, and Hyperledger Besu.

## 🏗️ Architecture

This system consists of:
- **4 Microservices** (Rust/Axum): Document, LC Workflow, Audit, Chain Adapter
- **2 Databases** (PostgreSQL): Main DB and Audit DB
- **IPFS**: Decentralized document storage
- **Hyperledger Besu**: Blockchain for immutable audit trail
- **Web UI**: Single-page application for user interaction

## 📋 Prerequisites

- **Rust** (1.70+): [Install Rust](https://rustup.rs/)
- **Docker & Docker Compose**: [Install Docker](https://docs.docker.com/get-docker/)
- **Python 3.8+** (for testing): `pip install requests pyjwt`

## 🚀 Quick Start

### 1. Start Infrastructure

Start PostgreSQL, IPFS, and Besu using Docker Compose:

```bash
docker-compose up -d
```

Verify all containers are running:

```bash
docker-compose ps
```

You should see:
- `postgres-main` on port 5432
- `postgres-audit` on port 5433
- `ipfs` on ports 4001, 5001, 8080
- `besu` on ports 8545, 8546

### 2. Set Environment Variables

Create a `.env` file in the project root:

```env
# Main Database
DATABASE_URL=postgres://admin:password@localhost:5432/lc_main

# Audit Database
AUDIT_DATABASE_URL=postgres://admin:password@localhost:5433/lc_audit

# JWT Secret (for authentication)
JWT_SECRET=supersecretkey

# Service Ports
DOCUMENT_SERVICE_PORT=3001
LC_WORKFLOW_SERVICE_PORT=3002
AUDIT_SERVICE_PORT=3003
CHAIN_ADAPTER_SERVICE_PORT=3004

# IPFS
IPFS_API_URL=http://localhost:5001

# Besu RPC
BESU_RPC_URL=http://localhost:8545
```

### 3. Run Database Migrations

```bash
cargo run --bin migration-runner
```

This will create the necessary tables in both databases.

### 4. Build All Services

```bash
cargo build --release
```

### 5. Start Microservices

Open **4 separate terminals** and run each service:

**Terminal 1 - Document Service:**
```bash
cargo run --bin document
```

**Terminal 2 - LC Workflow Service:**
```bash
cargo run --bin lc-workflow
```

**Terminal 3 - Audit Service:**
```bash
cargo run --bin audit
```

**Terminal 4 - Chain Adapter Service:**
```bash
cargo run --bin chain-adapter
```

Each service will start on its configured port:
- Document Service: `http://localhost:3001`
- LC Workflow Service: `http://localhost:3002`
- Audit Service: `http://localhost:3003`
- Chain Adapter Service: `http://localhost:3004`

### 6. Open the Web UI

Open `ui/index.html` in your browser:

```bash
# On Windows
start ui/index.html

# On macOS
open ui/index.html

# On Linux
xdg-open ui/index.html
```

Or simply double-click the file in your file explorer.

## 🧪 Testing

### Automated Test Flow

Run the Python test script to verify the complete workflow:

```bash
python test_flow.py
```

This will:
1. Upload a test document to IPFS
2. Create a Letter of Credit
3. Submit and approve the L/C through the workflow
4. Verify audit logs

### Test Scenarios

Run comprehensive test scenarios:

```bash
python test_scenarios.py
```

## 📖 Using the Web UI

### Role-Based Workflow

The UI supports 5 roles:
- **Importer**: Creates and submits L/C applications
- **Exporter**: Beneficiary of the L/C
- **Issuing Bank**: Reviews and approves/rejects L/Cs
- **Advising Bank**: Notifies the exporter
- **Auditor**: Views audit logs

### Typical Workflow

1. **Switch to Importer role** (bottom left dropdown)
2. **Create New Application**:
   - Click "New Application" in sidebar
   - Upload a document (PDF/image)
   - Fill in application details
   - Submit
3. **Submit for Review**:
   - Go to Dashboard
   - Click "Submit" on the draft L/C
4. **Switch to Issuing Bank role**
5. **Review and Approve**:
   - Click "Start Review"
   - Click "Approve" or "Reject"
6. **Switch to Auditor role**
7. **View Audit Logs**:
   - Click "Audit Logs" in sidebar
   - See complete audit trail

## 🛠️ Development

### Project Structure

```
LetterOfCredit/
├── common/              # Shared libraries (auth, models, DB)
├── services/
│   ├── document/        # Document storage service
│   ├── lc-workflow/     # L/C workflow engine
│   ├── audit/           # Audit logging service
│   ├── chain-adapter/   # Blockchain integration
│   └── migration-runner/# Database migration tool
├── migrations/          # Main DB migrations
├── migrations_audit/    # Audit DB migrations
├── contracts/           # Smart contracts (Solidity)
├── ui/                  # Web interface
├── docker-compose.yml   # Infrastructure setup
└── test_flow.py         # Integration tests
```

### Running Individual Services

```bash
# Document Service
cargo run --bin document

# LC Workflow Service
cargo run --bin lc-workflow

# Audit Service
cargo run --bin audit

# Chain Adapter Service
cargo run --bin chain-adapter
```

### Rebuilding After Changes

```bash
cargo build --release
```

## 🔧 Troubleshooting

### Services won't start

**Check if ports are already in use:**
```bash
# Windows
netstat -ano | findstr "3001 3002 3003 3004"

# Linux/macOS
lsof -i :3001,3002,3003,3004
```

### Database connection errors

**Verify Docker containers are running:**
```bash
docker-compose ps
```

**Check database connectivity:**
```bash
docker exec -it letterofcredit-postgres-main-1 psql -U admin -d lc_main
```

### IPFS not accessible

**Check IPFS container:**
```bash
docker logs letterofcredit-ipfs-1
```

**Test IPFS API:**
```bash
curl http://localhost:5001/api/v0/version
```

### UI can't connect to services

1. Ensure all 4 microservices are running
2. Check browser console for CORS errors
3. Verify service URLs in `ui/index.html` match your configuration

## 📝 API Endpoints

### Document Service (Port 3001)
- `POST /documents` - Upload document to IPFS
- `GET /documents/:id` - Retrieve document metadata

### LC Workflow Service (Port 3002)
- `POST /lc` - Create new L/C
- `GET /lc/:id` - Get L/C details
- `PUT /lc/:id/status` - Update L/C status

### Audit Service (Port 3003)
- `GET /audit` - List all audit logs
- `GET /audit/:id` - Get specific audit log

### Chain Adapter Service (Port 3004)
- `POST /chain/record` - Record event on blockchain
- `GET /chain/verify/:hash` - Verify blockchain record

## 🔐 Security Notes

⚠️ **This is a development/demo setup**:
- JWT secret is hardcoded
- No real authentication system
- CORS is wide open
- Database credentials are default

For production:
- Use proper authentication (OAuth2, OIDC)
- Secure JWT secrets in environment variables
- Configure CORS properly
- Use strong database credentials
- Enable TLS/SSL

## 📄 License

This project is for educational/demonstration purposes.
