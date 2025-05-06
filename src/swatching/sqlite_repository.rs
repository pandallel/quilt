use async_trait::async_trait;
use sqlx::{sqlite::SqliteRow, Row, SqlitePool, Transaction, Sqlite};
use std::fmt::Debug;
use tracing::{debug, error};

use super::repository::{Result, SwatchRepository, SwatchRepositoryError};
use super::swatch::Swatch;

// Helper function to serialize Vec<f32> to Vec<u8>
// Uses native endianness for potentially better performance on the same architecture.
fn f32_vec_to_bytes(vec: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(std::mem::size_of_val(vec));
    for &float in vec {
        bytes.extend_from_slice(&float.to_ne_bytes()); // Using native endianness
    }
    bytes
}

// Helper function to deserialize Vec<u8> to Vec<f32>
fn bytes_to_f32_vec(bytes: &[u8]) -> std::result::Result<Vec<f32>, String> {
    let float_size = std::mem::size_of::<f32>();
    if bytes.len() % float_size != 0 {
        return Err("Invalid byte slice length for f32 deserialization".to_string());
    }
    let num_floats = bytes.len() / float_size;
    let mut vec = Vec::with_capacity(num_floats);
    for i in 0..num_floats {
        let start = i * float_size;
        let end = start + float_size;
        let float_bytes: [u8; 4] = bytes[start..end]
            .try_into()
            .map_err(|e| format!("Failed to slice bytes: {}", e))?;
        vec.push(f32::from_ne_bytes(float_bytes));
    }
    Ok(vec)
}

#[derive(Debug)]
pub struct SqliteSwatchRepository {
    pool: SqlitePool,
}

impl SqliteSwatchRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /*
     * Transaction Management Pattern
     *
     * This repository uses a standardized transaction management pattern with three helper methods:
     *
     * 1. `execute_in_transaction` - For raw transactions with custom error handling
     * 2. `execute_query_in_transaction` - For write operations that need transaction guarantees
     * 3. `execute_read_query` - For read-only operations that can execute directly against the pool
     *
     * ## Benefits of this approach:
     *
     * - **Consistency**: All repository operations follow the same patterns for error handling
     * - **DRY Code**: Reduces duplication of transaction management logic and error handling
     * - **Clear Intent**: Method names explicitly communicate the transaction requirements
     * - **Testability**: Centralized transaction logic is easier to test and verify
     * - **Error Mapping**: Consistent translation of database errors to domain errors
     *
     * ## Examples:
     *
     * ```
     * // For write operations:
     * self.execute_query_in_transaction(move |tx| {
     *     Box::pin(async move {
     *         sqlx::query("INSERT INTO my_table (id, value) VALUES (?, ?)")
     *             .bind(&id)
     *             .bind(&value)
     *             .execute(&mut **tx)
     *             .await
     *     })
     * }).await
     *
     * // For read operations:
     * self.execute_read_query(move |pool| {
     *     Box::pin(async move {
     *         sqlx::query_as::<_, MyEntity>("SELECT * FROM my_table WHERE id = ?")
     *             .bind(&id)
     *             .fetch_optional(pool)
     *             .await
     *     })
     * }).await
     * ```
     *
     * ## Usage Guidelines:
     *
     * - Use `execute_query_in_transaction` for INSERT, UPDATE, DELETE operations
     * - Use `execute_read_query` for SELECT operations
     * - Use `execute_in_transaction` for multiple statements that need custom error handling
     * - Consider using domain-specific query extraction methods for complex queries
     */

    // MARK: - Helper methods for data conversion
    
    // Helper function to map a SqliteRow to a Swatch object
    fn map_row_to_swatch(row: &SqliteRow) -> std::result::Result<Swatch, sqlx::Error> {
        let embedding_bytes: Vec<u8> = row.try_get("embedding")?;
        let embedding = bytes_to_f32_vec(&embedding_bytes).map_err(|e| {
            sqlx::Error::Decode(format!("Failed to decode embedding bytes: {}", e).into())
        })?;

        let metadata_str: Option<String> = row.try_get("metadata")?;
        let metadata: Option<serde_json::Value> = match metadata_str {
            Some(s) if !s.is_empty() => serde_json::from_str(&s)
                .map_err(|e| sqlx::Error::Decode(format!("Invalid JSON metadata: {}", e).into()))?,
            _ => None,
        };

        Ok(Swatch {
            id: row.try_get("id")?,
            cut_id: row.try_get("cut_id")?,
            material_id: row.try_get("material_id")?,
            embedding,
            model_name: row.try_get("model_name")?,
            model_version: row.try_get("model_version")?,
            created_at: row.try_get("created_at")?, // Assuming Swatch uses OffsetDateTime from time crate
            dimensions: row.try_get::<i64, _>("dimensions")? as usize, // SQLite stores as i64
            metadata,
            similarity_threshold: row.try_get::<Option<f32>, _>("similarity_threshold")?,
        })
    }

    /// Execute a function within a transaction.
    /// 
    /// This helper method creates a new transaction, executes the provided function with the transaction,
    /// and handles committing or rolling back the transaction based on the function's result.
    /// 
    /// # Arguments
    /// * `f` - A function that takes a transaction and returns a Result
    /// 
    /// # Returns
    /// * The result of the function execution
    /// 
    /// # Future Enhancements
    /// A retry mechanism could be implemented for transient errors (like connection timeouts or deadlocks) by:
    /// 1. Adding a retry_count parameter with default value
    /// 2. Identifying retryable errors in the error handling section
    /// 3. Implementing exponential backoff for retries
    /// 4. Preserving the original error context while retrying
    ///
    /// Example implementation pattern for future reference:
    /// ```text
    /// async fn execute_with_retry<F, T>(&self, f: F, max_retries: u32) -> Result<T>
    /// where
    ///    F: for<'a> FnMut(&'a mut Transaction<'_, Sqlite>) -> ... + Send + 'static,
    /// {
    ///     let mut retries = 0;
    ///     let mut backoff = Duration::from_millis(10);
    ///     
    ///     loop {
    ///         match self.execute_in_transaction(f).await {
    ///             Ok(result) => return Ok(result),
    ///             Err(e) if retries < max_retries && is_retryable_error(&e) => {
    ///                 retries += 1;
    ///                 tokio::time::sleep(backoff).await;
    ///                 backoff *= 2; // Exponential backoff
    ///             },
    ///             Err(e) => return Err(e),
    ///         }
    ///     }
    /// }
    /// ```
    async fn execute_in_transaction<F, T>(&self, f: F) -> Result<T>
    where
        F: for<'a> FnOnce(&'a mut Transaction<'_, Sqlite>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send + 'a>> + Send + 'static,
    {
        let mut tx = self.pool.begin().await.map_err(|e| {
            error!("Failed to begin transaction: {}", e);
            SwatchRepositoryError::OperationFailed(format!("Failed to begin transaction: {}", e).into())
        })?;

        let result = f(&mut tx).await;

        match result {
            Ok(value) => {
                tx.commit().await.map_err(|e| {
                    error!("Failed to commit transaction: {}", e);
                    SwatchRepositoryError::OperationFailed(format!("Failed to commit transaction: {}", e).into())
                })?;
                Ok(value)
            }
            Err(err) => {
                if let Err(e) = tx.rollback().await {
                    error!("Failed to rollback transaction: {}", e);
                }
                Err(err)
            }
        }
    }

    /// Execute a query in a transaction.
    ///
    /// This is a convenience wrapper around execute_in_transaction for single query operations.
    /// It handles standard error mapping for SQLite errors.
    ///
    /// # Arguments
    /// * `f` - A function that takes a transaction and returns a SQLx query result
    ///
    /// # Returns
    /// * The result of the query execution mapped to our Result type
    async fn execute_query_in_transaction<F, T>(&self, f: F) -> Result<T>
    where
        F: for<'a> FnOnce(&'a mut Transaction<'_, Sqlite>) -> std::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<T, sqlx::Error>> + Send + 'a>> + Send + 'static,
    {
        self.execute_in_transaction(|tx| Box::pin(async move {
            match f(tx).await {
                Ok(value) => Ok(value),
                Err(e) => {
                    let err_msg = format!("Query execution failed: {}", e);
                    error!("{}", err_msg);
                    
                    // Map database errors to specific repository errors
                    match &e {
                        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                            return Err(SwatchRepositoryError::SwatchAlreadyExists(
                                "Duplicate ID detected".into(),
                            ));
                        }
                        _ => {}
                    }
                    
                    Err(SwatchRepositoryError::OperationFailed(err_msg.into()))
                }
            }
        })).await
    }

    /// Execute a read-only query.
    ///
    /// This method is optimized for read operations that don't require transaction
    /// guarantees and can be executed directly against the connection pool.
    ///
    /// # Arguments
    /// * `f` - A function that takes a SQL connection and returns a result
    ///
    /// # Returns
    /// * The result of the query execution mapped to our Result type
    async fn execute_read_query<F, T, E>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&SqlitePool) -> std::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<T, E>> + Send + '_>> + Send + 'static,
        E: std::error::Error + 'static,
    {
        match f(&self.pool).await {
            Ok(value) => Ok(value),
            Err(e) => {
                let err_msg = format!("Read query execution failed: {}", e);
                error!("{}", err_msg);
                Err(SwatchRepositoryError::OperationFailed(err_msg.into()))
            }
        }
    }

    /// Execute the save swatch query within a transaction.
    /// 
    /// This is an extracted helper method to handle the complex save swatch query logic.
    /// 
    /// # Arguments
    /// * `tx` - The transaction to execute the query within
    /// * `swatch_id` - The ID of the swatch
    /// * `cut_id` - The ID of the cut
    /// * `material_id` - The ID of the material
    /// * `embedding_bytes` - The serialized embedding bytes
    /// * `model_name` - The name of the embedding model
    /// * `model_version` - The version of the embedding model
    /// * `created_at` - When the swatch was created
    /// * `dimensions` - The number of dimensions in the embedding
    /// * `metadata_json` - Optional JSON metadata as string
    /// * `similarity_threshold` - Optional similarity threshold
    /// 
    /// # Returns
    /// * The result of the query execution
    #[allow(clippy::too_many_arguments)]
    async fn execute_save_swatch_query(
        tx: &mut Transaction<'_, Sqlite>,
        swatch_id: &str,
        cut_id: &str,
        material_id: &str,
        embedding_bytes: &[u8],
        model_name: &str,
        model_version: &str,
        created_at: time::OffsetDateTime,
        dimensions: i64,
        metadata_json: &Option<String>,
        similarity_threshold: Option<f32>,
    ) -> std::result::Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO swatches (
                id, cut_id, material_id, embedding, model_name, model_version, 
                created_at, dimensions, metadata, similarity_threshold
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                cut_id = excluded.cut_id,
                material_id = excluded.material_id,
                embedding = excluded.embedding,
                model_name = excluded.model_name,
                model_version = excluded.model_version,
                dimensions = excluded.dimensions,
                metadata = excluded.metadata,
                similarity_threshold = excluded.similarity_threshold
            "#,
        )
        .bind(swatch_id)
        .bind(cut_id)
        .bind(material_id)
        .bind(embedding_bytes)
        .bind(model_name)
        .bind(model_version)
        .bind(created_at)
        .bind(dimensions)
        .bind(metadata_json)
        .bind(similarity_threshold)
        .execute(&mut **tx)
        .await
    }
}

#[async_trait]
impl SwatchRepository for SqliteSwatchRepository {
    async fn save_swatch(&self, swatch: &Swatch) -> Result<()> {
        debug!("Saving swatch with id: {}", swatch.id);
        
        // Using execute_query_in_transaction for this write operation to ensure:
        // 1. ACID guarantees for the INSERT/UPDATE operation
        // 2. Proper error mapping for unique constraint violations
        // 3. Consistent transaction management with automatic rollback on error
        
        // Clone or copy all needed values to avoid borrowing issues
        let swatch_id = swatch.id.clone();
        let cut_id = swatch.cut_id.clone();
        let material_id = swatch.material_id.clone();
        let embedding_bytes = f32_vec_to_bytes(&swatch.embedding);
        let model_name = swatch.model_name.clone();
        let model_version = swatch.model_version.clone();
        let created_at = swatch.created_at;
        let dimensions = swatch.dimensions as i64;
        let similarity_threshold = swatch.similarity_threshold;
        
        // Serialize metadata outside the closure
        let metadata_json = swatch
            .metadata
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|e| {
                SwatchRepositoryError::OperationFailed(
                    format!("Failed to serialize metadata: {}", e).into(),
                )
            })?;

        // Use the transaction helper with the extracted query method
        self.execute_query_in_transaction(move |tx| {
            // Use the extracted helper method
            let bytes_for_query = embedding_bytes.clone();
            let metadata_for_query = metadata_json.clone();
            
            Box::pin(async move {
                Self::execute_save_swatch_query(
                    tx,
                    &swatch_id,
                    &cut_id,
                    &material_id,
                    &bytes_for_query,
                    &model_name,
                    &model_version,
                    created_at,
                    dimensions,
                    &metadata_for_query,
                    similarity_threshold,
                ).await
            })
        }).await.map(|_| ())
    }

    async fn save_swatches_batch(&self, swatches: &[Swatch]) -> Result<()> {
        debug!("Saving batch of {} swatches", swatches.len());
        
        // Using execute_query_in_transaction for this batch operation because:
        // 1. We need atomic transaction guarantees for the batch
        // 2. It provides consistent error handling and transaction management
        // 3. Batch operations should be committed or rolled back as a unit
        
        // Clone the swatches for use in the closure
        let swatches_for_closure = swatches.to_vec();
        
        self.execute_query_in_transaction(move |tx| {
            Box::pin(async move {
                for swatch in &swatches_for_closure {
                    let embedding_bytes = f32_vec_to_bytes(&swatch.embedding);
                    let metadata_json = swatch
                        .metadata
                        .as_ref()
                        .map(serde_json::to_string)
                        .transpose()
                        .map_err(|e| {
                            sqlx::Error::Decode(
                                format!("Failed to serialize metadata for {}: {}", swatch.id, e).into(),
                            )
                        })?;

                    sqlx::query(
                        r#"
                        INSERT INTO swatches (
                            id, cut_id, material_id, embedding, model_name, model_version, 
                            created_at, dimensions, metadata, similarity_threshold
                        )
                        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                        ON CONFLICT(id) DO UPDATE SET
                            cut_id = excluded.cut_id,
                            material_id = excluded.material_id,
                            embedding = excluded.embedding,
                            model_name = excluded.model_name,
                            model_version = excluded.model_version,
                            dimensions = excluded.dimensions,
                            metadata = excluded.metadata,
                            similarity_threshold = excluded.similarity_threshold
                        "#,
                    )
                    .bind(&swatch.id)
                    .bind(&swatch.cut_id)
                    .bind(&swatch.material_id)
                    .bind(&embedding_bytes)
                    .bind(&swatch.model_name)
                    .bind(&swatch.model_version)
                    .bind(swatch.created_at)
                    .bind(swatch.dimensions as i64)
                    .bind(&metadata_json)
                    .bind(swatch.similarity_threshold)
                    .execute(&mut **tx)
                    .await?;
                }
                
                Ok(())
            })
        }).await
    }

    async fn get_swatch_by_id(&self, swatch_id: &str) -> Result<Option<Swatch>> {
        debug!("Getting swatch by id: {}", swatch_id);
        
        // Using execute_read_query for this operation because:
        // 1. It's a read-only query that doesn't require transaction guarantees
        // 2. It provides consistent error handling and mapping to repository errors
        // 3. It's more efficient for read operations by avoiding transaction overhead
        
        // Clone for use in closure
        let id_for_closure = swatch_id.to_string();
        
        self.execute_read_query(move |pool| {
            Box::pin(async move {
                let row = sqlx::query("SELECT * FROM swatches WHERE id = ?")
                    .bind(&id_for_closure)
                    .fetch_optional(pool)
                    .await?;
                
                match row {
                    Some(row) => {
                        let swatch = Self::map_row_to_swatch(&row)?;
                        Ok::<Option<Swatch>, sqlx::Error>(Some(swatch))
                    },
                    None => Ok::<Option<Swatch>, sqlx::Error>(None)
                }
            })
        }).await
    }

    async fn get_swatches_by_cut_id(&self, cut_id: &str) -> Result<Vec<Swatch>> {
        debug!("Getting swatches by cut_id: {}", cut_id);
        
        // Using execute_read_query for this operation because:
        // 1. It's a read-only query that doesn't require transaction guarantees
        // 2. It provides consistent error handling and mapping to repository errors
        // 3. It's more efficient for read operations by avoiding transaction overhead
        
        // Clone for use in closure
        let cut_id_for_closure = cut_id.to_string();
        
        self.execute_read_query(move |pool| {
            Box::pin(async move {
                let rows = sqlx::query("SELECT * FROM swatches WHERE cut_id = ? ORDER BY created_at")
                    .bind(&cut_id_for_closure)
                    .fetch_all(pool)
                    .await?;
                
                rows.iter()
                    .map(Self::map_row_to_swatch)
                    .collect::<std::result::Result<Vec<_>, _>>()
            })
        }).await
    }

    async fn get_swatches_by_material_id(&self, material_id: &str) -> Result<Vec<Swatch>> {
        debug!("Getting swatches by material_id: {}", material_id);
        
        // Using execute_read_query for this operation because:
        // 1. It's a read-only query that doesn't require transaction guarantees
        // 2. It provides consistent error handling and mapping to repository errors
        // 3. It's more efficient for read operations by avoiding transaction overhead
        
        // Clone for use in closure
        let material_id_for_closure = material_id.to_string();
        
        self.execute_read_query(move |pool| {
            Box::pin(async move {
                let rows = sqlx::query("SELECT * FROM swatches WHERE material_id = ? ORDER BY created_at")
                    .bind(&material_id_for_closure)
                    .fetch_all(pool)
                    .await?;
                
                rows.iter()
                    .map(Self::map_row_to_swatch)
                    .collect::<std::result::Result<Vec<_>, _>>()
            })
        }).await
    }

    async fn delete_swatch(&self, swatch_id: &str) -> Result<()> {
        debug!("Deleting swatch with id: {}", swatch_id);
        
        // Using execute_query_in_transaction for this deletion operation to:
        // 1. Ensure atomic deletion with proper transaction handling
        // 2. Check affected rows to determine if the swatch existed
        // 3. Map to appropriate domain error (SwatchNotFound) when no rows affected
        
        // Clone to avoid borrowing issues
        let id_for_closure = swatch_id.to_string();
        
        // Use a query that returns the count of affected rows 
        let rows_affected = self.execute_query_in_transaction(move |tx| {
            Box::pin(async move {
                let result = sqlx::query("DELETE FROM swatches WHERE id = ?")
                    .bind(&id_for_closure)
                    .execute(&mut **tx)
                    .await?;
                
                Ok(result.rows_affected())
            })
        }).await?;
        
        // Check if any rows were affected
        if rows_affected == 0 {
            debug!("Attempted to delete non-existent swatch: {}", swatch_id);
            return Err(SwatchRepositoryError::SwatchNotFound(swatch_id.into()));
        }
        
        Ok(())
    }

    async fn delete_swatches_by_cut_id(&self, cut_id: &str) -> Result<()> {
        debug!("Deleting swatches by cut_id: {}", cut_id);
        
        // Using execute_query_in_transaction for this bulk deletion operation to:
        // 1. Ensure all deletions happen atomically in a single transaction
        // 2. Provide proper error handling and logging of affected row counts
        // 3. Allow transaction to be rolled back automatically on error
        
        // Clone for use in closure
        let cut_id_for_closure = cut_id.to_string();
        
        self.execute_query_in_transaction(move |tx| {
            Box::pin(async move {
                let result = sqlx::query("DELETE FROM swatches WHERE cut_id = ?")
                    .bind(&cut_id_for_closure)
                    .execute(&mut **tx)
                    .await?;
                
                debug!(
                    "Deleted {} swatches for cut_id {}",
                    result.rows_affected(),
                    cut_id_for_closure
                );
                
                Ok(result)
            })
        }).await.map(|_| ())
    }

    async fn delete_swatches_by_material_id(&self, material_id: &str) -> Result<()> {
        debug!("Deleting swatches by material_id: {}", material_id);
        
        // Using execute_query_in_transaction for this bulk deletion operation to:
        // 1. Ensure all deletions happen atomically in a single transaction
        // 2. Provide proper error handling and logging of affected row counts
        // 3. Allow transaction to be rolled back automatically on error
        
        // Clone for use in closure
        let material_id_for_closure = material_id.to_string();
        
        self.execute_query_in_transaction(move |tx| {
            Box::pin(async move {
                let result = sqlx::query("DELETE FROM swatches WHERE material_id = ?")
                    .bind(&material_id_for_closure)
                    .execute(&mut **tx)
                    .await?;
                
                debug!(
                    "Deleted {} swatches for material_id {}",
                    result.rows_affected(),
                    material_id_for_closure
                );
                
                Ok(result)
            })
        }).await.map(|_| ())
    }

    async fn search_similar(
        &self,
        _embedding: &[f32], // Mark as unused
        _limit: usize,      // Mark as unused
        _min_score: Option<f32>,
    ) -> Result<Vec<(Swatch, f32)>> {
        // Stubbed out as per plan
        debug!("search_similar called but not implemented for SQLite");
        Err(SwatchRepositoryError::OperationFailed(
            "Vector search not implemented for SQLite repository".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cutting::{Cut, CutsRepository, SqliteCutsRepository};
    use crate::db::init_memory_db;
    use crate::materials::{Material, MaterialRepository, SqliteMaterialRepository};
    use serde_json::json;
    use time::Duration;

    // Helper to create a test pool
    async fn setup() -> SqlitePool {
        init_memory_db().await.expect("Failed to init test DB")
    }

    // Helper to create a unique swatch with default threshold
    // Note: This no longer saves dependencies; tests must do that.
    fn create_test_swatch(cut_id: &str, material_id: &str) -> Swatch {
        let mut s = Swatch::new(
            cut_id.to_string(),
            material_id.to_string(),
            vec![0.1, 0.2, 0.3],
            "test-model".to_string(),
            "v1.0".to_string(),
        );
        s.similarity_threshold = Some(0.85); // Add a default threshold for tests
        s
    }

    // Helper to create a swatch with metadata and threshold
    // Note: This no longer saves dependencies; tests must do that.
    fn create_test_swatch_with_metadata(cut_id: &str, material_id: &str) -> Swatch {
        let mut swatch = create_test_swatch(cut_id, material_id);
        swatch.metadata = Some(json!({ "key": "value", "number": 123 }));
        swatch.similarity_threshold = Some(0.9); // Can override default if needed
        swatch
    }

    // Helper to insert prerequisite Material and Cut
    async fn insert_test_dependencies(
        pool: &SqlitePool,
        material_path_suffix: &str,
        cut_index: usize,
    ) -> (
        SqliteMaterialRepository,
        SqliteCutsRepository,
        String,
        String,
    ) {
        let material_repo = SqliteMaterialRepository::new(pool.clone());
        let cuts_repo = SqliteCutsRepository::new(pool.clone());

        let material = Material::new(format!("test/mat-{}.txt", material_path_suffix));
        let material_id = material.id.clone();
        material_repo
            .register_material(material)
            .await
            .expect("Failed to save test material");

        let cut = Cut::new(
            material_id.clone(),
            cut_index,
            format!("Test cut content {}", cut_index),
        );
        let cut_id = cut.id.clone();
        cuts_repo
            .save_cut(&cut)
            .await
            .expect("Failed to save test cut");

        (material_repo, cuts_repo, material_id, cut_id)
    }

    #[tokio::test]
    async fn test_save_and_get_swatch() {
        let pool = setup().await;
        let swatch_repo = SqliteSwatchRepository::new(pool.clone());
        let (_material_repo, _cuts_repo, material_id, cut_id) =
            insert_test_dependencies(&pool, "save-get", 0).await;

        let swatch = create_test_swatch(&cut_id, &material_id);

        swatch_repo
            .save_swatch(&swatch)
            .await
            .expect("Failed to save");

        let retrieved_swatch = swatch_repo
            .get_swatch_by_id(&swatch.id)
            .await
            .expect("Failed to get")
            .expect("Swatch not found");

        // Compare fields individually
        assert_eq!(retrieved_swatch.id, swatch.id);
        assert_eq!(retrieved_swatch.cut_id, cut_id);
        assert_eq!(retrieved_swatch.material_id, material_id);
        assert_eq!(retrieved_swatch.embedding, swatch.embedding);
        assert_eq!(retrieved_swatch.model_name, swatch.model_name);
        assert_eq!(retrieved_swatch.model_version, swatch.model_version);
        assert_eq!(retrieved_swatch.dimensions, swatch.dimensions);
        assert!(retrieved_swatch.metadata.is_none()); // Metadata is none in base helper
        assert_eq!(retrieved_swatch.similarity_threshold, Some(0.85)); // Check threshold
                                                                       // Allow a small tolerance for timestamp comparison
        assert!((retrieved_swatch.created_at - swatch.created_at).abs() < Duration::seconds(1));
    }

    #[tokio::test]
    async fn test_save_and_get_swatch_with_metadata() {
        let pool = setup().await;
        let swatch_repo = SqliteSwatchRepository::new(pool.clone());
        let (_material_repo, _cuts_repo, material_id, cut_id) =
            insert_test_dependencies(&pool, "save-get-meta", 0).await;

        let swatch = create_test_swatch_with_metadata(&cut_id, &material_id);

        swatch_repo
            .save_swatch(&swatch)
            .await
            .expect("Failed to save");

        let retrieved_swatch = swatch_repo
            .get_swatch_by_id(&swatch.id)
            .await
            .expect("Failed to get")
            .expect("Swatch not found");

        assert_eq!(retrieved_swatch.id, swatch.id);
        assert_eq!(retrieved_swatch.cut_id, cut_id);
        assert_eq!(retrieved_swatch.material_id, material_id);
        assert_eq!(retrieved_swatch.metadata, swatch.metadata);
        assert_eq!(retrieved_swatch.similarity_threshold, Some(0.9)); // Check threshold
    }

    #[tokio::test]
    async fn test_get_swatch_not_found() {
        let pool = setup().await;
        let swatch_repo = SqliteSwatchRepository::new(pool.clone());
        // No dependencies inserted for this test
        let non_existent_id = cuid2::cuid(); // Generate a valid CUID

        let result = swatch_repo.get_swatch_by_id(&non_existent_id).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_save_swatch_upsert() {
        let pool = setup().await;
        let swatch_repo = SqliteSwatchRepository::new(pool.clone());
        let (_material_repo, _cuts_repo, material_id, cut_id) =
            insert_test_dependencies(&pool, "upsert", 0).await;

        let mut swatch = create_test_swatch(&cut_id, &material_id);
        swatch.similarity_threshold = Some(0.7); // Initial threshold

        // Initial save
        swatch_repo
            .save_swatch(&swatch)
            .await
            .expect("Initial save failed");

        // Modify and save again (upsert)
        swatch.model_version = "v1.1".to_string();
        swatch.embedding = vec![0.4, 0.5];
        swatch.dimensions = swatch.embedding.len();
        swatch.similarity_threshold = Some(0.75); // Update threshold

        swatch_repo
            .save_swatch(&swatch)
            .await
            .expect("Upsert save failed");

        let retrieved_swatch = swatch_repo
            .get_swatch_by_id(&swatch.id)
            .await
            .expect("Failed to get after upsert")
            .expect("Swatch not found after upsert");

        assert_eq!(retrieved_swatch.id, swatch.id);
        assert_eq!(retrieved_swatch.model_version, "v1.1");
        assert_eq!(retrieved_swatch.embedding, vec![0.4, 0.5]);
        assert_eq!(retrieved_swatch.dimensions, 2);
        assert_eq!(retrieved_swatch.similarity_threshold, Some(0.75)); // Check updated threshold
    }

    #[tokio::test]
    async fn test_save_batch_and_get() {
        let pool = setup().await;
        let swatch_repo = SqliteSwatchRepository::new(pool.clone());
        let material_repo = SqliteMaterialRepository::new(pool.clone());
        let cuts_repo = SqliteCutsRepository::new(pool.clone());

        // --- Swatch 1 dependencies ---
        let material1 = Material::new("test/mat-batch1.txt".to_string());
        let material_id1 = material1.id.clone();
        material_repo
            .register_material(material1)
            .await
            .expect("Save mat1");
        let cut1 = Cut::new(material_id1.clone(), 0, "Cut 1".to_string());
        let cut_id1 = cut1.id.clone();
        cuts_repo.save_cut(&cut1).await.expect("Save cut1");
        let swatch1 = create_test_swatch(&cut_id1, &material_id1);

        // --- Swatch 2 dependencies ---
        let material2 = Material::new("test/mat-batch2.txt".to_string());
        let material_id2 = material2.id.clone();
        material_repo
            .register_material(material2)
            .await
            .expect("Save mat2");
        let cut2 = Cut::new(material_id2.clone(), 0, "Cut 2".to_string());
        let cut_id2 = cut2.id.clone();
        cuts_repo.save_cut(&cut2).await.expect("Save cut2");
        let swatch2 = create_test_swatch_with_metadata(&cut_id2, &material_id2);

        let swatches = vec![swatch1.clone(), swatch2.clone()];

        swatch_repo
            .save_swatches_batch(&swatches)
            .await
            .expect("Batch save failed");

        let retrieved1 = swatch_repo
            .get_swatch_by_id(&swatch1.id)
            .await
            .expect("Failed to get swatch1")
            .expect("Swatch1 not found");
        let retrieved2 = swatch_repo
            .get_swatch_by_id(&swatch2.id)
            .await
            .expect("Failed to get swatch2")
            .expect("Swatch2 not found");

        assert_eq!(retrieved1.id, swatch1.id);
        assert_eq!(retrieved2.id, swatch2.id);
        assert_eq!(retrieved2.metadata, swatch2.metadata);
        assert_eq!(
            retrieved2.similarity_threshold,
            swatch2.similarity_threshold
        );
    }

    #[tokio::test]
    async fn test_get_swatches_by_cut_id() {
        let pool = setup().await;
        let swatch_repo = SqliteSwatchRepository::new(pool.clone());
        let material_repo = SqliteMaterialRepository::new(pool.clone());
        let cuts_repo = SqliteCutsRepository::new(pool.clone());

        // --- Shared Material ---
        let material_shared = Material::new("test/mat-get-by-cut.txt".to_string());
        let material_id_shared = material_shared.id.clone();
        material_repo
            .register_material(material_shared)
            .await
            .expect("Save mat-shared");

        // --- Cut 1 (Shared) ---
        let cut1 = Cut::new(material_id_shared.clone(), 0, "Cut 1".to_string());
        let cut_id1 = cut1.id.clone();
        cuts_repo.save_cut(&cut1).await.expect("Save cut1");
        let swatch1 = create_test_swatch(&cut_id1, &material_id_shared);

        // --- Cut 2 (Shared) ---
        let cut2 = Cut::new(material_id_shared.clone(), 1, "Cut 2".to_string());
        let cut_id2 = cut2.id.clone();
        cuts_repo.save_cut(&cut2).await.expect("Save cut2");
        let swatch2 = create_test_swatch(&cut_id2, &material_id_shared);

        // Test getting by Cut ID 3
        let material_other = Material::new("test/mat-other.txt".to_string());
        let material_id_other = material_other.id.clone();
        material_repo
            .register_material(material_other)
            .await
            .expect("Save mat-other");
        let cut3 = Cut::new(material_id_other.clone(), 0, "Cut 3".to_string());
        let cut_id3 = cut3.id.clone();
        cuts_repo.save_cut(&cut3).await.expect("Save cut3");
        let swatch3 = create_test_swatch(&cut_id3, &material_id_other);

        swatch_repo
            .save_swatches_batch(&[swatch1.clone(), swatch2.clone(), swatch3.clone()])
            .await
            .expect("Batch save failed");

        // Test getting by Cut ID 1
        let results1 = swatch_repo
            .get_swatches_by_cut_id(&cut_id1)
            .await
            .expect("Failed to get by cut_id1");
        assert_eq!(results1.len(), 1);
        assert_eq!(results1[0].id, swatch1.id);

        // Test getting by Cut ID 2
        let results2 = swatch_repo
            .get_swatches_by_cut_id(&cut_id2)
            .await
            .expect("Failed to get by cut_id2");
        assert_eq!(results2.len(), 1);
        assert_eq!(results2[0].id, swatch2.id);

        // Test getting by Cut ID 3
        let results3 = swatch_repo
            .get_swatches_by_cut_id(&cut_id3)
            .await
            .expect("Failed to get by cut_id3");
        assert_eq!(results3.len(), 1);
        assert_eq!(results3[0].id, swatch3.id);
    }

    #[tokio::test]
    async fn test_get_swatches_by_material_id() {
        let pool = setup().await;
        let swatch_repo = SqliteSwatchRepository::new(pool.clone());
        let material_repo = SqliteMaterialRepository::new(pool.clone());
        let cuts_repo = SqliteCutsRepository::new(pool.clone());

        // --- Material 1 (Shared) ---
        let material1 = Material::new("test/mat-shared.txt".to_string());
        let material_id1 = material1.id.clone();
        material_repo
            .register_material(material1)
            .await
            .expect("Save mat1");

        // --- Cut A (Material 1) ---
        let cut_a = Cut::new(material_id1.clone(), 0, "Cut A".to_string());
        let cut_id_a = cut_a.id.clone();
        cuts_repo.save_cut(&cut_a).await.expect("Save cutA");
        let swatch_a = create_test_swatch(&cut_id_a, &material_id1);

        // --- Cut B (Material 1) ---
        let cut_b = Cut::new(material_id1.clone(), 1, "Cut B".to_string());
        let cut_id_b = cut_b.id.clone();
        cuts_repo.save_cut(&cut_b).await.expect("Save cutB");
        let swatch_b = create_test_swatch(&cut_id_b, &material_id1);

        // --- Material 2 (Different) ---
        let material2 = Material::new("test/mat-different.txt".to_string());
        let material_id2 = material2.id.clone();
        material_repo
            .register_material(material2)
            .await
            .expect("Save mat2");

        // --- Cut C (Material 2) ---
        let cut_c = Cut::new(material_id2.clone(), 0, "Cut C".to_string());
        let cut_id_c = cut_c.id.clone();
        cuts_repo.save_cut(&cut_c).await.expect("Save cutC");
        let swatch_c = create_test_swatch(&cut_id_c, &material_id2);

        swatch_repo
            .save_swatches_batch(&[swatch_a.clone(), swatch_b.clone(), swatch_c.clone()])
            .await
            .expect("Batch save failed");

        let results = swatch_repo
            .get_swatches_by_material_id(&material_id1)
            .await
            .expect("Failed to get by material_id");

        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|s| s.id == swatch_a.id));
        assert!(results.iter().any(|s| s.id == swatch_b.id));
        assert!(!results.iter().any(|s| s.id == swatch_c.id)); // Ensure swatchC is not included
    }

    #[tokio::test]
    async fn test_delete_swatch() {
        let pool = setup().await;
        let swatch_repo = SqliteSwatchRepository::new(pool.clone());
        let (_material_repo, _cuts_repo, material_id, cut_id) =
            insert_test_dependencies(&pool, "del-sw", 0).await;

        let swatch = create_test_swatch(&cut_id, &material_id);

        swatch_repo.save_swatch(&swatch).await.expect("Save failed");

        // Delete
        let result = swatch_repo.delete_swatch(&swatch.id).await;
        assert!(result.is_ok(), "Delete failed: {:?}", result.err());

        // Verify deleted
        let retrieved = swatch_repo
            .get_swatch_by_id(&swatch.id)
            .await
            .expect("Get after delete failed");
        assert!(retrieved.is_none(), "Swatch was not deleted");

        // Test deleting non-existent
        let result = swatch_repo.delete_swatch(&swatch.id).await; // Delete again
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            SwatchRepositoryError::SwatchNotFound(_)
        ));
    }

    #[tokio::test]
    async fn test_delete_swatches_by_cut_id() {
        let pool = setup().await;
        let swatch_repo = SqliteSwatchRepository::new(pool.clone());
        let material_repo = SqliteMaterialRepository::new(pool.clone());
        let cuts_repo = SqliteCutsRepository::new(pool.clone());

        // --- Material X ---
        let mat_x = Material::new("test/del-cut-matX.txt".to_string());
        let mat_x_id = mat_x.id.clone();
        material_repo
            .register_material(mat_x)
            .await
            .expect("Save matX");

        // --- Cut 1 (matX - to be deleted) ---
        let cut1 = Cut::new(mat_x_id.clone(), 0, "Cut 1".to_string());
        let cut1_id = cut1.id.clone();
        cuts_repo.save_cut(&cut1).await.expect("Save cut1");
        let swatch1 = create_test_swatch(&cut1_id, &mat_x_id);

        // --- Cut 2 (matX - different cut, same material - should be deleted too) ---
        // Re-use the same cut ID for deletion test
        let cut2 = Cut::new(mat_x_id.clone(), 1, "Cut 2".to_string());
        let cut2_id = cut2.id.clone();
        cuts_repo.save_cut(&cut2).await.expect("Save cut2");
        let swatch2 = create_test_swatch(&cut2_id, &mat_x_id);

        // --- Material Y ---
        let mat_y = Material::new("test/del-cut-matY.txt".to_string());
        let mat_y_id = mat_y.id.clone();
        material_repo
            .register_material(mat_y)
            .await
            .expect("Save matY");

        // --- Cut 3 (matY - different cut, should remain) ---
        let cut3 = Cut::new(mat_y_id.clone(), 0, "Cut 3".to_string());
        let cut3_id = cut3.id.clone();
        cuts_repo.save_cut(&cut3).await.expect("Save cut3");
        let swatch3 = create_test_swatch(&cut3_id, &mat_y_id);

        swatch_repo
            .save_swatches_batch(&[swatch1.clone(), swatch2.clone(), swatch3.clone()])
            .await
            .expect("Batch save failed");

        // Delete by cut_id (only deletes swatches associated with cut1_id)
        let result = swatch_repo.delete_swatches_by_cut_id(&cut1_id).await;
        assert!(
            result.is_ok(),
            "Delete by cut_id failed: {:?}",
            result.err()
        );

        // Verify deleted
        let retrieved1 = swatch_repo
            .get_swatch_by_id(&swatch1.id)
            .await
            .expect("Get s1 failed");
        let retrieved2 = swatch_repo
            .get_swatch_by_id(&swatch2.id)
            .await
            .expect("Get s2 failed");
        let retrieved3 = swatch_repo
            .get_swatch_by_id(&swatch3.id)
            .await
            .expect("Get s3 failed");

        assert!(retrieved1.is_none(), "Swatch1 not deleted by cut_id");
        assert!(
            retrieved2.is_some(),
            "Swatch2 (different cut_id) was incorrectly deleted"
        );
        assert!(
            retrieved3.is_some(),
            "Swatch3 (different cut_id) was incorrectly deleted"
        );
    }

    #[tokio::test]
    async fn test_delete_swatches_by_material_id() {
        let pool = setup().await;
        let swatch_repo = SqliteSwatchRepository::new(pool.clone());
        let material_repo = SqliteMaterialRepository::new(pool.clone());
        let cuts_repo = SqliteCutsRepository::new(pool.clone());

        // --- Material 1 (To be deleted) ---
        let mat1 = Material::new("test/del-mat-1.txt".to_string());
        let mat1_id = mat1.id.clone();
        material_repo
            .register_material(mat1)
            .await
            .expect("Save mat1");

        // --- Cut X (mat1) ---
        let cut_x = Cut::new(mat1_id.clone(), 0, "Cut X".to_string());
        let cut_x_id = cut_x.id.clone();
        cuts_repo.save_cut(&cut_x).await.expect("Save cutX");
        let swatch1 = create_test_swatch(&cut_x_id, &mat1_id);

        // --- Cut Y (mat1) ---
        let cut_y = Cut::new(mat1_id.clone(), 1, "Cut Y".to_string());
        let cut_y_id = cut_y.id.clone();
        cuts_repo.save_cut(&cut_y).await.expect("Save cutY");
        let swatch2 = create_test_swatch(&cut_y_id, &mat1_id);

        // --- Material 2 (Different) ---
        let mat2 = Material::new("test/del-mat-other.txt".to_string());
        let mat2_id = mat2.id.clone();
        material_repo
            .register_material(mat2)
            .await
            .expect("Save mat2");

        // --- Cut Z (mat2) ---
        let cut_z = Cut::new(mat2_id.clone(), 0, "Cut Z".to_string());
        let cut_z_id = cut_z.id.clone();
        cuts_repo.save_cut(&cut_z).await.expect("Save cutZ");
        let swatch3 = create_test_swatch(&cut_z_id, &mat2_id);

        swatch_repo
            .save_swatches_batch(&[swatch1.clone(), swatch2.clone(), swatch3.clone()])
            .await
            .expect("Batch save failed");

        // Delete by material_id (mat1_id)
        let result = swatch_repo.delete_swatches_by_material_id(&mat1_id).await;
        assert!(
            result.is_ok(),
            "Delete by material_id failed: {:?}",
            result.err()
        );

        // Verify deleted
        let retrieved1 = swatch_repo
            .get_swatch_by_id(&swatch1.id)
            .await
            .expect("Get s1 failed");
        let retrieved2 = swatch_repo
            .get_swatch_by_id(&swatch2.id)
            .await
            .expect("Get s2 failed");
        let retrieved3 = swatch_repo
            .get_swatch_by_id(&swatch3.id)
            .await
            .expect("Get s3 failed");

        assert!(retrieved1.is_none(), "Swatch1 not deleted by material_id");
        assert!(retrieved2.is_none(), "Swatch2 not deleted by material_id");
        assert!(
            retrieved3.is_some(),
            "Swatch3 (different material_id) was incorrectly deleted"
        );
    }

    #[tokio::test]
    async fn test_search_similar_stubbed() {
        let pool = setup().await;
        let repo = SqliteSwatchRepository::new(pool.clone());
        // No dependencies needed as it should error out anyway
        let dummy_embedding = vec![0.0; 3];

        let result = repo.search_similar(&dummy_embedding, 10, None).await;

        assert!(result.is_err());
        match result.err().unwrap() {
            SwatchRepositoryError::OperationFailed(msg) => {
                assert!(msg.to_string().contains("not implemented for SQLite"));
            }
            _ => panic!("Expected OperationFailed error"),
        }
    }

    #[tokio::test]
    async fn test_transaction_helper() {
        let pool = setup().await;
        let repo = SqliteSwatchRepository::new(pool.clone());
        
        // Test successful transaction
        let result = repo.execute_in_transaction(|tx| Box::pin(async move {
            // Perform some operation within the transaction
            sqlx::query("INSERT INTO test_tx_table (id, value) VALUES (?, ?)")
                .bind("test1")
                .bind("value1")
                .execute(&mut **tx)
                .await
                .map_err(|e| SwatchRepositoryError::OperationFailed(e.to_string().into()))?;
            
            Ok("success")
        })).await;
        
        assert!(result.is_err(), "Expected error since test_tx_table doesn't exist");
        
        // Create a test table for transaction testing
        sqlx::query("CREATE TABLE test_tx_table (id TEXT PRIMARY KEY, value TEXT)")
            .execute(&pool)
            .await
            .unwrap();
        
        // Test successful transaction again
        let result = repo.execute_in_transaction(|tx| Box::pin(async move {
            // Perform some operation within the transaction
            sqlx::query("INSERT INTO test_tx_table (id, value) VALUES (?, ?)")
                .bind("test1")
                .bind("value1")
                .execute(&mut **tx)
                .await
                .map_err(|e| SwatchRepositoryError::OperationFailed(e.to_string().into()))?;
            
            Ok("success")
        })).await;
        
        assert!(result.is_ok(), "Transaction should succeed");
        assert_eq!(result.unwrap(), "success");
        
        // Verify data was committed
        let row = sqlx::query("SELECT value FROM test_tx_table WHERE id = ?")
            .bind("test1")
            .fetch_one(&pool)
            .await
            .unwrap();
        
        let value: String = row.try_get("value").unwrap();
        assert_eq!(value, "value1");
        
        // Test transaction rollback
        let result: Result<&str> = repo.execute_in_transaction(|tx| Box::pin(async move {
            // Insert data
            sqlx::query("INSERT INTO test_tx_table (id, value) VALUES (?, ?)")
                .bind("test2")
                .bind("value2")
                .execute(&mut **tx)
                .await
                .map_err(|e| SwatchRepositoryError::OperationFailed(e.to_string().into()))?;
            
            // Return an error to trigger rollback
            Err(SwatchRepositoryError::OperationFailed("Simulated error".into()))
        })).await;
        
        assert!(result.is_err());
        
        // Verify data was rolled back
        let result = sqlx::query("SELECT value FROM test_tx_table WHERE id = ?")
            .bind("test2")
            .fetch_optional(&pool)
            .await
            .unwrap();
        
        assert!(result.is_none(), "Transaction should have been rolled back");
    }

    #[tokio::test]
    async fn test_query_transaction_helper() {
        let pool = setup().await;
        let repo = SqliteSwatchRepository::new(pool.clone());
        
        // Create a test table
        sqlx::query("CREATE TABLE test_query_tx (id TEXT PRIMARY KEY, value TEXT)")
            .execute(&pool)
            .await
            .unwrap();
        
        // Test successful query execution
        let result: Result<sqlx::sqlite::SqliteQueryResult> = repo.execute_query_in_transaction(|tx| Box::pin(async move {
            sqlx::query("INSERT INTO test_query_tx (id, value) VALUES (?, ?)")
                .bind("test-query-1")
                .bind("value-1")
                .execute(&mut **tx)
                .await
        })).await;
        
        assert!(result.is_ok(), "Query execution should succeed");
        
        // Verify data was committed
        let row = sqlx::query("SELECT value FROM test_query_tx WHERE id = ?")
            .bind("test-query-1")
            .fetch_one(&pool)
            .await
            .unwrap();
        
        let value: String = row.try_get("value").unwrap();
        assert_eq!(value, "value-1");
        
        // Test unique constraint violation mapping
        let result: Result<sqlx::sqlite::SqliteQueryResult> = repo.execute_query_in_transaction(|tx| Box::pin(async move {
            // Try to insert with the same primary key
            sqlx::query("INSERT INTO test_query_tx (id, value) VALUES (?, ?)")
                .bind("test-query-1") // Same ID, which will violate the primary key constraint
                .bind("different-value")
                .execute(&mut **tx)
                .await
        })).await;
        
        match result {
            Err(SwatchRepositoryError::SwatchAlreadyExists(_)) => {
                // This is the expected error for a unique constraint violation
            },
            _ => panic!("Expected a SwatchAlreadyExists error, got: {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_read_query_helper() {
        let pool = setup().await;
        let repo = SqliteSwatchRepository::new(pool.clone());
        
        // Create a test table for read query testing
        sqlx::query("CREATE TABLE test_read_query (id TEXT PRIMARY KEY, value TEXT)")
            .execute(&pool)
            .await
            .unwrap();
        
        // Insert some test data
        sqlx::query("INSERT INTO test_read_query (id, value) VALUES (?, ?)")
            .bind("test-read-1")
            .bind("read-value-1")
            .execute(&pool)
            .await
            .unwrap();
        
        // Test successful read query execution
        let result: Result<Option<String>> = repo.execute_read_query(|pool| {
            Box::pin(async move {
                let row = sqlx::query("SELECT value FROM test_read_query WHERE id = ?")
                    .bind("test-read-1")
                    .fetch_optional(pool)
                    .await?;
                
                match row {
                    Some(row) => Ok::<Option<String>, sqlx::Error>(Some(row.try_get("value")?)),
                    None => Ok::<Option<String>, sqlx::Error>(None)
                }
            })
        }).await;
        
        assert!(result.is_ok(), "Read query execution should succeed");
        assert_eq!(result.unwrap(), Some("read-value-1".to_string()));
        
        // Test read for non-existent entry
        let result: Result<Option<String>> = repo.execute_read_query(|pool| {
            Box::pin(async move {
                let row = sqlx::query("SELECT value FROM test_read_query WHERE id = ?")
                    .bind("non-existent-id")
                    .fetch_optional(pool)
                    .await?;
                
                match row {
                    Some(row) => Ok::<Option<String>, sqlx::Error>(Some(row.try_get("value")?)),
                    None => Ok::<Option<String>, sqlx::Error>(None)
                }
            })
        }).await;
        
        assert!(result.is_ok(), "Read query execution for non-existent entry should succeed");
        assert_eq!(result.unwrap(), None);
        
        // Test error handling (query against non-existent table)
        let result: Result<Option<String>> = repo.execute_read_query(|pool| {
            Box::pin(async move {
                let row = sqlx::query("SELECT value FROM non_existent_table WHERE id = ?")
                    .bind("any-id")
                    .fetch_optional(pool)
                    .await?;
                
                match row {
                    Some(row) => Ok::<Option<String>, sqlx::Error>(Some(row.try_get("value")?)),
                    None => Ok::<Option<String>, sqlx::Error>(None)
                }
            })
        }).await;
        
        assert!(result.is_err(), "Query against non-existent table should fail");
        match result {
            Err(SwatchRepositoryError::OperationFailed(_)) => {
                // This is the expected error for a read query failure
            },
            _ => panic!("Expected an OperationFailed error, got: {:?}", result),
        }
    }
}
