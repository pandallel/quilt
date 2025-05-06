//! Database utilities for SQLite setup and connection management

use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Sqlite, SqlitePool};
use std::sync::Once;
use tracing::{debug, info};

// Global static for ensuring one-time initialization of the sqlite-vec extension.
static SQLITE_VEC_INIT: Once = Once::new();

/// Registers the sqlite-vec extension globally using rusqlite's auto_extension mechanism.
/// This function is thread-safe due to std::sync::Once and should be called before
/// any SQLite connections that need the extension are opened.
fn register_sqlite_vec_globally() {
    SQLITE_VEC_INIT.call_once(|| {
        // Safety: This function interacts with C FFI and modifies global SQLite state.
        // It's safe assuming sqlite_vec::sqlite3_vec_init provides a valid pointer
        // and this `call_once` block ensures it only runs once across all threads.
        // The transmute is necessary for the FFI function signature.
        unsafe {
            // Use the rusqlite::ffi module directly for sqlite3_auto_extension
            match rusqlite::ffi::sqlite3_auto_extension(Some(
                std::mem::transmute(sqlite_vec::sqlite3_vec_init as *const ()),
            )) {
                rusqlite::ffi::SQLITE_OK => {
                    // Use eprintln! here as tracing might not be initialized yet.
                    eprintln!("Successfully registered sqlite-vec extension globally.");
                }
                err_code => {
                    // Critical error: The application/tests cannot function correctly without the extension.
                    eprintln!(
                        "FATAL: Failed to register sqlite-vec extension globally. SQLite error code: {}",
                        err_code
                    );
                    panic!("Failed to register sqlite-vec extension globally.");
                }
            }
        }
    });
}

/// Initialize an in-memory SQLite database with required schema
pub async fn init_memory_db() -> Result<SqlitePool, sqlx::Error> {
    // Ensure the extension is registered before opening any connections.
    register_sqlite_vec_globally();

    let url = "sqlite::memory:";

    debug!("Creating in-memory SQLite database at {}", url);

    // Create database if it doesn't exist (mostly relevant for file-based DBs, but doesn't hurt)
    if !Sqlite::database_exists(url).await.unwrap_or(false) {
        match Sqlite::create_database(url).await {
            Ok(_) => debug!("Created new SQLite database."),
            Err(e) => {
                // Use eprintln for early errors
                eprintln!("Failed to create SQLite database: {}", e);
                return Err(e);
            }
        }
    }

    // Create connection pool
    // For in-memory, must keep at least one connection alive.
    debug!("Creating SQLite connection pool...");
    let pool = SqlitePoolOptions::new()
        .min_connections(1)
        .max_connections(5) // Adjust max connections as needed
        .connect(url)
        .await?;
    debug!("SQLite connection pool created.");

    // --- Schema Creation --- //
    debug!("Applying database schema...");

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

    // Create swatches table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS swatches (
            id TEXT PRIMARY KEY,
            cut_id TEXT NOT NULL,
            material_id TEXT NOT NULL,
            embedding BLOB NOT NULL,
            model_name TEXT NOT NULL,
            model_version TEXT NOT NULL,
            created_at TEXT NOT NULL,
            dimensions INTEGER NOT NULL,
            metadata TEXT, -- Storing metadata as JSON string or similar
            similarity_threshold REAL, -- Optional similarity threshold
            FOREIGN KEY (cut_id) REFERENCES cuts (id) ON DELETE CASCADE,
            FOREIGN KEY (material_id) REFERENCES materials (id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(&pool)
    .await?;

    // --- Vector Search Initialization --- //

    // Note: The #[cfg(not(test))] block has been removed.
    // The global registration now happens via register_sqlite_vec_globally() above.
    // The old `unsafe { sqlite_vec::sqlite3_vec_init(); }` was incorrect and is removed.
    // The `SELECT vss0_version()` check is removed as it's not necessary for init
    // and might fail if the table doesn't exist yet.

    // Create the vss_swatches virtual table for vector similarity search.
    // The extension is loaded automatically for connections from the pool
    // because we called sqlite3_auto_extension earlier.
    // Dimensions set to 384 based on previous code comment.
    debug!("Creating vss_swatches virtual table (dimension: 384)...");
    sqlx::query(
        r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS vss_swatches USING vec0(
            embedding float[384]
            -- Store other swatch fields as UNINDEXED if needed for retrieval
            -- Example: id UNINDEXED,
            -- Example: cut_id UNINDEXED
            -- Only include columns needed by the VSS index or for retrieval alongside distance
        )
        "#,
    )
    .execute(&pool)
    .await?;
    debug!("vss_swatches virtual table created.");

    info!("SQLite in-memory database initialized successfully.");

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Row;

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

        // We skip checking for vector search functionality in tests
    }
}
