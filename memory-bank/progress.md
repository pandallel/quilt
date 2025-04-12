# Progress

## Project Status

The project is in the **implementation stage**. Milestone 6: "Material Text Cutting Implementation" and Milestone 7: "Cuts Repository Implementation" have been completed. The project is now moving to Milestone 7.5: "SQLite Repository Implementation" before proceeding to Milestone 8.

## Current Status

- **Architecture:** Defined and documented in `@actor-model-architecture.md`, incorporating event-driven actors, a shared event bus, a central registry, internal backpressure queues for processing actors, and a reconciliation actor for resilience.
- **Foundation:** Core repository, data structures, message channels, and basic actor framework are complete (Milestones 1-4).
- **Discovery:** `DiscoveryActor` implemented and successfully publishing `MaterialDiscovered` events to the shared `EventBus` (Milestone 4).
- **Cutting Actor (M5):** `CuttingActor` skeleton created, subscribing to events, and implementing the internal listener/mpsc/processor pattern (Milestone 5).
- **Cutting Logic (M6):** Completed. Integrated `text-splitter`, implemented processing logic within the `CuttingActor`'s processor task, and updated `MaterialRegistry` to handle state updates (`Discovered` -> `Cut`/`Error`) and publish corresponding events (`MaterialCut`/`ProcessingError`).
- **Cuts Repository (M7):** Completed. Implemented the full `Cut` data structure, `CutsRepository` interface with thread-safe in-memory implementation, and integration with the `CuttingActor`. The repository provides comprehensive CRUD operations, efficient indexing, and is fully connected to the processing pipeline.
- **Reconciliation:** Actor design included in architecture, implementation planned (Milestone 12).

## What Works

- Actor system initialization and basic message handling.
- Directory scanning and material discovery via `DiscoveryActor`.
- Event Bus setup (`broadcast`) and event publishing/subscription.
- Material Repository with CRUD and state management.
- Material Registry coordinating state management and event publishing (`MaterialDiscovered`, `MaterialCut`, `ProcessingError`).
- `CuttingActor` with internal listener/mpsc/processor pattern for backpressure.
- Text cutting using `text-splitter` within `CuttingActor`.
- Material status transition `Discovered` -> `Cut` or `Discovered` -> `Error` handled by `CuttingActor` via `MaterialRegistry`.
- Event publication (`MaterialCut`, `ProcessingError`) handled by `MaterialRegistry`.
- `Cut` data structure with complete metadata (id, material_id, chunk_index, content, token_count, byte offsets).
- `CutsRepository` with in-memory implementation for storing, retrieving, and managing cuts.
- Full integration of the cutting pipeline from discovery to storage (Material discovery → Cutting → Repository storage).
- Comprehensive error handling throughout the cutting pipeline.

## In Progress

1. **SQLite Repository Implementation (Milestone 7.5)**:

   - Designing SQLite-backed repository implementations to replace in-memory versions
   - Planning schema and migration strategy
   - Researching SQLite-vec integration for future vector search capability
   - Preparing connection management and lifecycle handling

2. **Processing Pipeline Enhancement & Performance Optimization**:
   - Implementing backpressure mechanism for large file batches (ongoing tuning).
   - Creating controlled processing rate with work queuing (ongoing tuning).
   - Addressing potential timeout issues in `CuttingActor` for large file batches.
   - Implementing throttling mechanism for event processing.
   - Optimizing memory usage during batch processing (ongoing).
   - Adding monitoring for processing rates and queue depths (ongoing).

## Next Major Milestone

**Milestone 7.5: "SQLite Repository Implementation"** - Focus is on creating SQLite-backed implementations of both repositories to enable persistence and vector search capabilities.

## Upcoming Work

1. **SQLite Repository Implementation** (Milestone 7.5):

   - Set up SQLite infrastructure with connection management
   - Implement schema definition and migration framework
   - Create SQLite-backed MaterialRepository implementation
   - Create SQLite-backed CutsRepository implementation
   - Develop repository factory pattern for runtime selection
   - Add configuration for connection parameters and operational modes
   - Set up SQLite-vec extension for future vector operations

2. **Swatching Actor Implementation** (Milestone 8):

   - Create basic Swatching Actor skeleton.
   - Implement event subscription for MaterialCut events.
   - Add actor lifecycle management and internal queue pattern.
   - Set up event flow monitoring.

3. **Performance Improvements** (Ongoing):
   - Continue tuning backpressure mechanism in actors.
   - Add rate limiting for processing events.
   - Create circuit breaker for system overload protection.
   - Optimize memory usage during batch processing.

## What's Left to Build (Immediate Milestones)

1. **SQLite Repository Infrastructure (M7.5):** Implement SQLite-backed versions of both repositories with vector search capability.
2. **Basic Swatching Actor (M8):** Create skeleton actor, subscribe to `MaterialCut` events, implement internal queue pattern.
3. **Swatching Logic (M9):** Implement swatch creation within the `SwatchingActor`'s processor task.
4. **Swatch Repository (M10):** Implement storage for swatches.
5. **Basic Query (M11):** Simple search capability.
6. **Reconciliation Actor (M12):** Implement the actor for handling stuck items and retries.
7. **Persistence (M13):** Implement file-based persistence for events and repositories.

## Future Enhancements (Post-Core Implementation)

Based on the recent code review, these enhancements have been identified for future development:

1. **Cutting Enhancements:**

   - Explicit backpressure handling when internal queue fills up
   - Retry mechanisms for recoverable errors
   - Configurable cutting parameters (chunk size, overlap)

2. **Storage Improvements:**

   - Disk-based repository options for cuts and materials
   - Streaming implementation for very large files
   - Efficient indexing strategies for large repositories

3. **Logging and Observability:**
   - Structured logging with span contexts for better traceability
   - Comprehensive tracing for request flows
   - Detailed performance metrics collection

## Known Issues & Blockers

- **Backpressure Tuning:** Internal queue sizes (`mpsc`, currently 128 for `CuttingActor`) and Event Bus capacity need empirical tuning once the pipeline is more complete.
- **Reconciliation Logic Details:** Specific timeouts and retry counts need finalization.
- **Error Handling:** Continues to be refined, especially around persistence and potential reconciliation loops.
