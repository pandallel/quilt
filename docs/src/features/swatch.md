# Swatch

A Swatch is a meaningful fragment cut from a Material. It represents a discrete, contextually relevant piece of information that can be embedded and retrieved.

## Overview

Swatches are the building blocks of Quilt's memory system. They are carefully extracted from Materials to maintain meaningful context while being granular enough for precise retrieval.

## Characteristics

- Meaningful fragments of content
- Maintains source context
- Embeddable for semantic search
- Structured for efficient retrieval
- Preserves relationships to source Material

## Behavior

### Feature: Swatch Creation

```gherkin
Scenario: Creating Swatches from Text
  Given a text file as Material
  When the Material is processed
  Then meaningful paragraphs should be extracted as Swatches
  And each Swatch should maintain its source context
  And the Swatches should be ready for embedding

Scenario: Creating Swatches from Markdown
  Given a markdown file as Material
  When the Material is processed
  Then sections should be extracted as Swatches
  And headers and content should be preserved
  And markdown formatting should be maintained
  And the Swatches should be ready for embedding
```

## Creation

Swatches are created by:

1. Analyzing Material content structure
2. Identifying meaningful boundaries
3. Extracting relevant fragments
4. Preserving metadata and context

## Usage

Swatches are used to:

- Build the searchable Swatch Book
- Retrieve original Materials for addition to the Spread
- Enable precise information retrieval
- Maintain relationships between content pieces
