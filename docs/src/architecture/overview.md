# Architecture Overview

## Project Definition

Quilt is a local-first, modular memory and context engine designed to watch a user's work, fragment documents into meaningful pieces (swatches), embed them into a searchable memory (the swatch book), and assemble contextual spreads in response to queries.

## Core Requirements

1. **Local-first Architecture** - All operations must run on the user's machine without cloud dependencies
2. **Modular Component System** - Independent, swappable components for watching, swatching, embedding, and querying
3. **Privacy-preserving** - No data leakage or external dependencies required
4. **Actor-based Processing Pipeline** - Implement using Rust/Tokio with direct messaging between actors
5. **Material Processing Workflow**:
   - Discovery of new documents/files
   - Cutting into meaningful fragments (swatches)
   - Labeling/embedding of swatches
   - Storage in a queryable repository
   - Assembly of contextual spreads for queries

## System Architecture Overview

Quilt uses an **event-driven actor model architecture** implemented with Actix for actor lifecycle management and Tokio for concurrency primitives. The system processes materials through a pipeline of independent actors that communicate via an Event Bus, with a Material Registry serving as the central coordinator for state and event publishing.

> For detailed implementation specifics of the actor model, see [Actor Model Architecture](actor-model-architecture.md).

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

## Key Technical Components

1. **Material Repository** - Thread-safe data store for materials and their processing state
2. **Discovery Worker** - Monitors input sources for new/updated materials
3. **Cutting Worker** - Processes materials by cutting them into swatches
4. **Labeling Worker** - Executes embedding operations on swatches
5. **Vector-based Storage** - Persists embedded swatches for semantic retrieval

## Material Processing Flow

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

## Domain Model

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
