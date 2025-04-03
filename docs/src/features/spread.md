# Spread

A Spread is a contextual bundle of Swatches and their source Material, assembled in response to a text query.

## Overview

Spreads represent the final output of Quilt's context retrieval system, bringing together relevant pieces of information in a meaningful way.

## Characteristics

- Contextual assembly
- Contains relevant Swatches
- Includes source Materials
- Maintains relationships
- Supports understanding

## Behavior

### Feature: Spread Assembly

```gherkin
Scenario: Assembling Content Search Spread
  Given a Swatch Book containing:
    | content                    | source     |
    | "The quick brown fox..."   | text.txt   |
    | "# Project Setup"         | setup.md   |
    | "1. Install dependencies"  | setup.md   |
  When searching for "how to install"
  Then a Spread should be assembled containing:
    | content                    | source     |
    | "1. Install dependencies"  | setup.md   |
  And the source file "setup.md" should be referenced
  And the markdown formatting should be preserved

Scenario: Assembling Related Content Spread
  Given a Swatch Book containing:
    | content                    | source     |
    | "# Project Setup"         | setup.md   |
    | "1. Install dependencies"  | setup.md   |
    | "2. Run tests"            | setup.md   |
  When requesting content related to "Project Setup"
  Then a Spread should be assembled containing:
    | content                    | source     |
    | "# Project Setup"         | setup.md   |
    | "1. Install dependencies"  | setup.md   |
    | "2. Run tests"            | setup.md   |
  And the Swatches should be in document order
  And the source file "setup.md" should be referenced
  And the markdown formatting should be preserved
```

## Operations

Spreads support:

- Assembling Swatches from content search
- Assembling Swatches from related content

## Components

A Spread includes:

1. Retrieved Swatches
2. Source Material references
3. Contextual relationships
4. Metadata
5. Assembly logic

## Assembly

Spreads are assembled by:

1. Processing the Query
2. Retrieving relevant Swatches
3. Including source Materials
4. Organizing context
5. Maintaining relationships

## Usage

Spreads are used to:

- Provide context for tasks
- Support decision making
- Enable understanding
- Maintain relationships
- Support LLM interactions
