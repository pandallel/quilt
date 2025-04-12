# Progress

## Project Status

The project is in the **implementation stage**. Milestone 6: "Material Text Cutting Implementation" has been completed. Part 1 of Milestone 7: "Cuts Repository Implementation" has been completed.

## Current Status

- **Architecture:** Defined and documented in `@actor-model-architecture.md`, incorporating event-driven actors, a shared event bus, a central registry, internal backpressure queues for processing actors, and a reconciliation actor for resilience.
- **Foundation:** Core repository, data structures, message channels, and basic actor framework are complete (Milestones 1-4).
- **Discovery:** `DiscoveryActor` implemented and successfully publishing `MaterialDiscovered` events to the shared `EventBus` (Milestone 4).
- **Cutting Actor (M5):** Basic `CuttingActor` skeleton created, subscribing to events, and implementing the internal listener/mpsc/processor pattern (Milestone 5).
- **Cutting Logic (M6):** Completed. Integrated `text-splitter`, implemented processing logic within the `CuttingActor`'s processor task, and updated `MaterialRegistry` to handle state updates (`Discovered` -> `Cut`/`Error`) and publish corresponding events (`MaterialCut`/`ProcessingError`).
- **Cuts Repository (M7 - Part 1):** Completed. Implemented `Cut` data structure and `CutsRepository` interface with an in-memory implementation. The repository supports CRUD operations for cuts, managing cuts by material ID, and has comprehensive test coverage.
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

## In Progress

1. **Cuts Repository Integration (Milestone 7 - Part 2)**:

   - Integrate `CutsRepository` with `CuttingActor`.
   - Update `CuttingActor` to save generated cuts.
   - Enhance the `MaterialCut` event to include cut IDs.
   - Create comprehensive tests for the integration.

2. **Processing Pipeline Enhancement & Performance Optimization**:

   - Implementing backpressure mechanism for large file batches (ongoing tuning).
   - Creating controlled processing rate with work queuing (ongoing tuning).
   - Addressing potential timeout issues in `CuttingActor` for large file batches (requires M7+).
   - Implementing throttling mechanism for event processing (requires M7+).
   - Optimizing memory usage during batch processing (ongoing).
   - Adding monitoring for processing rates and queue depths (ongoing).

## Next Major Milestone

**Milestone 7 (Part 2): "Cuts Repository Integration"** - Focus is on connecting the `CuttingActor` to the new `CutsRepository` implementation.

## Upcoming Work

1. **Cuts Repository Integration** (Milestone 7 - Part 2):

   - Modify `CuttingActor` to save cuts using `CutsRepository`.
   - Update `MaterialCut` event structure to include cut IDs.
   - Add integration tests.

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

1.  **Cuts Repository Integration (M7 Part 2):** Connect the `CuttingActor` to the new `CutsRepository`.
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
