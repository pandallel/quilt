# Swatch

A Swatch is a semantic embedding of a text chunk (Cut), representing its meaning in a high-dimensional vector space. It enables similarity search and semantic retrieval of content.

## Overview

Swatches are the foundation of Quilt's semantic search capabilities. They transform text into mathematical representations that capture meaning, allowing the system to find contextually relevant information beyond keyword matching.

## Characteristics

- Mathematical representation of semantic content
- Fixed-dimensionality vector (typically 384 dimensions)
- Enables similarity search via vector operations
- Preserves connections to source Material and Cut
- Model-agnostic representation (supports multiple embedding models)
- Normalized vector representation (unit vectors)

## Behavior

### Feature: Swatch Creation

```gherkin
Scenario: Creating Swatches from Cuts
  Given a Cut from a Material
  When the Cut is processed by the swatching system
  Then a vector embedding should be generated
  And the Swatch should be persisted in the repository
  And the Swatch should maintain references to its source Cut and Material

Scenario: Processing Different Text Types
  Given Cuts with different content types (code, markdown, text)
  When the Cuts are processed by the swatching system
  Then appropriate embeddings should be generated for each type
  And the semantic meaning should be preserved in the vector space

Scenario: Handling Model Loading Issues
  Given a configuration with an unavailable embedding model
  When the system attempts to create a Swatch
  Then a "Model Loading Error" should be reported
  And the system should attempt to retry with exponential backoff
  And detailed information about the model issue should be logged

Scenario: Handling Empty or Invalid Text
  Given a Cut with empty or invalid content
  When the system attempts to create a Swatch
  Then an "Embedding Generation Error" should be reported
  And the error should indicate the specific issue with the content
```

## Creation

Swatches are created by:

1. Retrieving a Cut from the Cut repository
2. Preprocessing the Cut's content if necessary
3. Feeding the text through an embedding model (e.g., BGE Small EN v1.5)
4. Storing the resulting vector with metadata
5. Preserving references to the source Cut and Material

## Structure

A Swatch includes:

- Unique identifier (`id`)
- Reference to source Cut (`cut_id`)
- Reference to source Material (`material_id`)
- Embedding vector (`embedding`) - a sequence of 32-bit floating-point numbers
- Model information (`model_name`, `model_version`)
- Creation timestamp (`created_at`)
- Dimensionality (`dimensions`)
- Optional similarity threshold (`similarity_threshold`)
- Optional metadata about the embedding process (`metadata`)

## Usage

Swatches are used to:

- Power semantic search capabilities
- Find contextually similar content
- Connect related information across materials
- Support natural language queries
- Enable relevance ranking beyond keyword matching

## Similarity Search

Swatches enable similarity search through:

1. Vector comparisons (typically cosine similarity)
2. Efficient nearest-neighbor algorithms
3. Configurable similarity thresholds
4. Context-aware retrieval of source Cuts and Materials
