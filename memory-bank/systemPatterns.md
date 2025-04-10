# System Patterns

## Architecture Overview

Quilt uses an **event-driven actor model architecture** implemented with Actix for actor lifecycle management and Tokio for concurrency primitives. The system processes materials through a pipeline of independent actors that communicate via an Event Bus, with a Material Registry serving as the central coordinator for state and event publishing.

```mermaid
graph TB
    subgraph EventBus["Event Bus"]
        E1[MaterialDiscovered]
        E2[MaterialCut]
        E3[MaterialSwatched]
    end

    subgraph ProcessingActors["Processing Actors"]
        DW[Discovery Actor]
        CW[Cutting Actor]
        SW[Swatching Actor]
    end

    Reg[Material Registry]
    DB[(Material Repository)]

    %% Discovery flow
    DW -->|"Register"| Reg
    Reg -->|"Publish"| E1
    E1 -->|"Subscribe"| CW

    %% Cutting flow
    CW -->|"Update Status"| Reg
    Reg -->|"Publish"| E2
    E2 -->|"Subscribe"| SW

    %% Swatching flow
    SW -->|"Update Status"| Reg
    Reg -->|"Publish"| E3

    %% Persistence flow
    Reg -.->|"Persist"| DB
```

## Key Design Patterns

### Event-Driven Architecture

- **Event Bus**: System-wide bus implemented with Tokio broadcast channels
- **Event Publishing**: Registry publishes events on state changes
- **Event Subscription**: Actors subscribe to events they need to process
- **Decoupling**: No direct actor-to-actor coupling, enabling independent scaling

### State Management and Registry Pattern

- **Material Registry**: Central coordinator for material state
- **State Changes**: Atomic operations with event publishing
- **Separation of Concerns**: Registry manages state, Repository handles persistence
- **Single Source of Truth**: Registry provides consistent view of system state

### Actor Model Implementation

- **Actix for Lifecycle**: Using Actix for actor management
- **Tokio for Concurrency**: Using Tokio primitives for synchronization
- **Event Subscription**: Actors subscribe to relevant event channels
- **Backpressure Handling**: Controlling event consumption rate

### Material Processing Flow

```mermaid
sequenceDiagram
    participant DA as Discovery Actor
    participant CA as Cutting Actor
    participant SA as Swatching Actor
    participant Reg as Material Registry
    participant EB as Event Bus
    participant Repo as Material Repository

    DA->>Reg: Register material
    Reg->>Repo: Persist material
    Reg->>EB: Publish MaterialDiscovered
    EB-->>CA: Subscribe to MaterialDiscovered
    CA->>Reg: Update status to Cut
    Reg->>Repo: Persist updated state
    Reg->>EB: Publish MaterialCut
    EB-->>SA: Subscribe to MaterialCut
    SA->>Reg: Update status to Swatched
    Reg->>Repo: Persist updated state
    Reg->>EB: Publish MaterialSwatched

    alt Error occurs
        CA->>Reg: Update status to Error
        Reg->>Repo: Persist error state
        Reg->>EB: Publish ErrorEvent
    end
```

### Domain Model

```mermaid
classDiagram
    class Material {
        +String id
        +String file_path
        +MaterialStatus status
        +DateTime last_modified
        +Option~String~ error
    }

    class Cut {
        +String id
        +String material_id
        +String content
        +CutMetadata metadata
    }

    class Swatch {
        +String id
        +String material_id
        +Vec~Cut~ cuts
        +Vec~f32~ embedding
        +SwatchMetadata metadata
    }

    class MaterialStatus {
        <<enumeration>>
        +DISCOVERED
        +CUT
        +SWATCHED
        +ERROR
    }

    class MaterialEvent {
        <<enumeration>>
        +Discovered
        +Cut
        +Swatched
    }

    class SystemEvent {
        <<enumeration>>
        +HealthCheck
        +Shutdown
    }

    class ErrorEvent {
        <<enumeration>>
        +ProcessingFailed
        +PersistenceFailed
    }

    Material --> MaterialStatus
    Cut --> Material
    Swatch --> Material
```

## Technical Decisions

### Event Bus Implementation

- **Tokio Broadcast Channels**: Using `tokio::sync::broadcast` for event distribution with a default capacity of 128
- **Multiple Consumers**: Supporting multiple subscribers with independent receivers
- **Event Types**: Implemented MaterialDiscovered and System events (Shutdown, HealthCheck)
- **Error Handling**: Comprehensive error handling with EventBusError and detailed logging
- **Subscription Management**: Simple subscribe() method returning a Receiver
- **Event Publishing**: Publish method with receiver count logging and error propagation
- **EventSubscriber Trait**: Helper trait for processing events from a receiver
- **Comprehensive Tests**: Test coverage for publishing, multiple subscribers, and event processing

### Actor Implementation

- **Actix Actors**: Implementing actors using the Actix framework
- **Event Processing**: Subscribing to events during actor initialization
- **Async Processing**: Using `ResponseFuture` for long-running operations
- **Supervision**: Implementing proper error handling and recovery strategies

### State Management

- **Thread-Safe Registry**: Using Tokio's RwLock for thread-safe state access
- **Atomic Operations**: Ensuring state changes and event publishing are atomic
- **Persistence Layer**: Separate repository for persistent storage

### Concurrency Safety

- **Synchronization Primitives**: Using appropriate primitives for shared state
- **Message Passing**: Relying on message passing for inter-actor communication
- **Ownership Model**: Leveraging Rust's ownership system for safety

## Implementation Approach

The system is being implemented incrementally, with each milestone providing a tangible, demonstrable change:

1. First establishing the Event Bus and Material Registry
2. Updating Discovery Actor to publish events
3. Creating processing actors that subscribe to events
4. Implementing the full pipeline with proper event flow
5. Adding query capabilities and persistence
