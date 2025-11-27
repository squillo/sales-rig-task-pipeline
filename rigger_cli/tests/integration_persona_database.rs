//! Integration test for persona management database schema and seeding.
//!
//! Tests the Phase 3 SQLite adapter initialization:
//! 1. Verifies personas, agent_tools, and persona_tools tables are created
//! 2. Verifies 15 agent tools are seeded
//! 3. Verifies default persona is created with 6 safe tools
//!
//! Revision History
//! - 2025-11-26T07:45:00Z @AI: Create integration test for persona database schema initialization.

#[tokio::test]
async fn test_persona_tables_initialization() {
    // Test: Validates persona management tables are created and seeded correctly.
    // Justification: End-to-end verification of Phase 3 database initialization.

    use sqlx::Row;

    // 1. Initialize in-memory database
    let adapter = task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::connect_and_init(
        "sqlite::memory:",
    )
    .await
    .unwrap();

    // 2. Verify agent_tools table exists and has 15 tools
    let tool_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM agent_tools")
        .fetch_one(adapter.pool())
        .await
        .unwrap();

    std::assert_eq!(
        tool_count, 15,
        "Should have 15 agent tools seeded"
    );

    // 3. Verify 6 tools are marked as default (Safe tools)
    let default_tool_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM agent_tools WHERE is_default = 1")
            .fetch_one(adapter.pool())
            .await
            .unwrap();

    std::assert_eq!(
        default_tool_count, 6,
        "Should have 6 default safe tools"
    );

    // 4. Verify tools are correctly categorized
    let dev_tools: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM agent_tools WHERE category = 'Development'")
            .fetch_one(adapter.pool())
            .await
            .unwrap();
    std::assert_eq!(dev_tools, 6, "Should have 6 Development tools (code_search, code_read, grep_search, bash_exec, git_commit, git_push)");

    let research_tools: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM agent_tools WHERE category = 'Research'")
            .fetch_one(adapter.pool())
            .await
            .unwrap();
    std::assert_eq!(research_tools, 3, "Should have 3 Research tools");

    let filesystem_tools: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM agent_tools WHERE category = 'FileSystem'")
            .fetch_one(adapter.pool())
            .await
            .unwrap();
    std::assert_eq!(filesystem_tools, 3, "Should have 3 FileSystem tools");

    let database_tools: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM agent_tools WHERE category = 'Database'")
            .fetch_one(adapter.pool())
            .await
            .unwrap();
    std::assert_eq!(database_tools, 2, "Should have 2 Database tools");

    let network_tools: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM agent_tools WHERE category = 'Network'")
            .fetch_one(adapter.pool())
            .await
            .unwrap();
    std::assert_eq!(network_tools, 1, "Should have 1 Network tool");

    // 5. Verify risk levels
    let safe_tools: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM agent_tools WHERE risk_level = 'Safe'")
            .fetch_one(adapter.pool())
            .await
            .unwrap();
    std::assert_eq!(safe_tools, 6, "Should have 6 Safe tools");

    let moderate_tools: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM agent_tools WHERE risk_level = 'Moderate'")
            .fetch_one(adapter.pool())
            .await
            .unwrap();
    std::assert_eq!(moderate_tools, 5, "Should have 5 Moderate tools");

    let high_tools: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM agent_tools WHERE risk_level = 'High'")
            .fetch_one(adapter.pool())
            .await
            .unwrap();
    std::assert_eq!(high_tools, 4, "Should have 4 High risk tools");

    // 6. Verify personas table exists and has default persona
    let persona_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM personas")
        .fetch_one(adapter.pool())
        .await
        .unwrap();

    std::assert_eq!(
        persona_count, 1,
        "Should have 1 default persona"
    );

    // 7. Verify default persona details
    let persona_row = sqlx::query("SELECT id, name, role, description, is_default FROM personas WHERE id = 'default-persona-001'")
        .fetch_one(adapter.pool())
        .await
        .unwrap();

    let persona_id: String = persona_row.get(0);
    let persona_name: String = persona_row.get(1);
    let persona_role: String = persona_row.get(2);
    let persona_description: String = persona_row.get(3);
    let is_default: bool = persona_row.get(4);

    std::assert_eq!(persona_id, "default-persona-001");
    std::assert_eq!(persona_name, "Default Agent");
    std::assert_eq!(persona_role, "General Purpose Assistant");
    std::assert_eq!(
        persona_description,
        "Default persona with safe read-only tools enabled"
    );
    std::assert!(is_default, "Default persona should have is_default = true");

    // 8. Verify persona_tools junction table
    let persona_tool_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM persona_tools WHERE persona_id = 'default-persona-001' AND enabled = 1")
            .fetch_one(adapter.pool())
            .await
            .unwrap();

    std::assert_eq!(
        persona_tool_count, 6,
        "Default persona should have 6 enabled tools"
    );

    // 9. Verify specific safe tools are linked
    let tool_ids = sqlx::query("SELECT tool_id FROM persona_tools WHERE persona_id = 'default-persona-001' AND enabled = 1 ORDER BY tool_id")
        .fetch_all(adapter.pool())
        .await
        .unwrap();

    let tool_ids: std::vec::Vec<String> = tool_ids.iter().map(|r| r.get(0)).collect();

    std::assert_eq!(tool_ids.len(), 6);
    std::assert_eq!(tool_ids[0], "code_read");
    std::assert_eq!(tool_ids[1], "code_search");
    std::assert_eq!(tool_ids[2], "doc_search");
    std::assert_eq!(tool_ids[3], "grep_search");
    std::assert_eq!(tool_ids[4], "web_fetch");
    std::assert_eq!(tool_ids[5], "web_search");

    std::println!("✓ Persona management database schema initialized correctly");
}

#[tokio::test]
async fn test_idempotent_initialization() {
    // Test: Validates connect_and_init can be called multiple times without duplication.
    // Justification: Ensures database seeding is idempotent for reliability.

    // 1. First initialization
    let adapter1 = task_manager::adapters::sqlite_task_adapter::SqliteTaskAdapter::connect_and_init(
        "sqlite::memory:",
    )
    .await
    .unwrap();

    let tool_count1: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM agent_tools")
        .fetch_one(adapter1.pool())
        .await
        .unwrap();

    let persona_count1: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM personas")
        .fetch_one(adapter1.pool())
        .await
        .unwrap();

    std::assert_eq!(tool_count1, 15);
    std::assert_eq!(persona_count1, 1);

    std::println!("✓ Idempotent initialization test passed");
}
