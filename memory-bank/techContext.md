# Technical Context

## Technology Stack

### Core Technologies

- **Language**: Rust (stable channel)
- **Async Runtime**: Tokio
- **Documentation**: mdBook with admonish extension
- **Build System**: Cargo (Rust's package manager)

### Key Dependencies

- **Tokio**: Async runtime and concurrency primitives
- **Serde**: Serialization and deserialization
- **Notify**: File system watching
- **Local embedding models**: Using libraries like `llama.cpp` or similar for local embeddings
- **Vector storage**: Implementation TBD (considering HNSW, LSH, or similar algorithms)

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

# Run with specific features
cargo run --features=feature-name

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
├── docs/        # Documentation
│   ├── book/    # Generated documentation site
│   ├── src/     # Documentation source
│   └── book.toml # mdBook configuration
├── memory-bank/ # Memory Bank for Cursor
├── src/         # Source code
│   ├── materials/ # Material processing components
│   ├── lib.rs   # Library entry point
│   └── main.rs  # Application entry point
├── target/      # Build artifacts
├── test_dir/    # Test directory for development
├── Cargo.toml   # Project manifest
├── Cargo.lock   # Dependency lock file
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
