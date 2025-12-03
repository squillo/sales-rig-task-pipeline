//! Quick test to verify vector search is working

use sqlx::Row;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_url = "sqlite:.rigger/tasks.db";

    // Connect with extension
    let connect_options = db_url
        .parse::<sqlx::sqlite::SqliteConnectOptions>()?
        .extension(".rigger/lib/vec0");

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(connect_options)
        .await?;

    // Count artifacts in vec table
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM artifacts_vec")
        .fetch_one(&pool)
        .await?;

    println!("✓ artifacts_vec table has {} rows", count);

    // Check a sample embedding
    if count > 0 {
        let row = sqlx::query("SELECT artifact_id, embedding FROM artifacts_vec LIMIT 1")
            .fetch_one(&pool)
            .await?;

        let artifact_id: String = row.get("artifact_id");
        let embedding_json: String = row.get("embedding");
        let embedding: Vec<f32> = serde_json::from_str(&embedding_json)?;

        println!("✓ Sample artifact: {}", artifact_id);
        println!("✓ Embedding dimensions: {}", embedding.len());
        println!("✓ First 5 values: {:?}", &embedding[..5.min(embedding.len())]);

        // Test a simple distance query
        let query_embedding = vec![0.1f32; 768];
        let query_json = serde_json::to_string(&query_embedding)?;

        let result = sqlx::query(
            "SELECT artifact_id, vec_distance_cosine(embedding, ?) as distance
             FROM artifacts_vec
             ORDER BY distance ASC
             LIMIT 3"
        )
        .bind(&query_json)
        .fetch_all(&pool)
        .await?;

        println!("\n✓ Distance query returned {} results:", result.len());
        for row in result {
            let id: String = row.get("artifact_id");
            let distance: f32 = row.get("distance");
            println!("  - {} (distance: {:.4})", id, distance);
        }
    }

    Ok(())
}
