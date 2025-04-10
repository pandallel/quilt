# Technical Context

## Core Technologies

- **Rust**: The primary programming language for the project
- **Actix**: Actor framework for managing actor lifecycle
- **Tokio**: Async runtime and concurrency primitives
- **Tokio-broadcast**: Event distribution for the Event Bus
- **RwLock**: Thread-safe state management

## Architecture Components

### Actor System

Quilt uses Actix for actor lifecycle management, leveraging its robust features:

- **Actor Trait**: For defining actor behavior and message handling
- **Message Types**: Strongly typed with `#[derive(Message)]` macro
- **Handler Implementation**: Using `ResponseFuture` for async processing
- **Actor Lifecycle**: Proper startup and shutdown management

### Event Bus

The Event Bus is implemented using Tokio's broadcast channels:

```rust
pub struct EventBus {
    material_tx: broadcast::Sender<MaterialEvent>,
    system_tx: broadcast::Sender<SystemEvent>,
    error_tx: broadcast::Sender<ErrorEvent>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (material_tx, _) = broadcast::channel(capacity);
        let (system_tx, _) = broadcast::channel(capacity);
        let (error_tx, _) = broadcast::channel(capacity);

        Self {
            material_tx,
            system_tx,
            error_tx,
        }
    }

    pub fn subscribe_to_material_events(&self) -> broadcast::Receiver<MaterialEvent> {
        self.material_tx.subscribe()
    }

    // Other methods for subscription and publishing
}
```

### Material Registry

The Material Registry coordinates state management and event publishing:

```rust
pub struct MaterialRegistry {
    materials: Arc<RwLock<HashMap<String, Material>>>,
    event_bus: Arc<EventBus>,
    repository: Arc<dyn MaterialRepository>,
}

impl MaterialRegistry {
    pub async fn register_material(&self, material: Material) -> Result<(), RegistryError> {
        // Update state
        let mut materials = self.materials.write().await;
        materials.insert(material.id.clone(), material.clone());

        // Persist change
        self.repository.save(&material).await?;

        // Publish event
        self.event_bus.publish_material_event(MaterialEvent::Discovered {
            material_id: material.id.clone(),
            file_path: material.file_path.clone(),
        })?;

        Ok(())
    }

    // Other methods for state management
}
```

### Material Repository

The Repository handles persistence operations:

```rust
#[async_trait]
pub trait MaterialRepository: Send + Sync + 'static {
    async fn save(&self, material: &Material) -> Result<()>;
    async fn load(&self, id: &str) -> Result<Option<Material>>;
    async fn delete(&self, id: &str) -> Result<()>;
}
```

### Event Types

The system uses domain events for communication:

```rust
// Strongly typed material identifier
pub struct MaterialId(String);

pub enum MaterialEvent {
    Discovered {
        material_id: MaterialId,
        file_path: String,
    },
    Cut {
        material_id: MaterialId,
        cut_ids: Vec<String>,
    },
    Swatched {
        material_id: MaterialId,
        swatch_id: String,
    },
}

pub enum SystemEvent {
    HealthCheck { actor_id: String },
    Shutdown,
}

pub enum ErrorEvent {
    ProcessingFailed {
        material_id: MaterialId,
        stage: ProcessingStage,
        error: String,
    },
    PersistenceFailed {
        material_id: MaterialId,
        operation: String,
        error: String,
    },
}

pub enum ProcessingStage {
    Discovery,
    Cutting,
    Swatching,
}
```

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
- **Run**: `cargo run -- --discovery-dir=/path/to/scan`

### Continuous Integration

- **GitHub Actions**: For automated testing
- **Rust Coverage**: For test coverage reporting

## Implementation Approach

The system is being implemented incrementally, with focused milestones:

1. **Foundation**: Actor system, Material Repository, Discovery system
2. **Event Infrastructure**: Event Bus, Material Registry, event types
3. **Actor Evolution**: Updating actors to use event-driven approach
4. **Processing Pipeline**: Implementing the full material processing pipeline
5. **Persistence**: Adding durable storage and recovery mechanisms

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
│   ├── actors/  # Actor system components
│   ├── discovery/ # Discovery actor module
│   ├── materials/ # Material processing
│   └── main.rs  # Application entry point
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

### Message Channel System

The message system uses Tokio's MPSC channels with capacity of 100 messages for direct actor-to-actor communication.

### Message Types

Each actor has its own specific message types for direct communication:

pub struct MaterialDiscovered {
pub material: Material, // Full material for initial registration
}

pub struct CutMaterial {
pub material_id: String, // Reference to original material
pub cut_ids: Vec<String>, // IDs of the generated cuts
}

pub struct MaterialSwatched {
pub material_id: String, // Reference to original material
pub cut_id: String, // Reference to the cut that was processed
pub swatch_id: String, // ID of the generated swatch
}

pub struct ProcessingError {
pub material_id: String,
pub stage: ProcessingStage,
pub error: String,
}

pub enum ProcessingStage {
Discovery,
Cutting,
Swatching,
}
