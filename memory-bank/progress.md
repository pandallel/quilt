# Progress

## Project Status

The project is in the **implementation stage**. Milestone 6: "Material Text Cutting Implementation" and Milestone 7: "Cuts Repository Implementation" have been completed. Milestone 7.5: "SQLite Repository Implementation" is partially completed, with successful implementation of trait-based repositories and a SQLite-backed Material Repository.

## Current Status

- **Architecture:** Defined and documented in `@actor-model-architecture.md`, incorporating event-driven actors, a shared event bus, a central registry, internal backpressure queues for processing actors, and a reconciliation actor for resilience.
- **Foundation:** Core repository, data structures, message channels, and basic actor framework are complete (Milestones 1-4).
- **Discovery:** `DiscoveryActor` implemented and successfully publishing `MaterialDiscovered` events to the shared `EventBus` (Milestone 4).
- **Cutting Actor (M5):** `CuttingActor` skeleton created, subscribing to events, and implementing the internal listener/mpsc/processor pattern (Milestone 5).
- **Cutting Logic (M6):** Completed. Integrated `text-splitter`, implemented processing logic within the `CuttingActor`'s processor task, and updated `MaterialRegistry` to handle state updates (`Discovered` -> `Cut`/`Error`) and publish corresponding events (`MaterialCut`/`ProcessingError`).
- **Cuts Repository (M7):** Completed. Implemented the full `Cut` data structure, `CutsRepository` interface with thread-safe in-memory implementation, and integration with the `CuttingActor`. The repository provides comprehensive CRUD operations, efficient indexing, and is fully connected to the processing pipeline.
- **Repository Refactoring (M7.5):** Completed. Transformed repositories to use the trait-based pattern. Successfully renamed `struct MaterialRepository` to `struct InMemoryMaterialRepository`, introduced `trait MaterialRepository` and `trait CutsRepository`, and updated all dependent code.
- **SQLite Repository (M7.5 - Partial):** Implemented `SqliteMaterialRepository` using SQLx, with in-memory database support. Added command-line option to select between SQLite and in-memory repositories. The `SqliteCutsRepository` implementation was deferred to the next phase.
- **Reconciliation:** Actor design included in architecture, implementation planned (Milestone 12).

## What Works

- Actor system initialization and basic message handling.
- Directory scanning and material discovery via `DiscoveryActor`.
- Event Bus setup (`broadcast`) and event publishing/subscription.
- Material Repository trait support with two implementations:
  - `InMemoryMaterialRepository` using `Arc<RwLock<HashMap<...>>>` for in-memory storage
  - `SqliteMaterialRepository` using `sqlx` with SQLite for persistent storage
- Material Registry coordinating state management and event publishing (`MaterialDiscovered`, `MaterialCut`, `ProcessingError`).
- `CuttingActor` with internal listener/mpsc/processor pattern for backpressure.
- Text cutting using `text-splitter` within `CuttingActor`.
- Material status transition `Discovered` -> `Cut` or `Discovered` -> `Error` handled by `CuttingActor` via `MaterialRegistry`.
- Event publication (`MaterialCut`, `ProcessingError`) handled by `MaterialRegistry`.
- `Cut` data structure with complete metadata (id, material_id, chunk_index, content, token_count, byte offsets).
- `CutsRepository` with in-memory implementation for storing, retrieving, and managing cuts.
- Full integration of the cutting pipeline from discovery to storage (Material discovery → Cutting → Repository storage).
- Comprehensive error handling throughout the cutting pipeline.
- Repository trait pattern implementation allowing easy swapping between storage backends.
- Command-line option (`--in-memory`) for selecting repository type.
- Event bus reliably transmits events (`MaterialDiscovered`, `MaterialCut`, `ProcessingError`).
- `CuttingActor` subscribes to `MaterialDiscovered`, processes files using `TextCutter`, and stores cuts in the `InMemoryCutsRepository`.
- `CuttingActor` correctly reads file content using absolute paths resolved by the `DiscoveryActor`.
- `MaterialRegistry` handles state transitions (`Discovered` -> `Cut`, `Discovered` -> `Error`) and publishes corresponding events.
- SQLite integration works for the `MaterialRepository`, selectable via command-line flag.

## In Progress

1. **Milestone 7.5 SQLite Implementation:**
   - ✅ Completed Repository Trait Pattern:
     - ✅ Renamed `MaterialRepository` to `struct InMemoryMaterialRepository`
     - ✅ Introduced `MaterialRepository` trait using `async-trait`
     - ✅ Implemented the trait for `InMemoryMaterialRepository` and updated the Registry
     - ✅ Updated Actor Dependencies to use the trait
     - ✅ Moved trait definitions to `mod.rs` for better organization
     - ✅ Applied the same pattern to the `CutsRepository`
   - ✅ Implemented SQLite Material Repository:
     - ✅ Added `sqlx` with SQLite features to the project
     - ✅ Created database connection and schema management module
     - ✅ Implemented `SqliteMaterialRepository` with comprehensive tests
     - ✅ Integrated SQLite repository with the application via the orchestrator
     - ✅ Added command-line flag for repository selection
   - ⚠️ Identified Path Resolution Issue:
     - File paths are currently interpreted relative to the working directory, causing errors
     - Future enhancement needed to resolve paths relative to the scanned directory
   - **Next:**
     - Implement `SqliteCutsRepository` to complete the SQLite storage backend
     - Enhance transaction support for database operations
     - Address path resolution issues in file processing

## Next Major Milestone

**Milestone 7.5 (Continued): "SQLite Repository Implementation"** - Complete the SQLite implementation with `SqliteCutsRepository` and address identified issues.

## Upcoming Work (Revised Plan)

1. ✅ **Complete Repository Trait Refactoring:**
   - ✅ All steps completed (1-6).
2. **Complete SQLite Implementation:**
   - ✅ Partial completion: Added SQLx dependency, created database module, implemented `SqliteMaterialRepository`
   - Implement `SqliteCutsRepository` using the same pattern
   - Add transaction support for operations requiring atomicity
   - Enhance error handling for database operations
   - Resolve path issues in file processing
   - Enhance the CLI options for repository selection
3. **Basic Swatching Actor (M8):**

   - Create skeleton actor, subscribe to `MaterialCut` events, implement internal queue pattern
   - Follow the same dual-task pattern (listener/processor) as the CuttingActor
   - Ensure proper lifecycle management and event handling

4. **Swatching Logic (M9):** Implement swatch creation within the `SwatchingActor`'s processor task.

5. **Swatch Repository (M10):** Implement storage for swatches.

6. **Basic Query (M11):** Simple search capability.

7. **Reconciliation Actor (M12):** Implement the actor for handling stuck items and retries.

8. **Persistence (M13):** Implement file-based persistence for events and repositories.

## What's Left to Build (Immediate Milestones)

- `SqliteCutsRepository` implementation.
- Swatching Actor implementation (Milestone 8+).
- Reconciliation Actor (Milestone 12).
- Persistence for events and repositories (Milestone 13).
- Advanced features (scaling, enhanced cutting, search, UI, etc.).

## Current Status

- Core pipeline (Discovery -> Cutting) is functional with both in-memory and SQLite options for the material repository (cuts repository is still in-memory).
- Basic text cutting is implemented.

## Known Issues

- `SqliteCutsRepository` is not yet implemented; cuts are only stored in memory.
