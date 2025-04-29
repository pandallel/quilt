# Swatching Error Handling

This document describes how errors are handled within the swatching system, including detection, reporting, recovery strategies, and user communication.

## Error Types

The swatching process can encounter various types of errors, categorized by their source and nature:

### 1. Embedding Generation Errors

Errors that occur during the embedding generation process:

```rust
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
```

#### Common Causes:

- Empty or invalid text input
- Model execution failures
- Resource constraints (memory, compute)
- Tokenization issues

### 2. Repository Errors

Errors related to swatch storage and retrieval:

```rust
#[derive(Error, Debug)]
pub enum SwatchRepositoryError {
    #[error("Swatch with id {0} not found")]
    SwatchNotFound(Box<str>),

    #[error("Swatch with id {0} already exists")]
    SwatchAlreadyExists(Box<str>),

    #[error("Operation failed: {0}")]
    OperationFailed(Box<str>),

    #[error("Search operation failed: {0}")]
    SearchFailed(Box<str>),
}
```

#### Common Causes:

- Database connectivity issues
- Serialization/deserialization failures
- Concurrent write conflicts
- Resource constraints
- Invalid data

### 3. Actor-Level Errors

Errors at the SwatchingActor level:

```rust
#[derive(Debug, Error)]
pub enum SwatchingError {
    /// Material not found error
    #[error("Material not found: {0}")]
    MaterialNotFound(MaterialId),

    /// Cuts not found error
    #[error("Cuts not found for material: {0}")]
    CutsNotFound(MaterialId),

    /// Generic swatching error
    #[error("Swatching operation failed: {0}")]
    OperationFailed(Box<str>),
}
```

#### Common Causes:

- Missing dependencies (cut or material not found)
- Event bus communication failures
- System resource constraints
- Configuration issues

## Error Detection

Errors are detected at multiple levels through the system:

1. **Input Validation**

   - Empty text check in `HfEmbeddingService.embed()`
   - Validation of embedding parameters
   - Repository parameter validation

2. **Runtime Errors**

   - Model loading failures in `HfEmbeddingService`
   - Database errors in `SqliteSwatchRepository`
   - Event handling errors in `SwatchingActor`

3. **Resource Monitoring**
   - Model loading timeout detection
   - Repository connection monitoring
   - Event bus health checks

## Recovery Strategies

The swatching system employs several recovery strategies:

### 1. Retries with Exponential Backoff

For transient errors such as model loading issues:

```rust
// Try up to 3 times to initialize the model, with a delay between attempts
let mut last_error = None;
for attempt in 1..=3 {
    match TextEmbedding::try_new(options.clone()) {
        Ok(embedder) => {
            return Ok(Self {
                embedder: Arc::new(embedder),
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
```

### 2. Graceful Degradation

For non-critical errors, the system continues processing other materials:

- Individual material processing failures don't stop the entire pipeline
- Errors are logged and published as events
- Material status is updated to reflect the error

### 3. Batch Processing with Transactions

For repository operations:

- Database operations use transactions for atomicity
- Batch operations can partially succeed
- Failed operations are reported individually

## Error Reporting

Errors are communicated through multiple channels:

1. **Logging**

   - Detailed error logs with context
   - Different log levels based on severity
   - Structured logging for machine processing

2. **Events**

   - Error events published to the event bus
   - Includes material ID, error type, and context
   - Allows downstream systems to react to failures

3. **Status Updates**
   - Material status updated in the registry
   - Error details persisted for later inspection
   - UI can display meaningful error messages

## Error Prevention

The system includes several measures to prevent common errors:

1. **Input Validation**

   - Text validation before embedding
   - Parameter checking for repository operations
   - Type-safe interfaces

2. **Resource Management**

   - Connection pooling for database access
   - Bounded work queues to prevent memory exhaustion
   - Graceful shutdown handling

3. **Monitoring**
   - Health check endpoints for actors (Ping)
   - Error rate monitoring
   - Resource utilization tracking

## User Facing Errors

When errors occur, they are presented to users in a clear, actionable format:

1. **Clear Error Messages**

   - Human-readable error descriptions
   - Context about what was being processed
   - Suggestions for fixes when possible

2. **Error Categories**

   - Configuration issues
   - Content issues
   - System limitations
   - Temporary failures

3. **Recovery Actions**
   - Retry options for transient failures
   - Configuration update suggestions
   - Content modification guidance

## Implementation Details

### Error Propagation

Errors are propagated through the system using Rust's `Result` type and the `?` operator:

```rust
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
```
