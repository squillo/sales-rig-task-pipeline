# Rigger Development Guide

Complete guide for contributing to Rigger, including setup, conventions, testing, and best practices.

## Table of Contents

- [Development Environment](#development-environment)
- [Project Structure](#project-structure)
- [Code Style and Conventions](#code-style-and-conventions)
- [Testing](#testing)
- [Adding New Features](#adding-new-features)
- [Debugging](#debugging)
- [Contributing](#contributing)
- [Release Process](#release-process)

## Development Environment

### Prerequisites

Install the following:

1. **Rust** (1.75.0+)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup update
   ```

2. **Development Tools**
   ```bash
   # Code formatter
   rustup component add rustfmt

   # Linter
   rustup component add clippy

   # IDE support (rust-analyzer)
   rustup component add rust-analyzer
   ```

3. **Protocol Buffers Compiler** (for gRPC development)
   ```bash
   # macOS
   brew install protobuf

   # Ubuntu/Debian
   sudo apt install protobuf-compiler

   # Verify
   protoc --version  # Should be 3.x or later
   ```

4. **grpcurl** (for testing gRPC)
   ```bash
   # macOS
   brew install grpcurl

   # Linux
   go install github.com/fullstorydev/grpcurl/cmd/grpcurl@latest
   ```

5. **Ollama** (for LLM testing)
   ```bash
   # Install from https://ollama.ai/download

   # Pull models
   ollama pull llama3.1
   ollama pull qwen2.5
   ```

### Clone and Build

```bash
# Clone repository
git clone https://github.com/anthropics/rig-task-pipeline.git
cd rig-task-pipeline

# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Build release
cargo build --release --package rigger_cli
```

### IDE Setup

#### VS Code

Install extensions:
- **rust-analyzer**: Rust language server
- **CodeLLDB**: Debugger
- **Better TOML**: TOML syntax highlighting
- **Protobuf**: Protobuf syntax highlighting

**Workspace settings** (`.vscode/settings.json`):
```json
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.check.command": "clippy",
  "editor.formatOnSave": true,
  "editor.rulers": [100],
  "files.exclude": {
    "**/target": true,
    "**/.rigger": true
  }
}
```

**Debug configuration** (`.vscode/launch.json`):
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug rigger_cli",
      "cargo": {
        "args": [
          "build",
          "--bin=rig",
          "--package=rigger_cli"
        ],
        "filter": {
          "name": "rig",
          "kind": "bin"
        }
      },
      "args": ["task", "list"],
      "cwd": "${workspaceFolder}"
    },
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug gRPC server",
      "cargo": {
        "args": [
          "build",
          "--bin=rig",
          "--package=rigger_cli"
        ],
        "filter": {
          "name": "rig",
          "kind": "bin"
        }
      },
      "args": ["grpc"],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

#### IntelliJ IDEA / CLion

Install **Rust** plugin, then:

1. Import project as Cargo project
2. Enable Clippy: Settings → Rust → External Linters → Clippy
3. Enable format on save: Settings → Tools → Actions on Save → Reformat code

### Environment Setup

Create `.env` file in project root:

```bash
# Database
RIGGER_DB_PATH=.rigger/tasks.db

# Logging
RUST_LOG=debug
RUST_BACKTRACE=1

# LLM
RIGGER_DEFAULT_MODEL=llama3.1

# gRPC
RIGGER_GRPC_PORT=50051
```

Load environment:
```bash
source .env  # or use direnv
```

## Project Structure

### Workspace Layout

```
rig-task-pipeline/
├── Cargo.toml                 # Workspace manifest
├── .github/
│   └── workflows/
│       └── ci.yml            # GitHub Actions CI
├── docs/                     # Documentation
│   ├── ARCHITECTURE.md
│   ├── API_REFERENCE.md
│   ├── DEPLOYMENT.md
│   └── DEVELOPMENT.md
├── task_manager/             # Core domain crate
│   ├── src/
│   │   ├── domain/           # Entities, value objects
│   │   ├── ports/            # Interface traits
│   │   ├── adapters/         # Implementations
│   │   ├── use_cases/        # Business logic
│   │   └── utils/            # Utilities
│   ├── tests/                # Integration tests
│   └── Cargo.toml
├── task_orchestrator/        # Orchestration crate
│   ├── src/
│   │   ├── ports/            # LLM interfaces
│   │   ├── adapters/         # Rig implementations
│   │   ├── use_cases/        # Orchestration flows
│   │   └── nodes/            # StateGraph nodes
│   ├── tests/                # Integration tests
│   └── Cargo.toml
├── transcript_processor/     # Transcription crate
├── rigger_cli/               # CLI & servers crate
│   ├── proto/
│   │   └── rigger.proto      # gRPC schema
│   ├── src/
│   │   ├── commands/         # CLI commands
│   │   ├── display/          # Output formatting
│   │   └── main.rs
│   ├── examples/             # Example programs
│   │   ├── sidecar_client.rs
│   │   └── test_grpc_client.rs
│   ├── build.rs              # Protobuf codegen
│   └── Cargo.toml
└── README.md
```

### Module Organization

Each crate follows hexagonal architecture:

```
crate/
├── domain/       # Pure business logic, zero dependencies
├── ports/        # Interface traits
├── adapters/     # Concrete implementations
├── use_cases/    # Application logic
└── utils/        # Shared utilities
```

**Dependency rule**: Dependencies always point inward
- `adapters` → `ports` → `domain`
- `use_cases` → `ports` → `domain`
- `domain` never imports from outer layers

## Code Style and Conventions

### Rust Style

Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/):

1. **Naming**
   - Types: `PascalCase`
   - Functions/variables: `snake_case`
   - Constants: `SCREAMING_SNAKE_CASE`
   - Traits: `PascalCase` (prefer noun or adjective)

2. **Formatting**
   ```bash
   # Format code
   cargo fmt

   # Check formatting
   cargo fmt -- --check
   ```

3. **Linting**
   ```bash
   # Run Clippy
   cargo clippy --workspace -- -D warnings

   # Fix issues automatically
   cargo clippy --fix
   ```

### File Headers

Add module-level documentation:

```rust
//! Brief description of the module.
//!
//! More detailed explanation of what this module does,
//! its purpose, and how it fits into the architecture.
//!
//! # Examples
//!
//! ```
//! use crate::module::function;
//! let result = function();
//! ```

use std::collections::HashMap;
// ...
```

### Documentation

1. **Public items**: Always document with `///`
   ```rust
   /// Creates a new task with the given parameters.
   ///
   /// # Arguments
   ///
   /// * `title` - The task title
   /// * `assignee` - Optional assignee name
   ///
   /// # Returns
   ///
   /// Returns a `Task` instance with generated ID and timestamp.
   ///
   /// # Examples
   ///
   /// ```
   /// use task_manager::domain::task::Task;
   ///
   /// let task = Task::new("Implement feature".to_string(), None, None, None, None);
   /// assert_eq!(task.status, TaskStatus::Todo);
   /// ```
   pub fn new(title: String, assignee: Option<String>, ...) -> Self {
       // ...
   }
   ```

2. **Private items**: Use `//` for implementation notes
   ```rust
   // Internal helper to validate task title
   fn validate_title(title: &str) -> bool {
       !title.is_empty() && title.len() <= 500
   }
   ```

### Error Handling

Use `HexResult<T>` for all fallible operations:

```rust
use hexser::HexResult;

pub fn create_task(title: String) -> HexResult<Task> {
    if title.is_empty() {
        return Err(HexError::ValidationError("Title cannot be empty".into()));
    }

    // ...
    Ok(task)
}
```

**Don't** use `unwrap()` or `expect()` in library code (tests are OK).

### Async Conventions

1. **Use `async/await`** for I/O operations
2. **Avoid blocking** in async functions
3. **Use `tokio::spawn`** for concurrent tasks

```rust
#[async_trait]
pub trait EnhancerPort {
    async fn enhance_task(&self, task: &Task) -> HexResult<Enhancement>;
}

pub async fn enhance_multiple(tasks: Vec<Task>) -> HexResult<Vec<Enhancement>> {
    let futures: Vec<_> = tasks.iter()
        .map(|task| enhance_task(task))
        .collect();

    let results = futures::future::try_join_all(futures).await?;
    Ok(results)
}
```

### Testing Conventions

1. **Unit tests**: Same file as code
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_task_creation() {
           let task = Task::new("Test".to_string(), None, None, None, None);
           assert_eq!(task.status, TaskStatus::Todo);
       }
   }
   ```

2. **Integration tests**: `tests/` directory
   ```rust
   // tests/sqlite_integration.rs
   use task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter;

   #[tokio::test]
   async fn test_save_and_retrieve() {
       // ...
   }
   ```

3. **Example tests**: `examples/` directory
   ```rust
   // examples/test_grpc_client.rs
   #[tokio::main]
   async fn main() -> Result<(), Box<dyn std::error::Error>> {
       // Full integration test
   }
   ```

## Testing

### Running Tests

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test --package task_manager

# Specific test
cargo test --package task_manager test_task_creation

# With output
cargo test -- --nocapture

# Single-threaded (for debugging)
cargo test -- --test-threads=1
```

### Unit Tests

**Location**: `src/` files with `#[cfg(test)]`

**Example**:
```rust
// task_manager/src/domain/task.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_new_sets_defaults() {
        let task = Task::new("Title".to_string(), None, None, None, None);

        assert_eq!(task.status, TaskStatus::Todo);
        assert!(task.id.starts_with("task-"));
        assert!(task.created_at.len() > 0);
        assert_eq!(task.updated_at, None);
    }

    #[test]
    fn test_task_status_transitions() {
        let mut task = Task::new("Title".to_string(), None, None, None, None);

        task.status = TaskStatus::InProgress;
        assert_eq!(task.status, TaskStatus::InProgress);

        task.status = TaskStatus::Completed;
        assert_eq!(task.status, TaskStatus::Completed);
    }
}
```

### Integration Tests

**Location**: `tests/` directory

**Example**:
```rust
// task_manager/tests/sqlite_integration.rs

use task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter;
use task_manager::domain::task::Task;
use task_manager::ports::task_repository_port::TaskRepositoryPort;

#[tokio::test]
async fn test_save_and_retrieve_task() {
    // Setup
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let adapter = SqliteTaskAdapter::new(&db_path);

    // Create task
    let task = Task::new("Test task".to_string(), Some("alice".to_string()), None, None, None);
    let task_id = task.id.clone();

    // Save
    adapter.save_async(task.clone()).await.unwrap();

    // Retrieve
    let filter = TaskFilter::by_id(&task_id);
    let retrieved = adapter.find_one_async(&filter).await.unwrap();

    // Assert
    assert!(retrieved.is_some());
    let retrieved_task = retrieved.unwrap();
    assert_eq!(retrieved_task.id, task_id);
    assert_eq!(retrieved_task.title, "Test task");
    assert_eq!(retrieved_task.assignee, Some("alice".to_string()));
}
```

### End-to-End Tests

**Location**: `tests/integration_*.rs`

**Example**:
```rust
// task_orchestrator/tests/integration_end_to_end_flow.rs

use task_orchestrator::use_cases::run_task_with_flow::run_task_with_flow;
use task_manager::domain::task::Task;
use task_manager::domain::task_status::TaskStatus;

#[tokio::test]
async fn test_complex_task_orchestration() {
    // Create complex task (triggers all nodes)
    let task = Task::new(
        "Refactor authentication system".to_string(),
        Some("alice".to_string()),
        None,
        None,
        None,
    );

    // Run orchestration
    let result = run_task_with_flow("llama3.1", "short_answer", task)
        .await
        .unwrap();

    // Assert orchestration completed
    assert_eq!(result.status, TaskStatus::OrchestrationComplete);

    // Assert enhancements added
    assert!(result.enhancements.is_some());
    assert!(result.enhancements.as_ref().unwrap().len() > 0);

    // Assert tests generated
    assert!(result.comprehension_tests.is_some());
    assert!(result.comprehension_tests.as_ref().unwrap().len() > 0);

    // Assert subtasks created
    assert!(result.subtasks.is_some());
    assert!(result.subtasks.as_ref().unwrap().len() > 0);
}
```

### Mocking

Use test doubles for external dependencies:

```rust
// Mock repository for testing use cases
struct MockTaskRepository {
    tasks: std::sync::Mutex<Vec<Task>>,
}

impl TaskRepositoryPort for MockTaskRepository {
    fn save(&self, entity: Task) -> HexResult<()> {
        self.tasks.lock().unwrap().push(entity);
        Ok(())
    }

    fn find_one(&self, filter: &TaskFilter) -> HexResult<Option<Task>> {
        let tasks = self.tasks.lock().unwrap();
        Ok(tasks.iter().find(|t| filter.matches(t)).cloned())
    }

    // ...
}

#[test]
fn test_create_task_with_mock() {
    let repo = MockTaskRepository::new();
    let task = create_task(&repo, "Test".to_string(), None, None, None, None).unwrap();

    assert_eq!(repo.tasks.lock().unwrap().len(), 1);
}
```

### Coverage

Generate coverage report:

```bash
# Install cargo-llvm-cov
cargo install cargo-llvm-cov

# Generate coverage
cargo llvm-cov --workspace --html

# Open report
open target/llvm-cov/html/index.html
```

## Adding New Features

### Adding a New Adapter

**Example**: Add PostgreSQL adapter

1. **Create adapter file**: `task_manager/src/adapters/postgres_task_adapter.rs`

2. **Implement port trait**:
   ```rust
   use crate::ports::task_repository_port::TaskRepositoryPort;
   use sqlx::postgres::PgPool;

   pub struct PostgresTaskAdapter {
       pool: PgPool,
   }

   impl PostgresTaskAdapter {
       pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
           let pool = PgPool::connect(database_url).await?;
           Ok(Self { pool })
       }
   }

   #[async_trait]
   impl TaskRepositoryPort for PostgresTaskAdapter {
       async fn save_async(&self, entity: Task) -> HexResult<()> {
           sqlx::query!(
               "INSERT INTO tasks (id, title, status, ...) VALUES ($1, $2, $3, ...)",
               entity.id,
               entity.title,
               entity.status as i32,
           )
           .execute(&self.pool)
           .await?;

           Ok(())
       }

       // Implement other methods...
   }
   ```

3. **Add to module**: `task_manager/src/adapters/mod.rs`
   ```rust
   pub mod postgres_task_adapter;
   ```

4. **Add tests**: `task_manager/tests/postgres_integration.rs`

5. **Update documentation**: Mention new adapter in README and ARCHITECTURE.md

### Adding a New gRPC RPC

**Example**: Add `CompleteTask` RPC

1. **Update protobuf schema**: `rigger_cli/proto/rigger.proto`
   ```protobuf
   service RiggerService {
     // ... existing RPCs

     rpc CompleteTask(CompleteTaskRequest) returns (CompleteTaskResponse);
   }

   message CompleteTaskRequest {
     string task_id = 1;
   }

   message CompleteTaskResponse {
     Task task = 1;
   }
   ```

2. **Rebuild**: Protobuf code will auto-generate on `cargo build`

3. **Implement RPC**: `rigger_cli/src/commands/grpc_server.rs`
   ```rust
   async fn complete_task(
       &self,
       request: Request<CompleteTaskRequest>,
   ) -> Result<Response<CompleteTaskResponse>, Status> {
       let req = request.into_inner();

       // Load adapter
       let adapter = SqliteTaskAdapter::new(&self.db_path);

       // Update task status
       let task = task_manager::use_cases::manage_task::update_task_status(
           &adapter,
           &req.task_id,
           TaskStatus::Completed,
       )
       .map_err(|e| Status::internal(format!("Failed to complete task: {}", e)))?;

       // Broadcast event
       let event = TaskEvent {
           event_type: TaskEventType::StatusChanged as i32,
           task: Some(self.task_to_proto(&task)),
           timestamp: chrono::Utc::now().to_rfc3339(),
           actor: Some("system".to_string()),
           metadata: HashMap::new(),
       };
       self.broadcast_event(event);

       Ok(Response::new(CompleteTaskResponse {
           task: Some(self.task_to_proto(&task)),
       }))
   }
   ```

4. **Test**: Add test to `rigger_cli/examples/test_grpc_client.rs`

5. **Update docs**: Document in `docs/API_REFERENCE.md`

### Adding a New StateGraph Node

**Example**: Add validation node

1. **Create node file**: `task_orchestrator/src/nodes/validate_node.rs`
   ```rust
   use task_manager::domain::task::Task;
   use hexser::HexResult;

   /// Validates task prerequisites before orchestration.
   pub async fn validate_task(task: Task) -> HexResult<Task> {
       // Perform validation
       if task.title.is_empty() {
           return Err(HexError::ValidationError("Title is empty".into()));
       }

       // Update status
       let mut validated_task = task;
       validated_task.status = TaskStatus::InProgress;

       Ok(validated_task)
   }
   ```

2. **Add to module**: `task_orchestrator/src/nodes/mod.rs`
   ```rust
   pub mod validate_node;
   ```

3. **Integrate into graph**: `task_orchestrator/src/use_cases/run_task_with_flow.rs`
   ```rust
   let graph = StateGraph::new(task)
       .add_node("validate", validate_node::validate_task)
       .add_node("enhance", enhance_node::enhance_task)
       .add_node("test", test_node::generate_test)
       .add_edge("validate", "enhance")
       .add_edge("enhance", "test")
       .build();
   ```

4. **Test**: Add integration test

5. **Update docs**: Document in ARCHITECTURE.md

## Debugging

### Print Debugging

```rust
// Simple print
println!("Task ID: {}", task.id);

// Debug format
dbg!(&task);

// With logging
log::debug!("Processing task: {:?}", task);
```

### Logging

Enable debug logging:

```bash
export RUST_LOG=debug
cargo run --bin rig task list
```

Add logging to code:

```rust
use log::{debug, info, warn, error};

info!("Starting task orchestration");
debug!("Task details: {:?}", task);
warn!("LLM call took longer than expected");
error!("Failed to save task: {}", err);
```

### Debugger (LLDB/GDB)

#### VS Code

Set breakpoint in editor, then press F5.

#### Command Line

```bash
# Build with debug symbols
cargo build

# Run with debugger
rust-lldb ./target/debug/rig
(lldb) breakpoint set --name create_task
(lldb) run task add "Test"
(lldb) continue
(lldb) print task
(lldb) quit
```

### Profiling

#### CPU Profiling

```bash
# Install flamegraph
cargo install flamegraph

# Profile CLI command
cargo flamegraph --bin rig -- task list

# Open flamegraph.svg
```

#### Memory Profiling

```bash
# Install valgrind
sudo apt install valgrind

# Run with valgrind
valgrind --leak-check=full ./target/debug/rig task list
```

### Tracing

Use `tracing` crate for structured logging:

```rust
use tracing::{info, instrument};

#[instrument]
async fn enhance_task(task: &Task) -> HexResult<Enhancement> {
    info!("Enhancing task");
    // ...
}
```

Enable tracing:
```bash
export RUST_LOG=trace
cargo run
```

## Contributing

### Workflow

1. **Fork repository** on GitHub

2. **Clone fork**:
   ```bash
   git clone https://github.com/YOUR_USERNAME/rig-task-pipeline.git
   cd rig-task-pipeline
   ```

3. **Create feature branch**:
   ```bash
   git checkout -b feature/add-postgres-adapter
   ```

4. **Make changes** following code conventions

5. **Run tests**:
   ```bash
   cargo test --workspace
   cargo clippy --workspace -- -D warnings
   cargo fmt -- --check
   ```

6. **Commit changes**:
   ```bash
   git add .
   git commit -m "Add PostgreSQL adapter

   - Implement TaskRepositoryPort for PostgreSQL
   - Add integration tests
   - Update documentation"
   ```

   **Commit message format**:
   - First line: Brief summary (50 chars max)
   - Blank line
   - Detailed description (wrap at 72 chars)

7. **Push branch**:
   ```bash
   git push origin feature/add-postgres-adapter
   ```

8. **Create Pull Request** on GitHub

### Pull Request Guidelines

1. **Title**: Clear, concise summary
2. **Description**: Explain what and why
3. **Tests**: All tests must pass
4. **Documentation**: Update docs if needed
5. **Commits**: Squash if many small commits

**PR template**:
```markdown
## Summary
Brief description of changes

## Motivation
Why is this change needed?

## Changes
- Added PostgreSQL adapter
- Updated documentation
- Added integration tests

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing performed

## Checklist
- [ ] Code follows style guide
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] No breaking changes (or documented)
```

### Code Review Process

1. **Automated checks**: CI runs tests and linting
2. **Manual review**: Maintainer reviews code
3. **Address feedback**: Make requested changes
4. **Approval**: PR gets approved
5. **Merge**: Maintainer merges PR

## Release Process

### Version Numbering

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes

**Example**:
- `0.1.0` → `0.2.0`: Added gRPC server (new feature)
- `0.2.0` → `0.2.1`: Fixed database bug (patch)
- `0.2.1` → `1.0.0`: Stable API (major)

### Release Checklist

1. **Update version** in all `Cargo.toml` files:
   ```toml
   [package]
   name = "rigger_cli"
   version = "0.2.0"  # Increment version
   ```

2. **Update CHANGELOG.md**:
   ```markdown
   ## [0.2.0] - 2025-11-23

   ### Added
   - gRPC server mode with event broadcasting
   - PostgreSQL adapter

   ### Changed
   - Improved task routing logic

   ### Fixed
   - SQLite connection pool leak
   ```

3. **Run full test suite**:
   ```bash
   cargo test --workspace --release
   cargo clippy --workspace -- -D warnings
   cargo fmt -- --check
   ```

4. **Build release binary**:
   ```bash
   cargo build --release --package rigger_cli
   ```

5. **Create Git tag**:
   ```bash
   git tag -a v0.2.0 -m "Release v0.2.0"
   git push origin v0.2.0
   ```

6. **Publish to crates.io** (if applicable):
   ```bash
   cargo publish --package task_manager
   cargo publish --package task_orchestrator
   cargo publish --package rigger_cli
   ```

7. **Create GitHub release**:
   - Go to Releases → New Release
   - Tag: `v0.2.0`
   - Title: `v0.2.0`
   - Description: Paste CHANGELOG entries
   - Upload binaries (if built)

### CI/CD

**GitHub Actions** (`.github/workflows/ci.yml`):

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: cargo test --workspace
      - name: Run Clippy
        run: cargo clippy --workspace -- -D warnings
      - name: Check formatting
        run: cargo fmt -- --check

  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Build release
        run: cargo build --release --package rigger_cli
      - name: Upload artifact
        uses: actions/upload-artifact@v3
        with:
          name: rig-linux-amd64
          path: target/release/rig
```

## Best Practices

1. **Write tests first** (TDD when possible)
2. **Keep functions small** (< 50 lines)
3. **Use type system** to enforce invariants
4. **Avoid premature optimization** (measure first)
5. **Document public APIs** thoroughly
6. **Run `cargo clippy`** before committing
7. **Squash WIP commits** before merging
8. **Ask for help** when stuck
9. **Review own code** before submitting PR
10. **Be respectful** in code reviews

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [async-book](https://rust-lang.github.io/async-book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [tonic Guide](https://github.com/hyperium/tonic/blob/master/examples/README.md)
- [Hexagonal Architecture](https://alistair.cockburn.us/hexagonal-architecture/)

---

**Last updated**: 2025-11-23
