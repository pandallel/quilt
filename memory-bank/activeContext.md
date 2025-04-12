# Active Context

## Current Focus

**Milestone 7.5: "SQLite Repository Implementation"** is now partially complete with the implementation of `SqliteMaterialRepository`. We've completed the initial work on the SQLite-backed Material Repository, but have deferred the `SqliteCutsRepository` implementation to keep the changes focused.

## Current Implementation Status

The codebase currently has these key components implemented:

1. **Actor System**:

   - Common actor module with Ping and Shutdown messages
   - DiscoveryActor with lifecycle management
   - CuttingActor with event subscription and processing
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
   - Efficient lookup by material ID using a secondary index for material_id → [cut_ids]
   - Comprehensive CRUD operations (save_cut, save_cuts, get_cut_by_id, get_cuts_by_material_id, delete_cut, delete_cuts_by_material_id, count_cuts_by_material_id)
   - Proper error handling with custom `CutsRepositoryError` type
   - Comprehensive test coverage for all repository operations
   - Full integration with CuttingActor for storing processed cuts

8. **Database Infrastructure**:
   - Added SQLite support via `sqlx` with in-memory database capability
   - Created database initialization module in `src/db.rs`
   - Implemented table schema creation and connection pooling
   - Added conversion between SQLite rows and domain objects

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
- **Milestone 7.5 Progress:**
  - ✅ **Completed Step 1 - Renaming `MaterialRepository`:** Successfully renamed the struct `MaterialRepository` to `InMemoryMaterialRepository` throughout the codebase and verified all tests pass.
  - ✅ **Completed Step 2 - Introduce `MaterialRepository` Trait:** Added `async-trait`, defined `trait MaterialRepository` in `materials/mod.rs`, implemented it for `InMemoryMaterialRepository`, and validated with tests.
  - ✅ **Completed Step 3 - Refactor `MaterialRegistry`:** Updated `MaterialRegistry` to use `Arc<dyn MaterialRepository>`.
  - ✅ **Completed Step 4 - Update Actor Dependencies:** Updated `Orchestrator`, `CuttingActor`, `DiscoveryActor` initialization and tests.
  - ✅ **Completed Step 5 - Move Trait Definitions:** Moved trait/error definitions to `mod.rs` for better organization.
  - ✅ **Completed Step 6 - Apply Pattern to CutsRepository:** Applied the same repository trait pattern to the `CutsRepository`.
  - ✅ **Completed Step 7 (Partial) - SQLite Material Repository:**
    - Added `sqlx` with SQLite features to Cargo.toml
    - Created database utility module with connection pooling
    - Implemented `SqliteMaterialRepository` with all required methods
    - Added comprehensive tests for the SQLite implementation
    - Integrated with the application by updating `QuiltOrchestrator` and CLI options
    - ⏩ **Deferred:** `SqliteCutsRepository` implementation left for future update
- **Focus:** Finishing Milestone 7.5 (SQLite Repository) and debugging file path issues.
- **Recent Changes:**
  - Completed trait-based repository refactoring for `MaterialRepository` and `CutsRepository`.
  - Implemented `SqliteMaterialRepository` using SQLx and integrated it.
  - Added `--in-memory` flag for repository selection.
  - Fixed the file path resolution issue: `DiscoveryActor` now converts relative paths to absolute paths before registration, ensuring `CuttingActor` receives correct paths.
- **Next Steps:**
  - **Refactor Material Timestamps (Immediate Next Task):** Rename `ingested_at` to `created_at`, add `status_updated_at` and `updated_at`. Leverage `sqlx` time feature for simplification.
  - Enhance transaction support and error handling in SQLite repositories.
  - Implement `SqliteCutsRepository`.
  - Begin work on Milestone 8 (Basic Swatching Actor).
- **Decisions/Considerations:**
  - Sticking with `sqlx` for SQLite interaction.
  - Confirmed the absolute path resolution approach is the correct fix for the file reading errors.

## Current Implementation Issues

1. **Path Resolution Issues:**

   - When running the application with relative paths (e.g., `--dir=./src`), file processing fails with errors like `Failed to read file 'materials/types.rs': No such file or directory (os error 2)`.
   - This is because the paths are interpreted relative to the current working directory rather than the source directory.
   - A future enhancement should implement proper path resolution to fix this issue.

2. **Database Transaction Handling:**
   - The current SQLite implementation performs individual queries for operations.
   - Future work should add proper transaction support for operations that require atomicity.

## Next Steps (Revised Plan)

1. **Complete SQLite Repositories (M7.5):**

   - Implement `SqliteCutsRepository` to replace `InMemoryCutsRepository`
   - Refine SQLite connection management (transaction support, etc.)
   - Enhance error handling for database operations
   - Address path resolution issues

2. **Implement Basic Swatching Actor (M8):**
   - **Preceded by:** Material Timestamp Refactoring Task
   - Create skeleton actor that subscribes to `MaterialCut` events
   - Implement internal listener/mpsc/processor pattern for backpressure
   - Add lifecycle management (start/stop)
   - Subscription mechanism for updates

## Active Decisions & Considerations

- **SQLite Implementation Strategy:** **Decision:** Using in-memory SQLite (`:memory:`) initially for simplified testing and development, deferring file-based persistence and migrations.
- **Migration Strategy:** Deferred until file-based persistence is implemented.
- **Connection Management:** Successfully implemented connection pooling using `sqlx::SqlitePool` for the in-memory database.
- **Repository Selection:** Added command-line flag `--in-memory` to allow runtime selection between SQLite and in-memory repositories.
- **Vector Search Integration:** **Deferred:** `sqlite-vec` integration will be addressed later when file-based persistence is added or vector search is explicitly needed.
- **Future Cutting Enhancements:** Consider improvements to the cutting functionality:
  - Explicit backpressure handling when the internal queue fills up.
  - Retry mechanisms for recoverable errors with exponential backoff.
  - Making cutting parameters configurable at runtime.
- **Path Resolution:** **New Issue:** Need to improve path handling in file processing to properly resolve paths relative to the scanned directory.
- **SwatchingActor Design:** Determine the optimal structure for the `SwatchingActor` based on lessons learned from `CuttingActor`.
- **MaterialCut Event Structure Update:** Decide whether to include all cut IDs in the event or just a reference to the material that now has cuts.
- **Backpressure Tuning:** The default sizes for the internal `mpsc` queues (currently 128 for `CuttingActor`) and the `broadcast` Event Bus capacity need to be determined and potentially made configurable.
- **Reconciliation Logic:** Finalize the specific timeouts per stage and the `max_retries` count. These will likely be configurable.
- **Error Handling:** Continue refining error types and handling, particularly around persistence (M13) and potential reconciliation loops.
- **Idempotency:** Ensure processing tasks within actors robustly check material state in the Registry before processing, especially when handling retried events from the `ReconciliationActor`.
