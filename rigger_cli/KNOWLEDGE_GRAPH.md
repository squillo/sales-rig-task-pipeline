# Rigger CLI - Comprehensive Knowledge Graph

Complete architectural documentation for the Rigger CLI Terminal User Interface (TUI) and command-line tools.

**Last Updated**: 2025-11-25T13:30:00Z

---

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [TUI Application State](#tui-application-state)
- [Command Structure](#command-structure)
- [Setup Wizard](#setup-wizard)
- [Main Views](#main-views)
- [Dialogs and Overlays](#dialogs-and-overlays)
- [Keyboard Shortcuts](#keyboard-shortcuts)
- [Data Flow](#data-flow)
- [Hexagonal Architecture Layers](#hexagonal-architecture-layers)
- [Error Handling](#error-handling)

---

## Overview

**Rigger CLI** (`rig`) is a terminal-based project management tool for AI agents. It provides:

- **Interactive TUI** with Kanban boards, task management, and LLM chat
- **Setup wizard** for first-time initialization with per-slot LLM configuration
- **PRD parsing** to generate tasks from Product Requirement Documents
- **Task orchestration** through LLM-powered workflows
- **gRPC and MCP servers** for IDE and distributed integration
- **SQLite persistence** for projects, PRDs, and tasks
- **Intelligent diagnostics** for troubleshooting LLM provider issues

---

## Architecture

### Project Structure

```
rigger_cli/
├── src/
│   ├── main.rs              # CLI entry point with clap command routing
│   ├── lib.rs               # Library exports for hexagonal modules
│   ├── commands/            # Command implementations
│   │   ├── mod.rs           # CLI structure and subcommand definitions
│   │   ├── init.rs          # 'rig init' - initialize .rigger directory
│   │   ├── parse.rs         # 'rig parse' - parse PRD and generate tasks
│   │   ├── list.rs          # 'rig list' - display tasks in table format
│   │   ├── do_task.rs       # 'rig do' - execute task via orchestrator
│   │   ├── server.rs        # 'rig server' - MCP stdio server mode
│   │   ├── grpc_server.rs   # 'rig grpc' - gRPC sidecar server mode
│   │   └── tui.rs           # 'rig tui' - Terminal User Interface (MAIN - 8000+ lines)
│   ├── display/             # Display formatters
│   │   ├── mod.rs
│   │   └── task_table.rs    # Table rendering for 'rig list'
│   ├── ports/               # Hexagonal architecture: abstract interfaces
│   │   ├── mod.rs
│   │   └── clipboard_port.rs  # ClipboardPort trait
│   ├── adapters/            # Hexagonal architecture: concrete implementations
│   │   ├── mod.rs
│   │   └── arboard_clipboard_adapter.rs  # Arboard clipboard implementation
│   └── services/            # Business logic services
│       ├── mod.rs
│       └── task_formatter.rs  # Format tasks as Markdown/plain text
└── build.rs                 # Protobuf compilation for gRPC
```

### Technology Stack

| Component | Technology | Version |
|-----------|------------|---------|
| TUI Framework | ratatui + crossterm | v0.29 / v0.28 |
| CLI Parsing | clap (derive macros) | v4.4 |
| Persistence | sqlx (SQLite) | workspace |
| Clipboard | arboard | v3.4 |
| Async Runtime | tokio | v1.41 |
| Serialization | serde + serde_json | v1.0 |
| gRPC | tonic + prost | v0.12 / v0.13 |
| Architecture | hexser framework | v0.4.7 |

---

## TUI Application State

The `App` struct (tui.rs:272-457) is the central state machine. Key categories:

### 1. Project Hierarchy

```rust
projects: Vec<Project>                    // All projects from database
selected_project_id: Option<String>       // Currently active project (filters PRDs/tasks)
tasks: Vec<Task>                          // Filtered by selected project
prds: Vec<PRD>                            // Filtered by selected project
```

**Entity Relationships**:
```
Project (1) ──┬── (n) PRD ──┬── (n) Task
              │              │
              └──────────────┴── Task.source_prd_id (FK)
```

### 2. Navigation State

```rust
selected_workspace: WorkspaceSection      // Tasks | PRDs | Projects
active_tool: DashboardTool                // Kanban | TaskEditor | LLMChat | Metrics | DevTools
nav_selection: usize                      // Selected item in left nav panel
show_details_panel: bool                  // Toggle right details column
```

**Layout Structure**:
```
┌──────────┬────────────────────┬─────────────┐
│   NAV    │    MAIN VIEW       │  DETAILS    │
│          │                    │ (optional)  │
│ PROJECT  │  [Active Tool]     │             │
│  LIST    │  Kanban / Metrics  │ Task Info   │
│          │  / Chat / Editor   │             │
│  TOOLS   │                    │             │
│  - Kanban│                    │             │
│  - Editor│                    │             │
│  - Chat  │                    │             │
│  - DevTls│                    │             │
└──────────┴────────────────────┴─────────────┘
      ↑              ↑                 ↑
   Tab/w/e      Active tool          'd' key
```

### 3. Kanban Board State

```rust
selected_column: KanbanColumn             // Todo | InProgress | Completed | Archived | Errored
selected_task_in_column: usize            // Index within column
current_sort: TaskSortOption              // CreatedNewest | UpdatedRecent | Title | Complexity
```

**Column Mapping**:
| Column | Status | Keyboard | Icon |
|--------|--------|----------|------|
| Todo | TaskStatus::Todo | F1 | 📋 |
| In Progress | TaskStatus::InProgress | F2 | 🔄 |
| Completed | TaskStatus::Completed | F3 | ✓ |
| Archived | TaskStatus::Archived | F4 | 📦 |
| Errored | TaskStatus::Errored | F5 | 🔴 |

**Kanban Layout**:
```
┌──────────────┬──────────────┬──────────────┬─────────────┐
│  📋 Todo     │ 🔄 InProgress│  ✓ Completed │ 📦 Archived │
│   (F1)       │   (F2)       │   (F3)       │   (F4)      │
│              │              │              │ ─────────── │
│ [Task 1] ←   │ [Task 3]     │ [Task 5]     │ 🔴 Errored  │
│ [Task 2]     │ [Task 4]     │ [Task 6]     │   (F5)      │
└──────────────┴──────────────┴──────────────┴─────────────┘
    ↑ selected
```

### 4. Setup Wizard State

```rust
setup_wizard_active: bool                 // Is wizard running?
setup_wizard_step: SetupWizardStep        // Current screen

// Per-slot configuration (main, research, fallback)
setup_wizard_main_provider: LLMProvider
setup_wizard_main_provider_selection: usize
setup_wizard_main_model: String

setup_wizard_research_provider: LLMProvider
setup_wizard_research_provider_selection: usize
setup_wizard_research_model: String

setup_wizard_fallback_provider: LLMProvider
setup_wizard_fallback_provider_selection: usize
setup_wizard_fallback_model: String

setup_wizard_db_path: String              // Database file path
```

**Wizard Flow**:
```
Welcome
  ↓ Enter
TaskToolSlots (explains main/research/fallback)
  ↓ Enter
ConfigureMainSlot (provider ←/→, model text input)
  ↓ Enter
ConfigureResearchSlot (provider ←/→, model text input)
  ↓ Enter
ConfigureFallbackSlot (provider ←/→, model text input)
  ↓ Enter
DatabaseConfiguration (path text input or default)
  ↓ Enter
Confirmation (review all settings)
  ↓ Enter
Complete (success + next steps)
  ↓ Enter
→ Main TUI (load data)
```

**LLM Provider Defaults**:
```rust
LLMProvider::Ollama   => "llama3.2:latest"
LLMProvider::Candle   => "microsoft/Phi-3.5-mini-instruct"
LLMProvider::Mistral  => "microsoft/Phi-3.5-mini-instruct"
LLMProvider::Rig      => "gpt-4o-mini"
```

**Exit Behavior**:
- **Esc or Ctrl+C** at ANY step → Exit wizard + Quit app
- No partial configuration saved

### 5. Dialog State

Each dialog has dedicated state fields:

| Dialog | State Fields | Trigger | Purpose |
|--------|--------------|---------|---------|
| Task Editor | `show_task_editor_dialog`, `task_editor_field`, `task_editor_input` | Enter (on task) | Edit selected task |
| Task Creator | `show_task_creator_dialog`, `task_creator_*` | 'a' | Create new task |
| LLM Chat | `show_llm_chat_dialog`, `llm_chat_input`, `llm_chat_history` | 'l' | Ask LLM with context |
| PRD Management | `show_prd_dialog`, `selected_prd` | 'r' | View PRDs for project |
| Spotlight Search | `show_spotlight_dialog`, `spotlight_query`, `spotlight_results` | '/' | Global fuzzy search |
| Jump to Task | `show_jump_dialog`, `jump_input` | 'g' | Quick jump by ID |
| Recent Items | `show_recent_dialog`, `recent_task_ids` | Ctrl+R | MRU cache (max 10) |
| Notifications | `show_notifications`, `notifications` (max 50) | 'n' | Event history log |
| Confirmation | `show_confirmation_dialog`, `confirmation_action` | (triggered) | Destructive ops |
| Shortcuts | `show_shortcuts` | '?' | Help overlay |
| Sort Menu | `show_sort_menu`, `sort_menu_selection` | 'o' | Task sorting |
| Markdown Browser | `show_markdown_browser`, `markdown_files` | 'm' | Select PRD file |
| PRD Processing | `show_prd_processing`, `prd_processing_*` | (auto) | LLM task gen |

### 6. Dev Tools State

```rust
active_dev_tool: Option<DevTool>          // SqliteBrowser | ConfigViewer

// SQLite Browser
db_tables: Vec<String>                    // Table names
db_table_data: Vec<HashMap<String, String>>  // Current table rows
db_table_columns: Vec<String>             // Column names
db_current_page: usize                    // Pagination
db_rows_per_page: usize                   // Default: 20

// Config Viewer
config_editor_items: Vec<(String, String)>  // Key-value pairs
config_editor_selected: usize             // Selected item
config_editor_editing: Option<ConfigEditorField>  // Key | Value
config_editor_buffer: String              // Edit buffer
```

**SQLite Browser Features**:
- Browse tables (Up/Down navigation)
- View table contents (Enter key)
- Pagination (PgUp/PgDn)
- Execute SQL queries ('q' key)
- Empty state guidance

**Config Viewer Features**:
- Edit `.rigger/config.json` key-value pairs
- Navigate with Up/Down
- Enter to edit key or value
- Changes saved immediately

### 7. Loading & Persistence State

```rust
is_loading: bool                          // Async operation in progress?
loading_message: Option<String>           // "Loading tasks..."
loading_frame: usize                      // Spinner animation (0-7)

last_saved_at: Option<DateTime<Utc>>      // Last DB save timestamp
is_saving: bool                           // Save in progress?
has_unsaved_changes: bool                 // Dirty flag

db_adapter: Option<SqliteTaskAdapter>     // Database connection pool
```

**Loading Spinner Animation**:
```
Frames: ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"]
Updates every 100ms (frame counter % 8)
```

### 8. Markdown & PRD Processing State

```rust
// Markdown Browser
show_markdown_browser: bool
markdown_files: Vec<String>               // *.md files in current dir
markdown_selected: usize

// PRD Processing View
show_prd_processing: bool
prd_processing_file: String               // File being processed
prd_processing_step: String               // Current step message
prd_tasks_generated: usize                // Task count
prd_processing_complete: bool             // Success flag
prd_processing_error: Option<String>      // Error with diagnostics
```

**Processing Steps**:
1. Reading PRD file...
2. Parsing PRD structure...
3. Loading config...
4. Generating tasks using [provider] with [model]...
5. Saving tasks to database...
6. ✓ Complete! Generated X tasks

**On Error**:
- Shows error message
- Runs `diagnose_ollama_error()` if provider = Ollama
- Provides specific fixes based on actual system state

### 9. Notification System

```rust
notifications: Vec<Notification>          // Max 50, newest first
show_notifications: bool

struct Notification {
    timestamp: DateTime<Utc>,
    level: NotificationLevel,             // Info | Success | Warning | Error
    message: String,
}
```

**Notification Levels**:
- **Info** (ℹ️ ) - Blue - General information
- **Success** (✅) - Green - Successful operations
- **Warning** (⚠️ ) - Yellow - Non-critical issues
- **Error** (🔴) - Red - Failures and exceptions

**Auto-pruning**:
- When adding 51st notification, oldest is removed
- Ensures bounded memory usage

---

## Command Structure

### CLI Commands (clap)

```rust
pub enum Commands {
    Init,                                 // Initialize .rigger directory
    Parse { prd_file: String },           // Parse PRD → generate tasks
    List { status, assignee, sort, ... }, // List tasks with filters
    Do { task_id: String },               // Execute task via orchestrator
    Server,                               // MCP stdio server mode
    Grpc { port: u16 },                   // gRPC server (default 50051)
    Tui,                                  // Launch TUI (main interface)
}
```

### Command Implementations

#### `rig init`

**Location**: `rigger_cli/src/commands/init.rs`

**Actions**:
1. Creates `.rigger/` directory
2. Creates `.rigger/prds/` subdirectory
3. Creates `.rigger/config.json` with defaults:
   ```json
   {
     "provider": "ollama",
     "model": {
       "main": "llama3.2:latest",
       "research": "llama3.2:latest",
       "fallback": "llama3.2:latest"
     },
     "database_url": "sqlite:.rigger/tasks.db"
   }
   ```
4. Initializes `.rigger/tasks.db` SQLite database
5. Creates tables: `tasks`, `prds`, `projects`

**Error Handling**:
- If `.rigger/` already exists → Error
- If DB creation fails → Error with details

#### `rig parse <PRD_FILE>`

**Location**: `rigger_cli/src/commands/parse.rs`

**Workflow**:
1. Check `.rigger/` exists (else error: "Run 'rig init' first")
2. Read PRD file
3. Parse markdown structure using `markdown_parsers::prd_parser`
4. Load config to determine provider
5. Create `RigPRDParserAdapter` for provider
6. Generate tasks via LLM
7. Save tasks to SQLite
8. Print summary (objectives, tech stack, tasks generated)

**Ollama Provider Default**: `llama3.2:latest`

**Error Handling**:
- PRD file not found → Error
- LLM request fails → Error with `diagnose_ollama_error()` output

#### `rig list`

**Location**: `rigger_cli/src/commands/list.rs`

**Options**:
```bash
--status <status>       Filter by status (e.g., "todo", "in_progress")
--assignee <name>       Filter by assignee
--sort <field>          Sort by priority | created_at | due_date
--limit <n>             Limit results
--offset <n>            Pagination offset
```

**Output**: ASCII table using `prettytable-rs`

#### `rig do <TASK_ID>`

**Location**: `rigger_cli/src/commands/do_task.rs`

**Workflow**:
1. Look up task by ID
2. Execute via `task_orchestrator::run_task_with_flow`
3. Show real-time progress
4. Update task status in database

#### `rig server`

**Location**: `rigger_cli/src/commands/server.rs`

**Purpose**: MCP (Model Context Protocol) server for IDE integration

**Transport**: stdio (reads stdin, writes stdout)

#### `rig grpc`

**Location**: `rigger_cli/src/commands/grpc_server.rs`

**Purpose**: gRPC server for distributed sidecar integration

**Default Port**: 50051

**RPC Methods**: Task management operations

#### `rig tui`

**Location**: `rigger_cli/src/commands/tui.rs` (8000+ lines)

**Startup Logic**:
```rust
async fn execute() -> anyhow::Result<()> {
    let current_dir = std::env::current_dir()?;
    let config_path = current_dir.join(".rigger/config.json");

    let mut app = App::new();

    if config_path.exists() {
        // Load existing project
        app.load_projects().await?;
        app.load_tasks().await?;
    } else {
        // Activate setup wizard
        app.setup_wizard_active = true;
        app.setup_wizard_step = SetupWizardStep::Welcome;
    }

    // Enter TUI event loop
    run_app(&mut terminal, &mut app).await?;

    Ok(())
}
```

---

## Setup Wizard

### Wizard Steps

```rust
enum SetupWizardStep {
    Welcome,                              // Introduction
    TaskToolSlots,                        // Explain slot purposes
    ConfigureMainSlot,                    // Main tool config
    ConfigureResearchSlot,                // Research tool config
    ConfigureFallbackSlot,                // Fallback tool config
    DatabaseConfiguration,                // DB path
    Confirmation,                         // Review settings
    Complete,                             // Success message
}
```

### Task Tool Slots Explained

**Purpose of Each Slot**:

- **🔧 Main**: Primary task execution and code generation
  - Standard task orchestration
  - Default: Ollama with llama3.2:latest
  - Used most frequently

- **🔍 Research**: Deep analysis and context gathering
  - Comprehension tests
  - Enhancement generation
  - Can use more powerful model (e.g., gpt-4o)

- **🛟 Fallback**: Backup when primary fails
  - Ensures robustness
  - Often embedded model (Candle) for offline capability
  - Kicks in on errors

### Per-Slot Configuration Example

**User selects**:
- Main: Ollama / llama3.2:latest
- Research: Rig / gpt-4o
- Fallback: Candle / microsoft/Phi-3.5-mini-instruct

**Generated `.rigger/config.json`**:
```json
{
  "provider": "ollama",
  "task_tools": {
    "main": {
      "provider": "ollama",
      "model": "llama3.2:latest"
    },
    "research": {
      "provider": "rig",
      "model": "gpt-4o"
    },
    "fallback": {
      "provider": "candle",
      "model": "microsoft/Phi-3.5-mini-instruct"
    }
  },
  "model": {
    "main": "llama3.2:latest",
    "research": "gpt-4o",
    "fallback": "microsoft/Phi-3.5-mini-instruct"
  },
  "database_url": "sqlite:.rigger/tasks.db"
}
```

### Wizard Keyboard Controls

| Key | Action | Notes |
|-----|--------|-------|
| Enter | Next step / Confirm | Advances wizard |
| Backspace | Delete character | Text input fields only |
| ← / → | Navigate providers | Provider selection |
| ↑ / ↓ | Navigate providers | Same as ←/→ |
| Esc | Exit wizard | Quits app entirely |
| Ctrl+C | Exit wizard | Quits app entirely |

**Important**: Esc and Ctrl+C work at ANY step to immediately exit wizard and quit app.

### Completion Flow

```rust
async fn setup_wizard_complete(&mut self) -> anyhow::Result<()> {
    // 1. Create directories
    std::fs::create_dir(".rigger")?;
    std::fs::create_dir(".rigger/prds")?;

    // 2. Write config.json
    let config = serde_json::json!({ ... });
    std::fs::write(".rigger/config.json", serde_json::to_string_pretty(&config)?)?;

    // 3. Initialize database
    let db_url = "sqlite:.rigger/tasks.db";
    SqliteTaskAdapter::connect_and_init(&db_url).await?;

    // 4. Move to Complete screen
    self.setup_wizard_step = SetupWizardStep::Complete;

    Ok(())
}

async fn setup_wizard_exit(&mut self) -> anyhow::Result<()> {
    self.setup_wizard_active = false;

    // Load data from newly created database
    self.load_projects().await?;
    self.load_tasks().await?;

    self.add_notification(NotificationLevel::Success, "Setup complete!");

    Ok(())
}
```

---

## Main Views

### 1. Kanban Board

**Dashboard Tool**: `DashboardTool::Kanban`

**Layout**: 4 columns + vertical split for Archived/Errored

```
┌──────────────┬──────────────┬──────────────┬─────────────┐
│  📋 Todo     │ 🔄 InProgress│  ✓ Completed │ 📦 Archived │
│              │              │              │ ─────────── │
│  [Task 1]    │  [Task 3]    │  [Task 5]    │ 🔴 Errored  │
│  [Task 2]    │  [Task 4]    │  [Task 6]    │             │
│              │              │              │ [Task 7]    │
└──────────────┴──────────────┴──────────────┴─────────────┘
```

**Task Card Format**:
```
╭─────────────────────────────╮
│ [task-123]                  │
│ Implement authentication    │
│ ─────────────────────────── │
│ 🕒 2d | 🔸 7/10             │
│ 👤 Alice                    │
╰─────────────────────────────╯
```

**Age Indicators**:
- 🕒 Fresh (< 1 day)
- ⏰ Recent (1-3 days)
- ⚠️  Aging (3-7 days)
- 🔴 Stale (> 7 days)

**Complexity Badges**:
- 🔹 Low (1-3)
- 🔸 Medium (4-7)
- 🔺 High (8-10)

**Column Selection (F1-F5)**:
- Highlights selected column
- Shows tasks matching column status
- Up/Down to navigate within column
- Enter to edit selected task

### 2. Metrics View

**Dashboard Tool**: `DashboardTool::Metrics`

**Displays**:
- Task completion rate (%)
- Average task age (days)
- Complexity distribution (chart)
- Assignee workload (tasks per person)
- Status breakdown (pie chart)

### 3. Dev Tools View

**Dashboard Tool**: `DashboardTool::DevTools`

**Opens submenu**:
```
┌─────────────────────────────────┐
│    🔧 Developer Tools           │
├─────────────────────────────────┤
│  → 🗄️  SQLite Browser           │
│    ⚙️  Config Viewer            │
└─────────────────────────────────┘
```

#### SQLite Browser

**Layout**:
```
┌────────────────────┬───────────────────────────────────────┐
│  Tables:           │  Table: tasks                         │
│  → tasks           │  ┌────────┬─────────┬────────┐        │
│    prds            │  │ id     │ title   │ status │        │
│    projects        │  ├────────┼─────────┼────────┤        │
│                    │  │ abc123 │ Fix bug │ Todo   │        │
│                    │  │ def456 │ Feature │ Done   │        │
│                    │  └────────┴─────────┴────────┘        │
│                    │  Page 1 of 3 | 42 total rows          │
└────────────────────┴───────────────────────────────────────┘
```

**Controls**:
- Up/Down: Navigate tables or rows
- Enter: Load table contents
- PgUp/PgDn: Navigate pages
- 'q': Execute SQL query
- Esc: Return to Dev Tools menu

**Empty State Guidance**:
- **tasks table empty**: "No tasks yet. Press 'a' to create task or 'rig parse <file.md>'"
- **prds table empty**: "No PRDs yet. Press 'm' to browse markdown files"
- **projects table empty**: "No projects yet. Create one with: rig createproject"

#### Config Viewer

**Edits**: `.rigger/config.json` key-value pairs

**Layout**:
```
┌─────────────────────────────────────────────────────┐
│  ⚙️  Configuration Editor                          │
├─────────────────────────────────────────────────────┤
│  → provider: ollama                                 │
│    model.main: llama3.2:latest                      │
│    model.research: gpt-4o-mini                      │
│    model.fallback: microsoft/Phi-3.5-mini-instruct  │
│    database_url: sqlite:.rigger/tasks.db            │
└─────────────────────────────────────────────────────┘
```

**Controls**:
- Up/Down: Select key-value pair
- Enter: Start editing (toggles key/value)
- Type: Edit value
- Enter again: Save changes
- Esc: Cancel edit or return to Dev Tools menu

---

## Dialogs and Overlays

### Task Editor Dialog

**Trigger**: Enter key on selected task in Kanban

**Layout**:
```
┌─────────────────────────────────────────────────────┐
│  ✏️  Edit Task                                      │
├─────────────────────────────────────────────────────┤
│  Title: _Implement user authentication              │
│  Description: Add OAuth login to app_               │
│  Assignee: Alice_                                   │
│  Status: IN PROGRESS ↑ ↓                            │
│  Complexity: 7_                                     │
│                                                     │
│  [Tab] Next field  [Enter] Save  [Esc] Cancel      │
└─────────────────────────────────────────────────────┘
```

**Fields** (Tab/Shift+Tab navigation):
1. Title
2. Description
3. Assignee
4. Status (↑/↓ to cycle)
5. Complexity

**Keyboard**:
- Tab / Shift+Tab: Navigate fields
- ↑ / ↓: Cycle status
- Backspace: Delete character
- Enter: Save changes
- Esc: Cancel (discard changes)

**Status Cycle**:
```
Todo → InProgress → Completed → Archived → Errored → Todo
```

### Task Creator Dialog

**Trigger**: 'a' key

**Same layout as Task Editor**, but creates new task.

**Auto-linking**:
- New task linked to current project via PRD
- `source_prd_id` set if project has PRD
- `created_at` and `updated_at` set to now
- Default status: Todo
- Default complexity: 5

### LLM Chat Dialog

**Trigger**: 'l' key

**Layout**:
```
┌─────────────────────────────────────────────────────┐
│  💬 LLM Chat                                        │
├─────────────────────────────────────────────────────┤
│  Context:                                           │
│  Project: MyProject                                 │
│  Task: [task-123] Implement feature X               │
├─────────────────────────────────────────────────────┤
│  User: How should I structure this component?      │
│  Assistant: Based on the PRD requirements, I       │
│  recommend using a modular approach with...        │
│                                                     │
│  User: _                                            │
└─────────────────────────────────────────────────────┘
```

**Features**:
- Shows current project + task context
- Conversation history (alternating user/assistant)
- Type message, press Enter to send
- Scrollable chat history

**Example Commands**:
- "Summarize this task"
- "Generate subtasks for this feature"
- "What dependencies does this need?"
- "Review the PRD and suggest improvements"

### PRD Management Dialog

**Trigger**: 'r' key

**Layout**:
```
┌─────────────────────────────────────────────────────┐
│  📄 PRDs for Project: MyProject                     │
├─────────────────────────────────────────────────────┤
│  → [prd-001] User Authentication System             │
│    [prd-002] Dashboard Analytics                    │
│    [prd-003] Payment Integration                    │
├─────────────────────────────────────────────────────┤
│  Selected PRD Details:                              │
│  Title: User Authentication System                  │
│  Objectives:                                        │
│  - Secure login/logout                              │
│  - OAuth integration (Google, GitHub)               │
│  - Role-based access control                        │
│  Tech Stack: React, Node.js, PostgreSQL, JWT       │
│  Constraints: GDPR compliance, < 200ms auth         │
└─────────────────────────────────────────────────────┘
```

**Keyboard**:
- ↑ / ↓: Navigate PRD list
- Enter: (future) Edit PRD
- Esc: Close dialog

### Spotlight Search Dialog

**Trigger**: '/' key

**Layout**:
```
┌─────────────────────────────────────────────────────┐
│  🔍 Spotlight Search                                │
├─────────────────────────────────────────────────────┤
│  Query: auth_                                       │
├─────────────────────────────────────────────────────┤
│  → 📋 [task-123] Implement auth middleware          │
│    📄 [prd-001] User Authentication System          │
│    🎯 [proj-5] AuthService Microservice             │
│    📋 [task-789] Test OAuth flow                    │
└─────────────────────────────────────────────────────┘
```

**Features**:
- Real-time fuzzy search as you type
- Searches: task titles, PRD names, project names
- Type indicators: 📋 task | 📄 PRD | 🎯 project
- Match highlighting (bold matching text)

**Keyboard**:
- Type: Live search
- ↑ / ↓: Navigate results
- Enter: Jump to selected item
- Esc: Close

### Jump to Task Dialog

**Trigger**: 'g' key

**Layout**:
```
┌─────────────────────────────────────────────────────┐
│  Jump to Task                                       │
├─────────────────────────────────────────────────────┤
│  Enter task ID: task-123_                           │
└─────────────────────────────────────────────────────┘
```

**Fuzzy Matching**:
- "123" → matches "task-123"
- "auth" → matches "task-auth-middleware"
- Case-insensitive

### Recent Items Dialog

**Trigger**: Ctrl+R

**Layout**:
```
┌─────────────────────────────────────────────────────┐
│  Recent Tasks                                       │
├─────────────────────────────────────────────────────┤
│  → [task-789] Fix navigation bug                    │
│    [task-456] Add user profile page                 │
│    [task-123] Implement auth middleware             │
│    [task-111] Update documentation                  │
└─────────────────────────────────────────────────────┘
```

**MRU Tracking**:
- Max 10 most recently viewed tasks
- Updates on: task view, task edit
- Persists across sessions (in app state)

### Notification Center Dialog

**Trigger**: 'n' key

**Layout**:
```
┌─────────────────────────────────────────────────────┐
│  🔔 Notifications                                   │
├─────────────────────────────────────────────────────┤
│  ✅ 12:34:56  Saved task 'Fix navigation bug'       │
│  ⚠️  12:30:12  Task age is stale (7 days)           │
│  🔴 12:15:00  Failed to load projects: DB error     │
│  ℹ️  12:00:00  Setup complete! Welcome to Rigger.   │
│  ✅ 11:45:30  Copied task to clipboard              │
└─────────────────────────────────────────────────────┘
```

**Features**:
- Newest first (reverse chronological)
- Max 50 notifications (auto-prune oldest)
- Color-coded by severity
- Timestamps in HH:MM:SS format

### Confirmation Dialog

**Trigger**: Destructive actions (e.g., archive task)

**Layout**:
```
┌─────────────────────────────────────────────────────┐
│  ⚠️  Archive Task?                                  │
├─────────────────────────────────────────────────────┤
│  Are you sure you want to archive this task?       │
│  This will move it to long-term storage.           │
│                                                     │
│  [Y] Yes, archive   [N] Cancel                      │
└─────────────────────────────────────────────────────┘
```

**Triggers**:
- Archiving tasks (Completed → Archived)
- (Future) Deleting tasks
- (Future) Deleting projects

**Keyboard**:
- Y or Enter: Confirm
- N or Esc: Cancel

### Markdown Browser Dialog

**Trigger**: 'm' key

**Layout**:
```
┌─────────────────────────────────────────────────────┐
│  📄 Markdown Files                                  │
├─────────────────────────────────────────────────────┤
│  → authentication.md                                │
│    dashboard-analytics.md                           │
│    payment-integration.md                           │
│    project-overview.md                              │
│    user-stories.md                                  │
└─────────────────────────────────────────────────────┘
```

**Workflow**:
1. Press 'm' → Scans current directory for `*.md` files
2. Files sorted alphabetically
3. Navigate with ↑/↓
4. Press Enter to select file
5. PRD processing view appears (see next section)

### PRD Processing View

**Trigger**: Auto-shows after selecting markdown file

**Layout (Processing)**:
```
┌─────────────────────────────────────────────────────┐
│  Processing PRD: authentication.md                  │
├─────────────────────────────────────────────────────┤
│                                                     │
│         ⠋  Generating tasks using ollama...        │
│            with llama3.2:latest                     │
│                                                     │
└─────────────────────────────────────────────────────┘
```

**Layout (Success)**:
```
┌─────────────────────────────────────────────────────┐
│  ✓ PRD Processing Complete                          │
├─────────────────────────────────────────────────────┤
│  File: authentication.md                            │
│  Tasks Generated: 12                                │
│                                                     │
│  Press Enter to return to Kanban                    │
└─────────────────────────────────────────────────────┘
```

**Layout (Error with Diagnostics)**:
```
┌─────────────────────────────────────────────────────┐
│  🔴 PRD Processing Failed                           │
├─────────────────────────────────────────────────────┤
│  File: authentication.md                            │
│  Error: Connection refused                          │
│                                                     │
│  🔍 Diagnosing Ollama setup...                      │
│                                                     │
│  ✓ Ollama is installed                              │
│  ✓ Version: ollama version 0.12.9                   │
│  ❌ Ollama is not running                           │
│                                                     │
│  Fix: Start Ollama service                          │
│  → ollama serve                                     │
│  → Or start as background: ollama serve &           │
│                                                     │
│  Press Esc to close                                 │
└─────────────────────────────────────────────────────┘
```

**Processing Steps**:
1. Reading PRD file...
2. Parsing PRD structure...
3. Loading config...
4. Generating tasks using [provider] with [model]...
5. Saving tasks to database...
6. ✓ Complete!

---

## Keyboard Shortcuts

### Global Commands

| Key | Action | Description |
|-----|--------|-------------|
| q | Quit | Exit TUI |
| ? | Help | Show keyboard shortcuts overlay |
| Esc | Close Dialog | Close any open dialog/overlay |
| Ctrl+C | Quit | Emergency exit (works everywhere, including wizard) |

### Navigation

| Key | Action | Description |
|-----|--------|-------------|
| Tab | Next Tool | Kanban → TaskEditor → LLMChat → Metrics → DevTools |
| w | Previous Project | Switch to previous project |
| e | Next Project | Switch to next project |
| d | Toggle Details | Show/hide right details panel |

### Kanban Board

| Key | Action | Description |
|-----|--------|-------------|
| F1 | Todo Column | Select and view Todo tasks |
| F2 | InProgress Column | Select and view In Progress tasks |
| F3 | Completed Column | Select and view Completed tasks |
| F4 | Archived Column | Select and view Archived tasks |
| F5 | Errored Column | Select and view Errored tasks |
| ↑ | Previous Task | Move up in current column |
| ↓ | Next Task | Move down in current column |

### Task Actions

| Key | Action | Description |
|-----|--------|-------------|
| Enter | Edit Task | Open task editor for selected task |
| a | Create Task | Open task creator dialog |
| s | Cycle Status | Todo → InProgress → Completed → ... |
| c | Copy Task | Copy task to clipboard as Markdown |
| g | Jump to Task | Quick jump by ID (fuzzy match) |
| / | Spotlight | Global fuzzy search |

### Agent Tools

| Key | Action | Description |
|-----|--------|-------------|
| l | LLM Chat | Open chat with project/task context |
| r | View PRDs | Show PRD management dialog |
| m | Browse Markdown | Open markdown browser for PRD import |

### Other

| Key | Action | Description |
|-----|--------|-------------|
| o | Sort | Open sort menu |
| n | Notifications | View event history |
| Ctrl+R | Recent | Show MRU tasks (max 10) |
| F5 | Refresh | Reload data from database |
| F6 | Toggle View | Kanban ↔ Metrics |

### Dev Tools

| Key | Action | Context | Description |
|-----|--------|---------|-------------|
| Enter | Open | Dev Tools menu | Launch selected tool |
| ↑ / ↓ | Navigate | Dev Tools menu | Select tool |
| Esc | Close | Dev Tools menu | Return to main view |
| Enter | Load Table | SQLite Browser | View table contents |
| q | SQL Query | SQLite Browser | Execute SQL |
| PgUp / PgDn | Page | SQLite Browser | Navigate pages |

---

## Data Flow

### Startup Sequence

```
main.rs
  ├─ clap::Cli::parse()
  ├─ match Commands::Tui
  └─ tui::execute()
      ├─ App::new()
      ├─ Check .rigger/config.json exists?
      │   ├─ YES:
      │   │   ├─ load_projects()
      │   │   ├─ load_tasks()
      │   │   └─ Show Kanban
      │   └─ NO:
      │       ├─ setup_wizard_active = true
      │       └─ Show Welcome screen
      └─ run_app(terminal, app)
          └─ Event loop (keyboard, render)
```

### PRD Processing Flow

```
User presses 'm'
  ↓
scan_markdown_files() (async)
  ├─ Find *.md in current dir
  ├─ Sort alphabetically
  └─ Store in app.markdown_files
  ↓
Show markdown_browser dialog
  ↓
User presses Enter on file
  ↓
create_prd_from_markdown(filename)
  ├─ show_prd_processing = true
  ├─ Step 1: Reading PRD file...
  ├─ Step 2: Parsing PRD structure...
  │   └─ markdown_parsers::prd_parser::parse_prd_markdown()
  ├─ Step 3: Loading config...
  │   └─ Read .rigger/config.json → get provider & model
  ├─ Step 4: Generating tasks using [provider] with [model]...
  │   ├─ RigPRDParserAdapter::parse_prd_to_tasks(prd)
  │   │   ├─ Send PRD to LLM
  │   │   ├─ Parse response into tasks
  │   │   └─ ON ERROR (Ollama):
  │   │       └─ diagnose_ollama_error(model_name)
  │   │           ├─ Check: ollama installed? (which ollama)
  │   │           ├─ Check: ollama version (ollama version)
  │   │           ├─ Check: ollama running? (pgrep ollama)
  │   │           ├─ Check: model available? (ollama list)
  │   │           ├─ Check: API responding? (curl API)
  │   │           └─ Return specific fix
  │   └─ prd_tasks_generated = tasks.len()
  ├─ Step 5: Saving tasks to database...
  │   └─ For each task: db_adapter.save(task)
  ├─ prd_processing_complete = true
  └─ Show success or error screen
  ↓
User presses Enter (success) or Esc (error)
  ├─ show_prd_processing = false
  └─ reload_tasks() (if success)
```

### Ollama Diagnostics Flow

```
diagnose_ollama_error(model_name)
  ├─ Step 1: Check installed
  │   ├─ Command: which ollama
  │   ├─ NOT FOUND → return "❌ Ollama not found\nFix: curl https://ollama.ai/install.sh | sh"
  │   └─ FOUND → continue
  ├─ Step 2: Check version
  │   ├─ Command: ollama version
  │   └─ return "✓ Version: [output]"
  ├─ Step 3: Check running
  │   ├─ Command: pgrep ollama
  │   ├─ NOT RUNNING → return "❌ Not running\nFix: ollama serve"
  │   └─ RUNNING → continue
  ├─ Step 4: Check model available
  │   ├─ Command: ollama list
  │   ├─ Parse output for model_name
  │   ├─ NOT FOUND → return "❌ Model 'llama3.2:latest' not found\nAvailable: [list]\nFix: ollama pull llama3.2:latest"
  │   └─ FOUND → continue
  ├─ Step 5: Check API connectivity
  │   ├─ Command: curl -s -o /dev/null -w "%{http_code}" http://localhost:11434/api/tags
  │   ├─ HTTP 200 → return "✓ API responding"
  │   └─ OTHER → return "⚠️  API status: [code]"
  └─ Step 6: All passed → return advanced troubleshooting
      ├─ Test model: ollama run llama3.2:latest "Hello"
      ├─ Restart: pkill ollama && ollama serve
      └─ Re-pull: ollama pull llama3.2:latest --force
```

### Task Editing Flow

```
User navigates Kanban (↑/↓)
  ↓
selected_task_in_column changes
  ↓
User presses Enter
  ↓
open_task_editor()
  ├─ show_task_editor_dialog = true
  ├─ task_editor_field = Title
  ├─ task_editor_input = current_task.title
  └─ Load all task fields into editor state
  ↓
User navigates fields (Tab/Shift+Tab)
  ├─ task_editor_field = Title | Description | Assignee | Status | Complexity
  └─ Render active field with cursor
  ↓
User types (Char keypress)
  ├─ handle_task_editor_input(c)
  └─ task_editor_input.push(c)
  ↓
User cycles status (↑/↓)
  ├─ cycle_task_status_forward() or backward()
  └─ Todo → InProgress → Completed → Archived → Errored
  ↓
User presses Enter to save
  ↓
save_task_editor()
  ├─ Update task object with editor values
  ├─ db_adapter.save(task)?
  ├─ has_unsaved_changes = false
  ├─ last_saved_at = now
  ├─ show_task_editor_dialog = false
  └─ add_notification(Success, "Saved task")
```

### Copy to Clipboard Flow

```
User presses 'c'
  ↓
copy_task_to_clipboard()
  ├─ Guard: tasks not empty?
  ├─ Guard: clipboard adapter exists?
  ├─ Get current task
  ├─ task_formatter::format_task_as_markdown(task)
  │   └─ Returns Markdown:
  │       # Task Title
  │       **ID:** `task-123`
  │       **Status:** IN PROGRESS
  │       **Assignee:** Alice
  │       **Complexity:** 7/10
  │       ## Reasoning
  │       [reasoning text]
  │       ## Dependencies
  │       - task-456
  ├─ clipboard.copy_text(markdown)?
  │   └─ arboard::Clipboard::set_text()
  ├─ ON SUCCESS:
  │   ├─ status_message = "Copied task 'X' to clipboard"
  │   └─ add_notification(Success, "Copied to clipboard")
  └─ ON ERROR:
      ├─ status_message = "Failed to copy: [error]"
      └─ add_notification(Error, "Clipboard error")
```

---

## Hexagonal Architecture Layers

### Domain (task_manager crate)

**Pure business logic** - zero framework dependencies:

```
task_manager::domain::
  ├─ task::Task
  ├─ task_status::TaskStatus (enum)
  ├─ prd::PRD
  ├─ project::Project
  ├─ comprehension_test::ComprehensionTest
  └─ enhancement::Enhancement
```

### Ports (Abstract Interfaces)

**Contracts for external dependencies**:

```rust
// Clipboard abstraction
rigger_cli::ports::clipboard_port::ClipboardPort
pub trait ClipboardPort {
    fn copy_text(&self, text: &str) -> Result<()>;
}

// Repository abstraction (HEXSER)
hexser::ports::repository::Repository<T>
pub trait Repository<T> {
    fn save(&mut self, entity: T) -> Result<()>;
    fn find(&self, filter: &Filter, opts: FindOptions) -> Result<Vec<T>>;
    fn delete(&mut self, id: &str) -> Result<()>;
}
```

### Adapters (Concrete Implementations)

**Bridge domain ↔ infrastructure**:

```rust
// Clipboard adapter
rigger_cli::adapters::arboard_clipboard_adapter::ArboardClipboardAdapter
impl ClipboardPort for ArboardClipboardAdapter {
    fn copy_text(&self, text: &str) -> Result<()> {
        let mut clipboard = arboard::Clipboard::new()?;
        clipboard.set_text(text)?;
        Ok(())
    }
}

// Database adapter
task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter
impl Repository<Task> for SqliteTaskAdapter {
    fn save(&mut self, entity: Task) -> Result<()> {
        // Map Task → SQL INSERT/UPDATE
    }
    fn find(&self, filter: &Filter, opts: FindOptions) -> Result<Vec<Task>> {
        // Map SQL SELECT → Vec<Task>
    }
}

// LLM adapter
task_orchestrator::adapters::rig_prd_parser_adapter::RigPRDParserAdapter
impl PRDParserPort for RigPRDParserAdapter {
    async fn parse_prd_to_tasks(&self, prd: &PRD) -> Result<Vec<Task>> {
        // Send PRD to LLM → parse response → return tasks
    }
}
```

### Services (Business Logic)

**Reusable domain services**:

```
rigger_cli::services::
  └─ task_formatter::
      ├─ format_task_as_markdown(task) -> String
      └─ format_task_as_plain_text(task) -> String

task_manager::utils::
  ├─ markdown_parsers::prd_parser::parse_prd_markdown() -> PRD
  └─ parse_action_items_tolerant() -> Vec<ExtractedActionItem>
      ├─ Extracts JSON from LLM prose
      ├─ Handles schema variations
      ├─ Normalizes assignee names
      └─ Used by ALL LLM adapters
```

### Application Layer (Use Cases)

**Orchestrate domain + ports**:

```
task_manager::use_cases::
  └─ manage_task::ManageTaskUseCase<R: TaskRepositoryPort>
      ├─ create_task(task) -> Result<Task>
      ├─ update_task(task) -> Result<Task>
      └─ delete_task(id) -> Result<()>

task_orchestrator::use_cases::
  └─ run_task_with_flow::RunTaskWithFlowUseCase
      ├─ execute(task_id) -> Result<()>
      └─ State machine: Todo → InProgress → Completed
```

### Infrastructure (External Tools)

**Framework integrations**:

- **Ratatui**: Terminal UI rendering
- **Crossterm**: Keyboard/mouse input
- **SQLx**: Database connection pool
- **Arboard**: System clipboard
- **Tokio**: Async runtime
- **Tonic**: gRPC server
- **Ollama**: Local LLM (HTTP API)

---

## Error Handling

### Ollama Error Diagnostics

**Triggered when**: PRD parsing fails with Ollama provider

**Function**: `diagnose_ollama_error(model_name: &str) -> String`

**Checks Performed** (in order):

1. **Ollama Installed?**
   - Command: `which ollama`
   - If NOT found:
     ```
     ❌ Ollama not found
     Fix: Install Ollama
     → curl https://ollama.ai/install.sh | sh
     → Or download from https://ollama.ai
     ```
   - If found: Continue

2. **Ollama Version**
   - Command: `ollama version`
   - Output: `✓ Version: ollama version 0.12.9`

3. **Ollama Running?**
   - Command: `pgrep ollama`
   - If NOT running:
     ```
     ❌ Ollama is not running
     Fix: Start Ollama service
     → ollama serve
     → Or start as background: ollama serve &
     ```
   - If running: Continue

4. **Model Available?**
   - Command: `ollama list`
   - Parses output for model name (e.g., "llama3.2:latest")
   - If NOT found:
     ```
     ❌ Model 'llama3.2:latest' not found
     Available models:
     - llama2:7b
     - codellama:13b

     Fix: Pull the model
     → ollama pull llama3.2:latest
     ```
   - If found: Continue

5. **API Connectivity**
   - Command: `curl -s -o /dev/null -w "%{http_code}" http://localhost:11434/api/tags`
   - If HTTP 200: `✓ Ollama API is responding`
   - If other: `⚠️  Ollama API returned status: [code]`

6. **All Checks Passed**
   ```
   All basic checks passed. Advanced troubleshooting:

   1. Test model directly:
      → ollama run llama3.2:latest "Hello"

   2. Check Ollama logs (if available):
      → Check system logs for Ollama errors

   3. Restart Ollama:
      → pkill ollama && ollama serve

   4. Re-pull model:
      → ollama pull llama3.2:latest --force

   5. Verify config.json:
      → Check .rigger/config.json has correct model name
      → Model names are case-sensitive
   ```

**Usage in PRD Processing**:

```rust
match parser.parse_prd_to_tasks(&prd).await {
    Ok(tasks) => tasks,
    Err(e) => {
        let diagnostics = self.diagnose_ollama_error(model_name).await;
        let error_msg = format!("Task generation failed: {}\n\n{}", e, diagnostics);
        self.prd_processing_error = Some(error_msg);
        return Err(anyhow::anyhow!(e));
    }
}
```

---

## File Organization

### Strict Coding Standards (per CLAUDE.md)

1. **No `use` statements** - All types use fully qualified paths (except prelude: Vec, String, Option, Result)
2. **One logical item per file** - Each `.rs` has exactly one struct/enum/fn
3. **Revision history required** - Every modification adds timestamped entry
4. **Function length limit** - Max 50 lines of code
5. **In-file tests** - Unit tests in `#[cfg(test)] mod tests { ... }`

### Module Hierarchy

```
rigger_cli/src/
├── main.rs (20 lines)
├── lib.rs (10 lines)
├── commands/
│   ├── mod.rs (86 lines) - CLI structure
│   ├── init.rs (100 lines) - Initialize project
│   ├── parse.rs (168 lines) - Parse PRD
│   ├── list.rs (150 lines) - List tasks
│   ├── do_task.rs (80 lines) - Execute task
│   ├── server.rs (200 lines) - MCP server
│   ├── grpc_server.rs (300 lines) - gRPC server
│   └── tui.rs (8000+ lines) - MAIN TUI APPLICATION
├── display/
│   ├── mod.rs
│   └── task_table.rs (200 lines) - ASCII table rendering
├── ports/
│   ├── mod.rs
│   └── clipboard_port.rs (30 lines) - ClipboardPort trait
├── adapters/
│   ├── mod.rs
│   └── arboard_clipboard_adapter.rs (50 lines) - Clipboard impl
└── services/
    ├── mod.rs
    └── task_formatter.rs (328 lines) - Markdown/text formatting
```

### tui.rs Structure (8000+ lines)

**Enums** (540 lines):
- NotificationLevel
- DashboardTool
- DevTool
- WorkspaceSection
- KanbanColumn
- TaskSortOption
- TaskEditorField
- TaskCreatorField
- SetupWizardStep
- LLMProvider
- ModelConfigField
- SearchResultType
- ConfirmationAction
- ConfigEditorField

**Structs** (200 lines):
- App (main state - 185 fields!)
- Notification
- ChatMessage

**App Methods** (3000 lines):
- State Management: new(), load_tasks(), load_projects(), apply_sort()
- Navigation: next_tool(), next_column(), next_task_in_column()
- Task Actions: cycle_task_status(), copy_task_to_clipboard()
- Dialogs: open_*/close_* for all 15 dialogs
- Dev Tools: load_db_tables(), load_table_data(), execute_sql_query()
- Setup Wizard: wizard_next_step(), wizard_complete(), wizard_exit()
- Markdown/PRD: scan_markdown_files(), create_prd_from_markdown(), diagnose_ollama_error()
- Utilities: add_notification(), calculate_task_age_days(), get_filtered_tasks()

**Top-level Functions** (4000 lines):
- execute() - TUI entry point
- run_app(terminal, app) - Event loop
- render_ui(frame, app) - Root render
- render_*() - 30+ render functions for views/dialogs
- Helper utilities: truncate_string(), format_duration(), centered_rect()

---

## Dependencies

```
rigger_cli
  ├─ task_manager (workspace) ────────┐
  │   ├─ hexser (v0.4.7)              │
  │   ├─ sqlx (workspace)             │
  │   └─ chrono (workspace)           │
  ├─ task_orchestrator (workspace)    │
  │   ├─ task_manager ────────────────┘
  │   ├─ rig (workspace)
  │   └─ graph-flow
  ├─ transcript_extractor (workspace)
  ├─ ratatui (v0.29)
  ├─ crossterm (v0.28)
  ├─ arboard (v3.4)
  ├─ clap (v4.4)
  ├─ prettytable-rs (v0.10)
  ├─ serde + serde_json (v1.0)
  ├─ tokio (v1.41)
  ├─ tonic (v0.12)
  ├─ prost (v0.13)
  ├─ anyhow (v1.0)
  └─ uuid (v1.11)
```

---

## Build and Run

### Build

```bash
# From workspace root
cargo build --release -p rigger_cli

# Binary: ./target/release/rig
```

### Install

```bash
# Install to ~/.cargo/bin/rig
cargo install --path rigger_cli
```

### Run Commands

```bash
# Initialize project
rig init

# Parse PRD
rig parse docs/authentication-prd.md

# List tasks
rig list --status in_progress --assignee Alice

# Execute task
rig do task-123

# Start servers
rig server           # MCP stdio
rig grpc --port 50051  # gRPC sidecar

# Launch TUI (main interface)
rig tui
```

---

## Configuration

### `.rigger/config.json`

```json
{
  "provider": "ollama",
  "task_tools": {
    "main": {
      "provider": "ollama",
      "model": "llama3.2:latest"
    },
    "research": {
      "provider": "rig",
      "model": "gpt-4o-mini"
    },
    "fallback": {
      "provider": "candle",
      "model": "microsoft/Phi-3.5-mini-instruct"
    }
  },
  "model": {
    "main": "llama3.2:latest",
    "research": "gpt-4o-mini",
    "fallback": "microsoft/Phi-3.5-mini-instruct"
  },
  "database_url": "sqlite:.rigger/tasks.db"
}
```

### `.rigger/tasks.db` (SQLite Schema)

```sql
CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    assignee TEXT,
    due_date TEXT,
    status TEXT NOT NULL,
    source_transcript_id TEXT,
    source_prd_id TEXT,
    parent_task_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    complexity INTEGER,
    reasoning TEXT
);

CREATE TABLE prds (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    title TEXT NOT NULL,
    objectives TEXT,  -- JSON array
    tech_stack TEXT,  -- JSON array
    constraints TEXT, -- JSON array
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

---

## Performance

### Database

- **Task limit**: 100 rows (configurable in load_tasks())
- **Pagination**: 20 rows/page in SQLite Browser
- **Connection pooling**: sqlx automatic

### Memory

- **Notifications**: Max 50 (auto-prune oldest)
- **Recent tasks**: Max 10 (MRU cache)
- **Chat history**: Unbounded (TODO: add limit)

### Async

- Tokio runtime for all async operations
- Database queries are async
- LLM requests are async
- UI remains responsive during long operations

---

## Troubleshooting

### Common Issues

1. **Setup Wizard Exits Immediately**
   - Cause: Esc or Ctrl+C pressed
   - Solution: Intentional - both exit wizard and quit app

2. **Ollama Model Not Found**
   - Cause: Model not pulled or wrong name
   - Solution: Check diagnostics output, run `ollama pull llama3.2:latest`

3. **Database Connection Failed**
   - Cause: .rigger/tasks.db missing or corrupted
   - Solution: Run `rig init` to recreate

4. **Clipboard Not Working**
   - Cause: No GUI environment (SSH) or missing dependencies
   - Solution: Run locally, install clipboard support

---

## Future Enhancements

- Multi-project workspaces
- Task dependency graph visualization
- Time tracking
- Custom task fields
- CSV/JSON export
- Git integration
- Real-time collaboration (gRPC)
- Plugin system
- Custom themes
- Configurable dashboard widgets

---

**Maintained By**: Rigger Contributors
**Last Updated**: 2025-11-25T13:30:00Z
