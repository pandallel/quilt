use async_trait::async_trait;
use sqlx::{sqlite::SqliteRow, Row, SqlitePool};
use std::fmt::Debug;
use tracing::{debug, error};

use super::repository::{Result, SwatchRepository, SwatchRepositoryError};
use super::swatch::Swatch;

// Helper function to serialize Vec<f32> to Vec<u8>
// Uses native endianness for potentially better performance on the same architecture.
fn f32_vec_to_bytes(vec: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(vec.len() * std::mem::size_of::<f32>());
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

    // Helper to map SqliteRow to Swatch
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
        })
    }
}

#[async_trait]
impl SwatchRepository for SqliteSwatchRepository {
    async fn save_swatch(&self, swatch: &Swatch) -> Result<()> {
        debug!("Saving swatch with id: {}", swatch.id);
        let embedding_bytes = f32_vec_to_bytes(&swatch.embedding);
        let metadata_json = swatch
            .metadata
            .as_ref()
            .map(|v| serde_json::to_string(v))
            .transpose()
            .map_err(|e| SwatchRepositoryError::OperationFailed(format!("Failed to serialize metadata: {}", e).into()))?;

        let result = sqlx::query(
            r#"
            INSERT INTO swatches (
                id, cut_id, material_id, embedding, model_name, model_version, 
                created_at, dimensions, metadata
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                cut_id = excluded.cut_id,
                material_id = excluded.material_id,
                embedding = excluded.embedding,
                model_name = excluded.model_name,
                model_version = excluded.model_version,
                created_at = excluded.created_at, -- Consider if this should be updated
                dimensions = excluded.dimensions,
                metadata = excluded.metadata
            "#,
        )
        .bind(&swatch.id)
        .bind(&swatch.cut_id)
        .bind(&swatch.material_id)
        .bind(embedding_bytes)
        .bind(&swatch.model_name)
        .bind(&swatch.model_version)
        .bind(swatch.created_at) // Assuming OffsetDateTime
        .bind(swatch.dimensions as i64) // Store usize as i64
        .bind(metadata_json)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
                // This case should ideally be handled by ON CONFLICT, but log just in case
                error!("Unique constraint violation for swatch {}: {}", swatch.id, db_err);
                Err(SwatchRepositoryError::SwatchAlreadyExists(swatch.id.clone().into()))
            }
            Err(e) => {
                error!("Failed to save swatch {}: {}", swatch.id, e);
                Err(SwatchRepositoryError::OperationFailed(e.to_string().into()))
            }
        }
    }

    async fn save_swatches_batch(&self, swatches: &[Swatch]) -> Result<()> {
        debug!("Saving batch of {} swatches", swatches.len());
        // Use a transaction for atomicity
        let mut tx = self.pool.begin().await.map_err(|e| {
            SwatchRepositoryError::OperationFailed(format!("Failed to begin transaction: {}", e).into())
        })?;

        for swatch in swatches {
            let embedding_bytes = f32_vec_to_bytes(&swatch.embedding);
            let metadata_json = swatch
                .metadata
                .as_ref()
                .map(|v| serde_json::to_string(v))
                .transpose()
                .map_err(|e| SwatchRepositoryError::OperationFailed(format!("Failed to serialize metadata for {}: {}", swatch.id, e).into()))?;

            sqlx::query(
                 r#"
                INSERT INTO swatches (
                    id, cut_id, material_id, embedding, model_name, model_version, 
                    created_at, dimensions, metadata
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(id) DO UPDATE SET
                    cut_id = excluded.cut_id,
                    material_id = excluded.material_id,
                    embedding = excluded.embedding,
                    model_name = excluded.model_name,
                    model_version = excluded.model_version,
                    created_at = excluded.created_at,
                    dimensions = excluded.dimensions,
                    metadata = excluded.metadata
                "#,
            )
            .bind(&swatch.id)
            .bind(&swatch.cut_id)
            .bind(&swatch.material_id)
            .bind(embedding_bytes)
            .bind(&swatch.model_name)
            .bind(&swatch.model_version)
            .bind(swatch.created_at)
            .bind(swatch.dimensions as i64)
            .bind(metadata_json)
            .execute(&mut *tx) // Execute within the transaction
            .await
            .map_err(|e| {
                 match e {
                    sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                         error!("Batch save failed on unique constraint for {}: {}", swatch.id, db_err);
                         SwatchRepositoryError::SwatchAlreadyExists(swatch.id.clone().into())
                    },
                    _ => {
                        error!("Failed to save swatch {} in batch: {}", swatch.id, e);
                        SwatchRepositoryError::OperationFailed(
                            format!("Failed during batch save for {}: {}", swatch.id, e).into(),
                        )
                    }
                 }
            })?;
        }

        tx.commit().await.map_err(|e| {
            error!("Failed to commit swatch batch transaction: {}", e);
            SwatchRepositoryError::OperationFailed(format!("Failed to commit transaction: {}", e).into())
        })
    }

    async fn get_swatch_by_id(&self, swatch_id: &str) -> Result<Option<Swatch>> {
        debug!("Getting swatch by id: {}", swatch_id);
        let result = sqlx::query("SELECT * FROM swatches WHERE id = ?")
            .bind(swatch_id)
            .fetch_optional(&self.pool)
            .await;

        match result {
            Ok(Some(row)) => {
                match Self::map_row_to_swatch(&row) {
                    Ok(swatch) => Ok(Some(swatch)),
                    Err(e) => {
                        error!("Failed to map row to swatch {}: {}", swatch_id, e);
                        Err(SwatchRepositoryError::OperationFailed(format!("Data corruption for swatch {}: {}", swatch_id, e).into()))
                    }
                }
            }
            Ok(None) => Ok(None), // Not found is not an error
            Err(e) => {
                error!("Failed to get swatch {}: {}", swatch_id, e);
                Err(SwatchRepositoryError::OperationFailed(e.to_string().into()))
            }
        }
    }

    async fn get_swatches_by_cut_id(&self, cut_id: &str) -> Result<Vec<Swatch>> {
        debug!("Getting swatches by cut_id: {}", cut_id);
        let result = sqlx::query("SELECT * FROM swatches WHERE cut_id = ? ORDER BY created_at")
            .bind(cut_id)
            .fetch_all(&self.pool)
            .await;

        match result {
            Ok(rows) => rows
                .iter()
                .map(Self::map_row_to_swatch)
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(|e| {
                    error!("Failed to map rows for cut {}: {}", cut_id, e);
                    SwatchRepositoryError::OperationFailed(format!("Data corruption for cut {}: {}", cut_id, e).into())
                }),
            Err(e) => {
                error!("Failed to get swatches for cut {}: {}", cut_id, e);
                Err(SwatchRepositoryError::OperationFailed(e.to_string().into()))
            }
        }
    }

    async fn get_swatches_by_material_id(&self, material_id: &str) -> Result<Vec<Swatch>> {
        debug!("Getting swatches by material_id: {}", material_id);
        let result = sqlx::query("SELECT * FROM swatches WHERE material_id = ? ORDER BY created_at")
            .bind(material_id)
            .fetch_all(&self.pool)
            .await;

        match result {
            Ok(rows) => rows
                .iter()
                .map(Self::map_row_to_swatch)
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(|e| {
                    error!("Failed to map rows for material {}: {}", material_id, e);
                     SwatchRepositoryError::OperationFailed(format!("Data corruption for material {}: {}", material_id, e).into())
                }),
            Err(e) => {
                error!("Failed to get swatches for material {}: {}", material_id, e);
                Err(SwatchRepositoryError::OperationFailed(e.to_string().into()))
            }
        }
    }

    async fn delete_swatch(&self, swatch_id: &str) -> Result<()> {
        debug!("Deleting swatch with id: {}", swatch_id);
        let result = sqlx::query("DELETE FROM swatches WHERE id = ?")
            .bind(swatch_id)
            .execute(&self.pool)
            .await;

        match result {
            Ok(exec_result) => {
                if exec_result.rows_affected() == 0 {
                    // Deleting something that doesn't exist is not strictly an error in some contexts,
                    // but the trait defines SwatchNotFound, so we return it.
                     debug!("Attempted to delete non-existent swatch: {}", swatch_id);
                    Err(SwatchRepositoryError::SwatchNotFound(swatch_id.into()))
                } else {
                    Ok(())
                }
            }
            Err(e) => {
                error!("Failed to delete swatch {}: {}", swatch_id, e);
                Err(SwatchRepositoryError::OperationFailed(e.to_string().into()))
            }
        }
    }

    async fn delete_swatches_by_cut_id(&self, cut_id: &str) -> Result<()> {
        debug!("Deleting swatches by cut_id: {}", cut_id);
        sqlx::query("DELETE FROM swatches WHERE cut_id = ?")
            .bind(cut_id)
            .execute(&self.pool)
            .await
            .map(|exec_result| {
                debug!("Deleted {} swatches for cut_id {}", exec_result.rows_affected(), cut_id);
                ()
            })
            .map_err(|e| {
                error!("Failed to delete swatches for cut {}: {}", cut_id, e);
                SwatchRepositoryError::OperationFailed(e.to_string().into())
            })
    }

    async fn delete_swatches_by_material_id(&self, material_id: &str) -> Result<()> {
        debug!("Deleting swatches by material_id: {}", material_id);
        sqlx::query("DELETE FROM swatches WHERE material_id = ?")
            .bind(material_id)
            .execute(&self.pool)
            .await
            .map(|exec_result| {
                 debug!("Deleted {} swatches for material_id {}", exec_result.rows_affected(), material_id);
                 ()
            })
            .map_err(|e| {
                error!("Failed to delete swatches for material {}: {}", material_id, e);
                SwatchRepositoryError::OperationFailed(e.to_string().into())
            })
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
