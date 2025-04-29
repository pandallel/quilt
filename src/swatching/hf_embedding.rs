use std::sync::Arc;
use std::thread;
use std::time::Duration;

use anyhow::Context;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

use crate::swatching::embedding::{EmbeddingError, EmbeddingService};

/// Default model to use for embeddings
const DEFAULT_MODEL: EmbeddingModel = EmbeddingModel::BGESmallENV15;
const DEFAULT_MODEL_VERSION: &str = "v1.5"; // Specify version for default model

/// HuggingFace-based embedding service using the fastembed library.
pub struct HfEmbeddingService {
    /// The text embedding model instance
    embedder: Arc<TextEmbedding>,
    /// The model enum used for initialization
    model_enum: EmbeddingModel,
}

impl HfEmbeddingService {
    /// Creates a new HfEmbeddingService with the default embedding model.
    ///
    /// # Returns
    ///
    /// A Result containing the new HfEmbeddingService or an EmbeddingError if model loading fails.
    pub fn new() -> Result<Self, EmbeddingError> {
        Self::with_model(DEFAULT_MODEL)
    }

    /// Creates a new HfEmbeddingService with a specific embedding model.
    ///
    /// # Arguments
    ///
    /// * `model` - The embedding model to use.
    ///
    /// # Returns
    ///
    /// A Result containing the new HfEmbeddingService or an EmbeddingError if model loading fails.
    pub fn with_model(model: EmbeddingModel) -> Result<Self, EmbeddingError> {
        // Clone model here to avoid move error later
        let options = InitOptions::new(model.clone());

        // Try up to 3 times to initialize the model, with a delay between attempts
        // to handle potential file lock issues
        let mut last_error = None;
        for attempt in 1..=3 {
            match TextEmbedding::try_new(options.clone()) {
                Ok(embedder) => {
                    return Ok(Self {
                        embedder: Arc::new(embedder),
                        model_enum: model, // Use the original (now unmoved) model
                    });
                }
                Err(err) => {
                    last_error = Some(err);
                    if attempt < 3 {
                        // Wait a bit before retrying
                        thread::sleep(Duration::from_millis(500 * attempt));
                    }
                }
            }
        }

        // If we've exhausted our retries, return the last error
        Err(EmbeddingError::ModelLoadFailed(
            last_error
                .map(|e| e.to_string())
                .unwrap_or_else(|| "Unknown model loading error".to_string()),
        ))
    }
}

impl EmbeddingService for HfEmbeddingService {
    fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        if text.trim().is_empty() {
            return Err(EmbeddingError::GenerationFailed(
                "Cannot embed empty text".to_string(),
            ));
        }

        let documents = vec![text.to_string()];
        let embeddings = self
            .embedder
            .embed(documents, None)
            .context("Failed to generate embedding")
            .map_err(|e| EmbeddingError::GenerationFailed(e.to_string()))?;

        // We only embedded one text, so we can safely extract the first embedding
        let embedding = embeddings.into_iter().next().ok_or_else(|| {
            EmbeddingError::GenerationFailed("No embedding was generated".to_string())
        })?;

        Ok(embedding)
    }

    fn model_name(&self) -> &str {
        // Use the associated function to get model info and correct field name
        match TextEmbedding::get_model_info(&self.model_enum) {
            Ok(info) => info.model_code.as_str(), // Use as_str() for String
            Err(_) => "unknown-model",            // Fallback if getting info fails
        }
    }

    fn model_version(&self) -> &str {
        // fastembed ModelInfo v4 doesn't have an explicit version string.
        // TODO: Enhance this if specific model version tracking is needed.
        DEFAULT_MODEL_VERSION // Using the constant defined for the default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_text_returns_error() {
        // Test with different empty strings
        let services = match HfEmbeddingService::new() {
            Ok(service) => service,
            Err(e) => {
                // If we can't create the service (e.g., in CI without models), skip the test
                eprintln!("Skipping test due to model loading failure: {}", e);
                return;
            }
        };

        // Test with empty string
        let result = services.embed("");
        assert!(result.is_err());
        if let Err(EmbeddingError::GenerationFailed(msg)) = result {
            assert_eq!(msg, "Cannot embed empty text");
        } else {
            panic!("Expected GenerationFailed error for empty string");
        }

        // Test with whitespace-only string
        let result = services.embed("   \t\n");
        assert!(result.is_err());
        if let Err(EmbeddingError::GenerationFailed(msg)) = result {
            assert_eq!(msg, "Cannot embed empty text");
        } else {
            panic!("Expected GenerationFailed error for whitespace string");
        }
    }

    #[test]
    fn test_successful_embedding() {
        // Skip test if model can't be loaded
        let service = match HfEmbeddingService::new() {
            Ok(service) => service,
            Err(e) => {
                eprintln!("Skipping test due to model loading failure: {}", e);
                return;
            }
        };

        // Test with simple text
        let text = "This is a sample text for embedding.";
        let result = service.embed(text);

        assert!(result.is_ok());

        // Check that embedding is non-empty and has expected dimensions
        let embedding = result.unwrap();
        assert!(!embedding.is_empty());

        // BGESmallENV15 produces 384-dimensional embeddings
        assert_eq!(embedding.len(), 384);

        // Embeddings should be normalized (unit vectors)
        let sum_squared: f32 = embedding.iter().map(|&x| x * x).sum();
        assert!(
            (sum_squared - 1.0).abs() < 1e-5,
            "Embedding vector is not normalized (sum of squares = {})",
            sum_squared
        );
    }

    #[test]
    fn test_consistency() {
        // Skip test if model can't be loaded
        let service = match HfEmbeddingService::new() {
            Ok(service) => service,
            Err(e) => {
                eprintln!("Skipping test due to model loading failure: {}", e);
                return;
            }
        };

        // Generate embeddings for the same text twice
        let text = "The quick brown fox jumps over the lazy dog.";
        let embedding1 = service.embed(text).unwrap();
        let embedding2 = service.embed(text).unwrap();

        // Embeddings for the same text should be identical
        assert_eq!(embedding1, embedding2);

        // Generate embeddings for different text
        let text2 = "A completely different sentence for comparison.";
        let embedding3 = service.embed(text2).unwrap();

        // Embeddings for different text should be different
        assert_ne!(embedding1, embedding3);
    }

    // Helper function to calculate cosine similarity between two vectors
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        assert_eq!(a.len(), b.len());
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        dot_product
    }

    #[test]
    fn test_semantic_similarity() {
        // Skip test if model can't be loaded
        let service = match HfEmbeddingService::new() {
            Ok(service) => service,
            Err(e) => {
                eprintln!("Skipping test due to model loading failure: {}", e);
                return;
            }
        };

        // Similar sentences should have higher similarity than dissimilar ones
        let text1 = "I like to eat pasta for dinner.";
        let text2 = "Pasta is my favorite food for the evening meal.";
        let text3 = "Quantum physics explores the fundamental nature of reality.";

        let emb1 = service.embed(text1).unwrap();
        let emb2 = service.embed(text2).unwrap();
        let emb3 = service.embed(text3).unwrap();

        let sim_similar = cosine_similarity(&emb1, &emb2);
        let sim_different = cosine_similarity(&emb1, &emb3);

        // Similar sentences should have higher similarity
        assert!(sim_similar > sim_different);

        // Similar sentences should have similarity above 0.7
        // (This is a heuristic threshold that may need adjustment)
        assert!(sim_similar > 0.7);
    }
}
