use cuid2::cuid;
use time::OffsetDateTime;

/// Represents a semantic embedding of a text chunk (Cut).
///
/// A Swatch is the result of processing a Cut through an embedding model.
/// It contains the embedding vector and metadata about the embedding process.
#[derive(Clone, Debug, PartialEq)]
pub struct Swatch {
    /// Unique identifier for this swatch.
    pub id: String,

    /// Identifier of the Cut this swatch was generated from.
    pub cut_id: String,

    /// Identifier of the Material this swatch's cut belongs to.
    pub material_id: String,

    /// The embedding vector representing the semantic content.
    /// Stored as a vector of 32-bit floating point numbers.
    pub embedding: Vec<f32>,

    /// The name/identifier of the embedding model used to generate this swatch.
    pub model_name: String,

    /// The version or configuration of the embedding model used.
    pub model_version: String,

    /// Timestamp when the swatch was created.
    pub created_at: OffsetDateTime,

    /// Dimensions of the embedding vector.
    pub dimensions: usize,

    /// Optional: Similarity threshold recommended for this embedding type.
    pub similarity_threshold: Option<f32>,

    /// Optional: Additional metadata about the embedding or processing.
    pub metadata: Option<serde_json::Value>,
}

impl Swatch {
    /// Create a new swatch with the specified parameters.
    pub fn new(
        cut_id: String,
        material_id: String,
        embedding: Vec<f32>,
        model_name: String,
        model_version: String,
    ) -> Self {
        let dimensions = embedding.len();

        Self {
            id: cuid(),
            cut_id,
            material_id,
            embedding,
            model_name,
            model_version,
            created_at: OffsetDateTime::now_utc(),
            dimensions,
            similarity_threshold: None,
            metadata: None,
        }
    }

    /// Create a new swatch with all details specified.
    pub fn with_details(
        cut_id: String,
        material_id: String,
        embedding: Vec<f32>,
        model_name: String,
        model_version: String,
        similarity_threshold: Option<f32>,
        metadata: Option<serde_json::Value>,
    ) -> Self {
        let dimensions = embedding.len();

        Self {
            id: cuid(),
            cut_id,
            material_id,
            embedding,
            model_name,
            model_version,
            created_at: OffsetDateTime::now_utc(),
            dimensions,
            similarity_threshold,
            metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_new_swatch() {
        let cut_id = "cut123".to_string();
        let material_id = "material456".to_string();
        let embedding = vec![0.1, 0.2, 0.3, 0.4];
        let model_name = "test-model".to_string();
        let model_version = "1.0".to_string();

        let swatch = Swatch::new(
            cut_id.clone(),
            material_id.clone(),
            embedding.clone(),
            model_name.clone(),
            model_version.clone(),
        );

        assert_eq!(swatch.cut_id, cut_id);
        assert_eq!(swatch.material_id, material_id);
        assert_eq!(swatch.embedding, embedding);
        assert_eq!(swatch.model_name, model_name);
        assert_eq!(swatch.model_version, model_version);
        assert_eq!(swatch.dimensions, 4);
        assert!(swatch.similarity_threshold.is_none());
        assert!(swatch.metadata.is_none());
    }

    #[test]
    fn test_with_details_swatch() {
        let cut_id = "cut123".to_string();
        let material_id = "material456".to_string();
        let embedding = vec![0.1, 0.2, 0.3, 0.4];
        let model_name = "test-model".to_string();
        let model_version = "1.0".to_string();
        let similarity_threshold = Some(0.85);
        let metadata = Some(json!({
            "processing_time_ms": 42,
            "token_count": 128
        }));

        let swatch = Swatch::with_details(
            cut_id.clone(),
            material_id.clone(),
            embedding.clone(),
            model_name.clone(),
            model_version.clone(),
            similarity_threshold,
            metadata.clone(),
        );

        assert_eq!(swatch.cut_id, cut_id);
        assert_eq!(swatch.material_id, material_id);
        assert_eq!(swatch.embedding, embedding);
        assert_eq!(swatch.model_name, model_name);
        assert_eq!(swatch.model_version, model_version);
        assert_eq!(swatch.dimensions, 4);
        assert_eq!(swatch.similarity_threshold, similarity_threshold);
        assert_eq!(swatch.metadata, metadata);
    }
}
