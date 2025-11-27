# Rigger gRPC Server Setup Guide

This guide explains how to set up and use the Rigger gRPC server for distributed task management with sidecar integration and real-time event broadcasting.

## Overview

The Rigger gRPC server provides a modern, high-performance alternative to the stdio JSON-RPC MCP server, designed specifically for distributed architectures where multiple services (sidecars) need to observe and react to task state changes.

### Why gRPC?

- **Broadcasting**: Publish task events to multiple sidecar subscribers simultaneously
- **Type Safety**: Strongly-typed protobuf schemas prevent runtime errors
- **Performance**: HTTP/2 multiplexing and binary encoding for efficiency
- **Bidirectional Streaming**: Real-time updates without polling
- **Language Agnostic**: Generate clients in any language (Go, Python, Java, etc.)

## Protocol Details

- **Transport**: gRPC over HTTP/2
- **Port**: `50051` (default, configurable)
- **Schema**: Protocol Buffers (`rigger_cli/proto/rigger.proto`)
- **Encoding**: Binary protobuf (smaller than JSON)

## Quick Start

### 1. Start the gRPC Server

```bash
cd /path/to/your/project
rig init  # Initialize .rigger directory if not done
rig grpc  # Start gRPC server on port 50051
```

You should see:
```
🚀 Rigger gRPC Server starting...
   Protocol: gRPC over HTTP/2
   Address: [::1]:50051
   Database: /path/to/project/.rigger/tasks.db
   Broadcast: Enabled (1000 event buffer)
```

### 2. Run the Sidecar Client Example

In another terminal:

```bash
cargo run --example sidecar_client
```

This starts a sidecar that subscribes to all task events and prints them to stdout.

### 3. Trigger Some Events

In a third terminal, interact with the gRPC server using `grpcurl` (install with `brew install grpcurl`):

```bash
# Add a task
grpcurl -plaintext -d '{
  "title": "Implement OAuth2 authentication",
  "assignee": "Alice",
  "due_date": "2025-12-31"
}' localhost:50051 rigger.v1.RiggerService/AddTask

# List all tasks
grpcurl -plaintext localhost:50051 rigger.v1.RiggerService/ListTasks

# Update task status
grpcurl -plaintext -d '{
  "task_id": "<paste-task-id-here>",
  "status": 2
}' localhost:50051 rigger.v1.RiggerService/UpdateTask
```

You should see events appear in the sidecar client terminal!

## gRPC RPCs

The Rigger gRPC service exposes the following RPCs:

### Task Management

#### `ListTasks`
List tasks with optional filters.

```bash
grpcurl -plaintext -d '{
  "status": 1,       # 1 = Todo, 2 = InProgress, 9 = Completed
  "assignee": "Alice",
  "limit": 10,
  "offset": 0
}' localhost:50051 rigger.v1.RiggerService/ListTasks
```

#### `AddTask`
Create a new task.

```bash
grpcurl -plaintext -d '{
  "title": "Implement feature X",
  "assignee": "Bob",
  "due_date": "2025-12-31",
  "source_prd_id": "prd-123"
}' localhost:50051 rigger.v1.RiggerService/AddTask
```

#### `UpdateTask`
Update an existing task.

```bash
grpcurl -plaintext -d '{
  "task_id": "abc-123",
  "status": 2,  # InProgress
  "assignee": "Charlie"
}' localhost:50051 rigger.v1.RiggerService/UpdateTask
```

#### `GetTask`
Retrieve a single task by ID.

```bash
grpcurl -plaintext -d '{
  "task_id": "abc-123"
}' localhost:50051 rigger.v1.RiggerService/GetTask
```

#### `DeleteTask`
Delete (archive) a task.

```bash
grpcurl -plaintext -d '{
  "task_id": "abc-123"
}' localhost:50051 rigger.v1.RiggerService/DeleteTask
```

### PRD Operations

#### `ParsePRD`
Parse a PRD markdown file.

```bash
grpcurl -plaintext -d '{
  "prd_file_path": "./docs/PRD.md"
}' localhost:50051 rigger.v1.RiggerService/ParsePRD
```

#### `GenerateTasksFromPRD` (TODO)
Generate tasks from a parsed PRD using LLM decomposition.

```bash
grpcurl -plaintext -d '{
  "prd_id": "prd-123",
  "model": "llama3.1"
}' localhost:50051 rigger.v1.RiggerService/GenerateTasksFromPRD
```

### Orchestration

#### `OrchestrateTask` (TODO)
Run a task through the orchestration flow (enhancement + comprehension test).

```bash
grpcurl -plaintext -d '{
  "task_id": "abc-123",
  "model": "llama3.1",
  "test_type": "short_answer"
}' localhost:50051 rigger.v1.RiggerService/OrchestrateTask
```

### Event Streaming

#### `SubscribeToTaskEvents`
Subscribe to a stream of task events for broadcast to sidecars.

```bash
grpcurl -plaintext -d '{
  "event_types": [1, 2, 3],  # 1=CREATED, 2=UPDATED, 3=DELETED
  "assignee_filter": "Alice"
}' localhost:50051 rigger.v1.RiggerService/SubscribeToTaskEvents
```

This returns a stream of `TaskEvent` messages that continues until the connection is closed.

## Task Status Enum

| Value | Status | Description |
|-------|--------|-------------|
| 0 | UNSPECIFIED | Default (should not be used) |
| 1 | TODO | Task is pending |
| 2 | IN_PROGRESS | Task is being worked on |
| 3 | PENDING_ENHANCEMENT | Waiting for LLM enhancement |
| 4 | PENDING_COMPREHENSION_TEST | Waiting for test generation |
| 5 | PENDING_FOLLOW_ON | Waiting for follow-on tasks |
| 6 | PENDING_DECOMPOSITION | Waiting for task decomposition |
| 7 | DECOMPOSED | Task has been decomposed into subtasks |
| 8 | ORCHESTRATION_COMPLETE | Orchestration flow complete |
| 9 | COMPLETED | Task is done |
| 10 | ARCHIVED | Task is archived (soft delete) |

## Task Event Types

| Value | Event Type | Description |
|-------|------------|-------------|
| 1 | CREATED | New task created |
| 2 | UPDATED | Task fields updated |
| 3 | DELETED | Task archived |
| 4 | STATUS_CHANGED | Task status changed |
| 5 | ASSIGNED | Task assigned to someone |
| 6 | DECOMPOSED | Task decomposed into subtasks |
| 7 | ORCHESTRATED | Task orchestration complete |

## Building a Sidecar Client

### Rust Example

See `rigger_cli/examples/sidecar_client.rs` for a complete working example.

```rust
use rigger_cli::commands::grpc_server::rigger::v1::*;
use rigger_cli::commands::grpc_server::rigger::v1::rigger_service_client::RiggerServiceClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to server
    let mut client = RiggerServiceClient::connect("http://[::1]:50051").await?;

    // Subscribe to events
    let request = tonic::Request::new(SubscribeToTaskEventsRequest {
        event_types: vec![],  // All events
        assignee_filter: None,
    });

    let mut stream = client.subscribe_to_task_events(request).await?.into_inner();

    // Process events
    while let Some(event) = stream.message().await? {
        println!("Event: {:?}", event);
    }

    Ok(())
}
```

### Python Example (using `grpcio`)

```python
import grpc
from rigger_pb2 import SubscribeToTaskEventsRequest
from rigger_pb2_grpc import RiggerServiceStub

def main():
    # Connect to server
    channel = grpc.insecure_channel('[::1]:50051')
    stub = RiggerServiceStub(channel)

    # Subscribe to events
    request = SubscribeToTaskEventsRequest()

    # Stream events
    for event in stub.SubscribeToTaskEvents(request):
        print(f"Event: {event.event_type} - Task: {event.task.title}")

if __name__ == '__main__':
    main()
```

### Go Example (using `grpc-go`)

```go
package main

import (
    "context"
    "io"
    "log"

    "google.golang.org/grpc"
    pb "path/to/rigger/pb"
)

func main() {
    conn, err := grpc.Dial("[::1]:50051", grpc.WithInsecure())
    if err != nil {
        log.Fatal(err)
    }
    defer conn.Close()

    client := pb.NewRiggerServiceClient(conn)
    stream, err := client.SubscribeToTaskEvents(context.Background(), &pb.SubscribeToTaskEventsRequest{})
    if err != nil {
        log.Fatal(err)
    }

    for {
        event, err := stream.Recv()
        if err == io.EOF {
            break
        }
        if err != nil {
            log.Fatal(err)
        }
        log.Printf("Event: %s - Task: %s\n", event.EventType, event.Task.Title)
    }
}
```

## Protobuf Schema

The protobuf schema is defined in `rigger_cli/proto/rigger.proto`. To generate client code for other languages:

```bash
# Python
python -m grpc_tools.protoc -I./proto --python_out=. --grpc_python_out=. rigger.proto

# Go
protoc -I./proto --go_out=. --go-grpc_out=. rigger.proto

# JavaScript/TypeScript
npm install -g grpc-tools
grpc_tools_node_protoc -I./proto --js_out=import_style=commonjs:. --grpc_out=. rigger.proto
```

## Architecture Patterns

### Sidecar Use Cases

1. **Logging & Auditing**: Subscribe to all task events and log to external system
2. **Notifications**: Send Slack/email alerts when tasks are created/updated
3. **Metrics & Monitoring**: Track task completion rates, cycle times
4. **Workflow Automation**: Trigger downstream actions when tasks reach certain states
5. **Data Synchronization**: Mirror task state to another system (JIRA, Linear, etc.)

### Multi-Sidecar Example

```
┌─────────────────┐
│  Rigger gRPC    │
│     Server      │
│  (Broadcaster)  │
└────────┬────────┘
         │
         ├──────────┐
         │          │
    ┌────▼───┐  ┌──▼──────┐
    │ Logger │  │ Metrics │
    │Sidecar │  │ Sidecar │
    └────────┘  └─────────┘
```

Each sidecar independently subscribes to the task event stream and processes events according to its purpose.

## Troubleshooting

### Server Won't Start

**Error**: `Address already in use`

**Solution**: Another process is using port 50051. Either stop that process or change the port:
```bash
rig grpc --port 50052
```

### Sidecar Can't Connect

**Error**: `Connection refused`

**Solution**: Ensure the gRPC server is running on `[::1]:50051`. Check with:
```bash
lsof -i :50051
```

### Events Not Arriving

**Problem**: Sidecar connected but no events received.

**Solution**:
1. Verify the server has the broadcast channel enabled (check startup logs)
2. Ensure you're triggering events (add/update tasks)
3. Check that your event filter isn't excluding all events

### Protobuf Version Mismatch

**Error**: `protobuf version mismatch`

**Solution**: Regenerate client code with the same protobuf version:
```bash
cargo clean
cargo build --package rigger_cli
```

## Performance Tuning

### Broadcast Buffer Size

The broadcast channel has a default buffer of 1000 events. If you have high-throughput scenarios, increase it in `grpc_server.rs`:

```rust
let (event_tx, _) = tokio::sync::broadcast::channel(10000); // Increase buffer
```

### Connection Pooling

For high-traffic sidecars, use connection pooling:

```rust
let channel = tonic::transport::Channel::from_static("http://[::1]:50051")
    .connect()
    .await?;

let client = RiggerServiceClient::new(channel);
```

## Security

### TLS (Production)

For production deployments, enable TLS:

```rust
// Server-side
use tonic::transport::ServerTlsConfig;

let tls_config = ServerTlsConfig::new()
    .identity(Identity::from_pem(cert_pem, key_pem));

Server::builder()
    .tls_config(tls_config)?
    .add_service(RiggerServiceServer::new(service))
    .serve(addr)
    .await?;
```

```rust
// Client-side
let tls_config = ClientTlsConfig::new();
let channel = Channel::from_static("https://[::1]:50051")
    .tls_config(tls_config)?
    .connect()
    .await?;
```

### Authentication

Add middleware for authentication:

```rust
use tonic::service::Interceptor;

#[derive(Clone)]
struct AuthInterceptor;

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        // Validate auth token
        let token = request.metadata().get("authorization")
            .ok_or_else(|| Status::unauthenticated("No token"))?;
        // ... verify token ...
        Ok(request)
    }
}

// Apply interceptor
Server::builder()
    .add_service(RiggerServiceServer::with_interceptor(service, AuthInterceptor))
    .serve(addr)
    .await?;
```

## Revision History

- 2025-11-23: Initial gRPC setup guide for distributed sidecar integration
