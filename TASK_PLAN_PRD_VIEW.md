---
task_id: PRD_VIEW_IMPLEMENTATION
status: planning
---

# Task: Implement PRD View - Unified Task Context Display with Inline Editing

## Overview

Create a new "PRD View" in the Tools navigation that displays all tasks related to a selected PRD as a single, scrollable Markdown document. Each task appears as a bordered section with colors matching its Kanban status, providing a comprehensive view of the entire PRD context. Support inline editing and priority reordering.

## Requirements

### Functional Requirements
1. **View Access**: Navigate to PRD View from Tools menu (similar to Artifacts, Metrics, DevTools)
2. **PRD Selection**: Choose which PRD to view (default to current/most recent)
3. **Task Display**: Show all tasks for selected PRD in list format
4. **Status Coloring**: Border colors match Kanban board status colors
5. **Markdown Rendering**: Each task section rendered as formatted markdown
6. **Inline Editing**: Click to edit task title, description, or details
7. **Priority Reordering**: Move tasks up/down with keyboard shortcuts
8. **Scrolling**: Support long documents with smooth scrolling
9. **Task Navigation**: Jump between task sections quickly

### Non-Functional Requirements
- Responsive rendering for large PRDs (100+ tasks)
- Consistent color scheme with existing UI
- Accessible keyboard navigation
- State persistence when switching views

## Plan

### Phase 1: Data Model & Query Layer
- [ ] 1.1. Add PRD selection state to App struct (selected_prd_id, prd_view_tasks)
- [ ] 1.2. Implement query to fetch all tasks by PRD ID from task repository
- [ ] 1.3. Add sort_order field support for task prioritization
- [ ] 1.4. Create helper to group tasks by PRD relationship (parent tasks, subtasks)
- [ ] 1.5. Add PRD metadata loading (title, objectives, constraints)

### Phase 2: View Infrastructure
- [ ] 2.1. Add PRDView variant to DashboardTool enum
- [ ] 2.2. Add "📋 PRD View" to tool navigation cycle
- [ ] 2.3. Create prd_view_selected field for current task selection within view
- [ ] 2.4. Add prd_view_scroll_offset for vertical scrolling
- [ ] 2.5. Implement load_prd_view_data() method to populate view state

### Phase 3: Rendering - Task Cards
- [ ] 3.1. Create render_prd_view() function skeleton
- [ ] 3.2. Implement task card rendering with status-colored borders:
  - Todo: Blue border
  - InProgress: Yellow border
  - Completed: Green border
  - Blocked: Red border
- [ ] 3.3. Render task title as H2 markdown header
- [ ] 3.4. Render task description as markdown body
- [ ] 3.5. Add task metadata footer (assignee, complexity, priority)
- [ ] 3.6. Add visual separator between task cards

### Phase 4: Rendering - Document Layout
- [ ] 4.1. Create PRD header section (title, objectives summary)
- [ ] 4.2. Add task count and status distribution summary
- [ ] 4.3. Implement scrollable viewport with ratatui List widget
- [ ] 4.4. Add scroll indicators (top/bottom of list)
- [ ] 4.5. Highlight currently selected task card
- [ ] 4.6. Add empty state handling (no tasks for PRD)

### Phase 5: Navigation & Selection
- [ ] 5.1. Implement Up/Down arrow keys to navigate between task cards
- [ ] 5.2. Implement Home/End keys to jump to first/last task
- [ ] 5.3. Add Page Up/Page Down for faster scrolling
- [ ] 5.5. Show PRD selector dialog if multiple PRDs available
- [ ] 5.6. Add breadcrumb showing current PRD in status bar

### Phase 6: Inline Editing
- [ ] 6.1. Add 'enter' key to enter edit mode for selected task
- [ ] 6.2. Create inline edit dialog overlaying task card
- [ ] 6.3. Support editing title, description, assignee
- [ ] 6.4. Implement 'Enter' to save edits, 'Esc' to cancel
- [ ] 6.5. Add validation for required fields
- [ ] 6.6. Update task in repository on save
- [ ] 6.7. Refresh view after edit without losing scroll position

### Phase 7: Priority Reordering
- [ ] 7.1. Implement Shift+Up to move task up in priority
- [ ] 7.2. Implement Shift+Down to move task down in priority
- [ ] 7.3. Update sort_order field in database
- [ ] 7.4. Show visual feedback during move (highlight, animation)
- [ ] 7.5. Persist reordering across view changes
- [ ] 7.6. Sync reordering with Kanban board view

### Phase 8: Additional Features
- [ ] 8.1. Add 'f' key to filter tasks by status
- [ ] 8.2. Implement search within PRD view (Ctrl+F)
- [ ] 8.3. Add 'x' key to mark task complete from PRD view
- [ ] 8.4. Show subtask hierarchy with indentation
- [ ] 8.5. Add task dependencies visualization
- [ ] 8.6. Export entire PRD view to markdown file

### Phase 9: Polish & Testing
- [ ] 9.1. Add help text footer showing available keybindings
- [ ] 9.2. Implement smooth scrolling animations
- [ ] 9.3. Add loading spinner while fetching PRD tasks
- [ ] 9.4. Test with empty PRD (no tasks)
- [ ] 9.5. Test with large PRD (100+ tasks)
- [ ] 9.6. Test all edit and reorder operations
- [ ] 9.7. Write unit tests for task grouping logic
- [ ] 9.8. Write integration test for full PRD view workflow

## Current Step

- **Action:** Planning complete, ready to begin implementation
- **Details:** Task plan created with 9 phases covering data model, rendering, navigation, editing, and reordering. Next step is Phase 1: Data Model & Query Layer.

## Technical Design Notes

### Color Mapping (Status → Border Color)
```rust
match task.status {
    TaskStatus::Todo => Color::Blue,
    TaskStatus::InProgress => Color::Yellow,
    TaskStatus::Completed => Color::Green,
    TaskStatus::Blocked => Color::Red,
    TaskStatus::OnHold => Color::Magenta,
    TaskStatus::Cancelled => Color::DarkGray,
}
```

### Task Card Layout
```
╔════════════════════════════════════════╗ Blue border (Todo)
║ ## Task Title                          ║
║                                        ║
║ Task description rendered as markdown  ║
║ with proper formatting and wrapping.   ║
║                                        ║
║ 👤 Assignee: Backend Dev               ║
║ 🎯 Complexity: 5  📌 Priority: High    ║
╚════════════════════════════════════════╝
```

### File Locations
- Main view logic: `rigger_cli/src/commands/tui.rs`
- Add PRDView to DashboardTool enum around line 750
- Add render_prd_view() function around line 9500
- Add keyboard handlers around line 6000

### Key Bindings
- `Tab` / `Shift+Tab`: Switch between tools (existing)
- `Up` / `Down`: Navigate between task cards
- `Home` / `End`: Jump to first/last task
- `Page Up` / `Page Down`: Fast scroll
- `e`: Edit selected task
- `Shift+Up` / `Shift+Down`: Reorder task priority
- `p`: Switch between PRDs
- `f`: Filter by status
- `Ctrl+F`: Search within PRD
- `x`: Mark task complete
- `?`: Show help

## Dependencies

### Existing Components to Leverage
- Task repository (`task_manager::adapters::sqlite_task_adapter`)
- PRD repository (`task_manager::adapters::sqlite_prd_adapter`)
- Task formatter (`rigger_cli::src::services::task_formatter`)
- Markdown rendering widgets (from existing dialogs)
- Status colors (from Kanban board implementation)

### New Components Needed
- Task card renderer with bordered layout
- PRD task query with sort_order support
- Inline edit dialog for task fields
- Reordering service to update sort_order atomically

## Blockers

None currently. All dependencies are available in existing codebase.

## Success Criteria

1. Can navigate to PRD View from Tools menu
2. Can see all tasks for a PRD in single scrollable view
3. Task borders match Kanban status colors
4. Can edit any task inline with 'e' key
5. Can reorder tasks with Shift+Up/Down
6. Changes persist across view switches
7. Large PRDs (100+ tasks) render smoothly
8. All keyboard shortcuts work as documented

## Future Enhancements (Out of Scope)

- Export to PDF with formatting
- Collaborative editing with conflict resolution
- Task templates for common patterns
- Diff view showing PRD changes over time
- AI-powered task suggestions within PRD context
