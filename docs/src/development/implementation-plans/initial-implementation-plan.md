# Quilt Implementation Plan

This document outlines the incremental implementation plan for Quilt's core architecture, following a feature-first approach with clear observable milestones.

## Legend

- ✅ Task completed
- ⚠️ Task requires special attention
- ⏩ Task intentionally deferred

## Completed Foundation Work

1. ✅ **Material Repository Setup**

   - ✅ Implemented thread-safe in-memory store using `Arc<RwLock<HashMap<...>>>`
   - ✅ Created material state tracking (Discovered, Cut, Swatched, Error)
   - ✅ Added basic CRUD operations with idempotence checks and state transition validation
   - ✅ Added comprehensive tests for all operations
   - ✅ Used Tokio's async synchronization primitives for better integration with the actor model

2. ✅ **Basic Material Data Structures**

   - ✅ Material struct with metadata implemented
   - ⏩ Swatch implementation deferred to a later milestone

3. ✅ **Message Channel System**

   - ✅ Defined actor-specific message types for direct communication
   - ✅ Set up Tokio mpsc channels with appropriate capacity (100)
   - ✅ Implemented channel creation and utilities for connecting stages
   - ✅ Added helper traits for sending/receiving with timeout capabilities
   - ✅ Created comprehensive tests for message passing and backpressure

4. ✅ **Configuration and Logging Improvements**
   - ✅ Renamed `ScanConfig` to `DiscoveryConfig` for better naming consistency
   - ✅ Enhanced discovery logging to show total repository material count
   - ✅ Updated actor framework to use current actix runtime

## E2E Development Milestones

### ✅ Milestone 1: "Actor System Logs Startup"

**Goal:** Establish the basic actor framework with proper initialization
**Implementation Time:** 2-3 days
**Status:** Completed

1. ✅ Setup actor framework foundation (1-2 days)

   - ✅ Define basic actor trait and message types
     - ✅ Created common message types (Ping, Shutdown) in actors module
     - ✅ Implemented actor-specific message types (StartDiscovery)
     - ✅ Ensured thread safety and proper ownership in async context
   - ✅ Create actor system initialization patterns
   - ✅ Ensure proper Actix/Tokio runtime integration
     - ✅ Used Actix runtime with #[actix::main] in main.rs
     - ✅ Added proper async actor initialization and message handling

2. ✅ Add logging infrastructure (1 day)
   - ✅ Setup structured logging with env_logger
   - ✅ Created actor lifecycle logging (start/stop/message events)

**Demonstration:** Running `main` shows "Actor system started" with proper configuration in logs, and the DiscoveryActor successfully handles messages.

### ✅ Milestone 2: "Discovery Actor Uses Scanner for Single Directory"

**Goal:** Connect existing DirectoryScanner to the actor framework
**Implementation Time:** 2-3 days
**Status:** Completed

1. ✅ Create DiscoveryActor using existing scanner (1-2 days)

   - ✅ Wrap DirectoryScanner in actor interface
   - ✅ Add configuration for target directory
   - ✅ Implement material creation from scanned files

2. ✅ Add basic material processing logic (1 day)
   - ✅ Log discovered materials with metadata
   - ✅ Implement material state tracking
   - ✅ Enhanced logging with repository statistics

**Demonstration:** Running `main` shows list of materials found in the configured directory with repository statistics

### ✅ Milestone 3: "Event Bus and Material Registry Foundation"

**Goal:** Establish the core communication infrastructure
**Implementation Time:** 2-3 days
**Status:** Completed

1. ✅ Implement basic Event Bus (1 day)

   - ✅ Create central event bus using `tokio::sync::broadcast` channels
   - ✅ Implement simple event types (MaterialDiscovered event only)
   - ✅ Add logging for event publishing and subscription
   - ✅ Create basic tests that verify event transmission

2. ✅ Create Material Registry prototype (1-2 days)
   - ✅ Implement basic registry that works alongside existing Repository
   - ✅ Add minimal event publishing for material discovery
   - ✅ Create simple validation of events using logging
   - ✅ Keep the existing Repository functionality intact

**Demonstration:** Running `main` shows "Event Bus initialized" in logs and demonstrates events flowing with log messages

### ✅ Milestone 4: "Discovery Actor Publishes Events"

**Goal:** Make Discovery Actor use the Event Bus for one simple operation
**Implementation Time:** 2-3 days
**Status:** Completed

1. ✅ Update Discovery Actor to publish events (1-2 days)

   - ✅ Add event publishing for discovered materials
   - ✅ Keep existing direct interactions for compatibility
   - ✅ Add logging to show event publishing
   - ✅ Create simple test harness for validation

2. ✅ Add event monitoring (1 day)
   - ✅ Implement simple event listener in the main application
   - ✅ Log all published events with timestamps
   - ✅ Display event counts in logs
   - ✅ Add basic metrics for event flow
   - ✅ Improved log level for event monitoring (changed from info to debug)

**Demonstration:** Running `main` shows "Published X MaterialDiscovered events" with event details in logs

### ✅ Milestone 5: "Basic Cutting Actor Creation"

**Goal:** Create a minimal Cutting Actor that listens for events and sets up internal backpressure queue
**Implementation Time:** 2-3 days
**Status:** Completed

1. ✅ Create Cutting Actor skeleton (1-2 days)

   - ✅ Implement simple actor that subscribes to MaterialDiscovered events from the shared `EventBus`
   - ✅ Set up internal bounded `mpsc` channel (sender/receiver pair)
   - ✅ Spawn internal 'Listener Task': receives events from `EventBus`, filters for `MaterialDiscovered`, logs receipt, tries sending to internal `mpsc` queue (no blocking/backpressure handling yet)
   - ✅ Spawn internal 'Processor Task': receives from internal `mpsc` queue, logs receipt (no processing yet)
   - ✅ Add logging for received events on both tasks
   - ✅ Create basic actor lifecycle (start/stop) including task cleanup

2. ✅ Set up actor monitoring (1 day)
   - ✅ Add heartbeat logging for the actor
   - ✅ Implement basic health checks
   - ✅ Add subscription metrics
   - ✅ Create actor configuration structure

**Demonstration:** Running `main` shows "Cutting Actor received X MaterialDiscovered events" in logs without processing them

### ✅ Milestone 6: "Material Text Cutting Implementation"

**Goal:** Implement the text-based document cutting functionality within the Cutting Actor's processor task
**Implementation Time:** ~4 days (Completed)
**Status:** ✅ Completed

1. ✅ Integrate text-splitter crate (1 day)

   - ✅ Add text-splitter dependency to Cargo.toml
   - ✅ Create TextCutter implementation using TextSplitter
   - ✅ Configure with default token sizes (target: 300, min: 150, max: 800)

2. ✅ Integrate with TextSplitter for content chunking (within Processor Task)

   - ✅ Use TextCutter to split file content into chunks (within Processor Task)
   - ✅ Implement error handling with fallback strategy (within Processor Task)
   - ✅ Handle backpressure: Listener task uses `await send()` and handles `EventBus` lag (`RecvError::Lagged`)

3. ✅ Implement State Update and Event Publishing (within Material Registry) (1 day)

   - ✅ Update material status in registry (Discovered → Cut or Discovered → Error) via `update_material_status`.
   - ✅ Implement MaterialCut event creation and publishing _within the registry_ upon successful transition to `Cut` state.
   - ✅ Implement ProcessingError event creation and publishing _within the registry_ upon transition to `Error` state, inferring the stage (`Cutting`) from the previous state.
   - ✅ Implement error reporting for failed cuts via the `Error` state transition.
   - ✅ Add metrics for cut creation (count, size distribution)

**Demonstration:** Running `main` shows "Created X cuts from Y materials using TextSplitter" with detailed metrics in logs

### ✅ Milestone 7: "Cuts Repository Implementation"

**Goal:** Store processed cuts with event integration
**Implementation Time:** 2-3 days
**Status:** ✅ Completed

1. ✅ Implement CutsRepository (1-2 days)

   - ✅ Create in-memory storage for cuts
   - ✅ Implement CRUD operations
   - ✅ Add integration with Registry
   - ✅ Create comprehensive tests

2. ✅ Connect Cutting Actor to repository (1 day)
   - ✅ Add repository interaction in actor
   - ✅ Implement proper error handling
   - ✅ Add storage metrics and logging
   - ✅ Create validation of stored cuts

**Demonstration:** Running `main` logs "Stored X cuts in repository" with metrics on storage operations

### Milestone 7.5: SQLite Repository Implementation

#### Prerequisites: Repository Trait Refactoring

1. ✅ Rename and validate Material Repository
   - ✅ Rename `struct MaterialRepository` to `struct InMemoryMaterialRepository`.
   - ✅ Update all external usages.
   - ✅ Validate with `cargo check` and `cargo test`.
2. ✅ Add trait for Material Repository
   - ✅ Add `async-trait` dependency.
   - ✅ Define `trait MaterialRepository` in `repository.rs` using `#[async_trait]`.
   - ✅ Implement the trait for `InMemoryMaterialRepository`.
   - ✅ Validate: ensure `cargo check` and `cargo test` pass.
3. ✅ Update the Registry to use the trait
   - ✅ Update `MaterialRegistry` to use `Arc<dyn MaterialRepository>`.
   - ✅ Update dependent code and tests.
   - ✅ Validate: ensure `cargo check` and `cargo test` pass.
4. ✅ Update Actor Dependencies
   - ✅ Update `Orchestrator`, `CuttingActor`, `DiscoveryActor` initialization and tests.
   - ✅ Validate: ensure `cargo check` and `cargo test` pass.
5. ✅ Move trait definitions to mod.rs
   - ✅ Move trait/error definitions to `mod.rs`.
   - ✅ Validate: ensure `cargo check` and `cargo test` pass.
6. ✅ Apply Repository Trait Pattern to CutsRepository
   - ✅ Define `trait CutsRepository` in appropriate location.
   - ✅ Update the `CuttingActor` to use the trait.
   - ✅ Validate: ensure `cargo check` and `cargo test` pass.

#### SQLite Implementation

1. ✅ Add SQLite dependencies

   - ✅ Add `sqlx` with `sqlite`, `runtime-tokio-rustls` and `time` features to Cargo.toml.
   - ✅ Add basic initialization and tests.

2. ✅ Create database setup module

   - ✅ Implement `init_memory_db()` function in new `src/db.rs` module
   - ✅ Set up schema definition for the `materials` table
   - ✅ Add test for database initialization

3. ✅ Implement SQLite Material Repository

   - ✅ Create `src/materials/sqlite_repository.rs` for `SqliteMaterialRepository`
   - ✅ Implement `MaterialRepository` trait with SQLite backend
   - ✅ Add proper row to struct conversion. Rename `ingested_at` column/field to `created_at`. Add `status_updated_at` and `updated_at` columns/fields.
   - ✅ Utilize `sqlx`\'s `"time"` feature for automatic `OffsetDateTime` encoding/decoding (removes manual parsing/formatting).
   - ✅ Add comprehensive tests comparing with in-memory implementation

4. ✅ Update app integration

   - ✅ Add module exports in `lib.rs`
   - ✅ Modify `QuiltOrchestrator` to include SQLite support
   - ✅ Add command-line option for selecting repository type
   - ✅ Implement fallback to in-memory repository if SQLite fails

5. ⚠️ Known Issues

   - ~~When running with `--dir=./src`, file paths are relative to the working directory, causing errors like `Failed to read file 'materials/types.rs': No such file or directory (os error 2)`~~ **Resolved (See Below)**
   - This issue was fixed by ensuring the `DiscoveryActor` resolves relative paths to absolute paths before registering materials. The `CuttingActor` now receives absolute paths via the `MaterialDiscoveredEvent`.

6. ✅ Implement SqliteCutsRepository
   - ✅ Create schema for cuts table in `src/db.rs`
   - ✅ Implement `SqliteCutsRepository` in `src/cutting/sqlite_repository.rs`
   - ✅ Add comprehensive tests for all repository operations
   - ✅ Update `QuiltOrchestrator` to use `SqliteCutsRepository` when in SQLite mode
   - ✅ Ensure proper foreign key constraints with materials table

**Demonstration:** Running `cargo run -- --dir=./src` uses SQLite by default for both materials and cuts, while `cargo run -- --dir=./src --in-memory` uses the original in-memory stores.

### ✅ Task: Refactor Material Timestamps

**Goal:** Improve timestamp tracking for materials by renaming `ingested_at` and adding status change and general update timestamps.
**Status:** Completed

1. ✅ **Add Timestamps:**
   - ✅ Renamed `ingested_at` field/column to `created_at`.
   - ✅ Added `status_updated_at` field/column.
   - ✅ Added `updated_at` field/column.
   - ✅ Updated `Material` struct, `Material::new()`, database schema (`src/db.rs`).
2. ✅ **Implement Update Logic:**
   - ✅ Updated `InMemoryMaterialRepository::update_material_status` to set `status_updated_at` and `updated_at` to `now` on success.
   - ✅ Updated `SqliteMaterialRepository::update_material_status` SQL to set `status_updated_at` and `updated_at` to `now` on success.
   - ✅ Updated `SqliteMaterialRepository::register_material` SQL to insert all three timestamps.
3. ✅ **Leverage `sqlx` Time Feature:**
   - ✅ Added `"time"` feature to `sqlx` dependency in `Cargo.toml`.
   - ✅ Refactored `SqliteMaterialRepository` to use automatic `OffsetDateTime` encoding/decoding via `sqlx`, removing manual parsing/formatting.
   - ✅ Updated tests in both repositories.

### ✅ Milestone 8: "Basic Swatching Actor Creation"

**Goal:** Create a minimal Swatching Actor that listens for cut events and sets up internal backpressure queue
**Implementation Time:** 2-3 days
**Status:** ✅ Completed

1. ✅ Create Swatching Actor skeleton (1-2 days)

   - ✅ Implement simple actor that subscribes to MaterialCut events from the shared `EventBus`
   - ✅ Set up internal bounded `mpsc` channel
   - ✅ Spawn internal 'Listener Task': receives events, filters for `MaterialCut`, logs receipt, tries sending to internal queue
   - ✅ Spawn internal 'Processor Task': receives from internal queue, logs receipt (no processing yet)
   - ✅ Add logging for received events on both tasks
   - ✅ Create basic actor lifecycle management including task cleanup

2. ✅ Integrate with Orchestrator
   - ✅ Add SwatchingActor to the QuiltOrchestrator initialization
   - ✅ Include in the shutdown sequence with proper order
   - ✅ Ensure event flow from CuttingActor to SwatchingActor
   - ✅ Add comprehensive tests for actor lifecycle

**Demonstration:** Running `main` shows "Swatching Actor received X MaterialCut events" in logs without processing them

### ✅ Milestone 9: "Define Swatch Structures & Repository Trait"

**Goal:** Define the core data structures and persistence contract for swatches.
**Implementation Time:** ~1 day
**Status:** ✅ Completed

1. ✅ Define Swatch data structures (1 day)
   - ✅ Define `Swatch` struct in `src/swatching/swatch.rs` with necessary fields (id, cut_id, material_id, embedding `Vec<f32>`, model_name, model_version, dimensions, metadata).
   - ✅ Define `SwatchRepository` trait in `src/swatching/repository.rs` with comprehensive async CRUD and search methods using `#[async_trait::async_trait]`.
   - ✅ Add appropriate error types and result type alias for repository operations.
   - ✅ Update module exports in `src/swatching/mod.rs` to expose the new types.
   - ✅ Add `serde` and `serde_json` dependencies for metadata serialization.
   - ✅ Implement comprehensive unit tests for the `Swatch` struct.

**Demonstration:** Code compiles with the new `Swatch` type and `SwatchRepository` trait definitions, including similarity search method signatures for future implementation.

### Milestone 10: "Implement SQLite Swatch Repository"

**Goal:** Implement the SQLite persistence layer for swatches, anticipating future `sqlite-vec` use.
**Implementation Time:** ~2 days

1. Implement SQLite Repository (2 days)
   - Define `swatches` table schema in `src/db.rs`, including a field for embeddings (e.g., `BLOB`) and foreign key to `materials`. Update DB initialization.
   - Implement `SqliteSwatchRepository` struct implementing the `SwatchRepository` trait using `sqlx`.
   - Add comprehensive unit tests for `SqliteSwatchRepository` operations.
   - Update `QuiltOrchestrator` to initialize and manage the `SqliteSwatchRepository` (unaffected by `--in-memory` flag). Inject the `Arc<dyn SwatchRepository>` into `SwatchingActor` upon creation.

**Demonstration:** Unit tests for `SqliteSwatchRepository` pass. Application runs, injecting the repository into `SwatchingActor`.

### Milestone 11: "Implement Swatching Actor Logic & Event"

**Goal:** Connect the Swatching Actor to retrieve cuts, generate **actual embeddings**, create swatches, store them via the repository, update registry status, and publish the `MaterialSwatched` event.
**Implementation Time:** ~3-4 days (Increased due to embedding integration)

1. Implement Embedding Generation (1-2 days)
   - Choose and integrate an embedding strategy (e.g., `rust-bert`, ONNX runtime with a sentence transformer model).
   - Implement logic within the `SwatchingActor`'s processor task to generate embeddings for retrieved `Cut`s.
2. Implement Swatching Logic (Integration) (1-2 days)
   - Modify `SwatchingActor`'s processor task:
     - Retrieve `Cut`s using injected `CutsRepository`.
     - Use the implemented embedding generation logic.
     - Create `Swatch` instances with actual embeddings.
     - Save `Swatch` instances using injected `SwatchRepository`.
     - Call `MaterialRegistry` to update material status to `Swatched` (or `Error`).
3. Implement MaterialSwatched Event (1 day)
   - Define `MaterialSwatched` event variant in `QuiltEvent` enum.
   - Update `MaterialRegistry` to publish `MaterialSwatched` event upon successful transition to `Swatched` state.
   - Add/update integration tests for the `SwatchingActor` covering cut retrieval, embedding generation, swatch saving, registry update, and event publication.

**Demonstration:** Running `main` shows "Created X swatches with embeddings..." logs, material status updates to `Swatched`, and `MaterialSwatched` events are published. Integration tests verify embedding creation and storage.

### Milestone 12: "Implement Basic Semantic Search"

**Goal:** Implement basic vector similarity search using the stored swatches and embeddings.
**Implementation Time:** ~2-3 days

1. Integrate Vector Search Extension (1 day)
   - Integrate `sqlite-vec` or a similar vector search extension with the SQLite setup (`src/db.rs`).
   - Update the `swatches` table schema and `SqliteSwatchRepository` (from M10) to store and index the embedding vectors correctly for search.
2. Implement Search Logic (1-2 days)
   - Add a function or method (e.g., in `SqliteSwatchRepository` or a new query service) to perform vector similarity search.
   - Implement a basic query interface (e.g., a new command-line argument or internal function) that takes query text, generates its embedding, and performs the search.
   - Add tests for the vector search functionality.

**Demonstration:** Running the application with a search query performs semantic search and returns relevant swatch IDs/content based on vector similarity.

### Milestone 13: "Reconciliation Actor Implementation"

**Goal:** Implement the Reconciliation Actor to handle stuck materials and retries
**Implementation Time:** 3-4 days

1. Create Reconciliation Actor skeleton (1 day)

   - Implement Actor structure
   - Add configuration for scan interval, retry limits, timeouts per stage
   - Set up periodic triggering using `ctx.run_interval`

2. Implement Registry Scanning Logic (1-2 days)

   - Query `MaterialRegistry` (or `MaterialRepository`) for materials in intermediate states (`Cutting`, `Swatching`) longer than configured timeout
   - Implement logic to check `retry_count` against `max_retries`

3. Implement Retry and Error Handling (1 day)
   - If retries remain: update retry count/timestamp in Registry/Repository, re-publish preceding event (e.g., `MaterialDiscovered`) to shared `EventBus`
   - If max retries exceeded: update material status to `Error` in Registry/Repository
   - Add comprehensive logging and metrics for reconciliation actions (scans, retries, errors)

**Demonstration:** Running `main` shows logs from the Reconciliation Actor identifying stuck items (if any manually created), attempting retries, and eventually marking items as Error after exceeding retries.

### Milestone 14: "Event and Data Persistence"

**Goal:** Ensure data and events persist between application runs (Focus on event log, repository persistence is largely covered by SQLite)
**Implementation Time:** 2-3 days

1. Implement event logging (1-2 days)

   - Add event serialization
   - Create file-based event log
   - Implement event replay on startup
   - Add validation for event consistency

2. Add repository persistence (1 day)
   - ⏩ Defer file-based serialization for repositories; SQLite handles persistence.
   - Add startup/shutdown procedures for event log
   - Create recovery mechanisms for event log

**Demonstration:** Stopping and restarting `main` shows "Recovered X events..." with intact data in SQLite and replayed events.

## Future Milestones

Future milestones will focus on more advanced features:

1. **Embedding Generation & Vector Search**

   - Integration with embedding models (e.g., Sentence Transformers via ONNX or similar)
   - Vector storage integration (e.g., `sqlite-vec`) with `SqliteSwatchRepository`
   - Implementation of vector similarity search logic
   - Query API for semantic search

2. **Scaling and Performance**

   - Swatching Router implementation for dynamic actor scaling
     - Router actor for managing multiple Swatching Actors
     - Queue length monitoring and health checks (both internal `mpsc` and `EventBus` lag)
     - Dynamic actor pool management
   - Enhanced caching strategies
   - Load balancing and monitoring
   - Performance optimization based on usage patterns (tuning queue sizes, buffer capacities)

3. **Cutting Enhancements**

   - Improve backpressure handling
     - Add explicit backpressure strategy when internal queue fills up
     - Implement circuit-breaking for continuous error situations
     - Add metrics for queue depth and backpressure events
   - Implement retry mechanisms
     - Add retry capability for recoverable errors
     - Implement exponential backoff strategy
     - Create configurable retry policies per error type
   - Enhance configuration
     - Make cutting parameters configurable (chunk size, overlap)
     - Allow runtime configuration updates

4. **Storage and Persistence Improvements**

   - Implement disk-based repository options for cuts and materials
   - Add streaming processing for very large files
   - Create efficient indexing strategies for large repositories
   - Implement data compression for storage efficiency
   - Add data integrity validation and repair mechanisms

5. **Enhanced Logging and Observability**

   - Implement structured logging with span contexts
   - Create comprehensive tracing for request flows
   - Add detailed performance metrics collection
   - Implement health monitoring dashboards
   - Create alerting for system issues

6. **Enhanced Text Processing**

   - Language detection
   - Text classification
   - Entity extraction

7. **Advanced Search and Queries**

   - Query language development
   - Search result ranking
   - Filter and facet implementation

8. **Enhanced Cutting Strategies**

   - Markdown content cutting
     - MarkdownCutter implementation using MarkdownSplitter
     - Format detection for Markdown content
     - Fallback to TextCutter on errors
   - Source code cutting
     - CodeCutter implementation using CodeSplitter
     - Language detection for code content
     - Specialized semantic boundary handling for code structures
   - Format-specific optimizations including language-specific tokenization and semantic boundary recognition

9. **User Interfaces**

   - Web-based dashboard
   - Search interface
   - Material management

10. **Integration APIs**
    - REST API for external access
    - Webhooks for processing events
    - Subscription mechanism for updates
