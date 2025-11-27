# Rigger MCP Server Setup Guide

This guide explains how to configure Cursor AI IDE and Windsurf to use the Rigger MCP (Model Context Protocol) server for intelligent task management integration.

## Prerequisites

1. **Install Rigger CLI**:
   ```bash
   cargo install --path rigger_cli
   ```

2. **Initialize Rigger in your project**:
   ```bash
   cd /path/to/your/project
   rig init
   ```

3. **Verify installation**:
   ```bash
   rig --version
   ```

## MCP Server Overview

The Rigger MCP server provides the following capabilities to your IDE:

### Tools
- `list_tasks`: List tasks with optional filters (status, assignee)
- `add_task`: Create a new task from title/assignee/due_date
- `update_task`: Update task status or assignee
- `parse_prd`: Parse PRD markdown file and extract objectives

### Resources
- `tasks.json`: Current task list from database
- `config.json`: Rigger configuration settings

## Cursor AI IDE Setup

### 1. Locate Cursor Configuration Directory

Cursor MCP configuration is typically stored in:
- **macOS/Linux**: `~/.cursor/mcp.json`
- **Windows**: `%APPDATA%\Cursor\mcp.json`

### 2. Add Rigger MCP Server Configuration

Add the following to your `mcp.json`:

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

### 3. Restart Cursor

After saving the configuration, restart Cursor AI IDE to activate the MCP server.

### 4. Verify Integration

In Cursor, you should now be able to use `@rig` to access Rigger tools:

- `@rig list tasks`
- `@rig add task "Implement OAuth2 authentication" --assignee Alice --due 2025-12-31`
- `@rig update task <task-id> --status InProgress`
- `@rig parse prd ./docs/PRD.md`

## Windsurf IDE Setup

### 1. Locate Windsurf Configuration Directory

Windsurf MCP configuration is typically stored in:
- **macOS/Linux**: `~/.windsurf/mcp.json`
- **Windows**: `%APPDATA%\Windsurf\mcp.json`

### 2. Add Rigger MCP Server Configuration

Add the following to your `mcp.json`:

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

### 3. Restart Windsurf

After saving the configuration, restart Windsurf IDE to activate the MCP server.

### 4. Verify Integration

In Windsurf, access Rigger tools through the MCP interface:

- Use the tool palette to access `rigger.list_tasks`
- Use the tool palette to access `rigger.add_task`
- Use the tool palette to access `rigger.update_task`
- Use the tool palette to access `rigger.parse_prd`

## MCP Server Protocol Details

### Transport
- **Protocol**: JSON-RPC 2.0
- **Transport**: stdio (stdin for requests, stdout for responses)
- **Logging**: stderr (to avoid polluting JSON-RPC stream)

### Tool Schemas

#### list_tasks
```json
{
  "status": "Todo" | "InProgress" | "Completed" | "Archived" | ...,
  "assignee": "string" (optional)
}
```

**Returns**:
```json
{
  "tasks": [
    {
      "id": "string",
      "title": "string",
      "status": "string",
      "assignee": "string | null",
      "due_date": "string | null",
      "created_at": "ISO 8601",
      "updated_at": "ISO 8601"
    }
  ],
  "count": number
}
```

#### add_task
```json
{
  "title": "string",
  "assignee": "string" (optional),
  "due_date": "YYYY-MM-DD" (optional)
}
```

**Returns**:
```json
{
  "task_id": "string",
  "title": "string",
  "status": "string"
}
```

#### update_task
```json
{
  "task_id": "string",
  "status": "Todo" | "InProgress" | "Completed" | ... (optional),
  "assignee": "string" (optional)
}
```

**Returns**:
```json
{
  "success": true,
  "task_id": "string",
  "status": "string"
}
```

#### parse_prd
```json
{
  "prd_file_path": "string"
}
```

**Returns**:
```json
{
  "prd_title": "string",
  "objectives_count": number,
  "tech_stack_count": number,
  "constraints_count": number,
  "tasks_generated": number
}
```

## Troubleshooting

### MCP Server Not Starting

1. **Check Rigger installation**:
   ```bash
   which rig
   ```

2. **Test MCP server manually**:
   ```bash
   cd /path/to/your/project
   rig server
   ```

   Send a test JSON-RPC request via stdin:
   ```json
   {"jsonrpc":"2.0","method":"list_tasks","params":{},"id":1}
   ```

3. **Check IDE logs**:
   - Cursor: View → Output → MCP
   - Windsurf: View → Output → MCP

### Database Errors

If you see "Database error" messages:

1. **Ensure .rigger directory exists**:
   ```bash
   ls -la .rigger/
   ```

2. **Reinitialize if needed**:
   ```bash
   rig init
   ```

### Permission Issues

On Unix systems, ensure `rig` binary is executable:
```bash
chmod +x $(which rig)
```

## Advanced Configuration

### Custom Database Path

Set `RIGGER_DB_PATH` environment variable in MCP config:

```json
{
  "mcpServers": {
    "rigger": {
      "command": "rig",
      "args": ["server"],
      "cwd": "${workspaceFolder}",
      "env": {
        "RIGGER_DB_PATH": "/custom/path/to/tasks.db"
      }
    }
  }
}
```

### Multiple Projects

Configure different MCP servers for different workspaces:

```json
{
  "mcpServers": {
    "rigger-project-a": {
      "command": "rig",
      "args": ["server"],
      "cwd": "/path/to/project-a"
    },
    "rigger-project-b": {
      "command": "rig",
      "args": ["server"],
      "cwd": "/path/to/project-b"
    }
  }
}
```

## Support

For issues and questions:
- GitHub Issues: https://github.com/your-org/rigger/issues
- Documentation: https://github.com/your-org/rigger/docs

## Revision History

- 2025-11-23: Initial MCP setup guide for Phase 4 Sprint 8
