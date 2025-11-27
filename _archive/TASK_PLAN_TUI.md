task_id: TUI-MASTER-001

status: in-progress (Phase 1 Basic TUI ✅ COMPLETE, Phase 2 Planning)

# Task: Build Feature-Complete Rigger TUI with Full CLI Parity

## Vision: The Ultimate Terminal Dashboard for AI Project Management

Transform `rig tui` from a basic task viewer into a **production-grade, feature-complete terminal application** that rivals modern GUI tools. Every CLI command must have a TUI equivalent, with additional visual analytics, real-time monitoring, and an intuitive keyboard-driven workflow.

**Guiding Principles**:
- **Full Feature Parity**: Everything in `rig` CLI must be accessible in TUI
- **Superior UX**: TUI should be FASTER and MORE INTUITIVE than CLI for power users
- **Rich Analytics**: Visualizations that aren't possible in CLI (charts, graphs, timelines)
- **Real-Time Updates**: Live monitoring of orchestration pipeline progress
- **Keyboard-First**: Vim-like navigation, minimal mouse dependency
- **Beautiful**: Professional aesthetics with thoughtful color schemes and layout

## Current State (Phase 1 ✅ COMPLETE)

**Location**: `rigger_cli/src/commands/tui.rs` (431 lines)

**Implemented Features**:
- ✅ Basic task board with TODO, IN PROGRESS, COMPLETED columns
- ✅ Mock chain-of-thought visualization
- ✅ Mock network request logging
- ✅ Tab navigation between views
- ✅ Help screen with keyboard controls
- ✅ Task loading from SQLite database

**Critical Gaps**:
- ❌ No PRD management (list, view, edit, create)
- ❌ No task creation/editing/deletion
- ❌ No sub-task editing/enhancement capabilities
- ❌ No orchestration execution (cannot run `rig do` from TUI)
- ❌ No real-time progress monitoring
- ❌ No analytics/metrics visualization
- ❌ No search/filtering or spotlight feature
- ❌ No task details view
- ❌ Static mock data for thinking/network logs
- ❌ No multi-project support or project switcher
- ❌ No animated status summary showing project activity

---

## Phase 0: Quick Wins & Low-Hanging Fruit (2-3 days)

**Goal**: Implement high-value, low-effort features that dramatically improve UX with minimal complexity.

### Sprint -1: Essential UX Improvements

- [ ] 0.1. Add keyboard shortcut overlay
  - Location: rigger_cli/src/commands/tui/widgets/shortcut_overlay.rs
  - Keyboard shortcut: `?` or `F1`
  - **Persistent cheat sheet in corner** (non-intrusive)
  - Shows top 5-8 most relevant shortcuts for current view
  - Example:
    ```
    ┌─ Quick Keys ──┐
    │ ? - Help      │
    │ n - New Task  │
    │ e - Edit      │
    │ / - Search    │
    │ q - Quit      │
    └───────────────┘
    ```
  - Toggle with `?` to show full list
  - Context-aware: Changes based on active view/selection
  - Test: Verify shortcuts update per view

- [ ] 0.2. Implement quick status cycling
  - Keyboard shortcut: `s` on selected task
  - **Instant status change** without opening dialog
  - Cycles: TODO → IN PROGRESS → COMPLETED → TODO
  - Visual feedback: Brief flash/highlight on change
  - Shows mini-toast: "Status changed to IN PROGRESS"
  - Alternative: `1`=TODO, `2`=IN PROGRESS, `3`=COMPLETED (number keys)
  - Test: Cycle through statuses, verify database updates

- [ ] 0.3. Add copy/paste operations
  - **Copy operations** (Ctrl+C or `yy` vim-style):
    - Task ID (for sharing: "Working on TUI-042")
    - Task title and description (formatted markdown)
    - Full task details (for external tools)
    - PRD content
  - **Paste operations** (Ctrl+V or `p` vim-style):
    - Create task from clipboard text
    - Import PRD from clipboard
  - Visual feedback: "Copied task ID: TUI-042" toast
  - Works with system clipboard (use `clipboard` crate)
  - Test: Copy task details, paste in external editor

- [ ] 0.4. Add autosave/sync indicator
  - Location: Top-right corner of header
  - Shows save status:
    - 💾 "Saving..." (spinner)
    - ✓ "Saved" (green, 2s then fade)
    - ⚠️ "Offline - changes queued" (yellow)
    - ❌ "Save failed - retry?" (red)
  - Auto-saves after every edit (debounced 500ms)
  - Click to force sync
  - Test: Edit task, verify indicator updates

- [ ] 0.5. Implement basic task sorting
  - Keyboard shortcut: `o` (order) on task board
  - Quick sort menu:
    ```
    Sort by:
    → Created (newest first)
      Updated (most recent)
      Priority (high → low)
      Alphabetical (A-Z)
      Due date (soonest first)
      Complexity (hardest first)
    ```
  - Arrow keys to select, Enter to apply
  - Remember last sort per view (persisted)
  - Visual indicator in column header: "▼ Priority"
  - Test: Sort by each option, verify order

- [ ] 0.6. Add visual loading states
  - **Universal loading pattern** for all async operations:
    - Small spinner for short operations (<1s expected)
    - Progress bar for long operations (>1s expected)
    - Skeleton screens for data loading (instead of blank)
  - Locations:
    - Task board loading: Show 3 skeleton task cards
    - PRD parsing: Progress bar with steps
    - Orchestration: Multi-stage progress
    - Search results: Spinner while searching
  - Prevents "is it frozen?" confusion
  - Test: Verify loading states for all async ops

- [ ] 0.7. Add Recent/MRU lists
  - Location: Quick access from Spotlight or sidebar
  - Track Most Recently Used (MRU):
    - Last 10 viewed tasks
    - Last 5 edited tasks
    - Last 3 parsed PRDs
    - Last 5 orchestrations
  - Keyboard shortcut: `Ctrl+R` (recent)
  - Quick dialog:
    ```
    ┌─ Recent Items ──────────────────┐
    │ Tasks:                          │
    │  → TUI-042: Implement auth (2m) │
    │    API-015: Rate limiting (5m)  │
    │    DB-008: User schema (12m)    │
    │                                 │
    │ PRDs:                           │
    │    authentication-system.md     │
    │                                 │
    │ [Enter] Open  [Esc] Cancel      │
    └─────────────────────────────────┘
    ```
  - Persisted in `.rigger/recent.json`
  - Test: Verify MRU updates and persists

- [ ] 0.8. Add quick filter views
  - Location: Top of task board or separate tab
  - Predefined filter buttons:
    - 👤 **My Tasks** (assigned to current user)
    - 📅 **Today** (due today or overdue)
    - 🔥 **Urgent** (high priority or blocked)
    - ⏰ **This Week** (due in next 7 days)
    - 🎯 **In Progress** (only active tasks)
    - ✓ **Recently Completed** (last 24h)
  - Click/press number key (F1-F6) to activate filter
  - Visual indicator showing active filter
  - Combine with search/other filters
  - Test: Apply each filter, verify results

- [ ] 0.9. Add notification center
  - Location: Bell icon in header (with badge count)
  - Keyboard shortcut: `Ctrl+N`
  - Collects missed events when user is in different view:
    - "Task TUI-042 completed while you were away"
    - "3 new tasks created"
    - "Orchestration failed: API-018"
  - Notification list:
    ```
    ┌─ Notifications (3 unread) ─────┐
    │ 2m ago  ✓ TUI-042 completed    │
    │ 5m ago  ⚠️  API-018 failed      │
    │ 8m ago  📄 New PRD created      │
    │                                │
    │ [c] Clear all  [Esc] Close     │
    └────────────────────────────────┘
    ```
  - Badge shows unread count
  - Click notification to jump to item
  - Auto-clear after 1 hour or mark as read
  - Test: Generate notifications, verify display

- [ ] 0.10. Add task ID quick jump
  - Keyboard shortcut: `g` (go to) anywhere
  - Type task ID to jump directly:
    ```
    ┌─ Go To Task ──────┐
    │ ID: TUI-042___    │
    │                   │
    │ [Enter] Jump      │
    └───────────────────┘
    ```
  - Fuzzy matching: "042" finds "TUI-042"
  - Autocomplete as you type
  - Shows task title preview
  - Opens task detail view immediately
  - Test: Jump to various tasks by ID

- [ ] 0.11. Add visual task age indicators
  - Show how long tasks have been in each status:
    - < 1 day: No indicator
    - 1-3 days: 📅 (calendar icon)
    - 3-7 days: ⚠️ (warning - getting stale)
    - > 7 days: 🔴 (urgent - very stale)
  - Color-code task cards by age:
    - Recent: Normal color
    - Aging: Slightly dimmed
    - Stale: Yellow tint
    - Very stale: Red tint
  - Helps identify tasks that need attention
  - Test: Create tasks with different ages, verify indicators

- [ ] 0.12. Add task count badges
  - Show counts in tab headers:
    - "📋 Tasks (42)" instead of just "📋 Tasks"
    - "📄 PRDs (7)"
    - "⚙️ Orchestrator (2 active)"
    - "📊 Analytics"
  - Update in real-time as tasks change
  - Helps with situational awareness
  - Test: Verify counts update correctly

---

## Phase 1.5: Foundational UI Enhancements (Week 1)

**Goal**: Implement essential navigation and awareness features that enhance the entire TUI experience.

### Sprint 0: Project Context & Navigation (2-3 days)

- [ ] 1.5.1. Add persistent project indicator header
  - Location: rigger_cli/src/commands/tui/widgets/project_header.rs
  - Always visible at top of screen, showing:
    - Current project name (from `.rigger/config.json` or directory name)
    - Project root path (abbreviated: `~/dev/my-app`)
    - Active branch (if Git repo)
    - Task counts: `15 TODO | 8 IN PROGRESS | 42 DONE`
  - Visual design:
    ```
    ╔════════════════════════════════════════════════════════════╗
    ║ 📦 my-awesome-project (~/dev/my-app) [main] │ 15→ 8⚡ 42✓ ║
    ╚════════════════════════════════════════════════════════════╝
    ```
  - Color-coded status indicators
  - Test: Display project info accurately

- [ ] 1.5.2. Implement project switcher dialog
  - Location: rigger_cli/src/commands/tui/widgets/project_switcher.rs
  - Keyboard shortcut: `Ctrl+P` (like VSCode)
  - Features:
    - Lists all projects with `.rigger/` directories in parent/common directories
    - Recent projects list (last 5 accessed)
    - Fuzzy search by project name
    - Shows project metadata: task count, last activity date
  - Layout:
    ```
    ┌─ Switch Project ────────────────────────┐
    │ Search: [my-app_______________]          │
    │                                          │
    │ Recent:                                  │
    │  → my-awesome-project  (15 tasks, 2d)    │
    │    client-dashboard   (8 tasks, 1w)      │
    │    api-service       (23 tasks, 3d)      │
    │                                          │
    │ All Projects:                            │
    │    legacy-project    (0 tasks, 6mo)      │
    │                                          │
    │ [Enter] Select  [Esc] Cancel             │
    └──────────────────────────────────────────┘
    ```
  - Keyboard shortcuts:
    - `↑`/`↓`: Navigate project list
    - `Enter`: Switch to selected project
    - `Esc`: Cancel
    - Type to filter
  - On switch: Reload all data for new project
  - Save last accessed project in `~/.rigger/recent_projects.json`
  - Test: Switch between projects, verify data isolation

- [ ] 1.5.3. Create Spotlight/Command Palette
  - Location: rigger_cli/src/commands/tui/widgets/spotlight.rs
  - Keyboard shortcut: `Ctrl+K` or `/` (universal search)
  - **The Ultimate TUI Navigation Feature**
  - Fuzzy search across ALL entities:
    - Tasks (by title, description, ID)
    - PRDs (by filename, content)
    - Recent actions/commands
    - Keyboard shortcuts (interactive help)
    - Views/tabs
  - Layout:
    ```
    ┌─ Spotlight Search ──────────────────────────────────────┐
    │ > auth implement________________________________        │
    │                                                          │
    │ Tasks (3):                                               │
    │  🎯 TUI-002: Implement authentication UI       [TODO]    │
    │  🎯 API-015: Auth middleware integration  [IN PROGRESS]  │
    │  🎯 DB-008: Auth token table schema         [COMPLETED]  │
    │                                                          │
    │ PRDs (1):                                                │
    │  📄 authentication-system.md                             │
    │                                                          │
    │ Commands (2):                                            │
    │  ⚡ Execute orchestration on selected task               │
    │  ✏️  Edit task details                                   │
    │                                                          │
    │ [↑↓] Navigate  [Enter] Open  [Esc] Cancel               │
    └──────────────────────────────────────────────────────────┘
    ```
  - Features:
    - Real-time fuzzy matching (using `sublime_fuzzy` crate)
    - Results grouped by type
    - Shows context/preview for each result
    - Recent searches history
    - Weighted scoring: Recent items ranked higher
  - Actions on selection:
    - Task: Open task detail view
    - PRD: Open PRD viewer
    - Command: Execute command
    - View: Switch to that tab
    - Shortcut: Show help and highlight that action
  - Test: Search for various entities, verify ranking and navigation

- [ ] 1.5.4. Add animated status summary widget
  - Location: rigger_cli/src/commands/tui/widgets/status_summary.rs
  - **Always-visible activity indicator** (top-right corner or side panel)
  - Real-time animated summary showing project health:
    ```
    ┌─ Project Activity ───────┐
    │ ⚡ Orchestrating...       │
    │   └─ TUI-042 [▓▓▓░░] 60% │
    │                          │
    │ 🔍 Research active       │
    │   └─ "How to implement   │
    │       WebSocket auth"    │
    │                          │
    │ ⏱️  In Progress (3)      │
    │   └─ 2 idle (>24h)       │
    │                          │
    │ ✓ Completed today: 5     │
    │                          │
    │ 🎯 Focus: High priority  │
    │   └─ 7 tasks need work   │
    └──────────────────────────┘
    ```
  - Animated elements:
    - Spinner for active orchestration
    - Progress bars for long-running tasks
    - Pulse effect for idle tasks (attention grabber)
    - Smooth count-up animations for metrics
  - Real-time indicators:
    - 🔴 Orchestration running
    - 🟡 Tasks blocked by dependencies
    - 🟢 All tasks progressing normally
    - 🔵 Research/thinking in progress
    - ⚠️  Attention needed (failed tests, old tasks)
  - Updates every 1-2 seconds
  - Click/Enter to expand for details
  - Collapsible (hide with `Ctrl+\`)
  - Test: Verify real-time updates during orchestration

- [ ] 1.5.5. Implement activity feed/timeline
  - Location: rigger_cli/src/commands/tui/widgets/activity_feed.rs
  - New tab: "📡 Activity"
  - Live feed of ALL project events:
    ```
    ┌─ Live Activity Feed ─────────────────────────────────────┐
    │ Just now   ⚡ Orchestration started: TUI-042              │
    │ 2m ago     ✏️  Task updated: TUI-041 → IN PROGRESS       │
    │ 5m ago     🤖 LLM call: llama3.1 (1.2s, 156 tokens)      │
    │ 8m ago     ✓ Task completed: API-015                     │
    │ 12m ago    📄 PRD parsed: authentication-system.md (5 ta…│
    │ 15m ago    🔍 Semantic routing: complexity=7 → decompose │
    │ 18m ago    👤 Task assigned: DB-008 → @alice             │
    │ 22m ago    🏷️  Tags added: TUI-040 +urgent +bug          │
    │ 30m ago    📊 Analytics updated: 42 tasks completed      │
    │                                                          │
    │ [Space] Pause  [f] Filter  [Esc] Close                   │
    └──────────────────────────────────────────────────────────┘
    ```
  - Features:
    - Auto-scroll (pausable with Space)
    - Filter by event type
    - Click event to jump to related entity
    - Export to log file
  - Test: Generate events, verify feed updates

- [ ] 1.5.6. Create Settings/Configuration UI
  - Location: rigger_cli/src/commands/tui/widgets/settings_view.rs
  - **New tab: "⚙️ Settings"** (or keyboard shortcut: `,` like Vim)
  - **Comprehensive configuration management without editing files**
  - Multi-section tabbed layout:
    ```
    ┌─ Settings ───────────────────────────────────────────────┐
    │ [API Keys] [Models] [Database] [UI] [Keybindings] [Adv] │
    ├──────────────────────────────────────────────────────────┤
    │ API Keys & Authentication                                │
    │                                                          │
    │ OpenAI API Key:                                          │
    │ [sk-proj-••••••••••••••••••••••••••] [Test] [Clear]     │
    │ Status: ✓ Valid (tested 2m ago)                         │
    │                                                          │
    │ Anthropic API Key:                                       │
    │ [sk-ant-••••••••••••••••••••••••••] [Test] [Clear]      │
    │ Status: ⚠️  Not configured                               │
    │                                                          │
    │ Ollama Endpoint:                                         │
    │ [http://localhost:11434_____________] [Test]             │
    │ Status: ✓ Connected (3 models available)                │
    │                                                          │
    │ [Apply] [Reset to Defaults] [Import] [Export]           │
    └──────────────────────────────────────────────────────────┘
    ```

  - **Section 1: API Keys & Authentication**
    - OpenAI API key input (masked with ••••)
    - Anthropic API key input
    - Ollama endpoint URL
    - Custom model endpoints
    - Test connection button per service
    - Visual status indicator (✓ valid, ⚠️ warning, ❌ error)
    - Clear/remove key option
    - Secure storage using OS keychain (keyring crate)

  - **Section 2: Model Configuration**
    ```
    ┌─ Model Settings ─────────────────────────────────────────┐
    │ Default Models:                                          │
    │                                                          │
    │ Routing/Semantic Analysis:                               │
    │ [llama3.1 ▼] Temperature: [0.3   ] Max Tokens: [512  ]  │
    │                                                          │
    │ Task Enhancement:                                        │
    │ [llama3.1 ▼] Temperature: [0.7   ] Max Tokens: [2048 ]  │
    │                                                          │
    │ Test Generation:                                         │
    │ [phi3     ▼] Temperature: [0.5   ] Max Tokens: [1024 ]  │
    │                                                          │
    │ Research/RAG:                                            │
    │ [orca2    ▼] Temperature: [0.4   ] Max Tokens: [1024 ]  │
    │                                                          │
    │ Model Preferences:                                       │
    │ [✓] Prefer local models (Ollama) over API               │
    │ [✓] Fallback to cloud if local unavailable              │
    │ [ ] Always confirm before API calls (cost control)      │
    │                                                          │
    │ Available Models: (auto-detected from Ollama)            │
    │  ✓ llama3.1:latest (8B)                                  │
    │  ✓ phi3:latest (3.8B)                                    │
    │  ✓ mistral:latest (7B)                                   │
    │  [Refresh] [Pull New Model]                              │
    └──────────────────────────────────────────────────────────┘
    ```
    - Per-node model selection (routing, enhancement, testing, etc.)
    - Temperature and max token controls
    - Model preferences (local vs cloud, fallbacks)
    - Auto-detect available Ollama models
    - Pull new models from TUI
    - Cost estimation settings

  - **Section 3: Database Configuration**
    ```
    ┌─ Database Settings ──────────────────────────────────────┐
    │ Connection:                                              │
    │ Type: [SQLite ▼]                                         │
    │ Path: [.rigger/tasks.db________________] [Browse]       │
    │ Status: ✓ Connected (42 tasks, 7 PRDs)                  │
    │                                                          │
    │ Backup:                                                  │
    │ [✓] Auto-backup daily                                    │
    │ Backup location: [.rigger/backups/______] [Browse]      │
    │ Keep last: [7  ] backups                                 │
    │ Last backup: 2h ago (2.3 MB)                             │
    │ [Backup Now] [Restore from Backup]                       │
    │                                                          │
    │ Maintenance:                                             │
    │ [Vacuum Database] [Check Integrity] [Export All Data]   │
    └──────────────────────────────────────────────────────────┘
    ```
    - Database type selector (SQLite, PostgreSQL future)
    - Connection string/path
    - Test connection
    - Backup configuration
    - Manual backup/restore
    - Database maintenance tools

  - **Section 4: UI Preferences**
    ```
    ┌─ UI Settings ────────────────────────────────────────────┐
    │ Appearance:                                              │
    │ Theme: [Default ▼] [Dark] [Light] [Dracula] [Nord]      │
    │ Preview: [████ Active] [████ Todo] [████ Done]           │
    │                                                          │
    │ Layout:                                                  │
    │ Default view: [Task Board ▼]                             │
    │ Column width: [Auto ▼] or [Fixed: 40 chars]             │
    │ Task card height: [Compact ▼] [Normal] [Expanded]       │
    │                                                          │
    │ Behavior:                                                │
    │ [✓] Auto-refresh every 30 seconds                        │
    │ [✓] Show notifications                                   │
    │ [✓] Sound effects (on completion, errors)                │
    │ [✓] Confirm before destructive actions                   │
    │ [ ] Mouse support (experimental)                         │
    │                                                          │
    │ Status Footer:                                           │
    │ [✓] Show animated status bar                             │
    │ Cycle interval: [3  ] seconds                            │
    │ [ ] Minimize footer by default                           │
    └──────────────────────────────────────────────────────────┘
    ```
    - Theme selection with live preview
    - Layout preferences
    - Auto-refresh settings
    - Notification preferences
    - Status footer customization

  - **Section 5: Keybindings**
    ```
    ┌─ Keybindings ────────────────────────────────────────────┐
    │ Preset: [Default ▼] [Vim] [Emacs] [Custom]              │
    │                                                          │
    │ Navigation:                                              │
    │ Next tab          [Tab        ] [Change]                 │
    │ Previous tab      [Shift+Tab  ] [Change]                 │
    │ Spotlight search  [Ctrl+K     ] [Change]                 │
    │ Go to task        [g          ] [Change]                 │
    │                                                          │
    │ Task Actions:                                            │
    │ New task          [n          ] [Change]                 │
    │ Edit task         [e          ] [Change]                 │
    │ Delete task       [D          ] [Change]                 │
    │ Toggle status     [s          ] [Change]                 │
    │                                                          │
    │ [Reset to Default] [Import] [Export]                     │
    │                                                          │
    │ Click [Change] to record new key combination             │
    └──────────────────────────────────────────────────────────┘
    ```
    - Visual keybinding editor
    - Preset options (Default, Vim, Emacs)
    - Click to record new key combination
    - Conflict detection
    - Import/export keybindings
    - Reset to defaults

  - **Section 6: Advanced Settings**
    ```
    ┌─ Advanced Settings ──────────────────────────────────────┐
    │ Performance:                                             │
    │ Max concurrent LLM calls: [2  ]                          │
    │ Request timeout: [30] seconds                            │
    │ Cache LLM responses: [✓] for [24] hours                 │
    │                                                          │
    │ Logging:                                                 │
    │ Log level: [Info ▼] (Debug/Info/Warn/Error)             │
    │ Log file: [.rigger/tui.log____________] [View]          │
    │ Max log size: [10] MB                                    │
    │                                                          │
    │ Development:                                             │
    │ [✓] Enable debug mode                                    │
    │ [ ] Show internal task IDs                               │
    │ [ ] Verbose LLM logging                                  │
    │                                                          │
    │ Data:                                                    │
    │ [Export All Settings] [Import Settings]                  │
    │ [Reset All to Defaults] (⚠️  destructive)                │
    └──────────────────────────────────────────────────────────┘
    ```
    - Performance tuning
    - Logging configuration
    - Debug options
    - Import/export all settings
    - Nuclear reset option

  - **Key Features**:
    - All settings persist to `.rigger/config.json` (or `.rigger/config.toml`)
    - Secure credential storage using OS keychain
    - Live validation (test API keys, connections)
    - Import/export for portability
    - Settings search (filter by keyword)
    - Help text for each setting (tooltip or sidebar)
    - Visual indicators for required vs optional settings
    - Change tracking: "3 unsaved changes" warning
    - Settings profiles: Dev, Production, Testing

  - **Keyboard Shortcuts**:
    - `,` or `Ctrl+,`: Open settings
    - `Tab`: Navigate between sections
    - `/`: Search settings
    - `Ctrl+S`: Save changes
    - `Esc`: Cancel without saving
    - `Ctrl+R`: Reset current section to defaults

  - **Test Coverage**:
    - Test API key validation (OpenAI, Anthropic, Ollama)
    - Test model availability detection
    - Test database connection
    - Test settings persistence
    - Test import/export
    - Test keybinding conflicts
    - Test secure credential storage

---

## Phase 2: Core CRUD Operations (Weeks 2-3)

**Goal**: Implement full Create, Read, Update, Delete operations for tasks and PRDs with proper state management.

### Sprint 1: Enhanced Task Management (3-4 days)

- [ ] 2.1. Implement TaskDetailView widget
  - Location: rigger_cli/src/commands/tui/widgets/task_detail_view.rs
  - Full task information display:
    - Title, description, status, priority
    - Assignee, due date, tags
    - Parent/child task relationships
    - Enhancement history
    - Comprehension test results
  - Keyboard shortcuts:
    - `Enter`: View selected task details
    - `e`: Edit task
    - `d`: Delete task (with confirmation)
    - `Esc`: Return to task board
  - Test: Navigate and display task details

- [ ] 2.2. Create TaskEditDialog widget
  - Location: rigger_cli/src/commands/tui/widgets/task_edit_dialog.rs
  - Modal dialog for editing task fields
  - Fields:
    - Title (text input)
    - Description (multiline textarea)
    - Status (dropdown: Todo | InProgress | Completed)
    - Priority (dropdown: Low | Medium | High)
    - Assignee (text input with autocomplete)
    - Due date (date picker)
    - Tags (comma-separated input)
    - Parent task (dropdown for task hierarchy)
    - Sub-tasks (list with add/remove/edit buttons)
  - Keyboard shortcuts:
    - `Tab`: Navigate between fields
    - `Enter`: Save changes
    - `Esc`: Cancel without saving
    - `Ctrl+S`: Add sub-task
    - `Ctrl+E`: Edit selected sub-task
    - `Ctrl+D`: Delete selected sub-task
  - Validation: Prevent empty titles, invalid dates, circular dependencies
  - Test: Edit task and verify database persistence

- [ ] 2.2.1. Implement sub-task management
  - Location: rigger_cli/src/commands/tui/widgets/subtask_editor.rs
  - Full CRUD operations for sub-tasks within parent task
  - Features:
    - Nested sub-task view (tree structure)
    - Drag-and-drop reordering (keyboard: `Ctrl+↑`/`↓`)
    - Quick status toggle (`Space` key)
    - Bulk operations (select multiple, change status)
    - Sub-task templates for common patterns
  - Visual representation:
    ```
    Sub-tasks (3/5 completed):
    ✓ [DONE] Set up authentication endpoint
    ✓ [DONE] Add JWT token validation
    ⚡ [IN PROGRESS] Implement password hashing
      └─ Research bcrypt vs argon2
      └─ Add salt generation
    ○ [TODO] Add session management
    ○ [TODO] Write integration tests
    ```
  - Keyboard shortcuts:
    - `s`: Add sub-task
    - `e`: Edit selected sub-task
    - `d`: Delete selected sub-task
    - `Enter`: View/expand sub-task details
    - `Space`: Toggle completion status
    - `Ctrl+↑`/`↓`: Move up/down
  - Percentage completion indicator in parent task
  - Test: Create nested sub-tasks, verify hierarchy persistence

- [ ] 2.2.2. Add task/sub-task enhancement workflow
  - Location: rigger_cli/src/commands/tui/widgets/task_enhancer.rs
  - Keyboard shortcut: `h` (enhance) on any task or sub-task
  - **AI-Powered Enhancement Features**:
    1. **Enhance Description**: Use LLM to expand/clarify task description
    2. **Generate Sub-tasks**: Automatically decompose task into sub-tasks
    3. **Suggest Acceptance Criteria**: Add testable completion criteria
    4. **Estimate Complexity**: Auto-assign complexity score (1-10)
    5. **Add Context**: Pull in related PRD sections or documentation
    6. **Generate Test Cases**: Create comprehension test scenarios
  - Interactive enhancement dialog:
    ```
    ┌─ Enhance Task: TUI-042 ─────────────────────────────────┐
    │ Original:                                               │
    │ "Implement user authentication"                         │
    │                                                         │
    │ AI Suggestions:                                         │
    │ [✓] Expand description with security considerations     │
    │ [✓] Generate 6 sub-tasks (JWT, OAuth, session mgmt...)  │
    │ [✓] Add acceptance criteria (3 items)                   │
    │ [ ] Estimate complexity (suggested: 8/10)               │
    │                                                         │
    │ Enhanced Description Preview:                           │
    │ ┌────────────────────────────────────────────────────┐  │
    │ │ Implement comprehensive user authentication        │  │
    │ │ system with JWT tokens, OAuth2 support, and        │  │
    │ │ secure session management. Must include:           │  │
    │ │ - Password hashing (bcrypt/argon2)                 │  │
    │ │ - Token refresh mechanism                          │  │
    │ │ - Rate limiting for login attempts                 │  │
    │ │ - MFA support (optional)                           │  │
    │ └────────────────────────────────────────────────────┘  │
    │                                                         │
    │ Generated Sub-tasks (6):                                │
    │  1. Set up JWT token generation and validation          │
    │  2. Implement OAuth2 provider integration               │
    │  3. Add password hashing and salt management            │
    │  4. Create session store (Redis/DB)                     │
    │  5. Add login rate limiting middleware                  │
    │  6. Write authentication integration tests              │
    │                                                         │
    │ [a] Apply all  [s] Select individual  [Esc] Cancel      │
    └─────────────────────────────────────────────────────────┘
    ```
  - Options:
    - Apply all suggestions
    - Cherry-pick individual enhancements
    - Regenerate with different parameters
    - Edit before applying
  - Uses existing `EnhanceNode` from orchestration pipeline
  - Enhancement history tracked (see who/when/what was enhanced)
  - Test: Enhance task, verify AI-generated content quality

- [ ] 2.3. Implement task creation workflow
  - Keyboard shortcut: `n` (new task)
  - Opens TaskEditDialog with empty fields
  - Save creates new task in database
  - Immediately refresh task board
  - Test: Create task, verify in database and UI

- [ ] 2.4. Implement task deletion with confirmation
  - Keyboard shortcut: `D` (capital D to prevent accidents)
  - Confirmation dialog: "Delete task 'X'? (y/N)"
  - On yes: Delete from database, refresh UI
  - On no: Cancel and return
  - Test: Delete task, verify removed from database

- [ ] 2.5. Add task filtering and search
  - Location: rigger_cli/src/commands/tui/widgets/filter_bar.rs
  - Filter widget at top of task board
  - Filters:
    - Status (checkboxes: TODO, IN PROGRESS, COMPLETED)
    - Assignee (dropdown with autocomplete)
    - Priority (checkboxes: Low, Medium, High)
    - Date range (start/end date pickers)
    - Text search (title/description)
  - Keyboard shortcut: `/` to focus search bar
  - Real-time filtering as user types
  - Test: Filter by each criterion, verify correct tasks displayed

- [ ] 2.5.1. Add inline task notes/comments (quick win)
  - Keyboard shortcut: `c` (comment) on selected task
  - Quick comment dialog (single line or small textarea)
  - Append timestamp and author
  - Display in task detail view as activity log
  - Useful for: "Blocked by API team", "Waiting on design"
  - No LLM needed - just plain text storage
  - Test: Add comment, verify persistence

- [ ] 2.5.2. Add task duplication
  - Keyboard shortcut: `y` (yank/copy) then `p` (paste)
  - Creates copy of selected task with "Copy of [title]"
  - Useful for: Similar tasks, templates, recurring work
  - Opens edit dialog immediately to modify
  - Sub-tasks also duplicated (optional)
  - Test: Duplicate task, verify all fields copied

- [ ] 2.5.3. Add task archiving (instead of delete)
  - Keyboard shortcut: `a` (archive)
  - Moves task to "Archived" status (hidden by default)
  - Safer than deletion - can be restored
  - View archived tasks: Filter → Show Archived
  - Bulk archive completed tasks: "Clean up completed"
  - Test: Archive and restore tasks

### Sprint 2: PRD Management (3-4 days)

- [ ] 2.6. Create PRDListView widget
  - Location: rigger_cli/src/commands/tui/widgets/prd_list_view.rs
  - New tab: "📄 PRDs"
  - Lists all PRD files from `.rigger/prds/` directory
  - Columns:
    - Filename
    - Last modified date
    - Task count (tasks generated from this PRD)
    - Status (Parsed | Unparsed)
  - Keyboard shortcuts:
    - `↑`/`↓` or `j`/`k`: Navigate PRD list
    - `Enter`: View PRD details
    - `e`: Edit PRD in default editor
    - `p`: Parse PRD (generate tasks)
    - `n`: Create new PRD
    - `D`: Delete PRD (with confirmation)
  - Test: Display PRD list, navigate, select

- [ ] 2.7. Implement PRD viewer
  - Location: rigger_cli/src/commands/tui/widgets/prd_viewer.rs
  - Full-screen PRD markdown viewer
  - Syntax highlighting for markdown
  - Sections highlighted:
    - Title (# Header)
    - Objectives (## Objectives)
    - Tech Stack (## Tech Stack)
    - Constraints (## Constraints)
  - Keyboard shortcuts:
    - `↑`/`↓` or `j`/`k`: Scroll content
    - `e`: Edit PRD
    - `p`: Parse PRD (generate tasks)
    - `Esc`: Return to PRD list
  - Test: View PRD, scroll, highlight sections

- [ ] 2.8. Integrate external editor for PRD editing
  - Use `$EDITOR` environment variable (fallback: `vim`, `nano`)
  - Keyboard shortcut: `e` from PRD viewer
  - Workflow:
    1. Suspend TUI (leave alternate screen)
    2. Open PRD file in editor (e.g., `vim .rigger/prds/auth-feature.md`)
    3. Wait for editor to close
    4. Resume TUI (re-enter alternate screen)
    5. Reload PRD content if modified
  - Test: Edit PRD, verify changes persist

- [ ] 2.9. Implement PRD creation wizard
  - Keyboard shortcut: `n` from PRD list
  - Interactive wizard:
    1. Enter PRD filename
    2. Enter project title
    3. Add objectives (press Enter to add, empty to finish)
    4. Add tech stack items
    5. Add constraints
    6. Confirm and save
  - Generates markdown file in `.rigger/prds/`
  - Opens editor for further editing (optional)
  - Test: Create PRD via wizard, verify markdown structure

- [ ] 2.10. Add PRD parsing with progress indicator
  - Keyboard shortcut: `p` from PRD viewer/list
  - Show progress modal:
    - "Parsing PRD with LLM..."
    - Spinner animation
    - Live status updates (if possible)
  - On completion:
    - Display success message: "Generated 5 tasks"
    - Automatically switch to task board view
    - Highlight newly created tasks
  - On error:
    - Display error message
    - Offer to retry or cancel
  - Test: Parse PRD, verify tasks created in database

---

## Phase 3: Real-Time Orchestration Monitoring (Weeks 3-4)

**Goal**: Visualize the orchestration pipeline in real-time, showing LLM reasoning, network calls, and state transitions.

### Sprint 3: Live Orchestration Execution (4-5 days)

- [ ] 3.1. Create OrchestratorView widget
  - Location: rigger_cli/src/commands/tui/widgets/orchestrator_view.rs
  - New tab: "⚙️ Orchestrator"
  - Layout (split screen):
    - **Left**: Task queue (pending orchestration)
    - **Right**: Active orchestration details
  - Shows tasks currently being processed
  - Keyboard shortcuts:
    - Select task → `Enter`: Execute orchestration
    - `s`: Stop current orchestration
    - `r`: Retry failed orchestration
  - Test: Display orchestration queue

- [ ] 3.2. Implement task execution from TUI
  - Keyboard shortcut: `Enter` on selected task in orchestrator view
  - Equivalent to `rig do <task-id>` in CLI
  - Workflow:
    1. Mark task as "In Progress"
    2. Spawn async orchestration task (tokio::spawn)
    3. Stream progress updates to UI
    4. Update UI in real-time as nodes execute
  - Test: Execute task, verify database updates

- [ ] 3.3. Add real-time ThinkingWidget updates
  - Replace mock data with live chain-of-thought logs
  - Listen to orchestration events:
    - Node entry: "Entering SemanticRouterNode..."
    - LLM call start: "Calling llama3.1 for routing decision..."
    - LLM response: "Complexity score: 7 (high)"
    - Routing decision: "Route: decompose"
  - Auto-scroll to latest entry
  - Color-coded log levels:
    - Info: White
    - Decision: Yellow
    - Success: Green
    - Error: Red
  - Test: Run orchestration, verify live logs appear

- [ ] 3.4. Add real-time NetworkLogWidget updates
  - Replace mock data with actual API requests/responses
  - Capture from Rig/Ollama HTTP calls:
    - Request: `→ POST http://localhost:11434/api/chat (llama3.1)`
    - Response: `← 200 OK (1.2s) {tokens: 156}`
  - Display:
    - Timestamp
    - HTTP method and URL
    - Model used
    - Response time
    - Response status (color-coded: green=2xx, yellow=4xx, red=5xx)
    - Token count (if available)
  - Test: Run orchestration, verify network logs populate

- [ ] 3.5. Create StateGraphVisualizer widget
  - Location: rigger_cli/src/commands/tui/widgets/state_graph_viz.rs
  - Visual representation of graph_flow state machine
  - ASCII art graph showing:
    - Nodes: [SemanticRouter] → [Enhance] → [Test] → [Check] → [End]
    - Current node highlighted
    - Completed nodes: Green
    - Active node: Yellow (blinking)
    - Pending nodes: Gray
  - Updates in real-time as orchestration progresses
  - Example:
    ```
    [Start] ✓
       ↓
    [SemanticRouter] ✓ (complexity: 7)
       ↓
    [TaskDecomposition] ⚡ (processing...)
       ↓
    [End] ⏸
    ```
  - Test: Run orchestration, verify graph animates

### Sprint 4: Enhanced Progress Indicators (2-3 days)

- [ ] 3.6. Add progress bars for long-running operations
  - LLM inference: Show progress bar while waiting for response
  - Estimated time remaining (if available)
  - Cancellation support (`Ctrl+C` or `s` key)
  - Test: Trigger long LLM call, verify progress bar

- [ ] 3.7. Implement completion notifications
  - On orchestration success:
    - Toast notification: "✓ Task orchestration complete"
    - Sound notification (optional, configurable)
  - On orchestration failure:
    - Toast notification: "✗ Orchestration failed: [error message]"
    - Highlight task in red on task board
  - Test: Complete orchestration, verify notification

- [ ] 3.8. Add orchestration history log
  - New tab: "📜 History"
  - Table view of all past orchestrations:
    - Task title
    - Start time
    - Duration
    - Final status (Success | Failed | Cancelled)
    - Model(s) used
  - Sortable by column
  - Click to view full execution log
  - Test: Run multiple orchestrations, verify history

---

## Phase 4: Analytics & Visualizations (Weeks 5-6)

**Goal**: Provide rich analytics and data visualizations to understand task performance, model efficiency, and project health.

### Sprint 5: Task Analytics Dashboard (3-4 days)

- [ ] 4.1. Create AnalyticsDashboard widget
  - Location: rigger_cli/src/commands/tui/widgets/analytics_dashboard.rs
  - New tab: "📊 Analytics"
  - Multi-panel layout:
    1. Task Overview (top-left)
    2. Completion Trend (top-right)
    3. Model Performance (bottom-left)
    4. Complexity Distribution (bottom-right)
  - Auto-refresh every 30 seconds
  - Test: Display analytics dashboard

- [ ] 4.2. Implement Task Overview panel
  - Metrics:
    - Total tasks: 42
    - TODO: 15 (36%)
    - IN PROGRESS: 8 (19%)
    - COMPLETED: 19 (45%)
  - Visual representation:
    - Horizontal stacked bar chart
    - Color-coded segments
  - Test: Display task counts

- [ ] 4.3. Add Completion Trend chart
  - Line/bar chart showing tasks completed over time
  - X-axis: Date (last 7/30 days, configurable)
  - Y-axis: Task count
  - ASCII art chart using `tui-rs` block characters
  - Example:
    ```
    Tasks Completed (Last 7 Days)
    10 │     ╭─╮
     9 │   ╭─╯ │
     8 │ ╭─╯   ╰─╮
     7 │─╯       ╰─
    ───┴──────────────
       Mon  Wed  Fri
    ```
  - Test: Generate test data, verify chart renders

- [ ] 4.4. Create Model Performance panel
  - Table showing model statistics:
    - Model name (llama3.1, phi3, orca2, mistral)
    - Total calls
    - Avg response time (ms)
    - Avg tokens/second
    - Success rate (%)
  - Sortable by column
  - Highlight best-performing model (green)
  - Test: Load metrics from .rigger/metrics.jsonl, display table

- [ ] 4.5. Add Complexity Distribution histogram
  - Bar chart showing task complexity distribution
  - X-axis: Complexity score (1-10)
  - Y-axis: Task count
  - Color gradient: Green (low) → Yellow (med) → Red (high)
  - Test: Calculate complexity distribution, render chart

### Sprint 6: Advanced Metrics (2-3 days)

- [ ] 4.6. Add time-to-completion metrics
  - For each task, track:
    - Time from creation → completion
    - Time in each status (TODO, IN PROGRESS)
    - Avg time per complexity level
  - Display in analytics dashboard
  - Test: Calculate and display metrics

- [ ] 4.7. Implement burndown chart
  - X-axis: Date
  - Y-axis: Remaining tasks
  - Ideal burndown line (linear)
  - Actual burndown line
  - Useful for sprint planning
  - Test: Generate burndown chart

- [ ] 4.8. Add LLM cost estimation
  - Calculate estimated cost based on:
    - Token count
    - Model pricing (configurable per model)
  - Display total cost per task, per project
  - Test: Estimate costs accurately

---

## Phase 5: Advanced Features & Polish (Weeks 7-8)

**Goal**: Add power-user features, customization, and polish to make the TUI production-ready.

### Sprint 7: Customization & Theming (2-3 days)

- [ ] 5.1. Implement color theme system
  - Location: rigger_cli/src/commands/tui/themes/mod.rs
  - Built-in themes:
    - Default (current)
    - Dark (high contrast)
    - Light (for bright terminals)
    - Dracula
    - Nord
    - Solarized
  - Configuration: `.rigger/config.json` → `tui.theme`
  - Keyboard shortcut: `t` to cycle themes
  - Test: Switch themes, verify colors change

- [ ] 5.2. Add configurable keybindings
  - Location: .rigger/keybindings.toml
  - Allow users to rebind all shortcuts
  - Example:
    ```toml
    [navigation]
    next_tab = "Tab"
    prev_tab = "Shift+Tab"
    quit = ["q", "Esc"]

    [tasks]
    new_task = "n"
    edit_task = "e"
    delete_task = "D"
    ```
  - Fallback to defaults if file missing
  - Test: Rebind keys, verify new bindings work

- [ ] 5.3. Implement layout customization
  - Users can choose layout presets:
    - Default (current)
    - Wide (task board takes 70% width)
    - Focused (single-column, larger task cards)
    - Dashboard (analytics-first view)
  - Configuration: `.rigger/config.json` → `tui.layout`
  - Test: Switch layouts, verify rendering

### Sprint 8: Power User Features (3-4 days)

- [ ] 5.4. Add bulk operations
  - Multi-select tasks (Space bar)
  - Bulk actions:
    - Change status (`s` → select status)
    - Assign to user (`a` → enter assignee)
    - Add tags (`t` → enter tags)
    - Delete (`D` with confirmation)
  - Visual feedback: Selected tasks highlighted
  - Test: Select multiple tasks, apply bulk action

- [ ] 5.5. Implement task dependencies visualization
  - If task has `depends_on` field, show dependency graph
  - New view: "🔗 Dependencies"
  - ASCII art graph showing task relationships
  - Example:
    ```
    [Task A] ──┬─→ [Task C]
               │
    [Task B] ──┘
    ```
  - Highlight blocked tasks (dependencies not complete)
  - Test: Create tasks with dependencies, display graph

- [ ] 5.6. Add export functionality
  - Export current view to file:
    - Task board → CSV
    - Analytics → JSON
    - Orchestration log → Markdown
  - Keyboard shortcut: `x` (export)
  - Prompt for filename and format
  - Test: Export each view, verify file contents

- [ ] 5.7. Implement task templates
  - Location: .rigger/templates/
  - Predefined task templates (e.g., "Bug Report", "Feature Request")
  - Keyboard shortcut: `T` (new from template)
  - Select template → Fill in placeholders → Save
  - Test: Create task from template

- [ ] 5.8. Add undo/redo for edit operations
  - Track edit history (last 20 operations)
  - Keyboard shortcuts:
    - `u`: Undo
    - `Ctrl+R`: Redo
  - Operations tracked:
    - Task edits
    - Task deletions
    - Bulk operations
  - Test: Edit task, undo, redo

### Sprint 9: Final Polish & Testing (3-4 days)

- [ ] 5.9. Performance optimization
  - Lazy loading for large task lists (virtual scrolling)
  - Debounce search input to reduce re-renders
  - Profile with `cargo flamegraph`, optimize hotspots
  - Target: <16ms render time for 60 FPS
  - Test: Load 1000 tasks, verify smooth scrolling

- [ ] 5.10. Error handling & recovery
  - Graceful degradation if database is locked
  - Clear error messages for all failure modes
  - Auto-retry for transient failures (network, LLM)
  - Log errors to .rigger/tui.log
  - Test: Trigger errors, verify recovery

- [ ] 5.11. Comprehensive help system
  - Contextual help for each view
  - Keyboard shortcut: `?` (help)
  - Shows available shortcuts for current view
  - Searchable help index
  - Test: Open help from each view

- [ ] 5.12. Integration tests
  - End-to-end TUI tests using `tui-test-backend`
  - Test scenarios:
    - Full CRUD lifecycle (create, read, update, delete task)
    - PRD parsing workflow
    - Orchestration execution
    - Analytics calculation
  - CI/CD integration
  - Test: All scenarios pass

- [ ] 5.13. Documentation
  - Update README.md with TUI features
  - Add docs/TUI_GUIDE.md with screenshots (ASCII recordings)
  - Record demo GIF using `asciinema` → `svg-term-cli`
  - Test: Verify documentation accuracy

---

## Phase 6: Bonus Features (Future)

**Status**: Optional enhancements after core TUI is production-ready.

- [ ] 6.1. Mouse support (optional)
  - Click to select tasks, tabs
  - Scroll with mouse wheel
  - Configurable (some users prefer keyboard-only)

- [ ] 6.2. Multi-pane layout (tmux-style)
  - Split screen into multiple views
  - Watch multiple metrics simultaneously
  - Keyboard shortcuts to manage panes

- [ ] 6.3. Collaborative features
  - Show which tasks are being edited by other users (via WebSocket)
  - Real-time updates when tasks change
  - Conflict resolution for concurrent edits

- [ ] 6.4. Plugin system
  - Allow custom widgets via Lua/WASM
  - Community-contributed visualizations
  - Plugin marketplace

---

## Technical Architecture

### State Management

Use a centralized state management pattern inspired by Elm/Redux:

```rust
struct AppState {
    tasks: Vec<Task>,
    prds: Vec<PRDInfo>,
    selected_task: Option<usize>,
    selected_prd: Option<usize>,
    current_tab: TabIndex,
    filter: TaskFilter,
    orchestration_status: OrchestrationStatus,
    metrics: MetricsData,
}

enum AppEvent {
    TaskSelected(usize),
    TaskCreated(Task),
    TaskUpdated(Task),
    TaskDeleted(Uuid),
    PRDParsed(PRDResult),
    OrchestrationProgress(OrchestrationEvent),
    RefreshRequested,
}

fn update(state: &mut AppState, event: AppEvent) -> Result<(), Error> {
    // Immutable state updates
    match event {
        AppEvent::TaskSelected(idx) => state.selected_task = Some(idx),
        // ...
    }
}
```

### Widget Hierarchy

```
App (main.rs)
├── TabBar
├── TaskBoardWidget
│   ├── FilterBar
│   ├── TaskColumn (TODO)
│   ├── TaskColumn (IN PROGRESS)
│   └── TaskColumn (COMPLETED)
├── TaskDetailView
│   ├── TaskMetadata
│   ├── EnhancementHistory
│   └── TestResults
├── PRDListView
│   ├── PRDListItem
│   └── PRDViewer
├── OrchestratorView
│   ├── TaskQueue
│   ├── StateGraphVisualizer
│   ├── ThinkingWidget
│   └── NetworkLogWidget
├── AnalyticsDashboard
│   ├── TaskOverviewPanel
│   ├── CompletionTrendChart
│   ├── ModelPerformanceTable
│   └── ComplexityHistogram
├── HelpScreen
├── StatusFooter (always visible, bottom)
│   ├── AnimatedStatusBar
│   ├── MessageCarousel
│   └── ProgressIndicators
└── StatusDetailPanel (expandable overlay from footer)
    ├── ActiveOrchestrations
    ├── ResearchActivity
    ├── TaskProgress
    ├── RecentCompletions
    └── TodayMetrics
```

### Global Keyboard Shortcuts

These shortcuts work from any view:

Use `tokio::sync::mpsc` channels for async communication:

```rust
// Main event loop
let (tx, mut rx) = mpsc::channel(100);

// Spawn async task for orchestration
let tx_clone = tx.clone();
tokio::spawn(async move {
    let events = run_orchestration(task_id).await;
    for event in events {
        tx_clone.send(AppEvent::OrchestrationProgress(event)).await.unwrap();
    }
});

// UI update loop
loop {
    terminal.draw(|f| ui(f, &app))?;

    if let Ok(event) = rx.try_recv() {
        update(&mut app.state, event)?;
    }

    // Handle keyboard input
    if event::poll(Duration::from_millis(100))? {
        // ...
    }
}
```

### Database Integration

Reuse existing SQLite adapters:

```rust
// Load tasks
let adapter = SqliteTaskAdapter::connect_and_init("sqlite:.rigger/tasks.db").await?;
let tasks = adapter.find_async(&TaskFilter::All, FindOptions::default()).await?;

// Save task
adapter.save_async(&updated_task).await?;
```

### Testing Strategy

1. **Unit Tests**: Individual widget logic
2. **Integration Tests**: Full state update cycles
3. **End-to-End Tests**: TUI automation with `tui-test-backend`
4. **Manual Testing**: User acceptance testing with stakeholders

---

## Success Metrics

- ✅ 100% feature parity with CLI
- ✅ All configuration accessible via TUI (no manual file editing required)
- ✅ Secure credential storage (API keys, tokens)
- ✅ Multi-project support with seamless switching
- ✅ Sub-second spotlight search (<500ms for 1000+ items)
- ✅ Real-time activity updates (max 2s latency)
- ✅ <100ms latency for all UI interactions
- ✅ Supports 1000+ tasks without performance degradation
- ✅ 95%+ keyboard navigation (minimal mouse dependency)
- ✅ Zero data loss (all operations persist correctly)
- ✅ AI-powered task enhancement with >80% useful suggestion rate
- ✅ Full sub-task hierarchy support (unlimited nesting)
- ✅ Comprehensive documentation with examples
- ✅ Positive user feedback ("This is awesome!" + "I can't work without this anymore!")

---

## References

- **Ratatui Documentation**: https://ratatui.rs/
- **Crossterm Documentation**: https://docs.rs/crossterm/
- **TUI Design Patterns**: https://github.com/ratatui-org/ratatui/tree/main/examples
- **Inspiration**: k9s (Kubernetes TUI), lazygit, btop

---

**Let's make this the most awesome terminal UI for AI project management!** 🚀
