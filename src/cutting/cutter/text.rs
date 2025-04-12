use text_splitter::TextSplitter;
use thiserror::Error;
use cuid2::create_id;

use crate::events::types::MaterialId;
use super::config::CutterConfig;

/// Errors that can occur during text cutting
#[derive(Error, Debug)]
pub enum CutterError {
    /// Error during text splitting
    #[error("Failed to split text: {0}")]
    SplittingFailed(String),
}

/// Information about a cut chunk of text
#[derive(Debug, Clone)]
pub struct ChunkInfo {
    /// Unique identifier for this chunk
    pub id: String,
    /// The content of the chunk
    pub content: String,
    /// Sequence number (position in the original document)
    pub sequence: usize,
    /// Material ID this chunk was cut from
    pub material_id: Option<MaterialId>,
}

/// TextCutter handles splitting text into semantic chunks
#[derive(Debug, Clone)]
pub struct TextCutter {
    config: CutterConfig,
}

impl TextCutter {
    /// Create a new TextCutter with the given configuration
    pub fn new(config: CutterConfig) -> Self {
        Self { config }
    }

    /// Create a new TextCutter with default configuration
    pub fn default() -> Self {
        Self {
            config: CutterConfig::default(),
        }
    }

    /// Cut text into chunks according to the configuration
    pub fn cut(&self, text: &str, material_id: Option<MaterialId>) -> Result<Vec<ChunkInfo>, CutterError> {
        // Create text splitter with our configuration
        let splitter = TextSplitter::new(self.config.min_size..=self.config.max_size);

        // Split the text and collect chunks
        let chunks = splitter.chunks(text);
        
        // Convert to our format with sequence numbers
        let result: Vec<ChunkInfo> = chunks
            .enumerate()
            .map(|(i, content)| ChunkInfo {
                id: create_id(),
                content: content.to_string(),
                sequence: i,
                material_id: material_id.clone(),
            })
            .collect();
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cut_empty_text() {
        let cutter = TextCutter::default();
        let result = cutter.cut("", None).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_cut_short_text() {
        let cutter = TextCutter::default();
        let text = "This is a short text.";
        let result = cutter.cut(text, None).unwrap();
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].content, text);
        assert_eq!(result[0].sequence, 0);
        assert!(result[0].id.len() > 0);
    }

    #[test]
    fn test_cut_long_text() {
        let cutter = TextCutter::default();
        let text = "This is a sentence. ".repeat(50);
        let result = cutter.cut(&text, None).unwrap();
        
        // Verify we get multiple chunks
        assert!(result.len() > 1);
        
        // Verify sequence numbers are in order
        for (i, chunk) in result.iter().enumerate() {
            assert_eq!(chunk.sequence, i);
            
            // Each chunk should have content
            assert!(!chunk.content.is_empty());
            
            // Chunk should be a subset of original text
            assert!(text.contains(&chunk.content));
        }
    }

    #[test]
    fn test_cut_with_custom_config() {
        let config = CutterConfig::new(50, 20, 100);
        let cutter = TextCutter::new(config);
        let text = "Testing with a custom configuration.".repeat(10);
        let material_id = Some(MaterialId::new("test-material".to_string()));
        
        let result = cutter.cut(&text, material_id.clone()).unwrap();

        // Verify we get chunks
        assert!(!result.is_empty());
        
        // Verify material_id is set correctly
        for chunk in &result {
            assert_eq!(chunk.material_id, material_id);
            
            // Each chunk should have content
            assert!(!chunk.content.is_empty());
            
            // Chunk should be a subset of original text
            assert!(text.contains(&chunk.content));
        }
    }
} 