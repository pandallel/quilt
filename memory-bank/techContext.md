# Technical Context

## Core Technologies

- **Rust**: The primary programming language for the project
- **Actix**: Actor framework for managing actor lifecycle
- **Tokio**: Async runtime and concurrency primitives
- **`tokio::sync::broadcast`**: Event distribution for the Event Bus
- **`tokio::sync::mpsc`**: Internal actor queues for backpressure
- **`tokio::sync::RwLock` / `Mutex`**: Thread-safe state management
- **`clap`**: Command-line argument parsing
- **`env_logger`**: Logging setup
- **`thiserror`**: Structured error handling
- **`sqlx`**: Asynchronous SQL interaction (for SQLite persistence)
- **`text-splitter`**: Text chunking for the cutting process

## Architecture Components

### Actor System

Quilt uses Actix for actor lifecycle management, leveraging its robust features:

- **Actor Trait**: For defining actor behavior and message handling
- **Message Types**: Strongly typed with `#[derive(Message)]` macro
- **Handler Implementation**: Using `ResponseFuture` or `async move` blocks for async processing
- **Actor Lifecycle**: Proper startup (`started`) and shutdown (`stopping`, `stopped`) management via `Context`.
- **Common Utilities**: Shared messages (`Ping`, `Shutdown`) and error types (`ActorError`) are defined in `src/actors/mod.rs` for reuse.

### Event Bus

The Event Bus is implemented using Tokio's broadcast channels for decoupled communication.

### Material Registry

The Material Registry coordinates state management and event publishing.

### Material & Cuts Repositories

The Repository traits handle persistence operations.

- **Concrete Implementations**: `SqliteMaterialRepository`, `InMemoryMaterialRepository`, `SqliteCutsRepository`, `InMemoryCutsRepository` provide actual storage logic.

### Orchestrator (`QuiltOrchestrator`)

- Manages application lifecycle: initializes actors, event bus, registry, repositories.
- Handles configuration parsing (`clap`) and dependency injection (selecting repositories).
- Coordinates startup, main processing flow (e.g., starting discovery), and shutdown.
- Uses `tokio::time::timeout` for managing potentially long-running operations.

### Event Types

The system uses a unified domain event enum (`QuiltEvent`) for communication via the Event Bus.

### Cutting Strategy (`TextCutter`)

- Responsible for splitting material content into chunks (`Cut`s).
- Uses the `text-splitter` crate.
- May employ different splitting strategies based on file type (Markdown, Code, Plain Text).

## Development Environment

### Required Tools

- **Rust Toolchain**: Latest stable version
- **Cargo**: For dependency management and building
- **Clippy**: For linting
- **Rustfmt**: For code formatting
- **Visual Studio Code**: Recommended IDE with rust-analyzer

### Build and Test

- **Build**: `cargo build`
- **Test**: `cargo test`
- **Run**: `cargo run -- --dir <path> [--in-memory] [--exclude <pattern>...] [--include-hidden]`

### Continuous Integration

- **GitHub Actions**: For automated testing
- **Rust Coverage**: For test coverage reporting

## Implementation Approach

The system is being implemented incrementally, with focused milestones:

1. **Foundation**: Actor system, Material Repository, Discovery system
2. **Event Infrastructure**: Event Bus, Material Registry, event types
3. **Actor Evolution**: Updating actors to use event-driven approach
4. **Processing Pipeline**: Implementing the full material processing pipeline
5. **Persistence**: Adding durable storage (SQLite) and recovery mechanisms
6. **Orchestration**: Implementing `QuiltOrchestrator` for lifecycle management

Each milestone focuses on providing tangible, demonstrable progress that can be validated through concrete log messages and metrics.

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
│   ├── actors/  # Common actor definitions (messages, errors)
│   ├── cutting/ # Cutting actor, logic, repository trait/impls
│   ├── db.rs    # Database initialization (e.g., SQLite pool)
│   ├── discovery/ # Discovery actor, logic, scanner
│   ├── events/  # Event definitions (e.g., QuiltEvent) and EventBus
│   ├── lib.rs   # Library root (re-exports)
│   ├── main.rs  # Application entry point, arg parsing (clap)
│   ├── materials/ # Material registry, repository trait/impls, types
│   ├── orchestrator.rs # QuiltOrchestrator implementation
│   └── swatching/ # Swatching actor, logic (potentially future)
├── test_dir/    # Test directory
├── Cargo.toml   # Project manifest
└── README.md    # Project overview
```

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

### Message Types & Communication

- **Event Bus (`QuiltEvent`)**: Primary mechanism for decoupled communication between stages (e.g., Discovery -> Registry -> EventBus -> CuttingActor). Events are published via the shared `EventBus` (using `tokio::sync::broadcast`).
- **Actix Messages**: Used for direct requests/responses between actors or components (e.g., `Orchestrator` sending `StartDiscovery` to `DiscoveryActor`, sending `Ping` for health checks). Defined using `#[derive(Message)]` and handled via `impl Handler<...>`.
