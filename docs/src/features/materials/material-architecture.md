# Material Architecture

This document outlines the high-level architecture of Quilt's Markdown-only material ingestion pipeline. The system is designed to be reactive and event-driven, with the **Material Registry** acting as both the state store and the event emitter.

## Core Types

### Material

```rust
/// A Material represents a Markdown file in Quilt
pub struct Material {
    /// Unique identifier for the material (CUID)
    pub id: String,
    /// Path to the file on the filesystem
    pub file_path: String,
    /// Type of the material file (currently Markdown)
    pub file_type: MaterialFileType,
    /// Timestamp when the material was first ingested
    pub ingested_at: OffsetDateTime,
    /// Current status of the material
    pub status: MaterialStatus,
    /// Error message if status is Invalid
    pub error: Option<String>,
}

/// The possible states of a material
pub enum MaterialStatus {
    /// Material has been discovered but not yet validated
    Discovered,
    /// Material has passed validation
    Valid,
    /// Material failed validation
    Invalid,
}

/// Events emitted during material processing
pub enum MaterialEvent {
    StatusChanged {
        material: Material,
        old_status: Option<MaterialStatus>,
        error: Option<String>,
    },
}
```

### Usage Examples

```rust
use quilt::{Material, MaterialStatus};

// Create a new material
let material = Material::new("notes/project-overview.md".to_string());
assert_eq!(material.status, MaterialStatus::Discovered);

// Material starts in Discovered state
assert_eq!(material.status, MaterialStatus::Discovered);
assert!(material.error.is_none());

// Material can be validated
let mut validated = material;
validated.status = MaterialStatus::Valid;
assert_eq!(validated.status, MaterialStatus::Valid);

// Or marked as invalid with an error
let mut invalid = material;
invalid.status = MaterialStatus::Invalid;
invalid.error = Some("Missing required sections".to_string());
```

## Component Architecture

### Material Registry

The Material Registry serves as both a state store and event emitter:

```rust
pub struct MaterialRegistry {
    materials: HashMap<String, Material>,
    events: EventEmitter<MaterialEvent>,
}
```

Key features:

- In-memory storage of materials
- Unified upsert operation for adding/updating materials
- Event emission on status changes
- Duplicate path detection
- Error handling for common cases

### Directory Scanner

The scanner provides configurable directory traversal:

```rust
pub struct DirectoryScanner<'a> {
    base_dir: PathBuf,
    registry: &'a mut MaterialRegistry,
    ignore_hidden: bool,
}

impl<'a> DirectoryScanner<'a> {
    pub fn new(base_dir: impl AsRef<Path>, registry: &'a mut MaterialRegistry) -> ScanResult<Self> {
        // Initialize scanner
    }

    pub fn ignore_hidden(mut self, ignore: bool) -> Self {
        // Configure hidden file handling
    }

    pub fn scan(&mut self) -> ScanResult<ScanResults> {
        // Scan directory and register materials
    }
}
```

### Event System

The event system uses a callback-based approach:

```rust
// Subscribe to events
registry.on(|event| {
    match event {
        MaterialEvent::StatusChanged { material, old_status, error } => {
            // Handle status change
        }
    }
});

// Emit events (internal)
self.events.emit(MaterialEvent::StatusChanged {
    material: material.clone(),
    old_status: Some(old_status),
    error: material.error.clone(),
});
```

## Material Lifecycle

1. **Discovery**

   - Scanner finds file in watched directory
   - Basic file system checks (existence, permissions)
   - Hidden file filtering (if configured)

2. **Registration**

   - Material instance created with CUID
   - Duplicate path checking
   - Initial status set to Discovered
   - Discovery event emitted

3. **Validation**

   - File type verification
   - Content validation (if configured)
   - Status updated to Valid/Invalid
   - Status change event emitted

4. **Updates**
   - File changes detected
   - Material re-validated
   - Status updated if needed
   - Events emitted for changes

## Error Handling

### Scanner Errors

```rust
pub enum ScanError {
    PathNotFound(PathBuf),
    IoError(std::io::Error),
}
```

### Registration Failures

- Duplicate path detection
- Permission issues
- IO errors during reading
- Hidden file filtering

### Event Error Handling

Events include:

- Previous status for tracking transitions
- Optional error messages
- Material reference for context

## State Machine

```mermaid
stateDiagram-v2
    [*] --> Discovered: Upsert (new)
    Discovered --> Valid: Validation Success
    Discovered --> Invalid: Validation Failure
    Valid --> Invalid: Validation Failure
    Invalid --> Valid: Validation Success
```

## Event Flow

```mermaid
sequenceDiagram
    participant Client
    participant Scanner
    participant Registry
    participant EventSystem
    participant Listener

    Client->>Scanner: scan()
    Scanner->>Registry: upsert(material)
    Registry->>EventSystem: emit(StatusChanged)
    EventSystem->>Listener: callback(event)

    Note over Scanner,Registry: For each file found
```

## Implementation Status

✅ Completed:

- Basic Material struct with CUID-based IDs
- MaterialStatus enum for state tracking
- MaterialRegistry with unified upsert operation
- Event system for status changes
- Error handling for common cases
- Directory scanning with hidden file support
- Configuration options for scanning behavior

🚧 In Progress:

- Basic validation rules
- File watching support

## Future Enhancements

1. **Async Support**

   ```rust
   impl MaterialRegistry {
       pub async fn upsert(&self, material: Material) -> Option<Material> {
           // Async implementation
       }
   }
   ```

2. **Improved Validation**

   ```rust
   pub trait MaterialValidator {
       fn validate(&self, material: &Material) -> ValidationResult;
   }
   ```

3. **File Watching**
   ```rust
   impl MaterialRegistry {
       pub async fn watch_directory(&self, path: &str) -> Result<(), Error> {
           // Implement file watching
       }
   }
   ```
