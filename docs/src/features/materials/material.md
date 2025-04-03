# Material

A Material is any text or markdown file that Quilt can read and work with. These files are the starting point for everything Quilt does — they’re where your content lives before it gets broken down and analyzed.

## Overview

Materials are the foundation of Quilt's memory system. When you tell Quilt to ingest a specific directory, it scans for `.md` files and brings them in as Materials. From there, they get registered and made available for processing.

## What Counts as a Material?

Materials are:

- Markdown files
- Stored in a folder you've chosen
- Used as the source of truth — Quilt doesn’t change them
- Structured or unstructured — both are fine

### Examples

- `notes/project-overview.md`

## What Quilt Tracks About Each Material

When Quilt picks up a file, it gathers just enough information to track it reliably — no full processing yet, just metadata:

- **Material ID**: A unique internal name for this file
- **File Path**: Where the file lives on your system
- **File Type**: `.md`
- **Ingested At**: The moment Quilt first saw it
- **Status**: Whether it's ready, failed, or waiting to be processed

This minimal data lets Quilt stay lightweight until it actually needs to analyze the file in depth.

## What Happens When You Add Files

### Quilt Automatically Tries to Ingest New Materials

```gherkin
Scenario: Adding Markdown Files
  Given you provide a directory containing `.md` files
  When Quilt sees those files
  Then it recognizes them as valid Materials
  And adds them to the Material Registry
  So they’re ready for downstream processing
```

## Handling Ingestion Issues

Quilt does its best to quietly and reliably pick up your materials — but sometimes a file might not be usable right away. It could be in the wrong format, unreadable, or already registered.

To help you stay informed, Quilt is designed to:

- Detect and handle problematic files gracefully
- Keep track of why certain files couldn't be ingested
- Let you see the status and reason for any skipped or failed files

This ensures Quilt remains stable and trustworthy, while giving you visibility and control over your content.

## Feature: Error Handling (User-Facing)

```gherkin
Scenario: File with Unsupported Extension is Encountered
  Given a directory contains a file with an unsupported extension
  When the file is scanned during ingestion
  Then the file should be skipped
  And the user can view a failed processing status with reason "Unsupported file type"

Scenario: Material Fails Validation
  Given a text or markdown file is detected
  When the file fails validation (e.g., unreadable, empty)
  Then the file should not be registered in the Material Registry
  And the user can view a failed processing status with reason "Validation failed"

Scenario: Duplicate Material is Found
  Given a material has already been registered with the same path
  When the ingestion process is run again
  Then the duplicate should be ignored
  And the user can view a status with reason "Already registered"

Scenario: Inaccessible File Path
  Given the directory includes a file with restricted access
  When the ingestion process attempts to read the file
  Then the file should be skipped
  And the user can view a failed processing status with reason "Permission denied"

Scenario: Unexpected IO Error During Ingestion
  Given a file is being read for ingestion
  When an unexpected IO error occurs
  Then the ingestion process should handle the exception gracefully
  And the user can view a failed processing status with reason "I/O error"
```

## Lifecycle of a Material

When you add a file, Quilt:

1. **Discovers it** in your folder
2. **Validates it** (file type, readability, etc.)
3. **Registers it** in the internal Material Registry
4. **Marks it as ready** for deeper processing in the Swatch Pipeline
5. **Keeps track** of changes if the file gets updated later

You’ll always have a clear view of what was picked up, what wasn’t, and why.
