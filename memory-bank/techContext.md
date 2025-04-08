# Technical Context

## Technology Stack

### Core Technologies

- **Language**: Rust (stable channel)
- **Async Runtime**: Tokio
- **Documentation**: mdBook with admonish extension
- **Build System**: Cargo (Rust's package manager)
- **CI/CD**: GitHub Actions for PR validation
- **Code Quality**: rustfmt, Clippy
- **Rust Language** (Edition 2021): Primary programming language for the project
- **Tokio** (v1.44.2): Async runtime, providing tasks, synchronization primitives, and channels
  - Features used: macros, rt, rt-multi-thread, sync, time
- **thiserror** (v1.0.57): Error handling with derive macros for custom error types
- **time** (v0.3): Date and time utilities with serde support
- **walkdir** (v2.4.0): Filesystem traversal for material discovery
- **cuid2** (v0.1.2): Collision-resistant IDs for materials and swatches

### Key Dependencies

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

1. **Core Material Processing Pipeline** (2-3 weeks)

   - Material Repository setup
   - Basic data structures
   - Message channel system
   - Minimal actor framework

2. **Basic File Monitoring and Processing** (3-4 weeks)

   - Discovery Worker enhancement
   - Cutting Worker implementation
   - Basic storage
   - Error handling and logging

3. **Embedding and Semantic Search** (4-5 weeks)

   - Embedding model integration
   - Vector store implementation
   - Query interface
   - Spread generation

4. **Concurrency and Scaling** (3-4 weeks)

   - Worker pools
   - Repository optimization
   - Backpressure mechanisms
   - Resource management

5. **User Experience and Integration** (4-6 weeks)

   - CLI interface
   - Simple web UI
   - Integration APIs
   - User configuration

6. **Production Readiness** (3-5 weeks)
   - Comprehensive testing
   - Security features
   - Stability and reliability
   - Documentation and examples

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
