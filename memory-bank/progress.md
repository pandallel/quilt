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

1.  **Milestone 7.5 Pre-Refactoring (In Progress):**
    - Starting the prerequisite refactoring for SQLite implementation.
    - **Current Step:** Renaming the `MaterialRepository` struct to `InMemoryMaterialRepository` across the project.
    - **Next:** Validate the rename thoroughly via `cargo check` and `cargo test` before introducing the trait.

## Next Major Milestone

**Milestone 7.5: "SQLite Repository Implementation"** - Requires completion of prerequisite refactoring before implementation begins.

## Upcoming Work (Revised Plan)

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
7.  **Basic Swatching Actor (M8):** Create skeleton actor, subscribe to `MaterialCut` events, implement internal queue pattern.
8.  **Swatching Logic (M9):** Implement swatch creation within the `SwatchingActor`'s processor task.
9.  **Swatch Repository (M10):** Implement storage for swatches.
10. **Basic Query (M11):** Simple search capability.
11. **Reconciliation Actor (M12):** Implement the actor for handling stuck items and retries.
12. **Persistence (M13):** Implement file-based persistence for events and repositories.

## What's Left to Build (Immediate Milestones)

1.  **Repository Refactoring:** Finalize trait-based repositories.
2.  **SQLite Repository Infrastructure (M7.5 - In-Memory):** Implement in-memory SQLite versions of both repositories.
3.  **Basic Swatching Actor (M8):** Create skeleton actor, subscribe to `MaterialCut` events, implement internal queue pattern.
4.  **Swatching Logic (M9):** Implement swatch creation within the `SwatchingActor`'s processor task.
5.  **Swatch Repository (M10):** Implement storage for swatches.
6.  **Basic Query (M11):** Simple search capability.
7.  **Reconciliation Actor (M12):** Implement the actor for handling stuck items and retries.
8.  **Persistence (M13):** Implement file-based persistence for events and repositories.

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
- **Repository Refactoring:** The need to refactor `MaterialRepository` into a trait before implementing SQLite was identified.
- **Cutting Actor Dependency:** The `CuttingActor` needs to be updated to use the `CutsRepository` trait for dependency injection.
