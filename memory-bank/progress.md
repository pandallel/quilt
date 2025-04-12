# Progress

## Project Status

The project is in the **implementation stage**, working on Milestone 6: "Material Text Cutting Implementation". The TextCutter component has been successfully implemented using the text-splitter crate, and work is in progress on integrating it fully with the CuttingActor for document processing.

## Current Status

- **Architecture:** Defined and documented in `@actor-model-architecture.md`, incorporating event-driven actors, a shared event bus, a central registry, internal backpressure queues for processing actors, and a reconciliation actor for resilience.
- **Foundation:** Core repository, data structures, message channels (for direct comms, distinct from event bus), and basic actor framework are complete (Milestones 1-4).
- **Discovery:** `DiscoveryActor` implemented and successfully publishing `MaterialDiscovered` events to the shared `EventBus` (Milestone 4).
- **Cutting Actor (M5):** Basic `CuttingActor` skeleton created and subscribing to events (Milestone 5 marked complete), BUT the implementation of the internal listener/mpsc/processor task structure (required by the updated architecture) is **pending**.
- **Cutting Logic (M6):** **In Progress** - Integrating `text-splitter`, planning implementation within the (pending) Processor Task structure, handling backpressure in the (pending) Listener Task.
- **Reconciliation:** Actor design included in architecture, implementation planned (Milestone 12).

## What Works

- Actor system initialization and basic message handling.
- Directory scanning and material discovery.
- Event Bus setup (`broadcast`) and event publishing/subscription for `MaterialDiscovered`.
- Material Registry prototype managing basic state and publishing discovery events.
- Basic `CuttingActor` skeleton (subscribes to events, basic lifecycle).
- **`CuttingActor` now implements internal listener/mpsc/processor pattern for backpressure.**
- `text-splitter` integrated.

## In Progress

1. **Material Cut Processing (Milestone 6)**:

   - Finalizing document cutting implementation **within `CuttingActor` processor task**
   - Completing MaterialCut event creation and publishing
   - Implementing material state transition (Discovered → Cut)
   - Adding error recovery for failed cuts
   - Creating metrics for cut creation and processing

2. **Processing Pipeline Enhancement**:

   - Implementing backpressure mechanism for large file batches
   - Creating controlled processing rate with work queuing
   - Planning the Cuts Repository implementation
   - Preparing for integration with the next pipeline stage

3. **Performance Optimization**:
   - Addressing timeout issues in CuttingActor for large file batches
   - Implementing throttling mechanism for event processing
   - Optimizing memory usage during batch processing
   - Adding monitoring for processing rates and queue depths

## Next Major Milestone

**Milestone 7: "Cuts Repository Implementation"** - After completing Milestone 6, the focus will be on implementing storage for processed cuts.

## Upcoming Work

1. **Complete Cutting Implementation** (Milestone 6):

   - Finish MaterialCut event publishing
   - Complete state transitions in Registry
   - Add validation through logging
   - Implement recovery for failed cuts

2. **Cuts Repository Implementation** (Milestone 7):

   - Create in-memory storage for cuts
   - Implement CRUD operations
   - Add integration with Registry
   - Create comprehensive tests

3. **Performance Improvements** (Milestone 6-7):

   - Implement backpressure mechanism in CuttingActor
   - Add rate limiting for processing events
   - Create circuit breaker for system overload protection
   - Optimize memory usage during batch processing

4. **Swatching Actor Implementation** (Milestone 8):

   - Create basic Swatching Actor skeleton
   - Implement event subscription for MaterialCut events
   - Add actor lifecycle management
   - Set up event flow monitoring

5. **Recent Improvements**:
   - Implemented TextCutter with text-splitter crate integration
   - Added CutterConfig for configurable token sizes
   - Created ChunkInfo model for tracking cut chunks
   - Enhanced CuttingActor with file content extraction
   - Added specialized error types for cutting operations
   - Enabled tokio::fs for asynchronous file operations

## What's Left to Build (Immediate Milestones)

1.  **Complete Cutting Logic (M6):** Finish text extraction, chunking, `MaterialCut` creation, backpressure handling, and event publishing within the `CuttingActor`'s Processor Task.
2.  **Cuts Repository (M7):** Implement storage for cuts and integrate with `CuttingActor`.
3.  **Basic Swatching Actor (M8):** Create skeleton actor, subscribe to `MaterialCut` events, implement internal queue pattern (listener/mpsc/processor).
4.  **Swatching Logic (M9):** Implement swatch creation within the `SwatchingActor`'s processor task.
5.  **Swatch Repository (M10):** Implement storage for swatches.
6.  **Basic Query (M11):** Simple search capability.
7.  **Reconciliation Actor (M12):** Implement the actor for handling stuck items and retries.
8.  **Persistence (M13):** Implement file-based persistence for events and repositories.

## Known Issues & Blockers

- **Backpressure Tuning:** Internal queue sizes and Event Bus capacity need empirical tuning once the pipeline is more complete.
- **Reconciliation Logic Details:** Specific timeouts and retry counts need finalization.
- **Error Handling:** Needs further refinement, especially around persistence and potential reconciliation loops.
