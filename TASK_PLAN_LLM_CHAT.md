---
task_id: LLM_CHAT_AGENT_IMPLEMENTATION
status: planning
---

# Task: Implement LLM Chat with Chain-of-Thought Agent and Streaming

## Overview

Transform the existing placeholder LLM Chat dialog into a fully functional chain-of-thought agent that can search artifacts, query tasks, inspect PRDs, and provide streaming responses. The chat will be accessible via the `l` key and will use Rig for agent orchestration following hexagonal architecture principles.

## Requirements

### Functional Requirements
1. **Activation**: Press `l` to open chat dialog from any view
2. **Context Awareness**: Agent knows current project, PRD, and selected task/artifact
3. **Tool Access**: Agent can use:
   - `search_artifacts` - Semantic search through knowledge base
   - `search_tasks` - Find tasks by title, status, or content
   - `get_task_details` - Retrieve full task information
   - `get_prd_summary` - Get PRD objectives and constraints
   - `list_project_artifacts` - List all artifacts for current project
4. **Streaming**: Real-time token streaming with visual feedback
5. **Chain of Thought**: Agent shows reasoning and tool calls inline
6. **History**: Maintain conversation history within session
7. **Keyboard Navigation**: Standard dialog controls (Esc to close, Enter to send)

### Non-Functional Requirements
- Follow hexagonal architecture (Port/Adapter pattern)
- Use Rig crate for agent orchestration
- Streaming must not block UI rendering
- Tool calls should show in chat with clear visual indicators
- Maximum response time: 30 seconds with timeout handling

## Plan

### Phase 1: Port and Adapter Infrastructure (Hexagonal Architecture)
- [ ] 1.1. Create `LLMAgentPort` trait in `task_orchestrator/src/ports/llm_agent_port.rs`
  - Define `chat_with_tools()` method signature
  - Define `AgentMessage` and `AgentResponse` types
  - Support streaming via callback or channel
- [ ] 1.2. Create `RigAgentAdapter` in `task_orchestrator/src/adapters/rig_agent_adapter.rs`
  - Implement `LLMAgentPort` using Rig
  - Configure OpenAI/Ollama client
  - Add system prompt for task management context
- [ ] 1.3. Add agent configuration to environment
  - `LLM_AGENT_PROVIDER` (openai, ollama, anthropic)
  - `LLM_AGENT_MODEL` (gpt-4o-mini, llama3.2, etc.)
  - `LLM_AGENT_STREAMING` (true/false)

### Phase 2: Tool Implementation
- [ ] 2.1. Create `SearchTasksTool` in `task_orchestrator/src/tools/search_tasks_tool.rs`
  - Implement Rig `Tool` trait
  - Search by title substring, status, agent_persona
  - Return formatted list of matching tasks
- [ ] 2.2. Create `GetTaskDetailsTool` in `task_orchestrator/src/tools/get_task_details_tool.rs`
  - Fetch task by ID
  - Return full task details (description, complexity, dependencies, reasoning)
- [ ] 2.3. Create `GetPRDSummaryTool` in `task_orchestrator/src/tools/get_prd_summary_tool.rs`
  - Fetch PRD by ID or current project
  - Return objectives, tech_stack, constraints
- [ ] 2.4. Create `ListProjectArtifactsTool` in `task_orchestrator/src/tools/list_project_artifacts_tool.rs`
  - List artifacts filtered by project_id
  - Return source types and content previews
- [ ] 2.5. Update `search_artifacts_tool.rs` to export for agent use
  - Already exists, ensure compatibility with Rig agent

### Phase 3: Agent State Management in TUI
- [ ] 3.1. Add agent-related fields to App struct
  - `llm_agent_streaming: bool` - Is response currently streaming?
  - `llm_agent_current_response: String` - Accumulated streaming response
  - `llm_agent_tool_calls: Vec<ToolCall>` - Record of tool invocations
  - `llm_agent_thinking: bool` - Agent is processing/using tools
- [ ] 3.2. Add `ToolCall` struct to track tool usage
  ```rust
  struct ToolCall {
      tool_name: String,
      args: String,
      result: Option<String>,
      status: ToolCallStatus, // Pending, Running, Success, Failed
  }
  ```
- [ ] 3.3. Add chat session management methods
  - `reset_chat_session()` - Clear history and state
  - `add_user_message()` - Append user message to history
  - `append_assistant_token()` - Add streaming token to current response

### Phase 4: Streaming Integration
- [ ] 4.1. Create async channel for streaming tokens
  - Use `tokio::sync::mpsc` channel
  - Sender passed to agent adapter
  - Receiver polled in event loop
- [ ] 4.2. Implement token streaming in `RigAgentAdapter`
  - Use Rig's streaming API
  - Send tokens through channel as they arrive
  - Handle tool calls mid-stream
- [ ] 4.3. Add streaming event handling to TUI event loop
  - Poll channel on every iteration
  - Append tokens to `llm_agent_current_response`
  - Trigger re-render when tokens arrive
- [ ] 4.4. Add timeout handling
  - 30-second timeout for agent responses
  - Show warning if timeout occurs
  - Allow cancellation with Esc key

### Phase 5: Chat UI Enhancements
- [ ] 5.1. Update `render_llm_chat_dialog()` to show streaming
  - Render current streaming response with pulsing cursor
  - Show "Thinking..." indicator when `llm_agent_thinking` is true
- [ ] 5.2. Add visual indicators for tool calls
  - Show tool name and args in chat with distinct color
  - Show tool results when available
  - Use icons: 🔍 search, 📋 get_task, 📄 get_prd, etc.
- [ ] 5.3. Improve message rendering
  - Word-wrap long responses
  - Scroll to bottom when new messages arrive
  - Add scroll position indicator if history is long
- [ ] 5.4. Add help text footer
  - Show available commands and shortcuts
  - List available tools for transparency

### Phase 6: Context Injection
- [ ] 6.1. Build context payload when opening chat
  - Current project name and ID
  - Selected PRD (if any)
  - Selected task (if any)
  - Selected artifact (if any)
  - Active view (Kanban, PRDView, ArtifactViewer, Metrics)
- [ ] 6.2. Add system message with context
  - Format context as structured text
  - Inject as first message in conversation
- [ ] 6.3. Update context on view changes (optional enhancement)
  - Detect if user switches views mid-chat
  - Optionally update context with new selection

### Phase 7: Keyboard Handling
- [ ] 7.1. Add `l` key binding to open chat
  - Works from any view
  - Initializes context before opening
- [ ] 7.2. Implement chat input handling
  - Character input appends to `llm_chat_input`
  - Backspace removes last character
  - Enter sends message and clears input
- [ ] 7.3. Add Esc key to close chat
  - Ask for confirmation if response is streaming
- [ ] 7.4. Add scroll controls for long history
  - Page Up/Down to scroll chat history

### Phase 8: Error Handling
- [ ] 8.1. Handle API errors gracefully
  - Network failures
  - Authentication errors (missing API key)
  - Rate limiting
- [ ] 8.2. Handle tool execution errors
  - Show error message inline
  - Continue conversation after error
- [ ] 8.3. Add retry mechanism
  - Allow user to retry failed requests
  - Show "Retry" option on error

### Phase 9: Testing
- [ ] 9.1. Write unit tests for tools
  - Test SearchTasksTool with various filters
  - Test GetTaskDetailsTool with valid/invalid IDs
  - Test GetPRDSummaryTool
  - Test ListProjectArtifactsTool
- [ ] 9.2. Write integration tests for agent
  - Test tool calling with mock Rig agent
  - Test streaming token accumulation
  - Test context injection
- [ ] 9.3. Manual testing scenarios
  - Ask about tasks in current project
  - Request PRD summary
  - Search for artifacts
  - Multi-turn conversation with context

### Phase 10: Documentation and Polish
- [ ] 10.1. Document agent capabilities in README
  - List available tools
  - Show example queries
- [ ] 10.2. Add inline help in chat
  - `/help` command to show available tools
  - `/clear` command to reset conversation
- [ ] 10.3. Optimize performance
  - Profile streaming latency
  - Reduce UI re-render overhead
- [ ] 10.4. Add configuration options
  - Temperature control
  - Max tokens
  - System prompt customization

## Current Step

- **Action:** Planning complete, ready to begin Phase 1 implementation
- **Details:** Will start with port/adapter infrastructure for LLMAgentPort and RigAgentAdapter following hexagonal architecture.

## Technical Design Notes

### Hexagonal Architecture Layers

```
┌─────────────────────────────────────────────────────────┐
│ TUI (rigger_cli)                                        │
│ - Renders chat dialog                                   │
│ - Handles keyboard input                                │
│ - Manages streaming state                               │
│ - Calls LLMAgentPort (dependency injection)             │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────┐
│ LLMAgentPort (task_orchestrator/ports)                  │
│ - chat_with_tools(messages, tools) -> Stream<Token>     │
│ - Defines abstract agent interface                      │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────┐
│ RigAgentAdapter (task_orchestrator/adapters)            │
│ - Implements LLMAgentPort using Rig                     │
│ - Configures Rig agent with tools                       │
│ - Handles streaming via OpenAI/Ollama client            │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────┐
│ Rig Tools (task_orchestrator/tools)                     │
│ - SearchArtifactsTool (already exists)                  │
│ - SearchTasksTool (new)                                 │
│ - GetTaskDetailsTool (new)                              │
│ - GetPRDSummaryTool (new)                               │
│ - ListProjectArtifactsTool (new)                        │
└─────────────────────────────────────────────────────────┘
```

### System Prompt Template

```
You are Rigger, an AI assistant helping developers manage their project tasks and knowledge.

Current Context:
- Project: {project_name} (ID: {project_id})
- Active View: {view_name}
- Selected PRD: {prd_title} (ID: {prd_id})
- Selected Task: {task_title} (ID: {task_id})

You have access to the following tools:
1. search_artifacts: Search the knowledge base for relevant information
2. search_tasks: Find tasks by title, status, or assignee
3. get_task_details: Get full details for a specific task
4. get_prd_summary: Retrieve PRD objectives and constraints
5. list_project_artifacts: List all artifacts for the current project

When answering questions, use your tools to provide accurate, context-aware responses.
Show your reasoning step-by-step when using tools.
```

### Streaming Token Flow

```
User presses Enter
    ↓
TUI sends message to agent via LLMAgentPort
    ↓
RigAgentAdapter calls Rig with tools
    ↓
Rig streams tokens back via channel
    ↓
TUI event loop polls channel
    ↓
Tokens appended to llm_agent_current_response
    ↓
UI re-renders with updated response
    ↓
Stream completes
    ↓
Response moved to llm_chat_history
```

### Tool Call Rendering Example

```
👤 You: What tasks are assigned to the Backend Developer?

🤖 Assistant: Let me search for tasks assigned to the Backend Developer.

🔍 search_tasks(filter: "Backend Developer", status: "all")
   → Found 3 tasks:
   1. [#123] Implement authentication API
   2. [#124] Add database migrations
   3. [#125] Configure Redis caching

Based on the search, the Backend Developer currently has 3 tasks...
```

## Dependencies

### Existing Components to Leverage
- SearchArtifactsTool (already implemented)
- EmbeddingPort and RigEmbeddingAdapter
- Task/PRD/Artifact repositories
- ChatMessage and ChatRole structs (already exist)

### New Crates Required
- `rig-core` (already in workspace dependencies)
- No additional external dependencies needed

## Blockers

None currently. All required ports and adapters exist or can be implemented following established patterns.

## Success Criteria

1. Can open chat dialog with `l` key from any view
2. Agent responds with streaming tokens visible in real-time
3. Agent can successfully call all 5 tools
4. Tool calls are visible in chat with clear formatting
5. Context is correctly injected (current project/PRD/task)
6. Chat history persists within session
7. Esc key closes dialog
8. No UI blocking during streaming
9. Error messages are user-friendly
10. All tests pass

## Future Enhancements (Out of Scope)

- Multi-modal support (images, diagrams)
- Voice input/output
- Chat history persistence across sessions
- Multi-agent collaboration
- Custom tool creation by user
- Export chat to markdown
- Share chat sessions via URL
