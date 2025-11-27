# IDE Configuration for Rigger MCP Server

This directory contains configuration templates for integrating Rigger with various IDEs via the Model Context Protocol (MCP).

## Supported IDEs

- **Cursor** - AI-first code editor
- **Windsurf** - AI-powered development environment
- **Any MCP-compatible IDE**

## Installation

### 1. Install Rigger CLI

```bash
# From the rig-task-pipeline directory
cargo install --path rigger_cli

# Verify installation
rig --version
```

### 2. Initialize Your Project

```bash
# Navigate to your project directory
cd /path/to/your/project

# Initialize Rigger
rig init
```

### 3. Configure Your IDE

#### Cursor

1. Open Cursor Settings (Cmd+, or Ctrl+,)
2. Navigate to "MCP Servers" or "Extensions"
3. Add a new MCP server with this configuration:

```json
{
  "mcpServers": {
    "rigger": {
      "command": "rig",
      "args": ["server"]
    }
  }
}
```

Or copy `cursor_mcp_config.json` to your Cursor settings location:
- macOS: `~/.cursor/mcp.json`
- Linux: `~/.config/cursor/mcp.json`
- Windows: `%APPDATA%\Cursor\mcp.json`

4. Restart Cursor

#### Windsurf

1. Open Windsurf settings
2. Navigate to MCP server configuration
3. Add the contents of `windsurf_config.json`
4. Restart Windsurf

### 4. Verify Installation

In your IDE's AI chat, try these commands:

```
@rig list_tasks
```

If you see a task list (or "No tasks found"), the integration is working!

## Available Tools

### `list_tasks`

List all tasks with optional filters.

**Parameters:**
- `status` (optional): Filter by task status (Todo, InProgress, Completed, etc.)
- `assignee` (optional): Filter by assignee name

**Example:**
```
@rig list_tasks {"status": "Todo"}
```

### `add_task`

Create a new task manually.

**Parameters:**
- `title` (required): Task title/description
- `assignee` (optional): Person responsible
- `priority` (optional): Task priority

**Example:**
```
@rig add_task {"title": "Implement authentication", "assignee": "Alice", "priority": "high"}
```

### `update_task`

Update an existing task's status or priority.

**Parameters:**
- `task_id` (required): ID of the task to update
- `status` (optional): New status (Todo, InProgress, Completed, etc.)
- `priority` (optional): New priority

**Example:**
```
@rig update_task {"task_id": "abc123", "status": "Completed"}
```

### `parse_prd`

Parse a Product Requirements Document (PRD) and generate tasks using LLM decomposition.

**Parameters:**
- `prd_file` (required): Path to the PRD markdown file

**Example:**
```
@rig parse_prd {"prd_file": "docs/PRD.md"}
```

## Available Resources

### `tasks.json`

Current list of all tasks in the database. Automatically updated when tasks change.

### `config.json`

Rigger configuration settings (read-only).

### `project_context`

Synthesized project context including file tree, architectural patterns, and recent decisions.

## Troubleshooting

### MCP Server Not Starting

1. Verify `rig` is in your PATH:
   ```bash
   which rig
   ```

2. Test the server manually:
   ```bash
   echo '{"jsonrpc":"2.0","method":"list_tasks","params":{},"id":1}' | rig server
   ```

3. Check stderr logs for error messages

### Tasks Not Found

1. Ensure you've run `rig init` in your project directory
2. Verify `.rigger/tasks.db` exists
3. Try creating a test task:
   ```bash
   rig add_task "Test task"
   ```

### IDE Not Recognizing Tools

1. Restart your IDE after configuration changes
2. Check IDE's MCP server logs
3. Verify JSON configuration is valid

## Example Workflow

1. **Create a PRD**: Write a `PRD.md` file describing your project requirements

2. **Parse it**: Use `@rig parse_prd {"prd_file": "PRD.md"}` in your IDE

3. **View tasks**: Use `@rig list_tasks` to see generated tasks

4. **Update status**: As you complete tasks, use `@rig update_task {"task_id": "...", "status": "Completed"}`

5. **Context-aware AI**: Your IDE's AI assistant now has access to your task list as context!

## Advanced Features

### Intelligent Task Decomposition

High-complexity tasks (score >= 7) are automatically decomposed into 3-5 manageable subtasks:

```
@rig parse_prd {"prd_file": "complex_project.md"}
```

Rigger analyzes each task's complexity using:
- Title length and keywords
- Architectural keywords (refactor, migrate, redesign)
- Clarity of ownership and timeline

### Project Context Synthesis

Rigger analyzes your codebase and provides relevant context to the LLM:

```
@rig get_resource {"resource": "project_context"}
```

This includes:
- File tree structure
- Detected architectural patterns
- Recent decisions and changes

## Security Notes

- The MCP server only has access to the `.rigger/` directory and your project files
- File system access is sandboxed (no `..` or absolute paths outside project)
- Database operations are scoped to the current project

## Support

For issues or feature requests:
- GitHub: https://github.com/squillo/rig-task-pipeline
- Documentation: See `docs/` directory in the repository

## Version

Rigger MCP Server v0.1.0 (Phase 4)
