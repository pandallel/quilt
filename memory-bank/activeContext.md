# Active Context

## Current Focus

The project has completed Milestone 5: "Basic Cutting Actor Creation" and is now implementing Milestone 6: "Material Text Cutting Implementation". The Cutting Actor has been successfully integrated with the event-driven architecture and the initial text-splitting functionality has been implemented using the text-splitter crate.

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
   - Fully transition from direct Repository use to Registry pattern

3. **Event System**:

   - Event Bus implemented using `tokio::sync::broadcast` channels
   - Material Registry coordinating state management and event publishing
   - Event types defined for material and system events
   - ProcessingError events for handling error cases
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
   - Event publishing for discovered materials

6. **Cutting System**:
   - CuttingActor that subscribes to MaterialDiscovered events
   - TextCutter implementation using text-splitter crate for document chunking
   - CutterConfig with configurable token size settings (target: 300, min: 150, max: 800)
   - File content extraction using tokio::fs for async file operations
   - Material processing logic with proper error handling
   - Chunk information model with material tracking (ChunkInfo struct)
   - Error handling with specialized CutterError types

## Recent Changes

1. **TextCutter Implementation**:

   - Implemented TextCutter using text-splitter crate for semantic document chunking
   - Created CutterConfig for configurable token size parameters
   - Added ChunkInfo data structure to store chunk metadata and content
   - Implemented comprehensive test coverage for text cutting functionality
   - Added tokio::fs for asynchronous file operations

2. **Cutting Actor Enhancements**:

   - Added file content reading capability using async file operations
   - Implemented basic material processing with text extraction
   - Enhanced error handling for file operations and cutting processes
   - Integrated TextCutter with CuttingActor for document processing

3. **Error Handling Improvements**:

   - Added ProcessingError event type for reporting processing errors
   - Implemented error publishing through the event bus
   - Enhanced error logging with stage and material information
   - Added specialized CutterError for text splitting operations
   - Improved test coverage for error scenarios

4. **Cargo Dependencies**:

   - Added text-splitter crate for semantic document chunking
   - Enabled tokio::fs feature for asynchronous file operations
   - Enhanced error handling using thiserror for better error types

5. **Performance Issues Identified**:
   - Discovered backpressure issue in CuttingActor when processing large batches of files
   - Actor hits 30-second timeout when processing thousands of files simultaneously
   - Need to implement backpressure mechanism for high-volume scenarios
   - Added temporary workaround by excluding large document directories

## Active Decisions and Considerations

### Key Architectural Decisions

- Using Actix for actor lifecycle management and Tokio for concurrency primitives
- Implementing Event Bus using `tokio::sync::broadcast` channels
- Organizing actors around event subscription rather than direct message passing
- Using Registry as the central coordinator for state changes and event publishing
- Following idiomatic Rust patterns for concurrency safety
- Implementing atomic operations for state changes with event publishing

### Open Questions

1. **Document Processing Strategies**:

   - What algorithms should be used for text splitting?
   - How should we handle different document formats?
   - What metadata should be preserved during cutting?
   - How to implement error recovery during processing?

2. **Actor Recovery and Supervision**:

   - How should actors handle failures and restart?
   - What's the proper supervisor hierarchy for our actors?
   - Should we implement circuit breakers for external dependencies?

3. **Event Ordering and Consistency**:

   - How do we ensure event ordering when needed?
   - Should we implement event versioning or sequence numbers?
   - How do we handle event replay for recovery scenarios?

4. **State Recovery**:

   - How do we handle Registry state recovery after crashes?
   - Should we implement event sourcing for state reconstruction?
   - What's the strategy for state snapshots?

5. **Backpressure Implementation**:
   - How should backpressure be implemented in the CuttingActor to handle large batches?
   - What's the appropriate throttling mechanism for event processing?
   - Should we implement a work queue with configurable rate limiting?
   - How can we balance responsiveness with system stability under heavy load?

## Next Steps

### Short-term Tasks (Current Sprint)

1. Implement document cutting functionality:

   - Develop text extraction and processing logic
   - Create document splitting strategies
   - Add cut creation from materials
   - Implement metrics collection for processing

2. Add Cut event publishing:

   - Create MaterialCut event type
   - Implement state transitions in Registry
   - Add event validation through logging
   - Create recovery mechanisms for failed cuts

3. Implement backpressure mechanism:
   - Add throttling to CuttingActor to prevent overwhelming the system
   - Implement work queue with controlled processing rate
   - Add monitoring for queue depths and processing times
   - Create circuit breaker for system protection
