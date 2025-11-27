//! SQLite-backed SessionStorage for graph_flow using sqlx.
//!
//! This adapter persists graph_flow sessions in a SQLite database table
//! for durability across process restarts. It is enabled behind the
//! `sqlite_persistence` Cargo feature to keep default builds lightweight.
//!
//! Table schema (created automatically):
//! - sessions(id TEXT PRIMARY KEY, data TEXT NOT NULL)
//!
//! Revision History
//! - 2025-11-18T11:22:30Z @AI: Add SQLiteSessionStorage implementing graph_flow::SessionStorage with sqlx backend and unit tests.

#[cfg(feature = "sqlite_persistence")]
pub struct SQLiteSessionStorage {
    pool: sqlx::SqlitePool,
}

#[cfg(feature = "sqlite_persistence")]
impl SQLiteSessionStorage {
    /// Connects to the SQLite database and ensures the sessions table exists.
    pub async fn connect(database_url: &str) -> std::result::Result<Self, std::string::String> {
        // Connect
        let pool = match sqlx::SqlitePool::connect(database_url).await {
            std::result::Result::Ok(p) => p,
            std::result::Result::Err(e) => return std::result::Result::Err(std::format!("sqlite connect error: {}", e)),
        };
        // Create table if not exists
        let query = "CREATE TABLE IF NOT EXISTS sessions (id TEXT PRIMARY KEY, data TEXT NOT NULL)";
        match sqlx::query(query).execute(&pool).await {
            std::result::Result::Ok(_) => {}
            std::result::Result::Err(e) => return std::result::Result::Err(std::format!("sqlite schema error: {}", e)),
        }
        std::result::Result::Ok(SQLiteSessionStorage { pool })
    }
}

#[cfg(feature = "sqlite_persistence")]
#[async_trait::async_trait]
impl graph_flow::SessionStorage for SQLiteSessionStorage {
    async fn save(&self, session: graph_flow::Session) -> graph_flow::Result<()> {
        let id = session.id.clone();
        let json = match serde_json::to_string(&session) {
            std::result::Result::Ok(j) => j,
            std::result::Result::Err(e) => {
                return std::result::Result::Err(graph_flow::GraphError::TaskExecutionFailed(std::format!(
                    "session serialize error: {}",
                    e
                )))
            }
        };
        let q = "INSERT INTO sessions (id, data) VALUES (?1, ?2) ON CONFLICT(id) DO UPDATE SET data=excluded.data";
        match sqlx::query(q).bind(id).bind(json).execute(&self.pool).await {
            std::result::Result::Ok(_) => std::result::Result::Ok(()),
            std::result::Result::Err(e) => std::result::Result::Err(graph_flow::GraphError::TaskExecutionFailed(std::format!(
                "sqlite save error: {}",
                e
            ))),
        }
    }

    async fn get(&self, session_id: &str) -> graph_flow::Result<std::option::Option<graph_flow::Session>> {
        let row = match sqlx::query_as::<_, (std::string::String,)>("SELECT data FROM sessions WHERE id = ?1")
            .bind(session_id)
            .fetch_optional(&self.pool)
            .await
        {
            std::result::Result::Ok(r) => r,
            std::result::Result::Err(e) => {
                return std::result::Result::Err(graph_flow::GraphError::TaskExecutionFailed(std::format!(
                    "sqlite get error: {}",
                    e
                )))
            }
        };
        if let std::option::Option::Some((data_json,)) = row {
            match serde_json::from_str::<graph_flow::Session>(&data_json) {
                std::result::Result::Ok(sess) => std::result::Result::Ok(std::option::Option::Some(sess)),
                std::result::Result::Err(e) => std::result::Result::Err(graph_flow::GraphError::TaskExecutionFailed(std::format!(
                    "session deserialize error: {}",
                    e
                ))),
            }
        } else {
            std::result::Result::Ok(std::option::Option::None)
        }
    }

    async fn delete(&self, session_id: &str) -> graph_flow::Result<()> {
        match sqlx::query("DELETE FROM sessions WHERE id = ?1").bind(session_id).execute(&self.pool).await {
            std::result::Result::Ok(_) => std::result::Result::Ok(()),
            std::result::Result::Err(e) => std::result::Result::Err(graph_flow::GraphError::TaskExecutionFailed(std::format!(
                "sqlite delete error: {}",
                e
            ))),
        }
    }
}

#[cfg(all(test, feature = "sqlite_persistence"))]
mod tests {
    #[tokio::test]
    async fn test_sqlite_session_storage_roundtrip() {
        // Connect to in-memory database
        let storage = super::SQLiteSessionStorage::connect("sqlite::memory:")
            .await
            .expect("connect");
        // Create a simple session and persist
        let start_task_id = "start_task";
        let session_id = uuid::Uuid::new_v4().to_string();
        let session = graph_flow::Session::new_from_task(session_id.clone(), start_task_id);
        match <super::SQLiteSessionStorage as graph_flow::SessionStorage>::save(&storage, session).await {
            std::result::Result::Ok(_) => {}
            std::result::Result::Err(e) => panic!("save error: {:?}", e),
        }
        // Retrieve
        let got = <super::SQLiteSessionStorage as graph_flow::SessionStorage>::get(&storage, &session_id)
            .await
            .expect("get");
        std::assert!(got.is_some());
        let s = got.unwrap();
        std::assert_eq!(s.id, session_id);
    }
}
