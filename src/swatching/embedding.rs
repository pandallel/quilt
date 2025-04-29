#[cfg(test)]
use mockall::automock;
use thiserror::Error;

/// Errors that can occur during the embedding process.
#[derive(Error, Debug)]
pub enum EmbeddingError {
    /// Error when embedding generation fails
    #[error("Failed to generate embedding: {0}")]
    GenerationFailed(String),

    /// Error when model loading fails
    #[error("Model loading failed: {0}")]
    ModelLoadFailed(String),

    /// Catch-all for other errors
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// A trait for services that generate vector embeddings from text.
#[cfg_attr(test, automock)]
pub trait EmbeddingService: Send + Sync {
    /// Generates an embedding for the given text.
    ///
    /// # Arguments
    ///
    /// * `text` - The input text slice to embed.
    ///
    /// # Returns
    ///
    /// A `Result` containing either the vector embedding (`Vec<f32>`) on success,
    /// or an `EmbeddingError` on failure.
    fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_error_messages() {
        // Test GenerationFailed error message
        let gen_error = EmbeddingError::GenerationFailed("test generation error".to_string());
        assert_eq!(
            gen_error.to_string(),
            "Failed to generate embedding: test generation error"
        );

        // Test ModelLoadFailed error message
        let model_error = EmbeddingError::ModelLoadFailed("model not found".to_string());
        assert_eq!(
            model_error.to_string(),
            "Model loading failed: model not found"
        );

        // Test Other error
        let other_error = EmbeddingError::Other(anyhow::anyhow!("some other error"));
        assert_eq!(other_error.to_string(), "some other error");
    }

    #[test]
    fn test_embedding_error_source() {
        // Create an error message
        let error_message = "underlying error";
        let error = EmbeddingError::Other(anyhow::anyhow!(error_message));

        // The error should display the underlying message due to the #[error(transparent)] attribute
        assert_eq!(error.to_string(), error_message);
    }
}
