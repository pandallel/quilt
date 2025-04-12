# Active Context

## Current Focus

The project has **completed Milestone 6: "Material Text Cutting Implementation"** and **Part 1 of Milestone 7: "Cuts Repository Implementation"**. The focus is now shifting to **Milestone 7 (Part 2): "Cuts Repository Integration"**.

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
   - **Registry now handles publishing of `MaterialCut` and `ProcessingError` events based on state transitions.**

3. **Event System**:

   - Event Bus implemented using `tokio::sync::broadcast` channels
   - Event types defined for material and system events (`MaterialDiscovered`, `MaterialCut`, `ProcessingError`)
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
   - Event publishing for discovered materials (via Registry)

6. **Cutting System**:

   - CuttingActor that subscribes to MaterialDiscovered events
   - Implemented internal listener/mpsc/processor pattern for backpressure
   - TextCutter implementation using text-splitter crate for document chunking
   - CutterConfig with configurable token size settings (target: 300, min: 150, max: 800)
   - File content extraction using tokio::fs for async file operations
   - Material processing logic: reads file, calls cutter, updates registry (which publishes events).
   - Specialized CutterError types

7. **Cuts Repository**:
   - `Cut` data structure with metadata (id, material_id, content, chunk_index, created_at, token_count, byte offsets)
   - `CutsRepository` trait defining async operations for cuts management
   - Thread-safe in-memory implementation using `Arc<RwLock<HashMap<...>>>`
   - Efficient lookup by material ID using a secondary index
   - Comprehensive test coverage for all repository operations
   - Uses native Rust async traits with explicit future returns and Send bounds

## Recent Changes & Current Focus

- **Completed M6:**
  - Integrated `text-splitter` into `CuttingActor`.
  - Refactored `MaterialRegistry` to publish `MaterialCut` and `ProcessingError` events upon successful state transitions.
  - Simplified `CuttingActor` to focus on processing and calling `registry.update_material_status`.
  - Updated event definitions and tests.
- **Completed M7 (Part 1):**

  - Designed and implemented `Cut` data structure with complete metadata.
  - Created `CutsRepository` trait with comprehensive async operations.
  - Implemented thread-safe in-memory repository pattern with `InMemoryCutsRepository`.
  - Used native Rust async traits with explicit future returns for better type safety.
  - Added comprehensive test coverage for all repository operations.

- **Current Focus:**
  1.  **Starting M7 (Part 2):** Integrate the `CuttingActor` with the new `CutsRepository`.

## Next Steps

1.  **Implement Milestone 7 (Part 2):** Modify the `CuttingActor` to save cuts using the `CutsRepository` and update the `MaterialCut` event to include cut IDs.
2.  **Implement Milestone 8:** Create the basic `SwatchingActor` with its internal queue pattern.
3.  **Implement Milestone 12 (Reconciliation):** Begin work on the `ReconciliationActor`.

## Active Decisions & Considerations

- **Integrating Cuts Repository with CuttingActor:** Determine where and how to inject the `CutsRepository` into the `CuttingActor`. Consider using dependency injection patterns similar to the existing code.
- **MaterialCut Event Structure Update:** Decide whether to include all cut IDs in the event or just a reference to the material that now has cuts.
- **Backpressure Tuning:** The default sizes for the internal `mpsc` queues (currently 32 for `CuttingActor`) and the `broadcast` Event Bus capacity need to be determined and potentially made configurable (likely after M8/M9).
- **Reconciliation Logic:** Finalize the specific timeouts per stage and the `max_retries` count. These will likely be configurable.
- **Error Handling:** Continue refining error types and handling, particularly around persistence (M13) and potential reconciliation loops.
- **Idempotency:** Ensure processing tasks within actors robustly check material state in the Registry before processing, especially when handling retried events from the `ReconciliationActor`.
