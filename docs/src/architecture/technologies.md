# Core Technologies

## Technology Stack

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

The Event Bus is implemented using Tokio's broadcast channels for decoupled communication:

- **Tokio Broadcast Channels**: Using `tokio::sync::broadcast` for event distribution with a default capacity of 128
- **Multiple Consumers**: Supporting multiple subscribers with independent receivers
- **Event Types**: Implemented MaterialDiscovered and System events (Shutdown, HealthCheck)
- **Error Handling**: Comprehensive error handling with EventBusError and detailed logging
- **Subscription Management**: Simple subscribe() method returning a Receiver
- **Event Publishing**: Publish method with receiver count logging and error propagation

### Material Registry & Repositories

- **Material Registry**: Coordinates state management and event publishing
- **Repository Traits**: Abstract persistence operations (`MaterialRepository`, `CutsRepository`)
- **Concrete Implementations**: `SqliteMaterialRepository`, `InMemoryMaterialRepository`, `SqliteCutsRepository`, `InMemoryCutsRepository` provide actual storage logic

### Orchestrator (`QuiltOrchestrator`)

- Manages application lifecycle: initializes actors, event bus, registry, repositories
- Handles configuration parsing (`clap`) and dependency injection (selecting repositories)
- Coordinates startup, main processing flow (e.g., starting discovery), and shutdown
- Uses `tokio::time::timeout` for managing potentially long-running operations

## Project Structure

```
quilt/
├── .github/     # GitHub workflows
├── docs/        # Documentation
│   ├── book/    # Generated site
│   ├── src/     # Documentation source
│   └── book.toml # mdBook configuration
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
│   └── swatching/ # Swatching actor, logic
├── test_dir/    # Test directory
├── Cargo.toml   # Project manifest
└── README.md    # Project overview
```

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
