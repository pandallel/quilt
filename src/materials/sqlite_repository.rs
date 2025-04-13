use std::collections::HashMap;

use async_trait::async_trait;
use sqlx::{Row, SqlitePool};
use time::OffsetDateTime;
use tracing::{debug, error};

use super::{
    Material, MaterialFileType, MaterialRepository, MaterialStatus, RepositoryError, Result,
};

/// SQLite implementation of the Material Repository
#[derive(Debug, Clone)]
pub struct SqliteMaterialRepository {
    /// Database connection pool
    pool: SqlitePool,
}

impl SqliteMaterialRepository {
    /// Create a new SQLite material repository
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Convert a database row to a Material
    fn row_to_material(row: sqlx::sqlite::SqliteRow) -> Material {
        let file_type = match row.get::<String, _>("file_type").as_str() {
            "Markdown" => MaterialFileType::Markdown,
            "Text" => MaterialFileType::Text,
            other => MaterialFileType::Other(other.to_string()),
        };

        let status = match row.get::<String, _>("status").as_str() {
            "Discovered" => MaterialStatus::Discovered,
            "Cut" => MaterialStatus::Cut,
            "Swatched" => MaterialStatus::Swatched,
            "Error" => MaterialStatus::Error,
            _ => MaterialStatus::Error, // Default to Error if unknown
        };

        let error: Option<String> = row.get("error");

        Material {
            id: row.get("id"),
            file_path: row.get("file_path"),
            file_type,
            status,
            error,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            status_updated_at: row.get("status_updated_at"),
        }
    }
}

#[async_trait]
impl MaterialRepository for SqliteMaterialRepository {
    async fn register_material(&self, material: Material) -> Result<()> {
        // First check if material with this ID already exists
        let existing = sqlx::query("SELECT id FROM materials WHERE id = ?")
            .bind(&material.id)
            .fetch_optional(&self.pool)
            .await;

        match existing {
            Ok(Some(_)) => {
                return Err(RepositoryError::MaterialAlreadyExists(material.id));
            }
            Ok(None) => {} // Material doesn't exist, continue
            Err(e) => {
                error!("Database error checking for existing material: {}", e);
                return Err(RepositoryError::MaterialAlreadyExists(material.id));
            }
        }

        // Convert MaterialFileType to string for storage
        let file_type = match material.file_type {
            MaterialFileType::Markdown => "Markdown".to_string(),
            MaterialFileType::Text => "Text".to_string(),
            MaterialFileType::Other(ref s) => s.clone(),
        };

        // Insert material
        let result = sqlx::query(
            r#"
            INSERT INTO materials (id, file_path, file_type, created_at, updated_at, status_updated_at, status, error)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&material.id)
        .bind(&material.file_path)
        .bind(file_type)
        .bind(material.created_at)
        .bind(material.updated_at)
        .bind(material.status_updated_at)
        .bind(material.status.to_string())
        .bind(&material.error)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {
                debug!("Successfully registered material: {}", material.id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to register material: {}", e);
                Err(RepositoryError::MaterialAlreadyExists(material.id))
            }
        }
    }

    async fn get_material(&self, id: &str) -> Option<Material> {
        let result = sqlx::query("SELECT * FROM materials WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await;

        match result {
            Ok(Some(row)) => Some(Self::row_to_material(row)),
            Ok(None) => None,
            Err(e) => {
                error!("Error fetching material {}: {}", id, e);
                None
            }
        }
    }

    async fn update_material_status(
        &self,
        id: &str,
        new_status: MaterialStatus,
        error_message: Option<String>,
    ) -> Result<()> {
        // First get the current material to check status
        let material = self.get_material(id).await;

        let material = match material {
            Some(m) => m,
            None => return Err(RepositoryError::MaterialNotFound(id.to_string())),
        };

        // Validate status transition
        match (material.status, new_status.clone()) {
            // Valid transitions
            (MaterialStatus::Discovered, MaterialStatus::Cut)
            | (MaterialStatus::Discovered, MaterialStatus::Error)
            | (MaterialStatus::Cut, MaterialStatus::Swatched)
            | (MaterialStatus::Cut, MaterialStatus::Error)
            | (MaterialStatus::Swatched, MaterialStatus::Error)
            | (MaterialStatus::Error, MaterialStatus::Discovered) => {
                let now = OffsetDateTime::now_utc();

                // Update the material in the database
                let result = sqlx::query(
                    r#"
                    UPDATE materials 
                    SET status = ?, error = ?, updated_at = ?, status_updated_at = ?
                    WHERE id = ?
                    "#,
                )
                .bind(new_status.to_string())
                .bind(error_message)
                .bind(now)
                .bind(now)
                .bind(id)
                .execute(&self.pool)
                .await;

                match result {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        error!("Failed to update material status: {}", e);
                        Err(RepositoryError::MaterialNotFound(id.to_string()))
                    }
                }
            }
            // Invalid transitions
            (from, to) => Err(RepositoryError::InvalidStateTransition { from, to }),
        }
    }

    async fn list_materials(&self) -> Vec<Material> {
        let result = sqlx::query("SELECT * FROM materials")
            .fetch_all(&self.pool)
            .await;

        match result {
            Ok(rows) => rows.into_iter().map(Self::row_to_material).collect(),
            Err(e) => {
                error!("Error listing materials: {}", e);
                Vec::new()
            }
        }
    }

    async fn list_materials_by_status(&self, status: MaterialStatus) -> Vec<Material> {
        let result = sqlx::query("SELECT * FROM materials WHERE status = ?")
            .bind(status.to_string())
            .fetch_all(&self.pool)
            .await;

        match result {
            Ok(rows) => rows.into_iter().map(Self::row_to_material).collect(),
            Err(e) => {
                error!("Error listing materials by status: {}", e);
                Vec::new()
            }
        }
    }

    async fn count_by_status(&self) -> HashMap<MaterialStatus, usize> {
        let mut counts = HashMap::new();

        // Initialize counts for all statuses
        counts.insert(MaterialStatus::Discovered, 0);
        counts.insert(MaterialStatus::Cut, 0);
        counts.insert(MaterialStatus::Swatched, 0);
        counts.insert(MaterialStatus::Error, 0);

        // Query the database for counts by status
        let result = sqlx::query("SELECT status, COUNT(*) as count FROM materials GROUP BY status")
            .fetch_all(&self.pool)
            .await;

        match result {
            Ok(rows) => {
                for row in rows {
                    let status_str: String = row.get("status");
                    let count: i64 = row.get("count");

                    let status = match status_str.as_str() {
                        "Discovered" => MaterialStatus::Discovered,
                        "Cut" => MaterialStatus::Cut,
                        "Swatched" => MaterialStatus::Swatched,
                        "Error" => MaterialStatus::Error,
                        _ => continue, // Skip unknown status
                    };

                    *counts.entry(status).or_insert(0) = count as usize;
                }
            }
            Err(e) => {
                error!("Error counting materials by status: {}", e);
            }
        }

        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_memory_db;
    use std::time::Duration;
    use tokio::time::sleep;

    async fn setup() -> SqliteMaterialRepository {
        let pool = init_memory_db()
            .await
            .expect("Failed to initialize test DB");
        SqliteMaterialRepository::new(pool)
    }

    fn create_test_material(status: MaterialStatus) -> Material {
        let mut material = Material::new("test/path.md".to_string());
        material.status = status;
        material
    }

    #[tokio::test]
    async fn test_register_and_get_material() {
        let repo = setup().await;
        let material = create_test_material(MaterialStatus::Discovered);
        let id = material.id.clone();
        let created_at = material.created_at;

        // Register the material
        repo.register_material(material).await.unwrap();

        // Retrieve the material
        let retrieved = repo.get_material(&id).await.unwrap();
        assert_eq!(retrieved.id, id);
        assert_eq!(retrieved.status, MaterialStatus::Discovered);
        assert_eq!(retrieved.created_at, created_at);
        assert_eq!(retrieved.updated_at, created_at);
        assert_eq!(retrieved.status_updated_at, created_at);
    }

    #[tokio::test]
    async fn test_register_duplicate_material() {
        let repo = setup().await;
        let material = create_test_material(MaterialStatus::Discovered);
        let id = material.id.clone();

        // Register the material
        repo.register_material(material.clone()).await.unwrap();

        // Try to register the same material again
        let result = repo.register_material(material).await;
        assert!(result.is_err());
        if let Err(RepositoryError::MaterialAlreadyExists(existing_id)) = result {
            assert_eq!(existing_id, id);
        } else {
            panic!("Expected MaterialAlreadyExists error");
        }
    }

    #[tokio::test]
    async fn test_update_material_status() {
        let repo = setup().await;
        let material = create_test_material(MaterialStatus::Discovered);
        let id = material.id.clone();
        let created_at = material.created_at;

        // Register the material
        repo.register_material(material).await.unwrap();

        // Small delay to ensure timestamps will be different
        sleep(Duration::from_millis(1)).await;

        // Update status to Cut
        repo.update_material_status(&id, MaterialStatus::Cut, None)
            .await
            .unwrap();

        // Verify status change and timestamps
        let updated_after_cut = repo.get_material(&id).await.unwrap();
        assert_eq!(updated_after_cut.status, MaterialStatus::Cut);
        assert_eq!(updated_after_cut.created_at, created_at);
        assert!(updated_after_cut.updated_at > created_at);
        assert!(updated_after_cut.status_updated_at > created_at);
        assert_eq!(
            updated_after_cut.updated_at,
            updated_after_cut.status_updated_at
        );
        let first_update_time = updated_after_cut.updated_at; // Capture time after first update

        // Small delay to ensure timestamps will be different
        sleep(Duration::from_millis(1)).await;

        // Update status to Swatched
        repo.update_material_status(&id, MaterialStatus::Swatched, None)
            .await
            .unwrap();

        // Verify status change and timestamps
        let updated_after_swatched = repo.get_material(&id).await.unwrap();
        assert_eq!(updated_after_swatched.status, MaterialStatus::Swatched);
        assert_eq!(updated_after_swatched.created_at, created_at);
        assert!(updated_after_swatched.updated_at >= first_update_time);
        assert!(updated_after_swatched.status_updated_at >= first_update_time);
        assert_eq!(
            updated_after_swatched.updated_at,
            updated_after_swatched.status_updated_at
        ); // Should be equal after this update
    }

    #[tokio::test]
    async fn test_update_material_status_with_error() {
        let repo = setup().await;
        let material = create_test_material(MaterialStatus::Discovered);
        let id = material.id.clone();
        let created_at = material.created_at;

        // Register the material
        repo.register_material(material).await.unwrap();

        // Small delay to ensure timestamps will be different
        sleep(Duration::from_millis(1)).await;

        // Update status to Error with a message
        let error_message = "Test error message".to_string();
        repo.update_material_status(&id, MaterialStatus::Error, Some(error_message.clone()))
            .await
            .unwrap();

        // Verify status change, error message, and timestamps
        let updated_after_error = repo.get_material(&id).await.unwrap();
        assert_eq!(updated_after_error.status, MaterialStatus::Error);
        assert_eq!(updated_after_error.error, Some(error_message));
        assert_eq!(updated_after_error.created_at, created_at);
        assert!(updated_after_error.updated_at > created_at);
        assert!(updated_after_error.status_updated_at > created_at);
        assert_eq!(
            updated_after_error.updated_at,
            updated_after_error.status_updated_at
        );
        let first_update_time = updated_after_error.updated_at; // Capture time after first update

        // Small delay to ensure timestamps will be different
        sleep(Duration::from_millis(1)).await;

        // Reset to Discovered (simulating retry)
        repo.update_material_status(&id, MaterialStatus::Discovered, None)
            .await
            .unwrap();

        // Verify status change, error message cleared, and timestamps
        let updated_after_reset = repo.get_material(&id).await.unwrap();
        assert_eq!(updated_after_reset.status, MaterialStatus::Discovered);
        assert_eq!(updated_after_reset.error, None);
        assert_eq!(updated_after_reset.created_at, created_at);
        assert!(updated_after_reset.updated_at >= first_update_time);
        assert!(updated_after_reset.status_updated_at >= first_update_time);
        assert_eq!(
            updated_after_reset.updated_at,
            updated_after_reset.status_updated_at
        ); // Should be equal after this update
    }

    #[tokio::test]
    async fn test_list_materials_by_status() {
        let repo = setup().await;

        // Create and register materials with different statuses
        let material1 = create_test_material(MaterialStatus::Discovered);
        let material2 = create_test_material(MaterialStatus::Cut);
        let material3 = create_test_material(MaterialStatus::Swatched);

        repo.register_material(material1).await.unwrap();
        repo.register_material(material2).await.unwrap();
        repo.register_material(material3).await.unwrap();

        // List by Discovered status
        let discovered = repo
            .list_materials_by_status(MaterialStatus::Discovered)
            .await;
        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered[0].status, MaterialStatus::Discovered);

        // List by Cut status
        let cut = repo.list_materials_by_status(MaterialStatus::Cut).await;
        assert_eq!(cut.len(), 1);
        assert_eq!(cut[0].status, MaterialStatus::Cut);

        // List by Swatched status
        let swatched = repo
            .list_materials_by_status(MaterialStatus::Swatched)
            .await;
        assert_eq!(swatched.len(), 1);
        assert_eq!(swatched[0].status, MaterialStatus::Swatched);
    }

    #[tokio::test]
    async fn test_count_by_status() {
        let repo = setup().await;

        // Create and register materials with different statuses
        let material1 = create_test_material(MaterialStatus::Discovered);
        let material2 = create_test_material(MaterialStatus::Cut);
        let material3 = create_test_material(MaterialStatus::Cut);

        repo.register_material(material1).await.unwrap();
        repo.register_material(material2).await.unwrap();
        repo.register_material(material3).await.unwrap();

        // Get counts
        let counts = repo.count_by_status().await;

        assert_eq!(*counts.get(&MaterialStatus::Discovered).unwrap(), 1);
        assert_eq!(*counts.get(&MaterialStatus::Cut).unwrap(), 2);
        assert_eq!(*counts.get(&MaterialStatus::Swatched).unwrap(), 0);
        assert_eq!(*counts.get(&MaterialStatus::Error).unwrap(), 0);
    }
}
