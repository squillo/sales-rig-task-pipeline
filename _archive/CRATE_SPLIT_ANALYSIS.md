# Crate Split Analysis - transcript_processor → Multiple Crates

## Overview

This document analyzes the current `transcript_processor` crate structure and defines the strategy for splitting it into focused, independent crates following single responsibility principles.

## Target Architecture

```
rig-task-pipeline/                      # Workspace root
├── transcript_extractor/               # NEW: LLM-based extraction
│   ├── src/
│   │   ├── domain/
│   │   │   ├── action_item.rs          # Extracted action item DTO
│   │   │   └── transcript_analysis.rs  # Analysis result aggregate
│   │   ├── ports/
│   │   │   └── transcript_extractor_port.rs
│   │   ├── adapters/
│   │   │   └── ollama_adapter.rs       # Ollama LLM implementation
│   │   └── lib.rs
│   └── Cargo.toml
│
├── task_manager/                       # NEW: Task lifecycle management
│   ├── src/
│   │   ├── domain/
│   │   │   ├── task.rs                 # Main task entity
│   │   │   ├── task_status.rs          # Status enum
│   │   │   ├── task_revision.rs        # History tracking
│   │   │   ├── checklist_item.rs       # Sub-tasks
│   │   │   ├── task_sort_key.rs        # Sort criteria
│   │   │   └── sort_order.rs           # Asc/Desc enum
│   │   ├── ports/
│   │   │   └── task_repository_port.rs # Repository interface
│   │   ├── use_cases/
│   │   │   └── manage_task.rs          # Task CRUD operations
│   │   ├── adapters/
│   │   │   └── in_memory_task_adapter.rs
│   │   └── lib.rs
│   └── Cargo.toml
│
└── pipeline_integration/               # NEW: Orchestration layer
    ├── src/
    │   ├── use_cases/
    │   │   └── process_transcript.rs   # Uses both transcript_extractor & task_manager
    │   ├── lib.rs
    │   └── main.rs                     # Demo binary
    └── Cargo.toml
```

## File Mapping

### transcript_extractor Crate

| Current File | New Location | Reason |
|--------------|--------------|--------|
| `domain/action_item.rs` | `transcript_extractor/src/domain/action_item.rs` | Output DTO from LLM extraction |
| `domain/transcript_analysis.rs` | `transcript_extractor/src/domain/transcript_analysis.rs` | Aggregates extracted action items |
| `application/ports/transcript_extractor_port.rs` | `transcript_extractor/src/ports/transcript_extractor_port.rs` | Port for extraction interface |
| `adapters/ollama_adapter.rs` | `transcript_extractor/src/adapters/ollama_adapter.rs` | LLM adapter implementation |

**Dependencies:**
- `hexser` - HexEntity, HexAdapter derives
- `serde`, `serde_json` - Serialization
- `schemars` - JSON schema for ActionItem
- `ollama-rs` - Ollama client
- `async-trait` - Async port trait
- `tokio` - Async runtime

**Public API:**
```rust
// Re-exported from lib.rs
pub use domain::action_item::ActionItem;
pub use domain::transcript_analysis::TranscriptAnalysis;
pub use ports::transcript_extractor_port::TranscriptExtractorPort;
pub use adapters::ollama_adapter::OllamaTranscriptExtractorAdapter;
```

### task_manager Crate

| Current File | New Location | Reason |
|--------------|--------------|--------|
| `domain/task.rs` | `task_manager/src/domain/task.rs` | Core task entity |
| `domain/task_status.rs` | `task_manager/src/domain/task_status.rs` | Task lifecycle states |
| `domain/task_revision.rs` | `task_manager/src/domain/task_revision.rs` | Audit trail |
| `domain/checklist_item.rs` | `task_manager/src/domain/checklist_item.rs` | Sub-task tracking |
| `domain/task_sort_key.rs` | `task_manager/src/domain/task_sort_key.rs` | Query sort options |
| `domain/sort_order.rs` | `task_manager/src/domain/sort_order.rs` | Asc/Desc enum |
| `application/ports/task_repository_port.rs` | `task_manager/src/ports/task_repository_port.rs` | Repository interface |
| `application/use_cases/manage_task.rs` | `task_manager/src/use_cases/manage_task.rs` | Task CRUD operations |
| `adapters/in_memory_task_adapter.rs` | `task_manager/src/adapters/in_memory_task_adapter.rs` | In-memory storage |

**Dependencies:**
- `hexser` - Repository patterns, HexEntity derives
- `serde`, `serde_json` - Serialization
- `chrono` - Timestamps
- `uuid` - Task ID generation
- `parking_lot` - Thread-safe storage

**Public API:**
```rust
// Re-exported from lib.rs
pub use domain::task::Task;
pub use domain::task_status::TaskStatus;
pub use domain::task_revision::TaskRevision;
pub use domain::checklist_item::ChecklistItem;
pub use ports::task_repository_port::{TaskRepositoryPort, TaskFilter, TaskSortKey};
pub use use_cases::manage_task::ManageTaskUseCase;
pub use adapters::in_memory_task_adapter::InMemoryTaskAdapter;
```

### pipeline_integration Crate

| Current File | New Location | Reason |
|--------------|--------------|--------|
| `application/use_cases/process_transcript.rs` | `pipeline_integration/src/use_cases/process_transcript.rs` | Orchestrates both crates |
| `main.rs` | `pipeline_integration/src/main.rs` | Demo binary |

**Dependencies:**
- `transcript_extractor` - For extraction functionality
- `task_manager` - For task persistence
- `hexser` - Repository patterns
- `tokio` - Async runtime

**Key Integration Logic:**
```rust
// ProcessTranscriptUseCase converts ActionItem → Task
impl ProcessTranscriptUseCase<R> {
    pub async fn process(&mut self, transcript: &str) 
        -> Result<Vec<task_manager::Task>, String> 
    {
        // 1. Extract using transcript_extractor
        let analysis: transcript_extractor::TranscriptAnalysis = 
            self.extractor.extract_analysis(transcript).await?;
        
        // 2. Convert ActionItem → Task
        let tasks: Vec<task_manager::Task> = analysis.action_items
            .iter()
            .map(|action| task_manager::Task::from_action_item(action, None))
            .collect();
        
        // 3. Persist using task_manager
        for task in &tasks {
            self.task_repo.save(task.clone())?;
        }
        
        Ok(tasks)
    }
}
```

## Cross-Crate Dependencies

### Task Creation from ActionItem

**Challenge:** `Task::from_action_item()` currently takes `&ActionItem` from the same crate.

**Solution:** Move this constructor to `task_manager` and make it generic or accept a trait:

```rust
// Option A: Direct dependency (task_manager depends on transcript_extractor)
impl Task {
    pub fn from_action_item(
        action: &transcript_extractor::ActionItem,
        transcript_id: Option<String>,
    ) -> Self { ... }
}

// Option B: Builder pattern (no direct dependency)
impl Task {
    pub fn from_title_assignee_due(
        title: String,
        assignee: Option<String>,
        due_date: Option<String>,
        transcript_id: Option<String>,
    ) -> Self { ... }
}
```

**Recommendation:** Use Option A for simplicity. task_manager can depend on transcript_extractor for the ActionItem type, which is a clean data dependency.

### Dependency Graph

```
pipeline_integration
    ├── depends on transcript_extractor
    └── depends on task_manager
        └── depends on transcript_extractor (for ActionItem type only)
```

## Module Structure Changes

### Current: transcript_processor/src/lib.rs
```rust
pub mod domain;
pub mod application;
pub mod adapters;
```

### New: transcript_extractor/src/lib.rs
```rust
//! Transcript extraction library using LLM-powered analysis.

pub mod domain;
pub mod ports;
pub mod adapters;

// Re-export public API
pub use domain::action_item::ActionItem;
pub use domain::transcript_analysis::TranscriptAnalysis;
pub use ports::transcript_extractor_port::TranscriptExtractorPort;
pub use adapters::ollama_adapter::OllamaTranscriptExtractorAdapter;
```

### New: task_manager/src/lib.rs
```rust
//! Task lifecycle management library with HEXSER patterns.

pub mod domain;
pub mod ports;
pub mod use_cases;
pub mod adapters;

// Re-export public API
pub use domain::task::Task;
pub use domain::task_status::TaskStatus;
pub use domain::task_revision::TaskRevision;
pub use domain::checklist_item::ChecklistItem;
pub use ports::task_repository_port::{TaskRepositoryPort, TaskFilter, TaskSortKey};
pub use use_cases::manage_task::ManageTaskUseCase;
pub use adapters::in_memory_task_adapter::InMemoryTaskAdapter;
```

### New: pipeline_integration/src/lib.rs
```rust
//! Integration layer orchestrating transcript extraction and task management.

pub mod use_cases;

pub use use_cases::process_transcript::ProcessTranscriptUseCase;
```

## Import Path Changes

### Example: OllamaTranscriptExtractorAdapter

**Before:**
```rust
// In process_transcript.rs
use crate::domain::transcript_analysis::TranscriptAnalysis;
use crate::application::ports::transcript_extractor_port::TranscriptExtractorPort;
```

**After:**
```rust
// In pipeline_integration/src/use_cases/process_transcript.rs
use transcript_extractor::{TranscriptAnalysis, TranscriptExtractorPort};
```

## Workspace Cargo.toml Updates

```toml
[workspace]
members = [
    "transcript_extractor",
    "task_manager",
    "pipeline_integration",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2024"
authors = ["AI Assistant"]
license = "MIT"

[workspace.dependencies]
# Shared dependencies remain the same
hexser = { git = "https://github.com/squillo/hexser", branch = "main" }
serde = { version = "1.0", features = ["derive"] }
# ... etc
```

## Testing Strategy

### Unit Tests
- **transcript_extractor**: 5 tests (action_item, transcript_analysis, ollama_adapter)
- **task_manager**: 23 tests (all domain entities, repository, use case)
- **pipeline_integration**: 1 integration test (full pipeline)

### Independent Compilation
```bash
# Test each crate independently
cd transcript_extractor && cargo test
cd task_manager && cargo test
cd pipeline_integration && cargo test

# Test full workspace
cd .. && cargo test
```

## Migration Checklist

- [ ] Create transcript_extractor crate structure
- [ ] Move and update files to transcript_extractor
- [ ] Update imports in transcript_extractor to remove crate:: prefix
- [ ] Build transcript_extractor independently
- [ ] Run transcript_extractor tests
- [ ] Create task_manager crate structure
- [ ] Move and update files to task_manager
- [ ] Add transcript_extractor dependency to task_manager
- [ ] Update Task::from_action_item to use transcript_extractor::ActionItem
- [ ] Update imports in task_manager
- [ ] Build task_manager independently
- [ ] Run task_manager tests
- [ ] Create pipeline_integration crate structure
- [ ] Move process_transcript use case to pipeline_integration
- [ ] Move main.rs to pipeline_integration
- [ ] Update imports to use external crate names
- [ ] Build pipeline_integration
- [ ] Run integration tests
- [ ] Update workspace Cargo.toml
- [ ] Remove old transcript_processor directory
- [ ] Build full workspace
- [ ] Run full test suite (should be 58 tests total)
- [ ] Update README.md
- [ ] Update TASK_PLAN.md

## Benefits of This Split

1. **Clear Separation of Concerns**: Extraction logic is completely separate from task management
2. **Reusability**: Each crate can be used independently in other projects
3. **Focused Testing**: Each crate has its own focused test suite
4. **Easier Maintenance**: Changes to extraction don't affect task management and vice versa
5. **Future Extensibility**: Easy to add new extractors or new storage backends
6. **Workspace Benefits**: Shared dependencies, unified build, consistent versioning

## Potential Issues & Solutions

### Issue 1: Circular Dependencies
**Risk:** If task_manager depends on transcript_extractor and vice versa.
**Solution:** Only task_manager depends on transcript_extractor (for ActionItem type). The dependency is one-way.

### Issue 2: Duplicate Code
**Risk:** Common utilities might need to be duplicated.
**Solution:** If common utilities emerge, create a shared `common` crate later.

### Issue 3: Test Data Setup
**Risk:** Integration tests need both crates working together.
**Solution:** pipeline_integration crate has integration tests that verify the full pipeline.

## Next Steps

1. Begin with transcript_extractor (smallest, no dependencies on task_manager)
2. Then task_manager (depends on transcript_extractor only for ActionItem)
3. Finally pipeline_integration (depends on both)
4. Verify all 58 tests still pass
5. Update documentation

---

**Analysis Complete**: Ready to begin implementation starting with transcript_extractor crate.
