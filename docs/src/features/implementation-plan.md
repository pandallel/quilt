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

**Goal:** Create a minimal Cutting Actor that listens for events
**Implementation Time:** 2-3 days
**Status:** Completed

1. ✅ Create Cutting Actor skeleton (1-2 days)

   - ✅ Implement simple actor that subscribes to MaterialDiscovered events
   - ✅ Add logging for received events
   - ✅ Create basic actor lifecycle (start/stop)
   - ✅ Do not implement actual material processing yet

2. ✅ Set up actor monitoring (1 day)
   - ✅ Add heartbeat logging for the actor
   - ✅ Implement basic health checks
   - ✅ Add subscription metrics
   - ✅ Create actor configuration structure

**Demonstration:** Running `main` shows "Cutting Actor received X MaterialDiscovered events" in logs without processing them

### Milestone 6: "Material Text Cutting Implementation"

**Goal:** Implement the text-based document cutting functionality that transforms discovered materials into meaningful cuts
**Implementation Time:** 3-4 days

1. Integrate text-splitter crate (1 day)

   - Add text-splitter dependency to Cargo.toml
   - Create TextCutter implementation using TextSplitter
   - Configure with default token sizes (target: 300, min: 150, max: 800)

2. Implement basic cutting process (1-2 days)

   - Add text extraction from materials
   - Integrate with TextSplitter for content chunking
   - Create MaterialCut model from text-splitter chunks
   - Implement error handling with fallback strategy

3. Implement Cut event publishing (1 day)

   - Add MaterialCut event creation and publishing
   - Update material status in registry (Processing → Cut)
   - Implement error reporting for failed cuts
   - Add metrics for cut creation (count, size distribution)

**Demonstration:** Running `main` shows "Created X cuts from Y materials using TextSplitter" with detailed metrics in logs

### Milestone 7: "Cuts Repository Implementation"

**Goal:** Store processed cuts with event integration
**Implementation Time:** 2-3 days

1. Implement CutsRepository (1-2 days)

   - Create in-memory storage for cuts
   - Implement CRUD operations
   - Add integration with Registry
   - Create comprehensive tests

2. Connect Cutting Actor to repository (1 day)
   - Add repository interaction in actor
   - Implement proper error handling
   - Add storage metrics and logging
   - Create validation of stored cuts

**Demonstration:** Running `main` logs "Stored X cuts in repository" with metrics on storage operations

### Milestone 8: "Basic Swatching Actor Creation"

**Goal:** Create a minimal Swatching Actor that listens for cut events
**Implementation Time:** 2-3 days

1. Create Swatching Actor skeleton (1-2 days)

   - Implement simple actor that subscribes to MaterialCut events
   - Add logging for received events
   - Create basic actor lifecycle management
   - Do not implement swatch creation yet

2. Set up event flow monitoring (1 day)
   - Add event flow tracking between actors
   - Implement metrics for the pipeline
   - Create visualization of event flow in logs
   - Add configuration options

**Demonstration:** Running `main` shows "Swatching Actor received X MaterialCut events" in logs without processing them

### Milestone 9: "Swatching Actor Processes Cuts"

**Goal:** Implement actual swatch creation in the Swatching Actor
**Implementation Time:** 3-4 days

1. Add swatch creation functionality (2 days)

   - Implement metadata extraction
   - Create content analysis features
   - Add embedding generation with async processing
   - Keep detailed metrics of swatch creation

2. Implement MaterialSwatched events (1-2 days)
   - Add event publishing for completed swatches
   - Create state transition in Registry
   - Add validation through logging
   - Implement recovery for failed swatches

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

### Milestone 12: "Event and Data Persistence"

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
     - Queue length monitoring and health checks
     - Dynamic actor pool management
   - Enhanced caching strategies
   - Load balancing and monitoring
   - Performance optimization based on usage patterns

2. **Enhanced Text Processing**

   - Language detection
   - Text classification
   - Entity extraction

3. **Embedding and Vector Search**

   - Integration with embedding models
   - Vector storage for semantic search
   - Similarity search implementation

4. **Advanced Search and Queries**

   - Query language development
   - Search result ranking
   - Filter and facet implementation

5. **Enhanced Cutting Strategies**

   - Markdown content cutting
     - MarkdownCutter implementation using MarkdownSplitter
     - Format detection for Markdown content
     - Fallback to TextCutter on errors
   - Source code cutting
     - CodeCutter implementation using CodeSplitter
     - Language detection for code content
     - Specialized semantic boundary handling for code structures
   - Format-specific optimizations including language-specific tokenization and semantic boundary recognition

6. **User Interfaces**

   - Web-based dashboard
   - Search interface
   - Material management

7. **Integration APIs**
   - REST API for external access
   - Webhooks for processing events
   - Subscription mechanism for updates
