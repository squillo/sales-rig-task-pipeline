# Rigger Deployment Guide

This guide covers deploying Rigger in development and production environments.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Building from Source](#building-from-source)
- [Installation Methods](#installation-methods)
- [Configuration](#configuration)
- [Running Rigger](#running-rigger)
- [Production Deployment](#production-deployment)
- [Security](#security)
- [Monitoring](#monitoring)
- [Backup and Recovery](#backup-and-recovery)
- [Troubleshooting](#troubleshooting)

## Prerequisites

### Development

- **Rust**: 1.75.0 or later (install via [rustup](https://rustup.rs/))
- **Cargo**: Comes with Rust
- **SQLite**: 3.35.0 or later (usually pre-installed on macOS/Linux)
- **Ollama** (optional): For local LLM inference

### Production

- **CPU**: 2+ cores recommended
- **RAM**: 4GB minimum, 8GB+ recommended (depending on LLM usage)
- **Disk**: 1GB for application, 10GB+ for models (if running local LLMs)
- **OS**: Linux (Ubuntu 22.04+, Debian 11+, RHEL 8+), macOS 12+

### LLM Requirements

If using local LLMs:
- **Ollama**: https://ollama.ai/download
- **Models**: `llama3.1`, `qwen2.5`, etc. (download via `ollama pull`)

If using cloud LLMs:
- **API keys**: OpenAI, Anthropic, etc.

## Building from Source

### Clone Repository

```bash
git clone https://github.com/anthropics/rig-task-pipeline.git
cd rig-task-pipeline
```

### Build Release Binary

```bash
cargo build --release --package rigger_cli
```

Binary location: `./target/release/rig`

### Run Tests

```bash
# Unit tests
cargo test --workspace

# Integration tests
cargo test --package task_orchestrator --test integration_end_to_end_flow

# gRPC tests (requires gRPC server running)
cargo run --example test_grpc_client
```

### Build Optimizations

For maximum performance:

```bash
# Enable LTO and other optimizations
RUSTFLAGS="-C target-cpu=native" cargo build --release --package rigger_cli
```

## Installation Methods

### Method 1: Cargo Install (Recommended)

```bash
cargo install --path rigger_cli
```

This installs `rig` to `~/.cargo/bin/` (ensure it's in your `PATH`).

Verify installation:
```bash
rig --version
```

### Method 2: Manual Binary Installation

```bash
# Build
cargo build --release --package rigger_cli

# Copy to system location
sudo cp target/release/rig /usr/local/bin/

# Verify
rig --version
```

### Method 3: Docker

```bash
# Build Docker image
docker build -t rigger:latest .

# Run CLI
docker run --rm rigger:latest rig --help

# Run gRPC server
docker run -d \
  -p 50051:50051 \
  -v $(pwd)/.rigger:/data/.rigger \
  --name rigger-grpc \
  rigger:latest rig grpc
```

### Method 4: From GitHub Releases (Future)

```bash
# Download latest release
curl -LO https://github.com/anthropics/rig-task-pipeline/releases/latest/download/rig-linux-amd64

# Make executable
chmod +x rig-linux-amd64

# Move to PATH
sudo mv rig-linux-amd64 /usr/local/bin/rig
```

## Configuration

### Initialize Workspace

Initialize Rigger in your project directory:

```bash
cd /path/to/your/project
rig init
```

This creates:
```
.rigger/
├── tasks.db          # SQLite database
└── config.toml       # Configuration (future)
```

### Environment Variables

Configure Rigger via environment variables:

```bash
# Database path (default: .rigger/tasks.db)
export RIGGER_DB_PATH=/custom/path/tasks.db

# gRPC server port (default: 50051)
export RIGGER_GRPC_PORT=50052

# Default LLM model (default: llama3.1)
export RIGGER_DEFAULT_MODEL=qwen2.5

# Log level (default: info)
export RUST_LOG=debug
```

### LLM Configuration

#### Ollama (Local)

```bash
# Start Ollama
ollama serve

# Pull model
ollama pull llama3.1

# Rigger will automatically use Ollama at http://localhost:11434
```

#### OpenAI (Cloud)

```bash
export OPENAI_API_KEY=sk-...
export RIGGER_LLM_PROVIDER=openai
export RIGGER_DEFAULT_MODEL=gpt-4
```

#### Anthropic (Cloud)

```bash
export ANTHROPIC_API_KEY=sk-ant-...
export RIGGER_LLM_PROVIDER=anthropic
export RIGGER_DEFAULT_MODEL=claude-3-sonnet
```

## Running Rigger

### CLI Mode

**Use case**: Local development, scripting

```bash
# Initialize workspace
rig init

# Add task
rig task add "Implement feature X"

# List tasks
rig task list

# Run orchestration
rig orchestrate run task-abc-123 --model llama3.1
```

### gRPC Server Mode

**Use case**: Distributed systems, sidecars, microservices

```bash
# Start server
rig grpc --port 50051
```

Server runs in foreground. Press `Ctrl+C` to stop.

**Test connectivity**:
```bash
grpcurl -plaintext localhost:50051 list
```

### MCP Server Mode

**Use case**: IDE integration (Claude Code, Cline, etc.)

```bash
# Start MCP server
rig server
```

Server reads from stdin, writes to stdout. Configure in your IDE's MCP settings.

**Example IDE configuration** (Claude Code):
```json
{
  "mcpServers": {
    "rigger": {
      "command": "/usr/local/bin/rig",
      "args": ["server"],
      "cwd": "/path/to/project"
    }
  }
}
```

## Production Deployment

### Systemd Service (Linux)

Create `/etc/systemd/system/rigger-grpc.service`:

```ini
[Unit]
Description=Rigger gRPC Server
After=network.target

[Service]
Type=simple
User=rigger
Group=rigger
WorkingDirectory=/opt/rigger
ExecStart=/usr/local/bin/rig grpc --port 50051
Restart=on-failure
RestartSec=5s

# Environment
Environment="RIGGER_DB_PATH=/var/lib/rigger/tasks.db"
Environment="RUST_LOG=info"

# Security
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/rigger

[Install]
WantedBy=multi-user.target
```

**Setup**:
```bash
# Create user
sudo useradd -r -s /bin/false rigger

# Create directories
sudo mkdir -p /var/lib/rigger
sudo chown rigger:rigger /var/lib/rigger

# Initialize database
sudo -u rigger rig init --db-path /var/lib/rigger/tasks.db

# Enable and start service
sudo systemctl enable rigger-grpc
sudo systemctl start rigger-grpc

# Check status
sudo systemctl status rigger-grpc

# View logs
sudo journalctl -u rigger-grpc -f
```

### Docker Compose

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  rigger-grpc:
    image: rigger:latest
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "50051:50051"
    volumes:
      - rigger-data:/data/.rigger
    environment:
      - RIGGER_DB_PATH=/data/.rigger/tasks.db
      - RUST_LOG=info
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "grpcurl", "-plaintext", "localhost:50051", "list"]
      interval: 30s
      timeout: 10s
      retries: 3

  # Example sidecar: Logger
  rigger-logger-sidecar:
    image: rigger:latest
    command: ["cargo", "run", "--example", "sidecar_client"]
    depends_on:
      - rigger-grpc
    environment:
      - RIGGER_GRPC_ADDR=rigger-grpc:50051
    restart: unless-stopped

volumes:
  rigger-data:
```

**Deploy**:
```bash
docker-compose up -d
docker-compose logs -f rigger-grpc
```

### Kubernetes

Create `rigger-deployment.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rigger-grpc
  namespace: rigger
spec:
  replicas: 3
  selector:
    matchLabels:
      app: rigger-grpc
  template:
    metadata:
      labels:
        app: rigger-grpc
    spec:
      containers:
      - name: rigger
        image: rigger:latest
        ports:
        - containerPort: 50051
          name: grpc
        env:
        - name: RIGGER_DB_PATH
          value: /data/tasks.db
        - name: RUST_LOG
          value: info
        volumeMounts:
        - name: data
          mountPath: /data
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
        livenessProbe:
          exec:
            command:
            - grpcurl
            - -plaintext
            - localhost:50051
            - list
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          exec:
            command:
            - grpcurl
            - -plaintext
            - localhost:50051
            - list
          initialDelaySeconds: 5
          periodSeconds: 10
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: rigger-data-pvc
---
apiVersion: v1
kind: Service
metadata:
  name: rigger-grpc-service
  namespace: rigger
spec:
  type: LoadBalancer
  selector:
    app: rigger-grpc
  ports:
  - port: 50051
    targetPort: 50051
    protocol: TCP
    name: grpc
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: rigger-data-pvc
  namespace: rigger
spec:
  accessModes:
  - ReadWriteOnce
  resources:
    requests:
      storage: 10Gi
```

**Deploy**:
```bash
kubectl create namespace rigger
kubectl apply -f rigger-deployment.yaml
kubectl get pods -n rigger
kubectl logs -n rigger -l app=rigger-grpc -f
```

### Load Balancing

For multiple gRPC server instances:

**Nginx gRPC load balancer** (`nginx.conf`):

```nginx
http {
    upstream rigger_grpc {
        server rigger-1:50051;
        server rigger-2:50051;
        server rigger-3:50051;
    }

    server {
        listen 50051 http2;

        location / {
            grpc_pass grpc://rigger_grpc;
        }
    }
}
```

**Note**: Event broadcasting only reaches subscribers on the same instance. For distributed broadcasting, use Redis Pub/Sub or NATS.

## Security

### TLS/SSL for gRPC

#### Generate Certificates

```bash
# Generate CA key and certificate
openssl genrsa -out ca.key 4096
openssl req -new -x509 -days 365 -key ca.key -out ca.crt -subj "/CN=Rigger CA"

# Generate server key and CSR
openssl genrsa -out server.key 4096
openssl req -new -key server.key -out server.csr -subj "/CN=rigger.example.com"

# Sign server certificate
openssl x509 -req -days 365 -in server.csr -CA ca.crt -CAkey ca.key -set_serial 01 -out server.crt

# Generate client key and certificate (for mTLS)
openssl genrsa -out client.key 4096
openssl req -new -key client.key -out client.csr -subj "/CN=rigger-client"
openssl x509 -req -days 365 -in client.csr -CA ca.crt -CAkey ca.key -set_serial 02 -out client.crt
```

#### Server Configuration (Rust)

Update `rigger_cli/src/commands/grpc_server.rs`:

```rust
use tonic::transport::{Server, ServerTlsConfig, Identity};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cert = tokio::fs::read("server.crt").await?;
    let key = tokio::fs::read("server.key").await?;
    let server_identity = Identity::from_pem(cert, key);

    let tls_config = ServerTlsConfig::new()
        .identity(server_identity);

    let addr = "[::1]:50051".parse()?;
    let service = RiggerServiceImpl::new(db_path);

    Server::builder()
        .tls_config(tls_config)?
        .add_service(RiggerServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
```

#### Client Configuration

```bash
# grpcurl with TLS
grpcurl -cacert ca.crt -d '{"title":"Task"}' rigger.example.com:50051 rigger.v1.RiggerService/AddTask

# Rust client
let tls_config = ClientTlsConfig::new()
    .ca_certificate(Certificate::from_pem(ca_cert))
    .domain_name("rigger.example.com");

let channel = Channel::from_static("https://rigger.example.com:50051")
    .tls_config(tls_config)?
    .connect()
    .await?;

let client = RiggerServiceClient::new(channel);
```

### Authentication

#### API Key Authentication

Implement gRPC interceptor:

```rust
use tonic::{Request, Status};
use tonic::service::Interceptor;

#[derive(Clone)]
struct AuthInterceptor {
    api_key: String,
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        let token = request
            .metadata()
            .get("authorization")
            .ok_or_else(|| Status::unauthenticated("No token provided"))?
            .to_str()
            .map_err(|_| Status::unauthenticated("Invalid token format"))?;

        if token != format!("Bearer {}", self.api_key) {
            return Err(Status::unauthenticated("Invalid token"));
        }

        Ok(request)
    }
}

// Apply to server
Server::builder()
    .add_service(RiggerServiceServer::with_interceptor(service, AuthInterceptor { api_key }))
    .serve(addr)
    .await?;
```

**Client**:
```bash
grpcurl -H "Authorization: Bearer secret-api-key" -plaintext localhost:50051 rigger.v1.RiggerService/ListTasks
```

### Firewall Rules

```bash
# Allow gRPC port
sudo ufw allow 50051/tcp

# Restrict to specific IPs
sudo ufw allow from 10.0.0.0/24 to any port 50051
```

## Monitoring

### Logging

Configure structured logging:

```bash
# Set log level
export RUST_LOG=rigger_cli=debug,task_orchestrator=info,task_manager=warn

# Run with logging
rig grpc 2>&1 | tee /var/log/rigger/grpc.log
```

### Metrics (Future)

Prometheus metrics endpoint:

```rust
// Add to gRPC server
use prometheus::{Encoder, TextEncoder};

let metrics_addr = "[::]:9090".parse()?;
let metrics_server = warp::serve(
    warp::path!("metrics").map(|| {
        let encoder = TextEncoder::new();
        let metric_families = prometheus::gather();
        let mut buffer = vec![];
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    })
);

tokio::spawn(metrics_server.run(metrics_addr));
```

**Prometheus config** (`prometheus.yml`):
```yaml
scrape_configs:
  - job_name: 'rigger'
    static_configs:
      - targets: ['localhost:9090']
```

### Health Checks

```bash
# gRPC health check
grpcurl -plaintext localhost:50051 list

# HTTP health endpoint (add to server)
curl http://localhost:8080/health
```

### Distributed Tracing (Future)

OpenTelemetry integration:

```rust
use opentelemetry::global;
use tracing_subscriber::layer::SubscriberExt;

global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
let tracer = opentelemetry_jaeger::new_pipeline()
    .with_service_name("rigger-grpc")
    .install_simple()?;

let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
tracing_subscriber::registry()
    .with(telemetry)
    .init();
```

## Backup and Recovery

### Database Backup

SQLite backup:

```bash
# Hot backup (while server running)
sqlite3 .rigger/tasks.db ".backup .rigger/tasks.db.backup"

# Scheduled backup (cron)
0 2 * * * sqlite3 /var/lib/rigger/tasks.db ".backup /backups/rigger-$(date +\%Y\%m\%d).db"
```

### Restore

```bash
# Stop server
sudo systemctl stop rigger-grpc

# Restore database
cp /backups/rigger-20251123.db /var/lib/rigger/tasks.db

# Start server
sudo systemctl start rigger-grpc
```

### Database Migration

When upgrading Rigger versions:

```bash
# Backup current database
sqlite3 .rigger/tasks.db ".backup .rigger/tasks.db.pre-upgrade"

# Run migration (future)
rig migrate --from v0.1.0 --to v0.2.0

# Verify migration
rig task list
```

## Troubleshooting

### Server Won't Start

**Problem**: `Address already in use`

**Solution**:
```bash
# Find process using port
lsof -i :50051

# Kill process
kill <PID>

# Or use different port
rig grpc --port 50052
```

---

**Problem**: `Database locked`

**Solution**:
```bash
# Check for other rig processes
ps aux | grep rig

# Kill hanging processes
killall rig

# Restart server
rig grpc
```

---

### Connection Refused

**Problem**: Client can't connect to gRPC server

**Solution**:
```bash
# Verify server is running
ps aux | grep rig

# Check port binding
lsof -i :50051

# Test connectivity
grpcurl -plaintext localhost:50051 list

# Check firewall
sudo ufw status
```

---

### High Memory Usage

**Problem**: Server using excessive memory

**Causes**:
- Large broadcast buffer (1000 events * event size)
- Many concurrent LLM calls
- SQLite connection pool

**Solutions**:
```bash
# Reduce broadcast buffer (edit grpc_server.rs)
let (event_tx, _) = tokio::sync::broadcast::channel(100);

# Limit concurrent LLM calls
# Set environment variable
export RIGGER_MAX_CONCURRENT_LLM=5

# Monitor memory
top -p $(pgrep rig)
```

---

### LLM Calls Failing

**Problem**: Orchestration fails with LLM errors

**Solution**:
```bash
# Check Ollama is running
curl http://localhost:11434/api/version

# Verify model is downloaded
ollama list

# Pull model if missing
ollama pull llama3.1

# Test LLM directly
ollama run llama3.1 "Hello"

# Check Rigger logs
export RUST_LOG=debug
rig orchestrate run task-abc-123
```

---

### Database Corruption

**Problem**: `database disk image is malformed`

**Solution**:
```bash
# Attempt recovery
sqlite3 .rigger/tasks.db ".recover" | sqlite3 .rigger/tasks-recovered.db

# Verify recovered database
sqlite3 .rigger/tasks-recovered.db "SELECT COUNT(*) FROM tasks;"

# Replace corrupted database
mv .rigger/tasks.db .rigger/tasks.db.corrupted
mv .rigger/tasks-recovered.db .rigger/tasks.db

# If recovery fails, restore from backup
cp /backups/rigger-latest.db .rigger/tasks.db
```

---

### Slow Queries

**Problem**: Database operations are slow

**Solution**:
```bash
# Add indexes (future migrations)
sqlite3 .rigger/tasks.db "CREATE INDEX idx_assignee ON tasks(assignee);"
sqlite3 .rigger/tasks.db "CREATE INDEX idx_status ON tasks(status);"

# Analyze query performance
sqlite3 .rigger/tasks.db "EXPLAIN QUERY PLAN SELECT * FROM tasks WHERE status = 2;"

# Vacuum database
sqlite3 .rigger/tasks.db "VACUUM;"
```

---

## Upgrading

### CLI Upgrade

```bash
# Via cargo
cargo install --path rigger_cli --force

# Verify version
rig --version
```

### Docker Upgrade

```bash
# Pull latest image
docker pull rigger:latest

# Restart containers
docker-compose down
docker-compose up -d
```

### Kubernetes Upgrade

```bash
# Update image tag
kubectl set image deployment/rigger-grpc rigger=rigger:v0.2.0 -n rigger

# Rolling update
kubectl rollout status deployment/rigger-grpc -n rigger

# Rollback if needed
kubectl rollout undo deployment/rigger-grpc -n rigger
```

---

## Performance Tuning

### SQLite Optimizations

Add to database initialization:

```sql
PRAGMA journal_mode = WAL;           -- Write-Ahead Logging
PRAGMA synchronous = NORMAL;         -- Faster writes
PRAGMA cache_size = -64000;          -- 64MB cache
PRAGMA temp_store = MEMORY;          -- Use memory for temp tables
```

### gRPC Tuning

```rust
// Increase connection limits
Server::builder()
    .concurrency_limit_per_connection(256)
    .tcp_nodelay(true)
    .tcp_keepalive(Some(Duration::from_secs(60)))
    .add_service(service)
    .serve(addr)
    .await?;
```

### LLM Batching

For high-throughput scenarios, batch LLM calls:

```rust
// Process tasks in batches
let tasks: Vec<Task> = /* ... */;
let chunks: Vec<_> = tasks.chunks(10).collect();

for chunk in chunks {
    let futures: Vec<_> = chunk.iter()
        .map(|task| enhance_task(task))
        .collect();

    let results = futures::future::join_all(futures).await;
}
```

---

## Best Practices

1. **Always run `rig init`** in new project directories
2. **Use systemd** for production Linux deployments
3. **Enable TLS** for gRPC in production
4. **Backup database daily** with automated scripts
5. **Monitor logs** for errors and performance issues
6. **Use load balancers** for horizontal scaling
7. **Set resource limits** in Docker/Kubernetes
8. **Version pin** dependencies in production
9. **Test upgrades** in staging before production
10. **Document configuration** changes

---

**Last updated**: 2025-11-23
