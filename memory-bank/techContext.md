# Technical Context

## Technology Stack

### Core Technologies

- **Language**: Rust (stable channel, Edition 2021)
- **Actor Framework**: Actix (v0.13.1)
- **Async Runtime**: Tokio (v1.44.2)
- **Documentation**: mdBook with admonish extension
- **Build System**: Cargo

### Key Dependencies

- **thiserror** (v2.0.12): Error handling with derive macros
- **time** (v0.3): Date and time utilities with serde support
- **walkdir** (v2.4.0): Filesystem traversal for material discovery
- **cuid2** (v0.1.2): Collision-resistant IDs for materials and swatches
- **log** (v0.4.20): Logging facade
- **env_logger** (v0.11.8): Environment-based logging configuration

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

# Check and apply code formatting
cargo fmt --all -- --check
cargo fmt

# Run Clippy linter
cargo clippy --all-targets --all-features -- -D warnings

# Documentation server
cd docs && mdbook serve
```

## Project Structure

```
quilt/
├── .github/     # GitHub workflows
├── docs/        # Documentation
│   ├── book/    # Generated site
│   ├── src/     # Documentation source
│   └── book.toml # mdBook configuration
├── memory-bank/ # Memory Bank
├── src/         # Source code
│   ├── actors/  # Actor system components
│   ├── discovery/ # Discovery actor module
│   ├── materials/ # Material processing
│   └── main.rs  # Application entry point
├── test_dir/    # Test directory
├── Cargo.toml   # Project manifest
└── README.md    # Project overview
```

## Implementation Approach

The project follows an incremental implementation plan with 10 clear milestones:

1. ✅ **Core Actor System**: Set up Actix actor framework, basic message types
2. ✅ **Discovery Actor**: Integrate DirectoryScanner with DiscoveryActor
3. **Message Channel System**: Connect discovery to message channel system
4. **Cutting Actor**: Implement document content extraction and fragmentation
5. **Cuts Repository**: Store processed cuts for retrieval
6. **Cutting to Labeling**: Set up channels and message passing
7. **Labeling Actor**: Implement swatch creation and metadata enrichment
8. **Swatch Repository**: Store processed swatches
9. **Query Interface**: Implement basic search functionality
10. **Persistence**: Add persistence for repositories

## Technical Constraints

### Performance Requirements

- Low latency for query responses (<100ms)
- Minimal memory footprint
- Efficient disk usage for embeddings
- Ability to process documents incrementally

### Development Constraints

- Local-first, privacy-preserving approach
- No cloud dependencies for core functionality
- Cross-platform compatibility (Linux, macOS, Windows)
- Modular, pluggable architecture

## Implementation Details

### Actor System

The actor system uses Actix with proper lifecycle management:

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
```

### Message Channel System

The message system uses Tokio's MPSC channels with capacity of 100 messages:

```rust
pub enum MaterialMessage {
    Discovered(Material),      // Full material for initial discovery
    Cut(String),               // Just material ID to minimize size
    Swatched(String),          // Just material ID to minimize size
    Error(String, String),     // Material ID and error message
    Shutdown,                  // Signal to stop processing
}
```
