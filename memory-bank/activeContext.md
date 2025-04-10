# Active Context

## Current Focus

The project has completed Milestone 5: "Basic Cutting Actor Creation" and is now preparing for Milestone 6: "Cutting Actor Processes Materials". The Cutting Actor has been successfully implemented and integrated with the event-driven architecture, subscribing to MaterialDiscovered events from the MaterialRegistry.

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
   - Error handling for materials not found in repository
   - ProcessingDiscoveredMaterial message for handling event data
   - Event-driven architecture for processing
   - Proper actor lifecycle management

## Recent Changes

1. **Cutting Actor Implementation**:

   - Created CuttingActor that subscribes to MaterialDiscovered events
   - Implemented proper error handling with ProcessingError events
   - Added comprehensive test coverage for success and error paths
   - Integrated with the QuiltOrchestrator for lifecycle management

2. **Error Handling Improvements**:

   - Added ProcessingError event type for reporting processing errors
   - Implemented error publishing through the event bus
   - Enhanced error logging with stage and material information
   - Improved test coverage for error scenarios

3. **Orchestrator Enhancements**:

   - Updated QuiltOrchestrator to initialize and manage the CuttingActor
   - Enhanced actor shutdown with proper timeout handling
   - Improved error propagation from actors to orchestrator
   - Added actor health check monitoring

4. **Code Quality Improvements**:
   - Enhanced documentation across the codebase
   - Improved error message clarity
   - Better organized test structure with dedicated test modules
   - Added TODOs for future implementation in Milestone 6

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
