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
   - ✅ Defined MaterialMessage enum (Discovered, Cut, Swatched, Error, Shutdown)
   - ✅ Set up Tokio mpsc channels with appropriate capacity (100)
   - ✅ Implemented channel creation and utilities for connect stages
   - ✅ Added helper traits for sending/receiving with timeout capabilities
   - ✅ Created comprehensive tests for message passing and backpressure

## E2E Development Milestones

### Milestone 1: "Actor System Logs Startup"

**Goal:** Establish the basic actor framework with proper initialization
**Implementation Time:** 2-3 days

1. Setup actor framework foundation (1-2 days)

   - Define basic actor trait and message types ⚠️
     - Challenge: Design message types balancing between passing full objects vs just IDs
     - Consider memory usage implications for large message volumes
     - Address thread safety and ownership in async context
   - Create actor system initialization patterns
   - Ensure proper Actix/Tokio runtime integration ⚠️
     - Challenge: Prevent nested runtime errors between Actix and Tokio
     - Actix system should be initialized first, with Tokio operations within its context
     - Alternative: Run Actix in a dedicated thread managed by Tokio

2. Add logging infrastructure (1 day)
   - Setup structured logging with context
   - Create actor lifecycle logging

**Demonstration:** Running `main` shows "Actor system started" with proper configuration in logs

### Milestone 2: "Discovery Actor Uses Scanner for Single Directory"

**Goal:** Connect existing DirectoryScanner to the actor framework
**Implementation Time:** 2-3 days

1. Create DiscoveryActor using existing scanner (1-2 days)

   - Wrap DirectoryScanner in actor interface
   - Add configuration for target directory
   - Implement material creation from scanned files

2. Add basic material processing logic (1 day)
   - Log discovered materials with metadata
   - Implement material state tracking

**Demonstration:** Running `main` shows list of materials found in the configured directory

### Milestone 3: "Discovery Actor Sends Material Messages"

**Goal:** Establish message passing between actors
**Implementation Time:** 2-3 days

1. Connect to message channel system (1-2 days)

   - Utilize existing MaterialMessage enum types
   - Configure Tokio mpsc channels
   - Implement channel registration and connection

2. Configure Discovery actor to send messages (1 day)
   - Serialize materials into messages ⚠️
     - Challenge: Balance between passing full objects vs just IDs
     - Consider ownership transfer between components
     - Address thread safety in async context
   - Implement proper channel selection
   - Add sending logic with logging

**Demonstration:** Running `main` logs "Sent X material messages to channel"

### Milestone 4: "Cutting Actor Creates Document Cuts"

**Goal:** Process materials into cuts
**Implementation Time:** 3-4 days

1. Create CuttingActor implementation (1-2 days)

   - Add message reception from Discovery
   - Implement basic cut creation logic
   - Add file content extraction ⚠️
     - Challenge: CPU-intensive operations can block the async runtime
     - Use `spawn_blocking` for long-running operations
     - Implement async I/O operations where possible

2. Implement document splitting strategies (2 days)
   - Create text extraction pipeline
   - Implement basic cutting strategies (paragraphs, fixed size)
   - Add metadata extraction for cuts

**Demonstration:** Running `main` shows "Created X cuts from material Y" with cut details

### Milestone 5: "CutsRepository Stores Document Cuts"

**Goal:** Store processed cuts for later retrieval
**Implementation Time:** 2-3 days

1. Implement CutsRepository (1-2 days)

   - Create in-memory storage with proper concurrency
   - Implement CRUD operations for cuts
   - Add indexing for efficient retrieval

2. Connect CuttingActor to repository (1 day)
   - Add repository interaction in actor ⚠️
     - Challenge: Select appropriate repository methods (register_material vs add_material)
     - Handle borrowed data that must not escape its scope
     - Manage ownership transfer between components carefully
   - Implement proper error handling
   - Add logging for storage operations

**Demonstration:** Running `main` logs "Stored cut ID X from material Y" for each processed cut

### Milestone 6: "Cutting Actor Sends Cuts to Labeling"

**Goal:** Complete the second stage of the pipeline
**Implementation Time:** 2-3 days

1. Set up channels between cutting and labeling (1 day)

   - Create CutMessage types
   - Implement channel connections
   - Add proper error handling

2. Configure CuttingActor to forward cuts (1-2 days)
   - Implement message transformation
   - Add sending logic with backpressure handling
   - Create comprehensive logging

**Demonstration:** Running `main` logs "Sent X cuts to labeling channel"

### Milestone 7: "Labeling Actor Creates Swatches from Cuts"

**Goal:** Implement the final stage of processing
**Implementation Time:** 3-4 days

1. Create LabelingActor implementation (1-2 days)

   - Add message reception from Cutting
   - Implement swatch creation logic
   - Add metadata enrichment

2. Implement swatch creation strategies (2 days)
   - Create metadata extraction and enhancement
   - Implement content analysis features
   - Add classification and tagging

**Demonstration:** Running `main` shows "Created swatch from cut X" with swatch details

### Milestone 8: "SwatchRepository Stores Processed Swatches"

**Goal:** Complete the storage of final processed items
**Implementation Time:** 2-3 days

1. Implement SwatchRepository (1-2 days)

   - Create in-memory storage with proper concurrency
   - Implement CRUD operations for swatches
   - Add indexing for efficient retrieval

2. Connect LabelingActor to repository (1 day)
   - Add repository interaction in actor
   - Implement proper error handling
   - Add logging for storage operations

**Demonstration:** Running `main` logs "Stored swatch ID X in repository"

### Milestone 9: "Query Swatches by Content"

**Goal:** Enable basic search functionality
**Implementation Time:** 2-3 days

1. Implement basic text search (1-2 days)

   - Add content indexing in repository
   - Implement simple query interface
   - Create results formatting

2. Add search commands to main (1 day)
   - Create simple query API
   - Implement results display
   - Add error handling

**Demonstration:** Running `main` shows "Found X swatches matching query 'Z'"

### Milestone 10: "Repositories Persist Data to Disk"

**Goal:** Ensure data persists between application runs
**Implementation Time:** 3-4 days

1. Implement basic persistence for repositories (2 days)

   - Add serialization for repository data
   - Implement file-based storage ⚠️
     - Challenge: I/O operations can block the async runtime
     - Use async file I/O where possible
     - Offload to dedicated threads for blocking operations
   - Create consistent loading/saving

2. Add startup/shutdown persistence (1-2 days)
   - Implement startup loading
   - Add shutdown saving
   - Create consistency checks and error recovery

**Demonstration:** Running `main` logs "Loaded X cuts and Y swatches from disk" on startup

## Future Milestones

Future milestones will focus on more advanced features:

1. **Enhanced Text Processing**

   - Language detection
   - Text classification
   - Entity extraction

2. **Embedding and Vector Search**

   - Integration with embedding models
   - Vector storage for semantic search
   - Similarity search implementation

3. **Advanced Search and Queries**

   - Query language development
   - Search result ranking
   - Filter and facet implementation

4. **User Interfaces**

   - Web-based dashboard
   - Search interface
   - Material management

5. **Integration APIs**
   - REST API for external access
   - Webhooks for processing events
   - Subscription mechanism for updates
