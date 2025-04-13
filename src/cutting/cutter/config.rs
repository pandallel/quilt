/// Configuration for text cutting behavior
#[derive(Debug, Clone)]
pub struct CutterConfig {
    /// Target number of characters per chunk
    pub target_size: usize,
    /// Minimum number of characters per chunk
    pub min_size: usize,
    /// Maximum number of characters per chunk
    pub max_size: usize,
}

impl Default for CutterConfig {
    fn default() -> Self {
        Self {
            target_size: 300,
            min_size: 150,
            max_size: 800,
        }
    }
}

impl CutterConfig {
    /// Create a new CutterConfig with custom sizes
    pub fn new(target_size: usize, min_size: usize, max_size: usize) -> Self {
        Self {
            target_size,
            min_size,
            max_size,
        }
    }
    
    /// Estimate token count from text content
    /// This is a simple approximation - words divided by 0.75
    /// (assuming ~4 characters per word, ~3 words per token)
    pub fn get_token_count(&self, content: &str) -> usize {
        // Very simple tokenization - just count words and divide by 0.75
        // This is just a rough approximation
        let word_count = content.split_whitespace().count();
        
        // Ensure we return at least 1 token for non-empty content
        if word_count > 0 {
            (word_count as f32 * 0.75).ceil() as usize
        } else {
            // Handle empty content
            0
        }
    }
}
