use time::OffsetDateTime;
use cuid2::cuid;

/// Supported file types (currently only Markdown)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaterialFileType {
    Markdown,
}

/// The possible states of a material during ingestion
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaterialStatus {
    /// Material has been discovered but not yet validated
    Discovered,
    /// Material has passed validation
    Valid,
    /// Material failed validation
    Invalid,
}

/// A Material represents a Markdown file in Quilt
#[derive(Debug, Clone)]
pub struct Material {
    /// Unique identifier for the material
    pub id: String,
    /// Path to the file on the filesystem
    pub file_path: String,
    /// Type of the material file
    pub file_type: MaterialFileType,
    /// Timestamp when the material was first ingested
    pub ingested_at: OffsetDateTime,
    /// Current status of the material
    pub status: MaterialStatus,
    /// Error message if status is Invalid
    pub error: Option<String>,
}

impl Material {
    /// Create a new Material with the given file path
    pub fn new(file_path: String) -> Self {
        Self {
            id: cuid(),
            file_path,
            file_type: MaterialFileType::Markdown,
            ingested_at: OffsetDateTime::now_utc(),
            status: MaterialStatus::Discovered,
            error: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_creation() {
        let material = Material::new("test/path/doc.md".to_string());

        assert_eq!(material.file_type, MaterialFileType::Markdown);
        assert_eq!(material.status, MaterialStatus::Discovered);
        assert!(material.error.is_none());
        assert_eq!(material.id.len(), 24);
        assert!(material.ingested_at <= OffsetDateTime::now_utc());
    }

    #[test]
    fn test_material_status_transitions() {
        let mut material = Material::new("test/path/doc.md".to_string());

        // Test transition to Valid
        material.status = MaterialStatus::Valid;
        assert_eq!(material.status, MaterialStatus::Valid);

        // Test transition to Invalid with error
        material.status = MaterialStatus::Invalid;
        material.error = Some(String::from("Validation failed"));
        assert_eq!(material.status, MaterialStatus::Invalid);
        assert_eq!(material.error.unwrap(), "Validation failed");
    }

    #[test]
    fn test_cuid_uniqueness() {
        let id1 = cuid();
        let id2 = cuid();
        
        assert_ne!(id1, id2, "CUIDs should be unique");
        assert_eq!(id1.len(), 24, "CUID should be 24 characters");
        assert_eq!(id2.len(), 24, "CUID should be 24 characters");
    }

    #[test]
    fn test_material_timestamps() {
        // Create two materials in sequence
        let material1 = Material::new("test/doc1.md".to_string());
        let material2 = Material::new("test/doc2.md".to_string());

        // Second material should have a later or equal timestamp
        assert!(material2.ingested_at >= material1.ingested_at);
    }
} 