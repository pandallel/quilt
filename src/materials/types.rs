use time::OffsetDateTime;
use cuid2::cuid;
use std::path::Path;
use std::fmt;

/// Supported file types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaterialFileType {
    /// Markdown files (.md)
    Markdown,
    /// Text files (.txt)
    Text,
    /// Other file types
    Other(String),
}

impl MaterialFileType {
    /// Determine file type from file extension
    pub fn from_path(path: &str) -> Self {
        let path = Path::new(path);
        match path.extension().and_then(|ext| ext.to_str()) {
            Some(ext) => match ext.to_lowercase().as_str() {
                "md" => Self::Markdown,
                "txt" => Self::Text,
                other => Self::Other(other.to_string()),
            },
            None => Self::Other("".to_string()),
        }
    }
}

/// The possible states of a material during processing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaterialStatus {
    /// Material has been discovered but not yet processed
    Discovered,
    /// Material has been cut into swatches but not yet embedded
    Cut,
    /// Material has been successfully processed into embeddings
    Swatched,
    /// Material could not be processed
    Error,
}

impl fmt::Display for MaterialStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MaterialStatus::Discovered => write!(f, "Discovered"),
            MaterialStatus::Cut => write!(f, "Cut"),
            MaterialStatus::Swatched => write!(f, "Swatched"),
            MaterialStatus::Error => write!(f, "Error"),
        }
    }
}

/// A Material represents a document in Quilt
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
    /// Error message if processing failed
    pub error: Option<String>,
}

impl Material {
    /// Create a new Material with the given file path
    pub fn new(file_path: String) -> Self {
        Self {
            id: cuid(),
            file_path: file_path.clone(),
            file_type: MaterialFileType::from_path(&file_path),
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
    fn test_file_type_inference() {
        assert_eq!(MaterialFileType::from_path("test.md"), MaterialFileType::Markdown);
        assert_eq!(MaterialFileType::from_path("test.txt"), MaterialFileType::Text);
        assert_eq!(MaterialFileType::from_path("test.rs"), MaterialFileType::Other("rs".to_string()));
        assert_eq!(MaterialFileType::from_path("test"), MaterialFileType::Other("".to_string()));
    }

    #[test]
    fn test_material_status_transitions() {
        let mut material = Material::new("test/path/doc.md".to_string());

        // Test transition to Cut
        material.status = MaterialStatus::Cut;
        assert_eq!(material.status, MaterialStatus::Cut);

        // Test transition to Swatched
        material.status = MaterialStatus::Swatched;
        assert_eq!(material.status, MaterialStatus::Swatched);

        // Test transition to Error with error message
        material.status = MaterialStatus::Error;
        material.error = Some(String::from("Could not identify meaningful content boundaries"));
        assert_eq!(material.status, MaterialStatus::Error);
        assert_eq!(material.error.unwrap(), "Could not identify meaningful content boundaries");
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