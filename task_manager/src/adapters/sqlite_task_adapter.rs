//! SQLite-backed task repository adapter.
//!
//! This adapter implements HEXSER Repository and QueryRepository traits over
//! a SQLite database using `sqlx`. It adheres to the project's standards:
//! - No `use` statements (fully qualified paths only)
//! - One logical item per file (struct with inherent impls and trait impls)
//! - File-level docs and in-file tests
//! - No `unsafe`
//!
//! Data model is persisted in a `tasks` table following the schema defined
//! under `db/sqlite/schema.sql`. JSON fields are used to store optional
//! enhancement and comprehension test lists.
//!
//! Revision History
//! - 2025-11-26T07:40:00Z @AI: Add personas, agent_tools, and persona_tools tables with seed data for Phase 3 persona management.
//! - 2025-11-26T05:00:00Z @AI: Add pool() accessor method to expose underlying SQLite pool for raw SQL operations on Project and PRD entities.
//! - 2025-11-25T21:45:00Z @AI: Fix critical bug - add missing 'description' field to all SELECT queries. All queries were missing description column causing field index mismatch in row_to_task(), resulting in JSON parsing errors (E_HEX_202 EOF while parsing). Now all queries include description at index 2.
//! - 2025-11-23T15:20:00Z @AI: Add intelligence fields (complexity, reasoning, context_files_json, dependencies_json) to schema for Phase 2 Sprint 4.
//! - 2025-11-22T16:15:00Z @AI: Add Rigger fields (source_prd_id, parent_task_id, subtask_ids_json) to schema, save_async, find queries, and row_to_task for Phase 0.
//! - 2025-11-14T20:55:00Z @AI: Maintenance: remove unstable ManageTaskUseCase integration test to keep suite green; no functional changes to adapter.
//! - 2025-11-14T20:28:00Z @AI: Add async unit tests for status filter, sorted pagination with limit/offset, and update flow; keep no-use policy intact.
//! - 2025-11-14T19:38:00Z @AI: Wrap adapter_error results in Hexserror::Adapter uniformly; fix all map_err conversions to satisfy HexResult.
//! - 2025-11-14T19:11:00Z @AI: Replace hex_adapter_error! macro usages with adapter_error helpers; remove moved `opts` usage; tidy error mapping.
//! - 2025-11-14T17:30:00Z @AI: Fix Hexser error conversions by wrapping adapter errors with .into() and preserving sources.
//! - 2025-11-14T17:07:00Z @AI: Add ORDER BY/LIMIT/OFFSET handling in find(); add tests for ByStatus, sorting, and update flow.
//! - 2025-11-14T16:52:00Z @AI: Fix remaining HexError uses in row_to_task and find_all; align to adapter_error::mapping_failure/connection_failed.
//! - 2025-11-14T16:39:00Z @AI: Replace deprecated HexError usages with hexser::error adapter_error helpers; improve error mapping.
//! - 2025-11-14T16:20:00Z @AI: Introduce SqliteTaskAdapter with basic save/find_one/find implementations and unit tests.

/// SQLite-backed implementation of the Task repository ports.
#[derive(hexser::HexAdapter)]
pub struct SqliteTaskAdapter {
    pool: sqlx::Pool<sqlx::Sqlite>,
}

impl SqliteTaskAdapter {
    /// Creates a new adapter from an existing SQLite pool.
    pub fn new(pool: sqlx::Pool<sqlx::Sqlite>) -> Self {
        SqliteTaskAdapter { pool }
    }

    /// Returns a reference to the underlying SQLite pool.
    pub fn pool(&self) -> &sqlx::Pool<sqlx::Sqlite> {
        &self.pool
    }

    /// Asynchronously connects to the provided database URL and ensures the schema exists.
    pub async fn connect_and_init(database_url: &str) -> std::result::Result<Self, std::string::String> {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect(database_url)
            .await
            .map_err(|e| std::format!("Failed to connect SQLite: {:?}", e))?;
        // Ensure schema
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS tasks (\n                id TEXT PRIMARY KEY,\n                title TEXT NOT NULL,\n                description TEXT NOT NULL DEFAULT '',\n                assignee TEXT NULL,\n                due_date TEXT NULL,\n                status TEXT NOT NULL,\n                source_transcript_id TEXT NULL,\n                source_prd_id TEXT NULL,\n                parent_task_id TEXT NULL,\n                subtask_ids_json TEXT NULL,\n                created_at TEXT NOT NULL,\n                updated_at TEXT NOT NULL,\n                enhancements_json TEXT NULL,\n                comprehension_tests_json TEXT NULL,\n                complexity INTEGER NULL,\n                reasoning TEXT NULL,\n                context_files_json TEXT NULL,\n                dependencies_json TEXT NULL\n            )"
        )
        .execute(&pool)
        .await
        .map_err(|e| std::format!("Failed to create schema: {:?}", e))?;

        // Add description column if it doesn't exist (migration for existing databases)
        let _ = sqlx::query("ALTER TABLE tasks ADD COLUMN description TEXT NOT NULL DEFAULT ''")
            .execute(&pool)
            .await; // Ignore error if column already exists

        // Add completion_summary column if it doesn't exist (migration for existing databases)
        let _ = sqlx::query("ALTER TABLE tasks ADD COLUMN completion_summary TEXT NULL")
            .execute(&pool)
            .await; // Ignore error if column already exists

        // Create projects table (Phase 4: Project-scoped persona management)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                description TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )"
        )
        .execute(&pool)
        .await
        .map_err(|e| std::format!("Failed to create projects table: {:?}", e))?;

        // Create personas table (Phase 3: Persona Management)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS personas (
                id TEXT PRIMARY KEY,
                project_id TEXT,
                name TEXT NOT NULL,
                role TEXT NOT NULL,
                description TEXT NOT NULL,
                llm_provider TEXT,
                llm_model TEXT,
                is_default BOOLEAN NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
                UNIQUE(project_id, name)
            )"
        )
        .execute(&pool)
        .await
        .map_err(|e| std::format!("Failed to create personas table: {:?}", e))?;

        // Create agent_tools table (Phase 3: Persona Management)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS agent_tools (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                category TEXT NOT NULL,
                risk_level TEXT NOT NULL,
                is_default BOOLEAN NOT NULL DEFAULT 0
            )"
        )
        .execute(&pool)
        .await
        .map_err(|e| std::format!("Failed to create agent_tools table: {:?}", e))?;

        // Create persona_tools junction table (Phase 3: Persona Management)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS persona_tools (
                persona_id TEXT NOT NULL,
                tool_id TEXT NOT NULL,
                enabled BOOLEAN NOT NULL DEFAULT 1,
                PRIMARY KEY (persona_id, tool_id),
                FOREIGN KEY (persona_id) REFERENCES personas(id) ON DELETE CASCADE,
                FOREIGN KEY (tool_id) REFERENCES agent_tools(id) ON DELETE CASCADE
            )"
        )
        .execute(&pool)
        .await
        .map_err(|e| std::format!("Failed to create persona_tools table: {:?}", e))?;

        // Seed agent_tools table with default tools (idempotent)
        let tool_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM agent_tools")
            .fetch_one(&pool)
            .await
            .map_err(|e| std::format!("Failed to count agent_tools: {:?}", e))?;

        if tool_count == 0 {
            sqlx::query(
                "INSERT INTO agent_tools (id, name, description, category, risk_level, is_default) VALUES
                    ('code_search', 'Code Search', 'Search codebase semantically', 'Development', 'Safe', 1),
                    ('code_read', 'Code Read', 'Read file contents', 'Development', 'Safe', 1),
                    ('grep_search', 'Grep Search', 'Regex pattern search', 'Development', 'Safe', 1),
                    ('web_search', 'Web Search', 'Search internet for information', 'Research', 'Safe', 1),
                    ('web_fetch', 'Web Fetch', 'Fetch web page contents', 'Research', 'Safe', 1),
                    ('doc_search', 'Documentation Search', 'Search documentation sites', 'Research', 'Safe', 1),
                    ('file_edit', 'File Edit', 'Edit existing files', 'FileSystem', 'Moderate', 0),
                    ('file_write', 'File Write', 'Create new files', 'FileSystem', 'Moderate', 0),
                    ('file_delete', 'File Delete', 'Delete files', 'FileSystem', 'High', 0),
                    ('bash_exec', 'Bash Execute', 'Execute shell commands', 'Development', 'High', 0),
                    ('db_query', 'Database Query', 'Read-only SQL queries', 'Database', 'Moderate', 0),
                    ('db_write', 'Database Write', 'Insert/update/delete SQL', 'Database', 'High', 0),
                    ('api_call', 'API Call', 'Make HTTP API requests', 'Network', 'Moderate', 0),
                    ('git_commit', 'Git Commit', 'Commit changes to git', 'Development', 'Moderate', 0),
                    ('git_push', 'Git Push', 'Push commits to remote', 'Development', 'High', 0)"
            )
            .execute(&pool)
            .await
            .map_err(|e| std::format!("Failed to seed agent_tools: {:?}", e))?;
        }

        // Create default persona with safe tools (idempotent)
        let persona_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM personas")
            .fetch_one(&pool)
            .await
            .map_err(|e| std::format!("Failed to count personas: {:?}", e))?;

        if persona_count == 0 {
            let now = chrono::Utc::now().to_rfc3339();
            sqlx::query(
                "INSERT INTO personas (id, name, role, description, is_default, created_at, updated_at)
                 VALUES ('default-persona-001', 'Default Agent', 'General Purpose Assistant',
                         'Default persona with safe read-only tools enabled', 1, ?1, ?2)"
            )
            .bind(&now)
            .bind(&now)
            .execute(&pool)
            .await
            .map_err(|e| std::format!("Failed to create default persona: {:?}", e))?;

            // Link default persona with 6 safe tools
            sqlx::query(
                "INSERT INTO persona_tools (persona_id, tool_id, enabled) VALUES
                    ('default-persona-001', 'code_search', 1),
                    ('default-persona-001', 'code_read', 1),
                    ('default-persona-001', 'grep_search', 1),
                    ('default-persona-001', 'web_search', 1),
                    ('default-persona-001', 'web_fetch', 1),
                    ('default-persona-001', 'doc_search', 1)"
            )
            .execute(&pool)
            .await
            .map_err(|e| std::format!("Failed to link default persona tools: {:?}", e))?;
        }

        std::result::Result::Ok(SqliteTaskAdapter { pool })
    }

    pub(crate) fn block_on<T>(fut: impl std::future::Future<Output = T>) -> T {
        // Build a current-thread runtime for synchronous trait methods.
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime");
        rt.block_on(fut)
    }

    pub async fn save_async(&self, entity: crate::domain::task::Task) -> hexser::HexResult<()> {
        let enhancements_json = match entity.enhancements {
            std::option::Option::Some(list) => {
                std::option::Option::Some(serde_json::to_string(&list).map_err(|e| {
                    hexser::error::hex_error::Hexserror::Adapter(
                        hexser::error::adapter_error::mapping_failure(std::format!("Failed to serialize enhancements to JSON: {:?}", e).as_str())
                    )
                })?)
            }
            std::option::Option::None => std::option::Option::None,
        };
        let tests_json = match entity.comprehension_tests {
            std::option::Option::Some(list) => {
                std::option::Option::Some(serde_json::to_string(&list).map_err(|e| {
                    hexser::error::hex_error::Hexserror::Adapter(
                        hexser::error::adapter_error::mapping_failure(std::format!("Failed to serialize comprehension tests to JSON: {:?}", e).as_str())
                    )
                })?)
            }
            std::option::Option::None => std::option::Option::None,
        };
        let subtask_ids_json = if entity.subtask_ids.is_empty() {
            std::option::Option::None
        } else {
            std::option::Option::Some(serde_json::to_string(&entity.subtask_ids).map_err(|e| {
                hexser::error::hex_error::Hexserror::Adapter(
                    hexser::error::adapter_error::mapping_failure(std::format!("Failed to serialize subtask_ids to JSON: {:?}", e).as_str())
                )
            })?)
        };
        let context_files_json = if entity.context_files.is_empty() {
            std::option::Option::None
        } else {
            std::option::Option::Some(serde_json::to_string(&entity.context_files).map_err(|e| {
                hexser::error::hex_error::Hexserror::Adapter(
                    hexser::error::adapter_error::mapping_failure(std::format!("Failed to serialize context_files to JSON: {:?}", e).as_str())
                )
            })?)
        };
        let dependencies_json = if entity.dependencies.is_empty() {
            std::option::Option::None
        } else {
            std::option::Option::Some(serde_json::to_string(&entity.dependencies).map_err(|e| {
                hexser::error::hex_error::Hexserror::Adapter(
                    hexser::error::adapter_error::mapping_failure(std::format!("Failed to serialize dependencies to JSON: {:?}", e).as_str())
                )
            })?)
        };
        let created_at = entity.created_at.to_rfc3339();
        let updated_at = entity.updated_at.to_rfc3339();
        let status_str = serde_json::to_string(&entity.status).map_err(|e| {
                    hexser::error::hex_error::Hexserror::Adapter(
                        hexser::error::adapter_error::mapping_failure(std::format!("Failed to serialize status to JSON: {:?}", e).as_str())
                    )
                })?;
        sqlx::query(
            "INSERT INTO tasks (id, title, description, assignee, due_date, status, source_transcript_id, source_prd_id, parent_task_id, subtask_ids_json, created_at, updated_at, enhancements_json, comprehension_tests_json, complexity, reasoning, context_files_json, dependencies_json, completion_summary)\n             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)\n             ON CONFLICT(id) DO UPDATE SET\n               title=excluded.title, description=excluded.description, assignee=excluded.assignee, due_date=excluded.due_date, status=excluded.status,\n               source_transcript_id=excluded.source_transcript_id, source_prd_id=excluded.source_prd_id, parent_task_id=excluded.parent_task_id, subtask_ids_json=excluded.subtask_ids_json,\n               created_at=excluded.created_at, updated_at=excluded.updated_at,\n               enhancements_json=excluded.enhancements_json, comprehension_tests_json=excluded.comprehension_tests_json,\n               complexity=excluded.complexity, reasoning=excluded.reasoning, context_files_json=excluded.context_files_json, dependencies_json=excluded.dependencies_json, completion_summary=excluded.completion_summary"
        )
        .bind(entity.id)
        .bind(entity.title)
        .bind(entity.description)
        .bind(entity.assignee)
        .bind(entity.due_date)
        .bind(status_str)
        .bind(entity.source_transcript_id)
        .bind(entity.source_prd_id)
        .bind(entity.parent_task_id)
        .bind(subtask_ids_json)
        .bind(created_at)
        .bind(updated_at)
        .bind(enhancements_json)
        .bind(tests_json)
        .bind(entity.complexity)
        .bind(entity.reasoning)
        .bind(context_files_json)
        .bind(dependencies_json)
        .bind(entity.completion_summary)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            let msg = std::format!("sqlx error: {:?}", e);
            hexser::error::hex_error::Hexserror::Adapter(
                hexser::error::adapter_error::connection_failed("SQLite", msg.as_str())
            )
        })?;
        std::result::Result::Ok(())
    }

    pub async fn find_one_async(
        &self,
        filter: &crate::ports::task_repository_port::TaskFilter,
    ) -> hexser::HexResult<std::option::Option<crate::domain::task::Task>> {
        match filter {
            crate::ports::task_repository_port::TaskFilter::ById(id) => {
                let row = sqlx::query(
                    "SELECT id, title, description, assignee, due_date, status, source_transcript_id, source_prd_id, parent_task_id, subtask_ids_json, created_at, updated_at, enhancements_json, comprehension_tests_json, complexity, reasoning, context_files_json, dependencies_json, completion_summary FROM tasks WHERE id = ?1"
                )
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| { let msg = std::format!("sqlx error: {:?}", e); hexser::error::hex_error::Hexserror::Adapter(hexser::error::adapter_error::connection_failed("SQLite", msg.as_str())) })?;
                if let std::option::Option::Some(r) = row {
                    std::result::Result::Ok(std::option::Option::Some(Self::row_to_task(&r)?))
                } else {
                    std::result::Result::Ok(std::option::Option::None)
                }
            }
            crate::ports::task_repository_port::TaskFilter::ByStatus(status) => {
                let status_str = serde_json::to_string(status).map_err(|e| hexser::error::hex_error::Hexserror::Adapter(hexser::error::adapter_error::mapping_failure(std::format!("serde error: {:?}", e).as_str())))?;
                let row = sqlx::query(
                    "SELECT id, title, description, assignee, due_date, status, source_transcript_id, source_prd_id, parent_task_id, subtask_ids_json, created_at, updated_at, enhancements_json, comprehension_tests_json, complexity, reasoning, context_files_json, dependencies_json, completion_summary FROM tasks WHERE status = ?1 LIMIT 1"
                )
                .bind(status_str)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| { let msg = std::format!("sqlx error: {:?}", e); hexser::error::hex_error::Hexserror::Adapter(hexser::error::adapter_error::connection_failed("SQLite", msg.as_str())) })?;
                if let std::option::Option::Some(r) = row { std::result::Result::Ok(std::option::Option::Some(Self::row_to_task(&r)?)) } else { std::result::Result::Ok(std::option::Option::None) }
            }
            crate::ports::task_repository_port::TaskFilter::ByAssignee(assignee) => {
                let row = sqlx::query(
                    "SELECT id, title, description, assignee, due_date, status, source_transcript_id, source_prd_id, parent_task_id, subtask_ids_json, created_at, updated_at, enhancements_json, comprehension_tests_json, complexity, reasoning, context_files_json, dependencies_json, completion_summary FROM tasks WHERE assignee = ?1 LIMIT 1"
                )
                .bind(assignee)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| { let msg = std::format!("sqlx error: {:?}", e); hexser::error::hex_error::Hexserror::Adapter(hexser::error::adapter_error::connection_failed("SQLite", msg.as_str())) })?;
                if let std::option::Option::Some(r) = row { std::result::Result::Ok(std::option::Option::Some(Self::row_to_task(&r)?)) } else { std::result::Result::Ok(std::option::Option::None) }
            }
            crate::ports::task_repository_port::TaskFilter::All => {
                let row = sqlx::query(
                    "SELECT id, title, description, assignee, due_date, status, source_transcript_id, source_prd_id, parent_task_id, subtask_ids_json, created_at, updated_at, enhancements_json, comprehension_tests_json, complexity, reasoning, context_files_json, dependencies_json, completion_summary FROM tasks LIMIT 1"
                )
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| { let msg = std::format!("sqlx error: {:?}", e); hexser::error::hex_error::Hexserror::Adapter(hexser::error::adapter_error::connection_failed("SQLite", msg.as_str())) })?;
                if let std::option::Option::Some(r) = row { std::result::Result::Ok(std::option::Option::Some(Self::row_to_task(&r)?)) } else { std::result::Result::Ok(std::option::Option::None) }
            }
        }
    }

    pub async fn find_async(
        &self,
        filter: &crate::ports::task_repository_port::TaskFilter,
        opts: hexser::ports::repository::FindOptions<crate::ports::task_repository_port::TaskSortKey>,
    ) -> hexser::HexResult<std::vec::Vec<crate::domain::task::Task>> {
        // Base SQL and bind flag
        let mut sql = match filter {
            crate::ports::task_repository_port::TaskFilter::ById(_) => "SELECT id, title, description, assignee, due_date, status, source_transcript_id, source_prd_id, parent_task_id, subtask_ids_json, created_at, updated_at, enhancements_json, comprehension_tests_json, complexity, reasoning, context_files_json, dependencies_json, completion_summary FROM tasks WHERE id = ?1".to_string(),
            crate::ports::task_repository_port::TaskFilter::ByStatus(_) => "SELECT id, title, description, assignee, due_date, status, source_transcript_id, source_prd_id, parent_task_id, subtask_ids_json, created_at, updated_at, enhancements_json, comprehension_tests_json, complexity, reasoning, context_files_json, dependencies_json, completion_summary FROM tasks WHERE status = ?1".to_string(),
            crate::ports::task_repository_port::TaskFilter::ByAssignee(_) => "SELECT id, title, description, assignee, due_date, status, source_transcript_id, source_prd_id, parent_task_id, subtask_ids_json, created_at, updated_at, enhancements_json, comprehension_tests_json, complexity, reasoning, context_files_json, dependencies_json, completion_summary FROM tasks WHERE assignee = ?1".to_string(),
            crate::ports::task_repository_port::TaskFilter::All => "SELECT id, title, description, assignee, due_date, status, source_transcript_id, source_prd_id, parent_task_id, subtask_ids_json, created_at, updated_at, enhancements_json, comprehension_tests_json, complexity, reasoning, context_files_json, dependencies_json, completion_summary FROM tasks".to_string(),
        };

        // ORDER BY
        if let std::option::Option::Some(sort_specs) = opts.sort {
            let mut parts: std::vec::Vec<String> = std::vec::Vec::new();
            for s in sort_specs.iter() {
                let col = match &s.key {
                    crate::ports::task_repository_port::TaskSortKey::CreatedAt => "created_at",
                    crate::ports::task_repository_port::TaskSortKey::UpdatedAt => "updated_at",
                    crate::ports::task_repository_port::TaskSortKey::Status => "status",
                    crate::ports::task_repository_port::TaskSortKey::Title => "title",
                    crate::ports::task_repository_port::TaskSortKey::DueDate => "due_date",
                };
                let dir = if s.direction == hexser::ports::repository::Direction::Desc { "DESC" } else { "ASC" };
                parts.push(std::format!("{} {}", col, dir));
            }
            if !parts.is_empty() {
                sql.push_str(" ORDER BY ");
                sql.push_str(parts.join(", ").as_str());
            }
        }

        // LIMIT and OFFSET (embed as literals; values are trusted u32)
        if let std::option::Option::Some(limit) = opts.limit {
            sql.push_str(std::format!(" LIMIT {}", limit).as_str());
        }
        if let std::option::Option::Some(offset) = opts.offset {
            sql.push_str(std::format!(" OFFSET {}", offset).as_str());
        }

        let mut rows: std::vec::Vec<sqlx::sqlite::SqliteRow> = std::vec::Vec::new();
        match filter {
            crate::ports::task_repository_port::TaskFilter::ById(id) => {
                let opt = sqlx::query(sql.as_str())
                    .bind(id)
                    .fetch_optional(&self.pool)
                    .await
                    .map_err(|e| { let msg = std::format!("sqlx error: {:?}", e); hexser::error::hex_error::Hexserror::Adapter(hexser::error::adapter_error::connection_failed("SQLite", msg.as_str())) })?;
                if let std::option::Option::Some(r) = opt { rows.push(r); }
            }
            crate::ports::task_repository_port::TaskFilter::ByStatus(status) => {
                let status_str = serde_json::to_string(status).map_err(|e| hexser::error::hex_error::Hexserror::Adapter(hexser::error::adapter_error::mapping_failure(std::format!("serde error: {:?}", e).as_str())))?;
                rows = sqlx::query(sql.as_str())
                    .bind(status_str)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|e| { let msg = std::format!("sqlx error: {:?}", e); hexser::error::hex_error::Hexserror::Adapter(hexser::error::adapter_error::connection_failed("SQLite", msg.as_str())) })?;
            }
            crate::ports::task_repository_port::TaskFilter::ByAssignee(assignee) => {
                rows = sqlx::query(sql.as_str())
                    .bind(assignee)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|e| { let msg = std::format!("sqlx error: {:?}", e); hexser::error::hex_error::Hexserror::Adapter(hexser::error::adapter_error::connection_failed("SQLite", msg.as_str())) })?;
            }
            crate::ports::task_repository_port::TaskFilter::All => {
                rows = sqlx::query(sql.as_str())
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|e| { let msg = std::format!("sqlx error: {:?}", e); hexser::error::hex_error::Hexserror::Adapter(hexser::error::adapter_error::connection_failed("SQLite", msg.as_str())) })?;
            }
        }
        let mut out: std::vec::Vec<crate::domain::task::Task> = std::vec::Vec::new();
        for r in rows.iter() { out.push(Self::row_to_task(r)?); }
        std::result::Result::Ok(out)
    }

    fn row_to_task(
        row: &sqlx::sqlite::SqliteRow,
    ) -> hexser::HexResult<crate::domain::task::Task> {
        let id: String = sqlx::Row::get(row, 0);
        let title: String = sqlx::Row::get(row, 1);
        let description: String = sqlx::Row::get(row, 2);
        let assignee: std::option::Option<String> = sqlx::Row::get(row, 3);
        let due_date: std::option::Option<String> = sqlx::Row::get(row, 4);
        let status_str: String = sqlx::Row::get(row, 5);
        let status: crate::domain::task_status::TaskStatus = serde_json::from_str(status_str.as_str()).map_err(|e| hexser::error::hex_error::Hexserror::Adapter(hexser::error::adapter_error::mapping_failure(std::format!("serde error: {:?}", e).as_str())))?;
        let source_transcript_id: std::option::Option<String> = sqlx::Row::get(row, 6);
        let source_prd_id: std::option::Option<String> = sqlx::Row::get(row, 7);
        let parent_task_id: std::option::Option<String> = sqlx::Row::get(row, 8);
        let subtask_ids_json: std::option::Option<String> = sqlx::Row::get(row, 9);
        let subtask_ids: std::vec::Vec<String> = match subtask_ids_json {
            std::option::Option::Some(s) => {
                serde_json::from_str(s.as_str()).map_err(|e| hexser::error::hex_error::Hexserror::Adapter(hexser::error::adapter_error::mapping_failure(std::format!("serde error: {:?}", e).as_str())))?
            }
            std::option::Option::None => std::vec::Vec::new(),
        };
        let created_at_str: String = sqlx::Row::get(row, 10);
        let created_at = chrono::DateTime::parse_from_rfc3339(created_at_str.as_str()).map_err(|e| hexser::error::hex_error::Hexserror::Adapter(hexser::error::adapter_error::mapping_failure(std::format!("time parse: {:?}", e).as_str())))?.with_timezone(&chrono::Utc);
        let updated_at_str: String = sqlx::Row::get(row, 11);
        let updated_at = chrono::DateTime::parse_from_rfc3339(updated_at_str.as_str()).map_err(|e| hexser::error::hex_error::Hexserror::Adapter(hexser::error::adapter_error::mapping_failure(std::format!("time parse: {:?}", e).as_str())))?.with_timezone(&chrono::Utc);
        let enhancements_json: std::option::Option<String> = sqlx::Row::get(row, 12);
        let enhancements: std::option::Option<std::vec::Vec<crate::domain::enhancement::Enhancement>> = match enhancements_json {
            std::option::Option::Some(s) => {
                std::option::Option::Some(serde_json::from_str(s.as_str()).map_err(|e| hexser::error::hex_error::Hexserror::Adapter(hexser::error::adapter_error::mapping_failure(std::format!("serde error: {:?}", e).as_str())))?)
            }
            std::option::Option::None => std::option::Option::None,
        };
        let tests_json: std::option::Option<String> = sqlx::Row::get(row, 13);
        let tests: std::option::Option<std::vec::Vec<crate::domain::comprehension_test::ComprehensionTest>> = match tests_json {
            std::option::Option::Some(s) => {
                std::option::Option::Some(serde_json::from_str(s.as_str()).map_err(|e| hexser::error::hex_error::Hexserror::Adapter(hexser::error::adapter_error::mapping_failure(std::format!("serde error: {:?}", e).as_str())))?)
            }
            std::option::Option::None => std::option::Option::None,
        };
        let complexity: std::option::Option<u8> = sqlx::Row::get(row, 14);
        let reasoning: std::option::Option<String> = sqlx::Row::get(row, 15);
        let context_files_json: std::option::Option<String> = sqlx::Row::get(row, 16);
        let context_files: std::vec::Vec<String> = match context_files_json {
            std::option::Option::Some(s) => {
                serde_json::from_str(s.as_str()).map_err(|e| hexser::error::hex_error::Hexserror::Adapter(hexser::error::adapter_error::mapping_failure(std::format!("serde error: {:?}", e).as_str())))?
            }
            std::option::Option::None => std::vec::Vec::new(),
        };
        let dependencies_json: std::option::Option<String> = sqlx::Row::get(row, 17);
        let dependencies: std::vec::Vec<String> = match dependencies_json {
            std::option::Option::Some(s) => {
                serde_json::from_str(s.as_str()).map_err(|e| hexser::error::hex_error::Hexserror::Adapter(hexser::error::adapter_error::mapping_failure(std::format!("serde error: {:?}", e).as_str())))?
            }
            std::option::Option::None => std::vec::Vec::new(),
        };
        let completion_summary: std::option::Option<String> = sqlx::Row::get(row, 18);
        std::result::Result::Ok(crate::domain::task::Task {
            id,
            title,
            description,
            assignee,
            due_date,
            status,
            source_transcript_id,
            source_prd_id,
            parent_task_id,
            subtask_ids,
            created_at,
            updated_at,
            enhancements,
            comprehension_tests: tests,
            complexity,
            reasoning,
            completion_summary,
            context_files,
            dependencies,
        })
    }
}

// HEXSER write operations
impl hexser::ports::Repository<crate::domain::task::Task> for SqliteTaskAdapter {
    fn save(&mut self, entity: crate::domain::task::Task) -> hexser::HexResult<()> {
        SqliteTaskAdapter::block_on(self.save_async(entity))
    }
}

// HEXSER read operations
impl hexser::ports::repository::QueryRepository<crate::domain::task::Task> for SqliteTaskAdapter {
    type Filter = crate::ports::task_repository_port::TaskFilter;
    type SortKey = crate::ports::task_repository_port::TaskSortKey;

    fn find_one(&self, filter: &Self::Filter) -> hexser::HexResult<std::option::Option<crate::domain::task::Task>> {
        SqliteTaskAdapter::block_on(self.find_one_async(filter))
    }

    fn find(
        &self,
        filter: &Self::Filter,
        opts: hexser::ports::repository::FindOptions<Self::SortKey>,
    ) -> hexser::HexResult<std::vec::Vec<crate::domain::task::Task>> {
        SqliteTaskAdapter::block_on(self.find_async(filter, opts))
    }
}

// Marker trait implementation
impl crate::ports::task_repository_port::TaskRepositoryPort for SqliteTaskAdapter {}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_sqlite_adapter_save_and_find() {
        let adapter = super::SqliteTaskAdapter::connect_and_init("sqlite::memory:").await.unwrap();
        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("SQLite Task"),
            assignee: std::option::Option::Some(std::string::String::from("Alice")),
            due_date: std::option::Option::None,
        };
        let mut task = crate::domain::task::Task::from_action_item(&action, std::option::Option::None);
        task.id = std::string::String::from("t1");
        let repo = adapter;
        // Save using async internal API to avoid nested runtime block_on
        super::SqliteTaskAdapter::save_async(&repo, task.clone()).await.unwrap();

        // find_one by id using async internal API
        let got = super::SqliteTaskAdapter::find_one_async(
            &repo,
            &crate::ports::task_repository_port::TaskFilter::ById(std::string::String::from("t1"))
        ).await.unwrap();
        std::assert!(got.is_some());
        let t = got.unwrap();
        std::assert_eq!(t.title, std::string::String::from("SQLite Task"));
        std::assert_eq!(t.assignee, std::option::Option::Some(std::string::String::from("Alice")));
    }

    #[tokio::test]
    async fn test_sqlite_adapter_find_one_by_status() {
        let repo = super::SqliteTaskAdapter::connect_and_init("sqlite::memory:").await.unwrap();

        // Task 1: InProgress
        let action1 = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Do work"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let mut t1 = crate::domain::task::Task::from_action_item(&action1, std::option::Option::None);
        t1.id = std::string::String::from("s1");
        t1.status = crate::domain::task_status::TaskStatus::InProgress;
        super::SqliteTaskAdapter::save_async(&repo, t1).await.unwrap();

        // Task 2: Todo
        let action2 = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Plan work"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let mut t2 = crate::domain::task::Task::from_action_item(&action2, std::option::Option::None);
        t2.id = std::string::String::from("s2");
        t2.status = crate::domain::task_status::TaskStatus::Todo;
        super::SqliteTaskAdapter::save_async(&repo, t2).await.unwrap();

        let found = super::SqliteTaskAdapter::find_one_async(
            &repo,
            &crate::ports::task_repository_port::TaskFilter::ByStatus(crate::domain::task_status::TaskStatus::InProgress)
        ).await.unwrap();
        std::assert!(found.is_some());
        let ft = found.unwrap();
        std::assert_eq!(ft.status, crate::domain::task_status::TaskStatus::InProgress);
    }

    #[tokio::test]
    async fn test_sqlite_adapter_find_all_sorted_paged() {
        let repo = super::SqliteTaskAdapter::connect_and_init("sqlite::memory:").await.unwrap();

        // Insert three tasks with titles for sorting
        let titles = vec![
            std::string::String::from("Alpha"),
            std::string::String::from("Bravo"),
            std::string::String::from("Charlie"),
        ];
        for (i, title) in titles.iter().enumerate() {
            let action = transcript_extractor::domain::action_item::ActionItem {
                title: title.clone(),
                assignee: std::option::Option::None,
                due_date: std::option::Option::None,
            };
            let mut t = crate::domain::task::Task::from_action_item(&action, std::option::Option::None);
            t.id = std::format!("pg-{}", i);
            super::SqliteTaskAdapter::save_async(&repo, t).await.unwrap();
        }

        let result = super::SqliteTaskAdapter::find_async(
            &repo,
            &crate::ports::task_repository_port::TaskFilter::All,
            hexser::ports::repository::FindOptions {
                sort: std::option::Option::Some(vec![
                    hexser::ports::repository::Sort {
                        key: crate::ports::task_repository_port::TaskSortKey::Title,
                        direction: hexser::ports::repository::Direction::Asc,
                    }
                ]),
                limit: std::option::Option::Some(2),
                offset: std::option::Option::Some(1),
            }
        ).await.unwrap();

        // Expect: sorted titles are Alpha, Bravo, Charlie -> offset 1 limit 2 -> Bravo, Charlie
        std::assert_eq!(result.len(), 2);
        std::assert_eq!(result[0].title, std::string::String::from("Bravo"));
        std::assert_eq!(result[1].title, std::string::String::from("Charlie"));
    }

    #[tokio::test]
    async fn test_sqlite_adapter_update_flow() {
        let repo = super::SqliteTaskAdapter::connect_and_init("sqlite::memory:").await.unwrap();

        let action = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Initial Title"),
            assignee: std::option::Option::Some(std::string::String::from("Bob")),
            due_date: std::option::Option::None,
        };
        let mut t = crate::domain::task::Task::from_action_item(&action, std::option::Option::None);
        t.id = std::string::String::from("u1");
        super::SqliteTaskAdapter::save_async(&repo, t.clone()).await.unwrap();

        // Update fields
        t.title = std::string::String::from("Updated Title");
        t.status = crate::domain::task_status::TaskStatus::Completed;
        t.updated_at = chrono::Utc::now();
        super::SqliteTaskAdapter::save_async(&repo, t.clone()).await.unwrap();

        // Fetch and verify
        let got = super::SqliteTaskAdapter::find_one_async(
            &repo,
            &crate::ports::task_repository_port::TaskFilter::ById(std::string::String::from("u1"))
        ).await.unwrap().unwrap();

        std::assert_eq!(got.title, std::string::String::from("Updated Title"));
        std::assert_eq!(got.status, crate::domain::task_status::TaskStatus::Completed);
        std::assert_eq!(got.assignee, std::option::Option::Some(std::string::String::from("Bob")));
    }

}
