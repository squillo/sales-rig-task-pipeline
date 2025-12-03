//! Implementation of the 'rig server' command (MCP server mode).
//!
//! Starts the Model Context Protocol server for IDE integration with Cursor/Windsurf.
//! The server communicates via JSON-RPC 2.0 over stdio (stdin for requests, stdout for responses).
//!
//! # Protocol
//!
//! - **Transport**: stdio (stdin/stdout)
//! - **Format**: JSON-RPC 2.0
//! - **Logging**: stderr (to avoid polluting JSON-RPC stream)
//!
//! # Supported Tools
//!
//! - `list_tasks`: List tasks with optional filters
//! - `add_task`: Create a new task
//! - `update_task`: Update task status/priority
//! - `parse_prd`: Parse PRD file and generate tasks
//!
//! # Supported Resources
//!
//! - `tasks.json`: Current task list from database
//! - `config.json`: Configuration settings
//!
//! Revision History
//! - 2025-11-23T18:30:00Z @AI: Implement MCP server for Phase 4 Sprint 8.
//! - 2025-11-22T16:40:00Z @AI: Placeholder server command for Sprint 0.2.

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

/// JSON-RPC 2.0 request structure.
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    #[serde(default)]
    params: serde_json::Value,
    id: serde_json::Value,
}

/// JSON-RPC 2.0 response structure.
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: std::option::Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: std::option::Option<JsonRpcError>,
    id: serde_json::Value,
}

/// JSON-RPC 2.0 error structure.
#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: std::option::Option<serde_json::Value>,
}

impl JsonRpcResponse {
    fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
        JsonRpcResponse {
            jsonrpc: String::from("2.0"),
            result: Some(result),
            error: None,
            id,
        }
    }

    fn error(id: serde_json::Value, code: i32, message: String) -> Self {
        JsonRpcResponse {
            jsonrpc: String::from("2.0"),
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data: None,
            }),
            id,
        }
    }
}

/// Executes the 'rig server' command.
///
/// Starts MCP server mode, listening on stdin for JSON-RPC requests and
/// responding on stdout. All logs go to stderr to avoid polluting the
/// JSON-RPC stream.
///
/// # Errors
///
/// Returns an error if server initialization fails or I/O errors occur.
pub async fn execute() -> anyhow::Result<()> {
    eprintln!("🚀 Rigger MCP Server starting...");
    eprintln!("   Protocol: JSON-RPC 2.0 over stdio");
    eprintln!("   Listening on stdin for requests");
    eprintln!("   Sending responses to stdout");
    eprintln!("   Logging to stderr");
    eprintln!();

    let stdin = tokio::io::stdin();
    let mut reader = tokio::io::BufReader::new(stdin);
    let mut stdout = tokio::io::stdout();

    let mut line = String::new();

    loop {
        line.clear();

        // Read one line from stdin (each JSON-RPC message is newline-delimited)
        match reader.read_line(&mut line).await {
            Ok(0) => {
                // EOF - client closed connection
                eprintln!("📡 Client disconnected (EOF)");
                break;
            }
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                eprintln!("📨 Received request: {}", trimmed);

                // Parse JSON-RPC request
                let response = match serde_json::from_str::<JsonRpcRequest>(trimmed) {
                    Ok(request) => handle_request(request).await,
                    Err(e) => {
                        eprintln!("❌ Parse error: {}", e);
                        JsonRpcResponse::error(
                            serde_json::Value::Null,
                            -32700,
                            format!("Parse error: {}", e),
                        )
                    }
                };

                // Send JSON-RPC response
                let response_json = serde_json::to_string(&response)?;
                stdout.write_all(response_json.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;

                eprintln!("📤 Sent response: {}", response_json);
            }
            Err(e) => {
                eprintln!("❌ I/O error reading stdin: {}", e);
                return Err(e.into());
            }
        }
    }

    eprintln!("✅ MCP server shut down");
    Ok(())
}

/// Handles a JSON-RPC request and routes it to the appropriate handler.
async fn handle_request(request: JsonRpcRequest) -> JsonRpcResponse {
    eprintln!("🔧 Handling method: {}", request.method);

    match request.method.as_str() {
        "list_tasks" => handle_list_tasks(request.id, request.params).await,
        "add_task" => handle_add_task(request.id, request.params).await,
        "update_task" => handle_update_task(request.id, request.params).await,
        "parse_prd" => handle_parse_prd(request.id, request.params).await,
        "get_resource" => handle_get_resource(request.id, request.params).await,
        _ => JsonRpcResponse::error(
            request.id,
            -32601,
            format!("Method not found: {}", request.method),
        ),
    }
}

/// Handles the 'list_tasks' tool.
async fn handle_list_tasks(
    id: serde_json::Value,
    params: serde_json::Value,
) -> JsonRpcResponse {
    #[derive(Deserialize)]
    struct ListTasksParams {
        #[serde(default)]
        status: std::option::Option<String>,
        #[serde(default)]
        assignee: std::option::Option<String>,
    }

    let params: ListTasksParams = match serde_json::from_value(params) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(id, -32602, format!("Invalid params: {}", e));
        }
    };

    // Get database path from .rigger directory
    let db_path = match get_rigger_db_path() {
        Ok(path) => path,
        Err(e) => {
            return JsonRpcResponse::error(id, -32603, format!("Database error: {}", e));
        }
    };

    eprintln!("   Status filter: {:?}", params.status);
    eprintln!("   Assignee filter: {:?}", params.assignee);

    // Connect to database
    let adapter = match task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::connect_and_init(
        &std::format!("sqlite:{}", db_path.display())
    ).await {
        Ok(a) => a,
        Err(e) => {
            return JsonRpcResponse::error(id, -32603, format!("Failed to connect to database: {}", e));
        }
    };

    // Build filter based on params
    let filter = if let std::option::Option::Some(status_str) = params.status {
        // Parse status string to TaskStatus enum
        let status = match status_str.as_str() {
            "Todo" => task_manager::domain::task_status::TaskStatus::Todo,
            "InProgress" => task_manager::domain::task_status::TaskStatus::InProgress,
            "Completed" => task_manager::domain::task_status::TaskStatus::Completed,
            "Archived" => task_manager::domain::task_status::TaskStatus::Archived,
            "PendingEnhancement" => task_manager::domain::task_status::TaskStatus::PendingEnhancement,
            "PendingComprehensionTest" => task_manager::domain::task_status::TaskStatus::PendingComprehensionTest,
            "PendingFollowOn" => task_manager::domain::task_status::TaskStatus::PendingFollowOn,
            "PendingDecomposition" => task_manager::domain::task_status::TaskStatus::PendingDecomposition,
            "Decomposed" => task_manager::domain::task_status::TaskStatus::Decomposed,
            "OrchestrationComplete" => task_manager::domain::task_status::TaskStatus::OrchestrationComplete,
            _ => {
                return JsonRpcResponse::error(id, -32602, format!("Invalid status: {}", status_str));
            }
        };
        task_manager::ports::task_repository_port::TaskFilter::ByStatus(status)
    } else if let std::option::Option::Some(assignee) = params.assignee {
        task_manager::ports::task_repository_port::TaskFilter::ByAgentPersona(assignee)
    } else {
        task_manager::ports::task_repository_port::TaskFilter::All
    };

    // Query tasks using async method
    let tasks = match task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::find_async(
        &adapter,
        &filter,
        hexser::ports::repository::FindOptions {
            sort: std::option::Option::Some(std::vec![hexser::ports::repository::Sort {
                key: task_manager::ports::task_repository_port::TaskSortKey::CreatedAt,
                direction: hexser::ports::repository::Direction::Desc,
            }]),
            limit: std::option::Option::None,
            offset: std::option::Option::None,
        }
    ).await {
        Ok(t) => t,
        Err(e) => {
            return JsonRpcResponse::error(id, -32603, format!("Database query failed: {:?}", e));
        }
    };

    // Serialize tasks to JSON
    let tasks_json: std::vec::Vec<serde_json::Value> = tasks
        .iter()
        .map(|task| {
            serde_json::json!({
                "id": task.id,
                "title": task.title,
                "status": format!("{:?}", task.status),
                "agent_persona": task.agent_persona,
                "due_date": task.due_date,
                "created_at": task.created_at.to_rfc3339(),
                "updated_at": task.updated_at.to_rfc3339(),
            })
        })
        .collect();

    let result = serde_json::json!({
        "tasks": tasks_json,
        "count": tasks.len()
    });

    JsonRpcResponse::success(id, result)
}

/// Handles the 'add_task' tool.
async fn handle_add_task(id: serde_json::Value, params: serde_json::Value) -> JsonRpcResponse {
    #[derive(Deserialize)]
    struct AddTaskParams {
        title: String,
        #[serde(default)]
        assignee: std::option::Option<String>,
        #[serde(default)]
        due_date: std::option::Option<String>,
    }

    let params: AddTaskParams = match serde_json::from_value(params) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(id, -32602, format!("Invalid params: {}", e));
        }
    };

    eprintln!("   Creating task: {}", params.title);

    // Create task from action item
    let action_item = transcript_extractor::domain::action_item::ActionItem {
        title: params.title,
        assignee: params.assignee,
        due_date: params.due_date,
    };

    let task = task_manager::domain::task::Task::from_action_item(&action_item, None);

    eprintln!("   Task ID: {}", task.id);

    // Get database path from .rigger directory
    let db_path = match get_rigger_db_path() {
        Ok(path) => path,
        Err(e) => {
            return JsonRpcResponse::error(id, -32603, format!("Database error: {}", e));
        }
    };

    // Connect to database
    let adapter = match task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::connect_and_init(
        &std::format!("sqlite:{}", db_path.display())
    ).await {
        Ok(a) => a,
        Err(e) => {
            return JsonRpcResponse::error(id, -32603, format!("Failed to connect to database: {}", e));
        }
    };

    // Save task to database
    match task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::save_async(&adapter, task.clone()).await {
        Ok(_) => {},
        Err(e) => {
            return JsonRpcResponse::error(id, -32603, format!("Failed to save task: {:?}", e));
        }
    };

    eprintln!("   Task saved to database");

    let result = serde_json::json!({
        "task_id": task.id,
        "title": task.title,
        "status": format!("{:?}", task.status)
    });

    JsonRpcResponse::success(id, result)
}

/// Handles the 'update_task' tool.
async fn handle_update_task(
    id: serde_json::Value,
    params: serde_json::Value,
) -> JsonRpcResponse {
    #[derive(Deserialize)]
    struct UpdateTaskParams {
        task_id: String,
        #[serde(default)]
        status: std::option::Option<String>,
        #[serde(default)]
        assignee: std::option::Option<String>,
    }

    let params: UpdateTaskParams = match serde_json::from_value(params) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(id, -32602, format!("Invalid params: {}", e));
        }
    };

    eprintln!("   Updating task: {}", params.task_id);
    eprintln!("   New status: {:?}", params.status);

    // Get database path from .rigger directory
    let db_path = match get_rigger_db_path() {
        Ok(path) => path,
        Err(e) => {
            return JsonRpcResponse::error(id, -32603, format!("Database error: {}", e));
        }
    };

    // Connect to database
    let adapter = match task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::connect_and_init(
        &std::format!("sqlite:{}", db_path.display())
    ).await {
        Ok(a) => a,
        Err(e) => {
            return JsonRpcResponse::error(id, -32603, format!("Failed to connect to database: {}", e));
        }
    };

    // Load existing task
    let mut task = match task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::find_one_async(
        &adapter,
        &task_manager::ports::task_repository_port::TaskFilter::ById(params.task_id.clone())
    ).await {
        Ok(Some(t)) => t,
        Ok(None) => {
            return JsonRpcResponse::error(id, -32602, format!("Task not found: {}", params.task_id));
        }
        Err(e) => {
            return JsonRpcResponse::error(id, -32603, format!("Database query failed: {:?}", e));
        }
    };

    // Update fields if provided
    if let std::option::Option::Some(status_str) = params.status {
        let status = match status_str.as_str() {
            "Todo" => task_manager::domain::task_status::TaskStatus::Todo,
            "InProgress" => task_manager::domain::task_status::TaskStatus::InProgress,
            "Completed" => task_manager::domain::task_status::TaskStatus::Completed,
            "Archived" => task_manager::domain::task_status::TaskStatus::Archived,
            "PendingEnhancement" => task_manager::domain::task_status::TaskStatus::PendingEnhancement,
            "PendingComprehensionTest" => task_manager::domain::task_status::TaskStatus::PendingComprehensionTest,
            "PendingFollowOn" => task_manager::domain::task_status::TaskStatus::PendingFollowOn,
            "PendingDecomposition" => task_manager::domain::task_status::TaskStatus::PendingDecomposition,
            "Decomposed" => task_manager::domain::task_status::TaskStatus::Decomposed,
            "OrchestrationComplete" => task_manager::domain::task_status::TaskStatus::OrchestrationComplete,
            _ => {
                return JsonRpcResponse::error(id, -32602, format!("Invalid status: {}", status_str));
            }
        };
        task.status = status;
    }

    if let std::option::Option::Some(assignee) = params.assignee {
        task.agent_persona = std::option::Option::Some(assignee);
    }

    task.updated_at = chrono::Utc::now();

    // Save updated task
    match task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::save_async(&adapter, task.clone()).await {
        Ok(_) => {},
        Err(e) => {
            return JsonRpcResponse::error(id, -32603, format!("Failed to save task: {:?}", e));
        }
    };

    eprintln!("   Task updated in database");

    let result = serde_json::json!({
        "success": true,
        "task_id": params.task_id,
        "status": format!("{:?}", task.status)
    });

    JsonRpcResponse::success(id, result)
}

/// Handles the 'parse_prd' tool.
async fn handle_parse_prd(id: serde_json::Value, params: serde_json::Value) -> JsonRpcResponse {
    #[derive(Deserialize)]
    struct ParsePrdParams {
        prd_file_path: String,
    }

    let params: ParsePrdParams = match serde_json::from_value(params) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(id, -32602, format!("Invalid params: {}", e));
        }
    };

    eprintln!("   Parsing PRD: {}", params.prd_file_path);

    // Read PRD file
    let content = match std::fs::read_to_string(&params.prd_file_path) {
        Ok(c) => c,
        Err(e) => {
            return JsonRpcResponse::error(
                id,
                -32603,
                format!("Failed to read PRD file: {}", e),
            );
        }
    };

    // Parse PRD markdown (using placeholder project ID for API compatibility)
    let prd = match task_manager::infrastructure::markdown_parsers::prd_parser::parse_prd_markdown("default-project", &content) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(id, -32603, format!("Failed to parse PRD: {}", e));
        }
    };

    eprintln!("   PRD title: {}", prd.title);
    eprintln!("   Objectives: {}", prd.objectives.len());

    // TODO: Use RigPRDParserAdapter to generate tasks from PRD via LLM

    let result = serde_json::json!({
        "prd_title": prd.title,
        "objectives_count": prd.objectives.len(),
        "tech_stack_count": prd.tech_stack.len(),
        "constraints_count": prd.constraints.len(),
        "tasks_generated": 0
    });

    JsonRpcResponse::success(id, result)
}

/// Handles the 'get_resource' method for MCP resources.
async fn handle_get_resource(
    id: serde_json::Value,
    params: serde_json::Value,
) -> JsonRpcResponse {
    #[derive(Deserialize)]
    struct GetResourceParams {
        resource_name: String,
    }

    let params: GetResourceParams = match serde_json::from_value(params) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(id, -32602, format!("Invalid params: {}", e));
        }
    };

    eprintln!("   Getting resource: {}", params.resource_name);

    match params.resource_name.as_str() {
        "tasks.json" => {
            // Get database path from .rigger directory
            let db_path = match get_rigger_db_path() {
                Ok(path) => path,
                Err(e) => {
                    return JsonRpcResponse::error(id, -32603, format!("Database error: {}", e));
                }
            };

            // Connect to database
            let adapter = match task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::connect_and_init(
                &std::format!("sqlite:{}", db_path.display())
            ).await {
                Ok(a) => a,
                Err(e) => {
                    return JsonRpcResponse::error(id, -32603, format!("Failed to connect to database: {}", e));
                }
            };

            // Query all tasks
            let tasks = match task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::find_async(
                &adapter,
                &task_manager::ports::task_repository_port::TaskFilter::All,
                hexser::ports::repository::FindOptions {
                    sort: std::option::Option::Some(std::vec![hexser::ports::repository::Sort {
                        key: task_manager::ports::task_repository_port::TaskSortKey::CreatedAt,
                        direction: hexser::ports::repository::Direction::Desc,
                    }]),
                    limit: std::option::Option::None,
                    offset: std::option::Option::None,
                }
            ).await {
                Ok(t) => t,
                Err(e) => {
                    return JsonRpcResponse::error(id, -32603, format!("Database query failed: {:?}", e));
                }
            };

            // Serialize tasks to JSON
            let tasks_json: std::vec::Vec<serde_json::Value> = tasks
                .iter()
                .map(|task| {
                    serde_json::json!({
                        "id": task.id,
                        "title": task.title,
                        "status": format!("{:?}", task.status),
                        "agent_persona": task.agent_persona,
                        "due_date": task.due_date,
                        "created_at": task.created_at.to_rfc3339(),
                        "updated_at": task.updated_at.to_rfc3339(),
                    })
                })
                .collect();

            let result = serde_json::json!({
                "tasks": tasks_json
            });
            JsonRpcResponse::success(id, result)
        }
        "config.json" => {
            // Read .rigger/config.json if it exists
            match get_rigger_config() {
                Ok(config) => JsonRpcResponse::success(id, config),
                Err(e) => {
                    JsonRpcResponse::error(id, -32603, format!("Failed to read config: {}", e))
                }
            }
        }
        _ => JsonRpcResponse::error(
            id,
            -32602,
            format!("Unknown resource: {}", params.resource_name),
        ),
    }
}

/// Gets the path to the Rigger database file.
fn get_rigger_db_path() -> anyhow::Result<std::path::PathBuf> {
    let cwd = std::env::current_dir()?;
    let rigger_dir = cwd.join(".rigger");

    if !rigger_dir.exists() {
        anyhow::bail!(".rigger directory not found. Run 'rig init' first.");
    }

    Ok(rigger_dir.join("tasks.db"))
}

/// Reads the Rigger configuration from .rigger/config.json.
fn get_rigger_config() -> anyhow::Result<serde_json::Value> {
    let cwd = std::env::current_dir()?;
    let config_path = cwd.join(".rigger").join("config.json");

    if !config_path.exists() {
        anyhow::bail!("config.json not found in .rigger directory");
    }

    let content = std::fs::read_to_string(config_path)?;
    let config: serde_json::Value = serde_json::from_str(&content)?;

    Ok(config)
}
