# Technical Context

## Technology Stack

### Core Technologies

- **Language**: Rust (stable channel)
- **Actor Framework**: Actix
- **Async Runtime**: Tokio
- **Documentation**: mdBook with admonish extension
- **Build System**: Cargo (Rust's package manager)
- **CI/CD**: GitHub Actions for PR validation
- **Code Quality**: rustfmt, Clippy
- **Rust Language** (Edition 2021): Primary programming language for the project
- **Actix** (v0.13.1): Actor framework for implementing message-based concurrency
- **Tokio** (v1.44.2): Async runtime, providing tasks, synchronization primitives, and channels
  - Features used: macros, rt, rt-multi-thread, sync, time
- **thiserror** (v2.0.12): Error handling with derive macros for custom error types
- **time** (v0.3): Date and time utilities with serde support
- **walkdir** (v2.4.0): Filesystem traversal for material discovery
- **cuid2** (v0.1.2): Collision-resistant IDs for materials and swatches
- **log** (v0.4.20): Logging facade for Rust
- **env_logger** (v0.11.8): Environment-based logging configuration

### Key Dependencies

- **Actix**: Actor framework for message-based concurrency
- **Tokio**: Async runtime and concurrency primitives
- **Serde**: Serialization and deserialization
- **Notify**: File system watching
- **Local embedding models**: Using libraries like `llama.cpp` or similar for local embeddings
- **Vector storage**: Implementation TBD (considering HNSW, LSH, or similar algorithms)

### Development Dependencies

- **tempfile** (v3.10.1): Creating temporary directories for testing
- **futures** (v0.3): Future utilities for testing async code

## Development Environment

### Setup Requirements

- Rust toolchain (rustc, cargo)
- mdBook (for documentation)
- mdbook-admonish (for advanced documentation features)

### Development Commands

```bash
# Build the project
cargo build

# Run tests
cargo test

# Check code formatting
cargo fmt --all -- --check

# Apply code formatting
cargo fmt

# Run Clippy linter
cargo clippy --all-targets --all-features -- -D warnings

# Documentation server
cd docs && mdbook serve

# Stop documentation server
pkill -f "mdbook serve"
```

## Project Structure

```
quilt/
├── .cursor/     # Cursor IDE configuration
├── .git/        # Git repository
├── .github/     # GitHub configuration
│   └── workflows/ # GitHub Actions workflows
├── docs/        # Documentation
│   ├── book/    # Generated documentation site
│   ├── src/     # Documentation source
│   │   └── development/ # Development docs
│   └── book.toml # mdBook configuration
├── memory-bank/ # Memory Bank for Cursor
├── src/         # Source code
│   ├── actors/  # Actor system components and common messages
│   ├── discovery/ # Discovery actor module
│   ├── materials/ # Material processing components
│   │   └── tests/ # Integration tests
│   ├── lib.rs   # Library entry point
│   └── main.rs  # Application entry point
├── target/      # Build artifacts
├── test_dir/    # Test directory for development
├── Cargo.toml   # Project manifest
├── Cargo.lock   # Dependency lock file
├── rustfmt.toml # Formatting configuration
├── .clippy.toml # Linting configuration
└── README.md    # Project overview
```

## Implementation Approach

### Milestone Plan

The project follows an incremental implementation plan with clear milestones:

1. **Core Actor System** (1-2 weeks) ✅

   - Set up Actix actor framework
   - Implement basic message types
   - Create discovery actor
   - Add structured logging

2. **Discovery Actor Enhancement** (2-3 weeks)

   - Integrate DirectoryScanner with DiscoveryActor
   - Implement material creation from scanned files
   - Connect to message channel system

3. **Cutting Actor Implementation** (2-3 weeks)

   - Implement Cutting Actor
   - Add document content extraction
   - Create text fragmentation strategies

4. **Labeling Actor Implementation** (2-3 weeks)

   - Implement Labeling Actor
   - Add metadata extraction
   - Integrate with embedding models

5. **Query Interface** (2-3 weeks)

   - Implement basic search functionality
   - Create query processing logic
   - Add results formatting

6. **Persistence** (2-3 weeks)
   - Add persistence for repositories
   - Implement startup/shutdown persistence
   - Create consistency checks and error recovery

## Technical Constraints

### Performance Requirements

- Low latency for query responses (<100ms)
- Minimal memory footprint
- Efficient disk usage for embeddings
- Ability to process documents incrementally

### Development Constraints

- Focus on local-first, privacy-preserving approach
- No cloud dependencies for core functionality
- Cross-platform compatibility (Linux, macOS, Windows)
- Modular, pluggable architecture

## Implementation Details

### Actor System

The actor system is implemented using the Actix framework:

- **Actor Trait**: Base trait for all actors in the system
- **Message Types**: Type-safe messages with defined response types
- **Actor Lifecycle**: Proper handling of actor startup and shutdown
- **Actor Organization**: Modular structure with actors in dedicated modules

Example of an actor implementation:

```rust
pub struct DiscoveryActor {
    name: String,
}

impl Actor for DiscoveryActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("DiscoveryActor '{}' started", self.name);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("DiscoveryActor '{}' stopped", self.name);
    }
}

impl Handler<Ping> for DiscoveryActor {
    type Result = bool;

    fn handle(&mut self, _msg: Ping, _ctx: &mut Self::Context) -> Self::Result {
        debug!("DiscoveryActor '{}' received ping", self.name);
        true
    }
}
```

### Material Repository

The repository is implemented as a thread-safe in-memory store using `Arc<RwLock<HashMap<String, Material>>>`. It provides CRUD operations with idempotence checks and state transition validation.

### Message Channel System

The message system uses Tokio's MPSC channels with a carefully chosen capacity (100 messages) that balances throughput with memory usage. It implements:

- **MaterialMessage Enum**: Five message types representing the processing pipeline:

  ```rust
  pub enum MaterialMessage {
      Discovered(Material),      // Full material for initial discovery
      Cut(String),               // Just material ID to minimize size
      Swatched(String),          // Just material ID to minimize size
      Error(String, String),     // Material ID and error message
      Shutdown,                  // Signal to stop processing
  }
  ```

- **Channel Management**: Factory functions and helper methods for creating and using channels:

  ```rust
  // Creating channels
  pub fn create_channel() -> ChannelPair { /* ... */ }

  // Extension trait for sender
  pub trait MaterialChannelExt {
      async fn send_message(&self, message: MaterialMessage) -> Result<(), ChannelError>;
      async fn try_send_message_timeout(&self, message: MaterialMessage,
                                      timeout_duration: Duration) -> Result<(), ChannelError>;
      async fn send_shutdown(&self) -> Result<(), ChannelError>;
  }
  ```

- **Error Handling**: Structured error types for different failure scenarios:
  ```rust
  pub enum ChannelError {
      SendError(String),
      ReceiveTimeout(Duration),
      ChannelClosed,
  }
  ```

### Logging System

The logging system is implemented using the log crate with env_logger:

- **Log Levels**: Different log levels for different types of information (debug, info, warn, error)
- **Environment Configuration**: Configurable via environment variables (RUST_LOG)
- **Structured Logging**: Includes timestamp, log level, and module path

### Type System

- **Material Struct**: Represents documents with metadata and state tracking
- **MaterialStatus Enum**: Tracks the processing state (Discovered, Cut, Swatched, Error)
- **MaterialFileType Enum**: Categorizes materials by file type (Markdown, Text, Other)

### Testing Approach

- **Unit Testing**: Comprehensive tests for each component's behavior
- **Integration Testing**: Tests for how components work together (e.g., message flow through channels)
- **Async Testing**: Using Tokio's test utilities for async code
- **CI/CD Pipeline**: GitHub Actions workflow for automatic PR validation
- **Code Quality Tools**:
  - rustfmt with custom configuration for code style
  - Clippy with tailored rules for static analysis
  - Documented standards in the development guide
- **Test Organization**:
  - Unit tests located alongside the code they test
  - Integration tests in dedicated test modules
  - Test utilities to simplify test creation
