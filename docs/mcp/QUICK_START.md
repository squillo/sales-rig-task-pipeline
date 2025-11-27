# Rigger MCP Quick Start

Get Rigger MCP server running in your IDE in under 5 minutes.

## Step 1: Install Rigger

```bash
cd /path/to/rig-task-pipeline
cargo install --path rigger_cli
```

Verify installation:
```bash
rig --version
```

## Step 2: Initialize Rigger in Your Project

Navigate to your project directory and initialize Rigger:

```bash
cd /path/to/your/project
rig init
```

This creates a `.rigger` directory with:
- `tasks.db` - SQLite database for tasks
- `config.json` - Configuration file

## Step 3: Configure Your IDE

### For Cursor AI IDE

1. Create/edit `~/.cursor/mcp.json`:

```bash
mkdir -p ~/.cursor
cp docs/mcp/cursor_mcp_config.json ~/.cursor/mcp.json
```

**Or manually create** `~/.cursor/mcp.json`:
```json
{
  "mcpServers": {
    "rigger": {
      "command": "rig",
      "args": ["server"],
      "cwd": "${workspaceFolder}",
      "env": {}
    }
  }
}
```

2. Restart Cursor AI IDE

3. Verify: Type `@rig` in the chat and you should see Rigger tools available

### For Windsurf IDE

1. Create/edit `~/.windsurf/mcp.json`:

```bash
mkdir -p ~/.windsurf
cp docs/mcp/windsurf_mcp_config.json ~/.windsurf/mcp.json
```

**Or manually create** `~/.windsurf/mcp.json`:
```json
{
  "mcpServers": {
    "rigger": {
      "command": "rig",
      "args": ["server"],
      "cwd": "${workspaceFolder}",
      "env": {}
    }
  }
}
```

2. Restart Windsurf IDE

3. Verify: Access MCP tools through the tool palette

## Step 4: Test MCP Server Manually (Optional)

Before IDE integration, you can test the MCP server standalone:

```bash
cd /path/to/your/project
rig server
```

The server will start and listen on stdin. Test with a JSON-RPC request:

```json
{"jsonrpc":"2.0","method":"list_tasks","params":{},"id":1}
```

Press Ctrl+D (EOF) to stop the server.

## Step 5: Use Rigger in Your IDE

### Cursor AI Examples

In Cursor chat, you can now:

**List tasks:**
```
@rig list all tasks
```

**Add a task:**
```
@rig add task "Implement OAuth2 authentication" assigned to Alice due 2025-12-31
```

**Update task status:**
```
@rig update task abc123 to status InProgress
```

**Parse a PRD:**
```
@rig parse PRD from docs/PRD.md
```

### Windsurf Examples

In Windsurf MCP tool palette:

1. **rigger.list_tasks**: List all tasks or filter by status/assignee
2. **rigger.add_task**: Create new tasks
3. **rigger.update_task**: Update task status or assignee
4. **rigger.parse_prd**: Parse PRD files and extract objectives

## Common MCP Tool Patterns

### List tasks by status
```json
{
  "tool": "list_tasks",
  "params": {
    "status": "InProgress"
  }
}
```

### List tasks by assignee
```json
{
  "tool": "list_tasks",
  "params": {
    "assignee": "Alice"
  }
}
```

### Add task with all fields
```json
{
  "tool": "add_task",
  "params": {
    "title": "Implement user authentication",
    "assignee": "Bob",
    "due_date": "2025-12-31"
  }
}
```

### Update task status
```json
{
  "tool": "update_task",
  "params": {
    "task_id": "abc123",
    "status": "Completed"
  }
}
```

### Parse PRD
```json
{
  "tool": "parse_prd",
  "params": {
    "prd_file_path": "./docs/PRD.md"
  }
}
```

## Troubleshooting

### "rig: command not found"

The binary isn't in your PATH. Either:

1. Add `~/.cargo/bin` to your PATH:
   ```bash
   export PATH="$HOME/.cargo/bin:$PATH"
   ```

2. Or use full path in MCP config:
   ```json
   {
     "command": "/Users/yourname/.cargo/bin/rig",
     ...
   }
   ```

### ".rigger directory not found"

Run `rig init` in your project directory first.

### MCP server won't start in IDE

1. Test server manually: `rig server`
2. Check IDE output logs for error messages
3. Verify `mcp.json` is in the correct location
4. Ensure IDE has been restarted after config changes

## Next Steps

- Read the full [MCP Setup Guide](./MCP_SETUP.md) for advanced configuration
- Explore task orchestration with `rig orchestrate`
- Set up PRD parsing workflows
- Configure custom task decomposition rules

## Revision History

- 2025-11-23: Initial quick start guide for Phase 4 Sprint 8
