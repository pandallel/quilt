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

### Milestone 8: "Basic Swatching Actor Creation"

**Goal:** Create a minimal Swatching Actor that listens for cut events and sets up internal backpressure queue
**Implementation Time:** 2-3 days

1. Create Swatching Actor skeleton (1-2 days)

   - Implement simple actor that subscribes to MaterialCut events from the shared `EventBus`
   - Set up internal bounded `mpsc` channel
   - Spawn internal 'Listener Task': receives events, filters for `MaterialCut`, logs receipt, tries sending to internal queue
   - Spawn internal 'Processor Task': receives from internal queue, logs receipt (no processing yet)
   - Add logging for received events on both tasks
   - Create basic actor lifecycle management including task cleanup

2. Set up event flow monitoring (1 day)
   - Add event flow tracking between actors
   - Implement metrics for the pipeline
   - Create visualization of event flow in logs
   - Add configuration options

**Demonstration:** Running `main` shows "Swatching Actor received X MaterialCut events" in logs without processing them

### Milestone 9: "Swatching Actor Processes Cuts"

**Goal:** Implement actual swatch creation in the Swatching Actor's processor task
**Implementation Time:** 3-4 days

1. Add swatch creation functionality (2 days)

   - Implement metadata extraction (within Processor Task)
   - Create content analysis features (within Processor Task)
   - Add embedding generation with async processing (within Processor Task, potentially using `spawn_blocking` or concurrent futures)
   - Keep detailed metrics of swatch creation

2. Implement MaterialSwatched events (1-2 days)
   - Add event publishing for completed swatches (from Processor Task)
   - Create state transition in Registry (from Processor Task)
   - Add validation through logging
   - Implement recovery for failed swatches (basic error reporting for now)

**Demonstration:** Running `main` shows "Created X swatches from Y cuts" with detailed processing metrics

### Milestone 10: "Swatch Repository and Complete Pipeline"

**Goal:** Complete the storage and finalize the processing pipeline
**Implementation Time:** 2-3 days

1. Implement SwatchRepository (1-2 days)

   - Create in-memory storage for swatches
   - Implement CRUD operations
   - Add indexing for efficient retrieval
   - Create comprehensive tests

2. Validate full event pipeline (1 day)
   - Add end-to-end metrics for the pipeline
   - Create visualization of complete material flow
   - Add system health checks
   - Implement recovery for pipeline failures

**Demonstration:** Running `main` displays "Full pipeline metrics: X discovered → Y cut → Z swatched" with complete flow statistics

### Milestone 11: "Basic Query Capability"

**Goal:** Enable simple searching of swatches
**Implementation Time:** 2-3 days

1. Implement basic search functionality (1-2 days)

   - Add simple text indexing in SwatchRepository
   - Create basic query API
   - Implement search results formatter
   - Add search metrics

2. Integrate with the main application (1 day)
   - Add search command to the interface
   - Create results display
   - Add error handling for searches
   - Implement logging for search operations

**Demonstration:** Running `main` with search parameter shows "Found X swatches matching query 'Z'" with results displayed

### Milestone 12: "Reconciliation Actor Implementation"

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

### Milestone 13: "Event and Data Persistence" (Renumbered from 12)

**Goal:** Ensure data and events persist between application runs
**Implementation Time:** 3-4 days

1. Implement event logging (1-2 days)

   - Add event serialization
   - Create file-based event log
   - Implement event replay on startup
   - Add validation for event consistency

2. Add repository persistence (1-2 days)
   - Implement serialization for repository data
   - Create file-based storage
   - Add startup/shutdown procedures
   - Create recovery mechanisms

**Demonstration:** Stopping and restarting `main` shows "Recovered X events and restored system state" with intact data

## Future Milestones

Future milestones will focus on more advanced features:

1. **Scaling and Performance**

   - Swatching Router implementation for dynamic actor scaling
     - Router actor for managing multiple Swatching Actors
     - Queue length monitoring and health checks (both internal `mpsc` and `EventBus` lag)
     - Dynamic actor pool management
   - Enhanced caching strategies
   - Load balancing and monitoring
   - Performance optimization based on usage patterns (tuning queue sizes, buffer capacities)

2. **Cutting Enhancements**

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

3. **Storage and Persistence Improvements**

   - Implement disk-based repository options for cuts and materials
   - Add streaming processing for very large files
   - Create efficient indexing strategies for large repositories
   - Implement data compression for storage efficiency
   - Add data integrity validation and repair mechanisms

4. **Enhanced Logging and Observability**

   - Implement structured logging with span contexts
   - Create comprehensive tracing for request flows
   - Add detailed performance metrics collection
   - Implement health monitoring dashboards
   - Create alerting for system issues

5. **Enhanced Text Processing**

   - Language detection
   - Text classification
   - Entity extraction

6. **Embedding and Vector Search**

   - Integration with embedding models
   - Vector storage for semantic search
   - Similarity search implementation

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
