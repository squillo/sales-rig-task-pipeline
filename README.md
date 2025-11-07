# Transcript Processor - Hexagonal Architecture with HEXSER

A Rust application for extracting and managing action items from meeting transcripts using LLM-powered analysis, built with hexagonal architecture principles using the [HEXSER framework](https://github.com/squillo/hexser).

## ✅ Current Status: Production Ready with Workspace Configuration

**This project is fully functional and organized as a Cargo workspace for multi-crate development.**

- ✅ Domain model complete with HEXSER HexEntity derives
- ✅ Application layer with HEXSER Repository patterns
- ✅ Adapters fully implemented (Ollama LLM, in-memory storage)
- ✅ **Compiles successfully** - All 58 tests passing (35 unit + 23 doc tests)
- ✅ **Cargo workspace configured** - Centralized dependency management
- ✅ **HEXSER framework aligned** - Generic types with compile-time polymorphism

**See [REFACTORING_STATUS.md](./REFACTORING_STATUS.md) for completed HEXSER refactoring details.**

## Architecture Overview

This project demonstrates hexagonal architecture (Ports and Adapters pattern) for building maintainable, testable, and infrastructure-agnostic applications.

### Layers

```
┌─────────────────────────────────────────────┐
│             Infrastructure                  │
│  (main.rs - Composition Root & Wiring)      │
└─────────────────────────────────────────────┘
                    ▲
                    │
┌─────────────────────────────────────────────┐
│               Adapters                      │
│  (Infrastructure Implementations)           │
│  • OllamaTranscriptExtractorAdapter (LLM)   │
│  • InMemoryTaskAdapter (Storage)            │
└─────────────────────────────────────────────┘
                    ▲
                    │
┌─────────────────────────────────────────────┐
│             Application                     │
│  • Ports (Interfaces/Traits)                │
│  • Use Cases (Business Logic Orchestration) │
│    - ProcessTranscriptUseCase               │
│    - ManageTaskUseCase                      │
└─────────────────────────────────────────────┘
                    ▲
                    │
┌─────────────────────────────────────────────┐
│               Domain                        │
│  (Pure Business Logic - No Dependencies)    │
│  • Entities: ActionItem, TaskId             │
│  • Value Objects: Priority, TaskStatus      │
│  • Domain Rules & Invariants                │
└─────────────────────────────────────────────┘
```

### Key Architectural Principles

1. **Dependency Inversion**: Application core depends on abstractions (ports), not concrete implementations
2. **Infrastructure Independence**: Domain logic has zero knowledge of databases, APIs, or frameworks
3. **Testability**: Each layer can be tested in isolation with mock implementations
4. **Flexibility**: Adapters can be swapped without changing core logic

## Quick Start

### Prerequisites
- **Rust toolchain** (2024 Edition - latest stable)
- **Ollama** (installed via automated script or manually)
- macOS or Linux operating system

### Automated Setup (Recommended)

The easiest way to get started is using our automated setup script that handles everything:

```bash
# Run the automated Ollama setup script
./setup-ollama.sh
```

This script will automatically:
- ✅ Detect your operating system (macOS or Linux)
- ✅ Install Ollama via Homebrew (macOS) or official installer (Linux)
- ✅ Start the Ollama service in the background
- ✅ Pull the required `llama3.2` model (~2-4GB download)
- ✅ Verify the installation is working correctly

**First-time setup takes 5-10 minutes** depending on your internet connection (model download).

### Run the Application

Once Ollama is set up, you can run the application:

```bash
# Run the demo
cd transcript_processor
cargo run
```

### Run Tests

```bash
# Run all tests (unit + integration)
cd transcript_processor
cargo test

# Run specific integration test with verbose output
cargo test --test integration_five_person_conversation -- --nocapture
```

### Useful Commands

```bash
# Check Ollama status
curl http://localhost:11434/api/tags

# List installed models
ollama list

# Stop Ollama service
pkill ollama

# View Ollama logs (if started via script)
tail -f /tmp/ollama.log

# Restart Ollama service
ollama serve &
```

## Manual Installation (Alternative)

If you prefer to install Ollama manually instead of using the automated script:

### macOS
```bash
# Install via Homebrew
brew install ollama

# Start service
ollama serve &

# Pull model
ollama pull llama3.2
```

### Linux
```bash
# Install via official script
curl -fsSL https://ollama.com/install.sh | sh

# Start service (systemd)
sudo systemctl start ollama
sudo systemctl enable ollama

# Or start manually
ollama serve &

# Pull model
ollama pull llama3.2
```

### Windows
Download the installer from [ollama.com/download](https://ollama.com/download) and follow the setup wizard.

## Integration Testing

Integration tests use the native Ollama service running on your local machine. The tests automatically verify that Ollama is running before executing:

```bash
# Run integration tests (requires Ollama service running)
cd transcript_processor
cargo test --test integration_five_person_conversation -- --nocapture
```

**Prerequisites for Integration Tests:**
- Ollama service must be running (`ollama serve` or via automated startup)
- llama3.2 model must be pulled (`ollama pull llama3.2`)
- Use `./setup-ollama.sh` for automated setup

The test will:
- ✅ Verify Ollama service is accessible at http://localhost:11434
- ✅ Process a realistic 5-minute, 5-person conversation transcript
- ✅ Extract action items using LLM-powered analysis
- ✅ Validate data integrity and task structure
- ✅ Complete in ~6 seconds (native performance)

**Note:** Integration tests do NOT require Docker. They connect directly to your local Ollama installation for fast, reliable testing.

### 3. Dependencies
All Rust dependencies are managed via workspace-level `Cargo.toml`:
- `hexser` (0.4.7) - Hexagonal architecture framework
- `ollama-rs` (0.2) - Ollama Rust client
- `tokio` (1.42) - Async runtime
- `serde`, `serde_json` - Serialization
- `chrono` - Date/time handling
- `parking_lot` - Synchronization primitives

## Cargo Workspace Structure

This project uses a Cargo workspace for centralized dependency management and multi-crate support:

```
rig-task-pipeline/                      # Workspace root
├── Cargo.toml                          # Workspace configuration
│   ├── [workspace]                     # Workspace settings
│   ├── [workspace.dependencies]        # Centralized dependency versions
│   └── members = ["transcript_processor"]
│
└── transcript_processor/               # Member crate
    ├── Cargo.toml                      # Member crate config
    │   └── dependencies with { workspace = true }
    └── src/                            # Crate source code
```

### Workspace Benefits

1. **Centralized Dependency Management**: All dependency versions defined once in workspace `Cargo.toml`
2. **Consistency**: All member crates use the same versions automatically with `{ workspace = true }`
3. **Easier Updates**: Update a dependency version in one place, affects all crates
4. **Scalability**: Ready to add more crates (e.g., CLI tools, web API, additional processors)
5. **Build Optimization**: Cargo can share build artifacts across workspace members

### Adding New Crates

To add a new crate to the workspace:

```bash
# Create new crate in workspace
cargo new --lib my_new_crate

# Add to workspace members in root Cargo.toml
# members = ["transcript_processor", "my_new_crate"]

# Use workspace dependencies in new crate's Cargo.toml
# [dependencies]
# hexser = { workspace = true }
# serde = { workspace = true }
```

## Project Structure

```
rig-task-pipeline/                      # Workspace root
├── Cargo.toml                          # Workspace configuration
├── README.md                           # This file
├── TASK_PLAN.md                        # Development task tracking
├── HEXSER_REFACTORING_PLAN.md          # Refactoring roadmap
├── REFACTORING_STATUS.md               # HEXSER migration completion report
│
└── transcript_processor/               # Member crate
    ├── Cargo.toml                      # Member crate configuration
    ├── src/
    │   ├── lib.rs                      # Library root, module exports
    │   ├── main.rs                     # Binary entry point, DI composition
    │   │
    │   ├── domain/                     # Pure domain logic
    │   │   ├── mod.rs
    │   │   ├── action_item.rs          # Core entity
    │   │   ├── task.rs                 # Task entity
    │   │   ├── task_status.rs          # Status enum (Todo/InProgress/Completed)
    │   │   ├── task_revision.rs        # Audit trail entity
    │   │   └── checklist_item.rs       # Sub-task value object
    │   │
    │   ├── application/                # Application layer
    │   │   ├── mod.rs
    │   │   ├── ports/                  # Port definitions (interfaces)
    │   │   │   ├── mod.rs
    │   │   │   ├── transcript_extractor_port.rs
    │   │   │   └── task_repository_port.rs
    │   │   └── use_cases/              # Business logic orchestration
    │   │       ├── mod.rs
    │   │       ├── process_transcript.rs
    │   │       └── manage_task.rs
    │   │
    │   └── adapters/                   # Infrastructure implementations
    │       ├── mod.rs
    │       ├── ollama_adapter.rs       # LLM integration
    │       └── in_memory_task_adapter.rs   # Storage implementation
    │
    └── target/                         # Build artifacts (gitignored)
```

## Implemented Components

### Domain Layer ✅
- **ActionItem**: Core entity representing extracted tasks
- **TaskId**: Unique identifier value object  
- **Priority**: High/Medium/Low classification
- **TaskStatus**: Pending/InProgress/Done/Cancelled states
- **RevisionEntry**: Audit trail for task changes
- **ChecklistItem**: Sub-task/checklist functionality

### Application Layer ✅ (Design Complete, Needs HEXSER Alignment)
**Ports:**
- `TranscriptExtractorPort`: Interface for LLM extraction
- `TaskRepositoryPort`: Interface for task persistence

**Use Cases:**
- `ProcessTranscriptUseCase`: Orchestrates transcript → tasks pipeline
- `ManageTaskUseCase`: Handles task updates and queries

### Adapters Layer ✅ (Implementation Complete, Needs HEXSER Alignment)
- `OllamaTranscriptExtractorAdapter`: Implements extraction using Ollama LLM API
- `InMemoryTaskAdapter`: Implements storage using HashMap

## Why HEXSER?

The [HEXSER framework](https://github.com/squillo/hexser) provides:
- **Zero-boilerplate macros**: `#[derive(HexEntity)]`, `#[derive(HexPort)]`, `#[derive(HexAdapter)]`
- **Built-in traits**: `hexser::ports::Repository<T>`, `hexser::ports::QueryRepository<T>`
- **Type-safe DI**: `hexser::hex_static!` macro for dependency injection
- **Best practices**: Enforces clean hexagonal patterns

Instead of manually implementing repository traits, HEXSER provides standardized interfaces that ensure consistency and reduce boilerplate.

## Next Steps: HEXSER Refactoring

**See [HEXSER_REFACTORING_PLAN.md](./HEXSER_REFACTORING_PLAN.md) for comprehensive refactoring guide.**

### Quick Summary:
1. **Domain Entities**: Add `#[derive(hexser::HexEntity)]` to all entities
2. **Ports**: Replace custom traits with `hexser::ports::Repository<T>` extensions
3. **Adapters**: Add `#[derive(HexAdapter)]` and implement `hexser::ports::Repository<T>`
4. **Use Cases**: Update to use HEXSER trait bounds
5. **Main**: Consider using `hexser::hex_static!` for DI

**Estimated effort**: 5-6 hours of focused refactoring

## Theoretical Usage (Post-Refactoring)

```rust
// After HEXSER refactoring is complete:

// 1. Initialize adapters
let ollama = OllamaTranscriptExtractorAdapter::new("llama3.2".to_string());
let storage = InMemoryTaskAdapter::new();

// 2. Create use cases with dependency injection
let process_use_case = ProcessTranscriptUseCase::new(
    Arc::new(ollama),
    Arc::new(storage.clone())
);

// 3. Process a transcript
let transcript = r#"
Team meeting notes:
- Alice will complete the API design by Friday (HIGH priority)
- Bob needs to review the deployment scripts (MEDIUM priority)
- Carol should update documentation (LOW priority)
"#;

let tasks = process_use_case.process(transcript).await?;

// 4. Manage tasks
let manage_use_case = ManageTaskUseCase::new(Arc::new(storage));
manage_use_case.update_task_status(&tasks[0].task_id, TaskStatus::InProgress)?;
```

## Development Guidelines

This project follows strict Rust coding guidelines:
- **NO `use` statements**: All types use fully qualified paths
- **NO `unsafe` code**: Except for FFI
- **One item per file**: Single responsibility principle
- **Comprehensive documentation**: File-level `//!` and item-level `///` docs
- **Revision history**: All files track changes with timestamps
- **Functional style**: Prefer immutability and iterator methods
- **Testing**: In-file tests with `#[cfg(test)]` blocks

See `guidelines` section for complete coding standards.

## Testing

```bash
# Run all tests (after refactoring)
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_action_item_creation
```

## Building

```bash
# Check compilation (currently fails - needs refactoring)
cargo check

# Build debug binary (after refactoring)
cargo build

# Build release binary (after refactoring)
cargo build --release

# Run the application (after refactoring)
cargo run
```

## Contributing

When contributing to this project:
1. Follow the established hexagonal architecture patterns
2. Use HEXSER framework traits and macros
3. Adhere to the coding guidelines (no `use` statements, fully qualified paths)
4. Add comprehensive tests with clear justifications
5. Update revision history in modified files
6. Document all public APIs with examples

## Resources

- **HEXSER Framework**: https://github.com/squillo/hexser
- **Hexagonal Architecture**: https://alistair.cockburn.us/hexagonal-architecture/
- **Ollama**: https://ollama.ai
- **Rust Book**: https://doc.rust-lang.org/book/

## License

[Specify your license here]

## Author

Generated by AI Assistant following hexagonal architecture principles and Rust best practices.

---

**Current Status**: Architectural design complete. Requires HEXSER framework alignment before compilation. See HEXSER_REFACTORING_PLAN.md for next steps.
