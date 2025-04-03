# Memory Building

The first feature of Sidecar is basic memory building from a static directory. This tiny slice demonstrates how Sidecar will understand and make your files searchable in a semantically meaningful way.

## Feature: Semantic Search

```gherkin
Feature: Semantic Search
  As a user
  I want to find relevant content from my files
  So that I can get context that matches my intent, not just exact matches

  Background:
    Given I have a directory of text files
    And the directory contains markdown and plain text files
    And I have a local embedding model available

  Scenario: Indexing a directory
    When I run "sidecar index /path/to/directory"
    Then I should see "Indexing /path/to/directory..."
    And I should see how many files were found
    And I should see "Building semantic index..."

  Scenario: Finding semantically relevant content
    Given I have indexed a directory
    When I run "sidecar search 'how do I handle errors in my code'"
    Then I should see a list of relevant file sections
    And each section should be semantically related to error handling
    And each result should show the file path and context

  Scenario: No relevant content found
    Given I have indexed a directory
    When I run "sidecar search 'quantum computing algorithms'"
    Then I should see "No relevant content found"

  Scenario: Empty directory
    Given I have an empty directory
    When I run "sidecar index /path/to/empty/directory"
    Then I should see "No files found to index"
```

## Implementation Notes

- Use a local embedding model (e.g., llama.cpp) from the start
- Focus on semantic similarity, not text matching
- Keep embeddings in memory initially
- Return relevant sections of text, not just line numbers
- Prioritize context and relevance over exact matches

## Next Steps

After this basic semantic search is working, we can:

- Add file watching for updates
- Implement more sophisticated embedding models
- Add persistence for embeddings
- Support more file types
- Add configuration for embedding models
