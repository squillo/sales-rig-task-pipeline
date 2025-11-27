//! SQLite-backed agent tool repository adapter implementation.
//!
//! This module implements the AgentToolRepositoryPort for SqliteTaskAdapter.
//! It provides CRUD operations for agent tools using the HEXSER Repository pattern.
//! All operations use the agent_tools table created during connect_and_init().
//!
//! Revision History
//! - 2025-11-26T08:00:00Z @AI: Initial AgentTool repository implementation for Phase 3 persona management.

impl hexser::ports::Repository<crate::domain::agent_tool::AgentTool> for crate::adapters::sqlite_task_adapter::SqliteTaskAdapter {
    fn save(&mut self, entity: crate::domain::agent_tool::AgentTool) -> hexser::HexResult<()> {
        crate::adapters::sqlite_task_adapter::SqliteTaskAdapter::block_on(async {
            sqlx::query(
                "INSERT INTO agent_tools (id, name, description, category, risk_level, is_default)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(id) DO UPDATE SET
                   name=excluded.name,
                   description=excluded.description,
                   category=excluded.category,
                   risk_level=excluded.risk_level,
                   is_default=excluded.is_default"
            )
            .bind(&entity.id)
            .bind(&entity.name)
            .bind(&entity.description)
            .bind(entity.category.display_name())
            .bind(entity.risk_level.display_name())
            .bind(entity.is_default)
            .execute(self.pool())
            .await
            .map_err(|e| {
                hexser::error::hex_error::Hexserror::Adapter(
                    hexser::error::adapter_error::connection_failed(
                        "SQLite",
                        std::format!("Failed to save agent_tool: {:?}", e).as_str()
                    )
                )
            })?;
            std::result::Result::Ok(())
        })
    }
}

impl hexser::ports::repository::QueryRepository<crate::domain::agent_tool::AgentTool> for crate::adapters::sqlite_task_adapter::SqliteTaskAdapter {
    type Filter = crate::ports::agent_tool_repository_port::ToolFilter;
    type SortKey = crate::ports::agent_tool_repository_port::ToolSortKey;

    fn find_one(&self, filter: &Self::Filter) -> hexser::HexResult<std::option::Option<crate::domain::agent_tool::AgentTool>> {
        crate::adapters::sqlite_task_adapter::SqliteTaskAdapter::block_on(async {
            let (where_clause, bind_value) = match filter {
                crate::ports::agent_tool_repository_port::ToolFilter::ById(id) => ("id = ?1", std::option::Option::Some(id.clone())),
                crate::ports::agent_tool_repository_port::ToolFilter::ByCategory(cat) => ("category = ?1", std::option::Option::Some(String::from(cat.display_name()))),
                crate::ports::agent_tool_repository_port::ToolFilter::ByRiskLevel(risk) => ("risk_level = ?1", std::option::Option::Some(String::from(risk.display_name()))),
                crate::ports::agent_tool_repository_port::ToolFilter::DefaultOnly => ("is_default = 1", std::option::Option::None),
                crate::ports::agent_tool_repository_port::ToolFilter::All => ("1=1", std::option::Option::None),
            };

            let sql = std::format!("SELECT id, name, description, category, risk_level, is_default FROM agent_tools WHERE {} LIMIT 1", where_clause);

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
                            std::format!("Failed to query agent_tools: {:?}", e).as_str()
                        )
                    )
                })?;

            if let std::option::Option::Some(r) = row {
                let id: String = sqlx::Row::get(&r, 0);
                let name: String = sqlx::Row::get(&r, 1);
                let description: String = sqlx::Row::get(&r, 2);
                let category_str: String = sqlx::Row::get(&r, 3);
                let risk_level_str: String = sqlx::Row::get(&r, 4);
                let is_default: bool = sqlx::Row::get(&r, 5);

                let category = parse_category(&category_str)?;
                let risk_level = parse_risk_level(&risk_level_str)?;

                std::result::Result::Ok(std::option::Option::Some(crate::domain::agent_tool::AgentTool {
                    id,
                    name,
                    description,
                    category,
                    risk_level,
                    is_default,
                }))
            } else {
                std::result::Result::Ok(std::option::Option::None)
            }
        })
    }

    fn find(
        &self,
        filter: &Self::Filter,
        opts: hexser::ports::repository::FindOptions<Self::SortKey>,
    ) -> hexser::HexResult<std::vec::Vec<crate::domain::agent_tool::AgentTool>> {
        crate::adapters::sqlite_task_adapter::SqliteTaskAdapter::block_on(async {
            let (where_clause, bind_value) = match filter {
                crate::ports::agent_tool_repository_port::ToolFilter::ById(id) => ("id = ?1", std::option::Option::Some(id.clone())),
                crate::ports::agent_tool_repository_port::ToolFilter::ByCategory(cat) => ("category = ?1", std::option::Option::Some(String::from(cat.display_name()))),
                crate::ports::agent_tool_repository_port::ToolFilter::ByRiskLevel(risk) => ("risk_level = ?1", std::option::Option::Some(String::from(risk.display_name()))),
                crate::ports::agent_tool_repository_port::ToolFilter::DefaultOnly => ("is_default = 1", std::option::Option::None),
                crate::ports::agent_tool_repository_port::ToolFilter::All => ("1=1", std::option::Option::None),
            };

            let mut sql = std::format!("SELECT id, name, description, category, risk_level, is_default FROM agent_tools WHERE {}", where_clause);

            // Add ORDER BY if sort keys provided
            if let std::option::Option::Some(sort_keys) = &opts.sort {
                if !sort_keys.is_empty() {
                    let order_clauses: std::vec::Vec<String> = sort_keys.iter().map(|s| {
                        let col = match s.key {
                            crate::ports::agent_tool_repository_port::ToolSortKey::Id => "id",
                            crate::ports::agent_tool_repository_port::ToolSortKey::Name => "name",
                            crate::ports::agent_tool_repository_port::ToolSortKey::Category => "category",
                            crate::ports::agent_tool_repository_port::ToolSortKey::RiskLevel => "risk_level",
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
                            std::format!("Failed to query agent_tools: {:?}", e).as_str()
                        )
                    )
                })?;

            let mut tools = std::vec::Vec::new();
            for r in rows {
                let id: String = sqlx::Row::get(&r, 0);
                let name: String = sqlx::Row::get(&r, 1);
                let description: String = sqlx::Row::get(&r, 2);
                let category_str: String = sqlx::Row::get(&r, 3);
                let risk_level_str: String = sqlx::Row::get(&r, 4);
                let is_default: bool = sqlx::Row::get(&r, 5);

                let category = parse_category(&category_str)?;
                let risk_level = parse_risk_level(&risk_level_str)?;

                tools.push(crate::domain::agent_tool::AgentTool {
                    id,
                    name,
                    description,
                    category,
                    risk_level,
                    is_default,
                });
            }

            std::result::Result::Ok(tools)
        })
    }
}

impl crate::ports::agent_tool_repository_port::AgentToolRepositoryPort for crate::adapters::sqlite_task_adapter::SqliteTaskAdapter {}

/// Parses a category string into ToolCategory enum.
fn parse_category(s: &str) -> hexser::HexResult<crate::domain::agent_tool::ToolCategory> {
    match s {
        "Development" => std::result::Result::Ok(crate::domain::agent_tool::ToolCategory::Development),
        "Research" => std::result::Result::Ok(crate::domain::agent_tool::ToolCategory::Research),
        "FileSystem" => std::result::Result::Ok(crate::domain::agent_tool::ToolCategory::FileSystem),
        "Network" => std::result::Result::Ok(crate::domain::agent_tool::ToolCategory::Network),
        "Database" => std::result::Result::Ok(crate::domain::agent_tool::ToolCategory::Database),
        "Communication" => std::result::Result::Ok(crate::domain::agent_tool::ToolCategory::Communication),
        _ => std::result::Result::Err(hexser::error::hex_error::Hexserror::Adapter(
            hexser::error::adapter_error::mapping_failure(
                std::format!("Unknown category: {}", s).as_str()
            )
        )),
    }
}

/// Parses a risk level string into RiskLevel enum.
fn parse_risk_level(s: &str) -> hexser::HexResult<crate::domain::agent_tool::RiskLevel> {
    match s {
        "Safe" => std::result::Result::Ok(crate::domain::agent_tool::RiskLevel::Safe),
        "Moderate" => std::result::Result::Ok(crate::domain::agent_tool::RiskLevel::Moderate),
        "High" => std::result::Result::Ok(crate::domain::agent_tool::RiskLevel::High),
        _ => std::result::Result::Err(hexser::error::hex_error::Hexserror::Adapter(
            hexser::error::adapter_error::mapping_failure(
                std::format!("Unknown risk level: {}", s).as_str()
            )
        )),
    }
}
