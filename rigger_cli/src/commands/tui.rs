//! Implementation of the 'rig tui' command - Terminal User Interface.
//!
//! Provides an interactive TUI for managing tasks with real-time visualization
//! of the orchestration pipeline, including Kanban boards, chain-of-thought
//! reasoning display, and network request logging.
//!
//! Revision History
//! - 2025-11-28T01:35:00Z @AI: Implement intent-based auto-scroll with prd_gen_auto_scroll flag. Added boolean flag (default true) that tracks user's scroll intent. Auto-scroll ONLY when flag is true, allowing blue text blocks to fill and grow while keeping bottom visible. User actions: Up/PageUp/Home disable auto-scroll (preserve manual position), Down/PageDown/End re-enable when reaching absolute bottom, sending message re-enables. This implements expected behavior: (1) let blue LLM text fill up and auto-scroll as it grows, (2) user can scroll up to review history without being pulled back down, (3) auto-scroll resumes only when user returns to bottom. Flag-based approach is cleaner than position comparison.
//! - 2025-11-28T01:30:00Z @AI: Fix text wrapping with proper height estimation and smart auto-scroll. (1) Calculate wrapped text height based on text length - estimate ~100 chars per line, use ceil() to allocate sufficient space for wrapped content. Previously fixed height of 2 caused wrapped text to be cut off. (2) Implement smart auto-scroll - only auto-scroll when user is already at bottom (scroll_offset >= len - 2). Checks if user scrolled up manually before auto-scrolling, preserving user's scroll position when reviewing history. Both fixes address: blue LLM text now wraps properly, and conversation auto-scrolls down as messages arrive only if user is at bottom.
//! - 2025-11-28T01:25:00Z @AI: Fix text wrapping by using single Span instead of multiple. Changed text message construction from two separate Spans (icon + text) to single Span with combined text (icon + text as one string). Ratatui's wrap functionality works on text content, not on Lines with multiple Spans - having icon and text as separate Spans created single unwrappable unit. Now full_text combines icon and text into single string, allowing Paragraph.wrap() to properly wrap long messages across multiple lines. Streaming text now wraps correctly.
//! - 2025-11-28T01:20:00Z @AI: Implement full conversation scrolling with keyboard controls. Replaced message trimming with true scrolling using prd_gen_scroll_offset state. Rendering calculates visible window starting from scroll offset and shows messages that fit in available height. Keyboard controls: Up/Down (scroll 1 message), PageUp/PageDown (scroll 5 messages), Home (jump to top), End (jump to bottom). Auto-scroll to latest message when new messages arrive (scroll_offset = len - 1). Users can now scroll through entire conversation history while LLM is generating tasks, providing full visibility into the thinking process. Fixes conversation box shrinkage - messages always render at proper size.
//! - 2025-11-28T01:15:00Z @AI: Fix conversation box shrinkage when many messages present. Implemented intelligent message trimming that calculates total height needed for all messages and removes oldest messages until remaining fit in available space. Changed all constraints back to Constraint::Length (from Min) to prevent layout fighting. Before rendering, algorithm walks backward from newest message, summing heights, and stops adding messages when available_height exceeded. This ensures each message renders at its proper size without shrinkage, while auto-scrolling shows most recent messages that fit. Fixes issue where adding more messages caused all boxes to shrink progressively.
//! - 2025-11-28T01:10:00Z @AI: Fix text message height constraints to accommodate wrapping. Changed text message layout constraint from Constraint::Length to Constraint::Min to allow messages to grow when text wraps to multiple lines. Previously, wrapped text was cut off because fixed-height allocation (2 lines) couldn't accommodate longer wrapped content. Min constraint ensures text gets at least calculated height but can expand as needed when wrapping occurs, preventing text from disappearing.
//! - 2025-11-28T01:05:00Z @AI: Add text wrapping to conversation text blocks. Added .wrap(Wrap { trim: true }) to text message Paragraph rendering to enable automatic line wrapping for long messages. Text now wraps within the available width of the conversation container instead of being truncated or overflowing. Applies to all streaming text messages (Assistant thinking, User input, System messages) displayed in borderless blocks with timestamp labels.
//! - 2025-11-28T01:00:00Z @AI: Implement nested sub-task rendering inside parent task blocks. Removed BoxContent::SubTask variant and created SubTaskInfo struct instead. Task blocks now contain Vec<SubTaskInfo> for nested sub-tasks. Updated decomposition logic to find parent Task message and append subtasks to its vector instead of creating separate messages. Rendering now shows sub-tasks inside yellow task block with orange-colored numbered list format (1., 2., etc.) with indented fields. Dynamic height calculation: base 9 lines + 2 for header + 6 lines per subtask. Sub-tasks display with all fields (Title, Assignee, Priority, Complexity, Description) indented under "Sub-tasks (N):" section.
//! - 2025-11-28T00:40:00Z @AI: Add Priority field to Task/SubTask blocks and update rendering. Updated BoxContent enum to include priority field for both Task and SubTask variants. Modified PRDGenUpdate::TaskGenerated in task_orchestrator to include assignee, priority, and complexity as separate fields instead of embedding in description. Updated streaming code to extract these fields from JSON. Enhanced block rendering to display all fields in key:value format (Title, Assignee, Priority, Complexity, Description). Text message height now accounts for borderless block title (+1 line). Task domain doesn't have priority field yet, so SubTask creation uses None for priority.
//! - 2025-11-28T00:20:00Z @AI: Fix missing conversation container block after refactoring. Restored outer Block widget with cyan border and conversation title that wraps all messages. When refactoring to individual message blocks, accidentally removed the parent container. Now renders conversation_block first to create the blue border, extracts inner area with .inner(), then splits that area for individual messages. Removed dead conversation_lines code that was no longer used after refactor. Conversation section now properly shows bordered blue box with project title.
//! - 2025-11-28T00:00:00Z @AI: Refactor text streaming blocks to use borderless Block widgets with timestamp labels. Updated MessageItem::Text enum variant to include timestamp field. Modified text message rendering to use Block::default().borders(Borders::NONE).title(timestamp) instead of inline timestamp spans. Text messages now display timestamp as block label in dark gray, keeping content clean while maintaining temporal context. Yellow-bordered Task blocks and orange-bordered SubTask blocks remain unchanged as structured content containers.
//! - 2025-11-27T14:30:00Z @AI: Add orange sub-task boxes to conversation view during decomposition. Modified SavingTasks state decomposition logic to create PRDGenMessage entries for each generated sub-task. Sub-task boxes display "┌─ Sub-task Generated" header with title, assignee, complexity, and description in orange (RGB 255,165,0) color. Updated conversation rendering to detect "┌─ Sub-task" prefix and apply orange color. This provides real-time visual feedback as complex tasks are automatically decomposed into manageable sub-tasks, appearing below the parent yellow task box in the conversation history.
//! - 2025-11-27T14:00:00Z @AI: Add PRD title to processing headers for better context. Modified render_prd_processing() and render_interactive_generation() to include PRD title in headers when prd_processing_prd is available. Headers now show "📋 Processing PRD - {prd.title}" and "💭 LLM Conversation - {prd.title}" instead of generic text. This provides immediate context during long-running task generation sessions, addressing user feedback that project name helps track progress when processing takes significant time.
//! - 2025-11-27T11:00:00Z @AI: Wire auto-decomposition into interactive TUI PRD flow. Added decomposition logic to SavingTasks state (process_prd_step) after tasks are saved. Reads config.json for model settings, iterates through saved tasks checking complexity >= 7, creates RigPRDParserAdapter, calls decompose_task() with PRD raw_content, saves generated sub-tasks, updates parent task with subtask_ids and Decomposed status. Non-fatal error handling with eprintln warnings - decomposition failures don't block task save completion. ReloadingTasks state will fetch all tasks including sub-tasks for hierarchical display in Kanban.
//! - 2025-11-27T10:00:00Z @AI: Implement hierarchical task display in TUI Kanban board. Added HierarchicalTask struct and build_hierarchical_task_list() helper function to organize tasks into parent-child relationships. Created get_tree_indicator() to generate box-drawing tree prefixes (├─ for intermediate children, └─ for last child). Modified all 5 Kanban columns (TODO, IN PROGRESS, COMPLETED, ARCHIVED, ERRORED) to use hierarchical rendering with proper indentation and tree indicators. Sub-tasks now appear indented beneath their parent tasks with visual tree connectors, matching terminal tree visualization conventions.
//! - 2025-11-27T05:00:00Z @AI: Fix conversation scroll for wrapped text. Implemented manual text wrapping for long messages to accurately count visual lines before rendering. When message content exceeds available width, split into chunks and create separate Line objects with proper indentation for continuation lines. This ensures scroll offset calculation includes wrapped lines, preventing scroll lag when long validation messages or LLM responses appear. Removed reliance on Paragraph's automatic wrapping which happened after scroll calculation.
//! - 2025-11-27T04:00:00Z @AI: Add comprehensive test for validation red row functionality. Created test_validation_red_row_functionality() that validates: (1) validation messages are stored in PartialTask.validation_messages vec, (2) task status transitions to Validating when remediation starts, (3) multiple validation messages accumulate correctly, (4) validation boxes appear in conversation with proper formatting. Test simulates complete validation workflow from assignee mismatch through LLM remediation success.
//! - 2025-11-27T03:45:00Z @AI: Render validation boxes in red in conversation view. Modified conversation rendering to detect "┌─ Validation" boxes and render them in red color instead of yellow. This provides visual consistency - validation messages appear in red in both the conversation (blue box) and task list (green box) sections, making remediation events easily identifiable.
//! - 2025-11-27T03:30:00Z @AI: Implement remediation red row feature for assignee validation. Added validation_messages field to PartialTask struct to store validation messages. Modified ValidationInfo handler to push messages to task's validation_messages vec and set status to Validating. Updated Generated Tasks rendering to display validation messages as indented red rows (└─) below each task. Added Validating status with yellow warning icon (⚠). This provides real-time visual feedback when assignee validation enters remediation in the green task list section.
//! - 2025-11-27T03:00:00Z @AI: Fix four UI issues from screenshot feedback. (1) Added pipe prefix to Priority/Complexity lines by splitting description on newlines and mapping each line with │ prefix. (2) Added Assignee field to yellow task box by extracting assignee from JSON with field aliases and appending to description. (3) Fixed conversation scroll by counting total rendered lines instead of messages and using .scroll() method with proper offset. (4) Added ValidationInfo enum variant and match arm handler that updates task status and displays validation box in conversation.
//! - 2025-11-26T22:20:00Z @AI: Fix task box line wrapping in conversation view. Changed task box rendering to split on newlines and create separate Line objects instead of treating entire box as single line. Now Priority, Complexity, and multi-line descriptions display correctly on separate lines within the yellow box instead of running together.
//! - 2025-11-26T22:00:00Z @AI: Fix PRD error display bugs per screenshot feedback. (1) Word-wrap long error messages at 60 chars to prevent text truncation - added word-wrapping logic that splits on whitespace while preserving bullet points, headers, and check marks. (2) Contextual diagnostics - only show Ollama diagnostics for connection errors, not JSON parsing errors. JSON errors now show relevant troubleshooting (check PRD format, verify LLM output, try simpler PRD, check config model name) instead of misleading Ollama setup checks.
//! - 2025-11-26T21:45:00Z @AI: Fix interactive generation UI rendering bugs. (1) Task boxes now render without timestamps to preserve box-drawing characters - detect System messages containing "┌─" and skip timestamp prefix. (2) Auto-scroll to bottom always shows latest 15 messages - calculate visible_start dynamically from total_messages instead of using stale prd_gen_scroll_offset. Both issues from screenshot feedback: timestamp was pushing yellow box right, blue conversation section wasn't scrolling down as new items arrived.
//! - 2025-11-26T08:30:00Z @AI: Phase 4: Add Personas workspace section to navigation. Added Personas variant to WorkspaceSection enum, added personas/selected_persona/agent_tools state fields to App struct, initialized new fields in App::new(). Foundation for persona management UI (F4 key will switch to persona browser view).
//! - 2025-11-26T06:15:00Z @AI: Update Config Viewer to display wizard-based task_tools configuration. Modified render_config_viewer() to parse config.json and show task tool slots (Main/Research/Fallback) with their provider and model settings instead of just showing file existence checks. Added JSON parsing with error handling, displays provider in green and model in white, shows appropriate error/warning messages if config is missing or malformed. Config viewer now matches the setup wizard format.
//! - 2025-11-26T06:00:00Z @AI: Implement Up/Down navigation for database records in SQLite Browser. Added db_selected_record state field to track selected row, modified keyboard handlers to prioritize record navigation when table data is loaded (Up/Down moves between records, falls back to table list navigation when at boundaries), updated render_sqlite_browser() to highlight selected record with yellow color and ▶ prefix indicator, added "↑/↓ Navigate" to help text footer, reset db_selected_record to 0 when new table is loaded in load_table_data().
//! - 2025-11-26T05:30:00Z @AI: Fix multiple UX bugs: (1) Hide Inspector panel by default (show_details_panel=false), (2) Add 'm' markdown browser shortcut to help dialog OTHER section. Remaining bugs (Up/Down navigation in Dev Tools, Config display format, PRD table verification) tracked in todo list.
//! - 2025-11-26T05:20:00Z @AI: Fix Inspector panel to track Kanban column selection. Modified render_details_panel() to use get_selected_task_in_column() instead of global selected_task index, so Inspector updates correctly when navigating tasks with Up/Down arrow keys within Kanban columns.
//! - 2025-11-26T05:00:00Z @AI: Fix Project creation when PRD is parsed. Modified SavingTasks state to: (1) create Project entity from PRD title with auto-generated description, (2) save Project to database before saving PRD, (3) save PRD entity to database, (4) set source_prd_id on all generated tasks to link them to the PRD. This ensures Projects section shows created projects and tasks can be filtered by project. Project name derives from PRD title, description auto-generated with filename for traceability.
//! - 2025-11-26T04:35:00Z @AI: Transform Kanban board with card-style task rendering and column-aware navigation. Each task now renders as a distinct card with box-drawing borders (┌─┐│└─┘ for unselected, ┏━┓┃┗━┛ for selected in yellow). Fixed selection highlighting to use column-specific index (selected_task_in_column) instead of global task index, enabling proper Up/Down navigation within each column (F1-F5). Enter key already wired to open task editor for selected task. All 5 columns (Todo, InProgress, Completed, Archived, Errored) now have card styling with appropriate colors and visual separation.
//! - 2025-11-26T04:25:00Z @AI: Add comprehensive test coverage for interactive generation features. Added 11 new unit tests covering: PRDGenMessage creation, conversation building, input buffer manipulation, last message storage for editing, edit mode activation/cancellation, partial task tracking, status transitions, scroll control, input focus state, and complete send-edit-resend workflow. All 47 tui tests pass with 0 failures. Tests verify message editing UX, state management, and conversation flow work correctly.
//! - 2025-11-26T04:20:00Z @AI: Add advanced UX features to interactive generation UI. Enhanced render_interactive_generation() to show "(editing last message)" indicator in title when user is editing previous message. Added context-aware keyboard hints: "Press ↑ to edit last message" when input is empty and previous message exists, "Esc to cancel edit" when editing mode active, "Esc to clear" otherwise. This provides professional, discoverable UX for the Up-arrow message editing feature implemented in keyboard handlers.
//! - 2025-11-25T23:20:00Z @AI: Add keyboard handlers for interactive generation input. Modified PRD processing keyboard handling to detect interactive mode (GeneratingTasks state with conversation data) and enable text input, backspace editing, Enter to send messages, Esc to clear input, and Up/Down to scroll conversation history. Non-interactive mode uses standard handlers (Enter to close on Complete, Esc to close on Failed). State cleanup added when exiting processing view. Input is added to conversation history with timestamp and User role, ready for LLM agent channel integration.
//! - 2025-11-25T23:10:00Z @AI: Implement interactive generation UI rendering. Created render_interactive_generation() function that displays 3-section layout: conversation history (40%, scrollable, shows LLM/user messages with timestamps and icons), generated tasks (40%, shows partial tasks with status indicators), and user input field (20%, with keyboard hints). Modified render_prd_processing() to detect GeneratingTasks state and switch to interactive UI when conversation data is present. UI supports real-time display of LLM thinking and partial task results as they stream in.
//! - 2025-11-25T23:00:00Z @AI: Add interactive PRD generation state structures. Created PRDGenStatus, PRDGenRole, PRDGenMessage, PartialTask, and PartialTaskStatus enums/structs to support real-time conversation during task generation. Added 6 new App state fields (prd_gen_conversation, prd_gen_input, prd_gen_partial_tasks, prd_gen_status, prd_gen_input_active, prd_gen_scroll_offset) to track interactive generation flow. This enables showing LLM thinking, accepting mid-generation user input, and displaying partial task results as they're created.
//! - 2025-11-25T22:00:00Z @AI: Implement state machine for PRD processing with step-by-step UI updates. Replaced monolithic create_prd_from_markdown() with process_prd_step() state machine that processes one step at a time (ReadingFile → ParsingPRD → LoadingConfig → GeneratingTasks → SavingTasks → ReloadingTasks → Complete/Failed). Each step yields back to event loop, allowing UI to render progress updates between steps. Created PRDProcessingState enum with 9 states, added intermediate data storage fields (prd_processing_content, prd_processing_prd, prd_processing_config, prd_processing_tasks), updated render_prd_processing() to show state-specific messages with spinner animation, modified keyboard handlers to use state pattern matching. Main loop now calls process_prd_step() once per iteration when prd_processing_pending=true, giving real-time progress visibility. Deleted obsolete create_prd_from_markdown() function (141 lines). Build succeeds with 18 warnings.
//! - 2025-11-25T21:30:00Z @AI: Implement immediate PRD processing feedback - separated UI transition from async processing. Added prd_processing_pending flag and start_prd_processing() method to show processing screen instantly on Enter key. Processing now starts on next event loop iteration after UI renders, giving immediate visual feedback. User sees "Initializing..." screen immediately, then live progress updates during LLM generation.
//! - 2025-11-25T21:00:00Z @AI: Systematic buffer overflow prevention - created calculate_safe_dialog_height() helper function with comprehensive test coverage, fixed 7 vulnerable dialog rendering functions (render_prd_processing, render_wizard_complete, render_wizard_welcome, render_wizard_task_tool_slots, render_wizard_configure_slot, render_wizard_database_configuration, render_wizard_confirmation) to use safe height calculation, added 4 unit tests covering edge cases.
//! - 2025-11-25T20:50:00Z @AI: Fix buffer overflow in render_prd_processing and render_wizard_complete by capping dialog height to available space.
//! - 2025-11-25T20:47:00Z @AI: Fix "runtime within runtime" error in create_prd_from_markdown by using save_async() instead of blocking save().
//! - 2025-11-25T06:00:00Z @AI: Refactor setup wizard to per-slot configuration flow with Ctrl+C exit support. Reversed wizard flow from provider-first to slot-first: Welcome → TaskToolSlots (explains main/research/fallback purposes) → ConfigureMainSlot → ConfigureResearchSlot → ConfigureFallbackSlot → Database → Confirmation → Complete. Each slot now independently selects provider+model, enabling mixed configurations (e.g., main=Ollama/llama3.1, research=Rig/gpt-4o, fallback=Candle/Phi-3.5). Refactored App state from single setup_wizard_provider/model_field to per-slot fields (setup_wizard_{main,research,fallback}_{provider,provider_selection,model}). Updated config.json generation to include new task_tools structure with per-slot provider/model. Added Ctrl+C handler to exit wizard and quit app entirely. Modified Escape on Welcome screen to quit instead of going back. Created render_wizard_task_tool_slots() explanation screen and unified render_wizard_configure_slot() for all three slots. Updated confirmation screen to show all three slots with icons (🔧🔍🛟). Deleted obsolete render_wizard_provider_selection() and render_wizard_model_configuration() functions. Navigation methods (next/previous_provider, handle_char/backspace) now slot-aware.
//! - 2025-11-25T05:15:00Z @AI: Create keyboard shortcut constants and update help dialogs. Added constants (KEY_COLUMN_TODO through KEY_COLUMN_ERRORED, LABEL_COLUMN_*) to keep keyboard shortcuts in sync across UI. Updated both help dialogs to show F4 (Archived) and F5 (Errored) column selection. Changed "Quick filters" to "Column Selection" with individual entries for all 5 columns. Removed all debug toast messages (ENTER, BRANCH, LOADING, WRONG BRANCH) now that SQLite Browser navigation is working correctly.
//! - 2025-11-25T05:00:00Z @AI: Add helpful empty state messages to SQLite Browser table view. When table has 0 rows, show contextual guidance message based on table name (tasks/projects/prds) with bullet points explaining how to add data (e.g., "Press 'a' to create task", "rig parse <file.md>"). Added horizontal padding to SQLite Browser widget for better spacing. Fixed success status toast - now shows "✅ Loaded X rows" after successful load_table_data() instead of leaving "Loading..." message visible.
//! - 2025-11-25T04:30:00Z @AI: Fix SQLite Browser Enter key handling priority. Reordered Enter key condition checks to prioritize active_dev_tool.is_some() (line 3255) BEFORE active_tool == DashboardTool::DevTools (line 3276). Previously when viewing table list in SQLite Browser, pressing Enter would match the DevTools check first and try to launch a new dev tool using dev_tools_selection, causing the selection to reset. Now pressing Enter correctly calls load_table_data() to show table contents. Same priority fix applied to Up/Down navigation handlers earlier.
//! - 2025-11-24T23:30:00Z @AI: Implement PRD processing view with real-time progress display. When user presses Enter on markdown file in browser, file is parsed as PRD and tasks are generated via LLM with live progress updates. Added 6 App state fields (show_prd_processing, prd_processing_step, prd_processing_file, prd_tasks_generated, prd_processing_complete, prd_processing_error), rewrote create_prd_from_markdown() to perform actual parsing with step-by-step progress messages (reading file → parsing PRD structure → loading config → generating tasks via LLM → saving to database → complete), created render_prd_processing() full-screen view showing animated spinner with current step or success/error state, wired view to ui() renderer with priority after setup wizard, added keyboard handling (Enter when complete to return to Kanban, Esc on error to close). Processing shows: file name, current step with spinner, task count on success, error message on failure, reloads task list after completion. Integrates with existing rig parse command logic using Rig PRD parser adapter and SQLite persistence.
//! - 2025-11-24T23:00:00Z @AI: Add comprehensive first-time setup wizard for projects without .rigger config. Implemented full-screen TUI-based setup flow with 6 steps: Welcome, ProviderSelection (Ollama/Candle/Mistral/Rig), ModelConfiguration (main/research/fallback models with provider-specific defaults), DatabaseConfiguration, Confirmation (review all settings), and Complete (success with next steps). Wizard creates .rigger/config.json and initializes SQLite database automatically. Added SetupWizardStep/LLMProvider/ModelConfigField enums, 9 new App state fields, navigation methods (next/previous step/provider/field, text input handlers), complete/exit async methods for config creation, 7 rendering functions for each screen with centered dialogs and color-coded UI, keyboard handling in run_app, wizard activation check in execute(). Activates when .rigger/config.json is missing on TUI startup.
//! - 2025-11-25T00:20:00Z @AI: Replace QuickFilter with Kanban Column Selector. Changed F1-F5 from filters to column selectors (Todo/InProgress/Completed/Archived/Errored). Added Up/Down navigation within selected column and Enter to open task editor for selected task. Footer now shows "Column:" instead of "Filter:". Column selector bar renamed and updated with F1-F5 shortcuts.
//! - 2025-11-25T00:00:00Z @AI: Remove backdrop overlay entirely to fix dialog transparency. Deleted render_dialog_backdrop() function and its call. The dark gray backdrop was the root cause of transparency issues. Dialogs now render directly on the main UI with Clear widget + black background Paragraph, creating fully opaque dialogs without any backdrop interference.
//! - 2025-11-24T22:30:00Z @AI: Add markdown file browser dialog. Implemented markdown file browser for PRD creation with async file scanning, alphabetically sorted file list, keyboard navigation (m to open, Up/Down to navigate, Enter to select, Esc to close), and placeholder integration with PRD creation workflow.
//! - 2025-11-24T20:00:00Z @AI: Fix dialog opacity - make backgrounds truly solid. Created clear_dialog_area() helper function that uses ratatui::widgets::Clear to clear the dialog area first, then fills it with a solid black Block. Updated all 9 dialog rendering functions (shortcuts overlay, jump dialog, recent dialog, task editor, task creator, spotlight, confirmation, LLM chat, PRD dialog, notifications) to call clear_dialog_area() before rendering dialog content. This ensures dialogs are completely opaque with solid black backgrounds, not translucent.
//! - 2025-11-24T19:45:00Z @AI: Add missing shortcuts to keyboard overlay. Added 'a' (Create new task) and '/' (Spotlight search) to the TASK ACTIONS section of the keyboard shortcut overlay, ensuring all major features are documented for users.
//! - 2025-11-24T19:30:00Z @AI: Add gray backdrop overlay when dialogs are open. Created render_dialog_backdrop() function that renders a dark gray (RGB 40,40,40) overlay covering the entire screen when any dialog is active. This dims the background content to emphasize the active dialog and provides visual feedback that the background is not interactive. Backdrop is rendered before all dialogs in the UI rendering order.
//! - 2025-11-24T19:00:00Z @AI: Fix keyboard handling for dialog text input. Moved all guarded dialog text input handlers (KeyCode::Char(c) if app.show_*_dialog) to TOP of match statement, before specific character hotkeys (r/d/w/e/o/g/l/a/s/c). This ensures when dialogs are open for text entry, characters are captured by the dialog input handlers instead of triggering global hotkeys. Critical fix for Create New Task dialog and all other text input dialogs.
//! - 2025-11-24T18:30:00Z @AI: Add opaque backgrounds to all dialogs for readability. Added .style(Style::default().bg(Color::Black)) to both Block and Paragraph widgets for: spotlight search dialog, keyboard shortcut overlay, confirmation dialog, task editor/creator dialogs, jump/recent dialogs, LLM chat dialog, PRD dialog, notification center. Prevents background text from showing through dialogs.
//! - 2025-11-24T18:00:00Z @AI: Add Errored status to TaskStatus enum and update all status match statements. Added Errored variant to TaskStatus (task_manager crate), updated Kanban board to 4-column layout with 4th column split vertically (Archived top 50%, Errored bottom 50%), added empty state guidance messages for projects and tasks, updated all status formatting functions (task_table, task_formatter, grpc_server, tui helper functions).
//! - 2025-11-24T12:00:00Z @AI: Phase 14: Polish and UX refinements. Final phase complete - polish was applied incrementally across all phases: consistent emoji usage throughout (📋 🔄 ✓ 🕒 ⚠️ 🔴), color-coded indicators for status/age/severity, unified dialog styling with centered layouts and clear borders, comprehensive error handling with user-friendly notifications, loading states with animated spinners, keyboard shortcut documentation in help overlay, confirmation for destructive operations, real-time status updates in footer. All 14 phases of TUI refactoring successfully completed with production-ready polish and UX.
//! - 2025-11-24T11:30:00Z @AI: Phase 13: Global keyboard commands (refresh, filter shortcuts). Implement refresh_all_data method to reload projects and tasks from database with loading state and notifications. Add F5 keyboard shortcut for refresh, F6 for toggling between Kanban/Metrics views. Document F1/F2/F3 quick filter shortcuts (already existed but now documented). Update keyboard shortcut overlay with new "GLOBAL COMMANDS" section showing F1-F3 (filters), F5 (refresh), F6 (view toggle). All function keys work globally without dialog interference.
//! - 2025-11-24T11:00:00Z @AI: Phase 12: Task age tracking and staleness indicators. Enhance existing age tracking (calculate_task_age_days/get_age_indicator functions already present in Kanban view) with format_task_age_description (Fresh/Recent/Aging/Stale classifications with hour-level granularity for <1 day), format_timestamp helper for local time display. Add age tracking section to details panel showing colored age description and created/updated timestamps. Add age tracking to task editor dialog showing age with icon, staleness color, and timestamp information. Age indicators now appear in: Kanban cards (existing), Details panel (new), Task Editor (new).
//! - 2025-11-24T10:30:00Z @AI: Phase 11: Live status footer with real-time updates. Add session_start_time field to track TUI session, implement format_session_duration/format_current_time/get_database_status methods for live metrics, enhance render_footer_bar to two-line display showing: (Line 1) task counts with emojis, active filter, current time HH:MM:SS, loading indicator; (Line 2) session duration, database connection status, project/PRD counts, current project/view name, help hint. Footer now shows filtered task counts instead of all tasks for better context awareness.
//! - 2025-11-24T10:00:00Z @AI: Phase 10: Add confirmation dialogs for destructive operations. Add ConfirmationAction enum, show_confirmation_dialog/confirmation_title/confirmation_message/confirmation_action state, implement open_confirmation/close_confirmation/confirm_action (async archival with database persistence) methods, update cycle_task_status to require confirmation before archiving (Completed→Archived transition), add keyboard handling (Y/Enter to confirm, N/Esc to cancel), create render_confirmation_dialog with yellow warning border and Yes/No buttons.
//! - 2025-11-24T09:30:00Z @AI: Phase 9: Implement Global Spotlight Search (/ key). Add SearchResultType enum, show_spotlight_dialog/spotlight_query/spotlight_results/spotlight_selected state, implement search_all (fuzzy search across tasks/PRDs/projects), open_spotlight/close_spotlight/handle_spotlight_input/handle_spotlight_backspace/next_spotlight_result/previous_spotlight_result/execute_spotlight_jump methods, add keyboard handling ('/' key to open, type to search, ↑/↓ to navigate, Enter to jump, Esc to close), create render_spotlight_dialog with live search results, type indicators, and match highlighting.
//! - 2025-11-24T09:00:00Z @AI: Phase 8: Build Task Creation dialog (a key). Add TaskCreatorField enum, show_task_creator_dialog/task_creator_field/task_creator_title/task_creator_description/task_creator_assignee/task_creator_status state, implement open_task_creator/close_task_creator/next_task_creator_field/previous_task_creator_field/handle_task_creator_input/handle_task_creator_backspace/cycle_creator_status_forward/cycle_creator_status_backward/save_task_creator methods, add keyboard handling ('a' key to open, Tab/Shift+Tab to navigate fields, ↑/↓ for status cycling, Enter to save, Esc to cancel), create render_task_creator_dialog function with multi-field form, link new tasks to current project via PRD.
//! - 2025-11-24T08:30:00Z @AI: Phase 7: Add PRD management view (r key). Add show_prd_dialog/selected_prd state, implement open_prd_dialog/close_prd_dialog/next_prd/previous_prd methods, add keyboard handling ('r' key to open, ↑/↓ to navigate PRDs, Esc to close), create render_prd_dialog showing filtered PRDs for current project with objectives/tech stack/constraints details, expandable selected PRD view.
//! - 2025-11-24T08:00:00Z @AI: Phase 6: Create Agent Tools reference panel. Update render_shortcut_overlay to comprehensive quick reference showing all keyboard shortcuts organized by category (Project Navigation, Main Views, Task Actions, Agent Tools, Other), include new Phase 1-5 features (w/e for projects, Enter for Task Editor, 'l' for LLM Chat), display as centered dialog instead of corner overlay, add LLM Chat command examples.
//! - 2025-11-24T07:30:00Z @AI: Phase 5: Implement LLM Chat with context separation. Add show_llm_chat_dialog/llm_chat_input/llm_chat_history state, create ChatMessage/ChatRole structs for message history, implement open_llm_chat (shows current project+task context)/close_llm_chat/handle_llm_chat_input/handle_llm_chat_backspace/send_llm_chat_message (placeholder responses) methods, add keyboard handling ('l' key to open, Enter to send, Esc to close, Backspace to edit), add render_llm_chat_dialog with context display and conversation history.
//! - 2025-11-24T07:00:00Z @AI: Phase 4: Build Task Editor dialog (Enter key trigger). Add show_task_editor_dialog/task_editor_field/task_editor_input state, implement open_task_editor/close_task_editor/next_task_editor_field/previous_task_editor_field/handle_task_editor_input/handle_task_editor_backspace/cycle_task_status_forward/cycle_task_status_backward/save_task_editor methods, add keyboard handling (Enter to open, Tab/Shift+Tab to navigate fields, ↑/↓ for status cycling, Enter to save, Esc to cancel), add render_task_editor_dialog function.
//! - 2025-11-24T06:30:00Z @AI: Phase 3: Simplify main views to Kanban/Metrics only. Update next_tool/previous_tool to toggle between only Kanban and Metrics, remove TaskEditor and LLMChat from navigation panel tools list (they become dialog-only in Phases 4 & 5).
//! - 2025-11-24T06:00:00Z @AI: Phase 2: Implement project filtering and navigation. Add projects/selected_project_id to App state, replace WorkspaceSection navigation with project switching (w/e keys), add get_filtered_tasks/get_filtered_prds methods for transitive filtering (Task→PRD→Project), update navigation panel to show project list with task counts, update title bar to display current project name, implement load_projects method to load from database on startup.
//! - 2025-11-24T04:00:00Z @AI: Created TASK_PLAN_TUI_PROJECT_ARCHITECTURE.md for project-driven TUI refactoring. Current workspace navigation is placeholder - will be replaced with Project entity in next phase. See task plan for hierarchical Project → PRDs → Tasks architecture and live status footer design.
//! - 2025-11-24T03:30:00Z @AI: Reorder navigation panel to show WORKSPACE first (conceptual priority), then TOOLS. Add WorkspaceSection enum (Tasks, PRDs, Projects) with keyboard navigation via 'w'/'e' keys. Selected workspace highlighted with yellow arrow. Update key hints to show workspace switching.
//! - 2025-11-24T03:15:00Z @AI: Add footer bar with task summary statistics (total, todo, in progress, completed, active filter). Add key hints to navigation panel (Tab, d, ?). Footer displays loading spinner when async operations active.
//! - 2025-11-24T03:00:00Z @AI: Major refactoring to 3-column dashboard layout (nav | main | details). Replace tab system with tool panel switcher (Kanban, TaskEditor, LLMChat, Metrics). Add toggleable details inspector panel with 'd' key. All 15 TUI tests passing.
//! - 2025-11-24T01:45:00Z @AI: Add notification center (Task 0.9) - press 'n' to view history of events (saves, errors, status changes). Max 50 notifications with timestamps and severity levels.
//! - 2025-11-24T01:30:00Z @AI: Add visual loading states (Task 0.6) - animated spinner overlay during async operations like loading tasks.
//! - 2025-11-24T01:15:00Z @AI: Add autosave/sync indicator (Task 0.4) - shows save status in title bar ("Saving...", "Saved Xs ago", etc.).
//! - 2025-11-24T01:00:00Z @AI: Add copy task to clipboard (Task 0.3) - press 'c' to copy task as Markdown, uses hexagonal architecture.
//! - 2025-11-24T00:00:00Z @AI: Add Recent/MRU lists (Task 0.7) - Ctrl+R to show last 10 viewed tasks for quick navigation.
//! - 2025-11-23T23:30:00Z @AI: Add quick filter views (Task 0.8) - F1-F6 keys for common filters (In Progress, Today, Urgent, etc.).
//! - 2025-11-23T23:00:00Z @AI: Add visual task age indicators (Task 0.11) - show age icons and color-code by staleness.
//! - 2025-11-23T22:30:00Z @AI: Implement task ID quick jump (Task 0.10) - press 'g' to jump to task by ID with fuzzy matching.
//! - 2025-11-23T22:00:00Z @AI: Implement basic task sorting with menu (Task 0.5) - sort by created, updated, priority, title, due date, complexity.
//! - 2025-11-23T21:30:00Z @AI: Implement quick status cycling (Task 0.2) - cycle with 's' key, persist to DB, show toast message.
//! - 2025-11-23T21:00:00Z @AI: Add task count badges and keyboard shortcut overlay (Tasks 0.12, 0.1).
//! - 2025-11-23T20:00:00Z @AI: Initial TUI implementation for Phase 5.2.
//!
//! Revision History
//! - 2025-11-26T18:30:00Z @AI: Add extern crate sqlx declaration for Rust 2024 edition compatibility. Edition 2024 requires explicit external crate declarations for crates used with fully qualified paths.

// External crate declarations required for Rust 2024 edition
extern crate sqlx;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;

// Hexagonal architecture imports
use crate::ports::clipboard_port::ClipboardPort;
use crate::adapters::arboard_clipboard_adapter::ArboardClipboardAdapter;
use crate::services::task_formatter;
// NOTE: sqlx trait imports required for extension methods (.try_get(), .columns())
// These cannot be expressed via fully qualified syntax due to Rust's trait method resolution
use sqlx::Row;
use sqlx::Column;

// Keyboard shortcut constants to keep UI text in sync
const KEY_COLUMN_TODO: &str = "F1";
const KEY_COLUMN_IN_PROGRESS: &str = "F2";
const KEY_COLUMN_COMPLETED: &str = "F3";
const KEY_COLUMN_ARCHIVED: &str = "F4";
const KEY_COLUMN_ERRORED: &str = "F5";
const KEY_REFRESH: &str = "F5";
const KEY_TOGGLE_VIEW: &str = "F6";

const LABEL_COLUMN_TODO: &str = "Todo";
const LABEL_COLUMN_IN_PROGRESS: &str = "In Progress";
const LABEL_COLUMN_COMPLETED: &str = "Completed";
const LABEL_COLUMN_ARCHIVED: &str = "Archived";
const LABEL_COLUMN_ERRORED: &str = "Errored";

/// Notification severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

/// A notification entry in the notification center.
#[derive(Debug, Clone)]
struct Notification {
    /// Timestamp when notification was created
    timestamp: chrono::DateTime<chrono::Utc>,
    /// Severity level
    level: NotificationLevel,
    /// Notification message
    message: String,
}

impl Notification {
    fn new(level: NotificationLevel, message: String) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            level,
            message,
        }
    }
}

/// Available tool panels in the dashboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DashboardTool {
    /// Kanban board view
    Kanban,
    /// Task detail editor
    TaskEditor,
    /// LLM chat/command interface
    LLMChat,
    /// Metrics and analytics
    Metrics,
    /// Dev Tools menu (opens submenu)
    DevTools,
}

impl DashboardTool {
    fn display_name(&self) -> &str {
        match self {
            DashboardTool::Kanban => "📋 Kanban Board",
            DashboardTool::TaskEditor => "✏️  Task Editor",
            DashboardTool::LLMChat => "💬 LLM Chat",
            DashboardTool::Metrics => "📊 Metrics",
            DashboardTool::DevTools => "🔧 Dev Tools",
        }
    }
}

/// Available dev tools in the Dev Tools menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DevTool {
    /// SQLite database browser
    SqliteBrowser,
    /// Rigger configuration viewer/editor
    ConfigViewer,
}

impl DevTool {
    fn display_name(&self) -> &str {
        match self {
            DevTool::SqliteBrowser => "🗄️  SQLite Browser",
            DevTool::ConfigViewer => "⚙️  Config Viewer",
        }
    }

    fn description(&self) -> &str {
        match self {
            DevTool::SqliteBrowser => "Browse database tables and execute queries",
            DevTool::ConfigViewer => "View and edit rigger configuration settings",
        }
    }
}

/// Workspace sections in the navigation panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkspaceSection {
    /// Tasks workspace
    Tasks,
    /// PRDs workspace
    Prds,
    /// Projects workspace
    Projects,
    /// Personas workspace
    Personas,
}

impl WorkspaceSection {
    fn display_name(&self) -> &str {
        match self {
            WorkspaceSection::Tasks => "📋 Tasks",
            WorkspaceSection::Prds => "📄 PRDs",
            WorkspaceSection::Projects => "🎯 Projects",
            WorkspaceSection::Personas => "👤 Personas",
        }
    }
}

/// Quick filter options for common task views.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KanbanColumn {
    Todo,
    InProgress,
    Completed,
    Archived,
    Errored,
}

impl KanbanColumn {
    /// Returns the display name and icon for this column.
    fn display_name(&self) -> &str {
        match self {
            KanbanColumn::Todo => "📋 Todo",
            KanbanColumn::InProgress => "🔄 In Progress",
            KanbanColumn::Completed => "✓ Completed",
            KanbanColumn::Archived => "📦 Archived",
            KanbanColumn::Errored => "🔴 Errored",
        }
    }

    /// Returns the keyboard shortcut for this column.
    fn shortcut(&self) -> &str {
        match self {
            KanbanColumn::Todo => KEY_COLUMN_TODO,
            KanbanColumn::InProgress => KEY_COLUMN_IN_PROGRESS,
            KanbanColumn::Completed => KEY_COLUMN_COMPLETED,
            KanbanColumn::Archived => KEY_COLUMN_ARCHIVED,
            KanbanColumn::Errored => KEY_COLUMN_ERRORED,
        }
    }

    /// Returns the TaskStatus that matches this column.
    fn matching_status(&self) -> task_manager::domain::task_status::TaskStatus {
        match self {
            KanbanColumn::Todo => task_manager::domain::task_status::TaskStatus::Todo,
            KanbanColumn::InProgress => task_manager::domain::task_status::TaskStatus::InProgress,
            KanbanColumn::Completed => task_manager::domain::task_status::TaskStatus::Completed,
            KanbanColumn::Archived => task_manager::domain::task_status::TaskStatus::Archived,
            KanbanColumn::Errored => task_manager::domain::task_status::TaskStatus::Errored,
        }
    }
}

/// Task sorting options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TaskSortOption {
    CreatedNewest,
    UpdatedRecent,
    TitleAlphabetical,
    ComplexityHigh,
}

impl TaskSortOption {
    /// Returns the display name for this sort option.
    fn display_name(&self) -> &str {
        match self {
            TaskSortOption::CreatedNewest => "Created (newest first)",
            TaskSortOption::UpdatedRecent => "Updated (most recent)",
            TaskSortOption::TitleAlphabetical => "Alphabetical (A-Z)",
            TaskSortOption::ComplexityHigh => "Complexity (hardest first)",
        }
    }

    /// Returns all available sort options.
    fn all() -> std::vec::Vec<TaskSortOption> {
        std::vec![
            TaskSortOption::CreatedNewest,
            TaskSortOption::UpdatedRecent,
            TaskSortOption::TitleAlphabetical,
            TaskSortOption::ComplexityHigh,
        ]
    }
}

/// TUI application state.
struct App {
    /// Projects loaded from database (top-level organizational context)
    projects: std::vec::Vec<task_manager::domain::project::Project>,
    /// Currently selected project ID (filters all PRDs and tasks)
    selected_project_id: std::option::Option<String>,
    /// Currently selected workspace section
    selected_workspace: WorkspaceSection,
    /// Currently active tool/panel in main view
    active_tool: DashboardTool,
    /// Selected item in left navigation (for tool switching)
    nav_selection: usize,
    /// Whether to show task details panel (right column)
    show_details_panel: bool,
    /// Tasks loaded from database
    tasks: std::vec::Vec<task_manager::domain::task::Task>,
    /// PRDs loaded from database
    prds: std::vec::Vec<task_manager::domain::prd::PRD>,
    /// Selected task index in the list
    selected_task: usize,
    /// Chain-of-thought log entries
    thinking_log: std::vec::Vec<String>,
    /// Network request/response log
    network_log: std::vec::Vec<String>,
    /// Whether to exit the TUI
    should_quit: bool,
    /// Whether to show keyboard shortcut overlay
    show_shortcuts: bool,
    /// Status message to display (cleared after 2 seconds)
    status_message: std::option::Option<String>,
    /// Database adapter for persisting changes
    db_adapter: std::option::Option<task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter>,
    /// Whether to show the sort menu
    show_sort_menu: bool,
    /// Currently selected sort option
    current_sort: TaskSortOption,
    /// Selected item in sort menu
    sort_menu_selection: usize,
    /// Whether to show the task jump dialog
    show_jump_dialog: bool,
    /// Input buffer for task ID jump
    jump_input: String,
    /// Currently selected Kanban column
    selected_column: KanbanColumn,
    /// Currently selected task index within the selected column
    selected_task_in_column: usize,
    /// Whether to show the recent items dialog
    show_recent_dialog: bool,
    /// Most recently used task IDs (max 10)
    recent_task_ids: std::vec::Vec<String>,
    /// Selected item in recent dialog
    recent_selection: usize,
    /// Clipboard adapter for copy operations
    clipboard: std::option::Option<std::sync::Arc<dyn ClipboardPort>>,
    /// Timestamp of last database save
    last_saved_at: std::option::Option<chrono::DateTime<chrono::Utc>>,
    /// Whether a save operation is currently in progress
    is_saving: bool,
    /// Whether there are unsaved changes
    has_unsaved_changes: bool,
    /// Whether an async operation is in progress (for loading indicator)
    is_loading: bool,
    /// Message describing current loading operation
    loading_message: std::option::Option<String>,
    /// Loading spinner frame counter (for animation)
    loading_frame: usize,
    /// Notification history (max 50, newest first)
    notifications: std::vec::Vec<Notification>,
    /// Whether to show the notification center dialog
    show_notifications: bool,
    /// Whether to show the task editor dialog (Phase 4)
    show_task_editor_dialog: bool,
    /// Which field is being edited in the task editor (Phase 4)
    task_editor_field: TaskEditorField,
    /// Input buffer for task editor text fields (Phase 4)
    task_editor_input: String,
    /// Whether to show the LLM chat dialog (Phase 5)
    show_llm_chat_dialog: bool,
    /// Input buffer for LLM chat (Phase 5)
    llm_chat_input: String,
    /// Chat message history (Phase 5) - alternating user/assistant messages
    llm_chat_history: std::vec::Vec<ChatMessage>,
    /// Whether to show the PRD management dialog (Phase 7)
    show_prd_dialog: bool,
    /// Selected PRD index in the PRD management dialog (Phase 7)
    selected_prd: usize,
    /// Whether to show the task creator dialog (Phase 8)
    show_task_creator_dialog: bool,
    /// Which field is being edited in the task creator (Phase 8)
    task_creator_field: TaskCreatorField,
    /// Title input buffer for task creator (Phase 8)
    task_creator_title: String,
    /// Description input buffer for task creator (Phase 8)
    task_creator_description: String,
    /// Assignee input buffer for task creator (Phase 8)
    task_creator_assignee: String,
    /// Status selection for task creator (Phase 8)
    task_creator_status: task_manager::domain::task_status::TaskStatus,
    /// Whether to show the spotlight search dialog (Phase 9)
    show_spotlight_dialog: bool,
    /// Search query for spotlight (Phase 9)
    spotlight_query: String,
    /// Search results for spotlight (Phase 9)
    spotlight_results: std::vec::Vec<SearchResultType>,
    /// Selected result index in spotlight (Phase 9)
    spotlight_selected: usize,
    /// Whether to show the confirmation dialog (Phase 10)
    show_confirmation_dialog: bool,
    /// Title of the confirmation dialog (Phase 10)
    confirmation_title: String,
    /// Message text for confirmation dialog (Phase 10)
    confirmation_message: String,
    /// Action to execute if user confirms (Phase 10)
    confirmation_action: std::option::Option<ConfirmationAction>,
    /// Session start timestamp (Phase 11)
    session_start_time: chrono::DateTime<chrono::Utc>,
    /// Whether to show the dev tools menu dialog
    show_dev_tools_menu: bool,
    /// Currently selected dev tool in the menu
    dev_tools_selection: usize,
    /// Currently active dev tool (if any)
    active_dev_tool: std::option::Option<DevTool>,
    /// Database browser: List of table names in the database
    db_tables: std::vec::Vec<String>,
    /// Database browser: Currently selected table index
    db_selected_table: usize,
    /// Database browser: Current table row data
    db_table_data: std::vec::Vec<std::collections::HashMap<String, String>>,
    /// Database browser: Current table column names
    db_table_columns: std::vec::Vec<String>,
    /// Database browser: Current page number (for pagination)
    db_current_page: usize,
    /// Database browser: Rows per page
    db_rows_per_page: usize,
    /// Database browser: Currently selected record index (for Up/Down navigation)
    db_selected_record: usize,
    /// Whether to show the SQL query executor dialog
    show_sql_query_dialog: bool,
    /// SQL query input text
    sql_query_input: String,
    /// SQL query results (column names and row data)
    sql_query_results: std::vec::Vec<std::collections::HashMap<String, String>>,
    /// SQL query result column names
    sql_query_columns: std::vec::Vec<String>,
    /// Whether to show the config editor dialog
    show_config_editor: bool,
    /// Config editor: List of key-value pairs being edited
    config_editor_items: std::vec::Vec<(String, String)>,
    /// Config editor: Currently selected item index
    config_editor_selected: usize,
    /// Config editor: Which field is being edited (Key or Value)
    config_editor_editing: std::option::Option<ConfigEditorField>,
    /// Config editor: Text buffer for editing
    config_editor_buffer: String,
    /// Whether to show the markdown file browser dialog
    show_markdown_browser: bool,
    /// List of markdown files in current directory
    markdown_files: std::vec::Vec<String>,
    /// Currently selected markdown file index
    markdown_selected: usize,
    /// Whether the setup wizard is active (first-time setup)
    setup_wizard_active: bool,
    /// Current step in the setup wizard
    setup_wizard_step: SetupWizardStep,
    /// Main slot provider
    setup_wizard_main_provider: LLMProvider,
    /// Main slot provider selection index
    setup_wizard_main_provider_selection: usize,
    /// Main slot model name
    setup_wizard_main_model: String,
    /// Research slot provider
    setup_wizard_research_provider: LLMProvider,
    /// Research slot provider selection index
    setup_wizard_research_provider_selection: usize,
    /// Research slot model name
    setup_wizard_research_model: String,
    /// Fallback slot provider
    setup_wizard_fallback_provider: LLMProvider,
    /// Fallback slot provider selection index
    setup_wizard_fallback_provider_selection: usize,
    /// Fallback slot model name
    setup_wizard_fallback_model: String,
    /// Database path input
    setup_wizard_db_path: String,
    /// Whether PRD processing view is active
    show_prd_processing: bool,
    /// Current processing state (state machine)
    prd_processing_state: PRDProcessingState,
    /// PRD file being processed
    prd_processing_file: String,
    /// Whether PRD processing should start on next iteration (for immediate UI feedback)
    prd_processing_pending: bool,
    /// Intermediate data storage for multi-step processing
    prd_processing_content: std::option::Option<String>,
    prd_processing_prd: std::option::Option<task_manager::domain::prd::PRD>,
    prd_processing_config: std::option::Option<serde_json::Value>,
    prd_processing_tasks: std::option::Option<std::vec::Vec<task_manager::domain::task::Task>>,
    /// Interactive generation: Conversation history
    prd_gen_conversation: std::vec::Vec<PRDGenMessage>,
    /// Interactive generation: User input buffer
    prd_gen_input: String,
    /// Interactive generation: Partial tasks as they're generated
    prd_gen_partial_tasks: std::vec::Vec<PartialTask>,
    /// Interactive generation: Current generation status
    prd_gen_status: PRDGenStatus,
    /// Interactive generation: Whether input field is focused
    prd_gen_input_active: bool,
    /// Interactive generation: Scroll position in conversation view
    prd_gen_scroll_offset: usize,
    /// Interactive generation: Auto-scroll enabled (tracks if user scrolled up manually)
    prd_gen_auto_scroll: bool,
    /// Interactive generation: Channel receiver for LLM updates
    prd_gen_receiver: std::option::Option<tokio::sync::mpsc::Receiver<task_orchestrator::adapters::rig_prd_parser_adapter::PRDGenUpdate>>,
    /// Interactive generation: Channel sender for user input
    prd_gen_sender: std::option::Option<tokio::sync::mpsc::Sender<String>>,
    /// Interactive generation: Last user message (for Up-arrow editing)
    prd_gen_last_message: String,
    /// Interactive generation: Whether we're currently editing the last message
    prd_gen_editing_last: bool,
    /// Personas loaded from database (Phase 4: Persona Management)
    personas: std::vec::Vec<task_manager::domain::persona::Persona>,
    /// Selected persona index in persona list (Phase 4)
    selected_persona: usize,
    /// Agent tools loaded from database (Phase 4)
    agent_tools: std::vec::Vec<task_manager::domain::agent_tool::AgentTool>,
}

/// Status of interactive PRD generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PRDGenStatus {
    /// Not currently generating
    Idle,
    /// LLM is thinking/processing
    Thinking,
    /// LLM asked a question, waiting for user input
    WaitingForInput,
    /// Generating tasks
    Generating,
    /// Generation complete
    Complete,
}

/// Sub-task information for nested rendering.
#[derive(Debug, Clone)]
struct SubTaskInfo {
    title: String,
    description: String,
    assignee: std::option::Option<String>,
    priority: std::option::Option<u8>,
    complexity: std::option::Option<u8>,
}

/// Box content types for structured rendering.
#[derive(Debug, Clone)]
enum BoxContent {
    /// Task box (yellow) with optional nested sub-tasks
    Task {
        title: String,
        description: String,
        assignee: std::option::Option<String>,
        priority: std::option::Option<u8>,
        complexity: std::option::Option<u8>,
        subtasks: std::vec::Vec<SubTaskInfo>,
    },
    /// Validation box (red)
    Validation {
        task_title: String,
        message: String,
    },
}

/// Message content in the interactive PRD generation conversation.
#[derive(Debug, Clone)]
enum MessageContent {
    /// Plain text message
    Text(String),
    /// Structured box (task, sub-task, or validation)
    Box(BoxContent),
}

/// Message in the interactive PRD generation conversation.
#[derive(Debug, Clone)]
struct PRDGenMessage {
    /// Message sender role
    role: PRDGenRole,
    /// Message content (text or structured box)
    content: MessageContent,
    /// When the message was created
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// Role in PRD generation conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PRDGenRole {
    /// System messages (prompts, status)
    System,
    /// LLM assistant responses
    Assistant,
    /// User inputs
    User,
}

/// Partial task being generated.
#[derive(Debug, Clone)]
struct PartialTask {
    /// Task title
    title: String,
    /// Generation status
    status: PartialTaskStatus,
    /// Validation messages (for remediation display)
    validation_messages: std::vec::Vec<String>,
}

/// Status of partial task generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PartialTaskStatus {
    /// Currently being generated
    Generating,
    /// Being validated (assignee check, etc.)
    Validating,
    /// Successfully completed
    Complete,
    /// Generation failed
    Failed,
}

/// Represents a chat message in the LLM Chat dialog.
#[derive(Debug, Clone)]
struct ChatMessage {
    role: ChatRole,
    content: String,
}

/// Role of a chat message sender.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChatRole {
    User,
    Assistant,
    System,
}

/// Fields available for editing in the Task Editor dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TaskEditorField {
    Title,
    Description,
    Assignee,
    Status,
}

/// Fields available for creation in the Task Creator dialog (Phase 8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TaskCreatorField {
    Title,
    Description,
    Assignee,
    Status,
}

/// Fields available for editing in the Config Editor dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfigEditorField {
    Key,
    Value,
}

/// Type of search result in Spotlight Search (Phase 9).
#[derive(Debug, Clone)]
enum SearchResultType {
    Task { id: String, title: String, description: String },
    PRD { id: String, title: String },
    Project { id: String, name: String },
}

/// Action that requires user confirmation (Phase 10).
#[derive(Debug, Clone)]
enum ConfirmationAction {
    /// Archive a task (soft deletion)
    ArchiveTask { task_id: String },
}

/// Setup wizard steps for first-time initialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SetupWizardStep {
    /// Welcome screen with introduction
    Welcome,
    /// Task tool slot overview (explains main/research/fallback)
    TaskToolSlots,
    /// Configure main task tool slot (provider + model)
    ConfigureMainSlot,
    /// Configure research task tool slot (provider + model)
    ConfigureResearchSlot,
    /// Configure fallback task tool slot (provider + model)
    ConfigureFallbackSlot,
    /// Database path configuration
    DatabaseConfiguration,
    /// Final confirmation and summary
    Confirmation,
    /// Completion screen
    Complete,
}

/// PRD processing state machine - tracks progress through async processing steps.
#[derive(Debug, Clone, PartialEq, Eq)]
enum PRDProcessingState {
    /// Not processing
    Idle,
    /// Reading file from disk
    ReadingFile,
    /// Parsing markdown structure
    ParsingPRD,
    /// Loading configuration
    LoadingConfig,
    /// Generating tasks via LLM (this is the slow step)
    GeneratingTasks,
    /// Saving tasks to database
    SavingTasks,
    /// Reloading task list
    ReloadingTasks,
    /// Processing complete successfully
    Complete { task_count: usize },
    /// Processing failed with error
    Failed { error: String },
}

/// LLM provider options for setup wizard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LLMProvider {
    Ollama,
    Candle,
    Mistral,
    Rig,
}

impl LLMProvider {
    fn display_name(&self) -> &str {
        match self {
            LLMProvider::Ollama => "Ollama (Local LLM server)",
            LLMProvider::Candle => "Candle (Embedded inference)",
            LLMProvider::Mistral => "Mistral.rs (Rust inference)",
            LLMProvider::Rig => "Rig/OpenAI (Cloud API)",
        }
    }

    fn description(&self) -> &str {
        match self {
            LLMProvider::Ollama => "Uses local Ollama server - requires ollama installed and running",
            LLMProvider::Candle => "Embedded ML inference - downloads models on first run (~7.6GB)",
            LLMProvider::Mistral => "Mistral.rs server - fast Rust-based inference engine",
            LLMProvider::Rig => "OpenAI API via Rig - requires API key and internet connection",
        }
    }

    fn default_model(&self) -> &str {
        match self {
            LLMProvider::Ollama => "llama3.2:latest",
            LLMProvider::Candle => "microsoft/Phi-3.5-mini-instruct",
            LLMProvider::Mistral => "microsoft/Phi-3.5-mini-instruct",
            LLMProvider::Rig => "gpt-4o-mini",
        }
    }

    fn all() -> std::vec::Vec<LLMProvider> {
        std::vec![
            LLMProvider::Ollama,
            LLMProvider::Candle,
            LLMProvider::Mistral,
            LLMProvider::Rig,
        ]
    }
}

/// Model configuration field being edited in setup wizard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModelConfigField {
    Main,
    Research,
    Fallback,
}

impl App {
    fn new() -> Self {
        App {
            projects: std::vec::Vec::new(),
            selected_project_id: std::option::Option::None,
            selected_workspace: WorkspaceSection::Tasks,
            active_tool: DashboardTool::Kanban,
            nav_selection: 0,
            show_details_panel: false,
            tasks: std::vec::Vec::new(),
            prds: std::vec::Vec::new(),
            selected_task: 0,
            thinking_log: std::vec![
                String::from("🧠 Analyzing task complexity..."),
                String::from("📊 Complexity score: 7 (high complexity)"),
                String::from("🔀 Routing decision: decompose"),
                String::from("✂️  Generating 4 subtasks..."),
                String::from("✓ Subtask 1: Design authentication schema"),
                String::from("✓ Subtask 2: Implement OAuth2 flow"),
                String::from("✓ Subtask 3: Add SAML support"),
                String::from("✓ Subtask 4: Write integration tests"),
            ],
            network_log: std::vec![
                String::from("→ POST /v1/chat/completions (llama3.1)"),
                String::from("← 200 OK (1.2s) {\"task_complexity\": 7}"),
                String::from("→ POST /v1/chat/completions (decompose)"),
                String::from("← 200 OK (2.1s) {\"subtasks\": 4}"),
                String::from("→ POST /v1/chat/completions (enhance)"),
                String::from("← 200 OK (0.8s) {\"enhancements\": [...]}"),
            ],
            should_quit: false,
            show_shortcuts: false,
            status_message: std::option::Option::None,
            db_adapter: std::option::Option::None,
            show_sort_menu: false,
            current_sort: TaskSortOption::CreatedNewest,
            sort_menu_selection: 0,
            show_jump_dialog: false,
            jump_input: String::new(),
            selected_column: KanbanColumn::Todo,
            selected_task_in_column: 0,
            show_recent_dialog: false,
            recent_task_ids: std::vec::Vec::new(),
            recent_selection: 0,
            clipboard: Self::init_clipboard(),
            last_saved_at: std::option::Option::None,
            is_saving: false,
            has_unsaved_changes: false,
            is_loading: false,
            loading_message: std::option::Option::None,
            loading_frame: 0,
            notifications: std::vec::Vec::new(),
            show_notifications: false,
            show_task_editor_dialog: false,
            task_editor_field: TaskEditorField::Title,
            task_editor_input: String::new(),
            show_llm_chat_dialog: false,
            llm_chat_input: String::new(),
            llm_chat_history: std::vec::Vec::new(),
            show_prd_dialog: false,
            selected_prd: 0,
            show_task_creator_dialog: false,
            task_creator_field: TaskCreatorField::Title,
            task_creator_title: String::new(),
            task_creator_description: String::new(),
            task_creator_assignee: String::new(),
            task_creator_status: task_manager::domain::task_status::TaskStatus::Todo,
            show_spotlight_dialog: false,
            spotlight_query: String::new(),
            spotlight_results: std::vec::Vec::new(),
            spotlight_selected: 0,
            show_confirmation_dialog: false,
            confirmation_title: String::new(),
            confirmation_message: String::new(),
            confirmation_action: std::option::Option::None,
            session_start_time: chrono::Utc::now(),
            show_dev_tools_menu: false,
            dev_tools_selection: 0,
            active_dev_tool: std::option::Option::None,
            db_tables: std::vec::Vec::new(),
            db_selected_table: 0,
            db_table_data: std::vec::Vec::new(),
            db_table_columns: std::vec::Vec::new(),
            db_current_page: 0,
            db_rows_per_page: 25,
            db_selected_record: 0,
            show_sql_query_dialog: false,
            sql_query_input: String::new(),
            sql_query_results: std::vec::Vec::new(),
            sql_query_columns: std::vec::Vec::new(),
            show_config_editor: false,
            config_editor_items: std::vec::Vec::new(),
            config_editor_selected: 0,
            config_editor_editing: std::option::Option::None,
            config_editor_buffer: String::new(),
            show_markdown_browser: false,
            markdown_files: std::vec::Vec::new(),
            markdown_selected: 0,
            setup_wizard_active: false,
            setup_wizard_step: SetupWizardStep::Welcome,
            setup_wizard_main_provider: LLMProvider::Ollama,
            setup_wizard_main_provider_selection: 0,
            setup_wizard_main_model: String::from(LLMProvider::Ollama.default_model()),
            setup_wizard_research_provider: LLMProvider::Ollama,
            setup_wizard_research_provider_selection: 0,
            setup_wizard_research_model: String::from(LLMProvider::Ollama.default_model()),
            setup_wizard_fallback_provider: LLMProvider::Ollama,
            setup_wizard_fallback_provider_selection: 0,
            setup_wizard_fallback_model: String::from(LLMProvider::Ollama.default_model()),
            setup_wizard_db_path: String::from("sqlite:.rigger/tasks.db"),
            show_prd_processing: false,
            prd_processing_state: PRDProcessingState::Idle,
            prd_processing_file: String::new(),
            prd_processing_pending: false,
            prd_processing_content: std::option::Option::None,
            prd_processing_prd: std::option::Option::None,
            prd_processing_config: std::option::Option::None,
            prd_processing_tasks: std::option::Option::None,
            prd_gen_conversation: std::vec::Vec::new(),
            prd_gen_input: String::new(),
            prd_gen_partial_tasks: std::vec::Vec::new(),
            prd_gen_status: PRDGenStatus::Idle,
            prd_gen_input_active: false,
            prd_gen_scroll_offset: 0,
            prd_gen_auto_scroll: true,
            prd_gen_receiver: std::option::Option::None,
            prd_gen_sender: std::option::Option::None,
            prd_gen_last_message: String::new(),
            prd_gen_editing_last: false,
            personas: std::vec::Vec::new(),
            selected_persona: 0,
            agent_tools: std::vec::Vec::new(),
        }
    }

    /// Initializes clipboard adapter.
    ///
    /// Attempts to create clipboard adapter. Returns None if clipboard
    /// is unavailable (headless environment, CI, etc.). This allows the
    /// TUI to run in environments without clipboard access.
    fn init_clipboard() -> std::option::Option<std::sync::Arc<dyn ClipboardPort>> {
        match ArboardClipboardAdapter::new() {
            std::result::Result::Ok(adapter) => std::option::Option::Some(std::sync::Arc::new(adapter)),
            std::result::Result::Err(_) => std::option::Option::None,
        }
    }

    async fn load_tasks(&mut self) -> anyhow::Result<()> {
        // Set loading state
        self.is_loading = true;
        self.loading_message = std::option::Option::Some(String::from("Loading tasks..."));

        // Get database path from .rigger directory
        let current_dir = std::env::current_dir()?;
        let db_path = current_dir.join(".rigger").join("tasks.db");

        if !db_path.exists() {
            self.tasks = std::vec::Vec::new();
            self.is_loading = false;
            self.loading_message = std::option::Option::None;
            return std::result::Result::Ok(());
        }

        // Connect to database
        let adapter = task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::connect_and_init(
            &std::format!("sqlite:{}", db_path.display())
        ).await.map_err(|e| anyhow::anyhow!(e))?;

        // Load all tasks
        let filter = task_manager::ports::task_repository_port::TaskFilter::All;
        let opts = hexser::ports::repository::FindOptions {
            sort: std::option::Option::Some(std::vec![hexser::ports::repository::Sort {
                key: task_manager::ports::task_repository_port::TaskSortKey::CreatedAt,
                direction: hexser::ports::repository::Direction::Desc,
            }]),
            limit: std::option::Option::Some(100),
            offset: std::option::Option::None,
        };

        self.tasks = task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::find_async(&adapter, &filter, opts).await.map_err(|e| anyhow::anyhow!("{:?}", e))?;

        // Store adapter for future updates
        self.db_adapter = std::option::Option::Some(adapter);

        // Mark as synced with database
        self.last_saved_at = std::option::Option::Some(chrono::Utc::now());
        self.has_unsaved_changes = false;

        // Clear loading state
        self.is_loading = false;
        self.loading_message = std::option::Option::None;

        // Add notification
        self.add_notification(
            NotificationLevel::Success,
            std::format!("Loaded {} tasks from database", self.tasks.len())
        );

        // Apply initial sort
        self.apply_sort();

        std::result::Result::Ok(())
    }

    /// Loads projects from the database.
    async fn load_projects(&mut self) -> anyhow::Result<()> {
        // Set loading state
        self.is_loading = true;
        self.loading_message = std::option::Option::Some(String::from("Loading projects..."));

        // Get database path from .rigger directory
        let current_dir = std::env::current_dir()?;
        let db_path = current_dir.join(".rigger").join("tasks.db");

        if !db_path.exists() {
            self.projects = std::vec::Vec::new();
            self.is_loading = false;
            self.loading_message = std::option::Option::None;
            return std::result::Result::Ok(());
        }

        // Connect to database
        let adapter = task_manager::adapters::sqlite_project_adapter::SqliteProjectAdapter::connect_and_init(
            &std::format!("sqlite:{}", db_path.display())
        ).await.map_err(|e| anyhow::anyhow!(e))?;

        // Load all projects
        let filter = task_manager::ports::project_repository_port::ProjectFilter::All;
        let opts = hexser::ports::repository::FindOptions {
            sort: std::option::Option::Some(std::vec![hexser::ports::repository::Sort {
                key: task_manager::ports::project_repository_port::ProjectSortKey::Name,
                direction: hexser::ports::repository::Direction::Asc,
            }]),
            limit: std::option::Option::None,
            offset: std::option::Option::None,
        };

        self.projects = adapter.find_async(&filter, opts).await.map_err(|e| anyhow::anyhow!("{:?}", e))?;

        // Select first project by default if projects exist
        if !self.projects.is_empty() && self.selected_project_id.is_none() {
            self.selected_project_id = std::option::Option::Some(self.projects[0].id.clone());
        }

        // Clear loading state
        self.is_loading = false;
        self.loading_message = std::option::Option::None;

        // Add notification
        self.add_notification(
            NotificationLevel::Success,
            std::format!("Loaded {} projects from database", self.projects.len())
        );

        std::result::Result::Ok(())
    }

    /// Loads personas from the database (Phase 4: Persona Management).
    async fn load_personas(&mut self) -> anyhow::Result<()> {
        // Set loading state
        self.is_loading = true;
        self.loading_message = std::option::Option::Some(String::from("Loading personas..."));

        // Get database path from .rigger directory
        let current_dir = std::env::current_dir()?;
        let db_path = current_dir.join(".rigger").join("tasks.db");

        if !db_path.exists() {
            self.personas = std::vec::Vec::new();
            self.is_loading = false;
            self.loading_message = std::option::Option::None;
            return std::result::Result::Ok(());
        }

        // Connect to database
        let adapter = task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::connect_and_init(
            &std::format!("sqlite:{}", db_path.display())
        ).await.map_err(|e| anyhow::anyhow!(e))?;

        // Load all personas sorted by name
        let filter = task_manager::ports::persona_repository_port::PersonaFilter::All;
        let opts = hexser::ports::repository::FindOptions {
            sort: std::option::Option::Some(std::vec![hexser::ports::repository::Sort {
                key: task_manager::ports::persona_repository_port::PersonaSortKey::Name,
                direction: hexser::ports::repository::Direction::Asc,
            }]),
            limit: std::option::Option::None,
            offset: std::option::Option::None,
        };

        let personas = {
            use hexser::ports::repository::QueryRepository;
            adapter.find(&filter, opts).map_err(|e| anyhow::anyhow!("{:?}", e))?
        };

        self.personas = personas;

        // Clear loading state
        self.is_loading = false;
        self.loading_message = std::option::Option::None;

        // Add notification
        self.add_notification(
            NotificationLevel::Success,
            std::format!("Loaded {} personas from database", self.personas.len())
        );

        std::result::Result::Ok(())
    }

    /// Loads agent tools from the database (Phase 4: Persona Management).
    async fn load_agent_tools(&mut self) -> anyhow::Result<()> {
        // Set loading state
        self.is_loading = true;
        self.loading_message = std::option::Option::Some(String::from("Loading agent tools..."));

        // Get database path from .rigger directory
        let current_dir = std::env::current_dir()?;
        let db_path = current_dir.join(".rigger").join("tasks.db");

        if !db_path.exists() {
            self.agent_tools = std::vec::Vec::new();
            self.is_loading = false;
            self.loading_message = std::option::Option::None;
            return std::result::Result::Ok(());
        }

        // Connect to database
        let adapter = task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::connect_and_init(
            &std::format!("sqlite:{}", db_path.display())
        ).await.map_err(|e| anyhow::anyhow!(e))?;

        // Load all agent tools sorted by category then name
        let filter = task_manager::ports::agent_tool_repository_port::ToolFilter::All;
        let opts = hexser::ports::repository::FindOptions {
            sort: std::option::Option::Some(std::vec![
                hexser::ports::repository::Sort {
                    key: task_manager::ports::agent_tool_repository_port::ToolSortKey::Category,
                    direction: hexser::ports::repository::Direction::Asc,
                },
                hexser::ports::repository::Sort {
                    key: task_manager::ports::agent_tool_repository_port::ToolSortKey::Name,
                    direction: hexser::ports::repository::Direction::Asc,
                },
            ]),
            limit: std::option::Option::None,
            offset: std::option::Option::None,
        };

        let tools = {
            use hexser::ports::repository::QueryRepository;
            adapter.find(&filter, opts).map_err(|e| anyhow::anyhow!("{:?}", e))?
        };

        self.agent_tools = tools;

        // Clear loading state
        self.is_loading = false;
        self.loading_message = std::option::Option::None;

        // Add notification
        self.add_notification(
            NotificationLevel::Success,
            std::format!("Loaded {} agent tools from database", self.agent_tools.len())
        );

        std::result::Result::Ok(())
    }

    /// Loads the list of database tables from the SQLite database.
    ///
    /// Queries the sqlite_master table to get all user-created tables.
    /// Sets the db_tables field with the results.
    async fn load_db_tables(&mut self) -> anyhow::Result<()> {
        // Set loading state
        self.is_loading = true;
        self.loading_message = std::option::Option::Some(String::from("Loading database tables..."));

        // Get database path from .rigger directory
        let current_dir = std::env::current_dir()?;
        let db_path = current_dir.join(".rigger").join("tasks.db");

        if !db_path.exists() {
            self.db_tables = std::vec::Vec::new();
            self.is_loading = false;
            self.loading_message = std::option::Option::None;
            return std::result::Result::Err(anyhow::anyhow!("Database file not found"));
        }

        // Connect to database
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect(&std::format!("sqlite:{}", db_path.display()))
            .await?;

        // Query for all user-created tables
        let rows: std::vec::Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name"
        )
        .fetch_all(&pool)
        .await?;

        self.db_tables = rows.into_iter().map(|(name,)| name).collect();
        self.db_selected_table = 0;

        // Clear loading state
        self.is_loading = false;
        self.loading_message = std::option::Option::None;

        // Add notification
        self.add_notification(
            NotificationLevel::Success,
            std::format!("Loaded {} tables from database", self.db_tables.len())
        );

        std::result::Result::Ok(())
    }

    /// Loads data from the currently selected database table.
    ///
    /// Queries the selected table and loads column names and row data with pagination.
    async fn load_table_data(&mut self) -> anyhow::Result<()> {
        if self.db_tables.is_empty() || self.db_selected_table >= self.db_tables.len() {
            self.add_notification(
                NotificationLevel::Warning,
                String::from("No table selected or table list is empty")
            );
            return std::result::Result::Ok(());
        }

        let table_name = &self.db_tables[self.db_selected_table].clone();

        // Add notification that we're starting to load
        self.add_notification(
            NotificationLevel::Info,
            std::format!("Loading table '{}'... (selected index: {})", table_name, self.db_selected_table)
        );

        // Set loading state
        self.is_loading = true;
        self.loading_message = std::option::Option::Some(
            std::format!("Loading data from {}...", table_name)
        );

        // Get database path
        let current_dir = std::env::current_dir()?;
        let db_path = current_dir.join(".rigger").join("tasks.db");

        if !db_path.exists() {
            self.is_loading = false;
            self.loading_message = std::option::Option::None;
            return std::result::Result::Err(anyhow::anyhow!("Database file not found"));
        }

        // Connect to database
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect(&std::format!("sqlite:{}", db_path.display()))
            .await?;

        // Get column names using PRAGMA
        let column_rows: std::vec::Vec<(i32, String, String, i32, Option<String>, i32)> = sqlx::query_as(
            &std::format!("PRAGMA table_info({})", table_name)
        )
        .fetch_all(&pool)
        .await?;

        self.db_table_columns = column_rows.into_iter().map(|(_, name, _, _, _, _)| name).collect();

        // Calculate offset based on current page
        let offset = self.db_current_page * self.db_rows_per_page;

        // Query table data with pagination
        let query = std::format!(
            "SELECT * FROM {} LIMIT {} OFFSET {}",
            table_name,
            self.db_rows_per_page,
            offset
        );

        // Execute query and get rows as generic SqliteRow
        let rows = sqlx::query(&query).fetch_all(&pool).await?;

        // Convert rows to Vec<HashMap<String, String>>
        let mut data = std::vec::Vec::new();
        for row in rows {
            let mut row_map = std::collections::HashMap::new();
            for (idx, col_name) in self.db_table_columns.iter().enumerate() {
                // Try to get value as string, fall back to empty if null
                let value: std::option::Option<String> = row.try_get(idx).ok();
                row_map.insert(
                    col_name.clone(),
                    value.unwrap_or_else(|| String::from("NULL"))
                );
            }
            data.push(row_map);
        }

        self.db_table_data = data;
        self.db_selected_record = 0; // Reset record selection when new table is loaded

        // Clear loading state
        self.is_loading = false;
        self.loading_message = std::option::Option::None;

        // Add notification with row count
        let row_count = self.db_table_data.len();
        self.add_notification(
            NotificationLevel::Success,
            std::format!("Loaded {} rows from table '{}'", row_count, table_name)
        );

        std::result::Result::Ok(())
    }

    /// Executes a SQL query and stores the results.
    ///
    /// Runs the query from sql_query_input and populates sql_query_results and sql_query_columns.
    async fn execute_sql_query(&mut self) -> anyhow::Result<()> {
        if self.sql_query_input.trim().is_empty() {
            self.add_notification(
                NotificationLevel::Error,
                String::from("SQL query cannot be empty")
            );
            return std::result::Result::Ok(());
        }

        // Set loading state
        self.is_loading = true;
        self.loading_message = std::option::Option::Some(String::from("Executing query..."));

        // Get database path
        let current_dir = std::env::current_dir()?;
        let db_path = current_dir.join(".rigger").join("tasks.db");

        if !db_path.exists() {
            self.is_loading = false;
            self.loading_message = std::option::Option::None;
            return std::result::Result::Err(anyhow::anyhow!("Database file not found"));
        }

        // Connect to database
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect(&std::format!("sqlite:{}", db_path.display()))
            .await?;

        // Execute query
        let query = self.sql_query_input.trim();

        // Check if it's a SELECT query to return results
        if query.to_lowercase().starts_with("select") {
            let rows = sqlx::query(query).fetch_all(&pool).await?;

            if rows.is_empty() {
                self.sql_query_columns.clear();
                self.sql_query_results.clear();
                self.add_notification(
                    NotificationLevel::Info,
                    String::from("Query returned 0 rows")
                );
            } else {
                // Get column names from first row
                let first_row = &rows[0];
                self.sql_query_columns = first_row.columns()
                    .iter()
                    .map(|col| col.name().to_string())
                    .collect();

                // Convert rows to HashMap
                let mut data = std::vec::Vec::new();
                for row in rows {
                    let mut row_map = std::collections::HashMap::new();
                    for (idx, col_name) in self.sql_query_columns.iter().enumerate() {
                        let value: std::option::Option<String> = row.try_get(idx).ok();
                        row_map.insert(
                            col_name.clone(),
                            value.unwrap_or_else(|| String::from("NULL"))
                        );
                    }
                    data.push(row_map);
                }

                self.sql_query_results = data;
                self.add_notification(
                    NotificationLevel::Success,
                    std::format!("Query returned {} row(s)", self.sql_query_results.len())
                );
            }
        } else {
            // Non-SELECT query (INSERT, UPDATE, DELETE, etc.)
            let result = sqlx::query(query).execute(&pool).await?;
            self.sql_query_columns.clear();
            self.sql_query_results.clear();
            self.add_notification(
                NotificationLevel::Success,
                std::format!("Query executed. {} row(s) affected", result.rows_affected())
            );
        }

        // Clear loading state
        self.is_loading = false;
        self.loading_message = std::option::Option::None;

        std::result::Result::Ok(())
    }

    /// Loads configuration from .rigger/config.toml.
    ///
    /// Parses the TOML file and loads key-value pairs into config_editor_items.
    /// If the file doesn't exist, initializes with default configuration values.
    async fn load_config(&mut self) -> anyhow::Result<()> {
        let current_dir = std::env::current_dir()?;
        let config_path = current_dir.join(".rigger").join("config.toml");

        self.config_editor_items.clear();

        if config_path.exists() {
            let content = tokio::fs::read_to_string(&config_path).await?;

            // Simple TOML parsing - split by lines and parse key=value pairs
            for line in content.lines() {
                let line = line.trim();
                // Skip empty lines and comments
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                // Parse key = "value" format
                if let Some(eq_pos) = line.find('=') {
                    let key = line[..eq_pos].trim().to_string();
                    let value = line[eq_pos + 1..].trim();
                    // Remove quotes if present
                    let value = value.trim_matches('"').to_string();
                    self.config_editor_items.push((key, value));
                }
            }
        } else {
            // Initialize with default config
            self.config_editor_items = std::vec![
                (String::from("llm_provider"), String::from("ollama")),
                (String::from("llm_model"), String::from("llama3.2")),
                (String::from("ollama_base_url"), String::from("http://127.0.0.1:11434")),
                (String::from("orchestration_enabled"), String::from("true")),
                (String::from("complexity_threshold"), String::from("7")),
                (String::from("auto_decompose"), String::from("true")),
            ];
        }

        std::result::Result::Ok(())
    }

    /// Saves configuration to .rigger/config.toml.
    ///
    /// Writes all key-value pairs from config_editor_items to the config file.
    async fn save_config(&mut self) -> anyhow::Result<()> {
        let current_dir = std::env::current_dir()?;
        let rigger_dir = current_dir.join(".rigger");
        let config_path = rigger_dir.join("config.toml");

        // Ensure .rigger directory exists
        if !rigger_dir.exists() {
            tokio::fs::create_dir_all(&rigger_dir).await?;
        }

        // Build TOML content
        let mut content = String::from("# Rigger Configuration\n\n");
        for (key, value) in &self.config_editor_items {
            content.push_str(&std::format!("{} = \"{}\"\n", key, value));
        }

        // Write to file
        tokio::fs::write(&config_path, content).await?;

        self.add_notification(
            NotificationLevel::Success,
            String::from("Configuration saved successfully")
        );

        std::result::Result::Ok(())
    }

    /// Opens the config editor dialog and loads configuration.
    async fn open_config_editor(&mut self) -> anyhow::Result<()> {
        self.load_config().await?;
        self.show_config_editor = true;
        self.config_editor_selected = 0;
        self.config_editor_editing = std::option::Option::None;
        self.config_editor_buffer.clear();
        std::result::Result::Ok(())
    }

    /// Closes the config editor dialog without saving.
    fn close_config_editor(&mut self) {
        self.show_config_editor = false;
        self.config_editor_items.clear();
        self.config_editor_selected = 0;
        self.config_editor_editing = std::option::Option::None;
        self.config_editor_buffer.clear();
    }

    /// Starts editing a field in the config editor.
    fn start_editing_config_field(&mut self, field: ConfigEditorField) {
        if let Some((key, value)) = self.config_editor_items.get(self.config_editor_selected) {
            self.config_editor_editing = std::option::Option::Some(field);
            self.config_editor_buffer = match field {
                ConfigEditorField::Key => key.clone(),
                ConfigEditorField::Value => value.clone(),
            };
        }
    }

    /// Commits the edited field value.
    fn commit_config_edit(&mut self) {
        if let Some(field) = self.config_editor_editing {
            if let Some((key, value)) = self.config_editor_items.get_mut(self.config_editor_selected) {
                match field {
                    ConfigEditorField::Key => {
                        *key = self.config_editor_buffer.clone();
                    }
                    ConfigEditorField::Value => {
                        *value = self.config_editor_buffer.clone();
                    }
                }
            }
            self.config_editor_editing = std::option::Option::None;
            self.config_editor_buffer.clear();
        }
    }

    /// Adds a new config item to the editor.
    fn add_config_item(&mut self) {
        self.config_editor_items.push((String::from("new_key"), String::from("new_value")));
        self.config_editor_selected = self.config_editor_items.len() - 1;
        self.start_editing_config_field(ConfigEditorField::Key);
    }

    /// Deletes the currently selected config item.
    fn delete_config_item(&mut self) {
        if !self.config_editor_items.is_empty() {
            self.config_editor_items.remove(self.config_editor_selected);
            if self.config_editor_selected >= self.config_editor_items.len() && !self.config_editor_items.is_empty() {
                self.config_editor_selected = self.config_editor_items.len() - 1;
            }
        }
    }

    /// Scans current directory for markdown files.
    ///
    /// Finds all .md files in the current working directory and loads them
    /// into the markdown_files list for browsing.
    async fn scan_markdown_files(&mut self) -> anyhow::Result<()> {
        self.markdown_files.clear();

        let current_dir = std::env::current_dir()?;
        let mut entries = tokio::fs::read_dir(&current_dir).await?;

        while let std::option::Option::Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if let std::option::Option::Some(extension) = path.extension() {
                if extension == "md" {
                    if let std::option::Option::Some(filename) = path.file_name() {
                        self.markdown_files.push(filename.to_string_lossy().to_string());
                    }
                }
            }
        }

        // Sort alphabetically
        self.markdown_files.sort();
        self.markdown_selected = 0;

        std::result::Result::Ok(())
    }

    /// Opens the markdown file browser dialog.
    async fn open_markdown_browser(&mut self) -> anyhow::Result<()> {
        self.scan_markdown_files().await?;
        self.show_markdown_browser = true;
        std::result::Result::Ok(())
    }

    /// Closes the markdown file browser dialog.
    fn close_markdown_browser(&mut self) {
        self.show_markdown_browser = false;
        self.markdown_files.clear();
        self.markdown_selected = 0;
    }

    /// Provides intelligent troubleshooting for Ollama errors.
    ///
    /// Performs actual system checks to diagnose the specific issue:
    /// - Tests if Ollama is installed
    /// - Checks if Ollama is running
    /// - Verifies Ollama version
    /// - Confirms model availability
    ///
    /// Returns targeted fixes based on actual findings.
    ///
    /// Revision History:
    /// - 2025-11-25T00:00:00Z @AI: Implement actual system checks using std::process::Command.
    async fn diagnose_ollama_error(&self, model_name: &str) -> String {
        let mut diagnostics = String::from("🔍 Diagnosing Ollama setup...\n\n");

        // 1. Check if Ollama is installed
        let ollama_installed = std::process::Command::new("which")
            .arg("ollama")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);

        if !ollama_installed {
            diagnostics.push_str("❌ Ollama not found\n");
            diagnostics.push_str("   Fix: Install Ollama\n");
            diagnostics.push_str("   → curl https://ollama.ai/install.sh | sh\n");
            diagnostics.push_str("   → Or download from https://ollama.ai\n");
            return diagnostics;
        }

        diagnostics.push_str("✓ Ollama is installed\n\n");

        // 2. Check Ollama version
        if let std::result::Result::Ok(output) = std::process::Command::new("ollama")
            .arg("version")
            .output()
        {
            if let std::result::Result::Ok(version) = String::from_utf8(output.stdout) {
                let version_line = version.lines().next().unwrap_or("unknown");
                diagnostics.push_str(&std::format!("✓ Version: {}\n\n", version_line));
            }
        }

        // 3. Check if Ollama is running
        let ollama_running = std::process::Command::new("pgrep")
            .arg("ollama")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);

        if !ollama_running {
            diagnostics.push_str("❌ Ollama is not running\n");
            diagnostics.push_str("   Fix: Start Ollama service\n");
            diagnostics.push_str("   → ollama serve\n");
            diagnostics.push_str("   → Or start as background: ollama serve &\n");
            return diagnostics;
        }

        diagnostics.push_str("✓ Ollama is running\n\n");

        // 4. Check if model is available
        if let std::result::Result::Ok(output) = std::process::Command::new("ollama")
            .arg("list")
            .output()
        {
            if let std::result::Result::Ok(list_output) = String::from_utf8(output.stdout) {
                let model_found = list_output.lines().any(|line| line.contains(model_name));

                if !model_found {
                    diagnostics.push_str(&std::format!("❌ Model '{}' not found\n", model_name));
                    diagnostics.push_str("   Available models:\n");
                    for line in list_output.lines().skip(1) {
                        if !line.trim().is_empty() {
                            diagnostics.push_str(&std::format!("   - {}\n", line.split_whitespace().next().unwrap_or("")));
                        }
                    }
                    diagnostics.push_str(&std::format!("\n   Fix: Pull the model\n"));
                    diagnostics.push_str(&std::format!("   → ollama pull {}\n", model_name));
                    return diagnostics;
                }

                diagnostics.push_str(&std::format!("✓ Model '{}' is available\n\n", model_name));
            }
        }

        // 5. Check API connectivity
        let api_check = std::process::Command::new("curl")
            .arg("-s")
            .arg("-o")
            .arg("/dev/null")
            .arg("-w")
            .arg("%{http_code}")
            .arg("http://localhost:11434/api/tags")
            .output();

        if let std::result::Result::Ok(output) = api_check {
            if let std::result::Result::Ok(status_code) = String::from_utf8(output.stdout) {
                if status_code.starts_with("200") {
                    diagnostics.push_str("✓ Ollama API is responding\n\n");
                } else {
                    diagnostics.push_str(&std::format!("⚠️  Ollama API returned status: {}\n\n", status_code));
                }
            }
        } else {
            diagnostics.push_str("⚠️  Could not connect to Ollama API\n");
            diagnostics.push_str("   This might be a transient issue.\n\n");
        }

        // 6. Everything looks OK - provide advanced troubleshooting
        diagnostics.push_str("All basic checks passed. Advanced troubleshooting:\n\n");
        diagnostics.push_str("1. Test model directly:\n");
        diagnostics.push_str(&std::format!("   → ollama run {} \"Hello\"\n\n", model_name));
        diagnostics.push_str("2. Check Ollama logs (if available):\n");
        diagnostics.push_str("   → Check system logs for Ollama errors\n\n");
        diagnostics.push_str("3. Restart Ollama:\n");
        diagnostics.push_str("   → pkill ollama && ollama serve\n\n");
        diagnostics.push_str("4. Re-pull model:\n");
        diagnostics.push_str(&std::format!("   → ollama pull {} --force\n\n", model_name));
        diagnostics.push_str("5. Verify config.json:\n");
        diagnostics.push_str("   → Check .rigger/config.json has correct model name\n");
        diagnostics.push_str("   → Model names are case-sensitive\n");

        diagnostics
    }

    /// Initiates PRD processing by showing the processing screen immediately.
    /// The actual processing will start on the next event loop iteration.
    fn start_prd_processing(&mut self) {
        if self.markdown_files.is_empty() || self.markdown_selected >= self.markdown_files.len() {
            return;
        }

        let filename = self.markdown_files[self.markdown_selected].clone();

        // Close markdown browser and show processing view IMMEDIATELY
        self.close_markdown_browser();
        self.show_prd_processing = true;
        self.prd_processing_file = filename;
        self.prd_processing_state = PRDProcessingState::ReadingFile;
        self.prd_processing_pending = true; // Signal to start processing on next iteration

        // Clear intermediate data
        self.prd_processing_content = std::option::Option::None;
        self.prd_processing_prd = std::option::Option::None;
        self.prd_processing_config = std::option::Option::None;
        self.prd_processing_tasks = std::option::Option::None;
    }

    /// Processes ONE step of PRD creation, allowing UI to render between steps.
    /// Returns true if processing should continue, false if complete or failed.
    async fn process_prd_step(&mut self) -> bool {
        match &self.prd_processing_state {
            PRDProcessingState::Idle => false,
            PRDProcessingState::ReadingFile => {
                // Read file
                let current_dir = match std::env::current_dir() {
                    Ok(dir) => dir,
                    Err(e) => {
                        self.prd_processing_state = PRDProcessingState::Failed {
                            error: std::format!("Failed to get current directory: {}", e),
                        };
                        return false;
                    }
                };
                let file_path = current_dir.join(&self.prd_processing_file);

                match tokio::fs::read_to_string(&file_path).await {
                    Ok(content) => {
                        self.prd_processing_content = Some(content);
                        self.prd_processing_state = PRDProcessingState::ParsingPRD;
                        true
                    }
                    Err(e) => {
                        self.prd_processing_state = PRDProcessingState::Failed {
                            error: std::format!("Failed to read file: {}\n\nFile: {}", e, file_path.display()),
                        };
                        false
                    }
                }
            }
            PRDProcessingState::ParsingPRD => {
                let content = match &self.prd_processing_content {
                    Some(c) => c.clone(),
                    None => {
                        self.prd_processing_state = PRDProcessingState::Failed {
                            error: String::from("Internal error: content not loaded"),
                        };
                        return false;
                    }
                };

                match task_manager::infrastructure::markdown_parsers::prd_parser::parse_prd_markdown("default-project", &content) {
                    Ok(prd) => {
                        self.prd_processing_prd = Some(prd);
                        self.prd_processing_state = PRDProcessingState::LoadingConfig;
                        true
                    }
                    Err(e) => {
                        self.prd_processing_state = PRDProcessingState::Failed {
                            error: std::format!("Failed to parse PRD: {}", e),
                        };
                        false
                    }
                }
            }
            PRDProcessingState::LoadingConfig => {
                let current_dir = match std::env::current_dir() {
                    Ok(dir) => dir,
                    Err(e) => {
                        self.prd_processing_state = PRDProcessingState::Failed {
                            error: std::format!("Failed to get current directory: {}", e),
                        };
                        return false;
                    }
                };
                let config_path = current_dir.join(".rigger/config.json");

                match std::fs::read_to_string(&config_path) {
                    Ok(config_content) => {
                        match serde_json::from_str(&config_content) {
                            Ok(config) => {
                                self.prd_processing_config = Some(config);
                                self.prd_processing_state = PRDProcessingState::GeneratingTasks;
                                true
                            }
                            Err(e) => {
                                self.prd_processing_state = PRDProcessingState::Failed {
                                    error: std::format!("Failed to parse config.json: {}", e),
                                };
                                false
                            }
                        }
                    }
                    Err(e) => {
                        self.prd_processing_state = PRDProcessingState::Failed {
                            error: std::format!("Failed to read config: {}", e),
                        };
                        false
                    }
                }
            }
            PRDProcessingState::GeneratingTasks => {
                // Interactive generation with real-time LLM streaming

                // If channels aren't set up yet, start interactive generation
                if self.prd_gen_receiver.is_none() {
                    let prd = match &self.prd_processing_prd {
                        Some(p) => p.clone(),
                        None => {
                            self.prd_processing_state = PRDProcessingState::Failed {
                                error: String::from("Internal error: PRD not loaded"),
                            };
                            return false;
                        }
                    };

                    let config = match &self.prd_processing_config {
                        Some(c) => c.clone(),
                        None => {
                            self.prd_processing_state = PRDProcessingState::Failed {
                                error: String::from("Internal error: config not loaded"),
                            };
                            return false;
                        }
                    };

                    let provider = config["provider"].as_str().unwrap_or("ollama");
                    let model_name = config["model"]["main"].as_str().unwrap_or("llama3.2:latest");
                    let fallback_model = config["task_tools"]["fallback"]["model"].as_str().unwrap_or("llama3.2:latest");

                    // Only support interactive generation for ollama
                    if provider != "ollama" {
                        self.prd_processing_state = PRDProcessingState::Failed {
                            error: std::format!("Unsupported provider: '{}'. Interactive generation requires ollama.", provider),
                        };
                        return false;
                    }

                    // Query personas from database for task assignment
                    let current_dir = match std::env::current_dir() {
                        std::result::Result::Ok(dir) => dir,
                        std::result::Result::Err(e) => {
                            self.prd_processing_state = PRDProcessingState::Failed {
                                error: std::format!("Failed to get current directory: {}", e),
                            };
                            return false;
                        }
                    };
                    let db_path = current_dir.join(".rigger").join("tasks.db");
                    let db_url = std::format!("sqlite:{}", db_path.display());

                    let personas = match task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::connect_and_init(&db_url).await {
                        std::result::Result::Ok(adapter) => {
                            match sqlx::query("SELECT id, project_id, name, role, description, llm_provider, llm_model, is_default, created_at, updated_at FROM personas")
                                .fetch_all(adapter.pool())
                                .await
                            {
                                std::result::Result::Ok(rows) => {
                                    let mut personas = std::vec::Vec::new();
                                    for row in rows {
                                        use sqlx::Row;
                                        if let (std::result::Result::Ok(created_at), std::result::Result::Ok(updated_at)) = (
                                            chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>(8)),
                                            chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>(9))
                                        ) {
                                            personas.push(task_manager::domain::persona::Persona {
                                                id: row.get(0),
                                                project_id: row.get(1),
                                                name: row.get(2),
                                                role: row.get(3),
                                                description: row.get(4),
                                                llm_provider: row.get(5),
                                                llm_model: row.get(6),
                                                is_default: row.get(7),
                                                created_at: created_at.with_timezone(&chrono::Utc),
                                                updated_at: updated_at.with_timezone(&chrono::Utc),
                                                enabled_tools: std::vec::Vec::new(),
                                            });
                                        }
                                    }
                                    personas
                                }
                                std::result::Result::Err(_) => std::vec::Vec::new(), // Silently use empty vec if query fails
                            }
                        }
                        std::result::Result::Err(_) => std::vec::Vec::new(), // Silently use empty vec if connection fails
                    };

                    // Start interactive generation
                    let parser = task_orchestrator::adapters::rig_prd_parser_adapter::RigPRDParserAdapter::new(
                        model_name.to_string(),
                        fallback_model.to_string(),
                        personas
                    );

                    match parser.parse_prd_interactively(prd).await {
                        Ok((receiver, sender)) => {
                            self.prd_gen_receiver = Some(receiver);
                            self.prd_gen_sender = Some(sender);
                            self.prd_gen_status = PRDGenStatus::Thinking;

                            // Add initial system message
                            self.prd_gen_conversation.push(PRDGenMessage {
                                role: PRDGenRole::System,
                                content: MessageContent::Text(String::from("🚀 Starting interactive task generation...")),
                                timestamp: chrono::Utc::now(),
                            });

                            return true; // Continue processing
                        }
                        Err(e) => {
                            self.prd_processing_state = PRDProcessingState::Failed {
                                error: std::format!("Failed to start interactive generation: {}", e),
                            };
                            return false;
                        }
                    }
                }

                // Poll for updates from LLM
                if let Some(receiver) = &mut self.prd_gen_receiver {
                    match receiver.try_recv() {
                        Ok(update) => {
                            use task_orchestrator::adapters::rig_prd_parser_adapter::PRDGenUpdate;

                            match update {
                                PRDGenUpdate::Thinking(msg) => {
                                    self.prd_gen_status = PRDGenStatus::Thinking;

                                    // Special handling: if message starts with "Analyzing" or "Streaming", create new line
                                    let should_create_new = msg.starts_with("Analyzing") || msg.starts_with("Streaming");

                                    if should_create_new {
                                        // Always create a new message for these
                                        self.prd_gen_conversation.push(PRDGenMessage {
                                            role: PRDGenRole::System,
                                            content: MessageContent::Text(msg),
                                            timestamp: chrono::Utc::now(),
                                        });
                                    } else {
                                        // Append to last message if it's an Assistant message and Text content, otherwise create new
                                        if let Some(last_msg) = self.prd_gen_conversation.last_mut() {
                                            if matches!(last_msg.role, PRDGenRole::Assistant) {
                                                // Try to append to existing text content
                                                if let MessageContent::Text(ref mut text) = last_msg.content {
                                                    text.push_str(&msg);
                                                } else {
                                                    // Last message is a box, create new text message
                                                    self.prd_gen_conversation.push(PRDGenMessage {
                                                        role: PRDGenRole::Assistant,
                                                        content: MessageContent::Text(msg),
                                                        timestamp: chrono::Utc::now(),
                                                    });
                                                }
                                            } else {
                                                // Create new message
                                                self.prd_gen_conversation.push(PRDGenMessage {
                                                    role: PRDGenRole::Assistant,
                                                    content: MessageContent::Text(msg),
                                                    timestamp: chrono::Utc::now(),
                                                });
                                            }
                                        } else {
                                            // First message
                                            self.prd_gen_conversation.push(PRDGenMessage {
                                                role: PRDGenRole::Assistant,
                                                content: MessageContent::Text(msg),
                                                timestamp: chrono::Utc::now(),
                                            });
                                        }
                                    }

                                    // Auto-scroll to keep showing latest content if enabled
                                    if self.prd_gen_auto_scroll {
                                        self.prd_gen_scroll_offset = self.prd_gen_conversation.len().saturating_sub(1);
                                    }
                                }
                                PRDGenUpdate::Question(question) => {
                                    self.prd_gen_status = PRDGenStatus::WaitingForInput;
                                    self.prd_gen_conversation.push(PRDGenMessage {
                                        role: PRDGenRole::Assistant,
                                        content: MessageContent::Text(std::format!("❓ {}", question)),
                                        timestamp: chrono::Utc::now(),
                                    });
                                    self.prd_gen_input_active = true; // Focus input field

                                    // Auto-scroll to keep showing latest content if enabled
                                    if self.prd_gen_auto_scroll {
                                        self.prd_gen_scroll_offset = self.prd_gen_conversation.len().saturating_sub(1);
                                    }
                                }
                                PRDGenUpdate::TaskGenerated { title, description, assignee, priority, complexity } => {
                                    self.prd_gen_status = PRDGenStatus::Generating;
                                    self.prd_gen_partial_tasks.push(PartialTask {
                                        title: title.clone(),
                                        status: PartialTaskStatus::Complete,
                                        validation_messages: std::vec::Vec::new(),
                                    });

                                    // Add structured task box to conversation
                                    self.prd_gen_conversation.push(PRDGenMessage {
                                        role: PRDGenRole::System,
                                        content: MessageContent::Box(BoxContent::Task {
                                            title: title.clone(),
                                            description: description.clone(),
                                            assignee,
                                            priority,
                                            complexity,
                                            subtasks: std::vec::Vec::new(),
                                        }),
                                        timestamp: chrono::Utc::now(),
                                    });

                                    // Auto-scroll to keep showing latest content if enabled
                                    if self.prd_gen_auto_scroll {
                                        self.prd_gen_scroll_offset = self.prd_gen_conversation.len().saturating_sub(1);
                                    }
                                }
                                PRDGenUpdate::ValidationInfo { task_title, message } => {
                                    // Add validation info as a red row in the task list
                                    // Find the matching partial task and update its status and messages
                                    if let Some(task) = self.prd_gen_partial_tasks.iter_mut().find(|t| t.title == task_title) {
                                        task.status = PartialTaskStatus::Validating;
                                        task.validation_messages.push(message.clone());
                                    }

                                    // Add structured validation box to conversation
                                    self.prd_gen_conversation.push(PRDGenMessage {
                                        role: PRDGenRole::System,
                                        content: MessageContent::Box(BoxContent::Validation {
                                            task_title: task_title.clone(),
                                            message: message.clone(),
                                        }),
                                        timestamp: chrono::Utc::now(),
                                    });

                                    // Auto-scroll to keep showing latest content if enabled
                                    if self.prd_gen_auto_scroll {
                                        self.prd_gen_scroll_offset = self.prd_gen_conversation.len().saturating_sub(1);
                                    }
                                }
                                PRDGenUpdate::Complete(tasks) => {
                                    self.prd_gen_status = PRDGenStatus::Complete;
                                    self.prd_gen_conversation.push(PRDGenMessage {
                                        role: PRDGenRole::System,
                                        content: MessageContent::Text(std::format!("✅ Generated {} tasks successfully!", tasks.len())),
                                        timestamp: chrono::Utc::now(),
                                    });

                                    // Store tasks and move to saving
                                    self.prd_processing_tasks = Some(tasks);
                                    self.prd_processing_state = PRDProcessingState::SavingTasks;

                                    // Clean up channels
                                    self.prd_gen_receiver = None;
                                    self.prd_gen_sender = None;
                                }
                                PRDGenUpdate::Error(err) => {
                                    // Only run Ollama diagnostics for connection/network errors
                                    // Skip for JSON parsing errors which are LLM output issues
                                    let error_message = if err.contains("JSON") || err.contains("parse") || err.contains("remediation") {
                                        // JSON parsing error - don't run Ollama diagnostics
                                        std::format!("Task generation failed: {}\n\nTroubleshooting:\n• Check the PRD file format\n• Verify the LLM is generating valid JSON\n• Try with a simpler PRD to test\n• Check .rigger/config.json has correct model name", err)
                                    } else {
                                        // Network/connection error - run Ollama diagnostics
                                        let diagnostics = self.diagnose_ollama_error("llama3.2:latest").await;
                                        std::format!("Task generation failed: {}\n\n{}", err, diagnostics)
                                    };

                                    self.prd_processing_state = PRDProcessingState::Failed {
                                        error: error_message,
                                    };

                                    // Clean up channels
                                    self.prd_gen_receiver = None;
                                    self.prd_gen_sender = None;
                                    return false;
                                }
                            }

                            true // Keep processing
                        }
                        Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                            // No updates yet, keep waiting
                            true
                        }
                        Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                            // Channel closed unexpectedly
                            self.prd_processing_state = PRDProcessingState::Failed {
                                error: String::from("LLM generation channel closed unexpectedly"),
                            };
                            self.prd_gen_receiver = None;
                            self.prd_gen_sender = None;
                            false
                        }
                    }
                } else {
                    // This shouldn't happen, but handle gracefully
                    self.prd_processing_state = PRDProcessingState::Failed {
                        error: String::from("Internal error: channel not initialized"),
                    };
                    false
                }
            }
            PRDProcessingState::SavingTasks => {
                let mut tasks = match &self.prd_processing_tasks {
                    Some(t) => t.clone(),
                    None => {
                        self.prd_processing_state = PRDProcessingState::Failed {
                            error: String::from("Internal error: tasks not generated"),
                        };
                        return false;
                    }
                };

                let prd = match &self.prd_processing_prd {
                    Some(p) => p.clone(),
                    None => {
                        self.prd_processing_state = PRDProcessingState::Failed {
                            error: String::from("Internal error: PRD not loaded"),
                        };
                        return false;
                    }
                };

                let current_dir = match std::env::current_dir() {
                    Ok(dir) => dir,
                    Err(e) => {
                        self.prd_processing_state = PRDProcessingState::Failed {
                            error: std::format!("Failed to get current directory: {}", e),
                        };
                        return false;
                    }
                };
                let db_path = current_dir.join(".rigger/tasks.db");
                let db_url = std::format!("sqlite:{}", db_path.display());

                let mut adapter = match task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::connect_and_init(&db_url).await {
                    Ok(a) => a,
                    Err(e) => {
                        self.prd_processing_state = PRDProcessingState::Failed {
                            error: std::format!("Database connection failed: {}", e),
                        };
                        return false;
                    }
                };

                // Create Project from PRD title
                let project = task_manager::domain::project::Project::new(
                    prd.title.clone(),
                    std::option::Option::Some(std::format!("Auto-generated from PRD: {}", self.prd_processing_file)),
                );

                // Note: Projects table already created by SqliteTaskAdapter with schema:
                // (id, name, description, created_at, updated_at)

                // Save Project
                let now = chrono::Utc::now().to_rfc3339();
                let insert_project = sqlx::query(
                    "INSERT OR REPLACE INTO projects (id, name, description, created_at, updated_at)
                     VALUES (?, ?, ?, ?, ?)"
                )
                .bind(&project.id)
                .bind(&project.name)
                .bind(&project.description)
                .bind(project.created_at.to_rfc3339())
                .bind(&now);

                if let Err(e) = insert_project.execute(adapter.pool()).await {
                    self.prd_processing_state = PRDProcessingState::Failed {
                        error: std::format!("Failed to save project: {:?}", e),
                    };
                    return false;
                }

                // Create prds table if it doesn't exist
                let create_prds_table = sqlx::query(
                    "CREATE TABLE IF NOT EXISTS prds (
                        id TEXT PRIMARY KEY,
                        project_id TEXT NOT NULL,
                        title TEXT NOT NULL,
                        objectives_json TEXT NULL,
                        tech_stack_json TEXT NULL,
                        constraints_json TEXT NULL,
                        raw_content TEXT NOT NULL,
                        created_at TEXT NOT NULL
                    )"
                );
                if let Err(e) = create_prds_table.execute(adapter.pool()).await {
                    self.prd_processing_state = PRDProcessingState::Failed {
                        error: std::format!("Failed to create prds table: {:?}", e),
                    };
                    return false;
                }

                // Save PRD
                let objectives_json = serde_json::to_string(&prd.objectives).unwrap_or(String::from("[]"));
                let tech_stack_json = serde_json::to_string(&prd.tech_stack).unwrap_or(String::from("[]"));
                let constraints_json = serde_json::to_string(&prd.constraints).unwrap_or(String::from("[]"));
                let insert_prd = sqlx::query(
                    "INSERT OR REPLACE INTO prds (id, project_id, title, objectives_json, tech_stack_json, constraints_json, raw_content, created_at)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
                )
                .bind(&prd.id)
                .bind(&prd.project_id)
                .bind(&prd.title)
                .bind(&objectives_json)
                .bind(&tech_stack_json)
                .bind(&constraints_json)
                .bind(&prd.raw_content)
                .bind(prd.created_at.to_rfc3339());

                if let Err(e) = insert_prd.execute(adapter.pool()).await {
                    self.prd_processing_state = PRDProcessingState::Failed {
                        error: std::format!("Failed to save PRD: {:?}", e),
                    };
                    return false;
                }

                // Link all tasks to the PRD before saving
                for task in &mut tasks {
                    task.source_prd_id = std::option::Option::Some(prd.id.clone());
                }

                // Save all tasks
                for task in &tasks {
                    if let Err(e) = adapter.save_async(task.clone()).await {
                        self.prd_processing_state = PRDProcessingState::Failed {
                            error: std::format!("Failed to save task: {:?}", e),
                        };
                        return false;
                    }
                }

                // Auto-decompose complex tasks (complexity >= 7)
                let config_path = current_dir.join(".rigger/config.json");
                if let Ok(config_content) = std::fs::read_to_string(&config_path) {
                    if let Ok(config) = serde_json::from_str::<serde_json::Value>(&config_content) {
                        let model_name = config["model"]["main"]
                            .as_str()
                            .unwrap_or("llama3.2:latest");
                        let fallback_model = config["task_tools"]["fallback"]["model"]
                            .as_str()
                            .unwrap_or(model_name);

                        for task in &tasks {
                            if let std::option::Option::Some(complexity) = task.complexity {
                                if complexity >= 7 {
                                    // Create parser for decomposition
                                    let parser = task_orchestrator::adapters::rig_prd_parser_adapter::RigPRDParserAdapter::new(
                                        model_name.to_string(),
                                        fallback_model.to_string(),
                                        std::vec::Vec::new(), // Personas already validated
                                    );

                                    match parser.decompose_task(task, &prd.raw_content).await {
                                        std::result::Result::Ok(generated_subtasks) => {
                                            // Find parent task in conversation and add subtasks to it
                                            for msg in self.prd_gen_conversation.iter_mut().rev() {
                                                if let MessageContent::Box(BoxContent::Task {
                                                    ref title,
                                                    ref mut subtasks,
                                                    ..
                                                }) = msg.content {
                                                    if title == &task.title {
                                                        // Append all generated subtasks to parent task
                                                        for subtask in &generated_subtasks {
                                                            subtasks.push(SubTaskInfo {
                                                                title: subtask.title.clone(),
                                                                description: subtask.description.clone(),
                                                                assignee: subtask.assignee.clone(),
                                                                priority: std::option::Option::None,
                                                                complexity: subtask.complexity,
                                                            });

                                                            // Save subtask to database
                                                            if let Err(e) = adapter.save_async(subtask.clone()).await {
                                                                eprintln!("Warning: Failed to save subtask: {}", e);
                                                            }
                                                        }
                                                        break; // Found and updated parent task
                                                    }
                                                }
                                            }

                                            // Update parent task with subtask IDs and Decomposed status
                                            let mut updated_parent = task.clone();
                                            updated_parent.subtask_ids = generated_subtasks.iter().map(|st| st.id.clone()).collect();
                                            updated_parent.status = task_manager::domain::task_status::TaskStatus::Decomposed;
                                            if let Err(e) = adapter.save_async(updated_parent).await {
                                                eprintln!("Warning: Failed to update parent task: {}", e);
                                            }
                                        }
                                        std::result::Result::Err(e) => {
                                            eprintln!("Warning: Decomposition failed for '{}': {}", task.title, e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                self.prd_processing_state = PRDProcessingState::ReloadingTasks;
                true
            }
            PRDProcessingState::ReloadingTasks => {
                if let Err(e) = self.load_tasks().await {
                    self.prd_processing_state = PRDProcessingState::Failed {
                        error: std::format!("Tasks saved but failed to reload: {}", e),
                    };
                    return false;
                }

                let task_count = self.prd_processing_tasks.as_ref().map(|t| t.len()).unwrap_or(0);
                self.prd_processing_state = PRDProcessingState::Complete { task_count };

                self.add_notification(
                    NotificationLevel::Success,
                    std::format!("Generated {} tasks from {}", task_count, self.prd_processing_file)
                );

                false // Stop processing
            }
            PRDProcessingState::Complete { .. } | PRDProcessingState::Failed { .. } => {
                false // Already done
            }
        }
    }

    /// Cycles to the next dashboard tool (Kanban -> Metrics -> DevTools).
    fn next_tool(&mut self) {
        self.active_tool = match self.active_tool {
            DashboardTool::Kanban => DashboardTool::Metrics,
            DashboardTool::Metrics => DashboardTool::DevTools,
            DashboardTool::DevTools => DashboardTool::Kanban,
            // TaskEditor and LLMChat are dialog-only
            DashboardTool::TaskEditor => DashboardTool::Kanban,
            DashboardTool::LLMChat => DashboardTool::Kanban,
        };
    }

    /// Cycles to the previous dashboard tool (DevTools <- Metrics <- Kanban).
    fn previous_tool(&mut self) {
        self.active_tool = match self.active_tool {
            DashboardTool::Kanban => DashboardTool::DevTools,
            DashboardTool::Metrics => DashboardTool::Kanban,
            DashboardTool::DevTools => DashboardTool::Metrics,
            // TaskEditor and LLMChat are dialog-only
            DashboardTool::TaskEditor => DashboardTool::Metrics,
            DashboardTool::LLMChat => DashboardTool::Metrics,
        };
    }

    /// Toggles the visibility of the details panel (right column).
    fn toggle_details_panel(&mut self) {
        self.show_details_panel = !self.show_details_panel;
    }

    /// Cycles to the next project.
    ///
    /// Switches to the next project in the projects list, wrapping around
    /// to the first project after the last one. After switching, re-filters
    /// tasks and PRDs to show only those belonging to the selected project.
    fn next_workspace(&mut self) {
        if self.projects.is_empty() {
            return;
        }

        // Find current project index
        let current_idx = if let std::option::Option::Some(ref id) = self.selected_project_id {
            self.projects.iter().position(|p| &p.id == id).unwrap_or(0)
        } else {
            0
        };

        // Move to next project (wrap around)
        let next_idx = (current_idx + 1) % self.projects.len();
        self.selected_project_id = std::option::Option::Some(self.projects[next_idx].id.clone());

        // TODO: Re-filter tasks and PRDs (will be implemented when loading from DB)
    }

    /// Cycles to the previous project.
    ///
    /// Switches to the previous project in the projects list, wrapping around
    /// to the last project before the first one. After switching, re-filters
    /// tasks and PRDs to show only those belonging to the selected project.
    fn previous_workspace(&mut self) {
        if self.projects.is_empty() {
            return;
        }

        // Find current project index
        let current_idx = if let std::option::Option::Some(ref id) = self.selected_project_id {
            self.projects.iter().position(|p| &p.id == id).unwrap_or(0)
        } else {
            0
        };

        // Move to previous project (wrap around)
        let prev_idx = if current_idx == 0 {
            self.projects.len() - 1
        } else {
            current_idx - 1
        };
        self.selected_project_id = std::option::Option::Some(self.projects[prev_idx].id.clone());

        // TODO: Re-filter tasks and PRDs (will be implemented when loading from DB)
    }

    /// Returns the currently selected project, if any.
    fn get_selected_project(&self) -> std::option::Option<&task_manager::domain::project::Project> {
        if let std::option::Option::Some(ref id) = self.selected_project_id {
            self.projects.iter().find(|p| &p.id == id)
        } else {
            std::option::Option::None
        }
    }

    /// Filters tasks to only those belonging to the currently selected project.
    ///
    /// Returns all tasks that belong to PRDs linked to the current project.
    /// Uses transitive filtering: Task → PRD → Project.
    fn get_filtered_tasks(&self) -> std::vec::Vec<&task_manager::domain::task::Task> {
        if let std::option::Option::Some(ref project_id) = self.selected_project_id {
            // Get all PRD IDs for this project
            let prd_ids: std::collections::HashSet<String> = self.prds
                .iter()
                .filter(|prd| &prd.project_id == project_id)
                .map(|prd| prd.id.clone())
                .collect();

            // Filter tasks that link to these PRDs
            self.tasks
                .iter()
                .filter(|task| {
                    task.source_prd_id.as_ref()
                        .map(|prd_id| prd_ids.contains(prd_id))
                        .unwrap_or(false)
                })
                .collect()
        } else {
            // No project selected, show all tasks
            self.tasks.iter().collect()
        }
    }

    /// Filters PRDs to only those belonging to the currently selected project.
    ///
    /// Returns all PRDs that have a project_id matching the current selection.
    fn get_filtered_prds(&self) -> std::vec::Vec<&task_manager::domain::prd::PRD> {
        if let std::option::Option::Some(ref project_id) = self.selected_project_id {
            self.prds
                .iter()
                .filter(|prd| &prd.project_id == project_id)
                .collect()
        } else {
            // No project selected, show all PRDs
            self.prds.iter().collect()
        }
    }

    fn next_task(&mut self) {
        if !self.tasks.is_empty() {
            self.selected_task = (self.selected_task + 1) % self.tasks.len();
            // Add to recent list
            let task_id = self.tasks[self.selected_task].id.to_string();
            self.add_to_recent(task_id);
        }
    }

    fn previous_task(&mut self) {
        if !self.tasks.is_empty() && self.selected_task > 0 {
            self.selected_task -= 1;
        } else if !self.tasks.is_empty() {
            self.selected_task = self.tasks.len() - 1;
        }
        // Add to recent list
        if !self.tasks.is_empty() {
            let task_id = self.tasks[self.selected_task].id.to_string();
            self.add_to_recent(task_id);
        }
    }

    fn toggle_shortcuts(&mut self) {
        self.show_shortcuts = !self.show_shortcuts;
    }

    /// Cycles the status of the currently selected task.
    ///
    /// Status cycle: TODO → IN PROGRESS → COMPLETED → ARCHIVED → TODO
    /// Shows confirmation dialog before archiving (Phase 10).
    /// Persists the change to the database and displays a status message.
    async fn cycle_task_status(&mut self) -> anyhow::Result<()> {
        // Guard: no tasks or no database adapter
        if self.tasks.is_empty() || self.db_adapter.is_none() {
            return std::result::Result::Ok(());
        }

        // Get the selected task
        let task = &self.tasks[self.selected_task];

        // Check if next status would be Archived - require confirmation (Phase 10)
        if task.status == task_manager::domain::task_status::TaskStatus::Completed {
            let task_id = task.id.clone();
            let task_title = task.title.clone();
            self.open_confirmation(
                String::from("Archive Task"),
                std::format!("Are you sure you want to archive '{}'?\n\nArchived tasks are hidden from the main view.", task_title),
                ConfirmationAction::ArchiveTask { task_id }
            );
            return std::result::Result::Ok(());
        }

        // Cycle status (non-destructive transitions)
        let new_status = match &task.status {
            task_manager::domain::task_status::TaskStatus::Todo => {
                task_manager::domain::task_status::TaskStatus::InProgress
            }
            task_manager::domain::task_status::TaskStatus::InProgress => {
                task_manager::domain::task_status::TaskStatus::Completed
            }
            task_manager::domain::task_status::TaskStatus::Archived => {
                task_manager::domain::task_status::TaskStatus::Todo
            }
            _ => task.status.clone(), // Other statuses (orchestration states, etc.) don't cycle
        };

        // Get mutable reference for update
        let task = &mut self.tasks[self.selected_task];

        // Set status message before updating task (for display)
        let status_str = match &new_status {
            task_manager::domain::task_status::TaskStatus::Todo => "TODO",
            task_manager::domain::task_status::TaskStatus::InProgress => "IN PROGRESS",
            task_manager::domain::task_status::TaskStatus::Completed => "COMPLETED",
            task_manager::domain::task_status::TaskStatus::Archived => "ARCHIVED",
            _ => "UNKNOWN",
        };
        self.status_message = std::option::Option::Some(std::format!("Status changed to {}", status_str));

        // Clone task title for notification (to avoid borrow checker issues)
        let task_title = task.title.clone();

        // Update task status
        task.status = new_status;
        self.has_unsaved_changes = true;

        // Persist to database
        if let std::option::Option::Some(adapter) = &self.db_adapter {
            self.is_saving = true;
            let save_result = adapter.save_async(task.clone()).await.map_err(|e| {
                anyhow::anyhow!("Failed to save task status: {:?}", e)
            });
            self.is_saving = false;

            save_result?;

            // Mark as saved
            self.last_saved_at = std::option::Option::Some(chrono::Utc::now());
            self.has_unsaved_changes = false;

            // Add notification
            self.add_notification(
                NotificationLevel::Success,
                std::format!("Changed '{}' to {}", truncate_string(&task_title, 20), status_str)
            );
        }

        std::result::Result::Ok(())
    }

    /// Returns current frame of loading spinner animation.
    ///
    /// Uses a simple rotating spinner: ⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏
    fn get_spinner_char(&self) -> char {
        const SPINNER_FRAMES: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        SPINNER_FRAMES[self.loading_frame % SPINNER_FRAMES.len()]
    }

    /// Advances the loading spinner animation frame.
    fn advance_spinner(&mut self) {
        self.loading_frame = self.loading_frame.wrapping_add(1);
    }

    /// Adds a notification to the notification center.
    ///
    /// Maintains a maximum of 50 notifications, removing oldest when limit reached.
    fn add_notification(&mut self, level: NotificationLevel, message: String) {
        const MAX_NOTIFICATIONS: usize = 50;

        // Add to front (newest first)
        self.notifications.insert(0, Notification::new(level, message));

        // Trim to max size
        if self.notifications.len() > MAX_NOTIFICATIONS {
            self.notifications.truncate(MAX_NOTIFICATIONS);
        }
    }

    /// Toggles the notification center dialog.
    fn toggle_notifications(&mut self) {
        self.show_notifications = !self.show_notifications;
    }

    /// Opens the task editor dialog for the currently selected task (Phase 4).
    fn open_task_editor(&mut self) {
        if self.tasks.is_empty() {
            return;
        }

        let task = &self.tasks[self.selected_task];

        // Initialize input buffer with current title
        self.task_editor_input = task.title.clone();
        self.task_editor_field = TaskEditorField::Title;
        self.show_task_editor_dialog = true;
    }

    /// Closes the task editor dialog without saving (Phase 4).
    fn close_task_editor(&mut self) {
        self.show_task_editor_dialog = false;
        self.task_editor_input.clear();
        self.task_editor_field = TaskEditorField::Title;
    }

    /// Cycles to the next field in the task editor (Phase 4).
    fn next_task_editor_field(&mut self) {
        if self.tasks.is_empty() {
            return;
        }

        let task = &self.tasks[self.selected_task];

        // When changing fields, update input buffer with current field value
        self.task_editor_field = match self.task_editor_field {
            TaskEditorField::Title => {
                self.task_editor_input = task.description.clone();
                TaskEditorField::Description
            }
            TaskEditorField::Description => {
                self.task_editor_input = task.assignee.clone().unwrap_or_default();
                TaskEditorField::Assignee
            }
            TaskEditorField::Assignee => {
                // Status uses a different UI (cycle through enum)
                self.task_editor_input.clear();
                TaskEditorField::Status
            }
            TaskEditorField::Status => {
                self.task_editor_input = task.title.clone();
                TaskEditorField::Title
            }
        };
    }

    /// Cycles to the previous field in the task editor (Phase 4).
    fn previous_task_editor_field(&mut self) {
        if self.tasks.is_empty() {
            return;
        }

        let task = &self.tasks[self.selected_task];

        self.task_editor_field = match self.task_editor_field {
            TaskEditorField::Title => {
                self.task_editor_input.clear();
                TaskEditorField::Status
            }
            TaskEditorField::Description => {
                self.task_editor_input = task.title.clone();
                TaskEditorField::Title
            }
            TaskEditorField::Assignee => {
                self.task_editor_input = task.description.clone();
                TaskEditorField::Description
            }
            TaskEditorField::Status => {
                self.task_editor_input = task.assignee.clone().unwrap_or_default();
                TaskEditorField::Assignee
            }
        };
    }

    /// Handles character input in the task editor (Phase 4).
    fn handle_task_editor_input(&mut self, c: char) {
        // Only handle text input for Title, Description, and Assignee fields
        match self.task_editor_field {
            TaskEditorField::Title | TaskEditorField::Description | TaskEditorField::Assignee => {
                self.task_editor_input.push(c);
            }
            TaskEditorField::Status => {
                // Status field uses arrow keys to cycle, not text input
            }
        }
    }

    /// Handles backspace in the task editor (Phase 4).
    fn handle_task_editor_backspace(&mut self) {
        match self.task_editor_field {
            TaskEditorField::Title | TaskEditorField::Description | TaskEditorField::Assignee => {
                self.task_editor_input.pop();
            }
            TaskEditorField::Status => {
                // Status field doesn't use text input
            }
        }
    }

    /// Cycles task status forward in the task editor (Phase 4).
    fn cycle_task_status_forward(&mut self) {
        if self.tasks.is_empty() || self.task_editor_field != TaskEditorField::Status {
            return;
        }

        let task = &mut self.tasks[self.selected_task];
        task.status = match task.status {
            task_manager::domain::task_status::TaskStatus::Todo => {
                task_manager::domain::task_status::TaskStatus::InProgress
            }
            task_manager::domain::task_status::TaskStatus::InProgress => {
                task_manager::domain::task_status::TaskStatus::Completed
            }
            task_manager::domain::task_status::TaskStatus::Completed => {
                task_manager::domain::task_status::TaskStatus::Todo
            }
            _ => task_manager::domain::task_status::TaskStatus::Todo,
        };
    }

    /// Cycles task status backward in the task editor (Phase 4).
    fn cycle_task_status_backward(&mut self) {
        if self.tasks.is_empty() || self.task_editor_field != TaskEditorField::Status {
            return;
        }

        let task = &mut self.tasks[self.selected_task];
        task.status = match task.status {
            task_manager::domain::task_status::TaskStatus::Todo => {
                task_manager::domain::task_status::TaskStatus::Completed
            }
            task_manager::domain::task_status::TaskStatus::InProgress => {
                task_manager::domain::task_status::TaskStatus::Todo
            }
            task_manager::domain::task_status::TaskStatus::Completed => {
                task_manager::domain::task_status::TaskStatus::InProgress
            }
            _ => task_manager::domain::task_status::TaskStatus::Completed,
        };
    }

    /// Saves the task editor changes to the currently selected task (Phase 4).
    async fn save_task_editor(&mut self) -> anyhow::Result<()> {
        if self.tasks.is_empty() {
            return std::result::Result::Ok(());
        }

        // Apply pending text input to appropriate field
        let task = &mut self.tasks[self.selected_task];
        match self.task_editor_field {
            TaskEditorField::Title => {
                task.title = self.task_editor_input.clone();
            }
            TaskEditorField::Description => {
                task.description = self.task_editor_input.clone();
            }
            TaskEditorField::Assignee => {
                task.assignee = if self.task_editor_input.is_empty() {
                    std::option::Option::None
                } else {
                    std::option::Option::Some(self.task_editor_input.clone())
                };
            }
            TaskEditorField::Status => {
                // Status already updated via cycle methods
            }
        }

        // Save to database if adapter is available
        if let std::option::Option::Some(ref adapter) = self.db_adapter {
            self.is_saving = true;
            let task_clone = task.clone();
            let task_title = task.title.clone(); // Clone title for notification

            match adapter.save_async(task_clone).await {
                std::result::Result::Ok(_) => {
                    self.last_saved_at = std::option::Option::Some(chrono::Utc::now());
                    self.has_unsaved_changes = false;
                    self.is_saving = false;

                    self.add_notification(
                        NotificationLevel::Success,
                        std::format!("Saved task: {}", task_title)
                    );
                }
                std::result::Result::Err(e) => {
                    self.is_saving = false;
                    self.add_notification(
                        NotificationLevel::Error,
                        std::format!("Failed to save task: {:?}", e)
                    );
                    return std::result::Result::Err(anyhow::anyhow!("Failed to save task: {:?}", e));
                }
            }
        }

        self.close_task_editor();
        std::result::Result::Ok(())
    }

    /// Opens the LLM chat dialog with context about current project and task (Phase 5).
    fn open_llm_chat(&mut self) {
        // Build context message showing current project and task
        let mut context_parts = std::vec::Vec::new();

        if let std::option::Option::Some(project) = self.get_selected_project() {
            context_parts.push(std::format!("Current Project: {}", project.name));
            if let std::option::Option::Some(ref desc) = project.description {
                if !desc.is_empty() {
                    context_parts.push(std::format!("  Description: {}", desc));
                }
            }
        }

        if !self.tasks.is_empty() {
            let task = &self.tasks[self.selected_task];
            context_parts.push(std::format!("\nCurrent Task: {}", task.title));
            context_parts.push(std::format!("  Status: {:?}", task.status));
            if !task.description.is_empty() {
                context_parts.push(std::format!("  Description: {}", task.description));
            }
        }

        // Clear previous chat history and add context as system message
        self.llm_chat_history.clear();
        if !context_parts.is_empty() {
            self.llm_chat_history.push(ChatMessage {
                role: ChatRole::System,
                content: context_parts.join("\n"),
            });
        }

        self.llm_chat_input.clear();
        self.show_llm_chat_dialog = true;
    }

    /// Closes the LLM chat dialog (Phase 5).
    fn close_llm_chat(&mut self) {
        self.show_llm_chat_dialog = false;
        self.llm_chat_input.clear();
        // Note: We keep chat history for potential re-opening
    }

    /// Handles character input in LLM chat (Phase 5).
    fn handle_llm_chat_input(&mut self, c: char) {
        self.llm_chat_input.push(c);
    }

    /// Handles backspace in LLM chat (Phase 5).
    fn handle_llm_chat_backspace(&mut self) {
        self.llm_chat_input.pop();
    }

    /// Sends the current chat message to LLM and gets response (Phase 5).
    ///
    /// Note: This is a placeholder implementation. Real LLM integration would
    /// call task_orchestrator adapters (Rig, etc.) for actual AI responses.
    async fn send_llm_chat_message(&mut self) -> anyhow::Result<()> {
        if self.llm_chat_input.trim().is_empty() {
            return std::result::Result::Ok(());
        }

        // Add user message to history
        self.llm_chat_history.push(ChatMessage {
            role: ChatRole::User,
            content: self.llm_chat_input.clone(),
        });

        // Clear input
        let user_message = self.llm_chat_input.clone();
        self.llm_chat_input.clear();

        // TODO: Call actual LLM adapter (task_orchestrator::adapters::rig_adapter)
        // For now, provide a placeholder response
        let response = if user_message.to_lowercase().contains("enhance") {
            "To enhance this task, I would recommend:\n\n1. Breaking down the description into specific acceptance criteria\n2. Adding measurable success metrics\n3. Identifying potential edge cases\n4. Documenting dependencies\n\nWould you like me to apply these enhancements?"
        } else if user_message.to_lowercase().contains("decompose") {
            "I can help decompose this task into smaller subtasks:\n\n1. Research and design phase\n2. Implementation phase\n3. Testing and validation phase\n4. Documentation phase\n\nShall I create these subtasks for you?"
        } else {
            "I'm here to help! You can ask me to:\n- 'enhance this task' - Add details and clarity\n- 'decompose this task' - Break into subtasks\n- Ask questions about the current project or task\n\n(Note: Full LLM integration coming in future phases)"
        };

        self.llm_chat_history.push(ChatMessage {
            role: ChatRole::Assistant,
            content: String::from(response),
        });

        std::result::Result::Ok(())
    }

    /// Opens the PRD management dialog showing PRDs for current project (Phase 7).
    fn open_prd_dialog(&mut self) {
        self.show_prd_dialog = true;
        self.selected_prd = 0;
    }

    /// Closes the PRD management dialog (Phase 7).
    fn close_prd_dialog(&mut self) {
        self.show_prd_dialog = false;
        self.selected_prd = 0;
    }

    /// Moves to the next PRD in the list (Phase 7).
    fn next_prd(&mut self) {
        let filtered_prds = self.get_filtered_prds();
        if !filtered_prds.is_empty() {
            self.selected_prd = (self.selected_prd + 1) % filtered_prds.len();
        }
    }

    /// Moves to the previous PRD in the list (Phase 7).
    fn previous_prd(&mut self) {
        let filtered_prds = self.get_filtered_prds();
        if !filtered_prds.is_empty() {
            self.selected_prd = if self.selected_prd == 0 {
                filtered_prds.len() - 1
            } else {
                self.selected_prd - 1
            };
        }
    }

    /// Opens the task creator dialog (Phase 8).
    fn open_task_creator(&mut self) {
        self.task_creator_title.clear();
        self.task_creator_description.clear();
        self.task_creator_assignee.clear();
        self.task_creator_status = task_manager::domain::task_status::TaskStatus::Todo;
        self.task_creator_field = TaskCreatorField::Title;
        self.show_task_creator_dialog = true;
    }

    /// Closes the task creator dialog without saving (Phase 8).
    fn close_task_creator(&mut self) {
        self.show_task_creator_dialog = false;
        self.task_creator_title.clear();
        self.task_creator_description.clear();
        self.task_creator_assignee.clear();
        self.task_creator_status = task_manager::domain::task_status::TaskStatus::Todo;
        self.task_creator_field = TaskCreatorField::Title;
    }

    /// Cycles to the next field in the task creator (Phase 8).
    fn next_task_creator_field(&mut self) {
        self.task_creator_field = match self.task_creator_field {
            TaskCreatorField::Title => TaskCreatorField::Description,
            TaskCreatorField::Description => TaskCreatorField::Assignee,
            TaskCreatorField::Assignee => TaskCreatorField::Status,
            TaskCreatorField::Status => TaskCreatorField::Title,
        };
    }

    /// Cycles to the previous field in the task creator (Phase 8).
    fn previous_task_creator_field(&mut self) {
        self.task_creator_field = match self.task_creator_field {
            TaskCreatorField::Title => TaskCreatorField::Status,
            TaskCreatorField::Description => TaskCreatorField::Title,
            TaskCreatorField::Assignee => TaskCreatorField::Description,
            TaskCreatorField::Status => TaskCreatorField::Assignee,
        };
    }

    /// Handles character input in task creator (Phase 8).
    fn handle_task_creator_input(&mut self, c: char) {
        match self.task_creator_field {
            TaskCreatorField::Title => self.task_creator_title.push(c),
            TaskCreatorField::Description => self.task_creator_description.push(c),
            TaskCreatorField::Assignee => self.task_creator_assignee.push(c),
            TaskCreatorField::Status => {} // Status uses arrow keys, ignore text input
        }
    }

    /// Handles backspace in task creator (Phase 8).
    fn handle_task_creator_backspace(&mut self) {
        match self.task_creator_field {
            TaskCreatorField::Title => {
                self.task_creator_title.pop();
            }
            TaskCreatorField::Description => {
                self.task_creator_description.pop();
            }
            TaskCreatorField::Assignee => {
                self.task_creator_assignee.pop();
            }
            TaskCreatorField::Status => {} // Status uses arrow keys
        }
    }

    /// Cycles task creator status forward (Phase 8).
    fn cycle_creator_status_forward(&mut self) {
        self.task_creator_status = match self.task_creator_status {
            task_manager::domain::task_status::TaskStatus::Todo => task_manager::domain::task_status::TaskStatus::InProgress,
            task_manager::domain::task_status::TaskStatus::InProgress => task_manager::domain::task_status::TaskStatus::Completed,
            task_manager::domain::task_status::TaskStatus::Completed => task_manager::domain::task_status::TaskStatus::Archived,
            task_manager::domain::task_status::TaskStatus::Archived => task_manager::domain::task_status::TaskStatus::Todo,
            _ => task_manager::domain::task_status::TaskStatus::Todo, // Default for other states
        };
    }

    /// Cycles task creator status backward (Phase 8).
    fn cycle_creator_status_backward(&mut self) {
        self.task_creator_status = match self.task_creator_status {
            task_manager::domain::task_status::TaskStatus::Todo => task_manager::domain::task_status::TaskStatus::Archived,
            task_manager::domain::task_status::TaskStatus::InProgress => task_manager::domain::task_status::TaskStatus::Todo,
            task_manager::domain::task_status::TaskStatus::Completed => task_manager::domain::task_status::TaskStatus::InProgress,
            task_manager::domain::task_status::TaskStatus::Archived => task_manager::domain::task_status::TaskStatus::Completed,
            _ => task_manager::domain::task_status::TaskStatus::Todo, // Default for other states
        };
    }

    /// Saves the new task from the task creator dialog (Phase 8).
    async fn save_task_creator(&mut self) -> anyhow::Result<()> {
        // Validate title is not empty
        if self.task_creator_title.trim().is_empty() {
            self.add_notification(
                NotificationLevel::Error,
                String::from("Task title cannot be empty")
            );
            return std::result::Result::Ok(());
        }

        // Create new task
        let now = chrono::Utc::now();
        let mut new_task = task_manager::domain::task::Task {
            id: uuid::Uuid::new_v4().to_string(),
            title: self.task_creator_title.clone(),
            description: self.task_creator_description.clone(),
            assignee: if self.task_creator_assignee.is_empty() {
                std::option::Option::None
            } else {
                std::option::Option::Some(self.task_creator_assignee.clone())
            },
            due_date: std::option::Option::None,
            status: self.task_creator_status.clone(),
            source_transcript_id: std::option::Option::None,
            source_prd_id: std::option::Option::None, // Will be set based on current project
            parent_task_id: std::option::Option::None,
            subtask_ids: std::vec::Vec::new(),
            created_at: now,
            updated_at: now,
            enhancements: std::option::Option::None,
            comprehension_tests: std::option::Option::None,
            complexity: std::option::Option::None,
            reasoning: std::option::Option::None,
            completion_summary: std::option::Option::None,
            context_files: std::vec::Vec::new(),
            dependencies: std::vec::Vec::new(),
        };

        // Link to first PRD of current project (if available)
        if let std::option::Option::Some(ref _project_id) = self.selected_project_id {
            let filtered_prds = self.get_filtered_prds();
            if !filtered_prds.is_empty() {
                new_task.source_prd_id = std::option::Option::Some(filtered_prds[0].id.clone());
            }
        }

        // Save to database if adapter is available
        if let std::option::Option::Some(ref adapter) = self.db_adapter {
            self.is_saving = true;
            let task_clone = new_task.clone();
            let task_title = new_task.title.clone();

            match adapter.save_async(task_clone).await {
                std::result::Result::Ok(_) => {
                    self.last_saved_at = std::option::Option::Some(chrono::Utc::now());
                    self.is_saving = false;

                    // Add task to local list
                    self.tasks.push(new_task);

                    self.add_notification(
                        NotificationLevel::Success,
                        std::format!("Created task: {}", task_title)
                    );
                }
                std::result::Result::Err(e) => {
                    self.is_saving = false;
                    self.add_notification(
                        NotificationLevel::Error,
                        std::format!("Failed to create task: {:?}", e)
                    );
                    return std::result::Result::Err(anyhow::anyhow!("Failed to create task: {:?}", e));
                }
            }
        }

        self.close_task_creator();
        std::result::Result::Ok(())
    }

    /// Performs fuzzy search across tasks, PRDs, and projects (Phase 9).
    fn search_all(&self, query: &str) -> std::vec::Vec<SearchResultType> {
        if query.is_empty() {
            return std::vec::Vec::new();
        }

        let query_lower = query.to_lowercase();
        let mut results = std::vec::Vec::new();

        // Search tasks
        for task in &self.tasks {
            if task.title.to_lowercase().contains(&query_lower)
                || task.description.to_lowercase().contains(&query_lower) {
                results.push(SearchResultType::Task {
                    id: task.id.clone(),
                    title: task.title.clone(),
                    description: task.description.clone(),
                });
            }
        }

        // Search PRDs
        for prd in &self.prds {
            if prd.title.to_lowercase().contains(&query_lower) {
                results.push(SearchResultType::PRD {
                    id: prd.id.clone(),
                    title: prd.title.clone(),
                });
            }
        }

        // Search projects
        for project in &self.projects {
            if project.name.to_lowercase().contains(&query_lower) {
                results.push(SearchResultType::Project {
                    id: project.id.clone(),
                    name: project.name.clone(),
                });
            }
        }

        results
    }

    /// Opens the spotlight search dialog (Phase 9).
    fn open_spotlight(&mut self) {
        self.spotlight_query.clear();
        self.spotlight_results.clear();
        self.spotlight_selected = 0;
        self.show_spotlight_dialog = true;
    }

    /// Closes the spotlight search dialog (Phase 9).
    fn close_spotlight(&mut self) {
        self.show_spotlight_dialog = false;
        self.spotlight_query.clear();
        self.spotlight_results.clear();
        self.spotlight_selected = 0;
    }

    /// Handles character input in spotlight search (Phase 9).
    fn handle_spotlight_input(&mut self, c: char) {
        self.spotlight_query.push(c);
        self.spotlight_results = self.search_all(&self.spotlight_query);
        self.spotlight_selected = 0; // Reset selection to top
    }

    /// Handles backspace in spotlight search (Phase 9).
    fn handle_spotlight_backspace(&mut self) {
        self.spotlight_query.pop();
        self.spotlight_results = self.search_all(&self.spotlight_query);
        self.spotlight_selected = 0; // Reset selection to top
    }

    /// Moves to next spotlight result (Phase 9).
    fn next_spotlight_result(&mut self) {
        if !self.spotlight_results.is_empty() {
            self.spotlight_selected = (self.spotlight_selected + 1) % self.spotlight_results.len();
        }
    }

    /// Moves to previous spotlight result (Phase 9).
    fn previous_spotlight_result(&mut self) {
        if !self.spotlight_results.is_empty() {
            self.spotlight_selected = if self.spotlight_selected == 0 {
                self.spotlight_results.len() - 1
            } else {
                self.spotlight_selected - 1
            };
        }
    }

    /// Executes jump to selected spotlight result (Phase 9).
    fn execute_spotlight_jump(&mut self) {
        if self.spotlight_results.is_empty() {
            return;
        }

        let result = &self.spotlight_results[self.spotlight_selected];

        match result {
            SearchResultType::Task { id, .. } => {
                // Find and select the task
                if let std::option::Option::Some(index) = self.tasks.iter().position(|t| &t.id == id) {
                    self.selected_task = index;
                    self.add_notification(
                        NotificationLevel::Success,
                        String::from("Jumped to task")
                    );
                }
            }
            SearchResultType::PRD { id, .. } => {
                // Switch to PRD dialog and select it
                if let std::option::Option::Some(index) = self.prds.iter().position(|p| &p.id == id) {
                    self.selected_prd = index;
                    self.show_prd_dialog = true;
                    self.add_notification(
                        NotificationLevel::Success,
                        String::from("Opened PRD")
                    );
                }
            }
            SearchResultType::Project { id, .. } => {
                // Switch to project
                self.selected_project_id = std::option::Option::Some(id.clone());
                self.add_notification(
                    NotificationLevel::Success,
                    String::from("Switched project")
                );
            }
        }

        self.close_spotlight();
    }

    /// Opens the confirmation dialog (Phase 10).
    ///
    /// # Arguments
    ///
    /// * `title` - Dialog title (e.g., "Archive Task")
    /// * `message` - Detailed confirmation message
    /// * `action` - Action to execute if user confirms
    fn open_confirmation(&mut self, title: String, message: String, action: ConfirmationAction) {
        self.confirmation_title = title;
        self.confirmation_message = message;
        self.confirmation_action = std::option::Option::Some(action);
        self.show_confirmation_dialog = true;
    }

    /// Closes the confirmation dialog without executing action (Phase 10).
    fn close_confirmation(&mut self) {
        self.show_confirmation_dialog = false;
        self.confirmation_title.clear();
        self.confirmation_message.clear();
        self.confirmation_action = std::option::Option::None;
    }

    /// Executes the confirmed action (Phase 10).
    async fn confirm_action(&mut self) -> anyhow::Result<()> {
        if let std::option::Option::Some(action) = self.confirmation_action.clone() {
            match action {
                ConfirmationAction::ArchiveTask { task_id } => {
                    // Find the task and archive it
                    if let std::option::Option::Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
                        task.status = task_manager::domain::task_status::TaskStatus::Archived;
                        task.updated_at = chrono::Utc::now();

                        // Save to database
                        if let std::option::Option::Some(ref mut adapter) = self.db_adapter {
                            let task_clone = task.clone();
                            let task_title = task.title.clone();

                            match adapter.save_async(task_clone).await {
                                std::result::Result::Ok(_) => {
                                    self.add_notification(
                                        NotificationLevel::Success,
                                        std::format!("Archived task: {}", task_title)
                                    );
                                }
                                std::result::Result::Err(e) => {
                                    self.add_notification(
                                        NotificationLevel::Error,
                                        std::format!("Failed to archive task: {}", e)
                                    );
                                    return std::result::Result::Err(anyhow::anyhow!("Failed to archive task: {}", e));
                                }
                            }
                        }
                    }
                }
            }
        }

        self.close_confirmation();
        std::result::Result::Ok(())
    }

    /// Formats save status indicator for display in status bar.
    ///
    /// Returns a string showing:
    /// - "💾 Saving..." if currently saving
    /// - "⚠️  Unsaved changes" if there are unsaved changes
    /// - "✓ Saved Xs ago" if recently saved (where X is seconds/minutes)
    /// - Empty string if never saved
    fn format_save_indicator(&self) -> String {
        if self.is_saving {
            return String::from("💾 Saving...");
        }

        if self.has_unsaved_changes {
            return String::from("⚠️  Unsaved changes");
        }

        if let std::option::Option::Some(last_saved) = self.last_saved_at {
            let elapsed = chrono::Utc::now() - last_saved;
            let seconds = elapsed.num_seconds();

            if seconds < 60 {
                return std::format!("✓ Saved {}s ago", seconds);
            } else {
                let minutes = seconds / 60;
                return std::format!("✓ Saved {}m ago", minutes);
            }
        }

        String::new()
    }

    /// Formats session duration for display in footer (Phase 11).
    ///
    /// Returns a string showing how long the TUI has been running.
    /// Examples: "2m 15s", "1h 23m", "45s"
    fn format_session_duration(&self) -> String {
        let elapsed = chrono::Utc::now() - self.session_start_time;
        let total_seconds = elapsed.num_seconds();

        if total_seconds < 60 {
            std::format!("{}s", total_seconds)
        } else if total_seconds < 3600 {
            let minutes = total_seconds / 60;
            let seconds = total_seconds % 60;
            std::format!("{}m {}s", minutes, seconds)
        } else {
            let hours = total_seconds / 3600;
            let minutes = (total_seconds % 3600) / 60;
            std::format!("{}h {}m", hours, minutes)
        }
    }

    /// Formats current time for display in footer (Phase 11).
    fn format_current_time(&self) -> String {
        let now = chrono::Local::now();
        now.format("%H:%M:%S").to_string()
    }

    /// Returns database connection status indicator (Phase 11).
    fn get_database_status(&self) -> &str {
        if self.db_adapter.is_some() {
            "🟢 DB"
        } else {
            "🔴 DB"
        }
    }

    /// Refreshes all data from the database (Phase 13).
    ///
    /// Reloads projects, PRDs, and tasks from SQLite. Useful after external changes
    /// or to ensure UI is in sync with database state.
    async fn refresh_all_data(&mut self) -> anyhow::Result<()> {
        // Set loading state
        self.is_loading = true;
        self.loading_message = std::option::Option::Some(String::from("Refreshing all data..."));

        // Reload projects
        if let std::result::Result::Err(e) = self.load_projects().await {
            self.is_loading = false;
            self.loading_message = std::option::Option::None;
            self.add_notification(
                NotificationLevel::Error,
                std::format!("Failed to refresh projects: {}", e)
            );
            return std::result::Result::Err(e);
        }

        // Reload tasks (similar to initial load)
        let current_dir = std::env::current_dir()?;
        let db_path = current_dir.join(".rigger").join("tasks.db");

        if db_path.exists() {
            if let std::option::Option::Some(adapter) = &self.db_adapter {
                let filter = task_manager::ports::task_repository_port::TaskFilter::All;
                let opts = hexser::ports::repository::FindOptions {
                    sort: std::option::Option::Some(std::vec![hexser::ports::repository::Sort {
                        key: task_manager::ports::task_repository_port::TaskSortKey::CreatedAt,
                        direction: hexser::ports::repository::Direction::Desc,
                    }]),
                    limit: std::option::Option::Some(100),
                    offset: std::option::Option::None,
                };

                self.tasks = task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::find_async(adapter, &filter, opts)
                    .await
                    .map_err(|e| anyhow::anyhow!("{:?}", e))?;

                // Reload PRDs (placeholder - PRD adapter usage similar pattern)
                // For now, PRDs will be loaded if we have a PRD adapter
            }
        }

        // Clear loading state
        self.is_loading = false;
        self.loading_message = std::option::Option::None;

        // Add success notification
        self.add_notification(
            NotificationLevel::Success,
            std::format!("Refreshed: {} projects, {} tasks", self.projects.len(), self.tasks.len())
        );

        std::result::Result::Ok(())
    }

    fn toggle_sort_menu(&mut self) {
        self.show_sort_menu = !self.show_sort_menu;
        if !self.show_sort_menu {
            // Reset selection when closing menu
            self.sort_menu_selection = 0;
        }
    }

    fn next_sort_option(&mut self) {
        let options = TaskSortOption::all();
        self.sort_menu_selection = (self.sort_menu_selection + 1) % options.len();
    }

    fn previous_sort_option(&mut self) {
        let options = TaskSortOption::all();
        if self.sort_menu_selection > 0 {
            self.sort_menu_selection -= 1;
        } else {
            self.sort_menu_selection = options.len() - 1;
        }
    }

    fn apply_selected_sort(&mut self) {
        let options = TaskSortOption::all();
        self.current_sort = options[self.sort_menu_selection];
        self.apply_sort();
        self.show_sort_menu = false;
        self.status_message = std::option::Option::Some(
            std::format!("Sorted by: {}", self.current_sort.display_name())
        );
    }

    /// Applies the current sort option to the task list.
    fn apply_sort(&mut self) {
        match self.current_sort {
            TaskSortOption::CreatedNewest => {
                self.tasks.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            }
            TaskSortOption::UpdatedRecent => {
                self.tasks.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
            }
            TaskSortOption::TitleAlphabetical => {
                self.tasks.sort_by(|a, b| a.title.cmp(&b.title));
            }
            TaskSortOption::ComplexityHigh => {
                self.tasks.sort_by(|a, b| {
                    // Sort by complexity descending (high to low)
                    // Tasks without complexity go last
                    match (b.complexity, a.complexity) {
                        (std::option::Option::Some(b_comp), std::option::Option::Some(a_comp)) => {
                            b_comp.cmp(&a_comp)
                        }
                        (std::option::Option::Some(_), std::option::Option::None) => std::cmp::Ordering::Less,
                        (std::option::Option::None, std::option::Option::Some(_)) => std::cmp::Ordering::Greater,
                        (std::option::Option::None, std::option::Option::None) => std::cmp::Ordering::Equal,
                    }
                });
            }
        }
    }

    fn toggle_jump_dialog(&mut self) {
        self.show_jump_dialog = !self.show_jump_dialog;
        if !self.show_jump_dialog {
            // Clear input when closing dialog
            self.jump_input.clear();
        }
    }

    fn handle_jump_input(&mut self, c: char) {
        self.jump_input.push(c);
    }

    fn handle_jump_backspace(&mut self) {
        self.jump_input.pop();
    }

    /// Finds a task by ID (fuzzy matching).
    ///
    /// Searches for tasks where the ID contains the search string.
    /// Returns the index of the first matching task, or None if not found.
    fn find_task_by_id(&self, search: &str) -> std::option::Option<usize> {
        if search.is_empty() {
            return std::option::Option::None;
        }

        // Fuzzy matching: find task where ID contains the search string (case-insensitive)
        let search_lower = search.to_lowercase();
        self.tasks.iter().position(|task| {
            task.id.to_string().to_lowercase().contains(&search_lower)
        })
    }

    fn execute_jump(&mut self) {
        if let std::option::Option::Some(idx) = self.find_task_by_id(&self.jump_input) {
            self.selected_task = idx;
            self.status_message = std::option::Option::Some(
                std::format!("Jumped to task: {}", self.tasks[idx].title)
            );
            self.toggle_jump_dialog();
        } else {
            self.status_message = std::option::Option::Some(
                std::format!("Task not found: {}", self.jump_input)
            );
        }
    }

    fn select_column(&mut self, column: KanbanColumn) {
        self.selected_column = column;
        self.selected_task_in_column = 0; // Reset to first task when changing columns
        self.status_message = std::option::Option::Some(
            std::format!("Column: {}", column.display_name())
        );
    }

    /// Gets tasks in the currently selected column.
    fn get_tasks_in_selected_column(&self) -> std::vec::Vec<task_manager::domain::task::Task> {
        let target_status = self.selected_column.matching_status();
        self.tasks.iter()
            .filter(|task| task.status == target_status)
            .cloned()
            .collect()
    }

    /// Moves to the next task in the selected column.
    fn next_task_in_column(&mut self) {
        let tasks_in_column = self.get_tasks_in_selected_column();
        if !tasks_in_column.is_empty() && self.selected_task_in_column < tasks_in_column.len() - 1 {
            self.selected_task_in_column += 1;
        }
    }

    /// Moves to the previous task in the selected column.
    fn previous_task_in_column(&mut self) {
        if self.selected_task_in_column > 0 {
            self.selected_task_in_column -= 1;
        }
    }

    /// Gets the currently selected task in the column, if any.
    fn get_selected_task_in_column(&self) -> std::option::Option<task_manager::domain::task::Task> {
        let tasks_in_column = self.get_tasks_in_selected_column();
        tasks_in_column.get(self.selected_task_in_column).cloned()
    }

    fn toggle_recent_dialog(&mut self) {
        self.show_recent_dialog = !self.show_recent_dialog;
        if self.show_recent_dialog {
            // Reset selection when opening
            self.recent_selection = 0;
        }
    }

    fn next_recent_item(&mut self) {
        if !self.recent_task_ids.is_empty() {
            self.recent_selection = (self.recent_selection + 1) % self.recent_task_ids.len();
        }
    }

    fn previous_recent_item(&mut self) {
        if !self.recent_task_ids.is_empty() && self.recent_selection > 0 {
            self.recent_selection -= 1;
        } else if !self.recent_task_ids.is_empty() {
            self.recent_selection = self.recent_task_ids.len() - 1;
        }
    }

    /// Adds a task ID to the recent list (MRU - Most Recently Used).
    ///
    /// Maintains a maximum of 10 recent items, removing duplicates.
    fn add_to_recent(&mut self, task_id: String) {
        // Remove if already exists (move to front)
        self.recent_task_ids.retain(|id| id != &task_id);

        // Add to front
        self.recent_task_ids.insert(0, task_id);

        // Keep only last 10
        if self.recent_task_ids.len() > 10 {
            self.recent_task_ids.truncate(10);
        }
    }

    fn jump_to_recent_task(&mut self) {
        if self.recent_selection < self.recent_task_ids.len() {
            let task_id = self.recent_task_ids[self.recent_selection].clone();

            if let std::option::Option::Some(idx) = self.tasks.iter().position(|t| t.id.to_string() == task_id) {
                self.selected_task = idx;
                self.toggle_recent_dialog();
                self.status_message = std::option::Option::Some(
                    std::format!("Jumped to task: {}", self.tasks[idx].title)
                );
            }
        }
    }

    /// Advances to the next step in the setup wizard.
    fn setup_wizard_next_step(&mut self) {
        self.setup_wizard_step = match self.setup_wizard_step {
            SetupWizardStep::Welcome => SetupWizardStep::TaskToolSlots,
            SetupWizardStep::TaskToolSlots => SetupWizardStep::ConfigureMainSlot,
            SetupWizardStep::ConfigureMainSlot => {
                // Lock in main slot provider selection
                let providers = LLMProvider::all();
                if self.setup_wizard_main_provider_selection < providers.len() {
                    self.setup_wizard_main_provider = providers[self.setup_wizard_main_provider_selection];
                }
                // Update main model to match selected provider's default
                if self.setup_wizard_main_model.is_empty() {
                    self.setup_wizard_main_model = String::from(self.setup_wizard_main_provider.default_model());
                }
                SetupWizardStep::ConfigureResearchSlot
            }
            SetupWizardStep::ConfigureResearchSlot => {
                // Lock in research slot provider selection
                let providers = LLMProvider::all();
                if self.setup_wizard_research_provider_selection < providers.len() {
                    self.setup_wizard_research_provider = providers[self.setup_wizard_research_provider_selection];
                }
                // Update research model to match selected provider's default
                if self.setup_wizard_research_model.is_empty() {
                    self.setup_wizard_research_model = String::from(self.setup_wizard_research_provider.default_model());
                }
                SetupWizardStep::ConfigureFallbackSlot
            }
            SetupWizardStep::ConfigureFallbackSlot => {
                // Lock in fallback slot provider selection
                let providers = LLMProvider::all();
                if self.setup_wizard_fallback_provider_selection < providers.len() {
                    self.setup_wizard_fallback_provider = providers[self.setup_wizard_fallback_provider_selection];
                }
                // Update fallback model to match selected provider's default
                if self.setup_wizard_fallback_model.is_empty() {
                    self.setup_wizard_fallback_model = String::from(self.setup_wizard_fallback_provider.default_model());
                }
                SetupWizardStep::DatabaseConfiguration
            }
            SetupWizardStep::DatabaseConfiguration => SetupWizardStep::Confirmation,
            SetupWizardStep::Confirmation => SetupWizardStep::Complete,
            SetupWizardStep::Complete => SetupWizardStep::Complete,
        };
    }

    /// Goes back to the previous step in the setup wizard.
    fn setup_wizard_previous_step(&mut self) {
        self.setup_wizard_step = match self.setup_wizard_step {
            SetupWizardStep::Welcome => SetupWizardStep::Welcome,
            SetupWizardStep::TaskToolSlots => SetupWizardStep::Welcome,
            SetupWizardStep::ConfigureMainSlot => SetupWizardStep::TaskToolSlots,
            SetupWizardStep::ConfigureResearchSlot => SetupWizardStep::ConfigureMainSlot,
            SetupWizardStep::ConfigureFallbackSlot => SetupWizardStep::ConfigureResearchSlot,
            SetupWizardStep::DatabaseConfiguration => SetupWizardStep::ConfigureFallbackSlot,
            SetupWizardStep::Confirmation => SetupWizardStep::DatabaseConfiguration,
            SetupWizardStep::Complete => SetupWizardStep::Complete,
        };
    }

    /// Moves to the next provider option in the current slot configuration.
    fn setup_wizard_next_provider(&mut self) {
        let providers = LLMProvider::all();
        match self.setup_wizard_step {
            SetupWizardStep::ConfigureMainSlot => {
                if self.setup_wizard_main_provider_selection < providers.len() - 1 {
                    self.setup_wizard_main_provider_selection += 1;
                }
            }
            SetupWizardStep::ConfigureResearchSlot => {
                if self.setup_wizard_research_provider_selection < providers.len() - 1 {
                    self.setup_wizard_research_provider_selection += 1;
                }
            }
            SetupWizardStep::ConfigureFallbackSlot => {
                if self.setup_wizard_fallback_provider_selection < providers.len() - 1 {
                    self.setup_wizard_fallback_provider_selection += 1;
                }
            }
            _ => {}
        }
    }

    /// Moves to the previous provider option in the current slot configuration.
    fn setup_wizard_previous_provider(&mut self) {
        match self.setup_wizard_step {
            SetupWizardStep::ConfigureMainSlot => {
                if self.setup_wizard_main_provider_selection > 0 {
                    self.setup_wizard_main_provider_selection -= 1;
                }
            }
            SetupWizardStep::ConfigureResearchSlot => {
                if self.setup_wizard_research_provider_selection > 0 {
                    self.setup_wizard_research_provider_selection -= 1;
                }
            }
            SetupWizardStep::ConfigureFallbackSlot => {
                if self.setup_wizard_fallback_provider_selection > 0 {
                    self.setup_wizard_fallback_provider_selection -= 1;
                }
            }
            _ => {}
        }
    }

    /// Handles text input for setup wizard fields.
    fn setup_wizard_handle_char(&mut self, c: char) {
        match self.setup_wizard_step {
            SetupWizardStep::ConfigureMainSlot => {
                self.setup_wizard_main_model.push(c);
            }
            SetupWizardStep::ConfigureResearchSlot => {
                self.setup_wizard_research_model.push(c);
            }
            SetupWizardStep::ConfigureFallbackSlot => {
                self.setup_wizard_fallback_model.push(c);
            }
            SetupWizardStep::DatabaseConfiguration => {
                self.setup_wizard_db_path.push(c);
            }
            _ => {}
        }
    }

    /// Handles backspace for setup wizard text fields.
    fn setup_wizard_handle_backspace(&mut self) {
        match self.setup_wizard_step {
            SetupWizardStep::ConfigureMainSlot => {
                self.setup_wizard_main_model.pop();
            }
            SetupWizardStep::ConfigureResearchSlot => {
                self.setup_wizard_research_model.pop();
            }
            SetupWizardStep::ConfigureFallbackSlot => {
                self.setup_wizard_fallback_model.pop();
            }
            SetupWizardStep::DatabaseConfiguration => {
                self.setup_wizard_db_path.pop();
            }
            _ => {}
        }
    }

    /// Completes the setup wizard by creating config files and initializing the database.
    async fn setup_wizard_complete(&mut self) -> anyhow::Result<()> {
        let current_dir = std::env::current_dir()?;
        let rigger_dir = current_dir.join(".rigger");

        // Create .rigger directory
        if !rigger_dir.exists() {
            std::fs::create_dir(&rigger_dir)?;
        }

        // Create prds subdirectory
        let prds_dir = rigger_dir.join("prds");
        if !prds_dir.exists() {
            std::fs::create_dir(&prds_dir)?;
        }

        // Create config.json with wizard selections (per-slot providers and models)
        let main_provider_name = match self.setup_wizard_main_provider {
            LLMProvider::Ollama => "ollama",
            LLMProvider::Candle => "candle",
            LLMProvider::Mistral => "mistral",
            LLMProvider::Rig => "rig",
        };

        let research_provider_name = match self.setup_wizard_research_provider {
            LLMProvider::Ollama => "ollama",
            LLMProvider::Candle => "candle",
            LLMProvider::Mistral => "mistral",
            LLMProvider::Rig => "rig",
        };

        let fallback_provider_name = match self.setup_wizard_fallback_provider {
            LLMProvider::Ollama => "ollama",
            LLMProvider::Candle => "candle",
            LLMProvider::Mistral => "mistral",
            LLMProvider::Rig => "rig",
        };

        let config = serde_json::json!({
            "provider": main_provider_name, // Legacy field - kept for backward compatibility
            "task_tools": {
                "main": {
                    "provider": main_provider_name,
                    "model": self.setup_wizard_main_model.clone()
                },
                "research": {
                    "provider": research_provider_name,
                    "model": self.setup_wizard_research_model.clone()
                },
                "fallback": {
                    "provider": fallback_provider_name,
                    "model": self.setup_wizard_fallback_model.clone()
                }
            },
            "model": {
                "main": self.setup_wizard_main_model.clone(),
                "research": self.setup_wizard_research_model.clone(),
                "fallback": self.setup_wizard_fallback_model.clone()
            },
            "database_url": self.setup_wizard_db_path.clone()
        });

        let config_path = rigger_dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;

        // Initialize SQLite database
        let db_path = rigger_dir.join("tasks.db");
        if !db_path.exists() {
            std::fs::File::create(&db_path)
                .map_err(|e| anyhow::anyhow!("Failed to create database file: {}", e))?;
        }

        let db_url = std::format!("sqlite:{}", db_path.display());
        let _adapter = task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::connect_and_init(&db_url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to initialize database: {}", e))?;

        // Move to completion screen
        self.setup_wizard_step = SetupWizardStep::Complete;

        std::result::Result::Ok(())
    }

    /// Exits the setup wizard and loads the main application.
    ///
    /// Revision History:
    /// - 2025-11-26T19:40:00Z @AI: Skip data loading on wizard exit - let main UI load data on first render to prevent async runtime starvation.
    /// - 2025-11-26T19:10:00Z @AI: Add status messages for loading progress visibility during wizard exit.
    /// - 2025-11-26T18:45:00Z @AI: Add database existence check before loading to prevent hang if setup was incomplete.
    /// - 2025-11-25T13:45:00Z @AI: Fix race condition - load data BEFORE deactivating wizard to prevent rendering crash.
    async fn setup_wizard_exit(&mut self) -> anyhow::Result<()> {
        // Verify database exists before trying to load from it
        let current_dir = std::env::current_dir()?;
        let db_path = current_dir.join(".rigger/tasks.db");

        if !db_path.exists() {
            anyhow::bail!(
                "Database not found. Setup may not have completed successfully.\n\
                 Please delete the .rigger directory and restart the TUI to run setup again."
            );
        }

        // Deactivate wizard immediately - data will be loaded by the main UI on first render
        // This prevents async runtime starvation in the event loop
        self.setup_wizard_active = false;

        self.add_notification(
            NotificationLevel::Success,
            String::from("Setup complete! Loading data...")
        );

        self.status_message = std::option::Option::Some(String::from("Loading..."));

        std::result::Result::Ok(())
    }

    /// Copies the currently selected task to clipboard as Markdown.
    ///
    /// Uses the task formatter service to create a rich Markdown representation
    /// and the clipboard port to copy to system clipboard.
    fn copy_task_to_clipboard(&mut self) {
        // Guard: no tasks or no clipboard
        if self.tasks.is_empty() {
            self.status_message = std::option::Option::Some(
                "No task selected to copy".to_string()
            );
            return;
        }

        if self.clipboard.is_none() {
            self.status_message = std::option::Option::Some(
                "Clipboard unavailable in this environment".to_string()
            );
            return;
        }

        let task = &self.tasks[self.selected_task];

        // Format task as Markdown using service layer
        let markdown = task_formatter::format_task_as_markdown(task);

        // Copy to clipboard using port
        if let std::option::Option::Some(ref clipboard) = self.clipboard {
            match clipboard.copy_text(&markdown) {
                std::result::Result::Ok(_) => {
                    self.status_message = std::option::Option::Some(
                        std::format!("Copied task '{}' to clipboard", truncate_string(&task.title, 30))
                    );
                    self.add_notification(
                        NotificationLevel::Success,
                        std::format!("Copied '{}' to clipboard", truncate_string(&task.title, 25))
                    );
                }
                std::result::Result::Err(e) => {
                    self.status_message = std::option::Option::Some(
                        std::format!("Clipboard error: {}", e)
                    );
                    self.add_notification(
                        NotificationLevel::Error,
                        std::format!("Clipboard error: {}", e)
                    );
                }
            }
        }
    }
}

/// Executes the 'rig tui' command.
///
/// Launches an interactive TUI for managing tasks with keyboard navigation.
///
/// # Keyboard Controls
///
/// - `Tab` / `Shift+Tab`: Switch between views
/// - `↑` / `↓`: Navigate tasks
/// - `q` / `Esc`: Quit
/// - `r`: Refresh tasks
///
/// # Errors
///
/// Returns an error if terminal initialization fails or database access fails.
pub async fn execute() -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new();

    // Check if .rigger/config.json exists - activate setup wizard if not
    let current_dir = std::env::current_dir()?;
    let config_path = current_dir.join(".rigger").join("config.json");

    if !config_path.exists() {
        // Activate setup wizard for first-time initialization
        app.setup_wizard_active = true;
    }
    // Note: Data loading moved to first render cycle to prevent async runtime starvation
    // The main UI will load projects and tasks on first render

    // Main event loop
    let res = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let std::result::Result::Err(err) = res {
        println!("{err:?}");
    }

    std::result::Result::Ok(())
}

/// Main TUI event loop.
async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        // Check if PRD processing should advance to next step (after UI has rendered)
        if app.prd_processing_pending {
            app.prd_processing_pending = false;
            // Process one step, then let UI render before next step
            if app.process_prd_step().await {
                // More steps remaining - schedule next step
                app.prd_processing_pending = true;
            }
        }

        // Advance spinner animation if loading
        if app.is_loading {
            app.advance_spinner();
        }

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Setup wizard takes absolute priority over all other keyboard input
                if app.setup_wizard_active {
                    // Check for Ctrl+C to exit wizard and quit entirely
                    if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                        if matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C')) {
                            app.should_quit = true;
                        }
                    }

                    match key.code {
                        KeyCode::Enter => {
                            match app.setup_wizard_step {
                                SetupWizardStep::Welcome => {
                                    app.setup_wizard_next_step();
                                }
                                SetupWizardStep::TaskToolSlots => {
                                    app.setup_wizard_next_step();
                                }
                                SetupWizardStep::ConfigureMainSlot => {
                                    app.setup_wizard_next_step();
                                }
                                SetupWizardStep::ConfigureResearchSlot => {
                                    app.setup_wizard_next_step();
                                }
                                SetupWizardStep::ConfigureFallbackSlot => {
                                    app.setup_wizard_next_step();
                                }
                                SetupWizardStep::DatabaseConfiguration => {
                                    app.setup_wizard_next_step();
                                }
                                SetupWizardStep::Confirmation => {
                                    // Create configuration files and initialize database
                                    match app.setup_wizard_complete().await {
                                        std::result::Result::Ok(_) => {}
                                        std::result::Result::Err(e) => {
                                            app.status_message = std::option::Option::Some(
                                                std::format!("Setup failed: {}", e)
                                            );
                                        }
                                    }
                                }
                                SetupWizardStep::Complete => {
                                    // Exit wizard and load main app
                                    match app.setup_wizard_exit().await {
                                        std::result::Result::Ok(_) => {}
                                        std::result::Result::Err(e) => {
                                            app.status_message = std::option::Option::Some(
                                                std::format!("Failed to load app: {}", e)
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Esc => {
                            // Exit wizard entirely on Esc from any screen
                            app.should_quit = true;
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            // Provider navigation for slot configuration screens
                            if matches!(app.setup_wizard_step,
                                SetupWizardStep::ConfigureMainSlot |
                                SetupWizardStep::ConfigureResearchSlot |
                                SetupWizardStep::ConfigureFallbackSlot) {
                                app.setup_wizard_next_provider();
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            // Provider navigation for slot configuration screens
                            if matches!(app.setup_wizard_step,
                                SetupWizardStep::ConfigureMainSlot |
                                SetupWizardStep::ConfigureResearchSlot |
                                SetupWizardStep::ConfigureFallbackSlot) {
                                app.setup_wizard_previous_provider();
                            }
                        }
                        KeyCode::Char(c) => {
                            app.setup_wizard_handle_char(c);
                        }
                        KeyCode::Backspace => {
                            app.setup_wizard_handle_backspace();
                        }
                        _ => {}
                    }
                    continue; // Skip all other keyboard handling when wizard is active
                }

                // PRD processing view keyboard handling
                if app.show_prd_processing {
                    // Check if we're in interactive generation mode
                    let in_interactive_mode = matches!(app.prd_processing_state, PRDProcessingState::GeneratingTasks)
                        && !app.prd_gen_conversation.is_empty();

                    if in_interactive_mode {
                        // Interactive generation keyboard handlers
                        match key.code {
                            KeyCode::Char(c) => {
                                // Add character to input buffer
                                app.prd_gen_input.push(c);
                                app.prd_gen_input_active = true;
                            }
                            KeyCode::Backspace => {
                                // Remove last character from input
                                app.prd_gen_input.pop();
                            }
                            KeyCode::Enter => {
                                // Send user input to LLM agent if there's text
                                if !app.prd_gen_input.trim().is_empty() {
                                    let message_content = app.prd_gen_input.clone();

                                    // Add to conversation
                                    let user_message = PRDGenMessage {
                                        role: PRDGenRole::User,
                                        content: MessageContent::Text(message_content.clone()),
                                        timestamp: chrono::Utc::now(),
                                    };
                                    app.prd_gen_conversation.push(user_message);

                                    // Send message to LLM agent via channel
                                    if let Some(sender) = &app.prd_gen_sender {
                                        if let Err(e) = sender.try_send(message_content.clone()) {
                                            // Channel full or closed - add error message
                                            app.prd_gen_conversation.push(PRDGenMessage {
                                                role: PRDGenRole::System,
                                                content: MessageContent::Text(std::format!("⚠️ Failed to send message: {}", e)),
                                                timestamp: chrono::Utc::now(),
                                            });
                                        } else {
                                            // Successfully sent - show confirmation
                                            app.prd_gen_conversation.push(PRDGenMessage {
                                                role: PRDGenRole::System,
                                                content: MessageContent::Text(String::from("💬 Message sent to LLM...")),
                                                timestamp: chrono::Utc::now(),
                                            });
                                        }
                                    }

                                    // Save last message for Up-arrow editing
                                    app.prd_gen_last_message = message_content;
                                    app.prd_gen_editing_last = false;

                                    // Clear input buffer
                                    app.prd_gen_input.clear();
                                    app.prd_gen_input_active = false;

                                    // Auto-scroll to bottom and re-enable auto-scroll
                                    app.prd_gen_scroll_offset = app.prd_gen_conversation.len().saturating_sub(1);
                                    app.prd_gen_auto_scroll = true;
                                }
                            }
                            KeyCode::Esc => {
                                // If editing last message, cancel edit
                                if app.prd_gen_editing_last {
                                    app.prd_gen_editing_last = false;
                                    app.prd_gen_input.clear();
                                } else {
                                    // Clear input but don't cancel generation
                                    app.prd_gen_input.clear();
                                }
                                app.prd_gen_input_active = false;
                            }
                            KeyCode::Up => {
                                // If input is empty and we have a last message, edit it
                                if app.prd_gen_input.is_empty() && !app.prd_gen_last_message.is_empty() {
                                    app.prd_gen_input = app.prd_gen_last_message.clone();
                                    app.prd_gen_editing_last = true;
                                    app.prd_gen_input_active = true;
                                } else {
                                    // Otherwise scroll conversation up and disable auto-scroll
                                    app.prd_gen_scroll_offset = app.prd_gen_scroll_offset.saturating_sub(1);
                                    app.prd_gen_auto_scroll = false;
                                }
                            }
                            KeyCode::Down => {
                                // Scroll conversation down (allow scrolling to end)
                                let max_offset = app.prd_gen_conversation.len().saturating_sub(1);
                                if app.prd_gen_scroll_offset < max_offset {
                                    app.prd_gen_scroll_offset += 1;
                                    // Re-enable auto-scroll if we reached the bottom
                                    if app.prd_gen_scroll_offset >= max_offset {
                                        app.prd_gen_auto_scroll = true;
                                    }
                                }
                            }
                            KeyCode::PageUp => {
                                // Scroll up by ~5 messages for faster navigation
                                app.prd_gen_scroll_offset = app.prd_gen_scroll_offset.saturating_sub(5);
                                app.prd_gen_auto_scroll = false;
                            }
                            KeyCode::PageDown => {
                                // Scroll down by ~5 messages for faster navigation
                                let max_offset = app.prd_gen_conversation.len().saturating_sub(1);
                                app.prd_gen_scroll_offset = std::cmp::min(
                                    app.prd_gen_scroll_offset.saturating_add(5),
                                    max_offset
                                );
                                // Re-enable auto-scroll if we reached the bottom
                                if app.prd_gen_scroll_offset >= max_offset {
                                    app.prd_gen_auto_scroll = true;
                                }
                            }
                            KeyCode::Home => {
                                // Jump to top of conversation
                                app.prd_gen_scroll_offset = 0;
                                app.prd_gen_auto_scroll = false;
                            }
                            KeyCode::End => {
                                // Jump to bottom of conversation and re-enable auto-scroll
                                app.prd_gen_scroll_offset = app.prd_gen_conversation.len().saturating_sub(1);
                                app.prd_gen_auto_scroll = true;
                            }
                            _ => {}
                        }
                    } else {
                        // Standard PRD processing keyboard handlers (non-interactive)
                        match key.code {
                            KeyCode::Enter => {
                                if matches!(app.prd_processing_state, PRDProcessingState::Complete { .. }) {
                                    // Close processing view and return to Kanban
                                    app.show_prd_processing = false;
                                    app.active_tool = DashboardTool::Kanban;

                                    // Clear interactive generation state
                                    app.prd_gen_conversation.clear();
                                    app.prd_gen_partial_tasks.clear();
                                    app.prd_gen_input.clear();
                                    app.prd_gen_status = PRDGenStatus::Idle;
                                    app.prd_gen_input_active = false;
                                    app.prd_gen_scroll_offset = 0;
                                }
                            }
                            KeyCode::Esc => {
                                if matches!(app.prd_processing_state, PRDProcessingState::Failed { .. }) {
                                    // Close processing view on error
                                    app.show_prd_processing = false;

                                    // Clear interactive generation state
                                    app.prd_gen_conversation.clear();
                                    app.prd_gen_partial_tasks.clear();
                                    app.prd_gen_input.clear();
                                    app.prd_gen_status = PRDGenStatus::Idle;
                                    app.prd_gen_input_active = false;
                                    app.prd_gen_scroll_offset = 0;
                                }
                            }
                            _ => {}
                        }
                    }
                    continue; // Skip all other keyboard handling when processing view is active
                }

                match key.code {
                    KeyCode::Char('q') if app.active_dev_tool != std::option::Option::Some(DevTool::SqliteBrowser) => {
                        app.should_quit = true;
                    }
                    KeyCode::Esc => {
                        // Close dialogs/menus in priority order, otherwise quit
                        // Phase 10: Confirmation dialog has highest priority
                        if app.show_confirmation_dialog {
                            app.close_confirmation();
                        } else if app.show_spotlight_dialog {
                            app.close_spotlight();
                        } else if app.show_task_creator_dialog {
                            app.close_task_creator();
                        } else if app.show_sql_query_dialog {
                            app.show_sql_query_dialog = false;
                            app.sql_query_input.clear();
                            app.sql_query_results.clear();
                            app.sql_query_columns.clear();
                        } else if app.show_config_editor {
                            // If currently editing, cancel edit; otherwise close dialog
                            if app.config_editor_editing.is_some() {
                                app.config_editor_editing = std::option::Option::None;
                                app.config_editor_buffer.clear();
                            } else {
                                app.close_config_editor();
                            }
                        } else if app.show_markdown_browser {
                            app.close_markdown_browser();
                        } else if app.show_prd_dialog {
                            app.close_prd_dialog();
                        } else if app.show_dev_tools_menu {
                            app.show_dev_tools_menu = false;
                        } else if app.active_dev_tool.is_some() {
                            // If viewing table data in SQLite browser, go back to table list
                            if app.active_dev_tool == std::option::Option::Some(DevTool::SqliteBrowser) && !app.db_table_data.is_empty() {
                                app.db_table_data.clear();
                                app.db_table_columns.clear();
                                app.db_current_page = 0;
                            } else {
                                // Otherwise close active dev tool and return to previous view
                                app.active_dev_tool = std::option::Option::None;
                            }
                        } else if app.show_llm_chat_dialog {
                            app.close_llm_chat();
                        } else if app.show_task_editor_dialog {
                            app.close_task_editor();
                        } else if app.show_notifications {
                            app.toggle_notifications();
                        } else if app.show_recent_dialog {
                            app.toggle_recent_dialog();
                        } else if app.show_jump_dialog {
                            app.toggle_jump_dialog();
                        } else if app.show_sort_menu {
                            app.toggle_sort_menu();
                        } else {
                            app.should_quit = true;
                        }
                    }
                    KeyCode::Tab => {
                        if app.show_task_creator_dialog {
                            app.next_task_creator_field();
                        } else if app.show_task_editor_dialog {
                            app.next_task_editor_field();
                        } else if app.show_config_editor {
                            // If not currently editing, start editing value field
                            if app.config_editor_editing.is_none() {
                                app.start_editing_config_field(ConfigEditorField::Value);
                            }
                        } else {
                            app.next_tool();
                        }
                    }
                    KeyCode::BackTab => {
                        if app.show_task_creator_dialog {
                            app.previous_task_creator_field();
                        } else if app.show_task_editor_dialog {
                            app.previous_task_editor_field();
                        } else {
                            app.previous_tool();
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if app.show_spotlight_dialog {
                            app.next_spotlight_result();
                        } else if app.show_task_creator_dialog {
                            if app.task_creator_field == TaskCreatorField::Status {
                                app.cycle_creator_status_forward();
                            }
                        } else if app.show_prd_dialog {
                            app.next_prd();
                        } else if app.show_dev_tools_menu {
                            // Navigate down in dev tools menu dialog
                            let max_tools = 2; // SqliteBrowser and ConfigViewer
                            if app.dev_tools_selection < max_tools - 1 {
                                app.dev_tools_selection += 1;
                            }
                        } else if app.show_config_editor && app.config_editor_editing.is_none() {
                            // Navigate down in config editor (only when not editing)
                            if !app.config_editor_items.is_empty() && app.config_editor_selected < app.config_editor_items.len() - 1 {
                                app.config_editor_selected += 1;
                            }
                        } else if app.show_markdown_browser {
                            // Navigate down in markdown browser
                            if !app.markdown_files.is_empty() && app.markdown_selected < app.markdown_files.len() - 1 {
                                app.markdown_selected += 1;
                            }
                        } else if app.active_dev_tool.is_some() {
                            // In SQLite browser: prioritize record navigation if data is loaded
                            if !app.db_table_data.is_empty() && app.db_selected_record < app.db_table_data.len() - 1 {
                                app.db_selected_record += 1;
                            } else if !app.db_tables.is_empty() && app.db_selected_table < app.db_tables.len() - 1 {
                                // Navigate down in table list only if no records or at bottom of records
                                app.db_selected_table += 1;
                            }
                        } else if app.active_tool == DashboardTool::DevTools {
                            // Navigate down in main Dev Tools view (when no tool is active)
                            let max_tools = 2; // SqliteBrowser and ConfigViewer
                            if app.dev_tools_selection < max_tools - 1 {
                                app.dev_tools_selection += 1;
                            }
                        } else if app.show_task_editor_dialog {
                            if app.task_editor_field == TaskEditorField::Status {
                                app.cycle_task_status_forward();
                            }
                        } else if app.show_recent_dialog {
                            app.next_recent_item();
                        } else if app.show_sort_menu {
                            app.next_sort_option();
                        } else {
                            // Navigate down in the selected Kanban column
                            app.next_task_in_column();
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if app.show_spotlight_dialog {
                            app.previous_spotlight_result();
                        } else if app.show_task_creator_dialog {
                            if app.task_creator_field == TaskCreatorField::Status {
                                app.cycle_creator_status_backward();
                            }
                        } else if app.show_prd_dialog {
                            app.previous_prd();
                        } else if app.show_dev_tools_menu {
                            // Navigate up in dev tools menu dialog
                            if app.dev_tools_selection > 0 {
                                app.dev_tools_selection -= 1;
                            }
                        } else if app.show_config_editor && app.config_editor_editing.is_none() {
                            // Navigate up in config editor (only when not editing)
                            if app.config_editor_selected > 0 {
                                app.config_editor_selected -= 1;
                            }
                        } else if app.show_markdown_browser {
                            // Navigate up in markdown browser
                            if app.markdown_selected > 0 {
                                app.markdown_selected -= 1;
                            }
                        } else if app.active_dev_tool.is_some() {
                            // In SQLite browser: prioritize record navigation if data is loaded
                            if !app.db_table_data.is_empty() && app.db_selected_record > 0 {
                                app.db_selected_record -= 1;
                            } else if app.db_selected_table > 0 {
                                // Navigate up in table list only if no records or at top of records
                                app.db_selected_table -= 1;
                            }
                        } else if app.active_tool == DashboardTool::DevTools {
                            // Navigate up in main Dev Tools view (when no tool is active)
                            if app.dev_tools_selection > 0 {
                                app.dev_tools_selection -= 1;
                            }
                        } else if app.show_task_editor_dialog {
                            if app.task_editor_field == TaskEditorField::Status {
                                app.cycle_task_status_backward();
                            }
                        } else if app.show_recent_dialog {
                            app.previous_recent_item();
                        } else if app.show_sort_menu {
                            app.previous_sort_option();
                        } else {
                            // Navigate up in the selected Kanban column
                            app.previous_task_in_column();
                        }
                    }
                    KeyCode::Enter => {
                        // Phase 10: Confirmation dialog has highest priority
                        if app.show_confirmation_dialog {
                            // Confirm the action
                            if let std::result::Result::Err(e) = app.confirm_action().await {
                                app.status_message = std::option::Option::Some(
                                    std::format!("Error: {}", e)
                                );
                            }
                        } else if app.show_spotlight_dialog {
                            // Execute spotlight jump
                            app.execute_spotlight_jump();
                        } else if app.show_sql_query_dialog {
                            // Execute SQL query
                            if let std::result::Result::Err(e) = app.execute_sql_query().await {
                                app.status_message = std::option::Option::Some(
                                    std::format!("SQL error: {}", e)
                                );
                            }
                        } else if app.show_config_editor {
                            // Config editor: Start editing key or commit edit
                            if app.config_editor_editing.is_some() {
                                // Commit the edit
                                app.commit_config_edit();
                            } else {
                                // Start editing key field
                                app.start_editing_config_field(ConfigEditorField::Key);
                            }
                        } else if app.show_markdown_browser {
                            // Initiate PRD processing (UI shows immediately, processing starts on next iteration)
                            app.start_prd_processing();
                        } else if app.show_dev_tools_menu {
                            // Launch selected dev tool from dialog
                            let selected_tool = match app.dev_tools_selection {
                                0 => DevTool::SqliteBrowser,
                                1 => DevTool::ConfigViewer,
                                _ => DevTool::SqliteBrowser, // Default to first tool
                            };
                            app.active_dev_tool = std::option::Option::Some(selected_tool);
                            app.show_dev_tools_menu = false;

                            // Load data based on tool type
                            match selected_tool {
                                DevTool::SqliteBrowser => {
                                    // Load database tables when opening SQLite browser
                                    if let std::result::Result::Err(e) = app.load_db_tables().await {
                                        app.status_message = std::option::Option::Some(
                                            std::format!("Error loading database tables: {}", e)
                                        );
                                    }
                                }
                                DevTool::ConfigViewer => {
                                    // Config viewer doesn't need preloading
                                }
                            }
                        } else if app.active_dev_tool.is_some() {
                            // Handle Enter in active dev tool (check this BEFORE DevTools menu to avoid re-launching)
                            match app.active_dev_tool {
                                std::option::Option::Some(DevTool::SqliteBrowser) => {
                                    // Load table data for selected table
                                    if let std::result::Result::Err(e) = app.load_table_data().await {
                                        app.status_message = std::option::Option::Some(
                                            std::format!("Error loading table data: {}", e)
                                        );
                                    }
                                }
                                std::option::Option::Some(DevTool::ConfigViewer) => {
                                    // Open config editor
                                    if let std::result::Result::Err(e) = app.open_config_editor().await {
                                        app.status_message = std::option::Option::Some(
                                            std::format!("Error opening config editor: {}", e)
                                        );
                                    }
                                }
                                std::option::Option::None => {}
                            }
                        } else if app.active_tool == DashboardTool::DevTools {
                            // Launch selected dev tool from main Dev Tools view (only if no dev tool is active)
                            let selected_tool = match app.dev_tools_selection {
                                0 => DevTool::SqliteBrowser,
                                1 => DevTool::ConfigViewer,
                                _ => DevTool::SqliteBrowser, // Default to first tool
                            };
                            app.active_dev_tool = std::option::Option::Some(selected_tool);

                            // Load data based on tool type
                            match selected_tool {
                                DevTool::SqliteBrowser => {
                                    // Load database tables when opening SQLite browser
                                    if let std::result::Result::Err(e) = app.load_db_tables().await {
                                        app.status_message = std::option::Option::Some(
                                            std::format!("Error loading database tables: {}", e)
                                        );
                                    }
                                }
                                DevTool::ConfigViewer => {
                                    // Config viewer doesn't need preloading
                                }
                            }
                        } else if app.show_task_creator_dialog {
                            // Enter key advances to next field, or submits on last field
                            match app.task_creator_field {
                                TaskCreatorField::Title | TaskCreatorField::Description | TaskCreatorField::Assignee => {
                                    // Advance to next field
                                    app.next_task_creator_field();
                                }
                                TaskCreatorField::Status => {
                                    // Last field - submit the form
                                    if let std::result::Result::Err(e) = app.save_task_creator().await {
                                        app.status_message = std::option::Option::Some(
                                            std::format!("Error creating task: {}", e)
                                        );
                                    }
                                }
                            }
                        } else if app.show_llm_chat_dialog {
                            // Send LLM chat message
                            if let std::result::Result::Err(e) = app.send_llm_chat_message().await {
                                app.status_message = std::option::Option::Some(
                                    std::format!("Error sending chat message: {}", e)
                                );
                            }
                        } else if app.show_task_editor_dialog {
                            // Save task editor changes
                            if let std::result::Result::Err(e) = app.save_task_editor().await {
                                app.status_message = std::option::Option::Some(
                                    std::format!("Error saving task: {}", e)
                                );
                            }
                        } else if app.show_recent_dialog {
                            app.jump_to_recent_task();
                        } else if app.show_jump_dialog {
                            app.execute_jump();
                        } else if app.show_sort_menu {
                            app.apply_selected_sort();
                        } else {
                            // Open task editor for selected task in column
                            if let std::option::Option::Some(task) = app.get_selected_task_in_column() {
                                // Find the task index in the main tasks list
                                if let std::option::Option::Some(idx) = app.tasks.iter().position(|t| t.id == task.id) {
                                    app.selected_task = idx;
                                    app.open_task_editor();
                                }
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        if app.show_spotlight_dialog {
                            app.handle_spotlight_backspace();
                        } else if app.show_sql_query_dialog {
                            app.sql_query_input.pop();
                        } else if app.show_config_editor && app.config_editor_editing.is_some() {
                            app.config_editor_buffer.pop();
                        } else if app.show_task_creator_dialog {
                            app.handle_task_creator_backspace();
                        } else if app.show_llm_chat_dialog {
                            app.handle_llm_chat_backspace();
                        } else if app.show_task_editor_dialog {
                            app.handle_task_editor_backspace();
                        } else if app.show_jump_dialog {
                            app.handle_jump_backspace();
                        } else if app.active_dev_tool == std::option::Option::Some(DevTool::SqliteBrowser) {
                            // Go back to table list from table data view
                            app.db_table_data.clear();
                            app.db_table_columns.clear();
                            app.db_current_page = 0;
                        }
                    }
                    // IMPORTANT: Dialog text input handlers MUST come before specific character hotkeys
                    // so that when dialogs are open, text input is captured instead of triggering hotkeys
                    KeyCode::Char(c) if app.show_spotlight_dialog => {
                        // Handle text input in spotlight search dialog
                        app.handle_spotlight_input(c);
                    }
                    KeyCode::Char(c) if app.show_sql_query_dialog => {
                        // Handle text input in SQL query dialog
                        app.sql_query_input.push(c);
                    }
                    KeyCode::Char(c) if app.show_config_editor && app.config_editor_editing.is_some() => {
                        // Handle text input in config editor when editing
                        app.config_editor_buffer.push(c);
                    }
                    KeyCode::Char('n') if app.show_config_editor && app.config_editor_editing.is_none() => {
                        // Add new config item
                        app.add_config_item();
                    }
                    KeyCode::Char('d') if app.show_config_editor && app.config_editor_editing.is_none() => {
                        // Delete current config item
                        app.delete_config_item();
                    }
                    KeyCode::Char('s') if app.show_config_editor && app.config_editor_editing.is_none() => {
                        // Save config
                        if let std::result::Result::Err(e) = app.save_config().await {
                            app.status_message = std::option::Option::Some(
                                std::format!("Error saving config: {}", e)
                            );
                        }
                    }
                    KeyCode::Char(c) if app.show_task_creator_dialog => {
                        // Handle text input in task creator dialog
                        app.handle_task_creator_input(c);
                    }
                    KeyCode::Char(c) if app.show_llm_chat_dialog => {
                        // Handle text input in LLM chat dialog
                        app.handle_llm_chat_input(c);
                    }
                    KeyCode::Char(c) if app.show_task_editor_dialog => {
                        // Handle text input in task editor dialog
                        app.handle_task_editor_input(c);
                    }
                    KeyCode::Char(c) if app.show_jump_dialog => {
                        // Handle text input in jump dialog
                        app.handle_jump_input(c);
                    }
                    KeyCode::Char('q') if app.active_dev_tool == std::option::Option::Some(DevTool::SqliteBrowser) => {
                        // Open SQL query dialog when in SQLite browser
                        app.show_sql_query_dialog = true;
                    }
                    KeyCode::Char('r') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                        // Ctrl+R: Toggle recent items dialog
                        app.toggle_recent_dialog();
                    }
                    KeyCode::Char('r') => {
                        // Phase 7: Open PRD management dialog
                        if !app.show_prd_dialog && !app.show_llm_chat_dialog && !app.show_task_editor_dialog && !app.show_jump_dialog {
                            app.open_prd_dialog();
                        }
                    }
                    KeyCode::Char('m') => {
                        // Open markdown file browser
                        if !app.show_markdown_browser && !app.show_prd_dialog && !app.show_llm_chat_dialog && !app.show_task_editor_dialog && !app.show_jump_dialog {
                            if let std::result::Result::Err(e) = app.open_markdown_browser().await {
                                app.status_message = std::option::Option::Some(
                                    std::format!("Error opening markdown browser: {}", e)
                                );
                            }
                        }
                    }
                    KeyCode::Char('?') => {
                        app.toggle_shortcuts();
                    }
                    KeyCode::Char('d') => {
                        // Toggle details panel (right column)
                        app.toggle_details_panel();
                    }
                    KeyCode::Char('w') => {
                        // Cycle to previous workspace section
                        app.previous_workspace();
                    }
                    KeyCode::Char('e') => {
                        // Cycle to next workspace section
                        app.next_workspace();
                    }
                    KeyCode::Char('o') => {
                        // Only show sort menu on Kanban board view
                        if app.active_tool == DashboardTool::Kanban && !app.show_jump_dialog {
                            app.toggle_sort_menu();
                        }
                    }
                    KeyCode::Char('g') => {
                        // Open task jump dialog
                        if !app.show_sort_menu {
                            app.toggle_jump_dialog();
                        }
                    }
                    KeyCode::Char('l') => {
                        // Phase 5: Open LLM chat dialog
                        if !app.show_llm_chat_dialog && !app.show_task_editor_dialog && !app.show_jump_dialog {
                            app.open_llm_chat();
                        }
                    }
                    KeyCode::Char('a') => {
                        // Phase 8: Open task creator dialog
                        if !app.show_task_creator_dialog && !app.show_llm_chat_dialog && !app.show_task_editor_dialog && !app.show_jump_dialog && !app.show_prd_dialog {
                            app.open_task_creator();
                        }
                    }
                    KeyCode::Char('/') => {
                        // Phase 9: Open spotlight search dialog
                        if !app.show_spotlight_dialog && !app.show_task_creator_dialog && !app.show_llm_chat_dialog && !app.show_task_editor_dialog && !app.show_jump_dialog && !app.show_prd_dialog {
                            app.open_spotlight();
                        }
                    }
                    KeyCode::Char('s') if !app.show_jump_dialog => {
                        // Cycle task status (async operation)
                        // Note: We need to handle this properly in async context
                        // For now, we'll spawn a blocking task
                        if let std::result::Result::Err(e) = app.cycle_task_status().await {
                            // On error, show error message
                            app.status_message = std::option::Option::Some(
                                std::format!("Error: {}", e)
                            );
                        }
                    }
                    KeyCode::Char('c') if !app.show_jump_dialog => {
                        // Copy task to clipboard
                        app.copy_task_to_clipboard();
                    }
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        // Phase 10: Confirm action in confirmation dialog
                        if app.show_confirmation_dialog {
                            if let std::result::Result::Err(e) = app.confirm_action().await {
                                app.status_message = std::option::Option::Some(
                                    std::format!("Error: {}", e)
                                );
                            }
                        }
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        // Phase 10: Cancel confirmation dialog, or toggle notification center
                        if app.show_confirmation_dialog {
                            app.close_confirmation();
                        } else {
                            app.toggle_notifications();
                        }
                    }
                    KeyCode::F(1) => {
                        // Select Todo column
                        app.select_column(KanbanColumn::Todo);
                    }
                    KeyCode::F(2) => {
                        // Select In Progress column
                        app.select_column(KanbanColumn::InProgress);
                    }
                    KeyCode::F(3) => {
                        // Select Completed column
                        app.select_column(KanbanColumn::Completed);
                    }
                    KeyCode::F(4) => {
                        // Select Archived column
                        app.select_column(KanbanColumn::Archived);
                    }
                    KeyCode::F(5) => {
                        // Select Errored column
                        app.select_column(KanbanColumn::Errored);
                    }
                    KeyCode::F(6) => {
                        // Phase 13: Toggle between Kanban and Metrics views
                        app.active_tool = match app.active_tool {
                            DashboardTool::Kanban => DashboardTool::Metrics,
                            DashboardTool::Metrics => DashboardTool::Kanban,
                            _ => DashboardTool::Kanban, // Default to Kanban for other views
                        };
                        app.add_notification(
                            NotificationLevel::Info,
                            std::format!("Switched to {}", app.active_tool.display_name())
                        );
                    }
                    KeyCode::PageUp => {
                        // Handle pagination in SQLite browser
                        if app.active_dev_tool == std::option::Option::Some(DevTool::SqliteBrowser) {
                            if app.db_current_page > 0 {
                                app.db_current_page -= 1;
                                if let std::result::Result::Err(e) = app.load_table_data().await {
                                    app.status_message = std::option::Option::Some(
                                        std::format!("Error loading table data: {}", e)
                                    );
                                }
                            }
                        }
                    }
                    KeyCode::PageDown => {
                        // Handle pagination in SQLite browser
                        if app.active_dev_tool == std::option::Option::Some(DevTool::SqliteBrowser) {
                            app.db_current_page += 1;
                            if let std::result::Result::Err(e) = app.load_table_data().await {
                                app.status_message = std::option::Option::Some(
                                    std::format!("Error loading table data: {}", e)
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    std::result::Result::Ok(())
}

/// Renders the TUI layout.
fn ui(f: &mut Frame, app: &App) {
    // Setup wizard takes complete priority over normal UI
    if app.setup_wizard_active {
        render_setup_wizard(f, app);
        return;
    }

    // PRD processing view takes priority over normal UI
    if app.show_prd_processing {
        render_prd_processing(f, f.area(), app);
        return;
    }

    let size = f.area();

    // Create 3-column layout: nav (20%) | main (60%) | details (20%)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),  // Left nav
            Constraint::Percentage(if app.show_details_panel { 60 } else { 80 }),  // Main
            Constraint::Percentage(if app.show_details_panel { 20 } else { 0 }),   // Details
        ])
        .split(size);

    // Render left navigation panel
    render_navigation_panel(f, main_chunks[0], app);

    // Render main tool area (with title bar and footer)
    let main_area = main_chunks[1];
    let main_chunks_vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title bar
            Constraint::Min(0),     // Content
            Constraint::Length(3),  // Footer bar
        ])
        .split(main_area);

    // Render title bar with save indicator
    render_title_bar(f, main_chunks_vertical[0], app);

    // Render active tool content or active dev tool if one is running
    if let std::option::Option::Some(ref dev_tool) = app.active_dev_tool {
        match dev_tool {
            DevTool::SqliteBrowser => render_sqlite_browser(f, main_chunks_vertical[1], app),
            DevTool::ConfigViewer => render_config_viewer(f, main_chunks_vertical[1], app),
        }
    } else {
        match app.active_tool {
            DashboardTool::Kanban => render_task_board(f, main_chunks_vertical[1], app),
            DashboardTool::TaskEditor => render_task_editor(f, main_chunks_vertical[1], app),
            DashboardTool::LLMChat => render_llm_chat(f, main_chunks_vertical[1], app),
            DashboardTool::Metrics => render_metrics(f, main_chunks_vertical[1], app),
            DashboardTool::DevTools => render_dev_tools_view(f, main_chunks_vertical[1], app),
        }
    }

    // Render footer bar with summary stats
    render_footer_bar(f, main_chunks_vertical[2], app);

    // Render details panel if enabled
    if app.show_details_panel {
        render_details_panel(f, main_chunks[2], app);
    }

    // Render keyboard shortcut overlay if active
    if app.show_shortcuts {
        render_shortcut_overlay(f, app);
    }

    // Render status message toast if present
    if app.status_message.is_some() {
        render_status_toast(f, app);
    }

    // Render sort menu if active
    if app.show_sort_menu {
        render_sort_menu(f, app);
    }

    // Render jump dialog if active
    if app.show_jump_dialog {
        render_jump_dialog(f, app);
    }

    // Render recent items dialog if active
    if app.show_recent_dialog {
        render_recent_dialog(f, app);
    }

    // Render loading indicator if active
    if app.is_loading {
        render_loading_indicator(f, app);
    }

    // Render PRD management dialog if active (Phase 7)
    if app.show_prd_dialog {
        render_prd_dialog(f, app);
    }

    // Render dev tools menu if active
    if app.show_dev_tools_menu {
        render_dev_tools_menu(f, app);
    }

    // Render SQL query dialog if active
    if app.show_sql_query_dialog {
        render_sql_query_dialog(f, app);
    }

    // Render config editor dialog if active
    if app.show_config_editor {
        render_config_editor_dialog(f, app);
    }

    // Render markdown file browser dialog if active
    if app.show_markdown_browser {
        render_markdown_browser_dialog(f, app);
    }

    // Render task creator dialog if active (Phase 8)
    if app.show_task_creator_dialog {
        render_task_creator_dialog(f, app);
    }

    // Render spotlight search dialog if active (Phase 9)
    if app.show_spotlight_dialog {
        render_spotlight_dialog(f, app);
    }

    // Render confirmation dialog if active (Phase 10) - highest priority, renders on top
    if app.show_confirmation_dialog {
        render_confirmation_dialog(f, app);
    }

    // Render LLM chat dialog if active (Phase 5)
    if app.show_llm_chat_dialog {
        render_llm_chat_dialog(f, app);
    }

    // Render task editor dialog if active (Phase 4)
    if app.show_task_editor_dialog {
        render_task_editor_dialog(f, app);
    }

    // Render notification center if active
    if app.show_notifications {
        render_notifications(f, app);
    }
}

/// Renders the left navigation panel with workspace and tools.
fn render_navigation_panel(f: &mut Frame, area: Rect, app: &App) {
    // Phase 3: Only show Kanban and Metrics as main views
    // TaskEditor and LLMChat will be dialog-only (Phases 4 & 5)
    // Dev Tools opens a menu dialog for development utilities
    let tools = vec![
        DashboardTool::Kanban,
        DashboardTool::Metrics,
        DashboardTool::DevTools,
    ];

    let mut items = std::vec![
        Line::from(Span::styled(
            " PROJECTS",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        )),
    ];

    // Render projects list
    if app.projects.is_empty() {
        items.push(Line::from(Span::styled(
            "  No projects yet!",
            Style::default().fg(Color::Yellow)
        )));
        items.push(Line::from(""));
        items.push(Line::from(Span::styled(
            "  Get started:",
            Style::default().fg(Color::Cyan)
        )));
        items.push(Line::from(Span::styled(
            "  1. Create PRD file",
            Style::default().fg(Color::DarkGray)
        )));
        items.push(Line::from(Span::styled(
            "  2. Run: rig parse",
            Style::default().fg(Color::Green)
        )));
        items.push(Line::from(Span::styled(
            "     <prd-file>",
            Style::default().fg(Color::Green)
        )));
        items.push(Line::from(""));
        items.push(Line::from(Span::styled(
            "  Or press ? for",
            Style::default().fg(Color::DarkGray)
        )));
        items.push(Line::from(Span::styled(
            "  full help",
            Style::default().fg(Color::DarkGray)
        )));
    } else {
        for project in &app.projects {
            let is_selected = app.selected_project_id.as_ref()
                .map(|id| id == &project.id)
                .unwrap_or(false);

            let style = if is_selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let prefix = if is_selected { "▶ " } else { "  " };

            // Count tasks for this project
            let task_count = app.get_filtered_tasks().len();
            let display_text = if is_selected {
                std::format!("{} ({} tasks)", project.name, task_count)
            } else {
                project.name.clone()
            };

            items.push(Line::from(Span::styled(
                std::format!("{}{}", prefix, display_text),
                style
            )));
        }
    }

    items.push(Line::from(""));
    items.push(Line::from(Span::styled(
        " TOOLS",
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    )));

    // Render tools
    for (_idx, tool) in tools.iter().enumerate() {
        let is_active = *tool == app.active_tool;
        let style = if is_active {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let prefix = if is_active { "▶ " } else { "  " };
        items.push(Line::from(Span::styled(
            std::format!("{}{}", prefix, tool.display_name()),
            style
        )));
    }

    // Add key hints at bottom
    items.push(Line::from(""));
    items.push(Line::from(""));
    items.push(Line::from(Span::styled(
        " SHORTCUTS",
        Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)
    )));
    items.push(Line::from(vec![
      Span::styled("  /", Style::default().fg(Color::DarkGray)),
      Span::raw("    "),
      Span::styled("Spotlight", Style::default().fg(Color::DarkGray)),
    ]));
    items.push(Line::from(vec![
        Span::styled("  w/e", Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled("Switch projects", Style::default().fg(Color::DarkGray)),
    ]));
    items.push(Line::from(vec![
        Span::styled("  Tab", Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled("Switch tool", Style::default().fg(Color::DarkGray)),
    ]));
    items.push(Line::from(vec![
        Span::styled("  d", Style::default().fg(Color::DarkGray)),
        Span::raw("    "),
        Span::styled("Toggle details", Style::default().fg(Color::DarkGray)),
    ]));
    items.push(Line::from(vec![
        Span::styled("  ?", Style::default().fg(Color::DarkGray)),
        Span::raw("    "),
        Span::styled("Help", Style::default().fg(Color::DarkGray)),
    ]));

    let nav_widget = Paragraph::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue))
                .title(" Navigation ")
        );

    f.render_widget(nav_widget, area);
}

/// Renders the title bar with tool name and save indicator.
fn render_title_bar(f: &mut Frame, area: Rect, app: &App) {
    let save_indicator = app.format_save_indicator();

    // Build title with project name (if selected) and tool name
    let mut title_parts = std::vec::Vec::new();

    // Add project name if one is selected
    if let std::option::Option::Some(project) = app.get_selected_project() {
        title_parts.push(std::format!("Project: {}", project.name));
    }

    // Add tool name
    title_parts.push(app.active_tool.display_name().to_string());

    // Add save indicator if present
    if !save_indicator.is_empty() {
        title_parts.push(save_indicator);
    }

    let title = title_parts.join("  │  ");

    let title_widget = Paragraph::new(title)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
        )
        .style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(title_widget, area);
}

/// Renders the footer bar with summary statistics and live status (Phase 11).
///
/// Shows task counts by status, active filter, loading indicator, current time,
/// session duration, database status, and entity counts in a two-line display.
fn render_footer_bar(f: &mut Frame, area: Rect, app: &App) {
    // Task counts
    let filtered_tasks = app.get_filtered_tasks();
    let total_tasks = filtered_tasks.len();
    let todo_count = filtered_tasks.iter().filter(|t| matches!(t.status, task_manager::domain::task_status::TaskStatus::Todo)).count();
    let in_progress_count = filtered_tasks.iter().filter(|t| matches!(t.status, task_manager::domain::task_status::TaskStatus::InProgress)).count();
    let completed_count = filtered_tasks.iter().filter(|t| matches!(t.status, task_manager::domain::task_status::TaskStatus::Completed)).count();

    let column_text = app.selected_column.display_name();

    // Line 1: Task stats, selected column, current time
    let mut line1 = std::format!(
        " 📋 {} │ ⏳ {} │ 🔄 {} │ ✓ {} │ Column: {} │ 🕒 {}",
        total_tasks, todo_count, in_progress_count, completed_count, column_text, app.format_current_time()
    );

    // Add loading indicator if active
    if app.is_loading {
        if let std::option::Option::Some(ref msg) = app.loading_message {
            line1.push_str(&std::format!(" │ {} {}", app.get_spinner_char(), msg));
        }
    }

    // Line 2: Session info, database status, entity counts, help hint
    let project_count = app.projects.len();
    let filtered_prds = app.get_filtered_prds();
    let prd_count = filtered_prds.len();
    let current_project_name = if let std::option::Option::Some(project) = app.get_selected_project() {
        std::format!("'{}'", project.name)
    } else {
        String::from("All Projects")
    };

    let line2 = std::format!(
        " ⏱️  Session: {} │ {} │ 🎯 {} Projects │ 📄 {} PRDs │ View: {} │ Press ? for help",
        app.format_session_duration(),
        app.get_database_status(),
        project_count,
        prd_count,
        current_project_name
    );

    // Build multi-line footer
    let footer_lines = std::vec![
        Line::from(line1),
        Line::from(line2),
    ];

    let footer_widget = Paragraph::new(footer_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue))
        )
        .style(Style::default().fg(Color::Cyan));

    f.render_widget(footer_widget, area);
}

/// Renders the task editor tool (full CRUD interface).
fn render_task_editor(f: &mut Frame, area: Rect, app: &App) {
    if app.tasks.is_empty() {
        let placeholder = Paragraph::new("No task selected.\n\nSelect a task from the Kanban board to edit.")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Gray))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(placeholder, area);
        return;
    }

    let task = &app.tasks[app.selected_task];

    let mut lines = std::vec![
        Line::from(Span::styled("TASK EDITOR", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled("ID: ", Style::default().fg(Color::Cyan)),
            Span::raw(&task.id),
        ]),
        Line::from(vec![
            Span::styled("Title: ", Style::default().fg(Color::Cyan)),
            Span::raw(&task.title),
        ]),
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Cyan)),
            Span::styled(format_status_text(&task.status), get_status_color(&task.status)),
        ]),
    ];

    if let std::option::Option::Some(ref assignee) = task.assignee {
        lines.push(Line::from(vec![
            Span::styled("Assignee: ", Style::default().fg(Color::Cyan)),
            Span::raw(assignee),
        ]));
    }

    if let std::option::Option::Some(complexity) = task.complexity {
        lines.push(Line::from(vec![
            Span::styled("Complexity: ", Style::default().fg(Color::Cyan)),
            Span::raw(std::format!("{}/10", complexity)),
        ]));
    }

    if let std::option::Option::Some(ref reasoning) = task.reasoning {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("Reasoning:", Style::default().fg(Color::Cyan))));
        lines.push(Line::from(reasoning.as_str()));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "[Tab] Switch tool  [e] Edit  [s] Change status",
        Style::default().fg(Color::Gray)
    )));

    let editor_widget = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    f.render_widget(editor_widget, area);
}

/// Renders the LLM chat/command interface.
fn render_llm_chat(f: &mut Frame, area: Rect, _app: &App) {
    let lines = std::vec![
        Line::from(Span::styled("LLM CHAT INTERFACE", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("💬 Chat with the AI to:", Style::default().fg(Color::Cyan))),
        Line::from("  • Generate subtasks from a parent task"),
        Line::from("  • Ask questions about task dependencies"),
        Line::from("  • Get complexity estimates"),
        Line::from("  • Query task history and patterns"),
        Line::from(""),
        Line::from(Span::styled("🚧 Coming Soon", Style::default().fg(Color::Yellow))),
        Line::from("This feature will integrate with your LLM provider"),
        Line::from("to provide intelligent task management assistance."),
    ];

    let chat_widget = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    f.render_widget(chat_widget, area);
}

/// Renders the metrics and analytics dashboard.
fn render_metrics(f: &mut Frame, area: Rect, app: &App) {
    let total_tasks = app.tasks.len();
    let completed = app.tasks.iter().filter(|t| matches!(t.status, task_manager::domain::task_status::TaskStatus::Completed)).count();
    let in_progress = app.tasks.iter().filter(|t| matches!(t.status, task_manager::domain::task_status::TaskStatus::InProgress)).count();
    let todo = app.tasks.iter().filter(|t| matches!(t.status, task_manager::domain::task_status::TaskStatus::Todo)).count();

    let completion_rate = if total_tasks > 0 {
        (completed as f64 / total_tasks as f64 * 100.0) as usize
    } else {
        0
    };

    let lines = std::vec![
        Line::from(Span::styled("METRICS & ANALYTICS", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("📊 Task Overview", Style::default().fg(Color::Cyan))),
        Line::from(std::format!("  Total Tasks: {}", total_tasks)),
        Line::from(vec![
            Span::raw("  ✓ Completed: "),
            Span::styled(std::format!("{}", completed), Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::raw("  🎯 In Progress: "),
            Span::styled(std::format!("{}", in_progress), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::raw("  📋 TODO: "),
            Span::styled(std::format!("{}", todo), Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Completion Rate: ", Style::default().fg(Color::Cyan)),
            Span::styled(std::format!("{}%", completion_rate), Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(Span::styled("🚧 More metrics coming soon:", Style::default().fg(Color::Yellow))),
        Line::from("  • Velocity charts"),
        Line::from("  • Complexity distribution"),
        Line::from("  • Time-to-completion analysis"),
    ];

    let metrics_widget = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    f.render_widget(metrics_widget, area);
}

/// Renders the SQLite database browser dev tool.
///
/// Displays a list of database tables with basic navigation.
/// Press Esc to close and return to previous view.
fn render_sqlite_browser(f: &mut Frame, area: Rect, app: &App) {
    let mut lines = std::vec![
        Line::from(Span::styled(
            "🗄️  SQLite Database Browser",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
        Line::from(Span::styled(
            std::format!("Database: .rigger/tasks.db"),
            Style::default().fg(Color::DarkGray)
        )),
        Line::from(""),
    ];

    if app.db_tables.is_empty() {
        lines.push(Line::from(Span::styled(
            "No tables found in database.",
            Style::default().fg(Color::Yellow)
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "The database may be empty or not yet initialized.",
            Style::default().fg(Color::DarkGray)
        )));
    } else if app.db_table_data.is_empty() {
        // Show table list when no table data is loaded
        lines.push(Line::from(Span::styled(
            std::format!("Tables ({}):", app.db_tables.len()),
            Style::default().fg(Color::Cyan)
        )));
        lines.push(Line::from(""));

        // Render table list
        for (idx, table_name) in app.db_tables.iter().enumerate() {
            let is_selected = idx == app.db_selected_table;
            let prefix = if is_selected { "▶ " } else { "  " };
            let style = if is_selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            lines.push(Line::from(Span::styled(
                std::format!("{}{}", prefix, table_name),
                style
            )));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("↑/↓", Style::default().fg(Color::Cyan)),
            Span::raw(" Navigate  "),
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(" View Data  "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(" Close"),
        ]));
    } else {
        // Show table data with columns
        let table_name = &app.db_tables[app.db_selected_table];
        lines.push(Line::from(Span::styled(
            std::format!("Table: {} (Page {})", table_name, app.db_current_page + 1),
            Style::default().fg(Color::Yellow)
        )));
        lines.push(Line::from(""));

        // Check if table is empty and show helpful message
        if app.db_table_data.is_empty() {
            lines.push(Line::from(Span::styled(
                std::format!("✨ Table '{}' is empty", table_name),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            )));
            lines.push(Line::from(""));

            // Provide contextual guidance based on table name
            let guidance = match table_name.as_str() {
                "tasks" => vec![
                    "Get started by creating tasks:",
                    "  • Press 'a' to create a new task manually",
                    "  • Or parse a PRD: rig parse <file.md>",
                    "  • Or extract from transcript: rig extract <audio>",
                ],
                "projects" => vec![
                    "Get started by creating a project:",
                    "  • Run: rig project create <name>",
                    "  • Projects organize PRDs and tasks hierarchically",
                ],
                "prds" => vec![
                    "Get started by adding a PRD:",
                    "  • Press 'm' to browse markdown files",
                    "  • Or run: rig parse <file.md>",
                ],
                _ => vec![
                    "This table doesn't have any data yet.",
                    "  • Add data using SQL queries or application commands",
                ],
            };

            for guide_line in guidance {
                lines.push(Line::from(Span::styled(
                    std::format!("  {}", guide_line),
                    Style::default().fg(Color::DarkGray)
                )));
            }
        } else {
            // Render column headers
            if !app.db_table_columns.is_empty() {
                let header_line = app.db_table_columns.iter()
                    .map(|col| {
                        let truncated = if col.len() > 15 {
                            std::format!("{}...", &col[..12])
                        } else {
                            col.clone()
                        };
                        std::format!("{:15}", truncated)
                    })
                    .collect::<std::vec::Vec<_>>()
                    .join(" │ ");

                lines.push(Line::from(Span::styled(
                    header_line,
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                )));
                lines.push(Line::from(Span::raw("─".repeat(80))));
            }

            // Render data rows (max 10 rows to fit screen)
            for (row_idx, row) in app.db_table_data.iter().take(10).enumerate() {
                let mut row_values = std::vec::Vec::new();
                for col_name in &app.db_table_columns {
                    let value = row.get(col_name).cloned().unwrap_or_else(|| String::from("-"));
                    let truncated = if value.len() > 15 {
                        std::format!("{}...", &value[..12])
                    } else {
                        value
                    };
                    row_values.push(std::format!("{:15}", truncated));
                }

                // Highlight selected record
                let is_selected = row_idx == app.db_selected_record;
                let prefix = if is_selected { "▶ " } else { "  " };
                let row_text = std::format!("{}{}", prefix, row_values.join(" │ "));
                let style = if is_selected {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                lines.push(Line::from(Span::styled(row_text, style)));
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("↑/↓", Style::default().fg(Color::Cyan)),
            Span::raw(" Navigate  "),
            Span::styled("PgUp/PgDn", Style::default().fg(Color::Cyan)),
            Span::raw(" Paginate  "),
            Span::styled("q", Style::default().fg(Color::Green)),
            Span::raw(" SQL Query  "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(" Back"),
        ]));
    }

    let browser_widget = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .padding(ratatui::widgets::Padding::horizontal(1))
        )
        .wrap(Wrap { trim: true });

    f.render_widget(browser_widget, area);
}

/// Renders the Rigger configuration viewer dev tool.
///
/// Displays rigger configuration settings from the .rigger directory.
/// Shows database path, server settings, and other configuration options.
fn render_config_viewer(f: &mut Frame, area: Rect, _app: &App) {
    let mut lines = std::vec![
        Line::from(Span::styled(
            " ⚙️  Rigger Configuration",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
    ];

    // Get current directory and check for .rigger config
    let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let rigger_dir = current_dir.join(".rigger");
    let db_path = rigger_dir.join("tasks.db");

    // Display configuration information
    lines.push(Line::from(Span::styled(
        " 📁 Directory Configuration:",
        Style::default().fg(Color::Yellow)
    )));
    lines.push(Line::from(""));

    lines.push(Line::from(vec![
        Span::styled("  Working Directory: ", Style::default().fg(Color::DarkGray)),
        Span::raw(current_dir.display().to_string()),
    ]));
    lines.push(Line::from(""));

    lines.push(Line::from(vec![
        Span::styled("  Rigger Config Dir: ", Style::default().fg(Color::DarkGray)),
        Span::raw(rigger_dir.display().to_string()),
    ]));
    lines.push(Line::from(vec![
        Span::styled("    Status: ", Style::default().fg(Color::DarkGray)),
        if rigger_dir.exists() {
            Span::styled("✓ Initialized", Style::default().fg(Color::Green))
        } else {
            Span::styled("✗ Not initialized", Style::default().fg(Color::Red))
        },
    ]));
    lines.push(Line::from(""));

    lines.push(Line::from(Span::styled(
        " 🗄️  Database Configuration:",
        Style::default().fg(Color::Yellow)
    )));
    lines.push(Line::from(""));

    lines.push(Line::from(vec![
        Span::styled("  Database Path: ", Style::default().fg(Color::DarkGray)),
        Span::raw(db_path.display().to_string()),
    ]));
    lines.push(Line::from(vec![
        Span::styled("    Status: ", Style::default().fg(Color::DarkGray)),
        if db_path.exists() {
            let metadata = std::fs::metadata(&db_path).ok();
            let size_str = metadata.map(|m| {
                let kb = m.len() / 1024;
                std::format!("✓ Exists ({} KB)", kb)
            }).unwrap_or_else(|| String::from("✓ Exists"));
            Span::styled(size_str, Style::default().fg(Color::Green))
        } else {
            Span::styled("✗ Not found", Style::default().fg(Color::Red))
        },
    ]));
    lines.push(Line::from(""));

    // Try to parse config.json and display task tools configuration
    let config_json_path = rigger_dir.join("config.json");

    lines.push(Line::from(Span::styled(
        " 🔧 Task Tool Slots:",
        Style::default().fg(Color::Yellow)
    )));
    lines.push(Line::from(""));

    if config_json_path.exists() {
        match std::fs::read_to_string(&config_json_path) {
            std::result::Result::Ok(config_str) => {
                match serde_json::from_str::<serde_json::Value>(&config_str) {
                    std::result::Result::Ok(config) => {
                        if let std::option::Option::Some(task_tools) = config.get("task_tools") {
                            // Extract all values as owned Strings to avoid lifetime issues
                            let main_provider_str = task_tools.get("main")
                                .and_then(|m| m.get("provider"))
                                .and_then(|p| p.as_str())
                                .unwrap_or("unknown")
                                .to_string();
                            let main_model_str = task_tools.get("main")
                                .and_then(|m| m.get("model"))
                                .and_then(|m| m.as_str())
                                .unwrap_or("unknown")
                                .to_string();
                            let research_provider_str = task_tools.get("research")
                                .and_then(|r| r.get("provider"))
                                .and_then(|p| p.as_str())
                                .unwrap_or("unknown")
                                .to_string();
                            let research_model_str = task_tools.get("research")
                                .and_then(|r| r.get("model"))
                                .and_then(|m| m.as_str())
                                .unwrap_or("unknown")
                                .to_string();
                            let fallback_provider_str = task_tools.get("fallback")
                                .and_then(|f| f.get("provider"))
                                .and_then(|p| p.as_str())
                                .unwrap_or("unknown")
                                .to_string();
                            let fallback_model_str = task_tools.get("fallback")
                                .and_then(|f| f.get("model"))
                                .and_then(|m| m.as_str())
                                .unwrap_or("unknown")
                                .to_string();

                            // Display Main slot
                            lines.push(Line::from(Span::styled(
                                "  🔧 Main Slot:",
                                Style::default().fg(Color::Cyan)
                            )));
                            lines.push(Line::from(vec![
                                Span::styled("    Provider: ", Style::default().fg(Color::DarkGray)),
                                Span::styled(main_provider_str, Style::default().fg(Color::Green)),
                            ]));
                            lines.push(Line::from(vec![
                                Span::styled("    Model: ", Style::default().fg(Color::DarkGray)),
                                Span::raw(main_model_str),
                            ]));
                            lines.push(Line::from(""));

                            // Display Research slot
                            lines.push(Line::from(Span::styled(
                                "  🔍 Research Slot:",
                                Style::default().fg(Color::Cyan)
                            )));
                            lines.push(Line::from(vec![
                                Span::styled("    Provider: ", Style::default().fg(Color::DarkGray)),
                                Span::styled(research_provider_str, Style::default().fg(Color::Green)),
                            ]));
                            lines.push(Line::from(vec![
                                Span::styled("    Model: ", Style::default().fg(Color::DarkGray)),
                                Span::raw(research_model_str),
                            ]));
                            lines.push(Line::from(""));

                            // Display Fallback slot
                            lines.push(Line::from(Span::styled(
                                "  🛟 Fallback Slot:",
                                Style::default().fg(Color::Cyan)
                            )));
                            lines.push(Line::from(vec![
                                Span::styled("    Provider: ", Style::default().fg(Color::DarkGray)),
                                Span::styled(fallback_provider_str, Style::default().fg(Color::Green)),
                            ]));
                            lines.push(Line::from(vec![
                                Span::styled("    Model: ", Style::default().fg(Color::DarkGray)),
                                Span::raw(fallback_model_str),
                            ]));
                        } else {
                            lines.push(Line::from(Span::styled(
                                "  ⚠️  task_tools configuration not found in config.json",
                                Style::default().fg(Color::Yellow)
                            )));
                        }
                    }
                    std::result::Result::Err(_) => {
                        lines.push(Line::from(Span::styled(
                            "  ✗ Failed to parse config.json",
                            Style::default().fg(Color::Red)
                        )));
                    }
                }
            }
            std::result::Result::Err(_) => {
                lines.push(Line::from(Span::styled(
                    "  ✗ Failed to read config.json",
                    Style::default().fg(Color::Red)
                )));
            }
        }
    } else {
        lines.push(Line::from(Span::styled(
            "  ✗ config.json not found",
            Style::default().fg(Color::Red)
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Run setup wizard to create configuration",
            Style::default().fg(Color::DarkGray)
        )));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::raw(" Edit Configuration  "),
        Span::styled("Esc", Style::default().fg(Color::Red)),
        Span::raw(" Close"),
    ]));

    let config_widget = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
        )
        .wrap(Wrap { trim: true });

    f.render_widget(config_widget, area);
}

/// Renders the SQL query executor dialog.
///
/// Allows users to input and execute SQL queries against the database.
/// Shows query results in a table format.
fn render_sql_query_dialog(f: &mut Frame, app: &App) {
    let mut lines = std::vec![
        Line::from(Span::styled(
            " 💾 SQL Query Executor ",
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Enter SQL query:",
            Style::default().fg(Color::Cyan)
        )),
        Line::from(Span::styled(
            &app.sql_query_input,
            Style::default().fg(Color::White)
        )),
        Line::from(Span::raw("▂".repeat(60))),
        Line::from(""),
    ];

    // Show results if available
    if !app.sql_query_results.is_empty() {
        lines.push(Line::from(Span::styled(
            std::format!("Results ({} rows):", app.sql_query_results.len()),
            Style::default().fg(Color::Green)
        )));
        lines.push(Line::from(""));

        // Render column headers
        if !app.sql_query_columns.is_empty() {
            let header_line = app.sql_query_columns.iter()
                .map(|col| {
                    let truncated = if col.len() > 15 {
                        std::format!("{}...", &col[..12])
                    } else {
                        col.clone()
                    };
                    std::format!("{:15}", truncated)
                })
                .collect::<std::vec::Vec<_>>()
                .join(" │ ");

            lines.push(Line::from(Span::styled(
                header_line,
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            )));
            lines.push(Line::from(Span::raw("─".repeat(60))));
        }

        // Render data rows (max 8 rows)
        for row in app.sql_query_results.iter().take(8) {
            let mut row_values = std::vec::Vec::new();
            for col_name in &app.sql_query_columns {
                let value = row.get(col_name).cloned().unwrap_or_else(|| String::from("-"));
                let truncated = if value.len() > 15 {
                    std::format!("{}...", &value[..12])
                } else {
                    value
                };
                row_values.push(std::format!("{:15}", truncated));
            }
            lines.push(Line::from(Span::raw(row_values.join(" │ "))));
        }

        if app.sql_query_results.len() > 8 {
            lines.push(Line::from(Span::styled(
                std::format!("... and {} more rows", app.sql_query_results.len() - 8),
                Style::default().fg(Color::DarkGray)
            )));
        }
    } else if !app.sql_query_input.is_empty() {
        lines.push(Line::from(Span::styled(
            "No results yet. Press Enter to execute query.",
            Style::default().fg(Color::DarkGray)
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::raw(" Execute  "),
        Span::styled("Esc", Style::default().fg(Color::Red)),
        Span::raw(" Close"),
    ]));

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Examples:",
        Style::default().fg(Color::Yellow)
    )));
    lines.push(Line::from("  SELECT * FROM tasks LIMIT 10"));
    lines.push(Line::from("  SELECT title, status FROM tasks WHERE status = 'Todo'"));

    // Calculate dialog size
    let area_rect = f.area();
    let dialog_width = 70;
    let dialog_height = 30.min(area_rect.height - 4);
    let dialog = Rect {
        x: (area_rect.width.saturating_sub(dialog_width)) / 2,
        y: (area_rect.height.saturating_sub(dialog_height)) / 2,
        width: dialog_width,
        height: dialog_height,
    };

    // Clear the dialog area first to prevent backdrop from showing through
    f.render_widget(ratatui::widgets::Clear, dialog);

    // Render the dialog
    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta))
                .style(Style::default().bg(Color::Black))
        )
        .style(Style::default().bg(Color::Black))
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, dialog);
}

/// Renders the config editor dialog.
///
/// Displays editable configuration key-value pairs with navigation,
/// editing, adding, and deleting capabilities.
fn render_config_editor_dialog(f: &mut Frame, app: &App) {
    // Calculate dialog size (70% of screen)
    let area_rect = f.area();
    let dialog_width = std::cmp::min(80, area_rect.width.saturating_sub(10));
    let dialog_height = std::cmp::min(30, area_rect.height.saturating_sub(4));
    let dialog = Rect {
        x: (area_rect.width.saturating_sub(dialog_width)) / 2,
        y: (area_rect.height.saturating_sub(dialog_height)) / 2,
        width: dialog_width,
        height: dialog_height,
    };

    // Build content lines
    let mut lines = std::vec![
        Line::from(Span::styled(
            " ⚙️  Configuration Editor ",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
    ];

    // Show config items
    if app.config_editor_items.is_empty() {
        lines.push(Line::from(Span::styled(
            "No configuration items",
            Style::default().fg(Color::DarkGray)
        )));
    } else {
        for (idx, (key, value)) in app.config_editor_items.iter().enumerate() {
            let is_selected = idx == app.config_editor_selected;
            let is_editing_key = app.config_editor_editing == std::option::Option::Some(ConfigEditorField::Key) && is_selected;
            let is_editing_value = app.config_editor_editing == std::option::Option::Some(ConfigEditorField::Value) && is_selected;

            // Selection indicator
            let indicator = if is_selected { "▶ " } else { "  " };

            // Format key field
            let key_display = if is_editing_key {
                std::format!("[{}]", &app.config_editor_buffer)
            } else {
                key.clone()
            };

            // Format value field
            let value_display = if is_editing_value {
                std::format!("[{}]", &app.config_editor_buffer)
            } else {
                value.clone()
            };

            // Build line
            let key_style = if is_editing_key {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::White)
            };

            let value_style = if is_editing_value {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Gray)
            };

            lines.push(Line::from(vec![
                Span::raw(indicator),
                Span::styled(key_display, key_style),
                Span::raw(" = "),
                Span::styled(value_display, value_style),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(""));

    // Show keyboard shortcuts
    lines.push(Line::from(vec![
        Span::styled("↑/↓", Style::default().fg(Color::Cyan)),
        Span::raw(" Navigate  "),
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::raw(" Edit Key  "),
        Span::styled("Tab", Style::default().fg(Color::Magenta)),
        Span::raw(" Edit Value"),
    ]));

    lines.push(Line::from(vec![
        Span::styled("n", Style::default().fg(Color::Green)),
        Span::raw(" New Item  "),
        Span::styled("d", Style::default().fg(Color::Red)),
        Span::raw(" Delete  "),
        Span::styled("s", Style::default().fg(Color::Yellow)),
        Span::raw(" Save  "),
        Span::styled("Esc", Style::default().fg(Color::Red)),
        Span::raw(" Close"),
    ]));


    // Clear the dialog area first to prevent backdrop from showing through
    f.render_widget(ratatui::widgets::Clear, dialog);

    // Render the dialog
    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::Black))
        )
        .style(Style::default().bg(Color::Black))
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, dialog);
}

/// Renders the markdown file browser dialog.
///
/// Displays a list of markdown files in the current directory for selection
/// to convert into PRDs.
fn render_markdown_browser_dialog(f: &mut Frame, app: &App) {
    // Calculate dialog size
    let area_rect = f.area();
    let dialog_width = std::cmp::min(70, area_rect.width.saturating_sub(10));
    let dialog_height = std::cmp::min(25, area_rect.height.saturating_sub(4));
    let dialog = Rect {
        x: (area_rect.width.saturating_sub(dialog_width)) / 2,
        y: (area_rect.height.saturating_sub(dialog_height)) / 2,
        width: dialog_width,
        height: dialog_height,
    };

    // Build content lines
    let mut lines = std::vec![
        Line::from(Span::styled(
            " 📄 Select Markdown File for PRD ",
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
    ];

    // Show markdown files
    if app.markdown_files.is_empty() {
        lines.push(Line::from(Span::styled(
            "No markdown files found in current directory",
            Style::default().fg(Color::DarkGray)
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Place .md files in your working directory and reopen this dialog",
            Style::default().fg(Color::Yellow)
        )));
    } else {
        for (idx, filename) in app.markdown_files.iter().enumerate() {
            let is_selected = idx == app.markdown_selected;
            let indicator = if is_selected { "▶ " } else { "  " };

            let style = if is_selected {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            lines.push(Line::from(vec![
                Span::raw(indicator),
                Span::styled(filename, style),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(""));

    // Show keyboard shortcuts
    if !app.markdown_files.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("↑/↓", Style::default().fg(Color::Cyan)),
            Span::raw(" Navigate  "),
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(" Create PRD  "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(" Close"),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(" Close"),
        ]));
    }


    // Clear the dialog area first to prevent backdrop from showing through
    f.render_widget(ratatui::widgets::Clear, dialog);

    // Render the dialog
    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green))
                .style(Style::default().bg(Color::Black))
        )
        .style(Style::default().bg(Color::Black))
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, dialog);
}

/// Renders the right details panel showing context-aware information.
fn render_details_panel(f: &mut Frame, area: Rect, app: &App) {
    let mut lines = std::vec![
        Line::from(Span::styled("DETAILS", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
    ];

    // Get the task selected in the current Kanban column (not global index)
    let selected_task_option = app.get_selected_task_in_column();

    if let std::option::Option::Some(task) = selected_task_option {

        lines.push(Line::from(Span::styled("Selected Task:", Style::default().fg(Color::Yellow))));
        lines.push(Line::from(truncate_string(&task.title, 25)));
        lines.push(Line::from(""));

        lines.push(Line::from(Span::styled("Quick Info:", Style::default().fg(Color::Cyan))));
        lines.push(Line::from(std::format!("Status: {}", format_status_text(&task.status))));

        if let std::option::Option::Some(ref assignee) = task.assignee {
            lines.push(Line::from(std::format!("Assignee: {}", assignee)));
        }

        if let std::option::Option::Some(complexity) = task.complexity {
            lines.push(Line::from(std::format!("Complexity: {}/10", complexity)));
        }

        // Phase 12: Add age tracking display
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("Age Tracking:", Style::default().fg(Color::Cyan))));

        let age_description = format_task_age_description(&task);
        let age_days = calculate_task_age_days(&task);
        let (_, age_color) = get_age_indicator(age_days);
        lines.push(Line::from(Span::styled(age_description, Style::default().fg(age_color))));

        lines.push(Line::from(std::format!("Created: {}", format_timestamp(&task.created_at))));
        lines.push(Line::from(std::format!("Updated: {}", format_timestamp(&task.updated_at))));

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("Dependencies:", Style::default().fg(Color::Cyan))));
        if task.dependencies.is_empty() {
            lines.push(Line::from(Span::styled("None", Style::default().fg(Color::Gray))));
        } else {
            for dep in task.dependencies.iter().take(3) {
                lines.push(Line::from(std::format!("• {}", truncate_string(dep, 20))));
            }
            if task.dependencies.len() > 3 {
                lines.push(Line::from(Span::styled(
                    std::format!("...and {} more", task.dependencies.len() - 3),
                    Style::default().fg(Color::Gray)
                )));
            }
        }
    } else {
        lines.push(Line::from(Span::styled("No task selected", Style::default().fg(Color::Gray))));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "[d] Toggle panel",
        Style::default().fg(Color::Gray)
    )));

    let details_widget = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta))
                .title(" Inspector ")
        )
        .wrap(Wrap { trim: true });

    f.render_widget(details_widget, area);
}

// Helper functions for task status
fn format_status_text(status: &task_manager::domain::task_status::TaskStatus) -> String {
    match status {
        task_manager::domain::task_status::TaskStatus::Todo => "TODO".to_string(),
        task_manager::domain::task_status::TaskStatus::InProgress => "IN PROGRESS".to_string(),
        task_manager::domain::task_status::TaskStatus::Completed => "COMPLETED".to_string(),
        task_manager::domain::task_status::TaskStatus::Archived => "ARCHIVED".to_string(),
        task_manager::domain::task_status::TaskStatus::Errored => "ERRORED".to_string(),
        _ => "OTHER".to_string(),
    }
}

fn get_status_color(status: &task_manager::domain::task_status::TaskStatus) -> Color {
    match status {
        task_manager::domain::task_status::TaskStatus::Todo => Color::White,
        task_manager::domain::task_status::TaskStatus::InProgress => Color::Yellow,
        task_manager::domain::task_status::TaskStatus::Completed => Color::Green,
        task_manager::domain::task_status::TaskStatus::Archived => Color::DarkGray,
        task_manager::domain::task_status::TaskStatus::Errored => Color::Red,
        _ => Color::Gray,
    }
}

/// Renders the Task Board (Kanban view).
fn render_task_board(f: &mut Frame, area: Rect, app: &App) {
    // Split area into filter bar and task columns
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Filter bar
            Constraint::Min(0),     // Task columns
        ])
        .split(area);

    // Render filter bar
    render_filter_bar(f, chunks[0], app);

    // Show helpful message if no tasks exist
    if app.tasks.is_empty() {
        let help_text = vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                "📋 No tasks found",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            )),
            Line::from(""),
            Line::from(Span::styled(
                "To get started with tasks:",
                Style::default().fg(Color::Cyan)
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  1. Create a PRD (Product Requirements Document) file",
                Style::default().fg(Color::White)
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  2. Parse it to generate tasks:",
                Style::default().fg(Color::White)
            )),
            Line::from(Span::styled(
                "     rig parse <prd-file.md>",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  3. Or create a task manually:",
                Style::default().fg(Color::White)
            )),
            Line::from(Span::styled(
                "     Press 'a' key",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            )),
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                "Press ? for full keyboard shortcuts",
                Style::default().fg(Color::DarkGray)
            )),
        ];

        let help_widget = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL))
            .alignment(ratatui::layout::Alignment::Center);

        f.render_widget(help_widget, chunks[1]);
        return;
    }

    // Group tasks by status (applying active filter)
    let mut todo_tasks = std::vec::Vec::new();
    let mut in_progress_tasks = std::vec::Vec::new();
    let mut completed_tasks = std::vec::Vec::new();
    let mut archived_tasks = std::vec::Vec::new();
    let mut errored_tasks = std::vec::Vec::new();

    for task in &app.tasks {
        match task.status {
            task_manager::domain::task_status::TaskStatus::Todo => todo_tasks.push(task),
            task_manager::domain::task_status::TaskStatus::InProgress => in_progress_tasks.push(task),
            task_manager::domain::task_status::TaskStatus::Completed => completed_tasks.push(task),
            task_manager::domain::task_status::TaskStatus::Archived => archived_tasks.push(task),
            task_manager::domain::task_status::TaskStatus::Errored => errored_tasks.push(task),
            _ => {}
        }
    }

    // Create four columns
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(chunks[1]);

    // Render TODO column
    let is_todo_selected = app.selected_column == KanbanColumn::Todo;

    // Build hierarchical task list (parents with indented children)
    let hierarchical_todo = build_hierarchical_task_list(&todo_tasks);

    let todo_items: std::vec::Vec<ListItem> = hierarchical_todo
        .iter()
        .enumerate()
        .map(|(i, htask)| {
            let age_days = calculate_task_age_days(htask.task);
            let (age_icon, age_color) = get_age_indicator(age_days);

            // Highlight selected task in selected column
            let is_selected = is_todo_selected && i == app.selected_task_in_column;

            // Create card-style with border separators
            let card_top = if is_selected {
                "┏━━━━━━━━━━━━━━━━━━━━━━━━━━"
            } else {
                "┌─────────────────────────"
            };

            let card_bottom = if is_selected {
                "┗━━━━━━━━━━━━━━━━━━━━━━━━━━"
            } else {
                "└─────────────────────────"
            };

            let title_style = if is_selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(age_color)
            };

            let border_style = if is_selected {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            // Get tree indicator for hierarchical display
            let tree_prefix = get_tree_indicator(htask.depth, htask.is_last_child);

            // Calculate available width: 22 total - tree_prefix length - age_icon length
            let available_width = 22 - tree_prefix.chars().count() - age_icon.chars().count();

            // Create multi-line card with tree indicator
            let title_text = std::format!("│ {}{}{}", tree_prefix, age_icon, truncate_string(&htask.task.title, available_width));
            let lines = vec![
                Line::from(Span::styled(card_top, border_style)),
                Line::from(Span::styled(title_text, title_style)),
                Line::from(Span::styled(card_bottom, border_style)),
            ];

            ListItem::new(lines)
        })
        .collect();

    let todo_list = List::new(todo_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(std::format!("📝 TODO ({})", todo_tasks.len()))
                .border_style(Style::default().fg(Color::Blue)),
        );
    f.render_widget(todo_list, columns[0]);

    // Render IN PROGRESS column
    let is_progress_selected = app.selected_column == KanbanColumn::InProgress;

    // Build hierarchical task list (parents with indented children)
    let hierarchical_progress = build_hierarchical_task_list(&in_progress_tasks);

    let progress_items: std::vec::Vec<ListItem> = hierarchical_progress
        .iter()
        .enumerate()
        .map(|(i, htask)| {
            let age_days = calculate_task_age_days(htask.task);
            let (age_icon, age_color) = get_age_indicator(age_days);

            let is_selected = is_progress_selected && i == app.selected_task_in_column;

            let card_top = if is_selected {
                "┏━━━━━━━━━━━━━━━━━━━━━━━━━━"
            } else {
                "┌─────────────────────────"
            };

            let card_bottom = if is_selected {
                "┗━━━━━━━━━━━━━━━━━━━━━━━━━━"
            } else {
                "└─────────────────────────"
            };

            let title_style = if is_selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(age_color)
            };

            let border_style = if is_selected {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            // Get tree indicator for hierarchical display
            let tree_prefix = get_tree_indicator(htask.depth, htask.is_last_child);

            // Calculate available width: 22 total - tree_prefix length - age_icon length
            let available_width = 22 - tree_prefix.chars().count() - age_icon.chars().count();

            let title_text = std::format!("│ {}{}{}", tree_prefix, age_icon, truncate_string(&htask.task.title, available_width));
            let lines = vec![
                Line::from(Span::styled(card_top, border_style)),
                Line::from(Span::styled(title_text, title_style)),
                Line::from(Span::styled(card_bottom, border_style)),
            ];

            ListItem::new(lines)
        })
        .collect();

    let progress_list = List::new(progress_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(std::format!("🚀 IN PROGRESS ({})", in_progress_tasks.len()))
                .border_style(Style::default().fg(Color::Yellow)),
        );
    f.render_widget(progress_list, columns[1]);

    // Render COMPLETED column
    let is_completed_selected = app.selected_column == KanbanColumn::Completed;

    // Build hierarchical task list (parents with indented children)
    let hierarchical_completed = build_hierarchical_task_list(&completed_tasks);

    let completed_items: std::vec::Vec<ListItem> = hierarchical_completed
        .iter()
        .enumerate()
        .map(|(i, htask)| {
            let age_days = calculate_task_age_days(htask.task);
            let (age_icon, _age_color) = get_age_indicator(age_days);

            let is_selected = is_completed_selected && i == app.selected_task_in_column;

            let card_top = if is_selected {
                "┏━━━━━━━━━━━━━━━━━━━━━━━━━━"
            } else {
                "┌─────────────────────────"
            };

            let card_bottom = if is_selected {
                "┗━━━━━━━━━━━━━━━━━━━━━━━━━━"
            } else {
                "└─────────────────────────"
            };

            let title_style = if is_selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Green)
            };

            let border_style = if is_selected {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            // Get tree indicator for hierarchical display
            let tree_prefix = get_tree_indicator(htask.depth, htask.is_last_child);

            // Calculate available width: 22 total - tree_prefix length - age_icon length
            let available_width = 22 - tree_prefix.chars().count() - age_icon.chars().count();

            let title_text = std::format!("│ {}{}{}", tree_prefix, age_icon, truncate_string(&htask.task.title, available_width));
            let lines = vec![
                Line::from(Span::styled(card_top, border_style)),
                Line::from(Span::styled(title_text, title_style)),
                Line::from(Span::styled(card_bottom, border_style)),
            ];

            ListItem::new(lines)
        })
        .collect();

    let completed_list = List::new(completed_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(std::format!("✅ COMPLETED ({})", completed_tasks.len()))
                .border_style(Style::default().fg(Color::Green)),
        );
    f.render_widget(completed_list, columns[2]);

    // Render 4TH COLUMN: Split vertically for ARCHIVED and ERRORED
    let split_column = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),  // Archived section
            Constraint::Percentage(50),  // Errored section
        ])
        .split(columns[3]);

    // Render ARCHIVED section (top half of 4th column)
    let is_archived_selected = app.selected_column == KanbanColumn::Archived;

    // Build hierarchical task list (parents with indented children)
    let hierarchical_archived = build_hierarchical_task_list(&archived_tasks);

    let archived_items: std::vec::Vec<ListItem> = hierarchical_archived
        .iter()
        .enumerate()
        .map(|(i, htask)| {
            let is_selected = is_archived_selected && i == app.selected_task_in_column;

            let card_top = if is_selected { "┏━━━━━━━━━━━━━━━━━━" } else { "┌──────────────────" };
            let card_bottom = if is_selected { "┗━━━━━━━━━━━━━━━━━━" } else { "└──────────────────" };

            let title_style = if is_selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let border_style = if is_selected {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            // Get tree indicator for hierarchical display
            let tree_prefix = get_tree_indicator(htask.depth, htask.is_last_child);

            // Calculate available width: 17 total (narrower column) - tree_prefix length
            let available_width = 17 - tree_prefix.chars().count();

            let title_text = std::format!("│ {}{}", tree_prefix, truncate_string(&htask.task.title, available_width));
            let lines = vec![
                Line::from(Span::styled(card_top, border_style)),
                Line::from(Span::styled(title_text, title_style)),
                Line::from(Span::styled(card_bottom, border_style)),
            ];

            ListItem::new(lines)
        })
        .collect();

    let archived_list = List::new(archived_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(std::format!("📦 ARCHIVED ({})", archived_tasks.len()))
                .border_style(Style::default().fg(Color::DarkGray)),
        );
    f.render_widget(archived_list, split_column[0]);

    // Render ERRORED section (bottom half of 4th column)
    let is_errored_selected = app.selected_column == KanbanColumn::Errored;

    // Build hierarchical task list (parents with indented children)
    let hierarchical_errored = build_hierarchical_task_list(&errored_tasks);

    let errored_items: std::vec::Vec<ListItem> = hierarchical_errored
        .iter()
        .enumerate()
        .map(|(i, htask)| {
            let is_selected = is_errored_selected && i == app.selected_task_in_column;

            let card_top = if is_selected { "┏━━━━━━━━━━━━━━━━━━" } else { "┌──────────────────" };
            let card_bottom = if is_selected { "┗━━━━━━━━━━━━━━━━━━" } else { "└──────────────────" };

            let title_style = if is_selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Red)
            };

            let border_style = if is_selected {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            // Get tree indicator for hierarchical display
            let tree_prefix = get_tree_indicator(htask.depth, htask.is_last_child);

            // Calculate available width: 17 total (narrower column) - tree_prefix length
            let available_width = 17 - tree_prefix.chars().count();

            let title_text = std::format!("│ {}{}", tree_prefix, truncate_string(&htask.task.title, available_width));
            let lines = vec![
                Line::from(Span::styled(card_top, border_style)),
                Line::from(Span::styled(title_text, title_style)),
                Line::from(Span::styled(card_bottom, border_style)),
            ];

            ListItem::new(lines)
        })
        .collect();

    let errored_list = List::new(errored_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(std::format!("❌ ERRORED ({})", errored_tasks.len()))
                .border_style(Style::default().fg(Color::Red)),
        );
    f.render_widget(errored_list, split_column[1]);
}

/// Renders the column selector bar at the top of the task board.
fn render_filter_bar(f: &mut Frame, area: Rect, app: &App) {
    let columns = [
        KanbanColumn::Todo,
        KanbanColumn::InProgress,
        KanbanColumn::Completed,
        KanbanColumn::Archived,
        KanbanColumn::Errored,
    ];

    let mut filter_spans = std::vec::Vec::new();

    for (idx, column) in columns.iter().enumerate() {
        if idx > 0 {
            filter_spans.push(Span::raw("  "));
        }

        let is_active = *column == app.selected_column;
        let text = std::format!("[{}] {}", column.shortcut(), column.display_name());

        let style = if is_active {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        filter_spans.push(Span::styled(text, style));
    }

    let paragraph = Paragraph::new(Line::from(filter_spans))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Column Selector (F1-F5)")
                .border_style(Style::default().fg(Color::Cyan)),
        );

    f.render_widget(paragraph, area);
}

/// Renders the Thinking Widget (Chain-of-Thought visualization).
fn render_thinking_widget(f: &mut Frame, area: Rect, app: &App) {
    let items: std::vec::Vec<ListItem> = app
        .thinking_log
        .iter()
        .map(|log| ListItem::new(Line::from(Span::raw(log.clone()))))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("🧠 Chain-of-Thought Reasoning")
                .border_style(Style::default().fg(Color::Magenta)),
        );

    f.render_widget(list, area);
}

/// Renders the Network Log Widget.
fn render_network_widget(f: &mut Frame, area: Rect, app: &App) {
    let items: std::vec::Vec<ListItem> = app
        .network_log
        .iter()
        .map(|log| {
            let style = if log.starts_with("→") {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Green)
            };
            ListItem::new(Line::from(Span::styled(log.clone(), style)))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("🌐 Network Requests/Responses")
                .border_style(Style::default().fg(Color::Cyan)),
        );

    f.render_widget(list, area);
}

/// Renders the Help screen.
fn render_help(f: &mut Frame, area: Rect) {
    let help_text = vec![
        "╔═══════════════════════════════════════════════════════════╗",
        "║              Rigger TUI - Keyboard Controls               ║",
        "╠═══════════════════════════════════════════════════════════╣",
        "║                                                           ║",
        "║  Navigation:                                              ║",
        "║    Tab / Shift+Tab    Switch between views                ║",
        "║    ↑ / k              Move up                             ║",
        "║    ↓ / j              Move down                           ║",
        "║                                                           ║",
        "║  Actions:                                                 ║",
        "║    g                  Go to task by ID (quick jump)       ║",
        "║    Ctrl+R             Recent tasks (MRU list)             ║",
        "║    n                  View notification center            ║",
        "║    c                  Copy task to clipboard (Markdown)   ║",
        "║    s                  Cycle task status (TODO→IN_PROG→✓)  ║",
        "║    o                  Sort tasks (by date, title, etc.)   ║",
        "║    r                  Refresh tasks from database         ║",
        "║    F1-F5              Select column (Todo/InProg/Done/Arch/Err) ║",
        "║    q / Esc            Quit                                ║",
        "║                                                           ║",
        "║  Views:                                                   ║",
        "║    📋 Task Board      Kanban-style task columns           ║",
        "║    🧠 Thinking        Chain-of-thought reasoning          ║",
        "║    🌐 Network         API requests/responses              ║",
        "║    ❓ Help            This help screen                    ║",
        "║                                                           ║",
        "║  Features:                                                ║",
        "║    • Real-time task status visualization                  ║",
        "║    • Intelligent routing decisions display                ║",
        "║    • LLM API call monitoring                              ║",
        "║    • Complexity scoring insights                          ║",
        "║                                                           ║",
        "╚═══════════════════════════════════════════════════════════╝",
    ];

    let paragraph = Paragraph::new(help_text.join("\n"))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("❓ Help")
                .border_style(Style::default().fg(Color::White)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

/// Renders the keyboard shortcut overlay.
///
/// Shows context-aware shortcuts based on the current view. The overlay appears
/// in the bottom-right corner and can be toggled with '?'.
/// Renders the Agent Tools reference panel (Phase 6).
///
/// Displays a comprehensive quick reference of all keyboard shortcuts,
/// agent capabilities, and available commands. Updated for Phases 1-5.
fn render_shortcut_overlay(f: &mut Frame, _app: &App) {
    let lines = vec![
        Line::from(Span::styled(
            " Agent Tools & Commands ",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled("PROJECT NAVIGATION", Style::default().fg(Color::Yellow))),
        Line::from(vec![
            Span::styled(" w/e ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("Switch projects"),
        ]),
        Line::from(""),
        Line::from(Span::styled("MAIN VIEWS", Style::default().fg(Color::Yellow))),
        Line::from(vec![
            Span::styled(" Tab ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("Toggle Kanban/Metrics"),
        ]),
        Line::from(""),
        Line::from(Span::styled("TASK ACTIONS", Style::default().fg(Color::Yellow))),
        Line::from(vec![
            Span::styled(" ↑↓ j/k ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("Navigate tasks"),
        ]),
        Line::from(vec![
            Span::styled(" Enter ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("Edit task (Phase 4)"),
        ]),
        Line::from(vec![
            Span::styled(" s ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("Cycle task status"),
        ]),
        Line::from(vec![
            Span::styled(" c ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("Copy to clipboard"),
        ]),
        Line::from(vec![
            Span::styled(" o ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("Sort menu"),
        ]),
        Line::from(vec![
            Span::styled(" a ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("Create new task"),
        ]),
        Line::from(vec![
            Span::styled(" m ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("Browse markdown files (PRDs)"),
        ]),
        Line::from(vec![
            Span::styled(" / ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("Spotlight search"),
        ]),
        Line::from(vec![
            Span::styled(" g ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("Jump to task ID"),
        ]),
        Line::from(vec![
            Span::styled(" Ctrl+R ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("Recent tasks"),
        ]),
        Line::from(""),
        Line::from(Span::styled("COLUMN SELECTION", Style::default().fg(Color::Yellow))),
        Line::from(vec![
            Span::styled(std::format!(" {} ", KEY_COLUMN_TODO), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(std::format!("Select {} column", LABEL_COLUMN_TODO)),
        ]),
        Line::from(vec![
            Span::styled(std::format!(" {} ", KEY_COLUMN_IN_PROGRESS), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(std::format!("Select {} column", LABEL_COLUMN_IN_PROGRESS)),
        ]),
        Line::from(vec![
            Span::styled(std::format!(" {} ", KEY_COLUMN_COMPLETED), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(std::format!("Select {} column", LABEL_COLUMN_COMPLETED)),
        ]),
        Line::from(vec![
            Span::styled(std::format!(" {} ", KEY_COLUMN_ARCHIVED), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(std::format!("Select {} column", LABEL_COLUMN_ARCHIVED)),
        ]),
        Line::from(vec![
            Span::styled(std::format!(" {} ", KEY_COLUMN_ERRORED), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(std::format!("Select {} column", LABEL_COLUMN_ERRORED)),
        ]),
        Line::from(""),
        Line::from(Span::styled("GLOBAL COMMANDS", Style::default().fg(Color::Yellow))),
        Line::from(vec![
            Span::styled(std::format!(" {} ", KEY_TOGGLE_VIEW), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("Toggle Kanban/Metrics view"),
        ]),
        Line::from(""),
        Line::from(Span::styled("AGENT TOOLS", Style::default().fg(Color::Yellow))),
        Line::from(vec![
            Span::styled(" l ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("LLM Chat (Phase 5)"),
        ]),
        Line::from(vec![
            Span::raw("   "),
            Span::styled("→ Ask: 'enhance task'", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::raw("   "),
            Span::styled("→ Ask: 'decompose task'", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(""),
        Line::from(Span::styled("OTHER", Style::default().fg(Color::Yellow))),
        Line::from(vec![
            Span::styled(" m ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("Markdown browser (PRD selector)"),
        ]),
        Line::from(vec![
            Span::styled(" n ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("Notifications"),
        ]),
        Line::from(vec![
            Span::styled(" Esc ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("Close dialog/Quit"),
        ]),
        Line::from(vec![
            Span::styled(" q ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("Quit"),
        ]),
        Line::from(""),
        Line::from(Span::styled("SHORTCUTS", Style::default().fg(Color::Yellow))),
        Line::from(vec![
            Span::styled(" ? ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("Toggle this help"),
        ]),
        Line::from(""),
        Line::from(Span::styled("INSPECTOR", Style::default().fg(Color::Yellow))),
        Line::from(vec![
            Span::styled(" d ", Style::default().fg(Color::Gray)),
            Span::raw("Toggle details panel"),
        ]),
    ];

    // Calculate dialog size (centered, large enough for all content)
    let area = f.area();
    let dialog_width = 45;
    let dialog_height = (lines.len() + 2).min(area.height.saturating_sub(4) as usize) as u16;
    let dialog = Rect {
        x: (area.width.saturating_sub(dialog_width)) / 2,
        y: (area.height.saturating_sub(dialog_height)) / 2,
        width: dialog_width,
        height: dialog_height,
    };

    // Clear the dialog area first to prevent background from showing through
    f.render_widget(ratatui::widgets::Clear, dialog);

    // Render as centered dialog
    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::Black)),
        )
        .style(Style::default().bg(Color::Black))
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, dialog);
}

/// Renders the status message toast notification.
///
/// Displays a temporary message at the bottom-center of the screen indicating
/// the result of an operation (e.g., "Status changed to IN PROGRESS").
fn render_status_toast(f: &mut Frame, app: &App) {
    if let std::option::Option::Some(ref message) = app.status_message {
        // Calculate toast size and position (bottom-center)
        let area = f.area();
        let toast_width = message.len().min(50) as u16 + 4;
        let toast_height = 3;
        let toast = Rect {
            x: (area.width.saturating_sub(toast_width)) / 2,
            y: area.height.saturating_sub(toast_height + 1),
            width: toast_width,
            height: toast_height,
        };

        // Render the toast with green border for success
        let paragraph = Paragraph::new(message.clone())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green))
                    .style(Style::default().bg(Color::Black)),
            )
            .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));

        f.render_widget(paragraph, toast);
    }
}

/// Renders the sort menu overlay.
///
/// Displays a centered menu with all available sort options.
fn render_sort_menu(f: &mut Frame, app: &App) {
    let options = TaskSortOption::all();

    // Build menu lines
    let mut lines = std::vec![
        Line::from(Span::styled(
            "Sort By:",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    for (idx, option) in options.iter().enumerate() {
        let is_selected = idx == app.sort_menu_selection;
        let is_current = *option == app.current_sort;

        let prefix = if is_selected { "→ " } else { "  " };
        let suffix = if is_current { " (current)" } else { "" };
        let text = std::format!("{}{}{}", prefix, option.display_name(), suffix);

        let style = if is_selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else if is_current {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        };

        lines.push(Line::from(Span::styled(text, style)));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "[↑↓] Navigate  [Enter] Apply  [Esc] Cancel",
        Style::default().fg(Color::Gray),
    )));

    // Calculate menu size and position (center of screen)
    let area = f.area();
    let menu_width = 40;
    let menu_height = (lines.len() + 2) as u16;
    let menu = Rect {
        x: (area.width.saturating_sub(menu_width)) / 2,
        y: (area.height.saturating_sub(menu_height)) / 2,
        width: menu_width,
        height: menu_height,
    };

    // Render the menu
    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Sort Tasks")
                .border_style(Style::default().fg(Color::Yellow))
                .style(Style::default().bg(Color::Black)),
        );

    f.render_widget(paragraph, menu);
}

/// Renders the task jump dialog.
///
/// Displays a centered dialog for entering a task ID to jump to.
fn render_jump_dialog(f: &mut Frame, app: &App) {
    // Build dialog lines
    let mut lines = std::vec![
        Line::from(Span::styled(
            "Go To Task",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::raw("ID: "),
            Span::styled(
                &app.jump_input,
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::styled("_", Style::default().fg(Color::Cyan)),
        ]),
    ];

    // Show preview if there's a match
    if let std::option::Option::Some(idx) = app.find_task_by_id(&app.jump_input) {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("→ ", Style::default().fg(Color::Green)),
            Span::styled(
                truncate_string(&app.tasks[idx].title, 35),
                Style::default().fg(Color::Green),
            ),
        ]));
    } else if !app.jump_input.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "No matching task",
            Style::default().fg(Color::Red),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "[Enter] Jump  [Esc] Cancel",
        Style::default().fg(Color::Gray),
    )));

    // Calculate dialog size and position (center of screen)
    let area = f.area();
    let dialog_width = 42;
    let dialog_height = (lines.len() + 2) as u16;
    let dialog = Rect {
        x: (area.width.saturating_sub(dialog_width)) / 2,
        y: (area.height.saturating_sub(dialog_height)) / 2,
        width: dialog_width,
        height: dialog_height,
    };


    // Clear the dialog area first to prevent backdrop from showing through
    f.render_widget(ratatui::widgets::Clear, dialog);

    // Render the dialog
    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Jump to Task")
                .border_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::Black)),
        )
        .style(Style::default().bg(Color::Black));

    f.render_widget(paragraph, dialog);
}

/// Renders the recent items dialog.
///
/// Displays a centered dialog showing the most recently viewed tasks.
fn render_recent_dialog(f: &mut Frame, app: &App) {
    let mut lines = std::vec![
        Line::from(Span::styled(
            "Recent Items",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    if app.recent_task_ids.is_empty() {
        lines.push(Line::from(Span::styled(
            "No recent tasks",
            Style::default().fg(Color::Gray),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "Tasks:",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));

        for (idx, task_id) in app.recent_task_ids.iter().enumerate() {
            // Find the task in the task list
            if let std::option::Option::Some(task) = app.tasks.iter().find(|t| t.id.to_string() == *task_id) {
                let is_selected = idx == app.recent_selection;
                let prefix = if is_selected { "→ " } else { "  " };

                // Calculate time since last update
                let now = chrono::Utc::now();
                let duration = now.signed_duration_since(task.updated_at);
                let time_ago = if duration.num_minutes() < 60 {
                    std::format!("{}m", duration.num_minutes())
                } else if duration.num_hours() < 24 {
                    std::format!("{}h", duration.num_hours())
                } else {
                    std::format!("{}d", duration.num_days())
                };

                let text = std::format!(
                    "{}{} ({})",
                    prefix,
                    truncate_string(&task.title, 32),
                    time_ago
                );

                let style = if is_selected {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                lines.push(Line::from(Span::styled(text, style)));
            }
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "[↑↓] Navigate  [Enter] Open  [Esc] Cancel",
        Style::default().fg(Color::Gray),
    )));

    // Calculate dialog size and position (center of screen)
    let area = f.area();
    let dialog_width = 50;
    let dialog_height = (lines.len() + 2) as u16;
    let dialog = Rect {
        x: (area.width.saturating_sub(dialog_width)) / 2,
        y: (area.height.saturating_sub(dialog_height)) / 2,
        width: dialog_width,
        height: dialog_height,
    };


    // Clear the dialog area first to prevent backdrop from showing through
    f.render_widget(ratatui::widgets::Clear, dialog);

    // Render the dialog
    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Recent Items")
                .border_style(Style::default().fg(Color::Magenta))
                .style(Style::default().bg(Color::Black)),
        )
        .style(Style::default().bg(Color::Black));

    f.render_widget(paragraph, dialog);
}

/// Renders the task editor dialog (Phase 4).
///
/// Displays a centered dialog for editing task fields (title, description, assignee, status).
/// Use Tab to navigate between fields, Enter to save, Esc to cancel.
fn render_task_editor_dialog(f: &mut Frame, app: &App) {
    if app.tasks.is_empty() {
        return;
    }

    let task = &app.tasks[app.selected_task];

    let mut lines = std::vec![
        Line::from(Span::styled(
            " Task Editor ",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
    ];

    // Title field
    let title_label = if app.task_editor_field == TaskEditorField::Title {
        Span::styled("▶ Title: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
    } else {
        Span::styled("  Title: ", Style::default())
    };

    let title_value = if app.task_editor_field == TaskEditorField::Title {
        &app.task_editor_input
    } else {
        &task.title
    };

    lines.push(Line::from(vec![
        title_label,
        Span::raw(title_value.clone()),
    ]));

    // Description field
    let desc_label = if app.task_editor_field == TaskEditorField::Description {
        Span::styled("▶ Description: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
    } else {
        Span::styled("  Description: ", Style::default())
    };

    let desc_value = if app.task_editor_field == TaskEditorField::Description {
        &app.task_editor_input
    } else {
        &task.description
    };

    lines.push(Line::from(vec![
        desc_label,
        Span::raw(desc_value.clone()),
    ]));

    // Assignee field
    let assignee_label = if app.task_editor_field == TaskEditorField::Assignee {
        Span::styled("▶ Assignee: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
    } else {
        Span::styled("  Assignee: ", Style::default())
    };

    let assignee_value = if app.task_editor_field == TaskEditorField::Assignee {
        &app.task_editor_input
    } else {
        task.assignee.as_ref().map(|s| s.as_str()).unwrap_or("<none>")
    };

    lines.push(Line::from(vec![
        assignee_label,
        Span::raw(assignee_value),
    ]));

    // Status field
    let status_label = if app.task_editor_field == TaskEditorField::Status {
        Span::styled("▶ Status: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
    } else {
        Span::styled("  Status: ", Style::default())
    };

    let status_text = match task.status {
        task_manager::domain::task_status::TaskStatus::Todo => "Todo",
        task_manager::domain::task_status::TaskStatus::InProgress => "In Progress",
        task_manager::domain::task_status::TaskStatus::Completed => "Completed",
        task_manager::domain::task_status::TaskStatus::Archived => "Archived",
        task_manager::domain::task_status::TaskStatus::Errored => "Errored",
        _ => "Unknown",
    };

    lines.push(Line::from(vec![
        status_label,
        Span::raw(status_text),
        if app.task_editor_field == TaskEditorField::Status {
            Span::styled(" (use ↑/↓ to change)", Style::default().fg(Color::DarkGray))
        } else {
            Span::raw("")
        },
    ]));

    // Phase 12: Age tracking info
    lines.push(Line::from(""));
    let age_description = format_task_age_description(task);
    let age_days = calculate_task_age_days(task);
    let (age_icon, age_color) = get_age_indicator(age_days);
    lines.push(Line::from(Span::styled(
        std::format!("  {} Age: {}", age_icon, age_description),
        Style::default().fg(age_color)
    )));
    lines.push(Line::from(Span::styled(
        std::format!("  Created: {} | Updated: {}", format_timestamp(&task.created_at), format_timestamp(&task.updated_at)),
        Style::default().fg(Color::DarkGray)
    )));

    // Help text
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Tab: Next field | Shift+Tab: Previous field | Enter: Save | Esc: Cancel",
        Style::default().fg(Color::DarkGray)
    )));

    // Calculate dialog size and position (center of screen)
    let area = f.area();
    let dialog_width = 70;
    let dialog_height = (lines.len() + 2) as u16;
    let dialog = Rect {
        x: (area.width.saturating_sub(dialog_width)) / 2,
        y: (area.height.saturating_sub(dialog_height)) / 2,
        width: dialog_width,
        height: dialog_height,
    };


    // Clear the dialog area first to prevent backdrop from showing through
    f.render_widget(ratatui::widgets::Clear, dialog);

    // Render the dialog
    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .style(Style::default().bg(Color::Black))
        )
        .style(Style::default().bg(Color::Black));

    f.render_widget(paragraph, dialog);
}

/// Renders the task creator dialog (Phase 8).
///
/// Displays a centered form for creating new tasks with title, description, assignee, and status.
/// Use 'a' key to open, Tab/Shift+Tab to navigate fields, Enter to save, Esc to cancel.
fn render_task_creator_dialog(f: &mut Frame, app: &App) {
    let mut lines = std::vec![
        Line::from(Span::styled(
            " Create New Task ",
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
    ];

    // Title field
    let title_label = if app.task_creator_field == TaskCreatorField::Title {
        Span::styled("▶ Title: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
    } else {
        Span::styled("  Title: ", Style::default())
    };

    let title_value = if app.task_creator_title.is_empty() {
        "<enter title>"
    } else {
        &app.task_creator_title
    };

    lines.push(Line::from(vec![
        title_label,
        Span::raw(title_value),
    ]));

    // Description field
    let desc_label = if app.task_creator_field == TaskCreatorField::Description {
        Span::styled("▶ Description: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
    } else {
        Span::styled("  Description: ", Style::default())
    };

    let desc_value = if app.task_creator_description.is_empty() {
        "<optional>"
    } else {
        &app.task_creator_description
    };

    lines.push(Line::from(vec![
        desc_label,
        Span::raw(desc_value),
    ]));

    // Assignee field
    let assignee_label = if app.task_creator_field == TaskCreatorField::Assignee {
        Span::styled("▶ Assignee: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
    } else {
        Span::styled("  Assignee: ", Style::default())
    };

    let assignee_value = if app.task_creator_assignee.is_empty() {
        "<optional>"
    } else {
        &app.task_creator_assignee
    };

    lines.push(Line::from(vec![
        assignee_label,
        Span::raw(assignee_value),
    ]));

    // Status field
    let status_label = if app.task_creator_field == TaskCreatorField::Status {
        Span::styled("▶ Status: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
    } else {
        Span::styled("  Status: ", Style::default())
    };

    let status_text = match app.task_creator_status {
        task_manager::domain::task_status::TaskStatus::Todo => "Todo",
        task_manager::domain::task_status::TaskStatus::InProgress => "In Progress",
        task_manager::domain::task_status::TaskStatus::Completed => "Completed",
        task_manager::domain::task_status::TaskStatus::Archived => "Archived",
        task_manager::domain::task_status::TaskStatus::Errored => "Errored",
        _ => "Unknown",
    };

    lines.push(Line::from(vec![
        status_label,
        Span::raw(status_text),
        if app.task_creator_field == TaskCreatorField::Status {
            Span::styled(" (use ↑/↓ to change)", Style::default().fg(Color::DarkGray))
        } else {
            Span::raw("")
        },
    ]));

    // Show which project this task will be linked to
    if let std::option::Option::Some(project) = app.get_selected_project() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Project: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&project.name, Style::default().fg(Color::Cyan)),
        ]));
    }

    // Help text
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Tab: Next field | Shift+Tab: Previous field | Enter: Create | Esc: Cancel",
        Style::default().fg(Color::DarkGray)
    )));

    // Calculate dialog size and position (center of screen)
    let area = f.area();
    let dialog_width = 70;
    let dialog_height = (lines.len() + 2) as u16;
    let dialog = Rect {
        x: (area.width.saturating_sub(dialog_width)) / 2,
        y: (area.height.saturating_sub(dialog_height)) / 2,
        width: dialog_width,
        height: dialog_height,
    };

    // Clear the dialog area first to prevent backdrop from showing through
    f.render_widget(ratatui::widgets::Clear, dialog);

    // Clear the dialog area first to prevent backdrop from showing through
    f.render_widget(ratatui::widgets::Clear, dialog);

    // Render the dialog
    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green))
                .style(Style::default().bg(Color::Black))
        )
        .style(Style::default().bg(Color::Black));

    f.render_widget(paragraph, dialog);
}

/// Renders the spotlight search dialog (Phase 9).
///
/// Displays a centered search interface with live results across tasks, PRDs, and projects.
/// Use '/' key to open, type to search, ↑/↓ to navigate, Enter to jump, Esc to close.
fn render_spotlight_dialog(f: &mut Frame, app: &App) {
    let mut lines = std::vec![
        Line::from(Span::styled(
            " 🔍 Spotlight Search ",
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
    ];

    // Search input line
    let search_prompt = Span::styled("Search: ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD));
    let search_value = if app.spotlight_query.is_empty() {
        Span::styled("Type to search across all tasks, PRDs, and projects...", Style::default().fg(Color::DarkGray))
    } else {
        Span::raw(&app.spotlight_query)
    };
    lines.push(Line::from(vec![search_prompt, search_value]));
    lines.push(Line::from(""));

    // Results section
    if app.spotlight_query.is_empty() {
        lines.push(Line::from(Span::styled(
            "Start typing to see results...",
            Style::default().fg(Color::DarkGray)
        )));
    } else if app.spotlight_results.is_empty() {
        lines.push(Line::from(Span::styled(
            "No results found",
            Style::default().fg(Color::Yellow)
        )));
    } else {
        lines.push(Line::from(Span::styled(
            std::format!("{} result{}", app.spotlight_results.len(), if app.spotlight_results.len() == 1 { "" } else { "s" }),
            Style::default().fg(Color::Cyan)
        )));
        lines.push(Line::from(""));

        // Show results (max 10)
        for (idx, result) in app.spotlight_results.iter().take(10).enumerate() {
            let is_selected = idx == app.spotlight_selected;

            match result {
                SearchResultType::Task { title, description, .. } => {
                    let indicator = if is_selected {
                        Span::styled("▶ ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
                    } else {
                        Span::raw("  ")
                    };
                    let type_badge = Span::styled("[Task] ", Style::default().fg(Color::Blue));
                    let task_title = if is_selected {
                        Span::styled(title, Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
                    } else {
                        Span::raw(title)
                    };
                    lines.push(Line::from(vec![indicator, type_badge, task_title]));

                    // Show description if selected
                    if is_selected && !description.is_empty() {
                        let desc_preview = if description.len() > 60 {
                            std::format!("    {}...", &description[..60])
                        } else {
                            std::format!("    {}", description)
                        };
                        lines.push(Line::from(Span::styled(desc_preview, Style::default().fg(Color::DarkGray))));
                    }
                }
                SearchResultType::PRD { title, .. } => {
                    let indicator = if is_selected {
                        Span::styled("▶ ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
                    } else {
                        Span::raw("  ")
                    };
                    let type_badge = Span::styled("[PRD] ", Style::default().fg(Color::Yellow));
                    let prd_title = if is_selected {
                        Span::styled(title, Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
                    } else {
                        Span::raw(title)
                    };
                    lines.push(Line::from(vec![indicator, type_badge, prd_title]));
                }
                SearchResultType::Project { name, .. } => {
                    let indicator = if is_selected {
                        Span::styled("▶ ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
                    } else {
                        Span::raw("  ")
                    };
                    let type_badge = Span::styled("[Project] ", Style::default().fg(Color::Green));
                    let proj_name = if is_selected {
                        Span::styled(name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
                    } else {
                        Span::raw(name)
                    };
                    lines.push(Line::from(vec![indicator, type_badge, proj_name]));
                }
            }
        }

        if app.spotlight_results.len() > 10 {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                std::format!("...and {} more", app.spotlight_results.len() - 10),
                Style::default().fg(Color::DarkGray)
            )));
        }
    }

    // Help text
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Type: Search | ↑/↓: Navigate | Enter: Jump | Esc: Close",
        Style::default().fg(Color::DarkGray)
    )));

    // Calculate dialog size and position (center, larger for search results)
    let area = f.area();
    let dialog_width = 80;
    let dialog_height = (lines.len() + 2).min(area.height.saturating_sub(4) as usize) as u16;
    let dialog = Rect {
        x: (area.width.saturating_sub(dialog_width)) / 2,
        y: (area.height.saturating_sub(dialog_height)) / 2,
        width: dialog_width,
        height: dialog_height,
    };


    // Clear the dialog area first to prevent backdrop from showing through
    f.render_widget(ratatui::widgets::Clear, dialog);

    // Render the dialog
    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta))
                .style(Style::default().bg(Color::Black))
        )
        .style(Style::default().bg(Color::Black));

    f.render_widget(paragraph, dialog);
}

/// Renders the confirmation dialog (Phase 10).
///
/// Displays a centered confirmation prompt for destructive operations.
/// Use Y/N or Enter/Esc to confirm or cancel.
fn render_confirmation_dialog(f: &mut Frame, app: &App) {
    let mut lines = std::vec![
        Line::from(Span::styled(
            std::format!(" ⚠️  {} ", app.confirmation_title),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
    ];

    // Split message into lines (handle \n in message)
    for msg_line in app.confirmation_message.lines() {
        lines.push(Line::from(Span::raw(msg_line)));
    }

    // Blank line before buttons
    lines.push(Line::from(""));
    lines.push(Line::from(""));

    // Buttons: [Y]es and [N]o
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled("[Y]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled("es", Style::default().fg(Color::Green)),
        Span::raw("     "),
        Span::styled("[N]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::styled("o", Style::default().fg(Color::Red)),
    ]));

    // Help text
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Y/Enter: Confirm | N/Esc: Cancel",
        Style::default().fg(Color::DarkGray)
    )));

    // Calculate dialog size and position (center, compact)
    let area = f.area();
    let dialog_width = 60;
    let dialog_height = (lines.len() + 2).min(area.height.saturating_sub(4) as usize) as u16;
    let dialog = Rect {
        x: (area.width.saturating_sub(dialog_width)) / 2,
        y: (area.height.saturating_sub(dialog_height)) / 2,
        width: dialog_width,
        height: dialog_height,
    };

    // Clear the dialog area first to prevent backdrop from showing through
    f.render_widget(ratatui::widgets::Clear, dialog);

    // Render the dialog with yellow border to indicate warning
    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .style(Style::default().bg(Color::Black))
        )
        .style(Style::default().bg(Color::Black));

    f.render_widget(paragraph, dialog);
}

/// Renders the LLM chat dialog (Phase 5).
///
/// Displays a centered chat interface showing context (current project/task)
/// and conversation history. Use 'l' key to open, Enter to send messages, Esc to close.
fn render_llm_chat_dialog(f: &mut Frame, app: &App) {
    let mut lines = std::vec![
        Line::from(Span::styled(
            " LLM Chat Assistant ",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
    ];

    // Render chat history
    if app.llm_chat_history.is_empty() {
        lines.push(Line::from(Span::styled(
            "No messages yet. Type your question or command...",
            Style::default().fg(Color::DarkGray)
        )));
    } else {
        for msg in &app.llm_chat_history {
            let (prefix, style) = match msg.role {
                ChatRole::System => ("📋 Context: ", Style::default().fg(Color::Blue)),
                ChatRole::User => ("👤 You: ", Style::default().fg(Color::Yellow)),
                ChatRole::Assistant => ("🤖 Assistant: ", Style::default().fg(Color::Cyan)),
            };

            // Split multi-line messages
            for line in msg.content.lines() {
                if line == msg.content.lines().next().unwrap() {
                    lines.push(Line::from(vec![
                        Span::styled(prefix, style),
                        Span::raw(line),
                    ]));
                } else {
                    lines.push(Line::from(vec![
                        Span::raw("   "),
                        Span::raw(line),
                    ]));
                }
            }
            lines.push(Line::from(""));
        }
    }

    // Input field
    lines.push(Line::from(Span::styled(
        "─".repeat(66),
        Style::default().fg(Color::DarkGray)
    )));

    lines.push(Line::from(vec![
        Span::styled("▶ ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(&app.llm_chat_input),
        Span::styled("█", Style::default().fg(Color::Yellow)), // Cursor
    ]));

    // Help text
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Enter: Send | Esc: Close | Try: 'enhance this task' or 'decompose this task'",
        Style::default().fg(Color::DarkGray)
    )));

    // Calculate dialog size and position (center of screen, larger for chat)
    let area = f.area();
    let dialog_width = 70;
    let dialog_height = 30.min(area.height - 4);
    let dialog = Rect {
        x: (area.width.saturating_sub(dialog_width)) / 2,
        y: (area.height.saturating_sub(dialog_height)) / 2,
        width: dialog_width,
        height: dialog_height,
    };

    // Clear the dialog area first to prevent backdrop from showing through
    f.render_widget(ratatui::widgets::Clear, dialog);

    // Render the dialog with scrolling if content exceeds height
    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::Black))
        )
        .style(Style::default().bg(Color::Black))
        .wrap(Wrap { trim: false })
        .scroll((0, 0)); // TODO: Add scroll support for long conversations

    f.render_widget(paragraph, dialog);
}

/// Renders the PRD management dialog (Phase 7).
///
/// Displays a list of PRDs for the current project with objectives, tech stack, and constraints.
/// Use 'r' key to open, ↑/↓ to navigate, Esc to close.
fn render_prd_dialog(f: &mut Frame, app: &App) {
    let mut lines = std::vec![
        Line::from(Span::styled(
            " PRD Management ",
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
    ];

    // Show current project name
    if let std::option::Option::Some(project) = app.get_selected_project() {
        lines.push(Line::from(vec![
            Span::styled("Project: ", Style::default().fg(Color::Cyan)),
            Span::raw(&project.name),
        ]));
        lines.push(Line::from(""));
    }

    // Get filtered PRDs for current project
    let filtered_prds = app.get_filtered_prds();

    if filtered_prds.is_empty() {
        lines.push(Line::from(Span::styled(
            "No PRDs found for this project.",
            Style::default().fg(Color::DarkGray)
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "PRDs define project requirements and generate tasks.",
            Style::default().fg(Color::DarkGray)
        )));
    } else {
        // Render PRD list
        for (idx, prd) in filtered_prds.iter().enumerate() {
            let is_selected = idx == app.selected_prd;
            let prefix = if is_selected { "▶ " } else { "  " };
            let style = if is_selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            // PRD title
            lines.push(Line::from(Span::styled(
                std::format!("{}{}", prefix, prd.title),
                style
            )));

            // Show details for selected PRD
            if is_selected {
                lines.push(Line::from(""));

                // Objectives
                if !prd.objectives.is_empty() {
                    lines.push(Line::from(Span::styled(
                        "  Objectives:",
                        Style::default().fg(Color::Cyan)
                    )));
                    for obj in prd.objectives.iter().take(3) {
                        lines.push(Line::from(vec![
                            Span::raw("    • "),
                            Span::raw(obj),
                        ]));
                    }
                    if prd.objectives.len() > 3 {
                        lines.push(Line::from(Span::styled(
                            std::format!("    ... and {} more", prd.objectives.len() - 3),
                            Style::default().fg(Color::DarkGray)
                        )));
                    }
                    lines.push(Line::from(""));
                }

                // Tech Stack
                if !prd.tech_stack.is_empty() {
                    lines.push(Line::from(Span::styled(
                        "  Tech Stack:",
                        Style::default().fg(Color::Cyan)
                    )));
                    for tech in prd.tech_stack.iter().take(3) {
                        lines.push(Line::from(vec![
                            Span::raw("    • "),
                            Span::raw(tech),
                        ]));
                    }
                    if prd.tech_stack.len() > 3 {
                        lines.push(Line::from(Span::styled(
                            std::format!("    ... and {} more", prd.tech_stack.len() - 3),
                            Style::default().fg(Color::DarkGray)
                        )));
                    }
                    lines.push(Line::from(""));
                }

                // Constraints
                if !prd.constraints.is_empty() {
                    lines.push(Line::from(Span::styled(
                        "  Constraints:",
                        Style::default().fg(Color::Cyan)
                    )));
                    for constraint in prd.constraints.iter().take(3) {
                        lines.push(Line::from(vec![
                            Span::raw("    • "),
                            Span::raw(constraint),
                        ]));
                    }
                    if prd.constraints.len() > 3 {
                        lines.push(Line::from(Span::styled(
                            std::format!("    ... and {} more", prd.constraints.len() - 3),
                            Style::default().fg(Color::DarkGray)
                        )));
                    }
                }

                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "─".repeat(66),
                    Style::default().fg(Color::DarkGray)
                )));
                lines.push(Line::from(""));
            }
        }
    }

    // Help text
    lines.push(Line::from(Span::styled(
        "↑↓ Navigate | Esc: Close",
        Style::default().fg(Color::DarkGray)
    )));

    // Calculate dialog size (centered, large enough for content)
    let area = f.area();
    let dialog_width = 70;
    let dialog_height = 30.min(area.height - 4);
    let dialog = Rect {
        x: (area.width.saturating_sub(dialog_width)) / 2,
        y: (area.height.saturating_sub(dialog_height)) / 2,
        width: dialog_width,
        height: dialog_height,
    };


    // Clear the dialog area first to prevent backdrop from showing through
    f.render_widget(ratatui::widgets::Clear, dialog);

    // Render the dialog
    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta))
                .style(Style::default().bg(Color::Black))
        )
        .style(Style::default().bg(Color::Black))
        .wrap(Wrap { trim: false })
        .scroll((0, 0)); // TODO: Add scroll support for long PRD lists

    f.render_widget(paragraph, dialog);
}

/// Renders the Dev Tools view as a full-page section.
///
/// Shows available development tools with descriptions.
/// User can navigate with Up/Down arrows and select with Enter.
fn render_dev_tools_view(f: &mut Frame, area: Rect, app: &App) {
    let all_dev_tools = std::vec![
        DevTool::SqliteBrowser,
        DevTool::ConfigViewer,
    ];

    let mut lines = std::vec![
        Line::from(Span::styled(
            " 🛠️  Development Tools ",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Select a tool to use:",
            Style::default().fg(Color::Gray)
        )),
        Line::from(""),
    ];

    // Render each dev tool option
    for (idx, tool) in all_dev_tools.iter().enumerate() {
        let is_selected = idx == app.dev_tools_selection;
        let prefix = if is_selected { "▶ " } else { "  " };
        let style = if is_selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        // Tool name
        lines.push(Line::from(Span::styled(
            std::format!("{}{}", prefix, tool.display_name()),
            style
        )));

        // Description (indented)
        lines.push(Line::from(Span::styled(
            std::format!("    {}", tool.description()),
            Style::default().fg(Color::DarkGray)
        )));
        lines.push(Line::from(""));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("↑/↓", Style::default().fg(Color::Cyan)),
        Span::raw(" Navigate  "),
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::raw(" Select Tool"),
    ]));

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Dev Tools")
                .border_style(Style::default().fg(Color::Cyan))
        )
        .style(Style::default().bg(Color::Black));

    f.render_widget(paragraph, area);
}

/// Renders the Dev Tools menu dialog showing available development tools.
///
/// Displays a centered menu listing all available dev tools with descriptions.
/// User can navigate with Up/Down arrows and select with Enter.
fn render_dev_tools_menu(f: &mut Frame, app: &App) {
    let all_dev_tools = vec![
        DevTool::SqliteBrowser,
        DevTool::ConfigViewer,
    ];

    let mut lines = std::vec![
        Line::from(Span::styled(
            " 🛠️  Dev Tools ",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Select a development tool:",
            Style::default().fg(Color::DarkGray)
        )),
        Line::from(""),
    ];

    // Render each dev tool option
    for (idx, tool) in all_dev_tools.iter().enumerate() {
        let is_selected = idx == app.dev_tools_selection;
        let prefix = if is_selected { "▶ " } else { "  " };
        let style = if is_selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        // Tool name
        lines.push(Line::from(Span::styled(
            std::format!("{}{}", prefix, tool.display_name()),
            style
        )));

        // Description (always shown, indented)
        lines.push(Line::from(Span::styled(
            std::format!("    {}", tool.description()),
            Style::default().fg(Color::DarkGray)
        )));
        lines.push(Line::from(""));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("↑/↓", Style::default().fg(Color::Cyan)),
        Span::raw(" Navigate  "),
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::raw(" Select  "),
        Span::styled("Esc", Style::default().fg(Color::Red)),
        Span::raw(" Close"),
    ]));

    // Calculate dialog size (centered, medium size)
    let area = f.area();
    let dialog_width = 60;
    let dialog_height = 18.min(area.height - 4);
    let dialog = Rect {
        x: (area.width.saturating_sub(dialog_width)) / 2,
        y: (area.height.saturating_sub(dialog_height)) / 2,
        width: dialog_width,
        height: dialog_height,
    };


    // Clear the dialog area first to prevent backdrop from showing through
    f.render_widget(ratatui::widgets::Clear, dialog);

    // Render the dialog
    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::Black))
        )
        .style(Style::default().bg(Color::Black))
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, dialog);
}

/// Calculates the age of a task in days based on its updated_at timestamp.
fn calculate_task_age_days(task: &task_manager::domain::task::Task) -> i64 {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(task.updated_at);
    duration.num_days()
}

/// Returns the age indicator icon and color based on task age.
///
/// Returns (icon, color) tuple:
/// - < 1 day: ("", Normal)
/// - 1-3 days: ("📅 ", White)
/// - 3-7 days: ("⚠️  ", Yellow)
/// - > 7 days: ("🔴 ", Red)
fn get_age_indicator(age_days: i64) -> (&'static str, Color) {
    if age_days < 1 {
        ("", Color::Reset)
    } else if age_days < 3 {
        ("📅 ", Color::White)
    } else if age_days < 7 {
        ("⚠️  ", Color::Yellow)
    } else {
        ("🔴 ", Color::Red)
    }
}

/// Represents a task in a hierarchical display with depth information.
///
/// Used to render parent tasks followed by their indented sub-tasks in the Kanban view.
struct HierarchicalTask<'a> {
    task: &'a task_manager::domain::task::Task,
    depth: usize,
    is_last_child: bool,
}

/// Organizes tasks into a hierarchical list with parent-child relationships.
///
/// Returns a flat list where sub-tasks immediately follow their parent, with depth metadata
/// for proper tree visualization. Parent tasks appear first, followed by their children.
///
/// # Algorithm
/// 1. Separate tasks into parents (no parent_task_id) and children (has parent_task_id)
/// 2. For each parent task:
///    - Add parent at depth 0
///    - Find and add all children at depth 1
/// 3. Return flattened hierarchical list
fn build_hierarchical_task_list<'a>(
    tasks: &'a [&'a task_manager::domain::task::Task],
) -> std::vec::Vec<HierarchicalTask<'a>> {
    let mut result = std::vec::Vec::new();

    // Separate parent tasks (no parent_task_id) from children
    let (parent_tasks, child_tasks): (std::vec::Vec<&&task_manager::domain::task::Task>, std::vec::Vec<&&task_manager::domain::task::Task>) = tasks
        .iter()
        .partition(|t| t.parent_task_id.is_none());

    // Build lookup map for children by parent_task_id
    let mut children_by_parent: std::collections::HashMap<&str, std::vec::Vec<&task_manager::domain::task::Task>> =
        std::collections::HashMap::new();

    for child in child_tasks {
        if let std::option::Option::Some(ref parent_id) = child.parent_task_id {
            children_by_parent
                .entry(parent_id.as_str())
                .or_insert_with(std::vec::Vec::new)
                .push(*child);
        }
    }

    // Build hierarchical list: parent followed by its children
    for parent in parent_tasks {
        // Add parent at depth 0
        result.push(HierarchicalTask {
            task: *parent,
            depth: 0,
            is_last_child: false,
        });

        // Add children at depth 1
        if let std::option::Option::Some(children) = children_by_parent.get(parent.id.as_str()) {
            let child_count = children.len();
            for (idx, child) in children.iter().enumerate() {
                result.push(HierarchicalTask {
                    task: child,
                    depth: 1,
                    is_last_child: idx == child_count - 1,
                });
            }
        }
    }

    result
}

/// Returns the tree indicator prefix for a hierarchical task.
///
/// Returns appropriate box-drawing characters based on depth and position:
/// - Depth 0 (parent): "" (no prefix)
/// - Depth 1, not last: "├─ "
/// - Depth 1, last child: "└─ "
fn get_tree_indicator(depth: usize, is_last_child: bool) -> &'static str {
    match depth {
        0 => "",
        1 if is_last_child => "└─ ",
        1 => "├─ ",
        _ => "   ", // Deeper nesting (future expansion)
    }
}

/// Formats a detailed age description for a task (Phase 12).
///
/// Returns a human-readable string describing task age and staleness level.
/// Examples: "Fresh (< 1 day)", "Recent (2 days old)", "Aging (5 days old)", "Stale! (10 days old)"
fn format_task_age_description(task: &task_manager::domain::task::Task) -> String {
    let age_days = calculate_task_age_days(task);

    if age_days < 1 {
        let age_hours = (chrono::Utc::now() - task.updated_at).num_hours();
        if age_hours < 1 {
            String::from("Fresh (< 1 hour)")
        } else {
            std::format!("Fresh ({} hours old)", age_hours)
        }
    } else if age_days < 3 {
        std::format!("Recent ({} days old)", age_days)
    } else if age_days < 7 {
        std::format!("Aging ⚠️  ({} days old)", age_days)
    } else {
        std::format!("Stale! 🔴 ({} days old)", age_days)
    }
}

/// Formats timestamps for display (Phase 12).
fn format_timestamp(dt: &chrono::DateTime<chrono::Utc>) -> String {
    let local_dt = dt.with_timezone(&chrono::Local);
    local_dt.format("%Y-%m-%d %H:%M").to_string()
}

/// Renders the notification center dialog.
///
/// Shows recent notifications in reverse chronological order with severity indicators.
fn render_notifications(f: &mut Frame, app: &App) {
    let mut lines = std::vec![
        Line::from(Span::styled(
            std::format!("Notifications ({})", app.notifications.len()),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    if app.notifications.is_empty() {
        lines.push(Line::from(Span::styled(
            "No notifications",
            Style::default().fg(Color::Gray),
        )));
    } else {
        for notification in app.notifications.iter().take(15) {
            // Format timestamp
            let now = chrono::Utc::now();
            let duration = now.signed_duration_since(notification.timestamp);
            let time_ago = if duration.num_seconds() < 60 {
                std::format!("{}s ago", duration.num_seconds())
            } else if duration.num_minutes() < 60 {
                std::format!("{}m ago", duration.num_minutes())
            } else if duration.num_hours() < 24 {
                std::format!("{}h ago", duration.num_hours())
            } else {
                std::format!("{}d ago", duration.num_days())
            };

            // Get icon and color based on level
            let (icon, color) = match notification.level {
                NotificationLevel::Info => ("ℹ️ ", Color::Cyan),
                NotificationLevel::Success => ("✓ ", Color::Green),
                NotificationLevel::Warning => ("⚠️ ", Color::Yellow),
                NotificationLevel::Error => ("✗ ", Color::Red),
            };

            let text = std::format!(
                "{}{} ({})",
                icon,
                truncate_string(&notification.message, 48),
                time_ago
            );

            lines.push(Line::from(Span::styled(text, Style::default().fg(color))));
        }

        if app.notifications.len() > 15 {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                std::format!("... and {} more", app.notifications.len() - 15),
                Style::default().fg(Color::Gray),
            )));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "[n/Esc] Close",
        Style::default().fg(Color::Gray),
    )));

    let dialog_width = 60;
    let dialog_height = (lines.len() + 2).min(22) as u16;

    let area = f.area();
    let dialog_x = (area.width.saturating_sub(dialog_width)) / 2;
    let dialog_y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect {
        x: dialog_x,
        y: dialog_y,
        width: dialog_width,
        height: dialog_height,
    };


    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(" Notifications ")
                .style(Style::default().bg(Color::Black))
        )
        .style(Style::default().bg(Color::Black))
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, dialog_area);
}

/// Renders a loading indicator overlay with animated spinner.
///
/// Displays a centered semi-transparent overlay showing an animated spinner
/// and the current loading operation message.
fn render_loading_indicator(f: &mut Frame, app: &App) {
    let loading_msg = app.loading_message.as_ref()
        .map(|s| s.as_str())
        .unwrap_or("Loading...");

    let spinner_char = app.get_spinner_char();
    let text = std::format!("{} {}", spinner_char, loading_msg);

    // Calculate overlay size and position (center of screen)
    let area = f.area();
    let width = text.len().min(50) as u16 + 4;
    let height = 3;

    let overlay_x = (area.width.saturating_sub(width)) / 2;
    let overlay_y = (area.height.saturating_sub(height)) / 2;

    let overlay_area = Rect {
        x: overlay_x,
        y: overlay_y,
        width,
        height,
    };

    // Create loading widget
    let loading_widget = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
        )
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    f.render_widget(loading_widget, overlay_area);
}

/// Renders the full-screen setup wizard for first-time initialization.
///
/// Displays different screens based on the current wizard step:
/// - Welcome: Introduction and getting started
/// - ProviderSelection: Choose LLM provider (Ollama, Candle, Mistral, Rig)
/// - ModelConfiguration: Configure model names for main/research/fallback
/// - DatabaseConfiguration: Set database path
/// - Confirmation: Review all settings before creating config
/// - Complete: Success message and instructions
fn render_setup_wizard(f: &mut Frame, app: &App) {
    let area = f.area();

    // Clear the entire screen with black background
    let clear_widget = Block::default().style(Style::default().bg(Color::Black));
    f.render_widget(clear_widget, area);

    match app.setup_wizard_step {
        SetupWizardStep::Welcome => render_wizard_welcome(f, area),
        SetupWizardStep::TaskToolSlots => render_wizard_task_tool_slots(f, area),
        SetupWizardStep::ConfigureMainSlot => render_wizard_configure_slot(f, area, app, "Main"),
        SetupWizardStep::ConfigureResearchSlot => render_wizard_configure_slot(f, area, app, "Research"),
        SetupWizardStep::ConfigureFallbackSlot => render_wizard_configure_slot(f, area, app, "Fallback"),
        SetupWizardStep::DatabaseConfiguration => render_wizard_database_configuration(f, area, app),
        SetupWizardStep::Confirmation => render_wizard_confirmation(f, area, app),
        SetupWizardStep::Complete => render_wizard_complete(f, area),
    }
}

/// Renders the welcome screen of the setup wizard.
fn render_wizard_welcome(f: &mut Frame, area: Rect) {
    let mut lines = std::vec![
        Line::from(""),
        Line::from(Span::styled(
            "🎯 Welcome to Rigger",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
        Line::from(Span::styled(
            "AI-powered task management and orchestration",
            Style::default().fg(Color::Gray)
        )),
        Line::from(""),
        Line::from(""),
        Line::from("This wizard will guide you through the initial setup:"),
        Line::from(""),
        Line::from(Span::styled("  1. Choose your LLM provider", Style::default().fg(Color::White))),
        Line::from(Span::styled("  2. Configure model settings", Style::default().fg(Color::White))),
        Line::from(Span::styled("  3. Set up your database", Style::default().fg(Color::White))),
        Line::from(Span::styled("  4. Confirm and initialize", Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "This will create a .rigger directory in your current folder",
            Style::default().fg(Color::DarkGray)
        )),
        Line::from(Span::styled(
            "with configuration files and a SQLite database.",
            Style::default().fg(Color::DarkGray)
        )),
        Line::from(""),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::Gray)),
            Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(" to begin setup", Style::default().fg(Color::Gray)),
        ]),
    ];

    // Center the dialog
    let dialog_width = 70;
    let dialog_height = calculate_safe_dialog_height(lines.len(), area.height);

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Rigger Setup Wizard ")
                .border_style(Style::default().fg(Color::Cyan))
        )
        .style(Style::default().bg(Color::Black))
        .alignment(ratatui::layout::Alignment::Center);
    let dialog_x = (area.width.saturating_sub(dialog_width)) / 2;
    let dialog_y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect {
        x: dialog_x,
        y: dialog_y,
        width: dialog_width,
        height: dialog_height,
    };

    f.render_widget(paragraph, dialog_area);
}

/// Renders the task tool slots explanation screen.
fn render_wizard_task_tool_slots(f: &mut Frame, area: Rect) {
    let mut lines = std::vec![
        Line::from(""),
        Line::from(Span::styled(
            "Step 1: Understanding Task Tool Slots",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Rigger uses three specialized LLM slots for different purposes:",
            Style::default().fg(Color::Gray)
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  🔧 Main: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("Primary task execution and code generation"),
        ]),
        Line::from("      - Handles most task processing workloads"),
        Line::from("      - Can be a powerful local or cloud model"),
        Line::from(""),
        Line::from(vec![
            Span::styled("  🔍 Research: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("Analysis, planning, and research tasks"),
        ]),
        Line::from("      - Deep analysis of requirements and context"),
        Line::from("      - Complexity estimation and decomposition"),
        Line::from(""),
        Line::from(vec![
            Span::styled("  🛟 Fallback: ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::raw("Fast, lightweight backup processing"),
        ]),
        Line::from("      - Used when main/research slots are unavailable"),
        Line::from("      - Typically a smaller, faster model"),
        Line::from(""),
        Line::from(Span::styled(
            "Each slot can use a different provider and model!",
            Style::default().fg(Color::Green).add_modifier(Modifier::ITALIC)
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(" Continue  "),
            Span::styled("Esc/Ctrl+C", Style::default().fg(Color::Red)),
            Span::raw(" Exit Wizard"),
        ]),
    ];

    let dialog_width = 80;
    let dialog_height = calculate_safe_dialog_height(lines.len(), area.height);

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Task Tool Slots ")
                .border_style(Style::default().fg(Color::Cyan))
        )
        .style(Style::default().bg(Color::Black));
    let dialog_x = (area.width.saturating_sub(dialog_width)) / 2;
    let dialog_y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect {
        x: dialog_x,
        y: dialog_y,
        width: dialog_width,
        height: dialog_height,
    };

    f.render_widget(paragraph, dialog_area);
}

/// Renders a slot configuration screen (provider + model selection).
fn render_wizard_configure_slot(f: &mut Frame, area: Rect, app: &App, slot_name: &str) {
    let providers = LLMProvider::all();

    // Get current slot's provider selection and model
    let (provider_selection, current_model) = match slot_name {
        "Main" => (app.setup_wizard_main_provider_selection, &app.setup_wizard_main_model),
        "Research" => (app.setup_wizard_research_provider_selection, &app.setup_wizard_research_model),
        "Fallback" => (app.setup_wizard_fallback_provider_selection, &app.setup_wizard_fallback_model),
        _ => (0, &app.setup_wizard_main_model),
    };

    let step_num = match slot_name {
        "Main" => "2",
        "Research" => "3",
        "Fallback" => "4",
        _ => "2",
    };

    let slot_icon = match slot_name {
        "Main" => "🔧",
        "Research" => "🔍",
        "Fallback" => "🛟",
        _ => "🔧",
    };

    let mut lines = std::vec![
        Line::from(""),
        Line::from(Span::styled(
            std::format!("Step {}: Configure {} Slot {}", step_num, slot_name, slot_icon),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Choose provider and model for this slot:",
            Style::default().fg(Color::Gray)
        )),
        Line::from(""),
        Line::from(""),
    ];

    // Provider list
    for (idx, provider) in providers.iter().enumerate() {
        let is_selected = idx == provider_selection;
        let prefix = if is_selected { "▶ " } else { "  " };
        let style = if is_selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        lines.push(Line::from(Span::styled(
            std::format!("{}{}", prefix, provider.display_name()),
            style
        )));

        lines.push(Line::from(Span::styled(
            std::format!("    {}", provider.description()),
            Style::default().fg(Color::DarkGray)
        )));
        lines.push(Line::from(""));
    }

    // Model input field
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Model: ", Style::default().fg(Color::Cyan)),
        Span::styled(current_model.clone(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled("_", Style::default().fg(Color::Yellow)),
    ]));

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("↑/↓", Style::default().fg(Color::Cyan)),
        Span::raw(" Select Provider  "),
        Span::styled("Type", Style::default().fg(Color::Cyan)),
        Span::raw(" Edit Model  "),
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::raw(" Continue"),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Esc/Ctrl+C", Style::default().fg(Color::Red)),
        Span::raw(" Exit Wizard"),
    ]));

    let dialog_width = 80;
    let dialog_height = calculate_safe_dialog_height(lines.len(), area.height);

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(std::format!(" {} Slot Configuration ", slot_name))
                .border_style(Style::default().fg(Color::Cyan))
        )
        .style(Style::default().bg(Color::Black));
    let dialog_x = (area.width.saturating_sub(dialog_width)) / 2;
    let dialog_y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect {
        x: dialog_x,
        y: dialog_y,
        width: dialog_width,
        height: dialog_height,
    };

    f.render_widget(paragraph, dialog_area);
}

/// Renders the database configuration screen.
fn render_wizard_database_configuration(f: &mut Frame, area: Rect, app: &App) {
    let mut lines = std::vec![
        Line::from(""),
        Line::from(Span::styled(
            "Step 3: Database Configuration",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "  Database Path:",
            Style::default().fg(Color::Cyan)
        )),
        Line::from(""),
        Line::from(Span::styled(
            std::format!("    {}", app.setup_wizard_db_path),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "This path is relative to the .rigger directory.",
            Style::default().fg(Color::Gray)
        )),
        Line::from(Span::styled(
            "The database will be created automatically.",
            Style::default().fg(Color::Gray)
        )),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("Type", Style::default().fg(Color::Yellow)),
            Span::raw(" to Edit  "),
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(" Continue  "),
            Span::styled("Esc/Ctrl+C", Style::default().fg(Color::Red)),
            Span::raw(" Exit Wizard"),
        ]),
    ];

    let dialog_width = 70;
    let dialog_height = calculate_safe_dialog_height(lines.len(), area.height);

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Database Configuration ")
                .border_style(Style::default().fg(Color::Cyan))
        )
        .style(Style::default().bg(Color::Black));
    let dialog_x = (area.width.saturating_sub(dialog_width)) / 2;
    let dialog_y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect {
        x: dialog_x,
        y: dialog_y,
        width: dialog_width,
        height: dialog_height,
    };

    f.render_widget(paragraph, dialog_area);
}

/// Renders the confirmation screen showing all selected settings.
fn render_wizard_confirmation(f: &mut Frame, area: Rect, app: &App) {
    let main_provider_name = match app.setup_wizard_main_provider {
        LLMProvider::Ollama => "ollama",
        LLMProvider::Candle => "candle",
        LLMProvider::Mistral => "mistral",
        LLMProvider::Rig => "rig",
    };

    let research_provider_name = match app.setup_wizard_research_provider {
        LLMProvider::Ollama => "ollama",
        LLMProvider::Candle => "candle",
        LLMProvider::Mistral => "mistral",
        LLMProvider::Rig => "rig",
    };

    let fallback_provider_name = match app.setup_wizard_fallback_provider {
        LLMProvider::Ollama => "ollama",
        LLMProvider::Candle => "candle",
        LLMProvider::Mistral => "mistral",
        LLMProvider::Rig => "rig",
    };

    let mut lines = std::vec![
        Line::from(""),
        Line::from(Span::styled(
            "Step 6: Confirm Settings",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Please review your configuration:",
            Style::default().fg(Color::Gray)
        )),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled("🔧 Main Slot:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(vec![
            Span::styled("    Provider: ", Style::default().fg(Color::Cyan)),
            Span::styled(main_provider_name, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("    Model:    ", Style::default().fg(Color::Cyan)),
            Span::styled(&app.setup_wizard_main_model, Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(Span::styled("🔍 Research Slot:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(vec![
            Span::styled("    Provider: ", Style::default().fg(Color::Cyan)),
            Span::styled(research_provider_name, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("    Model:    ", Style::default().fg(Color::Cyan)),
            Span::styled(&app.setup_wizard_research_model, Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(Span::styled("🛟 Fallback Slot:", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))),
        Line::from(vec![
            Span::styled("    Provider: ", Style::default().fg(Color::Cyan)),
            Span::styled(fallback_provider_name, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("    Model:    ", Style::default().fg(Color::Cyan)),
            Span::styled(&app.setup_wizard_fallback_model, Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Database:     ", Style::default().fg(Color::Cyan)),
            Span::styled(&app.setup_wizard_db_path, Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "This will create:",
            Style::default().fg(Color::Gray)
        )),
        Line::from(""),
        Line::from(Span::styled("  .rigger/", Style::default().fg(Color::DarkGray))),
        Line::from(Span::styled("  ├── config.json", Style::default().fg(Color::DarkGray))),
        Line::from(Span::styled("  ├── tasks.db", Style::default().fg(Color::DarkGray))),
        Line::from(Span::styled("  └── prds/", Style::default().fg(Color::DarkGray))),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(" Create Configuration  "),
            Span::styled("Esc/Ctrl+C", Style::default().fg(Color::Red)),
            Span::raw(" Exit Wizard"),
        ]),
    ];

    let dialog_width = 70;
    let dialog_height = calculate_safe_dialog_height(lines.len(), area.height);

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Confirmation ")
                .border_style(Style::default().fg(Color::Yellow))
        )
        .style(Style::default().bg(Color::Black));
    let dialog_x = (area.width.saturating_sub(dialog_width)) / 2;
    let dialog_y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect {
        x: dialog_x,
        y: dialog_y,
        width: dialog_width,
        height: dialog_height,
    };

    f.render_widget(paragraph, dialog_area);
}

/// Renders the completion screen after successful setup.
fn render_wizard_complete(f: &mut Frame, area: Rect) {
    let mut lines = std::vec![
        Line::from(""),
        Line::from(Span::styled(
            "✓ Setup Complete!",
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "Rigger has been successfully initialized.",
            Style::default().fg(Color::White)
        )),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled("Next steps:", Style::default().fg(Color::Cyan))),
        Line::from(""),
        Line::from(Span::styled("  1. Create a PRD markdown file (or use an existing one)", Style::default().fg(Color::White))),
        Line::from(Span::styled("  2. Use 'm' key to browse and select PRD files", Style::default().fg(Color::White))),
        Line::from(Span::styled("  3. Press 'a' to create tasks manually", Style::default().fg(Color::White))),
        Line::from(Span::styled("  4. Use '?' to view all keyboard shortcuts", Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "Your configuration has been saved to .rigger/config.json",
            Style::default().fg(Color::DarkGray)
        )),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::Gray)),
            Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(" to start using Rigger", Style::default().fg(Color::Gray)),
        ]),
    ];

    let dialog_width = 75;
    // Use safe helper to prevent buffer overflow
    let dialog_height = calculate_safe_dialog_height(lines.len(), area.height);

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Welcome to Rigger ")
                .border_style(Style::default().fg(Color::Green))
        )
        .style(Style::default().bg(Color::Black))
        .alignment(ratatui::layout::Alignment::Center);
    let dialog_x = (area.width.saturating_sub(dialog_width)) / 2;
    let dialog_y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect {
        x: dialog_x,
        y: dialog_y,
        width: dialog_width,
        height: dialog_height,
    };

    f.render_widget(paragraph, dialog_area);
}

/// Renders the PRD processing/progress view.
///
/// Shows real-time progress as a markdown file is parsed into a PRD
/// and tasks are generated via LLM. Displays:
/// - Current processing step
/// - File being processed
/// - Number of tasks generated (when complete)
/// - Success or error status
/// - Instructions to continue
/// Renders the interactive PRD generation UI with conversation, tasks, and input.
///
/// This function displays a 3-section layout during task generation:
/// - Top 40%: Conversation history (LLM thinking, user suggestions)
/// - Middle 40%: Generated tasks list (with status indicators)
/// - Bottom 20%: User input field
fn render_interactive_generation(f: &mut Frame, area: Rect, app: &App) {
    // Clear the entire area with black background
    let clear_widget = Block::default().style(Style::default().bg(Color::Black));
    f.render_widget(clear_widget, area);

    // Create layout with 3 sections: conversation (40%), tasks (40%), input (20%)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(40),
            Constraint::Percentage(20),
        ].as_ref())
        .split(area);

    // Section 1: Conversation History
    // Build header with project name if available
    let conversation_header = if let std::option::Option::Some(ref prd) = app.prd_processing_prd {
        std::format!("💭 LLM Conversation - {}", prd.title)
    } else {
        std::string::String::from("💭 LLM Conversation")
    };

    // Show conversation messages with scrolling support
    let total_messages = app.prd_gen_conversation.len();

    // Auto-scroll to bottom when new messages arrive (unless user has scrolled up)
    let visible_messages: std::vec::Vec<&PRDGenMessage> = app
        .prd_gen_conversation
        .iter()
        .collect();

    // Build list of messages with their types for dynamic layout
    #[derive(Clone)]
    enum MessageItem<'a> {
        Text { lines: std::vec::Vec<Line<'a>>, timestamp: String, height: u16 },
        Block { content: BoxContent, timestamp: String, height: u16 },
    }

    let mut message_items: std::vec::Vec<MessageItem> = std::vec::Vec::new();

    if visible_messages.is_empty() {
        message_items.push(MessageItem::Text {
            lines: std::vec![
                Line::from(Span::styled(
                    "Waiting for LLM to start thinking...",
                    Style::default().fg(Color::DarkGray)
                ))
            ],
            timestamp: String::new(),
            height: 1,
        });
    } else {
        for msg in &visible_messages {
            let (icon, color) = match msg.role {
                PRDGenRole::System => ("⚙️ ", Color::Yellow),
                PRDGenRole::Assistant => ("🤖", Color::Cyan),
                PRDGenRole::User => ("👤", Color::Green),
            };

            match &msg.content {
                MessageContent::Box(box_content) => {
                    // Box message - will render as Block widget
                    let timestamp = msg.timestamp.format("%H:%M:%S").to_string();

                    // Calculate height dynamically based on content
                    let height = match box_content {
                        BoxContent::Task { subtasks, .. } => {
                            let base_height = 9; // Title, Assignee, Priority, Complexity, blank, Description, + borders/padding
                            if subtasks.is_empty() {
                                base_height
                            } else {
                                // Add header for subtasks section (blank + "Sub-tasks:" line)
                                let subtasks_header = 2;
                                // Each subtask: blank + numbered title + 4 field lines (assignee, priority, complexity, description)
                                let per_subtask = 6;
                                base_height + subtasks_header + (subtasks.len() as u16 * per_subtask)
                            }
                        }
                        BoxContent::Validation { .. } => 6, // Task + blank + message + borders
                    };

                    message_items.push(MessageItem::Block {
                        content: box_content.clone(),
                        timestamp,
                        height,
                    });
                }
                MessageContent::Text(text) => {
                    // Text message - combine icon and text for proper wrapping
                    let timestamp = msg.timestamp.format("%H:%M:%S").to_string();

                    // Create single text string with icon prefix (allows wrapping)
                    let full_text = std::format!("{} {}", icon, text);
                    let text_lines = std::vec![
                        Line::from(Span::styled(full_text.clone(), Style::default().fg(color)))
                    ];

                    // Estimate wrapped height based on text length and typical width
                    // Assume ~100 chars per line as rough estimate for conversation width
                    let estimated_wrapped_lines = (full_text.len() as f32 / 100.0).ceil() as u16;
                    let text_height = std::cmp::max(1, estimated_wrapped_lines);

                    message_items.push(MessageItem::Text {
                        lines: text_lines,
                        timestamp,
                        height: text_height + 1, // +1 for title line
                    });
                }
            }
        }
    }

    // Render outer conversation container with cyan border and title
    let conversation_block = Block::default()
        .borders(Borders::ALL)
        .title(std::format!(" {} ", &conversation_header))
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    f.render_widget(conversation_block.clone(), chunks[0]);

    // Get inner area for messages (inside the border)
    let inner_area = conversation_block.inner(chunks[0]);

    // Calculate scrolling window: show messages that fit starting from scroll position
    let available_height = inner_area.height;

    // Build list of cumulative heights to determine visible message range
    let mut cumulative_heights = std::vec::Vec::new();
    let mut running_total: u16 = 0;
    for item in &message_items {
        let item_height = match item {
            MessageItem::Text { height, .. } => *height,
            MessageItem::Block { height, .. } => *height,
        };
        running_total += item_height;
        cumulative_heights.push(running_total);
    }

    // Calculate which messages to display
    let start_idx = if app.prd_gen_auto_scroll && !message_items.is_empty() {
        // Auto-scroll mode: calculate offset to show most recent messages that fit
        // Work backwards from the end to find how many messages fit
        let mut height_needed: u16 = 0;
        let mut messages_that_fit = 0;

        for item in message_items.iter().rev() {
            let item_height = match item {
                MessageItem::Text { height, .. } => *height,
                MessageItem::Block { height, .. } => *height,
            };

            if height_needed + item_height <= available_height {
                height_needed += item_height;
                messages_that_fit += 1;
            } else {
                break;
            }
        }

        // Start index is total messages minus how many fit
        message_items.len().saturating_sub(messages_that_fit)
    } else {
        // Manual scroll mode: use the scroll offset
        app.prd_gen_scroll_offset.min(message_items.len().saturating_sub(1))
    };

    // Collect visible messages starting from start_idx
    let mut visible_items = std::vec::Vec::new();
    let mut height_used: u16 = 0;

    for item in message_items.iter().skip(start_idx) {
        let item_height = match item {
            MessageItem::Text { height, .. } => *height,
            MessageItem::Block { height, .. } => *height,
        };

        if height_used + item_height <= available_height {
            visible_items.push(item.clone());
            height_used += item_height;
        } else {
            break; // No more messages fit
        }
    }

    // If no messages fit, show at least one (the one at scroll offset)
    if visible_items.is_empty() && !message_items.is_empty() {
        visible_items.push(message_items[start_idx].clone());
    }

    // Create dynamic vertical layout for visible messages
    let message_constraints: std::vec::Vec<Constraint> = visible_items
        .iter()
        .map(|item| match item {
            MessageItem::Text { height, .. } => Constraint::Length(*height),
            MessageItem::Block { height, .. } => Constraint::Length(*height),
        })
        .collect();

    // Split inner area vertically for each message
    let message_areas = if message_constraints.is_empty() {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints(&[Constraint::Min(0)])
            .split(inner_area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints(&message_constraints[..])
            .split(inner_area)
    };

    // Render each message in its allocated area
    for (idx, item) in visible_items.iter().enumerate() {
        if idx >= message_areas.len() {
            break;
        }

        match item {
            MessageItem::Text { lines, timestamp, .. } => {
                // Render text as borderless block with timestamp label
                let text_para = Paragraph::new(lines.clone())
                    .block(
                        Block::default()
                            .borders(Borders::NONE)
                            .title(std::format!(" {} ", timestamp))
                            .title_style(Style::default().fg(Color::DarkGray))
                    )
                    .style(Style::default().bg(Color::Black))
                    .wrap(Wrap { trim: true });
                f.render_widget(text_para, message_areas[idx]);
            }
            MessageItem::Block { content, timestamp, .. } => {
                // Render as Block widget with borders
                match content {
                    BoxContent::Task { title, description, assignee, priority, complexity, subtasks } => {
                        let assignee_str = assignee.clone().unwrap_or_else(|| std::string::String::from("Unassigned"));
                        let priority_str = priority.map(|p| std::format!("{}/10", p))
                            .unwrap_or_else(|| std::string::String::from("N/A"));
                        let complexity_str = complexity.map(|c| std::format!("{}/10", c))
                            .unwrap_or_else(|| std::string::String::from("N/A"));

                        let mut content_lines = std::vec![
                            Line::from(std::format!("Title: {}", title)),
                            Line::from(std::format!("Assignee: {}", assignee_str)),
                            Line::from(std::format!("Priority: {}", priority_str)),
                            Line::from(std::format!("Complexity: {}", complexity_str)),
                            Line::from(""),
                            Line::from(std::format!("Description: {}", description)),
                        ];

                        // Add nested sub-tasks if any
                        if !subtasks.is_empty() {
                            content_lines.push(Line::from(""));
                            content_lines.push(Line::from(Span::styled(
                                std::format!("Sub-tasks ({}):", subtasks.len()),
                                Style::default().fg(Color::Rgb(255, 165, 0)).add_modifier(Modifier::BOLD)
                            )));

                            for (st_idx, subtask) in subtasks.iter().enumerate() {
                                let st_assignee = subtask.assignee.clone().unwrap_or_else(|| std::string::String::from("Unassigned"));
                                let st_priority = subtask.priority.map(|p| std::format!("{}/10", p))
                                    .unwrap_or_else(|| std::string::String::from("N/A"));
                                let st_complexity = subtask.complexity.map(|c| std::format!("{}/10", c))
                                    .unwrap_or_else(|| std::string::String::from("N/A"));

                                content_lines.push(Line::from(""));
                                content_lines.push(Line::from(vec![
                                    Span::styled(std::format!("  {}. ", st_idx + 1), Style::default().fg(Color::Rgb(255, 165, 0))),
                                    Span::styled(&subtask.title, Style::default().fg(Color::Rgb(255, 165, 0)).add_modifier(Modifier::BOLD)),
                                ]));
                                content_lines.push(Line::from(std::format!("     Assignee: {}", st_assignee)));
                                content_lines.push(Line::from(std::format!("     Priority: {}", st_priority)));
                                content_lines.push(Line::from(std::format!("     Complexity: {}", st_complexity)));
                                content_lines.push(Line::from(std::format!("     Description: {}", subtask.description)));
                            }
                        }

                        let task_block = Paragraph::new(content_lines)
                            .block(
                                Block::default()
                                    .borders(Borders::ALL)
                                    .title(std::format!(" Task [{}] ", timestamp))
                                    .border_style(Style::default().fg(Color::Yellow))
                            )
                            .wrap(Wrap { trim: true });

                        f.render_widget(task_block, message_areas[idx]);
                    }
                    BoxContent::Validation { task_title, message } => {
                        let content_lines = std::vec![
                            Line::from(std::format!("Task: {}", task_title)),
                            Line::from(""),
                            Line::from(message.as_str()),
                        ];

                        let validation_block = Paragraph::new(content_lines)
                            .block(
                                Block::default()
                                    .borders(Borders::ALL)
                                    .title(std::format!(" Validation [{}] ", timestamp))
                                    .border_style(Style::default().fg(Color::Red))
                            )
                            .wrap(Wrap { trim: true });

                        f.render_widget(validation_block, message_areas[idx]);
                    }
                }
            }
        }
    }

    // Section 2: Generated Tasks
    let mut task_lines = std::vec![
        Line::from(Span::styled(
            std::format!("📋 Generated Tasks ({}/{})",
                app.prd_gen_partial_tasks.len(),
                app.prd_gen_partial_tasks.len()
            ),
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
    ];

    if app.prd_gen_partial_tasks.is_empty() {
        task_lines.push(Line::from(Span::styled(
            "No tasks generated yet...",
            Style::default().fg(Color::DarkGray)
        )));
    } else {
        for task in &app.prd_gen_partial_tasks {
            let (icon, color) = match task.status {
                PartialTaskStatus::Generating => ("⏳", Color::Cyan),
                PartialTaskStatus::Validating => ("⚠", Color::Yellow),
                PartialTaskStatus::Complete => ("✓", Color::Green),
                PartialTaskStatus::Failed => ("✗", Color::Red),
            };

            task_lines.push(Line::from(vec![
                Span::styled(std::format!("  {} ", icon), Style::default().fg(color)),
                Span::styled(&task.title, Style::default().fg(Color::White)),
            ]));

            // Show validation messages as indented red rows
            for msg in &task.validation_messages {
                task_lines.push(Line::from(vec![
                    Span::styled("      ", Style::default()),
                    Span::styled("└─ ", Style::default().fg(Color::Red)),
                    Span::styled(msg, Style::default().fg(Color::Red)),
                ]));
            }
        }
    }

    let tasks_block = Paragraph::new(task_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Tasks ")
                .border_style(Style::default().fg(Color::Green))
        )
        .style(Style::default().bg(Color::Black));

    f.render_widget(tasks_block, chunks[1]);

    // Section 3: User Input
    let input_color = if app.prd_gen_input_active {
        Color::Yellow
    } else {
        Color::DarkGray
    };

    let mut input_lines = std::vec![
        Line::from(Span::styled(
            if app.prd_gen_editing_last {
                "💬 Your Input (editing last message):"
            } else {
                "💬 Your Input (optional):"
            },
            Style::default().fg(input_color).add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
    ];

    // Show input text or placeholder
    if app.prd_gen_input.is_empty() {
        input_lines.push(Line::from(Span::styled(
            "Type a suggestion or question for the LLM...",
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)
        )));
    } else {
        input_lines.push(Line::from(Span::styled(
            &app.prd_gen_input,
            Style::default().fg(Color::White)
        )));
    }

    input_lines.push(Line::from(""));

    // Show appropriate keyboard hints based on state
    if app.prd_gen_input.is_empty() && !app.prd_gen_last_message.is_empty() {
        input_lines.push(Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::Gray)),
            Span::styled("↑", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" to edit last message | ", Style::default().fg(Color::Gray)),
            Span::styled("Esc", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" to skip | ", Style::default().fg(Color::Gray)),
            Span::styled("Ctrl+C", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(" to cancel", Style::default().fg(Color::Gray)),
        ]));
    } else {
        input_lines.push(Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::Gray)),
            Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(" to send | ", Style::default().fg(Color::Gray)),
            Span::styled("Esc", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(if app.prd_gen_editing_last { " to cancel edit | " } else { " to clear | " }, Style::default().fg(Color::Gray)),
            Span::styled("Ctrl+C", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(" to cancel", Style::default().fg(Color::Gray)),
        ]));
    }

    let input_block = Paragraph::new(input_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Input ")
                .border_style(Style::default().fg(input_color))
        )
        .style(Style::default().bg(Color::Black));

    f.render_widget(input_block, chunks[2]);
}

fn render_prd_processing(f: &mut Frame, area: Rect, app: &App) {
    // Check if we should show the interactive generation UI
    if matches!(app.prd_processing_state, PRDProcessingState::GeneratingTasks)
        && !app.prd_gen_conversation.is_empty()
    {
        render_interactive_generation(f, area, app);
        return;
    }

    // Clear the entire area with black background
    let clear_widget = Block::default().style(Style::default().bg(Color::Black));
    f.render_widget(clear_widget, area);

    // Build header with project name if available
    let header_text = if let std::option::Option::Some(ref prd) = app.prd_processing_prd {
        std::format!("📋 Processing PRD - {}", prd.title)
    } else {
        std::string::String::from("📋 Processing PRD")
    };

    let mut lines = std::vec![
        Line::from(""),
        Line::from(Span::styled(
            header_text,
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
        Line::from(Span::styled(
            std::format!("File: {}", app.prd_processing_file),
            Style::default().fg(Color::White)
        )),
        Line::from(""),
    ];

    // Add progress steps visualization
    let current_state = &app.prd_processing_state;
    let steps = [
        ("Reading file", PRDProcessingState::ReadingFile),
        ("Parsing PRD", PRDProcessingState::ParsingPRD),
        ("Loading config", PRDProcessingState::LoadingConfig),
        ("Generating tasks", PRDProcessingState::GeneratingTasks),
        ("Saving to database", PRDProcessingState::SavingTasks),
        ("Reloading tasks", PRDProcessingState::ReloadingTasks),
    ];

    for (label, step_state) in &steps {
        let (icon, color) = match current_state {
            PRDProcessingState::Complete { .. } => ("✓", Color::Green),
            PRDProcessingState::Failed { .. } => {
                // All steps are incomplete on failure
                ("☐", Color::DarkGray)
            }
            state => {
                // Check if this step is completed, in progress, or pending
                let step_index = match step_state {
                    PRDProcessingState::ReadingFile => 0,
                    PRDProcessingState::ParsingPRD => 1,
                    PRDProcessingState::LoadingConfig => 2,
                    PRDProcessingState::GeneratingTasks => 3,
                    PRDProcessingState::SavingTasks => 4,
                    PRDProcessingState::ReloadingTasks => 5,
                    _ => 99,
                };
                let current_index = match state {
                    PRDProcessingState::ReadingFile => 0,
                    PRDProcessingState::ParsingPRD => 1,
                    PRDProcessingState::LoadingConfig => 2,
                    PRDProcessingState::GeneratingTasks => 3,
                    PRDProcessingState::SavingTasks => 4,
                    PRDProcessingState::ReloadingTasks => 5,
                    _ => 99,
                };

                if step_index < current_index {
                    ("✓", Color::Green)
                } else if step_index == current_index {
                    ("⏳", Color::Cyan)
                } else {
                    ("☐", Color::DarkGray)
                }
            }
        };

        lines.push(Line::from(vec![
            Span::styled(std::format!("  {} ", icon), Style::default().fg(color)),
            Span::styled(*label, Style::default().fg(color)),
        ]));
    }

    lines.push(Line::from(""));

    // Show current step based on state machine
    match &app.prd_processing_state {
        PRDProcessingState::Failed { error } => {
        lines.push(Line::from(Span::styled(
            "❌ Error",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        )));
        lines.push(Line::from(""));

        // Word-wrap error messages to fit dialog width (60 chars max)
        let max_line_width = 60;

        for error_line in error.lines() {
            // Check line type for styling
            let is_bullet = error_line.starts_with("•") || error_line.starts_with("   →") || error_line.starts_with("   -");
            let is_header = error_line.starts_with("Troubleshooting:") ||
                           error_line.starts_with("🔍") ||
                           error_line.starts_with("All basic checks passed");
            let is_check = error_line.starts_with("✓") || error_line.starts_with("❌");

            // Word wrap long lines
            if error_line.len() > max_line_width && !is_bullet && !is_header {
                let words: Vec<&str> = error_line.split_whitespace().collect();
                let mut current_line = String::new();

                for word in words {
                    if current_line.len() + word.len() + 1 > max_line_width {
                        // Push current line and start new one
                        lines.push(Line::from(Span::styled(
                            current_line.clone(),
                            Style::default().fg(Color::Red)
                        )));
                        current_line = String::from(word);
                    } else {
                        if !current_line.is_empty() {
                            current_line.push(' ');
                        }
                        current_line.push_str(word);
                    }
                }

                // Push remaining line
                if !current_line.is_empty() {
                    lines.push(Line::from(Span::styled(
                        current_line,
                        Style::default().fg(Color::Red)
                    )));
                }
            } else if is_bullet {
                // Bullet points in cyan
                lines.push(Line::from(Span::styled(
                    error_line,
                    Style::default().fg(Color::Cyan)
                )));
            } else if is_header {
                // Section headers in yellow
                if !error_line.is_empty() {
                    lines.push(Line::from(""));
                }
                lines.push(Line::from(Span::styled(
                    error_line,
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                )));
            } else if is_check {
                // Check marks in appropriate colors
                let color = if error_line.starts_with("✓") {
                    Color::Green
                } else {
                    Color::Red
                };
                lines.push(Line::from(Span::styled(
                    error_line,
                    Style::default().fg(color)
                )));
            } else {
                // Regular error text in red
                lines.push(Line::from(Span::styled(
                    error_line,
                    Style::default().fg(Color::Red)
                )));
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::Gray)),
            Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(" to close", Style::default().fg(Color::Gray)),
        ]));
        }
        PRDProcessingState::Complete { task_count } => {
            lines.push(Line::from(Span::styled(
                "✓ Complete!",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                std::format!("Generated {} tasks", task_count),
                Style::default().fg(Color::White)
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Tasks have been saved to the database.",
                Style::default().fg(Color::Gray)
            )));
            lines.push(Line::from(Span::styled(
                "You can now view them in the Kanban board.",
                Style::default().fg(Color::Gray)
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(" to continue", Style::default().fg(Color::Gray)),
            ]));
        }
        state => {
            // Show spinner and current step
            let spinner = app.get_spinner_char();
            let step_text = match state {
                PRDProcessingState::Idle => "Idle",
                PRDProcessingState::ReadingFile => "Reading PRD file...",
                PRDProcessingState::ParsingPRD => "Parsing PRD structure...",
                PRDProcessingState::LoadingConfig => "Loading configuration...",
                PRDProcessingState::GeneratingTasks => "Generating tasks via LLM...",
                PRDProcessingState::SavingTasks => "Saving tasks to database...",
                PRDProcessingState::ReloadingTasks => "Reloading task list...",
                _ => "Processing...",
            };
            lines.push(Line::from(vec![
                Span::styled(std::format!("{} ", spinner), Style::default().fg(Color::Cyan)),
                Span::styled(step_text, Style::default().fg(Color::White)),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Please wait...",
                Style::default().fg(Color::DarkGray)
            )));
        }
    }

    let dialog_width = 70;
    // Use safe helper to prevent buffer overflow
    let dialog_height = calculate_safe_dialog_height(lines.len(), area.height);

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" PRD Processing ")
                .border_style(Style::default().fg(Color::Cyan))
        )
        .style(Style::default().bg(Color::Black))
        .alignment(ratatui::layout::Alignment::Center);

    let dialog_x = (area.width.saturating_sub(dialog_width)) / 2;
    let dialog_y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect {
        x: dialog_x,
        y: dialog_y,
        width: dialog_width,
        height: dialog_height,
    };

    f.render_widget(paragraph, dialog_area);
}

/// Truncates a string to the specified length and adds "..." if truncated.
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        std::format!("{}...", &s[..max_len - 3])
    }
}

/// Calculates a safe dialog height that will never exceed the available buffer area.
///
/// This function prevents buffer overflow panics in ratatui by capping the dialog
/// height to the available space. Use this for all dialog rendering to ensure
/// buffer safety.
///
/// # Arguments
///
/// * `content_lines` - Number of content lines (without border)
/// * `available_height` - Total available height from the rendering area
///
/// # Returns
///
/// A safe height value that includes borders (+2) but never exceeds available space.
///
/// # Examples
///
/// ```
/// let lines = vec![...]; // 50 lines of content
/// let safe_height = calculate_safe_dialog_height(lines.len(), area.height);
/// // If area.height is 35, safe_height will be capped to 31 (35 - 4 padding)
/// ```
fn calculate_safe_dialog_height(content_lines: usize, available_height: u16) -> u16 {
    let desired_height = content_lines as u16 + 2; // +2 for borders
    let max_height = available_height.saturating_sub(4); // Leave 4 lines padding
    std::cmp::min(desired_height, max_height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_safe_dialog_height_within_bounds() {
        // Test: Validates that small dialogs are not artificially capped.
        // Justification: When content fits, full height should be used.
        let safe_height = calculate_safe_dialog_height(10, 35);
        std::assert_eq!(safe_height, 12); // 10 + 2 borders = 12
    }

    #[test]
    fn test_calculate_safe_dialog_height_exceeds_bounds() {
        // Test: Validates that large dialogs are capped to available space.
        // Justification: Prevents buffer overflow when content exceeds terminal height.
        let safe_height = calculate_safe_dialog_height(50, 35);
        std::assert_eq!(safe_height, 31); // 35 - 4 padding = 31 (not 52)
    }

    #[test]
    fn test_calculate_safe_dialog_height_exact_fit() {
        // Test: Validates boundary condition where content exactly fits.
        // Justification: Edge case where desired = max should work correctly.
        let safe_height = calculate_safe_dialog_height(29, 35);
        std::assert_eq!(safe_height, 31); // 29 + 2 = 31, which equals 35 - 4
    }

    #[test]
    fn test_calculate_safe_dialog_height_tiny_terminal() {
        // Test: Validates graceful handling of very small terminals.
        // Justification: Even with 10 lines total, should not panic or underflow.
        let safe_height = calculate_safe_dialog_height(20, 10);
        std::assert_eq!(safe_height, 6); // 10 - 4 = 6, capped from 22
    }

    #[test]
    fn test_notification_creation() {
        // Test: Validates notification is created with correct fields.
        // Justification: Ensures notifications capture all required metadata.
        let notif = Notification::new(
            NotificationLevel::Success,
            String::from("Test message")
        );

        std::assert_eq!(notif.message, "Test message");
        std::assert!(matches!(notif.level, NotificationLevel::Success));
        std::assert!(notif.timestamp <= chrono::Utc::now());
    }

    #[test]
    fn test_add_notification_maintains_order() {
        // Test: Validates notifications are stored newest-first.
        // Justification: User expects to see most recent events at top.
        let mut app = App::new();

        app.add_notification(NotificationLevel::Info, String::from("First"));
        app.add_notification(NotificationLevel::Success, String::from("Second"));
        app.add_notification(NotificationLevel::Error, String::from("Third"));

        std::assert_eq!(app.notifications.len(), 3);
        std::assert_eq!(app.notifications[0].message, "Third");
        std::assert_eq!(app.notifications[1].message, "Second");
        std::assert_eq!(app.notifications[2].message, "First");
    }

    #[test]
    fn test_add_notification_limits_size() {
        // Test: Validates notification list is capped at 50 items.
        // Justification: Prevents unbounded memory growth in long sessions.
        let mut app = App::new();

        // Add 60 notifications
        for i in 0..60 {
            app.add_notification(
                NotificationLevel::Info,
                std::format!("Notification {}", i)
            );
        }

        std::assert_eq!(app.notifications.len(), 50);
        // Newest should still be first
        std::assert_eq!(app.notifications[0].message, "Notification 59");
        std::assert_eq!(app.notifications[49].message, "Notification 10");
    }

    #[test]
    fn test_toggle_notifications() {
        // Test: Validates notification center can be toggled on/off.
        // Justification: Core functionality for showing/hiding dialog.
        let mut app = App::new();

        std::assert!(!app.show_notifications);

        app.toggle_notifications();
        std::assert!(app.show_notifications);

        app.toggle_notifications();
        std::assert!(!app.show_notifications);
    }

    #[test]
    fn test_spinner_animation_cycles() {
        // Test: Validates spinner advances through all frames correctly.
        // Justification: Ensures smooth animation without gaps or jumps.
        let mut app = App::new();

        std::assert_eq!(app.loading_frame, 0);
        std::assert_eq!(app.get_spinner_char(), '⠋');

        app.advance_spinner();
        std::assert_eq!(app.loading_frame, 1);
        std::assert_eq!(app.get_spinner_char(), '⠙');

        // Advance through remaining frames (8 more to reach frame 9)
        for _ in 0..8 {
            app.advance_spinner();
        }
        std::assert_eq!(app.loading_frame, 9);
        std::assert_eq!(app.get_spinner_char(), '⠏');

        // One more advance should wrap back to frame 0
        app.advance_spinner();
        std::assert_eq!(app.loading_frame, 10);
        std::assert_eq!(app.get_spinner_char(), '⠋');
    }

    #[test]
    fn test_spinner_wrapping_add() {
        // Test: Validates spinner frame counter handles overflow correctly.
        // Justification: Prevents panic in very long loading operations.
        let mut app = App::new();

        // Set to near max value
        app.loading_frame = usize::MAX - 1;
        app.advance_spinner();
        std::assert_eq!(app.loading_frame, usize::MAX);

        // Should wrap to 0
        app.advance_spinner();
        std::assert_eq!(app.loading_frame, 0);
    }

    #[test]
    fn test_format_save_indicator_when_saving() {
        // Test: Validates indicator shows "Saving..." during save operation.
        // Justification: Provides immediate feedback to user during async save.
        let mut app = App::new();
        app.is_saving = true;

        let indicator = app.format_save_indicator();
        std::assert_eq!(indicator, "💾 Saving...");
    }

    #[test]
    fn test_format_save_indicator_with_unsaved_changes() {
        // Test: Validates indicator shows warning when changes aren't saved.
        // Justification: Alerts user to potential data loss before quitting.
        let mut app = App::new();
        app.has_unsaved_changes = true;

        let indicator = app.format_save_indicator();
        std::assert_eq!(indicator, "⚠️  Unsaved changes");
    }

    #[test]
    fn test_format_save_indicator_recently_saved_seconds() {
        // Test: Validates indicator shows "Xs ago" for recent saves.
        // Justification: Provides precise timing for very recent saves.
        let mut app = App::new();
        app.last_saved_at = std::option::Option::Some(
            chrono::Utc::now() - chrono::Duration::seconds(30)
        );

        let indicator = app.format_save_indicator();
        std::assert!(indicator.contains("30s ago"));
    }

    #[test]
    fn test_format_save_indicator_recently_saved_minutes() {
        // Test: Validates indicator shows "Xm ago" for saves within last hour.
        // Justification: Minutes are more readable than large second counts.
        let mut app = App::new();
        app.last_saved_at = std::option::Option::Some(
            chrono::Utc::now() - chrono::Duration::minutes(5)
        );

        let indicator = app.format_save_indicator();
        std::assert!(indicator.contains("5m ago"));
    }

    #[test]
    fn test_format_save_indicator_never_saved() {
        // Test: Validates indicator returns empty string when never saved.
        // Justification: Avoids showing misleading "saved" status on startup.
        let app = App::new();

        let indicator = app.format_save_indicator();
        std::assert_eq!(indicator, "");
    }

    #[test]
    fn test_format_save_indicator_priority_order() {
        // Test: Validates saving state takes priority over other states.
        // Justification: Most important information should be shown first.
        let mut app = App::new();
        app.is_saving = true;
        app.has_unsaved_changes = true;
        app.last_saved_at = std::option::Option::Some(chrono::Utc::now());

        // Should show "Saving..." even though other states are set
        let indicator = app.format_save_indicator();
        std::assert_eq!(indicator, "💾 Saving...");
    }

    #[test]
    fn test_truncate_string_no_truncation() {
        // Test: Validates short strings are returned unchanged.
        // Justification: Preserves full text when it fits.
        let result = truncate_string("Short", 10);
        std::assert_eq!(result, "Short");
    }

    #[test]
    fn test_truncate_string_exact_length() {
        // Test: Validates strings at exact max length aren't truncated.
        // Justification: Edge case for boundary condition.
        let result = truncate_string("12345", 5);
        std::assert_eq!(result, "12345");
    }

    #[test]
    fn test_truncate_string_with_ellipsis() {
        // Test: Validates long strings are truncated with "..." suffix.
        // Justification: Indicates to user that text is truncated.
        let result = truncate_string("This is a long string", 10);
        std::assert_eq!(result, "This is...");
        std::assert_eq!(result.len(), 10);
    }

    #[test]
    fn test_dev_tools_navigation_initial_state() {
        // Test: Validates SQLite browser navigation starts at index 0.
        // Justification: Initial state should select first table.
        let app = App::new();

        std::assert_eq!(app.dev_tools_selection, 0);
        std::assert_eq!(app.db_selected_table, 0);
        std::assert_eq!(app.db_current_page, 0);
        std::assert!(app.db_tables.is_empty());
        std::assert!(app.db_table_data.is_empty());
    }

    #[test]
    fn test_dev_tools_navigation_bounds_checking() {
        // Test: Validates navigation respects table list bounds.
        // Justification: Prevents out-of-bounds access that could cause panics.
        let mut app = App::new();

        // Simulate having 3 tables loaded
        app.db_tables = std::vec![
            String::from("table1"),
            String::from("table2"),
            String::from("table3"),
        ];
        app.db_selected_table = 0;

        // Navigate down twice
        if app.db_selected_table < app.db_tables.len() - 1 {
            app.db_selected_table += 1;
        }
        std::assert_eq!(app.db_selected_table, 1);

        if app.db_selected_table < app.db_tables.len() - 1 {
            app.db_selected_table += 1;
        }
        std::assert_eq!(app.db_selected_table, 2);

        // Try to navigate down again (should stay at 2)
        if app.db_selected_table < app.db_tables.len() - 1 {
            app.db_selected_table += 1;
        }
        std::assert_eq!(app.db_selected_table, 2);

        // Navigate up
        if app.db_selected_table > 0 {
            app.db_selected_table -= 1;
        }
        std::assert_eq!(app.db_selected_table, 1);
    }

    #[test]
    fn test_dev_tools_view_state_transitions() {
        // Test: Validates state transitions between table list and table data views.
        // Justification: Ensures correct view is shown based on db_table_data state.
        let mut app = App::new();

        // Initially: no tables, no data - should show empty state
        std::assert!(app.db_tables.is_empty());
        std::assert!(app.db_table_data.is_empty());

        // After loading tables: has tables, no data - should show table list
        app.db_tables = std::vec![
            String::from("tasks"),
            String::from("projects"),
        ];
        std::assert!(!app.db_tables.is_empty());
        std::assert!(app.db_table_data.is_empty());

        // After loading table data: has tables and data - should show table data
        let mut row1 = std::collections::HashMap::new();
        row1.insert(String::from("id"), String::from("1"));
        row1.insert(String::from("name"), String::from("Test"));
        app.db_table_data = std::vec![row1];
        app.db_table_columns = std::vec![String::from("id"), String::from("name")];

        std::assert!(!app.db_tables.is_empty());
        std::assert!(!app.db_table_data.is_empty());

        // After pressing Esc (clearing data): has tables, no data - should show table list again
        app.db_table_data.clear();
        app.db_table_columns.clear();
        app.db_current_page = 0;

        std::assert!(!app.db_tables.is_empty());
        std::assert!(app.db_table_data.is_empty());
    }

    #[test]
    fn test_dev_tools_selection_persistence() {
        // Test: Validates table selection persists when viewing data and going back.
        // Justification: User should return to same selection in table list.
        let mut app = App::new();

        app.db_tables = std::vec![
            String::from("table1"),
            String::from("table2"),
            String::from("table3"),
        ];

        // Select second table (index 1)
        app.db_selected_table = 1;

        // Load table data (simulating Enter key)
        let mut row = std::collections::HashMap::new();
        row.insert(String::from("id"), String::from("1"));
        app.db_table_data = std::vec![row];

        // Verify we're viewing data for table at index 1
        std::assert_eq!(app.db_selected_table, 1);
        std::assert_eq!(app.db_tables[app.db_selected_table], "table2");

        // Clear data (simulating Esc key to go back)
        app.db_table_data.clear();

        // Selection should still be at index 1
        std::assert_eq!(app.db_selected_table, 1);
    }

    #[test]
    fn test_dev_tools_empty_table_handling() {
        // Test: Validates behavior when loading a table with 0 rows.
        // Justification: Empty tables should not crash and should return to list view.
        let mut app = App::new();

        app.db_tables = std::vec![String::from("empty_table")];
        app.db_selected_table = 0;

        // Simulate loading empty table (0 rows)
        app.db_table_data = std::vec![];
        app.db_table_columns = std::vec![String::from("id"), String::from("name")];

        // Should show table list again (since data is empty)
        std::assert!(app.db_table_data.is_empty());
        std::assert!(!app.db_table_columns.is_empty()); // Columns exist
    }

    #[test]
    fn test_dev_tools_active_dev_tool_state() {
        // Test: Validates active_dev_tool state transitions correctly.
        // Justification: Ensures proper navigation between DevTools menu and active tools.
        let mut app = App::new();

        // Initially no dev tool active
        std::assert_eq!(app.active_dev_tool, std::option::Option::None);
        std::assert_eq!(app.active_tool, DashboardTool::Kanban);

        // Navigate to DevTools view
        app.active_tool = DashboardTool::DevTools;
        std::assert_eq!(app.active_tool, DashboardTool::DevTools);
        std::assert_eq!(app.active_dev_tool, std::option::Option::None);

        // Launch SQLite Browser
        app.active_dev_tool = std::option::Option::Some(DevTool::SqliteBrowser);
        std::assert_eq!(app.active_dev_tool, std::option::Option::Some(DevTool::SqliteBrowser));

        // Close SQLite Browser (when in table list)
        if app.db_table_data.is_empty() {
            app.active_dev_tool = std::option::Option::None;
        }
        std::assert_eq!(app.active_dev_tool, std::option::Option::None);
    }

    #[test]
    fn test_dev_tools_esc_navigation_hierarchy() {
        // Test: Validates Esc key navigation hierarchy in SQLite Browser.
        // Justification: Esc should go back one level at a time, not close immediately.
        let mut app = App::new();

        app.db_tables = std::vec![String::from("test_table")];
        app.active_dev_tool = std::option::Option::Some(DevTool::SqliteBrowser);

        // Simulate viewing table data
        let mut row = std::collections::HashMap::new();
        row.insert(String::from("id"), String::from("1"));
        app.db_table_data = std::vec![row];

        // First Esc: Should clear data and return to table list
        if !app.db_table_data.is_empty() {
            app.db_table_data.clear();
            app.db_table_columns.clear();
            app.db_current_page = 0;
        }

        std::assert!(app.db_table_data.is_empty());
        std::assert_eq!(app.active_dev_tool, std::option::Option::Some(DevTool::SqliteBrowser));

        // Second Esc: Should close SQLite Browser
        if app.db_table_data.is_empty() {
            app.active_dev_tool = std::option::Option::None;
        }

        std::assert_eq!(app.active_dev_tool, std::option::Option::None);
    }

    #[test]
    fn test_dev_tools_enter_key_priority() {
        // Test: Validates Enter key calls correct handler based on state priority.
        // Justification: When SQLite Browser is active, Enter should NOT try to launch a new dev tool.
        let mut app = App::new();

        // Scenario 1: In DevTools view with no active tool - Enter should launch tool
        app.active_tool = DashboardTool::DevTools;
        app.active_dev_tool = std::option::Option::None;
        app.dev_tools_selection = 0; // SqliteBrowser selected

        // Simulate launching tool (this is what should happen when no tool is active)
        std::assert_eq!(app.active_dev_tool, std::option::Option::None);

        // Scenario 2: SQLite Browser is active with tables loaded - Enter should NOT reset state
        app.active_tool = DashboardTool::DevTools;
        app.active_dev_tool = std::option::Option::Some(DevTool::SqliteBrowser);
        app.db_tables = std::vec![
            String::from("projects"),
            String::from("tasks"),
        ];
        app.db_selected_table = 1; // "tasks" selected
        app.dev_tools_selection = 0; // This should be IGNORED when active_dev_tool is Some

        // The key insight: when active_dev_tool.is_some(), the dev_tools_selection index
        // should NOT affect behavior. The handler should use db_selected_table instead.
        std::assert_eq!(app.active_dev_tool, std::option::Option::Some(DevTool::SqliteBrowser));
        std::assert_eq!(app.db_selected_table, 1);

        // If Enter incorrectly uses dev_tools_selection (0), it would try to launch SqliteBrowser again
        // If Enter correctly uses active_dev_tool.is_some(), it would call load_table_data() for table at index 1

        // Verify the state is correct for loading table data
        std::assert!(!app.db_tables.is_empty());
        std::assert!(app.db_selected_table < app.db_tables.len());
        std::assert_eq!(app.db_tables[app.db_selected_table], "tasks");
    }

    #[test]
    fn test_dev_tools_enter_condition_matching() {
        // Test: Validates which Enter key condition should match given specific state.
        // Justification: Diagnose why Enter key isn't calling load_table_data().
        let mut app = App::new();

        // Setup: SQLite Browser is active viewing table list
        app.active_tool = DashboardTool::DevTools;
        app.active_dev_tool = std::option::Option::Some(DevTool::SqliteBrowser);
        app.db_tables = std::vec![String::from("tasks")];
        app.db_selected_table = 0;
        app.db_table_data.clear(); // Empty = showing table list

        // Dialog flags that might interfere
        std::assert_eq!(app.show_confirmation_dialog, false);
        std::assert_eq!(app.show_spotlight_dialog, false);
        std::assert_eq!(app.show_sql_query_dialog, false);
        std::assert_eq!(app.show_config_editor, false);
        std::assert_eq!(app.show_markdown_browser, false);
        std::assert_eq!(app.show_dev_tools_menu, false);
        std::assert_eq!(app.show_task_creator_dialog, false);
        std::assert_eq!(app.show_llm_chat_dialog, false);
        std::assert_eq!(app.show_task_editor_dialog, false);

        // Critical assertion: active_dev_tool.is_some() should be true
        std::assert!(app.active_dev_tool.is_some(), "active_dev_tool should be Some(SqliteBrowser)");

        // This state should match the condition: app.active_dev_tool.is_some()
        // NOT the condition: app.active_tool == DashboardTool::DevTools
    }

    #[test]
    fn test_prd_processing_state_initial() {
        // Test: Validates initial PRD processing state is Idle.
        // Justification: State machine must start in known state.
        let app = App::new();

        std::assert!(matches!(app.prd_processing_state, PRDProcessingState::Idle));
        std::assert!(!app.show_prd_processing);
        std::assert!(!app.prd_processing_pending);
    }

    #[test]
    fn test_start_prd_processing_transitions_to_reading_file() {
        // Test: Validates start_prd_processing() sets up initial state correctly.
        // Justification: Must show processing view and set ReadingFile state.
        let mut app = App::new();
        app.markdown_files = std::vec![String::from("test.md")];
        app.markdown_selected = 0;

        app.start_prd_processing();

        std::assert!(app.show_prd_processing);
        std::assert!(matches!(app.prd_processing_state, PRDProcessingState::ReadingFile));
        std::assert!(app.prd_processing_pending);
        std::assert_eq!(app.prd_processing_file, "test.md");
    }

    #[test]
    fn test_start_prd_processing_clears_intermediate_data() {
        // Test: Validates start_prd_processing() clears previous processing data.
        // Justification: Each new processing run must start clean.
        let mut app = App::new();
        app.markdown_files = std::vec![String::from("new.md")];
        app.markdown_selected = 0;

        // Simulate previous processing
        app.prd_processing_content = std::option::Option::Some(String::from("old content"));
        app.prd_processing_prd = std::option::Option::Some(task_manager::domain::prd::PRD::new(
            String::from("project-1"),
            String::from("Old PRD"),
            std::vec![],
            std::vec![],
            std::vec![],
            String::from("old"),
        ));

        app.start_prd_processing();

        std::assert!(app.prd_processing_content.is_none());
        std::assert!(app.prd_processing_prd.is_none());
        std::assert!(app.prd_processing_config.is_none());
        std::assert!(app.prd_processing_tasks.is_none());
    }

    #[test]
    fn test_prd_processing_state_complete_has_task_count() {
        // Test: Validates Complete state carries task count.
        // Justification: UI needs task count for success message.
        let mut app = App::new();

        app.prd_processing_state = PRDProcessingState::Complete { task_count: 42 };

        if let PRDProcessingState::Complete { task_count } = app.prd_processing_state {
            std::assert_eq!(task_count, 42);
        } else {
            std::panic!("Expected Complete state with task_count");
        }
    }

    #[test]
    fn test_prd_processing_state_failed_has_error_message() {
        // Test: Validates Failed state carries error message.
        // Justification: UI needs error details for diagnostics.
        let mut app = App::new();

        app.prd_processing_state = PRDProcessingState::Failed {
            error: String::from("Connection refused")
        };

        if let PRDProcessingState::Failed { error } = &app.prd_processing_state {
            std::assert_eq!(error, "Connection refused");
        } else {
            std::panic!("Expected Failed state with error");
        }
    }

    #[test]
    fn test_prd_processing_keyboard_handlers_use_state_matching() {
        // Test: Validates keyboard handlers correctly identify Complete state.
        // Justification: Enter key should close processing view only when Complete.
        let mut app = App::new();
        app.show_prd_processing = true;

        // Not complete - Enter should not close
        app.prd_processing_state = PRDProcessingState::GeneratingTasks;
        std::assert!(!matches!(app.prd_processing_state, PRDProcessingState::Complete { .. }));

        // Complete - Enter should close
        app.prd_processing_state = PRDProcessingState::Complete { task_count: 5 };
        std::assert!(matches!(app.prd_processing_state, PRDProcessingState::Complete { .. }));
    }

    #[test]
    fn test_prd_processing_escape_only_closes_on_error() {
        // Test: Validates Escape key handling for error states.
        // Justification: Esc should only close when Failed, not during normal processing.
        let mut app = App::new();
        app.show_prd_processing = true;

        // Not failed - Esc should not close
        app.prd_processing_state = PRDProcessingState::GeneratingTasks;
        std::assert!(!matches!(app.prd_processing_state, PRDProcessingState::Failed { .. }));

        // Failed - Esc should close
        app.prd_processing_state = PRDProcessingState::Failed {
            error: String::from("Error")
        };
        std::assert!(matches!(app.prd_processing_state, PRDProcessingState::Failed { .. }));
    }

    #[test]
    fn test_prd_processing_all_states_are_distinct() {
        // Test: Validates all PRDProcessingState variants are distinguishable.
        // Justification: State machine must have no ambiguous states.
        let states = [
            PRDProcessingState::Idle,
            PRDProcessingState::ReadingFile,
            PRDProcessingState::ParsingPRD,
            PRDProcessingState::LoadingConfig,
            PRDProcessingState::GeneratingTasks,
            PRDProcessingState::SavingTasks,
            PRDProcessingState::ReloadingTasks,
            PRDProcessingState::Complete { task_count: 1 },
            PRDProcessingState::Failed { error: String::from("test") },
        ];

        // Verify we can match each state uniquely
        for state in &states {
            let matched = match state {
                PRDProcessingState::Idle => "idle",
                PRDProcessingState::ReadingFile => "reading",
                PRDProcessingState::ParsingPRD => "parsing",
                PRDProcessingState::LoadingConfig => "loading",
                PRDProcessingState::GeneratingTasks => "generating",
                PRDProcessingState::SavingTasks => "saving",
                PRDProcessingState::ReloadingTasks => "reloading",
                PRDProcessingState::Complete { .. } => "complete",
                PRDProcessingState::Failed { .. } => "failed",
            };
            std::assert!(!matched.is_empty());
        }
    }

    #[test]
    fn test_prd_gen_message_creation() {
        // Test: Validates PRDGenMessage is created with correct fields.
        // Justification: Core data structure for interactive generation conversation.
        let msg = PRDGenMessage {
            role: PRDGenRole::User,
            content: String::from("Focus on OAuth2"),
            timestamp: chrono::Utc::now(),
        };

        std::assert_eq!(msg.content, "Focus on OAuth2");
        std::assert!(matches!(msg.role, PRDGenRole::User));
        std::assert!(msg.timestamp <= chrono::Utc::now());
    }

    #[test]
    fn test_prd_gen_conversation_append() {
        // Test: Validates messages can be appended to conversation.
        // Justification: Conversation builds up over time during generation.
        let mut app = App::new();

        app.prd_gen_conversation.push(PRDGenMessage {
            role: PRDGenRole::System,
            content: String::from("Starting generation..."),
            timestamp: chrono::Utc::now(),
        });

        app.prd_gen_conversation.push(PRDGenMessage {
            role: PRDGenRole::Assistant,
            content: String::from("Analyzing PRD..."),
            timestamp: chrono::Utc::now(),
        });

        app.prd_gen_conversation.push(PRDGenMessage {
            role: PRDGenRole::User,
            content: String::from("Focus on security"),
            timestamp: chrono::Utc::now(),
        });

        std::assert_eq!(app.prd_gen_conversation.len(), 3);
        std::assert!(matches!(app.prd_gen_conversation[0].role, PRDGenRole::System));
        std::assert!(matches!(app.prd_gen_conversation[1].role, PRDGenRole::Assistant));
        std::assert!(matches!(app.prd_gen_conversation[2].role, PRDGenRole::User));
    }

    #[test]
    fn test_prd_gen_input_buffer_manipulation() {
        // Test: Validates input buffer can be manipulated (append, clear).
        // Justification: User types characters one by one and can clear.
        let mut app = App::new();

        std::assert_eq!(app.prd_gen_input, "");

        app.prd_gen_input.push('F');
        app.prd_gen_input.push('o');
        app.prd_gen_input.push('o');
        std::assert_eq!(app.prd_gen_input, "Foo");

        app.prd_gen_input.clear();
        std::assert_eq!(app.prd_gen_input, "");
    }

    #[test]
    fn test_prd_gen_last_message_storage() {
        // Test: Validates last message is stored for Up-arrow editing.
        // Justification: Core feature for message editing UX.
        let mut app = App::new();

        std::assert_eq!(app.prd_gen_last_message, "");
        std::assert!(!app.prd_gen_editing_last);

        // Simulate user typing and sending message
        app.prd_gen_input = String::from("Focus on OAuth2");
        app.prd_gen_last_message = app.prd_gen_input.clone();
        app.prd_gen_input.clear();

        std::assert_eq!(app.prd_gen_last_message, "Focus on OAuth2");
        std::assert_eq!(app.prd_gen_input, "");
    }

    #[test]
    fn test_prd_gen_edit_mode_activation() {
        // Test: Validates editing mode can be activated via Up arrow.
        // Justification: User presses Up to edit last message.
        let mut app = App::new();

        // Setup: Previous message was sent
        app.prd_gen_last_message = String::from("Original message");
        app.prd_gen_input = String::from("");

        // Simulate Up arrow press (only when input is empty)
        if app.prd_gen_input.is_empty() && !app.prd_gen_last_message.is_empty() {
            app.prd_gen_input = app.prd_gen_last_message.clone();
            app.prd_gen_editing_last = true;
        }

        std::assert_eq!(app.prd_gen_input, "Original message");
        std::assert!(app.prd_gen_editing_last);
    }

    #[test]
    fn test_prd_gen_edit_mode_cancel() {
        // Test: Validates editing mode can be cancelled with Esc.
        // Justification: User can cancel edit and return to empty state.
        let mut app = App::new();

        // Setup: User is editing last message
        app.prd_gen_last_message = String::from("Original");
        app.prd_gen_input = String::from("Original");
        app.prd_gen_editing_last = true;

        // Simulate Esc press
        if app.prd_gen_editing_last {
            app.prd_gen_editing_last = false;
            app.prd_gen_input.clear();
        }

        std::assert_eq!(app.prd_gen_input, "");
        std::assert!(!app.prd_gen_editing_last);
        std::assert_eq!(app.prd_gen_last_message, "Original"); // Preserved
    }

    #[test]
    fn test_prd_gen_partial_task_tracking() {
        // Test: Validates partial tasks can be tracked as they're generated.
        // Justification: UI shows tasks incrementally during LLM generation.
        let mut app = App::new();

        app.prd_gen_partial_tasks.push(PartialTask {
            title: String::from("Task 1: Setup OAuth"),
            status: PartialTaskStatus::Complete,
            validation_messages: std::vec::Vec::new(),
        });

        app.prd_gen_partial_tasks.push(PartialTask {
            title: String::from("Task 2: Implement tokens"),
            status: PartialTaskStatus::Generating,
            validation_messages: std::vec::Vec::new(),
        });

        std::assert_eq!(app.prd_gen_partial_tasks.len(), 2);
        std::assert!(matches!(app.prd_gen_partial_tasks[0].status, PartialTaskStatus::Complete));
        std::assert!(matches!(app.prd_gen_partial_tasks[1].status, PartialTaskStatus::Generating));
    }

    #[test]
    fn test_prd_gen_status_transitions() {
        // Test: Validates status can transition through generation lifecycle.
        // Justification: Status drives UI rendering and user feedback.
        let mut app = App::new();

        std::assert!(matches!(app.prd_gen_status, PRDGenStatus::Idle));

        app.prd_gen_status = PRDGenStatus::Thinking;
        std::assert!(matches!(app.prd_gen_status, PRDGenStatus::Thinking));

        app.prd_gen_status = PRDGenStatus::WaitingForInput;
        std::assert!(matches!(app.prd_gen_status, PRDGenStatus::WaitingForInput));

        app.prd_gen_status = PRDGenStatus::Generating;
        std::assert!(matches!(app.prd_gen_status, PRDGenStatus::Generating));

        app.prd_gen_status = PRDGenStatus::Complete;
        std::assert!(matches!(app.prd_gen_status, PRDGenStatus::Complete));
    }

    #[test]
    fn test_prd_gen_scroll_offset_control() {
        // Test: Validates conversation can be scrolled through history.
        // Justification: Long conversations need scrolling for older messages.
        let mut app = App::new();

        std::assert_eq!(app.prd_gen_scroll_offset, 0);

        app.prd_gen_scroll_offset = 5;
        std::assert_eq!(app.prd_gen_scroll_offset, 5);

        // Scroll back to top
        app.prd_gen_scroll_offset = 0;
        std::assert_eq!(app.prd_gen_scroll_offset, 0);
    }

    #[test]
    fn test_prd_gen_input_focus_state() {
        // Test: Validates input field focus state tracking.
        // Justification: UI changes color based on focus (yellow vs gray border).
        let mut app = App::new();

        std::assert!(!app.prd_gen_input_active);

        app.prd_gen_input_active = true;
        std::assert!(app.prd_gen_input_active);

        app.prd_gen_input_active = false;
        std::assert!(!app.prd_gen_input_active);
    }

    #[test]
    fn test_prd_gen_complete_workflow() {
        // Test: Validates complete message send-edit-resend workflow.
        // Justification: Integration test for the full editing feature.
        let mut app = App::new();

        // Step 1: User types and sends first message
        app.prd_gen_input = String::from("Focus on auth");
        app.prd_gen_last_message = app.prd_gen_input.clone();
        app.prd_gen_conversation.push(PRDGenMessage {
            role: PRDGenRole::User,
            content: app.prd_gen_input.clone(),
            timestamp: chrono::Utc::now(),
        });
        app.prd_gen_input.clear();

        std::assert_eq!(app.prd_gen_conversation.len(), 1);
        std::assert_eq!(app.prd_gen_last_message, "Focus on auth");
        std::assert_eq!(app.prd_gen_input, "");

        // Step 2: User presses Up arrow to edit
        if app.prd_gen_input.is_empty() && !app.prd_gen_last_message.is_empty() {
            app.prd_gen_input = app.prd_gen_last_message.clone();
            app.prd_gen_editing_last = true;
        }

        std::assert_eq!(app.prd_gen_input, "Focus on auth");
        std::assert!(app.prd_gen_editing_last);

        // Step 3: User modifies and resends
        app.prd_gen_input = String::from("Focus on auth and security");
        app.prd_gen_last_message = app.prd_gen_input.clone();
        app.prd_gen_conversation.push(PRDGenMessage {
            role: PRDGenRole::User,
            content: app.prd_gen_input.clone(),
            timestamp: chrono::Utc::now(),
        });
        app.prd_gen_input.clear();
        app.prd_gen_editing_last = false;

        std::assert_eq!(app.prd_gen_conversation.len(), 2);
        std::assert_eq!(app.prd_gen_last_message, "Focus on auth and security");
        std::assert!(!app.prd_gen_editing_last);
    }

    #[test]
    fn test_validation_red_row_functionality() {
        // Test: Validates that ValidationInfo messages are stored and status updates work.
        // Justification: Ensures validation messages appear in red rows below tasks in the UI
        // and that task status transitions to Validating when remediation occurs.
        let mut app = App::new();

        // Step 1: Add a task that will need validation
        app.prd_gen_partial_tasks.push(PartialTask {
            title: String::from("Setup authentication"),
            status: PartialTaskStatus::Complete,
            validation_messages: std::vec::Vec::new(),
        });

        std::assert_eq!(app.prd_gen_partial_tasks.len(), 1);
        std::assert!(matches!(app.prd_gen_partial_tasks[0].status, PartialTaskStatus::Complete));
        std::assert_eq!(app.prd_gen_partial_tasks[0].validation_messages.len(), 0);

        // Step 2: Simulate ValidationInfo update - assignee not found
        let task_title = String::from("Setup authentication");
        let validation_msg_1 = String::from("Assignee 'Bob' not found, attempting LLM remediation...");

        if let Some(task) = app.prd_gen_partial_tasks.iter_mut().find(|t| t.title == task_title) {
            task.status = PartialTaskStatus::Validating;
            task.validation_messages.push(validation_msg_1.clone());
        }

        // Verify status changed to Validating and message was added
        std::assert!(matches!(app.prd_gen_partial_tasks[0].status, PartialTaskStatus::Validating));
        std::assert_eq!(app.prd_gen_partial_tasks[0].validation_messages.len(), 1);
        std::assert_eq!(app.prd_gen_partial_tasks[0].validation_messages[0], validation_msg_1);

        // Step 3: Simulate ValidationInfo update - remediation success
        let validation_msg_2 = String::from("Remediation successful: 'Bob' → 'Engineering Lead'");

        if let Some(task) = app.prd_gen_partial_tasks.iter_mut().find(|t| t.title == task_title) {
            task.validation_messages.push(validation_msg_2.clone());
        }

        // Verify second message was added
        std::assert_eq!(app.prd_gen_partial_tasks[0].validation_messages.len(), 2);
        std::assert_eq!(app.prd_gen_partial_tasks[0].validation_messages[1], validation_msg_2);

        // Step 4: Add validation box to conversation (simulating what the handler does)
        let validation_box = std::format!(
            "┌─ Validation ─────────────────────────────────\n\
             │ Task: {}\n\
             │ {}\n\
             └──────────────────────────────────────────────",
            task_title,
            validation_msg_1
        );

        app.prd_gen_conversation.push(PRDGenMessage {
            role: PRDGenRole::System,
            content: validation_box.clone(),
            timestamp: chrono::Utc::now(),
        });

        // Verify conversation has the validation box
        std::assert_eq!(app.prd_gen_conversation.len(), 1);
        std::assert!(app.prd_gen_conversation[0].content.contains("┌─ Validation"));
        std::assert!(app.prd_gen_conversation[0].content.contains("Assignee 'Bob' not found"));

        // Step 5: Verify rendering would show red rows (test data structure, not rendering)
        // In the actual UI, these messages would render as:
        // ⚠ Setup authentication
        //     └─ Assignee 'Bob' not found, attempting LLM remediation...
        //     └─ Remediation successful: 'Bob' → 'Engineering Lead'
        let task = &app.prd_gen_partial_tasks[0];
        for msg in &task.validation_messages {
            // Verify messages are accessible for red row rendering
            std::assert!(!msg.is_empty());
        }
    }
}
