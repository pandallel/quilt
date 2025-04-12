use cuid2::cuid;
use time::OffsetDateTime;

/// Represents a single chunk of text processed from a Material.
#[derive(Clone, Debug, PartialEq)]
pub struct Cut {
    /// Unique identifier for this cut.
    pub id: String,
    /// Identifier of the Material this cut belongs to.
    pub material_id: String,
    /// The sequential index of this cut within the original Material.
    pub chunk_index: usize,
    /// The actual text content of this cut.
    pub content: String,
    /// Timestamp when the cut was created
    pub created_at: OffsetDateTime,
    /// Optional: Number of tokens in the content (if calculated during cutting).
    pub token_count: Option<usize>,
    /// Optional: Starting byte offset within the original material's content.
    pub byte_offset_start: Option<usize>,
    /// Optional: Ending byte offset within the original material's content.
    pub byte_offset_end: Option<usize>,
}

impl Cut {
    /// Create a new cut from the given material ID, index, and content
    pub fn new(material_id: String, chunk_index: usize, content: String) -> Self {
        Self {
            id: cuid(),
            material_id,
            chunk_index,
            content,
            created_at: OffsetDateTime::now_utc(),
            token_count: None,
            byte_offset_start: None,
            byte_offset_end: None,
        }
    }

    /// Create a new cut with all specified properties
    pub fn with_details(
        material_id: String,
        chunk_index: usize,
        content: String,
        token_count: Option<usize>,
        byte_offset_start: Option<usize>,
        byte_offset_end: Option<usize>,
    ) -> Self {
        Self {
            id: cuid(),
            material_id,
            chunk_index,
            content,
            created_at: OffsetDateTime::now_utc(),
            token_count,
            byte_offset_start,
            byte_offset_end,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cut_creation() {
        let material_id = "test_material_id".to_string();
        let chunk_index = 0;
        let content = "This is a test cut content".to_string();

        let cut = Cut::new(material_id.clone(), chunk_index, content.clone());

        assert_eq!(cut.material_id, material_id);
        assert_eq!(cut.chunk_index, chunk_index);
        assert_eq!(cut.content, content);
        assert!(!cut.id.is_empty());
        assert!(cut.token_count.is_none());
        assert!(cut.byte_offset_start.is_none());
        assert!(cut.byte_offset_end.is_none());
    }

    #[test]
    fn test_cut_with_details() {
        let material_id = "test_material_id".to_string();
        let chunk_index = 1;
        let content = "This is a test cut content with details".to_string();
        let token_count = Some(10);
        let byte_offset_start = Some(0);
        let byte_offset_end = Some(content.len());

        let cut = Cut::with_details(
            material_id.clone(),
            chunk_index,
            content.clone(),
            token_count,
            byte_offset_start,
            byte_offset_end,
        );

        assert_eq!(cut.material_id, material_id);
        assert_eq!(cut.chunk_index, chunk_index);
        assert_eq!(cut.content, content);
        assert_eq!(cut.token_count, token_count);
        assert_eq!(cut.byte_offset_start, byte_offset_start);
        assert_eq!(cut.byte_offset_end, byte_offset_end);
    }

    #[test]
    fn test_cuid_uniqueness() {
        let material_id = "test_material_id".to_string();
        let cut1 = Cut::new(material_id.clone(), 0, "Content 1".to_string());
        let cut2 = Cut::new(material_id.clone(), 1, "Content 2".to_string());

        assert_ne!(cut1.id, cut2.id, "Cut IDs should be unique");
    }
}
