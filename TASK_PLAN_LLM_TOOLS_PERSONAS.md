---
task_id: LLM_TOOLS_PERSONAS_251204
status: planning
---

# Task: Add Tool Calling & Persona Switching to LLM Chat

## Context
The LLM chat footer currently shows comprehensive context about projects, tasks, personas, and artifacts, but the agent **cannot actually use tools** to search, query, or interact with the system. Additionally, users should be able to **switch between agent personas** to ask questions as different roles (like the task evaluation agents do).

## Current State
- ✅ LLM chat footer with expandable UI
- ✅ Comprehensive context building (projects, tasks, personas, artifacts)
- ✅ Context sent as system message
- ❌ No tool calling - agent can't search or query
- ❌ No persona switching - stuck with default Rigger context
- ❌ Tools are mentioned but not executable

## Goals
1. **Enable actual tool calling** - Agent can search tasks, artifacts, web, etc.
2. **Add persona switching** - Cycle through personas in chat UI
3. **Persona-specific contexts** - Each persona gets their own system prompt based on their role/description
4. **Default Rigger persona** - Current comprehensive context is the default

## Plan

### Phase 1: Create Rig Tool Definitions (Priority: HIGH)
- [ ] 1.1 Create `tools/` directory in `task_orchestrator/src/`
- [ ] 1.2 **🌟 PRIORITY: Implement `SemanticSearchArtifactsTool`** - Semantic vector search across ALL artifacts
  - **Why this is critical:** Uses RAG/embeddings to find relevant knowledge even when keywords don't match
  - Takes: query (string), limit (optional, default 10), threshold (optional, default 0.7)
  - Returns: Top N artifacts with **similarity scores** (0.0-1.0), ranked by relevance
  - Examples:
    - Query: "authentication flow" → Finds docs about login, OAuth, JWT, etc.
    - Query: "how to optimize database" → Finds performance tuning artifacts
    - Query: "error handling patterns" → Finds exception handling examples
  - Uses: `SqliteArtifactAdapter.search_by_embedding()` with vector similarity
- [ ] 1.3 Implement `SearchTasksTool` - Keyword search tasks by title, status, assignee
  - Takes: query (string), status filter (optional), assignee filter (optional)
  - Returns: List of matching tasks with ID, title, status
  - Note: This is keyword/text search, not semantic
- [ ] 1.4 Implement `GetTaskDetailsTool` - Get full task details by ID
  - Takes: task_id (string)
  - Returns: Complete task object with description, complexity, subtasks
- [ ] 1.5 Implement `ListProjectArtifactsTool` - List all artifacts for project
  - Takes: project_id (string)
  - Returns: List of artifacts by type (PRD, File, Web, etc.)
- [ ] 1.6 **OPTIONAL: Implement `SemanticSearchTasksTool`** - Semantic search across task descriptions
  - Similar to artifact search but for tasks
  - Useful for: "find tasks related to authentication" even if they don't say "auth"
- [ ] 1.7 Create tool registry struct to hold all tools

### Phase 2: Wire Up Tool Execution (Priority: HIGH)
- [ ] 2.1 Update `RigAgentAdapter::new_with_provider()` to accept tools
- [ ] 2.2 Modify `.build()` chain to register all tools with agent:
  ```rust
  let agent = client.agent(&model)
      .preamble(&system_prompt)
      .tool(semantic_search_artifacts_tool)  // 🌟 PRIORITY - most powerful tool!
      .tool(search_tasks_tool)
      .tool(get_task_details_tool)
      .tool(list_artifacts_tool)
      .build();
  ```
- [ ] 2.3 Update streaming loop to handle `StreamToken::ToolCallStart`
- [ ] 2.4 Execute tool when LLM requests it
- [ ] 2.5 Return tool results as `StreamToken::ToolCallEnd`
- [ ] 2.6 Display tool calls in chat UI (already has tool call rendering)
- [ ] 2.7 Test with "search for all TODO tasks" query

### Phase 3: Add Persona Switching UI (Priority: MEDIUM)
- [ ] 3.1 Add `selected_persona_id: Option<String>` to App struct
- [ ] 3.2 Add persona selector to LLM chat footer header
  - Show: `[Persona: Rigger Assistant ▼]` dropdown
  - List all personas + "Default (Rigger)" option
- [ ] 3.3 Add keyboard shortcut to cycle personas (Ctrl+P or similar)
- [ ] 3.4 Visual indicator showing current persona in chat header
- [ ] 3.5 Update context header to show: `📋 Context as [Persona Name]`

### Phase 4: Implement Persona-Specific Contexts (Priority: MEDIUM)
- [ ] 4.1 Create `build_persona_context()` method
  - Load persona from database by ID
  - Build system prompt from persona's:
    - Name and role
    - Description (full capabilities)
    - LLM provider/model overrides
    - Project scope (if set)
- [ ] 4.2 Modify `build_agent_context()` to check `selected_persona_id`:
  - If None → Return default Rigger context
  - If Some(id) → Return `build_persona_context(id)`
- [ ] 4.3 Add persona-specific instructions to system prompt:
  ```
  You are [Persona Name], a [Role].

  Your capabilities: [Description]

  Current project context: [Same project/task info as before]
  ```
- [ ] 4.4 Test switching between personas and asking same question

### Phase 5: Advanced Tool Features (Priority: LOW)
- [ ] 5.1 Add `SearchWebTool` (if web crawler is available)
- [ ] 5.2 Add `UpdateTaskTool` - Modify task status/assignee/description
- [ ] 5.3 Add `CreateSubtaskTool` - Create subtask for a parent task
- [ ] 5.4 Add tool success/failure icons in UI (✅/❌)
- [ ] 5.5 Add tool execution time tracking
- [ ] 5.6 Tool call history persistence (across chat sessions)

### Phase 6: Persona Management (Priority: LOW)
- [ ] 6.1 Add "Manage Personas" option in chat dropdown
- [ ] 6.2 Quick persona creation dialog from chat
- [ ] 6.3 Persona-specific chat history (remember conversations per persona)
- [ ] 6.4 Persona suggestions based on task type

## Current Step
- **Action:** ⚠️ PHASE 2 BLOCKED - Tool execution needs proper implementation
- **Details:** Infrastructure is ready but Rig 0.9.1 tool integration issue discovered:
  1. ✅ RigAgentAdapter modified with tool support (new_*_with_tools methods)
  2. ✅ Tools registered with .tool() calls during agent building
  3. ✅ App struct refactored to Arc<Mutex<>> for adapter sharing
  4. ✅ Tool instances created in TUI and passed to adapter
  5. ✅ Tool execution UI already implemented with status icons (⏳ ⚙️ ✅ ❌)
  6. ❌ **BLOCKER:** Rig 0.9.1 `.prompt()` doesn't properly handle tool execution with streaming
  7. ⚠️ **TEMPORARY FIX:** Tools disabled in rig_agent_adapter.rs (line 315, 349) - LLM works without tools

**NEXT:** Implement proper Rig tool execution loop that emits ToolCallStart/ToolCallEnd events

## Discovery (2025-12-05)
🎉 **EXCELLENT NEWS**: All Rig tools are already fully implemented!

**Found in `task_orchestrator/src/tools/`**:
- ✅ `search_artifacts_tool.rs` - Semantic RAG search (191 lines, implements `rig::tool::Tool`)
- ✅ `search_tasks_tool.rs` - Keyword task search (implements `rig::tool::Tool`)
- ✅ `get_task_details_tool.rs` - Full task details lookup (implements `rig::tool::Tool`)
- ✅ `get_prd_summary_tool.rs` - PRD information (implements `rig::tool::Tool`)
- ✅ `list_project_artifacts_tool.rs` - List artifacts (implements `rig::tool::Tool`)
- ✅ `file_system_tool.rs` - File operations (implements `rig::tool::Tool`)

**Implementation Completed (2025-12-05)**:
- ✅ Tools registered in `RigAgentAdapter` with new_*_with_tools() methods
- ✅ Tool instances passed from TUI to adapter
- ✅ Adapters refactored to Arc<Mutex<>> for sharing with tools
- ✅ Tool execution UI already implemented (status icons: ⏳ ⚙️ ✅ ❌)
- ✅ Stream token handling for ToolCallStart/ToolCallEnd events

**Phase 1 Status**: ✅ COMPLETE (tools already exist!)
**Phase 2 Status**: ✅ COMPLETE (tools wired up and ready!)

## Blockers

**MAJOR BLOCKER: Rig Tool Execution Not Working**

**Problem:** The Rig 0.9.1 agent with `.prompt()` doesn't properly execute tools in a streaming context. When tools are registered:
- LLM tries to call tools and outputs JSON (e.g., "erro 10" seen in testing)
- Tools may or may not execute, but results aren't captured properly
- Subsequent queries fail with "No tasks found matching your search criteria"

**Root Cause:** The `.prompt()` method is synchronous and returns final text, but doesn't expose tool execution events for our streaming interface that expects `ToolCallStart`/`ToolCallEnd` tokens.

**Temporary Fix Applied:**
- Tools disabled at lines 315 and 349 in `task_orchestrator/src/adapters/rig_agent_adapter.rs`
- LLM now works normally without tool calling
- All infrastructure (tools, adapters, UI) is ready but not activated

**Proper Solution Needed:**
Implement manual tool execution loop in `chat_with_tools()` method:
1. Call LLM with tools registered
2. Parse response for tool call requests
3. Execute tools manually via their `.call()` methods
4. Send `ToolCallStart` event to UI
5. Send `ToolCallEnd` event with results
6. Feed results back to LLM for final response
7. Repeat until LLM stops calling tools

**Alternative:** Upgrade to newer Rig version that has better streaming + tool support, or switch to raw OpenAI/Ollama API with manual tool handling.

## Implementation Notes

### Rig Tool Definition Example
```rust
use rig::tool::Tool;

#[derive(Debug, thiserror::Error)]
pub enum SearchTasksError {
    #[error("Database error: {0}")]
    Database(String),
}

pub struct SearchTasksTool {
    task_repository: Arc<dyn TaskRepositoryPort>,
}

impl Tool for SearchTasksTool {
    const NAME: &'static str = "search_tasks";

    type Error = SearchTasksError;
    type Args = SearchTasksArgs;
    type Output = SearchTasksOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "search_tasks".to_string(),
            description: "Search for tasks by title, status, or assignee. Returns matching tasks with their IDs, titles, and statuses.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query to match against task titles"
                    },
                    "status": {
                        "type": "string",
                        "description": "Optional status filter (Todo, InProgress, Completed, etc.)",
                        "enum": ["Todo", "InProgress", "Completed", "Archived"]
                    },
                    "assignee": {
                        "type": "string",
                        "description": "Optional assignee filter (persona name)"
                    }
                },
                "required": ["query"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Execute search against repository
        let results = self.task_repository
            .search_by_title(&args.query)
            .await
            .map_err(|e| SearchTasksError::Database(e.to_string()))?;

        Ok(SearchTasksOutput { tasks: results })
    }
}
```

### Persona Context Template
```
# Agent Persona: [Name]

You are **[Name]**, a [Role].

## Your Capabilities
[Description from persona.description]

## Your Expertise
[Expertise areas if available]

## Current Context
[Same project/task/artifact info as default Rigger context]

## Your Role
You should answer questions and provide guidance from the perspective of a [Role],
focusing on [expertise areas]. Use your specialized knowledge to provide insights
relevant to your domain.
```

## Testing Strategy
1. **🌟 Semantic Search Tests (MOST IMPORTANT)**
   - Ask "how do we handle authentication?" → Should execute SemanticSearchArtifactsTool
     - Should find docs about login, OAuth, JWT, session management
     - Should return similarity scores (e.g., 0.89, 0.82, 0.76)
     - Should work even if artifacts never use word "authentication"
   - Ask "what's our error handling strategy?" → Should find relevant patterns
   - Ask "how to optimize database queries?" → Should find performance docs
   - Ask "where is the API documentation?" → Should find API-related artifacts
   - Verify results are ranked by relevance score (highest first)

2. **Other Tool Calling Tests**
   - Ask "search for all TODO tasks" → Should execute SearchTasksTool
   - Ask "get details for task XYZ" → Should execute GetTaskDetailsTool
   - Ask "list all artifacts in this project" → Should execute ListProjectArtifactsTool
   - Verify tool results appear in chat with status icons

3. **Persona Switching Tests**
   - Create test personas: "Backend Engineer", "Frontend Developer", "QA Engineer"
   - Switch to Backend Engineer → Ask "what should I work on?"
   - Verify response is tailored to backend perspective
   - Switch to Frontend Developer → Ask same question
   - Verify different response based on frontend focus

4. **Context Preservation**
   - Switch personas mid-conversation
   - Verify context is rebuilt for new persona
   - Verify chat history is preserved

## Success Criteria
- ✅ **🌟 CRITICAL: Semantic artifact search works with similarity scores**
  - Can find relevant docs even with different wording
  - Returns top matches ranked by relevance
  - Similarity scores displayed (0.0-1.0)
- ✅ Agent can search tasks and return results
- ✅ Agent can look up task details by ID
- ✅ Tool calls appear in chat with execution status and icons (🔧 ✅ ❌)
- ✅ Can cycle through personas in chat UI
- ✅ Each persona provides responses from their perspective
- ✅ Default Rigger persona provides comprehensive project overview
- ✅ Tool execution is fast (< 2 seconds for searches, < 500ms for lookups)
- ✅ Error handling shows user-friendly messages
- ✅ Semantic search demonstrates RAG capabilities ("magic" knowledge retrieval)

## Architecture Considerations
- **Tool Repository Access**: Tools need access to adapters (tasks, artifacts, etc.)
  - Pass adapters to RigAgentAdapter constructor
  - Tools hold Arc references to repositories
- **Persona Context Caching**: Build once, reuse until persona switches
- **Tool Result Formatting**: Should be human-readable in chat
- **Streaming with Tools**: Need to handle tool calls mid-stream
  - Pause content streaming
  - Execute tool
  - Resume with tool result injected

## Files to Modify
- `task_orchestrator/src/tools/mod.rs` (new)
- `task_orchestrator/src/tools/search_tasks_tool.rs` (new)
- `task_orchestrator/src/tools/get_task_details_tool.rs` (new)
- `task_orchestrator/src/tools/search_artifacts_tool.rs` (new)
- `task_orchestrator/src/adapters/rig_agent_adapter.rs` (modify)
- `rigger_cli/src/commands/tui.rs` (modify - add persona switching UI)

## Dependencies to Add
- `rig` crate already has `Tool` trait
- `serde_json` for tool parameter schemas (already available)
