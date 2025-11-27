# Interactive PRD Generation Design

## Problem Statement

When generating tasks from a PRD, the "Generating tasks via LLM" step can take 15-60 seconds. During this time:
- User has no visibility into what the LLM is thinking
- User cannot provide mid-generation feedback if LLM goes off-track
- User gets bored waiting with no engagement
- If LLM needs clarification, the generation fails instead of asking

## Solution: Interactive Agent Conversation

Transform the static "please wait" experience into an interactive conversation using Rig's agent capabilities.

### User Experience Flow

```
User presses Enter on markdown file
  ↓
Steps 1-3: Reading file, parsing PRD, loading config (fast, <1s)
  ↓
Step 4: INTERACTIVE GENERATION STARTS
  ↓
┌─────────────────────────────────────────────────────────────┐
│  Generating Tasks: authentication.md                       │
│  Model: llama3.2:latest via Ollama                         │
├─────────────────────────────────────────────────────────────┤
│  💭 LLM Thinking:                                           │
│  ┌─────────────────────────────────────────────────────────┐
│  │ 🤖 Analyzing PRD objectives... I see authentication,    │
│  │    authorization, and user profile requirements.        │
│  │                                                          │
│  │ 🤖 Breaking down into epics: Auth Flow, User Mgmt,      │
│  │    Permission System                                     │
│  │                                                          │
│  │ 👤 Suggestion: Focus on OAuth2 first, skip SAML         │
│  │                                                          │
│  │ 🤖 Acknowledged! Prioritizing OAuth2 implementation.     │
│  │    Creating tasks for: Setup OAuth2 client, Token mgmt, │
│  │    Refresh flow...                                       │
│  └─────────────────────────────────────────────────────────┘
│                                                             │
│  📋 Generated Tasks (3/12):                                 │
│  ✓ Setup OAuth2 client configuration                       │
│  ✓ Implement token management service                      │
│  ⏳ Build refresh token flow...                            │
│                                                             │
│  Your input: _                                              │
│  [Type suggestion and press Enter, or wait for completion] │
└─────────────────────────────────────────────────────────────┘
```

## Technical Architecture

### 1. New App State Fields

```rust
// Interactive generation conversation
prd_gen_conversation: std::vec::Vec<PRDGenMessage>,  // Conversation history
prd_gen_input: String,                               // User input buffer
prd_gen_partial_tasks: std::vec::Vec<PartialTask>,   // Tasks as they're generated
prd_gen_status: PRDGenStatus,                        // Current generation status

enum PRDGenStatus {
    Idle,
    Thinking,        // LLM is processing
    WaitingForInput, // LLM asked a question
    Generating,      // Creating tasks
    Complete,
}

struct PRDGenMessage {
    role: PRDGenRole,
    content: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

enum PRDGenRole {
    System,    // System prompts
    Assistant, // LLM responses/thinking
    User,      // User inputs
}

struct PartialTask {
    title: String,
    status: PartialTaskStatus,  // Generating | Complete | Failed
}
```

### 2. Modified RigPRDParserAdapter

Add new method `parse_prd_interactively()` that:

1. Returns a channel for streaming updates
2. Sends thinking updates as they occur
3. Accepts user input mid-generation
4. Yields partial task results

```rust
pub async fn parse_prd_interactively(
    &self,
    prd: &task_manager::domain::prd::PRD,
) -> std::result::Result<
    (
        tokio::sync::mpsc::Receiver<PRDGenUpdate>,
        tokio::sync::mpsc::Sender<String>,  // User input channel
    ),
    std::string::String
> {
    // Implementation uses Rig's agent with streaming
}

enum PRDGenUpdate {
    Thinking(String),         // LLM reasoning
    Question(String),         // LLM asking for clarification
    TaskGenerated(PartialTask), // New task created
    Complete(Vec<Task>),      // All tasks ready
    Error(String),            // Generation failed
}
```

### 3. Updated State Machine

```rust
enum PRDProcessingState {
    // ... existing states ...
    GeneratingTasks {
        thinking_visible: bool,  // Show thinking panel
        input_enabled: bool,     // Enable user input
    },
    // ... rest ...
}
```

### 4. UI Rendering (`render_prd_processing_interactive`)

Split screen into 3 sections:

**Top (30%)**: Conversation history (scrollable)
- Shows LLM thinking, user suggestions, questions

**Middle (50%)**: Generated tasks list
- Shows tasks as they're created
- Status indicators (⏳ generating, ✓ complete)

**Bottom (20%)**: User input
- Text field for suggestions
- Hint text explaining how to interact

### 5. Keyboard Handlers

When `show_prd_processing && prd_processing_state == GeneratingTasks`:

- **Char(c)**: Append to `prd_gen_input`
- **Backspace**: Remove last char from input
- **Enter**: Send `prd_gen_input` to LLM agent, clear buffer
- **Esc**: Close input (don't cancel generation)
- **Ctrl+C**: Cancel entire generation

### 6. Event Loop Integration

```rust
// In process_prd_step()
PRDProcessingState::GeneratingTasks { .. } => {
    // Check for messages from LLM agent channel
    if let Some(update) = prd_gen_receiver.try_recv() {
        match update {
            PRDGenUpdate::Thinking(msg) => {
                self.prd_gen_conversation.push(PRDGenMessage {
                    role: PRDGenRole::Assistant,
                    content: msg,
                    timestamp: chrono::Utc::now(),
                });
            }
            PRDGenUpdate::TaskGenerated(task) => {
                self.prd_gen_partial_tasks.push(task);
            }
            PRDGenUpdate::Complete(tasks) => {
                self.prd_processing_tasks = Some(tasks);
                self.prd_processing_state = PRDProcessingState::SavingTasks;
            }
            // ... handle other updates ...
        }
    }
    true // Keep processing
}
```

## Implementation Plan

### Phase 1: Foundation (Current Task 1-3)
- [x] Add conversation state fields to App
- [x] Create PRDGenMessage/PRDGenRole/PartialTask structs
- [ ] Implement basic UI layout (3-section split)
- [ ] Add text input handling

### Phase 2: Rig Integration (Current Task 4)
- [ ] Create `parse_prd_interactively()` method
- [ ] Set up channels for streaming updates
- [ ] Implement thinking extraction from Rig agent
- [ ] Handle user input injection

### Phase 3: Real-time Display (Current Task 5-6)
- [ ] Render conversation history with scrolling
- [ ] Display partial tasks as they arrive
- [ ] Auto-scroll to bottom on new messages
- [ ] Add timestamps to messages

### Phase 4: Testing (Current Task 7)
- [ ] Test with various PRD sizes
- [ ] Test user input mid-generation
- [ ] Test cancellation
- [ ] Test error handling

## Success Criteria

- ✅ User sees LLM thinking in real-time
- ✅ User can provide suggestions without breaking generation
- ✅ Tasks appear incrementally, not all at once
- ✅ Conversation history is preserved and scrollable
- ✅ Input field is always accessible during generation
- ✅ UI updates smoothly (no flickering or freezing)

## Example Interactions

**Scenario 1: User provides clarification**
```
🤖: I see requirements for authentication. Should I include social login?
👤: Yes, add Google and GitHub OAuth
🤖: Adding tasks for Google OAuth and GitHub OAuth integration...
```

**Scenario 2: User corrects course**
```
🤖: Creating tasks for real-time websocket implementation...
👤: Wait, we're using polling, not websockets
🤖: Understood, switching to polling-based approach...
```

**Scenario 3: User asks for more detail**
```
🤖: Task: Implement authentication
👤: Break that down further into subtasks
🤖: Splitting into: Setup auth routes, Create middleware, Add session mgmt...
```

## Technical Challenges

1. **Channel Communication**: Need to handle async channels in the TUI event loop
   - Solution: Use `try_recv()` to poll without blocking

2. **Partial JSON Parsing**: LLM may not output complete JSON mid-generation
   - Solution: Use line-by-line or chunk-based parsing

3. **Input Handling**: Distinguishing between global hotkeys and text input
   - Solution: Add `prd_gen_input_active` flag, suppress hotkeys when active

4. **Scrolling**: Auto-scroll conversation but allow manual scrolling
   - Solution: Track scroll position, only auto-scroll if at bottom

## Future Enhancements

- **Voice Input**: Allow voice commands during generation
- **Suggested Prompts**: Show common refinement options as buttons
- **Task Editing**: Edit tasks inline before saving to database
- **Generation History**: Save and replay previous generation sessions
- **Multi-Agent**: Use separate agents for research and validation
