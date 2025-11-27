# Rigger API Reference

Complete API documentation for all Rigger interfaces: CLI, gRPC, MCP, and Rust library APIs.

## Table of Contents

- [CLI API](#cli-api)
- [gRPC API](#grpc-api)
- [MCP API](#mcp-api)
- [Rust Library API](#rust-library-api)

## CLI API

The Rigger CLI (`rig`) provides commands for local task management, orchestration, and server modes.

### Global Options

```
rig [OPTIONS] <COMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information
```

### Commands

#### `rig init`

Initialize the `.rigger` directory and SQLite database in the current project.

**Usage**:
```bash
rig init
```

**What it does**:
- Creates `.rigger/` directory
- Initializes `tasks.db` SQLite database with schema
- Creates initial configuration

**Output**:
```
✅ Initialized Rigger workspace at /path/to/project/.rigger
   Database: /path/to/project/.rigger/tasks.db
```

**Exit codes**:
- `0`: Success
- `1`: Error (e.g., directory already exists, permission denied)

---

#### `rig task`

Manage tasks via CLI.

**Subcommands**:

##### `rig task add`

Create a new task.

**Usage**:
```bash
rig task add [OPTIONS] <TITLE>
```

**Arguments**:
- `<TITLE>`: Task title (required)

**Options**:
- `--assignee <ASSIGNEE>`: Person assigned to task
- `--due-date <DUE_DATE>`: Due date (YYYY-MM-DD format)
- `--prd-id <PRD_ID>`: Source PRD ID
- `--parent-id <PARENT_ID>`: Parent task ID

**Examples**:
```bash
# Simple task
rig task add "Implement OAuth2 authentication"

# Task with assignee and due date
rig task add "Fix login bug" --assignee alice --due-date 2025-12-31

# Subtask linked to parent
rig task add "Write unit tests" --parent-id task-abc-123
```

**Output**:
```
✅ Task created successfully

┌──────────────┬────────────────────────────────────┐
│ ID           │ task-abc-123                       │
│ Title        │ Implement OAuth2 authentication    │
│ Status       │ Todo                               │
│ Assignee     │ alice                              │
│ Due Date     │ 2025-12-31                         │
│ Created      │ 2025-11-23T10:30:00Z               │
└──────────────┴────────────────────────────────────┘
```

---

##### `rig task list`

List tasks with optional filters.

**Usage**:
```bash
rig task list [OPTIONS]
```

**Options**:
- `--status <STATUS>`: Filter by status (`todo`, `in_progress`, `completed`, etc.)
- `--assignee <ASSIGNEE>`: Filter by assignee
- `--limit <LIMIT>`: Max number of tasks to return (default: 50)
- `--offset <OFFSET>`: Skip first N tasks (default: 0)

**Examples**:
```bash
# List all tasks
rig task list

# List tasks assigned to alice
rig task list --assignee alice

# List in-progress tasks
rig task list --status in_progress

# List first 10 completed tasks
rig task list --status completed --limit 10
```

**Output**:
```
┌──────────────┬────────────────────────────┬─────────────┬──────────┐
│ ID           │ Title                      │ Status      │ Assignee │
├──────────────┼────────────────────────────┼─────────────┼──────────┤
│ task-abc-123 │ Implement OAuth2           │ InProgress  │ alice    │
│ task-def-456 │ Fix login bug              │ Todo        │ bob      │
│ task-ghi-789 │ Write API docs             │ Completed   │ alice    │
└──────────────┴────────────────────────────┴─────────────┴──────────┘

Total: 3 tasks
```

---

##### `rig task get`

Get details of a specific task.

**Usage**:
```bash
rig task get <TASK_ID>
```

**Arguments**:
- `<TASK_ID>`: Task ID (required)

**Example**:
```bash
rig task get task-abc-123
```

**Output**:
```
┌──────────────┬────────────────────────────────────┐
│ ID           │ task-abc-123                       │
│ Title        │ Implement OAuth2 authentication    │
│ Status       │ InProgress                         │
│ Assignee     │ alice                              │
│ Due Date     │ 2025-12-31                         │
│ Created      │ 2025-11-23T10:30:00Z               │
│ Updated      │ 2025-11-23T11:00:00Z               │
│ Parent Task  │ None                               │
│ Source PRD   │ None                               │
└──────────────┴────────────────────────────────────┘

Enhancements: 1
Comprehension Tests: 0
Subtasks: 0
```

---

##### `rig task update`

Update an existing task.

**Usage**:
```bash
rig task update <TASK_ID> [OPTIONS]
```

**Arguments**:
- `<TASK_ID>`: Task ID (required)

**Options**:
- `--status <STATUS>`: New status
- `--assignee <ASSIGNEE>`: New assignee
- `--due-date <DUE_DATE>`: New due date

**Examples**:
```bash
# Update status
rig task update task-abc-123 --status completed

# Reassign task
rig task update task-abc-123 --assignee bob

# Update multiple fields
rig task update task-abc-123 --status in_progress --assignee charlie --due-date 2025-12-15
```

**Output**:
```
✅ Task updated successfully

┌──────────────┬────────────────────────────────────┐
│ ID           │ task-abc-123                       │
│ Status       │ Completed                          │
│ Updated      │ 2025-11-23T12:00:00Z               │
└──────────────┴────────────────────────────────────┘
```

---

##### `rig task delete`

Delete (archive) a task.

**Usage**:
```bash
rig task delete <TASK_ID>
```

**Arguments**:
- `<TASK_ID>`: Task ID (required)

**Example**:
```bash
rig task delete task-abc-123
```

**Output**:
```
✅ Task task-abc-123 deleted successfully
```

**Note**: Tasks are soft-deleted (status set to `Archived`), not permanently removed.

---

#### `rig prd`

Work with Product Requirement Documents (PRDs).

**Subcommands**:

##### `rig prd parse`

Parse a PRD markdown file.

**Usage**:
```bash
rig prd parse <PRD_FILE>
```

**Arguments**:
- `<PRD_FILE>`: Path to PRD markdown file (required)

**Example**:
```bash
rig prd parse docs/feature-spec.md
```

**Output**:
```
✅ PRD parsed successfully

Title: OAuth2 Authentication Feature
Objectives: 3
Tech Stack: 5
Constraints: 2

Objectives:
  - Implement OAuth2 provider integration
  - Support multiple identity providers
  - Ensure secure token storage

Tech Stack:
  - Rust
  - OAuth2 crate
  - JWT
  - PostgreSQL
  - Redis (session storage)

Constraints:
  - Must support PKCE flow
  - Tokens must expire after 1 hour
```

---

##### `rig prd generate-tasks`

Generate tasks from a PRD using LLM.

**Usage**:
```bash
rig prd generate-tasks [OPTIONS] <PRD_FILE>
```

**Arguments**:
- `<PRD_FILE>`: Path to PRD markdown file (required)

**Options**:
- `--model <MODEL>`: LLM model to use (default: `llama3.1`)
- `--save`: Save generated tasks to database

**Example**:
```bash
rig prd generate-tasks docs/feature-spec.md --model llama3.1 --save
```

**Output**:
```
🤖 Generating tasks from PRD using llama3.1...

✅ Generated 5 tasks:

1. task-001: "Set up OAuth2 provider configuration"
   Assignee: None
   Status: Todo

2. task-002: "Implement authorization code flow"
   Assignee: None
   Status: Todo

3. task-003: "Add JWT token generation and validation"
   Assignee: None
   Status: Todo

4. task-004: "Integrate Redis for session storage"
   Assignee: None
   Status: Todo

5. task-005: "Write integration tests for OAuth flow"
   Assignee: None
   Status: Todo

💾 Saved 5 tasks to database
```

---

#### `rig orchestrate`

Run AI orchestration on tasks.

**Subcommands**:

##### `rig orchestrate run`

Run orchestration flow on a task.

**Usage**:
```bash
rig orchestrate run [OPTIONS] <TASK_ID>
```

**Arguments**:
- `<TASK_ID>`: Task ID (required)

**Options**:
- `--model <MODEL>`: LLM model (default: `llama3.1`)
- `--test-type <TEST_TYPE>`: Test type (`short_answer`, `multiple_choice`) (default: `short_answer`)

**Example**:
```bash
rig orchestrate run task-abc-123 --model llama3.1 --test-type short_answer
```

**Output**:
```
🧠 Orchestrating task: task-abc-123
   Model: llama3.1
   Test Type: short_answer

📍 Routing decision: complex

🔄 Executing orchestration flow...

  ✅ Step 1/3: Enhanced task description
     Routing: complex
     LLM reasoning: This task requires architectural design...

  ✅ Step 2/3: Generated comprehension test
     Question: What is the purpose of PKCE in OAuth2?
     Difficulty: Medium

  ✅ Step 3/3: Decomposed into 3 subtasks
     - Configure OAuth2 client credentials
     - Implement token endpoint
     - Add token refresh logic

✅ Orchestration complete!

Updated task: task-abc-123
  Status: OrchestrationComplete
  Enhancements: 1
  Tests: 1
  Subtasks: 3
```

---

#### `rig grpc`

Start gRPC server mode.

**Usage**:
```bash
rig grpc [OPTIONS]
```

**Options**:
- `--port <PORT>`: Server port (default: `50051`)

**Example**:
```bash
rig grpc --port 50051
```

**Output**:
```
🚀 Rigger gRPC Server starting...
   Protocol: gRPC over HTTP/2
   Address: [::1]:50051
   Database: /path/to/project/.rigger/tasks.db
   Broadcast: Enabled (1000 event buffer)

Press Ctrl+C to stop
```

**What it does**:
- Starts gRPC server on specified port
- Listens for gRPC requests (AddTask, ListTasks, etc.)
- Broadcasts task events to subscribers
- Runs until interrupted

---

#### `rig server`

Start MCP server mode (stdio JSON-RPC 2.0).

**Usage**:
```bash
rig server
```

**Example**:
```bash
rig server
```

**What it does**:
- Starts MCP server in stdio mode
- Reads JSON-RPC requests from stdin
- Writes JSON-RPC responses to stdout
- Used by IDEs (Claude Code, Cline, etc.)

**Example interaction**:
```json
// Request (stdin)
{"jsonrpc":"2.0","id":1,"method":"tools/list"}

// Response (stdout)
{"jsonrpc":"2.0","id":1,"result":{"tools":[...]}}
```

---

## gRPC API

The Rigger gRPC API provides high-performance, type-safe task management with event broadcasting.

### Connection

**Endpoint**: `[::1]:50051` (default)

**Protocol**: gRPC over HTTP/2

**Encoding**: Protocol Buffers (binary)

### Service Definition

```protobuf
service RiggerService {
  // Task Management
  rpc ListTasks(ListTasksRequest) returns (ListTasksResponse);
  rpc AddTask(AddTaskRequest) returns (AddTaskResponse);
  rpc UpdateTask(UpdateTaskRequest) returns (UpdateTaskResponse);
  rpc GetTask(GetTaskRequest) returns (GetTaskResponse);
  rpc DeleteTask(DeleteTaskRequest) returns (DeleteTaskResponse);

  // PRD Operations
  rpc ParsePRD(ParsePRDRequest) returns (ParsePRDResponse);
  rpc GenerateTasksFromPRD(GenerateTasksFromPRDRequest) returns (GenerateTasksFromPRDResponse);

  // Orchestration
  rpc OrchestrateTask(OrchestrateTaskRequest) returns (OrchestrateTaskResponse);

  // Event Streaming
  rpc SubscribeToTaskEvents(SubscribeToTaskEventsRequest) returns (stream TaskEvent);
  rpc TaskEventStream(stream TaskEventStreamRequest) returns (stream TaskEvent);
}
```

### RPCs

#### ListTasks

List tasks with optional filters.

**Request**:
```protobuf
message ListTasksRequest {
  optional int32 status = 1;           // TaskStatus enum value
  optional string assignee = 2;        // Filter by assignee
  optional uint32 limit = 3;           // Max results (default: 50)
  optional uint32 offset = 4;          // Skip first N results
}
```

**Response**:
```protobuf
message ListTasksResponse {
  repeated Task tasks = 1;             // List of tasks
  uint64 total_count = 2;              // Total matching tasks
}
```

**Example (grpcurl)**:
```bash
grpcurl -plaintext -d '{
  "status": 2,
  "assignee": "alice",
  "limit": 10,
  "offset": 0
}' localhost:50051 rigger.v1.RiggerService/ListTasks
```

**Response**:
```json
{
  "tasks": [
    {
      "id": "task-abc-123",
      "title": "Implement OAuth2",
      "status": 2,
      "assignee": "alice",
      "createdAt": "2025-11-23T10:30:00Z"
    }
  ],
  "totalCount": "1"
}
```

---

#### AddTask

Create a new task.

**Request**:
```protobuf
message AddTaskRequest {
  string title = 1;                    // Required
  optional string assignee = 2;
  optional string due_date = 3;        // YYYY-MM-DD format
  optional string source_prd_id = 4;
  optional string parent_task_id = 5;
}
```

**Response**:
```protobuf
message AddTaskResponse {
  Task task = 1;                       // Created task
}
```

**Example**:
```bash
grpcurl -plaintext -d '{
  "title": "Implement OAuth2 authentication",
  "assignee": "alice",
  "dueDate": "2025-12-31"
}' localhost:50051 rigger.v1.RiggerService/AddTask
```

**Response**:
```json
{
  "task": {
    "id": "task-abc-123",
    "title": "Implement OAuth2 authentication",
    "status": 1,
    "assignee": "alice",
    "dueDate": "2025-12-31",
    "createdAt": "2025-11-23T10:30:00Z"
  }
}
```

**Broadcast**: `TaskEvent` with type `CREATED` sent to all subscribers.

---

#### UpdateTask

Update an existing task.

**Request**:
```protobuf
message UpdateTaskRequest {
  string task_id = 1;                  // Required
  optional int32 status = 2;           // New status
  optional string assignee = 3;        // New assignee
  optional string due_date = 4;        // New due date
}
```

**Response**:
```protobuf
message UpdateTaskResponse {
  Task task = 1;                       // Updated task
}
```

**Example**:
```bash
grpcurl -plaintext -d '{
  "taskId": "task-abc-123",
  "status": 2
}' localhost:50051 rigger.v1.RiggerService/UpdateTask
```

**Broadcast**: `TaskEvent` with type `UPDATED` sent to all subscribers.

---

#### GetTask

Retrieve a specific task by ID.

**Request**:
```protobuf
message GetTaskRequest {
  string task_id = 1;                  // Required
}
```

**Response**:
```protobuf
message GetTaskResponse {
  Task task = 1;                       // Task details
}
```

**Example**:
```bash
grpcurl -plaintext -d '{
  "taskId": "task-abc-123"
}' localhost:50051 rigger.v1.RiggerService/GetTask
```

---

#### DeleteTask

Delete (archive) a task.

**Request**:
```protobuf
message DeleteTaskRequest {
  string task_id = 1;                  // Required
}
```

**Response**:
```protobuf
message DeleteTaskResponse {
  bool success = 1;                    // True if deleted
}
```

**Example**:
```bash
grpcurl -plaintext -d '{
  "taskId": "task-abc-123"
}' localhost:50051 rigger.v1.RiggerService/DeleteTask
```

**Broadcast**: `TaskEvent` with type `DELETED` sent to all subscribers.

---

#### ParsePRD

Parse a PRD markdown file.

**Request**:
```protobuf
message ParsePRDRequest {
  string prd_file_path = 1;            // Required
}
```

**Response**:
```protobuf
message ParsePRDResponse {
  string prd_id = 1;
  string prd_title = 2;
  repeated string objectives = 3;
  repeated string tech_stack = 4;
  repeated string constraints = 5;
}
```

**Example**:
```bash
grpcurl -plaintext -d '{
  "prdFilePath": "./docs/feature-spec.md"
}' localhost:50051 rigger.v1.RiggerService/ParsePRD
```

---

#### GenerateTasksFromPRD

Generate tasks from PRD using LLM.

**Request**:
```protobuf
message GenerateTasksFromPRDRequest {
  string prd_id = 1;                   // PRD file path
  string model = 2;                    // LLM model (e.g., "llama3.1")
}
```

**Response**:
```protobuf
message GenerateTasksFromPRDResponse {
  repeated Task tasks = 1;             // Generated tasks
  uint32 tasks_generated = 2;          // Count
}
```

**Example**:
```bash
grpcurl -plaintext -d '{
  "prdId": "./docs/feature-spec.md",
  "model": "llama3.1"
}' localhost:50051 rigger.v1.RiggerService/GenerateTasksFromPRD
```

**Broadcast**: `TaskEvent` with type `CREATED` for each generated task.

---

#### OrchestrateTask

Run orchestration flow on a task.

**Request**:
```protobuf
message OrchestrateTaskRequest {
  string task_id = 1;                  // Required
  string model = 2;                    // LLM model
  string test_type = 3;                // Test type ("short_answer", "multiple_choice")
}
```

**Response**:
```protobuf
message OrchestrateTaskResponse {
  Task task = 1;                       // Orchestrated task
  string routing_decision = 2;         // "simple", "moderate", "complex"
  repeated Enhancement enhancements = 3;
  repeated ComprehensionTest tests = 4;
  repeated Task subtasks = 5;
}
```

**Example**:
```bash
grpcurl -plaintext -d '{
  "taskId": "task-abc-123",
  "model": "llama3.1",
  "testType": "short_answer"
}' localhost:50051 rigger.v1.RiggerService/OrchestrateTask
```

**Broadcast**: `TaskEvent` with type `ORCHESTRATED` sent to all subscribers.

---

#### SubscribeToTaskEvents

Subscribe to a stream of task events (for sidecars).

**Request**:
```protobuf
message SubscribeToTaskEventsRequest {
  repeated int32 event_types = 1;      // Filter by event types (empty = all)
  optional string assignee_filter = 2; // Filter by assignee
}
```

**Response Stream**:
```protobuf
message TaskEvent {
  TaskEventType event_type = 1;
  Task task = 2;
  string timestamp = 3;
  optional string actor = 4;           // Who triggered the event
  map<string, string> metadata = 5;    // Additional context
}
```

**Example**:
```bash
grpcurl -plaintext -d '{
  "eventTypes": [1, 2, 3],
  "assigneeFilter": "alice"
}' localhost:50051 rigger.v1.RiggerService/SubscribeToTaskEvents
```

**Output** (streaming):
```json
{
  "eventType": "TASK_EVENT_TYPE_CREATED",
  "task": {
    "id": "task-abc-123",
    "title": "New task",
    ...
  },
  "timestamp": "2025-11-23T10:30:00Z",
  "actor": "alice"
}
```

The stream continues until the client disconnects.

---

### Enums

#### TaskStatus

```protobuf
enum TaskStatus {
  TASK_STATUS_UNSPECIFIED = 0;
  TASK_STATUS_TODO = 1;
  TASK_STATUS_IN_PROGRESS = 2;
  TASK_STATUS_PENDING_ENHANCEMENT = 3;
  TASK_STATUS_PENDING_COMPREHENSION_TEST = 4;
  TASK_STATUS_PENDING_FOLLOW_ON = 5;
  TASK_STATUS_PENDING_DECOMPOSITION = 6;
  TASK_STATUS_DECOMPOSED = 7;
  TASK_STATUS_ORCHESTRATION_COMPLETE = 8;
  TASK_STATUS_COMPLETED = 9;
  TASK_STATUS_ARCHIVED = 10;
}
```

#### TaskEventType

```protobuf
enum TaskEventType {
  TASK_EVENT_TYPE_UNSPECIFIED = 0;
  TASK_EVENT_TYPE_CREATED = 1;
  TASK_EVENT_TYPE_UPDATED = 2;
  TASK_EVENT_TYPE_DELETED = 3;
  TASK_EVENT_TYPE_STATUS_CHANGED = 4;
  TASK_EVENT_TYPE_ASSIGNED = 5;
  TASK_EVENT_TYPE_DECOMPOSED = 6;
  TASK_EVENT_TYPE_ORCHESTRATED = 7;
}
```

---

## MCP API

The Model Context Protocol (MCP) server exposes task management tools for IDE integration.

### Connection

**Transport**: stdio (standard input/output)

**Protocol**: JSON-RPC 2.0

### Available Tools

#### list_tasks

List tasks with filters.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "status": { "type": "string", "enum": ["todo", "in_progress", "completed", ...] },
    "assignee": { "type": "string" },
    "limit": { "type": "integer", "default": 50 },
    "offset": { "type": "integer", "default": 0 }
  }
}
```

**Example Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "list_tasks",
    "arguments": {
      "status": "in_progress",
      "assignee": "alice"
    }
  }
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [{
      "type": "text",
      "text": "Found 2 tasks:\n\n1. task-abc-123: Implement OAuth2\n   Status: InProgress\n   Assignee: alice\n\n2. task-def-456: Fix login bug\n   Status: InProgress\n   Assignee: alice"
    }]
  }
}
```

---

#### add_task

Create a new task.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "title": { "type": "string" },
    "assignee": { "type": "string" },
    "due_date": { "type": "string", "format": "date" },
    "prd_id": { "type": "string" },
    "parent_id": { "type": "string" }
  },
  "required": ["title"]
}
```

**Example Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "add_task",
    "arguments": {
      "title": "Implement OAuth2 authentication",
      "assignee": "alice",
      "due_date": "2025-12-31"
    }
  }
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "content": [{
      "type": "text",
      "text": "Task created successfully:\n\nID: task-abc-123\nTitle: Implement OAuth2 authentication\nStatus: Todo\nAssignee: alice\nDue Date: 2025-12-31"
    }]
  }
}
```

---

#### get_task

Get details of a specific task.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "task_id": { "type": "string" }
  },
  "required": ["task_id"]
}
```

---

#### update_task

Update an existing task.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "task_id": { "type": "string" },
    "status": { "type": "string" },
    "assignee": { "type": "string" },
    "due_date": { "type": "string", "format": "date" }
  },
  "required": ["task_id"]
}
```

---

#### orchestrate_task

Run orchestration flow on a task.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "task_id": { "type": "string" },
    "model": { "type": "string", "default": "llama3.1" },
    "test_type": { "type": "string", "enum": ["short_answer", "multiple_choice"], "default": "short_answer" }
  },
  "required": ["task_id"]
}
```

**Example Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "orchestrate_task",
    "arguments": {
      "task_id": "task-abc-123",
      "model": "llama3.1",
      "test_type": "short_answer"
    }
  }
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [{
      "type": "text",
      "text": "Orchestration complete:\n\nRouting: complex\nEnhancements: 1\nTests: 1\nSubtasks: 3\n\nTask status: OrchestrationComplete"
    }]
  }
}
```

---

#### parse_prd

Parse a PRD markdown file.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "prd_file_path": { "type": "string" }
  },
  "required": ["prd_file_path"]
}
```

---

## Rust Library API

The Rust library provides programmatic access to task management and orchestration.

### task_manager Crate

#### Domain

##### Task

**Location**: `task_manager::domain::task::Task`

```rust
pub struct Task {
    pub id: String,
    pub title: String,
    pub status: TaskStatus,
    pub assignee: Option<String>,
    pub due_date: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub parent_task_id: Option<String>,
    pub source_prd_id: Option<String>,
    pub enhancements: Option<Vec<Enhancement>>,
    pub comprehension_tests: Option<Vec<ComprehensionTest>>,
    pub subtasks: Option<Vec<Task>>,
}

impl Task {
    pub fn new(
        title: String,
        assignee: Option<String>,
        due_date: Option<String>,
        source_prd_id: Option<String>,
        parent_task_id: Option<String>,
    ) -> Self;
}
```

**Example**:
```rust
use task_manager::domain::task::Task;

let task = Task::new(
    "Implement OAuth2".to_string(),
    Some("alice".to_string()),
    Some("2025-12-31".to_string()),
    None,
    None,
);
```

---

##### TaskStatus

**Location**: `task_manager::domain::task_status::TaskStatus`

```rust
pub enum TaskStatus {
    Todo,
    InProgress,
    PendingEnhancement,
    PendingComprehensionTest,
    PendingFollowOn,
    PendingDecomposition,
    Decomposed,
    OrchestrationComplete,
    Completed,
    Archived,
}
```

---

#### Ports

##### TaskRepositoryPort

**Location**: `task_manager::ports::task_repository_port::TaskRepositoryPort`

```rust
pub trait TaskRepositoryPort: Repository<Task, TaskFilter, TaskSortKey> {
    fn save(&self, entity: Task) -> HexResult<()>;
    fn find_one(&self, filter: &TaskFilter) -> HexResult<Option<Task>>;
    fn find(&self, filter: &TaskFilter, opts: FindOptions<TaskSortKey>) -> HexResult<Vec<Task>>;
    fn delete(&self, filter: &TaskFilter) -> HexResult<()>;
}
```

---

#### Adapters

##### SqliteTaskAdapter

**Location**: `task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter`

```rust
pub struct SqliteTaskAdapter {
    db_path: PathBuf,
}

impl SqliteTaskAdapter {
    pub fn new(db_path: impl Into<PathBuf>) -> Self;

    // Async methods for gRPC server
    pub async fn save_async(&self, entity: Task) -> HexResult<()>;
    pub async fn find_one_async(&self, filter: &TaskFilter) -> HexResult<Option<Task>>;
    pub async fn find_async(&self, filter: &TaskFilter, opts: FindOptions<TaskSortKey>) -> HexResult<Vec<Task>>;
}
```

**Example**:
```rust
use task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter;
use task_manager::domain::task::Task;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = SqliteTaskAdapter::new(".rigger/tasks.db");
    let task = Task::new("Test task".to_string(), None, None, None, None);

    adapter.save_async(task).await?;
    Ok(())
}
```

---

#### Use Cases

##### create_task

**Location**: `task_manager::use_cases::manage_task::create_task`

```rust
pub fn create_task(
    repo: &dyn TaskRepositoryPort,
    title: String,
    assignee: Option<String>,
    due_date: Option<String>,
    source_prd_id: Option<String>,
    parent_task_id: Option<String>,
) -> HexResult<Task>;
```

**Example**:
```rust
use task_manager::use_cases::manage_task::create_task;
use task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter;
use task_manager::ports::task_repository_port::TaskRepositoryPort;

let adapter = SqliteTaskAdapter::new(".rigger/tasks.db");
let task = create_task(
    &adapter,
    "Implement feature X".to_string(),
    Some("alice".to_string()),
    Some("2025-12-31".to_string()),
    None,
    None,
)?;
```

---

##### list_tasks

**Location**: `task_manager::use_cases::manage_task::list_tasks`

```rust
pub fn list_tasks(
    repo: &dyn TaskRepositoryPort,
    status: Option<TaskStatus>,
    assignee: Option<String>,
    limit: Option<u64>,
    offset: Option<u64>,
) -> HexResult<(Vec<Task>, u64)>;
```

---

##### update_task_status

**Location**: `task_manager::use_cases::manage_task::update_task_status`

```rust
pub fn update_task_status(
    repo: &dyn TaskRepositoryPort,
    task_id: &str,
    new_status: TaskStatus,
) -> HexResult<Task>;
```

---

### task_orchestrator Crate

#### Ports

##### EnhancerPort

**Location**: `task_orchestrator::ports::enhancer_port::EnhancerPort`

```rust
#[async_trait]
pub trait EnhancerPort {
    async fn enhance_task(&self, task: &Task) -> HexResult<Enhancement>;
}
```

---

##### TestGeneratorPort

**Location**: `task_orchestrator::ports::test_generator_port::TestGeneratorPort`

```rust
#[async_trait]
pub trait TestGeneratorPort {
    async fn generate_test(&self, task: &Task, test_type: &str) -> HexResult<ComprehensionTest>;
}
```

---

#### Adapters

##### RigEnhancerAdapter

**Location**: `task_orchestrator::adapters::rig_enhancer_adapter::RigEnhancerAdapter`

```rust
pub struct RigEnhancerAdapter {
    model: String,
}

impl RigEnhancerAdapter {
    pub fn new(model: String) -> Self;
}

#[async_trait]
impl EnhancerPort for RigEnhancerAdapter {
    async fn enhance_task(&self, task: &Task) -> HexResult<Enhancement>;
}
```

**Example**:
```rust
use task_orchestrator::adapters::rig_enhancer_adapter::RigEnhancerAdapter;
use task_orchestrator::ports::enhancer_port::EnhancerPort;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let enhancer = RigEnhancerAdapter::new("llama3.1".to_string());
    let enhancement = enhancer.enhance_task(&task).await?;
    println!("Enhanced: {}", enhancement.enhanced_description);
    Ok(())
}
```

---

#### Use Cases

##### run_task_with_flow

**Location**: `task_orchestrator::use_cases::run_task_with_flow::run_task_with_flow`

```rust
pub async fn run_task_with_flow(
    model: &str,
    test_type: &str,
    task: Task,
) -> HexResult<Task>;
```

**Example**:
```rust
use task_orchestrator::use_cases::run_task_with_flow::run_task_with_flow;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let orchestrated_task = run_task_with_flow(
        "llama3.1",
        "short_answer",
        task,
    ).await?;

    println!("Orchestration complete!");
    println!("Status: {:?}", orchestrated_task.status);
    println!("Enhancements: {}", orchestrated_task.enhancements.as_ref().map(|e| e.len()).unwrap_or(0));
    println!("Tests: {}", orchestrated_task.comprehension_tests.as_ref().map(|t| t.len()).unwrap_or(0));
    println!("Subtasks: {}", orchestrated_task.subtasks.as_ref().map(|s| s.len()).unwrap_or(0));

    Ok(())
}
```

---

## Error Handling

### Rust Errors

All library functions return `HexResult<T>`, defined as:

```rust
pub type HexResult<T> = Result<T, HexError>;

pub enum HexError {
    NotFound(String),
    AlreadyExists(String),
    ValidationError(String),
    DatabaseError(String),
    Custom(String),
}
```

**Example**:
```rust
match create_task(&adapter, "Task".to_string(), None, None, None, None) {
    Ok(task) => println!("Created: {}", task.id),
    Err(HexError::DatabaseError(msg)) => eprintln!("Database error: {}", msg),
    Err(e) => eprintln!("Error: {:?}", e),
}
```

---

### gRPC Errors

gRPC errors use `tonic::Status`:

- `INVALID_ARGUMENT`: Invalid request parameters
- `NOT_FOUND`: Task not found
- `INTERNAL`: Server error (database, LLM, etc.)
- `UNAVAILABLE`: Service unavailable

**Example**:
```rust
Err(Status::invalid_argument("Task ID cannot be empty"))
Err(Status::not_found(format!("Task {} not found", task_id)))
Err(Status::internal("LLM call failed"))
```

---

### MCP Errors

MCP errors use JSON-RPC 2.0 error codes:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32600,
    "message": "Invalid request",
    "data": "Missing required field: task_id"
  }
}
```

**Error codes**:
- `-32700`: Parse error
- `-32600`: Invalid request
- `-32601`: Method not found
- `-32602`: Invalid params
- `-32603`: Internal error

---

## Rate Limits

No rate limits are currently enforced. For production use, consider implementing rate limiting at the load balancer or API gateway level.

## Versioning

**Current version**: v0.1.0

**API compatibility**: No breaking changes are planned before v1.0.0. Minor versions may add new fields/methods but will remain backward compatible.

**Protobuf schema**: The protobuf schema uses numbered fields that maintain backward compatibility when new fields are added.

## Support

For API questions or issues:
- GitHub Issues: https://github.com/anthropics/rig-task-pipeline/issues
- Documentation: See `docs/` directory

---

**Last updated**: 2025-11-23
