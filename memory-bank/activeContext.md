# Active Context

## Current Focus

The project has completed Milestone 4: "Discovery Actor Publishes Events" and is now preparing for Milestone 5: "Basic Cutting Actor Creation". The Discovery Actor is now fully integrated with the event-driven communication infrastructure via the Material Registry.

## Current Implementation Status

The codebase currently has these key components implemented:

1. **Actor System**:

   - Common actor module with Ping and Shutdown messages
   - DiscoveryActor with lifecycle management
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

## Recent Changes

1. **Registry Integration**:

   - Fully transitioned from MaterialRepository to MaterialRegistry in all actors
   - Updated DiscoveryActor to work with the Registry
   - Enhanced QuiltOrchestrator to initialize and manage the Registry
   - Improved test coverage for Registry operations

2. **Event System Enhancements**:

   - Implemented event monitoring in QuiltOrchestrator
   - Changed event logging from info to debug level for better production usage
   - Added proper subscriber setup in the test environment
   - Improved error handling in event-related error cases

3. **Code Quality Improvements**:
   - Enhanced error message extraction in DiscoveryActor debug logging
   - Simplified test code for better readability and maintainability
   - Improved pattern matching for error handling
   - Updated documentation to reflect architectural changes

## Active Decisions and Considerations

### Key Architectural Decisions

- Using Actix for actor lifecycle management and Tokio for concurrency primitives
- Implementing Event Bus using `tokio::sync::broadcast` channels
- Organizing actors around event subscription rather than direct message passing
- Using Registry as the central coordinator for state changes and event publishing
- Following idiomatic Rust patterns for concurrency safety
- Implementing atomic operations for state changes with event publishing

### Open Questions

1. **Actor Recovery and Supervision**:

   - How should actors handle failures and restart?
   - What's the proper supervisor hierarchy for our actors?
   - Should we implement circuit breakers for external dependencies?

2. **Event Ordering and Consistency**:

   - How do we ensure event ordering when needed?
   - Should we implement event versioning or sequence numbers?
   - How do we handle event replay for recovery scenarios?

3. **Backpressure Strategy**:

   - What are the optimal channel capacities for different event types?
   - How should actors handle backpressure when processing is slow?
   - Should we implement different priorities for different event types?

4. **State Recovery**:
   - How do we handle Registry state recovery after crashes?
   - Should we implement event sourcing for state reconstruction?
   - What's the strategy for state snapshots?

## Next Steps

### Short-term Tasks (Current Sprint)

1. Create Cutting Actor skeleton:

   - Implement simple actor that subscribes to MaterialDiscovered events
   - Add logging for received events
   - Create basic actor lifecycle (start/stop)
   - Ensure proper integration with the Registry

2. Set up actor monitoring:
   - Add heartbeat logging for the actor
   - Implement basic health checks
   - Add subscription metrics
   - Create actor configuration structure
