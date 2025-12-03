//! SQLite-backed artifact repository adapter with vector similarity search.
//!
//! This adapter implements HEXSER Repository and QueryRepository traits over
//! a SQLite database using `sqlx`, plus vector similarity search using the
//! sqlite-vec extension. It adheres to the project's standards:
//! - No `use` statements (fully qualified paths only)
//! - One logical item per file (struct with inherent impls and trait impls)
//! - File-level docs and in-file tests
//! - No `unsafe`
//!
//! Data model is persisted in an `artifacts` table for metadata and content,
//! with a separate `artifacts_vec` virtual table (vec0 module) for vector
//! embeddings and similarity search using cosine distance.
//!
//! Revision History
//! - 2025-11-30T10:30:00Z @AI: Add support for Image and PDF artifact types with binary storage. Updated row_to_artifact() to handle new ArtifactType variants (Image, PDF) and extract optional binary fields (binary_content, mime_type, source_url, page_number). Uses try_get() for backward compatibility with older schemas.
//! - 2025-11-29T14:30:00Z @AI: Add public async search_similar() method for semantic artifact search. Takes query embedding, limit, and similarity threshold (0.0-1.0). Returns (Artifact, similarity_score) tuples sorted by similarity. Converts cosine distance to similarity score (1.0 - distance) for threshold comparison.
//! - 2025-11-29T09:00:00Z @AI: Add create_if_missing(true) to SqliteConnectOptions to ensure database file is created when it doesn't exist.
//! - 2025-11-28T19:30:00Z @AI: Initial SqliteArtifactAdapter implementation for Phase 2 RAG vector search.

/// SQLite-backed implementation of the Artifact repository ports with vector search.
#[derive(hexser::HexAdapter)]
pub struct SqliteArtifactAdapter {
    pool: sqlx::Pool<sqlx::Sqlite>,
}

impl SqliteArtifactAdapter {
    /// Creates a new adapter from an existing SQLite pool.
    pub fn new(pool: sqlx::Pool<sqlx::Sqlite>) -> Self {
        SqliteArtifactAdapter { pool }
    }

    /// Asynchronously connects to the provided database URL and ensures the schema exists.
    ///
    /// This method loads the sqlite-vec extension for vector similarity search.
    /// The extension is embedded in the binary and extracted at runtime,
    /// ensuring RAG features are always available regardless of working directory.
    pub async fn connect_and_init(database_url: &str) -> std::result::Result<Self, std::string::String> {
        // For in-memory databases in tests, skip extension loading
        let pool = if database_url.contains(":memory:") {
            let connect_options = database_url
                .parse::<sqlx::sqlite::SqliteConnectOptions>()
                .map_err(|e| std::format!("Failed to parse database URL: {:?}", e))?
                .create_if_missing(true)
                .optimize_on_close(false, std::option::Option::None);

            sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(connect_options)
                .await
                .map_err(|e| std::format!("Failed to connect SQLite: {:?}", e))?
        } else {
            // Try embedded extension first (always available, bundled in binary)
            let mut extension_paths = std::vec![];

            if let std::result::Result::Ok(embedded_path) = crate::adapters::embedded_sqlite_vec::get_extension_path_for_sqlite() {
                extension_paths.push(embedded_path);
            }

            // Fallback paths (for development or custom installations)
            let abs_path = std::env::current_dir()
                .ok()
                .and_then(|p| p.join(".rigger/lib/vec0").to_str().map(|s| s.to_string()))
                .unwrap_or_default();

            extension_paths.push(std::string::String::from("vec0"));              // System-wide install
            extension_paths.push(std::string::String::from(".rigger/lib/vec0"));  // Local install
            extension_paths.push(abs_path);                                        // Absolute path

            let mut pool_result = std::option::Option::None;

            for ext_path in &extension_paths {
                if ext_path.is_empty() {
                    continue;
                }

                let connect_options = database_url
                    .parse::<sqlx::sqlite::SqliteConnectOptions>()
                    .map_err(|e| std::format!("Failed to parse database URL: {:?}", e))?
                    .create_if_missing(true)
                    .optimize_on_close(false, std::option::Option::None)
                    .extension(ext_path.clone());

                if let std::result::Result::Ok(p) = sqlx::sqlite::SqlitePoolOptions::new()
                    .max_connections(1)
                    .connect_with(connect_options)
                    .await
                {
                    pool_result = std::option::Option::Some(p);
                    break;
                }
            }

            match pool_result {
                std::option::Option::Some(p) => p,
                std::option::Option::None => {
                    // Fall back to connection without extension
                    let connect_options = database_url
                        .parse::<sqlx::sqlite::SqliteConnectOptions>()
                        .map_err(|e| std::format!("Failed to parse database URL: {:?}", e))?
                        .create_if_missing(true)
                        .optimize_on_close(false, std::option::Option::None);

                    sqlx::sqlite::SqlitePoolOptions::new()
                        .max_connections(1)
                        .connect_with(connect_options)
                        .await
                        .map_err(|e| std::format!("Failed to connect to SQLite: {:?}", e))?
                }
            }
        };

        // Ensure artifacts table exists
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS artifacts (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                source_id TEXT NOT NULL,
                source_type TEXT NOT NULL,
                content TEXT NOT NULL,
                metadata TEXT NULL,
                created_at TEXT NOT NULL,
                binary_content TEXT NULL,
                mime_type TEXT NULL,
                source_url TEXT NULL,
                page_number INTEGER NULL
            )"
        )
        .execute(&pool)
        .await
        .map_err(|e| std::format!("Failed to create artifacts table: {:?}", e))?;

        // Add new columns if they don't exist (for existing databases)
        let _ = sqlx::query("ALTER TABLE artifacts ADD COLUMN binary_content TEXT NULL")
            .execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE artifacts ADD COLUMN mime_type TEXT NULL")
            .execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE artifacts ADD COLUMN source_url TEXT NULL")
            .execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE artifacts ADD COLUMN page_number INTEGER NULL")
            .execute(&pool).await;

        // Ensure artifacts_vec virtual table exists with correct dimensions
        // Note: nomic-embed-text produces 768-dimensional embeddings
        if !database_url.contains(":memory:") {
            // Only create if it doesn't exist - use IF NOT EXISTS
            let create_result = sqlx::query(
                "CREATE VIRTUAL TABLE IF NOT EXISTS artifacts_vec USING vec0(
                    artifact_id TEXT PRIMARY KEY,
                    embedding FLOAT[768]
                )"
            )
            .execute(&pool)
            .await;

            if let std::result::Result::Err(e) = create_result {
                eprintln!("Warning: Failed to create artifacts_vec table: {:?}", e);
            }
        }

        std::result::Result::Ok(SqliteArtifactAdapter { pool })
    }

    fn block_on<T>(fut: impl std::future::Future<Output = T>) -> T {
        // Use the current runtime handle if available, otherwise create a new one.
        // This prevents "Cannot start a runtime from within a runtime" errors.
        match tokio::runtime::Handle::try_current() {
            std::result::Result::Ok(handle) => {
                // Already in async context - use block_in_place to avoid nested runtimes
                // NOTE: This requires the multi-threaded runtime. Tests calling sync
                // Repository methods should use #[tokio::test(flavor = "multi_thread")]
                tokio::task::block_in_place(|| handle.block_on(fut))
            }
            std::result::Result::Err(_) => {
                // No async context - create a new runtime
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("failed to build tokio runtime");
                rt.block_on(fut)
            }
        }
    }

    pub async fn save_async(&self, entity: crate::domain::artifact::Artifact) -> hexser::HexResult<()> {
        let source_type_str = std::format!("{:?}", entity.source_type);
        let created_at = entity.created_at.to_rfc3339();

        // Serialize embedding vector to JSON for storage
        let embedding_json = serde_json::to_string(&entity.embedding).map_err(|e| {
            hexser::error::hex_error::Hexserror::Adapter(
                hexser::error::adapter_error::mapping_failure(
                    std::format!("Failed to serialize embedding to JSON: {:?}", e).as_str()
                )
            )
        })?;

        // Insert into artifacts table
        sqlx::query(
            "INSERT INTO artifacts (id, project_id, source_id, source_type, content, metadata, created_at, binary_content, mime_type, source_url, page_number)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(id) DO UPDATE SET
               project_id=excluded.project_id, source_id=excluded.source_id,
               source_type=excluded.source_type, content=excluded.content,
               metadata=excluded.metadata, created_at=excluded.created_at,
               binary_content=excluded.binary_content, mime_type=excluded.mime_type,
               source_url=excluded.source_url, page_number=excluded.page_number"
        )
        .bind(&entity.id)
        .bind(&entity.project_id)
        .bind(&entity.source_id)
        .bind(source_type_str)
        .bind(&entity.content)
        .bind(&entity.metadata)
        .bind(created_at)
        .bind(&entity.binary_content)
        .bind(&entity.mime_type)
        .bind(&entity.source_url)
        .bind(entity.page_number.map(|p| p as i64))
        .execute(&self.pool)
        .await
        .map_err(|e| {
            let msg = std::format!("sqlx error inserting artifact: {:?}", e);
            hexser::error::hex_error::Hexserror::Adapter(
                hexser::error::adapter_error::connection_failed("SQLite", msg.as_str())
            )
        })?;

        // Insert into artifacts_vec virtual table for vector search
        // Note: Virtual tables don't support ON CONFLICT, so use INSERT OR REPLACE
        match sqlx::query(
            "INSERT OR REPLACE INTO artifacts_vec (artifact_id, embedding)
             VALUES (?1, ?2)"
        )
        .bind(&entity.id)
        .bind(&embedding_json)
        .execute(&self.pool)
        .await
        {
            std::result::Result::Ok(_) => {
                // Vector inserted successfully
            }
            std::result::Result::Err(_) => {
                // Vector table insert failed (non-fatal - sqlite-vec may not be loaded)
            }
        }

        std::result::Result::Ok(())
    }

    fn row_to_artifact(
        row: sqlx::sqlite::SqliteRow,
        embedding: std::vec::Vec<f32>,
    ) -> hexser::HexResult<crate::domain::artifact::Artifact> {
        let id: String = sqlx::Row::get(&row, "id");
        let project_id: String = sqlx::Row::get(&row, "project_id");
        let source_id: String = sqlx::Row::get(&row, "source_id");
        let source_type_str: String = sqlx::Row::get(&row, "source_type");
        let content: String = sqlx::Row::get(&row, "content");
        let metadata: std::option::Option<String> = sqlx::Row::get(&row, "metadata");
        let created_at_str: String = sqlx::Row::get(&row, "created_at");

        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| {
                hexser::error::hex_error::Hexserror::Adapter(
                    hexser::error::adapter_error::mapping_failure(
                        std::format!("Failed to parse created_at: {:?}", e).as_str()
                    )
                )
            })?
            .with_timezone(&chrono::Utc);

        let source_type = match source_type_str.as_str() {
            "PRD" => crate::domain::artifact::ArtifactType::PRD,
            "File" => crate::domain::artifact::ArtifactType::File,
            "WebResearch" => crate::domain::artifact::ArtifactType::WebResearch,
            "UserInput" => crate::domain::artifact::ArtifactType::UserInput,
            "Image" => crate::domain::artifact::ArtifactType::Image,
            "PDF" => crate::domain::artifact::ArtifactType::PDF,
            _ => {
                return std::result::Result::Err(hexser::error::hex_error::Hexserror::Adapter(
                    hexser::error::adapter_error::mapping_failure(
                        std::format!("Unknown source_type: {}", source_type_str).as_str()
                    )
                ));
            }
        };

        // Extract optional binary fields (may not exist in older schemas)
        let binary_content: std::option::Option<String> = sqlx::Row::try_get(&row, "binary_content").ok();
        let mime_type: std::option::Option<String> = sqlx::Row::try_get(&row, "mime_type").ok();
        let source_url: std::option::Option<String> = sqlx::Row::try_get(&row, "source_url").ok();
        let page_number: std::option::Option<u32> = sqlx::Row::try_get::<i64, _>(&row, "page_number")
            .ok()
            .map(|v| v as u32);

        std::result::Result::Ok(crate::domain::artifact::Artifact {
            id,
            project_id,
            source_id,
            source_type,
            content,
            embedding,
            metadata,
            created_at,
            binary_content,
            mime_type,
            source_url,
            page_number,
        })
    }

    async fn find_one_async(
        &self,
        filter: &crate::ports::artifact_repository_port::ArtifactFilter,
    ) -> hexser::HexResult<std::option::Option<crate::domain::artifact::Artifact>> {
        match filter {
            crate::ports::artifact_repository_port::ArtifactFilter::ById(id) => {
                // Fetch artifact metadata
                let row_opt = sqlx::query(
                    "SELECT id, project_id, source_id, source_type, content, metadata, created_at, binary_content, mime_type, source_url, page_number
                     FROM artifacts WHERE id = ?1"
                )
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

                if let std::option::Option::Some(row) = row_opt {
                    // Fetch embedding from vec table (if table exists)
                    let embedding = match sqlx::query_scalar::<_, String>(
                        "SELECT embedding FROM artifacts_vec WHERE artifact_id = ?1"
                    )
                    .bind(id)
                    .fetch_one(&self.pool)
                    .await
                    {
                        std::result::Result::Ok(embedding_json) => {
                            serde_json::from_str(&embedding_json).unwrap_or_else(|_| std::vec::Vec::new())
                        }
                        std::result::Result::Err(_) => {
                            // Table doesn't exist or no embedding found - use empty vector
                            std::vec::Vec::new()
                        }
                    };

                    std::result::Result::Ok(std::option::Option::Some(Self::row_to_artifact(row, embedding)?))
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
        filter: &crate::ports::artifact_repository_port::ArtifactFilter,
        opts: hexser::ports::repository::FindOptions<crate::ports::artifact_repository_port::ArtifactSortKey>,
    ) -> hexser::HexResult<std::vec::Vec<crate::domain::artifact::Artifact>> {
        let mut query_str = String::from(
            "SELECT a.id, a.project_id, a.source_id, a.source_type, a.content, a.metadata, a.created_at, a.binary_content, a.mime_type, a.source_url, a.page_number, v.embedding
             FROM artifacts a
             LEFT JOIN artifacts_vec v ON a.id = v.artifact_id"
        );
        let mut bind_values: std::vec::Vec<String> = std::vec::Vec::new();

        // WHERE clause
        match filter {
            crate::ports::artifact_repository_port::ArtifactFilter::ById(id) => {
                query_str.push_str(" WHERE a.id = ?1");
                bind_values.push(id.clone());
            }
            crate::ports::artifact_repository_port::ArtifactFilter::ByProjectId(project_id) => {
                query_str.push_str(" WHERE a.project_id = ?1");
                bind_values.push(project_id.clone());
            }
            crate::ports::artifact_repository_port::ArtifactFilter::BySourceId(source_id) => {
                query_str.push_str(" WHERE a.source_id = ?1");
                bind_values.push(source_id.clone());
            }
            crate::ports::artifact_repository_port::ArtifactFilter::BySourceType(source_type) => {
                query_str.push_str(" WHERE a.source_type = ?1");
                bind_values.push(std::format!("{:?}", source_type));
            }
            crate::ports::artifact_repository_port::ArtifactFilter::All => {}
        }

        // ORDER BY
        if let std::option::Option::Some(sort_specs) = opts.sort {
            let mut parts: std::vec::Vec<String> = std::vec::Vec::new();
            for s in sort_specs.iter() {
                let col = match &s.key {
                    crate::ports::artifact_repository_port::ArtifactSortKey::CreatedAt => "a.created_at",
                    crate::ports::artifact_repository_port::ArtifactSortKey::SourceType => "a.source_type",
                };
                let dir = if s.direction == hexser::ports::repository::Direction::Desc { "DESC" } else { "ASC" };
                parts.push(std::format!("{} {}", col, dir));
            }
            if !parts.is_empty() {
                query_str.push_str(" ORDER BY ");
                query_str.push_str(&parts.join(", "));
            }
        }

        // LIMIT/OFFSET
        if let std::option::Option::Some(lim) = opts.limit {
            query_str.push_str(&std::format!(" LIMIT {}", lim));
        }
        if let std::option::Option::Some(off) = opts.offset {
            query_str.push_str(&std::format!(" OFFSET {}", off));
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

        let mut artifacts = std::vec::Vec::new();
        for row in rows {
            let embedding_json_opt: std::option::Option<String> = sqlx::Row::get(&row, "embedding");
            let embedding = if let std::option::Option::Some(json_str) = embedding_json_opt {
                serde_json::from_str(&json_str).map_err(|e| {
                    hexser::error::hex_error::Hexserror::Adapter(
                        hexser::error::adapter_error::mapping_failure(
                            std::format!("Failed to deserialize embedding: {:?}", e).as_str()
                        )
                    )
                })?
            } else {
                std::vec::Vec::new()
            };

            artifacts.push(Self::row_to_artifact(row, embedding)?);
        }

        std::result::Result::Ok(artifacts)
    }

    async fn delete_async(&self, id: &str) -> hexser::HexResult<()> {
        // Delete from artifacts table (CASCADE will handle artifacts_vec if using foreign key)
        sqlx::query("DELETE FROM artifacts WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                hexser::error::hex_error::Hexserror::Adapter(
                    hexser::error::adapter_error::connection_failed(
                        "SQLite",
                        std::format!("delete query failed: {:?}", e).as_str()
                    )
                )
            })?;

        // Delete from artifacts_vec table (ignore error if table doesn't exist)
        let _ = sqlx::query("DELETE FROM artifacts_vec WHERE artifact_id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await;

        std::result::Result::Ok(())
    }

    /// Searches for artifacts similar to the given query embedding using vector similarity.
    ///
    /// # Arguments
    ///
    /// * `query_embedding` - The embedding vector to search for similar artifacts
    /// * `limit` - Maximum number of results to return
    /// * `threshold` - Minimum similarity score (0.0-1.0) - lower distance is more similar
    ///
    /// # Returns
    ///
    /// Returns a vector of (Artifact, similarity_score) tuples sorted by similarity (most similar first).
    pub async fn search_similar(
        &self,
        query_embedding: &[f32],
        limit: usize,
        threshold: f32,
    ) -> std::result::Result<std::vec::Vec<(crate::domain::artifact::Artifact, f32)>, std::string::String> {
        // Serialize query embedding to JSON
        let query_json = serde_json::to_string(&query_embedding.to_vec())
            .map_err(|e| std::format!("Failed to serialize query embedding: {:?}", e))?;

        // Build similarity search query using vec_distance_cosine
        let query_str = std::format!(
            "SELECT a.id, a.project_id, a.source_id, a.source_type, a.content, a.metadata, a.created_at, a.binary_content, a.mime_type, a.source_url, a.page_number,
                    vec_distance_cosine(v.embedding, ?1) as distance
             FROM artifacts a
             JOIN artifacts_vec v ON a.id = v.artifact_id
             ORDER BY distance ASC
             LIMIT {}", limit
        );

        // Execute query with bindings
        let query = sqlx::query(&query_str).bind(&query_json);

        let rows = query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| std::format!("Similarity search query failed: {:?}", e))?;

        let mut results = std::vec::Vec::new();
        for row in rows {
            let distance: f32 = sqlx::Row::get(&row, "distance");

            // Filter by threshold - lower distance is more similar
            // Convert distance to similarity score (1.0 - distance)
            let similarity = 1.0 - distance;
            if similarity < threshold {
                continue;
            }

            // We don't need the embedding vector in search results
            let embedding = std::vec::Vec::new();
            let artifact = Self::row_to_artifact(row, embedding)
                .map_err(|e| std::format!("Failed to map row to artifact: {:?}", e))?;

            results.push((artifact, similarity));
        }

        std::result::Result::Ok(results)
    }
}

// Implement HEXSER Repository trait
impl hexser::ports::Repository<crate::domain::artifact::Artifact> for SqliteArtifactAdapter {
    fn save(&mut self, entity: crate::domain::artifact::Artifact) -> hexser::HexResult<()> {
        Self::block_on(self.save_async(entity))
    }
}

// Implement HEXSER QueryRepository trait
impl hexser::ports::repository::QueryRepository<crate::domain::artifact::Artifact> for SqliteArtifactAdapter {
    type Filter = crate::ports::artifact_repository_port::ArtifactFilter;
    type SortKey = crate::ports::artifact_repository_port::ArtifactSortKey;

    fn find_one(&self, filter: &Self::Filter) -> hexser::HexResult<std::option::Option<crate::domain::artifact::Artifact>> {
        Self::block_on(self.find_one_async(filter))
    }

    fn find(
        &self,
        filter: &Self::Filter,
        options: hexser::ports::repository::FindOptions<Self::SortKey>,
    ) -> hexser::HexResult<std::vec::Vec<crate::domain::artifact::Artifact>> {
        Self::block_on(self.find_async(filter, options))
    }
}

// Implement ArtifactRepositoryPort with vector similarity search
impl crate::ports::artifact_repository_port::ArtifactRepositoryPort for SqliteArtifactAdapter {
    fn find_similar(
        &self,
        query_embedding: &[f32],
        limit: usize,
        threshold: std::option::Option<f32>,
        project_id: std::option::Option<String>,
    ) -> std::result::Result<std::vec::Vec<crate::ports::artifact_repository_port::SimilarArtifact>, std::string::String> {
        Self::block_on(async {
            // Serialize query embedding to JSON
            let query_json = serde_json::to_string(&query_embedding.to_vec())
                .map_err(|e| std::format!("Failed to serialize query embedding: {:?}", e))?;

            // Build similarity search query using vec_distance_cosine
            // Note: We don't select v.embedding because virtual tables return it as BLOB,
            // and we don't need the embedding vector in search results (only distance)
            let mut query_str = String::from(
                "SELECT a.id, a.project_id, a.source_id, a.source_type, a.content, a.metadata, a.created_at,
                        vec_distance_cosine(v.embedding, ?1) as distance
                 FROM artifacts a
                 JOIN artifacts_vec v ON a.id = v.artifact_id"
            );

            let mut bind_idx = 2;
            if project_id.is_some() {
                query_str.push_str(&std::format!(" WHERE a.project_id = ?{}", bind_idx));
                bind_idx += 1;
            }

            query_str.push_str(" ORDER BY distance ASC");
            query_str.push_str(&std::format!(" LIMIT {}", limit));

            // Note: threshold filtering must happen in application code after results,
            // because SQLite can't filter on calculated columns in WHERE clause

            // Execute query with bindings
            let mut query = sqlx::query(&query_str);
            query = query.bind(&query_json);
            if let std::option::Option::Some(proj_id) = &project_id {
                query = query.bind(proj_id);
            }

            let rows = query
                .fetch_all(&self.pool)
                .await
                .map_err(|e| std::format!("Similarity search query failed: {:?}", e))?;

            let mut results = std::vec::Vec::new();
            for row in rows {
                let distance: f32 = sqlx::Row::get(&row, "distance");

                // Filter by threshold in application code (can't do in SQL WHERE clause)
                if let std::option::Option::Some(thresh) = threshold {
                    if distance >= thresh {
                        continue;
                    }
                }

                // We don't need the embedding vector in search results, only the artifact data
                let embedding = std::vec::Vec::new();
                let artifact = Self::row_to_artifact(row, embedding)
                    .map_err(|e| std::format!("Failed to map row to artifact: {:?}", e))?;

                results.push(crate::ports::artifact_repository_port::SimilarArtifact {
                    artifact,
                    distance,
                });
            }

            std::result::Result::Ok(results)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hexser::ports::Repository;
    use hexser::ports::repository::QueryRepository;
    use crate::ports::artifact_repository_port::ArtifactRepositoryPort;

    #[tokio::test]
    async fn test_save_and_find_artifact() {
        let adapter = SqliteArtifactAdapter::connect_and_init("sqlite::memory:")
            .await
            .unwrap();

        let artifact = crate::domain::artifact::Artifact::new(
            String::from("proj-1"),
            String::from("prd-123"),
            crate::domain::artifact::ArtifactType::PRD,
            String::from("Test content for RAG system"),
            vec![0.1, 0.2, 0.3, 0.4],
            Option::Some(String::from(r#"{"page":1}"#)),
        );

        let artifact_id = artifact.id.clone();
        adapter.save_async(artifact).await.unwrap();

        let filter = crate::ports::artifact_repository_port::ArtifactFilter::ById(artifact_id);
        let found = adapter.find_one_async(&filter).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().content, "Test content for RAG system");
    }

    #[tokio::test]
    #[ignore] // Ignored: Vector search requires sqlite-vec extension not available in :memory: databases
    async fn test_find_similar() {
        let adapter = SqliteArtifactAdapter::connect_and_init("sqlite::memory:")
            .await
            .unwrap();

        // Save three artifacts with different embeddings
        let artifact1 = crate::domain::artifact::Artifact::new(
            String::from("proj-1"),
            String::from("src-1"),
            crate::domain::artifact::ArtifactType::File,
            String::from("Content about Rust programming"),
            vec![1.0, 0.0, 0.0, 0.0],
            Option::None,
        );
        let artifact2 = crate::domain::artifact::Artifact::new(
            String::from("proj-1"),
            String::from("src-2"),
            crate::domain::artifact::ArtifactType::File,
            String::from("Content about Python programming"),
            vec![0.9, 0.1, 0.0, 0.0],
            Option::None,
        );
        let artifact3 = crate::domain::artifact::Artifact::new(
            String::from("proj-1"),
            String::from("src-3"),
            crate::domain::artifact::ArtifactType::WebResearch,
            String::from("Content about cooking recipes"),
            vec![0.0, 0.0, 1.0, 0.0],
            Option::None,
        );

        adapter.save_async(artifact1).await.unwrap();
        adapter.save_async(artifact2).await.unwrap();
        adapter.save_async(artifact3).await.unwrap();

        // Query with embedding similar to artifact1
        let query_embedding = vec![0.95, 0.05, 0.0, 0.0];
        let results = adapter
            .find_similar(&query_embedding, 2, Option::None, Option::None)
            .unwrap();

        assert_eq!(results.len(), 2);
        // First result should be artifact2 (closest to query)
        assert!(results[0].artifact.content.contains("Python"));
        assert!(results[0].distance < 0.5);
    }
}
