# task_manager

Task Manager crate: provides domain entities, ports, use cases, and adapters for task persistence and lifecycle operations, aligned to HEXSER and DPAI layering. Persistence now includes a SQLite adapter using sqlx with robust error mapping.

## Rigger Integration (Phase 0)

This crate now includes Rigger-compatible entities for AI-driven project management:
- **PRD (Product Requirements Document)**: Parsed markdown PRDs with structured sections
- **ProjectContext**: Synthesized codebase analysis for context-aware task generation
- **Task Extensions**: Support for task hierarchies (parent/subtask) and PRD linkage

## Knowledge Graph

- Crate: task_manager
  - lib (crate root)
    - mod domain
      - task (Struct: Task; derives serde, HexEntity; from_action_item constructor)
        - **NEW Rigger fields**: source_prd_id, parent_task_id, subtask_ids
      - task_status (Enum: TaskStatus)
      - enhancement (Struct: Enhancement)
      - comprehension_test (Struct: ComprehensionTest)
      - **prd (Struct: PRD; new() constructor)**
      - **project_context (Struct: ProjectContext; new() and synthesize_context() methods)**
      - task_revision (Struct: TaskRevision)
      - checklist_item (Struct: ChecklistItem)
      - task_sort_key (Enum: TaskSortKey)
      - sort_order (Enum: SortOrder)
    - mod ports
      - task_repository_port (Trait alias: TaskRepositoryPort; Enums: TaskFilter, TaskSortKey)
    - mod use_cases
      - manage_task (Struct: ManageTaskUseCase<R>; methods: new, update_task_status, get_sorted_tasks)
    - mod adapters
      - in_memory_task_adapter (Struct: InMemoryTaskAdapter; implements Repository + QueryRepository + TaskRepositoryPort)
      - sqlite_task_adapter (Struct: SqliteTaskAdapter; implements Repository + QueryRepository + TaskRepositoryPort; async connect_and_init helper)
        - **Updated schema**: Added source_prd_id, parent_task_id, subtask_ids_json columns
    - mod utils
      - **prd_parser (parse_prd_markdown function; supports markdown sections, bullet lists, numbered lists)**
      - parse_action_items_tolerant (tolerant JSON parser)
      - extracted_action_item (ExtractedActionItem struct)
      - action_item_schema (JSON schema for LLM prompts)

## Architecture (DPAI)

- **Core Domain**: Task, TaskStatus, PRD, ProjectContext, Enhancement, ComprehensionTest, TaskRevision, ChecklistItem
- **Ports**: TaskRepositoryPort (extends HEXSER Repository and QueryRepository)
- **Adapters**: InMemoryTaskAdapter, SqliteTaskAdapter (sqlx with Rigger field support)
- **Application**: ManageTaskUseCase orchestrates persistence operations
- **Infrastructure**: SQLite via sqlx; error handling via hexser::error adapter_error mapping
- **Utilities**: PRD markdown parser, tolerant JSON parser for LLM outputs

## Rigger Entities

### PRD (Product Requirements Document)
- **Location**: `src/domain/prd.rs`
- **Purpose**: Represents parsed PRD markdown files with structured sections
- **Fields**:
  - `id`: Unique identifier (UUID v4)
  - `title`: Project/product title (extracted from first `#` header)
  - `objectives`: List of project objectives (from `## Objectives` section)
  - `tech_stack`: Technologies and frameworks (from `## Tech Stack` section)
  - `constraints`: Requirements and limitations (from `## Constraints` section)
  - `raw_content`: Original markdown content for reference
  - `created_at`: UTC timestamp
- **Constructor**: `PRD::new(title, objectives, tech_stack, constraints, raw_content)`
- **Parser**: `task_manager::utils::prd_parser::parse_prd_markdown(content) -> Result<PRD, String>`

### ProjectContext
- **Location**: `src/domain/project_context.rs`
- **Purpose**: Synthesized codebase analysis for context-aware task generation
- **Fields**:
  - `id`: Unique identifier (UUID v4)
  - `project_root`: Absolute path to project directory
  - `detected_languages`: Programming languages found in codebase
  - `detected_frameworks`: Frameworks/libraries identified
  - `key_directories`: Important directories with descriptions
  - `key_files`: Critical files with roles
  - `architectural_patterns`: Detected patterns (e.g., "Hexagonal Architecture", "MVC")
  - `entry_points`: Main application entry points (e.g., "src/main.rs")
  - `created_at`: UTC timestamp
- **Constructors**:
  - `ProjectContext::new(...)`: Manual construction with specified fields
  - `ProjectContext::synthesize_context(project_root) -> Result<ProjectContext, String>`: Analyzes codebase (placeholder in Phase 0)

### Task (Extended for Rigger)
- **Rigger Fields**:
  - `source_prd_id: Option<String>`: Links task to PRD that generated it
  - `parent_task_id: Option<String>`: Parent task ID for subtask hierarchies
  - `subtask_ids: Vec<String>`: List of decomposed subtask IDs
- **SQLite Schema**: Persists Rigger fields via JSON (subtask_ids_json) and TEXT columns

## PRD Parser

- **Function**: `parse_prd_markdown(content: &str) -> Result<PRD, String>`
- **Capabilities**:
  - Parses markdown with `## Objectives`, `## Tech Stack`, `## Constraints` sections
  - Supports bullet lists (`-`, `*`)
  - Supports numbered lists (`1.`, `1)`)
  - Extracts title from first `#` header (not `##`)
  - Validates presence of title (returns error if missing)
  - Preserves raw markdown content
- **Example**:
  ```rust
  let markdown = r#"
  # Build Rigger Platform

  ## Objectives
  - Enable AI agent task decomposition
  - Support multiple LLM providers

  ## Tech Stack
  - Rust
  - Rig framework

  ## Constraints
  - Must compile with Rust 2024 edition
  "#;

  let prd = parse_prd_markdown(markdown).unwrap();
  assert_eq!(prd.title, "Build Rigger Platform");
  assert_eq!(prd.objectives.len(), 2);
  ```

## SQLite Schema (Updated for Rigger)

```sql
CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    assignee TEXT NULL,
    due_date TEXT NULL,
    status TEXT NOT NULL,
    source_transcript_id TEXT NULL,
    source_prd_id TEXT NULL,              -- NEW: Link to PRD
    parent_task_id TEXT NULL,             -- NEW: Parent task for hierarchies
    subtask_ids_json TEXT NULL,           -- NEW: JSON array of subtask IDs
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    enhancements_json TEXT NULL,
    comprehension_tests_json TEXT NULL
)
```

## Status

### Completed (Phase 0 Sprint 0.1)
- ✅ PRD domain entity with UUID and timestamps
- ✅ PRD markdown parser supporting bullets and numbered lists
- ✅ Task entity extended with source_prd_id, parent_task_id, subtask_ids
- ✅ SQLite adapter updated for Rigger fields with JSON serialization
- ✅ ProjectContext domain entity with synthesize_context() placeholder
- ✅ All tests passing (50 tests total including new PRD and ProjectContext tests)

### In Progress
- SQLite adapter implemented with UPSERT save and flexible find/find_one (filters, sorting, paging)
- Unit tests cover save, find_one (ById/ByStatus), sorted pagination, and update flow
- Doc example demonstrates ManageTaskUseCase with SQLite adapter using a current-thread Tokio runtime (no-run)

### Next (Phase 0 Sprint 0.2)
- CLI interface for PRD ingestion
- Task decomposition service
- Dependency graph generation

## Testing

Run all tests:
```bash
cargo test -p task_manager
```

Run Rigger-specific tests:
```bash
cargo test -p task_manager domain::prd
cargo test -p task_manager domain::project_context
cargo test -p task_manager utils::prd_parser
cargo test -p task_manager adapters::sqlite_task_adapter
```

## Coding Standards

- **No `use` statements**: All paths are fully qualified within crate modules
- **One logical item per file**: Each file contains a single struct/enum with its impls
- **Revision history**: Every file has timestamped revision log in doc comments
- **Comprehensive tests**: All public APIs have unit tests with justification comments
- **No `unsafe` code**: Except for FFI where explicitly documented
- **Error handling**: Use hexser::error adapter_error helpers and propagate via HexResult

## Dependencies

- `hexser`: Hexagonal architecture framework (HexEntity, Repository, QueryRepository)
- `sqlx`: Async SQLite database access with compile-time query validation
- `serde`/`serde_json`: Serialization for JSON fields and status enums
- `chrono`: UTC timestamp handling
- `uuid`: Unique identifier generation (v4)
- `transcript_extractor`: ActionItem DTOs for task creation
- `tokio`: Async runtime for SQLite adapter

## Future Work (Rigger Roadmap)

### Phase 0 Sprint 0.2
- CLI interface for PRD ingestion
- Task decomposition service
- Dependency graph generation

### Phase 0 Sprint 0.3
- Rig integration for LLM-based task generation
- Complexity scoring
- Task prioritization

### Phase 1
- Orchestration graph-flow
- Enhancement generation
- Comprehension test creation

See workspace-level `TASK_PLAN.md` for full roadmap.
