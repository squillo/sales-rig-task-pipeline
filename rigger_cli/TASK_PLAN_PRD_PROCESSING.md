---
task_id: prd-processing-enhancement
status: in-progress
created: 2025-11-25T13:50:00Z
updated: 2025-11-25T20:50:00Z
---

# Task: Enhanced PRD Processing UX with Real-Time Progress

## Problem Statement

When user presses Enter on a markdown file in the browser:
1. **No immediate feedback** - User doesn't know processing has started
2. **No progress updates** - No visibility into what's happening during LLM generation
3. **No task breakdown visibility** - Can't see tasks/subtasks as they're being created
4. **No cancellation** - Can't abort long-running LLM requests
5. **Bug on Enter** - Crashes immediately when starting PRD processing

## Current Flow

```
User presses 'm'
  → Markdown browser opens
  → User navigates with ↑/↓
  → User presses Enter
  → ??? (crashes here - bug to investigate)
```

## Desired Flow

```
User presses Enter on markdown file
  ↓
Immediate visual transition to PRD Processing View
  ↓
Show processing header:
  - File name
  - Model being used (e.g., "llama3.2:latest via Ollama")
  - Timestamp started
  ↓
Progress steps (with live updates):
  ☐ Reading PRD file... (0.1s)
  ☐ Parsing PRD structure... (0.2s)
  ☐ Analyzing objectives and constraints... (0.5s)
  ☐ Generating tasks via LLM... (15-60s)
      ├─ Task 1: [title] ✓
      ├─ Task 2: [title] ✓
      ├─ Task 3: [title] ⏳
      └─ Task 4: [title] ...
  ☐ Saving tasks to database... (0.5s)
  ✓ Complete!
  ↓
Show summary:
  - Total tasks generated
  - Complexity breakdown
  - Estimated effort
  - Next actions
  ↓
User can:
  - Press Enter to view tasks in Kanban
  - Press 'Esc' to return to markdown browser
```

## Plan

### Task 1: Fix Immediate Crash Bug
**Status**: ✅ COMPLETED
**Priority**: P0

- [x] 1.1. Capture stack trace from crash
- [x] 1.2. Identify crashing code path (likely in `create_prd_from_markdown()`)
- [x] 1.3. Add error handling for the failure point
- [x] 1.4. Test fix with sample PRD

**Root Cause 1 (Runtime Error)**: The `save()` method from hexser::ports::Repository is synchronous but internally uses `block_on()` to create a new Tokio runtime at `task_manager/src/adapters/sqlite_task_adapter.rs:67:12`. This fails when called from within an already-running async context (`create_prd_from_markdown()`).

**Fix 1 Applied**: Changed both `src/commands/tui.rs:1493` and `src/commands/parse.rs:109` to use `adapter.save_async(task.clone()).await` instead of the blocking `adapter.save(task.clone())`.

**Root Cause 2 (Buffer Overflow)**: The `render_prd_processing()` function at line 7608 calculates `dialog_height = lines.len() as u16 + 2` without checking if it exceeds available buffer space. Long error messages (especially multi-line Ollama errors) can create 20+ lines, causing the dialog to exceed the terminal height of 35 and trigger "index outside of buffer" panic.

**Fix 2 Applied**:
- `render_prd_processing()` (line 7609): Changed to `std::cmp::min(lines.len() as u16 + 2, area.height.saturating_sub(4))`
- `render_wizard_complete()` (line 7474): Same fix applied to prevent similar overflow

**Acceptance Criteria**:
- ✅ Pressing Enter on markdown file doesn't crash with runtime error
- ✅ Pressing Enter on markdown file doesn't crash with buffer overflow
- ✅ Error messages are displayed gracefully (truncated if too long)
- ✅ Build succeeds without errors

---

### Task 2: Implement Progressive Status Display
**Status**: ⏳ Ready after Task 1
**Priority**: P0

- [ ] 2.1. Refactor `create_prd_from_markdown()` to emit progress events
- [ ] 2.2. Add `prd_processing_current_step` enum to App state
- [ ] 2.3. Add `prd_processing_substeps` Vec<String> to track task titles
- [ ] 2.4. Update `render_prd_processing()` to show step-by-step progress
- [ ] 2.5. Add timestamp tracking for each step

**Design**:
```rust
enum PRDProcessingStep {
    ReadingFile,
    ParsingStructure,
    AnalyzingContent,
    GeneratingTasks { current: usize, total: usize },
    SavingToDatabase,
    Complete,
}

struct App {
    // ... existing fields
    prd_processing_current_step: PRDProcessingStep,
    prd_processing_substeps: Vec<(String, TaskStatus)>,  // (task_title, status)
    prd_processing_start_time: Option<DateTime<Utc>>,
}
```

**Rendering**:
```
┌─────────────────────────────────────────────────┐
│  Processing PRD: authentication.md              │
│  Model: llama3.2:latest (Ollama)                │
│  Started: 13:45:22                              │
├─────────────────────────────────────────────────┤
│  ✓ Reading PRD file... (0.1s)                   │
│  ✓ Parsing PRD structure... (0.2s)              │
│  ✓ Analyzing objectives... (0.5s)               │
│  ⏳ Generating tasks via LLM... (23s elapsed)   │
│     ├─ ✓ Implement OAuth2 login                 │
│     ├─ ✓ Add SAML support                       │
│     ├─ ⏳ Create user profile system            │
│     └─ ... (2 more)                             │
│  ☐ Saving to database...                        │
│  ☐ Complete                                     │
├─────────────────────────────────────────────────┤
│  Press 'c' to cancel                            │
└─────────────────────────────────────────────────┘
```

**Acceptance Criteria**:
- Each step shows with checkmark when complete
- LLM generation shows task titles as they're created
- Elapsed time updates every second
- Smooth visual transitions between steps

---

### Task 3: Add Task Streaming from LLM
**Status**: ⏳ Ready after Task 2
**Priority**: P1

- [ ] 3.1. Modify `RigPRDParserAdapter` to support streaming responses
- [ ] 3.2. Emit task title immediately when LLM generates it
- [ ] 3.3. Update App state with partial results
- [ ] 3.4. Render tasks as they arrive (not just at the end)

**Implementation Notes**:
- Ollama supports streaming via `/api/generate` endpoint
- Parse JSON incrementally for task objects
- Show "⏳ Generating..." for in-progress tasks

**Acceptance Criteria**:
- Tasks appear one-by-one as LLM generates them
- User sees progress even during long LLM calls
- No degradation in final task quality

---

### Task 4: Implement Cancellation Support
**Status**: ⏳ Ready after Task 2
**Priority**: P1

- [ ] 4.1. Add `prd_processing_cancel_requested` flag to App state
- [ ] 4.2. Handle 'c' key press during processing
- [ ] 4.3. Add cancellation check between LLM tasks
- [ ] 4.4. Cancel ongoing HTTP request to LLM (if possible)
- [ ] 4.5. Clean up partial results on cancellation
- [ ] 4.6. Show cancellation confirmation dialog

**Design**:
```rust
// In keyboard handler
KeyCode::Char('c') if app.show_prd_processing => {
    app.open_confirmation(
        "Cancel PRD Processing?",
        "This will discard all tasks generated so far.",
        ConfirmationAction::CancelPRDProcessing,
    );
}

// In create_prd_from_markdown()
for task in llm_generated_tasks {
    if self.prd_processing_cancel_requested {
        self.prd_processing_error = Some("Cancelled by user".to_string());
        return Err(anyhow::anyhow!("Processing cancelled"));
    }
    // Save task...
}
```

**Acceptance Criteria**:
- 'c' key opens confirmation dialog
- Confirming cancellation stops processing
- Partial results are discarded
- User returns to markdown browser
- No orphaned database records

---

### Task 5: Add Progress Bar and ETA
**Status**: ⏳ Ready after Task 2
**Priority**: P2

- [ ] 5.1. Track total steps vs completed steps
- [ ] 5.2. Estimate remaining time based on elapsed time per step
- [ ] 5.3. Render progress bar at top of dialog
- [ ] 5.4. Show ETA (e.g., "~15s remaining")

**Design**:
```
┌─────────────────────────────────────────────────┐
│  [████████████░░░░░░░░░░░] 60% (~10s remaining) │
└─────────────────────────────────────────────────┘
```

**Acceptance Criteria**:
- Progress bar updates smoothly
- ETA is reasonably accurate
- Bar fills to 100% on completion

---

### Task 6: Add Post-Processing Summary
**Status**: ⏳ Ready after Task 2
**Priority**: P2

- [ ] 6.1. Calculate task statistics (count, complexity breakdown)
- [ ] 6.2. Estimate total effort (sum of complexity scores)
- [ ] 6.3. Show summary screen on completion
- [ ] 6.4. Add "View in Kanban" and "Create Another" buttons

**Design**:
```
┌─────────────────────────────────────────────────┐
│  ✓ PRD Processing Complete                      │
├─────────────────────────────────────────────────┤
│  File: authentication.md                        │
│  Duration: 42s                                  │
│  Tasks Generated: 12                            │
│                                                 │
│  Complexity Breakdown:                          │
│  🔹 Low (1-3):    4 tasks                       │
│  🔸 Medium (4-7): 6 tasks                       │
│  🔺 High (8-10):  2 tasks                       │
│                                                 │
│  Estimated Effort: 68 complexity points         │
│                                                 │
│  Next Actions:                                  │
│  - View tasks in Kanban board                   │
│  - Assign tasks to team members                 │
│  - Start working on highest priority            │
├─────────────────────────────────────────────────┤
│  [Enter] View in Kanban  [m] New PRD  [Esc] Close│
└─────────────────────────────────────────────────┘
```

**Acceptance Criteria**:
- Summary shows after successful completion
- Statistics are accurate
- Keyboard shortcuts work as expected

---

### Task 7: Enhanced Error Handling with Retry
**Status**: ⏳ Ready after Task 1
**Priority**: P2

- [ ] 7.1. On LLM error, show diagnostic results inline
- [ ] 7.2. Add "Retry" button to error screen
- [ ] 7.3. Preserve PRD file selection on retry
- [ ] 7.4. Log error details for debugging

**Design**:
```
┌─────────────────────────────────────────────────┐
│  🔴 PRD Processing Failed                       │
├─────────────────────────────────────────────────┤
│  File: authentication.md                        │
│  Error: Connection refused                      │
│                                                 │
│  🔍 Diagnostics:                                │
│  ✓ Ollama is installed                          │
│  ✓ Version: 0.12.9                              │
│  ❌ Ollama is not running                       │
│                                                 │
│  Fix: Start Ollama service                      │
│  → ollama serve                                 │
├─────────────────────────────────────────────────┤
│  [r] Retry  [Esc] Close                         │
└─────────────────────────────────────────────────┘
```

**Acceptance Criteria**:
- Diagnostics run automatically on error
- Retry button preserves all context
- User can fix issue and retry without restarting

---

## Current Step

**Action**: Tasks 1 & 2 complete - State machine implemented with real-time progress bar

**Status**: Core functionality working. LLM JSON parsing improved.

**Completed**:
1. ✅ Task 1.1-1.4: Fixed runtime error and buffer overflow crashes
2. ✅ Task 2.1-2.5: Implemented `PRDProcessingState` enum with 9 states
3. ✅ Task 2: Created `process_prd_step()` state machine that yields to UI
4. ✅ Task 2: Added visual progress tracker with checkmarks (✓), hourglass (⏳), boxes (☐)
5. ✅ Bonus: Enhanced JSON extraction to handle markdown code blocks and wrapped responses

**What Works Now**:
- User presses Enter → Immediate visual feedback
- Progress bar shows each step in real-time
- Steps complete one-by-one with UI updates between each
- Error messages show with detailed diagnostics
- Success shows task count

**Known Limitation**:
- "Generating tasks" step still blocks UI during LLM call (15-60s)
- However, user sees "⏳ Generating tasks" instead of frozen "Initializing..."

**Next Steps** (if desired):
1. Task 3: Streaming LLM responses to show tasks as they generate
2. Task 4: Cancellation support (press 'c' to abort)
3. Task 5: Progress percentage and ETA
4. Task 6: Post-processing summary with complexity breakdown

## Testing Plan

### Manual Tests
1. **Happy Path**: Select PRD → See progress → Tasks generated → View in Kanban
2. **Cancellation**: Start processing → Press 'c' → Confirm → Returns to browser
3. **Error Recovery**: Ollama not running → See diagnostics → Start Ollama → Retry → Success
4. **Large PRD**: 50+ objective PRD → Progress updates smoothly → All tasks saved
5. **Quick PRD**: 3 objective PRD → Steps complete rapidly → No UI flicker

### Edge Cases
1. Empty markdown file
2. Malformed markdown (not a valid PRD structure)
3. Network timeout during LLM call
4. Ollama crashes mid-generation
5. Database write failure
6. Extremely long task titles (truncation)

### Performance Tests
1. PRD with 100 objectives (stress test)
2. Multiple PRDs processed in sequence
3. Memory usage during long LLM generation

## Success Metrics

- **User Confidence**: Always know what's happening (no mysterious delays)
- **Cancellability**: Can abort at any point without data corruption
- **Transparency**: See tasks as they're created, not just at the end
- **Error Recovery**: Clear guidance when things fail
- **Performance**: No UI freezes during LLM calls

## Dependencies

- Ollama API (streaming support)
- SQLx (transaction support for atomic saves)
- Ratatui (progress bar rendering)
- Tokio (async cancellation)

## Risks

1. **Streaming Complexity**: Parsing partial JSON from LLM streams
2. **Cancellation Safety**: Ensuring no orphaned database records
3. **UI Responsiveness**: Keeping UI smooth during heavy LLM load
4. **Error States**: Handling all failure modes gracefully

## Notes

- Consider adding telemetry (processing time, success rate, common errors)
- Future: Multi-PRD batch processing
- Future: Resume interrupted processing (save checkpoint)
- Future: Show LLM reasoning/thinking for transparency

---

**Next Steps**:
1. Investigate and fix immediate crash bug
2. Design detailed state machine for PRD processing steps
3. Implement progressive status display
4. Add cancellation support
5. Polish UI with progress bars and summaries
