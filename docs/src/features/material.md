# Material

A Material represents a text or markdown file that Quilt processes. Materials serve as the source for creating Swatches.

## Overview

Materials are the foundational input for Quilt's processing pipeline. They are processed once to create Swatches.

## Characteristics

- Text or markdown files only
- Source of truth for content
- One-time processing
- Contains structured or unstructured information

## Examples

- Markdown files (.md)
- Text files (.txt)

## Behavior

### Feature: Material Processing

```gherkin
Scenario: Processing Text File
  Given a text file is provided to Quilt
  When the file is processed
  Then the file should be recognized as a Material
  And the Material should be processed for Swatches
  And the processing should complete once

Scenario: Processing Markdown File
  Given a markdown file is provided to Quilt
  When the file is processed
  Then the file should be recognized as a Material
  And the Material should be processed for Swatches
  And markdown structure should be preserved
  And the processing should complete once
```

## Processing

Materials are processed by:

1. Being processed once to create Swatches
2. Having their Swatches embedded and stored in the Swatch Book
