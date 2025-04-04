# Material

A Material is any text or markdown file that Quilt can read and work with. These files are the starting point for everything Quilt does — they represent the source files that will be processed into Swatches.

## Overview

Materials are the foundation of Quilt's memory system. When you tell Quilt to ingest a specific directory, it scans for `.md` files and brings them in as Materials. From there, they get registered and made available for Swatch creation.

## What Counts as a Material?

Materials are:

- Markdown files (currently the only supported type)
- Stored in a folder you've chosen
- Used as the source of truth — Quilt doesn't change them
- Tracked for changes and updates

## What Quilt Tracks

When Quilt picks up a file, it keeps track of:

- A unique identifier for each file
- The file's location on your system
- When it was first discovered
- Its current status (discovered, swatched, or error)
- Any errors or issues that come up during Swatch creation

## Lifecycle of a Material

When you add a file, Quilt:

1. **Discovers it** in your folder
2. **Registers it** in its internal system
3. **Processes it** into Swatches [planned]
4. **Tracks changes** if the file gets updated later [planned]

You'll always have a clear view of what was picked up, what was successfully processed into Swatches, and what had issues.

## Features

### Automatic Discovery

Quilt automatically finds and tracks your materials:

- Scans directories you specify
- Picks up new files as they're added
- Tracks file status and changes
- Configurable handling of hidden files and directories

### Smart Error Handling

Quilt does its best to quietly and reliably pick up your materials — but sometimes a file might not be usable right away. It could be inaccessible, already registered, or unsuitable for Swatch creation.

To help you stay informed, Quilt is designed to:

- Detect and handle problematic files gracefully
- Keep track of why certain files couldn't be processed
- Let you see the status and reason for any errors
- Provide configuration options for hidden files and symlinks

This ensures Quilt remains stable and trustworthy, while giving you visibility and control over your content.

## Common Scenarios and Error Handling

### Adding New Content

```gherkin
Scenario: Adding New Files
  Given you add new markdown files to a watched directory
  When Quilt scans the directory
  Then it automatically discovers and registers them
  And marks them as ready for Swatch creation
```

### Handling Hidden Files

```gherkin
Scenario: Hidden File is Encountered
  Given a directory contains hidden files or directories
  When the directory scanner runs
  Then hidden files are skipped by default
  But you can configure Quilt to include them using settings

Scenario: Hidden Directory Contains Important Files
  Given you have important files in a hidden directory
  When you configure Quilt to include hidden items
  Then it will process all files, including those in hidden directories
```

### Dealing with Problems

```gherkin
Scenario: Duplicate File is Found
  Given a file has already been registered
  When the same file is encountered again
  Then Quilt will:
    * Skip the duplicate file
    * Keep the original version
    * Track this in its processing results
    * Let you know about the duplicate

Scenario: Can't Access a File
  Given a file exists but can't be accessed
  When Quilt tries to process it
  Then it will:
    * Mark the file as Error
    * Record the access error
    * Continue processing other files
    * Include this in its processing report

Scenario: File System Issues
  Given unexpected problems occur (disk errors, etc)
  When Quilt encounters these issues
  Then it will:
    * Handle the error gracefully
    * Continue processing other files
    * Keep track of what failed
    * Provide error details in its report
```

## Current Status

- ✅ Automatic material discovery
- ✅ Detailed error reporting
- ✅ Hidden file configuration
- ✅ Robust error handling
- ✅ Status tracking
- 🚧 File watching (coming soon)
- 🚧 Swatch creation integration (coming soon)

## Next Steps

Quilt is continuously evolving. Coming soon:

1. Real-time file watching
2. Integration with Swatch creation
3. Support for more file types

For technical implementation details, see the [Material Architecture](./material-architecture.md) document.
