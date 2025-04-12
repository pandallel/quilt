# Active Context

## Current Focus

Refactoring the `MaterialRepository` as a prerequisite for **Milestone 7.5: "SQLite Repository Implementation"**. This involves renaming the current in-memory implementation and preparing for the introduction of a trait.

## Current Implementation Status

The codebase currently has these key components implemented:

1. **Actor System**:

   - Common actor module with Ping and Shutdown messages
   - DiscoveryActor with lifecycle management
   - CuttingActor with event subscription and processing
   - QuiltOrchestrator implementing the Orchestrator pattern
   - Proper Actix/Tokio runtime integration with #[actix::main]

2. **Material Repository and Registry**:

   - Thread-safe in-memory store using `Arc<RwLock<HashMap<...>>>`
   - Material state tracking with proper validation (Discovered → Cut → Swatched → Error)
   - CRUD operations with idempotence and state transition validation
   - Registry wrapping repository and providing event coordination
   - Registry handles publishing of `MaterialCut` and `ProcessingError` events based on state transitions

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
   - Material processing logic: reads file, calls cutter, updates registry, stores cuts in repository
   - Specialized CutterError types with proper error propagation

7. **Cuts Repository**:
   - `Cut` data structure with comprehensive metadata (id, material_id, content, chunk_index, created_at, token_count, byte offsets)
   - `CutsRepository` trait defining async operations for cuts management
   - Thread-safe in-memory implementation `InMemoryCutsRepository` using `Arc<RwLock<HashMap<...>>>`
   - Efficient lookup by material ID using a secondary index for material_id → [cut_ids]
   - Comprehensive CRUD operations (save_cut, save_cuts, get_cut_by_id, get_cuts_by_material_id, delete_cut, delete_cuts_by_material_id, count_cuts_by_material_id)
   - Proper error handling with custom `CutsRepositoryError` type
   - Comprehensive test coverage for all repository operations
   - Full integration with CuttingActor for storing processed cuts

## Recent Changes & Current Focus

- **Completed M6:**
  - Integrated `text-splitter` into `CuttingActor`.
  - Refactored `MaterialRegistry` to publish `MaterialCut` and `ProcessingError` events upon successful state transitions.
  - Simplified `CuttingActor` to focus on processing and calling `registry.update_material_status`.
  - Updated event definitions and tests.
- **Completed M7:**

  - Designed and implemented `Cut` data structure with comprehensive metadata.
  - Created `CutsRepository` trait with fully-featured async operations.
  - Implemented thread-safe in-memory repository with `InMemoryCutsRepository`.
  - Used native Rust async traits with explicit future returns for better type safety.
  - Added comprehensive test coverage for all repository operations.
  - Fully integrated `CutsRepository` with `CuttingActor` to store processed cuts.
  - Enhanced error handling in the cutting pipeline with proper error propagation.
  - Connected the full processing chain from discovery through cutting to storage.

- **Current Focus (M7.5 Refactoring):**
  1.  **Renaming `MaterialRepository`:** Renaming the struct `MaterialRepository` to `InMemoryMaterialRepository` throughout the codebase.
  2.  **Validation:** Need to run `cargo check` and `cargo test` to ensure the rename is complete and correct before proceeding.

## Next Steps (Revised Plan)

1.  **Complete `MaterialRepository` Rename & Validation:**
    - Finish renaming `struct MaterialRepository` to `struct InMemoryMaterialRepository`.
    - Update all external usages.
    - **Validate:** Ensure `cargo check` and `cargo test` pass.
2.  **Introduce `MaterialRepository` Trait & Validate:**
    - Add `async-trait` dependency.
    - Define `trait MaterialRepository` in `repository.rs` using `#[async_trait]`.
    - Implement the trait for `InMemoryMaterialRepository`.
    - **Validate:** Ensure `cargo check` and `cargo test` pass.
3.  **Refactor `MaterialRegistry` & Validate:**
    - Update `MaterialRegistry` to use `Arc<dyn MaterialRepository>`.
    - Update dependent code and tests.
    - **Validate:** Ensure `cargo check` and `cargo test` pass.
4.  **Refactor Dependent Actors/Tests & Validate:**
    - Update `Orchestrator`, `CuttingActor`, `DiscoveryActor` initialization and tests.
    - **Validate:** Ensure `cargo check` and `cargo test` pass.
5.  **(Optional Cleanup) Move Trait & Validate:**
    - Move trait/error definitions to `mod.rs`.
    - **Validate:** Ensure `cargo check` and `cargo test` pass.
6.  **Implement SQLite Repositories (M7.5 - In-Memory):**
    - Implement `SqliteMaterialRepository` and `SqliteCutsRepository` (using `:memory:`).
    - Integrate and test.

## Active Decisions & Considerations

- **SQLite Implementation Strategy:** **Decision:** Using in-memory SQLite (`:memory:`) initially for simplified testing and development, deferring file-based persistence and migrations.
- **Migration Strategy:** Deferred until file-based persistence is implemented.
- **Connection Management:** Design connection pooling (`sqlx::SqlitePool`) for the in-memory database.
- **Vector Search Integration:** **Deferred:** `sqlite-vec` integration will be addressed later when file-based persistence is added or vector search is explicitly needed.
- **Future Cutting Enhancements:** Consider improvements to the cutting functionality:
  - Explicit backpressure handling when the internal queue fills up.
  - Retry mechanisms for recoverable errors with exponential backoff.
  - Making cutting parameters configurable at runtime.
- **SwatchingActor Design:** Determine the optimal structure for the `SwatchingActor` based on lessons learned from `CuttingActor`.
- **MaterialCut Event Structure Update:** Decide whether to include all cut IDs in the event or just a reference to the material that now has cuts.
- **Backpressure Tuning:** The default sizes for the internal `mpsc` queues (currently 128 for `CuttingActor`) and the `broadcast` Event Bus capacity need to be determined and potentially made configurable.
- **Reconciliation Logic:** Finalize the specific timeouts per stage and the `max_retries` count. These will likely be configurable.
- **Error Handling:** Continue refining error types and handling, particularly around persistence (M13) and potential reconciliation loops.
- **Idempotency:** Ensure processing tasks within actors robustly check material state in the Registry before processing, especially when handling retried events from the `ReconciliationActor`.
