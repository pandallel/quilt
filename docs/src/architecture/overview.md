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

1. **Material Registry** - Central coordinator holding material state and publishing events.
2. **Repository Traits (`MaterialRepository`, `CutsRepository`, `SwatchRepository`)** - Define interfaces for data persistence (material status, cut content, swatch data & embeddings).
3. **Concrete Repositories (`SqliteMaterialRepository`, `SqliteCutsRepository`, `SqliteSwatchRepository`)** - Current implementations of the repository traits using SQLite for persistence. Swatches and their embeddings are stored here.
4. **Discovery Actor** - Monitors input sources for new/updated materials.
5. **Cutting Actor** - Processes materials by cutting them into swatches (structured text fragments).
6. **Swatching Actor** - Generates vector embeddings for swatches using a local model and triggers persistence via `SwatchRepository`.
7. **Event Bus** - Facilitates communication between actors via published events.

## Material Processing Flow

```mermaid
sequenceDiagram
    participant DA as Discovery Actor
    participant CA as Cutting Actor
    participant SA as Swatching Actor
    participant Reg as Material Registry
    participant EB as Event Bus
    participant MatRepo as Material Repository
    participant CutRepo as Cuts Repository
    participant SwatchRepo as Swatch Repository

    DA->>Reg: Register material
    Reg->>MatRepo: Persist material
    Reg->>EB: Publish MaterialDiscovered
    EB-->>CA: Subscribe to MaterialDiscovered
    CA->>CutRepo: Save Cuts
    CA->>Reg: Update status to Cut
    Reg->>MatRepo: Persist updated state
    Reg->>EB: Publish MaterialCut
    EB-->>SA: Subscribe to MaterialCut
    SA->>SwatchRepo: Save Swatches (with embeddings)
    SA->>Reg: Update status to Swatched
    Reg->>MatRepo: Persist updated state
    Reg->>EB: Publish MaterialSwatched

    alt Error occurs
        CA->>Reg: Update status to Error
        Reg->>MatRepo: Persist error state
        Reg->>EB: Publish ErrorEvent
        SA->>Reg: Update status to Error
        Reg->>MatRepo: Persist error state
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
