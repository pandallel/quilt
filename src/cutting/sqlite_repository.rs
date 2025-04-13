use std::fmt::Debug;

use async_trait::async_trait;
use sqlx::{Row, SqlitePool};
use tracing::{debug, error, info};

use super::{Cut, CutsRepository, CutsRepositoryError, Result};

/// SQLite implementation of the CutsRepository
#[derive(Debug, Clone)]
pub struct SqliteCutsRepository {
    /// Database connection pool
    pool: SqlitePool,
}

impl SqliteCutsRepository {
    /// Create a new SQLite cuts repository
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Convert a database row to a Cut
    fn row_to_cut(row: sqlx::sqlite::SqliteRow) -> Cut {
        let token_count: Option<i64> = row.get("token_count");
        let byte_offset_start: Option<i64> = row.get("byte_offset_start");
        let byte_offset_end: Option<i64> = row.get("byte_offset_end");

        Cut {
            id: row.get("id"),
            material_id: row.get("material_id"),
            chunk_index: row.get::<i64, _>("chunk_index") as usize,
            content: row.get("content"),
            created_at: row.get("created_at"),
            token_count: token_count.map(|v| v as usize),
            byte_offset_start: byte_offset_start.map(|v| v as usize),
            byte_offset_end: byte_offset_end.map(|v| v as usize),
        }
    }
}

#[async_trait]
impl CutsRepository for SqliteCutsRepository {
    async fn save_cut(&self, cut: &Cut) -> Result<()> {
        let cut_id = cut.id.clone();
        let material_id = cut.material_id.clone();

        // Check if the cut already exists
        let existing = sqlx::query("SELECT id FROM cuts WHERE id = ?")
            .bind(&cut_id)
            .fetch_optional(&self.pool)
            .await;

        match existing {
            Ok(Some(_)) => {
                return Err(CutsRepositoryError::CutAlreadyExists(
                    cut_id.into_boxed_str(),
                ));
            }
            Ok(None) => {} // Cut doesn't exist, continue
            Err(e) => {
                error!("Database error checking for existing cut: {}", e);
                return Err(CutsRepositoryError::OperationFailed(
                    e.to_string().into_boxed_str(),
                ));
            }
        }

        // Insert the cut
        let result = sqlx::query(
            r#"
            INSERT INTO cuts (id, material_id, chunk_index, content, created_at, token_count, byte_offset_start, byte_offset_end)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&cut.id)
        .bind(&cut.material_id)
        .bind(cut.chunk_index as i64)
        .bind(&cut.content)
        .bind(cut.created_at)
        .bind(cut.token_count.map(|v| v as i64))
        .bind(cut.byte_offset_start.map(|v| v as i64))
        .bind(cut.byte_offset_end.map(|v| v as i64))
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {
                debug!("Saved cut: {} for material: {}", cut_id, material_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to save cut: {}", e);
                Err(CutsRepositoryError::OperationFailed(
                    e.to_string().into_boxed_str(),
                ))
            }
        }
    }

    async fn save_cuts(&self, cuts: &[Cut]) -> Result<()> {
        if cuts.is_empty() {
            return Ok(());
        }

        // Begin transaction
        let mut tx = match self.pool.begin().await {
            Ok(tx) => tx,
            Err(e) => {
                error!("Failed to begin transaction: {}", e);
                return Err(CutsRepositoryError::OperationFailed(
                    e.to_string().into_boxed_str(),
                ));
            }
        };

        // Check for duplicates first
        for cut in cuts {
            let existing = sqlx::query("SELECT id FROM cuts WHERE id = ?")
                .bind(&cut.id)
                .fetch_optional(&mut *tx)
                .await;

            match existing {
                Ok(Some(_)) => {
                    if let Err(e) = tx.rollback().await {
                        error!("Failed to rollback transaction: {}", e);
                    }
                    return Err(CutsRepositoryError::CutAlreadyExists(
                        cut.id.clone().into_boxed_str(),
                    ));
                }
                Ok(None) => {} // Cut doesn't exist, continue
                Err(e) => {
                    if let Err(rollback_err) = tx.rollback().await {
                        error!("Failed to rollback transaction: {}", rollback_err);
                    }
                    error!("Database error checking for existing cut: {}", e);
                    return Err(CutsRepositoryError::OperationFailed(
                        e.to_string().into_boxed_str(),
                    ));
                }
            }
        }

        // Insert all cuts
        for cut in cuts {
            let result = sqlx::query(
                r#"
                INSERT INTO cuts (id, material_id, chunk_index, content, created_at, token_count, byte_offset_start, byte_offset_end)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&cut.id)
            .bind(&cut.material_id)
            .bind(cut.chunk_index as i64)
            .bind(&cut.content)
            .bind(cut.created_at)
            .bind(cut.token_count.map(|v| v as i64))
            .bind(cut.byte_offset_start.map(|v| v as i64))
            .bind(cut.byte_offset_end.map(|v| v as i64))
            .execute(&mut *tx)
            .await;

            if let Err(e) = result {
                if let Err(rollback_err) = tx.rollback().await {
                    error!("Failed to rollback transaction: {}", rollback_err);
                }
                error!("Failed to save cut: {}", e);
                return Err(CutsRepositoryError::OperationFailed(
                    e.to_string().into_boxed_str(),
                ));
            }
        }

        // Commit transaction
        if let Err(e) = tx.commit().await {
            error!("Failed to commit transaction: {}", e);
            return Err(CutsRepositoryError::OperationFailed(
                e.to_string().into_boxed_str(),
            ));
        }

        info!("Saved {} cuts in batch", cuts.len());
        Ok(())
    }

    async fn get_cut_by_id(&self, cut_id: &str) -> Result<Option<Cut>> {
        let result = sqlx::query("SELECT * FROM cuts WHERE id = ?")
            .bind(cut_id)
            .fetch_optional(&self.pool)
            .await;

        match result {
            Ok(Some(row)) => Ok(Some(Self::row_to_cut(row))),
            Ok(None) => Ok(None),
            Err(e) => {
                error!("Error fetching cut {}: {}", cut_id, e);
                Err(CutsRepositoryError::OperationFailed(
                    e.to_string().into_boxed_str(),
                ))
            }
        }
    }

    async fn get_cuts_by_material_id(&self, material_id: &str) -> Result<Vec<Cut>> {
        let result = sqlx::query("SELECT * FROM cuts WHERE material_id = ? ORDER BY chunk_index")
            .bind(material_id)
            .fetch_all(&self.pool)
            .await;

        match result {
            Ok(rows) => {
                let cuts = rows.into_iter().map(Self::row_to_cut).collect();
                Ok(cuts)
            }
            Err(e) => {
                error!("Error fetching cuts for material {}: {}", material_id, e);
                Err(CutsRepositoryError::OperationFailed(
                    e.to_string().into_boxed_str(),
                ))
            }
        }
    }

    async fn delete_cut(&self, cut_id: &str) -> Result<()> {
        // First check if the cut exists
        let cut = self.get_cut_by_id(cut_id).await?;

        if cut.is_none() {
            return Err(CutsRepositoryError::CutNotFound(
                cut_id.to_string().into_boxed_str(),
            ));
        }

        // Delete the cut
        let result = sqlx::query("DELETE FROM cuts WHERE id = ?")
            .bind(cut_id)
            .execute(&self.pool)
            .await;

        match result {
            Ok(_) => {
                debug!("Deleted cut: {}", cut_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to delete cut: {}", e);
                Err(CutsRepositoryError::OperationFailed(
                    e.to_string().into_boxed_str(),
                ))
            }
        }
    }

    async fn delete_cuts_by_material_id(&self, material_id: &str) -> Result<()> {
        // Count cuts for the material first to check if any exist
        let count = self.count_cuts_by_material_id(material_id).await?;

        // If no cuts exist, return early (not an error)
        if count == 0 {
            return Ok(());
        }

        // Delete all cuts for the material
        let result = sqlx::query("DELETE FROM cuts WHERE material_id = ?")
            .bind(material_id)
            .execute(&self.pool)
            .await;

        match result {
            Ok(result) => {
                info!(
                    "Deleted {} cuts for material: {}",
                    result.rows_affected(),
                    material_id
                );
                Ok(())
            }
            Err(e) => {
                error!("Failed to delete cuts for material {}: {}", material_id, e);
                Err(CutsRepositoryError::OperationFailed(
                    e.to_string().into_boxed_str(),
                ))
            }
        }
    }

    async fn count_cuts_by_material_id(&self, material_id: &str) -> Result<usize> {
        let result = sqlx::query("SELECT COUNT(*) as count FROM cuts WHERE material_id = ?")
            .bind(material_id)
            .fetch_one(&self.pool)
            .await;

        match result {
            Ok(row) => {
                let count: i64 = row.get("count");
                Ok(count as usize)
            }
            Err(e) => {
                error!("Error counting cuts for material {}: {}", material_id, e);
                Err(CutsRepositoryError::OperationFailed(
                    e.to_string().into_boxed_str(),
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_memory_db;

    async fn setup() -> SqliteCutsRepository {
        let pool = init_memory_db().await.expect("Failed to initialize DB");

        // Create test materials first to satisfy foreign key constraints
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO materials (id, file_path, file_type, status, created_at, updated_at, status_updated_at, error)
            VALUES 
                ('material1', 'path1.txt', 'text/plain', 'discovered', datetime('now'), datetime('now'), datetime('now'), NULL),
                ('material2', 'path2.txt', 'text/plain', 'discovered', datetime('now'), datetime('now'), datetime('now'), NULL),
                ('material3', 'path3.txt', 'text/plain', 'discovered', datetime('now'), datetime('now'), datetime('now'), NULL),
                ('material4', 'path4.txt', 'text/plain', 'discovered', datetime('now'), datetime('now'), datetime('now'), NULL),
                ('material5', 'path5.txt', 'text/plain', 'discovered', datetime('now'), datetime('now'), datetime('now'), NULL),
                ('material6', 'path6.txt', 'text/plain', 'discovered', datetime('now'), datetime('now'), datetime('now'), NULL)
            "#
        )
        .execute(&pool)
        .await
        .expect("Failed to insert test materials");

        SqliteCutsRepository::new(pool)
    }

    fn create_test_cut(material_id: &str, chunk_index: usize) -> Cut {
        Cut::new(
            material_id.to_string(),
            chunk_index,
            format!("Content for chunk {}", chunk_index),
        )
    }

    #[tokio::test]
    async fn test_save_and_get_cut() {
        let repo = setup().await;
        let cut = create_test_cut("material1", 0);
        let cut_id = cut.id.clone();

        // Save the cut
        repo.save_cut(&cut).await.unwrap();

        // Retrieve the cut
        let retrieved = repo.get_cut_by_id(&cut_id).await.unwrap().unwrap();
        assert_eq!(retrieved.id, cut_id);
        assert_eq!(retrieved.material_id, "material1");
        assert_eq!(retrieved.chunk_index, 0);
        assert_eq!(retrieved.content, "Content for chunk 0");
    }

    #[tokio::test]
    async fn test_save_duplicate_cut() {
        let repo = setup().await;
        let cut = create_test_cut("material1", 0);

        // Save the cut
        repo.save_cut(&cut).await.unwrap();

        // Try to save the same cut again
        let result = repo.save_cut(&cut).await;
        assert!(result.is_err());

        if let Err(CutsRepositoryError::CutAlreadyExists(id)) = result {
            assert_eq!(id, cut.id.into_boxed_str());
        } else {
            panic!("Expected CutAlreadyExists error");
        }
    }

    #[tokio::test]
    async fn test_save_and_get_cuts_by_material() {
        let repo = setup().await;
        let material_id = "material2";

        // Create multiple cuts for the same material
        let cut1 = create_test_cut(material_id, 0);
        let cut2 = create_test_cut(material_id, 1);
        let cut3 = create_test_cut(material_id, 2);

        // Save all cuts
        repo.save_cuts(&[cut1.clone(), cut2.clone(), cut3.clone()])
            .await
            .unwrap();

        // Retrieve cuts for the material
        let cuts = repo.get_cuts_by_material_id(material_id).await.unwrap();

        // Check results
        assert_eq!(cuts.len(), 3);
        assert_eq!(cuts[0].chunk_index, 0);
        assert_eq!(cuts[1].chunk_index, 1);
        assert_eq!(cuts[2].chunk_index, 2);

        // Check content
        assert_eq!(cuts[0].content, "Content for chunk 0");
        assert_eq!(cuts[1].content, "Content for chunk 1");
        assert_eq!(cuts[2].content, "Content for chunk 2");
    }

    #[tokio::test]
    async fn test_delete_cut() {
        let repo = setup().await;
        let cut = create_test_cut("material3", 0);
        let cut_id = cut.id.clone();

        // Save and verify
        repo.save_cut(&cut).await.unwrap();
        let retrieved = repo.get_cut_by_id(&cut_id).await.unwrap();
        assert!(retrieved.is_some());

        // Delete and verify
        repo.delete_cut(&cut_id).await.unwrap();
        let retrieved = repo.get_cut_by_id(&cut_id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_delete_cuts_by_material() {
        let repo = setup().await;
        let material_id = "material4";

        // Create multiple cuts
        let cuts = vec![
            create_test_cut(material_id, 0),
            create_test_cut(material_id, 1),
            create_test_cut(material_id, 2),
        ];

        // Save all cuts
        repo.save_cuts(&cuts).await.unwrap();

        // Verify count
        let count = repo.count_cuts_by_material_id(material_id).await.unwrap();
        assert_eq!(count, 3);

        // Delete all cuts for the material
        repo.delete_cuts_by_material_id(material_id).await.unwrap();

        // Verify count is now 0
        let count = repo.count_cuts_by_material_id(material_id).await.unwrap();
        assert_eq!(count, 0);

        // Verify retrieving cuts returns an empty vector
        let cuts = repo.get_cuts_by_material_id(material_id).await.unwrap();
        assert!(cuts.is_empty());
    }

    #[tokio::test]
    async fn test_count_cuts() {
        let repo = setup().await;

        // Create cuts for different materials
        let material1 = "material5";
        let material2 = "material6";

        // Save cuts for material1
        let cuts1 = vec![create_test_cut(material1, 0), create_test_cut(material1, 1)];
        repo.save_cuts(&cuts1).await.unwrap();

        // Save cut for material2
        let cut2 = create_test_cut(material2, 0);
        repo.save_cut(&cut2).await.unwrap();

        // Check counts
        let count1 = repo.count_cuts_by_material_id(material1).await.unwrap();
        let count2 = repo.count_cuts_by_material_id(material2).await.unwrap();
        let count3 = repo.count_cuts_by_material_id("nonexistent").await.unwrap();

        assert_eq!(count1, 2);
        assert_eq!(count2, 1);
        assert_eq!(count3, 0);
    }
}
