# Directory Scanner

The Directory Scanner is Quilt's tool for discovering and registering materials in your filesystem. It handles the initial discovery phase of the material lifecycle, managing how files are found and which ones are included or excluded.

## Overview

The Directory Scanner is responsible for:

- Walking through directories to find potential files
- Applying path-based filters (hidden files, exclude patterns)
- Converting found files into Materials
- Registering valid materials in the system

Note: The Directory Scanner does not filter by file type - it discovers all files in the directory. File type validation and filtering happens when files are converted to Materials.

## Basic Usage

The most basic way to use the Directory Scanner is to point it at a directory:

```rust
let mut registry = MaterialRegistry::new();
let mut scanner = DirectoryScanner::new(path, &mut registry)?;
let results = scanner.scan()?;
```

## Configuration Options

### Hidden Files

By default, the Directory Scanner ignores hidden files and directories (those starting with a dot). You can control this behavior:

```rust
// Include hidden files and directories
let scanner = DirectoryScanner::new(path, &mut registry)?
    .ignore_hidden(false);
```

### Exclude Patterns

You can tell the Directory Scanner to skip certain paths using exclude patterns. These patterns match against the full path, not file types:

```rust
let scanner = DirectoryScanner::new(path, &mut registry)?
    .exclude(vec!["target/", "node_modules/"]);
```

Common use cases for excludes:

- Build directories (`target/`, `dist/`)
- Package management (`node_modules/`, `vendor/`)
- Version control (`.git/`)
- Editor files (`.vscode/`, `.idea/`)

## Command Line Interface

When using Quilt from the command line, you can specify exclude patterns:

```bash
# Exclude a single directory
quilt scan . --exclude target/

# Exclude multiple directories (comma-separated)
quilt scan . --exclude target/,node_modules/,.git/
```

## Scan Results

The Directory Scanner provides detailed results of its operation:

```rust
struct ScanResults {
    registered: Vec<Material>,  // Successfully registered materials
    failed: Vec<Material>       // Materials that couldn't be registered
}
```

Common reasons for failed registration:

- Duplicate file paths
- File access errors
- Invalid file types

## Common Scenarios

### Basic Directory Scan

```gherkin
Scenario: Scanning a Directory
  Given a directory containing markdown files
  When the Directory Scanner processes the directory
  Then it finds all non-hidden files
  And registers them as materials
```

### Excluding Paths

```gherkin
Scenario: Excluding Build Directories
  Given a project with build artifacts
  When the Directory Scanner runs with exclude patterns
  Then it skips files in excluded directories
  But still processes other valid files

Scenario: Multiple Exclude Patterns
  Given multiple directories to exclude
  When the Directory Scanner is configured with multiple patterns
  Then it skips files matching any of the patterns
  And processes all other files normally
```

### Hidden Files

```gherkin
Scenario: Default Hidden File Behavior
  Given a directory with hidden files
  When the Directory Scanner runs with default settings
  Then it skips all hidden files and directories

Scenario: Including Hidden Files
  Given a directory with hidden files
  When the Directory Scanner is configured to include hidden files
  Then it processes both hidden and non-hidden files
```

## Current Status

- ✅ Basic directory scanning
- ✅ Hidden file configuration
- ✅ Path-based exclude patterns
- ✅ Detailed scan results
- ✅ Error handling

## Next Steps

Future improvements being considered:

1. Glob pattern support for more flexible path matching
2. Include patterns to specify which paths to process
3. Watch mode for real-time file system changes
4. More granular control over symlinks and special files
