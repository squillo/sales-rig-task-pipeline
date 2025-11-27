//! SQLite-backed persona repository adapter implementation.
//!
//! This module implements the PersonaRepositoryPort for SqliteTaskAdapter.
//! It provides CRUD operations for personas using the HEXSER Repository pattern,
//! including management of the persona_tools junction table for tool enablement.
//!
//! Revision History
//! - 2025-11-26T09:20:00Z @AI: Add project_id field to all persona SQL queries for project-scoped persona support.
//! - 2025-11-26T08:10:00Z @AI: Initial Persona repository implementation for Phase 3 persona management.

impl hexser::ports::Repository<crate::domain::persona::Persona> for crate::adapters::sqlite_task_adapter::SqliteTaskAdapter {
    fn save(&mut self, entity: crate::domain::persona::Persona) -> hexser::HexResult<()> {
        crate::adapters::sqlite_task_adapter::SqliteTaskAdapter::block_on(async {
            let created_at = entity.created_at.to_rfc3339();
            let updated_at = entity.updated_at.to_rfc3339();

            // Save persona entity
            sqlx::query(
                "INSERT INTO personas (id, project_id, name, role, description, llm_provider, llm_model, is_default, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                 ON CONFLICT(id) DO UPDATE SET
                   project_id=excluded.project_id,
                   name=excluded.name,
                   role=excluded.role,
                   description=excluded.description,
                   llm_provider=excluded.llm_provider,
                   llm_model=excluded.llm_model,
                   is_default=excluded.is_default,
                   updated_at=excluded.updated_at"
            )
            .bind(&entity.id)
            .bind(&entity.project_id)
            .bind(&entity.name)
            .bind(&entity.role)
            .bind(&entity.description)
            .bind(&entity.llm_provider)
            .bind(&entity.llm_model)
            .bind(entity.is_default)
            .bind(&created_at)
            .bind(&updated_at)
            .execute(self.pool())
            .await
            .map_err(|e| {
                hexser::error::hex_error::Hexserror::Adapter(
                    hexser::error::adapter_error::connection_failed(
                        "SQLite",
                        std::format!("Failed to save persona: {:?}", e).as_str()
                    )
                )
            })?;

            // Clear existing persona_tools entries for this persona
            sqlx::query("DELETE FROM persona_tools WHERE persona_id = ?1")
                .bind(&entity.id)
                .execute(self.pool())
                .await
                .map_err(|e| {
                    hexser::error::hex_error::Hexserror::Adapter(
                        hexser::error::adapter_error::connection_failed(
                            "SQLite",
                            std::format!("Failed to clear persona_tools: {:?}", e).as_str()
                        )
                    )
                })?;

            // Insert new persona_tools entries
            for tool_id in &entity.enabled_tools {
                sqlx::query(
                    "INSERT INTO persona_tools (persona_id, tool_id, enabled) VALUES (?1, ?2, 1)"
                )
                .bind(&entity.id)
                .bind(tool_id)
                .execute(self.pool())
                .await
                .map_err(|e| {
                    hexser::error::hex_error::Hexserror::Adapter(
                        hexser::error::adapter_error::connection_failed(
                            "SQLite",
                            std::format!("Failed to insert persona_tools: {:?}", e).as_str()
                        )
                    )
                })?;
            }

            std::result::Result::Ok(())
        })
    }
}

impl hexser::ports::repository::QueryRepository<crate::domain::persona::Persona> for crate::adapters::sqlite_task_adapter::SqliteTaskAdapter {
    type Filter = crate::ports::persona_repository_port::PersonaFilter;
    type SortKey = crate::ports::persona_repository_port::PersonaSortKey;

    fn find_one(&self, filter: &Self::Filter) -> hexser::HexResult<std::option::Option<crate::domain::persona::Persona>> {
        crate::adapters::sqlite_task_adapter::SqliteTaskAdapter::block_on(async {
            let (where_clause, bind_value) = match filter {
                crate::ports::persona_repository_port::PersonaFilter::ById(id) => ("id = ?1", std::option::Option::Some(id.clone())),
                crate::ports::persona_repository_port::PersonaFilter::ByName(name) => ("name = ?1", std::option::Option::Some(name.clone())),
                crate::ports::persona_repository_port::PersonaFilter::ByProject(project_id) => ("project_id = ?1", std::option::Option::Some(project_id.clone())),
                crate::ports::persona_repository_port::PersonaFilter::DefaultOnly => ("is_default = 1", std::option::Option::None),
                crate::ports::persona_repository_port::PersonaFilter::All => ("1=1", std::option::Option::None),
            };

            let sql = std::format!("SELECT id, project_id, name, role, description, llm_provider, llm_model, is_default, created_at, updated_at FROM personas WHERE {} LIMIT 1", where_clause);

            let mut query = sqlx::query(&sql);
            if let std::option::Option::Some(val) = bind_value {
                query = query.bind(val);
            }

            let row = query
                .fetch_optional(self.pool())
                .await
                .map_err(|e| {
                    hexser::error::hex_error::Hexserror::Adapter(
                        hexser::error::adapter_error::connection_failed(
                            "SQLite",
                            std::format!("Failed to query personas: {:?}", e).as_str()
                        )
                    )
                })?;

            if let std::option::Option::Some(r) = row {
                row_to_persona(&r, self.pool()).await
            } else {
                std::result::Result::Ok(std::option::Option::None)
            }
        })
    }

    fn find(
        &self,
        filter: &Self::Filter,
        opts: hexser::ports::repository::FindOptions<Self::SortKey>,
    ) -> hexser::HexResult<std::vec::Vec<crate::domain::persona::Persona>> {
        crate::adapters::sqlite_task_adapter::SqliteTaskAdapter::block_on(async {
            let (where_clause, bind_value) = match filter {
                crate::ports::persona_repository_port::PersonaFilter::ById(id) => ("id = ?1", std::option::Option::Some(id.clone())),
                crate::ports::persona_repository_port::PersonaFilter::ByName(name) => ("name = ?1", std::option::Option::Some(name.clone())),
                crate::ports::persona_repository_port::PersonaFilter::ByProject(project_id) => ("project_id = ?1", std::option::Option::Some(project_id.clone())),
                crate::ports::persona_repository_port::PersonaFilter::DefaultOnly => ("is_default = 1", std::option::Option::None),
                crate::ports::persona_repository_port::PersonaFilter::All => ("1=1", std::option::Option::None),
            };

            let mut sql = std::format!("SELECT id, project_id, name, role, description, llm_provider, llm_model, is_default, created_at, updated_at FROM personas WHERE {}", where_clause);

            // Add ORDER BY if sort keys provided
            if let std::option::Option::Some(sort_keys) = &opts.sort {
                if !sort_keys.is_empty() {
                    let order_clauses: std::vec::Vec<String> = sort_keys.iter().map(|s| {
                        let col = match s.key {
                            crate::ports::persona_repository_port::PersonaSortKey::Name => "name",
                            crate::ports::persona_repository_port::PersonaSortKey::Role => "role",
                            crate::ports::persona_repository_port::PersonaSortKey::CreatedAt => "created_at",
                            crate::ports::persona_repository_port::PersonaSortKey::UpdatedAt => "updated_at",
                        };
                        let dir = match s.direction {
                            hexser::ports::repository::Direction::Asc => "ASC",
                            hexser::ports::repository::Direction::Desc => "DESC",
                        };
                        std::format!("{} {}", col, dir)
                    }).collect();
                    sql.push_str(" ORDER BY ");
                    sql.push_str(&order_clauses.join(", "));
                }
            }

            // Add LIMIT/OFFSET if provided
            if let std::option::Option::Some(limit) = opts.limit {
                sql.push_str(&std::format!(" LIMIT {}", limit));
            }
            if let std::option::Option::Some(offset) = opts.offset {
                sql.push_str(&std::format!(" OFFSET {}", offset));
            }

            let mut query = sqlx::query(&sql);
            if let std::option::Option::Some(val) = bind_value {
                query = query.bind(val);
            }

            let rows = query
                .fetch_all(self.pool())
                .await
                .map_err(|e| {
                    hexser::error::hex_error::Hexserror::Adapter(
                        hexser::error::adapter_error::connection_failed(
                            "SQLite",
                            std::format!("Failed to query personas: {:?}", e).as_str()
                        )
                    )
                })?;

            let mut personas = std::vec::Vec::new();
            for r in rows {
                if let std::option::Option::Some(persona) = row_to_persona(&r, self.pool()).await? {
                    personas.push(persona);
                }
            }

            std::result::Result::Ok(personas)
        })
    }
}

impl crate::ports::persona_repository_port::PersonaRepositoryPort for crate::adapters::sqlite_task_adapter::SqliteTaskAdapter {
    fn find_default(&mut self) -> Result<std::option::Option<crate::domain::persona::Persona>, String> {
        let filter = crate::ports::persona_repository_port::PersonaFilter::DefaultOnly;
        <Self as hexser::ports::repository::QueryRepository<crate::domain::persona::Persona>>::find_one(self, &filter)
            .map_err(|e| std::format!("{:?}", e))
    }

    fn set_default(&mut self, persona_id: &str) -> Result<(), String> {
        crate::adapters::sqlite_task_adapter::SqliteTaskAdapter::block_on(async {
            // Clear all defaults first
            sqlx::query("UPDATE personas SET is_default = 0")
                .execute(self.pool())
                .await
                .map_err(|e| std::format!("Failed to clear defaults: {:?}", e))?;

            // Set the specified persona as default
            let rows_affected = sqlx::query("UPDATE personas SET is_default = 1 WHERE id = ?1")
                .bind(persona_id)
                .execute(self.pool())
                .await
                .map_err(|e| std::format!("Failed to set default: {:?}", e))?
                .rows_affected();

            if rows_affected == 0 {
                return std::result::Result::Err(std::format!("Persona with id '{}' not found", persona_id));
            }

            std::result::Result::Ok(())
        })
    }

    fn get_enabled_tools(&mut self, persona_id: &str) -> Result<std::vec::Vec<String>, String> {
        crate::adapters::sqlite_task_adapter::SqliteTaskAdapter::block_on(async {
            let rows = sqlx::query(
                "SELECT tool_id FROM persona_tools WHERE persona_id = ?1 AND enabled = 1 ORDER BY tool_id"
            )
            .bind(persona_id)
            .fetch_all(self.pool())
            .await
            .map_err(|e| std::format!("Failed to query persona_tools: {:?}", e))?;

            let tool_ids: std::vec::Vec<String> = rows.iter().map(|r| sqlx::Row::get(r, 0)).collect();
            std::result::Result::Ok(tool_ids)
        })
    }

    fn set_tool_enabled(&mut self, persona_id: &str, tool_id: &str, enabled: bool) -> Result<(), String> {
        crate::adapters::sqlite_task_adapter::SqliteTaskAdapter::block_on(async {
            // Insert or update the persona_tools entry
            sqlx::query(
                "INSERT INTO persona_tools (persona_id, tool_id, enabled) VALUES (?1, ?2, ?3)
                 ON CONFLICT(persona_id, tool_id) DO UPDATE SET enabled=excluded.enabled"
            )
            .bind(persona_id)
            .bind(tool_id)
            .bind(enabled)
            .execute(self.pool())
            .await
            .map_err(|e| std::format!("Failed to set tool enabled: {:?}", e))?;

            std::result::Result::Ok(())
        })
    }
}

/// Converts a SQLite row into a Persona entity, loading enabled tools from persona_tools junction table.
async fn row_to_persona(
    row: &sqlx::sqlite::SqliteRow,
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> hexser::HexResult<std::option::Option<crate::domain::persona::Persona>> {
    let id: String = sqlx::Row::get(row, 0);
    let project_id: std::option::Option<String> = sqlx::Row::get(row, 1);
    let name: String = sqlx::Row::get(row, 2);
    let role: String = sqlx::Row::get(row, 3);
    let description: String = sqlx::Row::get(row, 4);
    let llm_provider: std::option::Option<String> = sqlx::Row::get(row, 5);
    let llm_model: std::option::Option<String> = sqlx::Row::get(row, 6);
    let is_default: bool = sqlx::Row::get(row, 7);
    let created_at_str: String = sqlx::Row::get(row, 8);
    let updated_at_str: String = sqlx::Row::get(row, 9);

    let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
        .map_err(|e| {
            hexser::error::hex_error::Hexserror::Adapter(
                hexser::error::adapter_error::mapping_failure(
                    std::format!("Failed to parse created_at: {:?}", e).as_str()
                )
            )
        })?
        .with_timezone(&chrono::Utc);

    let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
        .map_err(|e| {
            hexser::error::hex_error::Hexserror::Adapter(
                hexser::error::adapter_error::mapping_failure(
                    std::format!("Failed to parse updated_at: {:?}", e).as_str()
                )
            )
        })?
        .with_timezone(&chrono::Utc);

    // Load enabled tools from persona_tools junction table
    let tool_rows = sqlx::query("SELECT tool_id FROM persona_tools WHERE persona_id = ?1 AND enabled = 1 ORDER BY tool_id")
        .bind(&id)
        .fetch_all(pool)
        .await
        .map_err(|e| {
            hexser::error::hex_error::Hexserror::Adapter(
                hexser::error::adapter_error::connection_failed(
                    "SQLite",
                    std::format!("Failed to load persona_tools: {:?}", e).as_str()
                )
            )
        })?;

    let enabled_tools: std::vec::Vec<String> = tool_rows.iter().map(|r| sqlx::Row::get(r, 0)).collect();

    std::result::Result::Ok(std::option::Option::Some(crate::domain::persona::Persona {
        id,
        project_id,
        name,
        role,
        description,
        llm_provider,
        llm_model,
        enabled_tools,
        is_default,
        created_at,
        updated_at,
    }))
}
