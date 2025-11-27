# rigger_cli

Command-line interface for the Rigger task management system. Provides intelligent task orchestration, PRD parsing, and IDE integration via Model Context Protocol (MCP).

## Installation

### Option 1: Install from workspace (Recommended for development)

```bash
# From repository root
cargo install --path rigger_cli
```

This installs the `rig` binary to `~/.cargo/bin/rig`.

### Option 2: Install from crates.io (Future)

```bash
cargo install rigger_cli
```

### Option 3: Build for local testing

```bash
# From repository root
cargo build --release --package rigger_cli

# Binary will be at: ./target/release/rig
```

## Quick Start

### Initialize a new project

```bash
cd your-project
rig init
```

This creates `.rigger/` directory with:
- `tasks.db` - SQLite task database
- `config.json` - Project configuration
- `prds/` - Directory for PRD markdown files

### Parse a PRD into tasks

```bash
rig parse path/to/your-prd.md
```

Rigger will:
1. Extract objectives, tech stack, and constraints from the PRD
2. Generate actionable tasks using LLM decomposition
3. Store tasks in `.rigger/tasks.db`

### List tasks

```bash
rig list                    # List all tasks
rig list --status todo      # Filter by status
rig list --assignee alice   # Filter by assignee
```

### Work on a task

```bash
rig do <task-id>
```

This runs the task through the orchestration flow:
1. **Triage**: Scores complexity and dependencies
2. **Route**: Simple tasks → enhancement; Complex tasks → decomposition
3. **Enhance**: Adds implementation details and acceptance criteria
4. **Test**: Generates comprehension tests
5. **Complete**: Marks task as done when tests pass

### Start MCP server (for IDE integration)

```bash
rig server
```

Starts a JSON-RPC 2.0 server over stdio, compatible with Model Context Protocol. See **IDE Integration** section below.

### Start gRPC server (for programmatic access)

```bash
rig grpc
```

Starts a gRPC server on `127.0.0.1:50051` for programmatic task management.

### Launch Terminal UI (TUI)

```bash
rig tui
```

Launches an interactive terminal user interface with:
- **Task Board**: Kanban-style columns (TODO, IN PROGRESS, COMPLETED)
- **Thinking Widget**: Chain-of-thought reasoning visualization
- **Network Log**: API request/response monitoring
- **Help Screen**: Full keyboard controls

**Keyboard Controls**:
- `Tab` / `Shift+Tab`: Switch between tabs
- `↑` / `k`: Navigate up
- `↓` / `j`: Navigate down
- `r`: Refresh tasks from database
- `q` / `Esc`: Quit TUI

## IDE Integration

Rigger supports the [Model Context Protocol](https://modelcontextprotocol.io) (MCP), enabling seamless integration with AI-powered IDEs like Cursor and Windsurf.

### MCP Server Features

When running `rig server`, the MCP server exposes:

**Tools**:
- `parse_prd`: Parse a PRD file into tasks
- `list_tasks`: Query tasks with filters (status, assignee)
- `add_task`: Create a new task
- `update_task`: Update task status, priority, or assignee

**Resources**:
- `tasks.json`: Live view of all tasks in the database
- `config.json`: Project configuration (provider, model)

### Setup for Cursor

1. **Install Rigger**:
   ```bash
   cargo install --path rigger_cli
   ```

2. **Add to Cursor MCP config** (`.cursor/mcp.json`):
   ```json
   {
     "mcpServers": {
       "rigger": {
         "command": "rig",
         "args": ["server"],
         "cwd": "${workspaceFolder}",
         "env": {},
         "description": "Rigger task management MCP server - provides intelligent task decomposition, tracking, and PRD parsing"
       }
     }
   }
   ```

   Template available at: `docs/mcp/cursor_mcp_config.json`

3. **Restart Cursor** to load the MCP server.

4. **Use in chat**:
   ```
   @rigger list_tasks status:todo
   @rigger parse_prd docs/my-feature.md
   ```

### Setup for Windsurf

1. **Install Rigger**:
   ```bash
   cargo install --path rigger_cli
   ```

2. **Add to Windsurf config**:
   ```json
   {
     "mcpServers": {
       "rigger": {
         "command": "rig",
         "args": ["server"],
         "cwd": "${workspaceFolder}",
         "env": {},
         "description": "Rigger task management MCP server - provides intelligent task decomposition, tracking, and PRD parsing"
       }
     }
   }
   ```

   Template available at: `docs/mcp/windsurf_mcp_config.json`

3. **Restart Windsurf** to load the MCP server.

### Binary Path Resolution

The MCP configuration uses `"command": "rig"`, which assumes the binary is in your `PATH`.

**If `rig` is not found**:

- **After `cargo install`**: The binary is at `~/.cargo/bin/rig`. Ensure `~/.cargo/bin` is in your `PATH`:
  ```bash
  export PATH="$HOME/.cargo/bin:$PATH"
  ```

- **For local builds**: Use absolute path in MCP config:
  ```json
  {
    "command": "/absolute/path/to/rig-task-pipeline/target/release/rig"
  }
  ```

- **macOS/Linux**: Verify binary location:
  ```bash
  which rig
  ```

- **Windows**: Use PowerShell:
  ```powershell
  Get-Command rig
  ```

### Testing MCP Integration

1. **Check server starts**:
   ```bash
   rig server
   ```
   Should wait for stdin (press `Ctrl+C` to exit).

2. **Send test message** (optional):
   ```bash
   echo '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' | rig server
   ```
   Should respond with available tools.

3. **In IDE**: Open chat and type `@rigger` - you should see task-related tools.

## Configuration

Rigger reads configuration from environment variables and `.rigger/config.json`.

### Environment Variables

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `TASK_ORCHESTRATOR_PROVIDER` | LLM provider | `ollama` | `ollama`, `openai`, `anthropic` |
| `OLLAMA_MODEL` | Ollama model | `llama3.1` | `llama3.1`, `qwen2.5` |
| `OPENAI_MODEL` | OpenAI model | `gpt-4` | `gpt-4`, `gpt-4-turbo` |
| `ANTHROPIC_MODEL` | Anthropic model | `claude-3-5-sonnet-20241022` | `claude-3-5-sonnet-20241022` |
| `OPENAI_API_KEY` | OpenAI API key | - | `sk-...` |
| `ANTHROPIC_API_KEY` | Anthropic API key | - | `sk-ant-...` |
| `TEST_TYPE` | Comprehension test type | `short_answer` | `short_answer`, `multiple_choice`, `true_false` |

### Provider Support

- **Ollama**: ✅ Fully supported (no API key required, runs locally)
- **OpenAI**: 🚧 Planned (requires API key)
- **Anthropic**: 🚧 Planned (requires API key)

### Example: Switching providers

```bash
# Use Ollama (default)
export TASK_ORCHESTRATOR_PROVIDER=ollama
export OLLAMA_MODEL=llama3.1

# Or use Anthropic (once implemented)
export TASK_ORCHESTRATOR_PROVIDER=anthropic
export ANTHROPIC_MODEL=claude-3-5-sonnet-20241022
export ANTHROPIC_API_KEY=your-key-here
```

## Architecture

Rigger follows **Hexagonal Architecture** (Ports & Adapters):

- **Domain Layer**: Task, PRD, Enhancement, ComprehensionTest entities
- **Ports**: Trait definitions (TaskRepository, TaskEnhancementPort, etc.)
- **Adapters**: Concrete implementations (SqliteTaskAdapter, OllamaEnhancementAdapter, RigPRDParserAdapter)
- **Use Cases**: Application orchestration logic
- **CLI**: User-facing commands (init, parse, list, do, server, grpc)

### Workflow: rig do <task-id>

```
┌─────────────┐
│   rig do    │
└──────┬──────┘
       │
       v
┌─────────────────────┐
│  1. Load Task from  │
│     SQLite DB       │
└──────┬──────────────┘
       │
       v
┌─────────────────────────────────────────┐
│  2. Run Orchestration Flow (graph_flow) │
│                                          │
│  ┌─────────────────────────┐            │
│  │  SemanticRouter         │            │
│  │  (Triage complexity)    │            │
│  └────┬────────────┬───────┘            │
│       │            │                     │
│   complexity < 7   complexity >= 7      │
│       │            │                     │
│       v            v                     │
│  ┌─────────┐  ┌──────────────┐          │
│  │ Enhance │  │  Decompose   │          │
│  └────┬────┘  │  (Subtasks)  │          │
│       │       └──────────────┘          │
│       v                                  │
│  ┌──────────────────┐                   │
│  │ ComprehensionTest│                   │
│  └────┬─────────────┘                   │
│       │                                  │
│       v                                  │
│  ┌──────────────┐                       │
│  │  Check Test  │                       │
│  └────┬─────────┘                       │
│       │                                  │
│   pass / fail                            │
│       │                                  │
│       v                                  │
│  ┌─────────┐                            │
│  │   End   │                            │
│  └─────────┘                            │
└─────────────────────────────────────────┘
       │
       v
┌──────────────────┐
│  3. Save Updated │
│     Task to DB   │
└──────────────────┘
```

## Development

### Running Tests

```bash
# All tests
cargo test --package rigger_cli

# Specific integration test
cargo test --package rigger_cli --test integration_prd_workflow

# Ignored tests (require Ollama)
cargo test --package rigger_cli -- --ignored
```

### Project Structure

```
rigger_cli/
├── Cargo.toml
├── README.md (this file)
├── src/
│   ├── main.rs              # CLI entry point (clap)
│   ├── commands/
│   │   ├── init.rs          # rig init
│   │   ├── parse.rs         # rig parse <prd>
│   │   ├── list.rs          # rig list
│   │   ├── do_task.rs       # rig do <id>
│   │   ├── server.rs        # rig server (MCP JSON-RPC)
│   │   └── grpc_server.rs   # rig grpc
│   └── lib.rs
└── tests/
    └── integration_prd_workflow.rs
```

### Adding a New Command

1. Create `src/commands/your_command.rs`
2. Implement `pub async fn execute(...) -> hexser::HexResult<()>`
3. Add subcommand to `main.rs` clap enum
4. Wire up in `match commands { ... }`
5. Add tests to `tests/integration_*.rs`

## Examples

### End-to-end PRD workflow

```bash
# 1. Initialize project
cd my-rust-api
rig init

# 2. Create a PRD
cat > .rigger/prds/auth-feature.md <<EOF
# Authentication Feature

## Objectives
- Implement JWT-based authentication
- Add user registration and login endpoints
- Secure existing API routes

## Tech Stack
- Rust (axum framework)
- jsonwebtoken crate
- bcrypt for password hashing

## Constraints
- Must support token refresh
- No passwords stored in plaintext
EOF

# 3. Parse PRD into tasks
rig parse .rigger/prds/auth-feature.md

# 4. List generated tasks
rig list

# Output:
# [ ] 1a3f - Implement JWT token generation
# [ ] 2b4g - Add user registration endpoint
# [ ] 3c5h - Add login endpoint
# [ ] 4d6i - Secure existing API routes

# 5. Work on first task
rig do 1a3f

# Rigger will:
# - Enhance task with implementation details
# - Generate comprehension test
# - Mark as complete when passing
```

### Using with MCP in Cursor

```
# In Cursor chat:
User: @rigger list_tasks status:todo

# Cursor calls Rigger MCP server, shows:
Tasks (Todo):
1. 1a3f - Implement JWT token generation
2. 2b4g - Add user registration endpoint
3. 3c5h - Add login endpoint

User: @rigger update_task task_id:1a3f status:in_progress

# Task updated in database, visible in next list call
```

## Troubleshooting

### `rig: command not found`

**Problem**: Binary not in PATH.

**Solution**:
```bash
export PATH="$HOME/.cargo/bin:$PATH"
# Add to ~/.bashrc or ~/.zshrc for persistence
```

### MCP server not appearing in Cursor

**Problem**: Config file not loaded or incorrect path.

**Solution**:
1. Verify config location: `.cursor/mcp.json` in project root
2. Check binary path: `which rig`
3. Restart Cursor completely
4. Check Cursor logs for MCP errors

### Ollama connection errors

**Problem**: `rig do` fails with "connection refused".

**Solution**:
```bash
# Start Ollama
ollama serve

# Verify model is installed
ollama list | grep llama3.1
ollama pull llama3.1  # if not present
```

### SQLite database locked

**Problem**: `rig` commands fail with "database is locked".

**Solution**:
- Close other processes using `.rigger/tasks.db`
- Stop `rig server` if running
- Check for stale lock files

## License

See repository root for license information.

## Contributing

See repository root `CONTRIBUTING.md` for guidelines.

---

**Revision History**:
- 2025-11-23 @AI: Create rigger_cli README with MCP installation instructions (Phase 4 Sprint 8 Task 4.5).
