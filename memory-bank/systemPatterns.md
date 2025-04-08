# System Patterns

## Architecture Overview

Quilt uses an **actor model architecture** implemented with Actix. The system processes materials through a pipeline of specialized actors, with a thread-safe repository serving as the single source of truth.

```mermaid
graph TB
    QO[QuiltOrchestrator]

    subgraph "Processing Actors"
        DA[Discovery Actor]
        CA[Cutting Actor]
        LA[Labeling Actor]
    end

    Repo[Material Repository]

    QO -->|Manages| DA
    QO -->|Manages| CA
    QO -->|Manages| LA

    DA -->|Detect Material| Repo
    DA -->|Send Discovered Material| CA
    CA -->|Process material and update state to Cut| Repo
    CA -->|Send Cut Material| LA
    LA -->|Process material and update state to Swatched| Repo
```

## Key Design Patterns

### Orchestrator Pattern

- **QuiltOrchestrator**: Central component responsible for actor lifecycle management
- Handles actor initialization, message flow coordination, and graceful shutdown
- Centralizes error handling and configuration management

### Actor Model Implementation

- **Actix Framework**: Using Actix for message-based concurrency
- **Actor Lifecycle**: Proper handling of actor startup and shutdown
- **Direct Messaging**: Actors communicate via typed message passing
- **Shared State**: Thread-safe repository provides consistent state

### Message Flow Patterns

```mermaid
sequenceDiagram
    participant DiscoveryActor
    participant CuttingActor
    participant LabelingActor
    participant Repository

    DiscoveryActor->>Repository: Register material (status: Discovered)
    DiscoveryActor->>CuttingActor: Send Discovered(Material)
    CuttingActor->>Repository: Get material if needed
    CuttingActor->>Repository: Update status to Cut
    CuttingActor->>LabelingActor: Send Cut(material_id)
    LabelingActor->>Repository: Get material
    LabelingActor->>Repository: Update status to Swatched

    alt Error occurs
        CuttingActor->>Repository: Update status to Error
        CuttingActor->>LabelingActor: Send Error(material_id, error_message)
    end
```

### Material Processing Pipeline

1. **Discovery Stage**: Scans for new/updated materials, registers them, sends discovery messages
2. **Cutting Stage**: Receives discovery messages, cuts materials into swatches, sends cut messages
3. **Labeling Stage**: Receives cut messages, embeds swatches, makes them available for queries

### Domain Model

```mermaid
classDiagram
    class Material {
        +String id
        +String path
        +MaterialState state
        +DateTime last_modified
        +String content
        +List~Swatch~ swatches
    }

    class Swatch {
        +String id
        +String material_id
        +String content
        +Vec~f32~ embedding
        +SwatchMetadata metadata
    }

    class MaterialState {
        <<enumeration>>
        +DISCOVERED
        +CUT
        +SWATCHED
        +ERROR
    }

    Material "1" --> "*" Swatch
    Material --> MaterialState
```

## Technical Decisions

### Actor System: Orchestrator Pattern

- Centralizes actor management for cleaner code organization
- QuiltOrchestrator manages actor creation, messaging, and shutdown

### Actor Framework: Actix

- Provides a robust, production-ready actor system for Rust
- Actors implement appropriate message handling and lifecycle management

### Runtime Integration: Actix and Tokio

- **#[actix::main]** macro initializes the Actix system on top of Tokio
- Leverages Tokio's async primitives for additional functionality

### State Management

- In-memory repository with thread-safe access for concurrent operations
- Plans to add persistent storage in future milestones
