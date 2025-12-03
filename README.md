# Rigger - AI-Powered Project Management for Agents

**An intelligent task management system with LLM-powered orchestration, designed for distributed AI agent workflows.**

[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org)
[![Tests](https://img.shields.io/badge/tests-passing-green.svg)](#testing)
[![Architecture](https://img.shields.io/badge/architecture-hexagonal-blue.svg)](#architecture)
[![gRPC](https://img.shields.io/badge/gRPC-tonic-blue.svg)](#grpc-server)

Rigger is a production-ready task orchestration platform that combines traditional task management with AI-powered enhancement, comprehension testing, and intelligent decomposition. Built with Rust using hexagonal architecture, it provides both gRPC and stdio interfaces for seamless integration with AI agents, IDEs, and distributed systems.

## ✨ Key Features

### 🤖 AI-Powered Orchestration
- **Intelligent Enhancement**: LLM-powered task description enrichment
- **Comprehension Testing**: Generate Q&A tests to verify task clarity
- **Automatic Decomposition**: Break complex tasks into manageable subtasks
- **Semantic Routing**: Intelligent task classification based on complexity

### 🔌 Multiple Integration Modes
- **gRPC Server**: Distributed architecture with real-time event broadcasting to sidecars
- **MCP Server**: stdio JSON-RPC 2.0 for IDE integration (Cursor AI, Windsurf)
- **CLI**: Full-featured command-line interface with orchestration support

### 📊 Task Management
- **PRD Parsing**: Extract tasks from Product Requirements Documents (markdown)
- **SQLite Persistence**: Reliable local database storage
- **Task Lifecycle**: Todo → InProgress → Enhanced → Tested → Completed
- **Parent-Child Relationships**: Track task decomposition hierarchies

### 🏗️ Production-Ready Architecture
- **Hexagonal Architecture**: Clean separation of concerns (ports & adapters)
- **Type-Safe gRPC**: Protocol Buffers for cross-language compatibility
- **Event Broadcasting**: Real-time task events to multiple subscribers
- **Async Throughout**: Fully non-blocking with tokio runtime

## 📚 Table of Contents

- [Quick Start](#quick-start)
- [Installation](#installation)
- [Usage](#usage)
  - [CLI Commands](#cli-commands)
  - [gRPC Server](#grpc-server)
  - [MCP Server](#mcp-server-ide-integration)
- [Architecture](#architecture)
- [Documentation](#documentation)
- [Development](#development)
- [Testing](#testing)
- [Contributing](#contributing)

## 🚀 Quick Start

### Prerequisites

- **Rust 2024 Edition** (latest stable)
- **LLM Backend** (choose one):
  - **Ollama** (cross-platform, recommended for beginners)
  - **MLX** (macOS Apple Silicon only, 30-50% faster - see [MLX Setup](docs/MLX_SETUP.md))
- macOS, Linux, or Windows

### Installation

```bash
# Clone the repository
git clone https://github.com/your-org/rig-task-pipeline
cd rig-task-pipeline

# Install Ollama (if not already installed)
./setup-ollama.sh  # macOS/Linux

# Pull required model
ollama pull llama3.1

# Build and install rigger CLI
cargo install --path rigger_cli

# Verify installation
rig --version
```

### Your First Project

```bash
# Initialize Rigger in your project
cd /path/to/your/project
rig init

# Create a simple PRD
cat > PRD.md <<EOF
# MVP Development

## Objectives
- Implement user authentication
- Create dashboard UI
- Add data export feature

## Tech Stack
- Rust backend
- React frontend

## Constraints
- Launch in 2 weeks
- Must support OAuth2
EOF

# Parse PRD and generate tasks
rig parse PRD.md

# List generated tasks
rig list

# Execute a task through orchestration (enhance + comprehension test)
rig do <task-id>
```

## 📖 Usage

### CLI Commands

Rigger provides a comprehensive CLI for task management and orchestration:

#### Initialize Project

```bash
rig init
```

Creates `.rigger/` directory with:
- `tasks.db` - SQLite database
- `config.json` - Configuration file
- `prds/` - PRD storage directory

#### Parse PRD

```bash
# Parse a PRD markdown file
rig parse docs/PRD.md

# The PRD should follow this structure:
# # Title
# ## Objectives
# - Objective 1
# - Objective 2
# ## Tech Stack
# - Technology 1
# ## Constraints
# - Constraint 1
```

#### List Tasks

```bash
# List all tasks
rig list

# Filter by status
rig list --status InProgress

# Filter by assignee
rig list --assignee Alice

# Sort by field
rig list --sort due_date

# Pagination
rig list --limit 10 --offset 20
```

#### Execute Task (Orchestration)

```bash
# Run a task through the orchestration pipeline
# - Complexity scoring
# - Semantic routing (enhance vs. decompose)
# - Enhancement (if needed)
# - Comprehension test generation
# - Task decomposition (if complex)
rig do <task-id>
```

The orchestration flow:
```
Task → Complexity Scorer → Router → [Enhance → Test] or [Decompose] → Complete
```

#### Task Decomposition & Hierarchy

Rigger automatically decomposes complex tasks (complexity >= 7) into manageable sub-tasks during PRD parsing. This creates a hierarchical task structure for better organization and execution.

**Automatic Decomposition**:
```bash
# Parse PRD - complex tasks automatically decompose
rig parse PRD.md
```

When a generated task has complexity >= 7:
1. LLM analyzes the task and PRD context
2. Generates 3-5 sub-tasks with lower complexity (default 3)
3. Links sub-tasks to parent via `parent_task_id`
4. Updates parent task status to `Decomposed`
5. Displays summary: "✓ Auto-decomposed X complex tasks into Y sub-tasks"

**Hierarchy in TUI**:
```
TODO Column:
┌─────────────────────
│ 🔹 Implement Auth System (8)     ← Parent task
├─────────────────────
├─ 🔹 Setup JWT middleware (3)      ← First sub-task
├─────────────────────
├─ 🔹 Add OAuth provider (3)        ← Intermediate sub-task
├─────────────────────
└─ 🔹 Write auth tests (3)          ← Last sub-task
└─────────────────────
```

**Visual Indicators**:
- `├─` - Intermediate child task
- `└─` - Last child task
- Indentation shows parent-child relationship
- Age icons (🔹/⏰/🔴) show task freshness

**Task Fields**:
```rust
Task {
    parent_task_id: Option<String>,  // Link to parent
    subtask_ids: Vec<String>,        // List of children
    complexity: Option<u8>,          // 1-10 scale
    status: TaskStatus::Decomposed,  // Parent status after decomposition
}
```

**Benefits**:
- Large epics broken into actionable items
- Better workload distribution
- Clearer progress tracking
- Hierarchical navigation in TUI

#### Terminal User Interface (TUI)

```bash
# Launch interactive Kanban-style TUI
rig tui
```

**Features**:
- **📋 Task Board**: Kanban columns (TODO, IN PROGRESS, COMPLETED, ARCHIVED, ERRORED)
- **🌳 Task Hierarchy**: Parent-child relationships with tree indicators (├─, └─)
- **🧠 Thinking**: Chain-of-thought reasoning visualization
- **🌐 Network**: API request/response logging
- **❓ Help**: Keyboard controls reference

**Keyboard Controls**:
- `Tab` / `Shift+Tab`: Switch between views
- `↑` / `k`: Move up
- `↓` / `j`: Move down
- `r`: Refresh tasks from database
- `q` / `Esc`: Quit

### gRPC Server

Start the gRPC server for distributed architectures and sidecar integration:

```bash
# Start on default port (50051)
rig grpc

# Start on custom port
rig grpc --port 50052
```

#### gRPC RPCs Available

**Task Management**:
- `ListTasks` - Query with filters, sorting, pagination
- `AddTask` - Create new tasks
- `UpdateTask` - Modify status, assignee, due date
- `GetTask` - Retrieve single task by ID
- `DeleteTask` - Archive tasks

**PRD Operations**:
- `ParsePRD` - Extract sections from PRD markdown
- `GenerateTasksFromPRD` - LLM-powered task generation

**Orchestration**:
- `OrchestrateTask` - Run task through enhancement + testing flow

**Event Streaming**:
- `SubscribeToTaskEvents` - Real-time event stream for sidecars

#### Example: Python Client

```python
import grpc
from rigger_pb2 import ListTasksRequest
from rigger_pb2_grpc import RiggerServiceStub

channel = grpc.insecure_channel('[::1]:50051')
stub = RiggerServiceStub(channel)

response = stub.ListTasks(ListTasksRequest())
for task in response.tasks:
    print(f"{task.id}: {task.title} [{task.status}]")
```

See [`docs/mcp/GRPC_SETUP.md`](docs/mcp/GRPC_SETUP.md) for comprehensive gRPC documentation.

### MCP Server (IDE Integration)

The MCP (Model Context Protocol) server provides stdio JSON-RPC 2.0 interface for IDE integration:

```bash
# Start MCP server
rig server
```

#### Configure Cursor AI

Add to `~/.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "rigger": {
      "command": "rig",
      "args": ["server"],
      "cwd": "${workspaceFolder}",
      "env": {}
    }
  }
}
```

Restart Cursor and use `@rig` commands:
- `@rig list all tasks`
- `@rig add task "Implement OAuth2" assigned to Alice`
- `@rig update task <id> to status InProgress`

See [`docs/mcp/MCP_SETUP.md`](docs/mcp/MCP_SETUP.md) and [`docs/mcp/QUICK_START.md`](docs/mcp/QUICK_START.md) for IDE setup.

## 🏛️ Architecture

Rigger follows **hexagonal architecture** (ports & adapters) with strict layer separation:

```
┌──────────────────────────────────────────────────────┐
│                    Presentation                      │
│  • CLI (rigger_cli)                                  │
│  • gRPC Server (tonic)                               │
│  • MCP Server (stdio JSON-RPC)                       │
└──────────────────┬───────────────────────────────────┘
                   │
┌──────────────────▼───────────────────────────────────┐
│                  Application                         │
│  • Use Cases (task_orchestrator)                     │
│    - run_task_with_flow (orchestration)              │
│    - ManageTaskUseCase                               │
│  • Ports (Interfaces)                                │
│    - TaskEnhancementPort                             │
│    - ComprehensionTestPort                           │
│    - TaskDecompositionPort                           │
│    - PRDParserPort                                   │
└──────────────────┬───────────────────────────────────┘
                   │
┌──────────────────▼───────────────────────────────────┐
│                  Adapters                            │
│  • Ollama LLM Adapters (cross-platform)              │
│    - OllamaEnhancementAdapter                        │
│    - OllamaComprehensionTestAdapter                  │
│  • MLX Adapters (macOS Apple Silicon, 30-50% faster) │
│    - MlxSubprocessAdapter (Enhancement + Decomp)     │
│  • Rig Framework Adapters                            │
│    - RigTaskDecompositionAdapter (Extractor API)     │
│    - RigPRDParserAdapter                             │
│  • SQLite Adapter (SqliteTaskAdapter)                │
└──────────────────┬───────────────────────────────────┘
                   │
┌──────────────────▼───────────────────────────────────┐
│                   Domain                             │
│  • Task Entity                                       │
│  • TaskStatus (enum)                                 │
│  • Enhancement                                       │
│  • ComprehensionTest                                 │
│  • PRD                                               │
│  • ComplexityScorer (service)                        │
└──────────────────────────────────────────────────────┘
```

### Workspace Structure

```
rig-task-pipeline/
├── rigger_cli/              # CLI + gRPC/MCP servers
│   ├── proto/rigger.proto   # gRPC schema
│   ├── src/
│   │   ├── commands/        # CLI command handlers
│   │   │   ├── grpc_server.rs    # gRPC server
│   │   │   ├── server.rs         # MCP server
│   │   │   ├── init.rs, list.rs, parse.rs, do_task.rs
│   │   └── display/         # CLI output formatting
│   └── examples/
│       ├── sidecar_client.rs     # gRPC event subscriber
│       └── test_grpc_client.rs   # gRPC testing
│
├── task_orchestrator/       # Orchestration engine
│   ├── src/
│   │   ├── graph/           # StateGraph-based flow
│   │   │   ├── nodes/       # Orchestration nodes
│   │   │   ├── flow_shims/  # graph-flow integration
│   │   │   └── assemble_orchestrator_flow.rs
│   │   ├── ports/           # Port interfaces
│   │   ├── adapters/        # LLM adapters (Ollama, Rig)
│   │   └── use_cases/       # Orchestration use cases
│   └── tests/               # Integration tests
│
├── task_manager/            # Task domain & persistence
│   ├── src/
│   │   ├── domain/          # Core entities
│   │   │   ├── task.rs
│   │   │   ├── task_status.rs
│   │   │   ├── enhancement.rs
│   │   │   ├── comprehension_test.rs
│   │   │   ├── prd.rs
│   │   │   └── services/complexity_scorer.rs
│   │   ├── adapters/
│   │   │   └── sqlite_task_adapter.rs
│   │   ├── ports/
│   │   │   └── task_repository_port.rs
│   │   ├── use_cases/
│   │   │   └── manage_task.rs
│   │   └── utils/
│   │       └── prd_parser.rs
│   └── tests/
│
└── transcript_extractor/    # Shared domain types
    └── src/domain/
        └── action_item.rs
```

### Key Design Patterns

1. **Ports & Adapters**: Clean interfaces, swappable implementations
2. **Dependency Injection**: Constructor injection via Arc<dyn Port>
3. **Event Sourcing**: Task events broadcast to subscribers
4. **CQRS**: Separate read (list/get) and write (add/update) paths
5. **StateGraph**: Orchestration flow modeled as directed graph

## 📘 Documentation

### Core Documentation
- **[Architecture Guide](docs/ARCHITECTURE.md)** - Detailed architectural overview
- **[API Reference](docs/API_REFERENCE.md)** - Complete API documentation
- **[Development Guide](docs/DEVELOPMENT.md)** - Contributing and development workflows
- **[Deployment Guide](docs/DEPLOYMENT.md)** - Production deployment strategies

### Integration Guides
- **[MLX Backend Setup](docs/MLX_SETUP.md)** - 30-50% faster inference on macOS Apple Silicon
- **[gRPC Setup](docs/mcp/GRPC_SETUP.md)** - Comprehensive gRPC server guide
- **[MCP Setup](docs/mcp/MCP_SETUP.md)** - IDE integration setup
- **[MCP Quick Start](docs/mcp/QUICK_START.md)** - 5-minute MCP setup

### Crate-Specific Docs
- **[rigger_cli README](rigger_cli/README.md)** - CLI and server documentation
- **[task_orchestrator README](task_orchestrator/README.md)** - Orchestration engine
- **[task_manager README](task_manager/README.md)** - Task domain and persistence
- **[transcript_extractor README](transcript_extractor/README.md)** - Shared domain types

## 🔧 Development

### Building

```bash
# Build all crates
cargo build --release

# Build specific crate
cargo build --package rigger_cli --release

# Install CLI binary
cargo install --path rigger_cli
```

### Testing

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test --package task_orchestrator

# Run integration tests
cargo test --test integration_end_to_end_flow

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_task_decomposition_end_to_end
```

**Test Coverage**:
- Unit tests: 77+ tests across all crates
- Integration tests: 6+ end-to-end scenarios
- Doctests: 32+ embedded examples

### Code Standards

Rigger follows strict Rust coding guidelines:

- ✅ **No `use` statements**: All types use fully qualified paths
- ✅ **No `unsafe` code**: Safe Rust only (except necessary FFI)
- ✅ **One item per file**: Single responsibility principle
- ✅ **Comprehensive documentation**: `//!` file docs + `///` item docs
- ✅ **Revision history**: Timestamped change tracking
- ✅ **Functional style**: Immutability, iterator methods
- ✅ **In-file tests**: `#[cfg(test)]` blocks

### Adding New Features

1. **Define Port**: Add trait to appropriate `ports/` module
2. **Implement Adapter**: Create adapter in `adapters/`
3. **Add Tests**: Unit tests + integration tests
4. **Update Docs**: Document public APIs
5. **Wire in Main**: Update DI composition

Example:

```rust
// 1. Define port (task_orchestrator/src/ports/my_port.rs)
#[async_trait::async_trait]
pub trait MyPort: Send + Sync {
    async fn do_something(&self, task: &Task) -> Result<String, String>;
}

// 2. Implement adapter (task_orchestrator/src/adapters/my_adapter.rs)
pub struct MyAdapter { /* ... */ }

#[async_trait::async_trait]
impl MyPort for MyAdapter {
    async fn do_something(&self, task: &Task) -> Result<String, String> {
        // Implementation
    }
}

// 3. Wire in use case
let my_adapter = Arc::new(MyAdapter::new());
let use_case = MyUseCase::new(my_adapter);
```

## 🧪 Testing

### Running Tests

```bash
# All tests
cargo test

# Specific test suite
cargo test --package task_orchestrator --test integration_end_to_end_flow

# gRPC integration tests
cargo build --example test_grpc_client
cargo run --example test_grpc_client
```

### Test Examples

**Unit Test**:
```rust
#[test]
fn test_complexity_scorer() {
    let scorer = ComplexityScorer::new();
    let action = ActionItem {
        title: String::from("Implement OAuth2 with SAML"),
        assignee: None,
        due_date: None,
    };
    let task = Task::from_action_item(&action, None);
    let score = scorer.score_task(&task);
    assert!(score >= 7); // Complex task
}
```

**Integration Test**:
```rust
#[tokio::test]
async fn test_orchestration_flow() {
    let task = create_complex_task();
    let result = run_task_with_flow("llama3.1", "short_answer", task).await;
    assert!(result.is_ok());
    let enhanced_task = result.unwrap();
    assert_eq!(enhanced_task.status, TaskStatus::OrchestrationComplete);
}
```

## 🤝 Contributing

We welcome contributions! Please follow these guidelines:

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/my-feature`
3. **Follow code standards**: See [Development](#development) section
4. **Add tests**: All new code must have tests
5. **Update documentation**: Keep docs in sync with changes
6. **Submit PR**: Include description of changes and test results

### Development Workflow

```bash
# 1. Create feature branch
git checkout -b feature/add-new-adapter

# 2. Make changes following code standards
# 3. Run tests
cargo test

# 4. Format code
cargo fmt

# 5. Check for issues
cargo clippy

# 6. Commit with clear message
git commit -m "feat: add new LLM adapter for Anthropic Claude"

# 7. Push and create PR
git push origin feature/add-new-adapter
```

## 📊 Project Status

### Current Version: 0.1.0

**Status**: Production-ready for local use

**Completed Features**:
- ✅ CLI with all commands (init, parse, list, do, artifacts)
- ✅ RAG (Retrieval-Augmented Generation) with embedded sqlite-vec extension
- ✅ Vector similarity search across knowledge artifacts
- ✅ gRPC server with 8/10 RPCs functional
- ✅ MCP server (stdio JSON-RPC 2.0)
- ✅ Task orchestration (enhancement + comprehension testing)
- ✅ Task decomposition (LLM-powered)
- ✅ PRD parsing and task generation
- ✅ SQLite persistence
- ✅ Real-time event broadcasting
- ✅ Comprehensive testing (77+ tests)

**In Progress**:
- ⏳ Bidirectional gRPC streaming
- ⏳ Web dashboard UI
- ⏳ Multi-user support with authentication

**Future Enhancements**:
- 🔮 Kubernetes deployment manifests
- 🔮 OpenTelemetry tracing
- 🔮 Prometheus metrics
- 🔮 TLS/mTLS for production
- 🔮 Rate limiting and quotas

## 📝 License

[Specify your license here - e.g., MIT, Apache 2.0]

## 👥 Authors

- AI Assistant - Initial architecture and implementation
- [Your Name] - Project lead

## 🙏 Acknowledgments

- **[HEXSER Framework](https://github.com/squillo/hexser)** - Hexagonal architecture patterns
- **[Rig](https://github.com/0xplaygrounds/rig)** - LLM framework with Extractor API
- **[tonic](https://github.com/hyperium/tonic)** - gRPC framework for Rust
- **[graph-flow](https://docs.rs/graph-flow)** - StateGraph orchestration
- **[Ollama](https://ollama.ai)** - Local LLM runtime
- **[MLX](https://github.com/ml-explore/mlx)** - Apple's machine learning framework for Apple Silicon

## 📚 Resources

- **Documentation**: [`docs/`](docs/)
- **Examples**: [`rigger_cli/examples/`](rigger_cli/examples/)
- **Tests**: `cargo test --workspace`
- **Issues**: [GitHub Issues](https://github.com/your-org/rig-task-pipeline/issues)

---

**Built with ❤️ using Rust, hexagonal architecture, and AI-powered orchestration.**

For questions, issues, or contributions, please visit our [GitHub repository](https://github.com/your-org/rig-task-pipeline).

**Last Updated**: 2025-11-24
