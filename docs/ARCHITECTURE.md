# Rigger Architecture Guide

This document provides a comprehensive overview of the Rigger system architecture, explaining the design decisions, patterns, and structure that enable AI-powered task management with distributed orchestration.

## Table of Contents

- [Architecture Overview](#architecture-overview)
- [Hexagonal Architecture](#hexagonal-architecture)
- [Workspace Structure](#workspace-structure)
- [Layer-by-Layer Breakdown](#layer-by-layer-breakdown)
- [Data Flow](#data-flow)
- [Integration Modes](#integration-modes)
- [Event Broadcasting](#event-broadcasting)
- [Orchestration Pipeline](#orchestration-pipeline)
- [Design Patterns](#design-patterns)
- [Module Dependencies](#module-dependencies)

## Architecture Overview

Rigger is built using **Hexagonal Architecture** (also known as Ports and Adapters), which provides clean separation of concerns and enables multiple integration modes while maintaining a pure domain core.

```
┌─────────────────────────────────────────────────────────────────┐
│                    Presentation Layer                            │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │ CLI      │  │ gRPC     │  │ MCP      │  │ REST     │        │
│  │ Commands │  │ Server   │  │ Server   │  │ (Future) │        │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘        │
└───────┼─────────────┼─────────────┼─────────────┼───────────────┘
        │             │             │             │
┌───────┼─────────────┼─────────────┼─────────────┼───────────────┐
│       │       Application Layer   │             │                │
│       │             │             │             │                │
│  ┌────▼─────────────▼─────────────▼─────────────▼────┐          │
│  │           Use Cases / Business Logic                │          │
│  │  - ManageTask                                       │          │
│  │  - RunTaskWithFlow (Orchestration)                 │          │
│  │  - PRD Parsing & Task Generation                   │          │
│  └────┬────────────────────────────────────────┬──────┘          │
└───────┼────────────────────────────────────────┼─────────────────┘
        │                                        │
┌───────┼────────────────────────────────────────┼─────────────────┐
│       │         Ports (Interfaces)             │                 │
│  ┌────▼──────────────┐          ┌─────────────▼──────┐          │
│  │ TaskRepository    │          │  LLM Ports          │          │
│  │ Port              │          │  - Enhancer         │          │
│  │                   │          │  - TestGenerator    │          │
│  │ PRDParser Port    │          │  - PRDParser        │          │
│  └────┬──────────────┘          └─────────────┬──────┘          │
└───────┼────────────────────────────────────────┼─────────────────┘
        │                                        │
┌───────┼────────────────────────────────────────┼─────────────────┐
│       │         Adapters (Implementations)     │                 │
│  ┌────▼──────────────┐          ┌─────────────▼──────┐          │
│  │ SQLite            │          │  Rig Framework      │          │
│  │ TaskAdapter       │          │  - RigEnhancer      │          │
│  │                   │          │  - RigTestGen       │          │
│  │ MarkdownPRD       │          │  - RigPRDParser     │          │
│  │ Parser            │          │                     │          │
│  └────┬──────────────┘          └─────────────┬──────┘          │
└───────┼────────────────────────────────────────┼─────────────────┘
        │                                        │
┌───────▼────────────────────────────────────────▼─────────────────┐
│                        Domain Layer                               │
│  ┌───────────────┐  ┌───────────────┐  ┌──────────────┐         │
│  │ Task          │  │ Enhancement   │  │ Comprehension│         │
│  │ (Entity)      │  │ (Value Object)│  │ Test (VO)    │         │
│  │               │  │               │  │              │         │
│  │ - id          │  │ - enhanced    │  │ - question   │         │
│  │ - title       │  │   description │  │ - answer     │         │
│  │ - status      │  │ - reasoning   │  │ - difficulty │         │
│  │ - assignee    │  │               │  │              │         │
│  │ - metadata    │  │               │  │              │         │
│  └───────────────┘  └───────────────┘  └──────────────┘         │
│                                                                   │
│  Domain Services: Routing, Validation, Business Rules            │
└───────────────────────────────────────────────────────────────────┘
```

### Key Principles

1. **Dependency Inversion**: Domain layer has no dependencies; all dependencies point inward
2. **Interface Segregation**: Ports define minimal, focused interfaces
3. **Separation of Concerns**: Each layer has a distinct responsibility
4. **Testability**: Pure domain logic can be tested without infrastructure
5. **Flexibility**: Easy to swap adapters (e.g., SQLite → PostgreSQL, Rig → OpenAI)

## Hexagonal Architecture

### Why Hexagonal Architecture?

Rigger uses hexagonal architecture to achieve:

- **Multiple Integration Points**: CLI, gRPC, MCP, and future REST API all use the same domain logic
- **Vendor Independence**: Swap LLM providers (Rig → OpenAI → Anthropic) without touching domain
- **Test Isolation**: Domain logic tested independently of databases and external services
- **Clear Boundaries**: Explicit ports prevent implicit coupling

### The Hexagon

```
                     ┌─────────────────────┐
                     │    gRPC Server      │
                     └──────────┬──────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
  ┌─────▼─────┐         ┌───────▼────────┐      ┌──────▼──────┐
  │    CLI    │         │  MCP Server    │      │  REST API   │
  └─────┬─────┘         └───────┬────────┘      └──────┬──────┘
        │                       │                       │
        └───────────────────────┼───────────────────────┘
                                │
                      ┌─────────▼──────────┐
                      │                    │
                      │   Application      │
                      │   Core (Hexagon)   │
                      │                    │
                      │  - Domain          │
                      │  - Use Cases       │
                      │  - Ports           │
                      │                    │
                      └─────────┬──────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
  ┌─────▼─────┐         ┌───────▼────────┐      ┌──────▼──────┐
  │  SQLite   │         │  Rig LLM       │      │  Broadcast  │
  │  Adapter  │         │  Adapters      │      │  Channel    │
  └───────────┘         └────────────────┘      └─────────────┘
```

### Port Types

**Driving Ports (Primary)**: Expose domain capabilities to the outside world
- `TaskRepositoryPort`: CRUD operations on tasks
- `PRDParserPort`: Parse PRD documents into tasks

**Driven Ports (Secondary)**: Domain calls out to external systems
- `LLMEnhancerPort`: Enhance task descriptions
- `TestGeneratorPort`: Generate comprehension tests
- `RouterPort`: Route tasks to appropriate orchestration paths

## Workspace Structure

Rigger is organized as a Cargo workspace with four main crates:

```
rig-task-pipeline/
├── Cargo.toml                    # Workspace root
│
├── task_manager/                 # Core task domain
│   ├── src/
│   │   ├── domain/
│   │   │   ├── task.rs          # Task entity
│   │   │   ├── task_status.rs   # Status enum
│   │   │   ├── enhancement.rs   # Enhancement value object
│   │   │   └── comprehension_test.rs  # Test value object
│   │   ├── ports/
│   │   │   └── task_repository_port.rs  # Repository interface
│   │   ├── adapters/
│   │   │   └── sqlite_task_adapter.rs  # SQLite implementation
│   │   ├── use_cases/
│   │   │   └── manage_task.rs   # CRUD operations
│   │   └── utils/
│   │       └── prd_parser.rs    # Markdown PRD parsing
│   └── Cargo.toml
│
├── task_orchestrator/            # Orchestration & AI integration
│   ├── src/
│   │   ├── domain/
│   │   │   └── mod.rs           # Re-exports from task_manager
│   │   ├── ports/
│   │   │   ├── enhancer_port.rs        # LLM enhancement interface
│   │   │   ├── test_generator_port.rs  # Test generation interface
│   │   │   ├── prd_parser_port.rs      # PRD parsing interface
│   │   │   └── router_port.rs          # Routing interface
│   │   ├── adapters/
│   │   │   ├── rig_enhancer_adapter.rs      # Rig LLM enhancement
│   │   │   ├── rig_test_generator_adapter.rs # Rig test generation
│   │   │   ├── rig_prd_parser_adapter.rs    # Rig PRD parsing
│   │   │   └── complexity_router_adapter.rs # Complexity routing
│   │   ├── use_cases/
│   │   │   └── run_task_with_flow.rs   # StateGraph orchestration
│   │   └── nodes/
│   │       ├── enhance_node.rs         # Enhancement graph node
│   │       ├── test_node.rs            # Test generation node
│   │       └── decompose_node.rs       # Decomposition node
│   └── Cargo.toml
│
├── transcript_processor/         # Audio transcription (separate domain)
│   ├── src/
│   │   ├── domain/
│   │   │   └── transcript.rs    # Transcript entity
│   │   ├── adapters/
│   │   │   ├── rig_adapter.rs          # Rig-based diarization
│   │   │   └── mistralrs_embed_adapter.rs  # MistralRS embeddings
│   │   └── main.rs              # Standalone transcript processor
│   └── Cargo.toml
│
└── rigger_cli/                   # Presentation layer
    ├── proto/
    │   └── rigger.proto         # gRPC protobuf schema
    ├── src/
    │   ├── commands/
    │   │   ├── init.rs          # Initialize .rigger directory
    │   │   ├── task.rs          # CLI task commands
    │   │   ├── prd.rs           # CLI PRD commands
    │   │   ├── orchestrate.rs   # CLI orchestration
    │   │   ├── grpc_server.rs   # gRPC server implementation
    │   │   └── server.rs        # MCP server (stdio JSON-RPC)
    │   ├── display/
    │   │   └── task_table.rs    # Pretty-print task tables
    │   └── main.rs              # CLI entry point
    ├── examples/
    │   ├── sidecar_client.rs    # Example event subscriber
    │   └── test_grpc_client.rs  # gRPC test client
    ├── build.rs                 # Protobuf code generation
    └── Cargo.toml
```

### Crate Responsibilities

| Crate | Responsibility | Dependencies |
|-------|----------------|--------------|
| `task_manager` | Core domain, persistence | `hexser`, `sqlx`, `chrono` |
| `task_orchestrator` | AI orchestration, StateGraph | `task_manager`, `rig-core`, `async-trait` |
| `transcript_processor` | Audio transcription (separate domain) | `rig-core`, `candle`, `hf-hub` |
| `rigger_cli` | User interfaces (CLI/gRPC/MCP) | `task_manager`, `task_orchestrator`, `tonic`, `clap` |

## Layer-by-Layer Breakdown

### 1. Domain Layer

**Location**: `task_manager/src/domain/`

**Purpose**: Pure business logic with zero external dependencies

**Key Components**:

#### Task Entity (`task.rs`)
The core domain entity representing a task with all its metadata, enhancements, tests, and subtasks.

#### TaskStatus Enum (`task_status.rs`)
```rust
pub enum TaskStatus {
    Todo,
    InProgress,
    PendingEnhancement,
    PendingComprehensionTest,
    PendingFollowOn,
    PendingDecomposition,
    Decomposed,
    OrchestrationComplete,
    Completed,
    Archived,
}
```

Status transitions enforce workflow:
```
Todo → InProgress → PendingEnhancement → PendingComprehensionTest
                                              ↓
                                    PendingDecomposition
                                              ↓
                                         Decomposed
                                              ↓
                                   OrchestrationComplete → Completed
```

#### Enhancement & ComprehensionTest Value Objects
Represent LLM-generated enhancements and comprehension tests attached to tasks.

### 2. Ports Layer

**Location**: `task_manager/src/ports/` & `task_orchestrator/src/ports/`

**Purpose**: Define contracts between domain and external world

Key ports:
- `TaskRepositoryPort`: CRUD operations
- `EnhancerPort`: LLM task enhancement
- `TestGeneratorPort`: Test generation
- `PRDParserPort`: PRD document parsing
- `RouterPort`: Complexity-based routing

### 3. Adapters Layer

**Purpose**: Implement ports using concrete technologies

Key adapters:
- `SqliteTaskAdapter`: SQLite persistence (task_manager/src/adapters/sqlite_task_adapter.rs:1)
- `RigEnhancerAdapter`: Rig framework LLM enhancement (task_orchestrator/src/adapters/rig_enhancer_adapter.rs:1)
- `RigTestGeneratorAdapter`: Rig framework test generation (task_orchestrator/src/adapters/rig_test_generator_adapter.rs:1)
- `RigPRDParserAdapter`: Rig framework PRD parsing (task_orchestrator/src/adapters/rig_prd_parser_adapter.rs:1)
- `ComplexityRouterAdapter`: Keyword-based routing (task_orchestrator/src/adapters/complexity_router_adapter.rs:1)

### 4. Use Cases Layer

**Purpose**: Orchestrate domain logic and coordinate between ports

Key use cases:
- `ManageTask`: Basic CRUD operations (task_manager/src/use_cases/manage_task.rs:1)
- `RunTaskWithFlow`: StateGraph orchestration (task_orchestrator/src/use_cases/run_task_with_flow.rs:1)

### 5. Presentation Layer

**Location**: `rigger_cli/`

**Purpose**: Expose domain functionality through multiple interfaces (CLI, gRPC, MCP)

## Data Flow

### CLI Flow (Synchronous)

```
User Input (CLI)
       ↓
clap::Parser (main.rs)
       ↓
Command Handler (commands/task.rs)
       ↓
Use Case (manage_task.rs)
       ↓
Port (TaskRepositoryPort)
       ↓
Adapter (SqliteTaskAdapter)
       ↓
SQLite Database
       ↓
Response (task_table.rs)
       ↓
Terminal Output
```

### gRPC Flow (Asynchronous with Broadcasting)

```
gRPC Client Request
       ↓
tonic::Server (grpc_server.rs)
       ↓
RPC Handler (e.g., add_task)
       ↓
Use Case (manage_task.rs)
       ↓
Adapter (SqliteTaskAdapter::save_async)
       ↓
SQLite Database
       ↓
Broadcast Event → tokio::broadcast::channel
       ↓                    ↓
gRPC Response       Multiple Sidecar Subscribers
       ↓                    ↓
Client receives     Sidecars receive TaskEvent
  Task                in real-time
```

### Orchestration Flow (StateGraph)

```
User triggers orchestration
       ↓
run_task_with_flow()
       ↓
1. Route Task (ComplexityRouterAdapter)
       ↓
2. Build StateGraph (enhance → test → decompose)
       ↓
3. Execute Graph
   ├─ enhance_node (RigEnhancerAdapter → LLM)
   ├─ test_node (RigTestGeneratorAdapter → LLM)
   └─ decompose_node (RigPRDParserAdapter → LLM)
       ↓
4. Update Task Status (OrchestrationComplete)
       ↓
5. Persist Result (SqliteTaskAdapter)
       ↓
Return orchestrated Task
```

## Integration Modes

### 1. CLI Mode

**Use case**: Local development, scripting, automation

**Protocol**: Native Rust function calls

**Example**:
```bash
rig task add "Implement feature X"
rig task list --status todo
rig orchestrate run task-123 --model llama3.1
```

### 2. gRPC Mode

**Use case**: Distributed systems, sidecars, polyglot clients

**Protocol**: gRPC over HTTP/2, protobuf encoding

**Port**: `50051` (default)

**Example**:
```bash
# Start server
rig grpc

# Add task via grpcurl
grpcurl -plaintext -d '{"title": "Task"}' localhost:50051 rigger.v1.RiggerService/AddTask

# Subscribe to events
cargo run --example sidecar_client
```

**Key Feature**: Broadcast channel enables 1-to-N event distribution

### 3. MCP Mode (Model Context Protocol)

**Use case**: IDE integration, Claude Code, Cline, etc.

**Protocol**: stdio JSON-RPC 2.0

**Example**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "add_task",
    "arguments": {
      "title": "Implement OAuth"
    }
  }
}
```

### 4. REST API (Future)

**Planned**: HTTP REST API for web frontends

## Event Broadcasting

### Architecture

```
┌─────────────────────────────────────────────────────┐
│              RiggerServiceImpl                      │
│                                                     │
│  tokio::sync::broadcast::Sender<TaskEvent>         │
│  (1000 event buffer)                                │
│                                                     │
│  fn broadcast_event(&self, event: TaskEvent)       │
└─────────────────┬───────────────────────────────────┘
                  │
                  ├────────────────┬─────────────────┬─────────
                  │                │                 │
         ┌────────▼────────┐ ┌────▼─────┐  ┌────────▼────────┐
         │  Sidecar 1      │ │ Sidecar 2│  │  Sidecar N      │
         │  (Logger)       │ │ (Metrics)│  │  (Notifier)     │
         └─────────────────┘ └──────────┘  └─────────────────┘
```

### Event Types

- `CREATED`: New task created
- `UPDATED`: Task fields updated
- `DELETED`: Task archived
- `STATUS_CHANGED`: Task status changed
- `ASSIGNED`: Task assigned
- `DECOMPOSED`: Task decomposed into subtasks
- `ORCHESTRATED`: Orchestration complete

### Broadcasting Points

Events are broadcast at:
1. Task Creation (`AddTask` RPC) - rigger_cli/src/commands/grpc_server.rs:200
2. Task Update (`UpdateTask` RPC) - rigger_cli/src/commands/grpc_server.rs:250
3. Task Deletion (`DeleteTask` RPC) - rigger_cli/src/commands/grpc_server.rs:300
4. Orchestration Complete (`OrchestrateTask` RPC) - rigger_cli/src/commands/grpc_server.rs:450

## Orchestration Pipeline

### StateGraph Implementation

Rigger uses a custom StateGraph implementation inspired by LangGraph for building dynamic orchestration workflows based on task complexity.

### Graph Nodes

- **Enhance Node**: LLM-based task description enhancement - task_orchestrator/src/nodes/enhance_node.rs:1
- **Test Node**: Comprehension test generation - task_orchestrator/src/nodes/test_node.rs:1
- **Decompose Node**: Task decomposition into subtasks - task_orchestrator/src/nodes/decompose_node.rs:1

### Routing Logic

The `ComplexityRouterAdapter` determines which nodes to execute:

- **Simple**: enhance only
- **Moderate**: enhance → test
- **Complex**: enhance → test → decompose

## Design Patterns

### 1. Hexagonal Architecture (Ports & Adapters)
Isolate domain logic from infrastructure

### 2. Dependency Injection
Use cases accept port traits as parameters

### 3. Builder Pattern
StateGraph construction with fluent API

### 4. Repository Pattern
Abstract data persistence via `TaskRepositoryPort`

### 5. Adapter Pattern
Adapt Rig framework to our port interfaces

### 6. Strategy Pattern
Swap routing algorithms via `RouterPort`

### 7. Observer Pattern (Pub/Sub)
Event broadcasting for sidecar integration

### 8. Command Pattern
Encapsulate requests as objects (CLI, gRPC, MCP)

## Module Dependencies

### Dependency Graph

```
rigger_cli
    ├── task_manager (domain, persistence)
    ├── task_orchestrator (AI orchestration)
    │   └── task_manager
    ├── tonic (gRPC)
    ├── clap (CLI)
    └── tokio (async runtime)

task_orchestrator
    ├── task_manager
    ├── rig-core (LLM framework)
    ├── async-trait
    └── tokio

task_manager
    ├── hexser (ports/adapters)
    ├── sqlx (SQLite)
    ├── chrono (timestamps)
    └── serde (JSON)

transcript_processor
    ├── rig-core
    ├── candle (ML framework)
    ├── hf-hub (model downloads)
    └── tokio
```

## Testing Strategy

### Unit Tests
**Target**: Domain logic - task_manager/src/domain/task.rs

### Integration Tests
**Target**: Port/Adapter interactions - task_manager/tests/sqlite_integration.rs

### End-to-End Tests
**Target**: Full orchestration flows - task_orchestrator/tests/integration_end_to_end_flow.rs

### gRPC Tests
**Target**: gRPC server functionality - rigger_cli/examples/test_grpc_client.rs

## Security Considerations

### 1. gRPC Security (Production)
**Current**: Plaintext HTTP/2 (development only)
**Production**: TLS with mutual authentication required

### 2. Input Validation
Domain layer enforces business rule invariants

### 3. SQL Injection Prevention
sqlx compile-time query verification

### 4. LLM Prompt Injection Mitigation
Separation of instructions from user input

## Performance Considerations

### 1. Async/Await Throughout
All I/O operations use async/await to avoid blocking

### 2. Connection Pooling
SQLite uses `sqlx::SqlitePool` for connection reuse

### 3. Broadcast Channel Sizing
Default: 1000 events (tunable for high-throughput)

### 4. Lazy Evaluation
StateGraph only executes nodes needed based on routing

### 5. Streaming Responses
gRPC subscriptions stream events incrementally

## Scalability

### Horizontal Scaling
gRPC server is stateless; can run multiple instances behind load balancer

### Vertical Scaling
SQLite sufficient for single-node; migrate to PostgreSQL for multi-node

### Caching (Future)
Cache LLM responses for identical prompts using Redis

## Future Enhancements

1. **REST API**: HTTP REST API for web frontends
2. **Web UI**: React/Svelte dashboard
3. **Multi-Tenancy**: Workspace/organization support
4. **Real-time Collaboration**: Operational Transform or CRDT
5. **Advanced Routing**: LLM semantic classifier
6. **Workflow Engine**: Conditional edges, loops, human-in-the-loop
7. **Plugin System**: Dynamic loading of custom nodes/adapters
8. **Time-Travel Debugging**: Event sourcing to replay flows

## Conclusion

Rigger's architecture prioritizes:

1. **Modularity**: Clean separation via hexagonal architecture
2. **Extensibility**: Easy to add new adapters, nodes, integration modes
3. **Testability**: Pure domain logic isolated from infrastructure
4. **Performance**: Async throughout, efficient event broadcasting
5. **Developer Experience**: Multiple integration modes (CLI, gRPC, MCP)

The combination of hexagonal architecture, StateGraph orchestration, and distributed event broadcasting creates a flexible, scalable foundation for AI-powered task management.

For specific implementation details, see:
- [API Reference](API_REFERENCE.md)
- [Development Guide](DEVELOPMENT.md)
- [Deployment Guide](DEPLOYMENT.md)
- [gRPC Setup Guide](mcp/GRPC_SETUP.md)
