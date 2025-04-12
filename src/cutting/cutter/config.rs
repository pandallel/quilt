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
}
