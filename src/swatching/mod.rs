// Quilt - Material Swatching Module
//
// This module handles the swatching of materials, transforming cut chunks
// into semantic embeddings (swatches).

mod actor;
pub mod embedding;
pub mod hf_embedding;
mod repository;
pub mod sqlite_repository;
mod swatch;

pub use actor::messages::{OperationComplete, SwatchingError};
pub use actor::SwatchingActor;
pub use embedding::{EmbeddingError, EmbeddingService};
pub use hf_embedding::HfEmbeddingService;
pub use repository::{Result, SwatchRepository, SwatchRepositoryError};
pub use sqlite_repository::SqliteSwatchRepository;
pub use swatch::Swatch;

#[cfg(test)]
mod tests {
    use super::*;
    use embedding::MockEmbeddingService;
    use std::sync::Arc;

    #[test]
    fn test_embedding_service_mock() {
        // Create a mock embedding service
        let mut mock = MockEmbeddingService::new();

        // Set up expectations
        mock.expect_embed()
            .with(mockall::predicate::eq("test text"))
            .times(1)
            .returning(|_| Ok(vec![0.1, 0.2, 0.3]));

        // Use the mock
        let service: Arc<dyn EmbeddingService> = Arc::new(mock);
        let result = service.embed("test text").unwrap();

        // Verify results
        assert_eq!(result, vec![0.1, 0.2, 0.3]);
    }

    #[test]
    fn test_embedding_service_error_propagation() {
        // Create a mock embedding service that returns errors
        let mut mock = MockEmbeddingService::new();

        // Set up expectations for different error types
        mock.expect_embed()
            .with(mockall::predicate::eq("generate error"))
            .times(1)
            .returning(|_| Err(EmbeddingError::GenerationFailed("test failure".to_string())));

        mock.expect_embed()
            .with(mockall::predicate::eq("model error"))
            .times(1)
            .returning(|_| {
                Err(EmbeddingError::ModelLoadFailed(
                    "model not available".to_string(),
                ))
            });

        // Use the mock as a trait object
        let service: Arc<dyn EmbeddingService> = Arc::new(mock);

        // Test generation error
        let result = service.embed("generate error");
        assert!(result.is_err());
        match result {
            Err(EmbeddingError::GenerationFailed(msg)) => {
                assert_eq!(msg, "test failure");
            }
            _ => panic!("Unexpected error type"),
        }

        // Test model error
        let result = service.embed("model error");
        assert!(result.is_err());
        match result {
            Err(EmbeddingError::ModelLoadFailed(msg)) => {
                assert_eq!(msg, "model not available");
            }
            _ => panic!("Unexpected error type"),
        }
    }
}
