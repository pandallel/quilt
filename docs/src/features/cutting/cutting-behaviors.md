# Cut

A Cut is a meaningful fragment cut from a Material. It represents a discrete, contextually relevant piece of information that can be embedded and retrieved.

## Overview

Cuts are the building blocks of Quilt's memory system. They are carefully extracted from Materials to maintain meaningful context while being granular enough for precise retrieval.

## Characteristics

- Meaningful fragments of content
- Maintains source context
- Embeddable for semantic search
- Structured for efficient retrieval
- Preserves relationships to source Material

## Behavior

### Feature: Cut Creation

```gherkin
Scenario: Creating Cuts from Text
  Given a text file as Material
  When the Material is processed
  Then meaningful paragraphs should be extracted as Cuts
  And each Cut should maintain its source context
  And the Cuts should be ready for embedding

Scenario: Creating Cuts from Markdown
  Given a markdown file as Material
  When the Material is processed
  Then sections should be extracted as Cuts
  And headers and content should be preserved
  And markdown formatting should be maintained
  And the Cuts should be ready for embedding

Scenario: Handling Inaccessible Files
  Given a file with permission issues or that doesn't exist
  When the user attempts to process the file
  Then an "Access Error" should be displayed
  And the error details should include the file path and reason
  And suggestions for fixing permissions should be provided

Scenario: Handling Unsupported File Types
  Given a file with an unsupported format
  When the user attempts to process the file
  Then a "Format Not Supported" error should be displayed
  And the error should list supported file formats
  And the user should be able to request support for additional formats

Scenario: Handling Malformed Content
  Given a file with corrupted or improperly formatted content
  When the user attempts to process the file
  Then a "Content Error" should be displayed
  And the error should indicate the approximate location of the issue
  And partial processing should be attempted where possible
```

## Creation

Cuts are created by:

1. Analyzing Material content structure
2. Identifying meaningful boundaries
3. Extracting relevant fragments
4. Preserving metadata and context

## Error Handling

Quilt uses a progressive fallback strategy to maximize successful cutting:

- Specialized format-specific strategies are tried first:
  - Markdown strategy: Uses MarkdownSplitter to preserve heading hierarchy and block structure
  - Code strategy: Uses CodeSplitter to respect function boundaries and code structure
- If a specialized strategy fails, the system falls back to the general-purpose TextSplitter
- Error reporting occurs only when all cutting strategies have been tried and failed

For detailed information on error handling during the cutting process, see [Cut Error Handling](./cutting-errors.md).

## Usage

Cuts are used to:

- Build the searchable Swatch Book
- Retrieve original Materials for addition to the Spread
- Enable precise information retrieval
- Maintain relationships between content pieces
