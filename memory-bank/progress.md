# Progress

## Project Status

The project is in the **implementation stage**. Milestone 6: "Material Text Cutting Implementation" has been completed.

## Current Status

- **Architecture:** Defined and documented in `@actor-model-architecture.md`, incorporating event-driven actors, a shared event bus, a central registry, internal backpressure queues for processing actors, and a reconciliation actor for resilience.
- **Foundation:** Core repository, data structures, message channels, and basic actor framework are complete (Milestones 1-4).
- **Discovery:** `DiscoveryActor` implemented and successfully publishing `MaterialDiscovered` events to the shared `EventBus` (Milestone 4).
- **Cutting Actor (M5):** Basic `CuttingActor` skeleton created, subscribing to events, and implementing the internal listener/mpsc/processor pattern (Milestone 5).
- **Cutting Logic (M6):** Completed. Integrated `text-splitter`, implemented processing logic within the `CuttingActor`'s processor task, and updated `MaterialRegistry` to handle state updates (`Discovered` -> `Cut`/`Error`) and publish corresponding events (`MaterialCut`/`ProcessingError`).
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

## In Progress

1. **Cuts Repository Implementation (Milestone 7)**:

   - Define `Cut` data structure.
   - Create in-memory storage for cuts (`CutsRepository`).
   - Implement CRUD operations.
   - Integrate with Registry/pipeline.
   - Create comprehensive tests.

2. **Processing Pipeline Enhancement & Performance Optimization**:

   - Implementing backpressure mechanism for large file batches (ongoing tuning).
   - Creating controlled processing rate with work queuing (ongoing tuning).
   - Addressing potential timeout issues in `CuttingActor` for large file batches (requires M7+).
   - Implementing throttling mechanism for event processing (requires M7+).
   - Optimizing memory usage during batch processing (ongoing).
   - Adding monitoring for processing rates and queue depths (ongoing).

## Next Major Milestone

**Milestone 7: "Cuts Repository Implementation"** - Focus is on implementing storage for processed cuts.

## Upcoming Work

1. **Cuts Repository Implementation** (Milestone 7):

   - Create in-memory storage for cuts.
   - Implement CRUD operations.
   - Add integration with Registry (potentially `CuttingActor` needs to store Cut IDs).
   - Create comprehensive tests.

2. **Swatching Actor Implementation** (Milestone 8):

   - Create basic Swatching Actor skeleton.
   - Implement event subscription for MaterialCut events.
   - Add actor lifecycle management and internal queue pattern.
   - Set up event flow monitoring.

3. **Performance Improvements** (Milestone 7+):

   - Continue tuning backpressure mechanism in `CuttingActor`.
   - Add rate limiting for processing events.
   - Create circuit breaker for system overload protection.
   - Optimize memory usage during batch processing.

## What's Left to Build (Immediate Milestones)

1.  **Cuts Repository (M7):** Implement storage for cuts and integrate with the pipeline.
2.  **Basic Swatching Actor (M8):** Create skeleton actor, subscribe to `MaterialCut` events, implement internal queue pattern.
3.  **Swatching Logic (M9):** Implement swatch creation within the `SwatchingActor`'s processor task.
4.  **Swatch Repository (M10):** Implement storage for swatches.
5.  **Basic Query (M11):** Simple search capability.
6.  **Reconciliation Actor (M12):** Implement the actor for handling stuck items and retries.
7.  **Persistence (M13):** Implement file-based persistence for events and repositories.

## Known Issues & Blockers

- **Backpressure Tuning:** Internal queue sizes (`mpsc`) and Event Bus capacity need empirical tuning once the pipeline is more complete (M7+).
- **Reconciliation Logic Details:** Specific timeouts and retry counts need finalization.
- **Error Handling:** Needs further refinement, especially around persistence and potential reconciliation loops.
- **`Cut` Data Model:** Precise definition needed for `Cut` data structure for M7.
