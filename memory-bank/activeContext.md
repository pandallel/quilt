# Active Context

## Current Focus

The project has completed Milestone 3: "Event Bus and Material Registry Foundation" and is now working on Milestone 4: "Discovery Actor Publishes Events" to integrate the Discovery Actor with the event-driven communication infrastructure.

## Current Implementation Status

The codebase currently has these key components implemented:

1. **Actor System**:

   - Common actor module with Ping and Shutdown messages
   - DiscoveryActor with lifecycle management
   - QuiltOrchestrator implementing the Orchestrator pattern
   - Proper Actix/Tokio runtime integration with #[actix::main]

2. **Material Repository**:

   - Thread-safe in-memory store using `Arc<RwLock<HashMap<...>>>`
   - Material state tracking with proper validation (Discovered → Cut → Swatched → Error)
   - CRUD operations with idempotence and state transition validation

3. **Event System**:

   - Event Bus implemented using `tokio::sync::broadcast` channels
   - Material Registry coordinating state management and event publishing
   - Event types defined for material and system events
   - Comprehensive test coverage for event publishing and subscription
   - Clear error handling for event operations

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

## Recent Changes

1. **Event Bus Implementation**:

   - Implemented EventBus using tokio::sync::broadcast channels
   - Created event types for MaterialDiscovered and System events
   - Added error handling and logging for event operations
   - Implemented comprehensive tests for event bus functionality

2. **Material Registry Creation**:

   - Created MaterialRegistry to coordinate state and events
   - Integrated registry with the existing repository system
   - Implemented event publishing for material operations
   - Added tests verifying registry and event bus integration

3. **Architecture Updates**:
   - Removed the old channels implementation in favor of the Event Bus
   - Updated the lib.rs exports to include the new event types
   - Enhanced error handling throughout the system
   - Improved documentation reflecting the event-driven architecture

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

1. Update Discovery Actor to publish events:

   - Add event publishing for discovered materials
   - Keep existing direct interactions for compatibility
   - Add logging to show event publishing
   - Create simple test harness for validation

2. Add event monitoring:
   - Implement simple event listener in the main application
   - Log all published events with timestamps
   - Display event counts in logs
   - Add basic metrics for event flow
