//! SQLite-backed project repository adapter.
//!
//! This adapter implements HEXSER Repository and QueryRepository traits over
//! a SQLite database using `sqlx`. It adheres to the project's standards:
//! - No `use` statements (fully qualified paths only)
//! - One logical item per file (struct with inherent impls and trait impls)
//! - File-level docs and in-file tests
//! - No `unsafe`
//!
//! Data model is persisted in a `projects` table with JSON encoding for the
//! prd_ids list field.
//!
//! Revision History
//! - 2025-11-30T20:00:00Z @AI: Add ALTER TABLE migration for prd_ids_json column. Handles databases created by SqliteTaskAdapter that don't have this column.
//! - 2025-11-24T05:00:00Z @AI: Initial SqliteProjectAdapter implementation for Phase 1 TUI project architecture.

/// SQLite-backed implementation of the Project repository ports.
#[derive(hexser::HexAdapter)]
pub struct SqliteProjectAdapter {
    pool: sqlx::Pool<sqlx::Sqlite>,
}

impl SqliteProjectAdapter {
    /// Creates a new adapter from an existing SQLite pool.
    pub fn new(pool: sqlx::Pool<sqlx::Sqlite>) -> Self {
        SqliteProjectAdapter { pool }
    }

    /// Asynchronously connects to the provided database URL and ensures the schema exists.
    pub async fn connect_and_init(database_url: &str) -> std::result::Result<Self, std::string::String> {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect(database_url)
            .await
            .map_err(|e| std::format!("Failed to connect SQLite: {:?}", e))?;

        // Ensure projects table exists
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NULL,
                created_at TEXT NOT NULL,
                prd_ids_json TEXT NULL
            )"
        )
        .execute(&pool)
        .await
        .map_err(|e| std::format!("Failed to create projects schema: {:?}", e))?;

        // Migration: Add prd_ids_json column if missing (for databases created by SqliteTaskAdapter)
        let _ = sqlx::query("ALTER TABLE projects ADD COLUMN prd_ids_json TEXT NULL")
            .execute(&pool)
            .await; // Ignore error if column already exists

        std::result::Result::Ok(SqliteProjectAdapter { pool })
    }

    fn block_on<T>(fut: impl std::future::Future<Output = T>) -> T {
        // Build a current-thread runtime for synchronous trait methods.
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime");
        rt.block_on(fut)
    }

    pub async fn save_async(&self, entity: crate::domain::project::Project) -> hexser::HexResult<()> {
        let prd_ids_json = if entity.prd_ids.is_empty() {
            std::option::Option::None
        } else {
            std::option::Option::Some(serde_json::to_string(&entity.prd_ids).map_err(|e| {
                hexser::error::hex_error::Hexserror::Adapter(
                    hexser::error::adapter_error::mapping_failure(std::format!("Failed to serialize prd_ids to JSON: {:?}", e).as_str())
                )
            })?)
        };

        let created_at = entity.created_at.to_rfc3339();

        sqlx::query(
            "INSERT INTO projects (id, name, description, created_at, prd_ids_json)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
               name=excluded.name, description=excluded.description,
               created_at=excluded.created_at, prd_ids_json=excluded.prd_ids_json"
        )
        .bind(entity.id)
        .bind(entity.name)
        .bind(entity.description)
        .bind(created_at)
        .bind(prd_ids_json)
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

    fn row_to_project(row: sqlx::sqlite::SqliteRow) -> hexser::HexResult<crate::domain::project::Project> {
        let id: String = sqlx::Row::get(&row, "id");
        let name: String = sqlx::Row::get(&row, "name");
        let description: Option<String> = sqlx::Row::get(&row, "description");
        let created_at_str: String = sqlx::Row::get(&row, "created_at");
        let prd_ids_json_opt: Option<String> = sqlx::Row::get(&row, "prd_ids_json");

        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| {
                hexser::error::hex_error::Hexserror::Adapter(
                    hexser::error::adapter_error::mapping_failure(
                        std::format!("Failed to parse created_at: {:?}", e).as_str()
                    )
                )
            })?
            .with_timezone(&chrono::Utc);

        let prd_ids = match prd_ids_json_opt {
            std::option::Option::Some(json_str) => {
                serde_json::from_str(&json_str).map_err(|e| {
                    hexser::error::hex_error::Hexserror::Adapter(
                        hexser::error::adapter_error::mapping_failure(
                            std::format!("Failed to deserialize prd_ids: {:?}", e).as_str()
                        )
                    )
                })?
            }
            std::option::Option::None => std::vec::Vec::new(),
        };

        std::result::Result::Ok(crate::domain::project::Project {
            id,
            name,
            description,
            created_at,
            prd_ids,
        })
    }

    async fn find_one_async(
        &self,
        filter: &crate::ports::project_repository_port::ProjectFilter,
    ) -> hexser::HexResult<std::option::Option<crate::domain::project::Project>> {
        match filter {
            crate::ports::project_repository_port::ProjectFilter::ById(id) => {
                let row = sqlx::query("SELECT id, name, description, created_at, prd_ids_json FROM projects WHERE id = ?1")
                    .bind(id)
                    .fetch_optional(&self.pool)
                    .await
                    .map_err(|e| {
                        hexser::error::hex_error::Hexserror::Adapter(
                            hexser::error::adapter_error::connection_failed(
                                "SQLite",
                                std::format!("find_one query failed: {:?}", e).as_str()
                            )
                        )
                    })?;

                if let std::option::Option::Some(r) = row {
                    std::result::Result::Ok(std::option::Option::Some(Self::row_to_project(r)?))
                } else {
                    std::result::Result::Ok(std::option::Option::None)
                }
            }
            _ => {
                // For other filters, use find and return first result
                let results = self.find_async(filter, hexser::ports::repository::FindOptions::default()).await?;
                std::result::Result::Ok(results.into_iter().next())
            }
        }
    }

    pub async fn find_async(
        &self,
        filter: &crate::ports::project_repository_port::ProjectFilter,
        opts: hexser::ports::repository::FindOptions<crate::ports::project_repository_port::ProjectSortKey>,
    ) -> hexser::HexResult<std::vec::Vec<crate::domain::project::Project>> {
        let mut query_str = String::from("SELECT id, name, description, created_at, prd_ids_json FROM projects");
        let mut bind_values: std::vec::Vec<String> = std::vec::Vec::new();

        match filter {
            crate::ports::project_repository_port::ProjectFilter::ById(id) => {
                query_str.push_str(" WHERE id = ?1");
                bind_values.push(id.clone());
            }
            crate::ports::project_repository_port::ProjectFilter::ByName(name) => {
                query_str.push_str(" WHERE name = ?1");
                bind_values.push(name.clone());
            }
            crate::ports::project_repository_port::ProjectFilter::All => {}
        }

        // ORDER BY
        if let std::option::Option::Some(sort_specs) = opts.sort {
            let mut parts: std::vec::Vec<String> = std::vec::Vec::new();
            for s in sort_specs.iter() {
                let col = match &s.key {
                    crate::ports::project_repository_port::ProjectSortKey::CreatedAt => "created_at",
                    crate::ports::project_repository_port::ProjectSortKey::Name => "name",
                };
                let dir = if s.direction == hexser::ports::repository::Direction::Desc { "DESC" } else { "ASC" };
                parts.push(std::format!("{} {}", col, dir));
            }
            if !parts.is_empty() {
                query_str.push_str(" ORDER BY ");
                query_str.push_str(parts.join(", ").as_str());
            }
        }

        if let std::option::Option::Some(limit) = opts.limit {
            query_str.push_str(&std::format!(" LIMIT {}", limit));
        }

        if let std::option::Option::Some(offset) = opts.offset {
            query_str.push_str(&std::format!(" OFFSET {}", offset));
        }

        let mut query = sqlx::query(&query_str);
        for val in bind_values {
            query = query.bind(val);
        }

        let rows = query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                hexser::error::hex_error::Hexserror::Adapter(
                    hexser::error::adapter_error::connection_failed(
                        "SQLite",
                        std::format!("find query failed: {:?}", e).as_str()
                    )
                )
            })?;

        rows.into_iter()
            .map(|row| Self::row_to_project(row))
            .collect()
    }

    /// Deletes a project by ID (custom method, not part of HEXSER Repository).
    pub async fn delete_async(&self, id: &str) -> hexser::HexResult<()> {
        sqlx::query("DELETE FROM projects WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                hexser::error::hex_error::Hexserror::Adapter(
                    hexser::error::adapter_error::connection_failed(
                        "SQLite",
                        std::format!("delete failed: {:?}", e).as_str()
                    )
                )
            })?;
        std::result::Result::Ok(())
    }

    /// Synchronous delete method.
    pub fn delete(&self, id: &str) -> hexser::HexResult<()> {
        Self::block_on(self.delete_async(id))
    }
}

impl hexser::ports::Repository<crate::domain::project::Project> for SqliteProjectAdapter {
    fn save(&mut self, entity: crate::domain::project::Project) -> hexser::HexResult<()> {
        Self::block_on(self.save_async(entity))
    }
}

impl hexser::ports::repository::QueryRepository<crate::domain::project::Project> for SqliteProjectAdapter {
    type Filter = crate::ports::project_repository_port::ProjectFilter;
    type SortKey = crate::ports::project_repository_port::ProjectSortKey;

    fn find_one(&self, filter: &Self::Filter) -> hexser::HexResult<std::option::Option<crate::domain::project::Project>> {
        Self::block_on(self.find_one_async(filter))
    }

    fn find(
        &self,
        filter: &Self::Filter,
        opts: hexser::ports::repository::FindOptions<Self::SortKey>,
    ) -> hexser::HexResult<std::vec::Vec<crate::domain::project::Project>> {
        Self::block_on(self.find_async(filter, opts))
    }
}

impl crate::ports::project_repository_port::ProjectRepositoryPort for SqliteProjectAdapter {}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_db() -> SqliteProjectAdapter {
        SqliteProjectAdapter::connect_and_init(":memory:")
            .await
            .expect("Failed to create in-memory database")
    }

    #[tokio::test]
    async fn test_save_and_find_one() {
        // Test: Validates basic save and retrieval operations.
        // Justification: Core repository functionality must persist and retrieve entities.
        let adapter = setup_test_db().await;
        let project = crate::domain::project::Project::new(
            String::from("test-project"),
            std::option::Option::Some(String::from("Test description")),
        );
        let project_id = project.id.clone();

        adapter.save_async(project.clone()).await.expect("Failed to save project");

        let filter = crate::ports::project_repository_port::ProjectFilter::ById(project_id.clone());
        let found = super::SqliteProjectAdapter::find_one_async(&adapter, &filter).await.expect("Failed to find project");
        std::assert!(found.is_some());

        let found_project = found.unwrap();
        std::assert_eq!(found_project.id, project_id);
        std::assert_eq!(found_project.name, "test-project");
        std::assert_eq!(found_project.description, std::option::Option::Some(String::from("Test description")));
    }

    #[tokio::test]
    async fn test_find_by_name() {
        // Test: Validates filtering by project name.
        // Justification: Users need to locate projects by name.
        let adapter = setup_test_db().await;

        let project1 = crate::domain::project::Project::new(
            String::from("alpha"),
            std::option::Option::None,
        );
        let project2 = crate::domain::project::Project::new(
            String::from("beta"),
            std::option::Option::None,
        );

        adapter.save_async(project1).await.expect("Failed to save project1");
        adapter.save_async(project2).await.expect("Failed to save project2");

        let filter = crate::ports::project_repository_port::ProjectFilter::ByName(String::from("beta"));
        let results = adapter.find_async(&filter, hexser::ports::repository::FindOptions::default())
            .await
            .expect("Failed to find projects");

        std::assert_eq!(results.len(), 1);
        std::assert_eq!(results[0].name, "beta");
    }

    #[tokio::test]
    async fn test_find_all_projects() {
        // Test: Validates retrieving all projects.
        // Justification: UI needs to list all projects.
        let adapter = setup_test_db().await;

        let project1 = crate::domain::project::Project::new(String::from("project-a"), std::option::Option::None);
        let project2 = crate::domain::project::Project::new(String::from("project-b"), std::option::Option::None);
        let project3 = crate::domain::project::Project::new(String::from("project-c"), std::option::Option::None);

        adapter.save_async(project1).await.expect("Failed to save");
        adapter.save_async(project2).await.expect("Failed to save");
        adapter.save_async(project3).await.expect("Failed to save");

        let results = adapter.find_async(
            &crate::ports::project_repository_port::ProjectFilter::All,
            hexser::ports::repository::FindOptions::default()
        )
        .await
        .expect("Failed to find projects");

        std::assert_eq!(results.len(), 3);
        // Verify all projects are returned (order not guaranteed without sort)
        std::assert!(results.iter().any(|p| p.name == "project-a"));
        std::assert!(results.iter().any(|p| p.name == "project-b"));
        std::assert!(results.iter().any(|p| p.name == "project-c"));
    }

    #[tokio::test]
    async fn test_update_existing_project() {
        // Test: Validates that save updates existing projects.
        // Justification: Projects need to be editable.
        let adapter = setup_test_db().await;

        let mut project = crate::domain::project::Project::new(
            String::from("original-name"),
            std::option::Option::None,
        );
        let project_id = project.id.clone();

        adapter.save_async(project.clone()).await.expect("Failed to save");

        // Update the project
        project.name = String::from("updated-name");
        project.description = std::option::Option::Some(String::from("New description"));

        adapter.save_async(project).await.expect("Failed to update");

        let filter = crate::ports::project_repository_port::ProjectFilter::ById(project_id);
        let found = super::SqliteProjectAdapter::find_one_async(&adapter, &filter).await.expect("Failed to find").expect("Project not found");
        std::assert_eq!(found.name, "updated-name");
        std::assert_eq!(found.description, std::option::Option::Some(String::from("New description")));
    }

    #[tokio::test]
    async fn test_delete_project() {
        // Test: Validates project deletion.
        // Justification: Users need to remove obsolete projects.
        let adapter = setup_test_db().await;

        let project = crate::domain::project::Project::new(
            String::from("to-delete"),
            std::option::Option::None,
        );
        let project_id = project.id.clone();

        adapter.save_async(project).await.expect("Failed to save");

        let filter1 = crate::ports::project_repository_port::ProjectFilter::ById(project_id.clone());
        std::assert!(super::SqliteProjectAdapter::find_one_async(&adapter, &filter1).await.expect("Find failed").is_some());

        adapter.delete_async(&project_id).await.expect("Failed to delete");

        let filter2 = crate::ports::project_repository_port::ProjectFilter::ById(project_id);
        std::assert!(super::SqliteProjectAdapter::find_one_async(&adapter, &filter2).await.expect("Find failed").is_none());
    }

    #[tokio::test]
    async fn test_prd_ids_persistence() {
        // Test: Validates that prd_ids list is correctly persisted and retrieved.
        // Justification: Projects must track their associated PRDs.
        let adapter = setup_test_db().await;

        let mut project = crate::domain::project::Project::new(
            String::from("with-prds"),
            std::option::Option::None,
        );
        project.add_prd(String::from("prd-001"));
        project.add_prd(String::from("prd-002"));
        project.add_prd(String::from("prd-003"));

        let project_id = project.id.clone();
        adapter.save_async(project).await.expect("Failed to save");

        let filter = crate::ports::project_repository_port::ProjectFilter::ById(project_id);
        let found = super::SqliteProjectAdapter::find_one_async(&adapter, &filter).await.expect("Find failed").expect("Project not found");
        std::assert_eq!(found.prd_ids.len(), 3);
        std::assert_eq!(found.prd_ids[0], "prd-001");
        std::assert_eq!(found.prd_ids[1], "prd-002");
        std::assert_eq!(found.prd_ids[2], "prd-003");
    }
}
