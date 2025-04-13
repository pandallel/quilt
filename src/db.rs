//! Database utilities for SQLite setup and connection management

use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Sqlite, SqlitePool};
use tracing::{debug, info};

/// Initialize an in-memory SQLite database with required schema
pub async fn init_memory_db() -> Result<SqlitePool, sqlx::Error> {
    let url = "sqlite::memory:";

    debug!("Creating in-memory SQLite database");

    // Create database
    if !Sqlite::database_exists(url).await.unwrap_or(false) {
        Sqlite::create_database(url).await?;
    }

    // Create connection pool with a single connection for in-memory DB
    // (must keep one connection alive for the in-memory DB to persist)
    let pool = SqlitePoolOptions::new()
        .min_connections(1)
        .max_connections(5)
        .connect(url)
        .await?;

    // Create materials table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS materials (
            id TEXT PRIMARY KEY,
            file_path TEXT NOT NULL,
            file_type TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            status_updated_at TEXT NOT NULL,
            status TEXT NOT NULL,
            error TEXT
        )
        "#,
    )
    .execute(&pool)
    .await?;

    // Create cuts table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS cuts (
            id TEXT PRIMARY KEY,
            material_id TEXT NOT NULL,
            chunk_index INTEGER NOT NULL,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL,
            token_count INTEGER,
            byte_offset_start INTEGER,
            byte_offset_end INTEGER,
            FOREIGN KEY (material_id) REFERENCES materials (id)
        )
        "#,
    )
    .execute(&pool)
    .await?;

    info!("SQLite in-memory database initialized with tables");

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_db_initialization() {
        let pool = init_memory_db().await.expect("Failed to initialize DB");

        // Check if materials table exists and has correct columns
        let result = sqlx::query(
            r#"
            SELECT name, sql FROM sqlite_master 
            WHERE type='table' AND name='materials'
            "#,
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to query sqlite_master");

        let table_sql: String = result.get("sql");
        assert!(
            table_sql.contains("created_at"),
            "Missing created_at column"
        );
        assert!(
            table_sql.contains("updated_at"),
            "Missing updated_at column"
        );
        assert!(
            table_sql.contains("status_updated_at"),
            "Missing status_updated_at column"
        );

        // Check if cuts table exists and has correct columns
        let result = sqlx::query(
            r#"
            SELECT name, sql FROM sqlite_master 
            WHERE type='table' AND name='cuts'
            "#,
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to query sqlite_master");

        let table_sql: String = result.get("sql");
        assert!(table_sql.contains("id"), "Missing id column");
        assert!(
            table_sql.contains("material_id"),
            "Missing material_id column"
        );
        assert!(
            table_sql.contains("chunk_index"),
            "Missing chunk_index column"
        );
        assert!(table_sql.contains("content"), "Missing content column");
        assert!(
            table_sql.contains("created_at"),
            "Missing created_at column"
        );
        assert!(
            table_sql.contains("token_count"),
            "Missing token_count column"
        );
        assert!(
            table_sql.contains("byte_offset_start"),
            "Missing byte_offset_start column"
        );
        assert!(
            table_sql.contains("byte_offset_end"),
            "Missing byte_offset_end column"
        );
    }
}
