# HEXSER Refactoring Plan

## Current Status

The transcript processor has been implemented with a custom hexagonal architecture approach, but it needs to be refactored to use the **HEXSER framework** (`/squillo/hexser`) properly. Currently, the code has 69 compilation errors because it uses custom port traits instead of HEXSER's built-in patterns.

## HEXSER Framework Patterns

Based on the documentation from `/squillo/hexser`, the framework provides:

1. **Derive Macros:**
   - `#[derive(HexEntity)]` or `#[derive(Entity, HexDomain)]` for domain entities
   - `#[derive(HexPort)]` for port trait definitions
   - `#[derive(HexAdapter)]` for adapter implementations
   - `#[derive(HexRepository)]` for repository structs

2. **Built-in Traits:**
   - `hexser::ports::Repository<T>` - Standard CRUD operations
   - `hexser::ports::repository::QueryRepository<T>` - Filtering and sorting
   - `hexser::HexResult<T>` - Standard result type

3. **Prelude Import:**
   - `use hexser::prelude::*;` - Imports all necessary traits

## Required Refactoring Changes

### 1. Domain Layer (`src/domain/`)

**Current:** Plain structs with `#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]`

**Required:** Add `HexEntity` derive macro

#### Files to Modify:
- `src/domain/action_item.rs` - Add `#[derive(hexser::HexEntity)]` to ActionItem
- `src/domain/task_id.rs` - Add `#[derive(hexser::HexEntity)]` to TaskId
- `src/domain/priority.rs` - Keep as-is (enum, not an entity)
- `src/domain/task_status.rs` - Keep as-is (enum, not an entity)
- `src/domain/revision_entry.rs` - Add `#[derive(hexser::HexEntity)]`
- `src/domain/checklist_item.rs` - Add `#[derive(hexser::HexEntity)]`

### 2. Application Layer - Ports (`src/application/ports/`)

**Current:** Custom traits `TranscriptExtractorPort` and `TaskRepositoryPort`

**Required:** Refactor to extend `hexser::ports::Repository<T>` and add `#[derive(HexPort)]`

#### Changes Needed:

**OLD `src/application/ports/task_repository_port.rs`:**
```rust
pub trait TaskRepositoryPort: Send + Sync {
    fn save(&self, task: ActionItem) -> Result<(), String>;
    fn get_by_id(&self, task_id: &TaskId) -> Result<Option<ActionItem>, String>;
    fn get_all(&self) -> Result<Vec<ActionItem>, String>;
    fn get_sorted(&self, sort_by: SortBy) -> Result<Vec<ActionItem>, String>;
}
```

**NEW Pattern:**
```rust
use hexser::prelude::*;

pub enum TaskFilter {
    ById(TaskId),
    All,
}

pub enum TaskSortKey {
    DueDate,
    Priority,
}

#[derive(HexPort)]
pub trait TaskRepositoryPort:
    hexser::ports::Repository<crate::domain::action_item::ActionItem>
    + hexser::ports::repository::QueryRepository<
        crate::domain::action_item::ActionItem,
        Filter = TaskFilter,
        SortKey = TaskSortKey,
    >
{}
```

**TranscriptExtractorPort:**
- Keep as custom async trait (not a repository pattern)
- Add `#[derive(HexPort)]` attribute

### 3. Adapters Layer (`src/adapters/`)

**Current:** Manual trait implementations

**Required:** Add `#[derive(HexAdapter)]` and implement `hexser::ports::Repository<T>`

#### Changes Needed:

**`src/adapters/in_memory_task_adapter.rs`:**
```rust
use hexser::prelude::*;

#[derive(HexAdapter)]
pub struct InMemoryTaskAdapter {
    tasks: std::sync::Arc<parking_lot::Mutex<std::collections::HashMap<TaskId, ActionItem>>>,
}

impl hexser::ports::Repository<ActionItem> for InMemoryTaskAdapter {
    fn find_by_id(&self, id: &TaskId) -> hexser::HexResult<std::option::Option<ActionItem>> {
        let tasks = self.tasks.lock();
        std::result::Result::Ok(tasks.get(id).cloned())
    }
    
    fn save(&mut self, entity: ActionItem) -> hexser::HexResult<()> {
        let mut tasks = self.tasks.lock();
        tasks.insert(entity.task_id.clone(), entity);
        std::result::Result::Ok(())
    }
}

impl hexser::ports::repository::QueryRepository<ActionItem> for InMemoryTaskAdapter {
    type Filter = TaskFilter;
    type SortKey = TaskSortKey;
    
    fn find_by_filter(&self, filter: &Self::Filter) -> hexser::HexResult<std::vec::Vec<ActionItem>> {
        // Implementation
    }
    
    fn find_sorted(&self, sort_key: &Self::SortKey) -> hexser::HexResult<std::vec::Vec<ActionItem>> {
        // Implementation  
    }
}

impl TaskRepositoryPort for InMemoryTaskAdapter {}
```

**`src/adapters/ollama_adapter.rs`:**
- Add `#[derive(HexAdapter)]`
- Keep async trait implementation for TranscriptExtractorPort

### 4. Use Cases (`src/application/use_cases/`)

**Current:** Accept `Arc<dyn CustomPort>` parameters

**Required:** Update to use hexser trait bounds

#### Changes Needed:

**OLD:**
```rust
pub fn new(
    extractor: std::sync::Arc<dyn TranscriptExtractorPort>,
    task_repo: std::sync::Arc<dyn TaskRepositoryPort>,
) -> Self
```

**NEW:**
```rust
pub fn new<E, R>(extractor: std::sync::Arc<E>, task_repo: std::sync::Arc<R>) -> Self
where
    E: TranscriptExtractorPort + 'static,
    R: TaskRepositoryPort + 'static,
```

### 5. Main Binary (`src/main.rs`)

**Current:** Manual dependency injection

**Consider:** Using `hexser::hex_static!` macro for DI

```rust
let app = hexser::hex_static!({
    let task_repo = InMemoryTaskAdapter::new();
    let ollama_adapter = OllamaTranscriptExtractorAdapter::new(String::from("llama3.2"));
    let process_use_case = ProcessTranscriptUseCase::new(
        std::sync::Arc::new(ollama_adapter),
        std::sync::Arc::new(task_repo.clone())
    );
    (task_repo, process_use_case)
});
```

## Migration Strategy

### Phase 1: Domain Entities (Low Risk)
1. Add `hexser::prelude::*` imports
2. Add `#[derive(HexEntity)]` to all domain entities
3. Verify compilation

### Phase 2: Ports Refactoring (Medium Risk)
1. Refactor `TaskRepositoryPort` to extend `hexser::ports::Repository<ActionItem>`
2. Define `TaskFilter` and `TaskSortKey` enums
3. Update trait to extend `QueryRepository`

### Phase 3: Adapters Refactoring (High Risk)
1. Add `#[derive(HexAdapter)]` to adapter structs
2. Implement `hexser::ports::Repository<T>` trait
3. Implement `QueryRepository<T>` trait
4. Update method signatures to match hexser patterns

### Phase 4: Use Cases Update (Medium Risk)
1. Update constructor signatures with generic bounds
2. Update method calls to use hexser trait methods
3. Replace `Result<T, String>` with `hexser::HexResult<T>`

### Phase 5: Testing & Validation
1. Run `cargo test`
2. Fix any remaining compilation errors
3. Verify all tests pass
4. Test end-to-end with `cargo run`

## Estimated Effort

- **Domain Refactoring:** ~30 minutes (straightforward macro additions)
- **Ports Refactoring:** ~1 hour (redesigning trait structure)
- **Adapters Refactoring:** ~2 hours (reimplementing trait methods)
- **Use Cases Update:** ~1 hour (updating signatures and calls)
- **Testing & Fixes:** ~1-2 hours (debugging and validation)

**Total:** ~5-6 hours of focused refactoring work

## Recommendations

Given the extensive refactoring required:

1. **Option A: Complete Refactoring**
   - Proceed with full HEXSER alignment
   - Results in production-ready, framework-compliant code
   - Takes 5-6 hours

2. **Option B: Documentation & Handoff**
   - Document current implementation
   - Provide this refactoring plan as a roadmap
   - Note that code uses custom hexagonal pattern (not HEXSER-compliant)
   - User can complete refactoring later

3. **Option C: Hybrid Approach**
   - Complete Phase 1 (Domain entities) - low hanging fruit
   - Document remaining phases
   - Provides partial HEXSER alignment

## Decision Point

**Question for User:** Should we proceed with the full HEXSER refactoring (Option A), or would you prefer to document the current state and provide this refactoring plan for future work (Option B)?

The current implementation is architecturally sound and demonstrates hexagonal principles, but it doesn't leverage the HEXSER framework's built-in traits and macros, which is why we're seeing compilation errors.
