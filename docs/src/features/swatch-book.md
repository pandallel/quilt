# Swatch Book

The Swatch Book is Quilt's searchable memory system that stores and indexes embedded Swatches for efficient retrieval.

## Overview

The Swatch Book serves as the central memory component of Quilt, providing fast and efficient access to embedded Swatches while maintaining their relationships and context.

## Characteristics

- Local-first storage
- Fast semantic search
- Efficient retrieval
- Maintains relationships
- Indexed for performance

## Behavior

### Feature: Swatch Book Operations

```gherkin
Scenario: Adding Swatches
  Given a Swatch containing "The quick brown fox jumps over the lazy dog" from "text.txt"
  And a Swatch containing "# Project Setup" from "setup.md"
  When the Swatches are added to the Swatch Book
  Then both Swatches should be stored locally
  And both Swatches should be searchable by content
  And the source file information should be preserved

Scenario: Searching Swatches
  Given a Swatch Book containing:
    | content                    | source     |
    | "The quick brown fox..."   | text.txt   |
    | "# Project Setup"         | setup.md   |
    | "1. Install dependencies"  | setup.md   |
  When searching for "how to install"
  Then the "1. Install dependencies" Swatch should be returned first
  And the source file "setup.md" should be included
  And the Swatch's markdown formatting should be preserved

Scenario: Retrieving Related Swatches
  Given a Swatch Book containing:
    | content                    | source     |
    | "# Project Setup"         | setup.md   |
    | "1. Install dependencies"  | setup.md   |
    | "2. Run tests"            | setup.md   |
  When requesting Swatches related to "Project Setup"
  Then the Swatches should be returned in a way that preserves their original context
  And the header Swatch "# Project Setup" should be returned first
  And the numbered list Swatches should follow in order
  And the markdown formatting should be preserved
  So that the content can be read as a complete, coherent section
```

## Operations

The Swatch Book supports:

- Adding new Swatches
- Searching Swatches by content
- Retrieving related Swatches in context
