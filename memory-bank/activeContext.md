# Active Context

## Current Focus

The project has completed Milestone 5: "Basic Cutting Actor Creation" and is now implementing Milestone 6: "Material Text Cutting Implementation". The Cutting Actor has been successfully integrated with the event-driven architecture and the initial text-splitting functionality has been implemented using the text-splitter crate.

## Current Implementation Status

The codebase currently has these key components implemented:

1. **Actor System**:

   - Common actor module with Ping and Shutdown messages
   - DiscoveryActor with lifecycle management
   - CuttingActor with event subscription and basic processing
   - QuiltOrchestrator implementing the Orchestrator pattern
   - Proper Actix/Tokio runtime integration with #[actix::main]

2. **Material Repository and Registry**:

   - Thread-safe in-memory store using `Arc<RwLock<HashMap<...>>>`
   - Material state tracking with proper validation (Discovered → Cut → Swatched → Error)
   - CRUD operations with idempotence and state transition validation
   - Registry wrapping repository and providing event coordination
   - Fully transition from direct Repository use to Registry pattern

3. **Event System**:

   - Event Bus implemented using `tokio::sync::broadcast` channels
   - Material Registry coordinating state management and event publishing
   - Event types defined for material and system events
   - ProcessingError events for handling error cases
   - Comprehensive test coverage for event publishing and subscription
   - Clear error handling for event operations
   - Improved logging with appropriate debug level for routine events

4. **Message System**:

   - Actor-specific message types for clear communication contracts
   - Typed message response handling with proper error types
   - Leveraging Actix's built-in mailbox and message handling
   - Direct actor-to-actor communication pattern

5. **Discovery System**:

   - DirectoryScanner that finds files in configured directories
   - DiscoveryActor that wraps the scanner in the actor interface
   - DiscoveryConfig for scanner parameters
   - Support for excluding patterns and hidden files
   - Event publishing for discovered materials

6. **Cutting System**:
   - CuttingActor that subscribes to MaterialDiscovered events
   - **Implemented internal listener/mpsc/processor pattern for backpressure**
   - TextCutter implementation using text-splitter crate for document chunking
   - CutterConfig with configurable token size settings (target: 300, min: 150, max: 800)
   - File content extraction using tokio::fs for async file operations
   - Material processing logic with proper error handling
   - Chunk information model with material tracking (ChunkInfo struct)
   - Error handling with specialized CutterError types

## Recent Changes & Current Focus

- Refined the `@actor-model-architecture.md` to incorporate details about internal actor backpressure queues (`mpsc`) between listener and processor tasks, and the role of the `ReconciliationActor`. Merged implementation/scaling details.
- Updated the `implementation-plan.md` accordingly, adding Milestone 12 for Reconciliation.
- Identified that while Milestone 5 (Basic Cutting Actor) was marked complete, the newly required internal queue/task structure is **pending implementation** within that actor. **<-- This is now COMPLETE.**
- **Refactored `CuttingActor` to implement the internal listener/mpsc/processor pattern.**
- Currently focusing on:
  1.  **Starting M6:** Integrating the `text-splitter` logic _within_ the new Processor Task structure and handling `MaterialCut` event creation/publishing and backpressure.

## Next Steps

1.  **Complete Milestone 6:** Finish implementing the cutting logic within the `CuttingActor`'s Processor Task, including `MaterialCut` creation, registry updates, event publishing, and backpressure handling.
2.  **Implement Milestone 7:** Build the `CutsRepository`.
3.  **Implement Milestone 8:** Create the basic `SwatchingActor` with its internal queue pattern.
4.  **Implement Milestone 12 (Reconciliation):** Begin work on the `ReconciliationActor`.

## Active Decisions & Considerations

- **Backpressure Tuning:** The default sizes for the internal `mpsc` queues (currently 32 for `CuttingActor`) and the `broadcast` Event Bus capacity need to be determined and potentially made configurable.
- **Reconciliation Logic:** Finalize the specific timeouts per stage and the `max_retries` count. These will likely be configurable.
- **Error Handling:** Continue refining error types and handling, particularly around `RecvError::Lagged` in listeners and potential failures during reconciliation.
- **Idempotency:** Ensure processing tasks within actors robustly check material state in the Registry before processing, especially when handling retried events from the `ReconciliationActor`.
