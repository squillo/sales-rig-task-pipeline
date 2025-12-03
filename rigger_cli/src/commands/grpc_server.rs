// Implementation of the 'rig grpc' command (gRPC server mode).
//!
//! Starts the gRPC server using tonic for distributed task management.
//! Supports bidirectional streaming for real-time task event broadcasting to sidecars.
//!
//! # Protocol
//!
//! - **Transport**: gRPC over HTTP/2
//! - **Format**: Protocol Buffers (protobuf)
//! - **Port**: 50051 (default)
//!
//! # Supported RPCs
//!
//! ## Task Management
//! - `ListTasks`: List tasks with optional filters
//! - `AddTask`: Create a new task
//! - `UpdateTask`: Update task status/assignee
//! - `GetTask`: Retrieve a single task by ID
//! - `DeleteTask`: Archive a task
//!
//! ## PRD Operations
//! - `ParsePRD`: Parse PRD file and extract sections
//! - `GenerateTasksFromPRD`: Generate tasks from PRD using LLM
//!
//! ## Orchestration
//! - `OrchestrateTask`: Run task through enhancement + comprehension test flow
//!
//! ## Event Streaming (for Sidecars)
//! - `SubscribeToTaskEvents`: Subscribe to task events stream
//! - `TaskEventStream`: Bidirectional streaming for real-time updates
//!
//! Revision History
//! - 2025-11-23T19:30:00Z @AI: Implement gRPC server with tonic for sidecar broadcast support.

pub mod rigger {
    pub mod v1 {
        tonic::include_proto!("rigger.v1");
    }
}

use rigger::v1::rigger_service_server::{RiggerService, RiggerServiceServer};
use rigger::v1::*;
use tonic::{Request, Response, Status};
use task_orchestrator::ports::prd_parser_port::PRDParserPort;

/// Rigger gRPC service implementation.
///
/// Implements all RPC methods defined in rigger.proto, with database integration
/// and event broadcasting for sidecar consumers.
pub struct RiggerServiceImpl {
    /// Database path for task persistence
    db_path: std::path::PathBuf,
    /// Broadcast channel for task events (sender)
    event_tx: tokio::sync::broadcast::Sender<TaskEvent>,
}

impl RiggerServiceImpl {
    /// Creates a new RiggerServiceImpl with the given database path.
    ///
    /// Initializes the broadcast channel for task events with capacity 1000.
    pub fn new(db_path: std::path::PathBuf) -> Self {
        let (event_tx, _) = tokio::sync::broadcast::channel(1000);
        RiggerServiceImpl { db_path, event_tx }
    }

    /// Gets the database URL string for SqliteTaskAdapter.
    fn db_url(&self) -> std::string::String {
        std::format!("sqlite:{}", self.db_path.display())
    }

    /// Broadcasts a task event to all sidecar subscribers.
    fn broadcast_event(&self, event: TaskEvent) {
        // Ignore send errors (no subscribers is OK)
        let _ = self.event_tx.send(event);
    }

    /// Converts domain Task to protobuf Task.
    fn task_to_proto(&self, task: &task_manager::domain::task::Task) -> Task {
        Task {
            id: task.id.clone(),
            title: task.title.clone(),
            assignee: task.agent_persona.clone(),
            due_date: task.due_date.clone(),
            status: self.status_to_proto(&task.status) as i32,
            source_transcript_id: task.source_transcript_id.clone(),
            source_prd_id: task.source_prd_id.clone(),
            parent_task_id: task.parent_task_id.clone(),
            subtask_ids: task.subtask_ids.clone(),
            created_at: task.created_at.to_rfc3339(),
            updated_at: task.updated_at.to_rfc3339(),
            complexity: task.complexity.map(|c| c as u32),
            reasoning: task.reasoning.clone(),
            context_files: task.context_files.clone(),
            dependencies: task.dependencies.clone(),
        }
    }

    /// Converts domain TaskStatus to protobuf TaskStatus.
    fn status_to_proto(&self, status: &task_manager::domain::task_status::TaskStatus) -> TaskStatus {
        match status {
            task_manager::domain::task_status::TaskStatus::Todo => TaskStatus::Todo,
            task_manager::domain::task_status::TaskStatus::InProgress => TaskStatus::InProgress,
            task_manager::domain::task_status::TaskStatus::PendingEnhancement => TaskStatus::PendingEnhancement,
            task_manager::domain::task_status::TaskStatus::PendingComprehensionTest => TaskStatus::PendingComprehensionTest,
            task_manager::domain::task_status::TaskStatus::PendingFollowOn => TaskStatus::PendingFollowOn,
            task_manager::domain::task_status::TaskStatus::PendingDecomposition => TaskStatus::PendingDecomposition,
            task_manager::domain::task_status::TaskStatus::Decomposed => TaskStatus::Decomposed,
            task_manager::domain::task_status::TaskStatus::OrchestrationComplete => TaskStatus::OrchestrationComplete,
            task_manager::domain::task_status::TaskStatus::Completed => TaskStatus::Completed,
            task_manager::domain::task_status::TaskStatus::Archived => TaskStatus::Archived,
            task_manager::domain::task_status::TaskStatus::Errored => TaskStatus::Archived, // Map Errored to Archived for protobuf
        }
    }

    /// Converts protobuf TaskStatus to domain TaskStatus.
    fn proto_to_status(&self, status: i32) -> std::result::Result<task_manager::domain::task_status::TaskStatus, Status> {
        match TaskStatus::try_from(status) {
            Ok(TaskStatus::Todo) => Ok(task_manager::domain::task_status::TaskStatus::Todo),
            Ok(TaskStatus::InProgress) => Ok(task_manager::domain::task_status::TaskStatus::InProgress),
            Ok(TaskStatus::PendingEnhancement) => Ok(task_manager::domain::task_status::TaskStatus::PendingEnhancement),
            Ok(TaskStatus::PendingComprehensionTest) => Ok(task_manager::domain::task_status::TaskStatus::PendingComprehensionTest),
            Ok(TaskStatus::PendingFollowOn) => Ok(task_manager::domain::task_status::TaskStatus::PendingFollowOn),
            Ok(TaskStatus::PendingDecomposition) => Ok(task_manager::domain::task_status::TaskStatus::PendingDecomposition),
            Ok(TaskStatus::Decomposed) => Ok(task_manager::domain::task_status::TaskStatus::Decomposed),
            Ok(TaskStatus::OrchestrationComplete) => Ok(task_manager::domain::task_status::TaskStatus::OrchestrationComplete),
            Ok(TaskStatus::Completed) => Ok(task_manager::domain::task_status::TaskStatus::Completed),
            Ok(TaskStatus::Archived) => Ok(task_manager::domain::task_status::TaskStatus::Archived),
            _ => Err(Status::invalid_argument("Invalid task status")),
        }
    }
}

#[tonic::async_trait]
impl RiggerService for RiggerServiceImpl {
    async fn list_tasks(
        &self,
        request: Request<ListTasksRequest>,
    ) -> std::result::Result<Response<ListTasksResponse>, Status> {
        let req = request.into_inner();

        // Connect to database
        let adapter = task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::connect_and_init(&self.db_url())
            .await
            .map_err(|e| Status::internal(std::format!("Database connection failed: {}", e)))?;

        // Build filter
        let filter = if let std::option::Option::Some(status) = req.status {
            let domain_status = self.proto_to_status(status)?;
            task_manager::ports::task_repository_port::TaskFilter::ByStatus(domain_status)
        } else if let std::option::Option::Some(assignee) = req.assignee {
            task_manager::ports::task_repository_port::TaskFilter::ByAgentPersona(assignee)
        } else {
            task_manager::ports::task_repository_port::TaskFilter::All
        };

        // Query tasks
        let tasks = task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::find_async(
            &adapter,
            &filter,
            hexser::ports::repository::FindOptions {
                sort: std::option::Option::Some(std::vec![hexser::ports::repository::Sort {
                    key: task_manager::ports::task_repository_port::TaskSortKey::CreatedAt,
                    direction: hexser::ports::repository::Direction::Desc,
                }]),
                limit: req.limit,
                offset: req.offset.map(|o| o as u64),
            },
        )
        .await
        .map_err(|e| Status::internal(std::format!("Database query failed: {:?}", e)))?;

        let proto_tasks: std::vec::Vec<Task> = tasks.iter().map(|t| self.task_to_proto(t)).collect();
        let total_count = proto_tasks.len() as u32;

        Ok(Response::new(ListTasksResponse {
            tasks: proto_tasks,
            total_count,
        }))
    }

    async fn add_task(
        &self,
        request: Request<AddTaskRequest>,
    ) -> std::result::Result<Response<AddTaskResponse>, Status> {
        let req = request.into_inner();

        // Create task from action item
        let action_item = transcript_extractor::domain::action_item::ActionItem {
            title: req.title,
            assignee: req.assignee,
            due_date: req.due_date,
        };

        let mut task = task_manager::domain::task::Task::from_action_item(&action_item, None);
        task.source_prd_id = req.source_prd_id;
        task.parent_task_id = req.parent_task_id;

        // Connect to database and save
        let adapter = task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::connect_and_init(&self.db_url())
            .await
            .map_err(|e| Status::internal(std::format!("Database connection failed: {}", e)))?;

        task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::save_async(&adapter, task.clone())
            .await
            .map_err(|e| Status::internal(std::format!("Failed to save task: {:?}", e)))?;

        // Broadcast event
        let event = TaskEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            event_type: TaskEventType::Created as i32,
            task: std::option::Option::Some(self.task_to_proto(&task)),
            actor: std::option::Option::None,
            metadata: std::collections::HashMap::new(),
        };
        self.broadcast_event(event);

        Ok(Response::new(AddTaskResponse {
            task: std::option::Option::Some(self.task_to_proto(&task)),
        }))
    }

    async fn update_task(
        &self,
        request: Request<UpdateTaskRequest>,
    ) -> std::result::Result<Response<UpdateTaskResponse>, Status> {
        let req = request.into_inner();

        // Connect to database
        let adapter = task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::connect_and_init(&self.db_url())
            .await
            .map_err(|e| Status::internal(std::format!("Database connection failed: {}", e)))?;

        // Load existing task
        let mut task = task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::find_one_async(
            &adapter,
            &task_manager::ports::task_repository_port::TaskFilter::ById(req.task_id.clone()),
        )
        .await
        .map_err(|e| Status::internal(std::format!("Database query failed: {:?}", e)))?
        .ok_or_else(|| Status::not_found(std::format!("Task not found: {}", req.task_id)))?;

        // Update fields
        if let std::option::Option::Some(status) = req.status {
            task.status = self.proto_to_status(status)?;
        }
        if let std::option::Option::Some(assignee) = req.assignee {
            task.agent_persona = std::option::Option::Some(assignee);
        }
        if let std::option::Option::Some(due_date) = req.due_date {
            task.due_date = std::option::Option::Some(due_date);
        }
        task.updated_at = chrono::Utc::now();

        // Save updated task
        task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::save_async(&adapter, task.clone())
            .await
            .map_err(|e| Status::internal(std::format!("Failed to save task: {:?}", e)))?;

        // Broadcast event
        let event = TaskEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            event_type: TaskEventType::Updated as i32,
            task: std::option::Option::Some(self.task_to_proto(&task)),
            actor: std::option::Option::None,
            metadata: std::collections::HashMap::new(),
        };
        self.broadcast_event(event);

        Ok(Response::new(UpdateTaskResponse {
            task: std::option::Option::Some(self.task_to_proto(&task)),
        }))
    }

    async fn get_task(
        &self,
        request: Request<GetTaskRequest>,
    ) -> std::result::Result<Response<GetTaskResponse>, Status> {
        let req = request.into_inner();

        // Connect to database
        let adapter = task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::connect_and_init(&self.db_url())
            .await
            .map_err(|e| Status::internal(std::format!("Database connection failed: {}", e)))?;

        // Load task
        let task = task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::find_one_async(
            &adapter,
            &task_manager::ports::task_repository_port::TaskFilter::ById(req.task_id.clone()),
        )
        .await
        .map_err(|e| Status::internal(std::format!("Database query failed: {:?}", e)))?
        .ok_or_else(|| Status::not_found(std::format!("Task not found: {}", req.task_id)))?;

        Ok(Response::new(GetTaskResponse {
            task: std::option::Option::Some(self.task_to_proto(&task)),
        }))
    }

    async fn delete_task(
        &self,
        request: Request<DeleteTaskRequest>,
    ) -> std::result::Result<Response<DeleteTaskResponse>, Status> {
        let req = request.into_inner();

        // Connect to database
        let adapter = task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::connect_and_init(&self.db_url())
            .await
            .map_err(|e| Status::internal(std::format!("Database connection failed: {}", e)))?;

        // Load task and set status to Archived
        let mut task = task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::find_one_async(
            &adapter,
            &task_manager::ports::task_repository_port::TaskFilter::ById(req.task_id.clone()),
        )
        .await
        .map_err(|e| Status::internal(std::format!("Database query failed: {:?}", e)))?
        .ok_or_else(|| Status::not_found(std::format!("Task not found: {}", req.task_id)))?;

        task.status = task_manager::domain::task_status::TaskStatus::Archived;
        task.updated_at = chrono::Utc::now();

        task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::save_async(&adapter, task.clone())
            .await
            .map_err(|e| Status::internal(std::format!("Failed to save task: {:?}", e)))?;

        // Broadcast event
        let event = TaskEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            event_type: TaskEventType::Deleted as i32,
            task: std::option::Option::Some(self.task_to_proto(&task)),
            actor: std::option::Option::None,
            metadata: std::collections::HashMap::new(),
        };
        self.broadcast_event(event);

        Ok(Response::new(DeleteTaskResponse { success: true }))
    }

    async fn parse_prd(
        &self,
        request: Request<ParsePrdRequest>,
    ) -> std::result::Result<Response<ParsePrdResponse>, Status> {
        let req = request.into_inner();

        // Read PRD file
        let content = std::fs::read_to_string(&req.prd_file_path)
            .map_err(|e| Status::not_found(std::format!("Failed to read PRD file: {}", e)))?;

        // Parse PRD markdown (using placeholder project ID for API compatibility)
        let prd = task_manager::infrastructure::markdown_parsers::prd_parser::parse_prd_markdown("default-project", &content)
            .map_err(|e| Status::invalid_argument(std::format!("Failed to parse PRD: {}", e)))?;

        Ok(Response::new(ParsePrdResponse {
            prd_id: prd.id.clone(),
            prd_title: prd.title.clone(),
            objectives: prd.objectives.clone(),
            tech_stack: prd.tech_stack.clone(),
            constraints: prd.constraints.clone(),
        }))
    }

    async fn generate_tasks_from_prd(
        &self,
        request: Request<GenerateTasksFromPrdRequest>,
    ) -> std::result::Result<Response<GenerateTasksFromPrdResponse>, Status> {
        let req = request.into_inner();

        // Connect to database
        let adapter = task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::connect_and_init(&self.db_url())
            .await
            .map_err(|e| Status::internal(std::format!("Database connection failed: {}", e)))?;

        // Load PRD from database (assuming it was saved when parsed)
        // For now, we'll reconstruct from the PRD ID if it's a file path
        // TODO: Store PRDs in database for proper retrieval

        // Try to read PRD file if prd_id looks like a path
        let prd_content = if req.prd_id.contains('/') || req.prd_id.ends_with(".md") {
            std::fs::read_to_string(&req.prd_id)
                .map_err(|e| Status::not_found(std::format!("Failed to read PRD file: {}", e)))?
        } else {
            return Err(Status::invalid_argument("prd_id must be a file path (PRD database storage not yet implemented)"));
        };

        // Parse PRD (using placeholder project ID for API compatibility)
        let prd = task_manager::infrastructure::markdown_parsers::prd_parser::parse_prd_markdown("default-project", &prd_content)
            .map_err(|e| Status::invalid_argument(std::format!("Failed to parse PRD: {}", e)))?;

        // Query personas from database for task assignment
        let persona_rows = sqlx::query("SELECT id, project_id, name, role, description, llm_provider, llm_model, is_default, created_at, updated_at FROM personas")
            .fetch_all(adapter.pool())
            .await
            .map_err(|e| Status::internal(std::format!("Failed to query personas: {}", e)))?;

        let mut personas = std::vec::Vec::new();
        for row in persona_rows {
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

        // Use RigPRDParserAdapter to generate tasks
        let prd_parser = task_orchestrator::adapters::rig_prd_parser_adapter::RigPRDParserAdapter::new(
            req.model.clone(),
            req.model.clone(), // Use same model for fallback
            personas
        );

        let tasks = prd_parser
            .parse_prd_to_tasks(&prd)
            .await
            .map_err(|e| Status::internal(std::format!("Failed to generate tasks from PRD: {}", e)))?;

        // Save generated tasks to database
        for task in &tasks {
            task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::save_async(&adapter, task.clone())
                .await
                .map_err(|e| Status::internal(std::format!("Failed to save task: {:?}", e)))?;

            // Broadcast task created event
            let event = TaskEvent {
                event_id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                event_type: TaskEventType::Created as i32,
                task: std::option::Option::Some(self.task_to_proto(task)),
                actor: std::option::Option::Some(std::format!("prd_parser:{}", req.model)),
                metadata: std::collections::HashMap::from([
                    ("source".to_string(), "prd_generation".to_string()),
                    ("prd_id".to_string(), prd.id.clone()),
                    ("prd_title".to_string(), prd.title.clone()),
                ]),
            };
            self.broadcast_event(event);
        }

        let proto_tasks: std::vec::Vec<Task> = tasks.iter().map(|t| self.task_to_proto(t)).collect();
        let tasks_generated = proto_tasks.len() as u32;

        Ok(Response::new(GenerateTasksFromPrdResponse {
            tasks: proto_tasks,
            tasks_generated,
        }))
    }

    async fn orchestrate_task(
        &self,
        request: Request<OrchestrateTaskRequest>,
    ) -> std::result::Result<Response<OrchestrateTaskResponse>, Status> {
        let req = request.into_inner();

        // Connect to database and load task
        let adapter = task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::connect_and_init(&self.db_url())
            .await
            .map_err(|e| Status::internal(std::format!("Database connection failed: {}", e)))?;

        let task = task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::find_one_async(
            &adapter,
            &task_manager::ports::task_repository_port::TaskFilter::ById(req.task_id.clone()),
        )
        .await
        .map_err(|e| Status::internal(std::format!("Database query failed: {:?}", e)))?
        .ok_or_else(|| Status::not_found(std::format!("Task not found: {}", req.task_id)))?;

        // Run task through orchestration flow
        // Create provider factory from model parameter
        let factory = task_orchestrator::adapters::provider_factory::ProviderFactory::new("ollama", &req.model)
            .map_err(|e| Status::internal(std::format!("Failed to create provider factory: {}", e)))?;

        let orchestrated_task = task_orchestrator::use_cases::run_task_with_flow::run_task_with_flow(
            &factory,
            &req.test_type,
            task,
        )
        .await
        .map_err(|e| Status::internal(std::format!("Orchestration failed: {}", e)))?;

        // Save orchestrated task back to database
        task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::save_async(&adapter, orchestrated_task.clone())
            .await
            .map_err(|e| Status::internal(std::format!("Failed to save task: {:?}", e)))?;

        // Build orchestration result
        let routing_decision = match orchestrated_task.status {
            task_manager::domain::task_status::TaskStatus::Decomposed => "decompose",
            task_manager::domain::task_status::TaskStatus::OrchestrationComplete => "pass",
            _ => "enhance",
        };

        // Convert enhancements if present
        let enhancement = orchestrated_task.enhancements.as_ref().and_then(|enhs| {
            enhs.last().map(|enh| Enhancement {
                enhancement_id: enh.enhancement_id.clone(),
                task_id: enh.task_id.clone(),
                timestamp: enh.timestamp.to_rfc3339(),
                enhancement_type: enh.enhancement_type.clone(),
                content: enh.content.clone(),
            })
        });

        // Convert comprehension test if present
        let comprehension_test = orchestrated_task.comprehension_tests.as_ref().and_then(|tests| {
            tests.last().map(|test| ComprehensionTest {
                test_id: test.test_id.clone(),
                task_id: test.task_id.clone(),
                timestamp: test.timestamp.to_rfc3339(),
                test_type: test.test_type.clone(),
                question: test.question.clone(),
                options: test.options.clone().unwrap_or_default(),
                correct_answer: test.correct_answer.clone(),
            })
        });

        // Get subtasks if decomposed
        let subtasks = if orchestrated_task.status == task_manager::domain::task_status::TaskStatus::Decomposed {
            let mut subtask_tasks = std::vec::Vec::new();
            for subtask_id in &orchestrated_task.subtask_ids {
                if let Ok(std::option::Option::Some(subtask)) = task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::find_one_async(
                    &adapter,
                    &task_manager::ports::task_repository_port::TaskFilter::ById(subtask_id.clone()),
                ).await {
                    subtask_tasks.push(self.task_to_proto(&subtask));
                }
            }
            subtask_tasks
        } else {
            std::vec::Vec::new()
        };

        let result = OrchestrationResult {
            success: true,
            routing_decision: routing_decision.to_string(),
            enhancement,
            comprehension_test,
            subtasks,
        };

        // Broadcast orchestration event
        let event = TaskEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            event_type: TaskEventType::Orchestrated as i32,
            task: std::option::Option::Some(self.task_to_proto(&orchestrated_task)),
            actor: std::option::Option::None,
            metadata: std::collections::HashMap::from([
                ("model".to_string(), req.model),
                ("test_type".to_string(), req.test_type),
                ("routing_decision".to_string(), routing_decision.to_string()),
            ]),
        };
        self.broadcast_event(event);

        Ok(Response::new(OrchestrateTaskResponse {
            task: std::option::Option::Some(self.task_to_proto(&orchestrated_task)),
            result: std::option::Option::Some(result),
        }))
    }

    async fn subscribe_to_task_events(
        &self,
        request: Request<SubscribeToTaskEventsRequest>,
    ) -> std::result::Result<Response<Self::SubscribeToTaskEventsStream>, Status> {
        let _req = request.into_inner();

        // Subscribe to broadcast channel
        let mut event_rx = self.event_tx.subscribe();

        // Create stream from receiver
        let stream = async_stream::stream! {
            while let Ok(event) = event_rx.recv().await {
                yield Ok(event);
            }
        };

        Ok(Response::new(Box::pin(stream)))
    }

    type SubscribeToTaskEventsStream = std::pin::Pin<
        Box<
            dyn tokio_stream::Stream<Item = std::result::Result<TaskEvent, Status>>
                + Send
                + 'static,
        >,
    >;

    async fn task_event_stream(
        &self,
        _request: Request<tonic::Streaming<TaskEventStreamRequest>>,
    ) -> std::result::Result<Response<Self::TaskEventStreamStream>, Status> {
        // TODO: Implement bidirectional streaming
        Err(Status::unimplemented("TaskEventStream not yet implemented"))
    }

    type TaskEventStreamStream = std::pin::Pin<
        Box<
            dyn tokio_stream::Stream<Item = std::result::Result<TaskEvent, Status>>
                + Send
                + 'static,
        >,
    >;
}

/// Executes the 'rig grpc' command.
///
/// Starts gRPC server on port 50051 (default) and listens for incoming connections.
/// All task events are broadcast to subscribed sidecars via the SubscribeToTaskEvents stream.
///
/// # Errors
///
/// Returns an error if server initialization fails or bind errors occur.
pub async fn execute() -> anyhow::Result<()> {
    // Get database path
    let cwd = std::env::current_dir()?;
    let rigger_dir = cwd.join(".rigger");
    if !rigger_dir.exists() {
        anyhow::bail!(".rigger directory not found. Run 'rig init' first.");
    }
    let db_path = rigger_dir.join("tasks.db");

    // Create service
    let service = RiggerServiceImpl::new(db_path);
    let addr = "[::1]:50051".parse()?;

    eprintln!("🚀 Rigger gRPC Server starting...");
    eprintln!("   Protocol: gRPC over HTTP/2");
    eprintln!("   Address: {}", addr);
    eprintln!("   Database: {:?}", service.db_path);
    eprintln!("   Broadcast: Enabled (1000 event buffer)");
    eprintln!();

    // Start server
    tonic::transport::Server::builder()
        .add_service(RiggerServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
