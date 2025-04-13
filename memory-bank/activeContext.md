# Active Context

## Current Focus

**Milestone 8: "Basic Swatching Actor Creation"** is now complete with the implementation of the `SwatchingActor` and its integration into the `QuiltOrchestrator`. We're now shifting focus to **Milestone 9: "Swatching Actor Processes Cuts"**.

## Current Implementation Status

The codebase currently has these key components implemented:

1. **Actor System**:

   - Common actor module with Ping and Shutdown messages
   - DiscoveryActor with lifecycle management
   - CuttingActor with event subscription and processing
   - SwatchingActor with event subscription
   - QuiltOrchestrator implementing the Orchestrator pattern
   - Proper Actix/Tokio runtime integration with #[actix::main]

2. **Material Repository and Registry**:

   - Trait-based repository pattern using `MaterialRepository` trait
   - Thread-safe in-memory store using `Arc<RwLock<HashMap<...>>>` (`InMemoryMaterialRepository`)
   - SQLite-backed implementation (`SqliteMaterialRepository`) using `sqlx` with connection pooling
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
   - SQLite-backed implementation `SqliteCutsRepository` using `sqlx` with connection pooling and foreign key constraints
   - Efficient lookup by material ID in both implementations
   - Comprehensive CRUD operations (save_cut, save_cuts, get_cut_by_id, get_cuts_by_material_id, delete_cut, delete_cuts_by_material_id, count_cuts_by_material_id)
   - Proper error handling with custom `CutsRepositoryError` type
   - Comprehensive test coverage for all repository operations
   - Full integration with CuttingActor for storing processed cuts

8. **Database Infrastructure**:

   - Added SQLite support via `sqlx` with in-memory database capability
   - Created database initialization module in `src/db.rs` with schemas for both materials and cuts tables
   - Implemented table schema creation with proper foreign key constraints
   - Added connection pooling with transaction support
   - Implemented efficient conversion between SQLite rows and domain objects
   - Added comprehensive tests for database operations

9. **Swatching System**:
   - SwatchingActor that subscribes to MaterialCut events
   - Implemented internal listener/mpsc/processor pattern for backpressure
   - Added proper error handling with SwatchingError types
   - Integrated with QuiltOrchestrator
   - Added lifecycle management (start/stop)
   - Added event flow from CuttingActor to SwatchingActor

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
- **Completed Milestone 7.5:**
  - ✅ **Completed Repository Trait Refactoring:** Successfully transformed repositories to use a trait-based pattern, allowing for multiple implementations.
  - ✅ **Implemented SQLite Material Repository:** Implemented `SqliteMaterialRepository` using SQLx with proper SQLite integration.
  - ✅ **Refactored Material Timestamps:** Renamed `ingested_at` to `created_at`, added `updated_at` and `status_updated_at`.
  - ✅ **Implemented SQLite Cuts Repository:** Created `SqliteCutsRepository` with all required operations, proper foreign key constraints, transaction support, and comprehensive tests.
  - ✅ **Fixed Path Resolution Issues:** Ensured that `DiscoveryActor` converts relative paths to absolute paths before registration, fixing path resolution issues.
  - ✅ **Enhanced Application Integration:** Updated `QuiltOrchestrator` to support SQLite for both materials and cuts repositories.
- **Completed Milestone 8:**
  - ✅ **Created SwatchingActor:** Implemented a minimal actor that listens for MaterialCut events via the EventBus.
  - ✅ **Set up Internal Processing Pattern:** Implemented the internal listener/mpsc/processor pattern for backpressure.
  - ✅ **Added Actor Lifecycle Management:** Implemented proper start/stop functionality and resource cleanup.
  - ✅ **Integrated with Orchestrator:** Added SwatchingActor to the QuiltOrchestrator's actor initialization and shutdown sequence.
  - ✅ **Added Event Handling:** Set up the actor to listen for MaterialCut events and log receipt.
  - ✅ **Implemented Error Handling:** Added SwatchingError types for different failure scenarios.
  - ✅ **Added Testing:** Created comprehensive unit tests validating the actor's functionality.
- **Focus:** Shifting to Milestone 9 (Swatching Actor Processes Cuts).
- **Next Steps:**
  - **Implement Swatching Processing Logic (M9):** Create the logic to process cuts and generate embeddings.

## Current Implementation Issues

1. ~~**Path Resolution Issues:**~~

   - ~~When running the application with relative paths (e.g., `--dir=./src`), file processing fails with errors like `Failed to read file 'materials/types.rs': No such file or directory (os error 2)`.~~
   - ~~This is because the paths are interpreted relative to the current working directory rather than the source directory.~~
   - ~~A future enhancement should implement proper path resolution to fix this issue.~~
   - **Resolved:** The issue was fixed by ensuring the `DiscoveryActor` resolves relative paths to absolute paths before registering materials.

2. ~~**Database Transaction Handling:**~~
   - ~~The current SQLite implementation performs individual queries for operations.~~
   - ~~Future work should add proper transaction support for operations that require atomicity.~~
   - **Resolved:** Transaction support has been implemented for batch operations in both repositories.

## Next Steps (Revised Plan)

1. ✅ **Complete SQLite Repositories (M7.5):**

   - ✅ Refactor Material Timestamps (Renamed `ingested_at`, added `updated_at`, `status_updated_at`)
   - ✅ Implement `SqliteCutsRepository` to replace `InMemoryCutsRepository`
   - ✅ Refine SQLite connection management (transaction support, etc.)
   - ✅ Enhance error handling for database operations
   - ✅ Address path resolution issues

2. ✅ **Implement Basic Swatching Actor (M8):**

   - ✅ Create skeleton actor that subscribes to `MaterialCut` events
   - ✅ Implement internal listener/mpsc/processor pattern for backpressure
   - ✅ Add lifecycle management (start/stop)
   - ✅ Setup event flow from cutting to swatching
   - ✅ Add logging for received events
   - ✅ Integrate with QuiltOrchestrator

3. **Implement Swatching Logic (M9):**
   - Design `Swatch` data structure with appropriate metadata
   - Implement logic to retrieve cuts from `CutsRepository`
   - Add embedding generation functionality
   - Create state transition logic (`Cut` → `Swatched` or `Cut` → `Error`)
   - Implement event publishing for completed swatches
   - Add comprehensive error handling

## Active Decisions & Considerations

- **SQLite Implementation Strategy:** **Complete:** Successfully implemented both in-memory SQLite repositories with proper transaction support and error handling.
- **Repository Selection:** **Complete:** Added command-line flag `--in-memory` to allow runtime selection between SQLite and in-memory repositories for both materials and cuts.
- **Path Resolution:** **Resolved:** Fixed path handling by ensuring `DiscoveryActor` resolves relative paths to absolute paths before registering materials.
- **Transaction Support:** **Complete:** Implemented transaction support for batch operations in both SQLite repositories.
- **SwatchingActor Design:** **Complete:** Implemented using the dual-task pattern (listener/processor) for backpressure handling.
- **MaterialCut Event Structure:** The current implementation includes the material ID in the event. The `SwatchingActor` will retrieve the cuts using the material ID via the `CutsRepository`.
- **Swatch Data Model:** Design the `Swatch` data structure and determine how to store embeddings efficiently.
- **Embedding Strategy:** Determine which embedding technique/library to use (e.g., simple TF-IDF, or more advanced embedding model).
- **Batch Processing:** Consider whether to process cuts individually or in batches for embedding generation.
- **Backpressure Tuning:** The default sizes for the internal `mpsc` queues (currently 128 for both `CuttingActor` and `SwatchingActor`) and the `broadcast` Event Bus capacity need to be determined and potentially made configurable.
- **Reconciliation Logic:** Finalize the specific timeouts per stage and the `max_retries` count. These will likely be configurable.
- **Error Handling:** Continue refining error types and handling, particularly around persistence (M13) and potential reconciliation loops.
- **Idempotency:** Ensure processing tasks within actors robustly check material state in the Registry before processing, especially when handling retried events from the `ReconciliationActor`.
- **Vector Search Integration:** Consider `sqlite-vec` integration when implementing advanced search capabilities.
- **Future Cutting Enhancements:** Consider improvements to the cutting functionality:
  - Explicit backpressure handling when the internal queue fills up.
  - Retry mechanisms for recoverable errors with exponential backoff.
  - Making cutting parameters configurable at runtime.
